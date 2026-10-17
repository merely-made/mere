#![allow(dead_code)]
#[path = "../../../ports/moot/src/coop.rs"]
mod coop;
#[path = "../../../ports/moot/src/resource_coop.rs"]
mod resource_coop;
use resource_coop::*;

fn snapshot(author: Option<[u8; 32]>) -> Snapshot {
    Snapshot {
        moot: [1; 32], reader: [2; 32], collection: None, activities: vec![],
        resources: vec![ResourceRead {
            reference: ResourceRef { moot: [1; 32], contribution: [3; 32], manifest: [4; 32] },
            shared_by: author, title: "Exact contribution".into(), schema: "text/plain".into(),
            content: ContentRead::AddressOnly, retention: RetentionRead::Unknown,
            terms: Default::default(), hosting: HostingRead::Unknown, offers: Default::default(),
        }],
    }
}

#[test]
fn hidden_sharing_root_allows_a_different_root_under_the_same_contribution() {
    let first = snapshot(Some([17; 32]));
    let mut pane = Pane::new(first.clone()).unwrap();
    pane.select_resource(first.resources[0].reference).unwrap();
    pane.refresh(snapshot(None)).unwrap();
    pane.refresh(snapshot(Some([18; 32]))).unwrap();
    assert_eq!(pane.selected_resource(), Some(first.resources[0].reference));
    assert_eq!(pane.snapshot().resources[0].shared_by, Some([18; 32]));
}

#[test]
fn direct_root_change_is_refused_and_preserves_the_first_snapshot() {
    let first = snapshot(Some([17; 32]));
    let mut pane = Pane::new(first.clone()).unwrap();
    assert_eq!(pane.refresh(snapshot(Some([18; 32]))), Err(Refusal::ChangedContribution));
    assert_eq!(pane.snapshot(), &first);
}

#[test]
fn first_disclosure_of_a_previously_unknown_root_is_valid() {
    let mut pane = Pane::new(snapshot(None)).unwrap();
    pane.refresh(snapshot(Some([17; 32]))).unwrap();
    assert_eq!(pane.snapshot().resources[0].shared_by, Some([17; 32]));
}
