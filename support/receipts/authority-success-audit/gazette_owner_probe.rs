use std::collections::BTreeMap;
use chirograph::{CoordinatedSelection,SelectionResolution};
use gazette::ledger::*;
use sceno::SourceRef;

fn citation() -> incipit::ShelfmarkV1 {
    project_ledger(&[Contact{id:"public-contact".into(),name:"Public fixture".into(),handle:"alice@example.org".into(),trust:"unverified".into(),freshness:"fixture".into()}],
      &[ContactFacet{id:"handle".into(),label:"Handle".into()}],CoordinatedSelection::new(SelectionResolution::Crossfilter)).unwrap()
      .shelfmark(LedgerCitationInputs{contacts_record:"fixture:contacts".into(),contacts_generation:"contacts-1".into(),facets_record:"fixture:facets".into(),facets_generation:"facets-1".into()},&[]).unwrap()
}
fn generations() -> BTreeMap<String,String> {BTreeMap::from([("rows".into(),"contacts-1".into()),("columns".into(),"facets-1".into())])}

#[test]
fn ledger_counts_unknown_instances_and_ignores_malformed_selection() {
    let mut mark=citation();
    let delta=LedgerInstanceDelta{instance:LedgerInstanceAddress{view:"unknown-view".into(),source:SourceRef::new("foreign.adapter/v1","absent-contact"),facet:"absent-facet".into()},visible:false};
    mark.delta.insert("gazette.instances".into(),serde_json::to_string(&[delta]).unwrap());
    mark.delta.insert("selection".into(),"AUDIT not JSON or a selection".into());
    let receipt=resolve_ledger_shelfmark(&mark,&generations()).unwrap();
    assert_eq!(receipt.honored_instance_deltas,1);
    let mut moved=generations();moved.insert("rows".into(),"contacts-2".into());
    assert!(resolve_ledger_shelfmark(&mark,&moved).unwrap_err().contains("authority moved"));
    mark.delta.insert("gazette.instances".into(),"AUDIT not JSON".into());
    assert!(resolve_ledger_shelfmark(&mark,&generations()).unwrap_err().contains("invalid Ledger instance state"));
    println!("AUDIT unknown instance counted honored=1 and malformed selection accepted; moved generation and malformed instance JSON refuse");
}

#[test]
fn resolver_and_intake_disagree_on_uppercase_account_scheme() {
    let resource="ACCT:Alice@EXAMPLE.org";
    let normalized=gazette::normalize_resource(resource).unwrap();
    assert_eq!(normalized,"acct:ACCT:Alice@EXAMPLE.org");
    let endpoint=gazette::endpoint_url(resource).unwrap();
    assert_eq!(endpoint.query_pairs().find(|(key,_)|key=="resource").unwrap().1,normalized);
    let intake=gazette::intake::WebFingerIntake::from_import(resource,gazette::WebFingerImport{subject:"acct:Alice@example.org".into(),..Default::default()}).unwrap();
    assert_eq!(intake.resource(),"acct:Alice@example.org");
    assert!(gazette::intake::WebFingerIntake::from_import(&normalized,intake.source().clone()).is_err());
    assert_eq!(gazette::normalize_resource("acct:Alice@example.org").unwrap(),intake.resource());
    println!("AUDIT uppercase scheme resolver query={normalized}; actual Gaz-backed intake account={}",intake.resource());
}

#[test]
fn classified_protocol_case_differs_and_generic_jsonld_is_actor() {
    let doc=gazette::parse_document(r#"{"subject":"acct:alice@example.org","links":[{"rel":"alternate","href":"GEMINI://example.org/capsule"},{"rel":"alternate","href":"gemini://example.org/capsule"},{"rel":"describedby","type":"application/ld+json","href":"https://example.org/plain-linked-data"}]}"#).unwrap();
    let import=gazette::WebFingerImport::from_document(&doc);
    assert_eq!(import.other_endpoints[0].href,"GEMINI://example.org/capsule");
    assert_eq!(import.gemini_capsules,vec!["gemini://example.org/capsule"]);
    assert_eq!(import.activitypub_actors,vec!["https://example.org/plain-linked-data"]);
    println!("AUDIT uppercase protocol uses catch-all; generic JSON-LD link classified as ActivityPub actor without ActivityStreams profile");
}
