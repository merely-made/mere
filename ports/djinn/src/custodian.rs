// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Djinn as a client of the independently launched custodian (custody
//! isolation plan, §5.6 and H1).
//!
//! The custodian is found through an installed client file that the desktop
//! user cannot write: on Unix it and its directory must be root-owned and
//! not group- or world-writable, so a same-user process cannot point Djinn
//! at an impostor. The file names the endpoint and the OS identity the
//! custodian must present; the client checks that identity before its
//! hello. Every answer is typed. A lost or refusing custodian is reported
//! as such and never replaced by an in-process key: identity stays pending.
//!
//! H1 routes two consumers here: Graphshell's guest delegations
//! (`mere.graphshell`, signed inside the custodian after its own consent)
//! and Distillery's mesh author (exactly its two release salts, checked
//! against the custodian's attestations).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use custody_wire::contract::{Answer, Outcome, Purpose};
use custody_wire::secret::Bytes;
use custody_wire::{Client, Operation, ServerExpectation};
use graphshell_endpoint::door::custody::CustodyRefusal;
use personae::{Ed25519Keypair, Ed25519PublicKey, RetainedKeys};
use serde::Deserialize;

/// The installed client file.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledCustodian {
    /// A Unix socket path, or a Windows pipe name.
    pub endpoint: String,
    /// The custodian's OS identity.
    pub server: ServerExpectation,
}

/// Where the installed client file lives for this user.
pub fn installed_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    return PathBuf::from(format!("/etc/djinn-custodian/clients/{}.json", custody_wire::peer::effective_uid()));
    #[cfg(target_os = "macos")]
    return PathBuf::from(format!(
        "/Library/Application Support/Merely/djinn-custodian/clients/{}.json",
        custody_wire::peer::effective_uid()
    ));
    #[cfg(windows)]
    return PathBuf::from(r"C:\ProgramData\djinn-custodian\client.json");
    #[allow(unreachable_code)]
    PathBuf::new()
}

impl InstalledCustodian {
    /// The installed client file, if this device has a custodian. A file
    /// the desktop user could have written is refused, not ignored.
    pub fn load() -> Result<Option<Self>, String> {
        let path = installed_path();
        if !path.exists() {
            return Ok(None);
        }
        Self::from_path(&path, true).map(Some)
    }

    /// Read a client file. `protected` requires root ownership on Unix.
    pub fn from_path(path: &Path, protected: bool) -> Result<Self, String> {
        #[cfg(unix)]
        if protected {
            use std::os::unix::fs::MetadataExt;
            for checked in [Some(path), path.parent()].into_iter().flatten() {
                let meta = std::fs::metadata(checked).map_err(|e| format!("{}: {e}", checked.display()))?;
                if meta.uid() != 0 || meta.mode() & 0o022 != 0 {
                    return Err(format!(
                        "{} must be root-owned and not group- or world-writable",
                        checked.display()
                    ));
                }
            }
        }
        let _ = protected;
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// A live link to the custodian, reconnecting through fresh admission
/// after a loss.
pub struct CustodianLink {
    installed: InstalledCustodian,
    client: Mutex<Option<Arc<Client>>>,
}

/// Noninteractive operations; consent petitions wait longer.
const PROMPTED: Duration = Duration::from_secs(150);

impl CustodianLink {
    pub fn new(installed: InstalledCustodian) -> Self {
        Self {
            installed,
            client: Mutex::new(None),
        }
    }

    fn client(&self) -> Result<Arc<Client>, CustodyRefusal> {
        let mut slot = self.client.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(client) = slot.as_ref()
            && client.observed().alive
        {
            return Ok(Arc::clone(client));
        }
        #[cfg(unix)]
        let connector = custody_wire::transport::UnixSocketConnector::new(&self.installed.endpoint)
            .with_signed(self.installed.server.signed.clone());
        #[cfg(windows)]
        let connector = custody_wire::transport::NamedPipeConnector::new(&self.installed.endpoint);
        let client = Arc::new(Client::open(&connector, &self.installed.server).map_err(|error| unavailable(&error.to_string()))?);
        *slot = Some(Arc::clone(&client));
        Ok(client)
    }

    fn call(&self, operation: Operation, deadline: Duration) -> Result<Answer, CustodyRefusal> {
        match self.client()?.call(operation, deadline) {
            Outcome::Success { answer } => Ok(answer),
            Outcome::Refused { code } if code == "locked" => Err(CustodyRefusal::Locked),
            Outcome::Refused { code } => Err(CustodyRefusal::Refused {
                reason: format!("the custodian refused: {code}"),
            }),
            Outcome::Degraded { code, .. } => Err(CustodyRefusal::Failed {
                reason: format!("the custodian reported a degraded outcome: {code}"),
            }),
            Outcome::Unknown { code } => Err(unavailable(&code)),
        }
    }

    /// Unlock with the start's passphrase unless already unlocked: the
    /// documented terminal-unlock exception (§5.3), over the channel whose
    /// server identity was checked.
    pub fn unlock(&self, passphrase: &[u8]) -> Result<(), CustodyRefusal> {
        let operation = Operation::Unlock {
            passphrase: custody_wire::Secret::copy_from(passphrase),
        };
        match self.call(operation, Duration::from_secs(60)) {
            Ok(Answer::Unlocked { .. }) => Ok(()),
            Err(CustodyRefusal::Refused { reason }) if reason.ends_with("already-unlocked") => Ok(()),
            Err(refusal) => Err(refusal),
            Ok(_) => Err(unexpected()),
        }
    }

    /// Exactly `salts` of `profile`, as an identity that refuses every other
    /// salt. Each key must check under the custodian's attestation and name
    /// one master.
    pub fn release(&self, profile: &str, salts: &[Vec<u8>]) -> Result<RetainedKeys, CustodyRefusal> {
        let operation = Operation::Release {
            profile: profile.into(),
            salts: salts.iter().map(|s| Bytes(s.clone())).collect(),
        };
        let Answer::Released { master_public_key, keys } = self.call(operation, PROMPTED)? else {
            return Err(unexpected());
        };
        if keys.len() != salts.len() || keys.iter().zip(salts).any(|(k, s)| &k.salt.0 != s) {
            return Err(unexpected());
        }
        let master = Ed25519PublicKey::from_bytes(&master_public_key).map_err(|_| unexpected())?;
        let mut released = Vec::with_capacity(keys.len());
        for key in keys {
            let seed = zeroize::Zeroizing::new(<[u8; 32]>::try_from(key.seed.expose()).map_err(|_| unexpected())?);
            let attestation = insigne::DerivedKeyAttestation::from_parts(
                key.attestation.master,
                key.attestation.derived,
                key.attestation.signature.0,
            );
            released.push((key.salt.0, Ed25519Keypair::from_seed(*seed), attestation));
        }
        RetainedKeys::from_released(master, released).map_err(|_| unexpected())
    }

    /// Sign one delegation statement in `audience` inside the custodian.
    pub fn sign_delegation(&self, profile: &str, audience: &str, salt: &[u8], message: &[u8]) -> Result<([u8; 32], [u8; 64]), CustodyRefusal> {
        let operation = Operation::Sign {
            profile: profile.into(),
            purpose: Purpose::Delegation {
                audience: audience.into(),
            },
            salt: Bytes(salt.to_vec()),
            message: Bytes(message.to_vec()),
        };
        let Answer::Signature {
            public_key,
            signature,
            attestation: Some(attestation),
        } = self.call(operation, PROMPTED)?
        else {
            return Err(unexpected());
        };
        let signer = insigne::DerivedKeyAttestation::from_parts(attestation.master, attestation.derived, attestation.signature.0);
        let checked = signer.check(salt).map_err(|_| unexpected())?;
        let verified = Ed25519PublicKey::from_bytes(&public_key)
            .map(|key| key.verify(message, &personae::Ed25519Signature::from_bytes(&signature)))
            .unwrap_or(false);
        if checked.derived() != &public_key || !verified {
            return Err(unexpected());
        }
        Ok((public_key, signature))
    }
}

fn unavailable(reason: &str) -> CustodyRefusal {
    CustodyRefusal::Failed {
        reason: format!("the custodian is unavailable ({reason}); identity is pending"),
    }
}

fn unexpected() -> CustodyRefusal {
    CustodyRefusal::Failed {
        reason: "the custodian's answer did not check".into(),
    }
}

/// Whether a persona salt is a Graphshell guest-delegation scope key: the
/// H1 signing consumer.
pub fn is_graphshell_delegation(salt: &[u8]) -> bool {
    let prefix = insigne::delegation::delegation_signing_prefix(GRAPHSHELL_AUDIENCE);
    salt.len() > prefix.len() && salt.starts_with(&prefix)
}

/// The delegation domain the H1 signing consumer uses.
pub const GRAPHSHELL_AUDIENCE: &str = "mere.graphshell";
