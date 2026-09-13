// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::producer::ResolvedAppearance;

impl OwnedLayout {
    /// Genet's common paint/input geometry, in document coordinates.
    pub fn element_geometry<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<genet_livery::ElementGeometry> {
        genet_livery::element_geometry(
            dom,
            &self.styles,
            &self.fragments,
            &self.element_scroll,
            node,
        )
    }

    pub(crate) fn custom_leaf_nodes<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
    ) -> Vec<(u64, NodeId)> {
        let mut leaves = Vec::new();
        walk(dom, dom.document(), &mut |node| {
            if let Some(key) = custom_leaf_key(dom, node) {
                leaves.push((key, node));
            }
        });
        leaves
    }

    pub(crate) fn resolved_appearance(
        &self,
        node: NodeId,
        properties: &[String],
    ) -> ResolvedAppearance {
        let mut result = ResolvedAppearance::default();
        for name in properties {
            if name == "color" {
                result.color = self.styles.used_color(node);
            }
            if let Some(value) = self.computed_value(node, name) {
                result.values.insert(name.clone(), value);
            }
        }
        result
    }

    pub(crate) fn local_coordinates<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        cursor: (f32, f32),
    ) -> Option<((f32, f32), (f32, f32))> {
        let geometry = self.element_geometry(dom, node)?;
        let point = (
            cursor.0 + self.viewport_scroll.0,
            cursor.1 + self.viewport_scroll.1,
        );
        if custom_leaf_key(dom, node).is_some() {
            Some((
                geometry.map_to_local(point.0, point.1)?,
                geometry.content_size(),
            ))
        } else {
            Some((
                geometry.map_to_border_local(point.0, point.1)?,
                geometry.border_size(),
            ))
        }
    }

    pub(crate) fn producer_admits_pointer<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        cursor: (f32, f32),
        producers: &crate::producer::ProducerRegistry,
    ) -> bool {
        let Some(_) = custom_leaf_key(dom, node).filter(|key| producers.contains(*key)) else {
            return true;
        };
        let Some(geometry) = self.element_geometry(dom, node) else {
            return false;
        };
        geometry
            .map_to_content(
                cursor.0 + self.viewport_scroll.0,
                cursor.1 + self.viewport_scroll.1,
            )
            .is_some()
    }
}
