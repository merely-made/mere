// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Placement-owned semantic observations over a controller with one live layout.
//! Engine revisions remain authoritative. Only pool-owned layout transitions
//! may be restored, and dispatch must prove complete projection equality.
//! This establishes the engine's projection contract, not a separate source epoch.
use crate::{PeltControllerPool, PeltHostEffect, PeltInput, PeltSessionIdentity};
use inker::{
    DocumentA11yAction, DocumentA11yActionData, DocumentA11yActionRequest, DocumentA11yProjection,
    SessionButtonState, SessionInput, SessionModifiers, SessionPointerButton,
};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// An owned published observation. Its request revision remains the captured
/// engine revision even while another placement temporarily owns live layout.
/// Fields are private so callers cannot manufacture or renew an admission.
#[derive(Clone, Debug, PartialEq)]
pub struct PeltPlacementA11y<K, P> {
    owner: u64,
    content: K,
    placement: P,
    generation: u64,
    identity: PeltSessionIdentity,
    publication: u64,
    projection: DocumentA11yProjection,
}
impl<K, P> PeltPlacementA11y<K, P> {
    pub fn projection(&self) -> &DocumentA11yProjection {
        &self.projection
    }
    pub fn identity(&self) -> PeltSessionIdentity {
        self.identity
    }
    pub fn placement(&self) -> &P {
        &self.placement
    }
    pub fn content(&self) -> &K {
        &self.content
    }
    pub fn placement_generation(&self) -> u64 {
        self.generation
    }
    pub fn publication(&self) -> u64 {
        self.publication
    }
}

pub(crate) struct PoolA11yState<K, P> {
    owner: u64,
    baseline: HashMap<K, (PeltSessionIdentity, u64)>,
    snapshots: HashMap<P, PeltPlacementA11y<K, P>>,
    publication: u64,
}
impl<K, P> Default for PoolA11yState<K, P> {
    fn default() -> Self {
        Self {
            owner: NEXT_OWNER
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("a11y pool owners exhausted"),
            baseline: HashMap::new(),
            snapshots: HashMap::new(),
            publication: 0,
        }
    }
}

fn same_projection(a: &DocumentA11yProjection, b: &DocumentA11yProjection) -> bool {
    a.root() == b.root() && a.support() == b.support() && a.nodes() == b.nodes()
}

impl<K: Eq + Hash + Clone, P: Eq + Hash + Clone, F: 'static> PeltControllerPool<K, P, F> {
    fn live_a11y(&self, key: &K) -> Option<(PeltSessionIdentity, DocumentA11yProjection)> {
        let controller = self.content(key)?.document()?;
        Some((
            controller.session_identity(),
            controller.accessibility_projection()?,
        ))
    }

    /// Observe before pool layout and on both sides of a pump. A revision
    /// change not accounted for by the pool's own layout retires every old
    /// observation of that content, even if values have since returned to A.
    pub(crate) fn a11y_observe(&mut self, key: &K) {
        let current = self
            .live_a11y(key)
            .map(|(identity, projection)| (identity, projection.revision()));
        if self.a11y.baseline.get(key).copied() != current {
            self.a11y_invalidate(key);
        }
        if let Some(current) = current {
            self.a11y.baseline.insert(key.clone(), current);
        }
    }
    /// Pool layout is a known transition, not proof that semantics stayed
    /// unchanged. Each candidate still has to prove full equality at dispatch.
    pub(crate) fn a11y_after_layout(&mut self, key: &K) {
        if let Some((identity, projection)) = self.live_a11y(key) {
            self.a11y
                .baseline
                .insert(key.clone(), (identity, projection.revision()));
        } else {
            self.a11y_invalidate(key);
        }
    }
    pub(crate) fn a11y_invalidate(&mut self, key: &K) {
        self.a11y.baseline.remove(key);
        self.a11y
            .snapshots
            .retain(|_, snapshot| &snapshot.content != key);
    }

    /// Capture semantics after preparing this exact visible placement. An
    /// unchanged complete placement snapshot keeps its publication identity,
    /// including when its layout is restored after another window's frame.
    pub fn accessibility_projection(&mut self, id: &P) -> Option<PeltPlacementA11y<K, P>> {
        self.prepare(id).ok()?;
        let content = self.placement(id)?.content.clone();
        let generation = self.placement_generation(id)?;
        let (identity, projection) = self.live_a11y(&content)?;
        if let Some(previous) = self.a11y.snapshots.get(id).filter(|previous| {
            previous.content == content
                && previous.identity == identity
                && previous.generation == generation
                && same_projection(&previous.projection, &projection)
        }) {
            return Some(previous.clone());
        }
        self.a11y.publication = self
            .a11y
            .publication
            .checked_add(1)
            .expect("a11y publications exhausted");
        let snapshot = PeltPlacementA11y {
            owner: self.a11y.owner,
            content,
            placement: id.clone(),
            generation,
            identity,
            publication: self.a11y.publication,
            projection,
        };
        self.a11y.snapshots.insert(id.clone(), snapshot.clone());
        Some(snapshot)
    }

    /// Validate the captured publication and dispatch atomically. The request
    /// revision must equal the captured revision. Only an exact restored full
    /// snapshot permits translation to the live engine revision, and the
    /// translated request never leaves this call.
    pub fn dispatch_accessibility_action(
        &mut self,
        snapshot: &PeltPlacementA11y<K, P>,
        request: &DocumentA11yActionRequest,
    ) -> PeltHostEffect {
        let refused = PeltHostEffect::default;
        if snapshot.owner != self.a11y.owner
            || request.revision != snapshot.projection.revision()
            || !snapshot
                .projection
                .node(request.target)
                .is_some_and(|node| {
                    !node.state.hidden
                        && !node.state.disabled
                        && node.actions.contains(&request.action)
                })
            || (request.action == DocumentA11yAction::SetValue
                && !matches!(&request.data, Some(DocumentA11yActionData::Value(_))))
        {
            return refused();
        }
        // This check precedes prepare: a source change cannot be laundered
        // into a pool-owned layout transition by restoring the target first.
        self.a11y_observe(&snapshot.content);
        if self.a11y.snapshots.get(&snapshot.placement) != Some(snapshot)
            || !self.admits(&snapshot.placement, snapshot.generation)
            || self
                .placement(&snapshot.placement)
                .is_none_or(|p| p.content != snapshot.content)
        {
            return refused();
        }
        if self.prepare(&snapshot.placement).is_err() {
            return refused();
        }
        let Some((identity, current)) = self.live_a11y(&snapshot.content) else {
            self.a11y_invalidate(&snapshot.content);
            return refused();
        };
        if identity != snapshot.identity
            || self.a11y.snapshots.get(&snapshot.placement) != Some(snapshot)
            || !same_projection(&current, &snapshot.projection)
        {
            self.a11y_invalidate(&snapshot.content);
            return refused();
        }
        if request.action == DocumentA11yAction::Click {
            if self.has_pointer_capture() {
                return refused();
            }
            let Some(target) = self
                .content(&snapshot.content)
                .and_then(|c| c.document())
                .and_then(|c| c.accessibility_click_target(request.target))
                .filter(|t| t.revision == current.revision())
            else {
                return refused();
            };
            let rect = self
                .placement(&snapshot.placement)
                .expect("placement admitted above")
                .rect;
            if !target.point.x.is_finite()
                || !target.point.y.is_finite()
                || target.point.x < 0.
                || target.point.y < 0.
                || target.point.x >= rect.width
                || target.point.y >= rect.height
            {
                return refused();
            }
            let mut effect = refused();
            for state in [SessionButtonState::Pressed, SessionButtonState::Released] {
                if self
                    .content(&snapshot.content)
                    .and_then(|c| c.document())
                    .is_none_or(|c| c.session_identity() != snapshot.identity)
                {
                    break;
                }
                let next = self.input(
                    &snapshot.placement,
                    PeltInput::Session(SessionInput::PointerButton {
                        x: target.point.x,
                        y: target.point.y,
                        button: SessionPointerButton::Primary,
                        state,
                        modifiers: SessionModifiers::default(),
                    }),
                );
                effect = merge(effect, next);
            }
            self.a11y_invalidate(&snapshot.content);
            return effect;
        }
        let live_request = DocumentA11yActionRequest {
            revision: current.revision(),
            ..request.clone()
        };
        // Unrestricted mutable access intentionally retires every candidate,
        // after all admission checks and immediately before owned delivery.
        let handled = self
            .content_mut(&snapshot.content)
            .and_then(|c| c.document_mut())
            .is_some_and(|controller| controller.dispatch_accessibility_action(&live_request));
        PeltHostEffect {
            handled,
            redraw: handled,
            ..Default::default()
        }
    }
}

fn merge(first: PeltHostEffect, last: PeltHostEffect) -> PeltHostEffect {
    PeltHostEffect {
        handled: first.handled || last.handled,
        redraw: first.redraw || last.redraw,
        cursor: last.cursor.or(first.cursor),
        pointer_capture: last.pointer_capture.or(first.pointer_capture),
        editable: last.editable,
        navigated: first.navigated || last.navigated,
        error: last.error.or(first.error),
        download: last.download.or(first.download),
        reroute: last.reroute.or(first.reroute),
        navigation: last.navigation.or(first.navigation),
        submission: last.submission.or(first.submission),
    }
}
