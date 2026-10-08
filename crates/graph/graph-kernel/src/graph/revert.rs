// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reverting one change: the edits that put back what a change altered,
//! leaving alone whatever changed after it.
//!
//! Reservoir plan V2, §7 items 17 and 18. A change's own edits name what it
//! touched: nodes, the relations from one node to another, the field layer and
//! the import records. For each such part, the graph just before the change,
//! just after it, and now are compared. A part that still reads as the change
//! left it goes back to how it stood before; a part changed since is kept and
//! reported. The edits are ordinary ones (a retitle, a relation set back), so a
//! Timeline reads an undo like any other change. Previews (node images) are
//! experience, not truth, and are left alone.

use std::collections::{BTreeMap, BTreeSet};

use euclid::default::Point2D;
use serde_json::Value;
use uuid::Uuid;

use super::apply::{GraphDelta, apply_graph_delta};
use super::capture::{CapturedDelta, persisted_coupling_from_coupling, persisted_field_from_field};
use super::{CouplingId, FieldId, Graph, NodeKey};
use crate::persistence::{PersistedEdge, PersistedResourceFacet, PersistedResourceRecord};

/// One part of the graph a change can alter.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Part {
    /// A node as a whole: made or removed by the change.
    Node(Uuid),
    Title(Uuid),
    Url(Uuid),
    Tag(Uuid, String),
    Body(Uuid),
    Content(Uuid),
    MediaType(Uuid),
    Nested(Uuid),
    Facet(Uuid, String),
    History(Uuid),
    /// Every relation from the first node to the second.
    Edges(Uuid, Uuid),
    Field(Uuid),
    Coupling(Uuid),
    ImportRecords,
    Resource(Uuid),
    ResourceFacet(Uuid, String),
    ResourceEdges(Uuid, Uuid),
    ShownResource(Uuid),
}

/// The edits that revert a change, and the parts left as they are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Revert {
    pub edits: Vec<CapturedDelta>,
    /// Parts changed after the change, kept rather than reverted.
    pub kept: Vec<Part>,
}

/// What a set of edits names: nodes, node pairs, and the graph-level stores.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Touched {
    pub nodes: BTreeSet<Uuid>,
    pub edges: BTreeSet<(Uuid, Uuid)>,
    pub fields: BTreeSet<Uuid>,
    pub couplings: BTreeSet<Uuid>,
    /// Fields whose couplings were retuned as a group.
    pub coupled_fields: BTreeSet<Uuid>,
    pub import_records: bool,
    pub resources: BTreeSet<Uuid>,
    pub resource_edges: BTreeSet<(Uuid, Uuid)>,
    pub shown_surfaces: BTreeSet<Uuid>,
}

fn uuid(text: &str) -> Option<Uuid> {
    Uuid::parse_str(text).ok()
}

impl Touched {
    /// What `edits` name.
    pub fn of<'a>(edits: impl IntoIterator<Item = &'a CapturedDelta>) -> Self {
        let mut touched = Self::default();
        for edit in edits {
            touched.add(edit);
        }
        touched
    }

    fn node(&mut self, id: &str) {
        self.nodes.extend(uuid(id));
    }

    fn edge(&mut self, from: &str, to: &str) {
        if let (Some(from), Some(to)) = (uuid(from), uuid(to)) {
            self.edges.insert((from, to));
        }
    }

    // Exhaustive on purpose: a new edit kind must say what it touches.
    fn add(&mut self, edit: &CapturedDelta) {
        use CapturedDelta as D;
        match edit {
            D::ReplayAddNodeWithIdIfMissing { id, .. } => self.node(id),
            D::ReplayRemoveNodeById { node_id }
            | D::ReplaySetNodeTitleById { node_id, .. }
            | D::ReplaySetNodeUrlById { node_id, .. }
            | D::ReplaySetNodeImageById { node_id, .. }
            | D::ReplaySetNodeThumbnailById { node_id, .. }
            | D::ReplaySetNodeFaviconById { node_id, .. }
            | D::ReplaySetNodeMimeHintById { node_id, .. }
            | D::ReplaySetNodeNestedById { node_id, .. }
            | D::ReplaySetNodePinnedById { node_id, .. }
            | D::ReplaySetNodeFacetById { node_id, .. }
            | D::ReplayRemoveNodeFacetById { node_id, .. }
            | D::ReplayInsertNodeTagById { node_id, .. }
            | D::ReplayRemoveNodeTagById { node_id, .. }
            | D::ReplaySetNodeBodyById { node_id, .. }
            | D::ReplayNavigateNodeById { node_id, .. }
            | D::ReplayNodeHistoryBackById { node_id, .. }
            | D::ReplayNodeHistoryForwardById { node_id, .. }
            | D::ReplayAppendNodePropertyById { node_id, .. }
            | D::ReplayAddNodeClassificationById { node_id, .. }
            | D::ReplayRemoveNodeClassificationById { node_id, .. }
            | D::ReplaySetNodeClassificationStatusById { node_id, .. }
            | D::ReplaySetNodePrimaryClassificationById { node_id, .. }
            | D::ReplayRecordNodeDerivationById { node_id, .. }
            | D::ReplaySetNodeTagIconOverrideById { node_id, .. }
            | D::ReplayAppendFrameLayoutHintById { node_id, .. }
            | D::ReplayRemoveFrameLayoutHintById { node_id, .. }
            | D::ReplayMoveFrameLayoutHintById { node_id, .. }
            | D::ReplaySetFrameSplitOfferSuppressedById { node_id, .. }
            | D::ReplayUpdateNodeHistoryById { node_id, .. }
            | D::ReplayTouchNodeLastVisitedById { node_id, .. }
            | D::ReplaySetNodeContentById { node_id, .. } => self.node(node_id),
            D::ReplayBranchHistoryByIds {
                child_id,
                parent_id,
            } => {
                self.node(child_id);
                self.node(parent_id);
            },
            D::ReplayAssertRelationByIds { from_id, to_id, .. }
            | D::ReplayRetractRelationsByIds { from_id, to_id, .. }
            | D::ReplayAppendTraversalByIds { from_id, to_id, .. }
            | D::ReplaySetEdgeSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplayAssertSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplaySetEdgesByIds { from_id, to_id, .. } => self.edge(from_id, to_id),
            D::ReplaySetImportRecords { .. } => self.import_records = true,
            D::ReplayAddField { field } => self.fields.extend(uuid(&field.id)),
            D::ReplayRetireFieldById { field_id }
            | D::ReplayActivateFieldById { field_id }
            | D::ReplayRemoveFieldById { field_id } => self.fields.extend(uuid(field_id)),
            D::ReplaySetFieldCouplingStrengthByFieldId { field_id, .. } => {
                self.coupled_fields.extend(uuid(field_id));
            },
            D::ReplayAddCoupling { coupling } => self.couplings.extend(uuid(&coupling.id)),
            D::ReplayRetractCouplingById { coupling_id } => {
                self.couplings.extend(uuid(coupling_id));
            },
            D::ReplaySetResourceRecordById { resource_id, .. } => {
                self.resources.extend(uuid(resource_id));
            },
            D::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                ..
            } => {
                if let (Some(from), Some(to)) = (uuid(from_resource_id), uuid(to_resource_id)) {
                    self.resource_edges.insert((from, to));
                }
            },
            D::ReplaySetShownResourceById {
                surface_id,
                resource_id,
            } => {
                self.shown_surfaces.extend(uuid(surface_id));
                self.resources.extend(resource_id.as_deref().and_then(uuid));
            },
        }
    }

    /// Whether these edits reach `part`: how a caller learns who changed a
    /// part that undo kept.
    pub fn reaches(&self, part: &Part) -> bool {
        match part {
            Part::Node(id)
            | Part::Title(id)
            | Part::Url(id)
            | Part::Tag(id, _)
            | Part::Body(id)
            | Part::Content(id)
            | Part::MediaType(id)
            | Part::Nested(id)
            | Part::Facet(id, _)
            | Part::History(id) => {
                self.nodes.contains(id)
                    || self.shown_surfaces.contains(id)
                    || self.edges.iter().any(|(from, to)| from == id || to == id)
            },
            Part::Edges(from, to) => {
                self.edges.contains(&(*from, *to))
                    || self.nodes.contains(from)
                    || self.nodes.contains(to)
            },
            Part::Field(id) => self.fields.contains(id),
            Part::Coupling(id) => self.couplings.contains(id) || !self.coupled_fields.is_empty(),
            Part::ImportRecords => self.import_records,
            Part::Resource(id) | Part::ResourceFacet(id, _) => {
                self.resources.contains(id)
                    || self
                        .resource_edges
                        .iter()
                        .any(|(from, to)| from == id || to == id)
            },
            Part::ResourceEdges(from, to) => {
                self.resource_edges.contains(&(*from, *to))
                    || self.resources.contains(from)
                    || self.resources.contains(to)
            },
            Part::ShownResource(id) => self.shown_surfaces.contains(id) || self.nodes.contains(id),
        }
    }
}

/// A node's revertible state, part by part.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NodeState {
    pub(super) title: String,
    pub(super) url: String,
    pub(super) tags: BTreeSet<String>,
    pub(super) body: Option<String>,
    pub(super) content: Option<[u8; 32]>,
    pub(super) media_type: Option<String>,
    pub(super) nested: Option<String>,
    pub(super) facets: BTreeMap<String, Value>,
    pub(super) history: (Vec<String>, usize),
}

pub(super) fn node_state(graph: &Graph, id: Uuid) -> Option<NodeState> {
    let (key, node) = graph.get_node_by_id(id)?;
    let history = graph.node_history_projection(key);
    Some(NodeState {
        title: node.title.clone(),
        url: node.primary_address().as_url_str().to_string(),
        tags: node.tags.iter().cloned().collect(),
        body: node.body.clone(),
        content: node.content.map(|hash| *hash.as_bytes()),
        media_type: node.media_type.clone(),
        nested: node.nested.as_ref().map(|log| log.as_str().to_string()),
        facets: graph
            .facets()
            .facets_of(&id)
            .map(|facets| {
                facets
                    .iter()
                    .map(|(facet, value)| (facet.as_str().to_string(), value.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        history: (history.entries, history.current_index),
    })
}

/// Every relation from `from` to `to`, in persisted form: compared as a
/// whole and written back as a whole.
pub(super) fn edges_between(graph: &Graph, from: Uuid, to: Uuid) -> Vec<PersistedEdge> {
    match (graph.get_node_by_id(from), graph.get_node_by_id(to)) {
        (Some((from, _)), Some((to, _))) => graph.persisted_edges_between(from, to),
        _ => Vec::new(),
    }
}

/// The pairs of nodes `id` has relations with, either way round.
fn incident_pairs(graph: &Graph, id: Uuid) -> Vec<(Uuid, Uuid)> {
    let Some((key, _)) = graph.get_node_by_id(id) else {
        return Vec::new();
    };
    let id_of = |key: NodeKey| graph.get_node(key).map(|node| node.id);
    graph
        .outgoing_relations(key)
        .chain(graph.incoming_relations(key))
        .filter_map(|relation| Some((id_of(relation.from)?, id_of(relation.to)?)))
        .collect()
}

fn shown_resource(graph: &Graph, surface: Uuid) -> Option<Uuid> {
    graph
        .get_node_by_id(surface)
        .and_then(|(key, _)| graph.shown_resource_id(key))
}

fn resource_incident_pairs(graph: &Graph, id: Uuid) -> Vec<(Uuid, Uuid)> {
    use petgraph::visit::EdgeRef;
    let Some(key) = graph.resources.key_of(&id) else {
        return Vec::new();
    };
    graph
        .resources
        .inner()
        .edges_directed(key, petgraph::Direction::Outgoing)
        .chain(
            graph
                .resources
                .inner()
                .edges_directed(key, petgraph::Direction::Incoming),
        )
        .filter_map(|edge| {
            Some((
                graph.resources.node(edge.source())?.id(),
                graph.resources.node(edge.target())?.id(),
            ))
        })
        .collect()
}

fn resource_facets(record: &PersistedResourceRecord) -> BTreeMap<String, Value> {
    record
        .facets
        .iter()
        .map(|facet| {
            (
                facet.facet.clone(),
                serde_json::from_str(&facet.value_json)
                    .expect("stored resource facets are valid JSON"),
            )
        })
        .collect()
}

impl Revert {
    fn keep(&mut self, part: Part) {
        self.kept.push(part);
    }

    fn resource_parts(
        &mut self,
        id: Uuid,
        before: &PersistedResourceRecord,
        after: &PersistedResourceRecord,
        live: &PersistedResourceRecord,
    ) {
        let was = resource_facets(before);
        let became = resource_facets(after);
        let mut now = resource_facets(live);
        let keys: BTreeSet<_> = was.keys().chain(became.keys()).collect();
        let mut changed = false;
        for key in keys {
            if was.get(key) == became.get(key) {
                continue;
            }
            if now.get(key) != became.get(key) {
                self.keep(Part::ResourceFacet(id, key.clone()));
                continue;
            }
            match was.get(key) {
                Some(value) => {
                    now.insert(key.clone(), value.clone());
                },
                None => {
                    now.remove(key);
                },
            }
            changed = true;
        }
        if changed {
            self.edits.push(CapturedDelta::ReplaySetResourceRecordById {
                resource_id: id.to_string(),
                record: Some(PersistedResourceRecord {
                    canonical_iri: live.canonical_iri.clone(),
                    facets: now
                        .into_iter()
                        .map(|(facet, value)| PersistedResourceFacet {
                            facet,
                            value_json: value.to_string(),
                        })
                        .collect(),
                }),
            });
        }
    }

    /// Revert one part: `changed` if the change altered it, `untouched` if it
    /// still reads as the change left it.
    fn part(&mut self, part: Part, changed: bool, untouched: bool, edit: CapturedDelta) {
        if !changed {
            return;
        }
        if untouched {
            self.edits.push(edit);
        } else {
            self.keep(part);
        }
    }

    /// Put back each node part the change altered and nobody touched since.
    fn node_parts(
        &mut self,
        id: Uuid,
        before: &NodeState,
        after: &NodeState,
        live: &NodeState,
        recreating: bool,
    ) {
        let node_id = id.to_string();
        self.part(
            Part::Title(id),
            before.title != after.title,
            live.title == after.title,
            CapturedDelta::ReplaySetNodeTitleById {
                node_id: node_id.clone(),
                title: before.title.clone(),
            },
        );
        self.part(
            Part::Url(id),
            before.url != after.url,
            live.url == after.url,
            CapturedDelta::ReplaySetNodeUrlById {
                node_id: node_id.clone(),
                new_url: before.url.clone(),
            },
        );
        self.part(
            Part::Body(id),
            before.body != after.body,
            live.body == after.body,
            CapturedDelta::ReplaySetNodeBodyById {
                node_id: node_id.clone(),
                body: before.body.clone(),
            },
        );
        self.part(
            Part::Content(id),
            before.content != after.content,
            live.content == after.content,
            CapturedDelta::ReplaySetNodeContentById {
                node_id: node_id.clone(),
                content: before.content,
            },
        );
        self.part(
            Part::MediaType(id),
            before.media_type != after.media_type,
            live.media_type == after.media_type,
            CapturedDelta::ReplaySetNodeMimeHintById {
                node_id: node_id.clone(),
                mime_hint: before.media_type.clone(),
            },
        );
        self.part(
            Part::Nested(id),
            before.nested != after.nested,
            live.nested == after.nested,
            CapturedDelta::ReplaySetNodeNestedById {
                node_id: node_id.clone(),
                nested: before.nested.clone(),
            },
        );
        for tag in before.tags.symmetric_difference(&after.tags) {
            let untouched = live.tags.contains(tag) == after.tags.contains(tag);
            let edit = if before.tags.contains(tag) {
                CapturedDelta::ReplayInsertNodeTagById {
                    node_id: node_id.clone(),
                    tag: tag.clone(),
                }
            } else {
                CapturedDelta::ReplayRemoveNodeTagById {
                    node_id: node_id.clone(),
                    tag: tag.clone(),
                }
            };
            self.part(Part::Tag(id, tag.clone()), true, untouched, edit);
        }
        let facet_ids: BTreeSet<&String> =
            before.facets.keys().chain(after.facets.keys()).collect();
        for facet in facet_ids {
            let (was, became) = (before.facets.get(facet), after.facets.get(facet));
            let edit = match was {
                Some(value) => CapturedDelta::ReplaySetNodeFacetById {
                    node_id: node_id.clone(),
                    facet: facet.clone(),
                    value_json: value.to_string(),
                },
                None => CapturedDelta::ReplayRemoveNodeFacetById {
                    node_id: node_id.clone(),
                    facet: facet.clone(),
                },
            };
            self.part(
                Part::Facet(id, facet.clone()),
                recreating || was != became,
                live.facets.get(facet) == became,
                edit,
            );
        }
        self.part(
            Part::History(id),
            before.history != after.history,
            live.history == after.history,
            CapturedDelta::ReplayUpdateNodeHistoryById {
                node_id,
                entries: before.history.0.clone(),
                current_index: before.history.1,
            },
        );
    }

    /// Make a node the change removed again, as it stood before.
    pub(super) fn recreate(&mut self, id: Uuid, before: &NodeState) {
        // What a node is born with, so every part below lands as it stood.
        let mut scratch = Graph::new();
        let _ = apply_graph_delta(
            &mut scratch,
            GraphDelta::ReplayAddNodeWithIdIfMissing {
                id,
                url: before.url.clone(),
                position: Point2D::new(0.0, 0.0),
            },
        );
        let born = node_state(&scratch, id).expect("the scratch node exists");
        self.recreate_from_birth_state(id, before, &born);
    }

    fn recreate_from_birth_state(&mut self, id: Uuid, before: &NodeState, born: &NodeState) {
        self.edits
            .push(CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: id.to_string(),
                url: before.url.clone(),
                position: [0.0, 0.0],
            });
        // Replay's birth clock can differ from the planner's; restore every facet.
        self.node_parts(id, before, born, born, true);
    }
}

/// The edits that revert `change`, given the graph just before it (`before`),
/// just after it (`after`) and now (`live`), and the parts kept because they
/// changed since.
pub fn revert_change(
    change: &[CapturedDelta],
    before: &Graph,
    after: &Graph,
    live: &Graph,
) -> Revert {
    let touched = Touched::of(change);
    let mut out = Revert::default();
    let mut resource_coming = BTreeSet::new();
    let mut resource_removals = BTreeSet::new();
    let mut resource_pairs = touched.resource_edges.clone();
    for id in &touched.resources {
        resource_pairs.extend(resource_incident_pairs(before, *id));
        resource_pairs.extend(resource_incident_pairs(after, *id));
        match (
            before.resource_record(*id),
            after.resource_record(*id),
            live.resource_record(*id),
        ) {
            (None, Some(became), Some(now)) => {
                if now == became {
                    resource_removals.insert(*id);
                } else {
                    out.keep(Part::Resource(*id));
                }
            },
            (Some(was), None, None) => {
                resource_coming.insert(*id);
                out.edits.push(CapturedDelta::ReplaySetResourceRecordById {
                    resource_id: id.to_string(),
                    record: Some(was),
                });
            },
            (Some(_), None, Some(_)) => out.keep(Part::Resource(*id)),
            (Some(was), Some(became), Some(now)) => out.resource_parts(*id, &was, &became, &now),
            (Some(was), Some(became), None) if was != became => out.keep(Part::Resource(*id)),
            _ => {},
        }
    }
    let mut pairs = touched.edges.clone();
    // A node made or removed takes its relations with it.
    for id in &touched.nodes {
        pairs.extend(incident_pairs(before, *id));
        pairs.extend(incident_pairs(after, *id));
    }

    let mut going = BTreeSet::new();
    let mut coming = BTreeSet::new();
    let mut removals = Vec::new();
    for id in &touched.nodes {
        match (
            node_state(before, *id),
            node_state(after, *id),
            node_state(live, *id),
        ) {
            // Made by the change: it goes, unless changed or linked since.
            (None, Some(became), Some(now)) => {
                let unlinked_since = incident_pairs(live, *id).into_iter().all(|(from, to)| {
                    edges_between(live, from, to) == edges_between(after, from, to)
                });
                if now == became
                    && unlinked_since
                    && shown_resource(live, *id) == shown_resource(after, *id)
                {
                    going.insert(*id);
                    removals.push(CapturedDelta::ReplayRemoveNodeById {
                        node_id: id.to_string(),
                    });
                } else {
                    out.keep(Part::Node(*id));
                }
            },
            // Removed by the change: it comes back, unless made again since.
            (Some(was), None, None) => {
                coming.insert(*id);
                out.recreate(*id, &was);
            },
            (Some(_), None, Some(_)) => out.keep(Part::Node(*id)),
            (Some(was), Some(became), Some(now)) => out.node_parts(*id, &was, &became, &now, false),
            // Removed by someone since the change.
            (Some(was), Some(became), None) => {
                if was != became {
                    out.keep(Part::Node(*id));
                }
            },
            (None, _, None) | (None, None, Some(_)) => {},
        }
    }

    let present = |id: &Uuid| {
        !going.contains(id) && (coming.contains(id) || live.get_node_by_id(*id).is_some())
    };
    for (from, to) in pairs {
        let (was, became) = (
            edges_between(before, from, to),
            edges_between(after, from, to),
        );
        if was == became || going.contains(&from) || going.contains(&to) {
            continue;
        }
        let untouched = edges_between(live, from, to) == became;
        if !untouched || !present(&from) || !present(&to) {
            out.keep(Part::Edges(from, to));
            continue;
        }
        out.edits.push(CapturedDelta::ReplaySetEdgesByIds {
            from_id: from.to_string(),
            to_id: to.to_string(),
            edges: was,
        });
    }
    for (from, to) in resource_pairs {
        let was = before.persisted_resource_edges_between(from, to);
        let became = after.persisted_resource_edges_between(from, to);
        if was == became {
            continue;
        }
        let present = |id| resource_coming.contains(&id) || live.resource_record(id).is_some();
        if live.persisted_resource_edges_between(from, to) != became
            || !present(from)
            || !present(to)
        {
            out.keep(Part::ResourceEdges(from, to));
            continue;
        }
        out.edits.push(CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: was,
        });
    }
    let shown_surfaces: BTreeSet<_> = touched
        .shown_surfaces
        .union(&touched.nodes)
        .copied()
        .collect();
    for id in shown_surfaces {
        let was = shown_resource(before, id);
        let became = shown_resource(after, id);
        if was == became {
            continue;
        }
        let surface_present = coming.contains(&id) || live.get_node_by_id(id).is_some();
        let resource_present = was.is_none_or(|resource| {
            resource_coming.contains(&resource) || live.resource_record(resource).is_some()
        });
        if shown_resource(live, id) != became || !surface_present || !resource_present {
            out.keep(Part::ShownResource(id));
            continue;
        }
        out.edits.push(CapturedDelta::ReplaySetShownResourceById {
            surface_id: id.to_string(),
            resource_id: was.map(|resource| resource.to_string()),
        });
    }
    out.edits.extend(removals);

    if touched.import_records {
        let (was, became) = (before.import_records(), after.import_records());
        if was != became {
            if live.import_records() == became {
                out.edits.push(CapturedDelta::ReplaySetImportRecords {
                    import_records: was.to_vec(),
                });
            } else {
                out.keep(Part::ImportRecords);
            }
        }
    }

    for id in &touched.fields {
        let state = |graph: &Graph| {
            graph
                .field(FieldId::from_uuid(*id))
                .map(persisted_field_from_field)
        };
        let (was, became) = (state(before), state(after));
        if was == became {
            continue;
        }
        if state(live) != became {
            out.keep(Part::Field(*id));
            continue;
        }
        out.edits.push(match was {
            Some(field) => CapturedDelta::ReplayAddField { field },
            None => CapturedDelta::ReplayRemoveFieldById {
                field_id: id.to_string(),
            },
        });
    }

    let mut couplings = touched.couplings.clone();
    for graph in [before, after] {
        couplings.extend(
            graph
                .couplings()
                .filter(|coupling| touched.coupled_fields.contains(&coupling.field.as_uuid()))
                .map(|coupling| coupling.id.as_uuid()),
        );
    }
    for id in &couplings {
        let state = |graph: &Graph| {
            graph
                .couplings()
                .find(|coupling| coupling.id == CouplingId::from_uuid(*id))
                .map(persisted_coupling_from_coupling)
        };
        let (was, became) = (state(before), state(after));
        if was == became {
            continue;
        }
        if state(live) != became {
            out.keep(Part::Coupling(*id));
            continue;
        }
        out.edits.push(match was {
            Some(coupling) => CapturedDelta::ReplayAddCoupling { coupling },
            None => CapturedDelta::ReplayRetractCouplingById {
                coupling_id: id.to_string(),
            },
        });
    }
    if !resource_removals.is_empty() {
        let mut scratch = live.clone();
        super::capture::replay_captured_deltas_onto(&mut scratch, out.edits.iter().cloned());
        for resource in resource_removals {
            if scratch.set_resource_record(resource, None) {
                out.edits.push(CapturedDelta::ReplaySetResourceRecordById {
                    resource_id: resource.to_string(),
                    record: None,
                });
            } else {
                out.keep(Part::Resource(resource));
            }
        }
    }
    out
}

#[cfg(test)]
pub(super) mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::graph::{
        Coupling, CouplingResponse, EdgeAssertion, Field, FieldDefinition, NavigationTrigger,
        NodeSelector, ScalarField, SemanticSubKind,
    };
    use crate::types::{ImportRecord, ImportRecordMembership};

    pub(in crate::graph) fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    pub(in crate::graph) fn add(n: u128) -> CapturedDelta {
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: id(n).to_string(),
            url: format!("https://{n}.test/"),
            position: [0.0, 0.0],
        }
    }

    pub(in crate::graph) fn title(n: u128, title: &str) -> CapturedDelta {
        CapturedDelta::ReplaySetNodeTitleById {
            node_id: id(n).to_string(),
            title: title.to_string(),
        }
    }

    pub(in crate::graph) fn tag(n: u128, tag: &str) -> CapturedDelta {
        CapturedDelta::ReplayInsertNodeTagById {
            node_id: id(n).to_string(),
            tag: tag.to_string(),
        }
    }

    pub(in crate::graph) fn facet(n: u128, facet: &str, value: &str) -> CapturedDelta {
        CapturedDelta::ReplaySetNodeFacetById {
            node_id: id(n).to_string(),
            facet: facet.to_string(),
            value_json: value.to_string(),
        }
    }

    pub(in crate::graph) fn relate(from: u128, to: u128) -> CapturedDelta {
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: id(from).to_string(),
            to_id: id(to).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: Some("cites".into()),
                decay_progress: None,
            },
        }
    }

    fn remove(n: u128) -> CapturedDelta {
        CapturedDelta::ReplayRemoveNodeById {
            node_id: id(n).to_string(),
        }
    }

    /// Run `edit` with a recorder on, and return what the graph recorded:
    /// the change as a journal would hold it.
    fn recorded(graph: &mut Graph, edit: impl FnOnce(&mut Graph)) -> Vec<CapturedDelta> {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        graph.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
            sink.lock().unwrap().push(delta.clone());
        })));
        edit(graph);
        graph.set_recorder(None);
        seen.lock().unwrap().clone()
    }

    pub(in crate::graph) fn apply_all(graph: &mut Graph, edits: &[CapturedDelta]) {
        for edit in edits {
            let _ = apply_graph_delta(graph, edit.replay_delta().expect("replayable"));
        }
    }

    type Nodes = Vec<(Uuid, Option<NodeState>)>;

    /// Every node's state and every relation, so two graphs compare whole.
    fn fingerprint(graph: &Graph) -> (Nodes, Vec<PersistedEdge>) {
        let mut ids: Vec<Uuid> = graph.nodes().map(|(_, node)| node.id).collect();
        ids.sort();
        let nodes = ids.iter().map(|n| (*n, node_state(graph, *n))).collect();
        let mut edges = Vec::new();
        for from in &ids {
            for to in &ids {
                edges.extend(edges_between(graph, *from, *to));
            }
        }
        (nodes, edges)
    }

    /// The graph before a change, the change as recorded, and the graph after.
    fn scene(
        base: &[CapturedDelta],
        edits: &[CapturedDelta],
    ) -> (Graph, Vec<CapturedDelta>, Graph) {
        let mut graph = Graph::new();
        apply_all(&mut graph, base);
        let before = graph.clone();
        let made = recorded(&mut graph, |graph| apply_all(graph, edits));
        (before, made, graph)
    }

    #[test]
    fn a_retitle_reverts_unless_someone_retitled_since() {
        let (before, made, after) = scene(&[add(1), title(1, "A")], &[title(1, "B")]);
        let mut live = after.clone();
        let revert = revert_change(&made, &before, &after, &live);
        assert_eq!(revert.edits, [title(1, "A")]);
        assert!(revert.kept.is_empty());
        apply_all(&mut live, &revert.edits);
        assert_eq!(fingerprint(&live), fingerprint(&before));

        let mut retitled = after.clone();
        apply_all(&mut retitled, &[title(1, "C")]);
        let revert = revert_change(&made, &before, &after, &retitled);
        assert!(revert.edits.is_empty());
        assert_eq!(revert.kept, [Part::Title(id(1))]);
    }

    #[test]
    fn a_made_node_goes_with_its_relations_unless_linked_since() {
        let (before, made, after) = scene(&[add(1)], &[add(2), title(2, "Two"), relate(1, 2)]);
        let mut live = after.clone();
        let revert = revert_change(&made, &before, &after, &live);
        assert_eq!(
            revert.edits,
            [remove(2)],
            "the node takes its relation with it"
        );
        apply_all(&mut live, &revert.edits);
        assert_eq!(fingerprint(&live), fingerprint(&before));

        let mut linked = after.clone();
        apply_all(&mut linked, &[add(3), relate(3, 2)]);
        let revert = revert_change(&made, &before, &after, &linked);
        assert!(
            revert.kept.contains(&Part::Node(id(2))),
            "{:?}",
            revert.kept
        );
        assert!(!revert.edits.contains(&remove(2)));
    }

    #[test]
    fn a_removed_node_comes_back_whole() {
        let base = [
            add(1),
            add(2),
            title(2, "Two"),
            tag(2, "kept"),
            facet(2, "custom.note", r#"{"text":"hello"}"#),
            relate(1, 2),
        ];
        let (before, made, after) = scene(&base, &[remove(2)]);
        let mut live = after.clone();
        let revert = revert_change(&made, &before, &after, &live);
        assert!(revert.kept.is_empty(), "{:?}", revert.kept);
        apply_all(&mut live, &revert.edits);
        assert_eq!(fingerprint(&live), fingerprint(&before));
    }

    #[test]
    fn recreation_restores_facets_when_the_birth_clock_changes() {
        for original_visit in [Some(90), None, Some(100)] {
            let mut before = Graph::new();
            apply_all(&mut before, &[add(1), add(2)]);
            let visit_facet = super::super::node_facets::VISIT_HISTORY;
            apply_all(
                &mut before,
                &[
                    facet(
                        1,
                        visit_facet,
                        r#"{"last_visited_ms":42,"last_session_visited":0}"#,
                    ),
                    facet(2, "custom.note", r#"{"text":"original"}"#),
                ],
            );
            let original = match original_visit {
                Some(timestamp) => facet(
                    2,
                    visit_facet,
                    &serde_json::json!({
                        "last_visited_ms": timestamp,
                        "last_session_visited": 0,
                    })
                    .to_string(),
                ),
                None => CapturedDelta::ReplayRemoveNodeFacetById {
                    node_id: id(2).to_string(),
                    facet: visit_facet.into(),
                },
            };
            apply_all(&mut before, &[original]);
            let was = node_state(&before, id(2)).unwrap();
            let mut scratch = Graph::new();
            apply_all(&mut scratch, &[add(2)]);
            let mut born = node_state(&scratch, id(2)).unwrap();
            born.facets.insert(
                visit_facet.into(),
                serde_json::json!({"last_visited_ms":100,"last_session_visited":0}),
            );
            let mut revert = Revert::default();
            revert.recreate_from_birth_state(id(2), &was, &born);

            let mut live = before.clone();
            apply_all(&mut live, &[remove(2)]);
            apply_all(&mut live, &revert.edits[..1]);
            // A later replay birth, even when original and planner both read 100.
            apply_all(
                &mut live,
                &[CapturedDelta::ReplayTouchNodeLastVisitedById {
                    node_id: id(2).to_string(),
                    timestamp_ms: 101,
                }],
            );
            apply_all(&mut live, &revert.edits[1..]);
            assert_eq!(
                fingerprint(&live),
                fingerprint(&before),
                "{original_visit:?}"
            );
            assert_eq!(
                node_state(&live, id(1)).unwrap().facets[visit_facet]["last_visited_ms"],
                42,
                "the untouched node keeps its own timestamp"
            );
        }
    }

    #[test]
    fn tags_and_facets_revert_part_by_part() {
        let (before, made, after) = scene(
            &[add(1), facet(1, "custom.a", "1")],
            &[
                tag(1, "new"),
                facet(1, "custom.a", "2"),
                facet(1, "custom.b", "true"),
            ],
        );
        let mut live = after.clone();
        // Another author changes custom.b afterwards; the rest still reverts.
        apply_all(&mut live, &[facet(1, "custom.b", "false")]);
        let revert = revert_change(&made, &before, &after, &live);
        assert_eq!(revert.kept, [Part::Facet(id(1), "custom.b".into())]);
        apply_all(&mut live, &revert.edits);
        let state = node_state(&live, id(1)).unwrap();
        assert!(!state.tags.contains("new"));
        assert_eq!(state.facets.get("custom.a"), Some(&serde_json::json!(1)));
        assert_eq!(
            state.facets.get("custom.b"),
            Some(&serde_json::json!(false))
        );
        assert!(before.get_node_by_id(id(1)).is_some());
    }

    #[test]
    fn an_undo_reverts_exactly_and_reverting_the_undo_redoes() {
        let traverse = CapturedDelta::ReplayAppendTraversalByIds {
            from_id: id(1).to_string(),
            to_id: id(2).to_string(),
            trigger: NavigationTrigger::LinkClick,
            timestamp_ms: 1_700_000_000_000,
        };
        let (before, made, after) = scene(&[add(1), add(2)], &[relate(1, 2), traverse]);
        let mut live = after.clone();
        let undo = revert_change(&made, &before, &after, &live);
        let undone = recorded(&mut live, |graph| apply_all(graph, &undo.edits));
        assert_eq!(fingerprint(&live), fingerprint(&before), "undo");

        // Redo is the undo reverted, with the undo's own before and after.
        let after_undo = live.clone();
        let redo = revert_change(&undone, &after, &after_undo, &live);
        assert!(redo.kept.is_empty(), "{:?}", redo.kept);
        apply_all(&mut live, &redo.edits);
        assert_eq!(fingerprint(&live), fingerprint(&after), "redo");
    }

    #[test]
    fn the_field_layer_and_import_records_revert() {
        let field_id = FieldId::from_uuid(id(40));
        let coupling_id = CouplingId::from_uuid(id(41));
        let field = Field::new(field_id, FieldDefinition::Scalar(ScalarField::Const(1.0)));
        let coupling = Coupling::new(
            coupling_id,
            field_id,
            NodeSelector::Kind("paper".into()),
            CouplingResponse::DampenInside { factor: 0.3 },
            1.5,
        );
        let records = vec![ImportRecord {
            record_id: "import-record:one".into(),
            source_id: "import:one".into(),
            source_label: "One".into(),
            imported_at_secs: 1_763_500_800,
            memberships: vec![ImportRecordMembership {
                node_id: id(1).to_string(),
                suppressed: false,
            }],
        }];
        let mut graph = Graph::new();
        apply_all(&mut graph, &[add(1)]);
        let before = graph.clone();
        let made = recorded(&mut graph, |graph| {
            let _ = apply_graph_delta(graph, GraphDelta::AddField { field });
            let _ = apply_graph_delta(graph, GraphDelta::AddCoupling { coupling });
            let _ = apply_graph_delta(
                graph,
                GraphDelta::SetImportRecords {
                    import_records: records,
                },
            );
        });
        let after = graph.clone();
        let revert = revert_change(&made, &before, &after, &graph);
        assert!(revert.kept.is_empty(), "{:?}", revert.kept);
        apply_all(&mut graph, &revert.edits);
        assert!(
            graph.field(field_id).is_none(),
            "the added field is gone, not retired"
        );
        assert_eq!(graph.couplings().count(), 0);
        assert!(graph.import_records().is_empty());
    }

    #[test]
    fn touched_parts_reach_the_edits_that_name_them() {
        let touched = Touched::of(&[title(1, "A"), relate(2, 3)]);
        assert!(touched.reaches(&Part::Title(id(1))));
        assert!(touched.reaches(&Part::Edges(id(2), id(3))));
        assert!(
            touched.reaches(&Part::Node(id(2))),
            "a relation touches its ends"
        );
        assert!(!touched.reaches(&Part::Title(id(4))));
        assert!(!touched.reaches(&Part::ImportRecords));
    }
}

#[cfg(test)]
mod resource_revert_tests {
    use super::*;
    use crate::graph::SemanticStatement;

    fn record(iri: &str, facets: &[(&str, Value)]) -> CapturedDelta {
        CapturedDelta::ReplaySetResourceRecordById {
            resource_id: chartulary::resource_id(iri).to_string(),
            record: Some(PersistedResourceRecord {
                canonical_iri: chartulary::canonical_url(iri),
                facets: facets
                    .iter()
                    .map(|(facet, value)| PersistedResourceFacet {
                        facet: (*facet).into(),
                        value_json: value.to_string(),
                    })
                    .collect(),
            }),
        }
    }

    fn shown(surface: Uuid, resource: Uuid) -> CapturedDelta {
        CapturedDelta::ReplaySetShownResourceById {
            surface_id: surface.to_string(),
            resource_id: Some(resource.to_string()),
        }
    }

    fn pair(from: Uuid, to: Uuid, handle: &str) -> CapturedDelta {
        let mut source = Graph::new();
        let a = source.add_node_with_id(from, "https://a.test".into(), Default::default());
        let b = source.add_node_with_id(to, "https://b.test".into(), Default::default());
        source.assert_surface_persisted_semantic_statement(
            a,
            b,
            SemanticStatement {
                statement_id: handle.into(),
                predicate: "https://vocab.test/relation".into(),
                recognized_sub_kind: None,
                label: None,
                graph_scope: crate::types::GraphScope::User,
                provenance_iri: Some("https://people.test/source".into()),
                asserted_at_ms: Some(100),
            },
        );
        CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: source.persisted_edges_between(a, b),
        }
    }

    fn apply(graph: &mut Graph, edits: &[CapturedDelta]) {
        super::tests::apply_all(graph, edits);
    }

    #[test]
    fn resource_undo_keeps_later_facet_keys_and_reports_same_key_conflicts() {
        let iri = "https://resource.test/a";
        let resource = chartulary::resource_id(iri);
        let mut before = Graph::new();
        apply(
            &mut before,
            &[record(iri, &[("a", Value::from(1)), ("b", Value::from(1))])],
        );
        let change = [record(iri, &[("a", Value::from(2)), ("b", Value::from(1))])];
        let mut after = before.clone();
        apply(&mut after, &change);
        let mut live = after.clone();
        apply(
            &mut live,
            &[record(
                iri,
                &[
                    ("a", Value::from(2)),
                    ("b", Value::from(3)),
                    ("foreign", serde_json::json!({"kept":true})),
                ],
            )],
        );
        let undo = revert_change(&change, &before, &after, &live);
        assert!(undo.kept.is_empty());
        apply(&mut live, &undo.edits);
        let facets = resource_facets(&live.resource_record(resource).unwrap());
        assert_eq!(facets["a"], Value::from(1));
        assert_eq!(facets["b"], Value::from(3));
        assert_eq!(facets["foreign"], serde_json::json!({"kept":true}));

        let mut conflict = after.clone();
        apply(
            &mut conflict,
            &[record(iri, &[("a", Value::from(4)), ("b", Value::from(3))])],
        );
        let undo = revert_change(&change, &before, &after, &conflict);
        assert_eq!(undo.kept, [Part::ResourceFacet(resource, "a".into())]);
        assert!(undo.edits.is_empty());
        assert_eq!(
            resource_facets(&conflict.resource_record(resource).unwrap())["a"],
            Value::from(4)
        );
    }

    fn creation_fixture() -> (Graph, Graph, Vec<CapturedDelta>, Uuid, Uuid, Uuid, Uuid) {
        let surface_a = Uuid::from_u128(1);
        let surface_b = Uuid::from_u128(2);
        let resource_a = chartulary::resource_id("https://resource.test/a");
        let resource_b = chartulary::resource_id("https://resource.test/b");
        let mut before = Graph::new();
        before.add_node_with_id(
            surface_a,
            "https://surface.test/a".into(),
            Default::default(),
        );
        before.add_node_with_id(
            surface_b,
            "https://surface.test/b".into(),
            Default::default(),
        );
        apply(
            &mut before,
            &[
                record("https://resource.test/b", &[]),
                shown(surface_b, resource_b),
            ],
        );
        let change = vec![
            record("https://resource.test/a", &[]),
            pair(resource_a, resource_b, "owned-handle"),
            shown(surface_a, resource_a),
        ];
        let mut after = before.clone();
        apply(&mut after, &change);
        (
            before, after, change, surface_a, surface_b, resource_a, resource_b,
        )
    }

    #[test]
    fn resource_undo_creation_clears_pairs_and_bindings_before_record() {
        let (before, after, change, surface_a, surface_b, resource_a, resource_b) =
            creation_fixture();
        let undo = revert_change(&change, &before, &after, &after);
        assert!(undo.kept.is_empty());
        let removal = undo
            .edits
            .iter()
            .position(|edit| {
                matches!(
                    edit,
                    CapturedDelta::ReplaySetResourceRecordById { record: None, .. }
                )
            })
            .unwrap();
        let pair = undo.edits.iter().position(|edit| matches!(edit, CapturedDelta::ReplaySetResourceEdgesByIds { edges, .. } if edges.is_empty())).unwrap();
        let binding = undo
            .edits
            .iter()
            .position(|edit| {
                matches!(
                    edit,
                    CapturedDelta::ReplaySetShownResourceById {
                        resource_id: None,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(pair < removal && binding < removal);
        let mut restored = after.clone();
        apply(&mut restored, &undo.edits);
        assert!(restored.resource_record(resource_a).is_none());
        assert!(
            restored
                .persisted_resource_edges_between(resource_a, resource_b)
                .is_empty()
        );
        assert_eq!(shown_resource(&restored, surface_a), None);
        assert_eq!(shown_resource(&restored, surface_b), Some(resource_b));
        assert_eq!(
            restored.resource_record(resource_b),
            before.resource_record(resource_b)
        );
        assert!(restored.get_node_by_id(surface_a).is_some());
    }

    #[test]
    fn resource_undo_creation_keeps_later_surface_and_pair_references() {
        let (before, after, change, surface_a, surface_b, resource_a, resource_b) =
            creation_fixture();
        for later_surface in [true, false] {
            let mut live = after.clone();
            let resource_c = chartulary::resource_id("https://resource.test/c");
            if later_surface {
                apply(&mut live, &[shown(surface_b, resource_a)]);
            } else {
                apply(
                    &mut live,
                    &[
                        record("https://resource.test/c", &[]),
                        pair(resource_a, resource_c, "later-handle"),
                    ],
                );
            }
            let undo = revert_change(&change, &before, &after, &live);
            assert!(undo.kept.contains(&Part::Resource(resource_a)));
            apply(&mut live, &undo.edits);
            assert!(live.resource_record(resource_a).is_some());
            assert!(
                live.persisted_resource_edges_between(resource_a, resource_b)
                    .is_empty(),
                "owned pair still reverts"
            );
            assert_eq!(
                shown_resource(&live, surface_a),
                None,
                "owned binding still reverts"
            );
            assert!(live.resource_record(resource_b).is_some());
            if later_surface {
                assert_eq!(shown_resource(&live, surface_b), Some(resource_a));
            } else {
                let surviving = live.persisted_resource_edges_between(resource_a, resource_c);
                assert_eq!(
                    surviving[0].semantic.as_ref().unwrap().statements[0].statement_id,
                    "later-handle"
                );
                assert_eq!(shown_resource(&live, surface_b), Some(resource_b));
            }
        }
    }

    #[test]
    fn surface_creation_undo_preserves_later_shown_binding() {
        let resource_a = chartulary::resource_id("https://resource.test/a");
        let resource_b = chartulary::resource_id("https://resource.test/b");
        let surface = Uuid::from_u128(99);
        let mut before = Graph::new();
        apply(
            &mut before,
            &[
                record("https://resource.test/a", &[]),
                record("https://resource.test/b", &[]),
            ],
        );
        for initial_binding in [None, Some(resource_a)] {
            let mut change = vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: surface.to_string(),
                url: "https://surface.test".into(),
                position: [0.0, 0.0],
            }];
            if let Some(resource) = initial_binding {
                change.push(shown(surface, resource));
            }
            let mut after = before.clone();
            apply(&mut after, &change);
            let undo = revert_change(&change, &before, &after, &after);
            assert!(undo.kept.is_empty());
            let mut original = after.clone();
            apply(&mut original, &undo.edits);
            assert!(original.get_node_by_id(surface).is_none());
            assert!(original.surface_ids_showing_resource(resource_a).is_empty());
            assert!(original.resource_record(resource_a).is_some());
            assert!(original.resource_record(resource_b).is_some());

            let mut later = after.clone();
            apply(&mut later, &[shown(surface, resource_b)]);
            let undo = revert_change(&change, &before, &after, &later);
            assert!(undo.kept.contains(&Part::Node(surface)));
            if initial_binding.is_some() {
                assert!(undo.kept.contains(&Part::ShownResource(surface)));
            }
            apply(&mut later, &undo.edits);
            assert!(later.get_node_by_id(surface).is_some());
            assert_eq!(shown_resource(&later, surface), Some(resource_b));
            assert_eq!(later.surface_ids_showing_resource(resource_b), [surface]);
            assert!(later.resource_record(resource_a).is_some());
        }
    }

    #[test]
    fn surface_removal_undo_restores_shown_binding_and_keeps_later_recreation() {
        let (_, before, _, surface_a, surface_b, resource_a, resource_b) = creation_fixture();
        let change = [CapturedDelta::ReplayRemoveNodeById {
            node_id: surface_a.to_string(),
        }];
        let mut after = before.clone();
        apply(&mut after, &change);
        assert!(after.get_node_by_id(surface_a).is_none());
        assert!(after.surface_ids_showing_resource(resource_a).is_empty());
        let undo = revert_change(&change, &before, &after, &after);
        assert!(undo.kept.is_empty());
        let mut restored = after.clone();
        apply(&mut restored, &undo.edits);
        assert_eq!(shown_resource(&restored, surface_a), Some(resource_a));
        assert_eq!(shown_resource(&restored, surface_b), Some(resource_b));
        assert_eq!(
            restored.resource_record(resource_a),
            before.resource_record(resource_a)
        );

        let mut later = after.clone();
        later.add_node_with_id(surface_a, "https://later.test".into(), Default::default());
        apply(&mut later, &[shown(surface_a, resource_b)]);
        let undo = revert_change(&change, &before, &after, &later);
        assert!(undo.kept.contains(&Part::Node(surface_a)));
        assert!(undo.kept.contains(&Part::ShownResource(surface_a)));
        apply(&mut later, &undo.edits);
        assert_eq!(shown_resource(&later, surface_a), Some(resource_b));
        assert_eq!(
            later.get_node_by_id(surface_a).unwrap().1.url(),
            "https://later.test"
        );
        assert_eq!(shown_resource(&later, surface_b), Some(resource_b));
    }

    #[test]
    fn resource_touched_parts_keep_equal_surface_ids_separate() {
        let from = Uuid::from_u128(1);
        let to = Uuid::from_u128(2);
        let touched = Touched::of(&[CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: Vec::new(),
        }]);
        assert!(touched.reaches(&Part::ResourceEdges(from, to)));
        assert!(touched.reaches(&Part::Resource(from)));
        assert!(!touched.reaches(&Part::Edges(from, to)));
        assert!(!touched.reaches(&Part::Node(from)));
        let touched = Touched::of(&[shown(from, to)]);
        assert!(touched.reaches(&Part::ShownResource(from)));
        assert!(touched.reaches(&Part::Resource(to)));
        assert!(!touched.reaches(&Part::Resource(from)));
    }
}
