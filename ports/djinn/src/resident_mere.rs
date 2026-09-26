// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's own route inside Djinn (reservoir plan V2, step 5).
//!
//! Each mere the reservoir holds is served on `mere/<domain>` (§7 items 20 and
//! 30): registered when the lane opens and whenever a mere is ensured, and
//! granted to the first-party applications the reservoir route is. An admitted
//! session attaches to one of the mere's sessions, the live one changed last
//! unless it asks for another (§7 item 24), and sees two projections: the
//! mere's sessions, with the lifecycle as intents that name their session by
//! id (§7 item 33) in `graphshell::session_item`'s vocabulary (§7 item 37),
//! and the attached session's graph, served through `MereHost` with its
//! session item (§7 items 21, 22 and 31). Every application attached to one
//! session shares one host, so each sees the others' edits, told by a
//! revision bell. The application an edit comes through is the one the door
//! admitted, never the client's claim.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chirograph::{
    AdvertisedAction, BoundsRelationship, CachePolicy, CardValueV1, CarrierNotice, ContentHash,
    EndpointDescriptor, IntentEffect, IntentInvocation, IntentReference, IntentResult,
    PortableCardV1, PresentationBinding, PresentationCapability, PresentationCodec,
    PresentationKey, PresentationManifest, PresentationOffer, PresentationSemantics,
    ProjectionOffer, ProjectionRequest, ProjectionSession, ProjectionSnapshot, ProtocolVersion,
    ResourceRequest, ResourceResponse, SemanticRole,
};
use graphshell::access::AccessContext;
use graphshell::handlers::HandlerRegistry;
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::mere_host::{MereHost, SelectedPersonaRef};
use graphshell::native::app_admission::{AppId, AppRouteGrants};
use graphshell::native::app_broker::AppEndpointCatalog;
use graphshell::native::endpoint_catalog::ResidentEndpointRoute;
use graphshell::session_item::{
    self, ATTACH_SESSION_INTENT, ApplyEditsV1, FORK_SESSION_INTENT, MERE_GRAPH, MERE_SESSIONS,
    MINT_SESSION_INTENT, RESTORE_SESSION_INTENT, SESSIONS_SCHEMA, SESSIONS_SOURCE,
    SessionsActionV1, SetViewV1, StepV1, TRASH_SESSION_INTENT,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use muniment::DirectoryBackend;
use pandect::{
    Author, GraphSessionManifest, MereRecord, MereSessions, SessionId, open_mere_backend,
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
    served: Mutex<BTreeMap<String, Arc<MereShared>>>,
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
                served: Mutex::new(BTreeMap::new()),
            }),
        }
    }

    /// Serve `mere` on `mere/<domain>`: open its store, register its route and
    /// grant it, unless it is served already.
    pub async fn serve(&self, mere: &MereRecord) -> Result<ResidentEndpointRoute, String> {
        let domain = mere.domain.to_string();
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
            backend,
            persona: self.inner.persona,
            hosts: Mutex::new(HashMap::new()),
            lifecycle: AtomicU64::new(1),
        });
        let opened = Arc::clone(&shared);
        self.inner
            .catalog
            .update(|catalog| {
                catalog.register_notifying(route.id(), format!("Mere: {domain}"), move |context| {
                    MereEndpoint::open(Arc::clone(&opened), context)
                })
            })
            .await
            .map_err(|error| error.to_string())?;
        for app in &self.inner.apps {
            self.inner.grants.grant(app.clone(), route.clone());
        }
        served.insert(domain, shared);
        Ok(route)
    }
}

type SharedHost = Arc<Mutex<MereHost<DirectoryBackend>>>;

/// One mere's store, and the host of each of its sessions an application has
/// attached to.
struct MereShared {
    domain: String,
    backend: DirectoryBackend,
    persona: PersonaId,
    hosts: Mutex<HashMap<SessionId, SharedHost>>,
    /// The sessions projection's revision: it moves with every lifecycle step.
    lifecycle: AtomicU64,
}

impl MereShared {
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

    /// The one host of session `id`, opened once and shared by every
    /// application attached to it.
    async fn host(&self, id: SessionId) -> Result<SharedHost, String> {
        let mut hosts = self.hosts.lock().await;
        if let Some(host) = hosts.get(&id) {
            return Ok(Arc::clone(host));
        }
        let persona = self.persona_id();
        let host = MereHost::open_session(
            self.backend.clone(),
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
        )
        .await
        .map_err(|error| error.to_string())?
        .with_projection_session(ProjectionSession(MERE_GRAPH.to_string()))
        .with_session_item();
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
struct MereEndpoint {
    shared: Arc<MereShared>,
    app: String,
    attached: SessionId,
    host: SharedHost,
    /// The graph epoch and revision this connection last saw or was told of.
    seen_graph: (SceneEpoch, Revision),
    /// The sessions revision this connection last saw or was told of.
    seen_sessions: u64,
}

impl MereEndpoint {
    fn open(shared: Arc<MereShared>, context: &AdmittedEndpointContext) -> Result<Self, String> {
        let app = context
            .application()
            .ok_or("a mere's route serves admitted applications only")?
            .to_string();
        run(async {
            let attached = shared.latest(&app).await?;
            let host = shared.host(attached).await?;
            let seen_graph = {
                let host = host.lock().await;
                (host.projection_epoch(), host.projection_revision())
            };
            Ok(Self {
                seen_sessions: shared.lifecycle.load(Ordering::SeqCst),
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
        let mut actions = Vec::new();
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
        let revision = self.sessions_revision();
        let cards = self.session_cards()?;
        let mut scene = Scene::new();
        let mut presentation = PresentationManifest::default();
        for (index, card) in cards.iter().enumerate() {
            let source = scene.intern_source(SourceRef::new(SESSIONS_SOURCE, &card.source));
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
        self.seen_sessions = revision.0;
        Ok(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: Self::sessions_session(),
            scene: SceneSnapshot::from_dense(SESSIONS_EPOCH, revision, scene)
                .map_err(|error| format!("{error:?}"))?,
            presentation,
            cache_policy: CachePolicy::default(),
        })
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

    fn invoke_graph(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        let host = Arc::clone(&self.host);
        let app = self.app.clone();
        let (result, seen) = run(async move {
            let mut host = host.lock().await;
            if Some(intent.target) != host.session_instance() {
                let result = host.invoke(intent).map_err(|error| error.to_string());
                return (result, None);
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
                            match host.through(app.clone(), |host| host.apply_edits(payload.edits))
                            {
                                Ok(()) => host.persist(now_secs()).await.map(|()| true),
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
                                host.undo(&app).await
                            } else {
                                host.redo(&app).await
                            };
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
                            .set_view(&app, &payload.view, payload.state)
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
            ],
        }
    }
}

impl ProjectionSource for MereEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        match request.session.0.as_str() {
            MERE_SESSIONS => self.sessions_snapshot(),
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

impl PresentationSource for MereEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        match request.session.0.as_str() {
            MERE_SESSIONS => {
                let bytes = self
                    .session_cards()?
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

impl IntentSink for MereEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        match intent.session.0.as_str() {
            MERE_SESSIONS => self.invoke_sessions(intent),
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
    use graphshell::mere_route;
    use graphshell::native::app_admission::{AllowedAppRoutes, AppHello, AppRouteId};
    use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointSession};
    use kernel::graph::{ContainmentSubKind, EdgeAssertion, SemanticSubKind};
    use mere_view::{MereViewModel, MereViewRequest, SessionStep};
    use pandect::{Change, ChangeKind, mere_dir};
    use uuid::Uuid;

    use crate::resident_reservoir::{
        EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
    };

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
    }

    async fn fixture(persona_n: u128, domain: &str) -> Fixture {
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
        catalog
            .update(|catalog| reservoir.register(catalog, Some(routes)))
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
}
