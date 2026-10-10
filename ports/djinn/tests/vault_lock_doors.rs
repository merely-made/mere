// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Vault lock rulings 46 and 47, in process: the app door admits sessions
//! while the vault is locked, through kept keys that open nothing else; the
//! control route unlocks by passphrase (`djinn --unlock`) and by the
//! resident's own prompt (`--native`); and the native unlock asks the OS
//! first and the passphrase box second. The OS prompt and the dialog are
//! Mark's attended steps: here the `NativeIdentityUi` trait is the seam.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{
    IdentityStorage, IdentityVault, OsPresence, Profile, SealedProfileStorage, UnlockMethod,
};
use djinn::identity_ui::{NativeIdentityUi, apply_native_identity_action};
use djinn::resident_status::{
    self, AgentListenerV1, ControlUnlock, LockStateV1, NativeUnlocker, RESIDENT_APP,
    RESIDENT_CONTROL_ROUTE, RESIDENT_STATUS_ROUTE, ResidentControlEndpoint, ResidentEndpointsV1,
    ResidentStatusEndpoint, ResidentStatusSource, ResidentStatusV1, STATUS_SCHEMA, StartupUnlockV1,
    StopSignal, Unlocker,
};
use graphshell::browser_carrier::{
    NativeIdentityAction, NativeIdentityFailure, NativeIdentityResult,
};
use graphshell::identity::{VaultLockView, VaultProtectionView};
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointRoute};
use graphshell::native::local_session::{DoorIdentity, door_salts};
use notochord::{NetworkId, ProfileRef, ProofBinding, SessionHello, TrafficClass};
use personae::{Ed25519Keypair, IdentityError, IdentityProvider, ProfileId};
use uuid::Uuid;
use zeroize::Zeroizing;

const PASSPHRASE: &[u8] = b"djinn door lock";

type Host = PersonaeHost<SealedProfileStorage>;

/// A host over a lockable temp vault (a sealed storage, passphrase enrolled).
fn lockable(dir: &Path) -> Arc<Host> {
    let storage = SealedProfileStorage::open_with_key(dir.join("vault"), [0x3d; 32]);
    storage.enroll_passphrase(PASSPHRASE).unwrap();
    let profile = Profile::new(
        ProfileId("door".into()),
        "Door",
        Ed25519Keypair::from_seed([0x3e; 32]),
    );
    storage.save_profile(&profile).unwrap();
    Arc::new(PersonaeHost::new(
        IdentityVault::open(storage, &ProfileId("door".into())).unwrap(),
        None,
        VaultProtectionView::OsProtected,
    ))
}

fn status_fixture() -> ResidentStatusV1 {
    ResidentStatusV1 {
        schema: STATUS_SCHEMA.into(),
        pid: 7,
        started_ms: 1,
        installed: false,
        ready: true,
        startup_unlock: StartupUnlockV1::AutoOs,
        protection: VaultProtectionView::OsProtected.into(),
        lock: LockStateV1::Unlocked,
        endpoints: ResidentEndpointsV1 {
            agent: "agent".into(),
            agent_listener: AgentListenerV1::Receipt,
            browser: "browser".into(),
            app: "app".into(),
        },
        sync: None,
    }
}

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-vault-lock-doors-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!("djinn-vault-lock-doors-{}.sock", Uuid::new_v4()))
        .display()
        .to_string();
}

async fn open(endpoint: &str, route: &str) -> Result<AppBrokerClient, AppClientError> {
    AppBrokerClient::open_route_at(
        endpoint,
        AppId::new(RESIDENT_APP),
        AppRouteId::new(route).unwrap(),
    )
    .await
}

/// Serve the app door over `host` with the status and control routes, the
/// control route unlocking through `unlock`.
async fn serve(host: Arc<Host>, unlock: ControlUnlock) -> (String, ResidentStatusSource) {
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([(
        AppId::new("turnstone"),
        ResidentEndpointRoute::new("identity", Duration::from_millis(50)).unwrap(),
    )]));
    let status = ResidentStatusSource::new(status_fixture());
    let route = catalog
        .update(|catalog| ResidentStatusEndpoint::register(status.clone(), catalog))
        .await
        .unwrap();
    resident_status::grant(&grants, route);
    let route = catalog
        .update(|catalog| {
            ResidentControlEndpoint::register_with_unlock(StopSignal::default(), unlock, catalog)
        })
        .await
        .unwrap();
    resident_status::grant(&grants, route);
    let endpoint = endpoint_name();
    {
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            let _ = serve_app_broker(
                &endpoint,
                Arc::new(djinn::keeper::Keeper::new(host)),
                grants,
                60_000,
                None,
                catalog,
            )
            .await;
        });
    }
    for _ in 0..100 {
        if let Ok(mut client) = open(&endpoint, RESIDENT_STATUS_ROUTE).await
            && resident_status::read_status(&mut client).await.is_ok()
        {
            let _ = client.close().await;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    (endpoint, status)
}

fn passphrase_unlocker(host: &Arc<Host>) -> Unlocker {
    let host = Arc::clone(host);
    Arc::new(move |passphrase: &[u8]| {
        host.unlock_vault(UnlockMethod::Passphrase(passphrase))
            .map_err(|error| error.to_string())
    })
}

// ─── Ruling 46: the door keeps two keys, and they open nothing else ──────

#[test]
fn the_door_keeps_two_keys_and_every_other_salt_is_locked() {
    let dir = tempfile::tempdir().unwrap();
    let host = lockable(dir.path());
    let subject = host.master_public_key().to_bytes();
    let kept = djinn::keeper::Keeper::new(Arc::clone(&host))
        .door_keys()
        .unwrap();
    host.lock_vault().unwrap();
    let again = djinn::keeper::Keeper::new(Arc::clone(&host))
        .door_keys()
        .expect("the lock leaves the door keys in place");
    assert!(Arc::ptr_eq(&kept, &again));
    assert_eq!(again.salts(), door_salts(subject));
    for salt in door_salts(subject) {
        assert!(again.derive_keypair(&salt).is_ok());
    }
    for salt in [
        &b"mere/network-policy/session-signer/v1"[..],
        b"mere.djinn/castellan/records/v1",
        b"knot",
        b"",
    ] {
        assert!(matches!(
            again.derive_keypair(salt),
            Err(IdentityError::Locked)
        ));
        assert!(matches!(
            again.attest_derived_key(salt),
            Err(IdentityError::Locked)
        ));
    }
    // A remote-style hello (the global salt) cannot be made with them.
    let hello = SessionHello::issue(
        again.as_ref(),
        NetworkId([5; 32]),
        ProfileRef {
            id: "mere.base".into(),
            revision: 1,
        },
        graphshell::admission::connect_action(),
        TrafficClass::Interactive,
        [1; 32],
        &ProofBinding::initiator(b"mere/graphshell/v1", None, None),
        Vec::new(),
    );
    assert!(hello.is_err());
    // And the vault itself is still locked behind them.
    assert!(matches!(
        host.derive_keypair(b"knot"),
        Err(IdentityError::Locked)
    ));
}

/// A resident that locked before its door was ever used has no door keys:
/// they are captured while unlocked.
#[test]
fn a_door_never_used_before_the_lock_stays_closed() {
    let dir = tempfile::tempdir().unwrap();
    let host = lockable(dir.path());
    host.lock_vault().unwrap();
    assert!(matches!(
        djinn::keeper::Keeper::new(Arc::clone(&host)).door_keys(),
        Err(IdentityError::Locked)
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn while_locked_the_status_route_reads_and_the_control_route_unlocks() {
    let dir = tempfile::tempdir().unwrap();
    let host = lockable(dir.path());
    let (endpoint, status) = serve(
        Arc::clone(&host),
        ControlUnlock {
            passphrase: Some(passphrase_unlocker(&host)),
            native: None,
        },
    )
    .await;
    host.lock_vault().unwrap();
    status.update(|status| status.lock = VaultLockView::Locked.into());

    let mut client = open(&endpoint, RESIDENT_STATUS_ROUTE).await.unwrap();
    let read = resident_status::read_status(&mut client).await.unwrap();
    assert_eq!(
        read.lock,
        LockStateV1::Locked,
        "the door admits while locked"
    );
    // The status route never takes the unlock.
    assert!(
        resident_status::request_unlock(&mut client, PASSPHRASE)
            .await
            .is_err()
    );
    let _ = client.close().await;
    assert!(host.is_locked());

    let mut control = open(&endpoint, RESIDENT_CONTROL_ROUTE).await.unwrap();
    assert!(
        resident_status::request_unlock(&mut control, b"wrong")
            .await
            .is_err()
    );
    let _ = control.close().await;
    assert!(host.is_locked(), "a wrong passphrase leaves it locked");
    let mut control = open(&endpoint, RESIDENT_CONTROL_ROUTE).await.unwrap();
    resident_status::request_unlock(&mut control, PASSPHRASE)
        .await
        .unwrap();
    let _ = control.close().await;
    assert!(!host.is_locked());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_native_unlock_command_carries_nothing_and_runs_the_residents_prompt() {
    let dir = tempfile::tempdir().unwrap();
    let host = lockable(dir.path());
    let prompts = Arc::new(Mutex::new(0usize));
    let native: NativeUnlocker = {
        let (host, prompts) = (Arc::clone(&host), Arc::clone(&prompts));
        Arc::new(move || {
            *prompts.lock().unwrap() += 1;
            match apply_native_identity_action(
                &host,
                &Scripted::passphrase(PASSPHRASE),
                NativeIdentityAction::UnlockVault,
            ) {
                NativeIdentityResult::UnlockedVault => Ok(()),
                other => Err(format!("{other:?}")),
            }
        })
    };
    let (endpoint, _) = serve(
        Arc::clone(&host),
        ControlUnlock {
            passphrase: None,
            native: Some(native),
        },
    )
    .await;
    host.lock_vault().unwrap();
    let mut control = open(&endpoint, RESIDENT_CONTROL_ROUTE).await.unwrap();
    // This resident takes no passphrase on its control route.
    assert!(
        resident_status::request_unlock(&mut control, PASSPHRASE)
            .await
            .is_err()
    );
    let _ = control.close().await;
    assert_eq!(*prompts.lock().unwrap(), 0);
    let mut control = open(&endpoint, RESIDENT_CONTROL_ROUTE).await.unwrap();
    resident_status::request_native_unlock(&mut control)
        .await
        .unwrap();
    let _ = control.close().await;
    assert_eq!(*prompts.lock().unwrap(), 1);
    assert!(!host.is_locked());
}

// ─── Ruling 47: the resident's own prompt ─────────────────────────────────

/// The seam for the attended prompts: scripted answers to the OS presence
/// check and the passphrase box.
struct Scripted {
    passphrase: Option<Vec<u8>>,
    asked: Mutex<Vec<&'static str>>,
}

impl Scripted {
    fn passphrase(passphrase: &[u8]) -> Self {
        Self {
            passphrase: Some(passphrase.to_vec()),
            asked: Mutex::new(Vec::new()),
        }
    }

    fn cancelling() -> Self {
        Self {
            passphrase: None,
            asked: Mutex::new(Vec::new()),
        }
    }
}

impl NativeIdentityUi for Scripted {
    fn pick_ssh_private_key(&self) -> Result<Option<std::path::PathBuf>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }

    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }

    /// No Hello here: the passphrase box follows, as on a device without one.
    fn verify_presence(&self) -> Result<Option<OsPresence>, NativeIdentityFailure> {
        self.asked.lock().unwrap().push("presence");
        Ok(None)
    }

    fn prompt_vault_passphrase(&self) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        self.asked.lock().unwrap().push("passphrase");
        Ok(self
            .passphrase
            .as_ref()
            .map(|p| Zeroizing::new(String::from_utf8(p.clone()).unwrap())))
    }
}

#[test]
fn the_native_unlock_asks_the_os_first_then_the_passphrase() {
    let dir = tempfile::tempdir().unwrap();
    let host = lockable(dir.path());
    host.lock_vault().unwrap();

    let wrong = Scripted::passphrase(b"wrong");
    assert_eq!(
        apply_native_identity_action(&host, &wrong, NativeIdentityAction::UnlockVault),
        NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::IncorrectPassphrase
        }
    );
    assert_eq!(*wrong.asked.lock().unwrap(), ["presence", "passphrase"]);
    assert!(host.is_locked());

    let cancelled = Scripted::cancelling();
    assert_eq!(
        apply_native_identity_action(&host, &cancelled, NativeIdentityAction::UnlockVault),
        NativeIdentityResult::Cancelled
    );
    assert!(host.is_locked());

    let right = Scripted::passphrase(PASSPHRASE);
    assert_eq!(
        apply_native_identity_action(&host, &right, NativeIdentityAction::UnlockVault),
        NativeIdentityResult::UnlockedVault
    );
    assert!(!host.is_locked());
    // Unlocked already: nothing is asked.
    let idle = Scripted::passphrase(PASSPHRASE);
    assert_eq!(
        apply_native_identity_action(&host, &idle, NativeIdentityAction::UnlockVault),
        NativeIdentityResult::UnlockedVault
    );
    assert!(idle.asked.lock().unwrap().is_empty());
}
