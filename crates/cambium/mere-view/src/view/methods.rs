/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl MereView<'_> {
    /// The graph's area, in pixels. A tier, when there is one, is inside it.
    pub fn graph_size(&self) -> (u32, u32) {
        graph_area(self.model, self.width, self.height)
    }

    /// The overmap band's height above the graph, in pixels; zero unless the
    /// sessions show as an overmap.
    pub fn overmap_height(&self) -> u32 {
        overmap_band(self.model, self.width, body_height(self.width, self.height))
    }

    /// The layout in force: the host's, or the default when it names none.
    pub fn layout(&self) -> &str {
        if self.model.layout.is_empty() {
            DEFAULT_LAYOUT
        } else {
            &self.model.layout
        }
    }

    /// The graph as the canvas swatch the view renders and the leaf paints.
    pub fn swatch(&self) -> GraphCanvasSwatch<String, MereNodeKind> {
        let GraphModel { nodes, relations } = &self.model.graph;
        let labels: HashMap<&str, &str> = nodes
            .iter()
            .map(|node| (node.key.as_str(), node.label.as_str()))
            .collect();
        let nodes = nodes
            .iter()
            .map(|node| GraphCanvasNode {
                id: node.key.clone(),
                kind: MereNodeKind::Graph(node.state),
                position: self.state.position(&node.key).unwrap_or((0.5, 0.5)),
                label: match node.state.word() {
                    Some(word) => format!("{}, {word}", node.label),
                    None => node.label.clone(),
                },
                key: Some(node.key.clone()),
            })
            .collect();
        let relations = relations
            .iter()
            .filter_map(|relation| {
                let from = labels.get(relation.from.as_str())?;
                let to = labels.get(relation.to.as_str())?;
                Some(GraphCanvasRelation {
                    link_key: None,
                    marks: Default::default(),
                    strength: 1.0,
                    id: relation.key.clone(),
                    from: relation.from.clone(),
                    to: relation.to.clone(),
                    kind: relation.provenance.token().to_string(),
                    label: format!("{from} to {to}"),
                    route: Vec::new(),
                    visible: true,
                    emphasized: false,
                })
            })
            .collect();
        let (width, height) = self.graph_size();
        let mut swatch = GraphCanvasSwatch::new(
            self.leaf_key,
            GraphCanvasSubgraph {
                nodes,
                edges: Vec::new(),
            },
        )
        .with_size(width, height)
        .with_label(format!("Graph of {}", self.model.title))
        .with_expand(false)
        .with_node_labels(true)
        // A narrow tile cannot hold every label; the focused and hovered keep
        // theirs, and every node keeps its accessible name.
        .with_label_culling(true)
        .with_relations(relations)
        .with_relation_lines(
            [
                (Provenance::Extracted, RelationLine::Solid),
                (Provenance::Authored, RelationLine::Heavy),
                (Provenance::Suggested, RelationLine::Dashed),
                (Provenance::Other(SAVED_FROM.into()), RelationLine::Dotted),
                (Provenance::Other(ON_SHELF.into()), RelationLine::Dotted),
            ]
            .into_iter()
            .map(|(kind, line)| (kind.token().to_string(), line))
            .collect(),
        );
        swatch = super::super::folds::project(swatch, &self.state.folds);
        // Targets as wide as the layout keeps nodes apart, so none holds
        // another's centre.
        swatch.hit_size = SEPARATION;
        swatch.focus = self.state.focused.clone();
        swatch.hovered = self.state.hovered.clone();
        if self.tier_height() > 0 {
            swatch = self.with_tier(swatch);
        }
        swatch
    }

    /// The paint leaf the host registers under [`leaf_key`](Self::leaf_key),
    /// with its palette for each node state.
    /// A palette that knows only graph node states can take
    /// [`MereNodeKind::state`].
    pub fn paint_leaf(&self, palette: impl Fn(&MereNodeKind) -> ColorF) -> GraphCanvas {
        self.swatch().paint_leaf(palette)
    }
}
