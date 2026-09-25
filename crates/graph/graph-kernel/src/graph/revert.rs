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
use crate::persistence::PersistedEdge;

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

impl Revert {
    fn keep(&mut self, part: Part) {
        self.kept.push(part);
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
    fn node_parts(&mut self, id: Uuid, before: &NodeState, after: &NodeState, live: &NodeState) {
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
                was != became,
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
        self.edits
            .push(CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: id.to_string(),
                url: before.url.clone(),
                position: [0.0, 0.0],
            });
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
        self.node_parts(id, before, &born, &born);
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
                if now == became && unlinked_since {
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
            (Some(was), Some(became), Some(now)) => out.node_parts(*id, &was, &became, &now),
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
