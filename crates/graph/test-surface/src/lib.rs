// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A minimal session graph with no kernel (engine split D245): surfaces that
//! each show a resource, and assertions written through the engine's
//! [`ResourceStore`]. Its journal holds [`JournalDelta`]s of its own two-variant
//! [`SurfaceDelta`] beside the store's [`ResourceDelta`]s, on one log. It is
//! Mere's shared test graph (engine split D272): pandect's and mere-facts'
//! tests run over it, so no kernel survives as a test crate (D211).

use std::collections::BTreeMap;

use mere::model::edge_decode::payload_from_persisted;
use mere::model::resource_rules::ContentError;
use mere::model::{
    AssertionWithdrawal, CoverageNote, EdgePayload, JournalDelta, PersistedEdge,
    PersistedResourceRecord, ResourceDelta, ResourceNode, ResourceStore, SemanticStatementSpec,
};
use mere::session::Recorder;
use portolan::{AcceptAll, FacetId, FacetStore};
use serde::{Deserialize, Serialize};

mod held;
mod merge;
mod projection;
mod replay;
mod session;
mod statements;

pub use held::{PendingLinks, Retention};
pub use projection::{SHOWS, surface_iri};
pub use replay::{Carry, edges_edit};
pub use statements::HeldRow;
/// The id type every surface, resource and edit names.
pub use uuid::Uuid;

/// The surface's own writes: which resource a surface shows, and its facets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SurfaceDelta {
    ShowResource {
        surface_id: Uuid,
        resource_id: Uuid,
    },
    HideSurface {
        surface_id: Uuid,
    },
    /// Set facet `facet` on a surface. Undo does not revert facets.
    SetFacet {
        surface_id: Uuid,
        facet: String,
        value: serde_json::Value,
    },
    /// An edit an older surface wrote: it replays as nothing and has no live
    /// form, so a change carrying one is refused.
    Retired {
        note: String,
    },
}

/// The surface facet naming where a host copied a surface from:
/// `{"origin": <surface id>, "container": <container id>}`, the derivation a
/// picked part merges back by.
pub const COPIED_FROM: &str = "mere-test-surface.copied-from";

/// The surface facet that holds a surface's own statements (literals), each
/// with a handle no other row may hold.
pub const SURFACE_PROPERTIES: &str = "mere-test-surface.properties";

pub type Delta = JournalDelta<SurfaceDelta>;

/// The surface graph: a surface-to-resource map over the engine store.
#[derive(Default)]
pub struct Surface {
    resources: ResourceStore,
    shown: BTreeMap<Uuid, Uuid>,
    facets: FacetStore<Uuid>,
    coverage: CoverageNote,
    recorder: Option<Recorder<Delta>>,
    pending: PendingLinks,
    launch: u64,
    observed: u64,
}

/// A clone records nothing: a scratch copy never writes into a session.
impl Clone for Surface {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone(),
            shown: self.shown.clone(),
            facets: self.facets.clone(),
            coverage: self.coverage.clone(),
            recorder: None,
            pending: self.pending.clone(),
            launch: self.launch,
            observed: self.observed,
        }
    }
}

/// The surface as a session stores it: the deltas that rebuild it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceSnapshot {
    pub shown: Vec<(Uuid, Uuid)>,
    pub resources: Vec<ResourceDelta>,
}

/// What an undo can keep: a surface binding, a resource, or a resource pair.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Part {
    Surface(Uuid),
    Resource(Uuid),
    Pair(Uuid, Uuid),
}

fn uuid(text: &str) -> Result<Uuid, ContentError> {
    Uuid::parse_str(text).map_err(|_| ContentError::MissingResource)
}

/// The edits that add the resource `iri` names and show it on `surface`.
pub fn show_edits(surface: Uuid, iri: &str) -> (Uuid, Vec<Delta>) {
    let resource = ResourceNode::for_term(iri);
    let id = resource.id();
    let record = ResourceDelta::ReplaySetResourceRecordById {
        resource_id: id.to_string(),
        record: Some(PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().to_owned(),
            facets: Vec::new(),
        }),
    };
    (
        id,
        vec![
            JournalDelta::Resource(record),
            JournalDelta::Surface(SurfaceDelta::ShowResource {
                surface_id: surface,
                resource_id: id,
            }),
        ],
    )
}

impl Surface {
    /// The resource surface `surface` shows.
    pub fn shown(&self, surface: Uuid) -> Option<Uuid> {
        self.shown.get(&surface).copied()
    }

    /// The IRI of the resource surface `surface` shows.
    pub fn iri_of(&self, surface: Uuid) -> Option<String> {
        let resource = self.resources.resource(self.shown(surface)?)?;
        Some(resource.canonical_iri().to_owned())
    }

    /// How many surfaces show a resource.
    pub fn surface_count(&self) -> usize {
        self.shown.len()
    }

    fn emit(&self, delta: Delta) {
        if let Some(recorder) = &self.recorder {
            recorder(&delta);
        }
    }

    /// [`show`](Self::show) with nothing recorded: the writer that skips the
    /// capture, for the session's replay check to refuse.
    pub fn show_unrecorded(&mut self, surface: Uuid, iri: &str) -> Uuid {
        let recorder = self.recorder.take();
        let id = self.show(surface, iri);
        self.recorder = recorder;
        id
    }

    /// Show the resource named `iri` on `surface`, writing the resource first
    /// when the store lacks it.
    pub fn show(&mut self, surface: Uuid, iri: &str) -> Uuid {
        let resource = ResourceNode::for_term(iri);
        let id = resource.id();
        if self.resources.resource(id).is_none() {
            let record = PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().to_owned(),
                facets: Vec::new(),
            };
            let written = self.resources.put_record(id, record).expect("a new record");
            self.emit(JournalDelta::Resource(written));
        }
        let delta = SurfaceDelta::ShowResource {
            surface_id: surface,
            resource_id: id,
        };
        self.apply_surface(&delta);
        self.emit(JournalDelta::Surface(delta));
        id
    }

    fn pair(&self, from: Uuid, to: Uuid) -> (Uuid, Uuid) {
        (
            self.shown(from).expect("from shows a resource"),
            self.shown(to).expect("to shows a resource"),
        )
    }

    /// The live payloads between resources `from` and `to`, session-only
    /// arrangements included, or one empty payload to write into.
    fn payloads(&self, from: Uuid, to: Uuid) -> Vec<EdgePayload> {
        let held: Vec<EdgePayload> = self
            .resources
            .resource_edges()
            .filter(|(a, b, _)| (a.id(), b.id()) == (from, to))
            .map(|(_, _, payload)| payload.clone())
            .collect();
        if held.is_empty() {
            vec![EdgePayload::new()]
        } else {
            held
        }
    }

    /// Assert `predicate` between the resources two surfaces show, through
    /// the store; the statement's id.
    pub fn assert(&mut self, from: Uuid, to: Uuid, predicate: &str) -> String {
        let (a, b) = self.pair(from, to);
        let mut payloads = self.payloads(a, b);
        let statement = payloads[0].assert_semantic_statement(SemanticStatementSpec {
            predicate: predicate.into(),
            provenance_iri: Some("urn:p8:probe".into()),
            ..Default::default()
        });
        let written = self
            .resources
            .replace_edges(a, b, payloads)
            .expect("a new statement");
        self.emit(JournalDelta::Resource(written));
        statement.statement_id
    }

    /// Withdraw statement `id` between the resources two surfaces show.
    pub fn withdraw(&mut self, from: Uuid, to: Uuid, id: &str, at_ms: u64) -> bool {
        let (a, b) = self.pair(from, to);
        let mut payloads = self.payloads(a, b);
        let withdrawn = payloads.iter_mut().any(|payload| {
            payload.withdraw_semantic_statement(
                id,
                AssertionWithdrawal {
                    at_ms: Some(at_ms),
                    by: None,
                },
            )
        });
        if let Some(written) = withdrawn
            .then(|| self.resources.replace_edges(a, b, payloads))
            .flatten()
        {
            self.emit(JournalDelta::Resource(written));
            return true;
        }
        false
    }

    fn apply_surface(&mut self, delta: &SurfaceDelta) -> bool {
        self.observed += 1;
        match delta {
            SurfaceDelta::ShowResource {
                surface_id,
                resource_id,
            } => {
                self.resources.resource(*resource_id).is_some()
                    && self.shown.insert(*surface_id, *resource_id) != Some(*resource_id)
            },
            SurfaceDelta::HideSurface { surface_id } => self.shown.remove(surface_id).is_some(),
            SurfaceDelta::SetFacet {
                surface_id,
                facet,
                value,
            } => self
                .facets
                .set(
                    *surface_id,
                    FacetId::new(facet.as_str()),
                    value.clone(),
                    &AcceptAll,
                )
                .is_ok(),
            SurfaceDelta::Retired { .. } => false,
        }
    }

    /// Set facet `facet` on `surface`, recorded.
    pub fn set_facet(&mut self, surface: Uuid, facet: &str, value: serde_json::Value) {
        let delta = SurfaceDelta::SetFacet {
            surface_id: surface,
            facet: facet.to_owned(),
            value,
        };
        self.apply_surface(&delta);
        self.emit(JournalDelta::Surface(delta));
    }

    /// The facets, for a writer that records nothing.
    pub fn facets_mut(&mut self) -> &mut FacetStore<Uuid> {
        &mut self.facets
    }

    /// Apply one delta as replay does: a store write or a surface write.
    fn apply_delta(&mut self, delta: &Delta) -> Result<(), ContentError> {
        match delta {
            JournalDelta::Resource(ResourceDelta::ReplaySetResourceRecordById {
                resource_id,
                record,
            }) => {
                let id = uuid(resource_id)?;
                match record {
                    Some(record) => {
                        self.resources.put_record(id, record.clone());
                        self.resources
                            .resource(id)
                            .map(|_| ())
                            .ok_or(ContentError::MissingResource)
                    },
                    None => {
                        self.resources.remove_record(id);
                        Ok(())
                    },
                }
            },
            JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                edges,
            }) => {
                let (from, to) = (uuid(from_resource_id)?, uuid(to_resource_id)?);
                if self.resources.resource(from).is_none() || self.resources.resource(to).is_none()
                {
                    return Err(ContentError::MissingResource);
                }
                if let Some(taken) = self.handle_held_elsewhere(from, to, edges) {
                    return Err(ContentError::HandleCollision(taken));
                }
                self.resources.replace_edges(
                    from,
                    to,
                    edges.iter().map(payload_from_persisted).collect(),
                );
                Ok(())
            },
            JournalDelta::Surface(delta) => {
                self.apply_surface(delta);
                Ok(())
            },
        }
    }

    /// The rows the resources surfaces `from` and `to` show hold between them.
    pub fn rows_between(&self, from: Uuid, to: Uuid) -> Vec<PersistedEdge> {
        let (a, b) = self.pair(from, to);
        self.resources.persisted_edges_between(a, b)
    }
}

/// Ordered so a revert writes resources before the surfaces showing them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PartKey {
    Resource(Uuid),
    Pair(Uuid, Uuid),
    Surface(Uuid),
}

impl PartKey {
    fn of(delta: &Delta) -> Option<Self> {
        Some(match delta {
            JournalDelta::Resource(ResourceDelta::ReplaySetResourceRecordById {
                resource_id,
                ..
            }) => Self::Resource(uuid(resource_id).ok()?),
            JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                ..
            }) => Self::Pair(uuid(from_resource_id).ok()?, uuid(to_resource_id).ok()?),
            JournalDelta::Surface(
                SurfaceDelta::ShowResource { surface_id, .. }
                | SurfaceDelta::HideSurface { surface_id },
            ) => Self::Surface(*surface_id),
            JournalDelta::Surface(SurfaceDelta::SetFacet { .. } | SurfaceDelta::Retired { .. }) => {
                return None;
            },
        })
    }

    fn part(&self) -> Part {
        match self {
            Self::Surface(id) => Part::Surface(*id),
            Self::Resource(id) => Part::Resource(*id),
            Self::Pair(a, b) => Part::Pair(*a, *b),
        }
    }

    fn of_part(part: &Part) -> Self {
        match part {
            Part::Surface(id) => Self::Surface(*id),
            Part::Resource(id) => Self::Resource(*id),
            Part::Pair(a, b) => Self::Pair(*a, *b),
        }
    }
}
