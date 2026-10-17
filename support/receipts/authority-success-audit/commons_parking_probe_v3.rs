#[path = "../../../crates/moot/commons/src/parking.rs"]
mod parking;
use muniment::{Backend, MemoryBackend, StoreError, WriteOp};
use p2panda_core::{Body, Header, Operation, SigningKey};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}};
use std::task::{Context, Poll, Wake, Waker};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ext { space: [u8;32] }
fn operation(seed:u8)->Operation<Ext> {
    let bytes=[9u8;16];
    let header=Header::builder().body(&bytes).seq_num(0).backlink(None)
        .build(&SigningKey::from_bytes(&[seed;32]),Ext{space:[8;32]});
    let operation=Operation{hash:header.hash(),header,body:Some(Body::from_bytes(&bytes))};
    assert!(p2panda_core::operation::validate_operation(&operation).is_ok());
    operation
}
struct Noop;
impl Wake for Noop { fn wake(self:Arc<Self>) {} }
fn block_on<T>(future:impl Future<Output=T>)->T {
    let waker=Waker::from(Arc::new(Noop));
    let mut context=Context::from_waker(&waker);
    let mut future=std::pin::pin!(future);
    loop { match future.as_mut().poll(&mut context) {
        Poll::Ready(value)=>return value,
        Poll::Pending=>std::thread::yield_now(),
    }}
}
#[derive(Clone)]
struct ConcurrentReads {
    inner:MemoryBackend,
    first_two:Arc<AtomicUsize>,
    barrier:Arc<Barrier>,
}
impl ConcurrentReads {
    fn new()->Self { Self{inner:MemoryBackend::new(),first_two:Arc::new(AtomicUsize::new(0)),barrier:Arc::new(Barrier::new(2))} }
}
#[async_trait::async_trait]
impl Backend for ConcurrentReads {
    async fn get(&self,k:&str)->Result<Option<Vec<u8>>,StoreError>{self.inner.get(k).await}
    async fn put(&self,k:&str,v:&[u8])->Result<(),StoreError>{self.inner.put(k,v).await}
    async fn delete(&self,k:&str)->Result<(),StoreError>{self.inner.delete(k).await}
    async fn scan(&self,a:&str,b:&str)->Result<Vec<String>,StoreError>{self.inner.scan(a,b).await}
    async fn apply(&self,ops:&[WriteOp])->Result<(),StoreError>{self.inner.apply(ops).await}
    async fn list(&self,prefix:&str)->Result<Vec<String>,StoreError>{
        let snapshot=self.inner.list(prefix).await?;
        if self.first_two.fetch_add(1,Ordering::SeqCst)<2 {self.barrier.wait();}
        Ok(snapshot)
    }
}
fn race(limits:parking::ParkingLimits)->parking::ParkingStatus {
    let backend=ConcurrentReads::new();
    let mut area=parking::Parking::new("audit/current-parking",[8;32]);
    area.limits=limits;
    let threads:Vec<_>=[1u8,2].into_iter().map(|seed|{
        let backend=backend.clone();let area=area.clone();
        std::thread::spawn(move||block_on(area.park(&backend,&operation(seed),0)).unwrap())
    }).collect();
    assert!(threads.into_iter().map(|t|t.join().unwrap()).all(|accepted|accepted));
    block_on(area.status(&backend)).unwrap()
}
#[test]
fn sequential_arrivals_obey_the_record_bound(){
    let backend=MemoryBackend::new();let mut area=parking::Parking::new("audit/sequential",[8;32]);
    area.limits.max_records=1;
    assert!(block_on(area.park(&backend,&operation(1),0)).unwrap());
    assert!(block_on(area.park(&backend,&operation(2),0)).unwrap());
    let status=block_on(area.status(&backend)).unwrap();
    assert_eq!(status.parked,1);assert_eq!(status.evicted_total,1);
}
#[test]
fn simultaneous_arrivals_exceed_the_record_bound(){
    let status=race(parking::ParkingLimits{max_records:1,max_bytes:16*1024*1024});
    assert_eq!(status.parked,2);assert_eq!(status.evicted_total,0);
}
#[test]
fn simultaneous_arrivals_exceed_the_byte_bound(){
    let bytes=|seed|p2panda_core::cbor::encode_cbor(&stickleback::operation_record(&operation(seed),true)).unwrap().len() as u64;
    let first=bytes(1);let second=bytes(2);let cap=first.max(second);
    let status=race(parking::ParkingLimits{max_records:10,max_bytes:cap});
    assert_eq!(status.parked,2);assert_eq!(status.parked_bytes,first+second);
    assert!(status.parked_bytes>cap);
    assert_eq!(status.evicted_total,0);
}
