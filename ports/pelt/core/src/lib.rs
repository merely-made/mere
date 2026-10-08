/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Window-neutral Pelt host controller.

mod content;
mod surface_policy;
mod workspace;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use genet_host_api::resolve_href;
use inker::{
    A11yCapability, ContentReport, DocumentA11yActionRequest, DocumentA11yClickTarget,
    DocumentA11yNodeId, DocumentA11yProjection, DocumentSession, DocumentZoomState, SessionEffect,
    SessionError, SessionFormMethod, SessionInput, SessionInputResult, SessionNavigationCommand,
    SessionRegistry, SessionScrollKey, SessionSpawnRequest, SurfaceEngineRegistry,
};

pub use content::{PeltContent, PeltLane, PeltLayer};
pub use page_load::{
    FetchFailure, FetchOutcome, FetchRequestId, Fetched, LoadPhase, LoadedDocument, PageProgress,
};
use page_load::{LoadAnswer, PageLoad};
pub use surface_policy::SurfaceResourcePolicy;
use workspace::LoadRoute;
pub use workspace::{
    PeltRegistries, PeltRoute, PeltRouteSource, PeltRouteState, PeltSurfaceLayer, PeltTileFrame,
    PeltTileInspection, PeltTileRequest, PeltTileRoute, PeltWorkspace, PeltWorkspaceFrame,
    PeltWorkspaceOutcome, WorkspaceRect,
};

/// Host-neutral state for a controller's document presentation.
///
/// [`Self::Loading`] does not describe transport progress. It records that a
/// replacement session needs one host-composed frame before the host can call
/// [`PeltController::mark_document_presented`]. Transport progress is
/// [`Self::Fetching`], which only a host-loading controller enters; the
/// current session stays on screen while it lasts. A failed replacement leaves
/// the current session and history intact while exposing the attempted address
/// and error to the host's own diagnostic document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeltDocumentState {
    Ready,
    Loading {
        address: String,
    },
    Error {
        address: String,
        message: String,
    },
    /// The host's transport is fetching `address` for exact `request`.
    Fetching {
        address: String,
        request: FetchRequestId,
    },
    /// The transport needs the host before `address` can load: an input
    /// prompt, a client identity, or a changed certificate. The conversation
    /// and its stores are the host's; the current session stays intact.
    Awaiting {
        address: String,
        failure: FetchFailure,
    },
}

/// Who fetches a controller's top-level documents.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PeltLoadMode {
    /// The document engine loads through the fetcher it was registered with,
    /// synchronously inside its spawn. Pelt's original mode.
    #[default]
    Engine,
    /// The host's transport fetches. The controller queues
    /// [`PeltLoadCommand`]s (drained with
    /// [`PeltController::take_load_commands`]) and takes the answers back
    /// through [`PeltController::accept_progress`] and
    /// [`PeltController::accept_outcome`], gated on their exact request.
    Host,
}

/// Who keeps a controller's back/forward history.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PeltHistoryMode {
    /// The controller keeps a linear history: links push, Back and Forward
    /// traverse. Pelt's original mode.
    #[default]
    Linear,
    /// The host keeps history. The controller holds only its current entry:
    /// a link or GET form becomes a [`PeltNavigationRequest`] in the host
    /// effect and nothing loads, Back and Forward come back unhandled, and the
    /// host loads its chosen entry with [`PeltController::open`]. Turnstone's
    /// history is its graph: a link opens or mints a node.
    Host,
}

/// Why a document asked to navigate, so a host-history policy can choose a
/// disposition (this content, a new node, a new tile).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeltNavigationCause {
    /// A link the user activated, with the modifiers held at the time.
    Link { modifiers: inker::SessionModifiers },
    /// A GET form submission, its fields already in the address.
    FormGet,
}

/// A navigation a host-history controller hands to its host instead of
/// loading. The address is resolved against the current document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeltNavigationRequest {
    pub request: SessionSpawnRequest,
    pub cause: PeltNavigationCause,
}

/// One command for the host's transport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeltLoadCommand {
    Fetch {
        request: FetchRequestId,
        url: String,
    },
    /// Abort one exact request; the controller has already retired it.
    Cancel { request: FetchRequestId },
}

/// A new load whose address routes to a surface lane. The controller leaves
/// its session, history and state untouched; [`PeltContent`] swaps lanes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeltReroute {
    pub request: SessionSpawnRequest,
    pub route: PeltRoute,
}

/// The shared registries and sticky engine pin a routed controller consults
/// for every new load.
pub(crate) struct PeltRouter<F> {
    pub(crate) registries: PeltRegistries<F>,
    pub(crate) engine_override: Option<String>,
}

/// A response the controller will not render. The host stores it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeltDownload {
    pub url: String,
    pub fetched: Fetched,
}

/// Replace a live session's body in place while a transfer streams. Engines
/// that cannot do so return `false`; the controller then shows the complete
/// document when the transfer settles.
pub type PeltBodyReplacer<F> = fn(&mut dyn DocumentSession<F>, &str, &str) -> bool;

/// A host-loading controller's in-flight load and how it commits to history.
#[derive(Clone, Debug)]
struct PendingLoad {
    request: FetchRequestId,
    entry: SessionSpawnRequest,
    /// The engine a traversal or reload reopens with. A new navigation has
    /// none and is routed when its response arrives.
    engine: Option<String>,
    commit: HistoryCommit,
    /// A session for this load has been installed from a streamed prefix.
    live: bool,
}

/// One history entry and the engine it was shown with. Traversal and reload
/// reopen with that engine; only a new navigation is routed.
#[derive(Clone, Debug)]
struct HistoryEntry {
    engine_id: String,
    request: SessionSpawnRequest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HistoryCommit {
    Push,
    Replace,
    Traverse(usize),
}

/// Identity of one live Pelt document session. The controller instance never
/// repeats within this process; generation advances when that controller
/// replaces its retained session. Hosts retaining namespaced observations use
/// both values so a reconstructed controller cannot alias a stale node.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PeltSessionIdentity {
    pub instance_id: u64,
    pub generation: u64,
}

static NEXT_CONTROLLER_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

fn next_controller_instance_id() -> u64 {
    NEXT_CONTROLLER_INSTANCE_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .expect("Pelt controller instance IDs exhausted")
}

/// A caller-owned monotonic clock. Pelt asks for the current time only while
/// pumping a retained session; it neither selects a system clock nor owns an
/// event loop.
pub trait PeltClock: 'static {
    fn now_ms(&self) -> f64;
}

/// Initial engine and document request for one retained Pelt controller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeltControllerConfig {
    pub engine_id: String,
    pub request: SessionSpawnRequest,
    pub load_mode: PeltLoadMode,
    pub history_mode: PeltHistoryMode,
}

impl PeltControllerConfig {
    pub fn new(
        engine_id: impl Into<String>,
        address: impl Into<String>,
        viewport: (u32, u32),
    ) -> Self {
        Self::from_request(
            engine_id,
            SessionSpawnRequest::new(address).with_viewport(viewport.0, viewport.1),
        )
    }

    /// Preserve a caller-held body, content type, visibility, and viewport in
    /// the first spawn request. Reader hosts use this to supply fleeced source
    /// bytes without teaching the controller how to fetch them.
    pub fn from_request(engine_id: impl Into<String>, request: SessionSpawnRequest) -> Self {
        Self {
            engine_id: engine_id.into(),
            request,
            load_mode: PeltLoadMode::Engine,
            history_mode: PeltHistoryMode::Linear,
        }
    }

    /// Let the host's transport fetch top-level documents. A request without a
    /// held body opens on an empty document of the same engine and queues its
    /// first fetch.
    pub fn with_host_loading(mut self) -> Self {
        self.load_mode = PeltLoadMode::Host;
        self
    }

    /// Let the host keep history (see [`PeltHistoryMode::Host`]).
    pub fn with_host_history(mut self) -> Self {
        self.history_mode = PeltHistoryMode::Host;
        self
    }
}

/// Host work requested after session input or a navigation command.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeltHostEffect {
    pub handled: bool,
    pub redraw: bool,
    pub cursor: Option<inker::SessionCursor>,
    pub pointer_capture: Option<bool>,
    pub editable: bool,
    pub navigated: bool,
    pub error: Option<String>,
    /// A host-loading controller's response that is a download, not a page.
    pub download: Option<PeltDownload>,
    /// A routed controller's new load belongs to a surface lane.
    pub reroute: Option<PeltReroute>,
    /// A host-history controller's navigation, for the host to place.
    pub navigation: Option<PeltNavigationRequest>,
}

/// Pelt's reusable one-session browser controller.
///
/// Concrete document engines and their resource policy arrive inside
/// `session_engines`. The routed workspace shares both registry pairs across
/// its controllers and owns surface producers beside them. Frames remain
/// generic, so the embedding host owns every wgpu resource and presentation
/// target.
pub struct PeltController<F> {
    session_engines: Arc<SessionRegistry<F>>,
    surface_engines: Arc<SurfaceEngineRegistry>,
    engine_id: String,
    session: Box<dyn DocumentSession<F>>,
    history: Vec<HistoryEntry>,
    history_index: usize,
    viewport: (u32, u32),
    clock: Box<dyn PeltClock>,
    document_state: PeltDocumentState,
    instance_id: u64,
    session_generation: u64,
    load_mode: PeltLoadMode,
    history_mode: PeltHistoryMode,
    load: PageLoad,
    pending: Option<PendingLoad>,
    load_commands: Vec<PeltLoadCommand>,
    body_replacer: Option<PeltBodyReplacer<F>>,
    router: Option<PeltRouter<F>>,
    route: Option<PeltRoute>,
}

impl<F: 'static> PeltController<F> {
    pub fn new(
        session_engines: SessionRegistry<F>,
        surface_engines: SurfaceEngineRegistry,
        config: PeltControllerConfig,
        clock: impl PeltClock,
    ) -> Result<Self, String> {
        Self::new_shared(
            Arc::new(session_engines),
            Arc::new(surface_engines),
            config,
            clock,
        )
    }

    /// Spawn a controller from host-long-lived registries. Every tile keeps its
    /// own session and history while sharing the immutable engine factories.
    pub fn new_shared(
        session_engines: Arc<SessionRegistry<F>>,
        surface_engines: Arc<SurfaceEngineRegistry>,
        config: PeltControllerConfig,
        clock: impl PeltClock,
    ) -> Result<Self, String> {
        Self::new_shared_boxed(session_engines, surface_engines, config, Box::new(clock))
    }

    pub(crate) fn new_shared_boxed(
        session_engines: Arc<SessionRegistry<F>>,
        surface_engines: Arc<SurfaceEngineRegistry>,
        config: PeltControllerConfig,
        clock: Box<dyn PeltClock>,
    ) -> Result<Self, String> {
        let engine_id = config.engine_id;
        let viewport = config.request.viewport;
        // A host-loading controller without a held body opens on an empty
        // document of the same engine; the engine never fetches.
        let fetch_first = config.load_mode == PeltLoadMode::Host && config.request.body.is_none();
        let opening = if fetch_first {
            config.request.clone().with_body("")
        } else {
            config.request.clone()
        };
        let session = session_engines
            .spawn(&engine_id, &opening)
            .map_err(|error| format!("could not spawn engine {engine_id}: {error}"))?;
        let mut controller = Self {
            session_engines,
            surface_engines,
            history: vec![HistoryEntry {
                engine_id: engine_id.clone(),
                request: config.request,
            }],
            engine_id,
            session,
            history_index: 0,
            viewport,
            clock,
            document_state: PeltDocumentState::Ready,
            instance_id: next_controller_instance_id(),
            // Generation zero is reserved as "no successfully opened
            // session" for hosts that retain child trees across tiles.
            session_generation: 1,
            load_mode: config.load_mode,
            history_mode: config.history_mode,
            load: PageLoad::default(),
            pending: None,
            load_commands: Vec::new(),
            body_replacer: None,
            router: None,
            route: None,
        };
        if fetch_first {
            let entry = controller.history[0].clone();
            controller.begin_load(entry.request, Some(entry.engine_id), HistoryCommit::Replace);
        }
        Ok(controller)
    }

    pub fn load_mode(&self) -> PeltLoadMode {
        self.load_mode
    }

    pub fn history_mode(&self) -> PeltHistoryMode {
        self.history_mode
    }

    /// Load `request` as the current entry, replacing it rather than adding
    /// history. A held body opens directly, whichever loading mode is set;
    /// otherwise the request loads as any other (engine fetch or host
    /// transport). It is routed like any new load, so it may change engine or
    /// hand the content a surface reroute. A host-history host calls this for
    /// the entry its own history chose.
    pub fn open(&mut self, request: SessionSpawnRequest) -> PeltHostEffect {
        let mut host_effect = PeltHostEffect::default();
        let request = request.with_viewport(self.viewport.0, self.viewport.1);
        self.load_request(request, HistoryCommit::Replace, &mut host_effect);
        host_effect
    }

    /// Route every new load through shared registries, keeping `route` as
    /// the decision that opened the current session.
    pub(crate) fn set_router(&mut self, router: PeltRouter<F>, route: PeltRoute) {
        // The first host fetch began before routing was attached; its
        // response is routed by media type unless the engine is pinned.
        if let Some(pending) = self.pending.as_mut() {
            pending.engine = router.engine_override.clone();
        }
        self.router = Some(router);
        self.route = Some(route);
    }

    /// The routing decision behind the current session, for a routed
    /// controller.
    pub fn route(&self) -> Option<&PeltRoute> {
        self.route.as_ref()
    }

    /// Install how this host's engines replace a live body while a transfer
    /// streams. Without one, a streamed document shows its first prefix and
    /// then its complete body.
    pub fn set_body_replacer(&mut self, replacer: PeltBodyReplacer<F>) {
        self.body_replacer = Some(replacer);
    }

    /// Drain the commands queued for the host's transport.
    pub fn take_load_commands(&mut self) -> Vec<PeltLoadCommand> {
        std::mem::take(&mut self.load_commands)
    }

    /// The exact request a host-loading controller is waiting on.
    pub fn pending_request(&self) -> Option<FetchRequestId> {
        self.pending.as_ref().map(|pending| pending.request)
    }

    /// Transfer phase of the current or last host load.
    pub fn load_phase(&self) -> Option<&LoadPhase> {
        self.load.phase()
    }

    /// The retained response of the last host load and its request address.
    /// Source capture deposits these exact bytes.
    pub fn loaded_document(&self) -> Option<(&str, &LoadedDocument)> {
        self.load.document()
    }

    pub fn engine_id(&self) -> &str {
        &self.engine_id
    }

    pub fn address(&self) -> &str {
        &self.history[self.history_index].request.address
    }

    pub fn request(&self) -> &SessionSpawnRequest {
        &self.history[self.history_index].request
    }

    pub fn title(&self) -> Option<String> {
        self.session.inspect().and_then(|report| report.title)
    }

    pub fn inspect(&self) -> Option<ContentReport> {
        self.session.inspect()
    }

    /// The current host-neutral document presentation state.
    pub fn document_state(&self) -> &PeltDocumentState {
        &self.document_state
    }

    /// Controller-local generation of the current successfully opened session.
    ///
    /// This starts at one for the initial session and increases only after a
    /// replacement session has been successfully spawned and installed. Hosts
    /// retaining externally visible observations must use
    /// [`Self::session_identity`] instead: a reconstructed controller begins
    /// again at generation one.
    pub fn session_generation(&self) -> u64 {
        self.session_generation
    }

    /// Full identity of the current retained session.
    pub fn session_identity(&self) -> PeltSessionIdentity {
        PeltSessionIdentity {
            instance_id: self.instance_id,
            generation: self.session_generation,
        }
    }

    /// The active document session's concrete observation surface.
    ///
    /// Engine-specific behavior stays behind [`DocumentSession`]. This is for
    /// host-owned observation of a public concrete session type, paired with
    /// [`Self::session_identity`] so retained host state never aliases a
    /// successful replacement or reconstructed controller.
    pub fn session_as_any_ref(&self) -> &dyn std::any::Any {
        self.session.as_any_ref()
    }

    /// The active document session's narrowly scoped concrete mutation surface.
    ///
    /// Engine-specific behavior stays behind [`DocumentSession`]. A host that
    /// owns a typed action route pairs a downcast here with
    /// [`Self::session_identity`] so a stale retained node cannot mutate a
    /// replacement or reconstructed session.
    pub fn session_as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self.session.as_any()
    }

    /// Request an engine-owned page zoom through Pelt's neutral session seam.
    ///
    /// The active engine reports its supported bounds and applied factor. Pelt
    /// neither chooses a zoom policy nor reaches into an engine concrete type.
    pub fn set_page_zoom(&mut self, factor: f32) -> Result<DocumentZoomState, SessionError> {
        self.session.set_page_zoom(factor)
    }

    /// Mark a successfully replaced document as visibly composed by its host.
    ///
    /// Pelt deliberately does not choose a presentation loop. An embedding host
    /// calls this only after it has composed the replacement session, preserving
    /// one deterministic loading-document frame without pretending the current
    /// synchronous registry spawn is asynchronous transport.
    pub fn mark_document_presented(&mut self) {
        if matches!(self.document_state, PeltDocumentState::Loading { .. }) {
            self.document_state = PeltDocumentState::Ready;
        }
    }

    /// The semantic capability declared by this controller's active document
    /// engine. Construction requires that engine to remain in the immutable
    /// shared registry, so this cannot silently degrade after a session opens.
    pub fn a11y_capability(&self) -> A11yCapability {
        self.session_engines
            .get(&self.engine_id)
            .expect("a live Pelt controller keeps its registered session engine")
            .a11y_capability()
    }

    /// The active session's renderer-neutral accessibility projection.
    ///
    /// Its support metadata describes the currently retained semantics more
    /// precisely than the engine registration's pre-spawn capability.
    pub fn accessibility_projection(&self) -> Option<DocumentA11yProjection> {
        self.session.accessibility_projection()
    }

    /// Revalidate a projected clickable target against the live session.
    /// Pelt's embedding host applies the returned point through its ordinary
    /// pointer route, preserving session and navigation custody.
    pub fn accessibility_click_target(
        &self,
        target: DocumentA11yNodeId,
    ) -> Option<DocumentA11yClickTarget> {
        self.session.accessibility_click_target(target)
    }

    /// Dispatch one non-pointer accessibility action through the active
    /// document session after the host has checked its tile/session identity.
    pub fn dispatch_accessibility_action(&mut self, request: &DocumentA11yActionRequest) -> bool {
        self.session.dispatch_accessibility_action(request)
    }

    /// Semantic clip from the current retained selection or document.
    ///
    /// Product receipts use this to verify engine-owned selection without
    /// reaching through the controller to a concrete document session.
    pub fn clip(&self) -> Option<inker::DocumentClip> {
        self.session.clip()
    }

    /// Links in the current retained frame, in document-local coordinates.
    /// Product hosts use this for semantic receipts and accessibility-driven
    /// activation without reaching through the controller to a concrete engine.
    pub fn links(&self) -> Vec<inker::SessionLink> {
        self.session.links()
    }

    /// Resolve retained text to document-local pointer endpoints without
    /// exposing the concrete document engine's DOM or layout identities.
    pub fn text_target(&self, text: &str) -> Option<inker::SessionTextTarget> {
        self.session.text_target(text)
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn session_engines(&self) -> &SessionRegistry<F> {
        &self.session_engines
    }

    /// Mutate an owned registry before sharing it. Returns `None` once another
    /// controller shares the registry.
    pub fn session_engines_mut(&mut self) -> Option<&mut SessionRegistry<F>> {
        Arc::get_mut(&mut self.session_engines)
    }

    pub fn surface_engines(&self) -> &SurfaceEngineRegistry {
        &self.surface_engines
    }

    /// Mutate an owned registry before sharing it. See
    /// [`Self::session_engines_mut`] for the routed-workspace rule.
    pub fn surface_engines_mut(&mut self) -> Option<&mut SurfaceEngineRegistry> {
        Arc::get_mut(&mut self.surface_engines)
    }

    /// Whether two live controllers use the same host registry pair.
    pub fn shares_registries_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.session_engines, &other.session_engines)
            && Arc::ptr_eq(&self.surface_engines, &other.surface_engines)
    }

    /// Advance session-owned time work using the injected clock. `true`
    /// requests another frame because the session has not settled.
    pub fn pump(&mut self) -> bool {
        self.session.pump(self.clock.now_ms());
        !self.session.settled()
    }

    pub fn frame(&mut self, width: u32, height: u32) -> F {
        self.viewport = (width.max(1), height.max(1));
        self.session.frame(self.viewport.0, self.viewport.1)
    }

    pub fn scroll_by(&mut self, dx: f32, dy: f32) -> bool {
        self.session.scroll_by(dx, dy)
    }

    pub fn scroll_at(&mut self, x: f32, y: f32, dx: f32, dy: f32) -> bool {
        self.session.scroll_at(x, y, dx, dy)
    }

    pub fn scroll_for_key(&mut self, key: SessionScrollKey) -> bool {
        self.session.scroll_for_key(key)
    }

    /// Tell the retained session whether its tile is currently visible.
    /// Hidden sessions keep their navigation and scroll state while avoiding
    /// work that only contributes to a visible frame.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.session.set_hidden(hidden);
    }

    pub fn input(&mut self, input: SessionInput) -> PeltHostEffect {
        let modifiers = input_modifiers(&input);
        let SessionInputResult {
            effect,
            cursor,
            capture,
            editable,
        } = self.session.input(input);
        let mut host_effect = PeltHostEffect {
            handled: effect.is_handled(),
            redraw: matches!(effect, SessionEffect::Handled | SessionEffect::Cancelled),
            cursor,
            pointer_capture: capture,
            editable,
            ..PeltHostEffect::default()
        };
        match effect {
            SessionEffect::Navigate(target) => self.follow(
                target,
                PeltNavigationCause::Link { modifiers },
                &mut host_effect,
            ),
            SessionEffect::Submit(submission) => match submission.method {
                SessionFormMethod::Get => {
                    let target = get_submission_target(&submission.action, &submission.fields);
                    self.follow(target, PeltNavigationCause::FormGet, &mut host_effect);
                },
                SessionFormMethod::Post => {
                    let address = self.address().to_owned();
                    self.document_error(
                        address,
                        "POST form submission needs an injected request-body transport".to_owned(),
                        &mut host_effect,
                    );
                },
            },
            SessionEffect::Ignored | SessionEffect::Handled | SessionEffect::Cancelled => {},
        }
        host_effect
    }

    pub fn command(&mut self, command: SessionNavigationCommand) -> PeltHostEffect {
        if self.history_mode == PeltHistoryMode::Host {
            match command {
                // The host's history traverses; this controller has none.
                SessionNavigationCommand::Back | SessionNavigationCommand::Forward => {
                    return PeltHostEffect::default();
                },
                // The host's own address request opens in place.
                SessionNavigationCommand::Address(address) => {
                    let target = resolve_href(self.address(), &address);
                    return self.open(SessionSpawnRequest::new(target));
                },
                SessionNavigationCommand::Reload | SessionNavigationCommand::Stop => {},
            }
        }
        if self.load_mode == PeltLoadMode::Host {
            return self.host_load_command(command);
        }
        let mut host_effect = PeltHostEffect::default();
        match command {
            SessionNavigationCommand::Address(address) => {
                self.navigate_effect(address, &mut host_effect);
            },
            SessionNavigationCommand::Reload => {
                let HistoryEntry { engine_id, request } = self.history[self.history_index].clone();
                match self.spawn_with(&engine_id, &request) {
                    Ok(session) => {
                        self.install_session_from(engine_id, session);
                        self.document_state = PeltDocumentState::Loading {
                            address: request.address.clone(),
                        };
                        host_effect.handled = true;
                        host_effect.redraw = true;
                        host_effect.navigated = true;
                    },
                    Err(error) => self.document_error(request.address, error, &mut host_effect),
                }
            },
            SessionNavigationCommand::Back => {
                if self.can_go_back() {
                    self.traverse_to(self.history_index - 1, &mut host_effect);
                }
            },
            SessionNavigationCommand::Forward => {
                if self.can_go_forward() {
                    self.traverse_to(self.history_index + 1, &mut host_effect);
                }
            },
            SessionNavigationCommand::Stop => {
                // The current registry spawn contract is synchronous. Stop is
                // consumed here; cancellable transport remains a separate seam.
                host_effect.handled = true;
            },
        }
        host_effect
    }

    /// Take one streamed fragment for the pending load. The first renderable
    /// prefix opens the document and commits it to history; later prefixes
    /// replace its body in place where the engine can.
    pub fn accept_progress(
        &mut self,
        progress: PageProgress,
        admitted_at_ms: u64,
    ) -> PeltHostEffect {
        let mut host_effect = PeltHostEffect::default();
        let Some(pending) = self
            .pending
            .clone()
            .filter(|pending| pending.request == progress.request)
        else {
            return host_effect;
        };
        let url = progress.url.clone();
        if self
            .load
            .accept_progress(progress, admitted_at_ms)
            .is_none()
        {
            return host_effect;
        }
        let document = self
            .load
            .fetched(&url)
            .cloned()
            .expect("an accepted fragment retains its prefix");
        if pending.live {
            host_effect.redraw = self.replace_live_body(&url, &document.body);
            host_effect.handled = host_effect.redraw;
            return host_effect;
        }
        match self.open_loaded(&pending, &document) {
            Ok(()) => {
                if let Some(pending) = self.pending.as_mut() {
                    pending.live = true;
                }
                self.document_state = PeltDocumentState::Loading { address: url };
                host_effect.handled = true;
                host_effect.redraw = true;
                host_effect.navigated = true;
            },
            Err(error) => {
                // The prefix could not open; the complete body gets one more
                // attempt when the transfer settles.
                host_effect.error = Some(error);
            },
        }
        host_effect
    }

    /// Take the pending load's terminal answer. Answers for any other request
    /// change nothing.
    pub fn accept_outcome(&mut self, outcome: FetchOutcome, admitted_at_ms: u64) -> PeltHostEffect {
        let mut host_effect = PeltHostEffect::default();
        if self.pending_request() != Some(outcome.request) {
            return host_effect;
        }
        let pending = self.pending.take().expect("checked pending request");
        let url = outcome.url.clone();
        match self.load.accept_outcome(outcome, admitted_at_ms) {
            LoadAnswer::Stale => {},
            LoadAnswer::Document => {
                let document = self
                    .load
                    .fetched(&url)
                    .cloned()
                    .expect("a settled document is retained");
                if pending.live && self.replace_live_body(&url, &document.body) {
                    host_effect.handled = true;
                    host_effect.redraw = true;
                    return host_effect;
                }
                match self.open_loaded(&pending, &document) {
                    Ok(()) => {
                        self.document_state = PeltDocumentState::Loading { address: url };
                        host_effect.handled = true;
                        host_effect.redraw = true;
                        host_effect.navigated = !pending.live;
                    },
                    Err(error) => self.document_error(url, error, &mut host_effect),
                }
            },
            LoadAnswer::Download(fetched) => {
                self.settle_document_state(pending.live);
                host_effect.handled = true;
                host_effect.download = Some(PeltDownload { url, fetched });
            },
            LoadAnswer::Failed(FetchFailure::Failed(message)) => {
                self.document_error(url, message, &mut host_effect);
            },
            LoadAnswer::Failed(FetchFailure::Cancelled) => {
                self.settle_document_state(pending.live);
                host_effect.redraw = true;
            },
            LoadAnswer::Failed(failure) => {
                host_effect.error = Some(failure.to_string());
                host_effect.handled = true;
                host_effect.redraw = true;
                self.document_state = PeltDocumentState::Awaiting {
                    address: url,
                    failure,
                };
            },
        }
        host_effect
    }

    fn host_load_command(&mut self, command: SessionNavigationCommand) -> PeltHostEffect {
        let mut host_effect = PeltHostEffect::default();
        match command {
            SessionNavigationCommand::Address(address) => {
                self.navigate_effect(address, &mut host_effect);
            },
            SessionNavigationCommand::Reload => {
                // A reload is a new request for the same entry: the retained
                // body is dropped so the transport fetches it again.
                let entry = self.history[self.history_index].clone();
                self.load.forget_fetched();
                self.begin_load(entry.request, Some(entry.engine_id), HistoryCommit::Replace);
                host_effect.handled = true;
                host_effect.redraw = true;
            },
            SessionNavigationCommand::Back => {
                if self.can_go_back() {
                    self.begin_traverse(self.history_index - 1, &mut host_effect);
                }
            },
            SessionNavigationCommand::Forward => {
                if self.can_go_forward() {
                    self.begin_traverse(self.history_index + 1, &mut host_effect);
                }
            },
            SessionNavigationCommand::Stop => {
                if let Some(pending) = self.pending.take() {
                    if let Some(request) = self.load.stop_active() {
                        self.load_commands.push(PeltLoadCommand::Cancel { request });
                    }
                    self.settle_document_state(pending.live);
                    host_effect.redraw = true;
                }
                host_effect.handled = true;
            },
        }
        host_effect
    }

    fn begin_traverse(&mut self, index: usize, host_effect: &mut PeltHostEffect) {
        let entry = self.history[index].clone();
        self.begin_load(
            entry.request,
            Some(entry.engine_id),
            HistoryCommit::Traverse(index),
        );
        host_effect.handled = true;
        host_effect.redraw = true;
        host_effect.editable = false;
    }

    /// Start one host fetch, superseding any load still in flight.
    fn begin_load(
        &mut self,
        mut entry: SessionSpawnRequest,
        engine: Option<String>,
        commit: HistoryCommit,
    ) {
        entry.body = None;
        entry.content_type = None;
        let request = page_load::next_fetch_request_id();
        if let Some(superseded) = self.load.begin(request) {
            self.load_commands.push(PeltLoadCommand::Cancel {
                request: superseded,
            });
        }
        self.load_commands.push(PeltLoadCommand::Fetch {
            request,
            url: entry.address.clone(),
        });
        self.document_state = PeltDocumentState::Fetching {
            address: entry.address.clone(),
            request,
        };
        self.pending = Some(PendingLoad {
            request,
            entry,
            engine,
            commit,
            live: false,
        });
    }

    /// Open a host-held document for `pending` and commit it to history.
    fn open_loaded(
        &mut self,
        pending: &PendingLoad,
        document: &LoadedDocument,
    ) -> Result<(), String> {
        let mut request = pending.entry.clone().with_body(document.body.clone());
        request.content_type = document.content_type.clone();
        let (engine_id, route) = match &pending.engine {
            Some(engine_id) => (engine_id.clone(), None),
            None => self.document_route(&request)?,
        };
        let session = self.spawn_with(&engine_id, &request)?;
        self.install_session_from(engine_id.clone(), session);
        if route.is_some() {
            self.route = route;
        }
        if !pending.live {
            let entry = HistoryEntry {
                engine_id,
                request: pending.entry.clone(),
            };
            self.commit_entry(entry, pending.commit);
        }
        Ok(())
    }

    fn commit_entry(&mut self, entry: HistoryEntry, commit: HistoryCommit) {
        match commit {
            HistoryCommit::Push => {
                self.history.truncate(self.history_index + 1);
                self.history.push(entry);
                self.history_index += 1;
            },
            HistoryCommit::Replace => self.history[self.history_index] = entry,
            HistoryCommit::Traverse(index) => {
                self.history[index] = entry;
                self.history_index = index;
            },
        }
    }

    /// Retire a host load in flight, cancelling its exact request.
    fn abandon_pending(&mut self) {
        if self.pending.take().is_some()
            && let Some(request) = self.load.stop_active()
        {
            self.load_commands.push(PeltLoadCommand::Cancel { request });
        }
    }

    fn pinned_engine(&self) -> Option<String> {
        self.router
            .as_ref()
            .and_then(|router| router.engine_override.clone())
    }

    fn replace_live_body(&mut self, url: &str, body: &str) -> bool {
        self.body_replacer
            .is_some_and(|replace| replace(self.session.as_mut(), url, body))
    }

    /// After a load ends without opening anything more, the current session
    /// stands. A streamed prefix already set its own presentation state, which
    /// the host may since have presented, so only a load that never opened
    /// leaves `Fetching` here.
    fn settle_document_state(&mut self, live: bool) {
        if !live {
            self.document_state = PeltDocumentState::Ready;
        }
    }

    /// A document's own navigation: placed by the host under host history,
    /// pushed onto this controller's history otherwise.
    fn follow(
        &mut self,
        target: String,
        cause: PeltNavigationCause,
        host_effect: &mut PeltHostEffect,
    ) {
        if self.history_mode == PeltHistoryMode::Host {
            let target = resolve_href(self.address(), &target);
            host_effect.handled = true;
            host_effect.navigation = Some(PeltNavigationRequest {
                request: SessionSpawnRequest::new(target)
                    .with_viewport(self.viewport.0, self.viewport.1),
                cause,
            });
            return;
        }
        self.navigate_effect(target, host_effect);
    }

    fn navigate_effect(&mut self, target: String, host_effect: &mut PeltHostEffect) {
        let target = resolve_href(self.address(), &target);
        let request =
            SessionSpawnRequest::new(target).with_viewport(self.viewport.0, self.viewport.1);
        self.load_request(request, HistoryCommit::Push, host_effect);
    }

    /// Route and load one new request, committing it to history as `commit`
    /// says once it opens.
    fn load_request(
        &mut self,
        request: SessionSpawnRequest,
        commit: HistoryCommit,
        host_effect: &mut PeltHostEffect,
    ) {
        let choice = match self.route_load(&request) {
            Ok(choice) => choice,
            Err(error) => return self.document_error(request.address, error, host_effect),
        };
        let (engine_id, route) = match choice {
            LoadRoute::Surface(route) => {
                // Another lane presents this address. The owner of this
                // controller swaps lanes; nothing here changes.
                host_effect.handled = true;
                host_effect.redraw = true;
                host_effect.reroute = Some(PeltReroute { request, route });
                return;
            },
            LoadRoute::Document { engine_id, route } => (engine_id, route),
        };
        if request.body.is_some() {
            // A held body opens now; any transfer still in flight is
            // superseded. Its media type may choose another document engine.
            self.abandon_pending();
            let (engine_id, route) = match &self.router {
                Some(_) if self.pinned_engine().is_none() => match self.document_route(&request) {
                    Ok(choice) => choice,
                    Err(error) => {
                        return self.document_error(request.address, error, host_effect);
                    },
                },
                _ => (engine_id, route),
            };
            return self.spawn_and_commit(engine_id, route, request, commit, host_effect);
        }
        if self.load_mode == PeltLoadMode::Host {
            // The response's media type may still choose another document
            // engine, so only an explicit pin is carried to the open.
            let pinned = self
                .pinned_engine()
                .or_else(|| self.router.is_none().then(|| engine_id.clone()));
            self.begin_load(request, pinned, commit);
            host_effect.handled = true;
            host_effect.redraw = true;
            host_effect.editable = false;
            return;
        }
        self.spawn_and_commit(engine_id, route, request, commit, host_effect);
    }

    fn spawn_and_commit(
        &mut self,
        engine_id: String,
        route: Option<PeltRoute>,
        request: SessionSpawnRequest,
        commit: HistoryCommit,
        host_effect: &mut PeltHostEffect,
    ) {
        match self.spawn_with(&engine_id, &request) {
            Ok(session) => {
                let address = request.address.clone();
                self.install_session_from(engine_id.clone(), session);
                if route.is_some() {
                    self.route = route;
                }
                self.commit_entry(HistoryEntry { engine_id, request }, commit);
                self.document_state = PeltDocumentState::Loading { address };
                host_effect.handled = true;
                host_effect.redraw = true;
                host_effect.navigated = true;
                host_effect.editable = false;
            },
            Err(error) => self.document_error(request.address, error, host_effect),
        }
    }

    fn traverse_to(&mut self, index: usize, host_effect: &mut PeltHostEffect) {
        let HistoryEntry { engine_id, request } = self.history[index].clone();
        match self.spawn_with(&engine_id, &request) {
            Ok(session) => {
                self.install_session_from(engine_id, session);
                self.history_index = index;
                self.document_state = PeltDocumentState::Loading {
                    address: request.address.clone(),
                };
                host_effect.handled = true;
                host_effect.redraw = true;
                host_effect.navigated = true;
                host_effect.editable = false;
            },
            Err(error) => self.document_error(request.address, error, host_effect),
        }
    }

    fn spawn_with(
        &self,
        engine_id: &str,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<F>>, String> {
        let mut request = request.clone();
        request.viewport = self.viewport;
        self.session_engines
            .spawn(engine_id, &request)
            .map_err(|error| format!("could not load {}: {error}", request.address))
    }

    /// Route one new load. Without a router the controller keeps its engine.
    fn route_load(&self, request: &SessionSpawnRequest) -> Result<LoadRoute, String> {
        let Some(router) = &self.router else {
            return Ok(LoadRoute::Document {
                engine_id: self.engine_id.clone(),
                route: None,
            });
        };
        router.registries.choose(
            &request.address,
            request.content_type.as_deref(),
            router.engine_override.as_deref(),
            false,
        )
    }

    /// Route a held document among document engines only, using its media
    /// type. The address already chose the document lane.
    fn document_route(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<(String, Option<PeltRoute>), String> {
        match &self.router {
            None => Ok((self.engine_id.clone(), None)),
            Some(router) => match router.registries.choose(
                &request.address,
                request.content_type.as_deref(),
                router.engine_override.as_deref(),
                true,
            )? {
                LoadRoute::Document { engine_id, route } => Ok((engine_id, route)),
                LoadRoute::Surface(_) => unreachable!("documents-only routing chose a surface"),
            },
        }
    }

    fn install_session_from(&mut self, engine_id: String, session: Box<dyn DocumentSession<F>>) {
        self.engine_id = engine_id;
        self.install_session(session);
    }

    /// Install a session only after its factory has completed successfully.
    /// Failed replacement attempts retain both this session and its generation.
    fn install_session(&mut self, session: Box<dyn DocumentSession<F>>) {
        self.session = session;
        self.session_generation = self
            .session_generation
            .checked_add(1)
            .expect("Pelt session generation exhausted");
    }

    fn document_error(
        &mut self,
        address: String,
        message: String,
        host_effect: &mut PeltHostEffect,
    ) {
        self.document_state = PeltDocumentState::Error {
            address,
            message: message.clone(),
        };
        host_effect.error = Some(message);
        // The host-owned diagnostic document is a visible transition even
        // though the active session and history stay unchanged.
        host_effect.handled = true;
        host_effect.redraw = true;
    }
}

/// The modifiers an input carried, for a link's disposition.
fn input_modifiers(input: &SessionInput) -> inker::SessionModifiers {
    match input {
        SessionInput::PointerButton { modifiers, .. }
        | SessionInput::PointerMoved { modifiers, .. }
        | SessionInput::Key { modifiers, .. } => *modifiers,
        _ => inker::SessionModifiers::default(),
    }
}

fn get_submission_target(action: &str, fields: &[(String, String)]) -> String {
    if fields.is_empty() {
        return action.to_owned();
    }
    let query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(
            fields
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str())),
        )
        .finish();
    let (base, fragment) = action
        .split_once('#')
        .map_or((action, None), |(base, fragment)| (base, Some(fragment)));
    let separator = if base.contains('?') { '&' } else { '?' };
    let mut target = format!("{base}{separator}{query}");
    if let Some(fragment) = fragment {
        target.push('#');
        target.push_str(fragment);
    }
    target
}
