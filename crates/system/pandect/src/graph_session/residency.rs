// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Addressable checkpoints share the recorded session's writer authority.
use super::*;
use crate::addressable_graph::{AddressableGraphStore, ResidencyRoots, ResidentGraph};
/// One immutable checkpoint and its addressable publication, prepared without I/O.
/// Only the originating, unchanged session may acknowledge these bytes.
pub struct PreparedResidentCheckpoint {
    pending: Pending,
    publication: crate::addressable_graph::PreparedGraphPublication,
    ops: Vec<muniment::WriteOp>,
    session: crate::SessionId,
    graph: uuid::Uuid,
    revision: u64,
    cursor: u64,
}
impl PreparedResidentCheckpoint {
    pub fn ops(&self) -> &[muniment::WriteOp] {
        &self.ops
    }
    pub fn session_id(&self) -> crate::SessionId {
        self.session
    }
    pub fn source_revision(&self) -> u64 {
        self.revision
    }
    pub fn source_cursor(&self) -> u64 {
        self.cursor
    }
    pub fn generation(&self) -> &str {
        self.publication.generation()
    }
}
impl<B: Backend + Clone> GraphSession<B> {
    pub fn prepare_resident_checkpoint(
        &self,
        at: SystemTime,
    ) -> Result<PreparedResidentCheckpoint, SessionError> {
        if self.placement.is_none() {
            return Err(SessionError::Validation(
                "addressable publication requires qualified recorded placement".into(),
            ));
        }
        if self
            .graph
            .known_coverage()
            .limits
            .iter()
            .any(|limit| limit.layer == kernel::graph::CoverageLayer::Residency)
        {
            return Err(SessionError::Validation(
                "cannot checkpoint a residency-limited graph".into(),
            ));
        }
        let pending = self.pending_with(at, true)?;
        let graph = *self.manifest.root_graph_id.as_uuid();
        let publication = AddressableGraphStore::new(self.slots.backend().clone(), graph)
            .prepare_recorded_graph(&self.graph)?;
        let mut ops = pending.ops().to_vec();
        ops.extend_from_slice(publication.ops());
        Ok(PreparedResidentCheckpoint {
            pending,
            publication,
            ops,
            graph,
            session: self.id(),
            revision: self.revision(),
            cursor: self.journal.live_cursor().0,
        })
    }
    pub fn acknowledge_resident_checkpoint(
        &mut self,
        prepared: PreparedResidentCheckpoint,
    ) -> Result<String, SessionError> {
        self.validate_resident_checkpoint(&prepared)?;
        let generation = prepared.generation().to_owned();
        self.stored(prepared.pending);
        Ok(generation)
    }
    /// Check completion identity without consuming the retained checkpoint.
    /// Hosts call this before taking their sole prepared batch for acknowledgement.
    pub fn validate_resident_checkpoint(
        &self,
        prepared: &PreparedResidentCheckpoint,
    ) -> Result<(), SessionError> {
        if self.id() != prepared.session
            || *self.manifest.root_graph_id.as_uuid() != prepared.graph
            || self.revision() != prepared.revision
            || self.journal.live_cursor().0 != prepared.cursor
        {
            return Err(SessionError::Validation(
                "stale resident checkpoint completion".into(),
            ));
        }
        Ok(())
    }

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
        AddressableGraphStore::new(
            self.slots.backend().clone(),
            *self.manifest.root_graph_id.as_uuid(),
        )
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
