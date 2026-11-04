/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Workspace: a whole [`workbench::Workspace`] — the tiled frame plus the
//! float layer — as one composition, for a host that owns the model and
//! composites foreign content itself.
//!
//! [`workspace_view`] is [`frisket_with_current`](crate::frisket_with_current)
//! for the tree, and one absolutely-positioned element per visible float over
//! it. Geometry stays proportional: a float's [`RelativeRect`] becomes CSS
//! percentages and its constraints become `min-*`/`max-*` pixels, so the
//! proportion is resolved by the host's style engine and this crate still
//! carries no layout dependency.
//!
//! # The three membranes
//!
//! A pane's state sits at exactly one of three depths, and which one it is
//! decides what survives a move.
//!
//! 1. **The host's app state, keyed by [`TileId`].** Where an in-house pane's
//!    *durable* state belongs — what the pane is showing, its scroll seat, its
//!    draft. The host lenses it into the child view it hands back from `fill`.
//!    It is the only depth that survives a tile moving to another projection
//!    (another window), because it is not view state at all.
//! 2. **[`Component`](crate::Component).** Widget-local ephemera the parent has
//!    no opinion about — a menu's open flag, a field's caret. It lives in the
//!    view tree, so it survives a rebuild, and a move within one projection
//!    (see 3), and nothing beyond that.
//! 3. **[`PortableKeyed`](crate::PortableKeyed), keyed by [`TileId`].** How the
//!    frame nests a [`Slot::View`] child: dragging a tile between stacks *moves*
//!    its element rather than rebuilding it, so the child keeps its DOM node and
//!    its view state — including any `Component` local state under it — across
//!    that move. This holds **within one projection only**.
//!
//! So: a tile moved to a second projection arrives with a fresh element and no
//! view state. That is not a loss to repair here — the host re-renders it from
//! its own keyed state in the new window, which is why durable pane state lives
//! at depth 1. The float layer is inline rather than portable (a float's child
//! is built directly, not through `PortableKeyed`), so moving a tile between
//! the tree and the float layer also rebuilds its child.

use layout_dom_api::{LayoutDom, LocalName, Namespace};
use workbench::{
    FloatSizeConstraints, FloatingTile, PresentationEvent, RelativeRect, Tile, TileEvent, TileId,
    WorkbenchPresentation, Workspace, WorkspaceEvent,
};

use crate::pod::GenetElement;
use crate::{
    FRISKET_TILE_ATTR, GenetCtx, PaneView, Slot, SlotKind, TabAccentColors, TabBar, TabBarNames,
    TabItem, TabMark, View, el, frisket_presented_with, frisket_with_marks, slot_kind,
    tab_bar_view,
};

/// `data-slot`: which of [`SlotKind`]'s three a content element carries. The
/// same mark the frame writes, read back by [`slot_kind`].
const ATTR_SLOT: &str = "data-slot";

/// The names a float's bar wears, matching the frame's, so one product
/// stylesheet dresses both. (Frisket's own copy is private to that module; the
/// tests here pin the two together through the shared readers.)
const FLOAT_TAB_NAMES: TabBarNames = TabBarNames {
    bar: Some("frisket-tabbar"),
    tab: Some("frisket-tab"),
    selected: Some("active"),
    close: Some("frisket-close"),
    label: Some("frisket-label"),
    key_alias: Some("data-tabid"),
};

/// The minimal structural stylesheet for the workspace's own classes: the
/// float layer's overlay box, and a float's clip. Everything else a float
/// needs is inline, because it is per-float geometry. Layers over
/// [`FRISKET_CSS`](crate::FRISKET_CSS); adds no colour of its own.
pub const WORKSPACE_CSS: &str = "\
    .workspace-floats { position: absolute; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none; } \
    .workspace-float { overflow: hidden; pointer-events: auto; }";

#[path = "workspace_chrome.rs"]
mod chrome;
pub use chrome::{INSTRUMENT_CSS, WorkspacePaneStatus, workspace_pane_header};

/// What [`workspace_view`] draws: the workspace, which tile the host holds
/// current, and whether the float layer is showing.
///
/// Borrowed rather than owned — the host's model is the truth, and this view
/// reads it for the length of one render.
pub struct WorkspaceModel<'a> {
    pub workspace: &'a Workspace,
    /// The host's current tile. Its stack (or its float) draws a current bar.
    pub current: Option<TileId>,
    /// When false only pinned floats are drawn — [`Workspace::visible_floating`]
    /// applies the rule; this view just asks it.
    pub float_layer_visible: bool,
}

/// Render a whole workspace: the tiled frame, then the float layer over it.
///
/// `on_event` receives every gesture either station raises, already in the
/// workspace's vocabulary — the frame's [`TileEvent`]s arrive wrapped as
/// [`WorkspaceEvent::Tile`] — for the caller to apply through
/// [`Workspace::apply`]. Nothing here mutates the model. `fill` answers what a
/// tile's content element holds, the same three [`Slot`]s the frame uses, and
/// is asked the same way at both stations.
///
/// A float's bar activates and closes through the tile vocabulary rather than a
/// float one, because [`Workspace::apply`] already reads those two float-aware:
/// activating a float raises it, closing one removes it.
pub fn workspace_view<State, AppAction, Ev, Fill>(
    model: &WorkspaceModel<'_>,
    on_event: Ev,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, WorkspaceEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    workspace_view_with_marks(model, &|_| None, on_event, fill)
}

/// [`workspace_view`], plus a live [`TabMark`] for any tile the host wants
/// marked, on the frame and on a float's bar alike. See
/// [`frisket_with_marks`](crate::frisket_with_marks).
pub fn workspace_view_with_marks<State, AppAction, Ev, Fill>(
    model: &WorkspaceModel<'_>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    on_event: Ev,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, WorkspaceEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    let tile_event = on_event.clone();
    let frame = frisket_with_marks(
        model.workspace.tiled(),
        model.current,
        marks,
        move |state: &mut State, event: TileEvent| tile_event(state, WorkspaceEvent::Tile(event)),
        fill.clone(),
    );
    let floats: Vec<PaneView<State, AppAction>> = model
        .workspace
        .visible_floating(model.float_layer_visible)
        .into_iter()
        .map(|float| render_float(float, model.current, marks, &on_event, &fill))
        .collect();
    let layer = el::<_, State, AppAction>("div", floats).attr("class", "workspace-floats");
    el::<_, State, AppAction>("div", (frame, layer))
        .attr("class", "workspace")
        .attr("style", "position: relative; width: 100%; height: 100%;")
}

/// [`workspace_view_with_marks`] with transient tiled rails and drawers.
/// The workspace and its float content custody remain the host's authority.
pub fn workspace_view_presented_with_marks<State, AppAction, Ev, Change, Fill>(
    model: &WorkspaceModel<'_>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    presentation: &WorkbenchPresentation,
    on_event: Ev,
    on_presentation_event: Change,
    fill: Fill,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, WorkspaceEvent) + Clone + 'static,
    Change: Fn(&mut State, PresentationEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    let tile_event = on_event.clone();
    let frame = frisket_presented_with(
        model.workspace.tiled(),
        model.current,
        marks,
        presentation,
        move |state: &mut State, event| tile_event(state, WorkspaceEvent::Tile(event)),
        on_presentation_event,
        fill.clone(),
    );
    let floats: Vec<PaneView<State, AppAction>> = model
        .workspace
        .visible_floating(model.float_layer_visible)
        .into_iter()
        .map(|float| render_float(float, model.current, marks, &on_event, &fill))
        .collect();
    let layer = el::<_, State, AppAction>("div", floats).attr("class", "workspace-floats");
    el::<_, State, AppAction>("div", (frame, layer))
        .attr("class", "workspace")
        .attr("style", "position:relative;width:100%;height:100%;")
}

fn render_float<State, AppAction, Ev, Fill>(
    float: &FloatingTile,
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    on_event: &Ev,
    fill: &Fill,
) -> PaneView<State, AppAction>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, WorkspaceEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    let tile = &float.tile;
    let id = tile.id;
    let mut item = TabItem::new(tile.title.clone()).with_key(id.0.to_string());
    if let Some(accent) = tile.accent {
        item = item.with_accent(TabAccentColors::new(accent.background, accent.foreground));
    }
    if let Some(mark) = marks(id) {
        item = item.with_mark(mark);
    }
    let items = [item];
    let activate = on_event.clone();
    let close = on_event.clone();
    // One tab, so the index the bar reports is always this tile's.
    let bar = tab_bar_view(
        TabBar::new(&items, 0)
            .with_names(FLOAT_TAB_NAMES)
            .with_current(current == Some(id)),
        move |state: &mut State, _index: usize| {
            activate(state, WorkspaceEvent::Tile(TileEvent::Activated(id)))
        },
        // The × stops propagation, so a close never also raises.
        Some(move |state: &mut State, _index: usize| {
            close(state, WorkspaceEvent::Tile(TileEvent::Closed(id)))
        }),
    );

    // The content element wears the frame's marks, so a host reads a float's
    // rect back with the same `content_target` / `slot_kind` it already uses.
    // A `View` slot is built inline here rather than through `PortableKeyed`:
    // a float is its own subtree, so there is no sibling stack to move between.
    let slot = fill(tile);
    let kind = match slot {
        Slot::View(_) => SlotKind::View,
        Slot::Surface => SlotKind::Surface,
        Slot::Hole => SlotKind::Hole,
    };
    let child: Option<PaneView<State, AppAction>> = match slot {
        Slot::View(view) => Some(view),
        Slot::Surface | Slot::Hole => None,
    };
    let content = el::<_, State, AppAction>("div", child)
        .attr("class", "frisket-content")
        .attr(FRISKET_TILE_ATTR, id.0.to_string())
        .attr(ATTR_SLOT, slot_attr(kind));

    Box::new(
        el::<_, State, AppAction>("div", (bar, content))
            .attr("class", "workspace-float")
            .attr("style", float_style(float))
            .attr(FRISKET_TILE_ATTR, id.0.to_string())
            .attr("role", "group")
            .attr("aria-label", tile.title.clone()),
    )
}

/// The `data-slot` value for a kind. Mirrors the frame's writer; `slot_kind` is
/// the shared reader, and the tests pin the pair.
fn slot_attr(kind: SlotKind) -> &'static str {
    match kind {
        SlotKind::View => "view",
        SlotKind::Surface => "surface",
        SlotKind::Hole => "hole",
    }
}

/// A float's inline style: proportional placement as percentages, the pixel
/// constraints, the stacking order, and the column skeleton its bar and content
/// sit in.
fn float_style(float: &FloatingTile) -> String {
    let RelativeRect {
        x,
        y,
        width,
        height,
    } = float.rect;
    let FloatSizeConstraints {
        min_width,
        min_height,
        max_width,
        max_height,
    } = float.constraints;
    let mut style = format!(
        "position: absolute; left: {}%; top: {}%; width: {}%; height: {}%; \
         min-width: {}px; min-height: {}px; ",
        pct(x),
        pct(y),
        pct(width),
        pct(height),
        px(min_width),
        px(min_height),
    );
    if let Some(max) = max_width {
        style.push_str(&format!("max-width: {}px; ", px(max)));
    }
    if let Some(max) = max_height {
        style.push_str(&format!("max-height: {}px; ", px(max)));
    }
    style.push_str(&format!(
        "z-index: {}; display: flex; flex-direction: column;",
        float.z
    ));
    style
}

/// A fraction as a percentage number, trimmed: `0.25` is `25`, not `25.0000`.
/// Non-finite reads as zero, matching [`FloatingTile::resolve`].
fn pct(fraction: f32) -> String {
    trim(finite(fraction) * 100.0)
}

/// A pixel length, trimmed the same way.
fn px(length: f32) -> String {
    trim(finite(length))
}

fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

/// Four decimals, then strip the trailing zeros — so an f32 that is a hair off
/// a round number does not print its error into the style.
fn trim(value: f32) -> String {
    let text = format!("{value:.4}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn attr<D: LayoutDom>(dom: &D, node: D::NodeId, name: &str) -> Option<String> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
        .map(|value| value.to_string())
}

/// Every content element under `root` — frame and floats alike — whose slot the
/// host must fill itself: the [`SlotKind::Surface`] and [`SlotKind::Hole`] ones,
/// with the tile each belongs to and the node whose rect to composite into.
/// [`SlotKind::View`] elements are the frame's own and are left out.
///
/// In document order, so the frame's slots come before the float layer's and
/// the floats follow their z order. A host calls this once a frame, after
/// layout, and has exactly the list of rects it owns.
pub fn composited_slots<D: LayoutDom>(
    dom: &D,
    root: D::NodeId,
) -> Vec<(TileId, SlotKind, D::NodeId)> {
    let mut out = Vec::new();
    collect_slots(dom, root, &mut out);
    out
}

fn collect_slots<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    out: &mut Vec<(TileId, SlotKind, D::NodeId)>,
) {
    if attr(dom, node, ATTR_SLOT).is_some() {
        // `slot_kind` answers for the node itself when the node carries the
        // mark, so the frame's reader stays the only decoder.
        if let Some(kind) = slot_kind(dom, node)
            && matches!(kind, SlotKind::Surface | SlotKind::Hole)
            && let Some(id) = attr(dom, node, FRISKET_TILE_ATTR).and_then(|v| v.parse::<u64>().ok())
        {
            out.push((TileId(id), kind, node));
        }
    }
    for child in dom.dom_children(node) {
        collect_slots(dom, child, out);
    }
}

#[cfg(test)]
#[path = "workspace/tests.rs"]
mod tests;
