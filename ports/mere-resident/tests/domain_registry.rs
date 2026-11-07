// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Conformance of the domain registry (cycle breaks P1b) over fixture
//! domains: what `MereRoutes::with_registry` installs, what `serve` refuses
//! and how, and the typed pending refusal of the authorities not yet built.

use std::sync::Arc;

use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use mere_resident::domain_registry::{
    Authority, DeclaredAuthority, DomainRegistration, DomainRegistry, GuestAuthority, PENDING,
    RegistryError, Unregistered,
};
use mere_resident::door::app_broker::AppEndpointCatalog;
use mere_resident::resident_mere::MereRoutes;
use mere_resident::resident_reservoir::ResidentReservoir;
use ostiary::app_admission::AppId;
use pandect::{DomainId, MereRecord, ReservoirStore, open_reservoir_backend};
use personae::PersonaId;
use uuid::Uuid;

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x7031_6272_6567))
}

fn permissive(domain: &str) -> DomainRegistration {
    DomainRegistration::new(
        domain,
        AppId::new("fixture"),
        "v1",
        Authority::graph(Arc::new(|_| Ok(()))),
    )
}

/// A routes value over a temporary reservoir with `domains` ensured.
async fn routes_with(
    domains: &[&str],
    registry: impl FnOnce(PersonaId) -> DomainRegistry,
) -> (tempfile::TempDir, MereRoutes, Vec<MereRecord>, AppEndpointCatalog) {
    let root = tempfile::tempdir().unwrap();
    let mut store = ReservoirStore::open(open_reservoir_backend(root.path(), persona()).unwrap(), persona())
        .await
        .unwrap();
    for domain in domains {
        store.ensure(DomainId::new(*domain).unwrap(), 1).await.unwrap();
    }
    drop(store);
    let reservoir = ResidentReservoir::open(root.path(), persona()).await.unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    let routes = MereRoutes::new(root.path().into(), persona(), catalog.clone(), grants)
        .with_registry(&registry(persona()))
        .unwrap();
    routes.load_access(&reservoir).await.unwrap();
    let meres = reservoir.meres().await;
    (root, routes, meres, catalog)
}

fn named<'a>(meres: &'a [MereRecord], domain: &str) -> &'a MereRecord {
    meres.iter().find(|mere| mere.domain.to_string() == domain).unwrap()
}

#[test]
fn a_duplicate_or_invalid_registration_is_refused() {
    let registry = DomainRegistry::new().register(permissive("fixture")).unwrap();
    assert_eq!(
        registry.clone().register(permissive("fixture")).err(),
        Some(RegistryError::Duplicate("fixture".into()))
    );
    assert!(matches!(
        registry.register(permissive("Not A Domain!")).err(),
        Some(RegistryError::InvalidDomain(_))
    ));
}

#[test]
fn mere_defaults_install_personal_facts_and_reserve_divination() {
    let defaults = DomainRegistry::mere_defaults(persona());
    assert_eq!(defaults.domains(), ["personal-facts"]);
    assert_eq!(defaults.get("personal-facts").unwrap().authority.kind(), "graph");
    // Divination is Cleromancy's: reserved, not registered, until P3.
    assert!(defaults.get("divination").is_none());
    let resident = DomainRegistry::resident_defaults(persona());
    assert_eq!(resident.domains(), ["divination", "personal-facts"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reserved_domain_without_a_registration_is_refused_with_the_typed_error() {
    let (_root, routes, meres, _catalog) = routes_with(&["divination"], DomainRegistry::mere_defaults).await;
    let refusal = routes.serve(named(&meres, "divination")).await.unwrap_err();
    assert_eq!(
        refusal,
        "domain-not-registered: divination (registered: personal-facts)"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn resident_defaults_serve_divination_and_personal_facts() {
    let (_root, routes, meres, _catalog) =
        routes_with(&["divination", "personal-facts"], DomainRegistry::resident_defaults).await;
    for mere in &meres {
        routes.serve(mere).await.unwrap();
    }
    let shown = routes.registered_domains();
    assert_eq!(
        shown.iter().map(|d| d.domain.as_str()).collect::<Vec<_>>(),
        ["divination", "personal-facts"]
    );
    assert_eq!(shown[0].owner.as_deref(), Some("cleromancy"));
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unregistered_domain_is_served_generically_until_strict() {
    let (_root, routes, meres, _catalog) = routes_with(&["notes"], DomainRegistry::mere_defaults).await;
    routes.serve(named(&meres, "notes")).await.unwrap();
    let strict = |persona| DomainRegistry::mere_defaults(persona).unregistered(Unregistered::Refuse);
    let (_root, routes, meres, _catalog) = routes_with(&["notes"], strict).await;
    let refusal = routes.serve(named(&meres, "notes")).await.unwrap_err();
    assert!(refusal.starts_with("domain-not-registered: notes"), "{refusal}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_domain_is_added_without_editing_the_composition() {
    let (_root, routes, meres, _catalog) = routes_with(&["fixture"], |persona| {
        DomainRegistry::mere_defaults(persona)
            .register(permissive("fixture"))
            .unwrap()
    })
    .await;
    routes.serve(named(&meres, "fixture")).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_registry_naming_an_installed_domain_is_refused() {
    let (_root, routes, _, _catalog) = routes_with(&[], DomainRegistry::mere_defaults).await;
    assert_eq!(
        routes
            .with_registry(&DomainRegistry::mere_defaults(persona()))
            .err(),
        Some(RegistryError::Duplicate("personal-facts".into()))
    );
}

#[test]
fn guest_and_declared_authorities_register_with_their_kind() {
    for authority in [
        Authority::Guest(GuestAuthority::new([7; 32])),
        Authority::Declared(DeclaredAuthority::new("fixture")),
    ] {
        let kind = authority.kind();
        let registration = DomainRegistration::new("fixture", AppId::new("fixture"), "v1", authority);
        let registry = DomainRegistry::new().register(registration).unwrap();
        assert_eq!(registry.get("fixture").unwrap().authority.kind(), kind);
    }
}
