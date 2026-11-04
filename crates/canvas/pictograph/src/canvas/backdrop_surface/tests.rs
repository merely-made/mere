// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
fn input() -> SceneInput {
    SceneInput::parse(include_str!(
        "../../../../../../support/design-studies/backdrops/fretboard.json"
    ))
    .unwrap()
}
#[test]
fn picking_and_refused_replacement_preserve_occurrence_identity() {
    let mut surface = BackdropSurface::new(&input()).unwrap();
    surface.select(Some("string-5-fret-3")).unwrap();
    let before = surface.scene().clone();
    let mut invalid = input();
    invalid.version = 99;
    assert!(surface.replace(&invalid).is_err());
    assert_eq!(surface.scene(), &before);
    assert_eq!(surface.selected(), Some("string-5-fret-3"));
    let mark = surface
        .scene()
        .marks
        .iter()
        .find(|m| m.key == "string-5-fret-3")
        .unwrap();
    let p = surface.scene().scene.items[mark.instance.0 as usize]
        .transform
        .translate;
    let fit = surface.fit(800, 500).unwrap();
    let (x, y) = fit.to_viewport((p.x, p.y));
    assert_eq!(surface.pick(x, y, 800, 500), Some("string-5-fret-3"));
    surface.select(Some("string-2-fret-1")).unwrap();
    assert!(surface.retained().contains("string-5-fret-3"));
    assert!(surface.select(Some("missing")).is_err());
    assert_eq!(surface.selected(), Some("string-2-fret-1"));
}
#[test]
fn progression_span_is_pickable_at_its_edge_and_refusal_preserves_attention() {
    let input = SceneInput::parse(include_str!(
        "../../../../../../support/design-studies/backdrops/templates/progression.json"
    ))
    .unwrap();
    let mut surface = BackdropSurface::new(&input).unwrap();
    let mark = surface
        .scene()
        .marks
        .iter()
        .find(|m| m.key == "bar-2")
        .unwrap();
    let item = &surface.scene().scene.items[mark.instance.0 as usize];
    let Footprint::Rect { size } = item.footprint else {
        panic!("span footprint required")
    };
    let p = item.transform.translate;
    let fit = surface.fit(900, 540).unwrap();
    let (x, y) = fit.to_viewport((p.x + size.w / 2.0 - 1.0, p.y));
    assert_eq!(surface.pick(x, y, 900, 540), Some("bar-2"));
    surface.select(Some("bar-2")).unwrap();
    let before = surface.scene().clone();
    let mut invalid = input;
    let scenomise::backdrops::SceneFacts::TemplatedSequence(sequence) = &mut invalid.facts else {
        unreachable!()
    };
    sequence.occurrences[2]
        .placement
        .as_mut()
        .unwrap()
        .span
        .length = u32::MAX;
    assert!(surface.replace(&invalid).is_err());
    assert_eq!(surface.selected(), Some("bar-2"));
    assert_eq!(surface.scene(), &before);
}
#[test]
fn font_roles_reach_the_label_sheet_only_when_set() {
    let mut surface = BackdropSurface::new(&input()).unwrap();
    assert_eq!(surface.text.rooted("a{}"), "a{}");
    surface.set_font_roles("--tabard-font-interface: Probe;");
    assert!(
        surface
            .text
            .rooted("a{}")
            .starts_with(":root { --tabard-font-interface: Probe;")
    );
}
#[test]
fn timeline_rows_use_height_and_stretched_spans_remain_pickable() {
    let input = SceneInput::parse(include_str!(
        "../../../../../../support/design-studies/backdrops/templates/progression.json"
    ))
    .unwrap();
    let surface = BackdropSurface::new(&input).unwrap();
    let fit = surface.fit(580, 300).unwrap();
    let supplied = surface
        .scene()
        .marks
        .iter()
        .find(|m| m.key == "bar-2")
        .unwrap();
    let expected = surface
        .scene()
        .marks
        .iter()
        .find(|m| m.key == "role-2")
        .unwrap();
    let at = |m: &scenomise::backdrops::Mark| {
        surface.scene().scene.items[m.instance.0 as usize]
            .transform
            .translate
    };
    let (_, supplied_y) = fit.to_viewport((at(supplied).x, at(supplied).y));
    let (_, expected_y) = fit.to_viewport((at(expected).x, at(expected).y));
    assert!(
        expected_y - supplied_y >= 48.0,
        "rows must leave room for labels and arcs"
    );
    let item = &surface.scene().scene.items[supplied.instance.0 as usize];
    let Footprint::Rect { size } = item.footprint else {
        panic!("span")
    };
    let (x, y) = fit.to_viewport((at(supplied).x, at(supplied).y + size.h * 0.45));
    assert_eq!(surface.pick(x, y, 580, 300), Some("bar-2"));
    let fretboard = BackdropSurface::new(&super::tests::input()).unwrap();
    let fit = fretboard.fit(580, 300).unwrap();
    assert_eq!(
        fit.scale_x, fit.scale_y,
        "instrument geometry retains its aspect ratio"
    );
}
#[test]
fn interaction_geometry_round_trips_across_template_consumers_and_viewports() {
    for json in [
        include_str!(
            "../../../../../../support/design-studies/backdrops/templates/progression.json"
        ),
        include_str!("../../../../../../support/design-studies/backdrops/templates/metre.json"),
    ] {
        let input = SceneInput::parse(json).unwrap();
        let surface = BackdropSurface::new(&input).unwrap();
        for (w, h) in [(580, 300), (1050, 300), (420, 500)] {
            for mark in surface.scene().marks.iter().filter(|m| m.selectable) {
                let item = &surface.scene().scene.items[mark.instance.0 as usize];
                let at = item.transform.translate;
                let point = surface.viewport_point(at, w, h).unwrap();
                let restored = surface.scene_point(point.x, point.y, w, h).unwrap();
                assert!((at.x - restored.x).abs() < 0.001);
                assert!((at.y - restored.y).abs() < 0.001);
                if let Some(rect) = surface.occurrence_rect(&mark.key, w, h) {
                    assert!((rect.x + rect.width / 2.0 - point.x).abs() < 0.001);
                    assert!((rect.y + rect.height / 2.0 - point.y).abs() < 0.001);
                    // The visible edge is also a valid hit, with the same stretched Y fit.
                    assert_eq!(
                        surface.pick(rect.x + 0.1, point.y, w, h),
                        Some(mark.key.as_str())
                    );
                }
            }
        }
        assert!(surface.scene_point(f32::NAN, 0.0, 580, 300).is_none());
        assert!(surface.occurrence_rect("missing", 580, 300).is_none());
        assert!(surface.scene_point(0.0, 0.0, 40, 40).is_none());
    }
}
#[test]
fn painting_requires_host_fonts_and_valid_viewport() {
    let mut surface = BackdropSurface::new(&input()).unwrap();
    assert!(
        surface
            .paint(800, 500, &SurfaceTheme::default())
            .unwrap_err()
            .contains("lend a font")
    );
    assert!(
        surface
            .paint(40, 40, &SurfaceTheme::default())
            .unwrap_err()
            .contains("viewport")
    );
}
