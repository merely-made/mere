// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A bounded consumer of Cambium's graph canvas and Sprigging's paint leaf.
//! Selection, hover and focus are specimen state; they never edit the theme.

use cambium::{
    AnyView, GenetCtx, GenetElement, GraphCanvas, GraphCanvasEdge, GraphCanvasNode,
    GraphCanvasSubgraph, GraphCanvasSwatch, graph_canvas_swatch_with_focus,
};
use sprigging::ColorF;
use tabard::Theme;
use tabard::theme::registry::Mode;
use tabard::theme::seed::{derive_from_def_for_mode, harmonized_seeds};
use tinct::{ModeProfile, Srgb, derive_palette_with};

// Host leaves and texture producers share the lower-62-bit key namespace.
pub(crate) const GRAPH_LEAF_KEY: u64 = 0x0000_7461_6261_7264;
const _: () = assert!(GRAPH_LEAF_KEY < (1_u64 << 62));

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GraphAccent {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphSpecimen {
    pub(crate) swatch: GraphCanvasSwatch<u8, GraphAccent>,
}

impl Default for GraphSpecimen {
    fn default() -> Self {
        let mut swatch = GraphCanvasSwatch::new(
            GRAPH_LEAF_KEY,
            GraphCanvasSubgraph {
                nodes: vec![
                    GraphCanvasNode {
                        id: 1,
                        kind: GraphAccent::Primary,
                        position: (0.18, 0.58),
                        label: "Notes".into(),
                        key: Some("tabard-notes".into()),
                    },
                    GraphCanvasNode {
                        id: 2,
                        kind: GraphAccent::Secondary,
                        position: (0.48, 0.22),
                        label: "Garden".into(),
                        key: Some("tabard-garden".into()),
                    },
                    GraphCanvasNode {
                        id: 3,
                        kind: GraphAccent::Tertiary,
                        position: (0.78, 0.67),
                        label: "Paths".into(),
                        key: Some("tabard-paths".into()),
                    },
                ],
                edges: vec![
                    GraphCanvasEdge { from: 1, to: 2 },
                    GraphCanvasEdge { from: 2, to: 3 },
                    GraphCanvasEdge { from: 1, to: 3 },
                ],
            },
        )
        // Fits the smallest specimen card without shrinking the canvas below
        // its hit-target projection. A larger pane can center this viewport.
        .with_size(210, 126)
        .with_label("Theme graph preview")
        .with_node_labels(true)
        .with_expand(false);
        swatch.node_radius = 7.0;
        swatch.edge_width = 1.5;
        swatch.selected = Some(2);
        Self { swatch }
    }
}

impl GraphSpecimen {
    pub(crate) fn view<State: 'static>(
        &self,
        graph_mut: fn(&mut State) -> &mut Self,
    ) -> Box<dyn AnyView<State, (), GenetCtx, GenetElement>> {
        Box::new(graph_canvas_swatch_with_focus(
            &self.swatch,
            move |state: &mut State, id| graph_mut(state).swatch.selected = Some(id),
            move |state: &mut State, id| graph_mut(state).swatch.hovered = id,
            move |state: &mut State, id| graph_mut(state).swatch.focus = id,
            |_: &mut State| {},
        ))
    }

    /// Hosts refresh the registered leaf after dispatch so its emphasis and
    /// theme colors match the retained targets produced by `view`.
    pub(crate) fn paint_leaf(&self, theme: &Theme, mode: &Mode) -> GraphCanvas {
        let tokens = derive_from_def_for_mode(theme, mode);
        let palette = derive_palette_with(
            &harmonized_seeds(theme),
            ModeProfile {
                dark: mode.dark(),
                high_contrast: mode.high_contrast(),
            },
        );
        let mut leaf = self.swatch.paint_leaf(|kind| {
            paint_color(match kind {
                GraphAccent::Primary => palette.primary,
                GraphAccent::Secondary => palette.secondary,
                GraphAccent::Tertiary => palette.tertiary,
            })
        });
        leaf.edge_color = paint_color(palette.text_dim);
        leaf.selection_color = paint_color(tokens.graph_node_selection);
        leaf.focus_color = paint_color(tokens.graph_node_focus_ring);
        leaf
    }
}

fn paint_color(color: Srgb) -> ColorF {
    ColorF::new(
        f32::from(color.r) / 255.0,
        f32::from(color.g) / 255.0,
        f32::from(color.b) / 255.0,
        f32::from(color.a) / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cambium_genet_winit_host::Harness;
    use sprigging::{Leaf, PaintCmd, PaintCx, Size};
    use taproot::Selector;
    use winit::keyboard::NamedKey;

    type GraphView = Box<dyn AnyView<GraphSpecimen, (), GenetCtx, GenetElement>>;

    fn view(graph: &GraphSpecimen) -> GraphView {
        graph.view(|graph| graph)
    }

    #[test]
    fn actual_node_targets_update_selection_focus_and_hover() {
        let mut host = Harness::new(
            cambium::GRAPH_CANVAS_SWATCH_CSS,
            GraphSpecimen::default(),
            view as fn(&GraphSpecimen) -> GraphView,
        );
        host.layout_at(300.0, 180.0);
        assert!(host.click_on(&Selector::role("button").containing("Notes")));
        assert_eq!(host.state().swatch.selected, Some(1));
        assert_eq!(host.state().swatch.focus, Some(1));
        host.key_named(NamedKey::Tab);
        assert_eq!(host.state().swatch.focus, Some(2));
        host.key_named(NamedKey::Enter);
        assert_eq!(host.state().swatch.selected, Some(2));
        let (id, (x, y)) = host.state().swatch.projected_positions()[2];
        let id = *id;
        host.move_to(x, y);
        assert_eq!(host.state().swatch.hovered, Some(id));
        host.move_to(290.0, 170.0);
        assert_eq!(host.state().swatch.hovered, None);
    }

    #[test]
    fn shared_leaf_paints_draft_accents_and_mode_feedback_tokens() {
        let theme = crate::WorkshopState::in_memory().draft_theme().clone();
        let mut graph = GraphSpecimen::default();
        graph.swatch.focus = Some(1);
        for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
            let tokens = derive_from_def_for_mode(&theme, &mode);
            let palette = derive_palette_with(
                &harmonized_seeds(&theme),
                ModeProfile {
                    dark: mode.dark(),
                    high_contrast: mode.high_contrast(),
                },
            );
            let mut leaf = graph.paint_leaf(&theme, &mode);
            let mut commands = Vec::new();
            leaf.paint(&mut PaintCx::new(
                &mut commands,
                Size {
                    width: 210.0,
                    height: 126.0,
                },
            ));
            let fills: Vec<_> = commands
                .iter()
                .filter_map(|command| match command {
                    PaintCmd::DrawPath(path) => path.fill,
                    _ => None,
                })
                .collect();
            let strokes: Vec<_> = commands
                .iter()
                .filter_map(|command| match command {
                    PaintCmd::DrawPath(path) => path.stroke.as_ref().map(|stroke| stroke.color),
                    _ => None,
                })
                .collect();
            assert_eq!(
                fills,
                [
                    paint_color(palette.primary),
                    paint_color(palette.secondary),
                    paint_color(palette.tertiary)
                ]
            );
            assert!(strokes.contains(&paint_color(palette.text_dim)));
            assert!(strokes.contains(&paint_color(tokens.graph_node_selection)));
            assert!(strokes.contains(&paint_color(tokens.graph_node_focus_ring)));
        }
    }

    #[test]
    fn leaf_and_hit_targets_share_fixture_projection() {
        let graph = GraphSpecimen::default();
        let state = crate::WorkshopState::in_memory();
        let leaf = graph.paint_leaf(state.draft_theme(), state.mode());
        for (index, (_, position)) in graph.swatch.projected_positions().into_iter().enumerate() {
            assert_eq!(
                leaf.node_local_position(
                    index,
                    Size {
                        width: 210.0,
                        height: 126.0
                    }
                ),
                Some(position)
            );
        }
        assert!(
            !graph.swatch.show_expand,
            "the specimen has no separate expanded view"
        );
    }
}
