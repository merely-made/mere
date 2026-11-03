// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Applications are admitted by declaration and the owner's grant alone
//! (capability door plan, M1, D24, D25), over the real door.
//!
//! A former label application gets nothing by its name; a request outside
//! an application's declaration is refused with a typed reason; an
//! application nobody has granted is put to the owner once, and a yes is
//! written to the owner's settings and admits its next connection, while a
//! no is kept.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityStorage, IdentityVault, InMemoryStorage, Profile};
use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use ostiary::app_admission::{AppAdmissionError, AppId, AppRequest, AppRouteId};
use castellan_client::custody::CustodyRefusal;
use castellan_client::custody_client::{CustodyClient, CustodyClientError};
use ostiary::declaration::{AppDeclaration, CustodyOperation, CustodyUse, NamePattern};
use dramatis::view::VaultProtectionView;
use mere_resident::applications::OwnerConsent;
use mere_resident::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use dramatis_resident::native::NativeIdentityFailure;
use dramatis_resident::identity_ui::NativeIdentityUi;
use dramatis_resident::keeper::Keeper;
use mere_resident::settings::OwnerSettings;
use personae::{Ed25519Keypair, ProfileId};
use zeroize::Zeroizing;

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\app-consent-{}", uuid::Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!("app-consent-{}.sock", uuid::Uuid::new_v4()))
        .display()
        .to_string();
}

fn resident_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(ProfileId("default".into()), "Default", Ed25519Keypair::from_seed([0x6d; 32]));
    let storage = InMemoryStorage::new();
    storage.save_profile(&profile).unwrap();
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

/// The owner, answering every confirm the same way and counting them.
struct Owner {
    yes: bool,
    asked: AtomicUsize,
}

impl NativeIdentityUi for Owner {
    fn confirm_application(&self, _app: &str, grant: &str) -> Result<bool, NativeIdentityFailure> {
        assert!(grant.contains("status"), "the dialog lists the declared calls: {grant}");
        self.asked.fetch_add(1, Ordering::SeqCst);
        Ok(self.yes)
    }
    fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn prompt_ssh_private_key_passphrase(&self) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
}

/// What a fixture application declares: custody status and roster.
fn declaration() -> AppDeclaration {
    AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        custody: CustodyUse {
            operations: [CustodyOperation::Status, CustodyOperation::Roster].into(),
            ..CustodyUse::default()
        },
        ..AppDeclaration::default()
    }
}

/// Serve the door as Djinn does: custody served to grants, no application
/// granted, and the owner asked about the rest.
async fn serve(settings: &Path, owner: Arc<Owner>) -> (String, AppRouteGrants) {
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    mere_resident::custody::grant(&grants).unwrap();
    let catalog = AppEndpointCatalog::default();
    catalog.set_consent(Arc::new(OwnerConsent::new(settings.to_path_buf(), grants.clone(), owner)));
    let endpoint = endpoint_name();
    let keeper = Arc::new(Keeper::new(resident_host()));
    let (serving, served) = (endpoint.clone(), grants.clone());
    tokio::spawn(async move {
        let _ = serve_app_broker(&serving, keeper, served, 60_000, None, catalog).await;
    });
    (endpoint, grants)
}

/// Connect until the door is up, or until the answer is in.
async fn connect(endpoint: &str, app: &str, declaration: Option<AppDeclaration>) -> Result<CustodyClient, CustodyClientError> {
    let mut last = None;
    for _ in 0..150 {
        let opened = match &declaration {
            Some(declaration) => CustodyClient::open_declared_at(endpoint, AppId::new(app), declaration.clone()).await,
            None => CustodyClient::open_at(endpoint, AppId::new(app)).await,
        };
        match opened {
            Ok(client) => return Ok(client),
            Err(error) => last = Some(error),
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(last.unwrap())
}

fn custody_request(app: &str, declaration: Option<AppDeclaration>) -> AppRequest {
    let mut request = AppRequest::new(AppId::new(app), AppRouteId::new("custody").unwrap());
    request.declaration = declaration;
    request
}

#[tokio::test(flavor = "multi_thread")]
async fn a_former_label_application_gets_nothing_by_its_name() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Arc::new(Owner { yes: false, asked: AtomicUsize::new(0) });
    let (endpoint, grants) = serve(&directory.path().join("settings.json"), owner.clone()).await;
    for app in ["woodshed", "hocket", "cleromancy", "graphshell", "distillery", "signalman", "personae-vault", "dramatis"] {
        assert_eq!(
            grants.current().admit(&custody_request(app, None)),
            Err(AppAdmissionError::NotAllowed(AppId::new(app))),
            "{app} holds no route by label"
        );
        assert!(connect(&endpoint, app, None).await.is_err());
    }
    assert_eq!(owner.asked.load(Ordering::SeqCst), 0, "an undeclared hello is not put to the owner");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_yes_is_kept_and_admits_exactly_the_declaration() {
    let directory = tempfile::tempdir().unwrap();
    let settings = directory.path().join("settings.json");
    let owner = Arc::new(Owner { yes: true, asked: AtomicUsize::new(0) });
    let (endpoint, grants) = serve(&settings, owner.clone()).await;
    let mut client = connect(&endpoint, "fixture-app", Some(declaration())).await.unwrap();
    assert_eq!(owner.asked.load(Ordering::SeqCst), 1, "asked once, then admitted");
    client.status().await.unwrap();
    client.roster().await.unwrap();
    assert!(
        matches!(client.lock_vault().await, Err(CustodyClientError::Refused(CustodyRefusal::Undeclared { .. }))),
        "a call outside the declaration is refused as undeclared"
    );
    let written = OwnerSettings::load(&settings).unwrap().applications.unwrap();
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].app, AppId::new("fixture-app"));
    assert_eq!(written[0].routes, declaration().routes);
    let wider = AppDeclaration {
        routes: vec![NamePattern::exact("custody"), NamePattern::exact("reservoir")],
        ..declaration()
    };
    let mut beyond = custody_request("fixture-app", Some(wider));
    beyond.route = AppRouteId::new("reservoir").unwrap();
    assert!(
        matches!(grants.current().admit(&beyond), Err(AppAdmissionError::RouteNotGranted { .. })),
        "a route beyond the grant is refused with its reason"
    );
    let mut undeclared = custody_request("fixture-app", Some(declaration()));
    undeclared.route = AppRouteId::new("reservoir").unwrap();
    assert!(matches!(
        grants.current().admit(&undeclared),
        Err(AppAdmissionError::RouteUndeclared { .. })
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_no_is_kept_and_not_asked_again() {
    let directory = tempfile::tempdir().unwrap();
    let settings = directory.path().join("settings.json");
    let owner = Arc::new(Owner { yes: false, asked: AtomicUsize::new(0) });
    let (endpoint, _) = serve(&settings, owner.clone()).await;
    assert!(connect(&endpoint, "fixture-app", Some(declaration())).await.is_err());
    assert_eq!(owner.asked.load(Ordering::SeqCst), 1, "asked once for every retry");
    let kept = OwnerSettings::load(&settings).unwrap();
    assert_eq!(kept.refused_applications, vec![AppId::new("fixture-app")]);
    assert!(kept.applications.is_none());

    // A fresh resident reads the refusal and does not ask.
    let again = Arc::new(Owner { yes: true, asked: AtomicUsize::new(0) });
    let (endpoint, _) = serve(&settings, again.clone()).await;
    assert!(connect(&endpoint, "fixture-app", Some(declaration())).await.is_err());
    assert_eq!(again.asked.load(Ordering::SeqCst), 0);
}
