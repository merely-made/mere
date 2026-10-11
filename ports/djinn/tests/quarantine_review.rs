// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real owner-local custody wire with synthetic sealed items and resident UI.
#![cfg(feature = "cxf")]
use castellan::{
    authority::PersonaeHost,
    custody::{IdentityStorage, IdentityVault, InMemoryStorage, Profile},
    resident::CastellanResident,
};
use chatelaine::ItemState;
use djinn::{identity_ui::NativeIdentityUi, keeper::Keeper};
use graphshell::{
    browser_carrier::NativeIdentityFailure,
    identity::VaultProtectionView,
    native::{
        app_admission::{AllowedAppRoutes, AppId, AppRouteGrants},
        app_broker::{AppEndpointCatalog, serve_app_broker},
        custody::CustodyRefusal,
        custody_client::{CustodyClient, CustodyClientError},
        quarantine::{ReviewAction, ReviewRequest},
    },
};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

#[derive(Default)]
struct Ui {
    allow: AtomicBool,
    count: AtomicUsize,
    on_prompt: Mutex<Option<Box<dyn Fn() + Send>>>,
    seen: Mutex<Vec<(String, String, String)>>,
}
impl NativeIdentityUi for Ui {
    fn pick_ssh_private_key(&self) -> Result<Option<std::path::PathBuf>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<zeroize::Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn confirm_quarantine(
        &self,
        profile: &str,
        title: &str,
        action: &str,
    ) -> Result<bool, NativeIdentityFailure> {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.seen
            .lock()
            .unwrap()
            .push((profile.into(), title.into(), action.into()));
        if let Some(f) = self.on_prompt.lock().unwrap().take() {
            f();
        }
        Ok(self.allow.load(Ordering::SeqCst))
    }
}
struct Fixture {
    _dir: tempfile::TempDir,
    host: Arc<PersonaeHost<InMemoryStorage>>,
    resident: CastellanResident,
    ui: Arc<Ui>,
    persona: PersonaId,
    endpoint: String,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Fixture {
    async fn new(credential: serde_json::Value) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let storage = InMemoryStorage::new();
        let family = Profile::new(
            ProfileId("family".into()),
            "Family",
            Ed25519Keypair::from_seed([0x43; 32]),
        );
        let work = Profile::new(
            ProfileId("work".into()),
            "Work",
            Ed25519Keypair::from_seed([0x44; 32]),
        );
        storage.save_profile(&family).unwrap();
        storage.save_profile(&work).unwrap();
        let host = Arc::new(PersonaeHost::new(
            IdentityVault::with_profile(storage, family),
            None,
            VaultProtectionView::Ephemeral,
        ));
        let resident = CastellanResident::claim(
            dir.path().join("records"),
            [0x51; 32],
            dir.path().join("freshness"),
            [0x52; 32],
        )
        .unwrap();
        let persona = PersonaId::new();
        let bytes = serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Account","email":"test@example.invalid","collections":[],"items":[{"id":"aQ","title":"Review card","credentials":[credential]}]}]})).unwrap();
        let choices = BTreeMap::from([(
            "YQ".to_string(),
            castellan::cxf::Destination {
                persona,
                profile: ProfileId("family".into()),
                ssh_unlock_policy: dramatis::intents::SshUnlockPolicyIntentV1::Session,
            },
        )]);
        castellan::cxf::Import::read(bytes)
            .unwrap()
            .commit(choices, &resident, &host)
            .unwrap();
        let ui = Arc::new(Ui::default());
        let keeper = Arc::new(Keeper::new(host.clone()).with_ui(ui.clone()));
        keeper.attach_credentials(resident.clone()).unwrap();
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        djinn::custody::grant(&grants).unwrap();
        #[cfg(windows)]
        let endpoint = format!(r"\\.\pipe\dramatis-review-{}", uuid::Uuid::new_v4());
        #[cfg(not(windows))]
        let endpoint = dir.path().join("review.sock").display().to_string();
        let serving = endpoint.clone();
        let server = tokio::spawn(async move {
            let _ = serve_app_broker(
                &serving,
                keeper,
                grants,
                60_000,
                None,
                AppEndpointCatalog::default(),
            )
            .await;
        });
        let fixture = Self {
            _dir: dir,
            host,
            resident,
            ui,
            persona,
            endpoint,
            server,
        };
        for _ in 0..100 {
            if CustodyClient::open_at(&fixture.endpoint, AppId::new("dramatis"))
                .await
                .is_ok()
            {
                return fixture;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("broker must bind");
    }
    async fn client(&self, app: &str) -> CustodyClient {
        CustodyClient::open_at(&self.endpoint, AppId::new(app))
            .await
            .unwrap()
    }
    async fn request(&self, action: ReviewAction) -> ReviewRequest {
        let card = self
            .client("dramatis")
            .await
            .quarantine(ProfileId("family".into()))
            .await
            .unwrap()
            .remove(0);
        ReviewRequest {
            profile: ProfileId("family".into()),
            persona: card.persona,
            item: card.item.id,
            revision: card.revision,
            action,
        }
    }
    fn state(&self) -> ItemState {
        self.resident.items(self.persona).list().unwrap()[0].state
    }
}
fn credit_card() -> serde_json::Value {
    json!({"type":"credit-card","number":{"fieldType":"concealed-string","value":"4111111111111111"}})
}

#[tokio::test(flavor = "multi_thread")]
async fn cards_are_secret_free_scoped_and_denial_preserves_the_item() {
    let f = Fixture::new(credit_card()).await;
    let mut c = f.client("dramatis").await;
    assert!(
        c.quarantine(ProfileId("work".into()))
            .await
            .unwrap()
            .is_empty()
    );
    let cards = c.quarantine(ProfileId("family".into())).await.unwrap();
    assert_eq!(cards.len(), 1);
    assert!(
        !serde_json::to_string(&cards)
            .unwrap()
            .contains("4111111111111111")
    );
    let before = f.resident.items(f.persona).get(cards[0].item.id).unwrap();
    assert!(
        c.review_quarantine(f.request(ReviewAction::Accept).await)
            .await
            .is_err()
    );
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 1);
    assert_eq!(
        before,
        f.resident.items(f.persona).get(cards[0].item.id).unwrap()
    );
    assert_eq!(
        f.ui.seen.lock().unwrap()[0],
        ("family".into(), "Review card".into(), "Accept".into())
    );
}
#[tokio::test(flavor = "multi_thread")]
async fn resident_approval_accepts_and_replay_is_refused() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let r = f.request(ReviewAction::Accept).await;
    let mut c = f.client("dramatis").await;
    c.review_quarantine(r.clone()).await.unwrap();
    assert_eq!(f.state(), ItemState::Vault);
    assert!(
        c.quarantine(ProfileId("family".into()))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(c.review_quarantine(r).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 1);
}
#[tokio::test(flavor = "multi_thread")]
async fn resident_approval_deletes_only_the_exact_quarantined_item() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let r = f.request(ReviewAction::Delete).await;
    f.client("dramatis")
        .await
        .review_quarantine(r)
        .await
        .unwrap();
    assert!(f.resident.items(f.persona).list().unwrap().is_empty());
}
#[tokio::test(flavor = "multi_thread")]
async fn wrong_persona_stale_revision_and_ordinary_app_cannot_mutate() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let r = f.request(ReviewAction::Delete).await;
    let mut wrong = r.clone();
    wrong.profile = ProfileId("work".into());
    assert!(
        f.client("dramatis")
            .await
            .review_quarantine(wrong)
            .await
            .is_err()
    );
    let mut stale = r.clone();
    stale.revision = "old".into();
    assert!(
        f.client("dramatis")
            .await
            .review_quarantine(stale)
            .await
            .is_err()
    );
    assert!(matches!(
        f.client("turnstone").await.review_quarantine(r).await,
        Err(CustodyClientError::Refused(CustodyRefusal::NotServed))
    ));
    assert_eq!(f.state(), ItemState::Quarantined);
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
}
#[tokio::test(flavor = "multi_thread")]
async fn lock_during_native_consent_refuses_the_commit() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let host = f.host.clone();
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || host.lock_vault().unwrap()));
    let r = f.request(ReviewAction::Accept).await;
    assert!(matches!(
        f.client("dramatis").await.review_quarantine(r).await,
        Err(CustodyClientError::Refused(CustodyRefusal::Locked))
    ));
    assert_eq!(f.state(), ItemState::Quarantined);
}
#[tokio::test(flavor = "multi_thread")]
async fn deletion_during_native_consent_cannot_accept_a_replacement() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let store = f.resident.items(f.persona);
    let r = f.request(ReviewAction::Accept).await;
    let id = store.list().unwrap()[0].id;
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || store.delete(id).unwrap()));
    assert!(
        f.client("dramatis")
            .await
            .review_quarantine(r)
            .await
            .is_err()
    );
}
#[tokio::test(flavor = "multi_thread")]
async fn unsupported_otp_and_unknown_types_stay_held_but_can_be_deleted() {
    for credential in [
        json!({"type":"totp","secret":"MY======","algorithm":"sha1","period":30,"digits":11}),
        json!({"type":"future-kind","secret":"synthetic-hidden-marker"}),
    ] {
        let f = Fixture::new(credential).await;
        f.ui.allow.store(true, Ordering::SeqCst);
        let mut c = f.client("dramatis").await;
        let cards = c.quarantine(ProfileId("family".into())).await.unwrap();
        assert!(cards[0].accept_blocked.is_some());
        assert!(
            !serde_json::to_string(&cards)
                .unwrap()
                .contains("synthetic-hidden-marker")
        );
        assert!(
            c.review_quarantine(f.request(ReviewAction::Accept).await)
                .await
                .is_err()
        );
        assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
        assert_eq!(f.state(), ItemState::Quarantined);
        c.review_quarantine(f.request(ReviewAction::Delete).await)
            .await
            .unwrap();
        assert!(f.resident.items(f.persona).list().unwrap().is_empty());
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn malformed_known_payload_is_not_activated() {
    let f = Fixture::new(json!({"type":"credit-card","number":false})).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    assert!(
        f.client("dramatis")
            .await
            .review_quarantine(f.request(ReviewAction::Accept).await)
            .await
            .is_err()
    );
    assert_eq!(f.state(), ItemState::Quarantined);
}
#[tokio::test(flavor = "multi_thread")]
async fn a_locked_resident_keeps_public_roster_but_refuses_cards_and_selection() {
    let f = Fixture::new(credit_card()).await;
    f.host.lock_vault().unwrap();
    let mut c = f.client("dramatis").await;
    assert_eq!(c.roster().await.unwrap().entries.len(), 2);
    assert!(c.quarantine(ProfileId("family".into())).await.is_err());
    assert!(c.choose_profile(ProfileId("work".into())).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
}
#[tokio::test(flavor = "multi_thread")]
async fn existing_namespace_binding_cannot_be_reassigned() {
    let f = Fixture::new(credit_card()).await;
    assert!(
        f.resident
            .bind_profile(f.persona, &ProfileId("work".into()))
            .is_err()
    );
    assert!(
        f.client("dramatis")
            .await
            .quarantine(ProfileId("work".into()))
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn switching_during_consent_keeps_review_bound_to_its_named_profile() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let host = f.host.clone();
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || {
        host.switch_profile(dramatis::intents::SwitchProfileIntentV1 {
            profile: "work".into(),
        })
        .unwrap();
    }));
    let r = f.request(ReviewAction::Accept).await;
    f.client("dramatis")
        .await
        .review_quarantine(r)
        .await
        .unwrap();
    assert_eq!(f.state(), ItemState::Vault);
    assert_eq!(f.host.current_profile_id(), ProfileId("work".into()));
    assert!(
        f.client("dramatis")
            .await
            .quarantine(ProfileId("work".into()))
            .await
            .unwrap()
            .is_empty()
    );
}

#[test]
fn the_wire_cannot_assert_owner_consent() {
    let request = serde_json::json!({"profile":"family", "persona":uuid::Uuid::nil(), "item":uuid::Uuid::nil(), "revision":"x", "action":"accept", "approved":true});
    assert!(serde_json::from_value::<ReviewRequest>(request).is_err());
}

fn older_namespace(f: &Fixture) -> PersonaId {
    let persona = PersonaId::new();
    f.resident
        .otp_items(persona)
        .import_otpauth_uri("otpauth://totp/Receipt:older?secret=MY======&issuer=Receipt")
        .unwrap();
    persona
}
#[tokio::test(flavor = "multi_thread")]
async fn older_import_assignment_needs_consent_and_preserves_item_state() {
    let f = Fixture::new(credit_card()).await;
    let persona = older_namespace(&f);
    let mut c = f.client("dramatis").await;
    let imports = c.unassigned_imports().await.unwrap();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].persona, *persona.as_uuid());
    assert!(
        !serde_json::to_string(&imports)
            .unwrap()
            .contains("MY======")
    );
    let request = graphshell::native::quarantine::AssignImportRequest {
        profile: ProfileId("work".into()),
        persona: *persona.as_uuid(),
        revision: imports[0].revision.clone(),
    };
    let before = f.resident.items(persona).list().unwrap();
    assert!(c.assign_import(request.clone()).await.is_err());
    assert_eq!(c.unassigned_imports().await.unwrap(), imports);
    f.ui.allow.store(true, Ordering::SeqCst);
    c.assign_import(request.clone()).await.unwrap();
    assert!(c.unassigned_imports().await.unwrap().is_empty());
    assert_eq!(f.resident.items(persona).list().unwrap(), before);
    assert!(
        f.resident
            .profile_personas(&ProfileId("work".into()))
            .unwrap()
            .contains(&persona)
    );
    assert!(c.assign_import(request).await.is_err());
}
#[tokio::test(flavor = "multi_thread")]
async fn older_import_assignment_rechecks_revision_and_lock_after_consent() {
    for locked in [false, true] {
        let f = Fixture::new(credit_card()).await;
        let persona = older_namespace(&f);
        let mut c = f.client("dramatis").await;
        let import = c.unassigned_imports().await.unwrap().remove(0);
        let request = graphshell::native::quarantine::AssignImportRequest {
            profile: ProfileId("work".into()),
            persona: *persona.as_uuid(),
            revision: import.revision,
        };
        f.ui.allow.store(true, Ordering::SeqCst);
        let host = f.host.clone();
        let store = f.resident.items(persona);
        let id = store.list().unwrap()[0].id;
        *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || {
            if locked {
                host.lock_vault().unwrap()
            } else {
                store.delete(id).unwrap()
            }
        }));
        assert!(c.assign_import(request).await.is_err());
        assert!(
            f.resident
                .profile_personas(&ProfileId("work".into()))
                .unwrap()
                .is_empty()
        );
    }
}
