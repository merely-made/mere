use audit_commons_spine::{GroupKeys, encrypted::EncryptedReplica};
use chartulary::{Author, Container};
use muniment::MemoryBackend;
use stickleback::DataKeyring;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
struct Noop;
impl Wake for Noop {fn wake(self:Arc<Self>) {}}
fn block_on<T>(future:impl Future<Output=T>)->T {
    let waker=Waker::from(Arc::new(Noop));
    let mut context=Context::from_waker(&waker);
    let mut future=std::pin::pin!(future);
    loop {match future.as_mut().poll(&mut context) {Poll::Ready(v)=>return v,Poll::Pending=>std::thread::yield_now()}}
}
fn keyring()->DataKeyring {
    let mut keys=DataKeyring::new();keys.rotate_random().unwrap();keys
}
#[test]
fn ordinary_edit_returns_a_stored_operation(){
    let mut replica=EncryptedReplica::new(MemoryBackend::new(),[8;32],[1;32],keyring());
    let operation=block_on(replica.edit(|g| {g.insert_node(&Author::new("audit"),Container::new("ordinary"));})).unwrap();
    assert!(block_on(replica.sync_store().has_operation(&operation.hash)).unwrap());
    assert_eq!(block_on(replica.sync_store().operation_count()).unwrap(),1);
    assert_eq!(block_on(replica.parking_status()).unwrap().parked,0);
}
#[test]
fn replacement_with_the_same_epoch_still_stores_the_edit(){
    let keys=GroupKeys::new(keyring());let persisted=keys.to_bytes().unwrap();let handle=keys.clone();
    let mut replica=EncryptedReplica::new(MemoryBackend::new(),[8;32],[2;32],keys);
    let operation=block_on(replica.edit(move|g| {
        g.insert_node(&Author::new("audit"),Container::new("same-epoch"));
        handle.replace_from_bytes(&persisted).unwrap();
    })).unwrap();
    assert!(block_on(replica.sync_store().has_operation(&operation.hash)).unwrap());
    assert_eq!(block_on(replica.parking_status()).unwrap().parked,0);
}
#[test]
fn replacement_with_an_unheld_epoch_returns_success_for_a_parked_edit(){
    let keys=GroupKeys::new(keyring());let handle=keys.clone();
    let mut replica=EncryptedReplica::new(MemoryBackend::new(),[8;32],[3;32],keys);
    let result=block_on(replica.edit(move|g| {
        g.insert_node(&Author::new("audit"),Container::new("parked-edit"));
        handle.replace(DataKeyring::new());
    }));
    let operation=result.expect("the current authoring API reports success");
    assert!(!block_on(replica.sync_store().has_operation(&operation.hash)).unwrap());
    assert_eq!(block_on(replica.sync_store().operation_count()).unwrap(),0);
    assert_eq!(block_on(replica.parking_status()).unwrap().parked,1);
    assert_eq!(block_on(replica.projection()).unwrap().graph.graph().node_count(),0);
}
