// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MPL-2.0
//! Real owner replacement and two admitted apps; public synthetic facts only.
use castellan::{
    authority::PersonaeHost,
    custody::{IdentityVault, InMemoryStorage, Profile},
};
use djinn::{
    embedded_reservoir::{ReservoirAttachment, ReservoirAttachmentOptions},
    personal_facts::PersonalFactsClient,
};
use graphshell_endpoint::door::{app_admission::AppId, identity::VaultProtectionView};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use std::{process::Command, sync::Arc, time::Duration};
#[test]
fn shared_births_survive_a_fresh_owner_process() {
    let root = tempfile::tempdir().unwrap();
    for mode in ["create", "reopen"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "personal_facts_child",
                "--nocapture",
            ])
            .env("PERSONAL_FACTS_TEST_ROOT", root.path())
            .env("PERSONAL_FACTS_TEST_MODE", mode)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{mode}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
#[test]
#[ignore = "real-process child launched by shared_births_survive_a_fresh_owner_process"]
fn personal_facts_child() {
    let root = std::path::PathBuf::from(
        std::env::var_os("PERSONAL_FACTS_TEST_ROOT").expect("parent-owned root"),
    );
    let mode = std::env::var("PERSONAL_FACTS_TEST_MODE").unwrap();
    let address = format!(
        "/tmp/pf-{}.sock",
        root.file_name().unwrap().to_string_lossy()
    );
    let identity = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(
            InMemoryStorage::new(),
            Profile::new(
                ProfileId("public-facts".into()),
                "Public synthetic owner",
                Ed25519Keypair::from_seed([0x38; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    ));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
  let attachment=ReservoirAttachment::open(ReservoirAttachmentOptions{endpoint:address.clone(),application:AppId::new("cleromancy"),applications:vec![AppId::new("cleromancy"),AppId::new("graphshell")],shared_root:root.join("shared"),persona:PersonaId::from_uuid(uuid::Uuid::from_u128(0x5046)),domain:pandect::DomainId::new("personal-facts").unwrap(),session_duration:Duration::from_secs(600)},||async{Ok(identity)}).await.unwrap();
  let mut first=PersonalFactsClient::open(&address,AppId::new("cleromancy")).await.unwrap();
  let mut second=PersonalFactsClient::open(&address,AppId::new("graphshell")).await.unwrap();
  if mode=="create"{
   let subject=first.create_contact("Public synthetic natal subject").await.unwrap();
   let facts:dramatis::birth::BirthFacts=serde_json::from_value(serde_json::json!({"schema":"dramatis.birth-facts/v1","subject":subject,"date":{"year":2000,"month":1,"day":1},"time":{"Known":{"hour":12,"minute":0,"second":0}},"place":null,"timezone":"UTC","source":{"label":"Public real-process fixture","reference":null}})).unwrap();
   let original=first.save_birth(&facts).await.unwrap();
   assert_eq!(second.catalog().await.unwrap().births,vec![original]);
   let mut corrected=facts;corrected.time=dramatis::birth::BirthTime::Known{hour:13,minute:0,second:0};second.save_birth(&corrected).await.unwrap();
   first.associate(&dramatis::resources::SubjectResourceAssociation::about(subject,dramatis::resources::ResourceRef{domain:"divination".into(),id:"cleromancy://natal/public-fixture".into(),revision:"fixture-v1".into()})).await.unwrap();
   let catalog=first.catalog().await.unwrap();assert_eq!(catalog.births.len(),2);assert_eq!(catalog.subjects.len(),1);assert_eq!(catalog.associations.len(),4);
   std::fs::write(root.join("expected.json"),serde_json::to_vec(&catalog).unwrap()).unwrap();
  }else{
   let expected:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("expected.json")).unwrap()).unwrap();
   assert_eq!(serde_json::to_value(first.catalog().await.unwrap()).unwrap(),expected);
   assert_eq!(serde_json::to_value(second.catalog().await.unwrap()).unwrap(),expected);
  }
  second.close().await.unwrap();first.close().await.unwrap();attachment.close().await.unwrap();
 });
}
