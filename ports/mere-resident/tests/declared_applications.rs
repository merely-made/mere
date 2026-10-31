// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared applications over the real door (capability door plan, M1).
//!
//! Knot carries its own declaration; Mere's default grants decide what it
//! receives. Nothing in the resident names Knot: its epoch keys come from its
//! declaration within the owner grant, an undeclared or ungranted request is
//! refused with that reason, a label-admitted application no longer reaches
//! Knot's keys, and a provider offers only the projections it declares.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityStorage, IdentityVault, InMemoryStorage, Profile};
use djinn::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId};
use djinn::door::app_client::AppConnection;
use djinn::door::custody::{CustodyCall, CustodyRefusal, EpochKeyRequest};
use djinn::door::custody_client::{CustodyClient, CustodyClientError};
use djinn::door::declaration::{
    AppDeclaration, AppGrant, CustodyOperation, CustodyUse, EpochUse, NamePattern,
};
use dramatis::view::VaultProtectionView;
use knot_editor::PersonalVaultKeys;
use mere_resident::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use mere_resident::door::projection_offers::ProviderPermission;
use mere_resident::keeper::Keeper;
use personae::{Ed25519Keypair, IdentityProvider, PersonaId, ProfileId};

const DEVICE: &str = "declared-applications-fixture";

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\declared-apps-{}", uuid::Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!("declared-apps-{}.sock", uuid::Uuid::new_v4()))
        .display()
        .to_string();
}

fn persona() -> PersonaId {
    PersonaId::from_uuid(uuid::Uuid::from_u128(0xdec1_2026_1030))
}

fn resident_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("default".into()),
        "Default",
        Ed25519Keypair::from_seed([0x5d; 32]),
    );
    let storage = InMemoryStorage::new();
    storage.save_profile(&profile).unwrap();
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

/// Serve the door as the resident does: label custody, Mere's default
/// application grants, and `extra` grants for fixture applications.
async fn serve(wallet: &Path, extra: Vec<AppGrant>, catalog: AppEndpointCatalog) -> String {
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    mere_resident::custody::grant(&grants).unwrap();
    let mut policy = mere_resident::applications::default_grants();
    policy.extend(extra);
    mere_resident::applications::install(
        &grants,
        mere_resident::applications::validated(policy).unwrap(),
    );
    let endpoint = endpoint_name();
    let keeper = Arc::new(Keeper::new(resident_host()).with_wallet_root(Some(wallet.to_path_buf())));
    let serving = endpoint.clone();
    tokio::spawn(async move {
        let _ = serve_app_broker(&serving, keeper, grants, 60_000, None, catalog).await;
    });
    endpoint
}

async fn open(
    endpoint: &str,
    app: &str,
    declaration: Option<AppDeclaration>,
) -> Result<CustodyClient, CustodyClientError> {
    let mut last = None;
    for _ in 0..100 {
        let opened = match &declaration {
            Some(declaration) => {
                CustodyClient::open_declared_at(endpoint, AppId::new(app), declaration.clone()).await
            },
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

fn release(keys: Vec<EpochKeyRequest>) -> CustodyCall {
    CustodyCall::ReleaseEpochKeys {
        persona: *persona().as_uuid(),
        device_label: Some(DEVICE.into()),
        keys,
    }
}

fn refusal(result: Result<impl std::fmt::Debug, CustodyClientError>) -> CustodyRefusal {
    match result {
        Err(CustodyClientError::Refused(refusal)) => refusal,
        other => panic!("expected a custody refusal, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn knot_receives_its_declared_custody_within_the_default_grant() {
    let wallet = tempfile::tempdir().unwrap();
    castellan::custody::wallet::ensure_wallet_state(wallet.path(), persona(), DEVICE).unwrap();
    let endpoint = serve(wallet.path(), Vec::new(), AppEndpointCatalog::default()).await;
    let knot = knot_editor::declaration::app_declaration();

    // Knot's three epoch keys, exactly as the in-process release derives them.
    let mut client = open(&endpoint, knot_editor::KNOT_APP_ID, Some(knot.clone()))
        .await
        .expect("Knot is admitted by its declaration");
    client.status().await.unwrap();
    client.roster().await.unwrap();
    let released = client
        .release_epoch_keys(*persona().as_uuid(), Some(DEVICE.into()), PersonalVaultKeys::requests())
        .await
        .unwrap();
    let expected = mere_resident::custody::epoch_keys(
        &mere_resident::custody::ReleasePolicy::none().with_declared_epochs(&knot.custody.epoch),
        wallet.path(),
        persona(),
        Some(DEVICE),
        &PersonalVaultKeys::requests(),
    )
    .unwrap();
    assert_eq!(released, expected);
    PersonalVaultKeys::from_released(&released).unwrap();

    // What Knot does not declare is refused as undeclared, before any policy.
    assert!(matches!(
        refusal(client.call(CustodyCall::LockVault).await),
        CustodyRefusal::Undeclared { .. }
    ));
    let legacy_secret = EpochKeyRequest::secret(knot_editor::SIGNING_KEY_CONTEXT, false);
    assert!(matches!(
        refusal(client.call(release(vec![legacy_secret])).await),
        CustodyRefusal::Undeclared { .. }
    ));
    assert!(matches!(
        refusal(
            client
                .release(djinn::door::custody::KeySource::Persona, vec![mesh::MESH_AUTHOR_SALT.to_vec()])
                .await
        ),
        CustodyRefusal::Undeclared { .. }
    ));

    // Knot's label without its declaration is refused at the door.
    assert!(open(&endpoint, knot_editor::KNOT_APP_ID, None).await.is_err());

    // A label-admitted application no longer reaches Knot's keys.
    let mut labelled = open(&endpoint, "cleromancy", None).await.unwrap();
    assert_eq!(
        refusal(labelled.call(release(PersonalVaultKeys::requests())).await),
        CustodyRefusal::NotReleasable
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_declared_request_beyond_the_owner_grant_is_refused_as_not_granted() {
    let wallet = tempfile::tempdir().unwrap();
    castellan::custody::wallet::ensure_wallet_state(wallet.path(), persona(), DEVICE).unwrap();
    let mut narrow = AppGrant::new(AppId::new("fixture-app"));
    narrow.routes = vec![NamePattern::exact("custody")];
    narrow.custody.operations = [CustodyOperation::Status, CustodyOperation::ReleaseEpochKeys].into();
    narrow.custody.epoch = vec![NamePattern::prefix("fixture.granted.")];
    let endpoint = serve(wallet.path(), vec![narrow], AppEndpointCatalog::default()).await;
    let declaration = AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        custody: CustodyUse {
            operations: [
                CustodyOperation::Status,
                CustodyOperation::Roster,
                CustodyOperation::ReleaseEpochKeys,
            ]
            .into(),
            epoch: vec![
                EpochUse::secret("fixture.granted.key", false),
                EpochUse::secret("fixture.other.key", false),
            ],
            ..CustodyUse::default()
        },
        ..AppDeclaration::default()
    };
    let mut client = open(&endpoint, "fixture-app", Some(declaration)).await.unwrap();
    client.status().await.unwrap();
    assert!(matches!(
        refusal(client.roster().await),
        CustodyRefusal::NotGranted { .. }
    ));
    assert!(matches!(
        refusal(client.call(release(vec![EpochKeyRequest::secret("fixture.other.key", false)])).await),
        CustodyRefusal::NotGranted { .. }
    ));
    let granted = client
        .call(release(vec![EpochKeyRequest::secret("fixture.granted.key", false)]))
        .await;
    assert!(
        granted.is_ok(),
        "a context the application declares and the owner grants is released: {granted:?}"
    );

    // A route the application does not declare is refused at the door.
    let undeclared = AppDeclaration {
        routes: vec![NamePattern::exact("published-site-v1")],
        ..AppDeclaration::default()
    };
    assert!(open(&endpoint, "fixture-app", Some(undeclared)).await.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn declared_salts_release_and_sign_only_within_declaration_and_grant() {
    use djinn::door::custody::KeySource;
    use djinn::door::declaration::{SaltPattern, SaltUse};
    let wallet = tempfile::tempdir().unwrap();
    let mut grant = AppGrant::new(AppId::new("salted-app"));
    grant.routes = vec![NamePattern::exact("custody")];
    grant.custody.operations =
        [CustodyOperation::Release, CustodyOperation::Sign, CustodyOperation::Attest].into();
    grant.custody.salts = vec![SaltUse::persona(SaltPattern::Prefix("salted.".into()))];
    grant.custody.acts = vec![SaltUse::persona(SaltPattern::Prefix("salted-acts/".into()))];
    let endpoint = serve(wallet.path(), vec![grant], AppEndpointCatalog::default()).await;
    let mut declaration = AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        ..AppDeclaration::default()
    };
    declaration.custody.operations =
        [CustodyOperation::Release, CustodyOperation::Sign, CustodyOperation::Attest].into();
    declaration.custody.salts = vec![
        SaltUse::persona(SaltPattern::Prefix("salted.place.".into())),
        SaltUse::persona(SaltPattern::Exact("undeclared-by-grant".into())),
    ];
    declaration.custody.acts = vec![SaltUse::persona(SaltPattern::Prefix("salted-acts/".into()))];
    let mut client = open(&endpoint, "salted-app", Some(declaration)).await.unwrap();

    // Released within both, and not from the label policy, which names none of these.
    let released = client
        .release(KeySource::Persona, vec![b"salted.place.one".to_vec()])
        .await
        .expect("a declared, granted salt is released");
    assert!(released.derive_keypair(b"salted.place.one").is_ok());
    // Signed with, never released.
    let act = [b"salted-acts/".as_slice(), &[0xff, 0x01]].concat();
    client
        .sign(KeySource::Persona, act.clone(), b"m".to_vec())
        .await
        .expect("a declared act signs");
    assert!(matches!(
        refusal(client.release(KeySource::Persona, vec![act]).await),
        CustodyRefusal::Undeclared { .. }
    ));
    assert!(matches!(
        refusal(client.release(KeySource::Persona, vec![b"undeclared-by-grant".to_vec()]).await),
        CustodyRefusal::NotGranted { .. }
    ));
    assert!(matches!(
        refusal(client.release(KeySource::Persona, vec![mesh::MESH_AUTHOR_SALT.to_vec()]).await),
        CustodyRefusal::Undeclared { .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn turnstone_releases_its_place_and_scope_keys_within_the_default_grant() {
    use djinn::door::custody::KeySource;
    use djinn::door::declaration::{SaltPattern, SaltUse, delegation_scope_prefix};
    let wallet = tempfile::tempdir().unwrap();
    let endpoint = serve(wallet.path(), Vec::new(), AppEndpointCatalog::default()).await;
    let mut declaration = AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        ..AppDeclaration::default()
    };
    declaration.custody.operations =
        [CustodyOperation::Status, CustodyOperation::Release, CustodyOperation::Attest].into();
    declaration.custody.salts = vec![
        SaltUse::persona(SaltPattern::Prefix("turnstone.place.".into())),
        SaltUse::persona(SaltPattern::DelegationScope("moot".into())),
        SaltUse::persona(SaltPattern::DelegationScope("mere.graphshell".into())),
    ];
    let mut client = open(&endpoint, "turnstone", Some(declaration)).await.unwrap();
    client
        .release(KeySource::Persona, vec![b"turnstone.place.transport.v1/moot".to_vec()])
        .await
        .expect("a place key is released");
    let scope = |domain: &str| [delegation_scope_prefix(domain), vec![4, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4]].concat();
    client
        .release(KeySource::Persona, vec![scope("moot")])
        .await
        .expect("a Moot's delegation scope key is released");
    // The owner grants a place's Graphshell guest scope as an act only.
    client
        .attest(KeySource::Persona, scope("mere.graphshell"))
        .await
        .expect("the Graphshell guest scope is attested");
    assert!(matches!(
        refusal(client.release(KeySource::Persona, vec![scope("mere.graphshell")]).await),
        CustodyRefusal::NotGranted { .. }
    ));
    // Turnstone's label without its declaration is refused at the door.
    assert!(open(&endpoint, "turnstone", None).await.is_err());
}

/// I1: a label the owner pinned to a macOS code signature is refused for a
/// peer without it; the same grant unpinned is admitted (development mode).
#[cfg(target_os = "macos")]
#[tokio::test(flavor = "multi_thread")]
async fn a_label_pinned_to_a_signature_refuses_an_unsigned_peer() {
    use djinn::door::declaration::{AppIdentity, SignedAppIdentity};
    let wallet = tempfile::tempdir().unwrap();
    let grant = |app: &str, pinned: bool| {
        let mut grant = AppGrant::new(AppId::new(app));
        grant.routes = vec![NamePattern::exact("custody")];
        grant.custody.operations = [CustodyOperation::Status].into();
        grant.identity = pinned.then(|| AppIdentity {
            macos: Some(SignedAppIdentity { team: "ABCDE12345".into(), identifier: "com.merely.pinned".into() }),
            ..AppIdentity::default()
        });
        grant
    };
    let endpoint = serve(
        wallet.path(),
        vec![grant("pinned-app", true), grant("unpinned-app", false)],
        AppEndpointCatalog::default(),
    )
    .await;
    let declaration = AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        custody: CustodyUse {
            operations: [CustodyOperation::Status].into(),
            ..CustodyUse::default()
        },
        ..AppDeclaration::default()
    };
    let mut unpinned = open(&endpoint, "unpinned-app", Some(declaration.clone())).await.unwrap();
    unpinned.status().await.unwrap();
    assert!(
        open(&endpoint, "pinned-app", Some(declaration)).await.is_err(),
        "this test binary is not signed by the pinned team"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_declared_provider_offers_only_what_it_declares() {
    use graphshell_endpoint::door::projection_offer::PROJECTION_PROVIDER_ROUTE;
    let wallet = tempfile::tempdir().unwrap();
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("field.knot"), "# Field\n").unwrap();
    let route = AppRouteId::new("knot-documents").unwrap();
    let other = AppRouteId::new("knot-other").unwrap();
    let catalog = AppEndpointCatalog::default();
    catalog
        .set_provider_permissions(
            [&route, &other]
                .into_iter()
                .map(|route| ProviderPermission {
                    provider: AppId::new(knot_editor::KNOT_APP_ID),
                    route: route.clone(),
                    viewers: vec![AppId::new("fixture-viewer")],
                    max_sessions: 2,
                    timeout: Duration::from_secs(3),
                })
                .collect(),
        )
        .await
        .unwrap();
    let endpoint = serve(wallet.path(), Vec::new(), catalog.clone()).await;
    let provider_route = AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap();
    let connect = |declaration: AppDeclaration| {
        let (endpoint, provider_route) = (endpoint.clone(), provider_route.clone());
        async move {
            let mut last = None;
            for _ in 0..100 {
                match AppConnection::open_declared_route_at(
                    &endpoint,
                    AppId::new(knot_editor::KNOT_APP_ID),
                    provider_route.clone(),
                    declaration.clone(),
                )
                .await
                {
                    Ok(connection) => return connection,
                    Err(error) => last = Some(error),
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            panic!("provider connection failed: {last:?}");
        }
    };
    let offered = |route: &AppRouteId| {
        let (catalog, id) = (catalog.clone(), route.as_str().to_owned());
        async move {
            catalog
                .update(|catalog| catalog.offers().iter().any(|offer| offer.id == id))
                .await
        }
    };

    // Offering a route the declaration does not name is refused.
    let undeclared = connect(knot_editor::declaration::offering(route.clone(), "Knot directory")).await;
    let root = directory.path().to_owned();
    let refused = tokio::time::timeout(
        Duration::from_secs(4),
        projection_provider_serve(undeclared, other.clone(), root.clone()),
    )
    .await
    .expect("an undeclared offer is answered, not served");
    assert!(refused.is_err(), "an undeclared offer is refused");
    assert!(!offered(&other).await);

    // The declared offer registers.
    let declared = connect(knot_editor::declaration::offering(route.clone(), "Knot directory")).await;
    let serving = tokio::spawn(projection_provider_serve(declared, route.clone(), root));
    tokio::time::timeout(Duration::from_secs(4), async {
        while !offered(&route).await {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("the declared offer registers");
    serving.abort();
}

async fn projection_provider_serve(
    connection: AppConnection,
    route: AppRouteId,
    root: std::path::PathBuf,
) -> Result<(), djinn::door::app_client::AppClientError> {
    graphshell_endpoint::door::projection_provider::serve(
        connection,
        route,
        "Knot directory".into(),
        2,
        move |_| knot_editor::KnotEndpoint::open(&root).map_err(|error| error.to_string()),
    )
    .await
}
