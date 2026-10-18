// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use muniment::{Backend, MemoryBackend, StoreError, WriteOp};
#[derive(Clone, Default)]
struct RefuseWrites(MemoryBackend);
#[async_trait::async_trait]
impl Backend for RefuseWrites {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.0.get(key).await
    }
    async fn put(&self, _: &str, _: &[u8]) -> Result<(), StoreError> {
        Err(StoreError::Backend("diagnostic write refusal".into()))
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.0.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.0.list(prefix).await
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.0.scan(start, end).await
    }
    async fn apply(&self, _: &[WriteOp]) -> Result<(), StoreError> {
        Err(StoreError::Backend("diagnostic write refusal".into()))
    }
}

#[test]
fn image_storage_failure_preserves_pixels_for_a_healthy_retry() {
    for role in [
        kernel::types::ImageRole::Preview,
        kernel::types::ImageRole::Favicon,
    ] {
        refused_write_and_retry(role);
    }
}

fn refused_write_and_retry(role: kernel::types::ImageRole) {
    pollster::block_on(async {
        let mut graph = kernel::graph::Graph::new();
        kernel::graph::apply::add_node(
            &mut graph,
            Some(uuid::Uuid::from_u128(401)),
            "https://audit.test/image".into(),
            Default::default(),
        );
        let mut snapshot = graph.to_snapshot();
        let node = &mut snapshot.nodes[0];
        if role == kernel::types::ImageRole::Preview {
            node.legacy_thumbnail_png = Some(vec![137, 80, 78, 71]);
            node.legacy_thumbnail_width = 1;
            node.legacy_thumbnail_height = 1;
        } else {
            node.legacy_favicon_rgba = Some(vec![255, 0, 0, 255]);
            node.legacy_favicon_width = 1;
            node.legacy_favicon_height = 1;
        }
        assert_eq!(snapshot.legacy_image_count(), 1);
        let mut failing = RefuseWrites::default();
        assert!(
            pandect::image_store::migrate_legacy_images(&mut snapshot, &mut failing, |_, _, _| {
                Some(vec![137, 80, 78, 71])
            })
            .await
            .is_err()
        );
        assert_eq!(
            snapshot.legacy_image_count(),
            1,
            "unstored pixels survive the fallible write"
        );
        assert!(!snapshot.nodes[0].images.contains_key(&role));
        let mut recovered = MemoryBackend::new();
        let written = pandect::image_store::migrate_legacy_images(
            &mut snapshot,
            &mut recovered,
            |_, _, _| Some(vec![137, 80, 78, 71]),
        )
        .await
        .unwrap();
        assert_eq!(written, 1);
        assert_eq!(snapshot.legacy_image_count(), 0);
        assert!(!recovered.list("").await.unwrap().is_empty());
    });
}
