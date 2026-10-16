// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Observed live edits against a complete session writer. No resident keys or
//! captured replay records cross this boundary.
use crate::{
    Applied, SessionError,
    addressable_graph::{GraphAddress, PreparedGraphPublication},
};
use kernel::graph::{
    Author, EdgeAssertion, Graph, GraphStratum, RelationKind,
    apply::{GraphDelta, apply_graph_delta},
    built_in_relation_stratum,
};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LiveCommand {
    CreateAddress {
        id: Uuid,
        address: String,
        title: String,
    },
    EditNode {
        surface: Uuid,
        title: String,
        tags: BTreeSet<String>,
    },
    SetFacet {
        surface: Uuid,
        facet: String,
        value: serde_json::Value,
    },
    SetPinned {
        surface: Uuid,
        pinned: bool,
    },
    SetMimeHint {
        surface: Uuid,
        mime_hint: Option<String>,
    },
    AssertRelation {
        from: Uuid,
        to: Uuid,
        assertion: EdgeAssertion,
    },
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LiveCheck {
    Exists(GraphAddress),
    Absent(GraphAddress),
    RecordUnchanged {
        address: GraphAddress,
        digest: String,
    },
    ShownUnchanged {
        surface: Uuid,
        resource: Uuid,
    },
    ContentTagsUnchanged {
        surface: Uuid,
        tags: BTreeSet<String>,
    },
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedCommand {
    pub command: LiveCommand,
    pub checks: Vec<LiveCheck>,
}
#[derive(Debug)]
pub enum LiveEditError {
    Session(SessionError),
    Missing(GraphAddress),
    Stale(GraphAddress),
    MissingObservation(GraphAddress),
    InvalidCommand(String),
    NeedsCompleteOperation,
}
impl From<SessionError> for LiveEditError {
    fn from(error: SessionError) -> Self {
        Self::Session(error)
    }
}
impl std::fmt::Display for LiveEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Session(e) => write!(f, "{e}"),
            Self::Missing(a) => write!(f, "missing graph record: {a:?}"),
            Self::Stale(a) => write!(f, "changed graph observation: {a:?}"),
            Self::MissingObservation(a) => write!(f, "missing graph observation: {a:?}"),
            Self::InvalidCommand(reason) => write!(f, "invalid live command: {reason}"),
            Self::NeedsCompleteOperation => {
                write!(f, "operation requires complete graph authority")
            },
        }
    }
}
impl std::error::Error for LiveEditError {}
#[derive(Debug)]
pub struct LiveApplied {
    pub applied: Applied,
    pub created: Vec<Uuid>,
}

// Private stable-ID program. It is constructed only after all input checks;
// execution is infallible and resolves keys inside each complete candidate.
pub(crate) struct Program {
    operations: Vec<Operation>,
    pub(crate) created: Vec<Uuid>,
}
#[derive(Clone)]
enum Operation {
    Create(Uuid, String),
    Title(Uuid, String),
    AddTag(Uuid, String),
    RemoveTag(Uuid, String),
    Facet(Uuid, String, serde_json::Value),
    Pinned(Uuid, bool),
    Mime(Uuid, Option<String>),
    Relation(Uuid, Uuid, EdgeAssertion),
}
impl Operation {
    fn apply(&self, graph: &mut Graph, author: &Author) {
        let key = |id| {
            graph
                .get_node_key_by_id(id)
                .expect("preflight resolved live target")
        };
        let delta = match self {
            Self::Create(id, url) => GraphDelta::AddNode {
                id: Some(*id),
                url: url.clone(),
                position: Default::default(),
            },
            Self::Title(id, title) => GraphDelta::SetNodeTitle {
                key: key(*id),
                title: title.clone(),
            },
            Self::AddTag(id, tag) => GraphDelta::InsertNodeTag {
                key: key(*id),
                tag: tag.clone(),
            },
            Self::RemoveTag(id, tag) => GraphDelta::RemoveNodeTag {
                key: key(*id),
                tag: tag.clone(),
            },
            Self::Facet(id, facet, value) => GraphDelta::SetNodeFacet {
                key: key(*id),
                facet: facet.clone(),
                value: value.clone(),
            },
            Self::Pinned(id, is_pinned) => GraphDelta::SetNodePinned {
                key: key(*id),
                is_pinned: *is_pinned,
            },
            Self::Mime(id, mime_hint) => GraphDelta::SetNodeMimeHint {
                key: key(*id),
                mime_hint: mime_hint.clone(),
            },
            Self::Relation(from, to, assertion) => GraphDelta::AssertRelation {
                from: key(*from),
                to: key(*to),
                assertion: assertion.clone(),
                asserter_iri: author.asserter_iri(),
            },
        };
        let _ = apply_graph_delta(graph, delta);
    }
}
impl Program {
    pub(crate) fn apply(&self, graph: &mut Graph, author: &Author) {
        for operation in &self.operations {
            operation.apply(graph, author);
        }
    }
}
fn exists(graph: &Graph, address: GraphAddress) -> bool {
    match address {
        GraphAddress::Surface(id) => graph.get_node_key_by_id(id).is_some(),
        GraphAddress::Resource(id) => graph.resource_record(id).is_some(),
    }
}
fn tags(graph: &Graph, surface: Uuid) -> Option<BTreeSet<String>> {
    graph
        .get_node_key_by_id(surface)
        .and_then(|key| graph.node_content_tags(key))
        .map(|tags| tags.into_iter().collect())
}
fn require_record(
    graph: &Graph,
    source: &PreparedGraphPublication,
    checks: &[LiveCheck],
    address: GraphAddress,
) -> Result<(), LiveEditError> {
    if !exists(graph, address) {
        return Err(LiveEditError::Missing(address));
    }
    let observed = if source.record_digest(address).is_some() {
        checks.iter().any(
            |check| matches!(check, LiveCheck::RecordUnchanged { address: a, .. } if *a == address),
        )
    } else {
        checks
            .iter()
            .any(|check| matches!(check, LiveCheck::Exists(a) if *a == address))
    };
    if !observed {
        return Err(LiveEditError::MissingObservation(address));
    }
    Ok(())
}
fn require_content(
    graph: &Graph,
    source: &PreparedGraphPublication,
    checks: &[LiveCheck],
    surface: Uuid,
) -> Result<Uuid, LiveEditError> {
    let address = GraphAddress::Surface(surface);
    require_record(graph, source, checks, address)?;
    let key = graph.get_node_key_by_id(surface).expect("checked surface");
    let resource = graph
        .shown_resource_id(key)
        .ok_or(LiveEditError::NeedsCompleteOperation)?;
    if source.record_digest(address).is_some() && !checks.iter().any(|check| matches!(check, LiveCheck::ShownUnchanged { surface: s, resource: r } if *s == surface && *r == resource)) {
        return Err(LiveEditError::MissingObservation(address));
    }
    // A newly created Surface can display a Resource that already existed.
    // That shared record still needs the original complete observation.
    if source
        .record_digest(GraphAddress::Resource(resource))
        .is_some()
    {
        require_record(graph, source, checks, GraphAddress::Resource(resource))?;
    }
    Ok(resource)
}
fn relation_stratum(assertion: &EdgeAssertion) -> GraphStratum {
    let kind = match assertion {
        EdgeAssertion::Semantic { sub_kind, .. } => RelationKind::Semantic(*sub_kind),
        EdgeAssertion::Containment { sub_kind } => RelationKind::Containment(*sub_kind),
        EdgeAssertion::Arrangement { sub_kind } => RelationKind::Arrangement(*sub_kind),
        EdgeAssertion::Imported { sub_kind } => RelationKind::Imported(*sub_kind),
        EdgeAssertion::Provenance { sub_kind } => RelationKind::Provenance(*sub_kind),
    };
    built_in_relation_stratum(kind)
}

pub(crate) fn preflight(
    source_graph: &Graph,
    source: &PreparedGraphPublication,
    author: &Author,
    commands: Vec<ObservedCommand>,
) -> Result<Program, LiveEditError> {
    // Validate external observations against one immutable original, before any
    // sequential scratch writes. Unrelated session revisions do not matter.
    for observed in &commands {
        for check in &observed.checks {
            match check {
                LiveCheck::RecordUnchanged { address, digest } => {
                    match source.record_digest(*address) {
                        None => return Err(LiveEditError::Missing(*address)),
                        Some(current) if current != digest => {
                            return Err(LiveEditError::Stale(*address));
                        },
                        _ => {},
                    }
                },
                LiveCheck::ShownUnchanged { surface, resource } => {
                    if source
                        .record_digest(GraphAddress::Surface(*surface))
                        .is_none()
                    {
                        return Err(LiveEditError::Missing(GraphAddress::Surface(*surface)));
                    }
                    if source.shown_resource(*surface) != Some(*resource) {
                        return Err(LiveEditError::Stale(GraphAddress::Surface(*surface)));
                    }
                },
                LiveCheck::ContentTagsUnchanged {
                    surface,
                    tags: expected,
                } => match tags(source_graph, *surface) {
                    None => return Err(LiveEditError::Missing(GraphAddress::Surface(*surface))),
                    Some(current) if current != *expected => {
                        return Err(LiveEditError::Stale(GraphAddress::Surface(*surface)));
                    },
                    _ => {},
                },
                LiveCheck::Exists(_) | LiveCheck::Absent(_) => {},
            }
        }
    }
    let mut scratch = source_graph.clone();
    scratch.set_recorder(None);
    kernel::graph::with_isolated_capture(|| {
        scratch.write_as(author.clone(), |scratch| {
        let mut program = Program { operations: vec![], created: vec![] };
        for ObservedCommand { command, checks } in commands {
            // URL identity takes precedence over the requested new UUID and its
            // absence observation. A hit supplies no implicit alias.
            let hit = match &command {
                LiveCommand::CreateAddress { address, .. } => scratch.get_node_by_url(address).map(|(_, node)| node.id),
                _ => None,
            };
            for check in &checks {
                match check {
                    LiveCheck::Exists(address) if !exists(scratch, *address) => return Err(LiveEditError::Missing(*address)),
                    LiveCheck::Absent(address) => {
                        let ignored = hit.is_some() && matches!(&command, LiveCommand::CreateAddress { id, .. } if *address == GraphAddress::Surface(*id));
                        if !ignored && exists(scratch, *address) { return Err(LiveEditError::Stale(*address)); }
                    },
                    _ => {},
                }
            }
            let mut next = vec![];
            match command {
                LiveCommand::CreateAddress { id, address, title } => {
                    if let Some(actual) = hit { program.created.push(actual); continue; }
                    if exists(scratch, GraphAddress::Surface(id)) { return Err(LiveEditError::Stale(GraphAddress::Surface(id))); }
                    if !checks.iter().any(|check| matches!(check, LiveCheck::Absent(a) if *a == GraphAddress::Surface(id))) { return Err(LiveEditError::MissingObservation(GraphAddress::Surface(id))); }
                    next.push(Operation::Create(id, address));
                    let title = title.trim();
                    if !title.is_empty() { next.push(Operation::Title(id, title.into())); }
                    program.created.push(id);
                },
                LiveCommand::EditNode { surface, title, tags: desired } => {
                    require_content(scratch, source, &checks, surface)?;
                    if source.record_digest(GraphAddress::Surface(surface)).is_some() && !checks.iter().any(|check| matches!(check, LiveCheck::ContentTagsUnchanged { surface: s, .. } if *s == surface)) { return Err(LiveEditError::MissingObservation(GraphAddress::Surface(surface))); }
                    let desired: BTreeSet<_> = desired.into_iter().map(|tag| tag.trim().to_owned()).filter(|tag| !tag.is_empty()).collect();
                    let current = tags(scratch, surface).expect("checked content Surface");
                    next.push(Operation::Title(surface, title.trim().into()));
                    for tag in current.difference(&desired) { next.push(Operation::RemoveTag(surface, tag.clone())); }
                    for tag in desired.difference(&current) { next.push(Operation::AddTag(surface, tag.clone())); }
                },
                LiveCommand::SetFacet { surface, facet, value } => {
                    require_record(scratch, source, &checks, GraphAddress::Surface(surface))?;
                    next.push(Operation::Facet(surface, facet, value));
                },
                LiveCommand::SetPinned { surface, pinned } => {
                    require_record(scratch, source, &checks, GraphAddress::Surface(surface))?;
                    next.push(Operation::Pinned(surface, pinned));
                },
                LiveCommand::SetMimeHint { surface, mime_hint } => {
                    require_record(scratch, source, &checks, GraphAddress::Surface(surface))?;
                    next.push(Operation::Mime(surface, mime_hint));
                },
                LiveCommand::AssertRelation { from, to, assertion } => {
                    if let EdgeAssertion::Semantic { decay_progress: Some(value), .. } = &assertion {
                        if !value.is_finite() { return Err(LiveEditError::InvalidCommand("non-finite relation decay".into())); }
                    }
                    for surface in [from, to] {
                        if relation_stratum(&assertion) == GraphStratum::Resource {
                            require_content(scratch, source, &checks, surface)?;
                        } else { require_record(scratch, source, &checks, GraphAddress::Surface(surface))?; }
                    }
                    next.push(Operation::Relation(from, to, assertion));
                },
            }
            for operation in &next { operation.apply(scratch, author); }
            program.operations.extend(next);
        }
        Ok(program)
    })
    })
}
