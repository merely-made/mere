use std::{collections::BTreeMap, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::SystemTime};
use muniment::{Backend, MemoryBackend, StoreError, TransactFn, WriteOp};
use pandect::{Author, CapturedDelta, MereSessions};
use uuid::Uuid;

fn author() -> Author { Author::person("audit").via("audit") }
fn add(n: u128) -> CapturedDelta { CapturedDelta::ReplayAddNodeWithIdIfMissing { id: Uuid::from_u128(n).to_string(), url: format!("https://audit.test/{n}"), position: [0., 0.] } }
fn title(n: u128, value: &str) -> CapturedDelta { CapturedDelta::ReplaySetNodeTitleById { node_id: Uuid::from_u128(n).to_string(), title: value.into() } }

#[derive(Clone, Default)]
struct Fault { inner: MemoryBackend, refuse: Arc<AtomicBool> }
#[async_trait::async_trait]
impl Backend for Fault {
    async fn get(&self, k: &str) -> Result<Option<Vec<u8>>, StoreError> { self.inner.get(k).await }
    async fn put(&self, k: &str, v: &[u8]) -> Result<(), StoreError> { self.inner.put(k, v).await }
    async fn delete(&self, k: &str) -> Result<(), StoreError> { self.inner.delete(k).await }
    async fn list(&self, p: &str) -> Result<Vec<String>, StoreError> { self.inner.list(p).await }
    async fn scan(&self, s: &str, e: &str) -> Result<Vec<String>, StoreError> { self.inner.scan(s, e).await }
    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        if self.refuse.load(Ordering::SeqCst) { return Err(StoreError::Backend("audit refusal".into())); }
        self.inner.apply(ops).await
    }
    async fn transact(&self, f: TransactFn) -> Result<(), StoreError> { self.inner.transact(f).await }
}

#[test]
fn stale_flush_overwrites_another_acknowledged_writer() { pollster::block_on(async {
    let sessions = MereSessions::new(MemoryBackend::new());
    let made = sessions.mint(author(), None).await.unwrap();
    let mut a = sessions.open(made.session_id).await.unwrap();
    let mut b = sessions.open(made.session_id).await.unwrap();
    a.apply(author(), vec![add(1)]).await.unwrap();
    b.apply(author(), vec![add(2)]).await.unwrap();
    let reopened = sessions.open(made.session_id).await.unwrap();
    assert!(a.graph().get_node_by_id(Uuid::from_u128(1)).is_some());
    assert!(reopened.graph().get_node_by_id(Uuid::from_u128(1)).is_none());
    assert!(reopened.graph().get_node_by_id(Uuid::from_u128(2)).is_some());
}); }

#[test]
fn transaction_accepts_a_batch_with_an_unapplied_missing_target() { pollster::block_on(async {
    let sessions = MereSessions::new(MemoryBackend::new());
    let made = sessions.mint(author(), None).await.unwrap();
    let mut session = sessions.open(made.session_id).await.unwrap();
    let deltas = vec![add(1), title(999, "never applied")].iter().map(|e| e.replay_delta_as(&author()).unwrap()).collect();
    let result = session.transact_deltas(author(), deltas, SystemTime::UNIX_EPOCH).await;
    assert!(result.is_ok());
    assert_eq!(session.graph().node_count(), 1);
    assert!(session.graph().get_node_by_id(Uuid::from_u128(999)).is_none());
}); }

#[test]
fn refused_merge_installs_unacknowledged_state_and_locks_source() { pollster::block_on(async {
    let store = Fault::default();
    let sessions = MereSessions::new(store.clone());
    let made = sessions.mint(author(), None).await.unwrap();
    let mut target = sessions.open(made.session_id).await.unwrap();
    target.apply(author(), vec![add(1), title(1, "before")]).await.unwrap();
    let made = sessions.branch_at(&target, target.journal().live_cursor(), author()).await.unwrap();
    let mut source = sessions.open(made.session_id).await.unwrap();
    source.apply(author(), vec![title(1, "merged")]).await.unwrap();
    let plan = sessions.merge_in(&target, &source).await.unwrap();
    assert!(plan.is_clean());
    store.refuse.store(true, Ordering::SeqCst);
    assert!(sessions.apply_merge(&mut target, &mut source, plan, &BTreeMap::new(), author()).await.is_err());
    assert_eq!(target.graph().get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "merged");
    assert!(source.ensure_writable().is_err());
    let disk_target = sessions.open(target.id()).await.unwrap();
    let disk_source = sessions.open(source.id()).await.unwrap();
    assert_eq!(disk_target.graph().get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "before");
    assert!(disk_source.ensure_writable().is_ok());
}); }

#[test]
fn successful_merge_leaves_the_published_graph_at_the_old_generation() { pollster::block_on(async {
    let sessions = MereSessions::new(MemoryBackend::new());
    let mut target = sessions.begin_recorded(author(), None);
    target.flush(SystemTime::UNIX_EPOCH).await.unwrap();
    let deltas = vec![add(1), title(1, "before")].iter().map(|e| e.replay_delta_as(&author()).unwrap()).collect();
    let (_, generation) = target.transact_deltas_published(author(), deltas, SystemTime::UNIX_EPOCH).await.unwrap();
    let made = sessions.branch_at(&target, target.journal().live_cursor(), author()).await.unwrap();
    let mut source = sessions.open(made.session_id).await.unwrap();
    source.apply(author(), vec![title(1, "merged")]).await.unwrap();
    let plan = sessions.merge_in(&target, &source).await.unwrap();
    sessions.apply_merge(&mut target, &mut source, plan, &BTreeMap::new(), author()).await.unwrap();
    let reader = sessions.open_resident_graph(target.id(), Default::default()).await.unwrap();
    assert_eq!(reader.generation(), generation);
    assert_eq!(reader.graph().get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "before");
    assert_eq!(target.graph().get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "merged");
}); }
