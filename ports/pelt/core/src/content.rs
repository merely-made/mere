// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One routed piece of browsing content: a document lane or a surface lane
//! behind one command, input, frame and routing API.
//!
//! A document lane is a [`PeltController`] whose new loads are routed through
//! the shared registries, so a navigation can change document engine. When a
//! load routes to a surface engine the content swaps to a surface lane, and a
//! surface lane asked for a document address swaps back. `PeltWorkspace`
//! holds one per tile; a host with its own arrangement (Turnstone keys by
//! graph node) holds one per key.

use std::sync::Arc;

use inker::{
    DocumentClipArtifactRole, FocusReason, KeyboardEvent, KeyboardModifiers, MouseButton,
    MouseEvent, MouseEventKind, NativeSurfaceHost, PhysicalPosition, SessionButtonState,
    SessionInput, SessionKey, SessionNavigationCommand, SessionPointerButton, SessionScrollKey,
    SessionSpawnRequest, SurfaceFrame, SurfaceProducer, SurfaceSpawnRequest, WebSurfaceEvent,
};

use crate::workspace::{LoadRoute, WorkspaceRect};
use crate::{
    FetchOutcome, PageProgress, PeltClock, PeltController, PeltControllerConfig, PeltHostEffect,
    PeltLoadCommand, PeltRegistries, PeltReroute, PeltRoute, PeltRouter, PeltSessionIdentity,
    PeltTileInspection, PeltTileRequest,
};

/// Which kind of lane presents a piece of content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeltLane {
    Document,
    Surface,
}

/// One content frame. A surface frame stays an Inker handle until the
/// embedding host imports it on its own wgpu device.
pub enum PeltLayer<F> {
    Document(F),
    Surface(Result<Option<SurfaceFrame>, String>),
}

/// See the module documentation.
pub struct PeltContent<F> {
    lane: Lane<F>,
    route: PeltRoute,
    routed: Option<Routed<F>>,
}

enum Lane<F> {
    Document(PeltController<F>),
    Surface(PeltSurface),
}

/// What a routed content keeps to reconstruct itself: the shared registries,
/// the held request (address, body, media type and engine pin) and a clock
/// factory for each new controller.
struct Routed<F> {
    registries: PeltRegistries<F>,
    request: PeltTileRequest,
    clock_for: Arc<dyn Fn() -> Box<dyn PeltClock>>,
}

impl<F: 'static> PeltContent<F> {
    /// Route `request` through shared registries and open its lane. A
    /// registered surface engine becomes a retained producer; an unavailable or
    /// unattached surface contract stays selected and visibly falls back to the
    /// configured document engine.
    pub fn routed(
        registries: PeltRegistries<F>,
        request: PeltTileRequest,
        clock_for: Arc<dyn Fn() -> Box<dyn PeltClock>>,
    ) -> Result<Self, String> {
        let choice = registries.choose(
            &request.request.address,
            request.request.content_type.as_deref(),
            request.engine_override.as_deref(),
            false,
        )?;
        let (lane, route) = open_lane(&registries, &request, &clock_for, choice)?;
        Ok(Self {
            lane,
            route,
            routed: Some(Routed {
                registries,
                request,
                clock_for,
            }),
        })
    }

    /// Wrap a controller the caller built and routed itself. It keeps its
    /// engine for every load and cannot change lanes.
    pub fn from_controller(controller: PeltController<F>, route: PeltRoute) -> Self {
        Self {
            lane: Lane::Document(controller),
            route,
            routed: None,
        }
    }

    pub fn lane(&self) -> PeltLane {
        match self.lane {
            Lane::Document(_) => PeltLane::Document,
            Lane::Surface(_) => PeltLane::Surface,
        }
    }

    /// The decision behind the current lane and session.
    pub fn route(&self) -> &PeltRoute {
        &self.route
    }

    pub fn document(&self) -> Option<&PeltController<F>> {
        match &self.lane {
            Lane::Document(controller) => Some(controller),
            Lane::Surface(_) => None,
        }
    }

    pub fn document_mut(&mut self) -> Option<&mut PeltController<F>> {
        match &mut self.lane {
            Lane::Document(controller) => Some(controller),
            Lane::Surface(_) => None,
        }
    }

    /// The held request a routed content reconstructs from.
    pub fn held_request(&self) -> Option<&PeltTileRequest> {
        self.routed.as_ref().map(|routed| &routed.request)
    }

    pub fn registries(&self) -> Option<&PeltRegistries<F>> {
        self.routed.as_ref().map(|routed| &routed.registries)
    }

    /// The document address, or the address a surface was opened at.
    pub fn address(&self) -> Option<&str> {
        match &self.lane {
            Lane::Document(controller) => Some(controller.address()),
            Lane::Surface(_) => self
                .routed
                .as_ref()
                .map(|routed| routed.request.request.address.as_str()),
        }
    }

    pub fn title(&self) -> Option<String> {
        self.document().and_then(PeltController::title)
    }

    pub fn session_generation(&self) -> Option<u64> {
        self.document().map(PeltController::session_generation)
    }

    pub fn session_identity(&self) -> Option<PeltSessionIdentity> {
        self.document().map(PeltController::session_identity)
    }

    /// Replace or clear the user engine choice. The held address and body
    /// stay, and the content is reconstructed through the same registries; a
    /// failed reconstruction leaves it unchanged.
    pub fn set_route_override(&mut self, engine_id: Option<String>) -> Result<bool, String> {
        self.refresh_held_request();
        let Some(routed) = &self.routed else {
            return Err("this content was built without capability routing".to_owned());
        };
        if routed.request.engine_override == engine_id {
            return Ok(false);
        }
        let mut request = routed.request.clone();
        request.engine_override = engine_id;
        *self = Self::routed(
            routed.registries.clone(),
            request,
            Arc::clone(&routed.clock_for),
        )?;
        Ok(true)
    }

    /// The active provider's declared capability and any structural report.
    /// A selected surface that fell back reports its document; a live surface
    /// reports only its registered engine's declared capability.
    pub fn inspection(&self) -> Option<PeltTileInspection> {
        match &self.lane {
            Lane::Document(controller) => Some(PeltTileInspection {
                capability: controller.a11y_capability(),
                report: controller.inspect(),
            }),
            Lane::Surface(_) => {
                let capability = self
                    .routed
                    .as_ref()?
                    .registries
                    .surfaces()
                    .engine(self.route.active_engine())?
                    .a11y_capability();
                Some(PeltTileInspection {
                    capability,
                    report: None,
                })
            },
        }
    }

    /// Produce this content's frame at host geometry. Document sessions take
    /// logical extents; surface producers take physical pixels.
    pub fn frame(&mut self, rect: WorkspaceRect, scale_factor: f32) -> PeltLayer<F> {
        match &mut self.lane {
            Lane::Document(controller) => {
                let (width, height) = rect.viewport();
                PeltLayer::Document(controller.frame(width, height))
            },
            Lane::Surface(surface) => PeltLayer::Surface(surface.frame(rect, scale_factor)),
        }
    }

    /// Advance session-owned time work. A surface has no settled bit, so it
    /// always asks for another poll.
    pub fn pump(&mut self) -> bool {
        match &mut self.lane {
            Lane::Document(controller) => controller.pump(),
            Lane::Surface(_) => true,
        }
    }

    pub fn mark_presented(&mut self) {
        if let Some(controller) = self.document_mut() {
            controller.mark_document_presented();
        }
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        if let Some(controller) = self.document_mut() {
            controller.set_hidden(hidden);
        }
    }

    pub fn focus(&mut self, focused: bool) {
        match &mut self.lane {
            Lane::Document(controller) => {
                let _ = controller.input(SessionInput::Focus(focused));
            },
            Lane::Surface(surface) if focused => surface.focus(),
            Lane::Surface(_) => {},
        }
    }

    /// Route neutral input in content-local coordinates.
    pub fn input(&mut self, input: SessionInput, scale_factor: f32) -> PeltHostEffect {
        match &mut self.lane {
            Lane::Document(controller) => {
                let effect = controller.input(input);
                self.after_document(effect)
            },
            Lane::Surface(surface) => surface_input(surface, input, scale_factor),
        }
    }

    pub fn scroll_at(&mut self, x: f32, y: f32, dx: f32, dy: f32, scale_factor: f32) -> bool {
        match &mut self.lane {
            Lane::Document(controller) => controller.scroll_at(x, y, dx, dy),
            Lane::Surface(surface) => {
                surface
                    .mouse(MouseEvent {
                        position: PhysicalPosition {
                            x: x * scale_factor,
                            y: y * scale_factor,
                        },
                        button: None,
                        kind: MouseEventKind::ScrollPixels {
                            delta_x: dx * scale_factor,
                            delta_y: dy * scale_factor,
                        },
                    })
                    .handled
            },
        }
    }

    pub fn scroll_for_key(&mut self, key: SessionScrollKey) -> bool {
        self.document_mut()
            .is_some_and(|controller| controller.scroll_for_key(key))
    }

    /// One navigation command for whichever lane is live. A document lane
    /// whose new load routes to a surface swaps lanes; a surface lane asked
    /// for an address that routes to a document swaps back.
    pub fn command(&mut self, command: SessionNavigationCommand) -> PeltHostEffect {
        match &mut self.lane {
            Lane::Document(controller) => {
                let effect = controller.command(command);
                self.after_document(effect)
            },
            Lane::Surface(_) => self.surface_command(command),
        }
    }

    /// Drain the next ordered web event from a surface lane. A document lane
    /// returns `Ok(None)`.
    pub fn poll_web_event(&mut self) -> Result<Option<WebSurfaceEvent>, String> {
        let Lane::Surface(surface) = &mut self.lane else {
            return Ok(None);
        };
        surface
            .producer
            .as_web_surface()
            .ok_or_else(|| "surface has no web event plane".to_owned())
            .map(|web| web.poll_web_event())
    }

    /// Move a live surface producer to another native host. See
    /// [`crate::PeltWorkspace::rehost_surface`].
    ///
    /// # Safety
    /// `host` must name a live native host the producer may attach to.
    pub unsafe fn rehost_surface(
        &mut self,
        host: NativeSurfaceHost,
    ) -> Result<(), inker::SurfaceError> {
        match &mut self.lane {
            Lane::Surface(surface) => unsafe { surface.producer.rehost(host) },
            Lane::Document(_) => Err(inker::SurfaceError::Unsupported(
                "content has no live surface producer".to_owned(),
            )),
        }
    }

    pub fn take_load_commands(&mut self) -> Vec<PeltLoadCommand> {
        self.document_mut()
            .map(PeltController::take_load_commands)
            .unwrap_or_default()
    }

    pub fn accept_progress(
        &mut self,
        progress: PageProgress,
        admitted_at_ms: u64,
    ) -> PeltHostEffect {
        match self.document_mut() {
            Some(controller) => {
                let effect = controller.accept_progress(progress, admitted_at_ms);
                self.after_document(effect)
            },
            None => PeltHostEffect::default(),
        }
    }

    pub fn accept_outcome(&mut self, outcome: FetchOutcome, admitted_at_ms: u64) -> PeltHostEffect {
        match self.document_mut() {
            Some(controller) => {
                let effect = controller.accept_outcome(outcome, admitted_at_ms);
                self.after_document(effect)
            },
            None => PeltHostEffect::default(),
        }
    }

    /// Keep the held request current with the live document: its address and,
    /// when the session retained its source response, that body and media
    /// type, so a reconstruction never refetches.
    pub(crate) fn refresh_held_request(&mut self) {
        let Lane::Document(controller) = &self.lane else {
            return;
        };
        let Some(routed) = &mut self.routed else {
            return;
        };
        let request = controller.request().clone();
        let source = request.body.is_none().then(|| {
            controller.clip().and_then(|clip| {
                clip.artifacts
                    .into_iter()
                    .find(|artifact| artifact.role == DocumentClipArtifactRole::SourceResponse)
            })
        });
        routed.request.request = request;
        if let Some(source) = source.flatten() {
            routed.request.request.address =
                retained_source_address(&source.canonical_uri, &routed.request.request.address);
            routed.request.request.body = Some(String::from_utf8_lossy(&source.bytes).into_owned());
            routed.request.request.content_type = Some(source.media_type.clone());
        }
    }

    /// Track the document's route and act on a surface reroute.
    fn after_document(&mut self, mut effect: PeltHostEffect) -> PeltHostEffect {
        if let Some(route) = self.document().and_then(PeltController::route) {
            self.route = route.clone();
        }
        if let Some(reroute) = effect.reroute.take() {
            self.swap_lane(reroute, &mut effect);
        }
        effect
    }

    fn surface_command(&mut self, command: SessionNavigationCommand) -> PeltHostEffect {
        if let SessionNavigationCommand::Address(address) = &command
            && let Some(routed) = &self.routed
        {
            let viewport = routed.request.request.viewport;
            let request =
                SessionSpawnRequest::new(address.clone()).with_viewport(viewport.0, viewport.1);
            match routed.registries.choose(
                &request.address,
                None,
                routed.request.engine_override.as_deref(),
                false,
            ) {
                Ok(LoadRoute::Document { route, .. }) => {
                    let route = route.expect("routed choices carry their route");
                    let mut effect = PeltHostEffect::default();
                    self.swap_lane(PeltReroute { request, route }, &mut effect);
                    return effect;
                },
                Ok(LoadRoute::Surface(_)) => {},
                Err(error) => {
                    return PeltHostEffect {
                        error: Some(error),
                        ..PeltHostEffect::default()
                    };
                },
            }
        }
        let Lane::Surface(surface) = &mut self.lane else {
            unreachable!("surface_command runs on a surface lane");
        };
        let web = surface.producer.as_web_surface();
        let result = web
            .ok_or_else(|| "surface has no web navigation plane".to_owned())
            .and_then(|web| {
                match command {
                    SessionNavigationCommand::Address(address) => web.navigate_to_url(&address),
                    SessionNavigationCommand::Reload => web.reload(),
                    SessionNavigationCommand::Back => web.go_back(),
                    SessionNavigationCommand::Forward => web.go_forward(),
                    SessionNavigationCommand::Stop => web.stop(),
                }
                .map_err(|error| error.to_string())
            });
        match result {
            Ok(()) => PeltHostEffect {
                handled: true,
                redraw: true,
                navigated: true,
                ..Default::default()
            },
            Err(error) => PeltHostEffect {
                error: Some(error),
                ..Default::default()
            },
        }
    }

    /// Open `reroute` in its lane and replace the current one. A failure
    /// keeps the current lane and reports the error. History does not cross
    /// lanes: each lane keeps its own.
    fn swap_lane(&mut self, reroute: PeltReroute, effect: &mut PeltHostEffect) {
        let Some(routed) = &self.routed else {
            effect.error = Some("this content was built without capability routing".to_owned());
            return;
        };
        let mut request = routed.request.clone();
        request.request = reroute.request;
        let choice = match reroute.route.state {
            crate::PeltRouteState::Surface => LoadRoute::Surface(reroute.route),
            _ => LoadRoute::Document {
                engine_id: reroute.route.active_engine().to_owned(),
                route: Some(reroute.route),
            },
        };
        match open_lane(&routed.registries, &request, &routed.clock_for, choice) {
            Ok((lane, route)) => {
                self.lane = lane;
                self.route = route;
                if let Some(routed) = &mut self.routed {
                    routed.request = request;
                }
                effect.handled = true;
                effect.redraw = true;
                effect.navigated = true;
                effect.error = None;
            },
            Err(error) => {
                effect.handled = true;
                effect.redraw = true;
                effect.navigated = false;
                effect.error = Some(error);
            },
        }
    }
}

/// Open the lane `choice` names for `request`.
fn open_lane<F: 'static>(
    registries: &PeltRegistries<F>,
    request: &PeltTileRequest,
    clock_for: &Arc<dyn Fn() -> Box<dyn PeltClock>>,
    choice: LoadRoute,
) -> Result<(Lane<F>, PeltRoute), String> {
    match choice {
        LoadRoute::Document { engine_id, route } => {
            let route = route.expect("routed choices carry their route");
            let mut config = PeltControllerConfig::from_request(engine_id, request.request.clone());
            if registries.host_loading() {
                config = config.with_host_loading();
            }
            let mut controller = PeltController::new_shared_boxed(
                registries.sessions_arc(),
                registries.surfaces_arc(),
                config,
                clock_for(),
            )?;
            controller.set_router(
                PeltRouter {
                    registries: registries.clone(),
                    engine_override: request.engine_override.clone(),
                },
                route.clone(),
            );
            Ok((Lane::Document(controller), route))
        },
        LoadRoute::Surface(route) => {
            let viewport = request.request.viewport;
            let spawn = SurfaceSpawnRequest {
                url: request.request.address.clone(),
                width: viewport.0,
                height: viewport.1,
                profile: request
                    .surface_profile
                    .clone()
                    .unwrap_or_else(|| registries.surface_profile().clone()),
                fence_handle: None,
            };
            let producer = registries
                .surfaces()
                .spawn(&route.decision, &spawn)
                .map_err(|error| {
                    format!(
                        "could not spawn surface {}: {error}",
                        route.selected_engine()
                    )
                })?;
            Ok((
                Lane::Surface(PeltSurface {
                    producer,
                    viewport,
                    offset: None,
                }),
                route,
            ))
        },
    }
}

fn retained_source_address(canonical_uri: &str, requested_address: &str) -> String {
    if canonical_uri.contains('#') {
        return canonical_uri.to_owned();
    }
    requested_address.split_once('#').map_or_else(
        || canonical_uri.to_owned(),
        |(_, fragment)| format!("{canonical_uri}#{fragment}"),
    )
}

/// A live composited surface producer and the geometry it was last given.
pub(crate) struct PeltSurface {
    pub(crate) producer: Box<dyn SurfaceProducer>,
    viewport: (u32, u32),
    offset: Option<(i32, i32)>,
}

impl PeltSurface {
    fn frame(
        &mut self,
        rect: WorkspaceRect,
        scale_factor: f32,
    ) -> Result<Option<SurfaceFrame>, String> {
        let viewport = (
            physical_extent(rect.width, scale_factor),
            physical_extent(rect.height, scale_factor),
        );
        if viewport != self.viewport {
            self.producer
                .resize(viewport.0, viewport.1)
                .map_err(|error| format!("surface resize failed: {error}"))?;
            self.viewport = viewport;
        }
        let offset = (
            physical_offset(rect.x, scale_factor),
            physical_offset(rect.y, scale_factor),
        );
        if Some(offset) != self.offset {
            self.producer
                .set_offset(offset.0, offset.1)
                .map_err(|error| format!("surface placement failed: {error}"))?;
            self.offset = Some(offset);
        }
        self.producer
            .acquire_frame()
            .map_err(|error| format!("surface frame failed: {error}"))
    }

    fn mouse(&mut self, event: MouseEvent) -> PeltHostEffect {
        match self.producer.send_mouse_input(event) {
            Ok(()) => PeltHostEffect {
                handled: true,
                redraw: true,
                ..Default::default()
            },
            Err(error) => PeltHostEffect {
                error: Some(format!("surface input failed: {error}")),
                ..Default::default()
            },
        }
    }

    fn keyboard(&mut self, event: KeyboardEvent) -> PeltHostEffect {
        match self.producer.send_keyboard_input(event) {
            Ok(()) => PeltHostEffect {
                handled: true,
                redraw: true,
                ..Default::default()
            },
            Err(error) => PeltHostEffect {
                error: Some(format!("surface input failed: {error}")),
                ..Default::default()
            },
        }
    }

    fn focus(&mut self) {
        let _ = self.producer.move_focus(FocusReason::Programmatic);
    }
}

fn surface_input(
    surface: &mut PeltSurface,
    input: SessionInput,
    scale_factor: f32,
) -> PeltHostEffect {
    match input {
        SessionInput::PointerMoved { x, y, .. } => surface.mouse(MouseEvent {
            position: PhysicalPosition {
                x: x * scale_factor,
                y: y * scale_factor,
            },
            button: None,
            kind: MouseEventKind::Moved,
        }),
        SessionInput::PointerButton {
            x,
            y,
            button,
            state,
            ..
        } => surface.mouse(MouseEvent {
            position: PhysicalPosition {
                x: x * scale_factor,
                y: y * scale_factor,
            },
            button: Some(match button {
                SessionPointerButton::Primary => MouseButton::Left,
                SessionPointerButton::Secondary => MouseButton::Right,
                SessionPointerButton::Auxiliary => MouseButton::Middle,
            }),
            kind: match state {
                SessionButtonState::Pressed => MouseEventKind::Pressed,
                SessionButtonState::Released => MouseEventKind::Released,
            },
        }),
        SessionInput::Key {
            key,
            state,
            modifiers,
            ..
        } => surface.keyboard(KeyboardEvent {
            key_code: surface_key_code(&key),
            scan_code: 0,
            modifiers: KeyboardModifiers {
                shift: modifiers.shift,
                ctrl: modifiers.control,
                alt: modifiers.alt,
                meta: modifiers.meta,
            },
            pressed: state == SessionButtonState::Pressed,
            text: match (state, key) {
                (SessionButtonState::Pressed, SessionKey::Character(text)) => Some(text),
                (SessionButtonState::Pressed, SessionKey::Space) => Some(" ".to_owned()),
                _ => None,
            },
        }),
        SessionInput::Text(text) => surface.keyboard(KeyboardEvent {
            key_code: 0,
            scan_code: 0,
            modifiers: KeyboardModifiers::default(),
            pressed: true,
            text: Some(text),
        }),
        SessionInput::Focus(true) => {
            surface.focus();
            PeltHostEffect {
                handled: true,
                ..Default::default()
            }
        },
        SessionInput::Focus(false)
        | SessionInput::FocusMove(_)
        | SessionInput::Ime(_)
        | SessionInput::Cancel => PeltHostEffect::default(),
    }
}

fn surface_key_code(key: &SessionKey) -> u32 {
    match key {
        SessionKey::Enter => 13,
        SessionKey::Tab => 9,
        SessionKey::Backspace => 8,
        SessionKey::Delete => 46,
        SessionKey::Escape => 27,
        SessionKey::Space => 32,
        SessionKey::ArrowLeft => 37,
        SessionKey::ArrowUp => 38,
        SessionKey::ArrowRight => 39,
        SessionKey::ArrowDown => 40,
        SessionKey::Home => 36,
        SessionKey::End => 35,
        SessionKey::PageUp => 33,
        SessionKey::PageDown => 34,
        SessionKey::Character(_) | SessionKey::Unidentified => 0,
    }
}

fn physical_extent(logical: f32, scale_factor: f32) -> u32 {
    ((logical.max(1.0) * scale_factor.max(1.0)).round() as u32).max(1)
}

fn physical_offset(logical: f32, scale_factor: f32) -> i32 {
    (logical * scale_factor.max(1.0)).round() as i32
}
