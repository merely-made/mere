// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custody route's answers: what djinn does for an application that
//! asks instead of opening a vault, storage or wallet (dramatis repo plan,
//! D5, D11, D12, D17).
//!
//! - **Reads** (status, roster) answer Locked or not, so an app can show its
//!   identity as pending (D12).
//! - **Root-level acts** (attest, sign, issue a station grant, revoke a
//!   device) happen here; only their results cross (D11).
//! - **Releases** hand over the derived keys for salts the [`ReleasePolicy`]
//!   names, never the master, each with the master's attestation. A lock
//!   revokes them through the existing lock broadcast: `WatchLock` answers
//!   from the host's lock watch channel, and the app drops what it holds.

use std::sync::Arc;

use castellan::custody::IdentityStorage;
use dramatis::intents::{CreateProfileIntentV1, SwitchProfileIntentV1};
use dramatis::roster::{Roster, RosterEntry};
use dramatis::view::VaultLockView;
use graphshell::native::app_admission::AppId;
use graphshell::native::custody::{
    CustodyAnswer, CustodyCall, CustodyRefusal, KeySource, ReleasedKey, ResidentStatus,
    StationGrantRequest,
};
use pandect::DeviceId;
use pandect::station_grant::{SitedStationGrant, SitedStationGrantRequest};
use personae::{IdentityError, IdentityProvider, InMemoryProvider, ProfileId};

use crate::keeper::Keeper;

/// The applications the custody route is granted to: the first-party apps
/// that call djinn instead of opening a vault (D5), and the two command-line
/// tools that do (Distillery's installed configuration, Signalman's station
/// commissioning, D17).
pub const CUSTODY_APPS: &[&str] = &[
    "turnstone",
    "knot-editor",
    "graphshell",
    "woodshed",
    "hocket",
    "distillery",
    "signalman",
];

/// Grant the custody route to [`CUSTODY_APPS`].
pub fn grant(
    grants: &graphshell::native::app_admission::AppRouteGrants,
) -> Result<(), graphshell::native::endpoint_catalog::ResidentEndpointRouteError> {
    let route = graphshell::native::endpoint_catalog::ResidentEndpointRoute::new(
        graphshell::native::custody::CUSTODY_ROUTE,
        std::time::Duration::from_millis(50),
    )?;
    for app in CUSTODY_APPS {
        grants.grant(AppId::new(app), route.clone());
    }
    Ok(())
}

/// Which derived keys djinn releases, by root and salt (D11). Everything
/// else stays in djinn: the master always, the doors' keys, the records'
/// keys, any salt nobody named.
#[derive(Clone)]
pub struct ReleasePolicy {
    rules: Vec<(KeySource, SaltRule)>,
}

/// One namespace a release may draw from.
#[derive(Clone)]
pub enum SaltRule {
    /// Exactly this salt.
    Exact(Vec<u8>),
    /// Any salt beginning with these bytes (and longer than them).
    Prefix(Vec<u8>),
    /// Any salt the function accepts.
    Matches(fn(&[u8]) -> bool),
}

impl SaltRule {
    fn allows(&self, salt: &[u8]) -> bool {
        match self {
            Self::Exact(exact) => salt == exact.as_slice(),
            Self::Prefix(prefix) => salt.len() > prefix.len() && salt.starts_with(prefix),
            Self::Matches(test) => test(salt),
        }
    }
}

impl ReleasePolicy {
    /// Release nothing.
    pub fn none() -> Self {
        Self { rules: Vec::new() }
    }

    /// Release salts matching `rule` from `source` as well.
    pub fn with(mut self, source: KeySource, rule: SaltRule) -> Self {
        self.rules.push((source, rule));
        self
    }

    /// Whether `salt` from `source` may leave djinn. An empty salt is the
    /// master's place and never may.
    pub fn allows(&self, source: KeySource, salt: &[u8]) -> bool {
        !salt.is_empty()
            && self
                .rules
                .iter()
                .any(|(from, rule)| *from == source && rule.allows(salt))
    }
}

impl Default for ReleasePolicy {
    /// The keys apps need continuously today: a mesh author, which is also
    /// the transport identity (vault lock ruling 92); Distillery's personal
    /// mesh name; Graphshell's per-session endpoint keys; and, from the
    /// wallet, the station- and controller-scoped Reticulum material a sited
    /// station runs on (D17).
    fn default() -> Self {
        Self::none()
            .with(
                KeySource::Persona,
                SaltRule::Exact(mesh::MESH_AUTHOR_SALT.to_vec()),
            )
            .with(
                KeySource::Persona,
                SaltRule::Exact(distillery::DISTILLERY_MESH_SALT.to_vec()),
            )
            .with(
                KeySource::Persona,
                SaltRule::Prefix(b"graphshell/endpoint-session/".to_vec()),
            )
            .with(
                KeySource::Wallet,
                SaltRule::Matches(personae::reticulum::is_reticulum_salt),
            )
    }
}

/// Answer one custody call for `app`.
pub(crate) async fn answer<S: IdentityStorage + 'static>(
    keeper: Arc<Keeper<S>>,
    app: AppId,
    call: CustodyCall,
) -> Result<CustodyAnswer, CustodyRefusal> {
    tracing::debug!(app = %app, call = call_name(&call), "custody call");
    match call {
        CustodyCall::Status => status(&keeper).map(CustodyAnswer::Status),
        CustodyCall::Roster => roster(&keeper).map(CustodyAnswer::Roster),
        CustodyCall::ChooseProfile { profile } => {
            unlocked(&keeper)?;
            keeper
                .switch_profile(SwitchProfileIntentV1 { profile: profile.0 })
                .map_err(refused)?;
            Ok(CustodyAnswer::Done)
        },
        CustodyCall::CreateProfile {
            profile,
            display_name,
        } => {
            unlocked(&keeper)?;
            keeper
                .create_profile(CreateProfileIntentV1 {
                    id: profile.0.clone(),
                    display_name,
                })
                .map_err(refused)?;
            keeper
                .switch_profile(SwitchProfileIntentV1 { profile: profile.0 })
                .map_err(refused)?;
            Ok(CustodyAnswer::Done)
        },
        CustodyCall::Release { source, salts } => {
            unlocked(&keeper)?;
            if salts.is_empty()
                || salts
                    .iter()
                    .any(|salt| !keeper.policy().allows(source, salt))
            {
                return Err(CustodyRefusal::NotReleasable);
            }
            let root = root(&keeper, source)?;
            let root = root.provider();
            let master = root.master_public_key().to_bytes();
            let keys = salts
                .into_iter()
                .map(|salt| {
                    Ok(ReleasedKey {
                        seed: root.derive_keypair(&salt).map_err(identity)?.to_seed(),
                        attestation: root.attest_derived_key(&salt).map_err(identity)?,
                        salt,
                    })
                })
                .collect::<Result<Vec<_>, CustodyRefusal>>()?;
            tracing::info!(app = %app, keys = keys.len(), ?source, "released derived keys");
            Ok(CustodyAnswer::Released { master, keys })
        },
        CustodyCall::Attest { source, salt } => {
            unlocked(&keeper)?;
            if !keeper.policy().allows(source, &salt) {
                return Err(CustodyRefusal::NotReleasable);
            }
            let attestation = root(&keeper, source)?
                .provider()
                .attest_derived_key(&salt)
                .map_err(identity)?;
            Ok(CustodyAnswer::Attestation(attestation))
        },
        CustodyCall::Sign {
            source,
            salt,
            message,
        } => {
            unlocked(&keeper)?;
            if !keeper.policy().allows(source, &salt) {
                return Err(CustodyRefusal::NotReleasable);
            }
            let key = root(&keeper, source)?
                .provider()
                .derive_keypair(&salt)
                .map_err(identity)?;
            let signature = key.sign(&message);
            Ok(CustodyAnswer::Signature {
                public_key: key.public_key().to_bytes(),
                signature: signature.to_bytes().to_vec(),
            })
        },
        CustodyCall::IssueStationGrant {
            request,
            issuer_public_key,
        } => {
            unlocked(&keeper)?;
            issue_station_grant(&keeper, request, issuer_public_key)
                .map(CustodyAnswer::StationGrant)
        },
        CustodyCall::RevokeDevice { device_id } => {
            unlocked(&keeper)?;
            let root = keeper.wallet_root().ok_or(CustodyRefusal::NoWallet)?;
            castellan::custody::wallet::revoke_remote_auth_device(
                root,
                DeviceId::from_uuid(device_id),
            )
            .map(CustodyAnswer::Revoked)
            .map_err(failed)
        },
        CustodyCall::WatchLock { seen } => {
            let mut lock = keeper.lock_state();
            loop {
                let now = *lock.borrow_and_update();
                if now != seen {
                    return Ok(CustodyAnswer::Lock(now));
                }
                if lock.changed().await.is_err() {
                    return Err(CustodyRefusal::Failed {
                        reason: "the resident is shutting down".into(),
                    });
                }
            }
        },
    }
}

fn call_name(call: &CustodyCall) -> &'static str {
    match call {
        CustodyCall::Status => "status",
        CustodyCall::Roster => "roster",
        CustodyCall::ChooseProfile { .. } => "choose_profile",
        CustodyCall::CreateProfile { .. } => "create_profile",
        CustodyCall::Release { .. } => "release",
        CustodyCall::Attest { .. } => "attest",
        CustodyCall::Sign { .. } => "sign",
        CustodyCall::IssueStationGrant { .. } => "issue_station_grant",
        CustodyCall::RevokeDevice { .. } => "revoke_device",
        CustodyCall::WatchLock { .. } => "watch_lock",
    }
}

/// Refuse while the vault is Locked: every act and release waits for an
/// unlock on djinn's own surface (D12).
fn unlocked<S: IdentityStorage + 'static>(keeper: &Keeper<S>) -> Result<(), CustodyRefusal> {
    match keeper.is_locked() {
        true => Err(CustodyRefusal::Locked),
        false => Ok(()),
    }
}

/// The root a call derives from: the persona in use, or the wallet's seed.
enum Root<'a, S: IdentityStorage + 'static> {
    Persona(&'a castellan::authority::PersonaeHost<S>),
    Wallet(InMemoryProvider),
}

impl<S: IdentityStorage + 'static> Root<'_, S> {
    fn provider(&self) -> &dyn IdentityProvider {
        match self {
            Self::Persona(host) => *host,
            Self::Wallet(wallet) => wallet,
        }
    }
}

fn root<S: IdentityStorage + 'static>(
    keeper: &Keeper<S>,
    source: KeySource,
) -> Result<Root<'_, S>, CustodyRefusal> {
    match source {
        KeySource::Persona => Ok(Root::Persona(keeper.host().as_ref())),
        KeySource::Wallet => {
            let root = keeper.wallet_root().ok_or(CustodyRefusal::NoWallet)?;
            let seed = castellan::custody::wallet::load_identity_seed(root)
                .map_err(failed)?
                .ok_or(CustodyRefusal::NoWallet)?;
            Ok(Root::Wallet(InMemoryProvider::from_seed(seed)))
        },
    }
}

fn status<S: IdentityStorage + 'static>(
    keeper: &Keeper<S>,
) -> Result<ResidentStatus, CustodyRefusal> {
    let snapshot = keeper.snapshot().map_err(failed)?;
    let lock = snapshot.vault.lock;
    let profile = snapshot
        .profiles
        .iter()
        .find(|profile| profile.selected)
        .map(|profile| ProfileId(profile.id.clone()));
    let (persona_public_key, wallet_public_key) = match lock {
        VaultLockView::Locked => (None, None),
        VaultLockView::Unlocked => (
            Some(keeper.master_public_key().to_bytes()),
            match root(keeper, KeySource::Wallet) {
                Ok(wallet) => Some(wallet.provider().master_public_key().to_bytes()),
                Err(_) => None,
            },
        ),
    };
    Ok(ResidentStatus {
        lock,
        protection: snapshot.vault.protection,
        profile,
        persona_public_key,
        wallet_public_key,
        wallet_root: keeper.wallet_root().map(Into::into),
    })
}

/// The roster from the keeper's public view, which it keeps while Locked
/// (vault lock ruling 11), so a picker draws either way.
fn roster<S: IdentityStorage + 'static>(keeper: &Keeper<S>) -> Result<Roster, CustodyRefusal> {
    let snapshot = keeper.snapshot().map_err(failed)?;
    let mut entries: Vec<RosterEntry> = snapshot
        .profiles
        .iter()
        .map(|profile| RosterEntry {
            id: ProfileId(profile.id.clone()),
            display_name: profile.display_name.clone(),
            slot_count: profile.slot_count,
            chosen: profile.selected,
        })
        .collect();
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    let chosen = entries
        .iter()
        .find(|entry| entry.chosen)
        .map(|entry| entry.id.clone())
        .or_else(graphshell::profile::env_profile)
        .unwrap_or_else(|| ProfileId(castellan::custody::roster::DEFAULT_PROFILE.into()));
    Ok(Roster {
        entries,
        chosen,
        description: format!("{:?}", snapshot.vault.protection),
    })
}

fn issue_station_grant<S: IdentityStorage + 'static>(
    keeper: &Keeper<S>,
    request: StationGrantRequest,
    issuer_public_key: [u8; 32],
) -> Result<personae::carry::DeviceGrantSet, CustodyRefusal> {
    let root = keeper.wallet_root().ok_or(CustodyRefusal::NoWallet)?;
    let wallet_public_key = self::root(keeper, KeySource::Wallet)?
        .provider()
        .master_public_key()
        .to_bytes();
    let request = SitedStationGrantRequest::new(
        DeviceId::from_uuid(request.device_id),
        request.station_ed25519_public_key,
        request.label,
        request.issued_at_ms,
        request.expires_at_ms,
    )
    .map_err(|error| CustodyRefusal::Refused {
        reason: error.to_string(),
    })?;
    let grant = SitedStationGrant::issue_with(
        root,
        issuer_public_key,
        wallet_public_key,
        request,
        |spec| castellan::custody::wallet::issue_remote_auth_device_grant(root, spec),
    )
    .map_err(|error| CustodyRefusal::Refused {
        reason: error.to_string(),
    })?;
    Ok(grant.signed().clone())
}

fn identity(error: IdentityError) -> CustodyRefusal {
    match error {
        IdentityError::Locked => CustodyRefusal::Locked,
        other => failed(other),
    }
}

fn refused(error: impl std::fmt::Display) -> CustodyRefusal {
    CustodyRefusal::Refused {
        reason: error.to_string(),
    }
}

fn failed(error: impl std::fmt::Display) -> CustodyRefusal {
    CustodyRefusal::Failed {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_policy_releases_named_namespaces_and_never_the_master() {
        let policy = ReleasePolicy::default();
        assert!(policy.allows(KeySource::Persona, mesh::MESH_AUTHOR_SALT));
        assert!(policy.allows(KeySource::Persona, b"graphshell/endpoint-session/s1"));
        assert!(!policy.allows(KeySource::Persona, b"graphshell/endpoint-session/"));
        assert!(!policy.allows(KeySource::Persona, b""));
        assert!(!policy.allows(KeySource::Wallet, mesh::MESH_AUTHOR_SALT));
        let [station, _] = personae::reticulum::station_salts(b"device");
        assert!(policy.allows(KeySource::Wallet, &station));
        assert!(!policy.allows(KeySource::Persona, &station));
        for door in graphshell::native::local_session::door_salts([7; 32]) {
            assert!(
                !policy.allows(KeySource::Persona, &door),
                "door keys stay in djinn"
            );
        }
    }
}
