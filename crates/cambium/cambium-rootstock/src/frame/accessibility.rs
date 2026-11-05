// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Latest owner-stamped publication observation; reading does not sync.
    pub fn a11y_publication(&self) -> crate::A11yPublicationSnapshot {
        crate::A11yPublicationSnapshot {
            host: self.s.presentation_host,
            sequence: self.s.a11y_publication_sequence,
            publication: self.s.a11y_publication,
        }
    }

    pub(crate) fn stamp_a11y_publication(
        &mut self,
        publication: crate::A11yPublication,
    ) -> crate::A11yPublicationSnapshot {
        self.s.a11y_publication = publication;
        self.s.a11y_publication_sequence = self
            .s
            .a11y_publication_sequence
            .checked_add(1)
            .expect("accessibility observation sequence exhausted");
        self.a11y_publication()
    }

    pub(crate) fn record_a11y_publication(&mut self, publication: crate::A11yPublication) {
        let snapshot = self.stamp_a11y_publication(publication);
        if let Some(observer) = &mut self.s.a11y_publication_observer {
            observer(snapshot);
        }
    }

    /// Publish after an accepted picture, or service the previous tree's
    /// actions while a native submission/refusal has no new displayed picture.
    /// Legacy adapters retain their combined synchronization path.
    pub fn sync_a11y(&mut self) -> Vec<crate::A11yDispatch> {
        if self.s.native_receiving
            && (self.s.last_redraw_presentation.is_none()
                || self.a11y_publication_skip()
                    == Some(crate::A11yPublicationSkip::PendingPresentation))
        {
            return self.drain_a11y_actions();
        }
        self.sync_a11y_tree()
    }

    /// Consume actions against the last submitted namespace without building
    /// another tree. Every request is re-admitted independently against the
    /// live owner, target and contributor. Opted-in adapters consume gestures
    /// on hidden/ended/recovering hosts without replaying them on later reveal.
    ///
    /// An adapter returning `None` has not opted into a separate drain; its
    /// original combined synchronization and publication guards still apply.
    pub fn drain_a11y_actions(&mut self) -> Vec<crate::A11yDispatch> {
        let started = crate::Instant::now();
        let owner = crate::A11yOwner::new(&self.s.a11y_owner);
        let requests = self
            .s
            .a11y
            .as_mut()
            .and_then(|a11y| a11y.drain_actions(&owner));
        let Some(requests) = requests else {
            return self.sync_a11y_tree();
        };
        let reason = self
            .a11y_publication_skip()
            .unwrap_or(crate::A11yPublicationSkip::PendingPresentation);
        self.record_a11y_skip(reason);
        let results = self.apply_a11y_requests(&requests);
        if let Some(profile) = self.s.last_frame_profile.as_mut() {
            profile.a11y_actions_us = profile
                .a11y_actions_us
                .saturating_add(elapsed_us(started.elapsed()));
        }
        results
    }

    fn native_a11y_context_available(&self) -> bool {
        !self.s.native_receiving
            || (self.s.displayed_presentation.is_some()
                && self.s.displayed_input.is_some()
                && !self.s.needs_recovery_surface
                && self
                    .s
                    .shared
                    .render_core
                    .as_ref()
                    .is_none_or(|core| core.device_failure().is_none())
                && self.s.recovery_status.as_ref().is_none_or(|status| {
                    matches!(status, genet_render_host::RecoveryStatus::Ready { .. })
                }))
    }

    fn a11y_publication_skip(&self) -> Option<crate::A11yPublicationSkip> {
        use crate::A11yPublicationSkip as Skip;
        if !self.s.a11y_owner.get() {
            Some(Skip::OwnerEnded)
        } else if self.s.a11y.is_none() {
            Some(Skip::NoAdapter)
        } else if self.s.runner.is_none() {
            Some(Skip::NoRunner)
        } else if self.s.layout.is_none() {
            Some(Skip::NoLayout)
        } else if self.s.native_receiving
            && (self.s.hidden
                || self.s.close_requested
                || !self.native_a11y_context_available()
                || !(self.s.displayed_presentation.is_some_and(|stamp| {
                    f64::from(stamp.layout_scale) == self.layout_scale()
                        && self.s.window.as_ref().is_some_and(|window| {
                            window.inner_size() == (stamp.width, stamp.height)
                        })
                }) && self.input_layout_is_displayed()))
        {
            Some(Skip::PendingPresentation)
        } else {
            None
        }
    }

    fn record_a11y_skip(&mut self, reason: crate::A11yPublicationSkip) {
        #[cfg(not(target_arch = "wasm32"))]
        let first_busy =
            reason == crate::A11yPublicationSkip::NoLayout && self.s.native_waiter.is_some();
        #[cfg(target_arch = "wasm32")]
        let first_busy = false;
        if (reason == crate::A11yPublicationSkip::PendingPresentation || first_busy)
            && !self.s.hidden
            && !self.s.close_requested
            && !self.s.needs_recovery_surface
        {
            if let Some(a11y) = self.s.a11y.as_mut() {
                a11y.prepare_native_window();
            }
        }
        let adapter = self
            .s
            .a11y
            .as_ref()
            .map_or(crate::A11yAdapterState::NotConfigured, |a11y| {
                a11y.adapter_state()
            });
        self.record_a11y_publication(crate::A11yPublication::skipped(adapter, reason));
    }

    fn sync_a11y_tree(&mut self) -> Vec<crate::A11yDispatch> {
        if let Some(reason) = self.a11y_publication_skip() {
            self.record_a11y_skip(reason);
            return Vec::new();
        }
        let layout_scale = self.layout_scale();
        let started = crate::Instant::now();
        let requests = {
            let runner = self
                .s
                .runner
                .as_ref()
                .expect("publication requires a runner");
            let (dom, mount, focus) = (runner.dom(), runner.mount(), runner.focus());
            let dom_ref = dom.borrow();
            let view = crate::WindowDom::new(&dom_ref, mount);
            let focus = focus.map(|node| view.opaque_id(node));
            self.s
                .a11y
                .as_mut()
                .expect("publication requires an adapter")
                .sync(
                    &crate::A11yOwner::new(&self.s.a11y_owner),
                    &view,
                    self.s.layout.as_ref().expect("publication requires layout"),
                    &self.s.shared.leaves,
                    &self.s.shared.producers,
                    focus,
                    layout_scale,
                )
        };
        let publish_us = elapsed_us(started.elapsed());
        self.record_a11y_publication(requests.publication);
        let started = crate::Instant::now();
        let results = self.apply_a11y_requests(&requests.requests);
        if let Some(profile) = self.s.last_frame_profile.as_mut() {
            profile.a11y_publish_us = profile.a11y_publish_us.saturating_add(publish_us);
            profile.a11y_actions_us = profile
                .a11y_actions_us
                .saturating_add(elapsed_us(started.elapsed()));
        }
        results
    }

    /// Route drained screen-reader requests into the retained DOM. Split out of
    /// [`sync_a11y`](Self::sync_a11y) so the routing is exercisable without an
    /// OS adapter.
    /// Results correspond to input order; dispatch is not terminal completion.
    pub fn apply_a11y_requests(
        &mut self,
        requests: &[crate::A11yRequest],
    ) -> Vec<crate::A11yDispatch> {
        use crate::A11yDispatch;
        let mut results = Vec::with_capacity(requests.len());
        let mut dispatched = false;
        for request in requests {
            // Raw calls name this operation's entry context. Freeze once before
            // contributor callbacks; bound publications retain their original life.
            let mut request = request.clone();
            if let Some(runner) = self.s.runner.as_ref() {
                let dom = runner.dom();
                let dom = dom.borrow();
                request = request.bind_publication(
                    &crate::A11yOwner::new(&self.s.a11y_owner),
                    &crate::WindowDom::new(&dom, runner.mount()),
                    &self.s.shared.leaves,
                    &self.s.shared.producers,
                );
            }
            // Recompute against the current DOM for each request: a previous
            // callback in this batch may have removed or disabled its target.
            let node = match self.admit_a11y_request(&request) {
                Ok(node) => node,
                Err(reason) => {
                    results.push(A11yDispatch::Rejected(reason));
                    continue;
                },
            };
            let runner = self.s.runner.as_mut().expect("admission requires a runner");
            let result = match &request.target {
                crate::A11yTarget::Produced(produced) => {
                    if request.action == A11yAction::Click {
                        match self.s.shared.producers.act_checked(
                            produced,
                            request.provenance.as_ref().and_then(|p| p.child()),
                        ) {
                            crate::ProducerActionDispatch::Dispatched => {
                                runner.set_focus(Some(node));
                                A11yDispatch::Dispatched
                            },
                            crate::ProducerActionDispatch::Declined => {
                                A11yDispatch::ProducerDeclined
                            },
                            crate::ProducerActionDispatch::StaleTarget => {
                                A11yDispatch::Rejected(crate::A11yRejection::StaleTarget)
                            },
                        }
                    } else {
                        runner.set_focus(Some(node));
                        A11yDispatch::Dispatched
                    }
                },
                crate::A11yTarget::Node(_) => {
                    match request.action {
                        A11yAction::Click => {
                            runner.dispatch_click(node, PointerClick::at((0.0, 0.0)));
                        },
                        A11yAction::Focus => runner.set_focus(Some(node)),
                        A11yAction::SetValue(value) => {
                            runner.dispatch_value(node, cambium::ValueEvent { value });
                        },
                    }
                    A11yDispatch::Dispatched
                },
            };
            dispatched |= result == A11yDispatch::Dispatched;
            results.push(result);
        }
        if dispatched {
            self.hover();
            self.after_dispatch();
        }
        results
    }

    /// Admission belongs to the host, not to a platform's last-synced ID map.
    /// Layout visibility is the retained host view. Bound publications also
    /// check host owner, document/mount and contributor lives. Raw requests name
    /// the current context. Neither form establishes an atomic presentation transaction.
    pub(super) fn admit_a11y_request(
        &mut self,
        request: &crate::A11yRequest,
    ) -> Result<crate::NodeId, crate::A11yRejection> {
        use crate::A11yRejection as Refusal;
        use layout_dom_api::LayoutDom as _;
        if !self.s.a11y_owner.get() {
            return Err(Refusal::Cancelled);
        }
        if self.s.hidden || self.s.close_requested || !self.native_a11y_context_available() {
            return Err(Refusal::Unavailable);
        }
        if matches!(request.action, A11yAction::SetValue(value) if !value.is_finite()) {
            return Err(Refusal::InvalidValue);
        }
        let (Some(runner), Some(layout)) = (self.s.runner.as_ref(), self.s.layout.as_ref()) else {
            return Err(Refusal::Unavailable);
        };
        let dom = runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, runner.mount());
        if !request.publication_is_current(
            &crate::A11yOwner::new(&self.s.a11y_owner),
            &view,
            &self.s.shared.leaves,
            &self.s.shared.producers,
        ) {
            return Err(Refusal::StaleTarget);
        }
        if let crate::A11yTarget::Node(node) = request.target {
            if !view.contains(node) {
                return Err(Refusal::TargetAbsent);
            }
        }
        let projection = crate::project_semantics(
            &view,
            layout,
            &mut self.s.shared.leaves,
            &mut self.s.shared.producers,
            runner.focus().map(|node| view.opaque_id(node)),
        );
        let node = match &request.target {
            crate::A11yTarget::Node(node) => *node,
            crate::A11yTarget::Produced(produced) => *projection
                .producer_slots
                .get(&produced.slot)
                .ok_or(Refusal::TargetAbsent)?,
        };
        let id = accesskit::NodeId(view.opaque_id(node));
        if !projection.dom_targets.contains_key(&id) {
            return Err(Refusal::TargetAbsent);
        }
        let semantic = projection
            .tree
            .nodes
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, node)| node)
            .filter(|node| !node.is_hidden())
            .ok_or(Refusal::TargetAbsent)?;
        if semantic.is_disabled() {
            return Err(Refusal::Disabled);
        }
        let action = match request.action {
            A11yAction::Click => accesskit::Action::Click,
            A11yAction::Focus => accesskit::Action::Focus,
            A11yAction::SetValue(_) => accesskit::Action::SetValue,
        };
        let semantic = if let crate::A11yTarget::Produced(produced) = &request.target {
            let id = projection
                .produced_targets
                .iter()
                .find_map(|(id, target)| (target == produced).then_some(*id))
                .ok_or(Refusal::Unsupported)?;
            projection
                .tree
                .nodes
                .iter()
                .find(|(candidate, _)| *candidate == id)
                .map(|(_, node)| node)
                .ok_or(Refusal::TargetAbsent)?
        } else {
            semantic
        };
        if !semantic.supports_action(action) {
            return Err(Refusal::Unsupported);
        }
        // A contributor callback is required to be observational. Recheck its
        // authored life even if interior mutation violated that contract.
        if !request.publication_is_current(
            &crate::A11yOwner::new(&self.s.a11y_owner),
            &view,
            &self.s.shared.leaves,
            &self.s.shared.producers,
        ) {
            return Err(Refusal::StaleTarget);
        }
        Ok(node)
    }
}
