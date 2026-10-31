// SPDX-License-Identifier: MPL-2.0
//! Real local custody routes over public synthetic vaults. Never opens a personal vault.
use castellan::custody::vault::{CredentialLineage, ProtocolKey, UnlockTier};
use castellan::{
    authority::PersonaeHost,
    custody::{
        IdentitySlot, IdentityStorage, IdentityVault, PassphraseEncryptedStorage, Profile,
        SealedProfileStorage, SecretBytes, UnlockMethod,
    },
};
use mere_resident::{
    door::{
        app_broker::{AppEndpointCatalog, serve_app_broker},
        browser_carrier::NativeIdentityFailure,
    },
    identity_ui::{NativeIdentityUi, SystemNativeIdentityUi},
    keeper::Keeper,
};
use dramatis::authentication::{PassphraseChangeFailureV1 as F, PassphraseChangeReceiptV1 as R};
use djinn::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use djinn::door::custody::{CustodyAnswer, CustodyCall};
use djinn::door::custody_client::CustodyClient;
use dramatis::view::VaultProtectionView;
use personae::{Ed25519Keypair, ProfileId};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use zeroize::Zeroizing;
const OLD: &str = "Old local fixture";
// Deliberately exercise exact Linux helper output, including two spaces and a glob.
const NEW: &str = "New  local fixture * ' café";
struct Ui(
    Mutex<VecDeque<Option<String>>>,
    Mutex<Option<Box<dyn FnOnce() + Send>>>,
);
impl Ui {
    fn answers(values: &[Option<&str>]) -> Self {
        Self(
            Mutex::new(values.iter().map(|v| v.map(str::to_string)).collect()),
            Mutex::new(None),
        )
    }
}
impl NativeIdentityUi for Ui {
    fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn ask_vault_passphrase(
        &self,
        _: &str,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        let answer = self.0.lock().unwrap().pop_front();
        if self.0.lock().unwrap().is_empty() {
            if let Some(hook) = self.1.lock().unwrap().take() {
                hook();
            }
        }
        answer
            .map(|v| v.map(Zeroizing::new))
            .ok_or(NativeIdentityFailure::UiUnavailable)
    }
}
fn seed(root: &Path, portable: bool) {
    let s: Box<dyn IdentityStorage> = if portable {
        Box::new(PassphraseEncryptedStorage::open(root.join("vault.json"), OLD.as_bytes()).unwrap())
    } else {
        let s = SealedProfileStorage::open_with_key(root.join("vault"), [7; 32]);
        s.enroll_passphrase(OLD.as_bytes()).unwrap();
        Box::new(s)
    };
    for (id, byte) in [("family", 41), ("work", 42)] {
        let mut p = Profile::new(
            ProfileId(id.into()),
            id,
            Ed25519Keypair::from_seed([byte; 32]),
        );
        p.slots.insert(
            ProtocolKey::new("fixture", None),
            IdentitySlot::Direct {
                kind: "fixture".into(),
                payload: SecretBytes::new(vec![byte; 48]),
                lineage: CredentialLineage::LocallyDerived,
                unlock_tier: UnlockTier::Session,
            },
        );
        s.save_profile(&p).unwrap();
    }
}
fn open(
    root: &Path,
    portable: bool,
    password: &str,
) -> Result<Box<dyn IdentityStorage>, personae::IdentityError> {
    if portable {
        assert!(root.join("vault.json").is_file());
        Ok(Box::new(PassphraseEncryptedStorage::open(
            root.join("vault.json"),
            password.as_bytes(),
        )?))
    } else {
        let s = SealedProfileStorage::open_locked(root.join("vault"));
        s.unlock(UnlockMethod::Passphrase(password.as_bytes()))?;
        Ok(Box::new(s))
    }
}
fn check_profiles(s: &dyn IdentityStorage) -> serde_json::Value {
    let mut facts = vec![];
    for (id, byte) in [("family", 41), ("work", 42)] {
        let p = s.load_profile(&ProfileId(id.into())).unwrap();
        assert_eq!(p.master.to_seed(), [byte; 32]);
        let IdentitySlot::Direct { payload, .. } =
            p.slots.get(&ProtocolKey::new("fixture", None)).unwrap()
        else {
            panic!()
        };
        assert_eq!(payload.as_slice(), vec![byte; 48]);
        facts.push(serde_json::json!({"profile":id,"master_public":p.master.public_key().to_bytes(),"slots":p.slots.len()}));
    }
    serde_json::json!(facts)
}
fn keeper(
    root: &Path,
    portable: bool,
    password: &str,
    ui: Arc<dyn NativeIdentityUi>,
) -> Arc<Keeper> {
    let storage = open(root, portable, password).unwrap();
    check_profiles(&*storage);
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::open(storage, &ProfileId("family".into())).unwrap(),
        None,
        VaultProtectionView::Passphrase,
    ));
    Arc::new(Keeper::new(host).with_ui(ui))
}
fn endpoint(root: &Path) -> String {
    #[cfg(windows)]
    {
        format!(
            r"\\.\pipe\authentication-{}",
            blake3::hash(root.to_string_lossy().as_bytes()).to_hex()
        )
    }
    #[cfg(not(windows))]
    {
        root.join("app.sock").to_string_lossy().to_string()
    }
}
async fn serve(
    k: Arc<Keeper>,
    endpoint: String,
) -> tokio::task::JoinHandle<()> {
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    mere_resident::custody::grant(&grants).unwrap();
    let path = endpoint.clone();
    let task = tokio::spawn(async move {
        serve_app_broker(
            &path,
            k,
            grants,
            60_000,
            None,
            AppEndpointCatalog::default(),
        )
        .await
        .unwrap();
    });
    for _ in 0..200 {
        if CustodyClient::open_at(&endpoint, AppId::new("dramatis"))
            .await
            .is_ok()
        {
            return task;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("synthetic custody broker did not bind")
}
#[tokio::test]
async fn resident_requires_dramatis_and_fresh_credentials_and_honors_cancellation_for_both_formats()
{
    for portable in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        seed(dir.path(), portable);
        let ui = Arc::new(Ui::answers(&[
            None,
            Some("wrong"),
            Some(NEW),
            Some(NEW),
            Some(OLD),
            Some(NEW),
            Some("mismatch"),
            Some(OLD),
            Some(""),
            Some(OLD),
            Some(NEW),
            Some(NEW),
        ]));
        let k = keeper(dir.path(), portable, OLD, ui.clone());
        let endpoint = endpoint(dir.path());
        let task = serve(k.clone(), endpoint.clone()).await;
        let mut other = CustodyClient::open_at(&endpoint, AppId::new("graphshell"))
            .await
            .unwrap();
        assert!(other.change_passphrase_native().await.is_err());
        assert_eq!(ui.0.lock().unwrap().len(), 12);
        let mut c = CustodyClient::open_at(&endpoint, AppId::new("dramatis"))
            .await
            .unwrap();
        k.lock_vault().unwrap();
        assert_eq!(
            c.change_passphrase_native().await.unwrap(),
            R::Rejected { reason: F::Locked }
        );
        assert_eq!(ui.0.lock().unwrap().len(), 12);
        k.unlock_with_passphrase(OLD.as_bytes()).unwrap();
        for expected in [
            R::Cancelled,
            R::Rejected {
                reason: F::CurrentPassphraseRejected,
            },
            R::Rejected {
                reason: F::ConfirmationMismatch,
            },
            R::Rejected {
                reason: F::InvalidNewPassphrase,
            },
        ] {
            assert_eq!(c.change_passphrase_native().await.unwrap(), expected);
            check_profiles(&*open(dir.path(), portable, OLD).unwrap());
        }
        assert_eq!(c.change_passphrase_native().await.unwrap(), R::Changed);
        assert!(ui.0.lock().unwrap().is_empty());
        assert!(open(dir.path(), portable, OLD).is_err());
        check_profiles(&*open(dir.path(), portable, NEW).unwrap());
        // No secret field can cross the public request or result.
        assert_eq!(
            serde_json::to_value(CustodyCall::ChangePassphraseNative(Default::default())).unwrap(),
            serde_json::json!({"call":"change_passphrase_native"})
        );
        assert!(
            !serde_json::to_string(&CustodyAnswer::PassphraseChange {
                receipt: R::Changed
            })
            .unwrap()
            .contains(NEW)
        );
        task.abort();
        let _ = task.await;
        drop(c);
        drop(other);
        drop(k);
    }
}

#[tokio::test]
async fn locking_while_the_confirmation_is_open_refuses_the_change() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), true);
    let ui = Arc::new(Ui::answers(&[Some(OLD), Some(NEW), Some(NEW)]));
    let k = keeper(dir.path(), true, OLD, ui.clone());
    let host = k.host().clone();
    *ui.1.lock().unwrap() = Some(Box::new(move || host.lock_vault().unwrap()));
    let endpoint = endpoint(dir.path());
    let server = serve(k, endpoint.clone()).await;
    let mut c = CustodyClient::open_at(&endpoint, AppId::new("dramatis"))
        .await
        .unwrap();
    assert_eq!(
        c.change_passphrase_native().await.unwrap(),
        R::Rejected { reason: F::Locked }
    );
    assert!(open(dir.path(), true, NEW).is_err());
    check_profiles(&*open(dir.path(), true, OLD).unwrap());
    server.abort();
    let _ = server.await;
}

/// A separate headless process proves refusal before any native/console input.
#[cfg(target_os = "linux")]
#[tokio::test]
#[ignore = "run with DISPLAY and WAYLAND_DISPLAY absent in a fresh process"]
async fn absent_graphical_password_provider_refuses_without_changing_the_vault() {
    assert!(std::env::var_os("DISPLAY").is_none());
    assert!(std::env::var_os("WAYLAND_DISPLAY").is_none());
    let backend = light_file_dialog::dialog::DialogBackend::query();
    println!(
        "password provider: {} (reported graphical: {})",
        backend.name, backend.graphic
    );
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), true);
    let k = keeper(
        dir.path(),
        true,
        OLD,
        Arc::new(SystemNativeIdentityUi::default()),
    );
    let endpoint = endpoint(dir.path());
    let server = serve(k, endpoint.clone()).await;
    let mut client = CustodyClient::open_at(&endpoint, AppId::new("dramatis"))
        .await
        .unwrap();
    assert_eq!(
        client.change_passphrase_native().await.unwrap(),
        R::Rejected {
            reason: F::UiUnavailable
        }
    );
    assert!(open(dir.path(), true, NEW).is_err());
    check_profiles(&*open(dir.path(), true, OLD).unwrap());
    server.abort();
    let _ = server.await;
}

/// Separate-process native host, selected only by an explicit ignored-test invocation.
#[tokio::test]
#[ignore = "owned synthetic headed fixture; never use with a personal vault"]
async fn headed_authentication_host() {
    let root = PathBuf::from(std::env::var_os("D_AUTH_ROOT").unwrap());
    let portable = std::env::var("D_AUTH_FORMAT").unwrap() == "portable";
    let reopen = std::env::var("D_AUTH_PHASE").unwrap() == "reopen";
    if !reopen {
        assert!(!root.exists());
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(".authentication-fixture"),
            "public synthetic authentication",
        )
        .unwrap();
        seed(&root, portable);
    }
    assert_eq!(
        std::fs::read_to_string(root.join(".authentication-fixture")).unwrap(),
        "public synthetic authentication"
    );
    let password = if reopen { NEW } else { OLD };
    let before = check_profiles(&*open(&root, portable, password).unwrap());
    std::fs::write(
        root.join(if reopen {
            "cold-identities.json"
        } else {
            "before-identities.json"
        }),
        serde_json::to_vec_pretty(&before).unwrap(),
    )
    .unwrap();
    if reopen {
        assert!(open(&root, portable, OLD).is_err());
        std::fs::write(root.join("old-rejected"), "true").unwrap();
    }
    let ui: Arc<dyn NativeIdentityUi> =
        if std::env::var("D_AUTH_NATIVE").ok().as_deref() == Some("1") {
            let backend = light_file_dialog::dialog::DialogBackend::query();
            std::fs::write(
                root.join("native-provider.json"),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"name":backend.name,"graphic":backend.graphic}),
                )
                .unwrap(),
            )
            .unwrap();
            Arc::new(SystemNativeIdentityUi::default())
        } else {
            Arc::new(Ui::answers(&[
                None,
                Some("wrong"),
                Some(NEW),
                Some(NEW),
                Some(OLD),
                Some(NEW),
                Some("mismatch"),
                Some(OLD),
                Some(""),
                Some(OLD),
                Some(NEW),
                Some(NEW),
            ]))
        };
    let k = keeper(&root, portable, password, ui);
    let endpoint = endpoint(&root);
    let server = serve(k, endpoint.clone()).await;
    std::fs::write(root.join("ready.tmp"), endpoint.as_bytes()).unwrap();
    std::fs::rename(root.join("ready.tmp"), root.join("ready")).unwrap();
    let end = tokio::time::Instant::now() + Duration::from_secs(600);
    while !root.join("stop").exists() {
        assert!(tokio::time::Instant::now() < end, "fixture timeout");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    server.abort();
    let _ = server.await;
}
