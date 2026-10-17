// Audit-only integration observations over a freshly compiled search library.
use eidetic_search::{Bm25Config, Bm25Index, IndexConfig, Tokenizer, TrailIndex};
use eidetic::browsing::{BrowsingTrace, PageRef, TraceEvent, TraceTransition};
#[path = "../../../../crates/intel/eidetic-search/src/bm25.rs"]
mod original_bm25;
#[path = "../../../../crates/intel/eidetic-search/src/fusion.rs"]
mod original_fusion;

fn tokens(s: &str) -> Vec<String> { vec![s.into()] }
fn trace() -> BrowsingTrace {
    BrowsingTrace::from_events("audit", vec![TraceEvent {
        from: None, to: PageRef { url: "https://audit.invalid/note".into(), title: Some("running".into()) },
        transition: TraceTransition::Imported, at_ms: 1, dwell_ms: None, candidates: vec![],
    }])
}
fn fixture(name: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(std::env::var("MERE_AUDIT_SEARCH_FIXTURE").unwrap());
    assert_eq!(std::fs::read_to_string(root.join(".audit-generated")).unwrap(), "mere authority audit disposable search fixtures\n");
    assert!(["stem", "replacement", "transient"].contains(&name));
    root.join(name)
}
#[test]
fn zero_limit_panics_only_for_a_matching_public_bm25_query() {
    let mut index = Bm25Index::new(Bm25Config::default(), vec![1.0]);
    index.add_document(&[tokens("alpha")]);
    assert!(index.search(&tokens("missing"), 0).is_empty());
    assert_eq!(index.search(&tokens("alpha"), 1).len(), 1);
    assert!(std::panic::catch_unwind(|| index.search(&tokens("alpha"), 0)).is_err());
}
#[test]
fn field_256_silently_uses_field_zero_and_loses_its_weight() {
    let mut weights = vec![0.0; 257]; weights[256] = 1.0;
    let mut fields = vec![Vec::new(); 257]; fields[256] = tokens("alpha");
    let mut wide = Bm25Index::new(Bm25Config::default(), weights);
    wide.add_document(&fields);
    assert!(wide.search(&tokens("alpha"), 1).is_empty());
    let mut narrow = Bm25Index::new(Bm25Config::default(), vec![1.0]);
    narrow.add_document(&[tokens("alpha")]);
    assert_eq!(narrow.search(&tokens("alpha"), 1).len(), 1);
}
#[test]
fn a_stemmed_persistent_index_reopens_as_current_with_different_answers() {
    fn stem(s: &str) -> String { s.trim_end_matches("ing").into() }
    let path = fixture("stem");
    let config = IndexConfig { tokenizer: Tokenizer::with_stemmer(stem), persist: true, ..Default::default() };
    let index = TrailIndex::rebuild_with_config(&path, [&trace()], |_| None, config).unwrap();
    assert_eq!(index.search("runn", 5).unwrap().len(), 1);
    let reopened = TrailIndex::open(&path).expect("sidecar accepts a stemmed projection as current");
    assert!(reopened.search("runn", 5).unwrap().is_empty());
    assert_eq!(reopened.search("running", 5).unwrap().len(), 1);
}
#[test]
fn persistent_rebuild_deletes_unrelated_marker_in_the_supplied_directory() {
    let path = fixture("replacement"); std::fs::create_dir(&path).unwrap();
    let marker = path.join("unrelated-marker.txt");
    std::fs::write(&marker, "disposable audit marker, not user data\n").unwrap();
    let index = TrailIndex::rebuild(&path, [&trace()]).unwrap();
    assert_eq!(index.doc_count().unwrap(), 1);
    assert!(!marker.exists());
}
#[test]
fn transient_rebuild_preserves_the_same_kind_of_directory_marker() {
    let path = fixture("transient"); std::fs::create_dir(&path).unwrap();
    let marker = path.join("unrelated-marker.txt");
    std::fs::write(&marker, "disposable audit marker, not user data\n").unwrap();
    let index = TrailIndex::rebuild_with_config(&path, [&trace()], |_| None, IndexConfig::default()).unwrap();
    assert_eq!(index.doc_count().unwrap(), 1);
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "disposable audit marker, not user data\n");
}
