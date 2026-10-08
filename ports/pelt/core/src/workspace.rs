// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use inker::routing::{
    EngineRouteDecision, EngineRoutePolicy, EngineRouteRequest, SurfaceContractMode,
    WorkspaceRouteId, is_surface_engine,
};
use inker::{
    A11yCapability, ContentReport, EngineProfileBinding, NativeSurfaceHost, SessionInput,
    SessionNavigationCommand, SessionRegistry, SessionScrollKey, SessionSpawnRequest,
    SurfaceEngineRegistry, SurfaceFrame, WebSurfaceEvent,
};
use workbench::{
    ContentSource, Tile, TileEvent, TileId, TileTree, Workbench, WorkbenchEffect, WorkbenchOutcome,
};

use crate::content::{PeltContent, PeltLane, PeltLayer};
use crate::{PeltClock, PeltController, PeltHostEffect};

/// Host-long-lived engine factories and routing policy shared by every tile.
pub struct PeltRegistries<F> {
    sessions: Arc<SessionRegistry<F>>,
    surfaces: Arc<SurfaceEngineRegistry>,
    policy: EngineRoutePolicy,
    workspace_id: WorkspaceRouteId,
    fallback_document_engine: String,
    surface_profile: EngineProfileBinding,
    host_loading: bool,
}

impl<F> Clone for PeltRegistries<F> {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            surfaces: self.surfaces.clone(),
            policy: self.policy.clone(),
            workspace_id: self.workspace_id.clone(),
            fallback_document_engine: self.fallback_document_engine.clone(),
            surface_profile: self.surface_profile.clone(),
            host_loading: self.host_loading,
        }
    }
}

impl<F> PeltRegistries<F> {
    pub fn new(
        sessions: SessionRegistry<F>,
        surfaces: SurfaceEngineRegistry,
        policy: EngineRoutePolicy,
        workspace_id: impl Into<String>,
        fallback_document_engine: impl Into<String>,
        surface_profile: EngineProfileBinding,
    ) -> Self {
        Self {
            sessions: Arc::new(sessions),
            surfaces: Arc::new(surfaces),
            policy,
            workspace_id: WorkspaceRouteId::new(workspace_id),
            fallback_document_engine: fallback_document_engine.into(),
            surface_profile,
            host_loading: false,
        }
    }

    /// Every document controller opened through these registries lets the
    /// host's transport fetch (see [`crate::PeltLoadMode::Host`]).
    pub fn with_host_loading(mut self) -> Self {
        self.host_loading = true;
        self
    }

    pub fn host_loading(&self) -> bool {
        self.host_loading
    }

    pub fn sessions(&self) -> &SessionRegistry<F> {
        &self.sessions
    }

    pub fn surfaces(&self) -> &SurfaceEngineRegistry {
        &self.surfaces
    }

    pub fn surface_profile(&self) -> &EngineProfileBinding {
        &self.surface_profile
    }

    pub(crate) fn sessions_arc(&self) -> Arc<SessionRegistry<F>> {
        Arc::clone(&self.sessions)
    }

    pub(crate) fn surfaces_arc(&self) -> Arc<SurfaceEngineRegistry> {
        Arc::clone(&self.surfaces)
    }

    /// Route one load. A pinned engine is kept visible even when unavailable,
    /// so the host can explain the active fallback; automatic routing filters
    /// unavailable lanes. `documents_only` restricts the choice to document
    /// engines, for a body whose address already chose the document lane.
    pub(crate) fn choose(
        &self,
        address: &str,
        content_type: Option<&str>,
        engine_override: Option<&str>,
        documents_only: bool,
    ) -> Result<LoadRoute, String> {
        let route_request = EngineRouteRequest {
            workspace_id: self.workspace_id.clone(),
            view: None,
            node: None,
            address: address.to_owned(),
            content_type: content_type.map(str::to_owned),
            pinned_engine: engine_override.map(str::to_owned),
        };
        let source = if engine_override.is_some() {
            PeltRouteSource::UserOverride
        } else {
            PeltRouteSource::Automatic
        };
        let decision = if engine_override.is_some() {
            self.policy.route(&route_request)
        } else {
            self.policy.route_filtered(&route_request, |engine| {
                self.sessions.contains(engine)
                    || (!documents_only && self.surfaces.contains(engine))
            })
        };
        let selected = decision.engine_id.clone();
        let mut route = PeltRoute {
            decision,
            source,
            state: PeltRouteState::Document,
        };
        if self.sessions.contains(&selected) {
            return Ok(LoadRoute::Document {
                engine_id: selected,
                route: Some(route),
            });
        }
        if !documents_only
            && self.surfaces.contains(&selected)
            && route.decision.surface_contract.mode == SurfaceContractMode::CompositedTexture
        {
            route.state = PeltRouteState::Surface;
            return Ok(LoadRoute::Surface(route));
        }
        let fallback = self.fallback_document_engine.clone();
        if !self.sessions.contains(&fallback) {
            return Err(format!(
                "selected engine {selected} is unavailable and fallback engine {fallback} is not registered"
            ));
        }
        let reason = if self.surfaces.contains(&selected) {
            format!(
                "surface contract {:?} needs an embedding adapter",
                route.decision.surface_contract.mode
            )
        } else if is_surface_engine(&selected) {
            "surface engine is not registered on this host".to_owned()
        } else {
            "document engine is not registered on this host".to_owned()
        };
        route.state = PeltRouteState::Fallback {
            active_engine: fallback.clone(),
            reason,
        };
        Ok(LoadRoute::Document {
            engine_id: fallback,
            route: Some(route),
        })
    }
}

/// One load's lane. A document choice without a route is an unrouted
/// controller keeping its own engine.
pub(crate) enum LoadRoute {
    Document {
        engine_id: String,
        route: Option<PeltRoute>,
    },
    Surface(PeltRoute),
}

/// Inputs that select one tile's lane. A source-capable document session can
/// refresh the held body after navigation, so routing never needs to refetch
/// merely to switch engines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeltTileRequest {
    pub request: SessionSpawnRequest,
    pub engine_override: Option<String>,
    /// The profile a surface lane opens with, when it is not the registries'
    /// shared one (Turnstone keeps one web profile per node).
    pub surface_profile: Option<EngineProfileBinding>,
}

impl PeltTileRequest {
    pub fn new(address: impl Into<String>, viewport: (u32, u32)) -> Self {
        Self::from_request(SessionSpawnRequest::new(address).with_viewport(viewport.0, viewport.1))
    }

    pub fn from_request(request: SessionSpawnRequest) -> Self {
        Self {
            request,
            engine_override: None,
            surface_profile: None,
        }
    }

    pub fn with_engine_override(mut self, engine_id: impl Into<String>) -> Self {
        self.engine_override = Some(engine_id.into());
        self
    }

    pub fn with_surface_profile(mut self, profile: EngineProfileBinding) -> Self {
        self.surface_profile = Some(profile);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeltRouteSource {
    Automatic,
    UserOverride,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeltRouteState {
    Document,
    Surface,
    Fallback {
        active_engine: String,
        reason: String,
    },
}

/// Selected route plus the lane that is actually active.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeltRoute {
    pub decision: EngineRouteDecision,
    pub source: PeltRouteSource,
    pub state: PeltRouteState,
}

impl PeltRoute {
    pub fn selected_engine(&self) -> &str {
        &self.decision.engine_id
    }

    pub fn active_engine(&self) -> &str {
        match &self.state {
            PeltRouteState::Fallback { active_engine, .. } => active_engine,
            PeltRouteState::Document | PeltRouteState::Surface => &self.decision.engine_id,
        }
    }

    pub fn for_tile(&self, tile: TileId) -> PeltTileRoute {
        PeltTileRoute {
            tile,
            decision: self.decision.clone(),
            source: self.source,
            state: self.state.clone(),
        }
    }
}

/// Selected route plus the lane that is actually active for the tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeltTileRoute {
    pub tile: TileId,
    pub decision: EngineRouteDecision,
    pub source: PeltRouteSource,
    pub state: PeltRouteState,
}

impl PeltTileRoute {
    pub fn selected_engine(&self) -> &str {
        &self.decision.engine_id
    }

    pub fn active_engine(&self) -> &str {
        match &self.state {
            PeltRouteState::Fallback { active_engine, .. } => active_engine,
            PeltRouteState::Document | PeltRouteState::Surface => &self.decision.engine_id,
        }
    }
}

/// The active tile's declared semantic capability and any structural report
/// its engine provides. Surface lanes retain their declared capability even
/// when they cannot expose a report, so hosts never guess from route kind.
#[derive(Clone, Debug, PartialEq)]
pub struct PeltTileInspection {
    pub capability: A11yCapability,
    pub report: Option<ContentReport>,
}

/// One Frisket content hole in workspace coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorkspaceRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WorkspaceRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width.max(0.0)
            && y < self.y + self.height.max(0.0)
    }

    fn local(self, x: f32, y: f32) -> (f32, f32) {
        (x - self.x, y - self.y)
    }

    pub(crate) fn viewport(self) -> (u32, u32) {
        (
            self.width.max(1.0).ceil() as u32,
            self.height.max(1.0).ceil() as u32,
        )
    }
}

/// One active document layer returned by [`PeltWorkspace::frame`].
pub struct PeltTileFrame<F> {
    pub tile: TileId,
    pub rect: WorkspaceRect,
    pub frame: F,
}

/// One surface-engine layer. A native frame remains an Inker handle until the
/// embedding host imports it on its own wgpu device.
pub struct PeltSurfaceLayer {
    pub tile: TileId,
    pub rect: WorkspaceRect,
    pub route: PeltTileRoute,
    pub frame: Result<Option<SurfaceFrame>, String>,
}

/// The document layers for one workspace frame. Frisket's frame scene remains
/// host-owned because the reusable core is generic over the document frame.
pub struct PeltWorkspaceFrame<F> {
    pub tiles: Vec<PeltTileFrame<F>>,
    pub surfaces: Vec<PeltSurfaceLayer>,
}

/// Pelt's result for a workspace command.
///
/// The embedded [`WorkbenchOutcome`] reports the shared arrangement result or
/// a host request. `focus_changed` covers Pelt's retained-controller focus,
/// which can change even when an already-active tab leaves the tree intact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeltWorkspaceOutcome {
    workbench: WorkbenchOutcome,
    focus_changed: bool,
}

impl PeltWorkspaceOutcome {
    pub const fn changed(self) -> bool {
        self.workbench.changed() || self.focus_changed
    }

    pub const fn effect(self) -> Option<WorkbenchEffect> {
        self.workbench.effect()
    }

    pub const fn workbench(self) -> WorkbenchOutcome {
        self.workbench
    }
}

struct RoutedWorkspace<F> {
    registries: PeltRegistries<F>,
    base_titles: HashMap<TileId, String>,
    clock_for: Arc<dyn Fn() -> Box<dyn PeltClock>>,
}

/// Pelt's window-neutral recursive workspace.
///
/// `Workbench` wraps the arrangement authority. Every document tile owns one
/// live [`PeltContent`], including inactive tabs; Frisket content-hole
/// rectangles arrive from the embedding host and are the only geometry used
/// for routing and frame sizing.
pub struct PeltWorkspace<F> {
    workbench: Workbench,
    contents: HashMap<TileId, PeltContent<F>>,
    /// Each tile's route, refreshed whenever its content may have changed lane
    /// or engine, so hosts can borrow it.
    routes: HashMap<TileId, PeltTileRoute>,
    content_rects: HashMap<TileId, WorkspaceRect>,
    focused: Option<TileId>,
    pointer_capture: Option<TileId>,
    surface_scale_factor: f32,
    routed: Option<RoutedWorkspace<F>>,
    surface_resource_policy: crate::SurfaceResourcePolicy,
    surface_frame_index: u64,
    surface_refresh_cursor: usize,
}

impl<F: 'static> PeltWorkspace<F> {
    /// Compatibility constructor for callers that already select and build
    /// one document controller per tile. New hosts use [`Self::try_routed`].
    pub fn try_new(
        tree: TileTree,
        mut controller_for: impl FnMut(&Tile) -> Result<PeltController<F>, String>,
    ) -> Result<Self, String> {
        let mut contents = HashMap::new();
        let mut tile_ids = HashSet::new();
        for tile in tree.tiles() {
            if !tile_ids.insert(tile.id) {
                return Err(format!("duplicate tile id {}", tile.id.0));
            }
            if matches!(tile.content, ContentSource::Document(_)) {
                let controller = controller_for(tile)
                    .map_err(|error| format!("could not open tile {}: {error}", tile.id.0))?;
                let route = PeltRoute {
                    decision: EngineRouteDecision {
                        engine_id: controller.engine_id().to_owned(),
                        surface_contract: inker::routing::SurfaceContract {
                            target: inker::routing::SurfaceTargetId::new(format!(
                                "pelt:tile:{}",
                                tile.id.0
                            )),
                            mode: SurfaceContractMode::CompositedTexture,
                        },
                    },
                    source: PeltRouteSource::UserOverride,
                    state: PeltRouteState::Document,
                };
                contents.insert(tile.id, PeltContent::from_controller(controller, route));
            }
        }
        Self::assemble(tree, contents, None)
    }

    /// Route every document tile through one pair of shared registries. A
    /// registered surface engine becomes a retained producer; an unavailable
    /// or unattached surface contract stays selected and visibly falls back to
    /// the configured document engine.
    pub fn try_routed(
        tree: TileTree,
        registries: PeltRegistries<F>,
        mut request_for: impl FnMut(&Tile) -> Result<PeltTileRequest, String>,
        clock_for: impl Fn() -> Box<dyn PeltClock> + 'static,
    ) -> Result<Self, String> {
        let clock_for: Arc<dyn Fn() -> Box<dyn PeltClock>> = Arc::new(clock_for);
        let mut tile_ids = HashSet::new();
        let mut requests = Vec::new();
        let mut base_titles = HashMap::new();
        for tile in tree.tiles() {
            if !tile_ids.insert(tile.id) {
                return Err(format!("duplicate tile id {}", tile.id.0));
            }
            if matches!(tile.content, ContentSource::Document(_)) {
                requests.push((
                    tile.id,
                    request_for(tile)
                        .map_err(|error| format!("could not route tile {}: {error}", tile.id.0))?,
                ));
                base_titles.insert(tile.id, tile.title.clone());
            }
        }
        let mut contents = HashMap::new();
        for (tile, request) in requests {
            contents.insert(
                tile,
                PeltContent::routed(registries.clone(), request, Arc::clone(&clock_for))?,
            );
        }
        Self::assemble(
            tree,
            contents,
            Some(RoutedWorkspace {
                registries,
                base_titles,
                clock_for,
            }),
        )
    }

    fn assemble(
        tree: TileTree,
        contents: HashMap<TileId, PeltContent<F>>,
        routed: Option<RoutedWorkspace<F>>,
    ) -> Result<Self, String> {
        let focused = active_tiles(&tree)
            .into_iter()
            .find(|id| contents.contains_key(id));
        let mut workspace = Self {
            workbench: Workbench::new(tree),
            contents,
            routes: HashMap::new(),
            content_rects: HashMap::new(),
            focused,
            pointer_capture: None,
            surface_scale_factor: 1.0,
            routed,
            surface_resource_policy: crate::SurfaceResourcePolicy::default(),
            surface_frame_index: 0,
            surface_refresh_cursor: 0,
        };
        workspace.sync_tile_metadata();
        workspace.sync_visibility();
        Ok(workspace)
    }

    pub fn tree(&self) -> &TileTree {
        self.workbench.tree()
    }

    pub fn focused_tile(&self) -> Option<TileId> {
        self.focused
    }

    /// Whether an ordinary physical pointer gesture still owns workspace
    /// routing. Synthetic semantic activation must wait for that gesture to
    /// finish rather than borrowing its capture target.
    pub fn has_active_pointer_capture(&self) -> bool {
        self.pointer_capture.is_some()
    }

    /// One tile's routed content, whichever lane is live.
    pub fn content(&self, tile: TileId) -> Option<&PeltContent<F>> {
        self.contents.get(&tile)
    }

    /// Mutate one tile's content directly. Call [`Self::sync_tile`] after a
    /// change that can move its route or address.
    pub fn content_mut(&mut self, tile: TileId) -> Option<&mut PeltContent<F>> {
        self.contents.get_mut(&tile)
    }

    /// Refresh one tile's route, title and address after a direct content
    /// change.
    pub fn sync_tile(&mut self, tile: TileId) {
        self.sync_one_tile_metadata(tile);
        self.sync_visibility();
    }

    pub fn controller(&self, tile: TileId) -> Option<&PeltController<F>> {
        self.contents.get(&tile).and_then(PeltContent::document)
    }

    pub fn controller_mut(&mut self, tile: TileId) -> Option<&mut PeltController<F>> {
        self.contents
            .get_mut(&tile)
            .and_then(PeltContent::document_mut)
    }

    /// Move a live composited surface producer to another native host while
    /// retaining its session and producer identity. Ordinary producer failures
    /// leave the prior host in place. `HostMigrationIndeterminate` is terminal:
    /// the host must transfer custody to the destination and keep its visual
    /// shell alive rather than resuming source presentation.
    ///
    /// # Safety
    /// `host` must name a live native host the producer may attach to.
    pub unsafe fn rehost_surface(
        &mut self,
        tile: TileId,
        host: NativeSurfaceHost,
    ) -> Result<(), inker::SurfaceError> {
        let content = self
            .contents
            .get_mut(&tile)
            .filter(|content| content.lane() == PeltLane::Surface)
            .ok_or_else(|| {
                inker::SurfaceError::Unsupported(format!(
                    "tile {} has no live surface producer",
                    tile.0
                ))
            })?;
        unsafe { content.rehost_surface(host) }
    }

    /// Identity generation of a tile's current successfully opened document
    /// session. Surface-only tiles return `None`.
    ///
    /// Hosts retaining engine-specific observations can pair this with the
    /// tile id and clear that state when a successful navigation, reload, or
    /// history traversal replaces the session.
    pub fn document_session_generation(&self, tile: TileId) -> Option<u64> {
        self.contents
            .get(&tile)
            .and_then(PeltContent::session_generation)
    }

    /// Process-unique controller identity plus its retained-session
    /// generation. Consumers that namespace externally visible nodes must use
    /// this rather than generation alone because route reconstruction creates
    /// a fresh controller at generation one.
    pub fn document_session_identity(&self, tile: TileId) -> Option<super::PeltSessionIdentity> {
        self.contents
            .get(&tile)
            .and_then(PeltContent::session_identity)
    }

    pub fn route(&self, tile: TileId) -> Option<&PeltTileRoute> {
        self.routes.get(&tile)
    }

    pub fn routes(&self) -> impl Iterator<Item = &PeltTileRoute> {
        self.routes.values()
    }

    /// Inspect the provider actually active for `tile`. A selected surface may
    /// have fallen back to a document controller, in which case the document's
    /// capability and report are authoritative. A live surface instead exposes
    /// only the capability declared by its registered surface engine.
    pub fn inspection(&self, tile: TileId) -> Option<PeltTileInspection> {
        self.contents.get(&tile)?.inspection()
    }

    /// Drain the next ordered web event from one routed surface. Document
    /// tiles return `Ok(None)` so hosts can probe a mixed workspace without
    /// duplicating route-state checks.
    pub fn poll_surface_web_event(
        &mut self,
        tile: TileId,
    ) -> Result<Option<WebSurfaceEvent>, String> {
        match self.contents.get_mut(&tile) {
            Some(content) => content
                .poll_web_event()
                .map_err(|error| format!("tile {} {error}", tile.0)),
            None => Ok(None),
        }
    }

    /// Configure polling pressure without changing activation, visibility, or
    /// focus routing. Deferred polls retain the producer's last composed image.
    pub fn set_surface_resource_policy(&mut self, policy: crate::SurfaceResourcePolicy) {
        self.surface_resource_policy = policy.normalized();
    }

    pub fn surface_resource_policy(&self) -> crate::SurfaceResourcePolicy {
        self.surface_resource_policy
    }

    /// Replace or clear the user engine choice for one live tile. The selected
    /// address/body stay held while the tile is reconstructed through the same
    /// shared registry pair.
    pub fn set_route_override(
        &mut self,
        tile: TileId,
        engine_id: Option<String>,
    ) -> Result<bool, String> {
        if self.routed.is_none() {
            return Err("this workspace was built without capability routing".to_owned());
        }
        let Some(content) = self.contents.get_mut(&tile) else {
            return Err(format!("tile {} has no routed content", tile.0));
        };
        let changed = content.set_route_override(engine_id)?;
        if changed {
            self.sync_one_tile_metadata(tile);
            self.sync_visibility();
        }
        Ok(changed)
    }

    pub fn content_rect(&self, tile: TileId) -> Option<WorkspaceRect> {
        self.content_rects.get(&tile).copied()
    }

    /// Physical pixels per workspace unit for native surface producers.
    /// Document sessions continue to receive logical content-hole extents.
    pub fn set_surface_scale_factor(&mut self, scale_factor: f32) {
        self.surface_scale_factor = scale_factor.max(1.0);
    }

    /// Replace the content-hole geometry read from the latest Frisket layout.
    /// Rectangles for inactive or closed tiles are deliberately discarded.
    pub fn set_content_rects(&mut self, rects: impl IntoIterator<Item = (TileId, WorkspaceRect)>) {
        self.content_rects = rects.into_iter().collect();
    }

    /// Apply a standalone Pelt arrangement gesture through the shared
    /// Workbench reducer, preserving Pelt's controller and focus custody.
    pub fn apply(&mut self, event: &TileEvent) -> bool {
        self.apply_outcome(event).changed()
    }

    /// Apply a workspace gesture and expose any request that needs a desktop
    /// host decision. A tearout leaves the tree and Pelt controller custody
    /// unchanged until that host accepts it.
    pub fn apply_outcome(&mut self, event: &TileEvent) -> PeltWorkspaceOutcome {
        let workbench = self.workbench.apply(event);
        if !workbench.changed() {
            let mut focus_changed = false;
            if let TileEvent::Activated(id) = event {
                if active_tiles(self.workbench.tree()).contains(id) && self.has_content(*id) {
                    focus_changed = self.focused != Some(*id);
                    self.focus(*id);
                }
            }
            return PeltWorkspaceOutcome {
                workbench,
                focus_changed,
            };
        }

        let retained = self
            .workbench
            .tree()
            .tiles()
            .into_iter()
            .map(|tile| tile.id)
            .collect::<HashSet<_>>();
        self.contents.retain(|id, _| retained.contains(id));
        self.routes.retain(|id, _| retained.contains(id));
        self.content_rects.retain(|id, _| retained.contains(id));
        if let Some(routed) = &mut self.routed {
            routed.base_titles.retain(|id, _| retained.contains(id));
        }
        if self
            .pointer_capture
            .is_some_and(|id| !retained.contains(&id))
        {
            self.pointer_capture = None;
        }

        let next_focus = match event {
            TileEvent::Activated(id) | TileEvent::Dragged { tile: id, .. }
                if self.has_content(*id) =>
            {
                Some(*id)
            },
            _ if self.focused.is_some_and(|id| retained.contains(&id)) => self.focused,
            _ => active_tiles(self.workbench.tree())
                .into_iter()
                .find(|id| self.has_content(*id)),
        };
        match next_focus {
            Some(id) => self.focus(id),
            None => self.focused = None,
        }
        self.sync_visibility();
        PeltWorkspaceOutcome {
            workbench,
            focus_changed: false,
        }
    }

    /// Transfer one live tile into a host-accepted destination workspace.
    ///
    /// This is deliberately separate from [`Self::apply_outcome`]: an outside
    /// drop only requests a tearout, while the native host decides whether it
    /// can create a destination window. Call this only after that acceptance
    /// succeeds. The returned workspace keeps the original [`TileId`], route,
    /// controller or surface producer, and focused-session identity.
    ///
    /// `None` means `tile` was not a live Pelt tile. In that case the source
    /// workspace is left untouched.
    pub fn accept_tearout(&mut self, tile: TileId) -> Option<Self> {
        let tile_record = self.workbench.tree().find(tile)?.clone();
        if !self.has_content(tile) {
            return None;
        }

        // This mutation occurs only after the embedding host has accepted the
        // effect. It cannot fail after the live tile check above, and keeping
        // it here makes content custody a single synchronous transfer.
        let removed = self.workbench.apply(&TileEvent::Closed(tile));
        debug_assert!(matches!(removed, WorkbenchOutcome::Applied));
        let content = self.contents.remove(&tile);
        let route = self.routes.remove(&tile);
        let content_rect = self.content_rects.remove(&tile);
        let routed = self.routed.as_mut().map(|source| RoutedWorkspace {
            registries: source.registries.clone(),
            base_titles: source
                .base_titles
                .remove(&tile)
                .map(|title| (tile, title))
                .into_iter()
                .collect::<HashMap<_, _>>(),
            clock_for: Arc::clone(&source.clock_for),
        });

        let retained = self
            .workbench
            .tree()
            .tiles()
            .into_iter()
            .map(|tile| tile.id)
            .collect::<HashSet<_>>();
        if self
            .pointer_capture
            .is_some_and(|captured| !retained.contains(&captured))
        {
            self.pointer_capture = None;
        }
        let next_focus = self
            .focused
            .filter(|focused| retained.contains(focused))
            .or_else(|| {
                active_tiles(self.workbench.tree())
                    .into_iter()
                    .find(|id| self.has_content(*id))
            });
        if next_focus != self.focused {
            self.focused = None;
            if let Some(next) = next_focus {
                self.focus(next);
            }
        }
        self.sync_visibility();

        let mut destination = Self {
            workbench: Workbench::new(TileTree::single(tile_record)),
            contents: content.map(|content| (tile, content)).into_iter().collect(),
            routes: route.map(|route| (tile, route)).into_iter().collect(),
            content_rects: content_rect.map(|rect| (tile, rect)).into_iter().collect(),
            focused: None,
            pointer_capture: None,
            surface_scale_factor: self.surface_scale_factor,
            routed,
            surface_resource_policy: self.surface_resource_policy,
            surface_frame_index: 0,
            surface_refresh_cursor: 0,
        };
        destination.focus(tile);
        destination.sync_tile_metadata();
        destination.sync_visibility();
        Some(destination)
    }

    fn has_content(&self, tile: TileId) -> bool {
        self.contents.contains_key(&tile)
    }

    fn is_surface(&self, tile: TileId) -> bool {
        self.contents
            .get(&tile)
            .is_some_and(|content| content.lane() == PeltLane::Surface)
    }

    /// Produce one frame for every active document hole, sized to that hole.
    pub fn frame(&mut self) -> PeltWorkspaceFrame<F> {
        self.frame_with_surface_polling(true)
    }

    /// Produce document frames and the active surface layers without polling
    /// their producers.
    ///
    /// A host can use this for a bounded compositor capture after it has
    /// already imported a native surface frame. The returned surface layers
    /// preserve their routes and geometry, while their `frame` is `Ok(None)`
    /// so the host reuses its cached imported view instead of advancing an
    /// external producer.
    pub fn frame_with_cached_surfaces(&mut self) -> PeltWorkspaceFrame<F> {
        self.frame_with_surface_polling(false)
    }

    /// Produce one tile's current frame at host-provided geometry without
    /// changing the workspace's active arrangement. A native host uses this
    /// to preflight a tearout destination while the source still owns the
    /// controller or surface producer.
    pub fn frame_tile(
        &mut self,
        tile: TileId,
        rect: WorkspaceRect,
    ) -> Option<PeltWorkspaceFrame<F>> {
        let mut tiles = Vec::new();
        let mut surfaces = Vec::new();
        let route = self.routes.get(&tile).cloned();
        match self
            .contents
            .get_mut(&tile)?
            .frame(rect, self.surface_scale_factor)
        {
            PeltLayer::Document(frame) => tiles.push(PeltTileFrame { tile, rect, frame }),
            PeltLayer::Surface(frame) => surfaces.push(PeltSurfaceLayer {
                tile,
                rect,
                route: route?,
                frame,
            }),
        }
        self.sync_one_tile_metadata(tile);
        Some(PeltWorkspaceFrame { tiles, surfaces })
    }

    fn frame_with_surface_polling(&mut self, poll_surfaces: bool) -> PeltWorkspaceFrame<F> {
        let active = active_tiles(self.workbench.tree());
        let frame_index = self.surface_frame_index;
        if poll_surfaces {
            self.surface_frame_index = self.surface_frame_index.wrapping_add(1);
        }
        let surface_ids = active
            .iter()
            .copied()
            .filter(|id| self.is_surface(*id))
            .collect::<Vec<_>>();
        let mut refresh_ids = std::collections::HashSet::new();
        if poll_surfaces && !surface_ids.is_empty() {
            let policy = self.surface_resource_policy.normalized();
            if policy.admits(frame_index, 0) {
                let limit = policy.max_refreshes_per_frame.min(surface_ids.len());
                for index in crate::surface_policy::rotated_indices(
                    surface_ids.len(),
                    self.surface_refresh_cursor,
                    limit,
                ) {
                    refresh_ids.insert(surface_ids[index]);
                }
                self.surface_refresh_cursor =
                    (self.surface_refresh_cursor + limit) % surface_ids.len();
            }
        }
        let mut surface_refreshes = 0;
        let mut tiles = Vec::with_capacity(active.len());
        let mut surfaces = Vec::new();
        for tile in active {
            let Some(rect) = self.content_rects.get(&tile).copied() else {
                continue;
            };
            if self.is_surface(tile) {
                let frame = if poll_surfaces
                    && refresh_ids.contains(&tile)
                    && self
                        .surface_resource_policy
                        .admits(frame_index, surface_refreshes)
                {
                    surface_refreshes += 1;
                    match self
                        .contents
                        .get_mut(&tile)
                        .expect("surface tile has content")
                        .frame(rect, self.surface_scale_factor)
                    {
                        PeltLayer::Surface(frame) => frame,
                        PeltLayer::Document(_) => unreachable!("checked surface lane"),
                    }
                } else {
                    Ok(None)
                };
                if let Some(route) = self.routes.get(&tile).cloned() {
                    surfaces.push(PeltSurfaceLayer {
                        tile,
                        rect,
                        route,
                        frame,
                    });
                }
            } else if let Some(content) = self.contents.get_mut(&tile) {
                if let PeltLayer::Document(frame) = content.frame(rect, self.surface_scale_factor) {
                    tiles.push(PeltTileFrame { tile, rect, frame });
                }
            }
        }
        self.sync_tile_metadata();
        PeltWorkspaceFrame { tiles, surfaces }
    }

    /// Advance visible sessions. Hidden tabs retain state without driving the
    /// foreground frame loop.
    pub fn pump(&mut self) -> bool {
        let mut more = false;
        for id in active_tiles(self.workbench.tree()) {
            // Surface producers do not expose a settled bit. Keep polling every
            // visible producer so a frame that arrives after the first acquire
            // is not stranded until unrelated document activity causes a
            // redraw.
            if let Some(content) = self.contents.get_mut(&id) {
                more |= content.pump();
            }
        }
        more
    }

    /// Mark every visible document session as composed by the embedding host.
    ///
    /// The host calls this after its presentation boundary. Hidden tabs retain
    /// their loading state until their own first visible composition.
    pub fn mark_visible_documents_presented(&mut self) {
        for id in active_tiles(self.workbench.tree()) {
            if let Some(content) = self.contents.get_mut(&id) {
                content.mark_presented();
            }
        }
    }

    /// Route neutral input. Pointer coordinates are workspace coordinates and
    /// are translated into the selected Frisket content hole; keyboard, text,
    /// IME, and focus route to the focused tile.
    pub fn input(&mut self, input: SessionInput) -> PeltHostEffect {
        let (target, local_input) = match input {
            SessionInput::PointerMoved { x, y, modifiers } => {
                let target = self.pointer_capture.or_else(|| self.tile_at(x, y));
                let Some(target) = target else {
                    return PeltHostEffect::default();
                };
                let Some(rect) = self.content_rect(target) else {
                    return PeltHostEffect::default();
                };
                let (x, y) = rect.local(x, y);
                (target, SessionInput::PointerMoved { x, y, modifiers })
            },
            SessionInput::PointerButton {
                x,
                y,
                button,
                state,
                modifiers,
            } => {
                let target = self.pointer_capture.or_else(|| self.tile_at(x, y));
                let Some(target) = target else {
                    return PeltHostEffect::default();
                };
                let Some(rect) = self.content_rect(target) else {
                    return PeltHostEffect::default();
                };
                self.focus(target);
                let (x, y) = rect.local(x, y);
                (
                    target,
                    SessionInput::PointerButton {
                        x,
                        y,
                        button,
                        state,
                        modifiers,
                    },
                )
            },
            other => {
                let Some(target) = self.focused else {
                    return PeltHostEffect::default();
                };
                (target, other)
            },
        };

        let scale_factor = self.surface_scale_factor;
        let Some(content) = self.contents.get_mut(&target) else {
            return PeltHostEffect::default();
        };
        let effect = content.input(local_input, scale_factor);
        if let Some(capture) = effect.pointer_capture {
            self.pointer_capture = capture.then_some(target);
        }
        if effect.navigated {
            self.sync_one_tile_metadata(target);
            self.sync_visibility();
        }
        effect
    }

    pub fn scroll_at(&mut self, x: f32, y: f32, dx: f32, dy: f32) -> bool {
        let Some(tile) = self.tile_at(x, y) else {
            return false;
        };
        let Some(rect) = self.content_rect(tile) else {
            return false;
        };
        let (x, y) = rect.local(x, y);
        let scale_factor = self.surface_scale_factor;
        self.contents
            .get_mut(&tile)
            .is_some_and(|content| content.scroll_at(x, y, dx, dy, scale_factor))
    }

    pub fn scroll_for_key(&mut self, key: SessionScrollKey) -> bool {
        let Some(tile) = self.focused else {
            return false;
        };
        self.contents
            .get_mut(&tile)
            .is_some_and(|content| content.scroll_for_key(key))
    }

    pub fn command(&mut self, command: SessionNavigationCommand) -> PeltHostEffect {
        let Some(tile) = self.focused else {
            return PeltHostEffect::default();
        };
        self.command_for(tile, command)
    }

    pub fn command_for(
        &mut self,
        tile: TileId,
        command: SessionNavigationCommand,
    ) -> PeltHostEffect {
        let Some(content) = self.contents.get_mut(&tile) else {
            return PeltHostEffect::default();
        };
        let effect = content.command(command);
        if effect.navigated {
            self.sync_one_tile_metadata(tile);
            self.sync_visibility();
        }
        effect
    }

    fn tile_at(&self, x: f32, y: f32) -> Option<TileId> {
        active_tiles(self.workbench.tree()).into_iter().find(|id| {
            self.content_rects
                .get(id)
                .is_some_and(|rect| rect.contains(x, y))
        })
    }

    fn focus(&mut self, tile: TileId) {
        if self.focused == Some(tile) {
            return;
        }
        if let Some(old) = self.focused.and_then(|id| self.contents.get_mut(&id)) {
            old.focus(false);
        }
        self.focused = Some(tile);
        if let Some(new) = self.contents.get_mut(&tile) {
            new.focus(true);
        }
    }

    fn sync_visibility(&mut self) {
        let active = active_tiles(self.workbench.tree())
            .into_iter()
            .collect::<HashSet<_>>();
        for (id, content) in &mut self.contents {
            content.set_hidden(!active.contains(id));
        }
    }

    fn sync_tile_metadata(&mut self) {
        let ids = self.contents.keys().copied().collect::<Vec<_>>();
        for id in ids {
            self.sync_one_tile_metadata(id);
        }
    }

    fn sync_one_tile_metadata(&mut self, id: TileId) {
        let Some(content) = self.contents.get_mut(&id) else {
            return;
        };
        content.refresh_held_request();
        let route = content.route().for_tile(id);
        let document_address = content
            .document()
            .map(|controller| controller.address().to_owned());
        let base_title = content
            .title()
            .filter(|title| !title.trim().is_empty())
            .or_else(|| {
                self.routed
                    .as_ref()
                    .and_then(|routed| routed.base_titles.get(&id).cloned())
            });
        if let Some(tile) = self.workbench.tree_mut().tile_mut(id) {
            if let Some(address) = document_address {
                tile.content = ContentSource::Document(workbench::DocumentRef(address));
            }
            let selected = route.selected_engine();
            let suffix = match &route.state {
                PeltRouteState::Fallback { active_engine, .. } => {
                    format!("[{selected} → {active_engine}]")
                },
                PeltRouteState::Document | PeltRouteState::Surface => {
                    format!("[{selected}]")
                },
            };
            tile.title = format!("{} {suffix}", base_title.as_deref().unwrap_or(selected));
        }
        self.routes.insert(id, route);
    }
}

fn active_tiles(tree: &TileTree) -> Vec<TileId> {
    fn visit(tree: &TileTree, active: &mut Vec<TileId>) {
        match tree {
            TileTree::Split { children, .. } => {
                for branch in children {
                    visit(&branch.tree, active);
                }
            },
            TileTree::Stack(stack) => {
                if let Some(tile) = stack.tabs.get(stack.active) {
                    active.push(tile.id);
                }
            },
        }
    }

    let mut active = Vec::new();
    visit(tree, &mut active);
    active
}
