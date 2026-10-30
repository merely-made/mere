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

use dramatis::intents::{CreateProfileIntentV1, SwitchProfileIntentV1};
use dramatis::roster::{Roster, RosterEntry};
use dramatis::view::VaultLockView;
use djinn::door::app_admission::AppId;
use djinn::door::declaration::AppAuthority;
#[cfg(feature = "in-process-custody")]
use djinn::door::custody::ReleasedEpochKey;
use djinn::door::custody::{
    CustodyAnswer, CustodyCall, CustodyRefusal, EpochKeyRequest, KeySource, ResidentStatus,
};
use personae::ProfileId;

use crate::custody_host::VaultOutput;
use crate::keeper::Keeper;
pub(crate) mod payload;

/// The applications the custody route is granted to by label: the
/// first-party apps that call djinn instead of opening a vault (D5), and the
/// two command-line tools that do (Distillery's installed configuration,
/// Signalman's station commissioning, D17). An application that declares its
/// custody (`djinn::door::declaration`) is admitted by its owner grant
/// instead (`crate::applications`) and is not listed here.
pub const CUSTODY_APPS: &[&str] = &[
    "cleromancy",
    "turnstone",
    "graphshell",
    "woodshed",
    "hocket",
    "distillery",
    "signalman",
    "personae-vault",
    "dramatis",
];

/// Grant the custody route to [`CUSTODY_APPS`], and serve it to declared
/// applications whose owner grant names it.
pub fn grant(
    grants: &djinn::door::app_admission::AppRouteGrants,
) -> Result<(), djinn::door::route::ResidentEndpointRouteError> {
    let route = djinn::door::route::ResidentEndpointRoute::new(
        djinn::door::custody::CUSTODY_ROUTE,
        std::time::Duration::from_millis(50),
    )?;
    for app in CUSTODY_APPS {
        grants.grant(AppId::new(app), route.clone());
    }
    grants.update(|routes| routes.serve(route));
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

    /// Release exactly the epoch derivations an application declares, for a
    /// resident that composes it in process.
    pub fn with_declared_epochs(self, uses: &[djinn::door::declaration::EpochUse]) -> Self {
        uses.iter().fold(self, |policy, used| {
            policy.with_epoch(EpochRule {
                context: used.context.clone(),
                device_bound: used.device_bound,
                secret: used.secret,
            })
        })
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
    /// station runs on (D17), plus Signalman's exact Tucket device/realm
    /// child. No persona-epoch derivation: an application that needs one
    /// declares it, and its owner grant decides (`crate::applications`).
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
            .with(
                KeySource::Wallet,
                SaltRule::Matches(personae::tucket::is_station_salt),
            )
            .with(
                KeySource::Wallet,
                SaltRule::Matches(personae::sennet::is_station_salt),
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
        // Knot's share grants and revocations are issued through a provider
        // by the host publishing them (Turnstone), so their scope key too.
        for domain in ["moot", "mere.knot"] {
            policy = persona(
                policy,
                &insigne::delegation::delegation_signing_prefix(domain),
            );
        }
        // The reader key Turnstone opens Knot's published shares with
        // (`knot_editor::KNOT_PUBLISH_READER_KEY_CONTEXT`).
        policy = policy.with(
            KeySource::Persona,
            SaltRule::Exact(b"mere/knot-publish/reader/v1".to_vec()),
        );
        // A participant install is the root's act: signed here, never released.
        policy
            .with_act(
                KeySource::Persona,
                SaltRule::Prefix(insigne::delegation::delegation_signing_prefix(
                    "mere.denizen",
                )),
            )
            .with_act(
                KeySource::Persona,
                // Inviting a Graphshell guest and withdrawing that invitation are
                // root acts. Sign inside custody; never release the scope key.
                SaltRule::Prefix(insigne::delegation::delegation_signing_prefix(
                    "mere.graphshell",
                )),
            )
    }
}

/// Answer one custody call for `app`.
///
/// A declared application (`authority`) is answered only the calls and epoch
/// derivations its declaration names and its owner grants; its epoch
/// releases are not drawn from the label policy. Salt releases, attestations
/// and signatures stay under the [`ReleasePolicy`] for every caller.
pub(crate) async fn answer(
    keeper: Arc<Keeper>,
    app: AppId,
    authority: Option<AppAuthority>,
    call: CustodyCall,
) -> Result<CustodyAnswer, CustodyRefusal> {
    tracing::debug!(app = %app, call = call_name(&call), declared = authority.is_some(), "custody call");
    if let Some(authority) = &authority {
        authority.admit_custody(&call)?;
    }
    let declared = authority.is_some();
    if app.as_str() == "hocket"
        && matches!(&call, CustodyCall::Status | CustodyCall::AsProfile { .. })
        && !keeper.is_locked()
    {
        // D13: Hocket's own record joins custody before Hocket asks who it is.
        if let Err(error) = keeper.host().adopt_hocket() {
            tracing::warn!(%error, "Hocket's identity was not adopted");
        }
    }
    match call {
        CustodyCall::Payload(request) => payload::answer(&keeper, &app, request),
        CustodyCall::LockVault => {
            keeper.host().lock_vault().map_err(refused)?;
            Ok(CustodyAnswer::Done)
        },
        CustodyCall::ChangePassphraseNative(_) => {
            if app.as_str() != "dramatis" { return Err(CustodyRefusal::NotServed); }
            let receipt = tokio::task::spawn_blocking(move || keeper.change_passphrase_native())
                .await.map_err(failed)?;
            Ok(CustodyAnswer::PassphraseChange { receipt })
        },
        CustodyCall::UnlockNative => tokio::task::spawn_blocking(move || keeper.unlock_native())
            .await
            .map_err(failed)?,
        CustodyCall::UnassignedImports => {
            #[cfg(feature = "cxf")]
            {
                unlocked(&keeper)?;
                keeper
                    .unassigned_imports()
                    .map(|imports| CustodyAnswer::UnassignedImports { imports })
            }
            #[cfg(not(feature = "cxf"))]
            {
                Err(CustodyRefusal::NotServed)
            }
        },
        CustodyCall::AssignImport(request) => {
            if app.as_str() != "dramatis" {
                return Err(CustodyRefusal::NotServed);
            }
            #[cfg(feature = "cxf")]
            {
                unlocked(&keeper)?;
                tokio::task::spawn_blocking(move || keeper.assign_import(request))
                    .await
                    .map_err(failed)?
            }
            #[cfg(not(feature = "cxf"))]
            {
                let _ = request;
                Err(CustodyRefusal::NotServed)
            }
        },
        CustodyCall::Quarantine { profile } => {
            #[cfg(feature = "cxf")]
            {
                unlocked(&keeper)?;
                keeper
                    .quarantine(&profile)
                    .map(|cards| CustodyAnswer::Quarantine { cards })
            }
            #[cfg(not(feature = "cxf"))]
            {
                let _ = profile;
                Err(CustodyRefusal::NotServed)
            }
        },
        CustodyCall::ReviewQuarantine(request) => {
            if app.as_str() != "dramatis" {
                return Err(CustodyRefusal::NotServed);
            }
            #[cfg(feature = "cxf")]
            {
                unlocked(&keeper)?;
                tokio::task::spawn_blocking(move || keeper.review_quarantine(request))
                    .await
                    .map_err(failed)?
            }
            #[cfg(not(feature = "cxf"))]
            {
                let _ = request;
                Err(CustodyRefusal::NotServed)
            }
        },
        CustodyCall::VaultCommand {
            profile,
            command,
            args,
        } => {
            if app.as_str() != "personae-vault" {
                return Err(CustodyRefusal::NotServed);
            }
            unlocked(&keeper)?;
            #[cfg(feature = "cxf")]
            if matches!(command.as_str(), "cxf-preview" | "cxf-import") {
                return tokio::task::spawn_blocking(move || {
                    keeper
                        .cxf_command(&command, &args)
                        .map(|text| CustodyAnswer::VaultOutput { text })
                        .map_err(refused)
                })
                .await
                .map_err(failed)?;
            }
            // File import and SSH commands are blocking. The keeper holds its
            // vault mutex for the act, so a lock cannot interleave with it.
            tokio::task::spawn_blocking(move || {
                keeper
                    .host()
                    .vault_command(&profile, &command, &args)
                    .map(|output| match output {
                        VaultOutput::Text(text) => CustodyAnswer::VaultOutput { text },
                        VaultOutput::Enrollment {
                            target,
                            script,
                            confirmation,
                        } => CustodyAnswer::VaultEnrollment {
                            target,
                            script,
                            confirmation,
                        },
                    })
            })
            .await
            .map_err(failed)?
        },
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
            release(&keeper, &app, source, None, salts).await
        },
        CustodyCall::Attest { source, salt } => {
            if !radio_station_app_allowed(&app, &salt) {
                return Err(CustodyRefusal::NotReleasable);
            }
            unlocked(&keeper)?;
            attest(&keeper, source, None, salt).await
        },
        CustodyCall::Sign {
            source,
            salt,
            message,
        } => {
            if !radio_station_app_allowed(&app, &salt) {
                return Err(CustodyRefusal::NotReleasable);
            }
            unlocked(&keeper)?;
            sign(&keeper, source, None, salt, message).await
        },
        CustodyCall::AsProfile { profile, inner } => {
            unlocked(&keeper)?;
            let source = KeySource::Persona;
            match *inner {
                CustodyCall::Status => {
                    let mut status = status(&keeper)?;
                    let host = Arc::clone(keeper.host());
                    let named = profile.clone();
                    status.persona_public_key = Some(
                        tokio::task::spawn_blocking(move || host.root_public_key(source, Some(&named)))
                            .await
                            .map_err(failed)??,
                    );
                    status.profile = Some(profile);
                    Ok(CustodyAnswer::Status(status))
                },
                CustodyCall::Release {
                    source: KeySource::Persona,
                    salts,
                } => release(&keeper, &app, source, Some(profile), salts).await,
                CustodyCall::Attest {
                    source: KeySource::Persona,
                    salt,
                } => attest(&keeper, source, Some(profile), salt).await,
                CustodyCall::Sign {
                    source: KeySource::Persona,
                    salt,
                    message,
                } => sign(&keeper, source, Some(profile), salt, message).await,
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
            let root = keeper.wallet_root().map(Path::to_path_buf).unwrap_or_default();
            let host = Arc::clone(keeper.host());
            tokio::task::spawn_blocking(move || host.station_grant(&root, &request, issuer_public_key))
                .await
                .map_err(failed)?
                .map(CustodyAnswer::StationGrant)
        },
        CustodyCall::RevokeDevice { device_id } => {
            unlocked(&keeper)?;
            let root = keeper.wallet_root().map(Path::to_path_buf).unwrap_or_default();
            let host = Arc::clone(keeper.host());
            tokio::task::spawn_blocking(move || host.revoke_device(&root, device_id))
                .await
                .map_err(failed)?
                .map(CustodyAnswer::Revoked)
        },
        CustodyCall::ReleaseEpochKeys {
            persona,
            device_label,
            keys,
        } => {
            unlocked(&keeper)?;
            if keys.is_empty()
                || (!declared && keys.iter().any(|request| !keeper.policy().allows_epoch(request)))
            {
                return Err(CustodyRefusal::NotReleasable);
            }
            let root = keeper.wallet_root().map(Path::to_path_buf).unwrap_or_default();
            let host = Arc::clone(keeper.host());
            let released = tokio::task::spawn_blocking(move || {
                host.epoch_keys(&root, personae::PersonaId::from_uuid(persona), device_label.as_deref(), &keys)
            })
            .await
            .map_err(failed)??;
            tracing::info!(app = %app, keys = released.len(), "released epoch keys");
            Ok(CustodyAnswer::EpochKeys { keys: released })
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
        CustodyCall::Payload(_) => "retained_payload",
        CustodyCall::VaultCommand { .. } => "vault_command",
        CustodyCall::UnlockNative => "unlock_native",
        CustodyCall::ChangePassphraseNative(_) => "change_passphrase_native",
        CustodyCall::UnassignedImports => "unassigned_imports",
        CustodyCall::AssignImport(_) => "assign_import",
        CustodyCall::LockVault => "lock_vault",
        CustodyCall::Quarantine { .. } => "quarantine",
        CustodyCall::ReviewQuarantine(_) => "review_quarantine",
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
/// `data_root`, each checked against `policy`, in this process. Only builds
/// with `in-process-custody` (tests and receipt hosts) have it; a resident
/// with a custodian asks [`ResidentCustody::epoch_keys`].
#[cfg(feature = "in-process-custody")]
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
    let wire: Vec<_> = requests
        .iter()
        .map(|r| custody_wire::EpochKeyRequest {
            context: r.context.clone(),
            device_bound: r.device_bound,
            public_only: r.public_only,
        })
        .collect();
    let keys = djinn_custodian::wallet::epoch_keys(data_root, persona, device_label, &wire)
        .map_err(|r| crate::custody_host::refusal(r.0))?;
    keys.into_iter()
        .zip(requests)
        .map(|(key, request)| {
            Ok(ReleasedEpochKey {
                request: request.clone(),
                key: <[u8; 32]>::try_from(key.key.expose()).map_err(failed)?,
            })
        })
        .collect()
}

fn radio_station_app_allowed(app: &AppId, salt: &[u8]) -> bool {
    !(personae::tucket::is_station_salt(salt) || personae::sennet::is_station_salt(salt))
        || app == &AppId::new("signalman")
}

/// Release `salts`' keys from `source`, each with its attestation (D11).
async fn release(
    keeper: &Keeper,
    app: &AppId,
    source: KeySource,
    profile: Option<ProfileId>,
    salts: Vec<Vec<u8>>,
) -> Result<CustodyAnswer, CustodyRefusal> {
    if !releasable(keeper.policy(), app, source, &salts) {
        return Err(CustodyRefusal::NotReleasable);
    }
    let host = Arc::clone(keeper.host());
    let (master, keys) = tokio::task::spawn_blocking(move || host.release(source, profile.as_ref(), &salts))
        .await
        .map_err(failed)??;
    tracing::info!(app = %app, keys = keys.len(), ?source, "released derived keys");
    Ok(CustodyAnswer::Released { master, keys })
}

/// Whether every one of `salts` may leave for `app` under `policy`.
fn releasable(policy: &ReleasePolicy, app: &AppId, source: KeySource, salts: &[Vec<u8>]) -> bool {
    !salts.is_empty()
        && salts
            .iter()
            .all(|salt| policy.allows(source, salt) && radio_station_app_allowed(app, salt))
}

/// The root's attestation of `salt`'s key; no key crosses.
async fn attest(keeper: &Keeper, source: KeySource, profile: Option<ProfileId>, salt: Vec<u8>) -> Result<CustodyAnswer, CustodyRefusal> {
    if !keeper.policy().allows_act(source, &salt) {
        return Err(CustodyRefusal::NotReleasable);
    }
    let host = Arc::clone(keeper.host());
    tokio::task::spawn_blocking(move || host.attest(source, profile.as_ref(), &salt))
        .await
        .map_err(failed)?
        .map(CustodyAnswer::Attestation)
}

/// Sign `message` with `salt`'s key where the key is; only the signature
/// crosses. A Graphshell guest delegation is decoded and checked for its
/// audience, issuer and scope key before it is signed.
async fn sign(keeper: &Keeper, source: KeySource, profile: Option<ProfileId>, salt: Vec<u8>, message: Vec<u8>) -> Result<CustodyAnswer, CustodyRefusal> {
    if !keeper.policy().allows_act(source, &salt) {
        return Err(CustodyRefusal::NotReleasable);
    }
    let host = Arc::clone(keeper.host());
    let delegation = source == KeySource::Persona && profile.is_none() && crate::custodian::is_graphshell_delegation(&salt);
    let (public_key, signature) = tokio::task::spawn_blocking(move || match delegation {
        true => host.sign_delegation(crate::custodian::GRAPHSHELL_AUDIENCE, &salt, &message),
        false => host.sign(source, profile.as_ref(), &salt, &message),
    })
    .await
    .map_err(failed)??;
    Ok(CustodyAnswer::Signature { public_key, signature })
}

/// Refuse while the vault is Locked: every act and release waits for an
/// unlock (D12).
fn unlocked(keeper: &Keeper) -> Result<(), CustodyRefusal> {
    match keeper.is_locked() {
        true => Err(CustodyRefusal::Locked),
        false => Ok(()),
    }
}

fn status(keeper: &Keeper) -> Result<ResidentStatus, CustodyRefusal> {
    let custody_epoch = keeper.host().custody_epoch()?;
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
            keeper.host().root_public_key(KeySource::Wallet, None).ok(),
        ),
    };
    if keeper.host().custody_epoch()? != custody_epoch {
        return Err(refused("custody changed during status observation"));
    }
    Ok(ResidentStatus {
        lock,
        protection: snapshot.vault.protection,
        profile,
        persona_public_key,
        wallet_public_key,
        wallet_root: keeper.wallet_root().map(Into::into),
        custody_epoch,
    })
}

/// The roster from the keeper's public view, which it keeps while Locked
/// (vault lock ruling 11), so a picker draws either way.
fn roster(keeper: &Keeper) -> Result<Roster, CustodyRefusal> {
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
        .or_else(graphshell_endpoint::door::profile::env_profile)
        .unwrap_or_else(|| ProfileId("default".into()));
    Ok(Roster {
        entries,
        chosen,
        description: format!("{:?}", snapshot.vault.protection),
    })
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
    fn radio_station_release_is_exact_wallet_scope_for_signalman_only() {
        let policy = ReleasePolicy::default();
        for salt in [
            personae::tucket::station_salt(&[4; 16], "ridge").unwrap(),
            personae::sennet::station_salt(&[4; 16], "ridge").unwrap(),
        ] {
            let signalman = AppId::new("signalman");
            assert!(releasable(&policy, &signalman, KeySource::Wallet, &[salt.clone()]));
            assert!(!releasable(&policy, &AppId::new("turnstone"), KeySource::Wallet, &[salt.clone()]));
            assert!(!releasable(&policy, &signalman, KeySource::Persona, &[salt.clone()]));
            for forbidden in [
                vec![],
                b"mere.signalman.tucket.station/v1/".to_vec(),
                [salt.as_slice(), b"x"].concat(),
            ] {
                assert!(!releasable(&policy, &signalman, KeySource::Wallet, &[salt.clone(), forbidden]));
            }
        }
    }
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
        for request in knot_editor::PersonalVaultKeys::requests() {
            assert!(
                !policy.allows_epoch(&request),
                "Knot's epoch keys leave only under its declaration and grant"
            );
        }
        let install =
            insigne::delegation::delegation_signing_salt(&insigne::delegation::CapabilityScope {
                domain: "mere.denizen".into(),
                resource: b"resident".to_vec(),
                path_prefix: "scope/".into(),
                actions: ["write".to_string()].into_iter().collect(),
            });
        assert!(
            policy.allows_act(KeySource::Persona, &install),
            "djinn signs an install"
        );
        assert!(
            !policy.allows(KeySource::Persona, &install),
            "and never releases its key"
        );
        let graphshell =
            insigne::delegation::delegation_signing_salt(&insigne::delegation::CapabilityScope {
                domain: "mere.graphshell".into(),
                resource: vec![2; 32],
                path_prefix: "/services/projection/coop/chosen-graph".into(),
                actions: ["connect".into(), "edit".into()].into_iter().collect(),
            });
        assert!(
            policy.allows_act(KeySource::Persona, &graphshell),
            "the holder signs a scoped Graphshell invitation and revocation"
        );
        assert!(
            !policy.allows(KeySource::Persona, &graphshell),
            "Graphshell never receives the delegation signing key"
        );
        assert!(
            !policy.allows_act(KeySource::Wallet, &graphshell),
            "the graph grant is not a wallet act"
        );
        for door in crate::door::local_session::door_salts([7; 32]) {
            assert!(
                !policy.allows(KeySource::Persona, &door),
                "door keys stay in djinn"
            );
        }
    }
}
