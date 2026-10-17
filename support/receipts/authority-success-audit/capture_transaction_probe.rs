use async_trait::async_trait;
use graphshell::{
    access::{AccessContext, AccessRecord},
    capture::{BrowserHistoryCapture, BrowserVisit, HistoryCapturePolicy, ForgetMode, BROWSER_HISTORY_FACET, browser_encounters},
    mere_host::{MereHost, SelectedPersonaRef, fixture_handlers},
};
use eidetic::import::encounters::{BrowserParent, BrowserSourceScope};
use mere::kernel::graph::{CapturedDelta, NavigationTrigger, RelationKind};
use muniment::{Backend, MemoryBackend, StoreError, WriteOp};
use std::sync::{Arc,atomic::{AtomicBool, Ordering}};

#[derive(Clone)]
struct FaultStore { inner: MemoryBackend, reject: Arc<AtomicBool> }
impl FaultStore {
    fn new() -> Self { Self { inner: MemoryBackend::new(), reject: Arc::new(AtomicBool::new(false)) } }
}
#[async_trait]
impl Backend for FaultStore {
    async fn get(&self,key:&str)->Result<Option<Vec<u8>>,StoreError>{self.inner.get(key).await}
    async fn put(&self,key:&str,value:&[u8])->Result<(),StoreError>{self.inner.put(key,value).await}
    async fn delete(&self,key:&str)->Result<(),StoreError>{self.inner.delete(key).await}
    async fn list(&self,prefix:&str)->Result<Vec<String>,StoreError>{self.inner.list(prefix).await}
    async fn scan(&self,start:&str,end:&str)->Result<Vec<String>,StoreError>{self.inner.scan(start,end).await}
    async fn apply(&self,ops:&[WriteOp])->Result<(),StoreError>{
        if self.reject.load(Ordering::SeqCst){return Err(StoreError::Backend("injected capture apply refusal".into()));}
        self.inner.apply(ops).await
    }
}
const A:&str="https://capture-a.test/";
const B:&str="https://capture-b.test/";
fn selected()->SelectedPersonaRef {SelectedPersonaRef{persona:"personae://synthetic-capture".into(),profile:"synthetic-audit".into()}}
fn context()->AccessContext {AccessContext{persona:"personae://synthetic-capture".into(),device:"personae://synthetic-device".into(),at_ms:10}}
fn visit(id:&str,url:&str)->BrowserVisit {
    BrowserVisit{source:"firefox".into(),visit_id:Some(id.into()),source_scope:Some(BrowserSourceScope{product:"firefox".into(),profile:"synthetic-profile".into(),generation:"synthetic-generation".into()}),parent:BrowserParent::Root,original_unix_micros:None,source_is_local:None,url:url.into(),title:Some(id.into()),favicon_url:None,referrer_url:None,transition:"link".into(),at_ms:1000,private:false}
}
async fn setup(policy:HistoryCapturePolicy)->(FaultStore,MereHost<FaultStore>,BrowserHistoryCapture) {
    let mut store=FaultStore::new();
    let mut host=MereHost::empty(store.clone(),selected(),fixture_handlers(),context());
    host.persist(1).await.unwrap();
    let capture=BrowserHistoryCapture::load(&mut store,policy).await.unwrap();
    (store,host,capture)
}
async fn ingest(store:&mut FaultStore,host:&mut MereHost<FaultStore>,capture:&mut BrowserHistoryCapture,visits:Vec<BrowserVisit>)->Result<(),graphshell::capture::CaptureError>{
    capture.ingest_batch(host,store,visits,"personae://synthetic-capture","personae://synthetic-device",2).await.map(|_|())
}

#[test]
fn refused_capture_graph_changes_are_committed_by_a_different_delivery_without_their_ledger_or_trace() {
    pollster::block_on(async {
        let (mut store,mut host,mut capture)=setup(HistoryCapturePolicy::consented()).await;
        store.reject.store(true,Ordering::SeqCst);
        assert!(ingest(&mut store,&mut host,&mut capture,vec![visit("a",A)]).await.is_err());
        assert!(host.graph().get_node_by_url(A).is_some());
        assert!(capture.memory().recent_corridor(10).is_empty());
        assert!(browser_encounters(&host,&store).await.unwrap().snapshot.visits.is_empty());
        store.reject.store(false,Ordering::SeqCst);
        ingest(&mut store,&mut host,&mut capture,vec![visit("b",B)]).await.unwrap();
        let ledger=browser_encounters(&host,&store).await.unwrap();
        assert_eq!(ledger.snapshot.visits.len(),1);
        assert_eq!(ledger.snapshot.visits[0].context.url,B);
        assert_eq!(capture.memory().recent_corridor(10).len(),1);
        assert_eq!(eidetic::list_typed::<AccessRecord>(&mut store).await.unwrap().len(),1);
        let reopened=MereHost::open(store.clone(),selected(),fixture_handlers(),context()).await.unwrap();
        assert!(reopened.graph().get_node_by_url(A).is_some(),"refused delivery became durable anyway");
        assert!(reopened.facet_value(A,BROWSER_HISTORY_FACET).is_some());
        assert_eq!(reopened.access_history_for(A).unwrap().records.len(),1);
    });
}

#[test]
fn successful_capture_keeps_graph_ledger_access_records_and_memory_consistent() {
    pollster::block_on(async {
        let(mut store,mut host,mut capture)=setup(HistoryCapturePolicy::consented()).await;
        ingest(&mut store,&mut host,&mut capture,vec![visit("a",A),visit("b",B)]).await.unwrap();
        assert_eq!(browser_encounters(&host,&store).await.unwrap().snapshot.visits.len(),2);
        assert_eq!(capture.memory().recent_corridor(10).len(),2);
        assert_eq!(eidetic::list_typed::<AccessRecord>(&mut store).await.unwrap().len(),2);
        let reopened=MereHost::open(store.clone(),selected(),fixture_handlers(),context()).await.unwrap();
        for url in[A,B]{assert!(reopened.facet_value(url,BROWSER_HISTORY_FACET).is_some());}
    });
}

#[test]
fn retrying_the_identical_delivery_masks_the_partial_graph_rollback() {
    pollster::block_on(async {
        let(mut store,mut host,mut capture)=setup(HistoryCapturePolicy::consented()).await;
        store.reject.store(true,Ordering::SeqCst);
        assert!(ingest(&mut store,&mut host,&mut capture,vec![visit("a",A)]).await.is_err());
        store.reject.store(false,Ordering::SeqCst);
        ingest(&mut store,&mut host,&mut capture,vec![visit("a",A)]).await.unwrap();
        assert_eq!(browser_encounters(&host,&store).await.unwrap().snapshot.visits.len(),1);
        assert_eq!(capture.memory().recent_corridor(10).len(),1);
        assert_eq!(eidetic::list_typed::<AccessRecord>(&mut store).await.unwrap().len(),1);
        assert_eq!(host.facet_value(A,BROWSER_HISTORY_FACET).unwrap()["visit_count"],1);
    });
}

#[test]
fn refused_forget_graph_changes_survive_a_different_capture_while_old_evidence_remains() {
    pollster::block_on(async {
        let(mut store,mut host,mut capture)=setup(HistoryCapturePolicy::consented()).await;
        ingest(&mut store,&mut host,&mut capture,vec![visit("a",A)]).await.unwrap();
        store.reject.store(true,Ordering::SeqCst);
        assert!(capture.forget_url(&mut host,&mut store,A,ForgetMode::HistoryOnly,3).await.is_err());
        assert_eq!(capture.memory().recent_corridor(10).len(),1);
        assert!(host.facet_value(A,BROWSER_HISTORY_FACET).is_none());
        store.reject.store(false,Ordering::SeqCst);
        ingest(&mut store,&mut host,&mut capture,vec![visit("b",B)]).await.unwrap();
        assert_eq!(browser_encounters(&host,&store).await.unwrap().snapshot.visits.len(),2);
        assert_eq!(eidetic::list_typed::<AccessRecord>(&mut store).await.unwrap().len(),2);
        assert_eq!(capture.memory().recent_corridor(10).len(),2);
        let reopened=MereHost::open(store.clone(),selected(),fixture_handlers(),context()).await.unwrap();
        assert!(reopened.graph().get_node_by_url(A).is_some());
        assert!(reopened.facet_value(A,BROWSER_HISTORY_FACET).is_none());
        assert!(reopened.access_history_for(A).unwrap().records.is_empty());
    });
}

#[test]
fn segment_size_one_and_sixty_four_both_store_all_batch_events_in_one_trace() {
    pollster::block_on(async {
        for size in[1,64]{
            let mut policy=HistoryCapturePolicy::consented();policy.segment_size=size;
            let(mut store,mut host,mut capture)=setup(policy).await;
            ingest(&mut store,&mut host,&mut capture,vec![visit("a",A),visit("b",B),visit("c","https://capture-c.test/")]).await.unwrap();
            assert_eq!(capture.memory().traces().count(),1);
            assert_eq!(capture.memory().traces().next().unwrap().events.len(),3);
        }
    });
}

#[test]
fn browser_history_forget_retracts_another_applications_traversal_for_the_same_object() {
    pollster::block_on(async {
        let(mut store,mut host,mut capture)=setup(HistoryCapturePolicy::consented()).await;
        let c="https://capture-c.test/";let d="https://capture-d.test/";
        ingest(&mut store,&mut host,&mut capture,vec![visit("a",A),visit("b",B),visit("c",c),visit("d",d)]).await.unwrap();
        for(from,to)in[(A,B),(c,d)]{
            let from_id=host.graph().get_node_by_url(from).unwrap().1.id.to_string();
            let to_id=host.graph().get_node_by_url(to).unwrap().1.id.to_string();
            host.through("synthetic.other-application".into(),|host|host.apply_edits(vec![CapturedDelta::ReplayAppendTraversalByIds{from_id,to_id,trigger:NavigationTrigger::AddressBarEntry,timestamp_ms:4000}])).unwrap();
        }
        host.persist(4).await.unwrap();
        assert_eq!(host.graph().relations().filter(|r|r.kind==RelationKind::Traversal).count(),2);
        capture.forget_url(&mut host,&mut store,A,ForgetMode::HistoryOnly,5).await.unwrap();
        assert!(host.graph().get_node_by_url(A).is_some());
        let held:Vec<_>=host.graph().relations().filter(|r|r.kind==RelationKind::Traversal).collect();
        assert_eq!(held.len(),1,"nonbrowser traversal incident to the forgotten object was retracted");
        assert_eq!(host.graph().get_node(held[0].from).unwrap().url(),c);
        assert_eq!(host.graph().get_node(held[0].to).unwrap().url(),d);
    });
}
