use eidetic::import::encounters::{BrowserEncounter,BrowserEncounterLedger,BrowserEncounterRef,BrowserSourceScope,BrowserParent};
fn visit()->BrowserEncounter {
 BrowserEncounter{identity:BrowserEncounterRef{scope:BrowserSourceScope{product:"firefox".into(),profile:"synthetic".into(),generation:"g".into()},visit_id:"a".into()},parent:BrowserParent::Root,url:"https://queue-a.test/".into(),title:Some("Synthetic".into()),raw_transition:"link".into(),at_ms:1000,original_unix_micros:Some(1000001),source_is_local:Some(true),persona:"synthetic-persona".into(),acquisition_device:"synthetic-device".into(),intake_at_ms:2000}
}
#[test]
fn ledger_rejects_the_changed_microseconds_the_js_queue_discards(){
 let mut ledger=BrowserEncounterLedger::default();let prior=visit();assert!(ledger.receive(prior.clone(),10).unwrap());
 let mut later=prior;later.original_unix_micros=Some(1000002);
 assert_eq!(ledger.receive(later,10).unwrap_err(),"conflicting browser encounter source facts");
}
#[test]
fn ledger_rejects_the_changed_locality_the_js_queue_discards(){
 let mut ledger=BrowserEncounterLedger::default();let prior=visit();assert!(ledger.receive(prior.clone(),10).unwrap());
 let mut later=prior;later.source_is_local=Some(false);
 assert_eq!(ledger.receive(later,10).unwrap_err(),"conflicting browser encounter source facts");
}
#[test]
fn identical_delivery_control_is_idempotent(){
 let mut ledger=BrowserEncounterLedger::default();let prior=visit();assert!(ledger.receive(prior.clone(),10).unwrap());
 assert!(!ledger.receive(prior,10).unwrap());assert_eq!(ledger.pending.len(),1);
}
