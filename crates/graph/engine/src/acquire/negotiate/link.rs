// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! RFC 8288 `Link` header parsing and the discovery dereferencing needs. No
//! maintained crate fits (see the L6b receipt), so this is a small parser:
//! `<target>` then `; name=token|"quoted"` parameters, links comma-separated.
//! Malformed link values are skipped, not fatal.

use super::format_for_media_type;
use crate::formats::RdfFormat;

/// The JSON-LD relation that points at a context document.
pub const JSON_LD_CONTEXT_REL: &str = "http://www.w3.org/ns/json-ld#context";

/// One link value: the raw target and its parameters, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The target URI-Reference as written, unresolved.
    pub target: String,
    /// Parameter names (lowercased) and values (unquoted).
    pub params: Vec<(String, String)>,
}

impl Link {
    /// The first value of parameter `name`.
    #[must_use]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The relation types: the first `rel` parameter split on whitespace
    /// (RFC 8288 section 3.3 ignores later `rel` parameters).
    pub fn rels(&self) -> impl Iterator<Item = &str> {
        self.param("rel").unwrap_or("").split_ascii_whitespace()
    }

    /// Whether `rel` names `relation` (case-insensitive).
    #[must_use]
    pub fn has_rel(&self, relation: &str) -> bool {
        self.rels().any(|r| r.eq_ignore_ascii_case(relation))
    }

    /// The target resolved against `base`; `None` if either is not an IRI.
    #[must_use]
    pub fn resolve(&self, base: &str) -> Option<String> {
        oxiri::Iri::parse(base)
            .ok()?
            .resolve(&self.target)
            .ok()
            .map(oxiri::Iri::into_inner)
    }
}

/// Parse a `Link` field value (several fields may be joined with commas).
#[must_use]
pub fn parse_links(value: &str) -> Vec<Link> {
    let mut links = Vec::new();
    let mut rest = value;
    loop {
        rest = rest.trim_start_matches(|c: char| c == ',' || c.is_ascii_whitespace());
        if rest.is_empty() {
            return links;
        }
        match link_value(rest) {
            Some((link, after)) => {
                links.push(link);
                rest = after;
            },
            None => rest = skip_to_comma(rest),
        }
    }
}

/// Past the next comma outside quotes and angle brackets.
fn skip_to_comma(s: &str) -> &str {
    let (mut quoted, mut angled, mut escaped) = (false, false, false);
    for (i, c) in s.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' if !angled => quoted = !quoted,
            '<' if !quoted => angled = true,
            '>' if !quoted => angled = false,
            ',' if !quoted && !angled => return &s[i + 1..],
            _ => {},
        }
    }
    ""
}

/// One `<target> *( ; param )`; returns the link and the text after it.
fn link_value(s: &str) -> Option<(Link, &str)> {
    let s = s.strip_prefix('<')?;
    let end = s.find('>')?;
    let target = s[..end].trim().to_owned();
    let mut rest = &s[end + 1..];
    let mut params = Vec::new();
    loop {
        rest = rest.trim_start();
        let Some(after) = rest.strip_prefix(';') else {
            break;
        };
        rest = after.trim_start();
        let name_end = rest
            .find(|c: char| c == '=' || c == ';' || c == ',' || c.is_ascii_whitespace())
            .unwrap_or(rest.len());
        let name = rest[..name_end].to_ascii_lowercase();
        rest = rest[name_end..].trim_start();
        let mut value = String::new();
        if let Some(after) = rest.strip_prefix('=') {
            let after = after.trim_start();
            if let Some(quoted) = after.strip_prefix('"') {
                let mut chars = quoted.char_indices();
                let mut closed = None;
                while let Some((i, c)) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some((_, escaped)) = chars.next() {
                                value.push(escaped);
                            }
                        },
                        '"' => {
                            closed = Some(i + 1);
                            break;
                        },
                        _ => value.push(c),
                    }
                }
                rest = &quoted[closed?..];
            } else {
                let end = after
                    .find(|c: char| c == ';' || c == ',')
                    .unwrap_or(after.len());
                value = after[..end].trim_end().to_owned();
                rest = &after[end..];
            }
        }
        if !name.is_empty() {
            params.push((name, value));
        }
    }
    Some((Link { target, params }, rest))
}

/// An alternate representation naming an RDF type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternate {
    /// The target, resolved against the response URL.
    pub iri: String,
    pub format: RdfFormat,
}

/// What a response's `Link` header offers a dereferencer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Discovery {
    /// `rel=alternate` links whose `type` is an RDF format.
    pub alternates: Vec<Alternate>,
    /// `rel=describedby` targets, resolved.
    pub described_by: Vec<String>,
    /// JSON-LD context targets, resolved (for a plain `application/json` body).
    pub json_ld_contexts: Vec<String>,
}

/// Parse `header` and discover the relations dereferencing needs, resolving
/// each target against `response_url`. A link that does not resolve is skipped.
#[must_use]
pub fn discover(header: &str, response_url: &str) -> Discovery {
    let mut found = Discovery::default();
    for link in parse_links(header) {
        let Some(iri) = link.resolve(response_url) else {
            continue;
        };
        if link.has_rel("alternate") {
            let format = link.param("type").and_then(format_for_media_type);
            if let Some(format) = format {
                found.alternates.push(Alternate {
                    iri: iri.clone(),
                    format,
                });
            }
        }
        if link.has_rel("describedby") {
            found.described_by.push(iri.clone());
        }
        if link.has_rel(JSON_LD_CONTEXT_REL) {
            found.json_ld_contexts.push(iri);
        }
    }
    found
}
