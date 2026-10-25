// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's own route inside Djinn (reservoir plan V2, step 5).
//!
//! Each mere the reservoir holds is served on `mere/<domain>` (§7 items 20 and
//! 30): registered when the lane opens and whenever a mere is ensured, and
//! available by default to every application the owner admits. Recorded
//! per-mere denials override this default, and `ambient/mere/<domain>` is a
//! separate read-only route requiring per-application opt-in. An admitted
//! session attaches to one of the mere's sessions, the live one changed last
//! unless it asks for another (§7 item 24), and sees two projections: the
//! mere's sessions, with the lifecycle as intents that name their session by
//! id (§7 item 33) in `mere_overmap::wire`'s vocabulary (§7 item 37),
//! and the attached session's graph, served through `MereHost` with its
//! session item (§7 items 21, 22 and 31). Every application attached to one
//! session shares one host, so each sees the others' edits, told by a
//! revision bell. The application an edit comes through is the one the door
//! admitted, never the client's claim.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::door::app_broker::{AppEndpointCatalog, WeakAppEndpointCatalog};
use crate::resident_reservoir::ResidentReservoir;
use chirograph::{
    AdvertisedAction, BoundsRelationship, CachePolicy, CardValueV1, CarrierNotice, ContentHash,
    EndpointDescriptor, IntentEffect, IntentInvocation, IntentReference, IntentResult,
    PortableCardV1, PresentationBinding, PresentationCapability, PresentationCodec,
    PresentationKey, PresentationManifest, PresentationOffer, PresentationSemantics, ProjectionAck,
    ProjectionOffer, ProjectionRequest, ProjectionSession, ProjectionSnapshot, ProtocolVersion,
    ResourceRequest, ResourceResponse, ResumeReply, ResumeRequest, SemanticRole,
};
use graphshell::access::AccessContext;
use graphshell::handlers::HandlerRegistry;
use graphshell::mere_host::{MereHost, MereHostError, SelectedPersonaRef};
use graphshell_endpoint::door::app_admission::{AppId, AppRequest, AppRouteGrants, AppRouteId};
use graphshell_endpoint::door::route::ResidentEndpointRoute;
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
    ResumableProjectionSource,
};
use graphshell_host::lifecycle::AdmittedEndpointContext;
use mere_overmap::wire::{
    self as session_item, ARCHIVE_SCHEMA, ARCHIVE_SOURCE, ATTACH_SESSION_INTENT, ApplyEditsV1,
    ArchiveActionV1, BRANCH_SESSION_INTENT, CANCEL_MERGE_INTENT, COMPOSE_CODICILS_INTENT,
    FORK_SESSION_INTENT, MERE_ARCHIVE, MERE_GRAPH, MERE_SESSIONS, MERGE_SCHEMA,
    MERGE_SESSION_INTENT, MERGE_SOURCE, MINT_SESSION_INTENT, MergeChoiceV1, MergeResolutionV1,
    OPEN_CODICIL_INTENT, PENDING_MERGE_SCHEMA, PendingMergeV1, RESOLVE_MERGE_INTENT,
    RESTORE_SESSION_INTENT, SAVE_CODICIL_INTENT, SESSIONS_SCHEMA, SESSIONS_SOURCE,
    SessionsActionV1, SetViewV1, StepV1, TRASH_SESSION_INTENT,
};
use muniment::{DirectoryBackend, RedbBackend};
use pandect::{
    Author, GraphSessionManifest, MereApplicationAccess, MereId, MereRecord, MereSessions,
    SessionId, open_mere_archive_backend, open_mere_backend,
};
use personae::PersonaId;
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use tokio::sync::Mutex;

/// How often a granted client polls a mere's route for bells.
pub const MERE_NOTICE_POLL: Duration = Duration::from_millis(50);

const SESSIONS_EPOCH: SceneEpoch = SceneEpoch(1);

/// The route a mere is served on (§7 item 30).
pub fn mere_route_id(domain: &str) -> String {
    format!("mere/{domain}")
}

/// A separately opted-in, read-only ambient crossing of a mere.
pub fn ambient_mere_route_id(domain: &str) -> String {
    format!("ambient/mere/{domain}")
}

/// Run `operation` to completion from the synchronous endpoint traits.
fn run<T>(operation: impl std::future::Future<Output = T>) -> T {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(operation))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

/// Serves each mere on its own route: shared by the lane, which serves the
/// meres already there, and the reservoir route, which serves each new one.
#[derive(Clone)]
pub struct MereRoutes {
    inner: Arc<Routes>,
}

type ProfileLookup = Arc<dyn Fn(&personae::ProfileId) -> Result<bool, String> + Send + Sync>;

struct Routes {
    custody_endpoint: Option<String>,
    coop_relays: Vec<transport::p2panda_transport::RelayUrl>,
    shared_root: PathBuf,
    persona: PersonaId,
    catalog: WeakAppEndpointCatalog,
    grants: AppRouteGrants,
    apps: Vec<AppId>,
    validators: BTreeMap<String, pandect::GraphValidator>,
    handlers: HandlerRegistry,
    profile_lookup: Option<ProfileLookup>,
    /// The dialable projection host's catalog, when one serves these routes
    /// to remote peers (remote mere route admission note, §6).
    remote: Option<graphshell_host::projection_host::SharedCatalog>,
    served: Mutex<BTreeMap<String, Arc<MereShared>>>,
    access: Mutex<BTreeMap<(MereId, AppId), MereApplicationAccess>>,
    access_loaded: AtomicBool,
}

impl MereRoutes {
    /// Compose Cleromancy's portable product authority in either resident mode.
    /// The supplier owns the record/replay semantics; Djinn owns installation
    /// and durability, so applications cannot select a weaker callback.
    pub fn with_divination_authority(self) -> Self {
        self.with_domain_validator(
            "divination",
            Arc::new(|graph| {
                cleromancy_domain::host::validate_divination_graph(graph)
                    .map_err(|error| error.to_string())
            }),
        )
        .with_personal_facts_authority()
    }

    /// Install the same biographical-fact rules in daemon and embedded owners.
    pub fn with_personal_facts_authority(self) -> Self {
        let persona = self.inner.persona.as_uuid().to_string();
        self.with_domain_validator(
            personal_facts::DOMAIN,
            Arc::new(move |graph| personal_facts::validate_for_persona(graph, &persona)),
        )
    }
    /// Resolve newly introduced profile subjects through the actual owner vault.
    /// Existing admitted references continue to replay after profile deletion.
    pub fn with_profile_subject_authority<S: castellan::custody::IdentityStorage + 'static>(
        mut self,
        host: Arc<castellan::authority::PersonaeHost<S>>,
    ) -> Self {
        Arc::get_mut(&mut self.inner)
            .expect("set profile authority before serving routes")
            .profile_lookup = Some(Arc::new(move |id| {
            Ok(host.with_profile_authority(id, || true).unwrap_or(false))
        }));
        self
    }

    /// Routes for `persona`'s meres, registered in `catalog` and granted to
    /// `apps` as each is served.
    pub fn new(
        shared_root: PathBuf,
        persona: PersonaId,
        catalog: AppEndpointCatalog,
        grants: AppRouteGrants,
        apps: Vec<AppId>,
    ) -> Self {
        Self {
            inner: Arc::new(Routes {
                custody_endpoint: None,
                coop_relays: Vec::new(),
                shared_root,
                persona,
                catalog: catalog.downgrade(),
                grants,
                apps,
                validators: BTreeMap::new(),
                handlers: HandlerRegistry::default(),
                profile_lookup: None,
                remote: None,
                served: Mutex::new(BTreeMap::new()),
                access: Mutex::new(BTreeMap::new()),
                access_loaded: AtomicBool::new(false),
            }),
        }
    }

    /// Install the owner's ordered address offers before serving routes.
    /// Offers use the existing open-address contract; native dispatch belongs
    /// to the application host and is not an executable command in the graph.
    pub fn with_handlers(mut self, handlers: HandlerRegistry) -> Self {
        Arc::get_mut(&mut self.inner)
            .expect("configure handlers before serving routes")
            .handlers = handlers;
        self
    }

    /// The daemon's own custody door. Embedded compositions that do not
    /// serve custody leave co-op controls absent rather than borrowing another
    /// resident's identity from process-global configuration.
    pub fn with_custody_endpoint(mut self, endpoint: String) -> Self {
        Arc::get_mut(&mut self.inner)
            .expect("compose custody before serving")
            .custody_endpoint = Some(endpoint);
        self
    }

    /// Owner-selected reachability for co-op, composed before any routes open.
    pub fn with_coop_relays(mut self, relays: Vec<transport::p2panda_transport::RelayUrl>) -> Self {
        Arc::get_mut(&mut self.inner)
            .expect("compose co-op relays before serving")
            .coop_relays = relays;
        self
    }

    /// Load the durable decisions before serving routes. This is mandatory
    /// at resident composition, including an embedded resident.
    pub async fn load_access(&self, reservoir: &ResidentReservoir) -> Result<(), String> {
        if reservoir.persona() != self.inner.persona {
            return Err("mere routes and reservoir belong to different personas".into());
        }
        let mut access = self.inner.access.lock().await;
        let meres = reservoir.meres().await;
        for (mere, app, decision) in reservoir.access_decisions().await {
            let record = meres
                .iter()
                .find(|record| record.id == mere)
                .ok_or("access names an unknown mere")?;
            self.publish_access(&record.domain.to_string(), &app, &decision);
            access.insert((mere, app), decision);
        }
        self.inner.access_loaded.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Owner-side permission management for trusted resident configuration.
    /// Ordinary admitted product endpoints never expose this capability. The
    /// `via` label records provenance; it is not evidence of owner approval.
    /// Commit first, then publish an atomic door update. A failed storage
    /// write changes no grant; a restarted resident replays the committed
    /// decisions before it opens any routes.
    pub async fn set_access(
        &self,
        reservoir: &ResidentReservoir,
        mere: MereId,
        app: AppId,
        denied: bool,
        ambient: bool,
        via: AppId,
    ) -> Result<(), String> {
        if reservoir.persona() != self.inner.persona {
            return Err("mere routes and reservoir belong to different personas".into());
        }
        let grants = self.inner.grants.current();
        if !grants.is_admitted(&app) || !grants.is_admitted(&via) {
            return Err("access decisions name an application this door does not admit".into());
        }
        let mut access = self.inner.access.lock().await;
        let record = reservoir
            .meres()
            .await
            .into_iter()
            .find(|record| record.id == mere)
            .ok_or("access names an unknown mere")?;
        reservoir
            .record_access(mere, app.clone(), denied, ambient, via)
            .await?;
        let decision = reservoir
            .access_decisions()
            .await
            .into_iter()
            .find(|(id, application, _)| *id == mere && *application == app)
            .ok_or("the committed access decision is missing")?
            .2;
        self.publish_access(record.domain.as_str(), &app, &decision);
        access.insert((mere, app), decision);
        Ok(())
    }

    fn publish_access(&self, domain: &str, app: &AppId, access: &MereApplicationAccess) {
        let explicit = AppRouteId::new(mere_route_id(domain)).expect("valid mere route");
        let ambient = ResidentEndpointRoute::new(ambient_mere_route_id(domain), MERE_NOTICE_POLL)
            .expect("valid ambient route");
        let ambient_id = AppRouteId::new(ambient.id()).expect("valid ambient route");
        self.inner.grants.update(|grants| {
            grants.set_denied(app.clone(), explicit, access.denied);
            grants.set_denied(app.clone(), ambient_id.clone(), access.denied);
            if access.ambient && grants.is_admitted(app) {
                grants.grant(app.clone(), ambient);
            } else {
                grants.revoke(app, &ambient_id);
            }
        });
    }

    /// Also register each mere's two routes in `catalog`, the dialable
    /// projection host's (`crate::resident_projection`), as each is served.
    /// The routes there are the same endpoints under the same owner
    /// decisions; what a remote session may open is that host's admission.
    /// Set before cloning or serving routes.
    pub fn with_remote_catalog(
        mut self,
        catalog: graphshell_host::projection_host::SharedCatalog,
    ) -> Self {
        Arc::get_mut(&mut self.inner)
            .expect("set the remote catalog before cloning or serving routes")
            .remote = Some(catalog);
        self
    }

    /// Register product authority before serving routes. The callback is reused
    /// for edits, archive save/open/compose and loaded sessions.
    pub fn with_domain_validator(
        mut self,
        domain: impl Into<String>,
        validator: pandect::GraphValidator,
    ) -> Self {
        Arc::get_mut(&mut self.inner)
            .expect("set validator before cloning or serving routes")
            .validators
            .insert(domain.into(), validator);
        self
    }

    /// Serve `mere` on `mere/<domain>`: open its store, register its route and
    /// grant it, unless it is served already.
    pub async fn serve(&self, mere: &MereRecord) -> Result<ResidentEndpointRoute, String> {
        if !self.inner.access_loaded.load(Ordering::SeqCst) {
            return Err("load reservoir access decisions before serving mere routes".into());
        }
        if mere.persona != self.inner.persona
            || mere.id != MereId::derive(mere.persona, &mere.domain)
        {
            return Err("mere does not belong to this reservoir persona/domain".into());
        }
        let domain = mere.domain.to_string();
        // Product records must never be writable under generic graph authority.
        // Both daemon and embedded composition install the same product callback.
        if matches!(domain.as_str(), "divination" | "personal-facts")
            && !self.inner.validators.contains_key(&domain)
        {
            return Err(format!(
                "{domain} domain authority is not composed into this resident"
            ));
        }
        let catalog = self
            .inner
            .catalog
            .upgrade()
            .ok_or("the mere's application catalog is closed")?;
        let route = ResidentEndpointRoute::new(mere_route_id(&domain), MERE_NOTICE_POLL)
            .map_err(|error| error.to_string())?;
        let mut served = self.inner.served.lock().await;
        if served.contains_key(&domain) {
            return Ok(route);
        }
        let backend = open_mere_backend(&self.inner.shared_root, self.inner.persona, mere.id)
            .map_err(|error| error.to_string())?;
        let shared = Arc::new(MereShared {
            domain: domain.clone(),
            custody_endpoint: self.inner.custody_endpoint.clone(),
            handlers: self.inner.handlers.clone(),
            coop_relays: self.inner.coop_relays.clone(),
            validator: self.inner.validators.get(&domain).cloned(),
            admission_validator: if domain == personal_facts::DOMAIN {
                let lookup = self.inner.profile_lookup.clone();
                Some(Arc::new(move |before, after| {
                    personal_facts::validate_new_profile_subjects(before, after, |id| match &lookup
                    {
                        Some(resolve) => resolve(id),
                        None => Err("profile subjects require the resident vault authority".into()),
                    })
                }))
            } else {
                None
            },
            archive: open_mere_archive_backend(
                &self.inner.shared_root,
                self.inner.persona,
                mere.id,
            )
            .map_err(|e| e.to_string())?,
            backend,
            persona: self.inner.persona,
            hosts: Mutex::new(HashMap::new()),
            coop: Default::default(),
            lifecycle: AtomicU64::new(1),
            codicil_sessions: Mutex::new(HashMap::new()),
            pending_merges: Mutex::new(BTreeMap::new()),
        });
        let ambient = ResidentEndpointRoute::new(ambient_mere_route_id(&domain), MERE_NOTICE_POLL)
            .map_err(|error| error.to_string())?;
        let grants = self.inner.grants.clone();
        catalog
            .update(|catalog| register_mere_routes(catalog, &route, &ambient, &shared, &grants))
            .await
            .map_err(|error| error.to_string())?;
        if let Some(remote) = &self.inner.remote {
            let mut remote = remote
                .lock()
                .map_err(|_| "the remote catalog lock is poisoned".to_string())?;
            register_mere_routes(&mut remote, &route, &ambient, &shared, &grants)
                .map_err(|error| error.to_string())?;
        }
        if self.inner.custody_endpoint.is_some() {
            let coop_route = ResidentEndpointRoute::new(format!("coop/{domain}"), MERE_NOTICE_POLL)
                .map_err(|e| e.to_string())?;
            let coop_shared = shared.clone();
            let coop_grants = self.inner.grants.clone();
            catalog
                .update(|catalog| coop::register(catalog, coop_shared, coop_grants))
                .await
                .map_err(|e| e.to_string())?;
            self.inner
                .grants
                .grant(AppId::new("graphshell"), coop_route);
        }
        for app in &self.inner.apps {
            self.inner.grants.grant(app.clone(), route.clone());
        }
        self.inner.grants.grant_default(route.clone());
        for ((id, app), access) in self.inner.access.lock().await.iter() {
            if *id == mere.id {
                self.publish_access(&domain, app, access);
            }
        }
        served.insert(domain, shared);
        Ok(route)
    }
}

/// Register `shared`'s explicit and ambient routes in `catalog`, each opening
/// an [`AccessEndpoint`] that rechecks `grants`.
fn register_mere_routes(
    catalog: &mut graphshell_host::endpoint_catalog::ResidentEndpointCatalog,
    route: &ResidentEndpointRoute,
    ambient: &ResidentEndpointRoute,
    shared: &Arc<MereShared>,
    grants: &AppRouteGrants,
) -> Result<(), graphshell_host::endpoint_catalog::ResidentEndpointCatalogError> {
    let (opened, explicit_grants, explicit_id) =
        (Arc::clone(shared), grants.clone(), route.id().to_string());
    catalog.register_resumable_notifying(
        route.id(),
        format!("Mere: {}", shared.domain),
        move |context| {
            AccessEndpoint::open(
                Arc::clone(&opened),
                context,
                explicit_grants.clone(),
                &explicit_id,
                false,
            )
        },
    )?;
    let (ambient_opened, ambient_grants, ambient_id) =
        (Arc::clone(shared), grants.clone(), ambient.id().to_string());
    catalog.register_resumable_notifying(
        ambient.id(),
        format!("Ambient mere: {}", shared.domain),
        move |context| {
            AccessEndpoint::open(
                Arc::clone(&ambient_opened),
                context,
                ambient_grants.clone(),
                &ambient_id,
                true,
            )
        },
    )
}

/// Recheck live owner policy before factory construction and on every read
/// or write. Existing sessions cannot keep reading after a denial or ambient
/// opt-out. Ambient routes offer graph reads only and accept no intents.
struct AccessEndpoint {
    endpoint: MereEndpoint,
    grants: AppRouteGrants,
    request: AppRequest,
    ambient: bool,
    ambient_resources: BTreeMap<ContentHash, Vec<u8>>,
    ambient_disclosed: BTreeSet<ContentHash>,
}

impl AccessEndpoint {
    fn open(
        shared: Arc<MereShared>,
        context: &AdmittedEndpointContext,
        grants: AppRouteGrants,
        route: &str,
        ambient: bool,
    ) -> Result<Self, String> {
        let app = context
            .application()
            .ok_or("mere access requires an admitted application")?;
        let request = AppRequest {
            app: AppId::new(app),
            route: AppRouteId::new(route).map_err(|error| error.to_string())?,
        };
        grants
            .current()
            .admit(&request)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            endpoint: MereEndpoint::open_mode(shared, context, ambient)?,
            grants,
            request,
            ambient,
            ambient_resources: BTreeMap::new(),
            ambient_disclosed: BTreeSet::new(),
        })
    }
    fn check(&self) -> Result<(), String> {
        self.grants
            .current()
            .admit(&self.request)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

impl ProjectionCatalog for AccessEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        let mut descriptor = self.endpoint.describe();
        if self.ambient {
            descriptor
                .projections
                .retain(|offer| offer.request.session.0 == MERE_GRAPH);
        }
        descriptor
    }
}
impl ProjectionSource for AccessEndpoint {
    type Error = String;
    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        self.check()?;
        if self.ambient && request.session.0 != MERE_GRAPH {
            return Err("ambient routes expose graph material only".into());
        }
        let mut snapshot = self.endpoint.snapshot(request)?;
        if matches!(
            self.endpoint.shared.domain.as_str(),
            "divination" | "personal-facts"
        ) && snapshot.session.0 == MERE_GRAPH
        {
            for binding in &snapshot.presentation.bindings {
                if binding.instance != InstanceId(0) {
                    if let Some(offers) = snapshot.presentation.offers.get_mut(&binding.key) {
                        for offer in offers {
                            offer.semantics.actions.clear();
                        }
                    }
                }
            }
        }
        if self.ambient {
            self.ambient_resources.clear();
            self.ambient_disclosed.clear();
            let session_key = snapshot
                .presentation
                .bindings
                .iter()
                .find(|binding| binding.instance == InstanceId(0))
                .map(|binding| binding.key.clone());
            for (key, offers) in &mut snapshot.presentation.offers {
                for offer in offers {
                    offer.semantics.actions.clear();
                    let response = self.endpoint.resource(ResourceRequest {
                        session: snapshot.session.clone(),
                        resource: offer.resource,
                    })?;
                    if let Ok(mut card) = serde_json::from_slice::<PortableCardV1>(&response.bytes)
                    {
                        if session_key.as_ref() == Some(key) {
                            // An ambient graph card is not authority to read the
                            // complete facet-bearing domain graph.
                            card.media.clear();
                            let bytes = serde_json::to_vec(&card).map_err(|e| e.to_string())?;
                            offer.resource = ContentHash::of(&bytes);
                            offer.byte_size = bytes.len() as u64;
                            self.ambient_resources.insert(offer.resource, bytes);
                        } else {
                            self.ambient_disclosed.extend(card.media);
                        }
                    }
                    self.ambient_disclosed.insert(offer.resource);
                }
            }
        }
        Ok(snapshot)
    }
}
impl PresentationSource for AccessEndpoint {
    type Error = String;
    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        self.check()?;
        if self.ambient && request.session.0 != MERE_GRAPH {
            return Err("ambient routes expose graph material only".into());
        }
        if self.ambient {
            if !self.ambient_disclosed.contains(&request.resource) {
                return Err("resource was not disclosed by this ambient projection".into());
            }
            if let Some(bytes) = self.ambient_resources.get(&request.resource) {
                return Ok(ResourceResponse {
                    session: request.session,
                    resource: request.resource,
                    bytes: bytes.clone(),
                });
            }
        }
        let response = self.endpoint.resource(request)?;
        if let Ok(graph) = serde_json::from_slice::<session_item::SessionGraphV1>(&response.bytes) {
            if self.ambient {
                return Err(
                    "ambient material excludes complete session graph and view state".into(),
                );
            }
            if graph.application != self.request.app.as_str() {
                return Err(
                    "session graph view state belongs to another admitted application".into(),
                );
            }
        }
        Ok(response)
    }
}
impl IntentSink for AccessEndpoint {
    type Error = String;
    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        self.check()?;
        if self.ambient {
            return Ok(IntentResult::Rejected {
                reason: "ambient mere access is read-only; open the mere explicitly to edit it"
                    .into(),
            });
        }
        self.endpoint.invoke(intent)
    }
}
impl ProjectionNoticeSource for AccessEndpoint {
    type Error = String;
    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> {
        self.check()?;
        let notice = self.endpoint.poll_notice()?;
        Ok(notice.filter(|notice| !self.ambient || notice.session.0 == MERE_GRAPH))
    }
}

impl ResumableProjectionSource for AccessEndpoint {
    type Error = String;
    fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, String> {
        self.check()?;
        let projection = self
            .describe()
            .projections
            .into_iter()
            .find(|offer| offer.request.session == request.session)
            .ok_or("resume names a projection not disclosed by this admitted route")?
            .request;
        let snapshot = self.snapshot(projection)?;
        if snapshot.scene.epoch == request.epoch && snapshot.scene.revision == request.revision {
            Ok(ResumeReply::Current(ProjectionAck {
                session: snapshot.session,
                epoch: snapshot.scene.epoch,
                revision: snapshot.scene.revision,
            }))
        } else {
            Ok(ResumeReply::Snapshot(Box::new(snapshot)))
        }
    }
}

#[path = "resident_mere_coop.rs"]
mod coop;

type SharedHost = Arc<Mutex<MereHost<DirectoryBackend>>>;

/// One mere's store, and the host of each of its sessions an application has
/// attached to.
struct MereShared {
    custody_endpoint: Option<String>,
    handlers: HandlerRegistry,
    coop_relays: Vec<transport::p2panda_transport::RelayUrl>,
    domain: String,
    validator: Option<pandect::GraphValidator>,
    admission_validator: Option<pandect::GraphAdmissionValidator>,
    backend: DirectoryBackend,
    archive: RedbBackend,
    persona: PersonaId,
    hosts: Mutex<HashMap<SessionId, SharedHost>>,
    coop: coop::Activities,
    /// The sessions projection's revision: it moves with every lifecycle step.
    lifecycle: AtomicU64,
    /// The sessions each codicil was saved from, by codicil id. A codicil
    /// is immutable, so each is read once.
    codicil_sessions: Mutex<HashMap<String, Vec<SessionId>>>,
    /// Merges waiting for a choice on each conflict, by source session.
    pending_merges: Mutex<BTreeMap<uuid::Uuid, HeldMerge>>,
}

impl MereShared {
    fn validate(&self, graph: &pandect::graph_codicil::GraphCandidate) -> Result<(), String> {
        if let Some(validator) = &self.validator {
            validator(graph)?;
        }
        Ok(())
    }

    fn sessions(&self) -> MereSessions<DirectoryBackend> {
        MereSessions::new(self.backend.clone())
    }

    fn persona_id(&self) -> String {
        self.persona.as_uuid().to_string()
    }

    fn author(&self, app: &str) -> Author {
        Author::person(self.persona_id()).via(app)
    }

    fn moved(&self) {
        self.lifecycle.fetch_add(1, Ordering::SeqCst);
    }

    /// The sessions the codicil `id` was saved from, read once. A codicil
    /// this resident cannot read names none.
    async fn codicil_sessions(&self, store: &mut RedbBackend, id: &str) -> Vec<SessionId> {
        if let Some(sessions) = self.codicil_sessions.lock().await.get(id) {
            return sessions.clone();
        }
        let mut sessions = Vec::new();
        if let Ok(parsed) = pandect::graph_codicil::parse_codicil_id(id)
            && let Ok(Some(codicil)) =
                pandect::graph_codicil::load_profiled_graph_codicil_sealed(store, None, parsed)
                    .await
        {
            sessions = codicil
                .codicil
                .sessions
                .iter()
                .map(|session| session.manifest.session_id)
                .collect();
        }
        self.codicil_sessions
            .lock()
            .await
            .insert(id.to_owned(), sessions.clone());
        sessions
    }

    /// The one host of session `id`, opened once and shared by every
    /// application attached to it.
    async fn host(&self, id: SessionId) -> Result<SharedHost, String> {
        let mut hosts = self.hosts.lock().await;
        if let Some(host) = hosts.get(&id) {
            return Ok(Arc::clone(host));
        }
        let persona = self.persona_id();
        let host = MereHost::open_session_checked(
            self.backend.clone(),
            id,
            SelectedPersonaRef {
                persona: persona.clone(),
                profile: "djinn".to_string(),
            },
            self.handlers.clone(),
            AccessContext {
                persona,
                device: "djinn".to_string(),
                at_ms: 0,
            },
            self.validator.clone(),
        )
        .await
        .map_err(|error| error.to_string())?
        .with_projection_session(ProjectionSession(MERE_GRAPH.to_string()))
        .with_session_item();
        let host = match &self.admission_validator {
            Some(validator) => host.with_graph_admission_validator(validator.clone()),
            None => host,
        };
        let host = Arc::new(Mutex::new(host));
        hosts.insert(id, Arc::clone(&host));
        Ok(host)
    }

    /// The live session changed last, minting one when there is none.
    async fn latest(&self, app: &str) -> Result<SessionId, String> {
        let sessions = self.sessions();
        if let Some(latest) = sessions
            .latest_live()
            .await
            .map_err(|error| error.to_string())?
        {
            return Ok(latest.session_id);
        }
        let minted = sessions
            .mint(self.author(app), None)
            .await
            .map_err(|error| error.to_string())?;
        self.moved();
        Ok(minted.session_id)
    }
}

/// A merge waiting for a choice on each conflict, with what an earlier plan
/// of it left: the choices carried over and those dropped, and why.
#[derive(Clone)]
struct HeldMerge {
    plan: pandect::PendingMerge,
    carried: BTreeMap<pandect::Part, pandect::MergeChoice>,
    dropped: Vec<pandect::DroppedChoice>,
}

impl HeldMerge {
    fn new(plan: pandect::PendingMerge) -> Self {
        Self {
            plan,
            carried: BTreeMap::new(),
            dropped: Vec::new(),
        }
    }
}

/// A card of the sessions projection and what it is served as.
#[derive(Clone)]
struct SessionsCard {
    source: String,
    label: String,
    bytes: Vec<u8>,
    actions: Vec<AdvertisedAction>,
    /// The scene source kind, when not the projection's own.
    kind: Option<&'static str>,
    /// Media the card names, served beside it.
    media: Vec<Vec<u8>>,
}

/// The session a branch or picked-off part merges into: a branch's parent,
/// or the session its parent was merged into; a part's source. `None` for a
/// fork, a minted session or one already merged.
fn merge_target(
    manifests: &[GraphSessionManifest],
    manifest: &GraphSessionManifest,
) -> Option<SessionId> {
    if manifest.merged.is_some() || manifest.trashed.is_some() {
        return None;
    }
    if let Some(picked) = &manifest.picked_from {
        return Some(picked.session);
    }
    if manifest.lineage != pandect::Lineage::Branch {
        return None;
    }
    let mut parent = manifest.parent_session?;
    for _ in 0..manifests.len() {
        let held = manifests.iter().find(|held| held.session_id == parent)?;
        match &held.merged {
            Some(mark) => parent = mark.into,
            None => return Some(parent),
        }
    }
    None
}

fn merge_action(intent: &str, label: &str, explanation: &str) -> AdvertisedAction {
    AdvertisedAction {
        payload_schema: MERGE_SCHEMA.to_string(),
        ..lifecycle_action(intent, label, explanation)
    }
}

fn lifecycle_action(intent: &str, label: &str, explanation: &str) -> AdvertisedAction {
    AdvertisedAction {
        intent: IntentReference(intent.to_string()),
        label: label.to_string(),
        explanation: explanation.to_string(),
        payload_schema: SESSIONS_SCHEMA.to_string(),
        input_form: None,
        effect: IntentEffect::DomainTruth,
    }
}

/// One admitted application's view of one mere.
struct MereEndpoint {
    shared: Arc<MereShared>,
    app: String,
    attached: SessionId,
    host: SharedHost,
    /// The graph epoch and revision this connection last saw or was told of.
    seen_graph: (SceneEpoch, Revision),
    /// The sessions revision this connection last saw or was told of.
    seen_sessions: u64,
    seen_archive: u64,
}

impl MereEndpoint {
    fn open_mode(
        shared: Arc<MereShared>,
        context: &AdmittedEndpointContext,
        ambient: bool,
    ) -> Result<Self, String> {
        let app = context
            .application()
            .ok_or("a mere's route serves admitted applications only")?
            .to_string();
        run(async {
            let attached = if ambient {
                shared
                    .sessions()
                    .latest_live()
                    .await
                    .map_err(|error| error.to_string())?
                    .ok_or("this mere has no live material for ambient reading")?
                    .session_id
            } else {
                shared.latest(&app).await?
            };
            let host = shared.host(attached).await?;
            let seen_graph = {
                let host = host.lock().await;
                (host.projection_epoch(), host.projection_revision())
            };
            Ok(Self {
                seen_sessions: shared.lifecycle.load(Ordering::SeqCst),
                seen_archive: shared.lifecycle.load(Ordering::SeqCst),
                shared,
                app,
                attached,
                host,
                seen_graph,
            })
        })
    }

    fn sessions_session() -> ProjectionSession {
        ProjectionSession(MERE_SESSIONS.to_string())
    }

    fn sessions_request() -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: Self::sessions_session(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }

    fn sessions_revision(&self) -> Revision {
        Revision(self.shared.lifecycle.load(Ordering::SeqCst))
    }

    /// The mere's card first, then one card per session, oldest first.
    fn session_cards(&self) -> Result<Vec<SessionsCard>, String> {
        let manifests: Vec<GraphSessionManifest> =
            run(self.shared.sessions().list()).map_err(|error| error.to_string())?;
        let pending: BTreeMap<uuid::Uuid, HeldMerge> =
            run(async { self.shared.pending_merges.lock().await.clone() });
        let mut cards = Vec::with_capacity(manifests.len() + 1);
        let mere = PortableCardV1 {
            title: self.shared.domain.clone(),
            values: vec![CardValueV1 {
                label: "Sessions".to_string(),
                value: manifests.len().to_string(),
            }],
            badges: Vec::new(),
            media: Vec::new(),
        };
        cards.push(SessionsCard {
            source: format!("mere:{}", self.shared.domain),
            label: format!("Mere: {}", self.shared.domain),
            bytes: serde_json::to_vec(&mere).map_err(|error| error.to_string())?,
            actions: vec![lifecycle_action(
                MINT_SESSION_INTENT,
                "Mint a session",
                "Begin a new, empty session in this mere.",
            )],
            kind: None,
            media: Vec::new(),
        });
        for manifest in &manifests {
            let id = manifest.session_id;
            let trashed = manifest.trashed.is_some();
            let mut badges = Vec::new();
            if id == self.attached {
                badges.push("Attached".to_string());
            }
            if manifest.codicil_read_only {
                badges.push("Read-only thaw; editing forks".into());
            }
            if trashed {
                badges.push("Trashed".to_string());
            }
            let card = PortableCardV1 {
                title: manifest
                    .display_name
                    .clone()
                    .unwrap_or_else(|| "Session".to_string()),
                values: vec![CardValueV1 {
                    label: "Session".to_string(),
                    value: id.as_uuid().to_string(),
                }],
                badges,
                media: Vec::new(),
            };
            cards.push(SessionsCard {
                source: id.as_uuid().to_string(),
                label: format!("Session {}", id.as_uuid()),
                bytes: serde_json::to_vec(&card).map_err(|error| error.to_string())?,
                actions: self.session_actions(&manifests, manifest, &pending),
                kind: None,
                media: Vec::new(),
            });
        }
        // Each pending merge, its conflicts disclosed as the card's media.
        let name = |id: SessionId| {
            manifests
                .iter()
                .find(|manifest| manifest.session_id == id)
                .and_then(|manifest| manifest.display_name.clone())
                .unwrap_or_else(|| "Session".to_string())
        };
        for held in pending.values() {
            let merge = &held.plan;
            let source = merge.source();
            let material = PendingMergeV1 {
                schema: PENDING_MERGE_SCHEMA.to_string(),
                source,
                source_name: name(source),
                target: merge.target(),
                target_name: name(merge.target()),
                conflicts: merge.conflicts().to_vec(),
                carried: held
                    .carried
                    .iter()
                    .map(|(part, choice)| MergeChoiceV1 {
                        part: part.clone(),
                        choice: choice.clone(),
                    })
                    .collect(),
                dropped: held.dropped.clone(),
            };
            let media = serde_json::to_vec(&material).map_err(|error| error.to_string())?;
            let card = PortableCardV1 {
                title: format!(
                    "Merge {} into {}",
                    material.source_name, material.target_name
                ),
                values: vec![CardValueV1 {
                    label: "Conflicts".to_string(),
                    value: material.conflicts.len().to_string(),
                }],
                badges: vec!["Pending merge".to_string()],
                media: vec![ContentHash::of(&media)],
            };
            cards.push(SessionsCard {
                source: source.as_uuid().to_string(),
                label: card.title.clone(),
                bytes: serde_json::to_vec(&card).map_err(|error| error.to_string())?,
                actions: vec![
                    merge_action(
                        RESOLVE_MERGE_INTENT,
                        "Apply merge",
                        "Write the merge with a choice for each conflict.",
                    ),
                    merge_action(
                        CANCEL_MERGE_INTENT,
                        "Cancel merge",
                        "Drop the pending merge; nothing is written.",
                    ),
                ],
                kind: Some(MERGE_SOURCE),
                media: vec![media],
            });
        }
        Ok(cards)
    }

    /// The steps a session's card offers as the session stands now: attach
    /// unless this connection holds it, fork, branch, merge into its source
    /// for a branch or picked-off part with no merge pending, and trash or
    /// restore.
    fn session_actions(
        &self,
        manifests: &[GraphSessionManifest],
        manifest: &GraphSessionManifest,
        pending: &BTreeMap<uuid::Uuid, HeldMerge>,
    ) -> Vec<AdvertisedAction> {
        let mut actions = vec![session_item::archive_action(
            SAVE_CODICIL_INTENT,
            "Save as codicil",
        )];
        if manifest.session_id != self.attached {
            actions.push(lifecycle_action(
                ATTACH_SESSION_INTENT,
                "Attach",
                "Show and edit this session instead.",
            ));
        }
        actions.push(lifecycle_action(
            FORK_SESSION_INTENT,
            "Fork",
            "Begin a new session from this one as it stands now.",
        ));
        actions.push(lifecycle_action(
            BRANCH_SESSION_INTENT,
            "Branch",
            "Begin a branch of this session as it stands now, meant to merge back.",
        ));
        if merge_target(manifests, manifest).is_some()
            && !pending.contains_key(manifest.session_id.as_uuid())
        {
            actions.push(lifecycle_action(
                MERGE_SESSION_INTENT,
                "Merge into source",
                "Merge this session back into the session it came from.",
            ));
        }
        actions.push(if manifest.trashed.is_some() {
            lifecycle_action(
                RESTORE_SESSION_INTENT,
                "Restore",
                "Take the session out of the trash.",
            )
        } else {
            lifecycle_action(
                TRASH_SESSION_INTENT,
                "Trash",
                "Put the session in the trash; nothing is deleted.",
            )
        });
        actions
    }

    fn sessions_snapshot(&mut self) -> Result<ProjectionSnapshot, String> {
        let cards = self.session_cards()?;
        self.cards_snapshot(cards, Self::sessions_session(), SESSIONS_SOURCE)
    }

    fn cards_snapshot(
        &mut self,
        cards: Vec<SessionsCard>,
        session: ProjectionSession,
        source_kind: &str,
    ) -> Result<ProjectionSnapshot, String> {
        let revision = self.sessions_revision();
        let mut scene = Scene::new();
        let mut presentation = PresentationManifest::default();
        for (index, card) in cards.iter().enumerate() {
            let kind = card.kind.unwrap_or(source_kind);
            let source = scene.intern_source(SourceRef::new(kind, &card.source));
            scene.items.push(ProjectedItem {
                source,
                space: Scene::WORLD,
                transform: Transform2::IDENTITY,
                footprint: Footprint::Rect {
                    size: Size2::new(1.0, 1.0),
                },
                representation: Representation::Card,
                layer: 0,
                visible: true,
                hit: None,
                channels: Vec::new(),
            });
            let key = PresentationKey(match card.kind {
                Some(kind) => format!("{kind}:{}", card.source),
                None => card.source.clone(),
            });
            presentation.bindings.push(PresentationBinding {
                instance: InstanceId(index as u32),
                key: key.clone(),
            });
            presentation.offers.insert(
                key,
                vec![PresentationOffer {
                    codec: PresentationCodec::PortableCardV1,
                    resource: ContentHash::of(&card.bytes),
                    byte_size: card.bytes.len() as u64,
                    requires: PresentationCapability::PortableCard,
                    semantics: PresentationSemantics {
                        label: card.label.clone(),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FitWithinFootprint,
                        actions: card.actions.clone(),
                    },
                }],
            );
        }
        if session.0 == MERE_ARCHIVE {
            self.seen_archive = revision.0;
        } else {
            self.seen_sessions = revision.0;
        }
        Ok(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session,
            scene: SceneSnapshot::from_dense(SESSIONS_EPOCH, revision, scene)
                .map_err(|error| format!("{error:?}"))?,
            presentation,
            cache_policy: CachePolicy::default(),
        })
    }

    fn archive_session() -> ProjectionSession {
        ProjectionSession(MERE_ARCHIVE.into())
    }

    fn archive_cards(&self) -> Result<Vec<SessionsCard>, String> {
        let shared = Arc::clone(&self.shared);
        run(async move {
            let mut store = shared.archive.clone();
            let mut manifests = pandect::graph_codicil::list_graph_codicils(&mut store)
                .await
                .map_err(|e| e.to_string())?;
            manifests.sort_by_key(|manifest| (manifest.created_at, manifest.id.to_string()));
            let mut cards = vec![SessionsCard {
                source: "archive".into(),
                label: "Eidetic archive".into(),
                bytes: serde_json::to_vec(&PortableCardV1 {
                    title: "Eidetic archive".into(),
                    values: Vec::new(),
                    badges: Vec::new(),
                    media: Vec::new(),
                })
                .map_err(|e| e.to_string())?,
                actions: vec![session_item::archive_action(
                    COMPOSE_CODICILS_INTENT,
                    "Compose codicils",
                )],
                kind: None,
                media: Vec::new(),
            }];
            for manifest in manifests {
                let id = manifest.id.to_string();
                // Each session the codicil was saved from, so a view can put
                // the codicil on its session.
                let saved_from = shared
                    .codicil_sessions(&mut store, &id)
                    .await
                    .into_iter()
                    .map(|session| CardValueV1 {
                        label: mere_overmap::wire::ARCHIVE_SESSION_LABEL.into(),
                        value: session.as_uuid().to_string(),
                    });
                let card = PortableCardV1 {
                    title: format!("Codicil {}", pandect::graph_codicil::codicil_short_id(&id)),
                    values: vec![
                        CardValueV1 {
                            label: "Codicil".into(),
                            value: id.clone(),
                        },
                        CardValueV1 {
                            label: "Sources".into(),
                            value: manifest
                                .provenance
                                .upstream
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(", "),
                        },
                    ]
                    .into_iter()
                    .chain(saved_from)
                    .collect(),
                    badges: vec!["Read-only archive".into()],
                    media: Vec::new(),
                };
                cards.push(SessionsCard {
                    source: id,
                    label: card.title.clone(),
                    bytes: serde_json::to_vec(&card).map_err(|e| e.to_string())?,
                    actions: vec![session_item::archive_action(
                        OPEN_CODICIL_INTENT,
                        "Open as session",
                    )],
                    kind: None,
                    media: Vec::new(),
                });
            }
            Ok(cards)
        })
    }

    fn invoke_archive(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        if intent.observed_epoch != SESSIONS_EPOCH {
            return Ok(IntentResult::Stale {
                current_epoch: SESSIONS_EPOCH,
                current_revision: self.sessions_revision(),
            });
        }
        let outcome = (|| -> Result<Option<(SessionId, SharedHost)>, String> {
            let payload: ArchiveActionV1 =
                serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
            if payload.schema != ARCHIVE_SCHEMA {
                return Err("unknown archive schema".into());
            }
            let ids = payload
                .codicils
                .iter()
                .map(|id| pandect::graph_codicil::parse_codicil_id(id))
                .collect::<Result<Vec<_>, _>>()?;
            let shared = Arc::clone(&self.shared);
            let app = self.app.clone();
            run(async move {
                let mut store = shared.archive.clone();
                match intent.intent.as_str() {
                    SAVE_CODICIL_INTENT if ids.is_empty() => {
                        let id = payload.session.ok_or("save names no session")?;
                        let host = shared.host(id).await?;
                        let mut host = host.lock().await;
                        let payload = host.graph_session().profiled_codicil_payload();
                        let id = pandect::graph_codicil::save_profiled_session_codicil_checked(
                            &mut store,
                            payload,
                            pandect::graph_codicil::archive_timestamp(),
                            |graph| shared.validate(graph),
                        )
                        .await
                        .map_err(|e| e.to_string())?;
                        host.archive_receipt(&app, id.to_string())
                            .await
                            .map_err(|e| e.to_string())?;
                    },
                    OPEN_CODICIL_INTENT if payload.session.is_none() && ids.len() == 1 => {
                        let manifest = shared
                            .sessions()
                            .open_codicil_from_checked(
                                &mut store,
                                ids[0],
                                shared.author(&app),
                                |graph| shared.validate(graph),
                            )
                            .await
                            .map_err(|e| e.to_string())?;
                        shared.moved();
                        return Ok(Some((
                            manifest.session_id,
                            shared.host(manifest.session_id).await?,
                        )));
                    },
                    COMPOSE_CODICILS_INTENT if payload.session.is_none() && ids.len() >= 2 => {
                        if pandect::graph_codicil::compose_graph_codicils_checked(
                            &mut store,
                            &ids,
                            pandect::graph_codicil::archive_timestamp(),
                            |graph| shared.validate(graph),
                        )
                        .await
                        .map_err(|e| e.to_string())?
                        .is_none()
                        {
                            return Err("compose names a missing codicil".into());
                        }
                    },
                    _ => return Err("invalid archive action or operands".into()),
                }
                shared.moved();
                Ok(None)
            })
        })();
        match outcome {
            Ok(Some((id, host))) => {
                self.attached = id;
                self.host = host;
                self.seen_graph = (SceneEpoch(0), Revision(0));
                Ok(IntentResult::Accepted)
            },
            Ok(None) => Ok(IntentResult::Accepted),
            Err(reason) => Ok(IntentResult::Rejected { reason }),
        }
    }

    fn invoke_sessions(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        // A step names its session by id, so it is taken at any revision of
        // the list (§7 item 33), though not across an epoch.
        if intent.observed_epoch != SESSIONS_EPOCH {
            return Ok(IntentResult::Stale {
                current_epoch: SESSIONS_EPOCH,
                current_revision: self.sessions_revision(),
            });
        }
        if matches!(
            intent.intent.as_str(),
            RESOLVE_MERGE_INTENT | CANCEL_MERGE_INTENT
        ) {
            return self.invoke_merge(intent);
        }
        let names_session = match intent.intent.as_str() {
            MINT_SESSION_INTENT => false,
            ATTACH_SESSION_INTENT
            | FORK_SESSION_INTENT
            | BRANCH_SESSION_INTENT
            | MERGE_SESSION_INTENT
            | TRASH_SESSION_INTENT
            | RESTORE_SESSION_INTENT => true,
            other => {
                return Ok(IntentResult::Rejected {
                    reason: format!("the sessions projection offers no {other:?}"),
                });
            },
        };
        let payload: SessionsActionV1 = match serde_json::from_slice(&intent.payload) {
            Ok(payload) => payload,
            Err(error) => {
                return Ok(IntentResult::Rejected {
                    reason: format!("invalid sessions payload: {error}"),
                });
            },
        };
        if payload.schema != SESSIONS_SCHEMA {
            return Ok(IntentResult::Rejected {
                reason: format!("unknown sessions schema {:?}", payload.schema),
            });
        }
        // The named session must be the mere's, and its card must offer the
        // step as the session stands now, whatever the list showed.
        let mut merge_into = None;
        let target = if names_session {
            let Some(id) = payload.session else {
                return Ok(IntentResult::Rejected {
                    reason: format!("{:?} names no session", intent.intent),
                });
            };
            let manifests =
                run(self.shared.sessions().list()).map_err(|error| error.to_string())?;
            let Some(manifest) = manifests.iter().find(|manifest| manifest.session_id == id) else {
                return Ok(IntentResult::Rejected {
                    reason: format!("this mere holds no session {}", id.as_uuid()),
                });
            };
            let pending = run(async { self.shared.pending_merges.lock().await.clone() });
            if !self
                .session_actions(&manifests, manifest, &pending)
                .iter()
                .any(|action| action.intent.0 == intent.intent)
            {
                return Ok(IntentResult::Rejected {
                    reason: format!(
                        "session {} does not offer {:?} now",
                        id.as_uuid(),
                        intent.intent
                    ),
                });
            }
            if intent.intent == MERGE_SESSION_INTENT {
                let Some(into) = merge_target(&manifests, manifest) else {
                    return Ok(IntentResult::Rejected {
                        reason: "only a branch or picked-off part merges back".into(),
                    });
                };
                merge_into = Some(into);
            }
            Some(id)
        } else {
            None
        };
        let shared = Arc::clone(&self.shared);
        let author = shared.author(&self.app);
        let app = self.app.clone();
        let outcome: Result<Option<(SessionId, SharedHost)>, String> = run(async move {
            match (intent.intent.as_str(), target) {
                (MINT_SESSION_INTENT, None) => {
                    shared
                        .sessions()
                        .mint(author, payload.display_name)
                        .await
                        .map_err(|error| error.to_string())?;
                },
                (ATTACH_SESSION_INTENT, Some(id)) => {
                    return Ok(Some((id, shared.host(id).await?)));
                },
                (FORK_SESSION_INTENT, Some(id)) => {
                    let parent = shared.host(id).await?;
                    let parent = parent.lock().await;
                    let session = parent.graph_session();
                    shared
                        .sessions()
                        .fork_at(session, session.journal().live_cursor(), author)
                        .await
                        .map_err(|error| error.to_string())?;
                },
                (BRANCH_SESSION_INTENT, Some(id)) => {
                    let parent = shared.host(id).await?;
                    let parent = parent.lock().await;
                    let session = parent.graph_session();
                    shared
                        .sessions()
                        .branch_at(session, session.journal().live_cursor(), author)
                        .await
                        .map_err(|error| error.to_string())?;
                },
                (MERGE_SESSION_INTENT, Some(id)) => {
                    let into = merge_into.ok_or("a merge names no session to merge into")?;
                    let (target, source) = (shared.host(into).await?, shared.host(id).await?);
                    let mut target = target.lock().await;
                    let mut source = source.lock().await;
                    let sessions = shared.sessions();
                    let plan = target
                        .plan_merge(&sessions, &source)
                        .await
                        .map_err(|error| error.to_string())?;
                    if plan.is_clean() {
                        target
                            .apply_merge(&sessions, &mut source, plan, &BTreeMap::new(), &app)
                            .await
                            .map_err(|error| error.to_string())?;
                    } else {
                        shared
                            .pending_merges
                            .lock()
                            .await
                            .insert(*id.as_uuid(), HeldMerge::new(plan));
                    }
                },
                (TRASH_SESSION_INTENT, Some(id)) => {
                    let host = shared.host(id).await?;
                    host.lock()
                        .await
                        .trash(&app)
                        .await
                        .map_err(|error| error.to_string())?;
                },
                (RESTORE_SESSION_INTENT, Some(id)) => {
                    let host = shared.host(id).await?;
                    host.lock()
                        .await
                        .restore(&app)
                        .await
                        .map_err(|error| error.to_string())?;
                },
                (other, _) => return Err(format!("the sessions projection offers no {other:?}")),
            }
            shared.moved();
            Ok(None)
        });
        match outcome {
            Ok(Some((id, host))) => {
                self.attached = id;
                self.host = host;
                // The attached session's graph is new to this connection; the
                // others' views of the sessions did not move.
                self.seen_graph = (SceneEpoch(0), Revision(0));
                Ok(IntentResult::Accepted)
            },
            Ok(None) => Ok(IntentResult::Accepted),
            Err(reason) => Ok(IntentResult::Rejected { reason }),
        }
    }

    /// Apply or cancel a pending merge, named by its source session.
    fn invoke_merge(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        let payload: MergeResolutionV1 = match serde_json::from_slice(&intent.payload) {
            Ok(payload) => payload,
            Err(error) => {
                return Ok(IntentResult::Rejected {
                    reason: format!("invalid merge payload: {error}"),
                });
            },
        };
        if payload.schema != MERGE_SCHEMA {
            return Ok(IntentResult::Rejected {
                reason: format!("unknown merge schema {:?}", payload.schema),
            });
        }
        let shared = Arc::clone(&self.shared);
        let app = self.app.clone();
        let cancel = intent.intent == CANCEL_MERGE_INTENT;
        let outcome: Result<(), String> = run(async move {
            let Some(held) = shared
                .pending_merges
                .lock()
                .await
                .remove(payload.source.as_uuid())
            else {
                return Err(format!(
                    "no merge of {} is pending",
                    payload.source.as_uuid()
                ));
            };
            if cancel {
                shared.moved();
                return Ok(());
            }
            let choices: BTreeMap<_, _> = payload
                .choices
                .into_iter()
                .map(|chosen| (chosen.part, chosen.choice))
                .collect();
            let plan = held.plan.clone();
            let (target, source) = (
                shared.host(plan.target()).await?,
                shared.host(plan.source()).await?,
            );
            let mut target = target.lock().await;
            let mut source = source.lock().await;
            let sessions = shared.sessions();
            let applied = target
                .apply_merge(&sessions, &mut source, plan.clone(), &choices, &app)
                .await;
            let Err(error) = applied else {
                shared.moved();
                return Ok(());
            };
            shared.moved();
            // A missing choice leaves the merge pending as it was.
            if matches!(
                error,
                MereHostError::Session(pandect::SessionError::Unresolved(_))
            ) {
                shared
                    .pending_merges
                    .lock()
                    .await
                    .insert(*plan.source().as_uuid(), held);
                return Err(error.to_string());
            }
            // Anything else (a session moved, a value given by hand that does
            // not fit): plan the merge again and carry over the choices that
            // still apply, naming the ones dropped.
            let again = target
                .plan_merge(&sessions, &source)
                .await
                .map_err(|replan| format!("{error}; planning it again failed: {replan}"))?;
            if again.is_clean() {
                target
                    .apply_merge(&sessions, &mut source, again, &BTreeMap::new(), &app)
                    .await
                    .map_err(|error| error.to_string())?;
                return Ok(());
            }
            let (carried, dropped) = again.carry_choices(&choices);
            let reason = format!(
                "{error}. The merge was planned again: {} of your choices carried over, {} dropped.",
                carried.len(),
                dropped.len()
            );
            shared.pending_merges.lock().await.insert(
                *again.source().as_uuid(),
                HeldMerge {
                    plan: again,
                    carried,
                    dropped,
                },
            );
            Err(reason)
        });
        Ok(match outcome {
            Ok(()) => IntentResult::Accepted,
            Err(reason) => IntentResult::Rejected { reason },
        })
    }

    fn invoke_graph(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        if matches!(self.shared.domain.as_str(), "divination" | "personal-facts")
            && intent.target != InstanceId(0)
        {
            return Ok(IntentResult::Rejected {
                reason: "domain writes require a complete journaled session candidate".into(),
            });
        }
        // Stage the first edit and its fork together. A refused candidate leaves
        // the thaw, session set and store byte-for-byte unchanged.
        let read_only = run(async {
            self.host
                .lock()
                .await
                .graph_session()
                .manifest()
                .codicil_read_only
        });
        if read_only {
            let shared = Arc::clone(&self.shared);
            let parent = Arc::clone(&self.host);
            let app = self.app.clone();
            let candidate = run(async move {
                let parent = parent.lock().await;
                if intent.observed_epoch != parent.projection_epoch() {
                    return Ok::<_, String>(Err(IntentResult::Stale {
                        current_epoch: parent.projection_epoch(),
                        current_revision: parent.projection_revision(),
                    }));
                }
                if Some(intent.target) != parent.session_instance() {
                    return Ok(Err(IntentResult::Rejected {
                        reason: "read-only archive; explicitly fork before node actions".into(),
                    }));
                }
                let source = parent.graph_session();
                let author = shared.author(&app);
                let mut fork = shared
                    .sessions()
                    .begin_fork_at(source, source.journal().live_cursor(), author.clone())
                    .map_err(|e| e.to_string())?;
                match intent.intent.as_str() {
                    session_item::APPLY_EDITS_INTENT => {
                        let payload: ApplyEditsV1 =
                            serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
                        if payload.schema != session_item::APPLY_EDITS_SCHEMA
                            || payload.edits.is_empty()
                        {
                            return Err("invalid or empty archive edit".into());
                        }
                        if payload
                            .expected_revision
                            .is_some_and(|expected| expected != parent.projection_revision())
                        {
                            return Ok(Err(IntentResult::Stale {
                                current_epoch: parent.projection_epoch(),
                                current_revision: parent.projection_revision(),
                            }));
                        }
                        fork.apply_now(author, payload.edits)
                            .map_err(|e| e.to_string())?;
                        if fork.journal().live_cursor().0 == 0 {
                            return Err("archive edit changes nothing".into());
                        }
                    },
                    session_item::SET_VIEW_INTENT => {
                        let payload: SetViewV1 =
                            serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
                        if payload.schema != session_item::SET_VIEW_SCHEMA {
                            return Err("unknown view schema".into());
                        }
                        fork.set_view_now(
                            author,
                            pandect::ViewKey::new(&app, payload.view).map_err(|e| e.to_string())?,
                            payload.state,
                        )
                        .map_err(|e| e.to_string())?;
                    },
                    _ => return Err("read-only archive; fork before this action".into()),
                }
                fork.flush(SystemTime::now())
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(Ok(fork.id()))
            });
            match candidate {
                Ok(Ok(id)) => {
                    self.attached = id;
                    self.host = run(self.shared.host(id))?;
                    self.shared.moved();
                    self.seen_graph = (SceneEpoch(0), Revision(0));
                    return Ok(IntentResult::Accepted);
                },
                Ok(Err(result)) => return Ok(result),
                Err(reason) => return Ok(IntentResult::Rejected { reason }),
            }
        }
        let host = Arc::clone(&self.host);
        let app = self.app.clone();
        let shared = Arc::clone(&self.shared);
        let (result, seen) = run(async move {
            let mut host = host.lock().await;
            if Some(intent.target) != host.session_instance() {
                host.set_access_context(AccessContext {
                    persona: shared.persona_id(),
                    device: "djinn".into(),
                    at_ms: u64::try_from(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis(),
                    )
                    .unwrap_or(u64::MAX),
                });
                let result = host.through(app.clone(), |host| host.invoke(intent));
                let result = match result {
                    Ok(IntentResult::Accepted) => host
                        .persist(now_secs())
                        .await
                        .map(|()| IntentResult::Accepted)
                        .map_err(|e| e.to_string()),
                    Ok(result) => Ok(result),
                    Err(error) => Ok(IntentResult::Rejected {
                        reason: error.to_string(),
                    }),
                };
                return (
                    result,
                    Some((host.projection_epoch(), host.projection_revision())),
                );
            }
            let (epoch, revision) = (host.projection_epoch(), host.projection_revision());
            // By id at any revision (§7 item 32), but not across an epoch.
            if intent.observed_epoch != epoch {
                let stale = IntentResult::Stale {
                    current_epoch: epoch,
                    current_revision: revision,
                };
                return (Ok(stale), None);
            }
            let outcome = match intent.intent.as_str() {
                session_item::APPLY_EDITS_INTENT => {
                    match serde_json::from_slice::<ApplyEditsV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::APPLY_EDITS_SCHEMA => {
                            if payload
                                .expected_revision
                                .is_some_and(|expected| expected != revision)
                            {
                                return (
                                    Ok(IntentResult::Stale {
                                        current_epoch: epoch,
                                        current_revision: revision,
                                    }),
                                    None,
                                );
                            }
                            host.transact_edits(&app, payload.edits)
                                .await
                                .map(|()| true)
                                .map_err(|error| error.to_string())
                        },
                        Ok(payload) => Err(format!("unknown apply schema {:?}", payload.schema)),
                        Err(error) => Err(format!("invalid apply payload: {error}")),
                    }
                },
                session_item::UNDO_INTENT | session_item::REDO_INTENT => {
                    match serde_json::from_slice::<StepV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::STEP_SCHEMA => {
                            let step = host
                                .transact_history(&app, intent.intent == session_item::REDO_INTENT)
                                .await;
                            // Undoing a merge frees its source; redoing marks
                            // it again. A host holding it reads the mark anew.
                            if step.as_ref().is_ok_and(Option::is_some)
                                && let Some(source) = host.graph_session().merge_stepped()
                            {
                                let held = shared.hosts.lock().await.get(&source).cloned();
                                if let Some(held) = held {
                                    let _ = held.lock().await.reload_merged_mark().await;
                                }
                                shared.moved();
                            }
                            step.map(|reverted| reverted.is_some())
                                .map_err(|error| error.to_string())
                        },
                        Ok(payload) => Err(format!("unknown step schema {:?}", payload.schema)),
                        Err(error) => Err(format!("invalid step payload: {error}")),
                    }
                },
                session_item::SET_VIEW_INTENT => {
                    match serde_json::from_slice::<SetViewV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::SET_VIEW_SCHEMA => host
                            .transact_view(&app, &payload.view, payload.state)
                            .await
                            .map(|()| true)
                            .map_err(|error| error.to_string()),
                        Ok(payload) => Err(format!("unknown view schema {:?}", payload.schema)),
                        Err(error) => Err(format!("invalid view payload: {error}")),
                    }
                },
                other => Err(format!("the session item does not offer {other:?}")),
            };
            let seen = Some((host.projection_epoch(), host.projection_revision()));
            let result = match outcome {
                Ok(true) => Ok(IntentResult::Accepted),
                Ok(false) => Ok(IntentResult::Rejected {
                    reason: "nothing of yours to step through".to_string(),
                }),
                Err(reason) => Ok(IntentResult::Rejected { reason }),
            };
            (result, seen)
        });
        // An application is not rung for its own change.
        if let Some(seen) = seen {
            self.seen_graph = seen;
        }
        result
    }
}

impl ProjectionCatalog for MereEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        let graph = run(async { self.host.lock().await.local_request() });
        EndpointDescriptor {
            label: format!("Mere: {}", self.shared.domain),
            projections: vec![
                ProjectionOffer {
                    label: "Sessions".to_string(),
                    request: Self::sessions_request(),
                },
                ProjectionOffer {
                    label: "Attached session".to_string(),
                    request: graph,
                },
                ProjectionOffer {
                    label: "Archive".into(),
                    request: ProjectionRequest {
                        version: ProtocolVersion::V1,
                        session: Self::archive_session(),
                        score: Score::new(Arrangement::Spiral(Default::default())),
                    },
                },
            ],
        }
    }
}

impl ProjectionSource for MereEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        match request.session.0.as_str() {
            MERE_SESSIONS => self.sessions_snapshot(),
            MERE_ARCHIVE => {
                let cards = self.archive_cards()?;
                self.cards_snapshot(cards, Self::archive_session(), ARCHIVE_SOURCE)
            },
            MERE_GRAPH => {
                let host = Arc::clone(&self.host);
                let app = self.app.clone();
                let snapshot = run(async move {
                    let mut host = host.lock().await;
                    host.through(app, |host| host.snapshot(request))
                        .map_err(|error| error.to_string())
                })?;
                self.seen_graph = (snapshot.scene.epoch, snapshot.scene.revision);
                Ok(snapshot)
            },
            other => Err(format!("this mere serves no projection {other:?}")),
        }
    }
}

impl PresentationSource for MereEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        match request.session.0.as_str() {
            MERE_SESSIONS | MERE_ARCHIVE => {
                let cards = if request.session.0 == MERE_ARCHIVE {
                    self.archive_cards()?
                } else {
                    self.session_cards()?
                };
                let bytes = cards
                    .into_iter()
                    .flat_map(|card| std::iter::once(card.bytes).chain(card.media))
                    .find(|bytes| ContentHash::of(bytes) == request.resource)
                    .ok_or("sessions resource is no longer current; request a new snapshot")?;
                Ok(ResourceResponse {
                    session: request.session,
                    resource: request.resource,
                    bytes,
                })
            },
            MERE_GRAPH => {
                let host = Arc::clone(&self.host);
                run(async move {
                    let mut host = host.lock().await;
                    host.resource(request).map_err(|error| error.to_string())
                })
            },
            other => Err(format!("this mere serves no projection {other:?}")),
        }
    }
}

impl IntentSink for MereEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        match intent.session.0.as_str() {
            MERE_SESSIONS if intent.intent == SAVE_CODICIL_INTENT => self.invoke_archive(intent),
            MERE_SESSIONS => self.invoke_sessions(intent),
            MERE_ARCHIVE => self.invoke_archive(intent),
            MERE_GRAPH => self.invoke_graph(intent),
            other => Ok(IntentResult::Rejected {
                reason: format!("this mere serves no projection {other:?}"),
            }),
        }
    }
}

impl ProjectionNoticeSource for MereEndpoint {
    type Error = String;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> {
        let sessions = self.shared.lifecycle.load(Ordering::SeqCst);
        if sessions != self.seen_sessions {
            self.seen_sessions = sessions;
            return Ok(Some(CarrierNotice {
                session: Self::sessions_session(),
                epoch: SESSIONS_EPOCH,
                revision: Revision(sessions),
            }));
        }
        if sessions != self.seen_archive {
            self.seen_archive = sessions;
            return Ok(Some(CarrierNotice {
                session: Self::archive_session(),
                epoch: SESSIONS_EPOCH,
                revision: Revision(sessions),
            }));
        }
        let host = Arc::clone(&self.host);
        let (graph, now) = run(async move {
            let host = host.lock().await;
            (
                host.session(),
                (host.projection_epoch(), host.projection_revision()),
            )
        });
        if now == self.seen_graph {
            return Ok(None);
        }
        self.seen_graph = now;
        Ok(Some(CarrierNotice {
            session: graph,
            epoch: now.0,
            revision: now.1,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphshell_endpoint::door::app_admission::{AllowedAppRoutes, AppHello, AppRouteId};
    use graphshell_host::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointSession};
    use kernel::graph::{ContainmentSubKind, EdgeAssertion, SemanticSubKind};
    use mere_overmap::route as mere_route;
    use mere_view::{MereViewModel, MereViewRequest, SessionStep};
    use pandect::{Change, ChangeKind, mere_dir};
    use uuid::Uuid;

    use crate::resident_reservoir::{
        EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn dropping_the_catalog_releases_a_populated_reservoir_and_its_registrar() {
        let Fixture {
            _root,
            reservoir,
            catalog,
            routes,
            ..
        } = fixture(0xc010, "catalog-release").await;
        let weak = catalog.downgrade();
        drop(routes);
        drop(catalog);
        assert!(
            weak.upgrade().is_none(),
            "the reservoir factory must not retain its owning catalog"
        );
        crate::resident_reservoir::ReservoirLane::Open(reservoir)
            .close()
            .await
            .unwrap();
        drop(_root);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn coop_owner_controls_are_local_and_do_not_follow_the_remote_mere_catalog() {
        let remote = Arc::new(std::sync::Mutex::new(ResidentEndpointCatalog::new()));
        let fixture = fixture_serving(0xc00f, "coop-scope", None, Some(remote.clone())).await;
        let route = "coop/coop-scope";
        assert!(
            fixture
                .grants
                .current()
                .admit(&AppRequest {
                    app: AppId::new("graphshell"),
                    route: AppRouteId::new(route).unwrap()
                })
                .is_ok()
        );
        assert!(
            fixture
                .grants
                .current()
                .admit(&AppRequest {
                    app: AppId::new("turnstone"),
                    route: AppRouteId::new(route).unwrap()
                })
                .is_err()
        );
        assert!(
            remote
                .lock()
                .unwrap()
                .open(route, &context("graphshell"))
                .is_err()
        );
        assert!(
            fixture
                .catalog
                .update(|catalog| catalog.open(route, &context("turnstone")))
                .await
                .is_err()
        );
        let endpoint = fixture
            .catalog
            .update(|catalog| catalog.open(route, &context("graphshell")))
            .await
            .unwrap();
        assert_eq!(endpoint.describe().projections.len(), 1);
    }

    fn persona(n: u128) -> PersonaId {
        PersonaId::from_uuid(Uuid::from_u128(n))
    }

    fn context(app: &str) -> AdmittedEndpointContext {
        AdmittedEndpointContext::new(ProjectionSession("v1:test".into()), [0x2b; 32])
            .with_application(app)
    }

    fn node(n: u128) -> pandect::CapturedDelta {
        pandect::CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: Uuid::from_u128(n).to_string(),
            url: format!("https://{n}.test/"),
            position: [0.0, 0.0],
        }
    }

    fn retitle(n: u128, title: &str) -> pandect::CapturedDelta {
        pandect::CapturedDelta::ReplaySetNodeTitleById {
            node_id: Uuid::from_u128(n).to_string(),
            title: title.to_string(),
        }
    }

    /// A reservoir with its route and the mere routes over one shared door,
    /// and the mere for `domain` ensured through the reservoir route.
    struct Fixture {
        _root: tempfile::TempDir,
        reservoir: ResidentReservoir,
        catalog: AppEndpointCatalog,
        grants: AppRouteGrants,
        route: String,
        routes: MereRoutes,
    }

    async fn fixture(persona_n: u128, domain: &str) -> Fixture {
        fixture_with_validator(persona_n, domain, None).await
    }

    async fn fixture_with_validator(
        persona_n: u128,
        domain: &str,
        validator: Option<pandect::GraphValidator>,
    ) -> Fixture {
        fixture_serving(persona_n, domain, validator, None).await
    }

    /// The fixture, with its routes also registered in a dialable host's
    /// catalog (`crate::resident_projection`).
    async fn fixture_serving(
        persona_n: u128,
        domain: &str,
        validator: Option<pandect::GraphValidator>,
        remote: Option<graphshell_host::projection_host::SharedCatalog>,
    ) -> Fixture {
        fixture_serving_handlers(
            persona_n,
            domain,
            validator,
            remote,
            HandlerRegistry::default(),
        )
        .await
    }

    async fn fixture_serving_handlers(
        persona_n: u128,
        domain: &str,
        validator: Option<pandect::GraphValidator>,
        remote: Option<graphshell_host::projection_host::SharedCatalog>,
        handlers: HandlerRegistry,
    ) -> Fixture {
        let root = tempfile::tempdir().unwrap();
        let reservoir = ResidentReservoir::open(root.path(), Some(persona(persona_n)))
            .await
            .unwrap();
        let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        let routes = MereRoutes::new(
            root.path().to_path_buf(),
            persona(persona_n),
            catalog.clone(),
            grants.clone(),
            vec![AppId::new("turnstone"), AppId::new("knot-editor")],
        )
        .with_handlers(handlers);
        let storage = castellan::custody::InMemoryStorage::new();
        let profile = castellan::custody::Profile::new(
            personae::ProfileId("fixture-profile".into()),
            "Public fixture profile",
            personae::Ed25519Keypair::from_seed([0x33; 32]),
        );
        castellan::custody::IdentityStorage::save_profile(&storage, &profile).unwrap();
        let owner = Arc::new(castellan::authority::PersonaeHost::new(
            castellan::custody::IdentityVault::with_profile(storage, profile),
            None,
            graphshell_endpoint::door::identity::VaultProtectionView::Ephemeral,
        ));
        let routes = routes
            .with_custody_endpoint("unused-in-unit-tests".into())
            .with_profile_subject_authority(owner);
        let routes = if let Some(validator) = validator {
            routes.with_domain_validator(domain, validator)
        } else {
            routes.with_divination_authority()
        };
        let routes = match remote {
            Some(remote) => routes.with_remote_catalog(remote),
            None => routes,
        };
        catalog
            .update(|catalog| reservoir.register(catalog, Some(routes.clone())))
            .await
            .unwrap();
        let mut door = catalog
            .update(|catalog| catalog.open(RESIDENT_RESERVOIR_ROUTE, &context("turnstone")))
            .await
            .unwrap();
        let request = door.describe().projections.remove(0).request;
        let listed = door.snapshot(request).unwrap();
        let ensure = IntentInvocation {
            session: listed.session.clone(),
            target: InstanceId(0),
            observed_epoch: listed.scene.epoch,
            observed_revision: listed.scene.revision,
            intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
            payload: serde_json::to_vec(&EnsureMereV1::new(domain)).unwrap(),
        };
        assert_eq!(door.invoke(ensure).unwrap(), IntentResult::Accepted);
        Fixture {
            _root: root,
            reservoir,
            catalog,
            grants,
            route: mere_route_id(domain),
            routes,
        }
    }

    impl Fixture {
        async fn attach(&self, app: &str) -> ResidentEndpointSession {
            self.catalog
                .update(|catalog| catalog.open(&self.route, &context(app)))
                .await
                .unwrap()
        }
    }

    fn projection(session: &mut ResidentEndpointSession, index: usize) -> ProjectionSnapshot {
        let request = session.describe().projections[index].request.clone();
        session.snapshot(request).unwrap()
    }

    fn graph(session: &mut ResidentEndpointSession) -> ProjectionSnapshot {
        projection(session, 1)
    }

    fn session_item_of(snapshot: &ProjectionSnapshot) -> InstanceId {
        snapshot
            .scene
            .active_items_in_order()
            .into_iter()
            .find_map(|(instance, item)| {
                snapshot.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .filter(|source| source.adapter == session_item::SESSION_ITEM_SOURCE)
                    .map(|_| instance)
            })
            .expect("the resident graph shows the session item")
    }

    fn on_item(snapshot: &ProjectionSnapshot, intent: &str, payload: Vec<u8>) -> IntentInvocation {
        IntentInvocation {
            session: snapshot.session.clone(),
            target: session_item_of(snapshot),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: intent.into(),
            payload,
        }
    }

    fn apply(
        snapshot: &ProjectionSnapshot,
        edits: Vec<pandect::CapturedDelta>,
    ) -> IntentInvocation {
        on_item(
            snapshot,
            session_item::APPLY_EDITS_INTENT,
            serde_json::to_vec(&ApplyEditsV1::new(edits)).unwrap(),
        )
    }

    fn undo(snapshot: &ProjectionSnapshot) -> IntentInvocation {
        on_item(
            snapshot,
            session_item::UNDO_INTENT,
            serde_json::to_vec(&StepV1::default()).unwrap(),
        )
    }

    fn nodes(snapshot: &ProjectionSnapshot) -> usize {
        snapshot.scene.active_items_in_order().len() - 1
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn natal_product_authority_admits_replay_and_refuses_tampering_across_apps() {
        let fixture = fixture(0x4e4154, "divination").await;
        let mut product = fixture.attach("turnstone").await;
        let mut other = fixture.attach("knot-editor").await;
        let initial = graph(&mut product);
        let mut candidate = disclosed_graph(&mut product, &initial)
            .into_graph()
            .unwrap();
        let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
        let held = captured.clone();
        candidate.set_recorder(Some(Arc::new(move |delta| {
            held.lock().unwrap().push(delta.clone())
        })));
        let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":"public-natal-subject"}},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":null,"timezone":"UTC","source":{"label":"Public product-authority fixture","reference":null}})).unwrap();
        let birth = personal_facts::birth_resource(&facts).unwrap();
        let record = cleromancy_domain::natal::NatalReading::new(
            "public-source-persona".into(),
            "Public natal subject".into(),
            birth,
            "A synthetic natal question".into(),
            cleromancy_domain::natal::FoldChoice::Refuse,
            1792149302000,
            "00000000000000000000000000000008".into(),
        )
        .unwrap();
        let mut host = cleromancy_domain::CleromancyHost::candidate(candidate).unwrap();
        host.insert_natal_reading(&record).unwrap();
        assert_eq!(
            product
                .invoke(apply(&initial, captured.lock().unwrap().clone()))
                .unwrap(),
            IntentResult::Accepted
        );
        let shared = graph(&mut other);
        let current = disclosed_graph(&mut other, &shared);
        let host =
            cleromancy_domain::CleromancyHost::candidate(current.clone().into_graph().unwrap())
                .unwrap();
        assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
        let mut bad = record.clone();
        bad.question = "Tampered under the sealed id".into();
        assert!(matches!(
            other
                .invoke(apply(
                    &shared,
                    vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                        node_id: kernel::graph::Graph::node_namespace_id(&format!(
                            "cleromancy://natal/{}",
                            record.id
                        ))
                        .to_string(),
                        facet: cleromancy_domain::natal::NATAL_FACET.into(),
                        value_json: serde_json::to_string(&bad).unwrap()
                    }]
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        let after = graph(&mut other);
        assert_eq!(
            serde_json::to_value(disclosed_graph(&mut other, &after)).unwrap(),
            serde_json::to_value(current).unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn natal_houses_authority_admits_both_systems_and_refuses_cusp_tamper() {
        for system in [
            cleromancy_domain::natal_houses::HouseSystem::WholeSign,
            cleromancy_domain::natal_houses::HouseSystem::Equal,
        ] {
            let fixture = fixture(0x4e4155, "divination").await;
            let mut product = fixture.attach("turnstone").await;
            let mut other = fixture.attach("knot-editor").await;
            let initial = graph(&mut product);
            let mut candidate = disclosed_graph(&mut product, &initial)
                .into_graph()
                .unwrap();
            let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
            let held = captured.clone();
            candidate.set_recorder(Some(Arc::new(move |delta| {
                held.lock().unwrap().push(delta.clone())
            })));
            let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":"public-natal-subject"}},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":{"label":"Public coordinate fixture","latitude_microdegrees":40712800,"longitude_microdegrees":-74006000,"precision":"Public synthetic"},"timezone":"UTC","source":{"label":"Public product-authority fixture","reference":null}})).unwrap();
            let birth = personal_facts::birth_resource(&facts).unwrap();
            let record = cleromancy_domain::natal::NatalReading::new(
                "public-source-persona".into(),
                "Public natal subject".into(),
                birth,
                "A synthetic natal question".into(),
                cleromancy_domain::natal::FoldChoice::Refuse,
                1792149302000,
                "00000000000000000000000000000008".into(),
            )
            .unwrap()
            .with_houses(system)
            .unwrap();
            let mut host = cleromancy_domain::CleromancyHost::candidate(candidate).unwrap();
            host.insert_natal_reading(&record).unwrap();
            assert_eq!(
                product
                    .invoke(apply(&initial, captured.lock().unwrap().clone()))
                    .unwrap(),
                IntentResult::Accepted
            );
            let shared = graph(&mut other);
            let current = disclosed_graph(&mut other, &shared);
            let host =
                cleromancy_domain::CleromancyHost::candidate(current.clone().into_graph().unwrap())
                    .unwrap();
            assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
            let mut bad = record.clone();
            if let cleromancy_domain::natal_houses::HouseOutcome::Calculated {
                cusps_microdegrees,
                ..
            } = &mut bad.houses.as_mut().unwrap().outcome
            {
                cusps_microdegrees[0] = (cusps_microdegrees[0] + 1) % 360_000_000;
            } else {
                panic!("expected actual houses");
            }
            assert!(matches!(
                other
                    .invoke(apply(
                        &shared,
                        vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                            node_id: kernel::graph::Graph::node_namespace_id(&format!(
                                "cleromancy://natal/{}",
                                record.id
                            ))
                            .to_string(),
                            facet: cleromancy_domain::natal::NATAL_FACET.into(),
                            value_json: serde_json::to_string(&bad).unwrap()
                        }]
                    ))
                    .unwrap(),
                IntentResult::Rejected { .. }
            ));
            let after = graph(&mut other);
            assert_eq!(
                serde_json::to_value(disclosed_graph(&mut other, &after)).unwrap(),
                serde_json::to_value(current).unwrap()
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn natal_cap_authority_admits_retained_observations_and_refuses_motion_tamper() {
        for observed in [false, true] {
            let fixture = fixture(0x4e4155, "divination").await;
            let mut product = fixture.attach("turnstone").await;
            let mut other = fixture.attach("knot-editor").await;
            let initial = graph(&mut product);
            let mut candidate = disclosed_graph(&mut product, &initial)
                .into_graph()
                .unwrap();
            let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
            let held = captured.clone();
            candidate.set_recorder(Some(Arc::new(move |delta| {
                held.lock().unwrap().push(delta.clone())
            })));
            let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":"public-natal-subject"}},"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":{"label":"Public coordinate fixture","latitude_microdegrees":40712800,"longitude_microdegrees":-74006000,"precision":"Public synthetic"},"timezone":"UTC","source":{"label":"Public product-authority fixture","reference":null}})).unwrap();
            let birth = personal_facts::birth_resource(&facts).unwrap();
            let record = cleromancy_domain::natal::NatalReading::new(
                "public-source-persona".into(),
                "Public natal subject".into(),
                birth,
                "A synthetic natal question".into(),
                cleromancy_domain::natal::FoldChoice::Refuse,
                1792149302000,
                "00000000000000000000000000000008".into(),
            )
            .unwrap()
            .with_supplement(
                Some(cleromancy_domain::natal_cap::HouseChoice::Placidus),
                observed.then(|| cleromancy_domain::natal_cap::OrientationSnapshot {
                    finals2000a: include_str!("../tests/fixtures/natal-cap-finals2000a.txt").into(),
                }),
            )
            .unwrap();
            let mut host = cleromancy_domain::CleromancyHost::candidate(candidate).unwrap();
            host.insert_natal_reading(&record).unwrap();
            assert_eq!(
                product
                    .invoke(apply(&initial, captured.lock().unwrap().clone()))
                    .unwrap(),
                IntentResult::Accepted
            );
            let shared = graph(&mut other);
            let current = disclosed_graph(&mut other, &shared);
            let host =
                cleromancy_domain::CleromancyHost::candidate(current.clone().into_graph().unwrap())
                    .unwrap();
            assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
            let mut bad = record.clone();
            if let cleromancy_domain::natal_cap::MotionOutcome::Calculated { planets, .. } =
                &mut bad.supplement.as_mut().unwrap().motion
            {
                planets[0].microdegrees_per_tt_day += 1;
            } else {
                panic!("expected motion")
            };
            assert!(matches!(
                other
                    .invoke(apply(
                        &shared,
                        vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                            node_id: kernel::graph::Graph::node_namespace_id(&format!(
                                "cleromancy://natal/{}",
                                record.id
                            ))
                            .to_string(),
                            facet: cleromancy_domain::natal::NATAL_FACET.into(),
                            value_json: serde_json::to_string(&bad).unwrap()
                        }]
                    ))
                    .unwrap(),
                IntentResult::Rejected { .. }
            ));
            let after = graph(&mut other);
            assert_eq!(
                serde_json::to_value(disclosed_graph(&mut other, &after)).unwrap(),
                serde_json::to_value(current).unwrap()
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn natal_aspects_authority_admits_cross_app_and_refuses_resealed_results_and_rules() {
        use cleromancy_domain::natal_aspects::{AspectOutcome, AspectRules};

        let fixture = fixture(0x4e4156, "divination").await;
        let mut product = fixture.attach("turnstone").await;
        let mut other = fixture.attach("knot-editor").await;
        let initial = graph(&mut product);
        let mut candidate = disclosed_graph(&mut product, &initial)
            .into_graph()
            .unwrap();
        let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
        let held = captured.clone();
        candidate.set_recorder(Some(Arc::new(move |delta| {
            held.lock().unwrap().push(delta.clone())
        })));
        let facts: dramatis::birth::BirthFacts = serde_json::from_value(serde_json::json!({
            "schema": "dramatis.birth-facts/v1",
            "subject": {"Profile": {"id": "public-natal-subject"}},
            "date": {"year": 2000, "month": 1, "day": 1},
            "time": {"Known": {"hour": 12, "minute": 0, "second": 0}},
            "place": null,
            "timezone": "UTC",
            "source": {"label": "Public product-authority fixture", "reference": null}
        }))
        .unwrap();
        let record = cleromancy_domain::natal::NatalReading::new(
            "public-source-persona".into(),
            "Public natal subject".into(),
            personal_facts::birth_resource(&facts).unwrap(),
            "A synthetic natal aspects question".into(),
            cleromancy_domain::natal::FoldChoice::Refuse,
            1792149302000,
            "00000000000000000000000000000008".into(),
        )
        .unwrap()
        .with_aspects(AspectRules::default())
        .unwrap();
        let AspectOutcome::Calculated { hits, .. } = &record.aspects.as_ref().unwrap().outcome
        else {
            panic!("expected actual aspects");
        };
        assert!(!hits.is_empty(), "fixture must exercise calculated results");
        assert!(hits.iter().any(|hit| hit.orb_millidegrees > 0));
        let mut host = cleromancy_domain::CleromancyHost::candidate(candidate).unwrap();
        host.insert_natal_reading(&record).unwrap();
        assert_eq!(
            product
                .invoke(apply(&initial, captured.lock().unwrap().clone()))
                .unwrap(),
            IntentResult::Accepted
        );
        let shared = graph(&mut other);
        let current = disclosed_graph(&mut other, &shared);
        let host =
            cleromancy_domain::CleromancyHost::candidate(current.clone().into_graph().unwrap())
                .unwrap();
        assert_eq!(host.natal_readings().unwrap(), vec![record.clone()]);
        let mere = fixture.reservoir.meres().await.remove(0);
        let sessions = mere_dir(fixture._root.path(), persona(0x4e4156), mere.id).join("sessions");
        let product_snapshot = graph(&mut product);
        let product_current = disclosed_graph(&mut product, &product_snapshot);
        let durable = session_tree_bytes(&sessions);
        assert!(durable.keys().any(|path| path.ends_with("changes.jsonl")));

        for fault in ["output", "rules"] {
            let mut value = serde_json::to_value(&record).unwrap();
            match fault {
                "output" => {
                    value["aspects"]["outcome"]["hits"][0]["orb_millidegrees"] =
                        serde_json::json!(99999)
                },
                _ => {
                    value["aspects"]["rules"] = serde_json::to_value(AspectRules {
                        conjunction_millidegrees: 0,
                        opposition_millidegrees: 0,
                        trine_millidegrees: 0,
                        square_millidegrees: 0,
                        sextile_millidegrees: 0,
                    })
                    .unwrap()
                },
            }
            // Forge a valid content seal and matching canonical node identity.
            // Refusal must come from replay, rather than an old id or URL.
            value.as_object_mut().unwrap().remove("id");
            let id = blake3::hash(&serde_json::to_vec(&value).unwrap())
                .to_hex()
                .to_string();
            value["id"] = serde_json::json!(id);
            let bad: cleromancy_domain::natal::NatalReading =
                serde_json::from_value(value).unwrap();
            assert_eq!(
                bad.validate().unwrap_err(),
                "Natal aspects differ from their retained rules and chart",
                "{fault}"
            );
            let url = format!("cleromancy://natal/{}", bad.id);
            let node_id = kernel::graph::Graph::node_namespace_id(&url).to_string();
            let before = graph(&mut other);
            let result = other
                .invoke(apply(
                    &before,
                    vec![
                        pandect::CapturedDelta::ReplayAddNodeWithIdIfMissing {
                            id: node_id.clone(),
                            url,
                            position: [0.0, 0.0],
                        },
                        pandect::CapturedDelta::ReplaySetNodeFacetById {
                            node_id,
                            facet: cleromancy_domain::natal::NATAL_FACET.into(),
                            value_json: serde_json::to_string(&bad).unwrap(),
                        },
                    ],
                ))
                .unwrap();
            assert!(
                matches!(&result, IntentResult::Rejected { reason }
                    if reason.contains("Natal aspects differ from their retained rules and chart")),
                "{fault}: {result:?}"
            );
            for (app, expected) in [(&mut product, &product_current), (&mut other, &current)] {
                let after = graph(app);
                assert_eq!(after.scene.revision, before.scene.revision, "{fault}");
                assert_eq!(
                    serde_json::to_value(disclosed_graph(app, &after)).unwrap(),
                    serde_json::to_value(expected).unwrap(),
                    "{fault}"
                );
            }
            assert_eq!(
                session_tree_bytes(&sessions),
                durable,
                "{fault}: durable session changed"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn configured_record_handlers_validate_targets_and_persist_access() {
        use graphshell::handlers::{HandlerOffer, OpenAddressV1, intent_id};
        let handler = HandlerOffer {
            id: "configured-browser".into(),
            label: "Open in browser".into(),
            explanation: "Owner-configured address handoff".into(),
            address_kinds: vec![kernel::address::AddressKind::Http],
        };
        let fixture = fixture_serving_handlers(
            0xac71,
            "handler-acceptance",
            None,
            None,
            HandlerRegistry::new(vec![handler.clone()]),
        )
        .await;
        let mut app = fixture.attach("turnstone").await;
        let initial = graph(&mut app);
        assert_eq!(
            app.invoke(apply(&initial, vec![node(1), node(2)])).unwrap(),
            IntentResult::Accepted
        );
        let current = graph(&mut app);
        let target = current
            .scene
            .active_items_in_order()
            .into_iter()
            .find_map(|(id, item)| {
                current.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .filter(|source| source.id == Uuid::from_u128(1).to_string())
                    .map(|_| id)
            })
            .unwrap();
        let binding = current
            .presentation
            .bindings
            .iter()
            .find(|b| b.instance == target)
            .unwrap();
        assert_eq!(
            current.presentation.offers[&binding.key][0]
                .semantics
                .actions,
            vec![handler.advertised_action()]
        );
        let request = |address: &str| IntentInvocation {
            session: current.session.clone(),
            target,
            observed_epoch: current.scene.epoch,
            observed_revision: current.scene.revision,
            intent: intent_id(&handler.id),
            payload: serde_json::to_vec(&OpenAddressV1 {
                address: address.into(),
                handler: handler.id.clone(),
            })
            .unwrap(),
        };
        assert!(matches!(
            app.invoke(request("https://2.test/")).unwrap(),
            IntentResult::Rejected { .. }
        ));
        let accepted = request("https://1.test/");
        assert_eq!(
            app.invoke(accepted.clone()).unwrap(),
            IntentResult::Accepted
        );
        assert!(matches!(
            app.invoke(accepted).unwrap(),
            IntentResult::Stale { .. }
        ));
        let saved = graph(&mut app);
        let material = disclosed_graph(&mut app, &saved).into_graph().unwrap();
        let key = material
            .nodes()
            .find(|(_, n)| n.id == Uuid::from_u128(1))
            .unwrap()
            .0;
        let history = graphshell::access::access_history(&material, key).unwrap();
        assert_eq!(history.records.len(), 1);
        assert_eq!(history.records[0].handler, handler.id);
        assert!(history.records[0].at_ms > 0);
        // Reopen the durable session with a new host, not just another view.
        drop(app);
        let shared = fixture
            .routes
            .inner
            .served
            .lock()
            .await
            .values()
            .next()
            .unwrap()
            .clone();
        shared.hosts.lock().await.clear();
        let mut reopened = fixture.attach("knot-editor").await;
        let snapshot = graph(&mut reopened);
        let material = disclosed_graph(&mut reopened, &snapshot)
            .into_graph()
            .unwrap();
        let key = material
            .nodes()
            .find(|(_, n)| n.id == Uuid::from_u128(1))
            .unwrap()
            .0;
        assert_eq!(
            graphshell::access::access_history(&material, key).unwrap(),
            history
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn two_applications_edit_one_session_and_hear_each_other() {
        let fixture = fixture(21, "divination").await;
        for app in ["turnstone", "knot-editor"] {
            let hello =
                AppHello::for_route(AppId::new(app), AppRouteId::new(&fixture.route).unwrap())
                    .accept()
                    .unwrap();
            assert!(
                fixture.grants.current().admit(&hello).is_ok(),
                "{app} is granted the mere's route as it is ensured"
            );
        }

        let mut turnstone = fixture.attach("turnstone").await;
        let mut knot = fixture.attach("knot-editor").await;
        let first = graph(&mut turnstone);
        let seen = graph(&mut knot);
        assert_eq!(
            first.scene.epoch, seen.scene.epoch,
            "both attach to one session"
        );
        assert_eq!(nodes(&first), 0);

        // Turnstone edits; Knot is rung and sees it, and Turnstone is not rung
        // for its own change.
        assert_eq!(
            turnstone.invoke(apply(&first, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(turnstone.poll_notice().unwrap(), None);
        let bell = knot.poll_notice().unwrap().expect("knot is rung");
        assert_eq!(bell.session, ProjectionSession(MERE_GRAPH.into()));
        assert_eq!(nodes(&graph(&mut knot)), 1);

        // Knot edits from the revision it saw first: accepted by id (§7 item 32).
        assert_eq!(
            knot.invoke(apply(&seen, vec![retitle(1, "Knot's title")]))
                .unwrap(),
            IntentResult::Accepted
        );
        assert!(
            turnstone.poll_notice().unwrap().is_some(),
            "turnstone is rung"
        );

        // Undo is each application's own: Knot's takes back Knot's retitle and
        // leaves Turnstone's node.
        let now = graph(&mut knot);
        assert_eq!(knot.invoke(undo(&now)).unwrap(), IntentResult::Accepted);
        assert_eq!(nodes(&graph(&mut turnstone)), 1);
        let again = graph(&mut knot);
        assert!(
            matches!(
                knot.invoke(undo(&again)).unwrap(),
                IntentResult::Rejected { .. }
            ),
            "knot has nothing more of its own to undo"
        );

        // The stored changes name each application, supplied by the door.
        let mere = fixture.reservoir.meres().await.remove(0);
        let sessions = mere_dir(fixture._root.path(), persona(21), mere.id).join("sessions");
        let mut changes = String::new();
        for entry in std::fs::read_dir(sessions).unwrap() {
            changes +=
                &std::fs::read_to_string(entry.unwrap().path().join("changes.jsonl")).unwrap();
        }
        assert!(changes.contains("\"via\":\"turnstone\""), "{changes}");
        assert!(changes.contains("\"via\":\"knot-editor\""), "{changes}");
    }

    /// The sessions a list shows, oldest first.
    fn session_ids(snapshot: &ProjectionSnapshot) -> Vec<SessionId> {
        snapshot
            .scene
            .active_items_in_order()
            .into_iter()
            .filter_map(|(_, item)| {
                snapshot.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .filter(|source| source.adapter == SESSIONS_SOURCE)
                    .and_then(|source| Uuid::parse_str(&source.id).ok())
                    .map(SessionId)
            })
            .collect()
    }

    /// A lifecycle step taken from `snapshot`. The payload names the session,
    /// so the target is only a placeholder.
    fn sessions_step(
        snapshot: &ProjectionSnapshot,
        intent: &str,
        payload: SessionsActionV1,
    ) -> IntentInvocation {
        IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: intent.into(),
            payload: serde_json::to_vec(&payload).unwrap(),
        }
    }

    fn archive_ids(snapshot: &ProjectionSnapshot) -> Vec<String> {
        snapshot
            .scene
            .active_items_in_order()
            .into_iter()
            .filter_map(|(_, item)| {
                snapshot.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .filter(|source| source.adapter == ARCHIVE_SOURCE && source.id != "archive")
                    .map(|source| source.id.clone())
            })
            .collect()
    }

    fn archive_step(
        snapshot: &ProjectionSnapshot,
        intent: &str,
        payload: ArchiveActionV1,
    ) -> IntentInvocation {
        IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: intent.into(),
            payload: serde_json::to_vec(&payload).unwrap(),
        }
    }

    fn disclosed_graph(
        session: &mut impl PresentationSource<Error = String>,
        snapshot: &ProjectionSnapshot,
    ) -> session_item::SessionGraphV1 {
        let offer = &snapshot
            .presentation
            .offers_for(session_item_of(snapshot))
            .unwrap()[0];
        let card = session
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: offer.resource,
            })
            .unwrap();
        assert!(card.has_valid_address());
        let card: PortableCardV1 = serde_json::from_slice(&card.bytes).unwrap();
        let graph = session
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: card.media[0],
            })
            .unwrap();
        assert!(graph.has_valid_address());
        serde_json::from_slice(&graph.bytes).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn checked_candidate_refuses_cross_application_race_without_installing_any_delta() {
        let fixture = fixture(29, "divination").await;
        let mut product = fixture.attach("turnstone").await;
        let mut other = fixture.attach("knot-editor").await;
        let original = graph(&mut product);
        let envelope = disclosed_graph(&mut product, &original);
        assert_eq!(envelope.epoch, original.scene.epoch);
        assert_eq!(envelope.revision, original.scene.revision);
        assert_eq!(envelope.into_graph().unwrap().node_count(), 0);
        let other_snapshot = graph(&mut other);
        assert_eq!(
            other
                .invoke(apply(
                    &other_snapshot,
                    vec![
                        node(1),
                        pandect::CapturedDelta::ReplaySetNodeFacetById {
                            node_id: Uuid::from_u128(1).to_string(),
                            facet: "fixture.unrelated/v1".into(),
                            value_json: serde_json::json!({"retained":"exact"}).to_string(),
                        }
                    ]
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let current = graph(&mut product);
        let hydrated = disclosed_graph(&mut product, &current)
            .into_graph()
            .unwrap();
        assert_eq!(
            hydrated.facets().get(
                &Uuid::from_u128(1),
                &pandect::facet_store::FacetId::new("fixture.unrelated/v1")
            ),
            Some(&serde_json::json!({"retained":"exact"}))
        );
        let before = serde_json::to_value(disclosed_graph(&mut product, &current)).unwrap();
        let checked = on_item(
            &original,
            session_item::APPLY_EDITS_INTENT,
            serde_json::to_vec(&ApplyEditsV1::checked(
                original.scene.revision,
                vec![node(2)],
            ))
            .unwrap(),
        );
        assert!(matches!(
            product.invoke(checked).unwrap(),
            IntentResult::Stale { .. }
        ));
        let after = graph(&mut product);
        assert_eq!(
            serde_json::to_value(disclosed_graph(&mut product, &after)).unwrap(),
            before
        );
        assert_eq!(nodes(&after), 1);
        assert!(
            after
                .presentation
                .offers_for(InstanceId(1))
                .unwrap()
                .iter()
                .all(|offer| offer.semantics.actions.is_empty())
        );
        let mut forged = apply(&after, vec![node(3)]);
        forged.target = InstanceId(1);
        assert!(
            matches!(product.invoke(forged).unwrap(), IntentResult::Rejected { reason } if reason.contains("complete journaled session candidate"))
        );
        assert_eq!(nodes(&graph(&mut product)), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn common_archive_adapter_preserves_resident_authority_on_divination() {
        use mere_view::{ArchiveRequest, MereViewRequest};
        let fixture = fixture(31, "divination").await;
        fixture.grants.grant(
            AppId::new("graphshell"),
            ResidentEndpointRoute::new("identity", MERE_NOTICE_POLL).unwrap(),
        );
        let mut app = fixture.attach("graphshell").await;
        let initial = graph(&mut app);
        assert_eq!(
            app.invoke(apply(&initial, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let list = projection(&mut app, 0);
        let id = session_ids(&list)[0];
        let archive = projection(&mut app, 2);
        let save = mere_overmap::route::archive_intent(
            &MereViewRequest::Archive(ArchiveRequest::Save(id)),
            &list,
            &archive,
        )
        .unwrap();
        assert_eq!(app.invoke(save).unwrap(), IntentResult::Accepted);
        let archive = projection(&mut app, 2);
        let codicil = archive_ids(&archive)[0].clone();
        let open = mere_overmap::route::archive_intent(
            &MereViewRequest::Archive(ArchiveRequest::Open(codicil)),
            &list,
            &archive,
        )
        .unwrap();
        assert_eq!(app.invoke(open).unwrap(), IntentResult::Accepted);
        let thaw = graph(&mut app);
        assert_eq!(nodes(&thaw), 1);
        let material = disclosed_graph(&mut app, &thaw);
        assert!(material.read_only);
        assert_eq!(material.into_graph().unwrap().node_count(), 1);
        assert!(
            mere_overmap::route::archive_intent(
                &MereViewRequest::Archive(ArchiveRequest::Compose(vec![
                    "unknown".into(),
                    "unknown".into()
                ])),
                &list,
                &archive
            )
            .is_none()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn view_media_and_writes_are_scoped_to_the_admitted_application() {
        let fixture = fixture(32, "divination").await;
        let mut first = fixture.attach("turnstone").await;
        let mut second = fixture.attach("knot-editor").await;
        let state = pandect::ViewIntent {
            strategy: Some("phyllotaxis.default".into()),
            ..Default::default()
        };
        let original = graph(&mut first);
        assert_eq!(
            first
                .invoke(on_item(
                    &original,
                    session_item::SET_VIEW_INTENT,
                    serde_json::to_vec(&SetViewV1::new("reading", state.clone())).unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let saved = graph(&mut first);
        let first_card = first
            .resource(ResourceRequest {
                session: saved.session.clone(),
                resource: saved
                    .presentation
                    .offers_for(session_item_of(&saved))
                    .unwrap()[0]
                    .resource,
            })
            .unwrap();
        let first_card: PortableCardV1 = serde_json::from_slice(&first_card.bytes).unwrap();
        assert!(
            second
                .resource(ResourceRequest {
                    session: saved.session.clone(),
                    resource: first_card.media[0]
                })
                .unwrap_err()
                .contains("another admitted application")
        );
        let first_data = disclosed_graph(&mut first, &saved);
        assert_eq!(first_data.application, "turnstone");
        assert_eq!(first_data.views["reading"], state);
        let other = graph(&mut second);
        let second_data = disclosed_graph(&mut second, &other);
        assert_eq!(second_data.application, "knot-editor");
        assert!(second_data.views.is_empty());
        let forged = serde_json::json!({"schema":session_item::SET_VIEW_SCHEMA,"view":"reading","state":{},"application":"turnstone"});
        assert!(matches!(
            second
                .invoke(on_item(
                    &other,
                    session_item::SET_VIEW_INTENT,
                    serde_json::to_vec(&forged).unwrap()
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            second
                .invoke(on_item(
                    &other,
                    session_item::SET_VIEW_INTENT,
                    serde_json::to_vec(&SetViewV1::new("reading", pandect::ViewIntent::default()))
                        .unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let first_again = graph(&mut first);
        assert_eq!(
            disclosed_graph(&mut first, &first_again).views["reading"],
            state
        );
    }

    /// A device of the persona, holding a grant for one route as one
    /// application, dials the resident's dialable host.
    struct RemoteDialler {
        owner: personae::InMemoryProvider,
        device: personae::InMemoryProvider,
        network: notochord::NetworkId,
        /// The host ends of each dial, kept for the test's life: a transport
        /// must outlive the sessions it carries (`graphshell_host::carrier`).
        hosts: std::sync::Mutex<Vec<transport::memory::MemoryTransport>>,
    }

    const REMOTE_ROOT: [u8; 32] = [0x64; 32];

    fn remote_profile() -> notochord::ProfileRef {
        notochord::ProfileRef {
            id: "mere.base".into(),
            revision: 1,
        }
    }

    impl RemoteDialler {
        fn new() -> Self {
            Self {
                owner: personae::InMemoryProvider::from_seed([0x61; 32]),
                device: personae::InMemoryProvider::from_seed([0x62; 32]),
                network: notochord::NetworkId([0x63; 32]),
                hosts: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn host(&self) -> crate::resident_projection::ResidentDialHost {
            use personae::IdentityProvider;
            crate::resident_projection::ResidentDialHost::new(
                crate::resident_projection::dial_policy(
                    self.network,
                    vec![notochord::TrustedRoot {
                        authority: REMOTE_ROOT,
                        issuer: self.owner.master_public_key().to_bytes(),
                    }],
                    vec![remote_profile()],
                    None,
                ),
            )
        }

        /// The persona's device grant: `path_prefix` under `action`.
        fn grant(
            &self,
            path_prefix: &str,
            action: &str,
        ) -> insigne::delegation::SignedDelegationCertificate {
            use insigne::delegation::{
                CapabilityScope, DelegationCertificate, DelegationParent,
                SignedDelegationCertificate,
            };
            use personae::IdentityProvider;
            use personae::delegation::Issue;
            SignedDelegationCertificate::issue(
                &self.owner,
                DelegationCertificate::new(
                    DelegationParent::Root(REMOTE_ROOT),
                    self.owner.master_public_key().to_bytes(),
                    self.device.master_public_key().to_bytes(),
                    CapabilityScope {
                        domain: graphshell_host::admission::GRAPHSHELL_DOMAIN.into(),
                        resource: self.network.0.to_vec(),
                        path_prefix: path_prefix.into(),
                        actions: [action.to_string()].into_iter().collect(),
                    },
                    0,
                    0,
                    None,
                    0,
                    [0x65; 32],
                ),
            )
            .expect("issue the device grant")
        }

        /// Dial `host` over a fresh memory pair, asking for `request` under
        /// `grant`, and serve the host's side of the one session. `Ok` is the
        /// admitted stream; `Err` is a handshake refusal.
        async fn dial(
            &self,
            host: &mut crate::resident_projection::ResidentDialHost,
            request: &mere_overmap::wire::RemoteMereRequest,
            grant: insigne::delegation::SignedDelegationCertificate,
        ) -> Result<tokio::io::DuplexStream, notochord::DenyReason> {
            use personae::IdentityProvider;
            use transport::PeerID;
            let device_peer =
                PeerID::from_bytes(&self.device.master_public_key().to_bytes()).unwrap();
            let host_peer = PeerID::from_bytes(
                &personae::InMemoryProvider::from_seed([0x66; 32])
                    .master_public_key()
                    .to_bytes(),
            )
            .unwrap();
            let (server, client) = transport::memory::MemoryTransport::pair(host_peer, device_peer);
            let hello = graphshell_host::admission::open_session_for(
                &self.device,
                self.network,
                remote_profile(),
                notochord::RequestedAction {
                    domain: graphshell_host::admission::GRAPHSHELL_DOMAIN.into(),
                    path: request.path(),
                    action: request.action().into(),
                },
                notochord::TrafficClass::Interactive,
                [0x67; 32],
                &graphshell_host::network_carrier::projection_binding(device_peer),
                vec![grant],
            )
            .unwrap();
            let limits = notochord::HandshakeLimits::default();
            let (accepted, dialled) = tokio::join!(
                host.accept_one(&server, || 1_000),
                graphshell_host::network_carrier::dial_projection_session(
                    &client, host_peer, &hello, &limits,
                )
            );
            // The host's side: a refusal or a served session; either way
            // the transport stays up while the dialler reads.
            let _ = accepted;
            self.hosts.lock().unwrap().push(server);
            dialled.expect("the dial reaches a decision")
        }
    }

    /// T4, T5 and T7 together: a same-persona device whose grant names
    /// `turnstone` for `mere/divination` reads the mere's sessions and graph
    /// through the dialable host, as `turnstone`, with the shared route panel
    /// over the native line pump.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_device_reads_a_mere_through_the_dialable_host_as_its_granted_application() {
        use graphshell_client::pump::LinePump;
        use mere_overmap::route_session::MereRouteSession;
        use mere_overmap::wire::RemoteMereRequest;
        let dialler = RemoteDialler::new();
        let mut host = dialler.host();
        let fixture = fixture_serving(41, "divination", None, Some(host.catalog())).await;
        let mut writer = fixture.attach("turnstone").await;
        let start = graph(&mut writer);
        assert_eq!(
            writer.invoke(apply(&start, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );

        let request = RemoteMereRequest::explicit("divination", "turnstone");
        let grant = dialler.grant(&request.path(), request.action());
        let stream = dialler
            .dial(&mut host, &request, grant)
            .await
            .expect("the device is admitted");
        let profile =
            chirograph::CapabilityProfile::new([chirograph::PresentationCapability::PortableCard]);
        let mut pump = LinePump::new(stream);
        let opened = pump.open(profile.clone()).await.expect("the route opens");
        let mut panel =
            MereRouteSession::from_descriptor(opened.descriptor, profile, "turnstone").unwrap();
        pump.drive(&mut panel, |panel| !panel.in_flight())
            .await
            .expect("the panel reads the route");
        let model = panel.model();
        assert!(!model.sessions.is_empty(), "the mere's sessions are read");
        assert!(
            model
                .graph
                .nodes
                .iter()
                .any(|node| node.key == Uuid::from_u128(1).to_string()),
            "the attached session's graph is read: {:?}",
            model.graph.nodes
        );
        assert_eq!(panel.view_name(), "turnstone.mere");
    }

    /// What the dialable host refuses: a grant naming another application,
    /// an explicit grant used for ambient, and ambient for an application the
    /// owner has not opted in; and that ambient, once opted in, reads the
    /// graph alone.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_dialable_host_serves_only_what_the_grant_and_the_owner_name() {
        use graphshell_client::pump::LinePump;
        use mere_overmap::wire::RemoteMereRequest;
        let dialler = RemoteDialler::new();
        let mut host = dialler.host();
        let fixture = fixture_serving(43, "divination", None, Some(host.catalog())).await;
        // A live session, so the ambient route has material to read.
        let mut writer = fixture.attach("turnstone").await;
        let start = graph(&mut writer);
        assert_eq!(
            writer.invoke(apply(&start, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let profile =
            chirograph::CapabilityProfile::new([chirograph::PresentationCapability::PortableCard]);

        // The grant names knot-editor; the hello asks as turnstone. The chain
        // covers the path no more than it covers the application: refused at
        // the handshake.
        let asked = RemoteMereRequest::explicit("divination", "turnstone");
        let named = RemoteMereRequest::explicit("divination", "knot-editor");
        let grant = dialler.grant(&named.path(), named.action());
        assert!(
            dialler.dial(&mut host, &asked, grant).await.is_err(),
            "a grant for another application does not admit this one"
        );

        // A broad grant (every mere) admits nothing here: the leaf must name
        // the application.
        let grant = dialler.grant(mere_overmap::wire::REMOTE_MERE_SERVICE, "connect");
        let stream = dialler
            .dial(&mut host, &asked, grant)
            .await
            .expect("admitted");
        assert!(
            LinePump::new(stream).open(profile.clone()).await.is_err(),
            "a grant that names no application serves no route"
        );

        // An explicit grant does not reach the ambient route.
        let ambient = RemoteMereRequest::ambient("divination", "turnstone");
        let grant = dialler.grant(&asked.path(), asked.action());
        assert!(
            dialler.dial(&mut host, &ambient, grant).await.is_err(),
            "an explicit grant does not confer ambient"
        );

        // An ambient grant for an application the owner has not opted in:
        // admitted, but the route will not open for it.
        let grant = dialler.grant(&ambient.path(), ambient.action());
        let stream = dialler
            .dial(&mut host, &ambient, grant)
            .await
            .expect("admitted");
        assert!(
            LinePump::new(stream).open(profile.clone()).await.is_err(),
            "ambient needs the owner's opt-in"
        );

        // Opted in, the ambient route offers the graph alone.
        let mere = fixture
            .reservoir
            .meres()
            .await
            .into_iter()
            .find(|mere| mere.domain.as_str() == "divination")
            .unwrap();
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("turnstone"),
                false,
                true,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        let grant = dialler.grant(&ambient.path(), ambient.action());
        let stream = dialler
            .dial(&mut host, &ambient, grant)
            .await
            .expect("admitted");
        let opened = LinePump::new(stream)
            .open(profile)
            .await
            .expect("the ambient route opens once opted in");
        let sessions: Vec<_> = opened
            .descriptor
            .projections
            .iter()
            .map(|offer| offer.request.session.0.clone())
            .collect();
        assert_eq!(sessions, vec![mere_overmap::wire::MERE_GRAPH.to_string()]);
    }

    /// Djinn's application door over `fixture`'s catalog, admitting Knot's
    /// overlay as `knot-editor`, with a keeper over a fixed fixture profile.
    /// Returns the door's address, its task scope and the server task.
    async fn knot_door(
        fixture: &Fixture,
    ) -> (
        String,
        armillary::tasks::ResidentTasks,
        tokio::task::JoinHandle<Result<(), crate::door::app_broker::AppBrokerError>>,
    ) {
        use crate::door::app_broker::serve_app_broker;
        use armillary::tasks::ResidentTasks;
        use castellan::{
            authority::PersonaeHost,
            custody::{IdentityVault, InMemoryStorage, Profile},
        };
        use personae::{Ed25519Keypair, ProfileId};
        let host = Arc::new(PersonaeHost::new(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("knot-overlay".into()),
                    "Knot overlay fixture",
                    Ed25519Keypair::from_seed([0x4b; 32]),
                ),
            ),
            None,
            graphshell_endpoint::door::identity::VaultProtectionView::Ephemeral,
        ));
        #[cfg(not(windows))]
        let address = format!("/tmp/knov-{}.sock", Uuid::new_v4());
        #[cfg(windows)]
        let address = format!(r"\\.\pipe\knov-{}", Uuid::new_v4());
        let tasks = ResidentTasks::new();
        let (scope, door, grants, socket) = (
            tasks.clone(),
            fixture.catalog.clone(),
            fixture.grants.clone(),
            address.clone(),
        );
        let server = tokio::spawn(async move {
            scope
                .scope(serve_app_broker(
                    &socket,
                    Arc::new(crate::keeper::Keeper::new(host)),
                    grants,
                    60_000,
                    None,
                    door,
                ))
                .await
        });
        for _ in 0..100 {
            if graphshell_endpoint::door::local_endpoint::connect_local(&address)
                .await
                .is_ok()
            {
                break;
            }
            assert!(!server.is_finished());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        (address, tasks, server)
    }

    /// Knot's overmap layer (knot-editor `mere_overlay`) saves the attached
    /// session to the archive and opens it again, through the first-party
    /// door as `knot-editor`, and reads the codicil back from the route.
    #[tokio::test(flavor = "multi_thread")]
    async fn knot_overlay_saves_and_opens_an_archive_through_the_door() {
        use knot_editor::mere_overlay::MereOverlay;
        use mere_view::{ArchiveRequest, MereViewRequest};
        let fixture = fixture(35, "divination").await;
        let mut writer = fixture.attach("turnstone").await;
        let initial = graph(&mut writer);
        assert_eq!(
            writer.invoke(apply(&initial, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let (address, tasks, server) = knot_door(&fixture).await;

        let mut overlay = MereOverlay::open_at(&address, "divination").await.unwrap();
        let before = overlay.read().await.unwrap();
        let attached = before
            .sessions
            .iter()
            .find(|session| session.attached)
            .expect("the route marks the attached session")
            .id;
        let archive = before.archive.expect("the route advertises its archive");
        assert!(archive.entries.is_empty());
        assert!(archive.save_sessions.contains(&attached));

        let saved = overlay
            .take(&MereViewRequest::Archive(ArchiveRequest::Save(attached)))
            .await
            .unwrap();
        assert_eq!(saved, None, "the route accepts Knot's save");
        let after = overlay.read().await.unwrap();
        let entries = after.archive.expect("the archive stays advertised").entries;
        assert_eq!(entries.len(), 1, "the saved codicil is read back");
        assert!(entries[0].can_open);
        assert_eq!(
            entries[0].session,
            Some(attached),
            "the codicil names the session it was saved from"
        );

        let opened = overlay
            .take(&MereViewRequest::Archive(ArchiveRequest::Open(
                entries[0].key.clone(),
            )))
            .await
            .unwrap();
        assert_eq!(opened, None, "the route accepts Knot's open");
        let names: Vec<String> = overlay
            .read()
            .await
            .unwrap()
            .sessions
            .into_iter()
            .map(|session| session.name)
            .collect();
        let source = before
            .sessions
            .iter()
            .find(|session| session.id == attached)
            .unwrap()
            .name
            .clone();
        assert_eq!(source, "Session");
        let thaw = format!(
            "Session (from {})",
            pandect::graph_codicil::codicil_short_id(&entries[0].key)
        );
        assert!(
            names.contains(&thaw),
            "the opened session is named apart from its source: {names:?}"
        );
        assert_eq!(
            names.iter().filter(|name| **name == source).count(),
            1,
            "only the source keeps its name: {names:?}"
        );
        assert!(
            overlay
                .take(&MereViewRequest::Archive(ArchiveRequest::Compose(vec![
                    "unknown".into(),
                    "unknown".into(),
                ])))
                .await
                .unwrap()
                .is_some(),
            "an unadvertised compose is declined before it reaches the route"
        );
        assert!(
            tasks.close_and_join().await.is_clean(),
            "task shutdown failed"
        );
        server.abort();
    }

    /// Knot's overmap layer carries a merge to the route: it asks the branch
    /// to merge into its source, reads the pending merge back as its card,
    /// keeps a choice for the conflict, is told Apply waits for one, and
    /// applies, through the first-party door as `knot-editor`.
    #[tokio::test(flavor = "multi_thread")]
    async fn knot_overlay_merges_a_branch_through_the_door() {
        use knot_editor::mere_overlay::MereOverlay;
        use mere_view::{MereViewRequest, MergeOption, MergeRequest, SessionStep};
        let fixture = fixture(37, "readings").await;
        let mut turnstone = fixture.attach("turnstone").await;
        let mut knot = fixture.attach("knot-editor").await;
        let start = graph(&mut turnstone);
        assert_eq!(
            turnstone
                .invoke(apply(&start, vec![node(1), retitle(1, "One")]))
                .unwrap(),
            IntentResult::Accepted
        );
        let listed = projection(&mut turnstone, 0);
        let parent = session_ids(&listed)[0];
        let branch = sessions_step(&listed, BRANCH_SESSION_INTENT, SessionsActionV1::on(parent));
        assert_eq!(turnstone.invoke(branch).unwrap(), IntentResult::Accepted);
        let listed = projection(&mut knot, 0);
        let branch = session_ids(&listed)[1];
        assert_eq!(
            knot.invoke(sessions_step(
                &listed,
                ATTACH_SESSION_INTENT,
                SessionsActionV1::on(branch)
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        let in_branch = graph(&mut knot);
        assert_eq!(
            knot.invoke(apply(&in_branch, vec![retitle(1, "Branch")]))
                .unwrap(),
            IntentResult::Accepted
        );
        let in_parent = graph(&mut turnstone);
        assert_eq!(
            turnstone
                .invoke(apply(&in_parent, vec![retitle(1, "Parent")]))
                .unwrap(),
            IntentResult::Accepted
        );

        let (address, tasks, server) = knot_door(&fixture).await;
        let mut overlay = MereOverlay::open_at(&address, "readings").await.unwrap();
        let before = overlay.read().await.unwrap();
        assert_eq!(before.merge, None, "nothing is merging yet");
        assert_eq!(
            overlay
                .take(&MereViewRequest::Session(branch, SessionStep::Merge))
                .await
                .unwrap(),
            None,
            "the route takes Knot's merge into source"
        );
        let pending = overlay
            .read()
            .await
            .unwrap()
            .merge
            .expect("a pending merge");
        assert_eq!(pending.source, branch);
        assert_eq!(pending.conflicts.len(), 1, "the title conflicts");
        let key = pending.conflicts[0].key.clone();
        assert_eq!(pending.conflicts[0].chosen, None);

        assert!(
            overlay
                .take(&MereViewRequest::Merge(MergeRequest::Apply))
                .await
                .unwrap()
                .is_some(),
            "Apply waits for a choice on every conflict"
        );
        assert!(
            overlay
                .take(&MereViewRequest::Merge(MergeRequest::Choose(
                    "unknown".into(),
                    MergeOption::Source
                )))
                .await
                .unwrap()
                .is_some(),
            "a choice for a conflict that is not pending is declined"
        );
        assert_eq!(
            overlay
                .take(&MereViewRequest::Merge(MergeRequest::Choose(
                    key.clone(),
                    MergeOption::Source
                )))
                .await
                .unwrap(),
            None
        );
        let chosen = overlay.read().await.unwrap().merge.unwrap();
        assert_eq!(chosen.conflicts[0].chosen, Some(MergeOption::Source));
        assert!(chosen.resolved());

        assert_eq!(
            overlay
                .take(&MereViewRequest::Merge(MergeRequest::Apply))
                .await
                .unwrap(),
            None,
            "the route applies the merge"
        );
        assert_eq!(overlay.read().await.unwrap().merge, None);
        let read = read_route(&mut turnstone);
        assert_eq!(
            label_of(&read.model, 1).as_deref(),
            Some("Branch"),
            "the branch's title wins, as chosen"
        );
        assert!(
            tasks.close_and_join().await.is_clean(),
            "task shutdown failed"
        );
        server.abort();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resident_proxy_forwards_one_graph_and_resume_then_refuses_revoked_grants() {
        use crate::door::app_broker::serve_app_broker;
        use armillary::tasks::ResidentTasks;
        use castellan::{
            authority::PersonaeHost,
            custody::{IdentityVault, InMemoryStorage, Profile},
        };
        use graphshell_endpoint::door::resident_mere_proxy::ResidentMereProxy;
        use personae::{Ed25519Keypair, ProfileId};
        let fixture = fixture(33, "divination").await;
        fixture
            .grants
            .grant(AppId::new("graphshell"), ResidentReservoir::route());
        let host = Arc::new(PersonaeHost::new(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("c1-proxy".into()),
                    "Public C1 proxy fixture",
                    Ed25519Keypair::from_seed([0xc1; 32]),
                ),
            ),
            None,
            graphshell_endpoint::door::identity::VaultProtectionView::Ephemeral,
        ));
        #[cfg(not(windows))]
        let address = format!("/tmp/c1px-{}.sock", Uuid::new_v4());
        #[cfg(windows)]
        let address = format!(r"\\.\pipe\c1px-{}", Uuid::new_v4());
        let tasks = ResidentTasks::new();
        let (scope, door, grants, socket) = (
            tasks.clone(),
            fixture.catalog.clone(),
            fixture.grants.clone(),
            address.clone(),
        );
        let server = tokio::spawn(async move {
            scope
                .scope(serve_app_broker(
                    &socket,
                    Arc::new(crate::keeper::Keeper::new(host)),
                    grants,
                    60_000,
                    None,
                    door,
                ))
                .await
        });
        for _ in 0..100 {
            if graphshell_endpoint::door::local_endpoint::connect_local(&address)
                .await
                .is_ok()
            {
                break;
            }
            assert!(!server.is_finished());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let mut proxy = ResidentMereProxy::divination(&address).unwrap();
        let request = proxy
            .describe()
            .projections
            .into_iter()
            .find(|offer| offer.request.session.0 == MERE_GRAPH)
            .unwrap()
            .request;
        let initial = proxy.snapshot(request.clone()).unwrap();
        let resumed = ResumeRequest {
            session: initial.session.clone(),
            epoch: initial.scene.epoch,
            revision: initial.scene.revision,
        };
        assert!(matches!(
            proxy.resume(resumed.clone()).unwrap(),
            ResumeReply::Current(_)
        ));
        let material = disclosed_graph(&mut proxy, &initial);
        assert_eq!(material.application, "graphshell");
        let card_offer = &initial.presentation.offers_for(InstanceId(0)).unwrap()[0];
        let whole = proxy
            .resource(ResourceRequest {
                session: initial.session.clone(),
                resource: card_offer.resource,
            })
            .unwrap();
        assert!(whole.has_valid_address());
        let chunk = proxy
            .resource_chunk(chirograph::ResourceChunkRequest {
                session: initial.session.clone(),
                resource: card_offer.resource,
                offset: 0,
                length: 16,
            })
            .unwrap();
        // The wire payload is padded base64 (24 characters for 16 raw
        // bytes). decode() verifies the independent chunk hash as well.
        let decoded = chunk
            .decode()
            .expect("chunk must decode and match its own address");
        assert_eq!(chunk.session, initial.session);
        assert_eq!(chunk.resource, card_offer.resource);
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.total_len, whole.bytes.len() as u64);
        assert_eq!(decoded.as_slice(), &whole.bytes[..16]);
        assert_eq!(
            proxy.invoke(apply(&initial, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let mut other = fixture.attach("turnstone").await;
        let current = graph(&mut other);
        assert_eq!(nodes(&current), 1);
        assert_eq!(
            other.invoke(apply(&current, vec![node(2)])).unwrap(),
            IntentResult::Accepted
        );
        let changed = graph(&mut other);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let observed = loop {
            if let Some(notice) = proxy.poll_notice().unwrap() {
                if notice.session == changed.session
                    && notice.epoch == changed.scene.epoch
                    && notice.revision == changed.scene.revision
                {
                    break notice;
                }
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "idle proxy must receive the actual acknowledged external-edit notice"
            );
            tokio::time::sleep(MERE_NOTICE_POLL).await;
        };
        assert_eq!(observed.revision, changed.scene.revision);
        let ResumeReply::Snapshot(fresh) = proxy.resume(resumed).unwrap() else {
            panic!("changed graph must resume with a fresh snapshot");
        };
        assert_eq!(nodes(&fresh), 2);
        let exact = ResumeRequest {
            session: fresh.session.clone(),
            epoch: fresh.scene.epoch,
            revision: fresh.scene.revision,
        };
        assert!(matches!(
            proxy.resume(exact.clone()).unwrap(),
            ResumeReply::Current(_)
        ));
        let mere = fixture.reservoir.meres().await.remove(0);
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("graphshell"),
                true,
                false,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        // Revocation must also terminate idle intake. Do not send a
        // snapshot/resume command merely to make the broker read its stream.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if proxy.poll_notice().is_err() {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "idle revoked proxy must be refused without a user command"
            );
            tokio::time::sleep(MERE_NOTICE_POLL).await;
        }
        assert!(proxy.resume(exact).is_err());
        assert!(proxy.snapshot(request).is_err());
        drop(proxy);
        server.abort();
        let _ = server.await;
        assert!(
            tasks.close_and_join().await.is_clean(),
            "task shutdown failed"
        );
    }

    fn drive_common_panel(
        panel: &mut mere_overmap::route_session::MereRouteSession,
        endpoint: &mut ResidentEndpointSession,
        wrong_role: bool,
    ) {
        for _ in 0..256 {
            let outgoing = panel.take_outgoing();
            if outgoing.is_empty() {
                assert!(!panel.in_flight());
                return;
            }
            for line in outgoing {
                let request: chirograph::CarrierRequest = serde_json::from_str(&line).unwrap();
                let mut response = graphshell_endpoint::dispatch_common(
                    endpoint,
                    request,
                    &mut |endpoint, request| endpoint.resume(request),
                )
                .unwrap();
                if wrong_role {
                    if let Ok(chirograph::CarrierResponseBody::Snapshot(snapshot)) =
                        &mut response.body
                    {
                        if snapshot.session.0 == MERE_GRAPH {
                            let key = snapshot
                                .presentation
                                .bindings
                                .iter()
                                .find(|binding| binding.instance == InstanceId(0))
                                .unwrap()
                                .key
                                .clone();
                            let offer = &mut snapshot.presentation.offers.get_mut(&key).unwrap()[0];
                            let card = endpoint
                                .resource(ResourceRequest {
                                    session: snapshot.session.clone(),
                                    resource: offer.resource,
                                })
                                .unwrap();
                            let card: PortableCardV1 = serde_json::from_slice(&card.bytes).unwrap();
                            let bytes = endpoint
                                .resource(ResourceRequest {
                                    session: snapshot.session.clone(),
                                    resource: card.media[0],
                                })
                                .unwrap();
                            offer.resource = bytes.resource;
                            offer.byte_size = bytes.bytes.len() as u64;
                            // The hash is genuine, but a complete graph envelope
                            // cannot stand in for a portable-card representation.
                        }
                    }
                }
                panel.on_line(&serde_json::to_string(&response).unwrap());
            }
        }
        panic!("common panel failed to settle its admitted carrier requests");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn common_panel_controller_runs_lifecycle_then_purges_on_denial_and_rejoins() {
        use mere_overmap::route_session::MereRouteSession;
        let fixture = fixture(34, "divination").await;
        fixture
            .grants
            .grant(AppId::new("graphshell"), ResidentReservoir::route());
        let mut endpoint = fixture.attach("graphshell").await;
        let mut panel = MereRouteSession::from_descriptor(
            endpoint.describe(),
            chirograph::CapabilityProfile::new([PresentationCapability::PortableCard]),
            "graphshell",
        )
        .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert_eq!(panel.model().sessions.len(), 1);
        assert!(
            panel
                .primary()
                .unwrap()
                .scene
                .active_items_in_order()
                .is_empty(),
            "synthetic session item never becomes a domain graph node"
        );
        panel.request(MereViewRequest::Mint).unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert_eq!(panel.model().sessions.len(), 2);
        let before_fork: BTreeSet<_> = panel
            .model()
            .sessions
            .iter()
            .map(|session| *session.id.as_uuid())
            .collect();
        let original = panel
            .model()
            .sessions
            .iter()
            .find(|session| session.attached)
            .unwrap()
            .id;
        panel
            .request(MereViewRequest::Session(original, SessionStep::Fork))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert_eq!(panel.model().sessions.len(), 3);
        let fork = panel
            .model()
            .sessions
            .iter()
            .find(|session| !before_fork.contains(session.id.as_uuid()))
            .unwrap()
            .id;
        panel
            .request(MereViewRequest::Session(fork, SessionStep::Switch))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert!(
            panel
                .model()
                .sessions
                .iter()
                .any(|session| session.id == fork && session.attached)
        );
        panel
            .request(MereViewRequest::Session(original, SessionStep::Trash))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert!(
            panel
                .model()
                .sessions
                .iter()
                .any(|session| session.id == original && session.trashed)
        );
        panel
            .request(MereViewRequest::Session(original, SessionStep::Restore))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert!(
            panel
                .model()
                .sessions
                .iter()
                .any(|session| session.id == original && !session.trashed)
        );
        let mere = fixture.reservoir.meres().await.remove(0);
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("graphshell"),
                true,
                false,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        panel
            .request(MereViewRequest::Host("refresh".into()))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert_eq!(panel.status(), "unavailable");
        assert!(panel.model().sessions.is_empty());
        assert!(panel.primary().is_none());
        assert!(panel.request(MereViewRequest::Mint).is_err());
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("graphshell"),
                false,
                false,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        let mut endpoint = fixture.attach("graphshell").await;
        let mut rejoined = MereRouteSession::from_descriptor(
            endpoint.describe(),
            chirograph::CapabilityProfile::new([PresentationCapability::PortableCard]),
            "graphshell",
        )
        .unwrap();
        drive_common_panel(&mut rejoined, &mut endpoint, false);
        assert_eq!(rejoined.model().sessions.len(), 3);
        assert_eq!(rejoined.status(), "open");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn common_panel_refuses_a_hash_valid_resource_in_the_wrong_presentation_role() {
        let fixture = fixture(35, "divination").await;
        fixture
            .grants
            .grant(AppId::new("graphshell"), ResidentReservoir::route());
        let mut endpoint = fixture.attach("graphshell").await;
        let mut panel = mere_overmap::route_session::MereRouteSession::from_descriptor(
            endpoint.describe(),
            chirograph::CapabilityProfile::new([PresentationCapability::PortableCard]),
            "graphshell",
        )
        .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, true);
        assert_eq!(panel.status(), "unavailable");
        assert!(panel.primary().is_none());
        assert!(panel.model().sessions.is_empty());
    }

    /// Read only the fixture's durable session tree. Comparing all bytes also
    /// catches implicit forks, view appends and manifest rewrites.
    fn session_tree_bytes(path: &std::path::Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
        fn visit(
            root: &std::path::Path,
            path: &std::path::Path,
            files: &mut BTreeMap<std::path::PathBuf, Vec<u8>>,
        ) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    visit(root, &path, files);
                } else {
                    files.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        std::fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        visit(path, path, &mut files);
        files
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn common_panel_saves_own_view_and_reopens_archive_without_implicit_writes() {
        use mere_overmap::route_session::MereRouteSession;
        use mere_view::ArchiveRequest;
        let fixture = fixture(36, "divination").await;
        fixture
            .grants
            .grant(AppId::new("graphshell"), ResidentReservoir::route());
        let mut endpoint = fixture.attach("graphshell").await;
        let initial = graph(&mut endpoint);
        assert_eq!(
            endpoint.invoke(apply(&initial, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let profile = || chirograph::CapabilityProfile::new([PresentationCapability::PortableCard]);
        let mut panel =
            MereRouteSession::from_descriptor(endpoint.describe(), profile(), "graphshell")
                .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        let key = Uuid::from_u128(1).to_string();
        assert!(panel.records().contains_key(&key));
        panel
            .request(MereViewRequest::Activate(key.clone()))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        panel
            .request(MereViewRequest::Layout("grid.default".into()))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert!(panel.view_saved());
        assert_eq!(panel.saved_view().focus.as_deref(), Some("https://1.test/"));
        assert_eq!(panel.saved_view().strategy.as_deref(), Some("grid.default"));
        let saved = panel.saved_view().clone();
        let current = graph(&mut endpoint);
        assert_eq!(
            disclosed_graph(&mut endpoint, &current).views["graphshell.mere"],
            saved
        );

        // A new admitted reader reconstructs the acknowledged view from the
        // owner's material rather than this controller's transient fields.
        drop(panel);
        drop(endpoint);
        let mut endpoint = fixture.attach("graphshell").await;
        let mut panel =
            MereRouteSession::from_descriptor(endpoint.describe(), profile(), "graphshell")
                .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert_eq!(panel.saved_view(), &saved);
        assert_eq!(panel.focus_key().as_deref(), Some(key.as_str()));
        assert_eq!(panel.model().layout, "grid.default");
        assert!(panel.view_saved());
        let mut other = fixture.attach("knot-editor").await;
        let other_graph = graph(&mut other);
        assert!(
            !disclosed_graph(&mut other, &other_graph)
                .views
                .contains_key("graphshell.mere")
        );
        assert_eq!(
            other
                .invoke(on_item(
                    &other_graph,
                    session_item::SET_VIEW_INTENT,
                    serde_json::to_vec(&SetViewV1::new(
                        "graphshell.mere",
                        pandect::ViewIntent::default()
                    ))
                    .unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let current = graph(&mut endpoint);
        assert_eq!(
            disclosed_graph(&mut endpoint, &current).views["graphshell.mere"],
            saved
        );
        panel
            .request(MereViewRequest::Host("refresh".into()))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        let live = panel
            .model()
            .sessions
            .iter()
            .find(|session| session.attached)
            .unwrap()
            .id;
        panel
            .request(MereViewRequest::Archive(ArchiveRequest::Save(live)))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        let archive = panel
            .model()
            .archive
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .find(|entry| entry.can_open)
            .unwrap()
            .key
            .clone();
        panel
            .request(MereViewRequest::Archive(ArchiveRequest::Open(archive)))
            .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert!(panel.read_only());
        assert!(panel.records().contains_key(&key));
        let thaw = panel
            .model()
            .sessions
            .iter()
            .find(|session| session.attached)
            .unwrap()
            .id;
        let before_ids: Vec<_> = panel
            .model()
            .sessions
            .iter()
            .map(|session| session.id)
            .collect();
        let before_graph = graph(&mut endpoint);
        let before_material =
            serde_json::to_value(disclosed_graph(&mut endpoint, &before_graph)).unwrap();
        let mere = fixture.reservoir.meres().await.remove(0);
        let sessions = mere_dir(fixture._root.path(), persona(36), mere.id).join("sessions");
        let before_bytes = session_tree_bytes(&sessions);
        panel
            .request(MereViewRequest::Activate(key.clone()))
            .unwrap();
        assert!(
            panel.take_outgoing().is_empty(),
            "archive activation emits no resident request"
        );
        panel
            .request(MereViewRequest::Layout("phyllotaxis.default".into()))
            .unwrap();
        assert!(
            panel.take_outgoing().is_empty(),
            "archive layout emits no resident request"
        );
        assert_eq!(panel.focus_key().as_deref(), Some(key.as_str()));
        assert_eq!(panel.model().layout, "phyllotaxis.default");
        assert_eq!(
            panel
                .model()
                .sessions
                .iter()
                .map(|session| session.id)
                .collect::<Vec<_>>(),
            before_ids
        );
        assert_eq!(session_tree_bytes(&sessions), before_bytes);
        let after_graph = graph(&mut endpoint);
        assert_eq!(
            serde_json::to_value(disclosed_graph(&mut endpoint, &after_graph)).unwrap(),
            before_material
        );
        drop(panel);
        drop(endpoint);
        let mut endpoint = fixture.attach("graphshell").await;
        let mut reopened =
            MereRouteSession::from_descriptor(endpoint.describe(), profile(), "graphshell")
                .unwrap();
        drive_common_panel(&mut reopened, &mut endpoint, false);
        // A fresh route chooses latest_live, which can already be this
        // thaw. Switch only if its actual card advertises attachment.
        let selected = reopened
            .model()
            .sessions
            .iter()
            .find(|session| session.id == thaw)
            .unwrap();
        if !selected.attached {
            assert!(selected.steps.contains(&SessionStep::Switch));
            reopened
                .request(MereViewRequest::Session(thaw, SessionStep::Switch))
                .unwrap();
            drive_common_panel(&mut reopened, &mut endpoint, false);
        } else {
            assert!(!selected.steps.contains(&SessionStep::Switch));
        }
        assert!(
            reopened
                .model()
                .sessions
                .iter()
                .any(|session| session.id == thaw && session.attached)
        );
        assert!(reopened.read_only());
        reopened.request(MereViewRequest::Activate(key)).unwrap();
        assert!(
            reopened.take_outgoing().is_empty(),
            "reopened readonly selection emits no request"
        );
        assert_eq!(
            reopened
                .model()
                .sessions
                .iter()
                .map(|session| session.id)
                .collect::<Vec<_>>(),
            before_ids
        );
        assert_eq!(session_tree_bytes(&sessions), before_bytes);
    }

    #[derive(Clone, Copy, Debug)]
    enum MediaFault {
        Schema,
        Application,
        AttachedSession,
        Epoch,
        Revision,
        ProjectionNamespace,
        RequestedHash,
        Address,
    }

    /// Corrupt only the in-flight transport, retaining the actual admitted
    /// fixture as authority. Envelope faults rebind both content addresses so
    /// they test role/binding checks after successful hash verification.
    fn drive_common_panel_media_fault(
        panel: &mut mere_overmap::route_session::MereRouteSession,
        endpoint: &mut ResidentEndpointSession,
        fault: MediaFault,
    ) {
        let mut replaced: HashMap<ContentHash, ResourceResponse> = HashMap::new();
        let mut media_hash = None;
        let mut applied = false;
        for _ in 0..256 {
            let outgoing = panel.take_outgoing();
            if outgoing.is_empty() {
                assert!(!panel.in_flight());
                assert!(applied, "fault {fault:?} must reach the media boundary");
                return;
            }
            for line in outgoing {
                let request: chirograph::CarrierRequest = serde_json::from_str(&line).unwrap();
                let forged = match &request.body {
                    chirograph::CarrierRequestBody::Resource(resource) => {
                        replaced.get(&resource.resource).cloned()
                    },
                    _ => None,
                };
                let mut response = if let Some(mut resource) = forged {
                    if Some(resource.resource) == media_hash {
                        applied = true;
                        match fault {
                            MediaFault::ProjectionNamespace => {
                                resource.session = ProjectionSession(MERE_ARCHIVE.into())
                            },
                            MediaFault::RequestedHash => {
                                // A valid address belonging to a different resource.
                                resource = ResourceResponse::new(resource.session, b"{}".to_vec());
                            },
                            MediaFault::Address => resource.bytes.push(b' '),
                            _ => {},
                        }
                    }
                    chirograph::CarrierResponse {
                        id: request.id,
                        body: Ok(chirograph::CarrierResponseBody::Resource(resource)),
                    }
                } else {
                    graphshell_endpoint::dispatch_common(
                        endpoint,
                        request,
                        &mut |endpoint, request| endpoint.resume(request),
                    )
                    .unwrap()
                };
                if let Ok(chirograph::CarrierResponseBody::Snapshot(snapshot)) = &mut response.body
                {
                    if snapshot.session.0 == MERE_GRAPH {
                        let key = snapshot
                            .presentation
                            .bindings
                            .iter()
                            .find(|binding| binding.instance == session_item_of(snapshot))
                            .unwrap()
                            .key
                            .clone();
                        let offer = &mut snapshot.presentation.offers.get_mut(&key).unwrap()[0];
                        let original_card = endpoint
                            .resource(ResourceRequest {
                                session: snapshot.session.clone(),
                                resource: offer.resource,
                            })
                            .unwrap();
                        let mut card: PortableCardV1 =
                            serde_json::from_slice(&original_card.bytes).unwrap();
                        let original_media = endpoint
                            .resource(ResourceRequest {
                                session: snapshot.session.clone(),
                                resource: card.media[0],
                            })
                            .unwrap();
                        let mut material: session_item::SessionGraphV1 =
                            serde_json::from_slice(&original_media.bytes).unwrap();
                        match fault {
                            MediaFault::Schema => {
                                material.schema = "mere.session.graph/v999".into()
                            },
                            MediaFault::Application => material.application = "cleromancy".into(),
                            MediaFault::AttachedSession => {
                                material.session = SessionId::from_uuid(Uuid::from_u128(0xffff))
                            },
                            MediaFault::Epoch => material.epoch = SceneEpoch(material.epoch.0 + 1),
                            MediaFault::Revision => {
                                material.revision = Revision(material.revision.0 + 1)
                            },
                            _ => {},
                        }
                        let media = ResourceResponse::new(
                            snapshot.session.clone(),
                            serde_json::to_vec(&material).unwrap(),
                        );
                        media_hash = Some(media.resource);
                        card.media[0] = media.resource;
                        let card = ResourceResponse::new(
                            snapshot.session.clone(),
                            serde_json::to_vec(&card).unwrap(),
                        );
                        offer.resource = card.resource;
                        offer.byte_size = card.bytes.len() as u64;
                        replaced.insert(media.resource, media);
                        replaced.insert(card.resource, card);
                    }
                }
                panel.on_line(&serde_json::to_string(&response).unwrap());
            }
        }
        panic!("media fault {fault:?} did not settle");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn common_panel_refuses_media_with_wrong_namespace_application_session_or_hash() {
        use mere_overmap::route_session::MereRouteSession;
        let fixture = fixture(37, "divination").await;
        fixture
            .grants
            .grant(AppId::new("graphshell"), ResidentReservoir::route());
        for fault in [
            MediaFault::Schema,
            MediaFault::Application,
            MediaFault::AttachedSession,
            MediaFault::Epoch,
            MediaFault::Revision,
            MediaFault::ProjectionNamespace,
            MediaFault::RequestedHash,
            MediaFault::Address,
        ] {
            let mut endpoint = fixture.attach("graphshell").await;
            let mut panel = MereRouteSession::from_descriptor(
                endpoint.describe(),
                chirograph::CapabilityProfile::new([PresentationCapability::PortableCard]),
                "graphshell",
            )
            .unwrap();
            drive_common_panel_media_fault(&mut panel, &mut endpoint, fault);
            assert_eq!(panel.status(), "unavailable", "{fault:?}");
            assert!(panel.primary().is_none(), "{fault:?}");
            assert!(panel.model().sessions.is_empty(), "{fault:?}");
            assert!(panel.records().is_empty(), "{fault:?}");
            assert_eq!(
                panel.saved_view(),
                &pandect::ViewIntent::default(),
                "{fault:?}"
            );
            assert!(panel.request(MereViewRequest::Mint).is_err(), "{fault:?}");
        }
        // Refusals leave the owner intact; an explicitly constructed fresh
        // admitted session can join the same graph again.
        let mut endpoint = fixture.attach("graphshell").await;
        let mut panel = MereRouteSession::from_descriptor(
            endpoint.describe(),
            chirograph::CapabilityProfile::new([PresentationCapability::PortableCard]),
            "graphshell",
        )
        .unwrap();
        drive_common_panel(&mut panel, &mut endpoint, false);
        assert_eq!(panel.status(), "open");
        assert_eq!(panel.model().sessions.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn personal_facts_two_apps_share_exact_revisions_and_refuse_tampering() {
        let fixture = fixture(0x5046, "personal-facts").await;
        let mut first = fixture.attach("turnstone").await;
        let mut second = fixture.attach("knot-editor").await;
        let start = graph(&mut first);
        let mut candidate = disclosed_graph(&mut first, &start).into_graph().unwrap();
        let edits = Arc::new(std::sync::Mutex::new(Vec::new()));
        let held = edits.clone();
        candidate.set_recorder(Some(Arc::new(move |d| {
            held.lock().unwrap().push(d.clone())
        })));
        let scope = fixture.reservoir.meres().await[0]
            .persona
            .as_uuid()
            .to_string();
        let subject = personal_facts::create_contact(
            &mut candidate,
            &scope,
            gaz::LocalId::from_random([3; 16]),
            "Synthetic subject",
        )
        .unwrap();
        let facts: dramatis::birth::BirthFacts = serde_json::from_value(serde_json::json!({
            "schema":"dramatis.birth-facts/v1", "subject":subject,
            "date":{"year":2000,"month":1,"day":1}, "time":"Unknown",
            "place":null,"timezone":null,"source":{"label":"Public test","reference":null}
        }))
        .unwrap();
        let expected = personal_facts::save_birth(&mut candidate, &facts).unwrap();
        let steps = edits.lock().unwrap().clone();
        assert_eq!(
            first.invoke(apply(&start, steps)).unwrap(),
            IntentResult::Accepted
        );
        let shared = graph(&mut second);
        let data = disclosed_graph(&mut second, &shared).into_graph().unwrap();
        assert_eq!(
            personal_facts::births(&data).unwrap(),
            vec![expected.clone()]
        );
        let before = serde_json::to_value(disclosed_graph(&mut second, &shared)).unwrap();
        let id = kernel::graph::Graph::node_namespace_id(&expected.resource.id);
        let mut changed = facts;
        changed.date.day = 2;
        assert!(matches!(
            second
                .invoke(apply(
                    &shared,
                    vec![pandect::CapturedDelta::ReplaySetNodeFacetById {
                        node_id: id.to_string(),
                        facet: personal_facts::BIRTH_FACET.into(),
                        value_json: serde_json::to_string(&changed).unwrap()
                    }]
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        let after = graph(&mut second);
        assert_eq!(
            serde_json::to_value(disclosed_graph(&mut second, &after)).unwrap(),
            before
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn profile_subject_admission_uses_owner_vault_and_refuses_fabricated_ids() {
        let fixture = fixture(0x50524f, "personal-facts").await;
        let mut app = fixture.attach("turnstone").await;
        let mut other = fixture.attach("knot-editor").await;
        for (profile, accept) in [("fabricated-profile", false), ("fixture-profile", true)] {
            let before = graph(&mut app);
            let envelope = disclosed_graph(&mut app, &before);
            let mut candidate = envelope.clone().into_graph().unwrap();
            let edits = Arc::new(std::sync::Mutex::new(Vec::new()));
            let held = edits.clone();
            candidate.set_recorder(Some(Arc::new(move |d| {
                held.lock().unwrap().push(d.clone())
            })));
            let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":{"Profile":{"id":profile}},"date":{"year":2000,"month":1,"day":1},"time":"Unknown","place":null,"timezone":null,"source":{"label":"Public profile-admission fixture","reference":null}})).unwrap();
            personal_facts::save_birth(&mut candidate, &facts).unwrap();
            let outcome = app
                .invoke(apply(&before, edits.lock().unwrap().clone()))
                .unwrap();
            if accept {
                assert_eq!(outcome, IntentResult::Accepted);
            } else {
                assert!(matches!(outcome, IntentResult::Rejected { .. }));
                let after = graph(&mut app);
                assert_eq!(
                    serde_json::to_value(disclosed_graph(&mut app, &after)).unwrap(),
                    serde_json::to_value(envelope).unwrap()
                );
            }
        }
        let shared = graph(&mut other);
        assert_eq!(
            personal_facts::births(&disclosed_graph(&mut other, &shared).into_graph().unwrap())
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn divination_requires_product_authority_and_refuses_unknown_domain_facets() {
        let fixture = fixture(30, "divination").await;
        let record = fixture.reservoir.meres().await.remove(0);
        let missing = MereRoutes::new(
            fixture._root.path().to_path_buf(),
            record.persona,
            AppEndpointCatalog::default(),
            AppRouteGrants::default(),
            vec![AppId::new("cleromancy")],
        );
        missing.load_access(&fixture.reservoir).await.unwrap();
        assert!(
            missing
                .serve(&record)
                .await
                .unwrap_err()
                .contains("domain authority")
        );
        let mut app = fixture.attach("turnstone").await;
        let initial = graph(&mut app);
        let facet = pandect::CapturedDelta::ReplaySetNodeFacetById {
            node_id: Uuid::from_u128(1).to_string(),
            facet: "cleromancy.unknown/v999".into(),
            value_json: serde_json::json!({"tampered": true}).to_string(),
        };
        assert!(
            matches!(app.invoke(apply(&initial, vec![node(1), facet])).unwrap(), IntentResult::Rejected { reason } if reason.contains("unsupported domain facet"))
        );
        assert_eq!(nodes(&graph(&mut app)), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn domain_authority_checks_route_batches_archive_compose_and_first_edit_forks() {
        let validator: pandect::GraphValidator = Arc::new(|graph| {
            if graph.nodes().count() > 1 || graph.nodes().any(|(_, n)| n.title == "Forbidden") {
                Err("domain forbids this complete candidate".into())
            } else {
                Ok(())
            }
        });
        let fixture = fixture_with_validator(25, "bounded-domain", Some(validator)).await;
        let mut app = fixture.attach("turnstone").await;
        let start = graph(&mut app);
        assert!(matches!(
            app.invoke(apply(&start, vec![node(1), retitle(1, "Forbidden")]))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            nodes(&graph(&mut app)),
            0,
            "complete rejected batch installs nothing"
        );
        assert_eq!(
            app.invoke(apply(&start, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let list = projection(&mut app, 0);
        let original = session_ids(&list)[0];
        assert_eq!(
            app.invoke(archive_step(
                &list,
                SAVE_CODICIL_INTENT,
                ArchiveActionV1::save(original)
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(
            app.invoke(sessions_step(
                &list,
                MINT_SESSION_INTENT,
                SessionsActionV1::mint(None)
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        let list = projection(&mut app, 0);
        let second = *session_ids(&list)
            .iter()
            .find(|id| **id != original)
            .unwrap();
        assert_eq!(
            app.invoke(sessions_step(
                &list,
                ATTACH_SESSION_INTENT,
                SessionsActionV1::on(second)
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        let current_graph = graph(&mut app);
        assert_eq!(
            app.invoke(apply(&current_graph, vec![node(2)])).unwrap(),
            IntentResult::Accepted
        );
        let list = projection(&mut app, 0);
        assert_eq!(
            app.invoke(archive_step(
                &list,
                SAVE_CODICIL_INTENT,
                ArchiveActionV1::save(second)
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        let archive = projection(&mut app, 2);
        let ids = archive_ids(&archive);
        assert_eq!(ids.len(), 2);
        assert!(matches!(
            app.invoke(archive_step(
                &archive,
                COMPOSE_CODICILS_INTENT,
                ArchiveActionV1::on(ids.clone())
            ))
            .unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            archive_ids(&projection(&mut app, 2)),
            ids,
            "refused union writes no codicil"
        );
        assert_eq!(
            app.invoke(archive_step(
                &archive,
                OPEN_CODICIL_INTENT,
                ArchiveActionV1::on(vec![ids[0].clone()])
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        let before = projection(&mut app, 0);
        let thaw_graph = projection(&mut app, 1);
        assert!(matches!(
            app.invoke(apply(&thaw_graph, vec![node(3)])).unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            session_ids(&projection(&mut app, 0)),
            session_ids(&before),
            "refused candidate creates no durable fork"
        );
        assert_eq!(nodes(&projection(&mut app, 1)), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn archive_intents_save_browse_fork_and_compose_through_the_resident() {
        let fixture = fixture(24, "archive-test").await;
        let mut first = fixture.attach("turnstone").await;
        let start = graph(&mut first);
        assert_eq!(
            first.invoke(apply(&start, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );
        let list = projection(&mut first, 0);
        let original = session_ids(&list)[0];
        assert_eq!(
            first
                .invoke(archive_step(
                    &list,
                    SAVE_CODICIL_INTENT,
                    ArchiveActionV1::save(original)
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let archive = projection(&mut first, 2);
        let first_id = archive_ids(&archive)[0].clone();
        assert_eq!(
            first
                .invoke(archive_step(
                    &archive,
                    OPEN_CODICIL_INTENT,
                    ArchiveActionV1::on(vec![first_id.clone()])
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let list = projection(&mut first, 0);
        let thaw = *session_ids(&list)
            .iter()
            .find(|id| **id != original)
            .unwrap();
        let mut other = fixture.attach("knot-editor").await;
        let before = graph(&mut first);
        assert_eq!(nodes(&before), 1);
        let mut invalid = apply(&before, vec![node(2)]);
        invalid.payload = b"invalid".to_vec();
        assert!(matches!(
            first.invoke(invalid).unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            session_ids(&projection(&mut first, 0)).len(),
            2,
            "rejected first edit creates no fork"
        );
        assert_eq!(
            first.invoke(apply(&before, vec![node(2)])).unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(nodes(&graph(&mut first)), 2);
        let list = projection(&mut first, 0);
        assert_eq!(session_ids(&list).len(), 3);
        assert_eq!(
            nodes(&graph(&mut other)),
            1,
            "another connection still browses the unchanged thaw"
        );
        let fork = *session_ids(&list)
            .iter()
            .find(|id| **id != original && **id != thaw)
            .unwrap();
        assert_eq!(
            first
                .invoke(archive_step(
                    &list,
                    SAVE_CODICIL_INTENT,
                    ArchiveActionV1::save(fork)
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let archive = projection(&mut first, 2);
        let ids = archive_ids(&archive);
        assert_eq!(ids.len(), 2);
        assert_eq!(
            first
                .invoke(archive_step(
                    &archive,
                    COMPOSE_CODICILS_INTENT,
                    ArchiveActionV1::on(ids.clone())
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let archive = projection(&mut first, 2);
        let composed = archive_ids(&archive)
            .into_iter()
            .find(|id| !ids.contains(id))
            .unwrap();
        let mere = fixture.reservoir.meres().await.remove(0);
        let archive_path =
            mere_dir(fixture._root.path(), persona(24), mere.id).join("archive.redb");
        assert!(archive_path.is_file());
        assert!(
            muniment::RedbBackend::open(&archive_path).is_err(),
            "resident remains the only archive owner"
        );
        assert_eq!(
            first
                .invoke(archive_step(
                    &archive,
                    OPEN_CODICIL_INTENT,
                    ArchiveActionV1::on(vec![composed])
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(nodes(&graph(&mut first)), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_sessions_projection_runs_the_lifecycle() {
        let fixture = fixture(22, "readings").await;
        let mut turnstone = fixture.attach("turnstone").await;
        let mut knot = fixture.attach("knot-editor").await;
        let start = graph(&mut turnstone);
        assert_eq!(
            turnstone.invoke(apply(&start, vec![node(1)])).unwrap(),
            IntentResult::Accepted
        );

        // The mere's card and one session card, the one both attached to.
        let listed = projection(&mut turnstone, 0);
        assert_eq!(listed.presentation.bindings.len(), 2);
        let parent = session_ids(&listed)[0];

        // Turnstone forks it, and Knot sees the fork.
        let fork = sessions_step(&listed, FORK_SESSION_INTENT, SessionsActionV1::on(parent));
        assert_eq!(turnstone.invoke(fork).unwrap(), IntentResult::Accepted);
        let knot_list = projection(&mut knot, 0);
        let fork = session_ids(&knot_list)[1];

        // Turnstone mints another from its first list, older than the fork: a
        // step is taken at any revision of the list (§7 item 33). Knot is rung.
        let mint = sessions_step(&listed, MINT_SESSION_INTENT, SessionsActionV1::mint(None));
        assert_eq!(turnstone.invoke(mint).unwrap(), IntentResult::Accepted);
        let three = projection(&mut turnstone, 0);
        assert_eq!(session_ids(&three).len(), 3);
        assert_ne!(
            three.scene.revision, knot_list.scene.revision,
            "the list moved"
        );
        assert!(knot.poll_notice().unwrap().is_some(), "knot is rung");

        // Knot attaches to the fork from its list, which the mint has moved
        // past: it holds the parent's node, and an edit there leaves the
        // parent alone.
        let step = |intent: &str, id: SessionId| {
            sessions_step(&knot_list, intent, SessionsActionV1::on(id))
        };
        assert_eq!(
            knot.invoke(step(ATTACH_SESSION_INTENT, fork)).unwrap(),
            IntentResult::Accepted
        );
        let fork_graph = graph(&mut knot);
        assert_eq!(nodes(&fork_graph), 1, "the fork starts from the parent");
        assert_eq!(
            knot.invoke(apply(&fork_graph, vec![node(2)])).unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(nodes(&graph(&mut knot)), 2);
        assert_eq!(nodes(&graph(&mut turnstone)), 1, "the parent is unchanged");

        // Trash the fork and restore it. A step the session's card does not
        // offer now, or one naming no session of this mere, is refused.
        assert_eq!(
            knot.invoke(step(TRASH_SESSION_INTENT, fork)).unwrap(),
            IntentResult::Accepted
        );
        let unnamed = sessions_step(
            &knot_list,
            FORK_SESSION_INTENT,
            SessionsActionV1::mint(None),
        );
        for (refused, why) in [
            (step(TRASH_SESSION_INTENT, fork), "does not offer"),
            (step(RESTORE_SESSION_INTENT, parent), "does not offer"),
            (step(ATTACH_SESSION_INTENT, fork), "does not offer"),
            (
                step(ATTACH_SESSION_INTENT, SessionId::new()),
                "holds no session",
            ),
            (unnamed, "names no session"),
        ] {
            match knot.invoke(refused).unwrap() {
                IntentResult::Rejected { reason } => assert!(reason.contains(why), "{reason}"),
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(
            knot.invoke(step(RESTORE_SESSION_INTENT, fork)).unwrap(),
            IntentResult::Accepted
        );

        // Across an epoch a step is stale.
        let mut across = step(TRASH_SESSION_INTENT, fork);
        across.observed_epoch = SceneEpoch(0);
        assert!(matches!(
            knot.invoke(across).unwrap(),
            IntentResult::Stale { .. }
        ));
    }

    /// The pending merge the sessions list discloses, read through its
    /// card's media, with the card's instance.
    fn pending_merge(
        session: &mut ResidentEndpointSession,
        listed: &ProjectionSnapshot,
    ) -> Option<(InstanceId, PendingMergeV1)> {
        let instance = *mere_route::merge_cards(listed).first()?;
        let offer = listed.presentation.offers_for(instance)?.first()?.clone();
        let card: PortableCardV1 = serde_json::from_slice(
            &session
                .resource(ResourceRequest {
                    session: listed.session.clone(),
                    resource: offer.resource,
                })
                .unwrap()
                .bytes,
        )
        .unwrap();
        let material = session
            .resource(ResourceRequest {
                session: listed.session.clone(),
                resource: card.media[0],
            })
            .unwrap();
        Some((instance, serde_json::from_slice(&material.bytes).unwrap()))
    }

    fn label_of(model: &MereViewModel, n: u128) -> Option<String> {
        let key = Uuid::from_u128(n).to_string();
        model
            .graph
            .nodes
            .iter()
            .find(|node| node.key == key)
            .map(|node| node.label.clone())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_branch_merges_back_through_the_route_with_a_choice_per_conflict() {
        let fixture = fixture(23, "readings").await;
        let mut turnstone = fixture.attach("turnstone").await;
        let mut knot = fixture.attach("knot-editor").await;
        let start = graph(&mut turnstone);
        assert_eq!(
            turnstone
                .invoke(apply(&start, vec![node(1), retitle(1, "One")]))
                .unwrap(),
            IntentResult::Accepted
        );
        let listed = projection(&mut turnstone, 0);
        let parent = session_ids(&listed)[0];
        let offers = |listed: &ProjectionSnapshot, id: SessionId| -> Vec<String> {
            let instance = listed
                .scene
                .active_items_in_order()
                .into_iter()
                .find(|(_, item)| {
                    listed.scene.tables.sources[item.source.0 as usize]
                        .as_ref()
                        .is_some_and(|source| {
                            source.adapter == SESSIONS_SOURCE
                                && source.id == id.as_uuid().to_string()
                        })
                })
                .unwrap()
                .0;
            listed.presentation.offers_for(instance).unwrap()[0]
                .semantics
                .actions
                .iter()
                .map(|action| action.intent.0.clone())
                .collect()
        };
        assert!(offers(&listed, parent).contains(&BRANCH_SESSION_INTENT.to_string()));
        assert!(
            !offers(&listed, parent).contains(&MERGE_SESSION_INTENT.to_string()),
            "a session with no source has nothing to merge into"
        );

        // Turnstone branches; the branch offers Merge into source.
        let branch = sessions_step(&listed, BRANCH_SESSION_INTENT, SessionsActionV1::on(parent));
        assert_eq!(turnstone.invoke(branch).unwrap(), IntentResult::Accepted);
        let listed = projection(&mut knot, 0);
        let branch = session_ids(&listed)[1];
        assert!(offers(&listed, branch).contains(&MERGE_SESSION_INTENT.to_string()));

        // Knot retitles in the branch while Turnstone retitles the parent.
        let step = |listed: &ProjectionSnapshot, intent: &str, id: SessionId| {
            sessions_step(listed, intent, SessionsActionV1::on(id))
        };
        assert_eq!(
            knot.invoke(step(&listed, ATTACH_SESSION_INTENT, branch))
                .unwrap(),
            IntentResult::Accepted
        );
        let in_branch = graph(&mut knot);
        assert_eq!(
            knot.invoke(apply(&in_branch, vec![retitle(1, "Branch"), node(2)]))
                .unwrap(),
            IntentResult::Accepted
        );
        let in_parent = graph(&mut turnstone);
        assert_eq!(
            turnstone
                .invoke(apply(&in_parent, vec![retitle(1, "Parent")]))
                .unwrap(),
            IntentResult::Accepted
        );

        // The merge has a conflict, so it waits as a card with its material.
        assert_eq!(
            knot.invoke(step(&listed, MERGE_SESSION_INTENT, branch))
                .unwrap(),
            IntentResult::Accepted
        );
        let listed = projection(&mut knot, 0);
        let (_, material) = pending_merge(&mut knot, &listed).expect("a pending merge");
        assert_eq!((material.source, material.target), (branch, parent));
        assert_eq!(material.conflicts.len(), 1);
        assert_eq!(
            material.conflicts[0].part,
            kernel::graph::Part::Title(Uuid::from_u128(1))
        );
        assert!(
            !offers(&listed, branch).contains(&MERGE_SESSION_INTENT.to_string()),
            "one merge pending at a time"
        );

        // Through the route adapter: the view's card, a choice, then Apply.
        let mut read = read_route(&mut turnstone);
        let chosen = BTreeMap::from([(
            mere_route::conflict_key(&material.conflicts[0].part),
            mere_view::MergeOption::Source,
        )]);
        mere_route::fill_merge(&mut read.model, Some(&material), &chosen);
        let merge = read.model.merge.as_ref().unwrap();
        assert_eq!(merge.conflicts[0].label, "Title of Parent");
        assert_eq!(
            (
                merge.conflicts[0].source.as_str(),
                merge.conflicts[0].target.as_str()
            ),
            ("Branch", "Parent")
        );
        assert!(
            mere_route::merge_intent(
                &MereViewRequest::Merge(mere_view::MergeRequest::Apply),
                &read.sessions,
                &material,
                &BTreeMap::new(),
            )
            .is_none(),
            "Apply waits for every choice"
        );
        let apply_merge = mere_route::merge_intent(
            &MereViewRequest::Merge(mere_view::MergeRequest::Apply),
            &read.sessions,
            &material,
            &chosen,
        )
        .unwrap();
        assert_eq!(
            turnstone.invoke(apply_merge).unwrap(),
            IntentResult::Accepted
        );
        let read = read_route(&mut turnstone);
        assert_eq!(label_of(&read.model, 1).as_deref(), Some("Branch"));
        assert!(
            label_of(&read.model, 2).is_some(),
            "the branch's node arrives"
        );
        let listed = projection(&mut turnstone, 0);
        assert!(pending_merge(&mut turnstone, &listed).is_none());
        assert!(
            !offers(&listed, branch).contains(&MERGE_SESSION_INTENT.to_string()),
            "a merged branch is marked"
        );
        // The merged branch is read-only now.
        let in_branch = graph(&mut knot);
        assert!(matches!(
            knot.invoke(apply(&in_branch, vec![retitle(2, "Late")]))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));

        // Turnstone undoes the merge, which frees the branch to merge again.
        let in_parent = graph(&mut turnstone);
        assert_eq!(
            turnstone.invoke(undo(&in_parent)).unwrap(),
            IntentResult::Accepted
        );
        let read = read_route(&mut turnstone);
        assert_eq!(label_of(&read.model, 1).as_deref(), Some("Parent"));
        let listed = projection(&mut turnstone, 0);
        assert!(offers(&listed, branch).contains(&MERGE_SESSION_INTENT.to_string()));
        let in_branch = graph(&mut knot);
        assert_eq!(
            knot.invoke(apply(&in_branch, vec![retitle(2, "Late")]))
                .unwrap(),
            IntentResult::Accepted,
            "the freed branch is editable"
        );

        // Merging again and cancelling writes nothing.
        let listed = projection(&mut knot, 0);
        assert_eq!(
            knot.invoke(step(&listed, MERGE_SESSION_INTENT, branch))
                .unwrap(),
            IntentResult::Accepted
        );
        let listed = projection(&mut knot, 0);
        let (_, material) = pending_merge(&mut knot, &listed).unwrap();
        let read = read_route(&mut knot);
        let cancel = mere_route::merge_intent(
            &MereViewRequest::Merge(mere_view::MergeRequest::Cancel),
            &read.sessions,
            &material,
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(knot.invoke(cancel).unwrap(), IntentResult::Accepted);
        let listed = projection(&mut knot, 0);
        assert!(pending_merge(&mut knot, &listed).is_none());
        assert!(offers(&listed, branch).contains(&MERGE_SESSION_INTENT.to_string()));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_failed_apply_plans_again_and_carries_the_choices_that_still_apply() {
        let fixture = fixture(24, "readings").await;
        let mut turnstone = fixture.attach("turnstone").await;
        let mut knot = fixture.attach("knot-editor").await;
        let start = graph(&mut turnstone);
        assert_eq!(
            turnstone
                .invoke(apply(
                    &start,
                    vec![node(1), retitle(1, "One"), node(2), retitle(2, "Two")]
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let listed = projection(&mut turnstone, 0);
        let parent = session_ids(&listed)[0];
        let branch = sessions_step(&listed, BRANCH_SESSION_INTENT, SessionsActionV1::on(parent));
        assert_eq!(turnstone.invoke(branch).unwrap(), IntentResult::Accepted);
        let listed = projection(&mut knot, 0);
        let branch = session_ids(&listed)[1];
        let step = |listed: &ProjectionSnapshot, intent: &str| {
            sessions_step(listed, intent, SessionsActionV1::on(branch))
        };
        assert_eq!(
            knot.invoke(step(&listed, ATTACH_SESSION_INTENT)).unwrap(),
            IntentResult::Accepted
        );
        let in_branch = graph(&mut knot);
        knot.invoke(apply(
            &in_branch,
            vec![retitle(1, "Branch one"), retitle(2, "Branch two")],
        ))
        .unwrap();
        let in_parent = graph(&mut turnstone);
        turnstone
            .invoke(apply(
                &in_parent,
                vec![retitle(1, "Parent one"), retitle(2, "Parent two")],
            ))
            .unwrap();
        assert_eq!(
            knot.invoke(step(&listed, MERGE_SESSION_INTENT)).unwrap(),
            IntentResult::Accepted
        );
        let listed = projection(&mut knot, 0);
        let (_, material) = pending_merge(&mut knot, &listed).unwrap();
        assert_eq!(material.conflicts.len(), 2);
        let key =
            |n: u128| mere_route::conflict_key(&kernel::graph::Part::Title(Uuid::from_u128(n)));
        let chosen = BTreeMap::from([
            (key(1), mere_view::MergeOption::Source),
            (key(2), mere_view::MergeOption::Target),
        ]);
        let read = read_route(&mut knot);
        let apply_merge = mere_route::merge_intent(
            &MereViewRequest::Merge(mere_view::MergeRequest::Apply),
            &read.sessions,
            &material,
            &chosen,
        )
        .unwrap();

        // The parent moves before Apply lands: it now agrees on node 2.
        let in_parent = graph(&mut turnstone);
        turnstone
            .invoke(apply(&in_parent, vec![retitle(2, "Branch two")]))
            .unwrap();
        match knot.invoke(apply_merge).unwrap() {
            IntentResult::Rejected { reason } => assert!(
                reason.contains("planned again")
                    && reason.contains("1 of your choices carried over, 1 dropped"),
                "{reason}"
            ),
            other => panic!("{other:?}"),
        }

        // The new plan waits, carrying the choice that still applies and
        // naming the one dropped; the panel shows both.
        let listed = projection(&mut knot, 0);
        let (_, material) = pending_merge(&mut knot, &listed).expect("planned again");
        assert_eq!(material.conflicts.len(), 1);
        assert_eq!(material.carried.len(), 1);
        assert_eq!(material.dropped.len(), 1);
        assert_eq!(material.dropped[0].reason, "no longer in conflict");
        let mut chosen = chosen;
        mere_route::adopt_merge(&mut chosen, &material);
        assert_eq!(
            chosen,
            BTreeMap::from([(key(1), mere_view::MergeOption::Source)])
        );
        let mut read = read_route(&mut knot);
        mere_route::fill_merge(&mut read.model, Some(&material), &chosen);
        let merge = read.model.merge.as_ref().unwrap();
        assert_eq!(merge.dropped.len(), 1);
        assert!(
            merge.dropped[0].contains("Target (no longer in conflict)"),
            "{:?}",
            merge.dropped
        );
        assert!(merge.resolved(), "the carried choice resolves the new plan");
        let apply_merge = mere_route::merge_intent(
            &MereViewRequest::Merge(mere_view::MergeRequest::Apply),
            &read.sessions,
            &material,
            &chosen,
        )
        .unwrap();
        assert_eq!(knot.invoke(apply_merge).unwrap(), IntentResult::Accepted);
        let read = read_route(&mut turnstone);
        assert_eq!(label_of(&read.model, 1).as_deref(), Some("Branch one"));
        assert_eq!(label_of(&read.model, 2).as_deref(), Some("Branch two"));
    }

    fn relate(from: u128, to: u128, assertion: EdgeAssertion) -> pandect::CapturedDelta {
        pandect::CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(from).to_string(),
            to_id: Uuid::from_u128(to).to_string(),
            assertion,
        }
    }

    fn semantic(sub_kind: SemanticSubKind) -> EdgeAssertion {
        EdgeAssertion::Semantic {
            sub_kind,
            label: None,
        }
    }

    /// The route as the adapter reads it: both projections, the cards they
    /// name, and the model they fill.
    struct RouteRead {
        sessions: ProjectionSnapshot,
        model: MereViewModel,
    }

    fn read_route(session: &mut ResidentEndpointSession) -> RouteRead {
        let (sessions, graph) =
            mere_route::requests(&session.describe()).expect("a mere's route offers both");
        let sessions = session.snapshot(sessions).unwrap();
        let graph = session.snapshot(graph).unwrap();
        let cards: HashMap<_, _> = mere_route::card_requests(&sessions)
            .into_iter()
            .map(|request| (request.resource, session.resource(request).unwrap().bytes))
            .collect();
        let mut model = MereViewModel::default();
        mere_route::fill(&mut model, &sessions, &graph, &cards);
        RouteRead { sessions, model }
    }

    /// Take `request` through the adapter, from what `read` saw.
    fn take(
        session: &mut ResidentEndpointSession,
        read: &RouteRead,
        request: MereViewRequest,
    ) -> IntentResult {
        let intent = mere_route::intent(&request, &read.sessions).expect("a step is the route's");
        session.invoke(intent).unwrap()
    }

    /// The one session `now` lists that `before` did not.
    fn new_session(before: &RouteRead, now: &RouteRead) -> SessionId {
        let known: Vec<SessionId> = before.model.sessions.iter().map(|entry| entry.id).collect();
        let fresh: Vec<SessionId> = now
            .model
            .sessions
            .iter()
            .map(|entry| entry.id)
            .filter(|id| !known.contains(id))
            .collect();
        assert_eq!(fresh.len(), 1, "{fresh:?}");
        fresh[0]
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_route_adapter_takes_the_lifecycle_and_shows_the_route() {
        let fixture = fixture(23, "notes").await;
        let mut knot = fixture.attach("knot-editor").await;
        let start = graph(&mut knot);
        let edits = vec![
            node(1),
            node(2),
            retitle(1, "Alpha"),
            relate(1, 2, semantic(SemanticSubKind::Hyperlink)),
            relate(1, 2, semantic(SemanticSubKind::UserGrouped)),
            relate(
                1,
                2,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::UserFolder,
                },
            ),
        ];
        assert_eq!(
            knot.invoke(apply(&start, edits)).unwrap(),
            IntentResult::Accepted
        );

        // The model shows the route: the mere, its one session attached, and
        // that session's graph. The shared link grammar paints one pair while
        // its admitted graph retains the three distinct source assertions.
        let first = read_route(&mut knot);
        assert_eq!(first.model.title, "notes");
        assert!(first.model.can_mint);
        let [parent] = first.model.sessions.as_slice() else {
            panic!("{:?}", first.model.sessions)
        };
        assert!(parent.attached && !parent.trashed);
        assert_eq!(
            parent.steps,
            [SessionStep::Fork, SessionStep::Branch, SessionStep::Trash]
        );
        let parent = parent.id;
        let (one, two) = (
            Uuid::from_u128(1).to_string(),
            Uuid::from_u128(2).to_string(),
        );
        let mut nodes: Vec<(&str, &str)> = first
            .model
            .graph
            .nodes
            .iter()
            .map(|node| (node.key.as_str(), node.label.as_str()))
            .collect();
        nodes.sort();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&(one.as_str(), "Alpha")), "{nodes:?}");
        let relations = &first.model.graph.relations;
        assert!(
            relations
                .iter()
                .all(|relation| relation.from == one && relation.to == two)
        );
        let mut kinds: Vec<&str> = relations
            .iter()
            .map(|relation| relation.provenance.token())
            .collect();
        kinds.sort();
        assert_eq!(kinds, ["link.surface"]);
        let projected = graph(&mut knot);
        let material = disclosed_graph(&mut knot, &projected).into_graph().unwrap();
        let source_kinds: Vec<_> = material.relations().map(|r| r.kind).collect();
        assert_eq!(source_kinds.len(), 3);
        assert!(
            source_kinds.contains(&kernel::graph::RelationKind::Containment(
                ContainmentSubKind::UserFolder
            ))
        );
        assert!(
            source_kinds.contains(&kernel::graph::RelationKind::Semantic(
                SemanticSubKind::Hyperlink
            ))
        );
        assert!(
            source_kinds.contains(&kernel::graph::RelationKind::Semantic(
                SemanticSubKind::UserGrouped
            ))
        );
        let mut keys: Vec<&str> = relations
            .iter()
            .map(|relation| relation.key.as_str())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 1, "the canonical pair keeps one link key");
        assert!(!keys[0].is_empty());

        // Mint, then fork the attached session from the first list, which the
        // mint has moved past (§7 item 33).
        assert_eq!(
            take(&mut knot, &first, MereViewRequest::Mint),
            IntentResult::Accepted
        );
        let minted = read_route(&mut knot);
        let fresh = new_session(&first, &minted);
        let fork_parent = MereViewRequest::Session(parent, SessionStep::Fork);
        assert_eq!(take(&mut knot, &first, fork_parent), IntentResult::Accepted);
        let forked = read_route(&mut knot);
        let fork = new_session(&minted, &forked);

        // Switch to the fork: the model shows it attached, over the graph it
        // took from its parent.
        let switch = MereViewRequest::Session(fork, SessionStep::Switch);
        assert_eq!(take(&mut knot, &forked, switch), IntentResult::Accepted);
        let switched = read_route(&mut knot);
        let attached: Vec<SessionId> = switched
            .model
            .sessions
            .iter()
            .filter(|entry| entry.attached)
            .map(|entry| entry.id)
            .collect();
        assert_eq!(attached, [fork]);
        assert_eq!(switched.model.graph.nodes.len(), 2);
        assert_eq!(switched.model.graph.relations.len(), 1);
        let projected = graph(&mut knot);
        let fork_graph = disclosed_graph(&mut knot, &projected).into_graph().unwrap();
        assert_eq!(fork_graph.relations().count(), 3);

        // Trash the minted session and restore it. A step its card does not
        // offer now is declined with the route's reason.
        let trash = MereViewRequest::Session(fresh, SessionStep::Trash);
        assert_eq!(
            take(&mut knot, &switched, trash.clone()),
            IntentResult::Accepted
        );
        let trashed = read_route(&mut knot);
        let entry = trashed
            .model
            .sessions
            .iter()
            .find(|entry| entry.id == fresh)
            .unwrap();
        assert!(entry.trashed && entry.steps.contains(&SessionStep::Restore));
        let reason = mere_route::declined(&take(&mut knot, &trashed, trash))
            .expect("the route declines a second trash");
        assert!(reason.contains("does not offer"), "{reason}");
        let restore = MereViewRequest::Session(fresh, SessionStep::Restore);
        assert_eq!(take(&mut knot, &trashed, restore), IntentResult::Accepted);
        let restored = read_route(&mut knot);

        // Opening a node, a host action and a layout are the host's own.
        for own in [
            MereViewRequest::Activate(one.clone()),
            MereViewRequest::Host("new".into()),
            MereViewRequest::Layout("grid.default".into()),
        ] {
            assert!(mere_route::intent(&own, &restored.sessions).is_none());
        }

        // The model lists the mere's sessions as stored, oldest first, as
        // `MereSessions::list` orders them. djinn holds the store open alone,
        // so its files are read directly.
        #[derive(serde::Deserialize)]
        struct StoredChange {
            entry: Change,
        }
        let mere = fixture.reservoir.meres().await.remove(0);
        let sessions = mere_dir(fixture._root.path(), persona(23), mere.id).join("sessions");
        let read = |path: std::path::PathBuf| std::fs::read_to_string(path).unwrap();
        let mut stored: Vec<(GraphSessionManifest, Vec<Change>)> = std::fs::read_dir(sessions)
            .unwrap()
            .map(|entry| {
                let dir = entry.unwrap().path();
                let manifest = serde_json::from_str(&read(dir.join("manifest.json"))).unwrap();
                let changes = read(dir.join("changes.jsonl"))
                    .lines()
                    .map(|line| serde_json::from_str::<StoredChange>(line).unwrap().entry)
                    .collect();
                (manifest, changes)
            })
            .collect();
        stored.sort_by_key(|(manifest, _)| (manifest.created_at, *manifest.session_id.as_uuid()));
        let listed: Vec<(SessionId, String, bool)> = stored
            .iter()
            .map(|(manifest, _)| {
                let name = manifest.display_name.clone();
                (
                    manifest.session_id,
                    name.unwrap_or_else(|| "Session".into()),
                    manifest.trashed.is_some(),
                )
            })
            .collect();
        let shown: Vec<(SessionId, String, bool)> = restored
            .model
            .sessions
            .iter()
            .map(|entry| (entry.id, entry.name.clone(), entry.trashed))
            .collect();
        assert_eq!(shown, listed);

        // Mint, fork, trash and restore are recorded with the admitted
        // application; the switch changed nothing stored (§7 item 41).
        let via = |id: SessionId| -> Vec<(ChangeKind, Option<String>)> {
            let (_, changes) = stored
                .iter()
                .find(|(manifest, _)| manifest.session_id == id)
                .expect("the session is stored");
            changes
                .iter()
                .map(|change| (change.kind.clone(), change.author.via.clone()))
                .collect()
        };
        let knot_editor = Some("knot-editor".to_string());
        assert_eq!(
            via(fresh),
            [
                (ChangeKind::Minted, knot_editor.clone()),
                (ChangeKind::Trashed, knot_editor.clone()),
                (ChangeKind::Restored, knot_editor.clone()),
            ]
        );
        let fork_log = via(fork);
        let [(ChangeKind::Forked { from, .. }, forked_via)] = fork_log.as_slice() else {
            panic!("{fork_log:?}")
        };
        assert_eq!((*from, forked_via), (parent, &knot_editor));
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn access_is_default_for_admitted_apps_and_ambient_is_independent() {
        let fixture = fixture(24, "divination").await;
        let mere = fixture.reservoir.meres().await.remove(0);
        // Admission can precede or follow serving; no per-mere grant needed.
        fixture.grants.grant(
            AppId::new("cleromancy"),
            ResidentEndpointRoute::new("identity", MERE_NOTICE_POLL).unwrap(),
        );
        let mut explicit = fixture.attach("cleromancy").await;
        let before = graph(&mut explicit);
        let ambient_id = ambient_mere_route_id("divination");
        let request = |app: &str, route: &str| AppRequest {
            app: AppId::new(app),
            route: AppRouteId::new(route).unwrap(),
        };
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &fixture.route))
                .is_ok()
        );
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("unknown", &fixture.route))
                .is_err()
        );
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &ambient_id))
                .is_err()
        );
        assert!(
            fixture
                .routes
                .set_access(
                    &fixture.reservoir,
                    mere.id,
                    AppId::new("unknown"),
                    false,
                    true,
                    AppId::new("turnstone")
                )
                .await
                .is_err()
        );

        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("cleromancy"),
                false,
                true,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("knot-editor", &ambient_id))
                .is_err()
        );
        let mut ambient = fixture
            .catalog
            .update(|catalog| catalog.open(&ambient_id, &context("cleromancy")))
            .await
            .unwrap();
        assert_eq!(ambient.describe().projections.len(), 1);
        let read = projection(&mut ambient, 0);
        let explicit_offer = &before
            .presentation
            .offers_for(session_item_of(&before))
            .unwrap()[0];
        let explicit_card = explicit
            .resource(ResourceRequest {
                session: before.session.clone(),
                resource: explicit_offer.resource,
            })
            .unwrap();
        let explicit_card: PortableCardV1 = serde_json::from_slice(&explicit_card.bytes).unwrap();
        let full_graph = explicit_card.media[0];
        let ambient_offer = &read
            .presentation
            .offers_for(session_item_of(&read))
            .unwrap()[0];
        let ambient_card = ambient
            .resource(ResourceRequest {
                session: read.session.clone(),
                resource: ambient_offer.resource,
            })
            .unwrap();
        let ambient_card: PortableCardV1 = serde_json::from_slice(&ambient_card.bytes).unwrap();
        assert!(
            ambient_card.media.is_empty(),
            "ambient card never advertises complete domain material"
        );
        assert!(
            ambient
                .resource(ResourceRequest {
                    session: read.session.clone(),
                    resource: full_graph
                })
                .is_err(),
            "knowing an explicit hash does not grant its complete graph through ambient access"
        );
        assert!(
            read.presentation
                .offers
                .values()
                .flatten()
                .all(|offer| offer.semantics.actions.is_empty())
        );
        for intent in [
            MINT_SESSION_INTENT,
            FORK_SESSION_INTENT,
            TRASH_SESSION_INTENT,
            session_item::SET_VIEW_INTENT,
            session_item::APPLY_EDITS_INTENT,
        ] {
            let mut invocation = apply(&read, vec![node(31)]);
            invocation.intent = intent.into();
            assert!(matches!(
                ambient.invoke(invocation).unwrap(),
                IntentResult::Rejected { .. }
            ));
        }
        // No opt-in on another domain and no access through the sessions path.
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &ambient_mere_route_id("notes")))
                .is_err()
        );
        let sessions = explicit
            .describe()
            .projections
            .into_iter()
            .find(|offer| offer.request.session == ProjectionSession(MERE_SESSIONS.into()))
            .expect("the explicit owner advertises its sessions projection")
            .request;
        assert!(ambient.snapshot(sessions).is_err());

        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("cleromancy"),
                true,
                true,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &fixture.route))
                .unwrap_err()
                .to_string()
                .contains("explicitly denied")
        );
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &ambient_id))
                .is_err()
        );
        assert!(explicit.invoke(apply(&before, vec![node(32)])).is_err());
        assert!(ambient.snapshot(read_request(&ambient)).is_err());
        assert!(
            fixture
                .catalog
                .update(|catalog| catalog.open(&fixture.route, &context("cleromancy")))
                .await
                .is_err()
        );
        // A denied application cannot enumerate or ensure the hidden mere.
        let mut reservoir = fixture
            .catalog
            .update(|catalog| catalog.open(RESIDENT_RESERVOIR_ROUTE, &context("cleromancy")))
            .await
            .unwrap();
        let listed = projection(&mut reservoir, 0);
        assert_eq!(listed.presentation.bindings.len(), 1);
        let ensure = IntentInvocation {
            session: listed.session.clone(),
            target: InstanceId(0),
            observed_epoch: listed.scene.epoch,
            observed_revision: listed.scene.revision,
            intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
            payload: serde_json::to_vec(&EnsureMereV1::new("divination")).unwrap(),
        };
        assert!(matches!(
            reservoir.invoke(ensure).unwrap(),
            IntentResult::Rejected { .. }
        ));

        // Removing the denial retains the separately recorded ambient opt-in.
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("cleromancy"),
                false,
                true,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &ambient_id))
                .is_ok()
        );
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                AppId::new("cleromancy"),
                false,
                false,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        assert!(ambient.snapshot(read_request(&ambient)).is_err());
        assert!(
            fixture
                .grants
                .current()
                .admit(&request("cleromancy", &fixture.route))
                .is_ok()
        );
        let decisions = fixture.reservoir.access_decisions().await;
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].2.author.via.as_deref(), Some("turnstone"));
    }

    fn read_request(endpoint: &ResidentEndpointSession) -> ProjectionRequest {
        endpoint.describe().projections[0].request.clone()
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn a_product_client_cannot_clear_denial_or_opt_in_ambient_through_the_reservoir() {
        let fixture = fixture(25, "divination").await;
        let mere = fixture.reservoir.meres().await.remove(0);
        let denied = AppId::new("knot-editor");
        fixture
            .routes
            .set_access(
                &fixture.reservoir,
                mere.id,
                denied.clone(),
                true,
                false,
                AppId::new("turnstone"),
            )
            .await
            .unwrap();
        let before = fixture.reservoir.access_decisions().await;
        // Both the denied client and another admitted product may inspect
        // the reservoir, but neither receives permission-management authority.
        for actor in ["knot-editor", "turnstone"] {
            fixture
                .grants
                .grant(AppId::new(actor), ResidentReservoir::route());
            let request = AppRequest {
                app: AppId::new(actor),
                route: AppRouteId::new(RESIDENT_RESERVOIR_ROUTE).unwrap(),
            };
            assert!(fixture.grants.current().admit(&request).is_ok());
            let mut control = fixture
                .catalog
                .update(|catalog| catalog.open(RESIDENT_RESERVOIR_ROUTE, &context(actor)))
                .await
                .unwrap();
            let snapshot = projection(&mut control, 0);
            assert!(
                snapshot
                    .presentation
                    .offers
                    .values()
                    .flatten()
                    .flat_map(|offer| &offer.semantics.actions)
                    .all(|action| action.intent.0 != "mere.reservoir.set-access")
            );
            let invocation = IntentInvocation { session: snapshot.session.clone(), target: InstanceId(0), observed_epoch: snapshot.scene.epoch, observed_revision: snapshot.scene.revision, intent: "mere.reservoir.set-access".into(), payload: serde_json::to_vec(&serde_json::json!({ "schema": "mere.reservoir.set-access/v1", "mere": mere.id, "application": "knot-editor", "denied": false, "ambient": true, "via": "turnstone" })).unwrap() };
            assert!(matches!(
                control.invoke(invocation).unwrap(),
                IntentResult::Rejected { .. }
            ));
        }
        assert_eq!(fixture.reservoir.access_decisions().await, before);
        for route in [&fixture.route, &ambient_mere_route_id("divination")] {
            assert!(
                fixture
                    .grants
                    .current()
                    .admit(&AppRequest {
                        app: denied.clone(),
                        route: AppRouteId::new(route).unwrap()
                    })
                    .is_err()
            );
            assert!(
                fixture
                    .catalog
                    .update(|catalog| catalog.open(route, &context("knot-editor")))
                    .await
                    .is_err()
            );
        }
    }

    mod mcp;
}
