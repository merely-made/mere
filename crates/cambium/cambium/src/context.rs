/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The view context for the Genet backend.
//!
//! It holds the `id_path` used for message routing, a shared handle to the
//! [`ScriptedDom`] every view mutates, the event listeners, focusability
//! markers, and the portable-child nursery.
//!
//! Click, key and focus listeners register in a [`genet_events::Registry`]
//! under [`CAMBIUM_OWNER`](crate::CAMBIUM_OWNER), beside any other owner's listeners on the same
//! nodes (see [`events`](crate::events)). Each has a propagation phase: a
//! listener registered with `capture == true` fires in the `root → target`
//! capture pass, one with `capture == false` (the browser/`xilem_web` default)
//! in the `target → root` bubble pass. A node may carry *several* listeners of
//! one kind (nested `on_click`s over one element, a handler beside an
//! instrumentation listener), and dispatch runs every one, in registration
//! order within the matching phase, rather than letting a later listener
//! silently clobber an earlier one.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::DomHandle;
use crate::events::{Kind, Listeners, NativeOwner};
use crate::pod::{GenetElement, GenetElementMut};
use genet_events::Registry;
use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom, LayoutDomMut};
use meristem::{View, ViewId, ViewPathTracker};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusRequest {
    Focus(NodeId),
    Blur(NodeId),
}

/// The [`ViewPathTracker`] context for all Genet views.
///
/// The context tracks the current Meristem view path, the shared DOM, and the
/// routing state built by event views. [`OnClick`](crate::OnClick) and
/// [`OnKey`](crate::OnKey) record their view paths against the DOM node they
/// wrap, as listeners in the event layer. [`GenetAppRunner`](crate::GenetAppRunner)
/// walks the hit or focused node's path through that layer and routes messages
/// down the retained view paths its listeners name.
///
/// Key handlers and explicit [`focusable`](crate::focusable) markers together
/// define the focus traversal set. Pointer, hover, and wheel handlers use the
/// same context for drag capture, hover transitions, and scroll routing.
pub struct GenetCtx {
    id_path: Vec<ViewId>,
    dom: DomHandle,
    /// Click, key and focus listeners: the `genet_events` registry (shared, so
    /// a host can hand one in) and the token table mapping each of Cambium's
    /// listeners to its routing view path.
    listeners: Listeners,
    /// Nodes marked focusable by an explicit [`focusable`](crate::focusable) view,
    /// independent of any key handler — so a plain `on_click` button (no `on_key`)
    /// can still be tab-reached and Enter/Space-activated. The `usize` is a
    /// refcount, so stacked markers and node-swap rebuilds stay balanced. Read
    /// (with the key listeners) by [`is_focusable`](Self::is_focusable).
    focusable: HashMap<NodeId, usize>,
    /// `NodeId → routing path` for pointer-drag handlers
    /// ([`OnPointer`](crate::OnPointer)). Unlike click/key there is no
    /// capture/bubble phase: a drag routes straight to the captured target, so
    /// the value is just the path (ending in `ON_POINTER_ID`). The runner walks a
    /// hit node's ancestor chain through this on `pointerdown` to find the
    /// element that captures the drag.
    pointer_handlers: HashMap<NodeId, Vec<ViewId>>,
    /// `NodeId → routing path` for accessible numeric value handlers.
    value_handlers: HashMap<NodeId, Vec<ViewId>>,
    /// `NodeId → routing path` for [`OpenFile`](crate::OpenFile) views, where
    /// a host's [`FileEvent`](crate::FileEvent) is dispatched.
    file_handlers: HashMap<NodeId, Vec<ViewId>>,
    /// `NodeId → routing path` for pointer-hover handlers
    /// ([`OnHover`](crate::OnHover)). Hover routes directly to the nearest
    /// registered ancestor selected by the host's hit-test transition.
    hover_handlers: HashMap<NodeId, Vec<ViewId>>,
    /// `NodeId → routing path` for wheel/scroll handlers
    /// ([`OnWheel`](crate::OnWheel)). Like pointer there is no capture/bubble
    /// phase: a wheel routes to the nearest scroll-handling ancestor of the hit
    /// node, so the value is just the path (ending in `ON_WHEEL_ID`). The runner
    /// walks the hit node's ancestor chain through this on a wheel event to find
    /// the element that handles the scroll.
    wheel_handlers: HashMap<NodeId, Vec<ViewId>>,
    /// One-shot programmatic focus requested by a view during build/rebuild.
    /// The runner consumes this after the DOM mutation pass, once the target is
    /// attached and its liveness can be checked.
    focus_request: Option<FocusRequest>,
    /// A file request a view filed during build/rebuild, for the runner to
    /// hand its host. The latest wins.
    file_request: Option<crate::FileRequest>,
    /// The portable-child nursery (moveBefore plan S5, cross-parent): children a
    /// [`PortableKeyed`](crate::PortableKeyed) parked because their key left its
    /// list, waiting within the same rebuild pass to be claimed by the sequence
    /// the key arrived in. Buckets are per concrete `(K, V)` instantiation,
    /// type-erased; the runner drains unclaimed children at the end of every
    /// rebuild ([`drain_nursery`](Self::drain_nursery)), tearing them down for
    /// real. A parked child's DOM node stays attached under its former parent
    /// until adoption moves it or the drain removes it.
    nursery: HashMap<TypeId, Box<dyn NurseryBucket>>,
}

/// A parked portable child: the previous view (an owned clone), its retained
/// view state, and its element — everything an adoption's rebuild or a drain's
/// teardown needs.
struct ParkedChild<V, VS> {
    view: V,
    state: VS,
    element: GenetElement,
}

/// One nursery bucket per concrete `(K, V, V::ViewState)` instantiation. The
/// monomorphized `teardown` fn pointer is captured at park time, so the bucket
/// itself needs no `View` bounds and the drain no type knowledge.
struct Bucket<K, V, VS> {
    parked: HashMap<K, ParkedChild<V, VS>>,
    teardown: fn(V, VS, GenetElement, &mut GenetCtx),
}

/// Type-erased nursery bucket: `Any` for the typed claim, plus the drain hook.
trait NurseryBucket {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn drain_teardowns(&mut self, ctx: &mut GenetCtx);
}

impl<K, V, VS> NurseryBucket for Bucket<K, V, VS>
where
    K: Eq + Hash + 'static,
    V: 'static,
    VS: 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn drain_teardowns(&mut self, ctx: &mut GenetCtx) {
        for (_key, parked) in self.parked.drain() {
            (self.teardown)(parked.view, parked.state, parked.element, ctx);
        }
    }
}

/// Tear down an unclaimed parked child for real: run the view teardown (which
/// unregisters its handlers by their stored paths), then remove the node — it
/// was left attached under its former parent by the park.
fn teardown_parked<State, Action, V>(
    view: V,
    mut state: V::ViewState,
    mut element: GenetElement,
    ctx: &mut GenetCtx,
) where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    let node = element.node;
    let parent = ctx.dom.borrow().parent(node);
    let el = GenetElementMut {
        node: &mut element.node,
        dom: ctx.dom.clone(),
        parent,
    };
    view.teardown(&mut state, ctx, el);
    ctx.dom.borrow_mut().remove(node);
}

impl GenetCtx {
    /// Create a context over an existing document handle, with an event
    /// registry of its own.
    pub fn new(dom: DomHandle) -> Self {
        Self::with_events(dom, Rc::default())
    }

    /// Create a context over an existing document handle whose listeners
    /// register in `events`, a registry the host shares with another owner (a
    /// script runtime's, say) so that both owners' listeners on a node
    /// interleave in registration order. Cambium's listeners are
    /// [`CAMBIUM_OWNER`](crate::CAMBIUM_OWNER)'s.
    pub fn with_events(dom: DomHandle, events: Rc<RefCell<Registry>>) -> Self {
        Self {
            id_path: Vec::new(),
            dom,
            listeners: Listeners::new(events),
            focusable: HashMap::new(),
            pointer_handlers: HashMap::new(),
            value_handlers: HashMap::new(),
            file_handlers: HashMap::new(),
            hover_handlers: HashMap::new(),
            wheel_handlers: HashMap::new(),
            focus_request: None,
            file_request: None,
            nursery: HashMap::new(),
        }
    }

    pub(crate) fn request_focus(&mut self, node: NodeId) {
        self.focus_request = Some(FocusRequest::Focus(node));
    }

    pub(crate) fn request_blur(&mut self, node: NodeId) {
        if !matches!(self.focus_request, Some(FocusRequest::Focus(_))) {
            self.focus_request = Some(FocusRequest::Blur(node));
        }
    }

    pub(crate) fn take_focus_request(&mut self) -> Option<FocusRequest> {
        self.focus_request.take()
    }

    /// File a request for the host's file chooser.
    pub fn request_file(&mut self, request: crate::FileRequest) {
        self.file_request = Some(request);
    }

    pub(crate) fn take_file_request(&mut self) -> Option<crate::FileRequest> {
        self.file_request.take()
    }

    /// Park a portable child whose key left its [`PortableKeyed`](crate::PortableKeyed)
    /// list: keep its (previous) view, view state, and element alive so the
    /// sequence its key arrives in — later in this same rebuild pass — can adopt
    /// it wholesale. The child's DOM node stays attached under its former parent
    /// until adoption moves it (one atomic `Moved`) or the end-of-rebuild
    /// [`drain_nursery`](Self::drain_nursery) tears it down. (moveBefore S5.)
    pub fn park_portable<State, Action, K, V>(
        &mut self,
        key: K,
        view: V,
        state: V::ViewState,
        element: GenetElement,
    ) where
        State: 'static,
        Action: 'static,
        K: Eq + Hash + 'static,
        V: View<State, Action, GenetCtx, Element = GenetElement> + 'static,
        V::ViewState: 'static,
    {
        let bucket = self
            .nursery
            .entry(TypeId::of::<Bucket<K, V, V::ViewState>>())
            .or_insert_with(|| {
                Box::new(Bucket::<K, V, V::ViewState> {
                    parked: HashMap::new(),
                    teardown: teardown_parked::<State, Action, V>,
                })
            });
        let bucket = bucket
            .as_any_mut()
            .downcast_mut::<Bucket<K, V, V::ViewState>>()
            .expect("nursery bucket keyed by its own TypeId");
        bucket.parked.insert(
            key,
            ParkedChild {
                view,
                state,
                element,
            },
        );
    }

    /// Claim a parked portable child by key, if one of this exact `(K, V)`
    /// instantiation was parked earlier in the current rebuild pass. Returns the
    /// previous view (for the rebuild diff), the retained view state, and the
    /// still-attached element.
    pub fn claim_portable<State, Action, K, V>(
        &mut self,
        key: &K,
    ) -> Option<(V, V::ViewState, GenetElement)>
    where
        State: 'static,
        Action: 'static,
        K: Eq + Hash + 'static,
        V: View<State, Action, GenetCtx, Element = GenetElement> + 'static,
        V::ViewState: 'static,
    {
        let bucket = self
            .nursery
            .get_mut(&TypeId::of::<Bucket<K, V, V::ViewState>>())?
            .as_any_mut()
            .downcast_mut::<Bucket<K, V, V::ViewState>>()?;
        let parked = bucket.parked.remove(key)?;
        Some((parked.view, parked.state, parked.element))
    }

    /// Tear down every still-parked child. The runner calls this at the end of
    /// each rebuild: a parked child not claimed within its own pass really is
    /// gone (its key left every portable list), so it gets an ordinary teardown
    /// and its node is removed. Loops until quiescent, since a teardown can
    /// itself park nested portable children.
    pub fn drain_nursery(&mut self) {
        while !self.nursery.is_empty() {
            let mut nursery = std::mem::take(&mut self.nursery);
            for bucket in nursery.values_mut() {
                bucket.drain_teardowns(self);
            }
        }
    }

    /// The document handle this context mutates.
    pub fn dom(&self) -> DomHandle {
        self.dom.clone()
    }

    /// The registry this context's listeners live in: its own unless the host
    /// handed one to [`with_events`](Self::with_events).
    pub fn events(&self) -> Rc<RefCell<Registry>> {
        self.listeners.events()
    }

    /// The endpoint for native calls that reach Cambium's owner from a dispatch
    /// the runner is not running. See [`NativeOwner`].
    pub fn native_owner(&self) -> NativeOwner {
        self.listeners.owner()
    }

    pub(crate) fn listeners(&self) -> &Listeners {
        &self.listeners
    }

    /// Register `path` (in phase `capture`) as *a* click handler for `node`,
    /// appended after any already there.
    ///
    /// Called by [`OnClick::build`](crate::OnClick) (and on rebuild when the
    /// wrapped node changes). `path` is the `view_path()` captured *inside* the
    /// handler's `with_id`, so it ends in `ON_CLICK_ID` and routes straight to
    /// the handler's `message`; `capture` is the per-listener phase
    /// ([`OnClick::capture`](crate::OnClick::capture), default `false` = bubble).
    /// Idempotent per `path`: re-registering the same path updates its phase in
    /// place rather than duplicating, so a redundant rebuild can't grow the list.
    pub fn register_click(&mut self, node: NodeId, path: Vec<ViewId>, capture: bool) {
        self.listeners.add(node, Kind::Click, path, capture, false);
    }

    /// Drop the click handler with `path` from `node` (teardown, or before a
    /// re-register onto a different node), leaving any sibling listeners.
    pub fn unregister_click(&mut self, node: NodeId, path: &[ViewId]) {
        self.listeners.remove(node, Kind::Click, path);
    }

    /// Register `path` (in phase `capture`) as *a* key handler for `node`.
    /// `focusable` controls whether this listener also puts the node in the
    /// focus traversal set. Appended after any already there.
    ///
    /// Called by [`OnKey::build`](crate::OnKey) (and on rebuild when the wrapped
    /// node changes). `path` is the `view_path()` captured *inside* the handler's
    /// `with_id`, so it ends in `ON_KEY_ID` and routes straight to the handler's
    /// `message`; `capture` is the per-listener phase
    /// ([`OnKey::capture`](crate::OnKey::capture), default `false` = bubble).
    /// Idempotent per `path`, like [`register_click`](Self::register_click).
    pub fn register_key(
        &mut self,
        node: NodeId,
        path: Vec<ViewId>,
        capture: bool,
        focusable: bool,
    ) {
        self.listeners
            .add(node, Kind::Key, path, capture, focusable);
    }

    /// Drop the key handler with `path` from `node`, leaving any siblings. When
    /// the last focusable key handler goes, `node` is no longer focusable via a
    /// handler (an explicit [`focusable`](crate::focusable) marker still counts).
    pub fn unregister_key(&mut self, node: NodeId, path: &[ViewId]) {
        self.listeners.remove(node, Kind::Key, path);
    }

    /// Whether `node` carries a click handler.
    pub(crate) fn has_click(&self, node: NodeId) -> bool {
        self.listeners.has(node, Kind::Click)
    }

    /// Whether `node` carries a key handler.
    pub(crate) fn has_key(&self, node: NodeId) -> bool {
        self.listeners.has(node, Kind::Key)
    }

    /// Mark `node` focusable explicitly (an [`focusable`](crate::focusable)
    /// marker), independent of any key handler. Refcounted, so stacked markers
    /// and node-swap rebuilds balance.
    pub fn register_focusable(&mut self, node: NodeId) {
        *self.focusable.entry(node).or_insert(0) += 1;
    }

    /// Drop one explicit focusable mark from `node` (teardown / re-key on node
    /// swap); the node stays focusable while any mark or key handler remains.
    pub fn unregister_focusable(&mut self, node: NodeId) {
        if let Some(count) = self.focusable.get_mut(&node) {
            *count -= 1;
            if *count == 0 {
                self.focusable.remove(&node);
            }
        }
    }

    /// Whether `node` is *focusable*: it carries a focusable key handler (in
    /// either phase) or an explicit [`focusable`](crate::focusable) marker.
    pub fn is_focusable(&self, node: NodeId) -> bool {
        self.listeners.has_focusable_key(node) || self.focusable.contains_key(&node)
    }

    /// Register the retained message path that observes focus for `node`,
    /// replacing any observer already there.
    pub fn register_focus(&mut self, node: NodeId, path: Vec<ViewId>) {
        self.listeners.remove_all(node, Kind::Focus);
        self.listeners.add(node, Kind::Focus, path, false, false);
    }

    /// Remove `node`'s focus observer during teardown or node replacement.
    pub fn unregister_focus(&mut self, node: NodeId) {
        self.listeners.remove_all(node, Kind::Focus);
    }

    /// Register `path` as the pointer-drag handler for `node`
    /// ([`OnPointer`](crate::OnPointer) build/rebuild). The path ends in
    /// `ON_POINTER_ID` and routes straight to the handler's `message`.
    pub fn register_pointer(&mut self, node: NodeId, path: Vec<ViewId>) {
        self.pointer_handlers.insert(node, path);
    }

    /// Drop the pointer handler for `node` (teardown / re-key on node swap).
    pub fn unregister_pointer(&mut self, node: NodeId) {
        self.pointer_handlers.remove(&node);
    }

    /// The pointer-drag routing path on `node`, if one is registered. The
    /// runner's `dispatch_pointer_down` walks the hit node's ancestors through
    /// this to find the drag-capturing element.
    pub fn pointer_handler(&self, node: NodeId) -> Option<&[ViewId]> {
        self.pointer_handlers.get(&node).map(Vec::as_slice)
    }

    /// Register `path` as the accessible numeric-value handler for `node`.
    pub fn register_value(&mut self, node: NodeId, path: Vec<ViewId>) {
        self.value_handlers.insert(node, path);
    }

    /// Drop the accessible numeric-value handler for `node`.
    pub fn unregister_value(&mut self, node: NodeId) {
        self.value_handlers.remove(&node);
    }

    /// Register `path` as where a file answer for `node` is dispatched.
    pub fn register_file(&mut self, node: NodeId, path: Vec<ViewId>) {
        self.file_handlers.insert(node, path);
    }

    /// Drop the file-answer route for `node`.
    pub fn unregister_file(&mut self, node: NodeId) {
        self.file_handlers.remove(&node);
    }

    /// The file-answer routing path on `node`, if registered.
    pub fn file_handler(&self, node: NodeId) -> Option<&[ViewId]> {
        self.file_handlers.get(&node).map(Vec::as_slice)
    }

    /// The accessible numeric-value routing path on `node`, if registered.
    pub fn value_handler(&self, node: NodeId) -> Option<&[ViewId]> {
        self.value_handlers.get(&node).map(Vec::as_slice)
    }

    /// Register `path` as the hover handler for `node`.
    pub fn register_hover(&mut self, node: NodeId, path: Vec<ViewId>) {
        self.hover_handlers.insert(node, path);
    }

    /// Drop the hover handler for `node` (teardown / re-key on node swap).
    pub fn unregister_hover(&mut self, node: NodeId) {
        self.hover_handlers.remove(&node);
    }

    /// The hover routing path on `node`, if one is registered.
    pub fn hover_handler(&self, node: NodeId) -> Option<&[ViewId]> {
        self.hover_handlers.get(&node).map(Vec::as_slice)
    }

    /// Register `path` as the wheel handler for `node`
    /// ([`OnWheel`](crate::OnWheel) build/rebuild). The path ends in
    /// `ON_WHEEL_ID` and routes straight to the handler's `message`.
    pub fn register_wheel(&mut self, node: NodeId, path: Vec<ViewId>) {
        self.wheel_handlers.insert(node, path);
    }

    /// Drop the wheel handler for `node` (teardown / re-key on node swap).
    pub fn unregister_wheel(&mut self, node: NodeId) {
        self.wheel_handlers.remove(&node);
    }

    /// The wheel routing path on `node`, if one is registered. The runner's
    /// `dispatch_wheel` walks the hit node's ancestors through this to find the
    /// scroll-handling element.
    pub fn wheel_handler(&self, node: NodeId) -> Option<&[ViewId]> {
        self.wheel_handlers.get(&node).map(Vec::as_slice)
    }
}

impl ViewPathTracker for GenetCtx {
    fn push_id(&mut self, id: ViewId) {
        self.id_path.push(id);
    }

    fn pop_id(&mut self) {
        self.id_path.pop();
    }

    fn view_path(&mut self) -> &[ViewId] {
        &self.id_path
    }
}
