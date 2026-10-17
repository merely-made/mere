use scenomise::{
    backdrops::SceneInput,
    document::{LinkedScene, SceneDocument},
};
fn document() -> SceneDocument {
    let primary = SceneInput::parse(include_str!(
        "../../../../../support/design-studies/backdrops/templates/progression.json"
    ))
    .unwrap();
    let board = SceneInput::parse(include_str!(
        "../../../../../support/design-studies/backdrops/fretboard.json"
    ))
    .unwrap();
    SceneDocument::new(
        primary,
        vec![LinkedScene {
            occurrence: "bar-2".into(),
            label: "Supplied voicing".into(),
            scene: board,
        }],
    )
}
#[test]
fn linked_reading_is_selected_by_occurrence_and_round_trips() {
    let d = document();
    d.validate().unwrap();
    assert!(d.for_occurrence(Some("bar-0")).is_none());
    assert!(d.for_occurrence(None).is_none());
    assert_eq!(
        d.for_occurrence(Some("bar-2")).unwrap().label,
        "Supplied voicing"
    );
    assert_eq!(
        d,
        SceneDocument::parse(&serde_json::to_string(&d).unwrap()).unwrap()
    );
    let bare = SceneDocument::parse(&serde_json::to_string(&d.primary).unwrap()).unwrap();
    assert!(bare.linked.is_empty());
    assert_eq!(bare.primary, d.primary);
}
#[test]
fn invalid_linked_geometry_or_unknown_bindings_refuse_the_whole_document() {
    let mut d = document();
    d.linked[0].scene.version = 99;
    assert!(d.validate().is_err());
    let mut d = document();
    d.linked[0].occurrence = "unknown".into();
    assert!(d.validate().is_err());
    let mut d = document();
    d.linked.push(d.linked[0].clone());
    assert!(d.validate().is_err());
}
#[test]
fn nested_unknown_fields_and_over_budget_documents_are_refused() {
    let mut value = serde_json::to_value(document()).unwrap();
    value["linked"][0]["scene"]["source"]["credential"] = "not a source field".into();
    assert!(SceneDocument::parse(&value.to_string()).is_err());
    let mut d = document();
    d.linked[0].label = "x".repeat(1024 * 1024);
    assert!(d.validate().is_err());
}
