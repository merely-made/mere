//! Explicit read/write interleavings over public production APIs.
//! This characterizes the resident hint critical section, not daemon timing.
use djinn::{pairing::{pair_device,unpair_device,PairOutcome,UnpairOutcome}, settings::{OwnerSettings,SyncSettings,settings_path}};
use personae::ProfileId;
use std::path::Path;
const ROOT: &str = "C:/t/cargo-targets/mere/authority-success-settings-fixture";
fn initial(profile: &ProfileId) -> std::path::PathBuf {
    let root=Path::new(ROOT);
    assert_eq!(std::fs::read_to_string(root.join("AUDIT-FIXTURE-MARKER")).unwrap().trim_end(),"authority-success audit generated settings fixture");
    let path=settings_path(root,profile);
    assert!(!path.exists());
    let mut sync=SyncSettings {graph:"authority-success-synthetic-graph".into(),..Default::default()};
    assert!(sync.pair([0x11;32],Some([0xa1;32]),"existing synthetic peer",1));
    OwnerSettings {sync:Some(sync),..Default::default()}.save(&path).unwrap();
    path
}
#[test]
fn stale_endpoint_hint_save_restores_successfully_unpaired_authority() {
    let profile=ProfileId("removed".into());let path=initial(&profile);
    // refresh_dial_hint's OwnerSettings::load occurs before this management edit.
    let mut hint=OwnerSettings::load(&path).unwrap();
    assert!(matches!(unpair_device(Path::new(ROOT),&profile,[0x11;32]).unwrap(),UnpairOutcome::Removed{..}));
    assert!(OwnerSettings::load(&path).unwrap().sync.unwrap().paired_devices.is_empty());
    assert!(hint.sync.as_mut().unwrap().record_endpoint(&[0x11;32],"synthetic-current-endpoint"));
    hint.save(&path).unwrap();
    let after=OwnerSettings::load(&path).unwrap().sync.unwrap();
    assert_eq!(after.paired_devices.len(),1);
    assert_eq!(after.roster_root_keys().unwrap(),vec![[0xa1;32]]);
    // Fresh-read control: a hint cannot target the device after its removal.
    unpair_device(Path::new(ROOT),&profile,[0x11;32]).unwrap();
    let mut fresh=OwnerSettings::load(&path).unwrap();
    assert!(!fresh.sync.as_mut().unwrap().record_endpoint(&[0x11;32],"synthetic-current-endpoint"));
    assert!(OwnerSettings::load(&path).unwrap().sync.unwrap().paired_devices.is_empty());
    println!("unpair returned Removed; stale hint restored paired device AND roster root; fresh-read control preserved removal");
}
#[test]
fn stale_endpoint_hint_save_loses_successfully_added_pair() {
    let profile=ProfileId("added".into());let path=initial(&profile);
    let mut hint=OwnerSettings::load(&path).unwrap();
    assert!(matches!(pair_device(Path::new(ROOT),&profile,[0x22;32],Some([0xa2;32]),"new synthetic peer",2).unwrap(),PairOutcome::Added{..}));
    assert_eq!(OwnerSettings::load(&path).unwrap().sync.unwrap().paired_devices.len(),2);
    assert!(hint.sync.as_mut().unwrap().record_endpoint(&[0x11;32],"synthetic-current-endpoint"));
    hint.save(&path).unwrap();
    let after=OwnerSettings::load(&path).unwrap().sync.unwrap();
    assert_eq!(after.paired_devices.len(),1);
    assert!(!after.paired_node_keys().unwrap().contains(&[0x22;32]));
    assert!(!after.roster_root_keys().unwrap().contains(&[0xa2;32]));
    // With no intervening edit, loading and updating preserves the new pair.
    pair_device(Path::new(ROOT),&profile,[0x22;32],Some([0xa2;32]),"new synthetic peer",3).unwrap();
    let mut fresh=OwnerSettings::load(&path).unwrap();
    assert!(fresh.sync.as_mut().unwrap().record_endpoint(&[0x11;32],"synthetic-newer-endpoint"));
    fresh.save(&path).unwrap();
    assert_eq!(OwnerSettings::load(&path).unwrap().sync.unwrap().paired_devices.len(),2);
    println!("pair returned Added; stale hint removed that pair AND roster root; fresh-read control preserved both pairs");
}

#[test]
fn installing_transfer_rule_replaces_callers_synthetic_projection_policy() {
    use graphshell::personal_sync::{PersonalGraphEvent,PersonalGraphReplica,SyncRoster,SyncSelection,SyntheticAddressRule};
    use personae::IdentityProvider;
    let identity=personae::InMemoryProvider::from_seed([0x45;32]);
    let roster=SyncRoster::new([identity.master_public_key().to_bytes()]);
    let supplied=SyncSelection::default().with_synthetic_addresses([SyntheticAddressRule {
        prefix:"mere://audit-hidden/".into(),facet:"audit.hidden".into(),device_scoped:false
    }]);
    // Exact builder composition in PersonalSyncHost::open_with_blob_custody.
    let installed=supplied.clone().with_local_device("synthetic-local-device").with_synthetic_addresses([graphshell::transfer_offer::transfer_offer_rule()]);
    let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let mut count=Vec::new();
        for selection in [supplied,installed] {
            let mut replica=PersonalGraphReplica::for_identity(muniment::MemoryBackend::new(),[0x46;32],&identity,roster.clone(),selection).unwrap();
            replica.author(vec![PersonalGraphEvent::AddNode {id:uuid::Uuid::from_u128(2),address:"https://audit.invalid/plain".into(),title:"plain control".into()}]).await.unwrap();
            let admitted=replica.author(vec![PersonalGraphEvent::AddNode {id:uuid::Uuid::from_u128(1),address:"mere://audit-hidden/record".into(),title:"hidden carrier".into()}]).await.is_ok();
            let projection=replica.projection().await.unwrap();
            let hidden=projection.graph.nodes().filter(|(_,n)|n.url()=="mere://audit-hidden/record").count();
            let plain=projection.graph.nodes().filter(|(_,n)|n.url()=="https://audit.invalid/plain").count();
            count.push((admitted,hidden,plain));
        }
        assert_eq!(count,vec![(false,0,1),(true,1,1)]);
        println!("supplied carrier rule admits/projects hidden/plain={:?}; host builder replacement yields {:?}",count[0],count[1]);
    });
}
