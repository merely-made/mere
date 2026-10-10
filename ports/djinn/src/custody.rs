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
//! - **Epoch releases** hand over keys derived from a persona wallet's current
//!   private epoch, for the derivations the policy names, never the epoch.
//!   Knot's vault and writer keys are the first (DR-C): djinn derives them
//!   with the same `blake3::derive_key` Knot used, so stores keep their keys.

use std::path::Path;
use std::sync::Arc;

use castellan::custody::IdentityStorage;
use dramatis::intents::{CreateProfileIntentV1, SwitchProfileIntentV1};
use dramatis::roster::{Roster, RosterEntry};
use dramatis::view::VaultLockView;
use graphshell::native::app_admission::AppId;
use graphshell::native::custody::{
    CustodyAnswer, CustodyCall, CustodyRefusal, EpochKeyRequest, KeySource, ReleasedEpochKey,
    ReleasedKey, ResidentStatus, StationGrantRequest,
};
use pandect::DeviceId;
use pandect::station_grant::{SitedStationGrant, SitedStationGrantRequest};
use personae::{Ed25519Keypair, IdentityError, IdentityProvider, InMemoryProvider, ProfileId};
use zeroize::Zeroizing;

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
    epoch: Vec<EpochRule>,
    acts: Vec<(KeySource, SaltRule)>,
}

/// One epoch derivation a release may hand over: its context, whether it
/// mixes in the device root, and whether its secret may leave or only its
/// public key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochRule {
    pub context: String,
    pub device_bound: bool,
    pub secret: bool,
}

impl EpochRule {
    fn allows(&self, request: &EpochKeyRequest) -> bool {
        self.context == request.context
            && self.device_bound == request.device_bound
            && (self.secret || request.public_only)
    }
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
        Self {
            rules: Vec::new(),
            epoch: Vec::new(),
            acts: Vec::new(),
        }
    }

    /// Sign and attest, but never release, salts matching `rule` from
    /// `source`: root-level acts done here on an app's behalf (D11).
    pub fn with_act(mut self, source: KeySource, rule: SaltRule) -> Self {
        self.acts.push((source, rule));
        self
    }

    /// Whether djinn may sign or attest with `salt`'s key for an app: any
    /// releasable salt, and the acts' salts. Never the master's place.
    pub fn allows_act(&self, source: KeySource, salt: &[u8]) -> bool {
        self.allows(source, salt)
            || (!salt.is_empty()
                && self
                    .acts
                    .iter()
                    .any(|(from, rule)| *from == source && rule.allows(salt)))
    }

    /// Release the epoch derivation `rule` names as well.
    pub fn with_epoch(mut self, rule: EpochRule) -> Self {
        self.epoch.push(rule);
        self
    }

    /// Whether the epoch derivation `request` may leave djinn.
    pub fn allows_epoch(&self, request: &EpochKeyRequest) -> bool {
        !request.context.is_empty() && self.epoch.iter().any(|rule| rule.allows(request))
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
    /// station runs on (D17). From the persona epoch: Knot's vault key and
    /// device-bound writer seed, and only the public half of its legacy
    /// writer.
    fn default() -> Self {
        let knot = |context: &str, device_bound, secret| EpochRule {
            context: context.into(),
            device_bound,
            secret,
        };
        Self::none()
            .with_epoch(knot(knot_editor::VAULT_KEY_CONTEXT, false, true))
            .with_epoch(knot(knot_editor::SIGNING_KEY_CONTEXT, true, true))
            .with_epoch(knot(knot_editor::SIGNING_KEY_CONTEXT, false, false))
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
            .with_apps()
    }
}

/// The application namespaces DR-C moved behind the custody route: the
/// derived keys Turnstone, Woodshed and Hocket need continuously (transport
/// and sealing, D11), and the delegations Turnstone issues through djinn.
impl ReleasePolicy {
    fn with_apps(self) -> Self {
        let persona = |policy: Self, prefix: &[u8]| {
            policy.with(KeySource::Persona, SaltRule::Prefix(prefix.to_vec()))
        };
        let mut policy = self.with(
            KeySource::Persona,
            SaltRule::Exact(b"woodshed.practice-session.seal.v1".to_vec()),
        );
        for prefix in [
            // Hocket's per-session hand-off signer.
            &b"hocket/handoff/v2/"[..],
            // Turnstone: a place's transport, sealed secrets and founding key;
            // the projection endpoint; capsule-scoped Gemini identities.
            b"turnstone.place.",
            b"turnstone/projection-endpoint/",
            b"personae/gemini-client-identity/v1/",
            // The shared-place writers Turnstone authors under: Commons
            // containers and chat, Gemot membership and objects, Stickleback
            // group keys and prekeys.
            b"mere.commons.writer.v1/",
            b"mere.commons.chat.writer.v1/",
            b"mere.gemot.membership.v1/",
            b"mere.gemot.objects.v1/",
            b"stickleback/group-key-writer/v1/",
            b"stickleback/group-prekey-identity/v1",
        ] {
            policy = persona(policy, prefix);
        }
        // Gemot authors a Moot's delegation facts under the scope key that
        // signed them, so a place's delegation key is held while it is open.
        policy = persona(
            policy,
            &insigne::delegation::delegation_signing_prefix("moot"),
        );
        // The reader key Turnstone opens Knot's published shares with
        // (`knot_editor::KNOT_PUBLISH_READER_KEY_CONTEXT`).
        policy = policy.with(
            KeySource::Persona,
            SaltRule::Exact(b"mere/knot-publish/reader/v1".to_vec()),
        );
        // A participant install is the root's act: signed here, never released.
        policy.with_act(
            KeySource::Persona,
            SaltRule::Prefix(insigne::delegation::delegation_signing_prefix(
                "mere.denizen",
            )),
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
    if app.as_str() == "hocket"
        && matches!(&call, CustodyCall::Status | CustodyCall::AsProfile { .. })
        && !keeper.is_locked()
    {
        // D13: Hocket's own record joins custody before Hocket asks who it is.
        if let Err(error) = crate::hocket_adoption::adopt(keeper.host()) {
            tracing::warn!(%error, "Hocket's identity was not adopted");
        }
    }
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
            let root = root(&keeper, source)?;
            release(keeper.policy(), &app, source, root.provider(), salts)
        },
        CustodyCall::Attest { source, salt } => {
            unlocked(&keeper)?;
            attest(keeper.policy(), source, root(&keeper, source)?.provider(), &salt)
        },
        CustodyCall::Sign {
            source,
            salt,
            message,
        } => {
            unlocked(&keeper)?;
            sign(
                keeper.policy(),
                source,
                root(&keeper, source)?.provider(),
                &salt,
                &message,
            )
        },
        CustodyCall::AsProfile { profile, inner } => {
            unlocked(&keeper)?;
            let persona = keeper.profile_provider(&profile).map_err(identity)?;
            let source = KeySource::Persona;
            match *inner {
                CustodyCall::Status => {
                    let mut status = status(&keeper)?;
                    status.persona_public_key = Some(persona.master_public_key().to_bytes());
                    status.profile = Some(profile);
                    Ok(CustodyAnswer::Status(status))
                },
                CustodyCall::Release {
                    source: KeySource::Persona,
                    salts,
                } => release(keeper.policy(), &app, source, &persona, salts),
                CustodyCall::Attest {
                    source: KeySource::Persona,
                    salt,
                } => attest(keeper.policy(), source, &persona, &salt),
                CustodyCall::Sign {
                    source: KeySource::Persona,
                    salt,
                    message,
                } => sign(keeper.policy(), source, &persona, &salt, &message),
                _ => Err(CustodyRefusal::Refused {
                    reason: "only a status, or a persona release, attestation or signature, answers as a named persona".into(),
                }),
            }
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
        CustodyCall::ReleaseEpochKeys {
            persona,
            device_label,
            keys,
        } => {
            unlocked(&keeper)?;
            let root = keeper.wallet_root().ok_or(CustodyRefusal::NoWallet)?;
            let released = epoch_keys(
                keeper.policy(),
                root,
                personae::PersonaId::from_uuid(persona),
                device_label.as_deref(),
                &keys,
            )?;
            tracing::info!(app = %app, keys = released.len(), "released epoch keys");
            Ok(CustodyAnswer::EpochKeys(released))
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
        CustodyCall::ReleaseEpochKeys { .. } => "release_epoch_keys",
        CustodyCall::AsProfile { .. } => "as_profile",
    }
}

/// Derive `requests` from `persona`'s current private epoch under
/// `data_root`, each checked against `policy`. djinn's own resident Knot
/// opens through this too, so the route and the resident cannot drift.
pub fn epoch_keys(
    policy: &ReleasePolicy,
    data_root: &Path,
    persona: personae::PersonaId,
    device_label: Option<&str>,
    requests: &[EpochKeyRequest],
) -> Result<Vec<ReleasedEpochKey>, CustodyRefusal> {
    if requests.is_empty() || requests.iter().any(|request| !policy.allows_epoch(request)) {
        return Err(CustodyRefusal::NotReleasable);
    }
    if pandect::wallet_store::load_persona_wallet(data_root, persona)
        .map_err(failed)?
        .is_none()
    {
        return Err(CustodyRefusal::NoPersona);
    }
    // A wallet whose epoch will not load is sealed and locked: pending (D12).
    let epoch = castellan::custody::wallet::load_current_private_epoch(data_root, persona)
        .map_err(failed)?
        .ok_or(CustodyRefusal::Locked)?;
    let epoch = Zeroizing::new(epoch.epoch_secret);
    let device_root = match requests.iter().any(|request| request.device_bound) {
        true => Some(device_root(data_root, device_label)?),
        false => None,
    };
    Ok(requests
        .iter()
        .map(|request| {
            let mut material = Zeroizing::new(Vec::with_capacity(64));
            material.extend_from_slice(&epoch);
            if let (true, Some(device_root)) = (request.device_bound, device_root.as_ref()) {
                material.extend_from_slice(device_root);
            }
            let derived = Zeroizing::new(blake3::derive_key(&request.context, &material));
            let key = match request.public_only {
                true => Ed25519Keypair::from_seed(*derived).public_key().to_bytes(),
                false => *derived,
            };
            ReleasedEpochKey {
                request: request.clone(),
                key,
            }
        })
        .collect())
}

/// This machine's public device key: distinct per device, so a
/// device-bound derivation differs between a persona's devices. A label
/// mints the identity when there is none.
fn device_root(data_root: &Path, label: Option<&str>) -> Result<[u8; 32], CustodyRefusal> {
    let identity = match label {
        Some(label) => castellan::custody::wallet::ensure_local_device_identity(data_root, label)
            .map(Some)
            .map_err(failed)?,
        None => {
            castellan::custody::wallet::load_local_device_identity(data_root).map_err(failed)?
        },
    };
    match identity {
        Some(identity) => Ok(Ed25519Keypair::from_seed(identity.device_seed)
            .public_key()
            .to_bytes()),
        None if pandect::wallet_store::local_device_identity_path(data_root).is_file() => {
            Err(CustodyRefusal::Locked)
        },
        None => Err(CustodyRefusal::Refused {
            reason: "this device has no identity yet".into(),
        }),
    }
}

/// Release `salts`' keys from `root`, each with its attestation (D11).
fn release(
    policy: &ReleasePolicy,
    app: &AppId,
    source: KeySource,
    root: &dyn IdentityProvider,
    salts: Vec<Vec<u8>>,
) -> Result<CustodyAnswer, CustodyRefusal> {
    if salts.is_empty() || salts.iter().any(|salt| !policy.allows(source, salt)) {
        return Err(CustodyRefusal::NotReleasable);
    }
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
}

/// The master's attestation of `salt`'s key; no key crosses.
fn attest(
    policy: &ReleasePolicy,
    source: KeySource,
    root: &dyn IdentityProvider,
    salt: &[u8],
) -> Result<CustodyAnswer, CustodyRefusal> {
    if !policy.allows_act(source, salt) {
        return Err(CustodyRefusal::NotReleasable);
    }
    root.attest_derived_key(salt)
        .map(CustodyAnswer::Attestation)
        .map_err(identity)
}

/// Sign `message` with `salt`'s key, here; only the signature crosses.
fn sign(
    policy: &ReleasePolicy,
    source: KeySource,
    root: &dyn IdentityProvider,
    salt: &[u8],
    message: &[u8],
) -> Result<CustodyAnswer, CustodyRefusal> {
    if !policy.allows_act(source, salt) {
        return Err(CustodyRefusal::NotReleasable);
    }
    let key = root.derive_keypair(salt).map_err(identity)?;
    let signature = key.sign(message);
    Ok(CustodyAnswer::Signature {
        public_key: key.public_key().to_bytes(),
        signature: signature.to_bytes().to_vec(),
    })
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
        assert!(policy.allows_epoch(&EpochKeyRequest::secret(
            knot_editor::SIGNING_KEY_CONTEXT,
            true
        )));
        assert!(policy.allows_epoch(&EpochKeyRequest::public(
            knot_editor::SIGNING_KEY_CONTEXT,
            false
        )));
        assert!(
            !policy.allows_epoch(&EpochKeyRequest::secret(
                knot_editor::SIGNING_KEY_CONTEXT,
                false
            )),
            "the legacy writer's secret stays in djinn"
        );
        let install = insigne::delegation::delegation_signing_salt(&insigne::delegation::CapabilityScope {
            domain: "mere.denizen".into(),
            resource: b"resident".to_vec(),
            path_prefix: "scope/".into(),
            actions: ["write".to_string()].into_iter().collect(),
        });
        assert!(policy.allows_act(KeySource::Persona, &install), "djinn signs an install");
        assert!(!policy.allows(KeySource::Persona, &install), "and never releases its key");
        for door in graphshell::native::local_session::door_salts([7; 32]) {
            assert!(
                !policy.allows(KeySource::Persona, &door),
                "door keys stay in djinn"
            );
        }
    }
}
