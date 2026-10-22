// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shared, read-only relationship navigation over the kernel's canonical projection.
//! Hosts retain navigation admission, actions, graph ownership and persistence.
//! Direction and kind are retained even when several relations share a member.
use kernel::graph::{Graph, RelationKind, SemanticSubKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RelationshipTarget<G = uuid::Uuid> {
    pub graph: G,
    pub source: uuid::Uuid,
    pub member: uuid::Uuid,
    pub kind: RelationKind,
    pub incoming: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipRow<G = uuid::Uuid> {
    pub target: RelationshipTarget<G>,
    pub title: String,
    pub relationship: String,
    pub address: String,
}

pub fn relationships<G: Copy + Eq + std::hash::Hash>(
    graph_id: G,
    graph: &Graph,
    source: uuid::Uuid,
) -> Vec<RelationshipRow<G>> {
    let Some((key, _)) = graph.get_node_by_id(source) else {
        return Vec::new();
    };
    let mut seen = std::collections::HashSet::new();
    let mut rows = Vec::new();
    for (_, relation) in graph.projected_relations() {
        let incoming = relation.to == key;
        let other = if relation.from == key {
            relation.to
        } else if incoming {
            relation.from
        } else {
            continue;
        };
        let Some(node) = graph.get_node(other) else {
            continue;
        };
        if node.id == source {
            continue;
        }
        let target = RelationshipTarget {
            graph: graph_id,
            source,
            member: node.id,
            kind: relation.kind,
            incoming: relation.from != key,
        };
        if seen.insert(target) {
            rows.push(RelationshipRow {
                target,
                title: document_label(graph, other),
                address: node.url().to_string(),
                relationship: relationship_label(relation.kind, target.incoming),
            });
        }
    }
    rows.sort_by(|a, b| {
        a.relationship
            .cmp(&b.relationship)
            .then(a.title.cmp(&b.title))
            .then(a.target.member.cmp(&b.target.member))
    });
    rows
}

/// Keep an unloaded or host-only title useful when several members share a
/// domain. Loaded document titles retain the kernel's ordinary presentation.
pub fn document_label(graph: &Graph, key: kernel::graph::NodeKey) -> String {
    let Some(node) = graph.get_node(key) else {
        return "Document unavailable".into();
    };
    let title = node.title.trim();
    let host_only = kernel::address::cached_host_from_url(node.url())
        .map(|host| host == title)
        .unwrap_or(false);
    if title.is_empty() || title == node.url() || host_only {
        node.url().into()
    } else {
        graph.node_display_label(key)
    }
}

fn relationship_label(kind: RelationKind, incoming: bool) -> String {
    use SemanticSubKind::*;
    let (out, into) = match kind {
        RelationKind::Traversal => ("Went to", "Reached from"),
        RelationKind::Semantic(Hyperlink) => ("Links to", "Linked from"),
        RelationKind::Semantic(Cites) => ("Cites", "Cited by"),
        RelationKind::Semantic(Quotes) => ("Quotes", "Quoted by"),
        RelationKind::Semantic(Summarizes) => ("Summarizes", "Summarized by"),
        RelationKind::Semantic(Elaborates) => ("Elaborates on", "Elaborated by"),
        RelationKind::Semantic(ExampleOf) => ("Example of", "Has example"),
        RelationKind::Semantic(Supports) => ("Supports", "Supported by"),
        RelationKind::Semantic(Contradicts) => ("Contradicts", "Contradicted by"),
        RelationKind::Semantic(Questions) => ("Questions", "Questioned by"),
        RelationKind::Semantic(SameEntityAs) => ("Same entity as", "Same entity as"),
        RelationKind::Semantic(DuplicateOf) => ("Duplicate of", "Has duplicate"),
        RelationKind::Semantic(CanonicalMirrorOf) => {
            ("Canonical mirror of", "Has canonical mirror")
        },
        RelationKind::Semantic(DependsOn) => ("Depends on", "Required by"),
        RelationKind::Semantic(Blocks) => ("Blocks", "Blocked by"),
        RelationKind::Semantic(NextStep) => ("Next step", "Previous step"),
        RelationKind::Semantic(UserGrouped) => ("Grouped with", "Grouped with"),
        RelationKind::Semantic(AgentDerived) => ("Derived relation to", "Derived relation from"),
        RelationKind::Containment(_) => ("Contains", "Contained by"),
        RelationKind::Arrangement(_) => ("Arranged with", "Arranged with"),
        RelationKind::Imported(_) => ("Import relation to", "Import relation from"),
        RelationKind::Provenance(_) => ("Derived from", "Source for"),
        RelationKind::OpenPredicate => ("Relation to", "Relation from"),
    };
    let label = if incoming { into } else { out };
    match kind {
        RelationKind::Containment(sub) => format!("{label} · {}", words(&format!("{sub:?}"))),
        RelationKind::Arrangement(sub) => format!("{label} · {}", words(&format!("{sub:?}"))),
        RelationKind::Imported(sub) => format!("{label} · {}", words(&format!("{sub:?}"))),
        RelationKind::Provenance(sub) => format!("{label} · {}", words(&format!("{sub:?}"))),
        _ => label.into(),
    }
}

fn words(value: &str) -> String {
    let mut result = String::new();
    for (i, c) in value.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push(' ');
        }
        result.extend(c.to_lowercase());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::{
        EdgeAssertion, SemanticSubKind,
        apply::{add_node, assert_relation},
    };
    #[test]
    fn shared_rows_keep_predicate_direction_graph_and_access_identity() {
        let mut graph = Graph::new();
        let a = add_node(
            &mut graph,
            Some(uuid::Uuid::from_u128(1)),
            "https://same.test/a".into(),
            Default::default(),
        );
        let b = add_node(
            &mut graph,
            Some(uuid::Uuid::from_u128(2)),
            "https://same.test/b".into(),
            Default::default(),
        );
        for (from, to, kind) in [
            (a, b, SemanticSubKind::Hyperlink),
            (a, b, SemanticSubKind::Cites),
            (b, a, SemanticSubKind::Quotes),
        ] {
            assert_relation(
                &mut graph,
                from,
                to,
                EdgeAssertion::Semantic {
                    sub_kind: kind,
                    label: None,
                },
            )
            .unwrap();
        }
        let before = graph.projected_relations().count();
        let rows = relationships(42u64, &graph, uuid::Uuid::from_u128(1));
        assert_eq!(rows.len(), 3);
        assert!(
            rows.iter()
                .all(|r| r.target.graph == 42 && r.target.member == uuid::Uuid::from_u128(2))
        );
        for label in ["Links to", "Cites", "Quoted by"] {
            assert!(rows.iter().any(|r| r.relationship == label));
        }
        let rows = relationships(42u64, &graph, uuid::Uuid::from_u128(2));
        for label in ["Linked from", "Cited by", "Quotes"] {
            assert!(rows.iter().any(|r| r.relationship == label));
        }
        assert!(relationships(42u64, &graph, uuid::Uuid::nil()).is_empty());
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.projected_relations().count(), before);
    }
}
