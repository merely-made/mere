// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The owner's review and pin of an authority guest (cycle breaks P3, D15 (h)), over the
//! fixture guest `authority-host` builds: what review refuses, what approval binds, what a
//! loaded guest decides, and every way a guest goes absent (which refuses as pending).

use eidetic::pack::{PackManifest, PackPart, PackPartRole, sign_pack};
use eidetic::{ManifestId, TrustEnvelope};
use mere_resident::authority_guest::{
    AuthorityReview, GuestStore, POWER, Pin, PinAgreement, ReviewError, component_hash,
};
use mere_resident::domain_registry::{Authority, DomainRegistry, PENDING};
use ostiary::app_admission::AppId;
use personae::{IdentityProvider, InMemoryProvider, PersonaId};
use uuid::Uuid;

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x7033_6775_6573))
}

fn component() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/script/authority-host/guest/target/wasm32-wasip2/release/authority_fixture_guest.wasm"
    );
    std::fs::read(path).unwrap_or_else(|error| {
        panic!("fixture guest missing ({error}); build authority-host (its build.rs builds the guest)")
    })
}

/// A signed pack for `component`, by the author whose seed is `seed`, asking for `scopes`.
fn pack(seed: u8, component: &[u8], scopes: &[&str]) -> (Vec<u8>, String) {
    let author = InMemoryProvider::from_seed([seed; 32]);
    let manifest = PackManifest {
        name: "fixture-authority".into(),
        version: "1".into(),
        author: author.master_public_key().to_bytes().iter().map(|b| format!("{b:02x}")).collect(),
        requested_scopes: scopes.iter().map(|s| s.to_string()).collect(),
        parts: vec![PackPart {
            name: "fixture.wasm".into(),
            role: PackPartRole::WasmComponent,
            blob: ManifestId::of_blob(component),
            bytes: component.len() as u64,
        }],
    };
    let mut trust = TrustEnvelope::self_asserted();
    trust.signatures.push(sign_pack(&manifest, author.master_keypair()));
    let bytes = serde_json::to_vec(&serde_json::json!({ "manifest": manifest, "trust": trust })).unwrap();
    let hash = ManifestId::of_blob(&bytes).0.to_hex();
    (bytes, hash)
}

fn review(seed: u8) -> AuthorityReview {
    let component = component();
    let (bytes, hash) = pack(seed, &component, &[POWER]);
    AuthorityReview::prepare(&bytes, &hash, &component).unwrap()
}

fn graph_titled(title: &str) -> kernel::graph::Graph {
    let mut graph = kernel::graph::Graph::new();
    let key = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.test/".into(),
        kernel::geometry::PortablePoint::new(0.0, 0.0),
    );
    kernel::graph::apply::apply_graph_delta(
        &mut graph,
        kernel::graph::apply::GraphDelta::SetNodeTitle { key, title: title.into() },
    );
    graph
}

fn approved() -> (tempfile::TempDir, GuestStore, Pin) {
    let root = tempfile::tempdir().unwrap();
    let store = GuestStore::at(root.path(), persona());
    let pin = review(1).approve(&store, AppId::new("fixture-app"), false).unwrap();
    (root, store, pin)
}

fn only(store: &GuestStore) -> mere_resident::domain_registry::DomainRegistration {
    let mut registrations = store.registrations();
    assert_eq!(registrations.len(), 1);
    registrations.remove(0)
}

#[test]
fn review_names_what_the_owner_decides_and_grants_nothing() {
    let review = review(1);
    assert_eq!(review.domain, "fixture");
    assert_eq!(review.version, "v1");
    assert_eq!(review.graph_format, authority_host::GRAPH_SNAPSHOT_AND_FACETS);
    assert!(review.admission);
    assert_eq!(review.author.len(), 64);
    assert_eq!(review.component, component_hash(&component()));
    let root = tempfile::tempdir().unwrap();
    let store = GuestStore::at(root.path(), persona());
    assert!(store.pins().is_empty() && store.registrations().is_empty(), "review installs nothing");
}

#[test]
fn review_refuses_a_pack_that_does_not_verify_or_asks_for_more() {
    let component = component();
    let (bytes, hash) = pack(1, &component, &[POWER]);
    assert!(matches!(
        AuthorityReview::prepare(&bytes, "00", &component),
        Err(ReviewError::Pack(_))
    ));
    let mut tampered = bytes.clone();
    let at = tampered.windows(10).position(|w| w == b"fixture-au").unwrap();
    tampered[at] = b'F';
    let tampered_hash = ManifestId::of_blob(&tampered).0.to_hex();
    assert!(matches!(
        AuthorityReview::prepare(&tampered, &tampered_hash, &component),
        Err(ReviewError::Pack(_))
    ), "a manifest changed after signing");
    assert!(matches!(
        AuthorityReview::prepare(&bytes, &hash, b"another component"),
        Err(ReviewError::Pack(_))
    ));
    for scopes in [vec![], vec!["power:navigate"], vec![POWER, "power:navigate"]] {
        let (bytes, hash) = pack(1, &component, &scopes);
        assert!(matches!(
            AuthorityReview::prepare(&bytes, &hash, &component),
            Err(ReviewError::Scope(_))
        ), "{scopes:?}");
    }
}

#[test]
fn an_approved_guest_is_registered_as_a_guest_authority_and_decides_candidates() {
    let (_root, store, pin) = approved();
    assert_eq!(pin.domain, "fixture");
    let registration = only(&store);
    assert_eq!(registration.domain, "fixture");
    assert_eq!(registration.owner.as_str(), "fixture-app");
    assert_eq!(registration.authority.kind(), "guest");
    let Authority::Guest(guest) = &registration.authority else { unreachable!() };
    let runtime = guest.runtime().expect("loaded");
    assert!(runtime.validate(&graph_titled("Allowed")).is_ok());
    assert_eq!(
        runtime.validate(&graph_titled("Forbidden")).unwrap_err(),
        "fixture forbids the complete candidate"
    );
    assert!(runtime.admit(&graph_titled("a"), &graph_titled("b")).is_ok());
    assert_eq!(
        runtime.admit(&graph_titled("a"), &graph_titled("Forbidden")).unwrap_err(),
        "fixture forbids the transition"
    );
    let stats = runtime.stats();
    assert_eq!((stats.calls, stats.refused, stats.trapped), (4, 2, 0));
    assert!(stats.max_fuel > 0 && stats.encode > std::time::Duration::ZERO);
    // And the registry takes it, beside Mere's own domains.
    let registry = DomainRegistry::resident_defaults(persona(), &store).unwrap();
    assert_eq!(registry.domains(), ["fixture", "personal-facts"]);
}

#[test]
fn the_domain_is_bound_to_the_first_authors_key() {
    let (_root, store, pin) = approved();
    let other = review(2);
    let offered = other.author.clone();
    assert_eq!(
        other.approve(&store, AppId::new("fixture-app"), false).unwrap_err(),
        ReviewError::AuthorMismatch { domain: "fixture".into(), bound: pin.author.clone(), offered },
    );
    assert_eq!(store.pin("fixture").unwrap(), pin, "a refused review changes nothing");
    // The same key may bring a new review; the owner may rebind the domain on purpose.
    review(1).approve(&store, AppId::new("fixture-app"), false).unwrap();
    let rebound = review(2).approve(&store, AppId::new("fixture-app"), true).unwrap();
    assert_ne!(rebound.author, pin.author);
    assert_eq!(rebound.compare(&pin), PinAgreement::AuthorDiffers);
    let mut moved = pin.clone();
    moved.component = "00".repeat(32);
    assert_eq!(pin.compare(&moved), PinAgreement::ComponentDiffers);
    assert_eq!(pin.compare(&pin.clone()), PinAgreement::Agree);
}

fn refusal_of(registration: &mere_resident::domain_registry::DomainRegistration) -> String {
    let Authority::Guest(guest) = &registration.authority else { panic!("a guest authority") };
    assert!(guest.runtime().is_none());
    registration.validate(&kernel::graph::Graph::new()).unwrap_err()
}

#[test]
fn a_guest_that_goes_absent_refuses_every_candidate_as_pending() {
    // Revoked by the owner.
    let (_root, store, _) = approved();
    store.revoke("fixture").unwrap();
    assert!(store.pin("fixture").unwrap().revoked);
    let text = refusal_of(&only(&store));
    assert!(text.starts_with(&format!("{PENDING} for fixture")) && text.contains("revoked"), "{text}");

    // Component file gone.
    let (root, store, _) = approved();
    std::fs::remove_file(root.path().join("authority-guests").join(persona().as_uuid().to_string()).join("fixture/component.wasm")).unwrap();
    let text = refusal_of(&only(&store));
    assert!(text.starts_with(&format!("{PENDING} for fixture")), "{text}");

    // Component altered on disk: it no longer matches the pin (nor the signed pack).
    let (root, store, _) = approved();
    let path = root.path().join("authority-guests").join(persona().as_uuid().to_string()).join("fixture/component.wasm");
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    std::fs::write(&path, bytes).unwrap();
    let text = refusal_of(&only(&store));
    assert!(text.starts_with(&format!("{PENDING} for fixture")), "{text}");

    // Pack swapped for another author's: the key bound to the domain does not match.
    let (root, store, _) = approved();
    let swapped = component();
    let (other_pack, _) = pack(9, &swapped, &[POWER]);
    std::fs::write(
        root.path().join("authority-guests").join(persona().as_uuid().to_string()).join("fixture/pack.json"),
        other_pack,
    )
    .unwrap();
    let text = refusal_of(&only(&store));
    assert!(text.starts_with(&format!("{PENDING} for fixture")) && text.contains("bound to author"), "{text}");
}

#[test]
fn a_guest_cannot_take_a_domain_mere_registers_itself() {
    // The fixture guest claims `fixture`; a pin forged for `personal-facts` collides.
    let (root, store, pin) = approved();
    let base = root.path().join("authority-guests").join(persona().as_uuid().to_string());
    std::fs::rename(base.join("fixture"), base.join("personal-facts")).unwrap();
    let mut forged = pin;
    forged.domain = "personal-facts".into();
    std::fs::write(base.join("personal-facts/pin.json"), serde_json::to_vec(&forged).unwrap()).unwrap();
    // It registers as pending (the claim names another domain)...
    let registration = only(&store);
    assert_eq!(registration.domain, "personal-facts");
    assert!(refusal_of(&registration).starts_with(&format!("{PENDING} for personal-facts")));
    // ...and the registry refuses the duplicate outright.
    assert!(DomainRegistry::resident_defaults(persona(), &store).is_err());
}
