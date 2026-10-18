// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Face-only override over the existing resolved body footprint.
use super::*;
impl Canvas {
    /// Local replacement epoch; never a source/session identity.
    pub fn graph_view_epoch(&self) -> u64 {
        self.graph_view_epoch
    }
    /// Current outer body side in screen pixels. Caption/occlusion changes
    /// cannot feed back into the resolved footprint used by face paint.
    pub fn node_presentation_screen_size(&self, member: uuid::Uuid) -> Option<f64> {
        let key = self.graph.get_node_key_by_id(member)?;
        let billboard = self.node_billboard(key)?;
        let size = f64::from(billboard.local_side * billboard.zoom);
        (size.is_finite() && size >= 0.).then_some(size)
    }
    /// Effective CSS body corner radius in local (pre-camera-zoom) pixels.
    /// This follows the current stylesheet cascade: grammar body/site/suggestion
    /// classes are emitted after representation classes and therefore override
    /// their radius; outside grammar, Glyph's circle rule follows shape rules.
    pub(crate) fn node_effective_local_corner_radius(&self, key: NodeKey, side: f32) -> f32 {
        let half = side * 0.5;
        if self.graph_grammar {
            let attention = self
                .graph
                .get_node(key)
                .and_then(|node| self.member_attention(node.id));
            return match attention {
                Some(scenograph::attention::Attention::Forward)
                | Some(scenograph::attention::Attention::Suggestion) => half,
                _ => 2.0_f32.min(half), // `.gnode-site { border-radius: 2px }`
            };
        }
        if self.node_representation_class(key).contains("glyph") {
            return half;
        }
        match self.node_shape(key) {
            NodeShape::Square => 0.0,
            NodeShape::Rounded => 9.0_f32.min(half),
            NodeShape::Circle => half,
        }
    }
    /// Replace this view's rule face layer atomically. No geometry/physics edits.
    pub fn set_node_rule_faces(
        &mut self,
        epoch: u64,
        faces: HashMap<uuid::Uuid, sceno::Representation>,
    ) -> Result<bool, String> {
        if epoch != self.graph_view_epoch {
            return Err("Node face view was replaced".into());
        }
        for (member, face) in &faces {
            if self.graph.get_node_by_id(*member).is_none() {
                return Err(format!("Access {member} is absent"));
            }
            if !matches!(
                face,
                sceno::Representation::Glyph | sceno::Representation::Card
            ) {
                return Err("Only Glyph/Card face treatments are supported".into());
            }
        }
        if self.node_rule_faces == faces {
            return Ok(false);
        }
        self.node_rule_faces = faces;
        Ok(true)
    }
    pub fn node_rule_representation(&self, member: uuid::Uuid) -> Option<&sceno::Representation> {
        self.node_rule_faces.get(&member)
    }
    pub(crate) fn node_representation_class(&self, key: NodeKey) -> &'static str {
        let requested = self
            .graph
            .get_node(key)
            .and_then(|node| self.node_rule_faces.get(&node.id))
            .or_else(|| self.projection_representation(key));
        match requested {
            Some(sceno::Representation::Glyph) => " gnode-representation-glyph",
            Some(sceno::Representation::LivePane) => " gnode-representation-live-pane",
            _ => " gnode-representation-card",
        }
    }
}
#[cfg(test)]
#[path = "node_presentation_tests.rs"]
mod tests;
