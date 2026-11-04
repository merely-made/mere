// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use paint_list_api::PaintCmd;

fn white() -> ColorF {
    ColorF {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }
}

fn paint_of(leaf: &mut dyn Leaf, w: f32, h: f32) -> Vec<PaintCmd> {
    let mut cmds = Vec::new();
    let mut cx = PaintCx::new(
        &mut cmds,
        Size {
            width: w,
            height: h,
        },
    );
    leaf.paint(&mut cx);
    cmds
}

fn count(cmds: &[PaintCmd], pred: impl Fn(&PaintCmd) -> bool) -> usize {
    cmds.iter().filter(|c| pred(c)).count()
}

#[test]
fn graph_glyph_paints_edges_under_nodes() {
    let mut g = GraphGlyph::new(
        vec![
            GraphGlyphNode {
                x: 0.0,
                y: 0.0,
                color: white(),
            },
            GraphGlyphNode {
                x: 1.0,
                y: 0.5,
                color: white(),
            },
            GraphGlyphNode {
                x: 0.5,
                y: 1.0,
                color: white(),
            },
        ],
        vec![(0, 1), (1, 2), (7, 0)], // (7,0) out of range: skipped
        Size {
            width: 20.0,
            height: 20.0,
        },
    );
    let cmds = paint_of(&mut g, 20.0, 20.0);
    let strokes = count(
        &cmds,
        |c| matches!(c, PaintCmd::DrawPath(p) if p.stroke.is_some()),
    );
    let fills = count(
        &cmds,
        |c| matches!(c, PaintCmd::DrawPath(p) if p.fill.is_some()),
    );
    assert_eq!(strokes, 2, "two in-range edges");
    assert_eq!(fills, 3, "three node circles");
    // The whole painting is clipped to the leaf's own box: a panned or
    // zoomed camera must not paint into the neighbours.
    assert!(matches!(&cmds[0], PaintCmd::PushClip(_)));
    assert!(matches!(cmds.last(), Some(PaintCmd::PopClip)));
    // Painter order inside the clip: edges first.
    assert!(matches!(&cmds[1], PaintCmd::DrawPath(p) if p.stroke.is_some()));
    assert!(!g.paint_dirty());
}

#[test]
fn graph_canvas_camera_and_emphasis_share_one_paint_path() {
    let mut canvas = GraphCanvas::new(
        vec![GraphGlyphNode {
            x: 0.25,
            y: 0.5,
            color: white(),
        }],
        Vec::new(),
        Size {
            width: 100.0,
            height: 60.0,
        },
    );
    canvas.set_viewport(GraphViewport {
        pan: (0.1, 0.0),
        zoom: 2.0,
    });
    canvas.set_emphasis(Some(0), Some(0), Some(0));
    let local = canvas
        .node_local_position(
            0,
            Size {
                width: 100.0,
                height: 60.0,
            },
        )
        .expect("projected node");
    assert!(local.0 < 50.0, "zoom and pan project through the camera");

    let cmds = paint_of(&mut canvas, 100.0, 60.0);
    assert_eq!(
        count(&cmds, |cmd| matches!(cmd, PaintCmd::DrawPath(_))),
        4,
        "hover wash + selection ring + focus ring + node"
    );
}

#[test]
fn graph_canvas_paints_authored_and_fanned_routes_as_polylines() {
    let mut canvas = GraphCanvas::new(
        Vec::new(),
        Vec::new(),
        Size {
            width: 100.0,
            height: 60.0,
        },
    );
    canvas.set_relations(vec![
        GraphGlyphRelation {
            start_arrow: false,
            end_arrow: false,
            opposed: false,
            strength: 1.0,
            points: vec![(0.1, 0.5), (0.5, 0.25), (0.9, 0.5)],
            emphasized: false,
            line: RelationLine::Dashed,
        },
        GraphGlyphRelation {
            start_arrow: false,
            end_arrow: false,
            opposed: false,
            strength: 1.0,
            points: vec![(0.1, 0.5), (0.5, 0.75), (0.9, 0.5)],
            emphasized: true,
            line: RelationLine::Solid,
        },
    ]);

    let cmds = paint_of(&mut canvas, 100.0, 60.0);
    assert_eq!(
        count(
            &cmds,
            |cmd| matches!(cmd, PaintCmd::DrawPath(path) if path.stroke.is_some())
        ),
        2,
        "each relation remains a separately painted route"
    );
    // A line style reaches the stroke: one dashed route, one solid.
    let dashes: Vec<bool> = cmds
        .iter()
        .filter_map(|cmd| match cmd {
            PaintCmd::DrawPath(path) => path.stroke.as_ref().map(|s| s.dash.is_some()),
            _ => None,
        })
        .collect();
    assert_eq!(dashes, [true, false]);
}

#[test]
fn meter_paints_track_fill_peak_and_gates_on_change() {
    let mut m = Meter::new(
        true,
        Size {
            width: 8.0,
            height: 40.0,
        },
    );
    m.set_level(0.5, Some(0.8));
    let cmds = paint_of(&mut m, 8.0, 40.0);
    assert_eq!(
        count(&cmds, |c| matches!(c, PaintCmd::DrawRect(_))),
        3,
        "track + fill + peak"
    );
    assert!(!m.paint_dirty());
    m.set_level(0.5, Some(0.8));
    assert!(!m.paint_dirty(), "same level does not dirty");
    m.set_level(0.6, Some(0.8));
    assert!(m.paint_dirty(), "changed level dirties");
}

#[test]
fn meter_at_zero_paints_track_only() {
    let mut m = Meter::new(
        false,
        Size {
            width: 40.0,
            height: 8.0,
        },
    );
    m.set_level(0.0, None);
    let cmds = paint_of(&mut m, 40.0, 8.0);
    assert_eq!(count(&cmds, |c| matches!(c, PaintCmd::DrawRect(_))), 1);
}

#[test]
fn knob_paints_track_value_needle_and_clamps() {
    let mut k = Knob::new(Size {
        width: 24.0,
        height: 24.0,
    });
    k.set_value(1.7);
    assert_eq!(k.value(), 1.0, "clamped");
    let cmds = paint_of(&mut k, 24.0, 24.0);
    let strokes = count(
        &cmds,
        |c| matches!(c, PaintCmd::DrawPath(p) if p.stroke.is_some()),
    );
    assert_eq!(strokes, 3, "track arc + value arc + needle");

    // At zero the value arc is skipped.
    let mut k0 = Knob::new(Size {
        width: 24.0,
        height: 24.0,
    });
    let cmds0 = paint_of(&mut k0, 24.0, 24.0);
    let strokes0 = count(
        &cmds0,
        |c| matches!(c, PaintCmd::DrawPath(p) if p.stroke.is_some()),
    );
    assert_eq!(strokes0, 2, "track + needle only");
}
