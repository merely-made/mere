// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A minimal session graph with no kernel (engine split D245): surfaces that
//! each show a resource, and assertions written through the engine's
//! [`ResourceStore`]. Its journal holds [`JournalDelta`]s of its own two-variant
//! [`SurfaceDelta`] beside the store's [`ResourceDelta`]s, on one log. Its
//! hooks are pandect's for any host graph (engine split D257): a publication
//! that names the surface's snapshot by its hash.

// Each test binary uses part of the surface.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use mere::ResourceGraph;
use mere::model::edge_decode::payload_from_persisted;
use mere::model::resource_rules::ContentError;
use mere::model::{
    AssertionWithdrawal, Attributed, Author, CoverageNote, EdgePayload, JournalDelta,
    PersistedResourceRecord, ResourceDelta, ResourceNode, ResourceStore, SemanticStatementSpec,
};
use mere::projection::{ProjectionKey, ProjectionSource, RdfSource, SourceQuads};
use mere::session::{OnRefusal, Recorder, ReplayRefused, SessionGraph};
use muniment::{Backend, WriteOp};
use oxrdf::Quad;
use pandect::SessionError;
use pandect::graph_session::{Publication, PublicationHook, PublishedGeneration, SessionBackend};
use portolan::FacetStore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The surface's own writes: which resource a surface shows.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SurfaceDelta {
    ShowResource { surface_id: Uuid, resource_id: Uuid },
    HideSurface { surface_id: Uuid },
}

pub type Delta = JournalDelta<SurfaceDelta>;

/// The surface graph: a surface-to-resource map over the engine store.
#[derive(Default)]
pub struct Surface {
    resources: ResourceStore,
    shown: BTreeMap<Uuid, Uuid>,
    facets: FacetStore<Uuid>,
    coverage: CoverageNote,
    recorder: Option<Recorder<Delta>>,
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
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Surface(Uuid),
    Resource(Uuid),
    Pair(Uuid, Uuid),
}

fn uuid(text: &str) -> Result<Uuid, ContentError> {
    Uuid::parse_str(text).map_err(|_| ContentError::MissingResource)
}

/// A surface the publication hook refuses to publish while it shows anything.
pub const UNPUBLISHABLE: Uuid = Uuid::from_u128(0xdead);

/// Where the surface's publication keeps its head, per container.
pub fn head_key(container: Uuid) -> String {
    format!("surfaces/{container}/head")
}

/// The generation the surface publishes as: its snapshot's hash.
pub fn generation_of(graph: &Surface) -> String {
    let bytes = serde_json::to_vec(&graph.to_snapshot()).expect("serializable snapshot");
    blake3::hash(&bytes).to_hex().to_string()
}

fn published(backend: SessionBackend<'_>, container: Uuid) -> PublishedGeneration<'_> {
    Box::pin(async move {
        let head = backend
            .get(&head_key(container))
            .await?
            .ok_or_else(|| SessionError::Corrupt("no surface publication".into()))?;
        String::from_utf8(head).map_err(|error| SessionError::Corrupt(error.to_string()))
    })
}

/// The surface's publication hook: one head write naming the snapshot.
pub static PUBLICATION: PublicationHook<Surface> = PublicationHook {
    check: |graph| match graph.shown(UNPUBLISHABLE) {
        Some(_) => Err(SessionError::Validation(
            "surface refuses publication".into(),
        )),
        None => Ok(()),
    },
    prepare: |graph, _backend, container| {
        let generation = generation_of(graph);
        let (key, value) = (head_key(container), generation.clone().into_bytes());
        Ok(Publication {
            generation,
            writes: Box::new(move |_reader| vec![WriteOp::Put { key, value }]),
        })
    },
    revision: |graph, _backend, _container| Ok(generation_of(graph)),
    published,
};

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

    fn emit(&self, delta: Delta) {
        if let Some(recorder) = &self.recorder {
            recorder(&delta);
        }
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

    fn payloads(&self, from: Uuid, to: Uuid) -> Vec<EdgePayload> {
        let held = self.resources.persisted_edges_between(from, to);
        if held.is_empty() {
            vec![EdgePayload::new()]
        } else {
            held.iter().map(payload_from_persisted).collect()
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
        }
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

    fn replay_all<'a>(
        &mut self,
        deltas: impl IntoIterator<Item = &'a Delta>,
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused> {
        let recorder = self.recorder.take();
        let mut outcome = Ok(());
        for (position, delta) in deltas.into_iter().enumerate() {
            if let Err(reason) = self.apply_delta(delta)
                && on_refusal == OnRefusal::Stop
            {
                outcome = Err(ReplayRefused { position, reason });
                break;
            }
        }
        self.recorder = recorder;
        outcome
    }

    fn touched(edits: &[Delta]) -> BTreeSet<PartKey> {
        edits.iter().filter_map(PartKey::of).collect()
    }

    fn state_of(&self, part: &PartKey) -> String {
        let value = match part {
            PartKey::Surface(id) => serde_json::to_string(&self.shown(*id)),
            PartKey::Resource(id) => serde_json::to_string(&self.resources.record_delta(*id)),
            PartKey::Pair(a, b) => serde_json::to_string(&self.resources.pair_delta(*a, *b)),
        };
        value.expect("serializable state")
    }

    fn restore(&self, part: &PartKey) -> Delta {
        match part {
            PartKey::Surface(id) => JournalDelta::Surface(match self.shown(*id) {
                Some(resource_id) => SurfaceDelta::ShowResource {
                    surface_id: *id,
                    resource_id,
                },
                None => SurfaceDelta::HideSurface { surface_id: *id },
            }),
            PartKey::Resource(id) => JournalDelta::Resource(self.resources.record_delta(*id)),
            PartKey::Pair(a, b) => JournalDelta::Resource(self.resources.pair_delta(*a, *b)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PartKey {
    Surface(Uuid),
    Resource(Uuid),
    Pair(Uuid, Uuid),
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

impl RdfSource for Surface {
    fn held_quads(&self) -> Box<dyn Iterator<Item = Quad> + '_> {
        RdfSource::held_quads(&*self.resources)
    }

    fn withdrawn_reifier_quads(&self) -> Vec<Quad> {
        RdfSource::withdrawn_reifier_quads(&*self.resources)
    }

    fn projection_key(&self) -> ProjectionKey {
        RdfSource::projection_key(&*self.resources)
    }

    fn sources(&self) -> Option<Vec<ProjectionSource>> {
        RdfSource::sources(&*self.resources)
    }

    fn source_quads(&self, source: ProjectionSource) -> Option<SourceQuads> {
        RdfSource::source_quads(&*self.resources, source)
    }

    fn dependents(&self, source: ProjectionSource) -> Vec<ProjectionSource> {
        RdfSource::dependents(&*self.resources, source)
    }

    fn resource_graph(&self) -> &ResourceGraph {
        &self.resources
    }
}

impl SessionGraph for Surface {
    type Delta = Delta;
    type Edit = Delta;
    type Snapshot = SurfaceSnapshot;
    type Held = ();
    type Part = Part;
    type Carry = ();

    fn to_snapshot(&self) -> SurfaceSnapshot {
        let mut resources: Vec<ResourceDelta> = self
            .resources
            .resource_nodes()
            .map(|resource| self.resources.record_delta(resource.id()))
            .collect();
        let pairs: BTreeSet<(Uuid, Uuid)> = self
            .resources
            .resource_edges()
            .map(|(from, to, _)| (from.id(), to.id()))
            .collect();
        resources.sort_by_key(|delta| serde_json::to_string(delta).expect("serializable"));
        resources.extend(
            pairs
                .into_iter()
                .map(|(a, b)| self.resources.pair_delta(a, b)),
        );
        SurfaceSnapshot {
            shown: self.shown.iter().map(|(s, r)| (*s, *r)).collect(),
            resources,
        }
    }

    fn from_snapshot(snapshot: &SurfaceSnapshot, _recorded: bool) -> Result<Self, String> {
        let mut graph = Self::default();
        let deltas: Vec<Delta> = snapshot
            .resources
            .iter()
            .cloned()
            .map(JournalDelta::Resource)
            .chain(snapshot.shown.iter().map(|(surface_id, resource_id)| {
                JournalDelta::Surface(SurfaceDelta::ShowResource {
                    surface_id: *surface_id,
                    resource_id: *resource_id,
                })
            }))
            .collect();
        graph
            .replay_all(&deltas, OnRefusal::Stop)
            .map_err(|refused| refused.to_string())?;
        Ok(graph)
    }

    fn facets(&self) -> &FacetStore<Uuid> {
        &self.facets
    }

    fn install_facets(&mut self, facets: FacetStore<Uuid>) -> Result<(), String> {
        self.facets = facets;
        Ok(())
    }

    fn carry_from(_baseline: &Self) {}

    fn replay_carried(
        &mut self,
        _carry: &mut (),
        entries: &[Attributed<Delta>],
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused> {
        self.replay_all(entries.iter().map(|entry| &entry.delta), on_refusal)
    }

    fn repair_checkpoint(_carry: &(), _graph: &mut Self) {}

    fn replay_tail(&mut self, entries: &[Attributed<Delta>]) -> Result<(), ReplayRefused> {
        self.replay_all(entries.iter().map(|entry| &entry.delta), OnRefusal::Stop)
    }

    fn replay_as(&mut self, _author: &Author, deltas: &[Delta]) {
        let _ = self.replay_all(deltas, OnRefusal::Drop);
    }

    fn replay_matches(replay: &Self, candidate: &Self) -> Result<bool, String> {
        Ok(replay.to_snapshot() == candidate.to_snapshot() && replay.facets == candidate.facets)
    }

    fn edit_of(delta: &Delta, _author: &Author) -> Result<Delta, String> {
        Ok(delta.clone())
    }

    fn apply(&mut self, edit: Delta) -> Result<(), String> {
        self.apply_delta(&edit)
            .map_err(|reason| reason.to_string())?;
        self.emit(edit);
        Ok(())
    }

    fn set_recorder(&mut self, recorder: Option<Recorder<Delta>>) {
        self.recorder = recorder;
    }

    fn write_as<R>(&mut self, _author: Author, write: impl FnOnce(&mut Self) -> R) -> R {
        write(self)
    }

    fn revert(
        change: &[Delta],
        before: &Self,
        after: &Self,
        live: &Self,
    ) -> (Vec<Delta>, Vec<Part>) {
        let (mut edits, mut kept) = (Vec::new(), Vec::new());
        for part in Self::touched(change) {
            if live.state_of(&part) == after.state_of(&part) {
                edits.push(before.restore(&part));
            } else {
                kept.push(part.part());
            }
        }
        (edits, kept)
    }

    fn reaches(edits: &[Delta], part: &Part) -> bool {
        Self::touched(edits).contains(&PartKey::of_part(part))
    }

    fn current_session(&self) -> u64 {
        self.launch
    }

    fn set_current_session(&mut self, launch: u64) {
        self.launch = launch;
    }

    fn set_browsing_idle_gap_ms(&mut self, _idle_gap_ms: u64) {}

    fn observation_revision(&self) -> u64 {
        self.observed
    }

    fn clear_semantic_context(&mut self) {}

    fn copy_semantic_context_from(&mut self, _other: &Self) {}

    fn held(&self) {}

    fn restore_held(&mut self, _held: ()) {}

    fn retained(_held: ()) {}

    fn known_coverage(&self) -> &CoverageNote {
        &self.coverage
    }

    fn set_known_coverage(&mut self, note: CoverageNote) {
        self.coverage = note;
    }

    fn resources(&self) -> &ResourceStore {
        &self.resources
    }

    fn validate_handles(&self) -> Result<(), String> {
        Ok(())
    }
}
