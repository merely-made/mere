//! Public Eidetic payload identity observations; synthetic memory only.
use eidetic::{ManifestId, MemoryBackend, NoFetcher, PrivacyClass, ProvenanceOrigin,
    ProvenanceRecord, SchemaRef, Timestamp, TrustEnvelope, TypedPayload,
    load_typed, save_typed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct A { ticks: u32 }
#[derive(Debug, Serialize, Deserialize)]
struct B { ticks: u32 }
impl TypedPayload for A {
    fn schema_ref() -> SchemaRef { SchemaRef::from_id(ManifestId::of_blob(b"audit:schema-a")) }
}
impl TypedPayload for B {
    fn schema_ref() -> SchemaRef { SchemaRef::from_id(ManifestId::of_blob(b"audit:schema-b")) }
}
fn provenance(at: u64) -> ProvenanceRecord {
    ProvenanceRecord { origin: ProvenanceOrigin::Generated, upstream: vec![],
        tooling: Some("synthetic-audit".into()), generated_at: Timestamp(at) }
}

#[test]
fn identical_bytes_under_different_schemas_replace_the_first_schema() {
    pollster::block_on(async {
        let mut store = MemoryBackend::default();
        let a = save_typed(&mut store, &A { ticks: 42 }, vec![], PrivacyClass::LocalOnly,
            provenance(1), TrustEnvelope::self_asserted(), Timestamp(1)).await.unwrap();
        assert_eq!(load_typed::<A>(&mut store, &mut NoFetcher, a).await.unwrap().unwrap().ticks, 42);
        let b = save_typed(&mut store, &B { ticks: 42 }, vec![], PrivacyClass::LocalOnly,
            provenance(2), TrustEnvelope::self_asserted(), Timestamp(2)).await.unwrap();
        assert_eq!(a, b);
        let error = load_typed::<A>(&mut store, &mut NoFetcher, a).await.unwrap_err();
        assert!(error.to_string().contains("schema mismatch"));
        assert_eq!(load_typed::<B>(&mut store, &mut NoFetcher, b).await.unwrap().unwrap().ticks, 42);
        println!("OBSERVATION: a successful second typed save aliases the first id and invalidates the first schema's read");
    });
}

#[test]
fn same_payload_can_replace_privacy_and_provenance_in_place() {
    pollster::block_on(async {
        let mut store = MemoryBackend::default();
        let id = save_typed(&mut store, &A { ticks: 42 }, vec![], PrivacyClass::LocalOnly,
            provenance(1), TrustEnvelope::self_asserted(), Timestamp(1)).await.unwrap();
        let before = eidetic::manifest::load_manifest(&mut store, id).await.unwrap().unwrap();
        let again = save_typed(&mut store, &A { ticks: 42 }, vec![], PrivacyClass::PublicPortable,
            provenance(2), TrustEnvelope::self_asserted(), Timestamp(2)).await.unwrap();
        assert_eq!(id, again);
        let after = eidetic::manifest::load_manifest(&mut store, id).await.unwrap().unwrap();
        assert_eq!(before.privacy, PrivacyClass::LocalOnly);
        assert_eq!(after.privacy, PrivacyClass::PublicPortable);
        assert_eq!(after.created_at, Timestamp(2));
        assert_eq!(after.provenance.generated_at, Timestamp(2));
        println!("OBSERVATION: the same payload id retains replacement privacy and provenance metadata, not an immutable manifest");
    });
}

#[test]
fn different_payload_bytes_keep_both_schema_reads_working() {
    pollster::block_on(async {
        let mut store = MemoryBackend::default();
        let a = save_typed(&mut store, &A { ticks: 42 }, vec![], PrivacyClass::LocalOnly,
            provenance(1), TrustEnvelope::self_asserted(), Timestamp(1)).await.unwrap();
        let b = save_typed(&mut store, &B { ticks: 43 }, vec![], PrivacyClass::LocalOnly,
            provenance(2), TrustEnvelope::self_asserted(), Timestamp(2)).await.unwrap();
        assert_ne!(a, b);
        assert_eq!(load_typed::<A>(&mut store, &mut NoFetcher, a).await.unwrap().unwrap().ticks, 42);
        assert_eq!(load_typed::<B>(&mut store, &mut NoFetcher, b).await.unwrap().unwrap().ticks, 43);
        println!("CONTROL: different serialized payloads retain independent readable typed records");
    });
}
