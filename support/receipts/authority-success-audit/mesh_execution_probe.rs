//! Native observations against unchanged public Mesh APIs. Green means observed.
use std::{collections::BTreeSet,sync::Arc,future::Future,task::Poll};
use mesh::*;
use mesh::namespace::{BoxFuture,JobNamespaceView};
use mesh::resource::{MeshResource,Prepared,ResourceDescriptor,ResourceError};
use mesh::resources::DelayedResource;
use identity::{IdentityProvider,InMemoryProvider,Ed25519Keypair};
use p2panda_core::Operation;
use stickleback::{CheckpointAuthority,DropExportSelector,DropExportDecision};
const M:[u8;32]=[71;32];
fn key(n:u8)->Ed25519Keypair {InMemoryProvider::from_seed([n;32]).derive_keypair(MESH_AUTHOR_SALT).unwrap()}
fn hash(op:&Operation<MeshExt>)->[u8;32] {*op.hash.as_bytes()}
fn op(k:&Ed25519Keypair,e:MeshEvent,seq:u32,back:Option<[u8;32]>)->Operation<MeshExt> {to_operation(k,M,&e,seq,back)}
fn simple(resource:&str,blob:BlobRef,class:DeterminismClass)->JobSpec {
 JobSpec::simple(ResourceId::parse(resource).unwrap(),"payload",blob,"result",64,class)
}
fn policy(k:&Ed25519Keypair)->MeshRetentionPolicy {
 MeshRetentionPolicy{revision:PolicyRevision([1;32]),checkpoint_authority:k.public_key().to_bytes(),availability:AvailabilityPolicy{promised_floor:KeepBound::Forever},erasure:ErasurePolicy{privacy_ceiling:KeepBound::UntilCheckpoint,terminal_job_payload:PayloadRule::Keep},lease:LeasePolicy{max_skew_ms:0}}
}
fn priorities()->MeshDropPriorities {MeshDropPriorities{retention_checkpoint:500,history_pruned:400,job_done:300,job_claimed:200,job_posted:100}}
fn privacy()->MeshDropPrivacy {MeshDropPrivacy{include_job_inputs:true,include_job_results:true}}

#[test]
fn legacy_job_ignores_resource_allowlist_while_v2_equivalent_is_blocked() {
 let p=key(1);let w=key(2);let me=w.public_key().to_bytes();let r=ResourceRegistry::builtin();
 let mut policy=DevicePolicy::permissive();policy.allowed_resources=BTreeSet::from([ResourceId::parse("mesh.echo/v1").unwrap()]);
 let offer=HostOffer::new(&r,HostFacts::cpu(4096),&policy);
 let old=op(&p,MeshEvent::JobPosted{kind:JobKind::Blake3,payload:b"input".to_vec(),nonce:1,at_ms:0},0,None);
 let job=JobId(hash(&old));let b=JobBoard::fold(M,[&old]);
 assert!(offer.can_run(b.job(job).unwrap()));assert_eq!(next_action(&b,&me,&offer),WorkerAction::Claim(job));
 let claim=op(&w,MeshEvent::JobClaimed{job:job.0,at_ms:1},0,None);
 assert_eq!(next_action(&JobBoard::fold(M,[&old,&claim]),&me,&offer),WorkerAction::Execute(job));
 let spec=simple("mesh.blake3/v1",BlobRef::blake3(b"input"),DeterminismClass::Exact);
 let new=op(&p,MeshEvent::JobPostedV2{spec:Box::new(spec),nonce:2,at_ms:2},1,Some(hash(&old)));
 assert_eq!(next_action(&JobBoard::fold(M,[&new]),&me,&offer),WorkerAction::Idle);
 println!("OBSERVATION: excluded legacy Blake3 is claimed/executed; equivalent V2 is idle");
}

fn held()->(Vec<Operation<MeshExt>>,JobId,[u8;32]) {
 let p=key(3);let w=key(4);let me=w.public_key().to_bytes();
 let spec=simple("mesh.echo/v1",BlobRef::blake3(b"input"),DeterminismClass::Exact).leased(LeaseTerms::new(60_000,10_000));
 let post=op(&p,MeshEvent::JobPostedV2{spec:Box::new(spec),nonce:0,at_ms:0},0,None);let id=JobId(hash(&post));
 let claim=op(&w,MeshEvent::JobClaimed{job:id.0,at_ms:100},0,None);
 let grant=op(&w,MeshEvent::LeaseGranted{job:id.0,epoch:0,granted_at_ms:200,expires_at_ms:60_200},1,Some(hash(&claim)));
 (vec![post,claim,grant],id,me)
}

#[test]
fn unstarted_held_job_executes_despite_owner_withholding_or_capacity_zero() {
 let (ops,id,me)=held();let b=JobBoard::fold(M,ops.iter());let r=ResourceRegistry::builtin();
 let conservative=DevicePolicy::conservative();let offer=HostOffer::new(&r,HostFacts::cpu(4096),&conservative).at(1000).observing(DeviceConditions::spare().in_use()).with_lease_policy(LeasePolicy{max_skew_ms:0});
 assert_eq!(offer.must_stop(),Some(ReclaimReason::ForegroundActivity));assert!(!offer.accepting());
 assert_eq!(next_action(&b,&me,&offer),WorkerAction::Execute(id));
 let running=[id];assert!(matches!(next_action(&b,&me,&offer.running(&running)),WorkerAction::Reclaim{..}));
 let mut zero=DevicePolicy::permissive();zero.max_concurrent_jobs=0;
 let no_capacity=HostOffer::new(&r,HostFacts::cpu(4096),&zero).at(1000);
 assert!(!no_capacity.accepting());assert_eq!(next_action(&b,&me,&no_capacity),WorkerAction::Execute(id));
 println!("OBSERVATION: held-but-unstarted job Execute with owner withholding, and with zero configured capacity; running control reclaims");
}

#[test]
fn delayed_adapter_units_are_unbound_by_exact_resource_and_implementation_identity() {
 tokio::runtime::Runtime::new().unwrap().block_on(async {
  let space=MemoryBlobSpace::in_memory();let blob=space.put(b"seed").await.unwrap();
  let spec=simple("mesh.delayed/v1",blob,DeterminismClass::Exact);let (_,c)=JobControl::new();
  let mut a=ResourceRegistry::new();a.register(Arc::new(DelayedResource::new(1,0))).unwrap();
  let mut b=ResourceRegistry::new();b.register(Arc::new(DelayedResource::new(2,0))).unwrap();
  let x=run_job(&a,&spec,&space,&space,&c).await.unwrap();let y=run_job(&b,&spec,&space,&space,&c).await.unwrap();
  assert_eq!(x.resource,y.resource);assert_eq!(x.implementation,y.implementation);assert_eq!(x.verification,y.verification);assert_ne!(x.blob,y.blob);
  assert!(matches!(verify_output(&b,&spec,&x,&space,&space,&c).await.unwrap(),Verdict::Diverged{by,..} if by==x.implementation));
  println!("OBSERVATION: same exact resource, implementation, input and spec yield different bytes solely by constructor unit count");
 });
}

#[test]
fn cancellation_during_final_real_adapter_pause_still_commits_output() {
 tokio::runtime::Runtime::new().unwrap().block_on(async {
  let space=MemoryBlobSpace::in_memory();let blob=space.put(b"seed").await.unwrap();
  let spec=simple("mesh.delayed/v1",blob,DeterminismClass::Exact);let (handle,c)=JobControl::new();
  let mut r=ResourceRegistry::new();r.register(Arc::new(DelayedResource::new(1,100))).unwrap();
  let mut run=Box::pin(run_job(&r,&spec,&space,&space,&c));
  std::future::poll_fn(|cx| {
   assert!(run.as_mut().poll(cx).is_pending());assert_eq!(handle.progress().done,1);handle.cancel();Poll::Ready(())
  }).await;
  assert!(c.is_cancelled());let out=run.await.unwrap();
  assert_eq!(space.get(&out.blob).await.unwrap(),Some(DelayedResource::expected(b"seed",1)));
  println!("OBSERVATION: cancel is visible during last adapter await, yet run_job returns success and stores output");
 });
}

struct Weak(ResourceDescriptor);
impl MeshResource for Weak {
 fn descriptor(&self)->&ResourceDescriptor {&self.0}
 fn prepare<'a>(&'a self,_:&'a JobNamespaceView<'a>)->BoxFuture<'a,Result<Prepared,ResourceError>> {Box::pin(async{Ok(Prepared::new(()))})}
 fn execute<'a>(&'a self,_:Prepared,_:&'a JobControl)->BoxFuture<'a,Result<Vec<u8>,ResourceError>> {Box::pin(async{Ok(b"bytes".to_vec())})}
}
#[test]
fn exact_verdict_is_reported_using_local_producer_only_adapter() {
 tokio::runtime::Runtime::new().unwrap().block_on(async {
  let space=MemoryBlobSpace::in_memory();let blob=space.put(b"input").await.unwrap();
  let spec=simple("test.mixed/v1",blob,DeterminismClass::Observed);let (_,c)=JobControl::new();
  let mut r=ResourceRegistry::new();r.register(Arc::new(Weak(ResourceDescriptor{resource:spec.resource.clone(),implementation:ImplementationId::parse("test.weak/v1").unwrap(),requires:ResourceRequirements::cpu(),verification:VerificationClass::ProducerOnly}))).unwrap();
  let local=run_job(&r,&spec,&space,&space,&c).await.unwrap();assert_eq!(local.verification,VerificationClass::ProducerOnly);
  let mut remote=local.clone();remote.implementation=ImplementationId::parse("test.exact/v1").unwrap();remote.verification=VerificationClass::ExactBytes;
  assert!(matches!(verify_output(&r,&spec,&remote,&space,&space,&c).await.unwrap(),Verdict::Reproduced{..}));
  remote.blob=BlobRef::blake3(b"other");
  assert!(matches!(verify_output(&r,&spec,&remote,&space,&space,&c).await.unwrap(),Verdict::Diverged{..}));
  let normal=verify_output(&r,&spec,&local,&space,&space,&c).await.unwrap();assert!(matches!(normal,Verdict::NotCheckable{..}));
  println!("OBSERVATION: local ProducerOnly is NotCheckable ordinarily, but yields Reproduced/Diverged against a producer ExactBytes declaration");
 });
}

#[test]
fn catchup_checkpoint_plus_tail_cannot_import_nonroot_tail_into_empty_store() {
 tokio::runtime::Runtime::new().unwrap().block_on(async {
  let k=key(9);let pol=policy(&k);let source=MeshStore::in_memory_with_retention(pol.clone());
  let first=op(&k,MeshEvent::JobPosted{kind:JobKind::Echo,payload:b"one".to_vec(),nonce:1,at_ms:0},0,None);
  assert!(source.accept(M,&first).await.unwrap());
  let cp=source.build_checkpoint(M,100).await.unwrap();
  let cp_op=op(&k,MeshEvent::RetentionCheckpoint{checkpoint:Box::new(cp)},0,None);assert!(source.accept(M,&cp_op).await.unwrap());
  let tail=op(&k,MeshEvent::JobPosted{kind:JobKind::Echo,payload:b"two".to_vec(),nonce:2,at_ms:200},1,Some(hash(&first)));
  assert!(source.accept(M,&tail).await.unwrap());
  let stored=source.latest_checkpoint(M).await.unwrap().unwrap();let selector=MeshDropSelector::catch_up(Some(&stored),privacy(),priorities());
  assert_eq!(selector.select(&first),DropExportDecision::Omit);assert!(matches!(selector.select(&cp_op),DropExportDecision::Full{..}));assert!(matches!(selector.select(&tail),DropExportDecision::Full{..}));
  let receiver=MeshStore::in_memory_with_retention(pol);assert!(receiver.accept(M,&cp_op).await.unwrap());
  let err=receiver.accept(M,&tail).await.unwrap_err();assert!(err.to_string().contains("missing its predecessor"),"{err}");
  assert!(receiver.board(M).await.unwrap().job(JobId(hash(&first))).is_some());assert!(receiver.board(M).await.unwrap().job(JobId(hash(&tail))).is_none());
  assert!(receiver.accept(M,&first).await.unwrap());assert!(receiver.accept(M,&tail).await.unwrap());
  println!("OBSERVATION: owner checkpoint imports; selected tail fails MissingPredecessor until policy-omitted original prefix supplied");
 });
}

#[test]
fn attestation_is_omitted_by_catchup_even_though_privacy_and_priority_keep_it() {
 let provider=InMemoryProvider::from_seed([7;32]);let k=provider.derive_keypair(MESH_AUTHOR_SALT).unwrap();
 let att=op(&k,MeshEvent::DeviceAttested{attestation:Box::new(provider.attest_derived_key(MESH_AUTHOR_SALT).unwrap())},0,None);
 let board=JobBoard::fold(M,[&att]);let pol=policy(&k);
 let cp=RetentionCheckpoint::new(M,pol.revision,pol.authority_revision(),vec![LogFrontier{author:k.public_key().to_bytes(),log_id:MeshLogId::Events,seq_num:0,operation:hash(&att)}],JobBoardSnapshot::from_board(&board),100);
 let cpop=op(&k,MeshEvent::RetentionCheckpoint{checkpoint:Box::new(cp.clone())},0,None);
 let stored=StoredCheckpoint{operation:hash(&cpop),checkpoint:cp};
 assert!(matches!(MeshDropSelector::archive(privacy(),priorities()).select(&att),DropExportDecision::Full{priority:500}));
 assert_eq!(MeshDropSelector::catch_up(Some(&stored),privacy(),priorities()).select(&att),DropExportDecision::Omit);
 println!("OBSERVATION: catch-up frontier overrides device-always-travels comment; snapshot has no device directory (Q202)");
}
