// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MPL-2.0

//! Admitted application client for the shared personal-facts mere.
//! This owns no files or private authority. Every edit is a revision-checked
//! candidate on the same resident journal/validation path as other meres.

use chirograph::{
    CarrierRequestBody, CarrierResponseBody, ContentHash, IntentInvocation, IntentResult,
    ProjectionSnapshot,
};
use dramatis::{birth::BirthFacts, resources::SubjectResourceAssociation, subject::SubjectRef};
use graphshell_endpoint::door::{
    app_admission::{AppId, AppRouteId},
    projection::AppBrokerClient,
};
use mere_overmap::wire;
use personal_facts::{BirthResource, ContactResource};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectSummary {
    pub subject: SubjectRef,
    pub label: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalFactsCatalog {
    pub persona: String,
    pub subjects: Vec<SubjectSummary>,
    pub births: Vec<BirthResource>,
    pub associations: Vec<SubjectResourceAssociation>,
}

pub struct PersonalFactsClient {
    client: AppBrokerClient,
    application: String,
    persona: String,
    profiles: Vec<SubjectSummary>,
    envelope: Option<wire::SessionGraphV1>,
    graph_projection: Option<ProjectionSnapshot>,
    sessions: Option<ProjectionSnapshot>,
}
impl PersonalFactsClient {
    /// Connect to an already running admitted resident; never embed or open a file.
    pub async fn open(endpoint: &str, app: AppId) -> Result<Self, String> {
        let mut reservoir = AppBrokerClient::open_route_at(
            endpoint,
            app.clone(),
            AppRouteId::new(crate::resident_reservoir::RESIDENT_RESERVOIR_ROUTE).map_err(err)?,
        )
        .await
        .map_err(err)?;
        let persona = reservoir
            .read_cards()
            .await
            .map_err(err)?
            .into_iter()
            .flat_map(|c| c.card.values)
            .find(|v| v.label == "Persona")
            .ok_or("resident did not disclose persona")?
            .value;
        let opened = reservoir.open_session().await.map_err(err)?;
        let request = opened
            .descriptor
            .projections
            .first()
            .ok_or("no reservoir projection")?
            .request
            .clone();
        let snapshot = reservoir.snapshot(request).await.map_err(err)?;
        let invocation = IntentInvocation {
            session: snapshot.session,
            target: sceno::InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: crate::resident_reservoir::RESERVOIR_ENSURE_MERE_INTENT.into(),
            payload: serde_json::to_vec(&crate::resident_reservoir::EnsureMereV1::new(
                personal_facts::DOMAIN,
            ))
            .map_err(err)?,
        };
        accepted(
            reservoir
                .request_body(CarrierRequestBody::Intent(invocation))
                .await
                .map_err(err)?,
        )?;
        reservoir.close().await.map_err(err)?;
        let profiles = match AppBrokerClient::open_at(endpoint, app.clone()).await {
            Ok(mut identity) => {
                let cards = identity.read_cards().await.map_err(err)?;
                let result = cards
                    .into_iter()
                    .filter(|c| {
                        c.card
                            .badges
                            .iter()
                            .any(|b| b == "profile" || b == "selected profile")
                    })
                    .filter_map(|c| {
                        let id = c
                            .card
                            .values
                            .iter()
                            .find(|v| v.label == "Profile")?
                            .value
                            .clone();
                        Some(SubjectSummary {
                            subject: SubjectRef::Profile {
                                id: personae::ProfileId(id),
                            },
                            label: c.card.title,
                        })
                    })
                    .collect();
                identity.close().await.map_err(err)?;
                result
            },
            Err(_) => Vec::new(), // No invented profile; explicitly created contacts still work.
        };
        let application = app.as_str().to_string();
        let client = AppBrokerClient::open_route_at(
            endpoint,
            app,
            AppRouteId::new("mere/personal-facts").map_err(err)?,
        )
        .await
        .map_err(err)?;
        let mut this = Self {
            client,
            application,
            persona,
            profiles,
            envelope: None,
            graph_projection: None,
            sessions: None,
        };
        this.refresh().await?;
        Ok(this)
    }
    async fn resource(&mut self, request: chirograph::ResourceRequest) -> Result<Vec<u8>, String> {
        let bytes = self
            .client
            .resource(request.session, request.resource)
            .await
            .map_err(err)?;
        if ContentHash::of(&bytes) != request.resource {
            return Err("personal-facts resource hash mismatch".into());
        }
        Ok(bytes)
    }
    pub async fn refresh(&mut self) -> Result<(), String> {
        // Clear previous material before a read, so denial/revocation cannot leave a usable cache.
        self.envelope = None;
        self.graph_projection = None;
        self.sessions = None;
        let opened = self.client.open_session().await.map_err(err)?;
        let request = |name: &str| {
            opened
                .descriptor
                .projections
                .iter()
                .find(|p| p.request.session.0 == name)
                .map(|p| p.request.clone())
                .ok_or_else(|| format!("missing personal-facts projection {name}"))
        };
        let sessions = self
            .client
            .snapshot(request(wire::MERE_SESSIONS)?)
            .await
            .map_err(err)?;
        let projection = self
            .client
            .snapshot(request(wire::MERE_GRAPH)?)
            .await
            .map_err(err)?;
        let item = projection
            .scene
            .active_items_in_order()
            .into_iter()
            .find_map(|(instance, item)| {
                let source = projection
                    .scene
                    .tables
                    .sources
                    .get(item.source.0 as usize)?
                    .as_ref()?;
                (source.adapter == wire::SESSION_ITEM_SOURCE).then(|| (instance, source.id.clone()))
            });
        if let Some((instance, session_id)) = item {
            let binding = projection
                .presentation
                .bindings
                .iter()
                .find(|b| b.instance == instance)
                .ok_or("missing personal-facts session binding")?;
            let offer = projection
                .presentation
                .offers
                .get(&binding.key)
                .and_then(|offers| offers.first())
                .ok_or("missing personal-facts session offer")?;
            let bytes = self
                .resource(chirograph::ResourceRequest {
                    session: projection.session.clone(),
                    resource: offer.resource,
                })
                .await?;
            let card: chirograph::PortableCardV1 = serde_json::from_slice(&bytes).map_err(err)?;
            let bytes = self
                .resource(chirograph::ResourceRequest {
                    session: projection.session.clone(),
                    resource: *card
                        .media
                        .first()
                        .ok_or("missing personal-facts material")?,
                })
                .await?;
            let envelope: wire::SessionGraphV1 = serde_json::from_slice(&bytes).map_err(err)?;
            if envelope.schema != wire::SESSION_GRAPH_SCHEMA
                || envelope.epoch != projection.scene.epoch
                || envelope.revision != projection.scene.revision
                || envelope.application != self.application
                || envelope.session.as_uuid().to_string() != session_id
            {
                return Err(
                    "personal-facts graph belongs to another application/session/revision".into(),
                );
            }
            personal_facts::validate_for_persona(&envelope.clone().into_graph()?, &self.persona)?;
            self.envelope = Some(envelope);
        }
        self.sessions = Some(sessions);
        self.graph_projection = Some(projection);
        Ok(())
    }
    pub async fn catalog(&mut self) -> Result<PersonalFactsCatalog, String> {
        self.refresh().await?;
        let graph = self
            .envelope
            .clone()
            .map(|e| e.into_graph())
            .transpose()?
            .unwrap_or_default();
        let mut subjects = self.profiles.clone();
        for ContactResource { subject, contact } in personal_facts::contacts(&graph)? {
            subjects.push(SubjectSummary {
                subject,
                label: contact.petname,
            });
        }
        Ok(PersonalFactsCatalog {
            persona: self.persona.clone(),
            subjects,
            births: personal_facts::births(&graph)?,
            associations: personal_facts::associations(&graph)?,
        })
    }
    async fn edit<T>(
        &mut self,
        f: impl FnOnce(&mut kernel::graph::Graph) -> Result<T, String>,
    ) -> Result<T, String> {
        self.refresh().await?;
        if self.envelope.is_none() {
            let sessions = self.sessions.as_ref().ok_or("no personal-facts sessions")?;
            let intent = IntentInvocation {
                session: sessions.session.clone(),
                target: sceno::InstanceId(0),
                observed_epoch: sessions.scene.epoch,
                observed_revision: sessions.scene.revision,
                intent: wire::MINT_SESSION_INTENT.into(),
                payload: serde_json::to_vec(&wire::SessionsActionV1::mint(Some(
                    "Personal facts".into(),
                )))
                .map_err(err)?,
            };
            accepted(
                self.client
                    .request_body(CarrierRequestBody::Intent(intent))
                    .await
                    .map_err(err)?,
            )?;
            self.refresh().await?;
        }
        let envelope = self.envelope.clone().ok_or("no personal-facts session")?;
        if envelope.read_only {
            return Err("attach a writable personal-facts session before editing".into());
        }
        let mut graph = envelope.clone().into_graph()?;
        let edits = Arc::new(Mutex::new(Vec::new()));
        let captured = edits.clone();
        graph.set_recorder(Some(Arc::new(move |delta| {
            captured
                .lock()
                .expect("personal-facts candidate")
                .push(delta.clone())
        })));
        let value = f(&mut graph)?;
        personal_facts::validate(&graph)?;
        let projection = self
            .graph_projection
            .as_ref()
            .ok_or("no graph projection")?;
        let steps = std::mem::take(&mut *edits.lock().map_err(err)?);
        if !steps.is_empty() {
            let intent = IntentInvocation {
                session: projection.session.clone(),
                target: sceno::InstanceId(0),
                observed_epoch: projection.scene.epoch,
                observed_revision: projection.scene.revision,
                intent: wire::APPLY_EDITS_INTENT.into(),
                payload: serde_json::to_vec(&wire::ApplyEditsV1::checked(envelope.revision, steps))
                    .map_err(err)?,
            };
            accepted(
                self.client
                    .request_body(CarrierRequestBody::Intent(intent))
                    .await
                    .map_err(err)?,
            )?;
        }
        self.refresh().await?;
        Ok(value)
    }
    pub async fn create_contact(&mut self, label: &str) -> Result<SubjectRef, String> {
        let scope = self.persona.clone();
        let id = gaz::LocalId::from_random(*uuid::Uuid::new_v4().as_bytes());
        self.edit(|graph| personal_facts::create_contact(graph, &scope, id, label))
            .await
    }
    pub async fn save_birth(&mut self, facts: &BirthFacts) -> Result<BirthResource, String> {
        let catalog = self.catalog().await?;
        if !catalog.subjects.iter().any(|s| s.subject == facts.subject) {
            return Err("choose an existing authorized subject before saving birth facts".into());
        }
        self.edit(|graph| personal_facts::save_birth(graph, facts))
            .await
    }
    pub async fn associate(&mut self, value: &SubjectResourceAssociation) -> Result<(), String> {
        self.edit(|graph| personal_facts::save_association(graph, value))
            .await
    }
    pub async fn close(self) -> Result<(), String> {
        self.client.close().await.map_err(err)
    }
}
fn accepted(body: CarrierResponseBody) -> Result<(), String> {
    match body {
        CarrierResponseBody::Intent(IntentResult::Accepted) => Ok(()),
        other => Err(format!("personal-facts resident refused: {other:?}")),
    }
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
