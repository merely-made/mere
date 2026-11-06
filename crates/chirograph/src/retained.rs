// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A bounded, read-only request for the page text a mere has retained.
//!
//! The reply is data, not a projection: per page its identity, address, title,
//! current version (the BLAKE3 hex of its text), the text, and the other held
//! versions, plus the links the named session's graph draws between them. The
//! request names the graph session it is read for, and an endpoint answers
//! only for a session the asking application holds. Limits are explicit on
//! both sides: the endpoint clamps the request to [`RETAINED_PAGES_CEILING`]
//! and reports every omission in [`RetainedTruncation`], so a client never
//! mistakes a bounded reply for the whole.
//!
//! Nothing here writes. Keeping page text is a separate owner-checked intent on
//! the mere (see the resident), because the store belongs to the reservoir.

use super::*;

/// The request's limits. Zero in any field asks for the default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedPagesLimits {
    /// Pages to return.
    pub max_pages: u32,
    /// Bytes of page text to return across every page and version.
    pub max_bytes: u64,
    /// Held versions per page, the current text included.
    pub max_versions: u32,
    /// Links to return.
    pub max_links: u32,
}

/// What an endpoint will never exceed, whatever a client asks for. Sized so one
/// reply stays under the carrier's frame cap
/// ([`MAX_RESOURCE_CHUNK_BYTES`] explains the arithmetic).
pub const RETAINED_PAGES_CEILING: RetainedPagesLimits = RetainedPagesLimits {
    max_pages: 512,
    max_bytes: 256 * 1024,
    max_versions: 8,
    max_links: 4096,
};

/// What a request that leaves a field at zero receives.
pub const RETAINED_PAGES_DEFAULT: RetainedPagesLimits = RetainedPagesLimits {
    max_pages: 128,
    max_bytes: 128 * 1024,
    max_versions: 4,
    max_links: 1024,
};

impl RetainedPagesLimits {
    /// Defaults for zero fields, then the ceiling.
    pub fn effective(self) -> Self {
        let pick = |asked: u64, default: u64, ceiling: u64| {
            if asked == 0 { default } else { asked }.min(ceiling)
        };
        Self {
            max_pages: pick(
                self.max_pages.into(),
                RETAINED_PAGES_DEFAULT.max_pages.into(),
                RETAINED_PAGES_CEILING.max_pages.into(),
            ) as u32,
            max_bytes: pick(
                self.max_bytes,
                RETAINED_PAGES_DEFAULT.max_bytes,
                RETAINED_PAGES_CEILING.max_bytes,
            ),
            max_versions: pick(
                self.max_versions.into(),
                RETAINED_PAGES_DEFAULT.max_versions.into(),
                RETAINED_PAGES_CEILING.max_versions.into(),
            ) as u32,
            max_links: pick(
                self.max_links.into(),
                RETAINED_PAGES_DEFAULT.max_links.into(),
                RETAINED_PAGES_CEILING.max_links.into(),
            ) as u32,
        }
    }
}

/// Read the retained pages for `session`, the graph the client holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedPagesRequest {
    pub session: ProjectionSession,
    pub limits: RetainedPagesLimits,
}

/// Whether the mere is keeping page text now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetainedKeeping {
    /// The "Keep page text" setting is off (the default); nothing is written.
    Off,
    /// On, and writes land.
    On,
    /// On, but writes are paused because custody does not permit them; the
    /// reason is for the person to read. Nothing is written in plaintext.
    Paused { reason: String },
}

/// One held version of a page's text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedVersionWire {
    /// BLAKE3 hex of `text`.
    pub version: String,
    pub text: String,
}

/// One retained page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedPageWire {
    /// The page's resource identity (a UUID string): the canonical address's
    /// common resource, which graph nodes showing that address share.
    pub id: String,
    pub address: String,
    pub title: Option<String>,
    /// BLAKE3 hex of the current text; always among `versions`.
    pub current: String,
    /// Held versions, current first.
    pub versions: Vec<RetainedVersionWire>,
}

/// A link the graph draws from a retained page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedLinkWire {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// A link target nothing is retained for; named so the person can see it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedUnretainedWire {
    pub id: String,
    pub address: String,
    pub title: Option<String>,
}

/// What a reply left out because of its limits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedTruncation {
    pub pages_omitted: u64,
    pub versions_omitted: u64,
    pub links_omitted: u64,
}

impl RetainedTruncation {
    pub fn is_complete(&self) -> bool {
        self.pages_omitted == 0 && self.versions_omitted == 0 && self.links_omitted == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedPagesResponse {
    pub session: ProjectionSession,
    /// The limits the endpoint applied.
    pub applied: RetainedPagesLimits,
    pub keeping: RetainedKeeping,
    pub pages: Vec<RetainedPageWire>,
    pub links: Vec<RetainedLinkWire>,
    pub unretained: Vec<RetainedUnretainedWire>,
    pub truncated: RetainedTruncation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_means_default_and_nothing_exceeds_the_ceiling() {
        let defaults = RetainedPagesLimits::default().effective();
        assert_eq!(defaults, RETAINED_PAGES_DEFAULT);
        let greedy = RetainedPagesLimits {
            max_pages: u32::MAX,
            max_bytes: u64::MAX,
            max_versions: u32::MAX,
            max_links: u32::MAX,
        }
        .effective();
        assert_eq!(greedy, RETAINED_PAGES_CEILING);
        let small = RetainedPagesLimits {
            max_pages: 3,
            max_bytes: 10,
            max_versions: 1,
            max_links: 2,
        };
        assert_eq!(small.effective(), small);
    }

    #[test]
    fn a_request_and_a_reply_round_trip_through_the_carrier_envelope() {
        let request = CarrierRequest {
            id: 7,
            body: CarrierRequestBody::RetainedPages(RetainedPagesRequest {
                session: ProjectionSession("mere:graph".into()),
                limits: RetainedPagesLimits::default(),
            }),
        };
        let wire = serde_json::to_vec(&request).unwrap();
        assert_eq!(serde_json::from_slice::<CarrierRequest>(&wire).unwrap(), request);
        let reply = CarrierResponse {
            id: 7,
            body: Ok(CarrierResponseBody::RetainedPages(Box::new(
                RetainedPagesResponse {
                    session: ProjectionSession("mere:graph".into()),
                    applied: RETAINED_PAGES_DEFAULT,
                    keeping: RetainedKeeping::Paused {
                        reason: "locked".into(),
                    },
                    pages: vec![RetainedPageWire {
                        id: "a".into(),
                        address: "https://example.test/".into(),
                        title: Some("t".into()),
                        current: "v".into(),
                        versions: vec![RetainedVersionWire {
                            version: "v".into(),
                            text: "body".into(),
                        }],
                    }],
                    links: vec![],
                    unretained: vec![],
                    truncated: RetainedTruncation::default(),
                },
            ))),
        };
        let wire = serde_json::to_vec(&reply).unwrap();
        assert_eq!(serde_json::from_slice::<CarrierResponse>(&wire).unwrap(), reply);
    }

    #[test]
    fn a_full_reply_at_the_ceiling_stays_under_the_native_message_frame() {
        // Plain prose with newlines is the realistic worst case for JSON text.
        let text = "line of kept page text\n".repeat(12_000);
        let text = &text[..RETAINED_PAGES_CEILING.max_bytes as usize];
        let reply = CarrierResponse {
            id: 1,
            body: Ok(CarrierResponseBody::RetainedPages(Box::new(
                RetainedPagesResponse {
                    session: ProjectionSession("mere:graph".into()),
                    applied: RETAINED_PAGES_CEILING,
                    keeping: RetainedKeeping::On,
                    pages: vec![RetainedPageWire {
                        id: "a".into(),
                        address: "https://example.test/".into(),
                        title: None,
                        current: "v".into(),
                        versions: vec![RetainedVersionWire {
                            version: "v".into(),
                            text: text.to_owned(),
                        }],
                    }],
                    links: vec![],
                    unretained: vec![],
                    truncated: RetainedTruncation::default(),
                },
            ))),
        };
        assert!(serde_json::to_vec(&reply).unwrap().len() < 1024 * 1024);
    }
}
