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
//! id (§7 item 33) in `graphshell::session_item`'s vocabulary (§7 item 37),
//! and the attached session's graph, served through `MereHost` with its
//! session item (§7 items 21, 22 and 31). Every application attached to one
//! session shares one host, so each sees the others' edits, told by a
//! revision bell. The application an edit comes through is the one the door
//! admitted, never the client's claim.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::resident_reservoir::ResidentReservoir;
use chirograph::{
    AdvertisedAction, BoundsRelationship, CachePolicy, CardValueV1, CarrierNotice, ContentHash,
    EndpointDescriptor, IntentEffect, IntentInvocation, IntentReference, IntentResult,
    PortableCardV1, PresentationBinding, PresentationCapability, PresentationCodec,
    PresentationKey, PresentationManifest, PresentationOffer, PresentationSemantics,
    ProjectionOffer, ProjectionRequest, ProjectionSession, ProjectionSnapshot, ProtocolVersion,
    ResourceRequest, ResourceResponse, ResumeReply, ResumeRequest, SemanticRole,
};
use graphshell::access::AccessContext;
use graphshell::handlers::HandlerRegistry;
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::mere_host::SelectedPersonaRef;
use graphshell::native::app_admission::{AppId, AppRequest, AppRouteGrants, AppRouteId};
use graphshell::native::app_broker::AppEndpointCatalog;
use graphshell::native::endpoint_catalog::ResidentEndpointRoute;
#[cfg(test)]
use graphshell::resident_host::test_support::{
    DirectoryFaultControl, DirectoryFixture, FixtureDirectory as DirectoryBackend,
};
use graphshell::resident_host::{CachedResidentSession, DirectoryHostCache};
use graphshell::session_item::{
    self, ARCHIVE_SCHEMA, ARCHIVE_SOURCE, ATTACH_SESSION_INTENT, ApplyEditsV1, ArchiveActionV1,
    COMPOSE_CODICILS_INTENT, FORK_SESSION_INTENT, MERE_ARCHIVE, MERE_GRAPH, MERE_SESSIONS,
    MINT_SESSION_INTENT, OPEN_CODICIL_INTENT, RESTORE_SESSION_INTENT, SAVE_CODICIL_INTENT,
    SESSIONS_SCHEMA, SESSIONS_SOURCE, SessionsActionV1, SetViewV1, StepV1, TRASH_SESSION_INTENT,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
    ResumableProjectionSource,
};
#[cfg(not(test))]
use muniment::DirectoryBackend;
use muniment::{Backend, RedbBackend};
use pandect::{
    Author, GraphSessionManifest, MereApplicationAccess, MereId, MereRecord, SessionId,
    open_mere_archive_backend,
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

struct Routes {
    shared_root: PathBuf,
    persona: PersonaId,
    catalog: AppEndpointCatalog,
    grants: AppRouteGrants,
    apps: Vec<AppId>,
    validators: BTreeMap<String, pandect::GraphValidator>,
    served: Mutex<BTreeMap<String, Arc<MereShared>>>,
    access: Mutex<BTreeMap<(MereId, AppId), MereApplicationAccess>>,
    access_loaded: AtomicBool,
}

impl MereRoutes {
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
                shared_root,
                persona,
                catalog,
                grants,
                apps,
                validators: BTreeMap::new(),
                served: Mutex::new(BTreeMap::new()),
                access: Mutex::new(BTreeMap::new()),
                access_loaded: AtomicBool::new(false),
            }),
        }
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
        let route = ResidentEndpointRoute::new(mere_route_id(&domain), MERE_NOTICE_POLL)
            .map_err(|error| error.to_string())?;
        let mut served = self.inner.served.lock().await;
        if served.contains_key(&domain) {
            return Ok(route);
        }
        #[cfg(not(test))]
        let cache = DirectoryHostCache::open(pandect::mere_dir(
            &self.inner.shared_root,
            self.inner.persona,
            mere.id,
        ))
        .map_err(|error| error.to_string())?;
        #[cfg(test)]
        let fixture_owner = DirectoryFixture::open(pandect::mere_dir(
            &self.inner.shared_root,
            self.inner.persona,
            mere.id,
        ))
        .map_err(|error| error.to_string())?;
        #[cfg(test)]
        let (cache, fixture_owner) = fixture_owner.into_parts();
        let shared = Arc::new(MereShared {
            domain: domain.clone(),
            validator: self.inner.validators.get(&domain).cloned(),
            archive: open_mere_archive_backend(
                &self.inner.shared_root,
                self.inner.persona,
                mere.id,
            )
            .map_err(|e| e.to_string())?,
            cache,
            #[cfg(test)]
            fixture_owner: Some(fixture_owner),
            persona: self.inner.persona,
            lifecycle: AtomicU64::new(1),
        });
        let opened = Arc::clone(&shared);
        let ambient_opened = Arc::clone(&shared);
        let explicit_grants = self.inner.grants.clone();
        let ambient_grants = self.inner.grants.clone();
        let explicit_id = route.id().to_string();
        let ambient = ResidentEndpointRoute::new(ambient_mere_route_id(&domain), MERE_NOTICE_POLL)
            .map_err(|error| error.to_string())?;
        let ambient_id = ambient.id().to_string();
        self.inner
            .catalog
            .update(|catalog| {
                catalog.register_resumable_notifying(
                    route.id(),
                    format!("Mere: {domain}"),
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
                catalog.register_resumable_notifying(
                    ambient.id(),
                    format!("Ambient mere: {domain}"),
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
            })
            .await
            .map_err(|error| error.to_string())?;
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

/// Recheck live owner policy before factory construction and on every read
/// or write. Existing sessions cannot keep reading after a denial or ambient
/// opt-out. Ambient routes offer graph reads only and accept no intents.
struct AccessEndpoint<B: Backend + Clone + Send + Sync + 'static = DirectoryBackend> {
    endpoint: MereEndpoint<B>,
    grants: AppRouteGrants,
    request: AppRequest,
    ambient: bool,
}

impl<B: Backend + Clone + Send + Sync + 'static> AccessEndpoint<B> {
    fn open(
        shared: Arc<MereShared<B>>,
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

impl<B: Backend + Clone + Send + Sync + 'static> ProjectionCatalog for AccessEndpoint<B> {
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
impl<B: Backend + Clone + Send + Sync + 'static> ProjectionSource for AccessEndpoint<B> {
    type Error = String;
    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        self.check()?;
        if self.ambient && request.session.0 != MERE_GRAPH {
            return Err("ambient routes expose graph material only".into());
        }
        let mut snapshot = self.endpoint.snapshot(request)?;
        if self.ambient {
            for offers in snapshot.presentation.offers.values_mut() {
                for offer in offers {
                    offer.semantics.actions.clear();
                }
            }
        }
        Ok(snapshot)
    }
}
impl<B: Backend + Clone + Send + Sync + 'static> ResumableProjectionSource for AccessEndpoint<B> {
    type Error = String;
    fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, String> {
        self.check()?;
        let projection = self
            .describe()
            .projections
            .into_iter()
            .find(|offer| offer.request.session == request.session)
            .ok_or("resume names an undisclosed projection")?;
        // Context is snapshot metadata, so a notice always obtains fresh context.
        // The client retains unchanged resources through apply_snapshot.
        self.snapshot(projection.request)
            .map(|snapshot| ResumeReply::Snapshot(Box::new(snapshot)))
    }
}
impl<B: Backend + Clone + Send + Sync + 'static> PresentationSource for AccessEndpoint<B> {
    type Error = String;
    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        self.check()?;
        if self.ambient && request.session.0 != MERE_GRAPH {
            return Err("ambient routes expose graph material only".into());
        }
        self.endpoint.resource(request)
    }
}
impl<B: Backend + Clone + Send + Sync + 'static> IntentSink for AccessEndpoint<B> {
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
impl<B: Backend + Clone + Send + Sync + 'static> ProjectionNoticeSource for AccessEndpoint<B> {
    type Error = String;
    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> {
        self.check()?;
        let notice = self.endpoint.poll_notice()?;
        Ok(notice.filter(|notice| !self.ambient || notice.session.0 == MERE_GRAPH))
    }
}

type SharedHost<B = DirectoryBackend> = Arc<Mutex<CachedResidentSession<B>>>;

/// One mere's store, and the host of each of its sessions an application has
/// attached to.
struct MereShared<B = DirectoryBackend> {
    domain: String,
    validator: Option<pandect::GraphValidator>,
    cache: DirectoryHostCache<B>,
    #[cfg(test)]
    fixture_owner: Option<DirectoryFaultControl>,
    archive: RedbBackend,
    persona: PersonaId,
    /// The sessions projection's revision: it moves with every lifecycle step.
    lifecycle: AtomicU64,
}

impl<B: Backend + Clone + Send + Sync + 'static> MereShared<B> {
    fn validate(&self, graph: &pandect::graph_codicil::GraphCandidate) -> Result<(), String> {
        if let Some(validator) = &self.validator {
            validator(graph)?;
        }
        Ok(())
    }

    fn sessions(&self) -> &DirectoryHostCache<B> {
        &self.cache
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

    /// The one host of session `id`, opened once and shared by every
    /// application attached to it.
    async fn host(&self, id: SessionId) -> Result<SharedHost<B>, String> {
        let persona = self.persona_id();
        self.cache
            .host_checked(
                id,
                SelectedPersonaRef {
                    persona: persona.clone(),
                    profile: "djinn".to_string(),
                },
                HandlerRegistry::new(Vec::new()),
                AccessContext {
                    persona,
                    device: "djinn".to_string(),
                    at_ms: 0,
                },
                self.validator.clone(),
                ProjectionSession(MERE_GRAPH.to_string()),
            )
            .await
            .map_err(|error| error.to_string())
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

/// A card of the sessions projection and what it is served as.
#[derive(Clone)]
struct SessionsCard {
    source: String,
    label: String,
    bytes: Vec<u8>,
    actions: Vec<AdvertisedAction>,
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
struct MereEndpoint<B: Backend + Clone + Send + Sync + 'static = DirectoryBackend> {
    shared: Arc<MereShared<B>>,
    app: String,
    attached: SessionId,
    host: SharedHost<B>,
    /// Minted by the admitted endpoint, independent of any payload view name.
    pane: pandect::ViewKey,
    /// The graph epoch and revision this connection last saw or was told of.
    seen_graph: (SceneEpoch, Revision),
    /// The sessions revision this connection last saw or was told of.
    seen_sessions: u64,
    seen_archive: u64,
}

impl<B: Backend + Clone + Send + Sync + 'static> MereEndpoint<B> {
    fn open_mode(
        shared: Arc<MereShared<B>>,
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
            let pane = pandect::ViewKey::new(&app, uuid::Uuid::new_v4().to_string())
                .map_err(|e| e.to_string())?;
            let seen_graph = {
                let mut host = host.lock().await;
                host.set_resident_pane(pane.clone(), Default::default());
                (host.projection_epoch(), host.projection_revision())
            };
            Ok(Self {
                seen_sessions: shared.lifecycle.load(Ordering::SeqCst),
                seen_archive: shared.lifecycle.load(Ordering::SeqCst),
                shared,
                app,
                attached,
                host,
                pane,
                seen_graph,
            })
        })
    }

    fn attach_host(&mut self, id: SessionId, host: SharedHost<B>) {
        if !Arc::ptr_eq(&self.host, &host) {
            run(async {
                let mut previous = self.host.lock().await;
                previous.close_resident_pane(&self.pane);
                let _ = previous.refresh_resident_reader().await;
                drop(previous);
                host.lock()
                    .await
                    .set_resident_pane(self.pane.clone(), Default::default());
            });
        }
        self.attached = id;
        self.host = host;
        self.seen_graph = (SceneEpoch(0), Revision(0));
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
        });
        for manifest in manifests {
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
                actions: self.session_actions(&manifest),
            });
        }
        Ok(cards)
    }

    /// The steps a session's card offers as the session stands now: attach
    /// unless this connection holds it, fork, and trash or restore.
    fn session_actions(&self, manifest: &GraphSessionManifest) -> Vec<AdvertisedAction> {
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
            let source = scene.intern_source(SourceRef::new(source_kind, &card.source));
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
            let key = PresentationKey(card.source.clone());
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
            coverage: None,
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
            }];
            for manifest in manifests {
                let id = manifest.id.to_string();
                let card = PortableCardV1 {
                    title: format!("Codicil {}", &id[..12]),
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
                    ],
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
        let outcome = (|| -> Result<Option<(SessionId, SharedHost<B>)>, String> {
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
                        host.ensure_metadata_idle()
                            .map_err(|error| error.to_string())?;
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
                self.attach_host(id, host);
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
        let names_session = match intent.intent.as_str() {
            MINT_SESSION_INTENT => false,
            ATTACH_SESSION_INTENT
            | FORK_SESSION_INTENT
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
            if !self
                .session_actions(manifest)
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
            Some(id)
        } else {
            None
        };
        let shared = Arc::clone(&self.shared);
        let author = shared.author(&self.app);
        let app = self.app.clone();
        let outcome: Result<Option<(SessionId, SharedHost<B>)>, String> = run(async move {
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
                    shared
                        .sessions()
                        .fork_at(&parent, author)
                        .await
                        .map_err(|error| error.to_string())?;
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
                self.attach_host(id, host);
                // The attached session's graph is new to this connection; the
                // others' views of the sessions did not move.
                self.seen_graph = (SceneEpoch(0), Revision(0));
                Ok(IntentResult::Accepted)
            },
            Ok(None) => Ok(IntentResult::Accepted),
            Err(reason) => Ok(IntentResult::Rejected { reason }),
        }
    }

    fn invoke_graph(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        use graphshell::resident_host::{
            DEMAND_INTENT, OBSERVATION_SCHEMA, ObservationRequestV1, REFRESH_INTENT,
        };
        if matches!(intent.intent.as_str(), DEMAND_INTENT | REFRESH_INTENT) {
            let payload: ObservationRequestV1 = match serde_json::from_slice(&intent.payload) {
                Ok(payload) => payload,
                Err(error) => {
                    return Ok(IntentResult::Rejected {
                        reason: format!("invalid observation payload: {error}"),
                    });
                },
            };
            if payload.schema != OBSERVATION_SCHEMA
                || (intent.intent == DEMAND_INTENT) != payload.address.is_some()
            {
                return Ok(IntentResult::Rejected {
                    reason: "observation schema or address does not match the advertised action"
                        .into(),
                });
            }
            let host = self.host.clone();
            let request = run(async {
                let mut host = host.lock().await;
                host.sync_projection_context();
                if intent.observed_epoch != host.projection_epoch()
                    || intent.observed_revision != host.projection_revision()
                {
                    return Err(IntentResult::Stale {
                        current_epoch: host.projection_epoch(),
                        current_revision: host.projection_revision(),
                    });
                }
                if Some(intent.target) != host.session_instance()
                    || host.resident_reader().is_none()
                {
                    return Err(IntentResult::Rejected {
                        reason: "observation action was not advertised for this target".into(),
                    });
                }
                if intent.intent == REFRESH_INTENT && host.recovery_pending() {
                    // An admitted owner retries its retained IO/publication only.
                    // The original command is never submitted again.
                    let _outcome = host.persist_owned(now_secs()).await;
                    return Ok(None);
                }
                let request = if let Some(address) = payload.address {
                    host.prepare_resident_demand(address)
                } else {
                    host.notice_resident_refresh();
                    host.prepare_resident_refresh()
                };
                request
                    .map_err(|e| IntentResult::Rejected {
                        reason: e.to_string(),
                    })?
                    .ok_or_else(|| IntentResult::Rejected {
                        reason: "presentation refresh is busy".into(),
                    })
                    .map(Some)
            });
            let request = match request {
                Ok(Some(request)) => request,
                Ok(None) => return Ok(IntentResult::Accepted),
                Err(result) => return Ok(result),
            };
            // No shared host borrow survives I/O; installation verifies the owner token.
            let completion = run(request.load());
            let result = run(async { host.lock().await.install_resident_refresh(completion) });
            return Ok(match result {
                Ok(_) => IntentResult::Accepted,
                Err(error) => IntentResult::Rejected {
                    reason: error.to_string(),
                },
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
                let author = shared.author(&app);
                let id = shared
                    .sessions()
                    .fork_with_intent(&parent, author, &app, &intent)
                    .await?;
                Ok(Ok(id))
            });
            match candidate {
                Ok(Ok(id)) => {
                    let host = run(self.shared.host(id))?;
                    self.attach_host(id, host);
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
        let pane = self.pane.clone();
        let (result, seen) = run(async move {
            let mut host = host.lock().await;
            if Some(intent.target) != host.session_instance() {
                let result = host.invoke_as(&app, intent);
                let result = match result {
                    Ok(IntentResult::Accepted) => {
                        let _outcome = host.persist_owned(now_secs()).await;
                        Ok(IntentResult::Accepted)
                    },
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
                session_item::OBSERVED_EDIT_INTENT => {
                    match serde_json::from_slice::<session_item::ObservedEditV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::OBSERVED_EDIT_SCHEMA => {
                            match host.apply_observed_commands_as(&app, payload.commands) {
                                Ok(()) => {
                                    let _outcome = host.persist_owned(now_secs()).await;
                                    Ok(true)
                                },
                                Err(error) => Err(error.to_string()),
                            }
                        },
                        Ok(_) => Err("unknown observed edit schema".into()),
                        Err(error) => Err(format!("invalid observed edit payload: {error}")),
                    }
                },
                session_item::RESIDENT_SETTINGS_INTENT => {
                    match serde_json::from_slice::<session_item::ResidentSettingsV1>(
                        &intent.payload,
                    ) {
                        Ok(payload) if payload.schema == session_item::RESIDENT_SETTINGS_SCHEMA => {
                            match host.ensure_metadata_idle() {
                                Err(error) => Err(error.to_string()),
                                Ok(()) => {
                                    host.set_resident_pane(
                                        pane.clone(),
                                        graphshell::resident_host::PaneRoots {
                                            roster: payload.roster,
                                            focus: payload.focus,
                                        },
                                    );
                                    match host.set_resident_settings(payload.settings) {
                                        Ok(()) => {
                                            let _outcome = host.persist_owned(now_secs()).await;
                                            Ok(true)
                                        },
                                        Err(error) => Err(format!(
                                            "pane context updated; settings refused: {error}"
                                        )),
                                    }
                                },
                            }
                        },
                        Ok(_) => Err("unknown resident settings schema".into()),
                        Err(error) => Err(format!("invalid resident settings payload: {error}")),
                    }
                },
                session_item::APPLY_EDITS_INTENT => {
                    match serde_json::from_slice::<ApplyEditsV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::APPLY_EDITS_SCHEMA => {
                            match host.apply_edits_as(&app, payload.edits) {
                                Ok(()) => {
                                    let _outcome = host.persist_owned(now_secs()).await;
                                    Ok(true)
                                },
                                Err(error) => Err(error),
                            }
                            .map_err(|error| error.to_string())
                        },
                        Ok(payload) => Err(format!("unknown apply schema {:?}", payload.schema)),
                        Err(error) => Err(format!("invalid apply payload: {error}")),
                    }
                },
                session_item::UNDO_INTENT | session_item::REDO_INTENT => {
                    match serde_json::from_slice::<StepV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::STEP_SCHEMA => {
                            let step = if intent.intent == session_item::UNDO_INTENT {
                                host.undo_now(&app)
                            } else {
                                host.redo_now(&app)
                            };
                            match step {
                                Ok(reverted) => {
                                    if reverted.is_some() {
                                        let _outcome = host.persist_owned(now_secs()).await;
                                    }
                                    Ok(reverted.is_some())
                                },
                                Err(error) => Err(error.to_string()),
                            }
                        },
                        Ok(payload) => Err(format!("unknown step schema {:?}", payload.schema)),
                        Err(error) => Err(format!("invalid step payload: {error}")),
                    }
                },
                session_item::SET_VIEW_INTENT => {
                    match serde_json::from_slice::<SetViewV1>(&intent.payload) {
                        Ok(payload) if payload.schema == session_item::SET_VIEW_SCHEMA => {
                            match host.set_view_now(&app, &payload.view, payload.state) {
                                Ok(()) => {
                                    let _outcome = host.persist_owned(now_secs()).await;
                                    Ok(true)
                                },
                                Err(error) => Err(error.to_string()),
                            }
                        },
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

impl<B: Backend + Clone + Send + Sync + 'static> ProjectionCatalog for MereEndpoint<B> {
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

impl<B: Backend + Clone + Send + Sync + 'static> ProjectionSource for MereEndpoint<B> {
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
                let snapshot = run(async move {
                    let mut host = host.lock().await;
                    host.snapshot(request).map_err(|error| error.to_string())
                })?;
                self.seen_graph = (snapshot.scene.epoch, snapshot.scene.revision);
                Ok(snapshot)
            },
            other => Err(format!("this mere serves no projection {other:?}")),
        }
    }
}

impl<B: Backend + Clone + Send + Sync + 'static> PresentationSource for MereEndpoint<B> {
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
                    .find(|card| ContentHash::of(&card.bytes) == request.resource)
                    .map(|card| card.bytes)
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

impl<B: Backend + Clone + Send + Sync + 'static> IntentSink for MereEndpoint<B> {
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

impl<B: Backend + Clone + Send + Sync + 'static> ProjectionNoticeSource for MereEndpoint<B> {
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
            let mut host = host.lock().await;
            host.sync_projection_context();
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

impl<B: Backend + Clone + Send + Sync + 'static> Drop for MereEndpoint<B> {
    fn drop(&mut self) {
        run(async {
            let mut host = self.host.lock().await;
            host.close_resident_pane(&self.pane);
            // Retain the last coherent reader on I/O failure; desired roots are
            // already released and the next successful refresh reconciles them.
            let _ = host.refresh_resident_reader().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphshell::mere_route;
    use graphshell::native::app_admission::{AllowedAppRoutes, AppHello, AppRouteId};
    use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointSession};
    use kernel::graph::{ContainmentSubKind, EdgeAssertion, SemanticSubKind};
    use mere_view::{MereViewModel, MereViewRequest, SessionStep};
    use pandect::{Change, ChangeKind, mere_dir};
    use std::collections::HashMap;
    use uuid::Uuid;

    use crate::resident_reservoir::{
        EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn admitted_endpoints_have_distinct_pane_roots_and_drop_only_their_own() {
        use graphshell::resident_host::PaneRoots;
        use pandect::addressable_graph::GraphAddress;
        use std::collections::BTreeSet;
        let fixture = fixture(49, "pane-roots").await;
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
        let first = MereEndpoint::open_mode(shared.clone(), &context("turnstone"), false).unwrap();
        let second = MereEndpoint::open_mode(shared.clone(), &context("turnstone"), false).unwrap();
        assert!(Arc::ptr_eq(&first.host, &second.host));
        assert_ne!(first.pane, second.pane);
        assert_eq!(first.pane.app(), "turnstone");
        assert_eq!(second.pane.app(), "turnstone");
        let a = Uuid::from_u128(4950);
        let b = Uuid::from_u128(4951);
        let first_focus = GraphAddress::Resource(Uuid::from_u128(4952));
        let second_focus = GraphAddress::Resource(Uuid::from_u128(4953));
        let pin = GraphAddress::Surface(Uuid::from_u128(4954));
        let host = first.host.clone();
        {
            let mut host = host.lock().await;
            // Both admitted endpoints resolve the same authored projection.
            let roster = BTreeSet::from([a, b]);
            host.set_resident_pane(
                first.pane.clone(),
                PaneRoots {
                    roster: roster.clone(),
                    focus: Some(first_focus),
                },
            );
            host.set_resident_pane(
                second.pane.clone(),
                PaneRoots {
                    roster,
                    focus: Some(second_focus),
                },
            );
            host.set_resident_pins(BTreeSet::from([pin]));
        }
        let first_id = first.pane.clone();
        drop(first);
        {
            let mut host = host.lock().await;
            let roots = host.resident_roots().union();
            assert_eq!(roots.projection_surfaces, BTreeSet::from([a, b]));
            assert_eq!(roots.transient, BTreeSet::from([second_focus]));
            assert_eq!(roots.pinned, BTreeSet::from([pin]));
            let revision = host.resident_roots().revision();
            host.close_resident_pane(&first_id);
            assert_eq!(
                host.resident_roots().revision(),
                revision,
                "drop already released the pane"
            );
        }
        drop(second);
        let host = host.lock().await;
        assert!(host.resident_roots().union().projection_surfaces.is_empty());
        assert!(host.resident_roots().union().transient.is_empty());
        assert_eq!(host.resident_roots().union().pinned, BTreeSet::from([pin]));
        assert_eq!(
            shared.cache.cached_sessions().await,
            1,
            "one cached writer serves both endpoints"
        );
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
        );
        let routes = if let Some(validator) = validator {
            routes.with_domain_validator(domain, validator)
        } else {
            routes
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

    async fn plain_directory_shared(
        root: &std::path::Path,
        validator: Option<pandect::GraphValidator>,
    ) -> Arc<MereShared<muniment::DirectoryBackend>> {
        Arc::new(MereShared {
            domain: "plain-owner".into(),
            validator,
            cache: DirectoryHostCache::open(root.join("mere")).unwrap(),
            fixture_owner: None,
            archive: RedbBackend::open(root.join("archive.redb")).unwrap(),
            persona: persona(73),
            lifecycle: AtomicU64::new(1),
        })
    }

    fn plain_endpoint(
        shared: Arc<MereShared<muniment::DirectoryBackend>>,
        app: &str,
    ) -> AccessEndpoint<muniment::DirectoryBackend> {
        let route =
            ResidentEndpointRoute::new(mere_route_id("plain-owner"), MERE_NOTICE_POLL).unwrap();
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        grants.grant(AppId::new(app), route.clone());
        AccessEndpoint::open(shared, &context(app), grants, route.id(), false).unwrap()
    }

    fn plain_graph(
        endpoint: &mut AccessEndpoint<muniment::DirectoryBackend>,
    ) -> ProjectionSnapshot {
        let request = endpoint
            .describe()
            .projections
            .into_iter()
            .find(|offer| offer.request.session.0 == MERE_GRAPH)
            .unwrap()
            .request;
        endpoint.snapshot(request).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn plain_directory_admitted_endpoints_share_exact_cached_writer_and_lease() {
        let root = tempfile::tempdir().unwrap();
        let shared = plain_directory_shared(root.path(), None).await;
        let mut first = plain_endpoint(shared.clone(), "turnstone");
        let before = shared.cache.all_stored_bytes().await.unwrap();
        let mut second = plain_endpoint(shared.clone(), "knot-editor");
        assert!(
            shared.fixture_owner.is_none(),
            "this control uses the actual production Directory owner"
        );
        assert!(Arc::ptr_eq(&first.endpoint.host, &second.endpoint.host));
        assert_eq!(shared.cache.cached_sessions().await, 1);
        assert_eq!(
            shared.cache.all_stored_bytes().await.unwrap(),
            before,
            "second endpoint selection must not advance a durable epoch"
        );
        assert!(DirectoryHostCache::open(shared.cache.root()).is_err());
        assert!(muniment::DirectoryBackend::open(shared.cache.root()).is_err());
        let snapshot = plain_graph(&mut first);
        assert_eq!(
            first.invoke(apply(&snapshot, vec![node(7301)])).unwrap(),
            IntentResult::Accepted
        );
        let peer = plain_graph(&mut second);
        assert_eq!(nodes(&peer), 1);
        let host = second.endpoint.host.lock().await;
        assert!(!host.has_unstored());
        assert_eq!(
            host.graph().node_count(),
            host.presentation_graph().node_count()
        );
        assert!(host.resident_reader().is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn plain_directory_endpoints_share_validated_writer_after_peer_reconnect() {
        let root = tempfile::tempdir().unwrap();
        let validator: pandect::GraphValidator = Arc::new(|graph| {
            if graph.get_node_by_url("https://7399.test/").is_some() {
                Err("plain edit validation refusal".into())
            } else {
                Ok(())
            }
        });
        let shared = plain_directory_shared(root.path(), Some(validator)).await;
        let mut first = plain_endpoint(shared.clone(), "turnstone");
        let snapshot = plain_graph(&mut first);
        assert_eq!(
            first.invoke(apply(&snapshot, vec![node(7302)])).unwrap(),
            IntentResult::Accepted
        );
        let second = plain_endpoint(shared.clone(), "knot-editor");
        assert!(Arc::ptr_eq(&first.endpoint.host, &second.endpoint.host));
        drop(second);
        let before = shared.cache.all_stored_bytes().await.unwrap();
        let mut reconnected = plain_endpoint(shared.clone(), "knot-editor");
        assert!(Arc::ptr_eq(
            &first.endpoint.host,
            &reconnected.endpoint.host
        ));
        assert_eq!(shared.cache.cached_sessions().await, 1);
        assert_eq!(shared.cache.all_stored_bytes().await.unwrap(), before);
        let peer = plain_graph(&mut reconnected);
        assert_eq!(nodes(&peer), 1);
        assert!(
            matches!(reconnected.invoke(apply(&peer, vec![node(7399)])).unwrap(), IntentResult::Rejected { reason } if reason.contains("plain edit validation refusal"))
        );
        assert_eq!(
            shared.cache.all_stored_bytes().await.unwrap(),
            before,
            "a refused peer edit writes no bytes"
        );
        let host = first.endpoint.host.lock().await;
        assert_eq!(host.graph().node_count(), 1);
        assert!(!host.has_unstored());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn plain_directory_endpoint_owner_validates_before_epoch_and_refuses_unadmitted_access() {
        let root = tempfile::tempdir().unwrap();
        let validator: pandect::GraphValidator =
            Arc::new(|_| Err("plain owner validation refusal".into()));
        let shared = plain_directory_shared(root.path(), Some(validator)).await;
        let id = shared.latest("turnstone").await.unwrap();
        let before = shared.cache.all_stored_bytes().await.unwrap();
        assert!(!before.contains_key(graphshell::mere_host::EPOCH_SLOT));
        let route = mere_route_id("plain-owner");
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        assert!(
            AccessEndpoint::open(
                shared.clone(),
                &context("turnstone"),
                grants.clone(),
                &route,
                false
            )
            .is_err()
        );
        assert_eq!(shared.cache.all_stored_bytes().await.unwrap(), before);
        grants.grant(
            AppId::new("turnstone"),
            ResidentEndpointRoute::new(&route, MERE_NOTICE_POLL).unwrap(),
        );
        let result =
            AccessEndpoint::open(shared.clone(), &context("turnstone"), grants, &route, false);
        assert!(matches!(result, Err(reason) if reason.contains("plain owner validation refusal")));
        assert_eq!(shared.cache.all_stored_bytes().await.unwrap(), before);
        assert_eq!(shared.cache.cached_sessions().await, 0);
        assert_eq!(shared.cache.list().await.unwrap()[0].session_id, id);
    }

    fn plain_session_card(
        endpoint: &mut AccessEndpoint<muniment::DirectoryBackend>,
        snapshot: &ProjectionSnapshot,
    ) -> PortableCardV1 {
        let instance = session_item_of(snapshot);
        let binding = snapshot
            .presentation
            .bindings
            .iter()
            .find(|binding| binding.instance == instance)
            .unwrap();
        let offer = snapshot.presentation.offers[&binding.key]
            .iter()
            .find(|offer| offer.codec == PresentationCodec::PortableCardV1)
            .unwrap();
        let response = endpoint
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: offer.resource,
            })
            .unwrap();
        serde_json::from_slice(&response.bytes).unwrap()
    }

    async fn held_refresh_completion_recovers_peer_publication(failed_load: bool) {
        use graphshell::client::ClientState;
        use graphshell::resident_host::{OBSERVATION_SCHEMA, ObservationRequestV1, REFRESH_INTENT};
        use pandect::addressable_graph::GraphAddress;
        let root = tempfile::tempdir().unwrap();
        let shared = plain_directory_shared(root.path(), None).await;
        let mut first = plain_endpoint(shared.clone(), "turnstone");
        let mut peer = plain_endpoint(shared.clone(), "knot-editor");
        assert!(Arc::ptr_eq(&first.endpoint.host, &peer.endpoint.host));
        assert!(
            shared.fixture_owner.is_none(),
            "actual production plain Directory owner"
        );
        let before = plain_graph(&mut first);
        assert_eq!(
            first.invoke(apply(&before, vec![node(7401)])).unwrap(),
            IntentResult::Accepted
        );
        let peer_before = plain_graph(&mut peer);
        let mut client = ClientState::default();
        client.apply_snapshot(peer_before.clone()).unwrap();
        let owned = first.endpoint.host.clone();
        let request = {
            let mut host = owned.lock().await;
            host.notice_resident_refresh();
            if failed_load {
                host.prepare_resident_demand(GraphAddress::Surface(Uuid::from_u128(7499)))
            } else {
                host.prepare_resident_refresh()
            }
            .unwrap()
            .unwrap()
        };
        // Loading finishes outside the shared host lock, but its completion is
        // held while the actual peer endpoint commits a different command.
        let held = request.load().await;
        assert_eq!(
            held.failed_for_test(),
            failed_load,
            "assert the actual held load result"
        );
        assert!(owned.lock().await.refresh_in_flight_for_test());
        assert_eq!(
            peer.invoke(apply(&peer_before, vec![node(7402)])).unwrap(),
            IntentResult::Accepted
        );
        let pending = plain_graph(&mut peer);
        let card = plain_session_card(&mut peer, &pending);
        assert_eq!(card_value(&card, "Store").as_deref(), Some("stored"));
        assert_eq!(card_value(&card, "Unstored").as_deref(), Some("false"));
        assert_eq!(card_value(&card, "Publication").as_deref(), Some("pending"));
        assert!(
            card_value(&card, "Store error").is_some(),
            "the occupied slot prevents publication"
        );
        let (cursor, changes) = {
            let mut host = owned.lock().await;
            assert_eq!(host.graph().node_count(), 2);
            assert_eq!(host.presentation_graph().node_count(), 1);
            assert!(!host.has_unstored());
            assert!(host.recovery_pending());
            let cursor = host.graph_session().journal().live_cursor();
            let changes = host.graph_session().changes().len();
            let (newest_notice, consumed_notice) = host.refresh_notices_for_test();
            assert!(newest_notice > consumed_notice);
            let _outcome = host.install_resident_refresh(held);
            assert!(
                !host.refresh_in_flight_for_test(),
                "matching held completion must retire its slot even during publication recovery; failed_load={failed_load}"
            );
            assert!(
                host.recovery_pending(),
                "retirement cannot consume the peer's newer publication work"
            );
            let (newest_after, consumed_after) = host.refresh_notices_for_test();
            assert_eq!(
                newest_after, newest_notice,
                "retirement preserves the newer notice"
            );
            assert!(
                newest_after > consumed_after,
                "the peer publication notice remains pending"
            );
            assert_eq!(host.graph_session().journal().live_cursor(), cursor);
            assert_eq!(host.graph_session().changes().len(), changes);
            (cursor, changes)
        };
        let still_pending = plain_graph(&mut peer);
        let card = plain_session_card(&mut peer, &still_pending);
        assert_eq!(card_value(&card, "Publication").as_deref(), Some("pending"));
        assert!(
            matches!(
                peer.invoke(apply(&still_pending, vec![node(7498)]))
                    .unwrap(),
                IntentResult::Rejected { .. }
            ),
            "edits remain paused until peer publication recovers"
        );
        let retry = plain_graph(&mut peer);
        assert_eq!(
            peer.invoke(on_item(
                &retry,
                REFRESH_INTENT,
                serde_json::to_vec(&ObservationRequestV1 {
                    schema: OBSERVATION_SCHEMA.into(),
                    address: None
                })
                .unwrap()
            ))
            .unwrap(),
            IntentResult::Accepted
        );
        {
            let host = owned.lock().await;
            assert!(!host.refresh_in_flight_for_test());
            assert!(!host.recovery_pending());
            assert_eq!(host.presentation_graph().node_count(), 2);
            assert_eq!(
                host.graph_session().journal().live_cursor(),
                cursor,
                "owner refresh cannot replay the peer command"
            );
            assert_eq!(host.graph_session().changes().len(), changes);
        }
        let settled = plain_graph(&mut peer);
        let card = plain_session_card(&mut peer, &settled);
        assert_eq!(card_value(&card, "Store").as_deref(), Some("stored"));
        assert_eq!(card_value(&card, "Unstored").as_deref(), Some("false"));
        assert_eq!(
            card_value(&card, "Publication").as_deref(),
            Some("published")
        );
        assert!(card_value(&card, "Store error").is_none());
        assert!(first.poll_notice().unwrap().is_some());
        client
            .apply_resume(
                &peer_before.session,
                first
                    .resume(client.resume_request(&peer_before.session).unwrap())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            client
                .mounted(&peer_before.session)
                .unwrap()
                .scene
                .active_items_in_order()
                .len()
                - 1,
            2
        );
        assert_eq!(
            peer.invoke(apply(&settled, vec![node(7403)])).unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(nodes(&plain_graph(&mut first)), 3);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cached_endpoint_successful_held_refresh_completion_retires_during_peer_publication() {
        held_refresh_completion_recovers_peer_publication(false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cached_endpoint_failed_held_refresh_completion_retires_during_peer_publication() {
        held_refresh_completion_recovers_peer_publication(true).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn admitted_endpoint_write_after_apply_retries_without_command_replay() {
        use graphshell::resident_host::test_support::DirectoryFault;
        for operation in [
            RecoveryOperation::Apply,
            RecoveryOperation::Undo,
            RecoveryOperation::View,
        ] {
            endpoint_recovers_without_replay(DirectoryFault::AfterApplyError, operation).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn admitted_endpoint_post_commit_reader_failure_retries_only_publication() {
        use graphshell::resident_host::test_support::DirectoryFault;
        for operation in [RecoveryOperation::Apply, RecoveryOperation::Undo] {
            endpoint_recovers_without_replay(DirectoryFault::ReadRecordsAfterApply, operation)
                .await;
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum RecoveryOperation {
        Apply,
        Undo,
        View,
    }

    fn session_card(
        endpoint: &mut ResidentEndpointSession,
        snapshot: &ProjectionSnapshot,
    ) -> PortableCardV1 {
        let instance = session_item_of(snapshot);
        let binding = snapshot
            .presentation
            .bindings
            .iter()
            .find(|b| b.instance == instance)
            .unwrap();
        let offer = snapshot.presentation.offers[&binding.key]
            .iter()
            .find(|offer| offer.codec == PresentationCodec::PortableCardV1)
            .unwrap();
        let response = endpoint
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: offer.resource,
            })
            .unwrap();
        serde_json::from_slice(&response.bytes).unwrap()
    }

    fn card_value(card: &PortableCardV1, label: &str) -> Option<String> {
        card.values
            .iter()
            .find(|value| value.label == label)
            .map(|value| value.value.clone())
    }

    async fn endpoint_recovers_without_replay(
        fault: graphshell::resident_host::test_support::DirectoryFault,
        operation: RecoveryOperation,
    ) {
        use graphshell::client::ClientState;
        use graphshell::resident_host::test_support::DirectoryFault;
        use graphshell::resident_host::{OBSERVATION_SCHEMA, ObservationRequestV1, REFRESH_INTENT};
        use muniment::WriteOp;
        let fixture = fixture(
            72,
            &format!("recovery-{fault:?}-{operation:?}").to_lowercase(),
        )
        .await;
        let mut first = fixture.attach("turnstone").await;
        let mut peer = fixture.attach("knot-editor").await;
        // Two independently journaled changes leave one resident record after undo.
        for id in [7201, 7202] {
            let snapshot = graph(&mut first);
            assert_eq!(
                first.invoke(apply(&snapshot, vec![node(id)])).unwrap(),
                IntentResult::Accepted
            );
        }
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
        let owned = shared
            .host(shared.latest("turnstone").await.unwrap())
            .await
            .unwrap();
        assert_eq!(shared.cache.cached_sessions().await, 1);
        let before = graph(&mut first);
        let peer_before = graph(&mut peer);
        let mut client = ClientState::default();
        client.apply_snapshot(peer_before.clone()).unwrap();
        let (cursor_before, revision_before) = {
            let host = owned.lock().await;
            (
                host.graph_session().journal().live_cursor(),
                host.graph().revision(),
            )
        };
        shared.fixture_owner.as_ref().unwrap().clear_batches();
        shared.fixture_owner.as_ref().unwrap().fault(fault);
        let command = match operation {
            RecoveryOperation::Apply => apply(&before, vec![node(7203)]),
            RecoveryOperation::Undo => undo(&before),
            RecoveryOperation::View => on_item(
                &before,
                session_item::SET_VIEW_INTENT,
                serde_json::to_vec(&SetViewV1::new(
                    "recovery-view",
                    pandect::ViewIntent {
                        focus: Some("https://7201.test/".into()),
                        ..Default::default()
                    },
                ))
                .unwrap(),
            ),
        };
        assert_eq!(
            first.invoke(command).unwrap(),
            IntentResult::Accepted,
            "a storage/publication failure must not reject the already applied {operation:?}"
        );
        if fault == DirectoryFault::ReadRecordsAfterApply {
            let intercepted = shared.fixture_owner.as_ref().unwrap().intercepted_reads();
            assert!(
                intercepted
                    .iter()
                    .any(|key| key.contains("/resident/v1/catalog/")),
                "{operation:?} must encounter an actual post-commit reader catalog fault; intercepted={intercepted:?}"
            );
        }
        let pending = graph(&mut first);
        let card = session_card(&mut first, &pending);
        let uncertain = fault == DirectoryFault::AfterApplyError;
        assert_eq!(
            card_value(&card, "Store").as_deref(),
            Some(if uncertain { "failed" } else { "stored" })
        );
        assert_eq!(
            card_value(&card, "Unstored").as_deref(),
            Some(if uncertain { "true" } else { "false" })
        );
        assert_eq!(card_value(&card, "Publication").as_deref(), Some("pending"));
        assert!(
            card_value(&card, "Store error")
                .unwrap()
                .contains(if uncertain {
                    "fixture lost completion after apply"
                } else {
                    "fixture reader record failure"
                })
        );
        let (cursor, changes, truth) = {
            let host = owned.lock().await;
            assert_eq!(host.has_unstored(), uncertain);
            let expected_nodes = match operation {
                RecoveryOperation::Apply => 3,
                RecoveryOperation::Undo => 1,
                RecoveryOperation::View => 2,
            };
            assert_eq!(
                host.graph().node_count(),
                expected_nodes,
                "the original command already changed its complete writer"
            );
            if matches!(operation, RecoveryOperation::View) {
                assert_eq!(
                    host.graph_session().journal().live_cursor(),
                    cursor_before,
                    "view persistence creates no truth journal entry"
                );
                assert_eq!(host.graph().revision(), revision_before);
                assert_eq!(
                    host.view("turnstone", "recovery-view")
                        .unwrap()
                        .focus
                        .as_deref(),
                    Some("https://7201.test/")
                );
            }
            (
                host.graph_session().journal().live_cursor(),
                host.graph_session().changes().len(),
                host.graph().node_count(),
            )
        };
        let batches = shared.fixture_owner.as_ref().unwrap().batches();
        assert_eq!(
            batches.len(),
            1,
            "one real Directory apply happened before the injected failure"
        );
        shared
            .fixture_owner
            .as_ref()
            .unwrap()
            .fault(DirectoryFault::None);
        // The actual Directory values prove that an error is not evidence of no write.
        for op in &batches[0] {
            match op {
                WriteOp::Put { key, value } => assert_eq!(
                    shared.cache.stored_bytes(key).await.unwrap().as_ref(),
                    Some(value)
                ),
                WriteOp::Delete { key } => {
                    assert_eq!(shared.cache.stored_bytes(key).await.unwrap(), None)
                },
            }
        }
        assert!(
            matches!(
                first.invoke(apply(&pending, vec![node(7299)])).unwrap(),
                IntentResult::Rejected { .. }
            ),
            "new edits remain paused until retained write/publication recovery"
        );
        assert_eq!(owned.lock().await.graph().node_count(), truth);
        let retry = graph(&mut first);
        assert_eq!(
            first
                .invoke(on_item(
                    &retry,
                    REFRESH_INTENT,
                    serde_json::to_vec(&ObservationRequestV1 {
                        schema: OBSERVATION_SCHEMA.into(),
                        address: None,
                    })
                    .unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        assert_eq!(
            shared.fixture_owner.as_ref().unwrap().batches(),
            batches,
            "retry reconciles the exact retained bytes or reloads the reader, without a second apply"
        );
        {
            let host = owned.lock().await;
            assert_eq!(
                host.graph_session().journal().live_cursor(),
                cursor,
                "retry cannot replay edit or undo"
            );
            assert_eq!(host.graph_session().changes().len(), changes);
            assert!(!host.has_unstored());
            assert_eq!(host.presentation_graph().node_count(), truth);
        }
        let settled = graph(&mut first);
        let card = session_card(&mut first, &settled);
        assert_eq!(card_value(&card, "Store").as_deref(), Some("stored"));
        assert_eq!(card_value(&card, "Unstored").as_deref(), Some("false"));
        assert_eq!(
            card_value(&card, "Publication").as_deref(),
            Some("published")
        );
        assert!(card_value(&card, "Store error").is_none_or(|error| error.is_empty()));
        assert!(peer.poll_notice().unwrap().is_some());
        client
            .apply_resume(
                &peer_before.session,
                peer.resume(client.resume_request(&peer_before.session).unwrap())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            client
                .mounted(&peer_before.session)
                .unwrap()
                .scene
                .active_items_in_order()
                .len()
                - 1,
            truth
        );
        assert_eq!(
            first.invoke(apply(&settled, vec![node(7204)])).unwrap(),
            IntentResult::Accepted,
            "editing resumes after recovery"
        );
        assert_eq!(nodes(&graph(&mut first)), truth + 1);
    }

    // Root-run candidate: real broker bytes over a private local socket, using
    // the production Directory facade. Owner seams only seed projection/context.
    #[tokio::test(flavor = "multi_thread")]
    async fn broker_resident_controls_preserve_peer_roots_and_context_only_client_cache() {
        use castellan::authority::PersonaeHost;
        use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
        use chirograph::{CarrierRequestBody, CarrierResponseBody};
        use graphshell::client::{ClientState, PresentationResolution};
        use graphshell::identity::VaultProtectionView;
        use graphshell::native::app_broker::serve_app_broker;
        use graphshell::native::app_client::AppBrokerClient;
        use graphshell::projection_editor::{Channel, ProjectionDraft, Reading, SourceBinding};
        use graphshell::resident_host::{
            DEMAND_INTENT, OBSERVATION_SCHEMA, ObservationRequestV1, REFRESH_INTENT,
            ResidentSettings,
        };
        use graphshell::session_item::{ObservedEditV1, ResidentSettingsV1};
        use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote, PendingLinkRetention};
        use pandect::addressable_graph::{GraphAddress, ResidencyPolicy, ResidencyStatus};
        use pandect::live_graph::{LiveCheck, LiveCommand, ObservedCommand};
        use personae::{Ed25519Keypair, ProfileId};
        use std::collections::BTreeSet;

        async fn invoke(client: &mut AppBrokerClient, intent: IntentInvocation) -> IntentResult {
            match client
                .request_body(CarrierRequestBody::Intent(intent))
                .await
                .unwrap()
            {
                CarrierResponseBody::Intent(result) => result,
                other => panic!("expected an intent response, received {other:?}"),
            }
        }
        async fn resume(
            client: &mut AppBrokerClient,
            state: &mut ClientState,
            session: &ProjectionSession,
        ) {
            let request = state.resume_request(session).unwrap();
            match client
                .request_body(CarrierRequestBody::Resume(request))
                .await
                .unwrap()
            {
                CarrierResponseBody::Resume(reply) => state.apply_resume(session, reply).unwrap(),
                other => panic!("expected a resume response, received {other:?}"),
            };
        }
        async fn hear(
            client: &mut AppBrokerClient,
            session: &ProjectionSession,
            revision: Revision,
        ) {
            // Earlier sessions/archive/graph notices are legal; require this
            // exact new context revision, never accept an old queued bell.
            println!(
                "ENDPOINT CONTROL waiting-context session={} revision={}",
                session.0, revision.0
            );
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    let notice = client.wait_for_notice().await.unwrap();
                    if notice.session == *session && notice.revision == revision {
                        break;
                    }
                }
            })
            .await
            .expect("required context-only broker notice did not arrive");
        }
        fn surface_ids(snapshot: &ProjectionSnapshot) -> BTreeSet<Uuid> {
            snapshot.scene.active_items_in_order().into_iter().filter_map(|(_, item)| {
                let source = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap();
                (source.adapter == "mere.graph").then(|| source.id.parse().expect("portable Surface UUID"))
            }).collect()
        }
        struct AbortBroker(tokio::task::AbortHandle);
        impl Drop for AbortBroker {
            fn drop(&mut self) {
                self.0.abort();
            }
        }
        tokio::time::timeout(Duration::from_secs(30), async {
                let root = tempfile::tempdir().unwrap();
                let shared = plain_directory_shared(root.path(), None).await;
                assert!(shared.fixture_owner.is_none(), "actual production Directory owner");
                let id = shared.latest("turnstone").await.unwrap();
                let owned = shared.host(id).await.unwrap();
                let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
                let grants = AppRouteGrants::new(AllowedAppRoutes::none());
                let route = ResidentEndpointRoute::new(mere_route_id("plain-owner"), MERE_NOTICE_POLL).unwrap();
                for app in ["turnstone", "knot-editor"] { grants.grant(AppId::new(app), route.clone()); }
                let ambient_route = ResidentEndpointRoute::new(ambient_mere_route_id("plain-owner"), MERE_NOTICE_POLL).unwrap();
                grants.grant(AppId::new("knot-editor"), ambient_route.clone());
                let ambient_shared = shared.clone();
                let ambient_grants = grants.clone();
                let ambient_id = ambient_route.id().to_owned();
                catalog.update(|catalog| catalog.register_resumable_notifying(
                    ambient_route.id(), "Ambient endpoint residency control", move |context| {
                        AccessEndpoint::open(ambient_shared.clone(), context, ambient_grants.clone(), &ambient_id, true)
                    },
                )).await.unwrap();
                let opened = shared.clone();
                let admitted = grants.clone();
                let route_id = route.id().to_owned();
                catalog.update(|catalog| catalog.register_resumable_notifying(
                    route.id(), "Endpoint residency control", move |context| {
                        AccessEndpoint::open(opened.clone(), context, admitted.clone(), &route_id, false)
                    },
                )).await.unwrap();
                let identity = Arc::new(PersonaeHost::new(
                    IdentityVault::with_profile(InMemoryStorage::new(), Profile::new(
                        ProfileId("endpoint-controls".into()), "Isolated endpoint controls",
                        Ed25519Keypair::from_seed([0x79; 32]),
                    )), None, VaultProtectionView::Ephemeral,
                ));
                #[cfg(windows)]
                let address = format!(r"\\.\pipe\mere-controls-{}", Uuid::new_v4());
                #[cfg(not(windows))]
                let address = root.path().join("broker.sock").display().to_string();
                let broker_address = address.clone();
                let broker_grants = grants.clone();
                let broker_catalog = catalog.clone();
                let tasks = graphshell::native::tasks::ResidentTasks::new();
                let scoped = tasks.clone();
                let server = tokio::spawn(async move {
                    scoped.scope(serve_app_broker(&broker_address, Arc::new(crate::keeper::Keeper::new(identity)),
                        broker_grants, 60_000, None, broker_catalog)).await
                });
                let abort = AbortBroker(server.abort_handle());
                let mut first = loop {
                    if let Ok(client) = AppBrokerClient::open_route_at(
                        &address, AppId::new("turnstone"), AppRouteId::new(route.id()).unwrap(),
                    ).await { break client; }
                    assert!(!server.is_finished(), "broker startup failed");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                };
                let mut second = AppBrokerClient::open_route_at(
                    &address, AppId::new("knot-editor"), AppRouteId::new(route.id()).unwrap(),
                ).await.unwrap();
                let request = first.open_session().await.unwrap().descriptor.projections
                    .into_iter().find(|offer| offer.request.session.0 == MERE_GRAPH).unwrap().request;
                let peer_request = second.open_session().await.unwrap().descriptor.projections
                    .into_iter().find(|offer| offer.request.session.0 == MERE_GRAPH).unwrap().request;
                let initial = first.snapshot(request.clone()).await.unwrap();
                assert_eq!(initial.scene.epoch, second.snapshot(peer_request.clone()).await.unwrap().scene.epoch);
                assert_eq!(invoke(&mut first, apply(&initial, vec![node(9701), node(9702), node(9703)])).await, IntentResult::Accepted);
                let (a, b, c) = (Uuid::from_u128(9701), Uuid::from_u128(9702), Uuid::from_u128(9703));
                let saved_projection = {
                    let mut host = owned.lock().await;
                    let mut draft = ProjectionDraft::new();
                    draft.id = "broker-scope".into(); draft.label = "Broker scope".into();
                    draft.source = SourceBinding { authority: "local".into(), domain: "notes".into(), resource: "graph:notes".into() };
                    draft.reading = Reading { kind: "nodes".into(), key: "id".into(), value: None };
                    draft.encoding.x = Channel::Field("x".into()); draft.encoding.y = Channel::Field("y".into());
                    draft.appearance.title = "Broker scope".into(); draft.appearance.realization = "canvas".into();
                    draft.provenance.author = "fixture-owner".into(); draft.provenance.source_revision = Some("1".into());
                    let snapshot = graphshell::projection_compile::ProjectionSnapshot {
                        definition: draft.to_definition().unwrap(), selected_occurrence: None,
                    };
                    let saved = host.save_projection(&snapshot).unwrap();
                    host.persist_owned(0).await.unwrap();
                    let (_, node) = host.graph().get_node_by_id(saved).unwrap();
                    assert_eq!(node.url(), graphshell::product::projection_address("broker-scope"));
                    assert_eq!(node.title, "Broker scope");
                    let facets = host.graph().facets().facets_of(&saved).unwrap();
                    let definition = facets.iter().find(|(facet, _)| facet.as_str() == graphshell::product::PROJECTION_DEFINITION_FACET).map(|(_, value)| value).unwrap();
                    assert_eq!(definition, &serde_json::to_value(&snapshot).unwrap(), "saved definition Surface carries the exact authored projection");
                    assert_eq!(host.graph().nodes().map(|(_, node)| node.id).collect::<BTreeSet<_>>(), BTreeSet::from([a, b, c, saved]), "fixture complete writer contains only three seeded Surfaces and saved definition Surface");
                    println!("FULL FIXTURE saved_projection_surface={} address={} title={} complete_writer_ids={:?}", saved, node.url(), node.title, BTreeSet::from([a, b, c, saved]));
                    saved
                };
                let complete_surfaces = BTreeSet::from([a, b, c, saved_projection]);
                assert_eq!(complete_surfaces.len(), 4, "saved definition has a distinct Surface identity");
                let settings = ResidentSettings { policy: ResidencyPolicy::Neighborhood { hops: 0 }, projection: Some("broker-scope".into()), ..Default::default() };
                for (client, id) in [(&mut first, a), (&mut second, b)] {
                    let snapshot = client.snapshot(request.clone()).await.unwrap();
                    let payload = ResidentSettingsV1 { schema: session_item::RESIDENT_SETTINGS_SCHEMA.into(), settings: settings.clone(), roster: BTreeSet::from([id]), focus: Some(GraphAddress::Surface(id)) };
                    assert_eq!(invoke(client, on_item(&snapshot, session_item::RESIDENT_SETTINGS_INTENT, serde_json::to_vec(&payload).unwrap())).await, IntentResult::Accepted);
                }
                {
                    let host = owned.lock().await;
                    assert_eq!(host.resident_roots().union().projection_surfaces, BTreeSet::from([a, b]));
                    assert_eq!(host.resident_roots().union().transient, BTreeSet::from([GraphAddress::Surface(a), GraphAddress::Surface(b)]));
                    assert_eq!(host.resident_reader().unwrap().status(GraphAddress::Surface(c)), ResidencyStatus::NotLoaded);
                }
                // The ambient connection cannot execute the new partial settings action.
                let mut ambient = AppBrokerClient::open_route_at(
                    &address, AppId::new("knot-editor"), AppRouteId::new(ambient_route.id()).unwrap(),
                ).await.unwrap();
                let ambient_request = ambient.open_session().await.unwrap().descriptor.projections
                    .into_iter().find(|offer| offer.request.session.0 == MERE_GRAPH).unwrap().request;
                let ambient_snapshot = ambient.snapshot(ambient_request).await.unwrap();
                assert!(ambient_snapshot.presentation.offers.values().flatten().all(|offer| offer.semantics.actions.is_empty()));
                let bytes_before = shared.cache.all_stored_bytes().await.unwrap();
                let ambient_payload = ResidentSettingsV1 { schema: session_item::RESIDENT_SETTINGS_SCHEMA.into(), settings: ResidentSettings::default(), roster: BTreeSet::new(), focus: None };
                assert!(matches!(invoke(&mut ambient, on_item(&ambient_snapshot, session_item::RESIDENT_SETTINGS_INTENT, serde_json::to_vec(&ambient_payload).unwrap())).await,
                    IntentResult::Rejected { reason } if reason.contains("read-only")));
                assert_eq!(shared.cache.all_stored_bytes().await.unwrap(), bytes_before);
                ambient.close().await.unwrap();
                assert_eq!(shared.cache.cached_sessions().await, 1);
                assert!(muniment::DirectoryBackend::open(shared.cache.root()).is_err());
                for (client, app, pinned) in [(&mut first, "turnstone", true), (&mut second, "knot-editor", false)] {
                    let snapshot = client.snapshot(request.clone()).await.unwrap();
                    let (digest, count) = {
                        let host = owned.lock().await;
                        (host.resident_reader().unwrap().record_digest(GraphAddress::Surface(b)).unwrap().to_owned(), host.graph_session().changes().len())
                    };
                    let payload = ObservedEditV1 { schema: session_item::OBSERVED_EDIT_SCHEMA.into(), commands: vec![ObservedCommand {
                        command: LiveCommand::SetPinned { surface: b, pinned },
                        checks: vec![LiveCheck::RecordUnchanged { address: GraphAddress::Surface(b), digest }],
                    }] };
                    assert_eq!(invoke(client, on_item(&snapshot, session_item::OBSERVED_EDIT_INTENT, serde_json::to_vec(&payload).unwrap())).await, IntentResult::Accepted);
                    let host = owned.lock().await;
                    assert_eq!(host.graph_session().changes().len(), count + 1);
                    assert_eq!(host.graph_session().changes().last().unwrap().author, shared.author(app));
                    assert!(!host.has_unstored());
                }
                // Close is a carrier request. The broker must drop only that pane.
                first.close().await.unwrap();
                loop {
                    let host = owned.lock().await;
                    let roots = host.resident_roots().union();
                    if roots.projection_surfaces == BTreeSet::from([b]) {
                        assert_eq!(roots.transient, BTreeSet::from([GraphAddress::Surface(b)]));
                        assert_eq!(host.resident_reader().unwrap().status(GraphAddress::Surface(a)), ResidencyStatus::NotLoaded);
                        break;
                    }
                    drop(host); tokio::time::sleep(Duration::from_millis(10)).await;
                }
                let peer = second.snapshot(peer_request.clone()).await.unwrap();
                assert_eq!(surface_ids(&peer), BTreeSet::from([b]), "only the remaining pane Surface is presented before any demand");
                assert_eq!(nodes(&peer), 1);
                let mut state = ClientState::default(); state.apply_snapshot(peer.clone()).unwrap();
                let binding = peer.presentation.bindings.iter().find(|binding| peer.presentation.offers[&binding.key]
                    .iter().any(|offer| offer.codec == PresentationCodec::PortableCardV1)).unwrap();
                let instance = binding.instance;
                let resource = peer.presentation.offers[&binding.key].iter().find(|offer| offer.codec == PresentationCodec::PortableCardV1).unwrap().resource;
                match second.request_body(CarrierRequestBody::Resource(ResourceRequest { session: peer.session.clone(), resource })).await.unwrap() {
                    CarrierResponseBody::Resource(response) => state.apply_resource(response).unwrap(),
                    other => panic!("expected resource bytes, received {other:?}"),
                }
                let before = state.mounted(&peer.session).unwrap().clone();
                let (truth, cursor, writer, new_revision) = {
                    let mut host = owned.lock().await;
                    let truth = host.presentation_graph().revision();
                    let cursor = host.graph_session().journal().live_cursor();
                    let writer = host.graph_session().revision();
                    let mut limit = CoverageLimit::new(CoverageLayer::Disclosure, "broker context control"); limit.resources.push(c);
                    host.set_resident_host_coverage(CoverageNote { limits: vec![limit] }).unwrap();
                    host.sync_projection_context();
                    assert_eq!(host.presentation_graph().revision(), truth);
                    (truth, cursor, writer, host.projection_revision())
                };
                assert!(new_revision > peer.scene.revision, "context advances delivery revision");
                hear(&mut second, &peer.session, new_revision).await;
                state.mark_stale(&peer.session); resume(&mut second, &mut state, &peer.session).await;
                let mounted = state.mounted(&peer.session).unwrap();
                assert_eq!(mounted.scene.tables, before.scene.tables); assert_eq!(mounted.presentation, before.presentation);
                assert_eq!(mounted.status, chirograph::SessionStatus::Live);
                assert!(mounted.coverage.as_ref().unwrap().limits.iter().any(|limit| limit.layer == chirograph::PortableCoverageLayerV1::Disclosure));
                assert!(matches!(state.resolve(&peer.session, instance, &chirograph::CapabilityProfile::new([PresentationCapability::PortableCard])).unwrap(), PresentationResolution::Ready(_)));
                let previous = state.mounted(&peer.session).unwrap().clone();
                {
                    let host = owned.lock().await;
                    host.publish_pending_retention_for_test(PendingLinkRetention::UntilPurged).await.unwrap();
                }
                let current = second.snapshot(peer_request.clone()).await.unwrap();
                let refresh = ObservationRequestV1 { schema: OBSERVATION_SCHEMA.into(), address: None };
                assert_eq!(invoke(&mut second, on_item(&current, REFRESH_INTENT, serde_json::to_vec(&refresh).unwrap())).await, IntentResult::Accepted);
                let pending_revision = owned.lock().await.projection_revision();
                assert!(pending_revision > new_revision, "equal-coverage Pending context advances delivery revision");
                hear(&mut second, &peer.session, pending_revision).await;
                resume(&mut second, &mut state, &peer.session).await;
                assert_eq!(state.mounted(&peer.session).unwrap().scene.tables, previous.scene.tables);
                assert_eq!(state.mounted(&peer.session).unwrap().coverage, previous.coverage);
                {
                    let host = owned.lock().await;
                    assert_eq!(host.presentation_graph().revision(), truth);
                    assert_eq!(host.graph_session().journal().live_cursor(), cursor);
                    assert_eq!(host.graph_session().revision(), writer);
                    assert_eq!(host.presentation_graph().pending_link_state().retention, PendingLinkRetention::UntilPurged);
                }
                // Exact empty roster stays empty; Full clears computed Residency limits.
            for (configuration, expected_nodes) in [(settings.clone(), 0), (ResidentSettings::default(), complete_surfaces.len())] {
                let snapshot = second.snapshot(peer_request.clone()).await.unwrap();
                let payload = ResidentSettingsV1 { schema: session_item::RESIDENT_SETTINGS_SCHEMA.into(), settings: configuration, roster: BTreeSet::new(), focus: None };
                let encoded = serde_json::to_vec(&payload).unwrap();
                let decoded: ResidentSettingsV1 = serde_json::from_slice(&encoded).unwrap();
                assert!(decoded.roster.is_empty(), "empty roster survives public payload serialization");
                assert_eq!(decoded.focus, None, "None is a replacement focus, not retain-old-focus");
                let empty_bytes_before = if expected_nodes == 0 { Some(shared.cache.all_stored_bytes().await.unwrap()) } else { None };
                let empty_before = if expected_nodes == 0 {
                    let host = owned.lock().await;
                    assert_eq!(decoded.settings, settings, "empty control sends the same public policy/projection settings");
                    assert_eq!(host.resident_reader().unwrap().status(GraphAddress::Surface(b)), ResidencyStatus::Loaded, "old reader b is the pre-control witness");
                    assert!(!host.has_unstored());
                    assert!(!host.recovery_pending());
                    println!("EMPTY SCOPE before roots_revision={} roots={:?} generation={} policy={:?} b_status={:?} unstored={} recovery={}",
                        host.resident_roots().revision(), host.resident_roots().union(), host.resident_reader().unwrap().generation(), host.resident_reader().unwrap().policy(),
                        host.resident_reader().unwrap().status(GraphAddress::Surface(b)), host.has_unstored(), host.recovery_pending());
                    Some((host.resident_roots().revision(), host.graph().revision(), host.graph_session().journal().live_cursor(), host.graph_session().revision(), host.resident_reader().unwrap().generation().to_string()))
                } else { None };
                assert_eq!(invoke(&mut second, on_item(&snapshot, session_item::RESIDENT_SETTINGS_INTENT, encoded)).await, IntentResult::Accepted);
                if expected_nodes == 0 {
                    let host = owned.lock().await;
                    let roots = host.resident_roots().union();
                    assert!(roots.projection_surfaces.is_empty(), "public setting replaced authored roster");
                    assert!(roots.transient.is_empty(), "public focus None cleared pane focus");
                    assert!(roots.pinned.is_empty(), "no independent host pins");
                    assert_eq!(roots.focused, None, "no separate host focus");
                    let key = host.graph().get_node_key_by_id(b).unwrap();
                    assert_eq!(host.graph().node_is_pinned(key), Some(false), "second observed unpin cleared the complete graph pin");
                    assert!(!host.has_unstored(), "root-only settings did not stage writer edits");
                    assert!(!host.recovery_pending(), "no held storage or publication recovery");
                    let before = empty_before.as_ref().unwrap();
                    assert!(host.resident_roots().revision() > before.0, "accepted empty roots advance the pane roots revision");
                    assert_eq!(host.graph().revision(), before.1, "empty control does not mutate writer graph");
                    assert_eq!(host.graph_session().journal().live_cursor(), before.2, "empty control does not append a writer command");
                    assert_eq!(host.graph_session().revision(), before.3, "empty control does not advance writer revision");
                    let reader = host.resident_reader().unwrap();
                    assert_eq!(reader.generation().to_string(), before.4, "empty control keeps the stored graph generation");
                    assert_eq!(reader.policy(), ResidencyPolicy::Neighborhood { hops: 0 });
                    let reader_pin = reader.graph().get_node_key_by_id(b).and_then(|key| reader.graph().node_is_pinned(key));
                    println!("EMPTY SCOPE after roots_revision={} roots={:?} generation={} policy={:?} a_status={:?} b_status={:?} c_status={:?} writer_pin=false reader_pin={:?}",
                        host.resident_roots().revision(), roots, reader.generation(), reader.policy(), reader.status(GraphAddress::Surface(a)),
                        reader.status(GraphAddress::Surface(b)), reader.status(GraphAddress::Surface(c)), reader_pin);
                }
                if let Some(before) = empty_bytes_before { assert_eq!(shared.cache.all_stored_bytes().await.unwrap(), before, "root-only empty settings write no graph/settings bytes"); }
                let changed = second.snapshot(peer_request.clone()).await.unwrap();
                assert_eq!(surface_ids(&changed), if expected_nodes == 0 { BTreeSet::new() } else { complete_surfaces.clone() });
                assert_eq!(nodes(&changed), expected_nodes);
                if expected_nodes == complete_surfaces.len() {
                    let host = owned.lock().await;
                    assert_eq!(host.graph().nodes().map(|(_, node)| node.id).collect::<BTreeSet<_>>(), complete_surfaces, "Full renders the exact known complete writer Surface set");
                    assert_eq!(host.resident_reader().unwrap().status(GraphAddress::Surface(saved_projection)), ResidencyStatus::Loaded, "Full includes persisted definition Surface");
                    assert!(changed.coverage.as_ref().unwrap().limits.iter().all(|limit| limit.layer != chirograph::PortableCoverageLayerV1::Residency));
                }
            }
            // Demand retention is reader-wide and survives ordinary refresh.
            // Exercise it after the strict pane-close and empty-roster controls,
            // so their fixture contains no independent demand root.
            let snapshot = second.snapshot(peer_request.clone()).await.unwrap();
            let payload = ResidentSettingsV1 { schema: session_item::RESIDENT_SETTINGS_SCHEMA.into(), settings: settings.clone(), roster: BTreeSet::from([a, b]), focus: Some(GraphAddress::Surface(b)) };
            assert_eq!(invoke(&mut second, on_item(&snapshot, session_item::RESIDENT_SETTINGS_INTENT, serde_json::to_vec(&payload).unwrap())).await, IntentResult::Accepted);
            assert_eq!(owned.lock().await.resident_reader().unwrap().status(GraphAddress::Surface(c)), ResidencyStatus::NotLoaded);
            let mut demand_peer = AppBrokerClient::open_route_at(
                &address, AppId::new("turnstone"), AppRouteId::new(route.id()).unwrap(),
            ).await.unwrap();
            let demand_peer_request = demand_peer.open_session().await.unwrap().descriptor.projections
                .into_iter().find(|offer| offer.request.session.0 == MERE_GRAPH).unwrap().request;
            assert_eq!(surface_ids(&demand_peer.snapshot(demand_peer_request.clone()).await.unwrap()), BTreeSet::from([a, b]), "authored roster explicitly restored before the separate demand control");
            let scoped = second.snapshot(peer_request.clone()).await.unwrap();
            let demand = ObservationRequestV1 { schema: OBSERVATION_SCHEMA.into(), address: Some(GraphAddress::Surface(c)) };
            assert_eq!(invoke(&mut second, on_item(&scoped, DEMAND_INTENT, serde_json::to_vec(&demand).unwrap())).await, IntentResult::Accepted);
            let demand_revision = { let mut host = owned.lock().await; host.sync_projection_context(); host.projection_revision() };
            hear(&mut demand_peer, &scoped.session, demand_revision).await;
            let demanded = demand_peer.snapshot(demand_peer_request).await.unwrap();
            assert_eq!(surface_ids(&demanded), BTreeSet::from([a, b, c]), "peer sees exactly restored authored roster plus explicitly demanded Surface c");
            assert_eq!(nodes(&demanded), 3);
            {
                let host = owned.lock().await;
                assert_eq!(host.resident_roots().union().projection_surfaces, BTreeSet::from([a, b]));
                assert_eq!(host.resident_roots().union().transient, BTreeSet::from([GraphAddress::Surface(b)]));
                assert_eq!(host.resident_reader().unwrap().status(GraphAddress::Surface(c)), ResidencyStatus::Loaded);
            }
            demand_peer.close().await.unwrap();
            second.close().await.unwrap();
                loop {
                    let host = owned.lock().await;
                    if host.resident_roots().union().projection_surfaces.is_empty() {
                        assert!(host.resident_roots().union().transient.is_empty()); break;
                    }
                    drop(host); tokio::time::sleep(Duration::from_millis(10)).await;
                }
                // Resident shutdown must release a still-live endpoint, not only
                // an already closed one. Use the same tracked task scope as Djinn.
                let mut shutdown_client = AppBrokerClient::open_route_at(
                    &address, AppId::new("turnstone"), AppRouteId::new(route.id()).unwrap(),
                ).await.unwrap();
                let shutdown_request = shutdown_client.open_session().await.unwrap().descriptor.projections
                    .into_iter().find(|offer| offer.request.session.0 == MERE_GRAPH).unwrap().request;
                let snapshot = shutdown_client.snapshot(shutdown_request).await.unwrap();
                let payload = ResidentSettingsV1 { schema: session_item::RESIDENT_SETTINGS_SCHEMA.into(), settings, roster: BTreeSet::from([a]), focus: Some(GraphAddress::Surface(a)) };
                assert_eq!(invoke(&mut shutdown_client, on_item(&snapshot, session_item::RESIDENT_SETTINGS_INTENT, serde_json::to_vec(&payload).unwrap())).await, IntentResult::Accepted);
                let shutdown_cursor = {
                    let host = owned.lock().await;
                    assert_eq!(host.resident_roots().union().projection_surfaces, BTreeSet::from([a]));
                    host.graph_session().journal().live_cursor()
                };
                server.abort(); let _ = server.await;
                assert!(tasks.cancel_and_join().await > 0, "live broker connection was tracked");
                {
                    let host = owned.lock().await;
                    assert!(host.resident_roots().union().projection_surfaces.is_empty(), "shutdown released live pane roster");
                    assert!(host.resident_roots().union().transient.is_empty(), "shutdown released live pane focus");
                    assert_eq!(host.graph_session().journal().live_cursor(), shutdown_cursor);
                }
                drop(shutdown_client); drop(abort);
            }).await.expect("socket control completes within its explicit deadline");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn admitted_observed_edits_and_settings_share_writer_and_preserve_peer_roots() {
        use graphshell::resident_host::ResidentSettings;
        use graphshell::session_item::{ObservedEditV1, ResidentSettingsV1};
        use pandect::addressable_graph::GraphAddress;
        use pandect::live_graph::{LiveCheck, LiveCommand, ObservedCommand};
        use std::collections::BTreeSet;
        let fixture = fixture(76, "observed-product-route").await;
        let mut first = fixture.attach("turnstone").await;
        let mut second = fixture.attach("knot-editor").await;
        let initial = graph(&mut first);
        assert_eq!(
            first
                .invoke(apply(&initial, vec![node(7601), node(7602)]))
                .unwrap(),
            IntentResult::Accepted
        );
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
        let owner = MereEndpoint::open_mode(shared.clone(), &context("turnstone"), false).unwrap();
        let owned = owner.host.clone();
        let a = Uuid::from_u128(7601);
        let b = Uuid::from_u128(7602);
        for (endpoint, id) in [(&mut first, a), (&mut second, b)] {
            let before = graph(endpoint);
            let payload = ResidentSettingsV1 {
                schema: session_item::RESIDENT_SETTINGS_SCHEMA.into(),
                settings: ResidentSettings::default(),
                roster: BTreeSet::from([id]),
                focus: None,
            };
            assert_eq!(
                endpoint
                    .invoke(on_item(
                        &before,
                        session_item::RESIDENT_SETTINGS_INTENT,
                        serde_json::to_vec(&payload).unwrap()
                    ))
                    .unwrap(),
                IntentResult::Accepted
            );
        }
        {
            let host = owned.lock().await;
            assert_eq!(
                host.resident_roots().union().projection_surfaces,
                BTreeSet::from([a, b])
            );
        }
        assert_eq!(shared.cache.cached_sessions().await, 1);
        let before = graph(&mut first);
        let (digest, cursor, changes) = {
            let host = owned.lock().await;
            (
                host.resident_reader()
                    .unwrap()
                    .record_digest(GraphAddress::Surface(a))
                    .unwrap()
                    .to_owned(),
                host.graph_session().journal().live_cursor(),
                host.graph_session().changes().len(),
            )
        };
        let payload = ObservedEditV1 {
            schema: session_item::OBSERVED_EDIT_SCHEMA.into(),
            commands: vec![ObservedCommand {
                command: LiveCommand::SetPinned {
                    surface: a,
                    pinned: true,
                },
                checks: vec![LiveCheck::RecordUnchanged {
                    address: GraphAddress::Surface(a),
                    digest,
                }],
            }],
        };
        let mut invalid = serde_json::to_value(&payload).unwrap();
        invalid["commands"][0]["command"]["SetPinned"]["surface"] = serde_json::json!("not-a-uuid");
        assert!(matches!(
            first
                .invoke(on_item(
                    &before,
                    session_item::OBSERVED_EDIT_INTENT,
                    serde_json::to_vec(&invalid).unwrap()
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        let mut forged = serde_json::to_value(&payload).unwrap();
        forged["author"] = serde_json::json!("someone-else");
        assert!(matches!(
            first
                .invoke(on_item(
                    &before,
                    session_item::OBSERVED_EDIT_INTENT,
                    serde_json::to_vec(&forged).unwrap()
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            owned.lock().await.graph_session().journal().live_cursor(),
            cursor
        );
        assert_eq!(
            first
                .invoke(on_item(
                    &before,
                    session_item::OBSERVED_EDIT_INTENT,
                    serde_json::to_vec(&payload).unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        {
            let host = owned.lock().await;
            assert_eq!(host.graph_session().changes().len(), changes + 1);
            assert_eq!(
                host.graph_session()
                    .changes()
                    .last()
                    .unwrap()
                    .author
                    .via
                    .as_deref(),
                Some("turnstone")
            );
            assert!(!host.has_unstored());
            let key = host.presentation_graph().get_node_key_by_id(a).unwrap();
            assert_eq!(host.presentation_graph().node_is_pinned(key), Some(true));
        }
        assert!(second.poll_notice().unwrap().is_some());
        let stale = graph(&mut first);
        assert!(matches!(
            first
                .invoke(on_item(
                    &stale,
                    session_item::OBSERVED_EDIT_INTENT,
                    serde_json::to_vec(&payload).unwrap()
                ))
                .unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            owned.lock().await.graph_session().changes().len(),
            changes + 1
        );
        // Closing one real routed endpoint releases only that endpoint's roots.
        drop(first);
        assert_eq!(
            owned
                .lock()
                .await
                .resident_roots()
                .union()
                .projection_surfaces,
            BTreeSet::from([b])
        );
        drop(second);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn scoped_endpoint_coverage_demand_and_context_resume_retain_client_resources() {
        use graphshell::client::ClientState;
        use graphshell::projection_editor::{Channel, ProjectionDraft, Reading, SourceBinding};
        use graphshell::resident_host::{
            DEMAND_INTENT, OBSERVATION_SCHEMA, ObservationRequestV1, PaneRoots, REFRESH_INTENT,
            ResidentSettings,
        };
        use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote, PendingLinkRetention};
        use pandect::addressable_graph::{GraphAddress, ResidencyPolicy, ResidencyStatus};
        use std::collections::BTreeSet;
        let fixture = fixture(61, "portable-coverage").await;
        let mut first = fixture.attach("turnstone").await;
        let mut second = fixture.attach("knot-editor").await;
        let initial = graph(&mut first);
        let a = Uuid::from_u128(6101);
        let b = Uuid::from_u128(6102);
        assert_eq!(
            first
                .invoke(apply(&initial, vec![node(6101), node(6102)]))
                .unwrap(),
            IntentResult::Accepted
        );
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
        let mut first_owner =
            MereEndpoint::open_mode(shared.clone(), &context("turnstone"), false).unwrap();
        let second_owner =
            MereEndpoint::open_mode(shared.clone(), &context("knot-editor"), false).unwrap();
        assert!(Arc::ptr_eq(&first_owner.host, &second_owner.host));
        assert_eq!(shared.cache.cached_sessions().await, 1);
        let owned = first_owner.host.clone();
        {
            let mut host = owned.lock().await;
            assert!(shared.cache.root().is_dir());
            let mut draft = ProjectionDraft::new();
            draft.id = "coverage-scope".into();
            draft.label = "Coverage scope".into();
            draft.source = SourceBinding {
                authority: "local".into(),
                domain: "notes".into(),
                resource: "graph:notes".into(),
            };
            draft.reading = Reading {
                kind: "nodes".into(),
                key: "id".into(),
                value: None,
            };
            draft.encoding.x = Channel::Field("x".into());
            draft.encoding.y = Channel::Field("y".into());
            draft.appearance.title = "Coverage scope".into();
            draft.appearance.realization = "canvas".into();
            draft.provenance.author = "fixture-owner".into();
            draft.provenance.source_revision = Some("1".into());
            host.save_projection(&graphshell::projection_compile::ProjectionSnapshot {
                definition: draft.to_definition().unwrap(),
                selected_occurrence: None,
            })
            .unwrap();
            host.persist_owned(0).await.unwrap();
            host.set_resident_pane(
                first_owner.pane.clone(),
                PaneRoots {
                    roster: BTreeSet::from([a]),
                    focus: None,
                },
            );
            host.set_resident_settings(ResidentSettings {
                policy: ResidencyPolicy::Neighborhood { hops: 0 },
                projection: Some("coverage-scope".into()),
                ..Default::default()
            })
            .unwrap();
            host.persist_owned(0).await.unwrap();
            assert_eq!(
                host.resident_reader()
                    .unwrap()
                    .status(GraphAddress::Surface(b)),
                ResidencyStatus::NotLoaded
            );
            assert!(host.presentation_graph().node_count() < host.graph().node_count());
        }
        let scoped = graph(&mut first);
        let peer = graph(&mut second);
        assert_eq!(nodes(&scoped), 1);
        assert_eq!(scoped.scene, peer.scene);
        let limits = &scoped.coverage.as_ref().unwrap().limits;
        assert!(limits.iter().any(|limit| limit.layer
            == chirograph::PortableCoverageLayerV1::Residency
            && limit.surfaces.contains(&b.to_string())));
        assert!(scoped.presentation.offers.values().flatten().any(|offer| {
            offer
                .semantics
                .actions
                .iter()
                .any(|action| action.intent.0 == DEMAND_INTENT)
        }));
        let mut client = ClientState::default();
        client.apply_snapshot(peer.clone()).unwrap();
        let offer = peer.presentation.offers.values().flatten().next().unwrap();
        let resource = second
            .resource(ResourceRequest {
                session: peer.session.clone(),
                resource: offer.resource,
            })
            .unwrap();
        client.apply_resource(resource).unwrap();
        let cached = offer.resource;
        let demand = ObservationRequestV1 {
            schema: OBSERVATION_SCHEMA.into(),
            address: Some(GraphAddress::Surface(b)),
        };
        let malformed = on_item(
            &scoped,
            DEMAND_INTENT,
            br#"{"schema":"mere.resident.observation/v1","address":{"Surface":"bad-uuid"}}"#
                .to_vec(),
        );
        assert!(matches!(
            first.invoke(malformed).unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            first
                .invoke(on_item(
                    &scoped,
                    DEMAND_INTENT,
                    serde_json::to_vec(&demand).unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        let bell = second
            .poll_notice()
            .unwrap()
            .expect("demand updates peer presentation");
        assert_eq!(bell.session, peer.session);
        client
            .apply_resume(
                &peer.session,
                second
                    .resume(client.resume_request(&peer.session).unwrap())
                    .unwrap(),
            )
            .unwrap();
        let loaded = graph(&mut first);
        assert_eq!(nodes(&loaded), 2);
        let before = client.mounted(&peer.session).unwrap().clone();
        let (truth, journal, writer_revision) = {
            let mut host = owned.lock().await;
            assert_eq!(
                host.resident_reader()
                    .unwrap()
                    .status(GraphAddress::Surface(b)),
                ResidencyStatus::Loaded
            );
            let truth = host.presentation_graph().revision();
            let journal = host.graph_session().journal().live_cursor();
            let writer_revision = host.graph_session().revision();
            let mut limit =
                CoverageLimit::new(CoverageLayer::Disclosure, "private material withheld");
            limit.resources.push(b);
            host.set_resident_host_coverage(CoverageNote {
                limits: vec![limit],
            })
            .unwrap();
            assert_eq!(host.presentation_graph().revision(), truth);
            (truth, journal, writer_revision)
        };
        assert!(second.poll_notice().unwrap().is_some());
        client.mark_stale(&peer.session);
        client
            .apply_resume(
                &peer.session,
                second
                    .resume(client.resume_request(&peer.session).unwrap())
                    .unwrap(),
            )
            .unwrap();
        let context = client.mounted(&peer.session).unwrap();
        assert_eq!(context.scene.tables, before.scene.tables);
        assert_eq!(context.presentation, before.presentation);
        assert_eq!(context.status, chirograph::SessionStatus::Live);
        assert!(
            context
                .coverage
                .as_ref()
                .unwrap()
                .limits
                .iter()
                .any(|limit| limit.layer == chirograph::PortableCoverageLayerV1::Disclosure)
        );
        assert!(matches!(
            client
                .resolve(
                    &peer.session,
                    peer.presentation
                        .bindings
                        .iter()
                        .find(|binding| peer.presentation.offers[&binding.key]
                            .iter()
                            .any(|offer| offer.resource == cached))
                        .unwrap()
                        .instance,
                    &chirograph::CapabilityProfile::new([PresentationCapability::PortableCard])
                )
                .unwrap(),
            graphshell::client::PresentationResolution::Ready(_)
        ));
        // Publish only retained Pending context through the same owner's store.
        // Coverage and available truth stay equal, but the observation must ring.
        let current = graph(&mut first);
        {
            let host = owned.lock().await;
            host.publish_pending_retention_for_test(PendingLinkRetention::UntilPurged)
                .await
                .unwrap();
        }
        let refresh = ObservationRequestV1 {
            schema: OBSERVATION_SCHEMA.into(),
            address: None,
        };
        assert_eq!(
            first
                .invoke(on_item(
                    &current,
                    REFRESH_INTENT,
                    serde_json::to_vec(&refresh).unwrap()
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        assert!(
            second.poll_notice().unwrap().is_some(),
            "Pending-only observation rings despite equal coverage"
        );
        let previous = client.mounted(&peer.session).unwrap().clone();
        client
            .apply_resume(
                &peer.session,
                second
                    .resume(client.resume_request(&peer.session).unwrap())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            client.mounted(&peer.session).unwrap().scene.tables,
            previous.scene.tables
        );
        assert_eq!(
            client.mounted(&peer.session).unwrap().coverage,
            previous.coverage
        );
        let host = owned.lock().await;
        assert_eq!(host.presentation_graph().revision(), truth);
        assert_eq!(host.graph_session().journal().live_cursor(), journal);
        assert_eq!(host.graph_session().revision(), writer_revision);
        assert_eq!(
            host.presentation_graph().pending_link_state().retention,
            PendingLinkRetention::UntilPurged
        );
        drop(host);
        // Attach/switch resets endpoint seen state without depending on epoch inequality.
        first_owner.attach_host(first_owner.attached, first_owner.host.clone());
        assert_eq!(first_owner.seen_graph, (SceneEpoch(0), Revision(0)));
        assert!(first_owner.poll_notice().unwrap().is_some());
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
            decay_progress: None,
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
        // that session's graph. Three relations share one pair and stay
        // apart, each by its family (§7 item 43).
        let first = read_route(&mut knot);
        assert_eq!(first.model.title, "notes");
        assert!(first.model.can_mint);
        let [parent] = first.model.sessions.as_slice() else {
            panic!("{:?}", first.model.sessions)
        };
        assert!(parent.attached && !parent.trashed);
        assert_eq!(parent.steps, [SessionStep::Fork, SessionStep::Trash]);
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
        assert_eq!(kinds, ["containment", "semantic", "semantic"]);
        let mut keys: Vec<&str> = relations
            .iter()
            .map(|relation| relation.key.as_str())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 3, "each relation keeps its own key");

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
        assert_eq!(switched.model.graph.relations.len(), 3);

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
        assert!(
            read.presentation
                .offers
                .values()
                .flatten()
                .all(|offer| offer.semantics.actions.is_empty())
        );
        for intent in [
            graphshell::resident_host::DEMAND_INTENT,
            graphshell::resident_host::REFRESH_INTENT,
            MINT_SESSION_INTENT,
            FORK_SESSION_INTENT,
            TRASH_SESSION_INTENT,
            session_item::SET_VIEW_INTENT,
            session_item::APPLY_EDITS_INTENT,
            session_item::OBSERVED_EDIT_INTENT,
            session_item::RESIDENT_SETTINGS_INTENT,
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
        let sessions = MereEndpoint::<DirectoryBackend>::sessions_request();
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
}
