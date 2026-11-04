// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ephemeral resident materialization. Hosts retain replay, persistence and
//! publication authority; a failed read returns no materialized state.

use std::collections::HashMap;

use chirograph::{
    ContentHash, EndpointDescriptor, PortableCardV1, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot,
};
use graphshell_endpoint::door::projection::AppBrokerClient;
use mere_view::MereViewModel;
use sceno::InstanceId;

pub use crate::validate_graph_material;
use crate::{route, wire};

/// How much of the attached graph to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphRead {
    ProjectionOnly,
    Complete,
}

/// An already admitted source. Each read opens a fresh descriptor. No method
/// on this boundary writes domain truth or performs product replay.
#[allow(async_fn_in_trait)]
pub trait MaterializationSource {
    async fn open_descriptor(&mut self) -> Result<EndpointDescriptor, String>;
    async fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String>;
    async fn resource(
        &mut self,
        session: ProjectionSession,
        hash: ContentHash,
    ) -> Result<Vec<u8>, String>;
}

impl MaterializationSource for AppBrokerClient {
    async fn open_descriptor(&mut self) -> Result<EndpointDescriptor, String> {
        self.open_session()
            .await
            .map(|opened| opened.descriptor)
            .map_err(|e| e.to_string())
    }
    async fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        AppBrokerClient::snapshot(self, request)
            .await
            .map_err(|e| e.to_string())
    }
    async fn resource(
        &mut self,
        session: ProjectionSession,
        hash: ContentHash,
    ) -> Result<Vec<u8>, String> {
        AppBrokerClient::resource(self, session, hash)
            .await
            .map_err(|e| e.to_string())
    }
}

/// One fully checked read; resources remain only in memory.
#[derive(Clone, Debug)]
pub struct MaterializedMere {
    pub sessions: ProjectionSnapshot,
    pub graph: ProjectionSnapshot,
    pub archive: Option<ProjectionSnapshot>,
    pub cards: HashMap<ContentHash, Vec<u8>>,
    pub graph_material: Option<wire::SessionGraphV1>,
    pub pending_merge: Option<wire::PendingMergeV1>,
}

impl MaterializedMere {
    /// Replace the disclosed model reading, retaining host layout, status and
    /// actions. An archive or merge absent from this read clears the old one.
    pub fn fill_model(&self, model: &mut MereViewModel) {
        model.archive = None;
        model.merge = None;
        route::fill(model, &self.sessions, &self.graph, &self.cards);
        if let Some(archive) = &self.archive {
            route::fill_archive(model, &self.sessions, archive, &self.cards);
        }
    }
}

fn request(
    descriptor: &EndpointDescriptor,
    name: &str,
    mandatory: bool,
) -> Result<Option<ProjectionRequest>, String> {
    let mut offers = descriptor
        .projections
        .iter()
        .filter(|offer| offer.request.session.0 == name);
    let first = offers.next().map(|offer| offer.request.clone());
    if offers.next().is_some() {
        return Err(format!("Duplicate projection advertisement: {name}"));
    }
    if mandatory && first.is_none() {
        return Err(format!("Missing projection advertisement: {name}"));
    }
    Ok(first)
}

async fn snapshot(
    source: &mut impl MaterializationSource,
    request: ProjectionRequest,
) -> Result<ProjectionSnapshot, String> {
    let expected = request.session.clone();
    let snapshot = source.snapshot(request).await?;
    if snapshot.session != expected {
        return Err("The snapshot names another projection session.".into());
    }
    snapshot
        .scene
        .validate()
        .map_err(|error| format!("Invalid projection scene: {error:?}"))?;
    Ok(snapshot)
}

async fn resource(
    source: &mut impl MaterializationSource,
    session: &ProjectionSession,
    hash: ContentHash,
) -> Result<Vec<u8>, String> {
    let bytes = source.resource(session.clone(), hash).await?;
    if ContentHash::of(&bytes) != hash {
        return Err("The resource does not match its content hash.".into());
    }
    Ok(bytes)
}

fn source_of(snapshot: &ProjectionSnapshot, instance: InstanceId) -> Option<&sceno::SourceRef> {
    let item = snapshot
        .scene
        .tables
        .items
        .get(instance.0 as usize)?
        .as_ref()?;
    snapshot
        .scene
        .tables
        .sources
        .get(item.source.0 as usize)?
        .as_ref()
}

async fn cards(
    source: &mut impl MaterializationSource,
    snapshot: &ProjectionSnapshot,
    result: &mut HashMap<ContentHash, Vec<u8>>,
) -> Result<(), String> {
    // Required cards come from active source items, not the possibly
    // incomplete presentation sidecar. Graph cards are fetched only for a
    // Complete read. Keep checking every other advertised binding too.
    let mut instances = route::required_card_instances(snapshot, true);
    for binding in &snapshot.presentation.bindings {
        if !instances.contains(&binding.instance) {
            instances.push(binding.instance);
        }
    }
    for instance in instances {
        let hash = route::card_hash(snapshot, instance)?;
        // Fetch in each projection scope, even if another scope offered the same hash.
        let bytes = resource(source, &snapshot.session, hash).await?;
        serde_json::from_slice::<PortableCardV1>(&bytes)
            .map_err(|e| format!("Invalid portable card: {e}"))?;
        result.insert(hash, bytes);
    }
    Ok(())
}

/// Read fresh projections and validate every fetched resource before decoding.
/// Only Complete fetches graph cards and the scoped graph envelope; projection
/// reads still fetch sessions/archive cards and any advertised pending merge.
pub async fn read(
    source: &mut impl MaterializationSource,
    application: &str,
    graph_read: GraphRead,
) -> Result<MaterializedMere, String> {
    let descriptor = source.open_descriptor().await?;
    let sessions_request = request(&descriptor, wire::MERE_SESSIONS, true)?.unwrap();
    let graph_request = request(&descriptor, wire::MERE_GRAPH, true)?.unwrap();
    let archive_request = request(&descriptor, wire::MERE_ARCHIVE, false)?;
    let sessions = snapshot(source, sessions_request).await?;
    let graph = snapshot(source, graph_request).await?;
    let archive = if let Some(request) = archive_request {
        Some(snapshot(source, request).await?)
    } else {
        None
    };
    let mut result = MaterializedMere {
        sessions,
        graph,
        archive,
        cards: HashMap::new(),
        graph_material: None,
        pending_merge: None,
    };
    cards(source, &result.sessions, &mut result.cards).await?;
    if let Some(archive) = &result.archive {
        cards(source, archive, &mut result.cards).await?;
    }
    if let Some(instance) = route::unique_merge_card(&result.sessions)? {
        let card_hash = route::card_hash(&result.sessions, instance)?;
        let bytes = result
            .cards
            .get(&card_hash)
            .ok_or("Missing material card resource.")?;
        let card: PortableCardV1 =
            serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        let hash = route::pending_merge_media(&card)?;
        let bytes = resource(source, &result.sessions.session, hash).await?;
        result.pending_merge = Some(route::decode_pending_merge(&bytes)?);
    }
    if graph_read == GraphRead::Complete {
        cards(source, &result.graph, &mut result.cards).await?;
        let attached: Vec<_> = result
            .graph
            .scene
            .active_items_in_order()
            .into_iter()
            .filter_map(|(instance, _)| {
                source_of(&result.graph, instance)
                    .filter(|source| source.adapter == wire::SESSION_ITEM_SOURCE)
                    .map(|source| (instance, source))
            })
            .collect();
        if attached.len() > 1 {
            return Err("Multiple attached session cards.".into());
        }
        if let Some((instance, attached)) = attached.first() {
            let session = pandect::SessionId(
                uuid::Uuid::parse_str(&attached.id)
                    .map_err(|_| "The material card names no attached session.")?,
            );
            let card_hash = route::card_hash(&result.graph, *instance)?;
            let bytes = result
                .cards
                .get(&card_hash)
                .ok_or("Missing material card resource.")?;
            let card: PortableCardV1 = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
            let hash = wire::session_media(&card)?;
            let bytes = resource(source, &result.graph.session, hash).await?;
            let envelope: wire::SessionGraphV1 = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid scoped graph material: {e}"))?;
            validate_graph_material(
                &envelope,
                application,
                session,
                result.graph.scene.epoch,
                result.graph.scene.revision,
            )?;
            result.graph_material = Some(envelope);
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "materialization/tests.rs"]
mod tests;
