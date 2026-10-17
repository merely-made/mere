use graphshell::{
    access::AccessContext,
    mere_host::{MereHost, SelectedPersonaRef, fixture_handlers},
    product::EditableRelation,
    link_inspection::{LinkTarget, inspect_link},
    presentation_binding::{Occurrence, ViewInputs, bind},
};
use muniment::MemoryBackend;

fn family(relation: EditableRelation) -> (String, String) {
    let mut host = MereHost::empty(MemoryBackend::new(),
        SelectedPersonaRef { persona: "audit:person".into(), profile: "audit".into() }, fixture_handlers(),
        AccessContext { persona: "audit:person".into(), device: "audit:device".into(), at_ms: 1 });
    let a = host.create_address("https://family-a.test/", "A").unwrap();
    let b = host.create_address("https://family-b.test/", "B").unwrap();
    host.assert_product_relation(a, b, relation).unwrap();
    let pair = LinkTarget::new(a, b);
    let inspection = inspect_link(host.graph(), pair).unwrap();
    let held = inspection.records.iter().flat_map(|r| &r.statements).next().unwrap();
    let actual = mere::kernel::graph::RelationKind::from_predicate_iri(&held.predicate).unwrap().family();
    let bound = bind(host.graph(), &Occurrence::Assertion { pair, statement: held.statement_id.clone() }, &ViewInputs::default(), None).unwrap();
    let observed = serde_json::to_value(&bound.context.facts["family"]).unwrap();
    (format!("{actual:?}"), observed.as_str().unwrap().to_owned())
}

#[test]
fn containment_assertion_is_disclosed_to_rules_as_semantic() {
    assert_eq!(family(EditableRelation::CollectionMember), ("Containment".into(), "Semantic".into()));
}

#[test]
fn semantic_assertion_remains_correctly_disclosed_as_semantic() {
    assert_eq!(family(EditableRelation::Cites), ("Semantic".into(), "Semantic".into()));
}
