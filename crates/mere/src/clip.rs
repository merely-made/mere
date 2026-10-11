// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Graph insertion for clips gathered by Fleece.
//!
//! The caller chooses the document format and persists visual bytes before
//! supplying their image reference. The adapter records the graph relation;
//! destination-session write authority remains with the caller.
//! `ClippedFrom` belongs to the clip and source resources and is projected onto
//! their browsing surfaces by the graph's public relation read APIs.

use fleece::clip::{ClipFragment, clip_title};
use forme::GraphMemberId;
use kernel::graph::{EdgeAssertion, Graph, NodeKey, ProvenanceSubKind};
use kernel::types::{ImageRef, ImageRole};

/// Insert an already formatted clip and relate it to its source node.
///
/// `body` and `media_type` belong to the receiving document adapter. Fleece's
/// semantic clip can become plain text, Djot, or another representation without
/// assigning that format authority to the gathering capability.
pub fn write_clip_node(
    graph: &mut Graph,
    source_key: NodeKey,
    clip_url: &str,
    fragment: &ClipFragment,
    body: &str,
    media_type: &str,
    visual: Option<ImageRef>,
) -> Option<GraphMemberId> {
    if graph.get_node(source_key).is_none() {
        return None;
    }
    use kernel::graph::apply::{self as graph_apply, GraphDelta, apply_graph_delta};

    let clip_key = graph_apply::add_node(graph, None, clip_url.to_string(), Default::default());
    let _ = apply_graph_delta(
        graph,
        GraphDelta::SetNodeTitle {
            key: clip_key,
            title: clip_title(fragment),
        },
    );
    let _ = apply_graph_delta(
        graph,
        GraphDelta::SetNodeMimeHint {
            key: clip_key,
            mime_hint: Some(media_type.to_string()),
        },
    );
    let _ = apply_graph_delta(
        graph,
        GraphDelta::SetNodeBody {
            key: clip_key,
            body: Some(body.to_string()),
        },
    );
    let member = graph.get_node(clip_key).map(|n| n.id);
    if let Some(visual) = visual {
        let _ = apply_graph_delta(
            graph,
            GraphDelta::SetNodeImage {
                key: clip_key,
                role: ImageRole::Preview,
                image: visual,
            },
        );
    }
    let _ = graph_apply::assert_relation(
        graph,
        clip_key,
        source_key,
        EdgeAssertion::Provenance {
            sub_kind: ProvenanceSubKind::ClippedFrom,
        },
    );
    member
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{RelationKey, RelationSelector};

    #[test]
    fn clip_insertion_keeps_receiving_format_relation_and_thumbnail() {
        let mut graph = Graph::new();
        let source = graph.add_node("gemini://example.test/post".to_string(), Default::default());
        let clip = fleece::clip::fragment_from_text(
            "gemini://example.test/post",
            Some("Interesting bit".into()),
            "A useful paragraph.",
            None,
            Vec::new(),
        );
        let stored = ImageRef::new([21u8; 32], 2, 2);
        let body = "# Interesting bit\n\nA useful paragraph.\n";
        let member = write_clip_node(
            &mut graph,
            source,
            "file:///clips/test.djot",
            &clip,
            body,
            "text/djot",
            Some(stored),
        )
        .expect("clip member");
        let (clip_key, clip_node) = graph.get_node_by_id(member).expect("clip node");
        assert_eq!(clip_node.preview(), Some(&stored));
        assert_eq!(clip_node.body.as_deref(), Some(body));
        assert_eq!(clip_node.media_type.as_deref(), Some("text/djot"));
        assert!(
            graph
                .projected_relations_between(clip_key, source)
                .any(|(key, payload)| {
                    matches!(key, RelationKey::Resource(_))
                        && payload.has_relation(RelationSelector::Provenance(
                            ProvenanceSubKind::ClippedFrom,
                        ))
                })
        );
    }
}
