// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Current Gemot reads and local Eidetic verification; action policy stays with the host.
use super::*;
use gemot::moot::{CollectionId, MootRoster};
use servitor::AuthorityProvider;

/// Only facts already disclosed by the content/action owner to this reader.
pub struct ResourceFacts {
    pub content: ContentRead,
    pub retention: RetentionRead,
    pub terms: BTreeMap<ResourceAction, ActionTerms>,
    pub hosting: HostingRead,
    pub offers: BTreeMap<ResourceAction, Offer>,
}
pub trait Disclosures {
    fn resource(&mut self, reader: [u8; 32], reference: ResourceRef) -> ResourceFacts;
    fn collection_fork(&mut self, reader: [u8; 32], collection: &CollectionRead) -> Option<Offer>;
    fn activities(&mut self, reader: [u8; 32], moot: [u8; 32]) -> Vec<ActivityRead>;
}

/// The caller supplies the roster and authority from the same actual Moot owner.
/// It must first permit this reader to view that Moot's metadata. Gemot's fauna
/// filter checks contribution authors' current write authority, not reader admission.
/// A missing selected collection refuses; it never widens to all fauna. Content,
/// hosting and operation offers come from their own owner rather than membership.
pub fn snapshot(
    roster: &MootRoster,
    authority: &impl AuthorityProvider,
    moot: [u8; 32],
    reader: [u8; 32],
    selected: Option<CollectionId>,
    disclosures: &mut impl Disclosures,
) -> Result<Snapshot, Refusal> {
    let resolved = selected
        .map(|id| {
            roster
                .authorized_collection(moot, id, authority)
                .ok_or(Refusal::NoCollection)
        })
        .transpose()?;
    let mut collection = resolved.as_ref().map(|view| CollectionRead {
        id: view.collection.collection_id.0,
        name: view.name.clone(),
        heads: view.version.frontier.clone(),
        membership_commitment: view.version.membership_commitment,
        fork: None,
    });
    if let Some(collection) = &mut collection {
        collection.fork = disclosures.collection_fork(reader, collection);
    }
    let resources = roster
        .authorized_fauna(authority)
        .into_iter()
        .filter(|share| {
            resolved.as_ref().is_none_or(|view| {
                view.effective_selected
                    .iter()
                    .any(|member| member.moot_id == moot && member.share == share.op_hash)
            })
        })
        .map(|share| {
            let reference = ResourceRef {
                moot,
                contribution: share.op_hash,
                manifest: share.manifest_id,
            };
            let facts = disclosures.resource(reader, reference);
            ResourceRead {
                reference,
                title: share.title.clone(),
                schema: share.schema_id.clone(),
                content: facts.content,
                retention: facts.retention,
                terms: facts.terms,
                hosting: facts.hosting,
                offers: facts.offers,
            }
        })
        .collect();
    let read = Snapshot {
        moot,
        reader,
        collection,
        resources,
        activities: disclosures.activities(reader, moot),
    };
    read.check()?;
    Ok(read)
}

/// Call only after the owner permits disclosure. Reads existing local/embedded
/// bytes through Eidetic's verifier. Never fetches remote sources or promises
/// durable custody. The byte budget is chosen by the caller; backend `get` still
/// materializes a stored value before the actual-size check.
pub async fn local_content(
    store: &mut dyn eidetic::Store,
    manifest: [u8; 32],
    max_bytes: u64,
) -> ContentRead {
    use eidetic::{BlobSource, Hash, ManifestId, NoFetcher};
    let hex: String = manifest.iter().map(|byte| format!("{byte:02x}")).collect();
    let id = ManifestId::from_hash(Hash::parse(&format!("blake3:{hex}")).expect("32 byte digest"));
    let result = async {
        let Some(mut record) = eidetic::manifest::load_manifest(store, id)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Ok(ContentRead::AddressOnly);
        };
        if record.id != id || record.manifest_version != eidetic::BlobManifest::CURRENT_VERSION {
            return Err(
                "Stored manifest identity/version does not match the selected content".into(),
            );
        }
        if record.byte_size > max_bytes {
            return Err("Content exceeds the owner's read budget".into());
        }
        record.sources.retain(|source| {
            matches!(
                source,
                BlobSource::Local { .. } | BlobSource::Embedded { .. }
            )
        });
        if record.sources.is_empty() {
            return Ok(ContentRead::AddressOnly);
        }
        if record.content_hash != id.0 {
            return Err("Local content hash differs from the selected manifest identity".into());
        }
        let bytes = eidetic::manifest::resolve_blob(store, &mut NoFetcher, &record)
            .await
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > max_bytes || bytes.len() as u64 != record.byte_size {
            return Err(
                "Resolved content size differs from its manifest or exceeds the read budget".into(),
            );
        }
        Ok(ContentRead::Resolved)
    }
    .await;
    result.unwrap_or_else(|reason| ContentRead::Unavailable { reason })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
    use gemot::moot::{FaunaEntry, MootAuthority, MootDelegations};
    struct Facts;

    fn manifest(bytes: &[u8]) -> eidetic::BlobManifest {
        use eidetic::*;
        BlobManifest {
            id: ManifestId::of_blob(bytes),
            schema: SchemaRef::from_id(ManifestId::of_blob(b"text/plain")),
            content_hash: Hash::of(bytes),
            byte_size: bytes.len() as u64,
            created_at: Timestamp(1),
            last_accessed: None,
            sources: vec![BlobSource::Embedded {
                bytes: bytes.to_vec(),
                media_type: "text/plain".into(),
            }],
            privacy: PrivacyClass::LocalOnly,
            provenance: ProvenanceRecord::self_imported("fixture"),
            trust: TrustEnvelope::self_asserted(),
            schema_metadata: serde_json::Value::Null,
            manifest_version: BlobManifest::CURRENT_VERSION,
        }
    }
    #[tokio::test]
    async fn local_reads_verify_content_budget_and_identity_without_writes_or_remote_fetch() {
        use eidetic::Store;
        let mut store = eidetic::MemoryBackend::default();
        let mut record = manifest(b"Notes");
        eidetic::manifest::save_manifest(&mut store, &record)
            .await
            .unwrap();
        let id = *record.id.0.as_bytes();
        let before = store.list("").await.unwrap();
        assert_eq!(
            local_content(&mut store, id, 20).await,
            ContentRead::Resolved
        );
        assert_eq!(store.list("").await.unwrap(), before);
        assert!(matches!(
            local_content(&mut store, id, 4).await,
            ContentRead::Unavailable { .. }
        ));
        record.sources = vec![eidetic::BlobSource::Embedded {
            bytes: b"wrong".to_vec(),
            media_type: "text/plain".into(),
        }];
        eidetic::manifest::save_manifest(&mut store, &record)
            .await
            .unwrap();
        assert!(matches!(
            local_content(&mut store, id, 20).await,
            ContentRead::Unavailable { .. }
        ));
        // Changing both bytes and their integrity field must not reinterpret
        // the exact selected local manifest as a different payload.
        record.content_hash = eidetic::Hash::of(b"wrong");
        eidetic::manifest::save_manifest(&mut store, &record)
            .await
            .unwrap();
        assert!(matches!(
            local_content(&mut store, id, 20).await,
            ContentRead::Unavailable { .. }
        ));
        record.sources = vec![eidetic::BlobSource::Https {
            url: "https://must-not-fetch.invalid/".into(),
        }];
        eidetic::manifest::save_manifest(&mut store, &record)
            .await
            .unwrap();
        assert_eq!(
            local_content(&mut store, id, 20).await,
            ContentRead::AddressOnly
        );
        record.id = eidetic::ManifestId::of_blob(b"different identity");
        let expected = manifest(b"Notes").id;
        store
            .put(&expected.store_key(), &serde_json::to_vec(&record).unwrap())
            .await
            .unwrap();
        assert!(matches!(
            local_content(&mut store, id, 20).await,
            ContentRead::Unavailable { .. }
        ));
    }

    #[test]
    fn collection_selection_uses_effective_members_and_exact_owner_frontier() {
        use gemot::moot::{
            CollectionChange, CollectionEvent, CollectionFact, CollectionRef, ContributionRef,
        };
        let moot = [1; 32];
        let id = CollectionId([8; 32]);
        let author = [2; 32];
        let mut roster = MootRoster::fold(moot, std::iter::empty());
        roster.fauna = (3..=4)
            .map(|n| FaunaEntry {
                manifest_id: [9; 32],
                schema_id: "text/plain".into(),
                title: format!("Note {n}"),
                shared_by: author,
                at_ms: 0,
                op_hash: [n; 32],
            })
            .collect();
        roster.collections = vec![
            CollectionFact {
                by: author,
                op_hash: [5; 32],
                event: CollectionEvent::Declared {
                    collection_id: id,
                    name: "Selected notes".into(),
                    fork: None,
                    at_ms: 0,
                },
            },
            CollectionFact {
                by: author,
                op_hash: [6; 32],
                event: CollectionEvent::Changed {
                    collection: CollectionRef {
                        moot_id: moot,
                        collection_id: id,
                    },
                    parents: vec![[5; 32]],
                    at_ms: 1,
                    change: CollectionChange::SetMembership {
                        contribution: ContributionRef {
                            moot_id: moot,
                            share: [3; 32],
                        },
                        included: true,
                    },
                },
            },
        ];
        let mut rules = ConstitutionRules::founder_only(author);
        for (tag, path) in [
            (1, "scope/moot/fauna".into()),
            (
                2,
                format!(
                    "scope/moot/collection/{}",
                    id.0.iter().map(|b| format!("{b:02x}")).collect::<String>()
                ),
            ),
        ] {
            rules.grant(CapabilityGrant {
                id: [tag; 32],
                subject: author,
                path_prefix: path,
                not_before_ms: 0,
                expires_at_ms: None,
                delegation_depth: 0,
            });
        }
        let delegations = MootDelegations::new();
        let authority = MootAuthority {
            delegations: &delegations,
            rules: &rules,
            moot_id: moot,
            now_ms: 5,
        };
        let read = snapshot(&roster, &authority, moot, [7; 32], Some(id), &mut Facts).unwrap();
        assert_eq!(read.resources.len(), 1);
        assert_eq!(read.resources[0].reference.contribution, [3; 32]);
        let collection = read.collection.unwrap();
        assert_eq!(collection.heads, vec![[6; 32]]);
        assert_eq!(
            collection.membership_commitment,
            roster
                .authorized_collection(moot, id, &authority)
                .unwrap()
                .version
                .membership_commitment
        );
        assert!(collection.fork.is_none());
    }
    impl Disclosures for Facts {
        fn resource(&mut self, _: [u8; 32], _: ResourceRef) -> ResourceFacts {
            ResourceFacts {
                content: ContentRead::AddressOnly,
                retention: RetentionRead::Unknown,
                terms: BTreeMap::new(),
                hosting: HostingRead::Unknown,
                offers: BTreeMap::new(),
            }
        }
        fn collection_fork(&mut self, _: [u8; 32], _: &CollectionRead) -> Option<Offer> {
            None
        }
        fn activities(&mut self, _: [u8; 32], _: [u8; 32]) -> Vec<ActivityRead> {
            vec![]
        }
    }
    #[test]
    fn real_gemot_authority_withholds_expired_and_foreign_contributions() {
        let roster = MootRoster::fold([1; 32], std::iter::empty());
        let mut roster = roster;
        roster.fauna = vec![
            FaunaEntry {
                manifest_id: [3; 32],
                schema_id: "text/plain".into(),
                title: "Visible".into(),
                shared_by: [2; 32],
                at_ms: 0,
                op_hash: [4; 32],
            },
            FaunaEntry {
                manifest_id: [3; 32],
                schema_id: "text/plain".into(),
                title: "Withheld".into(),
                shared_by: [9; 32],
                at_ms: 0,
                op_hash: [5; 32],
            },
        ];
        let mut rules = ConstitutionRules::founder_only([2; 32]);
        rules.grant(CapabilityGrant {
            id: [6; 32],
            subject: [2; 32],
            path_prefix: "scope/moot/fauna".into(),
            not_before_ms: 0,
            expires_at_ms: Some(10),
            delegation_depth: 0,
        });
        let delegations = MootDelegations::new();
        let authority = MootAuthority {
            delegations: &delegations,
            rules: &rules,
            moot_id: [1; 32],
            now_ms: 5,
        };
        let read = snapshot(&roster, &authority, [1; 32], [7; 32], None, &mut Facts).unwrap();
        assert_eq!(read.resources.len(), 1);
        assert_eq!(read.resources[0].reference.contribution, [4; 32]);
        assert!(read.resources[0].offers.is_empty());
        assert_eq!(
            snapshot(
                &roster,
                &authority,
                [1; 32],
                [7; 32],
                Some(CollectionId([8; 32])),
                &mut Facts
            ),
            Err(Refusal::NoCollection)
        );
        let expired = MootAuthority {
            now_ms: 11,
            ..authority
        };
        assert!(
            snapshot(&roster, &expired, [1; 32], [7; 32], None, &mut Facts)
                .unwrap()
                .resources
                .is_empty()
        );
    }
}
