use scenomise::{
    backdrops::SceneInput,
    span_edit::{SpanEdit, SpanEditIntent},
};
const METRE: &str =
    include_str!("../../../../../support/design-studies/backdrops/templates/metre.json");
const MUSIC: &str =
    include_str!("../../../../../support/design-studies/backdrops/templates/progression.json");
#[test]
fn metre_and_progressions_share_proposals_without_source_mutation() {
    for json in [METRE, MUSIC] {
        let source = SceneInput::parse(json).unwrap();
        let key = source
            .compile()
            .unwrap()
            .marks
            .iter()
            .find(|m| source.span_target(&m.key).is_some())
            .unwrap()
            .key
            .clone();
        let before = source.clone();
        let t = source.span_target(&key).unwrap();
        for edit in [SpanEdit::Move, SpanEdit::Start, SpanEdit::End] {
            let p = SpanEditIntent::propose(
                &source,
                &key,
                edit,
                2.0 * t.ticks_per_unit as f32,
                u32::from(t.ticks_per_unit),
            )
            .unwrap();
            let mut copy = source.clone();
            copy.facts = p.proposed_facts(&source).unwrap();
            copy.validate().unwrap();
            assert_eq!(source, before);
            assert_eq!(copy.span_target(&key).unwrap().span, p.after);
            assert_eq!(
                p,
                serde_json::from_str::<SpanEditIntent>(&serde_json::to_string(&p).unwrap())
                    .unwrap()
            );
            for cause in 0..6 {
                let mut bad = p.clone();
                match cause {
                    0 => bad.revision.push('x'),
                    1 => bad.source.id.push('x'),
                    2 => bad.occurrence = "missing".into(),
                    3 => bad.lane += 1,
                    4 => bad.before.start += 1,
                    _ => bad.after.length = 0,
                };
                assert!(bad.proposed_facts(&source).is_err());
                assert_eq!(source, before);
            }
        }
    }
}
#[test]
fn checked_span_math_refuses_invalid_inputs_and_clamps_edges() {
    use scenomise::template::Span;
    let s = Span {
        start: 2,
        length: 2,
    };
    assert_eq!(SpanEdit::Move.apply(s, -100.0, 8, 1).unwrap().start, 0);
    assert_eq!(SpanEdit::Move.apply(s, 100.0, 8, 1).unwrap().start, 6);
    assert_eq!(SpanEdit::Start.apply(s, 100.0, 8, 1).unwrap().length, 1);
    assert_eq!(SpanEdit::End.apply(s, -100.0, 8, 1).unwrap().length, 1);
    for edit in [SpanEdit::Move, SpanEdit::Start, SpanEdit::End] {
        assert_eq!(edit.apply(s, 0.0, 8, 4).unwrap(), s);
    }
    assert!(SpanEdit::Move.apply(s, f32::NAN, 8, 1).is_err());
    assert!(SpanEdit::Move.apply(s, 1.0, 8, 0).is_err());
    assert!(
        SpanEdit::Move
            .apply(
                Span {
                    start: u32::MAX,
                    length: 2
                },
                1.0,
                8,
                1
            )
            .is_err()
    );
}

#[test]
fn metre_alignment_is_bounded_explicit_and_legacy_compatible() {
    use scenomise::{backdrops::SceneFacts, template::Span};
    let mut input = SceneInput::parse(METRE).unwrap();
    let original = serde_json::to_value(&input).unwrap();
    assert!(
        original["facts"]["data"]["lines"][0]
            .get("alignment")
            .is_none()
    );
    let key = "line-1/syllable/0";
    assert_eq!(
        input.span_target(key).unwrap().span,
        Span {
            start: 0,
            length: 1
        }
    );
    let SceneFacts::MetreGrid(grid) = &mut input.facts else {
        unreachable!()
    };
    let placements = grid.lines[0]
        .syllables
        .iter()
        .enumerate()
        .map(|(i, _)| {
            Some(Span {
                start: i as u32,
                length: 1,
            })
        })
        .collect::<Vec<_>>();
    grid.lines[0].alignment = Some(placements);
    grid.lines[0].alignment.as_mut().unwrap()[0] = None;
    input.validate().unwrap();
    assert!(input.span_target(key).is_none());
    let scene = input.compile().unwrap();
    assert!(scene.unplaced.iter().any(|m| m.key == key));
    assert!(!scene.marks.iter().any(|m| m.key == key));
    for cause in 0..4 {
        let mut bad = input.clone();
        let SceneFacts::MetreGrid(grid) = &mut bad.facts else {
            unreachable!()
        };
        match cause {
            0 => {
                grid.lines[0].alignment.as_mut().unwrap().pop();
            },
            1 => {
                grid.lines[0].alignment.as_mut().unwrap()[0] = Some(Span {
                    start: 0,
                    length: 0,
                })
            },
            2 => {
                grid.lines[0].alignment.as_mut().unwrap()[0] = Some(Span {
                    start: u32::MAX,
                    length: 2,
                })
            },
            _ => grid.lines[1].key = key.into(),
        }
        assert!(bad.validate().is_err());
    }
    assert_eq!(
        SceneInput::parse(&serde_json::to_string(&input).unwrap()).unwrap(),
        input
    );
}
