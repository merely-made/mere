use sceno::SourceRef;
use scenomise::{
    backdrops::{SceneFacts, SceneInput},
    template::*,
};
fn input() -> SceneInput {
    SceneInput {
        version: 1,
        source: SourceRef {
            adapter: "test.template".into(),
            id: "exercise".into(),
        },
        revision: "1".into(),
        title: "Expected and supplied".into(),
        facts: SceneFacts::TemplatedSequence(TemplatedSequence {
            template: SequenceTemplate {
                lanes: vec!["Phrase".into()],
                extent: 32,
                ticks_per_unit: 4,
                unit_label: "beat".into(),
                groups: vec![
                    TemplateGroup {
                        key: "phrase".into(),
                        lane: 0,
                        span: Span {
                            start: 0,
                            length: 32,
                        },
                        parent: None,
                        label: "Phrase".into(),
                    },
                    TemplateGroup {
                        key: "bar".into(),
                        lane: 0,
                        span: Span {
                            start: 0,
                            length: 16,
                        },
                        parent: Some("phrase".into()),
                        label: "Bar 1".into(),
                    },
                ],
            },
            slots: vec![TemplateSlot {
                key: "expect".into(),
                lane: 0,
                span: Span {
                    start: 0,
                    length: 16,
                },
                label: "ii".into(),
                expectation: "relative role".into(),
            }],
            occurrences: vec![TemplateOccurrence {
                key: "a".into(),
                source: SourceRef {
                    adapter: "test.chord".into(),
                    id: "Dm7".into(),
                },
                label: "Dm7".into(),
                detail: "authored".into(),
                placement: Some(Placement {
                    lane: 0,
                    span: Span {
                        start: 2,
                        length: 24,
                    },
                }),
                slot: Some("expect".into()),
            }],
            relations: vec![],
        }),
    }
}
fn seq(input: &mut SceneInput) -> &mut TemplatedSequence {
    let SceneFacts::TemplatedSequence(seq) = &mut input.facts else {
        unreachable!()
    };
    seq
}
#[test]
fn anticipated_and_sustained_occurrences_do_not_become_errors() {
    let input = input();
    let compiled = input.compile().unwrap();
    assert!(compiled.diagnostics.is_empty());
    assert_eq!(compiled.marks.len(), 2);
    assert_eq!(compiled.relations.len(), 1);
    assert_eq!(
        compiled.scene.relations[0].kind.as_deref(),
        Some("template.alignment")
    );
    assert_eq!(
        input,
        SceneInput::parse(&serde_json::to_string(&input).unwrap()).unwrap()
    );
}
#[test]
fn open_expectations_and_unplaced_readings_are_preserved() {
    let mut input = input();
    let s = seq(&mut input);
    s.occurrences[0].placement = None;
    s.occurrences[0].slot = None;
    let compiled = input.compile().unwrap();
    assert_eq!(compiled.unplaced.len(), 1);
    assert_eq!(compiled.marks.len(), 1);
    assert!(compiled.diagnostics[0].contains("expectation remains open"));
}
#[test]
fn invalid_spans_hierarchy_references_and_budgets_are_refused() {
    let mutations: [fn(&mut TemplatedSequence); 7] = [
        |s| s.template.groups[0].parent = Some("bar".into()),
        |s| s.template.groups[1].span.length = 33,
        |s| s.occurrences[0].slot = Some("absent".into()),
        |s| {
            s.occurrences[0].placement.as_mut().unwrap().span = Span {
                start: u32::MAX,
                length: 10,
            }
        },
        |s| s.template.ticks_per_unit = 0,
        |s| s.slots[0].lane = 1,
        |s| {
            s.relations.push(TemplateRelation {
                from: "a".into(),
                to: "missing".into(),
                kind: "repeat".into(),
                label: "Repeat".into(),
                why: "supplied".into(),
            })
        },
    ];
    for change in mutations {
        let mut input = input();
        change(seq(&mut input));
        assert!(input.compile().is_err());
    }
}
#[test]
fn nested_unknown_fields_are_rejected() {
    let mut value = serde_json::to_value(input()).unwrap();
    value["facts"]["data"]["occurrences"][0]["placement"]["span"]["inferred"] = true.into();
    assert!(SceneInput::parse(&value.to_string()).is_err());
}
