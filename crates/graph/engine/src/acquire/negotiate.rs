// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Content negotiation for dereferencing (D142): the `Accept` policy a host
//! sends, the `Content-Type` dispatch to the parsers, and `Link`-header
//! discovery. No network and no HTTP client; the host fetches.

/// RFC 8288 `Link` parsing and the discovery the engine needs.
pub mod link;

use crate::acquire::ingest::{ContextCache, IngestError};
use crate::dataset::IndexedDataset;
use crate::formats::{RdfFormat, parse};
use oxrdfio::JsonLdProfileSet;

/// The formats the engine parses, in the default preference order.
pub const PARSED_FORMATS: [RdfFormat; 6] = [
    RdfFormat::Turtle,
    RdfFormat::JsonLd {
        profile: JsonLdProfileSet::empty(),
    },
    RdfFormat::TriG,
    RdfFormat::NQuads,
    RdfFormat::NTriples,
    RdfFormat::RdfXml,
];

/// The engine's format for a media type, or `None`. `oxrdfio` is lenient
/// (`application/json`, `text/plain` and `application/xml` map to JSON-LD,
/// N-Triples and RDF/XML); a dereferencer must not guess, so those are refused,
/// as are formats the engine does not parse (N3). A bare `application/json`
/// body needs a `Link` context instead (see [`link::discover`]).
#[must_use]
pub fn format_for_media_type(media_type: &str) -> Option<RdfFormat> {
    let subtype = media_type
        .split(';')
        .next()?
        .split_once('/')?
        .1
        .trim()
        .to_ascii_lowercase();
    let subtype = subtype.strip_prefix("x-").unwrap_or(&subtype);
    if matches!(subtype, "json" | "plain" | "xml") {
        return None;
    }
    RdfFormat::from_media_type(media_type).filter(|format| !matches!(format, RdfFormat::N3))
}

/// One format and its preference, in thousandths (`1000` is `q=1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcceptEntry {
    pub format: RdfFormat,
    pub q_milli: u16,
}

/// The formats a host asks for, with weights. Data, so a host can narrow it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptPolicy {
    pub entries: Vec<AcceptEntry>,
}

impl Default for AcceptPolicy {
    /// Turtle and JSON-LD first, the quad and line formats below, legacy
    /// RDF/XML last.
    fn default() -> Self {
        const Q: [u16; 6] = [1000, 1000, 900, 900, 800, 500];
        AcceptPolicy {
            entries: PARSED_FORMATS
                .into_iter()
                .zip(Q)
                .map(|(format, q_milli)| AcceptEntry { format, q_milli })
                .collect(),
        }
    }
}

impl AcceptPolicy {
    /// This policy restricted to `formats`, weights kept.
    #[must_use]
    pub fn narrowed_to(&self, formats: &[RdfFormat]) -> AcceptPolicy {
        AcceptPolicy {
            entries: self
                .entries
                .iter()
                .copied()
                .filter(|entry| formats.contains(&entry.format))
                .collect(),
        }
    }

    /// The `Accept` header value. `q=1` is left implicit; entries keep order;
    /// weights above 1000 clamp to 1.
    #[must_use]
    pub fn header_value(&self) -> String {
        self.entries
            .iter()
            .map(|entry| {
                let q = entry.q_milli.min(1000);
                if q == 1000 {
                    entry.format.media_type().to_owned()
                } else {
                    let text = format!("{q:03}");
                    format!(
                        "{};q=0.{}",
                        entry.format.media_type(),
                        text.trim_end_matches('0')
                    )
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Why a response body could not be dispatched.
#[derive(Debug)]
pub enum NegotiateError {
    /// The media type (or its charset) is not one the engine parses; absent too.
    UnsupportedMediaType(String),
    /// The parser rejected the body.
    Parse(IngestError),
}

impl std::fmt::Display for NegotiateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NegotiateError::UnsupportedMediaType(t) => write!(f, "unsupported media type: {t:?}"),
            NegotiateError::Parse(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for NegotiateError {}

/// The format a `Content-Type` value names: media type case-insensitive,
/// `charset` (UTF-8 only) and JSON-LD `profile` parameters honored by `oxrdfio`.
///
/// # Errors
///
/// [`NegotiateError::UnsupportedMediaType`] for anything but the six formats
/// (see [`format_for_media_type`]), or a non-UTF-8 charset.
pub fn parse_content_type(value: &str) -> Result<RdfFormat, NegotiateError> {
    format_for_media_type(value)
        .ok_or_else(|| NegotiateError::UnsupportedMediaType(value.trim().to_owned()))
}

/// Parse a response body by its `Content-Type`: [`parse_content_type`] then
/// [`parse::parse`].
///
/// # Errors
///
/// [`NegotiateError`] for an unknown type or a parse failure.
pub fn parse_response(
    bytes: &[u8],
    content_type: &str,
    base_iri: Option<&str>,
    contexts: ContextCache,
) -> Result<IndexedDataset, NegotiateError> {
    parse::parse(parse_content_type(content_type)?, bytes, base_iri, contexts)
        .map_err(NegotiateError::Parse)
}

#[cfg(test)]
mod tests;
