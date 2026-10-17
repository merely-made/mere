//! Diagnostic reproductions of observed storage contract violations.
use muniment::{Backend, FjallStore, Journal, JsonSlots, MemoryBackend, Seq, WriteOp};
use std::sync::{Arc, Barrier, atomic::{AtomicBool, Ordering}};

#[test]
fn a_reader_observes_a_partial_fjall_apply_batch() {
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(FjallStore::open(directory.path()).unwrap());
    let count = 40_000;
    let operations: Vec<_> = (0..count).map(|n| WriteOp::Put {
        key: format!("batch/{n:05}"), value: vec![b'x'],
    }).collect();
    let ready = Arc::new(Barrier::new(2));
    let committed = Arc::new(AtomicBool::new(false));
    let writer_store = store.clone();
    let writer_ready = ready.clone();
    let writer_committed = committed.clone();
    let writer = std::thread::spawn(move || {
        writer_ready.wait();
        pollster::block_on(writer_store.apply(&operations)).unwrap();
        writer_committed.store(true, Ordering::SeqCst);
    });
    ready.wait();
    let mut observed_partial = false;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while std::time::Instant::now() < deadline && !committed.load(Ordering::SeqCst) {
        if pollster::block_on(store.get("batch/00000")).unwrap().is_some()
            && pollster::block_on(store.get("batch/39999")).unwrap().is_none() {
            observed_partial = true;
            break;
        }
        std::thread::yield_now();
    }
    writer.join().unwrap();
    assert!(pollster::block_on(store.get("batch/39999")).unwrap().is_some());
    assert!(observed_partial, "a successful observation of the atomicity breach is required");
}

#[test]
fn whole_journal_load_accepts_future_causes_that_entry_load_refuses() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let slots = JsonSlots::new(backend.clone());
        backend.put("whole", br#"{"entries":["effect","cause"],"parents":[[1],[]]}"#).await.unwrap();
        let whole: Journal<String> = Journal::load(&slots, "whole").await.unwrap();
        assert_eq!(whole.parents(Seq(0)), &[Seq(1)]);
        assert_eq!(whole.causes(Seq(0)), vec![Seq(1)]);
        assert!(whole.effects(Seq(1)).is_empty());
        backend.put("entries/0000000000000000", br#"{"entry":"effect","causes":[1]}"#).await.unwrap();
        assert!(Journal::<String>::load_entries(&slots, "entries", None, None).await.is_err());
    });
}
