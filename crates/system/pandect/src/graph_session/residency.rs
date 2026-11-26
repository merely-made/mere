// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Addressable checkpoints share the recorded session's writer authority.
use super::publication::{Publication, PublicationHook, PublishedGeneration, SessionBackend};
use super::*;
use crate::addressable_graph::{AddressableGraphStore, ResidencyRoots, ResidentGraph};

/// The kernel graph's publication: the addressable store's (engine split
/// D257). Each is the call a kernel session made before the hook.
pub(super) static KERNEL_PUBLICATION: PublicationHook<Graph> = PublicationHook {
    check: |graph| {
        crate::addressable_graph::refuse_residency_limited(graph).map_err(SessionError::Store)
    },
    prepare: kernel_prepare,
    revision: kernel_revision,
    published: kernel_published,
};

fn kernel_prepare(
    graph: &Graph,
    backend: SessionBackend<'_>,
    container: Uuid,
) -> Result<Publication, SessionError> {
    let store = AddressableGraphStore::new(backend, container);
    let publication = store
        .prepare_recorded_graph(graph)
        .map_err(SessionError::Store)?;
    let generation = publication.generation().to_owned();
    let staged = store.stage(publication);
    Ok(Publication {
        generation,
        writes: Box::new(move |reader: &dyn muniment::TransactionReader| staged.decide(reader)),
    })
}

/// Snapshot reconstruction synthesizes visit times; the exact authoritative
/// facet sidecar replaces them before the generation is named.
fn kernel_revision(
    graph: &Graph,
    backend: SessionBackend<'_>,
    container: Uuid,
) -> Result<String, SessionError> {
    let mut normalized = Graph::try_from_recorded_snapshot(&graph.to_snapshot_at(0))
        .map_err(|error| SessionError::Validation(error.to_string()))?;
    normalized
        .install_facets(graph.facets().clone())
        .map_err(|error| SessionError::Validation(error.to_string()))?;
    let publication =
        AddressableGraphStore::new(backend, container).prepare_recorded_graph(&normalized)?;
    Ok(publication.generation().to_owned())
}

fn kernel_published(backend: SessionBackend<'_>, container: Uuid) -> PublishedGeneration<'_> {
    Box::pin(async move {
        let resident = AddressableGraphStore::new(backend, container)
            .open(ResidencyRoots::default())
            .await?;
        Ok(resident.generation().to_owned())
    })
}
impl<B: Backend + Clone> GraphSession<B> {
    /// The addressable store of this session's root graph, for a host that
    /// stages its own settings writes beside the policy.
    pub fn resident_store(&self) -> AddressableGraphStore<B> {
        AddressableGraphStore::new(
            self.slots.backend().clone(),
            *self.manifest.root_graph_id.as_uuid(),
        )
    }

    /// The addressable publication of this session's graph, prepared without
    /// storage. The same qualification and completeness rules as
    /// [`publish_resident_checkpoint`](Self::publish_resident_checkpoint).
    pub(crate) fn prepare_resident_publication(
        &self,
    ) -> Result<crate::addressable_graph::PreparedGraphPublication, SessionError> {
        self.check_resident_publishable()?;
        self.resident_store()
            .prepare_recorded_graph(&self.graph)
            .map_err(SessionError::Store)
    }
    /// The refusals of
    /// [`prepare_resident_publication`](Self::prepare_resident_publication),
    /// an unqualified writer and a residency-limited graph, without preparing
    /// the publication (engine split D171).
    pub(crate) fn check_resident_publishable(&self) -> Result<(), SessionError> {
        if self.placement.is_none() {
            return Err(SessionError::Validation(
                "addressable publication requires qualified recorded placement".into(),
            ));
        }
        crate::addressable_graph::refuse_residency_limited(&self.graph).map_err(SessionError::Store)
    }
}
impl<B: LegacyTransactionBackend + Clone> GraphSession<B> {
    pub async fn publish_resident_checkpoint(&mut self) -> Result<String, SessionError> {
        if self.placement.is_none() {
            return Err(SessionError::Validation(
                "addressable publication requires qualified recorded placement".into(),
            ));
        }
        // Refuse before checkpointing too: a limited view cannot become the
        // authority for an existing session's baseline or journal.
        if self
            .graph
            .known_coverage()
            .limits
            .iter()
            .any(|l| l.layer == kernel::graph::CoverageLayer::Residency)
        {
            return Err(SessionError::Validation(
                "cannot checkpoint a residency-limited graph".into(),
            ));
        }
        self.checkpoint().await?;
        self.resident_store()
            .write_recorded_graph(&self.graph)
            .await
            .map_err(SessionError::Store)
    }
}
impl<B: Backend + Clone> MereSessions<B> {
    pub async fn open_resident_graph(
        &self,
        id: SessionId,
        roots: ResidencyRoots,
    ) -> Result<ResidentGraph<B>, SessionError> {
        let slots = self.slots();
        let manifest: GraphSessionManifest = read(&slots, &Keys::new(id).at(MANIFEST))
            .await?
            .ok_or(SessionError::Missing(id))?;
        if manifest.session_id != id {
            return Err(SessionError::Corrupt(
                "resident session manifest identity mismatch".into(),
            ));
        }
        AddressableGraphStore::new(self.backend.clone(), *manifest.root_graph_id.as_uuid())
            .open(roots)
            .await
            .map_err(SessionError::Store)
    }
}
