// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident reservoir inside Djinn (reservoir plan, V1).
//!
//! Djinn holds the only open handle on the selected persona's reservoir: the
//! index of every mere the persona keeps, one per data domain. Applications
//! reach it through the admitted `reservoir` route. Each session sees the
//! persona's meres and may ensure the mere for a domain; nobody else opens the
//! reservoir's files, and a second owner is refused by the store's lock.
//!
//! The lane is on unless the owner turns it off (Mark, 2026-09-23: meres are
//! reachable by default). A reservoir that cannot open, for example because no
//! single persona wallet exists yet, leaves the lane unavailable with its
//! reason and every other lane running.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chirograph::{
    AdvertisedAction, BoundsRelationship, CachePolicy, CardValueV1, ContentHash,
    EndpointDescriptor, IntentEffect, IntentInvocation, IntentReference, IntentResult,
    PortableCardV1, PresentationBinding, PresentationCapability, PresentationCodec,
    PresentationKey, PresentationManifest, PresentationOffer, PresentationSemantics,
    ProjectionOffer, ProjectionRequest, ProjectionSession, ProjectionSnapshot, ProtocolVersion,
    ResourceRequest, ResourceResponse, SemanticRole,
};
use graphshell::native::endpoint_catalog::{
    ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointRoute,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use muniment::RedbBackend;
use pandect::wallet_store::resolve_persona;
use pandect::{DomainId, MereRecord, ReservoirStore, open_reservoir_backend};
use personae::PersonaId;
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use serde::{Deserialize, Serialize};

use crate::settings::ReservoirLaneSettings;

/// The stable first-party route an admitted application requests.
pub const RESIDENT_RESERVOIR_ROUTE: &str = "reservoir";
/// How often a granted client polls the route for notices.
pub const RESIDENT_RESERVOIR_NOTICE_POLL: Duration = Duration::from_millis(50);
/// Ensure the mere for a data domain, creating it if the reservoir has none.
pub const RESERVOIR_ENSURE_MERE_INTENT: &str = "mere.reservoir.ensure";
/// Schema of [`EnsureMereV1`].
pub const ENSURE_MERE_SCHEMA: &str = "mere.reservoir.ensure/v1";

const SESSION: &str = "djinn.reservoir/v1";
const SOURCE_KIND: &str = "mere.reservoir";
const EPOCH: SceneEpoch = SceneEpoch(1);
/// The reservoir's own card; meres follow it in domain order.
const RESERVOIR_INSTANCE: InstanceId = InstanceId(0);

/// Payload of [`RESERVOIR_ENSURE_MERE_INTENT`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnsureMereV1 {
    pub schema: String,
    pub domain: String,
}

impl EnsureMereV1 {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            schema: ENSURE_MERE_SCHEMA.to_string(),
            domain: domain.into(),
        }
    }
}

/// What the resident's reservoir lane is doing.
pub enum ReservoirLane {
    /// The owner turned the lane off.
    Off,
    /// The lane is on but the reservoir could not open; the reason says why.
    Unavailable(String),
    Open(ResidentReservoir),
}

impl ReservoirLane {
    /// Open the lane as the owner's settings say. This never fails the
    /// resident: a reservoir that cannot open is reported with its reason.
    pub async fn open(shared_root: &Path, settings: &ReservoirLaneSettings) -> Self {
        if !settings.enabled {
            return Self::Off;
        }
        match ResidentReservoir::open(shared_root, settings.persona).await {
            Ok(reservoir) => Self::Open(reservoir),
            Err(reason) => Self::Unavailable(reason),
        }
    }

    pub fn reservoir(&self) -> Option<&ResidentReservoir> {
        match self {
            Self::Open(reservoir) => Some(reservoir),
            Self::Off | Self::Unavailable(_) => None,
        }
    }

    /// Release the reservoir, and with it the store's lock. Refused while an
    /// endpoint session still holds the reservoir, the way the published-site
    /// close refuses active borrowers.
    pub async fn close(self) -> Result<(), String> {
        match self {
            Self::Open(reservoir) => Arc::try_unwrap(reservoir.shared)
                .map(drop)
                .map_err(|_| "reservoir route still has active borrowers".to_string()),
            Self::Off | Self::Unavailable(_) => Ok(()),
        }
    }
}

struct Shared {
    persona: PersonaId,
    store: tokio::sync::Mutex<ReservoirStore<RedbBackend>>,
    revision: AtomicU64,
}

/// The one open reservoir of one persona, shared by every admitted session.
pub struct ResidentReservoir {
    shared: Arc<Shared>,
}

impl ResidentReservoir {
    /// Open `persona`'s reservoir under `shared_root`, or the sole persona
    /// wallet's when none is named. Takes the reservoir's exclusive lock.
    pub async fn open(shared_root: &Path, persona: Option<PersonaId>) -> Result<Self, String> {
        let persona = resolve_persona(shared_root, persona).map_err(|error| error.to_string())?;
        let backend =
            open_reservoir_backend(shared_root, persona).map_err(|error| error.to_string())?;
        let store = ReservoirStore::open(backend, persona)
            .await
            .map_err(|error| error.to_string())?;
        Ok(Self {
            shared: Arc::new(Shared {
                persona,
                store: tokio::sync::Mutex::new(store),
                revision: AtomicU64::new(1),
            }),
        })
    }

    pub fn persona(&self) -> PersonaId {
        self.shared.persona
    }

    /// Every mere, in domain order.
    pub async fn meres(&self) -> Vec<MereRecord> {
        self.shared.store.lock().await.meres().cloned().collect()
    }

    /// Register the reservoir route. Every admitted open receives its own
    /// session over the one shared reservoir.
    pub fn register(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<(), ResidentEndpointCatalogError> {
        let shared = Arc::clone(&self.shared);
        catalog.register(RESIDENT_RESERVOIR_ROUTE, "Reservoir", move |_| {
            Ok(ReservoirEndpoint {
                shared: Arc::clone(&shared),
            })
        })
    }

    /// The route descriptor granted to first-party clients.
    pub fn route() -> ResidentEndpointRoute {
        ResidentEndpointRoute::new(RESIDENT_RESERVOIR_ROUTE, RESIDENT_RESERVOIR_NOTICE_POLL)
            .expect("the resident reservoir route is valid")
    }
}

/// One admitted session's view of the shared reservoir.
struct ReservoirEndpoint {
    shared: Arc<Shared>,
}

/// A card and the bytes it is served as.
struct ServedCard {
    source: String,
    label: String,
    bytes: Vec<u8>,
    actions: Vec<AdvertisedAction>,
}

impl ReservoirEndpoint {
    fn session() -> ProjectionSession {
        ProjectionSession(SESSION.into())
    }

    fn request() -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: Self::session(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }

    fn revision(&self) -> Revision {
        Revision(self.shared.revision.load(Ordering::SeqCst))
    }

    fn run<T>(&self, operation: impl std::future::Future<Output = T>) -> T {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(operation))
    }

    fn ensure_action() -> AdvertisedAction {
        AdvertisedAction {
            intent: IntentReference(RESERVOIR_ENSURE_MERE_INTENT.into()),
            label: "Ensure a mere".into(),
            explanation: "Open the mere for a data domain, creating it if this persona has none. \
                          A domain has exactly one mere."
                .into(),
            payload_schema: ENSURE_MERE_SCHEMA.into(),
            input_form: None,
            effect: IntentEffect::DomainTruth,
        }
    }

    /// The reservoir card first, then one card per mere in domain order.
    fn cards(&self) -> Result<Vec<ServedCard>, String> {
        let meres = self.run(async {
            self.shared
                .store
                .lock()
                .await
                .meres()
                .cloned()
                .collect::<Vec<_>>()
        });
        let mut cards = Vec::with_capacity(meres.len() + 1);
        let reservoir = PortableCardV1 {
            title: "Reservoir".into(),
            values: vec![
                CardValueV1 {
                    label: "Persona".into(),
                    value: self.shared.persona.as_uuid().to_string(),
                },
                CardValueV1 {
                    label: "Meres".into(),
                    value: meres.len().to_string(),
                },
            ],
            badges: Vec::new(),
            media: Vec::new(),
        };
        cards.push(ServedCard {
            source: "reservoir".into(),
            label: "Reservoir".into(),
            bytes: serde_json::to_vec(&reservoir).map_err(|error| error.to_string())?,
            actions: vec![Self::ensure_action()],
        });
        for mere in meres {
            let card = PortableCardV1 {
                title: mere.domain.to_string(),
                values: vec![
                    CardValueV1 {
                        label: "Mere".into(),
                        value: mere.id.to_string(),
                    },
                    CardValueV1 {
                        label: "Created (ms)".into(),
                        value: mere.created_at_ms.to_string(),
                    },
                ],
                badges: Vec::new(),
                media: Vec::new(),
            };
            cards.push(ServedCard {
                source: mere.id.to_string(),
                label: format!("Mere: {}", mere.domain),
                bytes: serde_json::to_vec(&card).map_err(|error| error.to_string())?,
                actions: Vec::new(),
            });
        }
        Ok(cards)
    }

    fn check_intent(&self, intent: &IntentInvocation) -> Result<(), IntentResult> {
        if intent.session != Self::session() || intent.target != RESERVOIR_INSTANCE {
            return Err(IntentResult::Rejected {
                reason: "intent names another reservoir session or target".into(),
            });
        }
        let current = self.revision();
        if intent.observed_epoch != EPOCH || intent.observed_revision != current {
            return Err(IntentResult::Stale {
                current_epoch: EPOCH,
                current_revision: current,
            });
        }
        Ok(())
    }
}

impl ProjectionCatalog for ReservoirEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Djinn reservoir".into(),
            projections: vec![ProjectionOffer {
                label: "This persona's meres".into(),
                request: Self::request(),
            }],
        }
    }
}

impl ProjectionSource for ReservoirEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        if request.session != Self::session() {
            return Err("reservoir snapshot names another session".into());
        }
        let revision = self.revision();
        let cards = self.cards()?;
        let mut scene = Scene::new();
        let mut presentation = PresentationManifest::default();
        for (index, card) in cards.iter().enumerate() {
            let source = scene.intern_source(SourceRef::new(SOURCE_KIND, &card.source));
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
        Ok(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: Self::session(),
            scene: SceneSnapshot::from_dense(EPOCH, revision, scene)
                .map_err(|error| format!("{error:?}"))?,
            presentation,
            cache_policy: CachePolicy::default(),
        })
    }
}

impl PresentationSource for ReservoirEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        if request.session != Self::session() {
            return Err("reservoir resource names another session".into());
        }
        let bytes = self
            .cards()?
            .into_iter()
            .find(|card| ContentHash::of(&card.bytes) == request.resource)
            .map(|card| card.bytes)
            .ok_or("reservoir resource is no longer current; request a new snapshot")?;
        Ok(ResourceResponse {
            session: Self::session(),
            resource: request.resource,
            bytes,
        })
    }
}

impl IntentSink for ReservoirEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        if let Err(result) = self.check_intent(&intent) {
            return Ok(result);
        }
        match intent.intent.as_str() {
            RESERVOIR_ENSURE_MERE_INTENT => {
                let ensure: EnsureMereV1 = match serde_json::from_slice(&intent.payload) {
                    Ok(ensure) => ensure,
                    Err(error) => {
                        return Ok(IntentResult::Rejected {
                            reason: format!("invalid ensure-mere payload: {error}"),
                        });
                    },
                };
                if ensure.schema != ENSURE_MERE_SCHEMA {
                    return Ok(IntentResult::Rejected {
                        reason: format!("unknown ensure-mere schema {:?}", ensure.schema),
                    });
                }
                let domain = match DomainId::new(ensure.domain) {
                    Ok(domain) => domain,
                    Err(error) => {
                        return Ok(IntentResult::Rejected {
                            reason: error.to_string(),
                        });
                    },
                };
                let created_at_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|elapsed| elapsed.as_millis() as u64)
                    .unwrap_or_default();
                let result = self.run(async {
                    self.shared
                        .store
                        .lock()
                        .await
                        .ensure(domain, created_at_ms)
                        .await
                });
                match result {
                    Ok((_, created)) => {
                        if created {
                            self.shared.revision.fetch_add(1, Ordering::SeqCst);
                        }
                        Ok(IntentResult::Accepted)
                    },
                    Err(error) => Ok(IntentResult::Rejected {
                        reason: error.to_string(),
                    }),
                }
            },
            other => Ok(IntentResult::Rejected {
                reason: format!("the reservoir has no intent {other:?}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphshell::lifecycle::AdmittedEndpointContext;
    use uuid::Uuid;

    fn persona(n: u128) -> PersonaId {
        PersonaId::from_uuid(Uuid::from_u128(n))
    }

    fn ensure(snapshot: &ProjectionSnapshot, domain: &str) -> IntentInvocation {
        IntentInvocation {
            session: ReservoirEndpoint::session(),
            target: RESERVOIR_INSTANCE,
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
            payload: serde_json::to_vec(&EnsureMereV1::new(domain)).unwrap(),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn an_admitted_session_ensures_and_lists_meres_through_the_route() {
        let root = tempfile::tempdir().unwrap();
        let reservoir = ResidentReservoir::open(root.path(), Some(persona(7)))
            .await
            .unwrap();
        let mut catalog = ResidentEndpointCatalog::new();
        reservoir.register(&mut catalog).unwrap();
        let context = AdmittedEndpointContext::new(
            chirograph::ProjectionSession("v1:cleromancy".into()),
            [0xa7; 32],
        );
        let mut session = catalog.open(RESIDENT_RESERVOIR_ROUTE, &context).unwrap();
        let request = session.describe().projections.remove(0).request;

        let empty = session.snapshot(request.clone()).unwrap();
        assert_eq!(
            empty.presentation.bindings.len(),
            1,
            "only the reservoir card"
        );

        assert_eq!(
            session.invoke(ensure(&empty, "divination")).unwrap(),
            IntentResult::Accepted
        );
        let one = session.snapshot(request.clone()).unwrap();
        assert_eq!(one.presentation.bindings.len(), 2);
        assert_ne!(one.scene.revision, empty.scene.revision);

        // Ensuring the same domain again is accepted and creates nothing.
        assert_eq!(
            session.invoke(ensure(&one, "divination")).unwrap(),
            IntentResult::Accepted
        );
        let same = session.snapshot(request.clone()).unwrap();
        assert_eq!(same.scene.revision, one.scene.revision);

        // A second admitted session over the same reservoir sees the mere.
        let mut other = catalog.open(RESIDENT_RESERVOIR_ROUTE, &context).unwrap();
        let seen = other.snapshot(request).unwrap();
        assert_eq!(seen.presentation.bindings.len(), 2);

        let meres = reservoir.meres().await;
        assert_eq!(meres.len(), 1);
        assert_eq!(meres[0].domain.as_str(), "divination");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_bad_domain_or_a_stale_revision_changes_nothing() {
        let root = tempfile::tempdir().unwrap();
        let reservoir = ResidentReservoir::open(root.path(), Some(persona(8)))
            .await
            .unwrap();
        let mut catalog = ResidentEndpointCatalog::new();
        reservoir.register(&mut catalog).unwrap();
        let context =
            AdmittedEndpointContext::new(chirograph::ProjectionSession("v1:test".into()), [1; 32]);
        let mut session = catalog.open(RESIDENT_RESERVOIR_ROUTE, &context).unwrap();
        let request = session.describe().projections.remove(0).request;
        let snapshot = session.snapshot(request).unwrap();

        assert!(matches!(
            session.invoke(ensure(&snapshot, "Not A Domain")).unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(
            session.invoke(ensure(&snapshot, "notes")).unwrap(),
            IntentResult::Accepted
        );
        // The snapshot is now a revision behind.
        assert!(matches!(
            session.invoke(ensure(&snapshot, "seeds")).unwrap(),
            IntentResult::Stale { .. }
        ));
        assert_eq!(reservoir.meres().await.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_lane_reports_why_it_is_unavailable_and_refuses_a_second_owner() {
        let root = tempfile::tempdir().unwrap();
        // No persona wallet exists under this root and none is named.
        match ReservoirLane::open(root.path(), &ReservoirLaneSettings::default()).await {
            ReservoirLane::Unavailable(reason) => assert!(reason.contains("no persona wallet")),
            _ => panic!("a root with no persona wallet must leave the lane unavailable"),
        }
        let off = ReservoirLaneSettings {
            enabled: false,
            ..ReservoirLaneSettings::default()
        };
        assert!(matches!(
            ReservoirLane::open(root.path(), &off).await,
            ReservoirLane::Off
        ));

        let named = ReservoirLaneSettings {
            persona: Some(persona(9)),
            ..ReservoirLaneSettings::default()
        };
        let first = ReservoirLane::open(root.path(), &named).await;
        assert!(first.reservoir().is_some());
        match ReservoirLane::open(root.path(), &named).await {
            ReservoirLane::Unavailable(reason) => {
                assert!(reason.contains("could not open the reservoir"))
            },
            _ => panic!("a second owner must be refused while the first holds the reservoir"),
        }
        drop(first);
        assert!(
            ReservoirLane::open(root.path(), &named)
                .await
                .reservoir()
                .is_some()
        );
    }
}
