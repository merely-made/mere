// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A live mere over a reservoir, its route and one shared app door, for tests.
//!
//! Compiled for this crate's own tests and, behind the `test-fixtures` feature,
//! for consumers that drive the route from outside (Knot's overlay tests).

use super::*;
use crate::resident_reservoir::{
    EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, ResidentReservoir,
    RESIDENT_RESERVOIR_ROUTE,
};
use djinn::door::app_admission::AllowedAppRoutes;
use graphshell_host::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointSession};
use uuid::Uuid;

/// A mere client's declaration, co-op routes included.
pub(super) fn mere_client() -> ostiary::declaration::AppDeclaration {
    let mut declaration = crate::resident_reservoir::mere_client_declaration();
    declaration.routes.push(ostiary::declaration::NamePattern::prefix("coop/"));
    declaration
}

pub fn persona(n: u128) -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(n))
}

pub fn context(app: &str) -> AdmittedEndpointContext {
    AdmittedEndpointContext::new(ProjectionSession("v1:test".into()), [0x2b; 32])
        .with_application(app)
}

pub(super) type RetainedHost = castellan::authority::PersonaeHost<castellan::custody::SealedProfileStorage>;
pub(super) const VAULT_PASSPHRASE: &[u8] = b"isolated retained-content fixture";

/// A real custodian host with a wallet and a staged epoch under `root`,
/// reopened if it is already there, and the sealing that uses it.
pub(super) fn retained_custody(
    root: &std::path::Path,
) -> (Arc<RetainedHost>, crate::retained_custody::RetainedSealing) {
    use castellan::custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage};
    let dir = root.join("retained-custody");
    let wallet = PersonaId::from_uuid(Uuid::from_u128(0x5336_77));
    let profile = Profile::new(
        personae::ProfileId("retained".into()),
        "Retained fixture",
        personae::Ed25519Keypair::from_seed([0x52; 32]),
    );
    let fresh = !dir.exists();
    let storage = SealedProfileStorage::open_with_key(dir.join("vault"), [0x63; 32]);
    if fresh {
        castellan::custody::wallet::ensure_wallet_state(&dir, wallet, "Retained fixture")
            .unwrap();
        storage.enroll_passphrase(VAULT_PASSPHRASE).unwrap();
        storage.save_profile(&profile).unwrap();
    }
    let host = Arc::new(castellan::authority::PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        dramatis::view::VaultProtectionView::OsProtected,
    ));
    let sealing = crate::retained_custody::RetainedSealing::custodial(
        host.clone(),
        dir,
        wallet,
        personae::ProfileId("retained".into()),
    );
    (host, sealing)
}

/// A reservoir with its route and the mere routes over one shared door,
/// and the mere for `domain` ensured through the reservoir route.
pub struct Fixture {
    pub(super) retained_host: Option<Arc<RetainedHost>>,
    pub(super) _root: tempfile::TempDir,
    pub reservoir: ResidentReservoir,
    pub catalog: AppEndpointCatalog,
    pub grants: AppRouteGrants,
    pub route: String,
    pub routes: MereRoutes,
}

/// The stand-in authority for a fixture domain: refuses a candidate carrying a
/// facet version no grammar here knows (`v999`).
pub(super) fn standin_authority() -> pandect::GraphValidator {
    Arc::new(|graph| {
        if serde_json::to_string(graph.facets()).is_ok_and(|facets| facets.contains("v999")) {
            Err("unsupported domain facet".into())
        } else {
            Ok(())
        }
    })
}

pub async fn fixture(persona_n: u128, domain: &str) -> Fixture {
    fixture_with_validator(persona_n, domain, None).await
}

pub(super) async fn fixture_with_validator(
    persona_n: u128,
    domain: &str,
    validator: Option<pandect::GraphValidator>,
) -> Fixture {
    fixture_serving(persona_n, domain, validator, None).await
}

/// The fixture, with its routes also registered in a dialable host's
/// catalog (`crate::resident_projection`).
pub(super) async fn fixture_serving(
    persona_n: u128,
    domain: &str,
    validator: Option<pandect::GraphValidator>,
    remote: Option<graphshell_host::projection_host::SharedCatalog>,
) -> Fixture {
    fixture_serving_handlers(
        persona_n,
        domain,
        validator,
        remote,
        HandlerRegistry::default(),
    )
    .await
}

pub(super) async fn fixture_serving_handlers(
    persona_n: u128,
    domain: &str,
    validator: Option<pandect::GraphValidator>,
    remote: Option<graphshell_host::projection_host::SharedCatalog>,
    handlers: HandlerRegistry,
) -> Fixture {
    fixture_inner(persona_n, domain, validator, remote, handlers, None, None).await
}

pub(super) async fn fixture_inner(
    persona_n: u128,
    domain: &str,
    validator: Option<pandect::GraphValidator>,
    remote: Option<graphshell_host::projection_host::SharedCatalog>,
    handlers: HandlerRegistry,
    custody: Option<crate::retained_custody::RetainedSealing>,
    reopen: Option<tempfile::TempDir>,
) -> Fixture {
    let root = reopen.unwrap_or_else(|| tempfile::tempdir().unwrap());
    let reservoir = ResidentReservoir::open(root.path(), persona(persona_n))
        .await
        .unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    crate::applications::install(
        &grants,
        vec![
            crate::applications::grant_for(&AppId::new("turnstone"), &crate::resident_reservoir::mere_client_declaration(), None),
            crate::applications::grant_for(&AppId::new("knot-editor"), &crate::resident_reservoir::mere_client_declaration(), None),
            // Graphshell alone holds the co-op routes.
            crate::applications::grant_for(&AppId::new("graphshell"), &mere_client(), None),
        ],
    );
    let routes = MereRoutes::new(
        root.path().to_path_buf(),
        persona(persona_n),
        catalog.clone(),
        grants.clone(),
    )
    .with_handlers(handlers);
    let storage = castellan::custody::InMemoryStorage::new();
    let profile = castellan::custody::Profile::new(
        personae::ProfileId("fixture-profile".into()),
        "Public fixture profile",
        personae::Ed25519Keypair::from_seed([0x33; 32]),
    );
    castellan::custody::IdentityStorage::save_profile(&storage, &profile).unwrap();
    let owner = Arc::new(castellan::authority::PersonaeHost::new(
        castellan::custody::IdentityVault::with_profile(storage, profile),
        None,
        dramatis::view::VaultProtectionView::Ephemeral,
    ));
    let routes = routes
        .with_custody_endpoint("unused-in-unit-tests".into())
        .with_profile_subject_authority(owner);
    // Mere links no application's authority (cycle breaks P3). A fixture that names no
    // validator gets Mere's defaults and, for its domain, a stand-in that refuses a
    // facet it does not know, so the tests of the route's own behaviour keep a gate.
    let routes = if let Some(validator) = validator {
        routes.with_domain_validator(domain, validator)
    } else if domain == "divination" {
        routes
            .with_resident_defaults()
            .unwrap()
            .with_domain_validator(domain, standin_authority())
    } else {
        routes.with_resident_defaults().unwrap()
    };
    let routes = match remote {
        Some(remote) => routes.with_remote_catalog(remote),
        None => routes,
    };
    let (retained_host, routes) = match custody {
        Some(custody) => (None, routes.with_retained_sealing(custody)),
        None => {
            let (host, sealing) = retained_custody(root.path());
            (Some(host), routes.with_retained_sealing(sealing))
        },
    };
    catalog
        .update(|catalog| reservoir.register(catalog, Some(routes.clone())))
        .await
        .unwrap();
    let mut door = catalog
        .update(|catalog| catalog.open(RESIDENT_RESERVOIR_ROUTE, &context("turnstone")))
        .await
        .unwrap();
    let request = door.describe().projections.remove(0).request;
    let listed = door.snapshot(request).unwrap();
    let ensure = IntentInvocation {
        session: listed.session.clone(),
        target: InstanceId(0),
        observed_epoch: listed.scene.epoch,
        observed_revision: listed.scene.revision,
        intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
        payload: serde_json::to_vec(&EnsureMereV1::new(domain)).unwrap(),
    };
    assert_eq!(door.invoke(ensure).unwrap(), IntentResult::Accepted);
    Fixture {
        retained_host,
        _root: root,
        reservoir,
        catalog,
        grants,
        route: mere_route_id(domain),
        routes,
    }
}

impl Fixture {
    /// The scratch directory the fixture's data root lives in.
    pub fn root(&self) -> &std::path::Path {
        self._root.path()
    }

    pub async fn attach(&self, app: &str) -> ResidentEndpointSession {
        self.catalog
            .update(|catalog| catalog.open(&self.route, &context(app)))
            .await
            .unwrap()
    }
}
