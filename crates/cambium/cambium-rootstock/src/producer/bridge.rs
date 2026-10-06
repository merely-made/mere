// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{ProducerFrameStats, ResolvedAppearance};
use crate::{AppCtx, Host, NodeId, meristem_bounds::RootView};

impl<State: 'static, Logic, V, T> AppCtx<'_, State, Logic, V, T>
where
    T: crate::HostTree<State>,
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
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        Some(self.layout?.element_geometry(&view, node)?.content_size())
    }

    /// Inverse document paint mapping into the node's content box. Rejects
    /// clips, outside points and singular transforms. Ordinary DOM hit routing
    /// still decides which node receives input before an application queries it.
    pub fn content_point(&self, node: NodeId, x: f32, y: f32) -> Option<(f32, f32)> {
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        let scroll = layout.viewport_scroll();
        layout
            .element_geometry(&view, node)?
            .map_to_content(x + scroll.0, y + scroll.1)
    }
}

impl<State: 'static, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Draw with a caller-supplied monotonic timestamp. Ordinary redraw remains
    /// untimed for deterministic callers; timestamps never leak into later draws.
    pub fn redraw_at(&mut self, timestamp: std::time::Duration) {
        self.s.shared.producers.timestamp = Some(timestamp);
        self.redraw();
        self.s.shared.producers.timestamp = None;
    }

    /// Update platform visibility immediately, even if no frame will be delivered.
    pub fn set_hidden(&mut self, hidden: bool) {
        if self.s.hidden == hidden {
            return;
        }
        self.s.hidden = hidden;
        if hidden {
            self.suspend_producers();
        } else if let Some(window) = &self.s.window {
            window.request_redraw();
        }
    }

    /// Retire staged images before a platform drops/replaces its surface, or
    /// suspend transient targets while the containing window is hidden.
    pub fn suspend_producers(&mut self) {
        let dom = self
            .s
            .runner
            .as_ref()
            .map(|runner| (runner.dom(), runner.mount()));
        let dom_ref = dom.as_ref().map(|(dom, mount)| (dom.borrow(), *mount));
        let view = dom_ref
            .as_ref()
            .map(|(dom, mount)| crate::WindowDom::new(dom, *mount));
        let shared = &mut self.s.shared;
        let held = &shared.held_elsewhere;
        let elsewhere = |key: u64, owner: Option<NodeId>| {
            held.contains(&key)
                || owner
                    .zip(view.as_ref())
                    .is_some_and(|(n, v)| v.elsewhere(n))
        };
        shared.producers.suspend_except(
            &elsewhere,
            self.s.surface.as_ref().map(|surface| surface.renderer()),
        );
        if held.is_empty() {
            shared.producers.forget_device();
        }
    }

    pub(crate) fn prepare_producers(&mut self, scale: f32) -> ProducerFrameStats {
        let (Some(surface), Some(layout), Some(runner)) = (
            self.s.surface.as_deref(),
            self.s.layout.as_ref(),
            self.s.runner.as_ref(),
        ) else {
            let shared = &mut self.s.shared;
            let held = &shared.held_elsewhere;
            shared
                .producers
                .suspend_except(&|key, _| held.contains(&key), None);
            if held.is_empty() {
                shared.producers.forget_device();
            }
            return ProducerFrameStats::default();
        };
        let dom = runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, runner.mount());
        let shared = &mut self.s.shared;
        let held = &shared.held_elsewhere;
        // Another window owns a producer whose key it last laid out, or whose
        // node now lives under its root.
        let elsewhere = |key: u64, owner: Option<NodeId>| {
            held.contains(&key) || owner.is_some_and(|n| view.elsewhere(n))
        };
        if self.s.hidden {
            shared
                .producers
                .suspend_except(&elsewhere, Some(surface.renderer()));
            if held.is_empty() {
                shared.producers.forget_device();
            }
            return ProducerFrameStats::default();
        }
        shared
            .producers
            .prepare_window(surface, layout, &view, scale, &elsewhere)
    }
}
