// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Shared event-propagation state for the native dispatcher.
//
// The native dispatch path sends an event *by value* (a `Clone`) to each
// listener it routes to, so a handler cannot mutate one shared event struct the
// way the JS DOM dispatcher does (`event.stopPropagation()` flipping a field
// every later listener sees). To get the same semantics here, the cancellation
// flags live behind a shared `Rc<Cell<…>>`: cloning the event clones the
// *handle*, not the flags, so a handler that calls `stop_propagation()` /
// `prevent_default()` on the event it received is seen by the dispatch loop and
// by every other clone.
//
// The flags are the native twin of the JS `Event`'s stop / stop-immediate /
// canceled state, and `genet_events::dispatch` reads them back after each
// listener, so both owners satisfy one propagation/cancellation contract in one
// walk (genet `docs/2026-10-17_native_event_dispatch_plan.md`).

use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default)]
struct Flags {
    stop: bool,
    stop_immediate: bool,
    default_prevented: bool,
    key_target: Option<(genet_scripted_dom::NodeId, genet_scripted_dom::NodeId)>,
    tab_focus: Option<(
        genet_scripted_dom::NodeId,
        genet_scripted_dom::NodeId,
        usize,
    )>,
}

/// Shared, clone-through cancellation state for one dispatched event.
///
/// Embedded in the native event payloads ([`PointerClick`](crate::PointerClick),
/// [`KeyEvent`](crate::KeyEvent)). All clones of an event share one flag cell, so
/// a handler calling [`stop_propagation`](Self::stop_propagation) /
/// [`prevent_default`](Self::prevent_default) is observed by the dispatch loop
/// (which reads the flags back after each listener) and by the host
/// (which reads [`default_prevented`](Self::default_prevented) to decide whether
/// to run the event's default action).
#[derive(Clone, Debug)]
pub struct Propagation {
    flags: Rc<Cell<Flags>>,
}

impl Default for Propagation {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagation {
    // A stock-tab request belongs to one actual key-handler invocation. It is
    // separate from cancellation: custom/manual tabs can prevent arrows freely.
    pub(crate) fn set_key_target(
        &self,
        target: Option<(genet_scripted_dom::NodeId, genet_scripted_dom::NodeId)>,
    ) {
        let mut flags = self.flags.get();
        flags.key_target = target;
        self.flags.set(flags);
    }

    pub(crate) fn request_tab_focus(&self, index: usize) {
        let mut flags = self.flags.get();
        if let Some((origin, group)) = flags.key_target {
            flags.tab_focus = Some((origin, group, index));
        }
        self.flags.set(flags);
    }

    pub(crate) fn take_tab_focus(
        &self,
    ) -> Option<(
        genet_scripted_dom::NodeId,
        genet_scripted_dom::NodeId,
        usize,
    )> {
        let mut flags = self.flags.get();
        let request = flags.tab_focus.take();
        self.flags.set(flags);
        request
    }

    /// A fresh propagation state (nothing stopped or prevented). One per
    /// dispatch; the event's clones share it.
    pub fn new() -> Self {
        Self {
            flags: Rc::new(Cell::new(Flags::default())),
        }
    }

    /// Stop the event reaching any *later* node in the propagation path. The
    /// current node's other listeners still run, as in the DOM. Mirrors
    /// `Event.stopPropagation`.
    pub fn stop_propagation(&self) {
        let mut f = self.flags.get();
        f.stop = true;
        self.flags.set(f);
    }

    /// Stop the event reaching any later listener at all, including remaining
    /// listeners on the current node — and imply `stop_propagation`. Mirrors
    /// `Event.stopImmediatePropagation`.
    pub fn stop_immediate_propagation(&self) {
        let mut f = self.flags.get();
        f.stop = true;
        f.stop_immediate = true;
        self.flags.set(f);
    }

    /// Mark the event's default action as canceled. The host reads
    /// [`default_prevented`](Self::default_prevented) after dispatch to decide
    /// whether to run that default (form activation, drag start, caret move).
    /// Mirrors `Event.preventDefault` (the native event is always cancelable —
    /// Genet has no passive-listener distinction yet).
    pub fn prevent_default(&self) {
        let mut f = self.flags.get();
        f.default_prevented = true;
        self.flags.set(f);
    }

    /// Whether propagation has been stopped (`stop_propagation` or
    /// `stop_immediate_propagation`).
    pub fn stopped(&self) -> bool {
        self.flags.get().stop
    }

    /// Whether `stop_immediate_propagation` was called. The layer's dispatch
    /// reads it after each listener, to skip the rest of the node's list.
    pub(crate) fn stopped_immediately(&self) -> bool {
        self.flags.get().stop_immediate
    }

    /// Whether `prevent_default` was called. Read by the host after dispatch.
    pub fn default_prevented(&self) -> bool {
        self.flags.get().default_prevented
    }
}
