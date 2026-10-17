//! Diagnostic assertions of interrupted migration behavior.
use muniment::{Backend, MemoryBackend, StoreError, WriteOp};

#[test]
#[cfg(windows)]
fn a_partial_identity_adoption_permanently_skips_personas_on_retry() {
    use std::os::windows::fs::OpenOptionsExt;
    let legacy = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir(legacy.path().join("identity")).unwrap();
    std::fs::write(legacy.path().join("identity/device.json"), b"dummy device marker").unwrap();
    std::fs::create_dir_all(legacy.path().join("personas/example")).unwrap();
    std::fs::write(legacy.path().join("personas/example/wallet.json"), b"dummy wallet marker").unwrap();
    // A replaceable file blocks only the second directory operation. It is
    // not an identity directory, so the migration's initial guard permits it.
    std::fs::write(shared.path().join("personas"), b"fixture obstacle").unwrap();
    let obstacle = std::fs::OpenOptions::new().read(true).share_mode(0).open(shared.path().join("personas")).unwrap();
    assert!(pandect::shared_root::adopt_legacy_identity(shared.path(), legacy.path()).is_err());
    assert!(shared.path().join("identity/device.json").is_file());
    assert!(!legacy.path().join("identity").exists());
    assert!(legacy.path().join("personas/example/wallet.json").is_file());
    drop(obstacle);
    std::fs::remove_file(shared.path().join("personas")).unwrap();
    assert!(!pandect::shared_root::adopt_legacy_identity(shared.path(), legacy.path()).unwrap(), "retry reports ordinary no-op instead of completing adoption");
    assert!(!shared.path().join("personas").exists());
    assert!(legacy.path().join("personas/example/wallet.json").is_file(), "control: original wallet bytes remain recoverable");
}

#[derive(Clone, Default)]
struct RefuseWrites(MemoryBackend);
#[async_trait::async_trait]
impl Backend for RefuseWrites {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> { self.0.get(key).await }
    async fn put(&self, _: &str, _: &[u8]) -> Result<(), StoreError> { Err(StoreError::Backend("diagnostic write refusal".into())) }
    async fn delete(&self, key: &str) -> Result<(), StoreError> { self.0.delete(key).await }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> { self.0.list(prefix).await }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> { self.0.scan(start, end).await }
    async fn apply(&self, _: &[WriteOp]) -> Result<(), StoreError> { Err(StoreError::Backend("diagnostic write refusal".into())) }
}

#[test]
fn image_storage_failure_removes_the_unstored_pixels_from_the_supplied_snapshot() {
    pollster::block_on(async {
        let mut graph = kernel::graph::Graph::new();
        kernel::graph::apply::add_node(&mut graph, Some(uuid::Uuid::from_u128(401)), "https://audit.test/image".into(), Default::default());
        let mut snapshot = graph.to_snapshot();
        snapshot.nodes[0].legacy_thumbnail_png = Some(vec![137, 80, 78, 71]);
        snapshot.nodes[0].legacy_thumbnail_width = 1;
        snapshot.nodes[0].legacy_thumbnail_height = 1;
        assert_eq!(snapshot.legacy_image_count(), 1);
        let mut failing = RefuseWrites::default();
        assert!(pandect::image_store::migrate_legacy_images(&mut snapshot, &mut failing, |_, _, _| None).await.is_err());
        assert_eq!(snapshot.legacy_image_count(), 0, "unstored pixels were taken before the fallible write");
        assert!(!snapshot.nodes[0].images.contains_key(&kernel::types::ImageRole::Preview));
        let mut recovered = MemoryBackend::new();
        let written = pandect::image_store::migrate_legacy_images(&mut snapshot, &mut recovered, |_, _, _| None).await.unwrap();
        assert_eq!(written, 0, "retry now succeeds but has no pixels left to store");
        assert!(recovered.list("").await.unwrap().is_empty());
    });
}
