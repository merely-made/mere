/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDom, LocalName, Namespace};

use super::*;
use crate::{AnyView, DomHandle, GenetAppRunner, GenetCtx, GenetElement, el};

fn node_text(dom: &ScriptedDom, node: genet_scripted_dom::NodeId) -> String {
    if let Some(text) = dom.text(node) {
        return text.to_owned();
    }
    dom.dom_children(node)
        .map(|child| node_text(dom, child))
        .collect()
}

/// A line's visible text, with `…` for a marker.
fn visible(projection: &FoldProjection<'_>, line: &FoldProjectionLine) -> String {
    line.segments
        .iter()
        .map(|segment| match segment {
            FoldProjectionSegment::Source(range) => &projection.source()[range.clone()],
            FoldProjectionSegment::FoldMarker(_) => "…",
        })
        .collect()
}

#[test]
fn a_marker_stays_on_the_line_it_starts_and_lines_keep_source_numbers() {
    let source = "# A\nbody one\nbody two\n## B\nsee also: x";
    let section = 4..source.find("\n## B").unwrap();
    let inline = source.find("also").unwrap()..source.find(": x").unwrap();
    let projection =
        fold_projection(source, source.len(), &[section.clone(), inline], &[]).unwrap();
    let lines = projection.lines();
    let seen = lines
        .iter()
        .map(|line| (line.start, line.number, visible(&projection, line)))
        .collect::<Vec<_>>();
    assert_eq!(
        seen,
        vec![
            (0, 1, "# A".to_owned()),
            (4, 2, "…".to_owned()),
            (22, 4, "## B".to_owned()),
            (27, 5, "see …: x".to_owned()),
        ]
    );
    assert_eq!(lines[1].line_break, Some(section.end..section.end + 1));
    assert_eq!(lines[3].line_break, None);
}

#[test]
fn crlf_is_one_break_and_an_empty_line_is_a_line() {
    let source = "a\r\n\r\nb\n";
    let projection = fold_projection(source, source.len(), &[], &[]).unwrap();
    let lines = projection.lines();
    assert_eq!(lines.len(), 3, "a trailing break opens no empty line");
    assert_eq!(
        (
            lines[0].start,
            lines[0].number,
            visible(&projection, &lines[0])
        ),
        (0, 1, "a".to_owned())
    );
    assert_eq!(lines[0].line_break, Some(1..3));
    assert_eq!((lines[1].start, lines[1].number), (3, 2));
    assert!(lines[1].segments.is_empty());
    assert_eq!(lines[1].line_break, Some(3..5));
    assert_eq!(
        (
            lines[2].start,
            lines[2].number,
            visible(&projection, &lines[2])
        ),
        (5, 3, "b".to_owned())
    );
    assert_eq!(lines[2].line_break, Some(6..7));
}

#[test]
fn rows_put_a_gutter_cell_beside_each_lines_text() {
    #[derive(Clone)]
    struct Model;
    fn view(_: &Model) -> Box<dyn AnyView<Model, (), GenetCtx, GenetElement>> {
        let source = "# A\nbody\n\n## B\n";
        let styles = [StyleRange {
            range: 0..source.len(),
            class: "syntax".into(),
        }];
        let body = source.find("body").unwrap()..source.find("\n\n").unwrap();
        let projection = fold_projection(source, source.len(), &[body], &styles).unwrap();
        Box::new(el(
            "div",
            projection.rows(|line: &FoldProjectionLine| {
                if line.number == 1 {
                    vec![Box::new(el::<_, Model, ()>("span", "▾")) as FieldChild<Model, ()>]
                } else {
                    Vec::new()
                }
            }),
        ))
    }

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom.clone(), view, Model);
    let dom = runner.dom();
    let dom = dom.borrow();
    let attribute = |node, name: &str| {
        dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
            .map(str::to_owned)
    };
    let rows: Vec<_> = dom.dom_children(runner.root()).collect();
    let cells = |row| dom.dom_children(row).collect::<Vec<_>>();
    let seen = rows
        .iter()
        .map(|row| {
            let cells = cells(*row);
            (
                attribute(*row, "class"),
                attribute(*row, "data-line"),
                node_text(&dom, cells[0]),
                node_text(&dom, cells[1]),
            )
        })
        .collect::<Vec<_>>();
    let row = |line: &str, gutter: &str, text: &str| {
        (
            Some(FOLD_ROW_CLASS.to_owned()),
            Some(line.to_owned()),
            gutter.to_owned(),
            text.to_owned(),
        )
    };
    assert_eq!(
        seen,
        vec![
            row("1", "▾", "# A\n"),
            row("2", "", "…\n"),
            row("3", "", "\n"),
            row("4", "", "## B\n"),
        ]
    );
    let first = cells(rows[0]);
    assert_eq!(
        attribute(first[0], "class").as_deref(),
        Some(FOLD_GUTTER_CLASS)
    );
    assert_eq!(
        attribute(first[1], "class").as_deref(),
        Some(FOLD_LINE_CLASS)
    );
    let styled = dom.dom_children(first[1]).next().expect("styled text");
    assert_eq!(
        attribute(styled, "class").as_deref(),
        Some("syntax"),
        "a style spanning lines is clipped to each row"
    );
}

#[test]
fn unicode_source_is_borrowed_and_folded_at_character_boundaries() {
    let source = "a\u{301} 👩\u{200d}🔬\nbody\n";
    let fold_start = source.find('👩').unwrap();
    let fold_end = source.find("\nbody").unwrap();
    let projection = fold_projection(source, source.len(), &[fold_start..fold_end], &[]).unwrap();

    assert_eq!(projection.source(), source);
    assert_eq!(projection.visible_text(), "a\u{301} …\nbody\n");
    assert_eq!(
        projection.segments(),
        &[
            FoldProjectionSegment::Source(0..fold_start),
            FoldProjectionSegment::FoldMarker(fold_start..fold_end),
            FoldProjectionSegment::Source(fold_end..source.len()),
        ]
    );
    assert_eq!(
        source, "a\u{301} 👩\u{200d}🔬\nbody\n",
        "projection never changes its source"
    );
}

#[test]
fn rejects_stale_length_and_invalid_fold_ranges() {
    let source = "éx";
    assert!(matches!(
        fold_projection(source, source.len() + 1, &[], &[]),
        Err(FoldProjectionError::SourceLengthMismatch {
            expected: 4,
            actual: 3,
        })
    ));
    for range in [0..0, 0..4, 1..2] {
        assert!(matches!(
            fold_projection(source, source.len(), &[range.clone()], &[]),
            Err(FoldProjectionError::InvalidFoldRange(actual)) if actual == range
        ));
    }
}

#[test]
fn outer_fold_supersedes_nested_ranges_and_crossing_ranges_fail() {
    let source = "0123456789";
    let projection = fold_projection(source, source.len(), &[2..5, 1..8, 3..4], &[]).unwrap();
    assert_eq!(
        projection.segments(),
        &[
            FoldProjectionSegment::Source(0..1),
            FoldProjectionSegment::FoldMarker(1..8),
            FoldProjectionSegment::Source(8..10),
        ]
    );
    assert!(matches!(
        fold_projection(source, source.len(), &[1..5, 4..8], &[]),
        Err(FoldProjectionError::CrossingFoldRanges {
            first,
            second,
        }) if first == (1..5) && second == (4..8)
    ));
}

#[test]
fn styles_are_clipped_to_visible_source_and_markers_are_semantic() {
    #[derive(Clone)]
    struct Model {
        source: String,
    }
    fn view(model: &Model) -> Box<dyn AnyView<Model, (), GenetCtx, GenetElement>> {
        let source = model.source.as_str();
        let fold_start = source.find('👩').unwrap();
        let fold_end = source.find("\nbody").unwrap();
        let styles = [
            StyleRange {
                range: 0..fold_end,
                class: "syntax-a".into(),
            },
            StyleRange {
                range: fold_start..fold_end,
                class: "hidden".into(),
            },
            StyleRange {
                range: 1..2,
                class: "invalid-boundary".into(),
            },
        ];
        let projection =
            fold_projection(source, source.len(), &[fold_start..fold_end], &styles).unwrap();
        Box::new(el("div", projection.field_children::<Model, ()>()))
    }

    let model = Model {
        source: "a\u{301} 👩\u{200d}🔬\nbody".into(),
    };
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom.clone(), view, model);
    let dom = runner.dom();
    let dom = dom.borrow();
    let children: Vec<_> = dom.dom_children(runner.root()).collect();
    assert_eq!(
        children.len(),
        3,
        "styled visible source, marker, visible source"
    );
    assert_eq!(node_text(&dom, children[0]), "a\u{301} ");
    assert_eq!(node_text(&dom, children[1]), "…");
    assert_eq!(node_text(&dom, children[2]), "\nbody");
    assert_eq!(
        dom.attribute(children[0], &Namespace::from(""), &LocalName::from("class")),
        Some("syntax-a")
    );
    assert_eq!(
        dom.attribute(children[1], &Namespace::from(""), &LocalName::from("class")),
        Some(FOLD_MARKER_CLASS)
    );
    assert_eq!(
        dom.attribute(
            children[1],
            &Namespace::from(""),
            &LocalName::from("aria-label")
        ),
        Some(FOLD_MARKER_ACCESSIBLE_LABEL)
    );
    assert_eq!(
        dom.attribute(children[1], &Namespace::from(""), &LocalName::from("role")),
        Some("note")
    );
}
