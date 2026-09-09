// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Derived search view over a Moot's authorized captured pages.
//!
//! Gemot's signed fauna stays authoritative. This module filters that fauna
//! through current authority, validates locally resolved Fleece records, then
//! groups exact canonical text into one search result. Every share and capture
//! remains attached to the group as provenance.

use std::collections::{BTreeMap, BTreeSet};

use eidetic_search::{DocumentIndex, DocumentIndexConfig, SearchDocument};
use gemot::moot::{FaunaEntry, MootRoster};
use mere_document_lanes::eidetic_bridge::{
    CaptureIdentity, FLEECE_ANNOTATION_SCHEMA_ID, FleeceAnnotationRecord,
};
use servitor::AuthorityProvider;

/// Exact Fleece identity used for display and search grouping.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapturedPageId {
    pub extraction_schema: String,
    pub normalization: String,
    pub reader_profile: String,
    pub canonical_text_hash: String,
}

/// One signed act of sharing and the acquisition identity it carried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedContribution {
    pub share: FaunaEntry,
    pub capture: CaptureIdentity,
}

/// One exact canonical text with all contributing acts retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedPage {
    pub id: CapturedPageId,
    pub canonical_text_iri: String,
    pub canonical_text: String,
    pub contributions: Vec<CapturedContribution>,
}

/// Why an authorized fauna reference did not enter the searchable view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RejectedCaptureReason {
    UnsupportedSchema(String),
    MissingRecord,
    InvalidRecord(String),
}

/// An authorized reference retained for inspection despite rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedCapture {
    pub share: FaunaEntry,
    pub reason: RejectedCaptureReason,
}

/// A remintable collection view. It owns neither fauna nor capture custody.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapturedCollectionProjection {
    pub pages: Vec<CapturedPage>,
    pub rejected: Vec<RejectedCapture>,
}

impl CapturedCollectionProjection {
    /// Build from current authorized fauna and caller-resolved records.
    ///
    /// `records` is local custody keyed by the manifest id named by `Shared`.
    /// Missing, unsupported, and invalid entries remain explicit.
    pub fn from_roster(
        roster: &MootRoster,
        authority: &impl AuthorityProvider,
        records: &BTreeMap<[u8; 32], FleeceAnnotationRecord>,
    ) -> Self {
        let mut groups: BTreeMap<CapturedPageId, CapturedPage> = BTreeMap::new();
        let mut rejected = Vec::new();

        for share in roster.authorized_fauna(authority) {
            if share.schema_id != FLEECE_ANNOTATION_SCHEMA_ID {
                rejected.push(RejectedCapture {
                    share: share.clone(),
                    reason: RejectedCaptureReason::UnsupportedSchema(share.schema_id.clone()),
                });
                continue;
            }
            let Some(record) = records.get(&share.manifest_id) else {
                rejected.push(RejectedCapture {
                    share: share.clone(),
                    reason: RejectedCaptureReason::MissingRecord,
                });
                continue;
            };
            if let Err(error) = record.validate_integrity() {
                rejected.push(RejectedCapture {
                    share: share.clone(),
                    reason: RejectedCaptureReason::InvalidRecord(error.to_string()),
                });
                continue;
            }

            let preserved = &record.extraction.canonical_text_record;
            let id = CapturedPageId {
                extraction_schema: preserved.schema_id.clone(),
                normalization: preserved.normalization.clone(),
                reader_profile: preserved.reader_profile.clone(),
                canonical_text_hash: record.extraction.canonical_text_hash.to_hex(),
            };
            let contribution = CapturedContribution {
                share: share.clone(),
                capture: record.extraction.capture.clone(),
            };
            groups
                .entry(id.clone())
                .and_modify(|page| page.contributions.push(contribution.clone()))
                .or_insert_with(|| CapturedPage {
                    id,
                    canonical_text_iri: preserved.canonical_text_iri.clone(),
                    canonical_text: preserved.canonical_text.clone(),
                    contributions: vec![contribution],
                });
        }

        let mut pages: Vec<_> = groups.into_values().collect();
        for page in &mut pages {
            page.contributions
                .sort_by_key(|item| (item.share.at_ms, item.share.op_hash));
        }
        rejected.sort_by_key(|item| (item.share.at_ms, item.share.op_hash));
        Self { pages, rejected }
    }

    /// Mint one transient BM25 result per exact-content group.
    pub fn search_index(&self, config: DocumentIndexConfig) -> DocumentIndex<CapturedPageId> {
        DocumentIndex::from_documents(
            config,
            self.pages.iter().map(|page| {
                let latest = page
                    .contributions
                    .last()
                    .expect("a captured page always has a contribution");
                let primary_address = latest.capture.canonical_source.clone();
                let mut aliases: BTreeSet<_> = page
                    .contributions
                    .iter()
                    .map(|item| item.capture.canonical_source.clone())
                    .collect();
                aliases.remove(&primary_address);
                SearchDocument {
                    key: page.id.clone(),
                    primary_address,
                    aliases: aliases.into_iter().collect(),
                    title: Some(latest.share.title.clone()),
                    body: Some(page.canonical_text.clone()),
                }
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genet_static_dom::StaticDocument;
    use servitor::{Cap, Mode, Subject};

    struct Allowed(BTreeSet<[u8; 32]>);

    impl AuthorityProvider for Allowed {
        fn covers(&self, subject: Subject, _: &Cap, mode: Mode) -> bool {
            mode == Mode::Write && self.0.contains(&subject.0)
        }
    }

    fn record(source: &str, html: &str, raw_tag: u8) -> FleeceAnnotationRecord {
        let document = fleece::extract_document(&StaticDocument::parse(html));
        let exact = "shared passage";
        let byte_start = document.page.text.find(exact).expect("fixture passage");
        let start = document.page.text[..byte_start].chars().count() as u64;
        let end = start + exact.chars().count() as u64;
        let anchor = fleece::anchor_for_range(
            &document.page.text,
            fleece::TextPositionSelector { start, end },
            document.contract.quote_context,
        )
        .expect("fixture anchor");
        FleeceAnnotationRecord::from_fleece(
            CaptureIdentity::new(source, eidetic::Hash::of(&[raw_tag])).unwrap(),
            &document,
            &anchor,
        )
        .unwrap()
    }

    fn share(manifest: u8, author: u8, title: &str, at_ms: u64) -> FaunaEntry {
        FaunaEntry {
            manifest_id: [manifest; 32],
            schema_id: FLEECE_ANNOTATION_SCHEMA_ID.to_string(),
            title: title.to_string(),
            shared_by: [author; 32],
            at_ms,
            op_hash: [manifest.wrapping_add(author); 32],
        }
    }

    #[test]
    fn authorized_duplicates_group_and_keep_each_contribution() {
        let same = "<main><p>A shared passage carries the mycelial index term.</p></main>";
        let changed = "<main><p>A shared passage carries the basaltic revision term.</p></main>";
        let first = share(1, 1, "origin", 10);
        let second = share(2, 2, "mirror", 20);
        let revision = share(3, 1, "revision", 30);
        let unauthorized = share(4, 4, "decoy", 40);
        let mut unsupported = share(5, 1, "other", 50);
        unsupported.schema_id = "example.other/v1".to_string();
        let missing = share(6, 2, "missing", 60);
        let invalid = share(7, 1, "invalid", 70);

        let mut roster = MootRoster::default();
        roster.fauna = vec![
            unauthorized,
            invalid.clone(),
            missing.clone(),
            second.clone(),
            unsupported.clone(),
            revision.clone(),
            first.clone(),
        ];
        let authority = Allowed([[1; 32], [2; 32]].into_iter().collect());
        let mut invalid_record = record("https://broken.test/page", same, 7);
        invalid_record
            .extraction
            .canonical_text_record
            .canonical_text
            .push_str(" tampered");
        let records = [
            ([1; 32], record("https://one.test/page", same, 1)),
            ([2; 32], record("https://two.test/copy", same, 2)),
            ([3; 32], record("https://one.test/page", changed, 3)),
            ([4; 32], record("https://decoy.test/page", same, 4)),
            ([7; 32], invalid_record),
        ]
        .into_iter()
        .collect();

        let projection = CapturedCollectionProjection::from_roster(&roster, &authority, &records);
        assert_eq!(projection.pages.len(), 2);
        let duplicate = projection
            .pages
            .iter()
            .find(|page| page.canonical_text.contains("mycelial"))
            .unwrap();
        assert_eq!(duplicate.contributions.len(), 2);
        assert_eq!(
            duplicate
                .contributions
                .iter()
                .map(|item| item.share.shared_by)
                .collect::<Vec<_>>(),
            [[1; 32], [2; 32]]
        );
        assert_eq!(
            duplicate
                .contributions
                .iter()
                .map(|item| item.capture.canonical_source.as_str())
                .collect::<Vec<_>>(),
            ["https://one.test/page", "https://two.test/copy"]
        );
        assert_eq!(projection.rejected.len(), 3);
        assert!(projection.rejected.iter().any(|item| {
            item.share == missing && item.reason == RejectedCaptureReason::MissingRecord
        }));
        assert!(projection.rejected.iter().any(|item| {
            item.share == unsupported
                && matches!(item.reason, RejectedCaptureReason::UnsupportedSchema(_))
        }));
        assert!(projection.rejected.iter().any(|item| {
            item.share == invalid && matches!(item.reason, RejectedCaptureReason::InvalidRecord(_))
        }));

        let index = projection.search_index(DocumentIndexConfig::default());
        assert_eq!(index.len(), 2);
        assert_eq!(index.search("mycelial", 5).len(), 1);
        assert_eq!(index.search("basaltic", 5).len(), 1);
        assert!(index.search("decoy", 5).is_empty());
        let alias = index.search("https://one.test/page", 5);
        assert_eq!(
            alias.len(),
            2,
            "both exact revisions keep the source address"
        );
    }
}
