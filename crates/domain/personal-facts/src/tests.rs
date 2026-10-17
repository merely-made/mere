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
