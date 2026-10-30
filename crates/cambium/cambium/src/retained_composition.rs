// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A flat retained owner table for composition children inside one pane runner.
//!
//! Authored placement is only a presentation tree. Each component's erased
//! view, state and DOM element live in the table and survive wrapper rebuilds.
//! This view never creates a runner or mount per component.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::marker::PhantomData;
use std::rc::Rc;

use genet_scripted_dom::NodeId;
use meristem::{AnyView, View};

const MAX_LIFECYCLE_EVENTS: usize = 256;
use crate::composition::{
    ComponentDescriptor, ComponentKey, Composition, CompositionTransaction, ContainerKey,
    ContainerKind, PlacementLocation,
};
use crate::{GenetCtx, GenetElement};

mod layout;
mod placement;
#[cfg(test)]
mod tests;
mod view;

const ROOT_CONTAINER: ContainerKey = ContainerKey {
    kind: ContainerKind::Root,
    key: String::new(),
};

pub type ErasedComponentView<State: 'static, Action: 'static> =
    Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;
pub type RetainedComponentFactory<State: 'static, Action: 'static> =
    Rc<dyn Fn(&ComponentDescriptor) -> ErasedComponentView<State, Action>>;
pub(super) type ErasedComponentState<State: 'static, Action: 'static> =
    <dyn AnyView<State, Action, GenetCtx, GenetElement> as View<State, Action, GenetCtx>>::ViewState;

/// Stable routing identity for a live table entry at one placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentRoute {
    /// Table slot; it is never persisted as the authored component key.
    pub slot: u64,
    /// Changes when the binding/kind incarnation or route is replaced.
    pub route_generation: u64,
    /// Changes when placement or visibility custody changes.
    pub placement_generation: u64,
}

/// Local, bounded lifecycle evidence. It deliberately excludes config and
/// binding payloads; a product collector may project it into its own schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleEvent {
    Created {
        key: ComponentKey,
        route: ComponentRoute,
    },
    Moved {
        key: ComponentKey,
        route: ComponentRoute,
    },
    Hidden {
        key: ComponentKey,
        route: ComponentRoute,
    },
    Revealed {
        key: ComponentKey,
        route: ComponentRoute,
    },
    Replaced {
        key: ComponentKey,
        route: ComponentRoute,
    },
    Disposed {
        key: ComponentKey,
        route: ComponentRoute,
    },
    Refused {
        reason: LifecycleRefusal,
    },
}

/// Why an admitted candidate could not be committed to the retained DOM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleRefusal {
    ForeignDocument,
    ActiveInteraction,
    FocusedRetirement,
    RouteGenerationExhausted,
    PlacementGenerationExhausted,
    SlotExhausted,
}

/// Bounded local sink for P2 lifecycle evidence.
#[derive(Default)]
struct LifecycleState {
    events: VecDeque<LifecycleEvent>,
    routes: BTreeMap<ComponentKey, ComponentRouteSnapshot>,
    last_refusal: Option<LifecycleRefusal>,
    dropped_events: u64,
}

#[derive(Clone, Default)]
pub struct LifecycleEventSink(Rc<RefCell<LifecycleState>>);

impl LifecycleEventSink {
    /// Drain events in occurrence order.
    pub fn drain(&self) -> Vec<LifecycleEvent> {
        self.0.borrow_mut().events.drain(..).collect()
    }

    /// Current route for one authored component key.
    pub fn route(&self, key: &ComponentKey) -> Option<ComponentRouteSnapshot> {
        self.0.borrow().routes.get(key).cloned()
    }

    /// Current route table. A hidden entry remains present with `visible=false`.
    pub fn routes(&self) -> Vec<ComponentRouteSnapshot> {
        self.0.borrow().routes.values().cloned().collect()
    }

    /// Last fail-closed reconciliation refusal, if any.
    pub fn last_refusal(&self) -> Option<LifecycleRefusal> {
        self.0.borrow().last_refusal
    }

    /// Number of lifecycle events evicted from the bounded queue.
    pub fn dropped_events(&self) -> u64 {
        self.0.borrow().dropped_events
    }

    fn push(&self, event: LifecycleEvent) {
        let mut queue = self.0.borrow_mut();
        if queue.events.len() == MAX_LIFECYCLE_EVENTS {
            queue.events.pop_front();
            queue.dropped_events = queue.dropped_events.saturating_add(1);
        }
        queue.events.push_back(event);
    }

    fn refuse(&self, reason: LifecycleRefusal) {
        self.0.borrow_mut().last_refusal = Some(reason);
        self.push(LifecycleEvent::Refused { reason });
    }

    fn clear_refusal(&self) {
        self.0.borrow_mut().last_refusal = None;
    }

    fn publish_route(&self, key: &ComponentKey, route: ComponentRoute, visible: bool) {
        self.0.borrow_mut().routes.insert(
            key.clone(),
            ComponentRouteSnapshot {
                key: key.clone(),
                route,
                visible,
            },
        );
    }

    fn forget_route(&self, key: &ComponentKey) {
        self.0.borrow_mut().routes.remove(key);
    }
}

/// One retained component entry's current route, if the accepted model owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRouteSnapshot {
    pub key: ComponentKey,
    pub route: ComponentRoute,
    pub visible: bool,
}

/// Immutable view props built from an admitted composition transaction.
pub struct RetainedComposition<State: 'static, Action: 'static> {
    model: Composition,
    factory: RetainedComponentFactory<State, Action>,
    collapsed: BTreeSet<ContainerKey>,
    lifecycle: LifecycleEventSink,
    marker: PhantomData<fn(State) -> Action>,
}

impl<State: 'static, Action: 'static> Clone for RetainedComposition<State, Action> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            factory: self.factory.clone(),
            collapsed: self.collapsed.clone(),
            lifecycle: self.lifecycle.clone(),
            marker: PhantomData,
        }
    }
}

impl<State: 'static, Action: 'static> RetainedComposition<State, Action> {
    /// Create a view from the already admitted snapshot. All product binding
    /// validation belongs to the transaction's `CompositionValidator`.
    pub fn new(
        transaction: &CompositionTransaction,
        factory: RetainedComponentFactory<State, Action>,
    ) -> Self {
        Self {
            model: transaction.accepted().clone(),
            factory,
            collapsed: BTreeSet::new(),
            lifecycle: LifecycleEventSink::default(),
            marker: PhantomData,
        }
    }

    /// Start with groups/drawers collapsed. Their children remain live and
    /// parked until they are shown again.
    pub fn with_collapsed(mut self, collapsed: BTreeSet<ContainerKey>) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Share the bounded lifecycle trace sink across successive view props.
    pub fn with_lifecycle_sink(mut self, sink: LifecycleEventSink) -> Self {
        self.lifecycle = sink;
        self
    }

    /// Read current routes after construction/rebuild. Messages are validated
    /// against the route generation and placement generation internally.
    pub fn lifecycle_sink(&self) -> LifecycleEventSink {
        self.lifecycle.clone()
    }
}

pub(super) struct LiveChild<State: 'static, Action: 'static> {
    pub(super) view: ErasedComponentView<State, Action>,
    pub(super) view_state: ErasedComponentState<State, Action>,
    pub(super) element: GenetElement,
}

pub(super) struct Entry<State: 'static, Action: 'static> {
    pub(super) slot: u64,
    pub(super) route_generation: u64,
    pub(super) placement_generation: u64,
    pub(super) descriptor: ComponentDescriptor,
    pub(super) child: Option<LiveChild<State, Action>>,
    pub(super) container: ContainerKey,
    pub(super) location: PlacementLocation,
    pub(super) location_signature: Vec<(ContainerKey, usize)>,
    pub(super) hidden: bool,
}

impl<State: 'static, Action: 'static> Entry<State, Action> {
    pub(super) fn route(&self) -> ComponentRoute {
        ComponentRoute {
            slot: self.slot,
            route_generation: self.route_generation,
            placement_generation: self.placement_generation,
        }
    }
}

#[doc(hidden)]
pub struct RetainedState<State: 'static, Action: 'static> {
    pub(super) dom: crate::DomHandle,
    pub(super) parking: NodeId,
    pub(super) entries: BTreeMap<ComponentKey, Entry<State, Action>>,
    pub(super) containers: BTreeMap<ContainerKey, NodeId>,
    pub(super) next_slot: u64,
}
