// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Sealing for a mere's retained page text, through the custodian.
//!
//! Page text is written only through [`CustodialRetained`]: every value in
//! `retained.redb` is sealed by the custodian's retained-payload operation
//! (`ResidentCustody::payload`), so the key never enters this process, and a
//! locked or absent custody pauses writing instead of falling back to
//! plaintext. The resident has no plaintext production path; a resident
//! composed without a wallet answers "paused: not configured".
//!
//! The lane is [`RETAINED_LANE`], one name shared by every application that
//! keeps or reads page text. A custody-door client (Turnstone's retained
//! clips, Graphshell) whose declaration names this payload scope derives the
//! identical binding, so either side's kept text opens in the other's goal
//! surface. This resident derives the same binding itself:
//! `(lane, profile, master public key)`, as the custodian does for an app.

use std::path::PathBuf;
use std::sync::Arc;

use castellan_client::custody::CustodyRefusal;
use castellan_client::custody_payload::{PayloadBytes, PayloadOperation, PayloadRequest, PayloadResult};
use dramatis_resident::custody_host::ResidentCustody;
use eidetic::{Hash, PayloadSealer, SealedBlobRef};
use pandect::{RetainedCustody, SharedSealer};
use personae::{PersonaId, ProfileId};

/// The payload lane shared by Graphshell and Turnstone for retained page text.
pub const RETAINED_LANE: &str = "mere.retained-page-text/shared-v1";

const LOCKED: &str = "The vault is locked; page text is not being kept or read.";

/// How a resident seals retained page text: the sealer for the values and the
/// gate that pauses writing. There is no unsealed variant.
#[derive(Clone)]
pub struct RetainedSealing {
    pub(crate) sealer: SharedSealer,
    pub(crate) gate: Arc<dyn RetainedCustody>,
}

impl RetainedSealing {
    /// No wallet is composed: nothing is written, and the person is told why.
    pub fn unconfigured() -> Self {
        Self {
            sealer: Arc::new(Refusing),
            gate: Arc::new(Refusing),
        }
    }

    /// Seal under the custodian for `persona`'s wallet at `wallet_root`,
    /// admitted to `profile`. Works for any [`ResidentCustody`]: the
    /// in-process host of the daemon and the embedded resident, and the
    /// remote custodian's host.
    pub fn custodial(
        host: Arc<dyn ResidentCustody>,
        wallet_root: PathBuf,
        persona: PersonaId,
        profile: ProfileId,
    ) -> Self {
        let custody = Arc::new(CustodialRetained {
            host,
            wallet_root,
            persona,
            profile,
        });
        Self {
            sealer: custody.clone(),
            gate: custody,
        }
    }

    /// An explicit sealer and gate, for tests that supply their own.
    pub fn with(sealer: SharedSealer, gate: Arc<dyn RetainedCustody>) -> Self {
        Self { sealer, gate }
    }
}

struct Refusing;

impl RetainedCustody for Refusing {
    fn writable(&self) -> Result<(), String> {
        Err("Page text cannot be kept: no wallet custody is configured for this resident.".into())
    }
}

impl PayloadSealer for Refusing {
    fn seal(&self, _: &Hash, _: &[u8]) -> eidetic::Result<(Vec<u8>, SealedBlobRef)> {
        Err(eidetic::Error::new("retained page text has no custody configured"))
    }
    fn unseal(&self, _: &Hash, _: &SealedBlobRef, _: &[u8]) -> eidetic::Result<Vec<u8>> {
        Err(eidetic::Error::new("retained page text has no custody configured"))
    }
}

/// The custodian-backed sealer and gate.
pub struct CustodialRetained {
    host: Arc<dyn ResidentCustody>,
    wallet_root: PathBuf,
    persona: PersonaId,
    profile: ProfileId,
}

impl CustodialRetained {
    fn call(&self, hash: &Hash, operation: PayloadOperation) -> eidetic::Result<PayloadResult> {
        if self.host.is_locked() {
            return Err(eidetic::Error::new(LOCKED));
        }
        let master = self.host.master_public_key().to_bytes();
        let scope = serde_json::to_vec(&(RETAINED_LANE, &self.profile.0, &master))
            .map_err(|error| eidetic::Error::new(error.to_string()))?;
        let request = PayloadRequest {
            profile: self.profile.clone(),
            master,
            content_hash: *hash,
            operation,
            custody_epoch: self.host.custody_epoch().ok().flatten(),
        };
        request
            .validate()
            .map_err(|reason| eidetic::Error::new(reason))?;
        self.host
            .payload(&self.wallet_root, self.persona, request, scope)
            .map_err(|refusal| match refusal {
                CustodyRefusal::Locked => eidetic::Error::new(LOCKED),
                other => eidetic::Error::new(format!("retained-content custody: {other}")),
            })
    }
}

impl RetainedCustody for CustodialRetained {
    fn writable(&self) -> Result<(), String> {
        if self.host.is_locked() {
            Err(LOCKED.into())
        } else {
            Ok(())
        }
    }
}

impl PayloadSealer for CustodialRetained {
    fn seal(&self, hash: &Hash, cleartext: &[u8]) -> eidetic::Result<(Vec<u8>, SealedBlobRef)> {
        match self.call(
            hash,
            PayloadOperation::Seal {
                bytes: PayloadBytes(cleartext.to_vec()),
            },
        )? {
            PayloadResult::Sealed { bytes, marker } => Ok((bytes.0, marker)),
            PayloadResult::Clear { .. } => Err(eidetic::Error::new("custody did not seal")),
        }
    }

    fn unseal(
        &self,
        hash: &Hash,
        marker: &SealedBlobRef,
        sealed: &[u8],
    ) -> eidetic::Result<Vec<u8>> {
        match self.call(
            hash,
            PayloadOperation::Unseal {
                bytes: PayloadBytes(sealed.to_vec()),
                marker: marker.clone(),
            },
        )? {
            PayloadResult::Clear { bytes } if Hash::of(&bytes.0) == *hash => Ok(bytes.0),
            _ => Err(eidetic::Error::new("custody returned invalid retained bytes")),
        }
    }
}
