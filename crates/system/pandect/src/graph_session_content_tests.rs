// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::legacy_resource_migration::LegacyContentContext;
use kernel::graph::resource_tags::tag_concept_iri;

#[test]
fn unknown_tags_require_original_mere_and_reopen_preserves_that_namespace() {
    pollster::block_on(async {
        let backend = muniment::MemoryBackend::new();
        let mut baseline = Graph::new();
        let key = kernel::graph::apply::add_node(
            &mut baseline,
            None,
            "https://example.test/page".into(),
            Default::default(),
        );
        let mut snapshot = baseline.to_snapshot();
        snapshot.nodes[0].tags = vec!["saved".into()];
        baseline = Graph::try_from_snapshot(&snapshot).unwrap();
        let surface_id = baseline.get_node(key).unwrap().id;
        let mut session =
            MereSessions::new(backend.clone()).begin(Author::person("current"), Some(baseline));
        session.flush(SystemTime::now()).await.unwrap();
        let id = session.id();
        let keys = Keys::new(id);
        let original_baseline = backend.get(&keys.at(BASELINE)).await.unwrap();
        assert!(
            GraphSession::open_legacy(backend.clone(), id, Seq(0), vec![])
                .await
                .is_err()
        );
        assert!(backend.get(&keys.at(TRANSLATION)).await.unwrap().is_none());
        assert!(backend.get(&keys.at(PLACEMENT)).await.unwrap().is_none());
        assert_eq!(
            backend.get(&keys.at(BASELINE)).await.unwrap(),
            original_baseline
        );
        let context = LegacyContentContext {
            original_mere_iri: "urn:mere:original:test".into(),
        };
        let mut qualified = GraphSession::open_legacy_with_content_context(
            backend.clone(),
            id,
            Seq(0),
            vec![],
            &context,
        )
        .await
        .unwrap();
        let key = qualified.graph().get_node_key_by_id(surface_id).unwrap();
        let resource = qualified.graph().shown_resource_id(key).unwrap();
        assert!(
            qualified
                .graph()
                .resource_tag_labels(resource)
                .contains("saved")
        );
        let concept = chartulary::resource_id_from_canonical_iri(&tag_concept_iri(
            &context.original_mere_iri,
            "saved",
        ));
        assert_eq!(
            qualified
                .graph()
                .resource_tag_concept(concept)
                .unwrap()
                .owner_iri,
            context.original_mere_iri
        );
        let frozen = FrozenGraph::of(qualified.graph());
        qualified
            .qualify_legacy_with_content_context(Seq(0), vec![], &context)
            .await
            .unwrap();
        let wrong = LegacyContentContext {
            original_mere_iri: "urn:mere:destination:test".into(),
        };
        assert!(
            qualified
                .qualify_legacy_with_content_context(Seq(0), vec![], &wrong)
                .await
                .is_err()
        );
        assert_eq!(FrozenGraph::of(qualified.graph()), frozen);
        qualified.checkpoint().await.unwrap();
        let reopened = GraphSession::open(backend.clone(), id).await.unwrap();
        assert_eq!(FrozenGraph::of(reopened.graph()), frozen);
        assert_eq!(FrozenGraph::of(&reopened.graph_at(Seq(0)).unwrap()), frozen);
        let bytes = backend.get(&keys.at(TRANSLATION)).await.unwrap().unwrap();
        let mut receipt: LegacyTranslationReceipt = serde_json::from_slice(&bytes).unwrap();
        receipt.content_context = Some(wrong);
        backend
            .put(
                &keys.at(TRANSLATION),
                &serde_json::to_vec(&receipt).unwrap(),
            )
            .await
            .unwrap();
        assert!(GraphSession::open(backend.clone(), id).await.is_err());
        backend.put(&keys.at(TRANSLATION), &bytes).await.unwrap();
        assert_eq!(
            FrozenGraph::of(GraphSession::open(backend, id).await.unwrap().graph()),
            frozen
        );
    });
}
