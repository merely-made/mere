use muniment_opfs_probe::{fault::{FaultBackend,FaultPlan,SharedMemory},workload};
use redb::StorageBackend;
use muniment::{Backend,MemoryBackend,StoreError,WriteOp};

fn cut_bytes(torn:usize)->usize {
    let image=SharedMemory::new();
    let fault=FaultBackend::new(image.clone(),FaultPlan{cut_at_write:Some(1),torn_bytes:torn,..Default::default()});
    assert!(fault.write(0,b"abcdefgh").is_err());
    assert!(fault.watch().counters().dead);
    image.len_bytes()
}

#[test]
fn named_half_write_policies_actually_repeat_full_write() {
    let native=[0,usize::MAX/2,usize::MAX].map(cut_bytes);
    let browser=[0,2usize.pow(31),u32::MAX as usize].map(cut_bytes);
    assert_eq!(native,[0,8,8]);assert_eq!(browser,[0,8,8]);
    assert_eq!(cut_bytes(4),4);
    println!("AUDIT native={native:?} browser={browser:?}; explicit torn_bytes=4 actually writes half of the eight-byte buffer");
}

struct MissingListedKey;
#[async_trait::async_trait]
impl Backend for MissingListedKey {
    async fn get(&self,_:&str)->Result<Option<Vec<u8>>,StoreError>{Ok(None)}
    async fn put(&self,_:&str,_:&[u8])->Result<(),StoreError>{unreachable!()}
    async fn delete(&self,_:&str)->Result<(),StoreError>{unreachable!()}
    async fn list(&self,_:&str)->Result<Vec<String>,StoreError>{Ok(vec!["ghost".into()])}
    async fn scan(&self,_:&str,_:&str)->Result<Vec<String>,StoreError>{unreachable!()}
    async fn apply(&self,_:&[WriteOp])->Result<(),StoreError>{unreachable!()}
}

#[test]
fn content_oracle_equates_missing_listed_key_with_real_empty_value() {
    pollster::block_on(async {
        let real=MemoryBackend::new();real.put("ghost",b"").await.unwrap();
        assert_eq!(real.get("ghost").await.unwrap(),Some(vec![]));
        assert_eq!(MissingListedKey.get("ghost").await.unwrap(),None);
        let present=workload::digest(&real).await.unwrap();
        let absent=workload::digest(&MissingListedKey).await.unwrap();
        assert_eq!(present,absent);assert_eq!(absent.1,1);
        println!("AUDIT real empty value and missing listed value produce identical digest/count: {absent:?}");
    });
}
