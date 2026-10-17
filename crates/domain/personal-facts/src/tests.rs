use super::*;
use kernel::graph::capture::replay_captured_deltas;
use serde_json::json;
use std::sync::{Arc, Mutex};
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
#[test]
fn corrections_keep_prior_revision_and_its_subject_association() {
    let mut g = Graph::new();
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
    let mut g = Graph::new();
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
    let mut missing = Graph::new();
    assert!(save_birth(&mut missing, &facts(subject)).is_err());
    assert_eq!(missing.nodes().count(), 0);
}
#[test]
fn association_subject_revision_or_dangling_target_is_refused_without_mutation() {
    let mut g = Graph::new();
    let record = save_birth(&mut g, &facts(profile())).unwrap();
    let before = serde_json::to_value(g.facets()).unwrap();
    let n = g.nodes().count();
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
        assert_eq!(serde_json::to_value(g.facets()).unwrap(), before);
        assert_eq!(g.nodes().count(), n);
    }
}
#[test]
fn journal_replay_preserves_exact_inputs_and_links() {
    let mut g = Graph::new();
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
    let deltas: Vec<kernel::graph::CapturedDelta> = serde_json::from_slice(&bytes).unwrap();
    let replayed = replay_captured_deltas(deltas);
    validate_for_persona(&replayed, "persona-a").unwrap();
    assert_eq!(births(&replayed).unwrap(), vec![expected]);
    assert_eq!(contacts(&replayed).unwrap(), contacts(&g).unwrap());
    assert_eq!(associations(&replayed).unwrap(), associations(&g).unwrap());
}
#[test]
fn tampering_cannot_reuse_an_existing_birth_revision() {
    let mut g = Graph::new();
    let record = save_birth(&mut g, &facts(profile())).unwrap();
    let key = g.get_node_by_url(&record.resource.id).unwrap().0;
    let mut changed = record.facts;
    changed.date.day = 2;
    apply_graph_delta(
        &mut g,
        GraphDelta::SetNodeFacet {
            key,
            facet: BIRTH_FACET.into(),
            value: serde_json::to_value(changed).unwrap(),
        },
    );
    assert!(validate(&g).is_err());
}
#[test]
fn external_resource_links_are_explicit_and_can_concern_multiple_subjects() {
    let mut g = Graph::new();
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
    let mut g = Graph::new();
    let mut original = facts(profile());
    original.time = dramatis::birth::BirthTime::Unknown;
    let record = save_birth(&mut g, &original).unwrap();
    assert_eq!(
        births(&g).unwrap()[0].facts.time,
        dramatis::birth::BirthTime::Unknown
    );
    let key = g.get_node_by_url(&record.resource.id).unwrap().0;
    apply_graph_delta(
        &mut g,
        GraphDelta::SetNodeFacet {
            key,
            facet: "dramatis.birth-facts/v999".into(),
            value: json!({}),
        },
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
    let mut graph = Graph::new();
    let id = LocalId::from_random([7; 16]);
    let local = create_contact(&mut graph, "persona-a", id, "Local subject").unwrap();
    let edits = Arc::new(Mutex::new(Vec::new()));
    let captured = edits.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        captured.lock().unwrap().push(delta.clone());
    })));
    let before_facets = serde_json::to_value(graph.facets()).unwrap();
    let before_nodes = graph.nodes().count();
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
        assert_eq!(serde_json::to_value(graph.facets()).unwrap(), before_facets);
        assert_eq!(graph.nodes().count(), before_nodes);
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
    let mut graph = Graph::new();
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
    let deltas: Vec<kernel::graph::CapturedDelta> = serde_json::from_slice(&bytes).unwrap();
    let replayed = replay_captured_deltas(deltas);
    validate_for_persona(&replayed, "persona-b").unwrap();
    assert!(validate_for_persona(&replayed, "persona-a").is_err());
    assert!(associations(&replayed).unwrap().contains(&association));
}

#[test]
fn newly_introduced_birth_profile_requires_positive_lookup_without_mutation() {
    let before = Graph::new();
    let mut after = before.clone();
    save_birth(&mut after, &facts(profile())).unwrap();
    let facets = serde_json::to_value(after.facets()).unwrap();
    let nodes = after.nodes().count();
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
    assert_eq!(serde_json::to_value(after.facets()).unwrap(), facets);
    assert_eq!(after.nodes().count(), nodes);
    assert_eq!(before.nodes().count(), 0);
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
    let before = Graph::new();
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
    let mut before = Graph::new();
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
    let mut valid = Graph::new();
    let birth = save_birth(&mut valid, &facts(profile())).unwrap();
    let mut invalid = valid.clone();
    let key = invalid.get_node_by_url(&birth.resource.id).unwrap().0;
    let mut changed = birth.facts;
    changed.date.day = 2;
    apply_graph_delta(
        &mut invalid,
        GraphDelta::SetNodeFacet {
            key,
            facet: BIRTH_FACET.into(),
            value: serde_json::to_value(changed).unwrap(),
        },
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
