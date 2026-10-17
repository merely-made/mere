//! Audit-only current-library observations; synthetic in-memory values only.
use state_witness::{Entry, Witness};
use proofs::{BlobRef, Commitment, CommitmentDomain, Digest, DigestAlg};

#[test]
fn unique_out_of_order_entries_are_reported_as_duplicate_labels() {
    let entries = vec![Entry { label: "site:10".into(), digest: 1 }, Entry { label: "site:9".into(), digest: 2 }];
    assert_ne!(entries[0].label, entries[1].label);
    assert_eq!(Witness::try_from(entries), Err(state_witness::Error::DuplicateLabel("site:9".into())));
    let mut sorted = Witness::new(); sorted.insert_bytes("site:10", b"one").unwrap(); sorted.insert_bytes("site:9", b"two").unwrap();
    assert_eq!(sorted.entries().iter().map(|e| e.label.as_str()).collect::<Vec<_>>(), ["site:9", "site:10"]);
    assert_eq!(Witness::from_framed(&sorted.to_framed().unwrap()).unwrap(), sorted);
    println!("Witness refusal: unique out-of-order entries return DuplicateLabel for a label that occurs once; normal insertion sorts and framed reload succeeds");
}

#[test]
fn label_helper_is_a_plain_formatter_not_an_injective_structured_identity() {
    let flat = state_witness::label("kind", ["a/b"]);
    let nested = state_witness::label("kind", ["a", "b"]);
    assert_eq!(flat, nested);
    let mut a = Witness::new(); a.insert(flat, &7u64).unwrap();
    let mut b = Witness::new(); b.insert(nested, &7u64).unwrap();
    assert_eq!(state_witness::first_divergence(&a, &b), None);
    assert_eq!(a.digest(), b.digest());
    println!("Label boundary: Display parts containing '/' collide with path separators; plain String/helper was explicitly ruled, so producers own grammar/escaping. No current colliding consumer or new identity-bypass finding");
}

#[test]
fn typed_proof_checks_preserve_tags_but_as_32_checks_only_width() {
    let body = b"synthetic proof bytes";
    let reference = BlobRef::blake3(body); assert!(reference.verifies(body));
    for alg in [DigestAlg::Sha256, DigestAlg::P2pandaOperation, DigestAlg::William3] {
        let mut other = reference.clone(); other.digest.alg = alg;
        assert_eq!(other.digest.as_32().unwrap(), reference.digest.as_32().unwrap());
        assert!(!other.verifies(body));
    }
    assert!(Digest { alg: DigestAlg::Blake3, bytes: vec![0; 31] }.as_32().is_err());
    let commitment = Commitment::direct(CommitmentDomain::StorageCheckpoints, body);
    assert!(commitment.verifies_direct(CommitmentDomain::StorageCheckpoints, body));
    assert!(!commitment.verifies_direct(CommitmentDomain::StorageChunks, body));
    assert!(!commitment.verifies_direct(CommitmentDomain::StorageCheckpoints, b"other"));
    let mut newer = commitment.clone(); newer.version += 1;
    assert!(!newer.verifies_direct(CommitmentDomain::StorageCheckpoints, body));
    println!("Proof boundary: complete BlobRef/Commitment verification checks the tags; as_32 returns the same raw array for differently tagged32-byte digests and is not an algorithm guard. No signature, inclusion-tree or canonicalization proof is implied");
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}
fn spec(reference: &BlobRef) -> mesh::spec::JobSpec {
    mesh::spec::JobSpec::simple(mesh::ResourceId::parse("mesh.echo/v1").unwrap(), "payload", reference.clone(), "result", 16, mesh::spec::DeterminismClass::Exact)
}
#[test]
fn ambient_mesh_lookup_accepts_invalid_reference_but_resource_view_refuses_it() {
    use mesh::namespace::{MemoryBlobSpace, JobNamespaceView, NamespaceError};
    runtime().block_on(async {
        let space = MemoryBlobSpace::in_memory(); let body = b"synthetic";
        let valid = space.put(body).await.unwrap();
        assert!(space.has(&valid).await.unwrap()); assert_eq!(space.get(&valid).await.unwrap().unwrap(), body);
        let good = spec(&valid); good.validate().unwrap();
        assert_eq!(JobNamespaceView::grant(&good, &space, &space).read("payload").await.unwrap(), body);
        let mut wrong_alg = valid.clone(); wrong_alg.digest.alg = DigestAlg::Sha256;
        let mut wrong_length = valid.clone(); wrong_length.byte_len += 1;
        for bad in [wrong_alg, wrong_length] {
            assert!(!bad.verifies(body));
            assert!(space.has(&bad).await.unwrap());
            assert_eq!(space.get(&bad).await.unwrap().unwrap(), body);
            let job = spec(&bad); job.validate().unwrap();
            assert_eq!(JobNamespaceView::grant(&job, &space, &space).read("payload").await, Err(NamespaceError::DigestMismatch("payload".into())));
        }
    });
    println!("Mesh reference boundary: ambient has/get report present/return bytes for wrong-algorithm or wrong-length references, and spec validation admits them. JobNamespaceView correctly refuses both before adapter read; no permission bypass or durable files");
}
