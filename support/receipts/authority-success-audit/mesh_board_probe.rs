//! Audit observations against unchanged public Mesh APIs. Passing means the
//! named defect or qualified limit was observed, not repaired.
use identity::{IdentityProvider, InMemoryProvider, Ed25519Keypair};
use mesh::*;
use p2panda_core::{Operation, Header};
use stickleback::CheckpointAuthority;

const MESH: [u8;32] = [77;32];
fn key(seed:u8)->Ed25519Keypair {
    InMemoryProvider::from_seed([seed;32]).derive_keypair(MESH_AUTHOR_SALT).unwrap()
}
fn id(op:&Operation<MeshExt>)->[u8;32] { *op.hash.as_bytes() }
fn event(k:&Ed25519Keypair,e:MeshEvent,seq:u32,back:Option<[u8;32]>)->Operation<MeshExt> {
    to_operation(k,MESH,&e,seq,back)
}
fn output(bytes:&[u8])->JobOutput {
    JobOutput { name:"out".into(), blob:BlobRef::blake3(bytes),
        resource:ResourceId::parse("mesh.echo/v1").unwrap(),
        implementation:ImplementationId::parse("mesh.echo.native/v1").unwrap(),
        verification:VerificationClass::ExactBytes }
}
fn leased()->(JobSpec,Vec<Operation<MeshExt>>,JobId,LeaseId) {
    let poster=key(11); let worker=key(12);
    let spec=JobSpec::simple(ResourceId::parse("mesh.echo/v1").unwrap(),"in",
        BlobRef::blake3(b"input"),"out",16,DeterminismClass::Exact)
        .leased(LeaseTerms::new(60_000,10_000));
    spec.validate().unwrap();
    let post=event(&poster,MeshEvent::JobPostedV2{spec:Box::new(spec.clone()),nonce:1,at_ms:100},0,None);
    let job=JobId(id(&post));
    let claim=event(&worker,MeshEvent::JobClaimed{job:job.0,at_ms:200},0,None);
    let grant=event(&worker,MeshEvent::LeaseGranted{job:job.0,epoch:0,
        granted_at_ms:300,expires_at_ms:60_300},1,Some(id(&claim)));
    let lease=LeaseId(id(&grant));
    let done=event(&worker,MeshEvent::JobCompletedUnderLease{job:job.0,lease:lease.0,
        output:Box::new(output(b"valid")),at_ms:400},2,Some(id(&grant)));
    let ops=vec![post,claim,grant,done];
    assert!(ops.iter().all(verify));
    let board=JobBoard::fold(MESH,ops.iter());
    assert!(matches!(&board.job(job).unwrap().state,JobState::Committed{output:o,..} if **o==output(b"valid")));
    (spec,ops,job,lease)
}

#[test]
fn two_signed_results_from_one_winner_make_fold_input_order_visible() {
    let p=key(1);let w=key(2);
    let post=event(&p,MeshEvent::JobPosted{kind:JobKind::Echo,payload:b"input".to_vec(),nonce:1,at_ms:100},0,None);
    let job=JobId(id(&post));
    let claim=event(&w,MeshEvent::JobClaimed{job:job.0,at_ms:200},0,None);
    let a=event(&w,MeshEvent::JobDone{job:job.0,result:b"A".to_vec(),at_ms:300},1,Some(id(&claim)));
    let b=event(&w,MeshEvent::JobDone{job:job.0,result:b"B".to_vec(),at_ms:400},2,Some(id(&a)));
    assert!([&post,&claim,&a,&b].into_iter().all(verify));
    let ab=JobBoard::fold(MESH,[&post,&claim,&a,&b]);
    let ba=JobBoard::fold(MESH,[&post,&claim,&b,&a]);
    assert!(matches!(&ab.job(job).unwrap().state,JobState::Done{result,..} if result==b"B"));
    assert!(matches!(&ba.job(job).unwrap().state,JobState::Done{result,..} if result==b"A"));
    println!("OBSERVATION: identical signed operation set yields result B or A by iteration order");
}

#[test]
fn mutated_decoded_mesh_field_is_accepted_while_cached_signed_bytes_name_foreign_mesh() {
    let foreign=[88;32]; let p=key(3);
    let e=MeshEvent::JobPosted{kind:JobKind::Echo,payload:b"input".to_vec(),nonce:9,at_ms:100};
    let mut op=to_operation(&p,foreign,&e,0,None);
    let signed=op.header.encode(); let hash=id(&op);
    assert!(JobBoard::fold(MESH,[&op]).is_empty());
    op.header.extensions.mesh_id=MESH;
    assert_eq!(op.header.encode(),signed);
    assert_eq!(id(&op),hash);
    assert!(verify(&op));
    assert_eq!(from_operation(&op).unwrap().0,MESH);
    assert!(JobBoard::fold(MESH,[&op]).job(JobId(hash)).is_some());
    let original=Header::<MeshExt>::decode(&signed).unwrap();
    assert_eq!(original.extensions.mesh_id,foreign);
    println!("OBSERVATION: verify/fold trusts mutated parsed address; redecoded signed bytes still name foreign mesh");
}

#[test]
fn completed_lease_checkpoint_passes_policy_then_empty_tail_loses_output() {
    let (_,ops,job,_)=leased();let board=JobBoard::fold(MESH,ops.iter());
    let snapshot=JobBoardSnapshot::from_board(&board);
    let authority=key(11).public_key().to_bytes();
    let policy=MeshRetentionPolicy { revision:PolicyRevision([1;32]),checkpoint_authority:authority,
        availability:AvailabilityPolicy{promised_floor:KeepBound::Forever},
        erasure:ErasurePolicy{privacy_ceiling:KeepBound::UntilCheckpoint,terminal_job_payload:PayloadRule::Keep},
        lease:LeasePolicy{max_skew_ms:0} };
    assert!(snapshot.live_leases(500,&policy.lease).is_empty());
    let checkpoint=RetentionCheckpoint::new(MESH,policy.revision,policy.authority_revision(),vec![],snapshot,500);
    policy.validate_checkpoint(MESH,authority,None,&checkpoint).unwrap();
    let restored=JobBoard::fold_from_snapshot(MESH,&checkpoint.snapshot,std::iter::empty());
    assert!(matches!(board.job(job).unwrap().state,JobState::Committed{..}));
    assert!(matches!(restored.job(job).unwrap().state,JobState::Claimed{..}));
    assert!(matches!(restored.job(job).unwrap().lease_at(500,&policy.lease),LeasePhase::Done{..}));
    println!("OBSERVATION: accepted non-live checkpoint restores Claimed while lease phase remains Done");
}

#[test]
fn invalid_lower_hash_completion_supplies_output_after_valid_completion_closes_lease() {
    let (spec,mut ops,job,lease)=leased();let w=key(12);
    let good_hash=id(&ops[3]); let mut chosen=None;
    for n in 0u32..256 {
        let mut bad=output(&n.to_le_bytes());bad.blob.byte_len=17;
        assert!(bad.validate_self().is_ok()); assert!(bad.validate_against(&spec).is_err());
        let op=event(&w,MeshEvent::JobCompletedUnderLease{job:job.0,lease:lease.0,
            output:Box::new(bad.clone()),at_ms:500},3,Some(good_hash));
        if id(&op)<good_hash {chosen=Some((op,bad,n+1));break;}
    }
    let (bad,bad_output,attempts)=chosen.expect("bounded lower hash search");
    assert!(verify(&bad));
    let only_bad=JobBoard::fold(MESH,[&ops[0],&ops[1],&ops[2],&bad]);
    assert!(matches!(only_bad.job(job).unwrap().state,JobState::Claimed{..}));
    ops.push(bad);
    let board=JobBoard::fold(MESH,ops.iter());
    assert!(matches!(&board.job(job).unwrap().state,JobState::Committed{output:o,..} if **o==bad_output));
    assert!(matches!(board.job(job).unwrap().lease.epochs()[0].end,Some(LeaseEnd::Completed{at_ms:400})));
    println!("OBSERVATION: invalid 17-byte output against 16-byte grant selected; accepted closing fact remains valid at 400; hash attempts={attempts}");
}

#[test]
fn snapshot_only_roundtrip_drops_the_device_directory() {
    let provider=InMemoryProvider::from_seed([8;32]);
    let k=provider.derive_keypair(MESH_AUTHOR_SALT).unwrap();
    let attestation=provider.attest_derived_key(MESH_AUTHOR_SALT).unwrap();
    let op=event(&k,MeshEvent::DeviceAttested{attestation:Box::new(attestation)},0,None);
    let board=JobBoard::fold(MESH,[&op]);
    assert_eq!(board.devices().len(),1);
    let snapshot=JobBoardSnapshot::from_board(&board);
    let restored=JobBoard::fold_from_snapshot(MESH,&snapshot,std::iter::empty());
    assert!(restored.devices().is_empty());
    println!("OBSERVATION: snapshot-only reconstruction loses device; live store retention/replay not asserted");
}

fn policy(authority:&Ed25519Keypair)->MeshRetentionPolicy {
    MeshRetentionPolicy { revision:PolicyRevision([1;32]),checkpoint_authority:authority.public_key().to_bytes(),
        availability:AvailabilityPolicy{promised_floor:KeepBound::Forever},
        erasure:ErasurePolicy{privacy_ceiling:KeepBound::UntilCheckpoint,terminal_job_payload:PayloadRule::Keep},
        lease:LeasePolicy{max_skew_ms:0} }
}

#[test]
fn real_store_build_accept_board_loses_completed_lease_output() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let (_,ops,job,_)=leased();let authority=key(11);
        let store=MeshStore::in_memory_with_retention(policy(&authority));
        for op in &ops {assert!(store.accept(MESH,op).await.unwrap());}
        assert!(matches!(store.board(MESH).await.unwrap().job(job).unwrap().state,JobState::Committed{..}));
        let cp=store.build_checkpoint(MESH,500).await.unwrap();
        let op=event(&authority,MeshEvent::RetentionCheckpoint{checkpoint:Box::new(cp)},0,None);
        assert!(store.accept(MESH,&op).await.unwrap());
        assert!(matches!(store.board(MESH).await.unwrap().job(job).unwrap().state,JobState::Claimed{..}));
        assert!(store.operation(&ops[3].hash).await.unwrap().is_some());
        println!("OBSERVATION: real in-memory store checkpoint acceptance loses completed output before pruning, original completion still retained");
    });
}

#[test]
fn real_store_checkpoint_loses_attested_directory_before_prune() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let provider=InMemoryProvider::from_seed([8;32]);
        let k=provider.derive_keypair(MESH_AUTHOR_SALT).unwrap();
        let store=MeshStore::in_memory_with_retention(policy(&k));
        let op=event(&k,MeshEvent::DeviceAttested{attestation:Box::new(provider.attest_derived_key(MESH_AUTHOR_SALT).unwrap())},0,None);
        assert!(store.accept(MESH,&op).await.unwrap());
        assert_eq!(store.board(MESH).await.unwrap().devices().len(),1);
        let cp=store.build_checkpoint(MESH,500).await.unwrap();
        assert_eq!(cp.frontier.len(),1);
        let cp_op=event(&k,MeshEvent::RetentionCheckpoint{checkpoint:Box::new(cp)},0,None);
        assert!(store.accept(MESH,&cp_op).await.unwrap());
        assert!(store.board(MESH).await.unwrap().devices().is_empty());
        assert!(store.operation(&op.hash).await.unwrap().is_some());
        println!("OBSERVATION: real store checkpoint filters retained device attestation below frontier and snapshot omits directory");
    });
}

#[test]
fn real_store_accepts_overgrant_completion_then_board_publishes_it() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let (spec,mut ops,job,lease)=leased();let w=key(12);let good=id(&ops[3]);
        let mut chosen=None;
        for n in 0u32..256 {
            let mut bad=output(&n.to_le_bytes());bad.blob.byte_len=17;
            assert!(bad.validate_against(&spec).is_err());
            let op=event(&w,MeshEvent::JobCompletedUnderLease{job:job.0,lease:lease.0,output:Box::new(bad),at_ms:500},3,Some(good));
            if id(&op)<good {chosen=Some(op);break;}
        }
        ops.push(chosen.unwrap());let store=MeshStore::in_memory();
        for op in &ops {assert!(store.accept(MESH,op).await.unwrap());}
        let board=store.board(MESH).await.unwrap();
        assert!(matches!(&board.job(job).unwrap().state,JobState::Committed{output:o,..} if o.blob.byte_len==17));
        println!("OBSERVATION: store accepts self-valid over-grant record; real store board publishes rejected 17-byte output");
    });
}

#[test]
fn real_store_success_can_index_signed_foreign_header_under_mutated_mesh() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let foreign=[88;32];let p=key(3);let store=MeshStore::in_memory();
        let e=MeshEvent::JobPosted{kind:JobKind::Echo,payload:b"input".to_vec(),nonce:9,at_ms:100};
        let mut op=to_operation(&p,foreign,&e,0,None);
        assert!(store.accept(MESH,&op).await.is_err());
        op.header.extensions.mesh_id=MESH;
        assert!(store.accept(MESH,&op).await.unwrap());
        let retained=store.operation(&op.hash).await.unwrap().unwrap();
        assert_eq!(retained.header.extensions.mesh_id,foreign);
        assert_eq!(store.ops(MESH).await.unwrap().len(),1);
        assert!(store.board(MESH).await.unwrap().is_empty());
        println!("OBSERVATION: accept reports insertion under mesh M, retained signed bytes name foreign mesh and M board is empty");
    });
}
