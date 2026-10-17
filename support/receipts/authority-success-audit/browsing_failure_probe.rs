use async_trait::async_trait;
use eidetic::browsing::{BrowsingMemory, BrowsingTrace, PageRef, TraceEvent, TraceTransition};
use eidetic::browsing::text::PageTextStore;
use muniment::{Backend, MemoryBackend, StoreError, WriteOp};
use std::sync::atomic::{AtomicBool, Ordering};

struct FaultStore {
    inner: MemoryBackend,
    writes: AtomicBool,
    deletes: AtomicBool,
}
impl FaultStore {
    fn new() -> Self {
        Self { inner: MemoryBackend::new(), writes: AtomicBool::new(false), deletes: AtomicBool::new(false) }
    }
}
#[async_trait]
impl Backend for FaultStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> { self.inner.get(key).await }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        if self.writes.load(Ordering::SeqCst) { return Err(StoreError::Backend("injected write refusal".into())); }
        self.inner.put(key, bytes).await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        if self.deletes.load(Ordering::SeqCst) { return Err(StoreError::Backend("injected delete refusal".into())); }
        self.inner.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> { self.inner.list(prefix).await }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> { self.inner.scan(start, end).await }
    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        if self.writes.load(Ordering::SeqCst) || (self.deletes.load(Ordering::SeqCst) && ops.iter().any(|op| matches!(op,WriteOp::Delete{..}))) {
            return Err(StoreError::Backend("injected batch refusal".into()));
        }
        self.inner.apply(ops).await
    }
}
fn event(url: &str, at_ms: u64) -> TraceEvent {
    TraceEvent { from: None, to: PageRef { url: url.into(), title: None }, transition: TraceTransition::LinkClick, at_ms, dwell_ms: None, candidates: vec![] }
}
fn times(memory: &BrowsingMemory, n: usize) -> Vec<u64> {
    memory.recent_corridor(n).iter().map(|e| e.at_ms).collect()
}

#[test]
fn a_failed_flush_loses_the_uncommitted_segment_and_retry_reports_nothing() {
    pollster::block_on(async {
        let mut store = FaultStore::new(); let mut memory = BrowsingMemory::new(1);
        memory.record_traversal("owner", event("https://pending.test/", 10));
        assert_eq!(times(&memory,10), vec![10]);
        store.writes.store(true,Ordering::SeqCst);
        assert!(memory.flush(&mut store,20).await.unwrap_err().to_string().contains("injected"));
        assert!(store.inner.list("").await.unwrap().is_empty());
        assert!(times(&memory,10).is_empty(), "actual implementation already removed uncommitted events");
        store.writes.store(false,Ordering::SeqCst);
        assert!(memory.flush(&mut store,30).await.unwrap().is_empty());
        assert!(BrowsingMemory::load(&mut store,1).await.unwrap().recent_corridor(10).is_empty());
    });
}

#[test]
fn a_successful_flush_retains_the_event_in_memory_and_on_reopen() {
    pollster::block_on(async {
        let mut store=FaultStore::new(); let mut memory=BrowsingMemory::new(1);
        memory.record_traversal("owner",event("https://pending.test/",10));
        assert_eq!(memory.flush(&mut store,20).await.unwrap().len(),1);
        assert_eq!(times(&memory,10),vec![10]);
        assert_eq!(times(&BrowsingMemory::load(&mut store,1).await.unwrap(),10),vec![10]);
    });
}

async fn populated(store: &mut FaultStore) -> BrowsingMemory {
    let mut memory=BrowsingMemory::new(1);
    for (url,time) in [("https://keep.test/",10),("https://forget.test/",20),("https://other.test/",30)] {
        memory.record_traversal("owner",event(url,time));
        memory.flush(store,40).await.unwrap();
    }
    memory
}

#[test]
fn failed_forget_drops_all_loaded_traces_but_none_of_the_stored_manifests() {
    pollster::block_on(async {
        let mut store=FaultStore::new(); let mut memory=populated(&mut store).await;
        assert_eq!(memory.traces().count(),3);
        let before=store.inner.list("").await.unwrap();
        store.deletes.store(true,Ordering::SeqCst);
        assert!(memory.forget_url(&mut store,"https://forget.test/").await.unwrap_err().to_string().contains("injected"));
        assert_eq!(store.inner.list("").await.unwrap(),before);
        assert_eq!(memory.traces().count(),0,"unrelated loaded traces were also discarded");
        store.deletes.store(false,Ordering::SeqCst);
        assert_eq!(memory.forget_url(&mut store,"https://forget.test/").await.unwrap(),0);
        assert_eq!(BrowsingMemory::load(&mut store,1).await.unwrap().traces().count(),3);
    });
}

#[test]
fn successful_forget_removes_the_target_and_preserves_unrelated_traces() {
    pollster::block_on(async {
        let mut store=FaultStore::new(); let mut memory=populated(&mut store).await;
        assert_eq!(memory.forget_url(&mut store,"https://forget.test/").await.unwrap(),1);
        assert_eq!(times(&memory,10),vec![10,30]);
        assert_eq!(times(&BrowsingMemory::load(&mut store,1).await.unwrap(),10),vec![10,30]);
    });
}

#[test]
fn open_owner_name_order_is_not_the_most_recent_event_order() {
    let mut memory=BrowsingMemory::new(8);
    memory.record_traversal("a",event("https://new.test/",20));
    memory.record_traversal("z",event("https://old.test/",10));
    assert_eq!(times(&memory,10),vec![20,10]);
    assert_eq!(times(&memory,1),vec![10]);
    let urls:Vec<_>=memory.nodes_visited_in_window(0,30).into_iter().map(|p|p.url).collect();
    assert_eq!(urls,vec!["https://new.test/","https://old.test/"]);
}

#[test]
fn individual_sorted_traces_can_overlap_and_return_the_wrong_recent_subset() {
    let mut memory=BrowsingMemory::new(8);
    memory.adopt(BrowsingTrace::from_events("a",vec![event("https://first.test/",10),event("https://last.test/",30)]));
    memory.adopt(BrowsingTrace::from_events("b",vec![event("https://middle.test/",20)]));
    assert_eq!(times(&memory,10),vec![20,10,30]);
    assert_eq!(times(&memory,2),vec![10,30]);
}

#[test]
fn current_text_read_accepts_corrupt_bytes_that_the_exact_version_reader_rejects() {
    pollster::block_on(async {
        let backend=MemoryBackend::new(); let store=PageTextStore::new(backend.clone());
        store.put("https://page.test/","original text",10).await.unwrap();
        let reference=store.reference("https://page.test/").await.unwrap().unwrap();
        backend.put(&format!("blob/{}",reference.blob),b"different valid UTF-8").await.unwrap();
        assert_eq!(store.text_for("https://page.test/").await.unwrap().as_deref(),Some("different valid UTF-8"));
        assert_eq!(store.load_all().await.unwrap().get("https://page.test/"),Some("different valid UTF-8"));
        assert!(store.text_at(&reference).await.unwrap_err().to_string().contains("hash mismatch"));
    });
}

#[test]
fn uncorrupted_current_and_exact_text_readers_agree() {
    pollster::block_on(async {
        let backend=MemoryBackend::new(); let store=PageTextStore::new(backend);
        store.put("https://page.test/","original text",10).await.unwrap();
        let reference=store.reference("https://page.test/").await.unwrap().unwrap();
        assert_eq!(store.text_for("https://page.test/").await.unwrap(),store.text_at(&reference).await.unwrap());
    });
}
