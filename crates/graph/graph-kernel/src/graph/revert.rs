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
use super::node_facets::ARRANGEMENT_FRAME_LAYOUT;
use super::{CouplingId, FieldId, FrameLayoutHint, Graph, NodeKey};
use crate::persistence::{
    PersistedEdge, PersistedFieldExtent, PersistedResourceFacet, PersistedResourceRecord,
};

/// One part of the graph a change can alter.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
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

/// Rewrites an id in place when the map names it.
type Swap<'a> = &'a dyn Fn(&mut String);

fn remap_ends(edges: &mut [PersistedEdge], swap: Swap) {
    for edge in edges {
        swap(&mut edge.from_node_id);
        swap(&mut edge.to_node_id);
    }
}

fn remap_hint(hint: &mut FrameLayoutHint, swap: Swap) {
    use FrameLayoutHint as H;
    match hint {
        H::SplitHalf { first, second, .. } => [first, second].into_iter().for_each(swap),
        H::SplitPamphlet { members, .. } => members.iter_mut().for_each(swap),
        H::SplitTriptych {
            dominant, wings, ..
        } => {
            swap(dominant);
            wings.iter_mut().for_each(swap);
        },
        H::SplitQuartered {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        } => [top_left, top_right, bottom_left, bottom_right]
            .into_iter()
            .for_each(swap),
    }
}

/// The value of node facet `facet` with the ids it declares rewritten, and
/// the text unchanged for every other facet: facets are open JSON, so a facet
/// names ids only by declaring them here. The kernel's own:
///
/// - `arrangement.frame-layout` holds hints whose members are node ids of the
///   graph itself: rewritten.
/// - `provenance.derivations` holds `source_node`, a node of the source
///   graph, which is what a pick merge matches on: kept.
/// - `arrangement.pin` and `arrangement.split-offer-suppressed` are flags,
///   `visit.history` clocks, `presentation.tags` tag names and icons,
///   `provenance.import` an import source's id and label,
///   `semantic.classifications` and `semantic.properties` scheme values and
///   literals: none holds a node id.
fn remap_facet_value(facet: &str, value_json: &str, swap: Swap) -> String {
    match facet {
        ARRANGEMENT_FRAME_LAYOUT => serde_json::from_str::<Vec<FrameLayoutHint>>(value_json)
            .ok()
            .and_then(|mut hints| {
                hints.iter_mut().for_each(|hint| remap_hint(hint, swap));
                serde_json::to_string(&hints).ok()
            })
            .unwrap_or_else(|| value_json.to_string()),
        _ => value_json.to_string(),
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

/// The part one revert or merge edit sets: what a caller picking edits by
/// part (a merge-in applying one side of a conflict) filters on. An edit that
/// recreates or removes a node is [`Part::Node`]; a whole resource record is
/// [`Part::Resource`]. `None` for edits revert never writes (traversals,
/// previews, layout hints and the like).
pub fn part_of(edit: &CapturedDelta) -> Option<Part> {
    use CapturedDelta as D;
    let id = |text: &str| uuid(text);
    Some(match edit {
        D::ReplayAddNodeWithIdIfMissing { id: node, .. }
        | D::ReplayRemoveNodeById { node_id: node } => Part::Node(id(node)?),
        D::ReplaySetNodeTitleById { node_id, .. } => Part::Title(id(node_id)?),
        D::ReplaySetNodeUrlById { node_id, .. } => Part::Url(id(node_id)?),
        D::ReplayInsertNodeTagById { node_id, tag }
        | D::ReplayRemoveNodeTagById { node_id, tag } => Part::Tag(id(node_id)?, tag.clone()),
        D::ReplaySetNodeBodyById { node_id, .. } => Part::Body(id(node_id)?),
        D::ReplaySetNodeContentById { node_id, .. } => Part::Content(id(node_id)?),
        D::ReplaySetNodeMimeHintById { node_id, .. } => Part::MediaType(id(node_id)?),
        D::ReplaySetNodeNestedById { node_id, .. } => Part::Nested(id(node_id)?),
        D::ReplaySetNodeFacetById { node_id, facet, .. }
        | D::ReplayRemoveNodeFacetById { node_id, facet } => {
            Part::Facet(id(node_id)?, facet.clone())
        },
        D::ReplayUpdateNodeHistoryById { node_id, .. } => Part::History(id(node_id)?),
        D::ReplaySetEdgesByIds { from_id, to_id, .. } => Part::Edges(id(from_id)?, id(to_id)?),
        D::ReplaySetImportRecords { .. } => Part::ImportRecords,
        D::ReplayAddField { field } => Part::Field(id(&field.id)?),
        D::ReplayRemoveFieldById { field_id } => Part::Field(id(field_id)?),
        D::ReplayAddCoupling { coupling } => Part::Coupling(id(&coupling.id)?),
        D::ReplayRetractCouplingById { coupling_id } => Part::Coupling(id(coupling_id)?),
        D::ReplaySetResourceRecordById { resource_id, .. } => Part::Resource(id(resource_id)?),
        D::ReplaySetResourceEdgesByIds {
            from_resource_id,
            to_resource_id,
            ..
        } => Part::ResourceEdges(id(from_resource_id)?, id(to_resource_id)?),
        D::ReplaySetShownResourceById { surface_id, .. } => Part::ShownResource(id(surface_id)?),
        _ => return None,
    })
}

/// The node a node-scoped part belongs to, if it is one.
pub fn part_node(part: &Part) -> Option<Uuid> {
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
        | Part::History(id) => Some(*id),
        _ => None,
    }
}

/// What `part` holds in `graph`, as JSON, or `None` when the graph does not
/// hold it (a missing node, an absent tag or facet, no relations between two
/// nodes). Two graphs agree on a part exactly when these are equal: how a
/// merge-in tells a conflict from two sides making the same change.
pub fn part_value(graph: &Graph, part: &Part) -> Option<Value> {
    let node = |id: &Uuid| node_state(graph, *id);
    match part {
        Part::Node(id) => node(id).map(|state| {
            serde_json::json!({
                "title": state.title,
                "url": state.url,
                "tags": state.tags,
                "body": state.body,
                "content": state.content.map(|hash| hash.to_vec()),
                "media_type": state.media_type,
                "nested": state.nested,
                "facets": state.facets,
                "history": state.history,
            })
        }),
        Part::Title(id) => node(id).map(|state| Value::from(state.title)),
        Part::Url(id) => node(id).map(|state| Value::from(state.url)),
        Part::Tag(id, tag) => node(id)
            .filter(|state| state.tags.contains(tag))
            .map(|_| Value::Bool(true)),
        Part::Body(id) => node(id).and_then(|state| state.body.map(Value::from)),
        Part::Content(id) => node(id)
            .and_then(|state| state.content)
            .map(|hash| Value::from(hash.to_vec())),
        Part::MediaType(id) => node(id).and_then(|state| state.media_type.map(Value::from)),
        Part::Nested(id) => node(id).and_then(|state| state.nested.map(Value::from)),
        Part::Facet(id, facet) => node(id).and_then(|mut state| state.facets.remove(facet)),
        Part::History(id) => node(id).map(|state| serde_json::json!(state.history)),
        Part::Edges(from, to) => {
            let edges = edges_between(graph, *from, *to);
            (!edges.is_empty()).then(|| serde_json::to_value(edges).expect("edges encode"))
        },
        Part::Field(id) => graph.field(FieldId::from_uuid(*id)).map(|field| {
            serde_json::to_value(persisted_field_from_field(field)).expect("field encodes")
        }),
        Part::Coupling(id) => graph
            .couplings()
            .find(|coupling| coupling.id == CouplingId::from_uuid(*id))
            .map(|coupling| {
                serde_json::to_value(persisted_coupling_from_coupling(coupling))
                    .expect("coupling encodes")
            }),
        Part::ImportRecords => {
            let records = graph.import_records();
            (!records.is_empty()).then(|| serde_json::to_value(records).expect("records encode"))
        },
        Part::Resource(id) => graph
            .resource_record(*id)
            .map(|record| serde_json::to_value(record).expect("record encodes")),
        Part::ResourceFacet(id, facet) => graph
            .resource_record(*id)
            .and_then(|record| resource_facets(&record).remove(facet)),
        Part::ResourceEdges(from, to) => {
            let edges = graph.persisted_resource_edges_between(*from, *to);
            (!edges.is_empty()).then(|| serde_json::to_value(edges).expect("edges encode"))
        },
        Part::ShownResource(id) => {
            shown_resource(graph, *id).map(|resource| Value::from(resource.to_string()))
        },
    }
}

/// The edits that make node `id` again, whole, as it stands in `graph`: what
/// a merge-in uses to bring a node across that its target never held. `None`
/// when `graph` has no such node. Relations are not included.
pub fn recreate_node_edits(graph: &Graph, id: Uuid) -> Option<Vec<CapturedDelta>> {
    let state = node_state(graph, id)?;
    let mut made = Revert::default();
    made.recreate(id, &state);
    Some(made.edits)
}

/// A node state read back from [`part_value`]'s `Part::Node` form.
fn node_state_from(value: &Value) -> Option<NodeState> {
    let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_string);
    let content = match value.get("content") {
        None | Some(Value::Null) => None,
        Some(bytes) => Some(
            serde_json::from_value::<Vec<u8>>(bytes.clone())
                .ok()?
                .try_into()
                .ok()?,
        ),
    };
    Some(NodeState {
        title: text("title")?,
        url: text("url")?,
        tags: serde_json::from_value(value.get("tags")?.clone()).ok()?,
        body: text("body"),
        content,
        media_type: text("media_type"),
        nested: text("nested"),
        facets: serde_json::from_value(value.get("facets")?.clone()).ok()?,
        history: serde_json::from_value(value.get("history")?.clone()).ok()?,
    })
}

/// The edits that make `part` in `graph` read as `value`, given in
/// [`part_value`]'s form, `None` for absent: how a merge-in applies a value
/// someone supplied by hand. An error names a value that does not fit the
/// part.
pub fn part_edits(
    graph: &Graph,
    part: &Part,
    value: Option<&Value>,
) -> Result<Vec<CapturedDelta>, String> {
    use CapturedDelta as D;
    let unfit = || format!("{value:?} does not fit {part:?}");
    fn decode<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, String> {
        serde_json::from_value(value.clone()).map_err(|error| error.to_string())
    }
    let text = || value.and_then(Value::as_str).map(str::to_string);
    Ok(match part {
        Part::Node(id) => match value {
            None => graph
                .get_node_by_id(*id)
                .map(|_| D::ReplayRemoveNodeById {
                    node_id: id.to_string(),
                })
                .into_iter()
                .collect(),
            Some(value) => {
                let wanted = node_state_from(value).ok_or_else(unfit)?;
                let mut out = Revert::default();
                match node_state(graph, *id) {
                    Some(now) => out.node_parts(*id, &wanted, &now, &now, false),
                    None => out.recreate(*id, &wanted),
                }
                out.edits
            },
        },
        Part::Title(id) => vec![D::ReplaySetNodeTitleById {
            node_id: id.to_string(),
            title: text().ok_or_else(unfit)?,
        }],
        Part::Url(id) => vec![D::ReplaySetNodeUrlById {
            node_id: id.to_string(),
            new_url: text().ok_or_else(unfit)?,
        }],
        Part::Tag(id, tag) => {
            let node_id = id.to_string();
            let tag = tag.clone();
            if value.is_some_and(|value| value != &Value::Bool(false)) {
                vec![D::ReplayInsertNodeTagById { node_id, tag }]
            } else {
                vec![D::ReplayRemoveNodeTagById { node_id, tag }]
            }
        },
        Part::Body(id) => vec![D::ReplaySetNodeBodyById {
            node_id: id.to_string(),
            body: text(),
        }],
        Part::Content(id) => vec![D::ReplaySetNodeContentById {
            node_id: id.to_string(),
            content: match value {
                None => None,
                Some(bytes) => Some(decode::<Vec<u8>>(bytes)?.try_into().map_err(|_| unfit())?),
            },
        }],
        Part::MediaType(id) => vec![D::ReplaySetNodeMimeHintById {
            node_id: id.to_string(),
            mime_hint: text(),
        }],
        Part::Nested(id) => vec![D::ReplaySetNodeNestedById {
            node_id: id.to_string(),
            nested: text(),
        }],
        Part::Facet(id, facet) => vec![match value {
            Some(value) => D::ReplaySetNodeFacetById {
                node_id: id.to_string(),
                facet: facet.clone(),
                value_json: value.to_string(),
            },
            None => D::ReplayRemoveNodeFacetById {
                node_id: id.to_string(),
                facet: facet.clone(),
            },
        }],
        Part::History(id) => {
            let (entries, current_index): (Vec<String>, usize) = decode(value.ok_or_else(unfit)?)?;
            vec![D::ReplayUpdateNodeHistoryById {
                node_id: id.to_string(),
                entries,
                current_index,
            }]
        },
        Part::Edges(from, to) => vec![D::ReplaySetEdgesByIds {
            from_id: from.to_string(),
            to_id: to.to_string(),
            edges: value.map(decode).transpose()?.unwrap_or_default(),
        }],
        Part::Field(id) => vec![match value {
            Some(value) => D::ReplayAddField {
                field: decode(value)?,
            },
            None => D::ReplayRemoveFieldById {
                field_id: id.to_string(),
            },
        }],
        Part::Coupling(id) => vec![match value {
            Some(value) => D::ReplayAddCoupling {
                coupling: decode(value)?,
            },
            None => D::ReplayRetractCouplingById {
                coupling_id: id.to_string(),
            },
        }],
        Part::ImportRecords => vec![D::ReplaySetImportRecords {
            import_records: value.map(decode).transpose()?.unwrap_or_default(),
        }],
        Part::Resource(id) => vec![D::ReplaySetResourceRecordById {
            resource_id: id.to_string(),
            record: value.map(decode).transpose()?,
        }],
        Part::ResourceFacet(id, facet) => {
            let mut record = graph.resource_record(*id).ok_or_else(unfit)?;
            record.facets.retain(|held| &held.facet != facet);
            if let Some(value) = value {
                record.facets.push(PersistedResourceFacet {
                    facet: facet.clone(),
                    value_json: value.to_string(),
                });
            }
            vec![D::ReplaySetResourceRecordById {
                resource_id: id.to_string(),
                record: Some(record),
            }]
        },
        Part::ResourceEdges(from, to) => vec![D::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: value.map(decode).transpose()?.unwrap_or_default(),
        }],
        Part::ShownResource(id) => vec![D::ReplaySetShownResourceById {
            surface_id: id.to_string(),
            resource_id: text(),
        }],
    })
}

#[cfg(test)]
#[path = "revert/tests.rs"]
pub(super) mod tests;

#[cfg(test)]
#[path = "revert/resource_revert_tests.rs"]
mod resource_revert_tests;

#[cfg(test)]
#[path = "revert/remap_tests.rs"]
mod remap_tests;

#[path = "revert/captured.rs"]
mod captured;

#[path = "revert/operations.rs"]
mod operations;

#[path = "revert/touched.rs"]
mod touched;

#[path = "revert/replay.rs"]
mod replay;

pub use replay::revert_change;
