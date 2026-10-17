//! Public legacy/current corpus loading using synthetic memory-backed records.
use eidetic::{ManifestId, MemoryBackend, NoFetcher, PrivacyClass, ProvenanceRecord,
    SchemaRef, Timestamp, TrustEnvelope, TypedPayload, save_typed};
use eidetic::models::{TrainingCorpus, LEGACY_TRAINING_CORPUS_SCHEMA_REF, load_training_corpus};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct V1 {
    training_source_engrams: Vec<ManifestId>,
    evaluation_source_engrams: Vec<ManifestId>,
}
impl TypedPayload for V1 {
    fn schema_ref() -> SchemaRef { *LEGACY_TRAINING_CORPUS_SCHEMA_REF }
}
fn id(text: &[u8]) -> ManifestId { ManifestId::of_blob(text) }
fn malformed() -> Vec<TrainingCorpus> {
    let mut ordered=vec![id(b"a"),id(b"b")];ordered.sort_by_key(ToString::to_string);
    vec![
        TrainingCorpus {training_source_codicils:vec![],evaluation_source_codicils:vec![id(b"eval")]},
        TrainingCorpus {training_source_codicils:vec![id(b"shared")],evaluation_source_codicils:vec![id(b"shared")]},
        TrainingCorpus {training_source_codicils:vec![id(b"dup"),id(b"dup")],evaluation_source_codicils:vec![id(b"eval")]},
        TrainingCorpus {training_source_codicils:ordered.into_iter().rev().collect(),evaluation_source_codicils:vec![id(b"eval")]},
    ]
}
async fn save_v1(store:&mut MemoryBackend, corpus:&TrainingCorpus)->ManifestId {
    save_typed(store,&V1 {training_source_engrams:corpus.training_source_codicils.clone(),
        evaluation_source_engrams:corpus.evaluation_source_codicils.clone()},vec![],PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("synthetic legacy fixture"),TrustEnvelope::self_asserted(),Timestamp(1)).await.unwrap()
}

#[test]
fn legacy_reader_returns_corpora_that_violate_current_domain_invariants() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();
        for input in malformed() {
            assert!(input.validate().is_err());
            let reference=save_v1(&mut store,&input).await;
            let loaded=load_training_corpus(&mut store,&mut NoFetcher,reference).await.unwrap().unwrap();
            assert_eq!(loaded,input);
            assert!(loaded.validate().is_err());
        }
        println!("OBSERVATION: legacy corpus loader returns empty, overlapping, duplicated and reverse-ordered source partitions without validating them");
    });
}

#[test]
fn the_current_schema_refuses_the_same_malformed_partitions() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();
        for input in malformed() {
            assert!(save_typed(&mut store,&input,vec![],PrivacyClass::LocalOnly,
                ProvenanceRecord::self_imported("synthetic current fixture"),TrustEnvelope::self_asserted(),Timestamp(1)).await.is_err());
        }
        println!("CONTROL: current corpus serializer rejects all four equivalent malformed partitions");
    });
}

#[test]
fn valid_legacy_and_current_corpora_load_to_the_same_valid_value() {
    pollster::block_on(async {
        let input=TrainingCorpus {training_source_codicils:vec![id(b"train")],evaluation_source_codicils:vec![id(b"eval")]};
        let mut store=MemoryBackend::default();
        let old=save_v1(&mut store,&input).await;
        let new=save_typed(&mut store,&input,vec![],PrivacyClass::LocalOnly,
            ProvenanceRecord::self_imported("synthetic current fixture"),TrustEnvelope::self_asserted(),Timestamp(1)).await.unwrap();
        assert_ne!(old,new);
        for reference in [old,new] {
            let loaded=load_training_corpus(&mut store,&mut NoFetcher,reference).await.unwrap().unwrap();
            assert_eq!(loaded,input);loaded.validate().unwrap();
        }
        println!("CONTROL: valid v1 and v2 corpus loads agree and satisfy the shared domain validator");
    });
}
