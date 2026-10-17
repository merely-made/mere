#![allow(dead_code)]
#[path="../../../ports/moot/src/coop.rs"] mod coop;
#[path="community_portable_resource.rs"] mod resource_coop;
use resource_coop::*;
use std::collections::BTreeMap;
    fn read() -> Snapshot {
        Snapshot {
            moot: [1; 32],
            reader: [2; 32],
            collection: None,
            activities: vec![],
            resources: vec![ResourceRead {
                reference: ResourceRef {
                    moot: [1; 32],
                    contribution: [3; 32],
                    manifest: [4; 32],
                },
                title: "Notebook".into(),
                schema: "text/plain".into(),
                content: ContentRead::AddressOnly,
                retention: RetentionRead::Unknown,
                terms: BTreeMap::new(),
                hosting: HostingRead::Unknown,
                offers: [(ResourceAction::Open, Offer::Offered)].into(),
            }],
        }
    }

fn main() {}
