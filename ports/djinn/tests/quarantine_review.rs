// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real owner-local custody wire with synthetic credentials, the resident's
//! native consent and a real custodian holding the catalog (custody
//! isolation plan, H2). Djinn shows the exact item and forwards only an
//! approved act; the custodian re-checks the item or import revision under
//! profile authority. What happens to an item after a forwarded act is the
//! custodian's own test (`djinn-custodian` credential tests).
#![cfg(all(unix, feature = "cxf"))]

mod custodian_fixture;

use custodian_fixture::{TestCustodian, manage_grants};
use mere_resident::custody_host::remote::RemoteHost;
use mere_resident::custody_host::{CredentialCall, ResidentCustody};
use mere_resident::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use mere_resident::door::browser_carrier::NativeIdentityFailure;
use mere_resident::{identity_ui::NativeIdentityUi, keeper::Keeper};
use djinn::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use djinn::door::custody::CustodyRefusal;
use djinn::door::custody_client::{CustodyClient, CustodyClientError};
use djinn::door::quarantine::{QuarantineCard, ReviewAction, ReviewRequest};
use personae::{PersonaId, ProfileId};
use serde_json::json;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

const PASSPHRASE: &str = "synthetic review receipt";

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
    /// Keeps the custodian's vault and socket for the fixture's life.
    _custodian: TestCustodian,
    remote: Arc<RemoteHost>,
    ui: Arc<Ui>,
    endpoint: String,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

fn cxf(title: &str, credential: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Account","email":"test@example.invalid","collections":[],"items":[{"id":"aQ","title":title,"credentials":[credential]}]}]})).unwrap()
}

/// Import through the custodian into the store of the profile it has open.
fn import(remote: &RemoteHost, bytes: Vec<u8>, persona: PersonaId, profile: &str) {
    let preview = remote.credentials(CredentialCall::CxfPreview(bytes.clone())).unwrap();
    remote
        .credentials(CredentialCall::CxfImport {
            source: bytes,
            destinations: json!({"YQ": {"persona": persona, "profile": profile}}),
            preview_sha256: preview["source_sha256"].as_str().unwrap().into(),
        })
        .unwrap();
}

/// An older import: an item in a namespace no profile owns, written into
/// the family store before the custodian first claims it.
fn seed_older_namespace(custodian: &TestCustodian) -> PersonaId {
    let root = djinn_custodian::host::credential_profile_root(&custodian.data_root, &ProfileId("family".into()));
    let family = custodian.direct("family");
    let store = castellan::resident::CastellanResident::claim_derived(
        root.join("records"),
        root.join("freshness"),
        &family,
        djinn_custodian::host::CREDENTIAL_SALTS,
    )
    .unwrap();
    let persona = PersonaId::new();
    store
        .otp_items(persona)
        .import_otpauth_uri(
            "otpauth://totp/Receipt:older?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Receipt",
        )
        .unwrap();
    persona
}

/// A CXF card committed into `profile`'s own store before the custodian
/// first claims it, as an earlier resident under that profile would have.
fn seed_card(custodian: &TestCustodian, profile: &str, title: &str) {
    use castellan::custody::bootstrap::{self, Unlock};
    let id = ProfileId(profile.into());
    let root = djinn_custodian::host::credential_profile_root(&custodian.data_root, &id);
    let opened = bootstrap::open_storage(&custodian.vault, Unlock::passphrase(PASSPHRASE)).unwrap();
    let host = castellan::authority::PersonaeHost::new(
        castellan::custody::IdentityVault::open(opened.storage, &id).unwrap(),
        None,
        dramatis::view::VaultProtectionView::Passphrase,
    );
    let store = castellan::resident::CastellanResident::claim_derived(
        root.join("records"),
        root.join("freshness"),
        &host,
        djinn_custodian::host::CREDENTIAL_SALTS,
    )
    .unwrap();
    let choices = std::collections::BTreeMap::from([(
        "YQ".to_string(),
        castellan::cxf::Destination {
            persona: PersonaId::new(),
            profile: id,
            ssh_unlock_policy: dramatis::intents::SshUnlockPolicyIntentV1::Session,
        },
    )]);
    castellan::cxf::Import::read(cxf(title, credit_card()))
        .unwrap()
        .commit(choices, &store, &host)
        .unwrap();
}

impl Fixture {
    async fn new(credential: serde_json::Value) -> Self {
        Self::build(credential, &|_| None).await.0
    }

    /// `seed` writes into the data root before the custodian first opens it.
    async fn build(
        credential: serde_json::Value,
        seed: &dyn Fn(&TestCustodian) -> Option<PersonaId>,
    ) -> (Self, Option<PersonaId>) {
        let custodian = TestCustodian::start(PASSPHRASE, &[("family", [0x43; 32]), ("work", [0x44; 32])], manage_grants());
        let older = seed(&custodian);
        let remote = custodian.remote();
        remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
        import(&remote, cxf("Review card", credential), PersonaId::new(), "family");
        let ui = Arc::new(Ui::default());
        let keeper = Arc::new(Keeper::new(remote.clone()).with_ui(ui.clone()));
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        mere_resident::custody::grant(&grants).unwrap();
        let endpoint = custodian.dir.path().join("review.sock").display().to_string();
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
            _custodian: custodian,
            remote,
            ui,
            endpoint,
            server,
        };
        for _ in 0..100 {
            if CustodyClient::open_at(&fixture.endpoint, AppId::new("dramatis"))
                .await
                .is_ok()
            {
                return (fixture, older);
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
    async fn cards(&self, profile: &str) -> Vec<QuarantineCard> {
        self.client("dramatis")
            .await
            .quarantine(ProfileId(profile.into()))
            .await
            .unwrap()
    }
    async fn request(&self, action: ReviewAction) -> ReviewRequest {
        let card = self.cards("family").await.remove(0);
        ReviewRequest {
            store: card.store,
            profile: ProfileId("family".into()),
            persona: card.persona,
            item: card.item.id,
            revision: card.revision,
            action,
        }
    }
    /// The family card is still held, unchanged.
    async fn still_held(&self, request: &ReviewRequest) {
        let cards = self.cards("family").await;
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].item.id, request.item);
        assert_eq!(cards[0].revision, request.revision);
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
    let request = f.request(ReviewAction::Accept).await;
    assert!(c.review_quarantine(request.clone()).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 1);
    f.still_held(&request).await;
    assert_eq!(
        f.ui.seen.lock().unwrap()[0],
        (
            "family".into(),
            "Review card (import store family)".into(),
            "Accept".into()
        )
    );
}
#[tokio::test(flavor = "multi_thread")]
async fn resident_approval_accepts_and_replay_is_refused() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let r = f.request(ReviewAction::Accept).await;
    let mut c = f.client("dramatis").await;
    c.review_quarantine(r.clone()).await.unwrap();
    assert!(f.cards("family").await.is_empty());
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
    assert!(f.cards("family").await.is_empty());
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
        f.client("turnstone").await.review_quarantine(r.clone()).await,
        Err(CustodyClientError::Refused(CustodyRefusal::NotServed))
    ));
    f.still_held(&r).await;
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
}
#[tokio::test(flavor = "multi_thread")]
async fn lock_during_native_consent_refuses_the_commit() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let remote = f.remote.clone();
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || remote.lock_vault().unwrap()));
    let r = f.request(ReviewAction::Accept).await;
    assert!(matches!(
        f.client("dramatis").await.review_quarantine(r.clone()).await,
        Err(CustodyClientError::Refused(CustodyRefusal::Locked))
    ));
    f.remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    f.still_held(&r).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn deletion_during_native_consent_cannot_accept_a_replacement() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let r = f.request(ReviewAction::Accept).await;
    let mut delete = r.clone();
    delete.action = ReviewAction::Delete;
    let remote = f.remote.clone();
    let delete = serde_json::to_value(&delete).unwrap();
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || {
        remote.credentials(CredentialCall::Review(delete.clone())).unwrap();
    }));
    assert!(
        f.client("dramatis")
            .await
            .review_quarantine(r)
            .await
            .is_err()
    );
    assert!(f.cards("family").await.is_empty());
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
        let accept = f.request(ReviewAction::Accept).await;
        assert!(c.review_quarantine(accept.clone()).await.is_err());
        assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
        f.still_held(&accept).await;
        c.review_quarantine(f.request(ReviewAction::Delete).await)
            .await
            .unwrap();
        assert!(f.cards("family").await.is_empty());
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn malformed_known_payload_is_not_activated() {
    let f = Fixture::new(json!({"type":"credit-card","number":false})).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let accept = f.request(ReviewAction::Accept).await;
    assert!(
        f.client("dramatis")
            .await
            .review_quarantine(accept.clone())
            .await
            .is_err()
    );
    f.still_held(&accept).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn a_locked_resident_keeps_public_roster_but_refuses_cards_and_selection() {
    let f = Fixture::new(credit_card()).await;
    f.remote.lock_vault().unwrap();
    let mut c = f.client("dramatis").await;
    assert_eq!(c.roster().await.unwrap().entries.len(), 2);
    assert!(c.quarantine(ProfileId("family".into())).await.is_err());
    assert!(c.choose_profile(ProfileId("work".into())).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
}
#[tokio::test(flavor = "multi_thread")]
async fn an_owned_namespace_is_not_offered_for_assignment() {
    let f = Fixture::new(credit_card()).await;
    let card = f.cards("family").await.remove(0);
    let mut c = f.client("dramatis").await;
    assert!(c.unassigned_imports().await.unwrap().is_empty());
    let request = djinn::door::quarantine::AssignImportRequest {
        store: ProfileId("family".into()),
        profile: ProfileId("work".into()),
        persona: card.persona,
        revision: card.revision,
    };
    assert!(c.assign_import(request).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
    assert!(f.cards("work").await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn switching_during_consent_keeps_review_bound_to_its_named_profile() {
    let f = Fixture::new(credit_card()).await;
    f.ui.allow.store(true, Ordering::SeqCst);
    let remote = f.remote.clone();
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || {
        remote
            .switch_profile(dramatis::intents::SwitchProfileIntentV1 {
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
    assert!(f.cards("family").await.is_empty());
    assert_eq!(f.remote.current_profile_id(), ProfileId("work".into()));
    assert!(f.cards("work").await.is_empty());
}

#[test]
fn the_wire_cannot_assert_owner_consent() {
    let request = serde_json::json!({"profile":"family", "persona":uuid::Uuid::nil(), "item":uuid::Uuid::nil(), "revision":"x", "action":"accept", "approved":true});
    assert!(serde_json::from_value::<ReviewRequest>(request).is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn older_import_assignment_needs_consent_and_is_not_replayed() {
    let (f, persona) = Fixture::build(credit_card(), &|c| Some(seed_older_namespace(c))).await;
    let persona = persona.unwrap();
    let mut c = f.client("dramatis").await;
    let imports = c.unassigned_imports().await.unwrap();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].persona, *persona.as_uuid());
    assert!(
        !serde_json::to_string(&imports)
            .unwrap()
            .contains("GEZDGNBV")
    );
    let request = djinn::door::quarantine::AssignImportRequest {
        store: ProfileId("family".into()),
        profile: ProfileId("work".into()),
        persona: *persona.as_uuid(),
        revision: imports[0].revision.clone(),
    };
    assert!(c.assign_import(request.clone()).await.is_err());
    assert_eq!(c.unassigned_imports().await.unwrap(), imports);
    f.ui.allow.store(true, Ordering::SeqCst);
    c.assign_import(request.clone()).await.unwrap();
    assert!(c.unassigned_imports().await.unwrap().is_empty());
    assert!(c.assign_import(request).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 2);
}
#[tokio::test(flavor = "multi_thread")]
async fn older_import_assignment_rechecks_the_lock_after_consent() {
    let (f, persona) = Fixture::build(credit_card(), &|c| Some(seed_older_namespace(c))).await;
    let persona = persona.unwrap();
    let mut c = f.client("dramatis").await;
    let import = c.unassigned_imports().await.unwrap().remove(0);
    let request = djinn::door::quarantine::AssignImportRequest {
        store: ProfileId("family".into()),
        profile: ProfileId("work".into()),
        persona: *persona.as_uuid(),
        revision: import.revision.clone(),
    };
    f.ui.allow.store(true, Ordering::SeqCst);
    let remote = f.remote.clone();
    *f.ui.on_prompt.lock().unwrap() = Some(Box::new(move || remote.lock_vault().unwrap()));
    assert!(c.assign_import(request).await.is_err());
    f.remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    assert_eq!(c.unassigned_imports().await.unwrap(), vec![import]);
}

#[tokio::test(flavor = "multi_thread")]
async fn persona_selection_and_lock_preserve_the_original_credential_root() {
    let f = Fixture::new(credit_card()).await;
    f.remote
        .switch_profile(dramatis::intents::SwitchProfileIntentV1 {
            profile: "work".into(),
        })
        .unwrap();
    f.remote.lock_vault().unwrap();
    f.remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    assert_eq!(f.remote.current_profile_id(), ProfileId("work".into()));
    assert_eq!(f.cards("family").await.len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn multiple_stores_keep_keys_and_review_targets_distinct() {
    let (f, _) = Fixture::build(credit_card(), &|c| {
        seed_card(c, "work", "Work card");
        None
    })
    .await;
    f.remote
        .switch_profile(dramatis::intents::SwitchProfileIntentV1 {
            profile: "work".into(),
        })
        .unwrap();
    f.remote.lock_vault().unwrap();
    f.remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    let mut c = f.client("dramatis").await;
    let family = c.quarantine(ProfileId("family".into())).await.unwrap();
    let work = c.quarantine(ProfileId("work".into())).await.unwrap();
    assert_eq!(family.len(), 1);
    assert_eq!(work.len(), 1);
    assert_eq!(family[0].store, ProfileId("family".into()));
    assert_eq!(work[0].store, ProfileId("work".into()));
    let request = ReviewRequest {
        store: work[0].store.clone(),
        profile: ProfileId("work".into()),
        persona: work[0].persona,
        item: work[0].item.id,
        revision: work[0].revision.clone(),
        action: ReviewAction::Accept,
    };
    let mut forged = request.clone();
    forged.store = ProfileId("family".into());
    assert!(c.review_quarantine(forged).await.is_err());
    assert_eq!(f.ui.count.load(Ordering::SeqCst), 0);
    f.ui.allow.store(true, Ordering::SeqCst);
    c.review_quarantine(request).await.unwrap();
    assert!(
        c.quarantine(ProfileId("work".into()))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(c.quarantine(ProfileId("family".into())).await.unwrap(), family);
}
