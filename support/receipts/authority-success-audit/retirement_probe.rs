//! Exact production Athanor module against retained Cargo-built dependencies.
#[path = "../../../ports/distillery/athanor/src/lib.rs"]
mod athanor;
use eidetic::{DeletedNode, MemoryBackend, list_deleted, record_deleted};

fn deleted(id: &str, at: u64) -> DeletedNode {
    DeletedNode { node_id: id.into(), url: "https://synthetic.invalid/".into(),
        title: None, tags: vec![], graph_id: None, deleted_at_ms: at,
        nested: None, facets: None }
}

async fn exercise(same_id: bool) {
    let mut store = MemoryBackend::default();
    record_deleted(&mut store, &deleted("first", 100)).await.unwrap();
    record_deleted(&mut store, &deleted(if same_id { "first" } else { "second" }, 999)).await.unwrap();
    let records = list_deleted(&mut store).await.unwrap();
    assert_eq!(records.len(), 2);
    let proposal = athanor::propose_retirement(&records, 100, 1_000);
    assert_eq!(proposal.node_ids, vec!["first"]);
    let removed = athanor::apply_retirement(&mut store, &proposal).await.unwrap();
    let after = list_deleted(&mut store).await.unwrap();
    if same_id {
        assert_eq!(removed, 2);
        assert!(after.is_empty());
        println!("OBSERVATION: retirement selected one expired node id and removed both its expired and unexpired tombstones");
    } else {
        assert_eq!(removed, 1);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].deleted_at_ms, 999);
        println!("CONTROL: an unexpired tombstone with another node id survives retirement");
    }
}

fn main() {
    pollster::block_on(exercise(true));
    pollster::block_on(exercise(false));
    println!("2 checks passed");
}
