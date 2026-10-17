use graphshell::{
    access::AccessContext,
    mere_host::{MereHost, SelectedPersonaRef, fixture_handlers},
    product::{ExportRequest, LocalFileMetadata, TransferScope},
    transfer::{self, AccessTransferPolicy, ApplyTransferContext, TransferAuthorization,
        TransferBlobInput, TransferEndpointV1, TransferOperation, TransferRequest, TransferRouteV1},
};
use muniment::{BlobStore, MemoryBackend};
use uuid::Uuid;

fn host(store: MemoryBackend) -> MereHost<MemoryBackend> {
    MereHost::empty(store, SelectedPersonaRef { persona: "personae://audit/person".into(), profile: "audit".into() },
        fixture_handlers(), AccessContext { persona: "personae://audit/person".into(), device: "audit/device".into(), at_ms: 1 })
}
fn export(focused: Uuid) -> ExportRequest {
    ExportRequest { focused, selected: vec![focused], scope: TransferScope::ObjectOnly,
        exported_at_ms: 1000, include_local_file_locations: false, scene: None }
}
fn endpoint(graph: &str) -> TransferEndpointV1 {
    TransferEndpointV1 { graph: graph.into(), persona: "personae://audit/person".into(), device: graph.into() }
}

#[test]
fn excluding_local_file_locations_still_exports_an_explicit_file_address() {
    let mut source = host(MemoryBackend::new());
    let path = "file:///C:/synthetic-audit/private-folder/example.txt";
    let id = source.create_address(path, "Synthetic address only").unwrap();
    let bytes = source.export_product_codicil(export(id)).unwrap();
    assert!(String::from_utf8(bytes).unwrap().contains(path));
}

async fn apply_file(altered: bool) {
    let source_store = MemoryBackend::new();
    let source_blobs = BlobStore::new(source_store.clone());
    let mut source = host(source_store.clone());
    // Known SHA-256 of b"abc". No physical file or user content is involved.
    let id = source.create_file_metadata(LocalFileMetadata {
        content_hash: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
        name: "synthetic.txt".into(), media_type: "text/plain".into(), byte_len: 3, last_modified_ms: 0,
    }).unwrap();
    let original_address = source.graph().get_node_by_id(id).unwrap().1.url().to_owned();
    let mut manifest = transfer::prepare_transfer(&source, &mut source_store.clone(), &source_blobs,
        TransferRequest { transfer_id: Uuid::new_v4(), operation: TransferOperation::Replicate,
            source: endpoint("source"), destination: endpoint("destination"), route: TransferRouteV1 { carrier: "audit".into(), peer: "audit".into() },
            selection: export(id), access_policy: AccessTransferPolicy::ExcludeSourceHistory, privacy: eidetic::PrivacyClass::TrustedPeersOnly },
        vec![TransferBlobInput { node_id: id, role: "primary".into(), media_type: "text/plain".into(), bytes: b"abc".to_vec() }]).await.unwrap();
    let bytes: &[u8] = if altered { b"xyz" } else { b"abc" };
    if altered {
        // Keep the valid immutable selection and its SHA-256 content identity.
        // Substitute a separately valid BLAKE3-addressed blob descriptor.
        source_blobs.put(bytes).await.unwrap();
        manifest.blobs[0].content_hash = eidetic::Hash::of(bytes);
    }
    manifest.selection.verify_integrity().unwrap();
    let destination_store = MemoryBackend::new();
    let destination_blobs = BlobStore::new(destination_store.clone());
    let mut destination = host(destination_store.clone());
    let receipt = transfer::apply_transfer(&mut destination, &source_blobs, &destination_blobs,
        &mut destination_store.clone(), &manifest, &ApplyTransferContext {
            authorization: TransferAuthorization { grant_id: "audit-trusted-caller".into(), revoked: false },
            application: "audit".into(), handler: "audit".into(), completed_at_ms: 2000, access_privacy: eidetic::PrivacyClass::LocalOnly,
        }).await.unwrap();
    assert_eq!(receipt.result, transfer::TransferResult::Completed);
    let descriptor = &manifest.blobs[0];
    let hash = muniment::Hash::from_hex(&descriptor.content_hash.to_hex()).unwrap();
    assert_eq!(destination_blobs.get(&hash).await.unwrap().unwrap(), bytes);
    assert_eq!(destination.graph().get_node_by_id(id).unwrap().1.url(), original_address);
    assert_eq!(destination.facet_value(&original_address, transfer::TRANSFER_CONTENT_FACET).unwrap()[0]["content_hash"], serde_json::to_value(descriptor.content_hash).unwrap());
}

#[test]
fn ordinary_file_transfer_preserves_the_expected_content() { pollster::block_on(apply_file(false)); }

#[test]
fn receiver_completes_with_blob_bytes_that_disagree_with_the_preserved_content_identity() { pollster::block_on(apply_file(true)); }
