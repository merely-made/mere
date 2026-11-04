/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Frisket: a pane frame — splits and tab-stacks — as one composition.
//!
//! On a hand press the *frisket* is the hinged frame whose cut-out apertures
//! decide what prints where; this is the same frame, over a window. The name
//! comes from turnstone's crate, which this module retires: the family had spelled
//! "a tree of resizable panes" four times (this view trapped in `ports/pelt`, a
//! second tree in turnstone, the reusable `workbench` contract, the furniture here),
//! and they resolve to one implementation. Direction:
//! `genet:docs/2026-07-24_frisket_pane_component_direction.md`.
//!
//! **The tree is the caller's state**, like the split's ratio and the strip's
//! selection: [`frisket`] renders whatever tree the caller hands it and reports
//! gestures as [`TileEvent`]s for the caller to apply through
//! [`TileTree::apply`]. The component never mutates the tree, so one host can
//! hold a plain tree while another projects one from richer truth (mere's platen
//! projects a forme arrangement onto it) and both drive the same view.
//!
//! **No layout dependency, by design.** The parts here are the view and the DOM
//! *semantics*: which element is a divider, which is a tab, which tab bar
//! belongs to which stack. A host lays the DOM out and hit-tests it, then asks
//! [`divider_target`] / [`tab_target`] / [`stack_target`] what it hit. That
//! keeps the walking logic in one place — it was duplicated per host before —
//! while layout stays where the host's engine is. The one gesture that needs
//! real geometry, resolving a tab drop to an insertion index, takes the rects
//! through a closure ([`tab_drop_index`]).
//!
//! Content is a [`Slot`]. [`frisket`] draws a placeholder marked with the
//! active tile's id and nothing else, and the host composites a document scene,
//! an external texture, or its own surface into that rect. [`frisket_with`]
//! widens that to three: the same hole, a surface the host names as one, or a
//! child view the frame nests. So a browser pane, a practice-set pane, a
//! tactical map pane and a pane made of ordinary views are the same frame.
//!
//! A [`Slot::View`] child is rendered through a
//! [`PortableKeyed`](crate::PortableKeyed) keyed by [`TileId`], so dragging a
//! tile into another stack *moves* its element rather than rebuilding it: the
//! child keeps its DOM node, its view state and its handlers.
//!
//! The tab bar is [`tab_bar_view`](crate::tab_bar_view), the crate's one tab
//! bar, wearing Frisket's `frisket-*` names on top of the shared `tablist` /
//! `tab` / `tab-close` vocabulary so a product stylesheet written against
//! either keeps matching.

use std::marker::PhantomData;

use layout_dom_api::{LayoutDom, LocalName, Namespace};
use meristem::{MessageCtx, MessageResult, Mut, ViewMarker};
use workbench::{SplitAxis, TabStack, Tile, TileEvent, TileId, TilePath, TileTree};

use crate::pod::GenetElement;
use crate::{
    AnyView, GenetCtx, PortableKeyed, TabAccentColors, TabBar, TabBarNames, TabItem, TabMark, View,
    el, tab_bar_view,
};

/// The erased child-view type a pane frame nests: a split holds splits or
/// stacks, so the recursion needs one boxed type.
pub type PaneView<State, AppAction> = Box<dyn AnyView<State, AppAction, GenetCtx, GenetElement>>;

/// `data-divider`: the split path a divider resizes.
const ATTR_DIVIDER: &str = "data-divider";
/// `data-dindex`: which boundary within that split.
const ATTR_DINDEX: &str = "data-dindex";
/// `data-tabkey`: the tile a tab represents, in the shared tab-bar vocabulary.
const ATTR_TABKEY: &str = "data-tabkey";
/// `data-tabid`: the same value under Frisket's older name, kept because hosts
/// address tabs by it from their own code.
const ATTR_TABID: &str = "data-tabid";
/// `data-stack`: the path of the stack a tab bar belongs to.
const ATTR_STACK: &str = "data-stack";
/// `data-slot`: which of [`SlotKind`]'s three a content element carries.
const ATTR_SLOT: &str = "data-slot";

/// The names Frisket's bar wears on top of the shared `tablist` / `tab` /
/// `tab selected` / `tab-close` vocabulary. Products style `frisket-*`
/// directly, so the bar answers to both.
const FRISKET_TAB_NAMES: TabBarNames = TabBarNames {
    bar: Some("frisket-tabbar"),
    tab: Some("frisket-tab"),
    selected: Some("active"),
    close: Some("frisket-close"),
    label: Some("frisket-label"),
    key_alias: Some(ATTR_TABID),
};
/// `data-tile`: the active tile whose content area this is. A host finds this
/// element's rect to composite the tile's content into it.
pub const FRISKET_TILE_ATTR: &str = "data-tile";

/// The default structural stylesheet: flex skeleton, tab bar, divider, and the
/// content hole. A theme layers over it; nothing here is decorative beyond the
/// minimum needed to be usable unstyled.
pub const FRISKET_CSS: &str = "\
    .frisket-split { width: 100%; height: 100%; } \
    .frisket-branch { display: flex; } \
    .frisket-stack { width: 100%; height: 100%; } \
    .frisket-tabbar { display: flex; flex-grow: 0; flex-shrink: 0; flex-basis: 44px; align-items: stretch; height: 44px; padding: 4px 2px 0 2px; background: #33333a; } \
    .frisket-tab { display: flex; align-items: center; flex-grow: 1; flex-shrink: 1; flex-basis: 0px; min-width: 0; max-width: 360px; overflow: hidden; padding: 8px 10px; font-size: 15px; line-height: 1.2; color: #cccccc; background: #2a2a30; margin-right: 3px; } \
    .frisket-tab.active { color: #ffffff; background: #4a4a55; } \
    .frisket-label { flex-grow: 1; flex-shrink: 1; flex-basis: 0px; min-width: 0; overflow: hidden; white-space: nowrap; } \
    .frisket-close { flex-grow: 0; flex-shrink: 0; flex-basis: 28px; width: 28px; height: 28px; margin-left: 4px; padding: 4px 0; text-align: center; font-size: 15px; color: #999999; } \
    .frisket-content { flex-grow: 1; flex-shrink: 1; flex-basis: 0px; min-height: 0; background: #ffffff; } \
    .frisket-divider { flex-grow: 0; flex-shrink: 0; flex-basis: 10px; background: #1a1a1f; }";

/// Encode a split path (`[0, 1]`) as an attribute string (`"0.1"`); the root
/// split is the empty string.
pub fn encode_pane_path(path: &[usize]) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

/// Decode a `data-divider` / `data-stack` value back to a [`TilePath`].
pub fn decode_pane_path(s: &str) -> TilePath {
    if s.is_empty() {
        TilePath(Vec::new())
    } else {
        TilePath(s.split('.').filter_map(|p| p.parse().ok()).collect())
    }
}

/// What fills one stack's content element.
///
/// The frame draws the element either way; the difference is who owns what is
/// inside it. A [`Slot::View`] is a child view the frame nests and keeps alive
/// across a tile's move between stacks; [`Slot::Surface`] and [`Slot::Hole`]
/// leave it empty for a host to composite into, and differ only in what the
/// host reads back off the DOM ([`slot_kind`]).
pub enum Slot<State, AppAction> {
    /// A child view, rendered inside the content element.
    View(PaneView<State, AppAction>),
    /// Empty; the host composites its own surface into the element's rect.
    Surface,
    /// Empty and unclaimed — the original content hole.
    Hole,
}

/// Which of [`Slot`]'s three a content element carries, read back off the DOM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    View,
    Surface,
    Hole,
}

impl SlotKind {
    /// The `data-slot` value for this kind.
    fn attr_value(self) -> &'static str {
        match self {
            SlotKind::View => "view",
            SlotKind::Surface => "surface",
            SlotKind::Hole => "hole",
        }
    }

    fn from_attr(value: &str) -> Option<Self> {
        match value {
            "view" => Some(SlotKind::View),
            "surface" => Some(SlotKind::Surface),
            "hole" => Some(SlotKind::Hole),
            _ => None,
        }
    }
}

/// One stack's content child, rebuilt from its tile and the host's `fill` on
/// demand rather than held as a value.
///
/// [`PortableKeyed`] needs a `Clone` child view — the parked previous view is
/// what an adopting rebuild diffs against — and an erased `PaneView` is not
/// `Clone`. Cloning the *recipe* (the tile plus the fill closure) instead is
/// the `TaggedText` pattern, and it is what buys a tile its element and view
/// state across a drag into another stack.
struct SlotChild<State, AppAction, Fill> {
    tile: Tile,
    fill: Fill,
    marker: PhantomData<fn() -> (State, AppAction)>,
}

impl<State, AppAction, Fill> SlotChild<State, AppAction, Fill>
where
    State: 'static,
    AppAction: 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction>,
{
    fn new(tile: Tile, fill: Fill) -> Self {
        Self {
            tile,
            fill,
            marker: PhantomData,
        }
    }

    /// The view this child stands for. Only ever built for a tile whose slot is
    /// a [`Slot::View`]; the other arms cannot be reached from `render_stack`,
    /// and degrade to an empty element rather than a panic if `fill` is not a
    /// function of its tile alone.
    fn inner(&self) -> PaneView<State, AppAction> {
        match (self.fill)(&self.tile) {
            Slot::View(view) => view,
            Slot::Surface | Slot::Hole => Box::new(el::<_, State, AppAction>("div", ())),
        }
    }
}

impl<State, AppAction, Fill: Clone> Clone for SlotChild<State, AppAction, Fill> {
    fn clone(&self) -> Self {
        Self {
            tile: self.tile.clone(),
            fill: self.fill.clone(),
            marker: PhantomData,
        }
    }
}

impl<State, AppAction, Fill> ViewMarker for SlotChild<State, AppAction, Fill> {}

impl<State, AppAction, Fill> View<State, AppAction, GenetCtx> for SlotChild<State, AppAction, Fill>
where
    State: 'static,
    AppAction: 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + 'static,
{
    type Element = GenetElement;
    type ViewState = <PaneView<State, AppAction> as View<State, AppAction, GenetCtx>>::ViewState;

    fn build(&self, ctx: &mut GenetCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        self.inner().build(ctx, app_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        self.inner()
            .rebuild(&prev.inner(), view_state, ctx, element, app_state);
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
    ) {
        self.inner().teardown(view_state, ctx, element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<AppAction> {
        self.inner()
            .message(view_state, message, element, app_state)
    }
}

/// Render `tree` as a pane frame with every content element left a hole.
/// `on_event` receives every gesture the frame raises (a tab activated, a tab
/// closed); the caller applies it to its own authoritative tree.
///
/// The returned view fills its parent, so a host wraps it in whatever frame it
/// wants — a status bar under it, chrome above it.
///
/// [`frisket_with`] with a `fill` of [`Slot::Hole`], exactly.
pub fn frisket<State, AppAction, Ev>(
    tree: &TileTree,
    on_event: Ev,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
{
    frisket_with(tree, on_event, |_: &Tile| Slot::Hole)
}

/// Render `tree` as a pane frame, asking `fill` what each stack's active tile
/// puts in its content element.
///
/// A [`Slot::View`] child is rendered through a [`PortableKeyed`] keyed by
/// [`TileId`], so dragging a tile to another stack moves its element rather
/// than rebuilding it: the child keeps its DOM node, its view state and its
/// handlers across the one rebuild that relocates it.
pub fn frisket_with<State, AppAction, Ev, Fill>(
    tree: &TileTree,
    on_event: Ev,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    frisket_with_current(tree, None, on_event, fill)
}

/// [`frisket_with`], plus which tile the host holds current.
///
/// The stack that *contains* `current` draws its bar
/// [`with_current`](crate::TabBar::with_current) — `aria-current` on the
/// `tablist`, and a `current` class token — so a window of several stacks says
/// which one has the focus. `None` marks nothing, which is
/// [`frisket_with`] exactly. The current tile need not be its stack's active
/// tab; the mark is about the stack, not the tab.
pub fn frisket_with_current<State, AppAction, Ev, Fill>(
    tree: &TileTree,
    current: Option<TileId>,
    on_event: Ev,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    frisket_with_marks(tree, current, &|_| None, on_event, fill)
}

/// [`frisket_with_current`], plus a live [`TabMark`] for any tile the host
/// wants marked: unsaved changes, or something that needs attention.
///
/// `marks` is asked once per tile while the frame is built. Marks are the
/// host's current truth, not tree state, so nothing a layout persists carries
/// them. `&|_| None` is [`frisket_with_current`] exactly.
pub fn frisket_with_marks<State, AppAction, Ev, Fill>(
    tree: &TileTree,
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    on_event: Ev,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    frisket_with_tab_policy(
        tree,
        current,
        marks,
        crate::TabStripPolicy::Always,
        on_event,
        fill,
    )
}

/// The host chooses when each stack needs a strip; source arrangement and
/// durable tile identities remain the same under either presentation.
pub fn frisket_with_tab_policy<State, AppAction, Ev, Fill>(
    tree: &TileTree,
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    policy: crate::TabStripPolicy,
    on_event: Ev,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    el::<_, State, AppAction>(
        "div",
        render_node(tree, &[], current, marks, policy, &on_event, &fill),
    )
    .attr("class", "frisket-body")
    .attr(
        "style",
        "display: flex; width: 100%; height: 100%; min-height: 0;",
    )
}

/// What a divider press resolves to: the split it resizes and which boundary.
/// The host supplies the split's pixel extent from its own layout to turn a drag
/// delta into a fraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DividerTarget {
    pub path: TilePath,
    pub index: usize,
}

fn attr<D: LayoutDom>(dom: &D, node: D::NodeId, name: &str) -> Option<String> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
        .map(|value| value.to_string())
}

fn has_class<D: LayoutDom>(dom: &D, node: D::NodeId, class: &str) -> bool {
    attr(dom, node, "class")
        .is_some_and(|value| value.split_whitespace().any(|token| token == class))
}

/// Walk up from a hit node to the divider it belongs to, if any.
///
/// The host hit-tests its laid-out DOM and hands the node here rather than
/// re-implementing the walk; this is the duplication the module exists to end.
pub fn divider_target<D: LayoutDom>(dom: &D, hit: D::NodeId) -> Option<DividerTarget> {
    let mut node = hit;
    loop {
        if let Some(path) = attr(dom, node, ATTR_DIVIDER) {
            return Some(DividerTarget {
                path: decode_pane_path(&path),
                index: attr(dom, node, ATTR_DINDEX)?.parse().ok()?,
            });
        }
        node = dom.parent(node)?;
    }
}

/// Walk up from a hit node to the tab it belongs to. A press on the close ×
/// resolves to `None`, because that press is a close and not the start of a tab
/// drag.
pub fn tab_target<D: LayoutDom>(dom: &D, hit: D::NodeId) -> Option<TileId> {
    if has_class(dom, hit, "tab-close") {
        return None;
    }
    let mut node = hit;
    loop {
        if let Some(id) = attr(dom, node, ATTR_TABKEY).and_then(|s| s.parse::<u64>().ok()) {
            return Some(TileId(id));
        }
        node = dom.parent(node)?;
    }
}

/// Resolve a press on a tab's close affordance to the tile it closes.
/// Returns `None` for every other descendant of the tab.
pub fn close_target<D: LayoutDom>(dom: &D, hit: D::NodeId) -> Option<TileId> {
    if !has_class(dom, hit, "tab-close") {
        return None;
    }
    let mut node = dom.parent(hit)?;
    loop {
        if let Some(id) = attr(dom, node, ATTR_TABKEY).and_then(|s| s.parse::<u64>().ok()) {
            return Some(TileId(id));
        }
        node = dom.parent(node)?;
    }
}

/// Walk up from a content descendant to the active tile whose Frisket hole
/// contains it.
pub fn content_target<D: LayoutDom>(dom: &D, hit: D::NodeId) -> Option<TileId> {
    let mut node = hit;
    loop {
        if let Some(id) = attr(dom, node, FRISKET_TILE_ATTR).and_then(|s| s.parse::<u64>().ok()) {
            return Some(TileId(id));
        }
        node = dom.parent(node)?;
    }
}

/// Walk up from a content descendant to which slot its Frisket content element
/// carries: a host composites into [`SlotKind::Surface`] and
/// [`SlotKind::Hole`] elements and leaves [`SlotKind::View`] ones to the frame.
pub fn slot_kind<D: LayoutDom>(dom: &D, hit: D::NodeId) -> Option<SlotKind> {
    let mut node = hit;
    loop {
        if let Some(value) = attr(dom, node, ATTR_SLOT) {
            return SlotKind::from_attr(&value);
        }
        node = dom.parent(node)?;
    }
}

/// Walk up from a hit node to the stack whose tab bar it is in.
pub fn stack_target<D: LayoutDom>(dom: &D, hit: D::NodeId) -> Option<TilePath> {
    let mut node = hit;
    loop {
        if let Some(path) = attr(dom, node, ATTR_STACK) {
            return Some(decode_pane_path(&path));
        }
        node = dom.parent(node)?;
    }
}

/// Where a tab dropped at `x` would land in the tab bar under `hit`: the stack's
/// path and the insertion index, counting the tabs whose horizontal centre sits
/// left of the cursor.
///
/// The one gesture that needs real geometry, so the rects arrive through
/// `rect_of` (`(x, y, w, h)`, host space) instead of this module growing a
/// layout dependency.
pub fn tab_drop_index<D: LayoutDom>(
    dom: &D,
    hit: D::NodeId,
    x: f32,
    rect_of: impl Fn(D::NodeId) -> Option<(f32, f32, f32, f32)>,
) -> Option<(TilePath, usize)> {
    let mut node = hit;
    let bar = loop {
        if attr(dom, node, ATTR_STACK).is_some() {
            break node;
        }
        node = dom.parent(node)?;
    };
    let path = decode_pane_path(&attr(dom, bar, ATTR_STACK)?);
    let mut before = 0usize;
    let mut stack = vec![bar];
    while let Some(current) = stack.pop() {
        if attr(dom, current, ATTR_TABKEY).is_some() {
            if let Some((rx, _, rw, _)) = rect_of(current) {
                if rx + rw / 2.0 < x {
                    before += 1;
                }
            }
        }
        for child in dom.dom_children(current) {
            stack.push(child);
        }
    }
    Some((path, before))
}

#[cfg(test)]
#[path = "frisket/tests.rs"]
mod tests;

#[path = "frisket/rendering.rs"]
mod rendering;

use rendering::render_node;

pub(super) use rendering::render_stack;
