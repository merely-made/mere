// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{ProducerFrameStats, ResolvedAppearance};
use crate::{AppCtx, Host, NodeId, meristem_bounds::RootView};

impl<State: 'static, Logic, V> AppCtx<'_, State, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// Read only the declared properties from the retained, resolved style.
    /// CSS interpretation and color-context resolution stay in Genet.
    pub fn resolved_appearance(
        &self,
        node: NodeId,
        properties: &[&str],
    ) -> Option<ResolvedAppearance> {
        let properties: Vec<_> = properties.iter().map(|value| (*value).to_owned()).collect();
        Some(self.layout?.resolved_appearance(node, &properties))
    }

    pub fn content_size(&self, node: NodeId) -> Option<(f32, f32)> {
        let dom = self.runner.dom();
        let dom = dom.borrow();
        Some(self.layout?.element_geometry(&*dom, node)?.content_size())
    }

    /// Inverse document paint mapping into the node's content box. Rejects
    /// clips, outside points and singular transforms. Ordinary DOM hit routing
    /// still decides which node receives input before an application queries it.
    pub fn content_point(&self, node: NodeId, x: f32, y: f32) -> Option<(f32, f32)> {
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let scroll = layout.viewport_scroll();
        layout
            .element_geometry(&*dom, node)?
            .map_to_content(x + scroll.0, y + scroll.1)
    }
}

impl<State: 'static, Logic, V> Host<State, Logic, V>
where
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Retire staged images before a platform drops/replaces its surface, or
    /// suspend transient targets while the containing window is hidden.
    pub fn suspend_producers(&mut self) {
        self.s
            .producers
            .suspend_all(self.s.surface.as_ref().map(|surface| surface.renderer()));
    }

    pub(crate) fn prepare_producers(&mut self, scale: f32) -> ProducerFrameStats {
        let (Some(surface), Some(layout), Some(runner)) = (
            self.s.surface.as_deref(),
            self.s.layout.as_ref(),
            self.s.runner.as_ref(),
        ) else {
            self.s.producers.suspend_all(None);
            return ProducerFrameStats::default();
        };
        if self.s.hidden {
            self.s.producers.suspend_all(Some(surface.renderer()));
            return ProducerFrameStats::default();
        }
        let dom = runner.dom();
        let dom = dom.borrow();
        self.s.producers.prepare(surface, layout, &*dom, scale)
    }
}
