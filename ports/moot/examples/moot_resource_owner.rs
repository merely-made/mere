// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Local owner fixture for rendered acceptance. Uses actual Gemot/Eidetic reads;
//! synthetic unsealed identities and an in-memory store, not live peer delivery.
use eidetic::*;
use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
use gemot::moot::{
    CollectionChange, CollectionEvent, CollectionFact, CollectionId, CollectionRef,
    ContributionRef, FaunaEntry, MootAuthority, MootDelegations, MootRoster,
};
use moot::coop::{LifecycleReport, Verdict};
use moot::resource_coop::owners::{self, Disclosures, ResourceFacts};
use moot::resource_coop::*;
use std::io::Read;
const MOOT: [u8; 32] = [1; 32];
const READER: [u8; 32] = [9; 32];
const AUTHOR: [u8; 32] = [2; 32];
const COLLECTION: CollectionId = CollectionId([8; 32]);
const BODY:&[u8]=b"Shared protocol notes\n\nThis exact local reading is verified by Eidetic.\nOpening it does not retain a copy, offer hosting, or join an activity.\n";
struct Facts {
    content: ContentRead,
}
impl Disclosures for Facts {
    fn resource(&mut self, _: [u8; 32], _: ResourceRef) -> ResourceFacts {
        ResourceFacts {
            content: self.content.clone(),
            hosting: HostingRead::Promise {
                owner_record: "fixture-host-promise; no retrieval receipt".into(),
            },
            offers: [
                (ResourceAction::Open, Offer::Offered),
                (
                    ResourceAction::RetainCopy,
                    Offer::Withheld {
                        reason: "Owner storage quota is exhausted".into(),
                    },
                ),
                (
                    ResourceAction::ForkContent,
                    Offer::Unsupported {
                        reason: "Content fork writes are not installed".into(),
                    },
                ),
            ]
            .into(),
        }
    }
    fn collection_fork(&mut self, _: [u8; 32], _: &CollectionRead) -> Option<Offer> {
        Some(Offer::Unsupported {
            reason: "Collection writes are not installed".into(),
        })
    }
    fn activities(&mut self, _: [u8; 32], moot: [u8; 32]) -> Vec<ActivityRead> {
        vec![ActivityRead {
            reference: ActivityRef {
                moot,
                owner: "fixture/protocol-review".into(),
                activity: "session/protocol-notes".into(),
            },
            title: "Protocol review".into(),
            lifecycle: LifecycleReport::new(
                Verdict::NotJoined,
                Some("This reader has not joined".into()),
                10,
            ),
            resources: vec![],
            offers: [(
                ActivityAction::Join,
                Offer::Unsupported {
                    reason: "This owner has no participation adapter".into(),
                },
            )]
            .into(),
        }]
    }
}
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut store = MemoryBackend::default();
    let manifest = BlobManifest {
        id: ManifestId::of_blob(BODY),
        schema: SchemaRef::from_id(ManifestId::of_blob(b"text/plain")),
        content_hash: Hash::of(BODY),
        byte_size: BODY.len() as u64,
        created_at: Timestamp(10),
        last_accessed: None,
        sources: vec![BlobSource::Embedded {
            bytes: BODY.to_vec(),
            media_type: "text/plain".into(),
        }],
        privacy: PrivacyClass::LocalOnly,
        provenance: ProvenanceRecord::self_imported("local owner fixture"),
        trust: TrustEnvelope::self_asserted(),
        schema_metadata: serde_json::Value::Null,
        manifest_version: BlobManifest::CURRENT_VERSION,
    };
    eidetic::manifest::save_manifest(&mut store, &manifest).await?;
    let manifest_id = *manifest.id.0.as_bytes();
    let mut roster = MootRoster::fold(MOOT, std::iter::empty());
    roster.fauna = (3..=5)
        .map(|tag| FaunaEntry {
            manifest_id,
            schema_id: "text/plain".into(),
            title: format!("Protocol notes contribution {tag}"),
            shared_by: if tag == 5 { [99; 32] } else { AUTHOR },
            at_ms: 10,
            op_hash: [tag; 32],
        })
        .collect();
    roster.collections.push(CollectionFact {
        by: AUTHOR,
        op_hash: [6; 32],
        event: CollectionEvent::Declared {
            collection_id: COLLECTION,
            name: "Protocol notes".into(),
            fork: None,
            at_ms: 10,
        },
    });
    for (tag, parent) in [(3, 6), (4, 3)] {
        roster.collections.push(CollectionFact {
            by: AUTHOR,
            op_hash: [tag + 10; 32],
            event: CollectionEvent::Changed {
                collection: CollectionRef {
                    moot_id: MOOT,
                    collection_id: COLLECTION,
                },
                parents: vec![[if parent == 6 { 6 } else { 13 }; 32]],
                change: CollectionChange::SetMembership {
                    contribution: ContributionRef {
                        moot_id: MOOT,
                        share: [tag; 32],
                    },
                    included: true,
                },
                at_ms: 11,
            },
        });
    }
    let mut rules = ConstitutionRules::founder_only(AUTHOR);
    for (id, path) in [
        (1, "scope/moot/fauna".into()),
        (
            2,
            format!("scope/moot/collection/{}", hex::encode(COLLECTION.0)),
        ),
    ] {
        rules.grant(CapabilityGrant {
            id: [id; 32],
            subject: AUTHOR,
            path_prefix: path,
            not_before_ms: 0,
            expires_at_ms: Some(100),
            delegation_depth: 0,
        });
    }
    let delegations = MootDelegations::new();
    let authority = MootAuthority {
        delegations: &delegations,
        rules: &rules,
        moot_id: MOOT,
        now_ms: 10,
    };
    let mut facts = Facts {
        content: owners::local_content(&mut store, manifest_id, 4096).await,
    };
    let current = owners::snapshot(
        &roster,
        &authority,
        MOOT,
        READER,
        Some(COLLECTION),
        &mut facts,
    )?;
    let mode = std::env::args().nth(1).unwrap_or_else(|| "read".into());
    if mode == "read" {
        println!(
            "{}",
            serde_json::json!({"snapshots":[{"reading":1,"snapshot":current},{"reading":2,"snapshot":current}]})
        );
    } else if mode == "dispatch" {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        let command: serde_json::Value = serde_json::from_str(&input)?;
        let reader: Option<[u8; 32]> = serde_json::from_value(command["reader"].clone()).ok();
        let reference: Option<ResourceRef> =
            serde_json::from_value(command["target"]["resource"].clone()).ok();
        let action: Option<ResourceAction> =
            serde_json::from_value(command["target"]["action"].clone()).ok();
        let allowed = reader == Some(READER)
            && action == Some(ResourceAction::Open)
            && reference.is_some_and(|r| {
                current.resources.iter().any(|item| {
                    item.reference == r
                        && item.offers.get(&ResourceAction::Open) == Some(&Offer::Offered)
                })
            });
        if allowed
            && owners::local_content(&mut store, manifest_id, 4096).await == ContentRead::Resolved
        {
            let bytes =
                eidetic::manifest::resolve_blob(&mut store, &mut NoFetcher, &manifest).await?;
            println!(
                "{}",
                serde_json::json!({"outcome":Outcome::Accepted{owner_receipt:format!("Verified local read {}",manifest.id)},"text":String::from_utf8(bytes)?})
            );
        } else {
            println!(
                "{}",
                serde_json::json!({"outcome":Outcome::Refused{reason:"Exact owner policy or current content verification refused the request".into()}})
            );
        }
    } else {
        return Err("Expected read or dispatch".into());
    }
    Ok(())
}
