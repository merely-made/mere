// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use mere::model::{Author, JournalDelta, ResourceDelta};
use mere_test_surface::{Delta, Surface, Uuid};
use serde_json::json;
use std::sync::{Arc, Mutex};

/// The shared test graph (engine split D272).
type Graph = Surface;

fn facts(subject: SubjectRef) -> BirthFacts {
    serde_json::from_value(json!({"schema":"dramatis.birth-facts/v1","subject":subject,"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":{"label":"Public synthetic birthplace","latitude_microdegrees":0,"longitude_microdegrees":0,"precision":"synthetic point"},"timezone":"UTC","source":{"label":"Public synthetic receipt","reference":null}})).unwrap()
}
fn profile() -> SubjectRef {
    SubjectRef::Profile {
        id: personae_id("receipt"),
    }
}
fn personae_id(value: &str) -> personae::ProfileId {
    personae::ProfileId(value.into())
}
/// Every resource record, for comparing a graph before and after a refusal.
fn records(g: &Graph) -> serde_json::Value {
    let resources = g.resources();
    let all: Vec<_> = resources
        .resource_nodes()
        .filter_map(|r| resources.resource_record(r.id()))
        .collect();
    serde_json::to_value(all).unwrap()
}
/// Overwrites one facet of the resource at `iri`, bypassing the checks.
fn overwrite(g: &mut Graph, iri: &str, name: &str, value: serde_json::Value) {
    let resource = ResourceNode::for_term(iri);
    let mut record = g
        .resources()
        .resource_record(resource.id())
        .unwrap_or(PersistedResourceRecord {
            canonical_iri: iri.into(),
            facets: Vec::new(),
        });
    record.facets.retain(|f| f.facet != name);
    record.facets.push(PersistedResourceFacet {
        facet: name.into(),
        value_json: value.to_string(),
    });
    let write = JournalDelta::Resource(ResourceDelta::ReplaySetResourceRecordById {
        resource_id: resource.id().to_string(),
        record: Some(record),
    });
    SessionGraph::apply(g, write).unwrap();
}
/// A surface showing the resource at `url`, as a host shows a record.
fn node_at(g: &mut Graph, url: &str) -> Uuid {
    let surface = ResourceNode::for_term(url).id();
    g.show(surface, url);
    surface
}
/// The graph the recorded `bytes` rebuild, each edit applied as it was made.
fn replay_captured_deltas(bytes: &[u8]) -> Graph {
    let deltas: Vec<Delta> = serde_json::from_slice(bytes).unwrap();
    let mut graph = Graph::default();
    for delta in &deltas {
        let edit = Graph::edit_of(delta, &Author::user()).unwrap();
        SessionGraph::apply(&mut graph, edit).unwrap();
    }
    graph
}
#[test]
fn corrections_keep_prior_revision_and_its_subject_association() {
    let mut g = Graph::default();
    let original = facts(profile());
    let first = save_birth(&mut g, &original).unwrap();
    let mut changed = original.clone();
    changed.time = dramatis::birth::BirthTime::Known {
        hour: 13,
        minute: 0,
        second: 0,
    };
    let second = save_birth(&mut g, &changed).unwrap();
    assert_ne!(first.resource, second.resource);
    assert_eq!(births(&g).unwrap().len(), 2);
    assert!(births(&g).unwrap().contains(&first));
    assert_eq!(associations(&g).unwrap().len(), 2);
    save_birth(&mut g, &original).unwrap();
    assert_eq!(births(&g).unwrap().len(), 2);
}
#[test]
fn a_keyless_subject_is_shared_and_persona_scoped() {
    let mut g = Graph::default();
    let subject = create_contact(
        &mut g,
        "persona-a",
        LocalId::from_random([1; 16]),
        "Someone else",
    )
    .unwrap();
    save_birth(&mut g, &facts(subject.clone())).unwrap();
    assert_eq!(contacts(&g).unwrap()[0].subject, subject);
    validate_for_persona(&g, "persona-a").unwrap();
    assert!(validate_for_persona(&g, "persona-b").is_err());
    let mut missing = Graph::default();
    assert!(save_birth(&mut missing, &facts(subject)).is_err());
    assert_eq!(missing.resources().resource_nodes().count(), 0);
}
#[test]
fn association_subject_revision_or_dangling_target_is_refused_without_mutation() {
    let mut g = Graph::default();
    let record = save_birth(&mut g, &facts(profile())).unwrap();
    let before = records(&g);
    for bad in [
        SubjectResourceAssociation::about(
            SubjectRef::Profile {
                id: personae_id("other"),
            },
            record.resource.clone(),
        ),
        SubjectResourceAssociation::about(
            profile(),
            ResourceRef {
                revision: "wrong".into(),
                ..record.resource.clone()
            },
        ),
        SubjectResourceAssociation::about(
            profile(),
            ResourceRef {
                domain: DOMAIN.into(),
                id: "missing".into(),
                revision: "x".into(),
            },
        ),
    ] {
        assert!(save_association(&mut g, &bad).is_err());
        assert_eq!(records(&g), before);
    }
}
#[test]
fn journal_replay_preserves_exact_inputs_and_links() {
    let mut g = Graph::default();
    let edits = Arc::new(Mutex::new(Vec::new()));
    let held = edits.clone();
    g.set_recorder(Some(Arc::new(move |d| {
        held.lock().unwrap().push(d.clone())
    })));
    let subject = create_contact(
        &mut g,
        "persona-a",
        LocalId::from_random([2; 16]),
        "Replay person",
    )
    .unwrap();
    let expected = save_birth(&mut g, &facts(subject)).unwrap();
    let bytes = serde_json::to_vec(&*edits.lock().unwrap()).unwrap();
    let replayed = replay_captured_deltas(&bytes);
    validate_for_persona(&replayed, "persona-a").unwrap();
    assert_eq!(births(&replayed).unwrap(), vec![expected]);
    assert_eq!(contacts(&replayed).unwrap(), contacts(&g).unwrap());
    assert_eq!(associations(&replayed).unwrap(), associations(&g).unwrap());
}
#[test]
fn tampering_cannot_reuse_an_existing_birth_revision() {
    let mut g = Graph::default();
    let record = save_birth(&mut g, &facts(profile())).unwrap();
    // Control: the same overwrite with the saved facts stays valid.
    overwrite(
        &mut g,
        &record.resource.id,
        BIRTH_FACET,
        serde_json::to_value(&record.facts).unwrap(),
    );
    validate(&g).unwrap();
    let mut changed = record.facts;
    changed.date.day = 2;
    overwrite(
        &mut g,
        &record.resource.id,
        BIRTH_FACET,
        serde_json::to_value(changed).unwrap(),
    );
    assert!(validate(&g).is_err());
}
#[test]
fn external_resource_links_are_explicit_and_can_concern_multiple_subjects() {
    let mut g = Graph::default();
    let target = ResourceRef {
        domain: "address-owner".into(),
        id: "household/address".into(),
        revision: "exact-v1".into(),
    };
    for who in ["first", "second"] {
        save_association(
            &mut g,
            &SubjectResourceAssociation::about(
                SubjectRef::Profile {
                    id: personae_id(who),
                },
                target.clone(),
            ),
        )
        .unwrap();
    }
    assert_eq!(associations(&g).unwrap().len(), 2);
    assert!(births(&g).unwrap().is_empty());
}
#[test]
fn unknown_time_remains_unknown_and_unsupported_facets_refuse() {
    let mut g = Graph::default();
    let mut original = facts(profile());
    original.time = dramatis::birth::BirthTime::Unknown;
    let record = save_birth(&mut g, &original).unwrap();
    assert_eq!(
        births(&g).unwrap()[0].facts.time,
        dramatis::birth::BirthTime::Unknown
    );
    overwrite(
        &mut g,
        &record.resource.id,
        "dramatis.birth-facts/v999",
        json!({}),
    );
    assert!(validate(&g).is_err());
}

fn external_target() -> ResourceRef {
    ResourceRef {
        domain: "divination".into(),
        id: "cleromancy://natal/synthetic".into(),
        revision: "synthetic-revision".into(),
    }
}

#[test]
fn external_contact_associations_refuse_foreign_or_dangling_subject_without_mutation() {
    let mut graph = Graph::default();
    let id = LocalId::from_random([7; 16]);
    let local = create_contact(&mut graph, "persona-a", id, "Local subject").unwrap();
    let edits = Arc::new(Mutex::new(Vec::new()));
    let captured = edits.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        captured.lock().unwrap().push(delta.clone());
    })));
    let before = records(&graph);
    for invalid in [
        SubjectRef::Contact {
            scope: PersonaScope::new("persona-b"),
            anchor: gaz::Anchor::Local(id),
        },
        SubjectRef::Contact {
            scope: PersonaScope::new("persona-a"),
            anchor: gaz::Anchor::Local(LocalId::from_random([8; 16])),
        },
    ] {
        let association = SubjectResourceAssociation::about(invalid, external_target());
        assert!(save_association(&mut graph, &association).is_err());
        assert_eq!(records(&graph), before);
        assert!(
            edits.lock().unwrap().is_empty(),
            "refusal must emit no edits"
        );

        // Direct graph candidates must not bypass the public save check.
        let mut forged = graph.clone();
        forged.set_recorder(None);
        put_association(&mut forged, &association).unwrap();
        assert!(validate(&forged).is_err());
        assert!(validate_for_persona(&forged, "persona-a").is_err());
    }
    save_association(
        &mut graph,
        &SubjectResourceAssociation::about(local, external_target()),
    )
    .unwrap();
    validate_for_persona(&graph, "persona-a").unwrap();
}

#[test]
fn external_association_to_retained_foreign_contact_still_requires_owner_scope_on_replay() {
    let mut graph = Graph::default();
    let edits = Arc::new(Mutex::new(Vec::new()));
    let captured = edits.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        captured.lock().unwrap().push(delta.clone());
    })));
    let subject = create_contact(
        &mut graph,
        "persona-b",
        LocalId::from_random([9; 16]),
        "Retained subject",
    )
    .unwrap();
    let association = SubjectResourceAssociation::about(subject, external_target());
    save_association(&mut graph, &association).unwrap();
    validate(&graph).unwrap();
    assert!(validate_for_persona(&graph, "persona-a").is_err());

    let bytes = serde_json::to_vec(&*edits.lock().unwrap()).unwrap();
    let replayed = replay_captured_deltas(&bytes);
    validate_for_persona(&replayed, "persona-b").unwrap();
    assert!(validate_for_persona(&replayed, "persona-a").is_err());
    assert!(associations(&replayed).unwrap().contains(&association));
}

#[test]
fn newly_introduced_birth_profile_requires_positive_lookup_without_mutation() {
    let before = Graph::default();
    let mut after = before.clone();
    save_birth(&mut after, &facts(profile())).unwrap();
    let before_records = records(&after);
    let edits = Arc::new(Mutex::new(Vec::new()));
    let captured = edits.clone();
    after.set_recorder(Some(Arc::new(move |delta| {
        captured.lock().unwrap().push(delta.clone());
    })));
    assert!(validate_new_profile_subjects(&before, &after, |_| Ok(false)).is_err());
    assert_eq!(
        validate_new_profile_subjects(&before, &after, |_| Err("lookup denied".into())),
        Err("lookup denied".into())
    );
    assert_eq!(records(&after), before_records);
    assert_eq!(before.resources().resource_nodes().count(), 0);
    assert!(edits.lock().unwrap().is_empty());

    let calls = std::cell::RefCell::new(Vec::new());
    validate_new_profile_subjects(&before, &after, |id| {
        calls.borrow_mut().push(id.clone());
        Ok(id == &personae_id("receipt"))
    })
    .unwrap();
    assert_eq!(
        *calls.borrow(),
        vec![personae_id("receipt")],
        "birth and association share one lookup"
    );
}

#[test]
fn association_only_profile_introduction_also_requires_lookup() {
    let before = Graph::default();
    let mut after = before.clone();
    save_association(
        &mut after,
        &SubjectResourceAssociation::about(profile(), external_target()),
    )
    .unwrap();
    assert!(births(&after).unwrap().is_empty());
    assert!(validate_new_profile_subjects(&before, &after, |_| Ok(false)).is_err());
    validate_new_profile_subjects(&before, &after, |id| Ok(id == &personae_id("receipt"))).unwrap();
}

#[test]
fn historical_profiles_survive_unavailable_lookup_and_unrelated_edits() {
    let mut before = Graph::default();
    let original = facts(profile());
    save_birth(&mut before, &original).unwrap();
    let mut after = before.clone();
    create_contact(
        &mut after,
        "persona-a",
        LocalId::from_random([10; 16]),
        "Unrelated contact",
    )
    .unwrap();
    let mut corrected = original;
    corrected.date.day = 2;
    save_birth(&mut after, &corrected).unwrap();
    save_association(
        &mut after,
        &SubjectResourceAssociation::about(profile(), external_target()),
    )
    .unwrap();
    validate_new_profile_subjects(&before, &after, |_| {
        panic!("historical references need no live lookup")
    })
    .unwrap();
    validate_new_profile_subjects(&after, &after, |_| Err("lookup unavailable".into())).unwrap();

    let mut new_facts = corrected;
    new_facts.subject = SubjectRef::Profile {
        id: personae_id("new-profile"),
    };
    save_birth(&mut after, &new_facts).unwrap();
    let calls = std::cell::RefCell::new(Vec::new());
    validate_new_profile_subjects(&before, &after, |id| {
        calls.borrow_mut().push(id.clone());
        Ok(id == &personae_id("new-profile"))
    })
    .unwrap();
    assert_eq!(*calls.borrow(), vec![personae_id("new-profile")]);
}

#[test]
fn invalid_before_or_after_shape_refuses_before_any_profile_lookup() {
    let mut valid = Graph::default();
    let birth = save_birth(&mut valid, &facts(profile())).unwrap();
    let mut invalid = valid.clone();
    let mut changed = birth.facts;
    changed.date.day = 2;
    overwrite(
        &mut invalid,
        &birth.resource.id,
        BIRTH_FACET,
        serde_json::to_value(changed).unwrap(),
    );
    for (before, after) in [(&invalid, &valid), (&valid, &invalid)] {
        assert!(
            validate_new_profile_subjects(before, after, |_| panic!(
                "validate graphs before lookup"
            ))
            .is_err()
        );
    }
}

#[test]
fn records_are_resource_facets_and_place_no_node() {
    let mut g = Graph::default();
    let edits = Arc::new(Mutex::new(Vec::new()));
    let held = edits.clone();
    g.set_recorder(Some(Arc::new(move |d| {
        held.lock().unwrap().push(d.clone())
    })));
    let record = save_birth(&mut g, &facts(profile())).unwrap();
    assert_eq!(g.surface_count(), 0, "placing a node is the host's job");
    let resource = ResourceNode::for_term(&record.resource.id);
    let stored = g.resources().resource_record(resource.id()).unwrap();
    assert_eq!(stored.canonical_iri, record.resource.id);
    assert_eq!(stored.facets.len(), 1);
    assert_eq!(stored.facets[0].facet, BIRTH_FACET);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored.facets[0].value_json).unwrap(),
        serde_json::to_value(&record.facts).unwrap()
    );
    let edits = edits.lock().unwrap();
    assert!(!edits.is_empty());
    assert!(
        edits
            .iter()
            .all(|d| matches!(
                d,
                JournalDelta::Resource(ResourceDelta::ReplaySetResourceRecordById { .. })
            )),
        "only resource-record writes: {edits:?}"
    );
}

#[test]
fn a_node_facet_record_is_refused_with_no_legacy_reader() {
    // The form before D216: a node at the record's address holding the facet.
    let facts = facts(profile());
    let birth = birth_resource(&facts).unwrap();
    let mut legacy = Graph::default();
    let surface = node_at(&mut legacy, &birth.resource.id);
    legacy.set_facet(surface, BIRTH_FACET, serde_json::to_value(&facts).unwrap());
    assert!(validate(&legacy).is_err());
    assert!(births(&legacy).is_err());
    // Control: a host may still show a record with a bare node at its address.
    let mut shown = Graph::default();
    let record = save_birth(&mut shown, &facts).unwrap();
    node_at(&mut shown, &record.resource.id);
    validate(&shown).unwrap();
    assert_eq!(births(&shown).unwrap(), vec![record]);
}
