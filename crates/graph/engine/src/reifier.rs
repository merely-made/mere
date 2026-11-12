// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use oxrdf::NamedNode;

pub const LEGACY_PREFIX: &str = "urn:mere:statement:";
pub const ENCODED_PREFIX: &str = "urn:mere:statement-id:v1:";

pub fn statement_reifier_id(statement_id: &str) -> String {
    let legacy = format!("{LEGACY_PREFIX}{statement_id}");
    if NamedNode::new(legacy.clone()).is_ok() {
        return legacy;
    }
    let mut encoded = String::from(ENCODED_PREFIX);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in statement_id.bytes() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 15)]));
    }
    encoded
}

/// Distinguishes foreign reifiers from malformed handles in the reserved format.
pub fn statement_id_from_reifier(iri: &str) -> Result<Option<String>, &'static str> {
    if let Some(id) = iri.strip_prefix(LEGACY_PREFIX) {
        return Ok(Some(id.to_string()));
    }
    let Some(encoded) = iri.strip_prefix(ENCODED_PREFIX) else {
        return Ok(None);
    };
    if encoded.len() % 2 != 0 {
        return Err("encoded assertion id has an incomplete hexadecimal byte");
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = char::from(pair[0])
            .to_digit(16)
            .ok_or("encoded assertion id contains a non-hexadecimal byte")?;
        let low = char::from(pair[1])
            .to_digit(16)
            .ok_or("encoded assertion id contains a non-hexadecimal byte")?;
        bytes.push((high * 16 + low) as u8);
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "encoded assertion id is not UTF-8")
}
