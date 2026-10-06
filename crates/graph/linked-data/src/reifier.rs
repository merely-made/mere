// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use oxrdf::NamedNode;

const LEGACY_PREFIX: &str = "urn:mere:statement:";
const ENCODED_PREFIX: &str = "urn:mere:statement-id:v1:";

pub(crate) fn statement_reifier_id(statement_id: &str) -> String {
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
pub(crate) fn statement_id_from_reifier(iri: &str) -> Result<Option<String>, &'static str> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_legacy_ids_keep_their_iris_and_exact_handles() {
        for id in [
            "",
            "valid-handle",
            "猫",
            "statement-id:v1:6f70617175650a68616e646c65",
        ] {
            let iri = statement_reifier_id(id);
            assert_eq!(iri, format!("{LEGACY_PREFIX}{id}"));
            assert!(NamedNode::new(&iri).is_ok());
            assert_eq!(statement_id_from_reifier(&iri), Ok(Some(id.to_string())));
        }
    }

    #[test]
    fn unsafe_handles_have_distinct_reversible_valid_reifier_iris() {
        let handles = [
            " ",
            "  ",
            "opaque\nhandle",
            "猫\n犬",
            "\0",
            "urn:mere:statement-id:v1:20",
        ];
        let mut seen = std::collections::HashSet::new();
        for id in handles {
            let iri = statement_reifier_id(id);
            assert!(NamedNode::new(&iri).is_ok());
            assert!(
                seen.insert(iri.clone()),
                "different handles must not collide"
            );
            assert_eq!(statement_id_from_reifier(&iri), Ok(Some(id.to_string())));
            if id.contains([' ', '\n', '\0']) {
                assert!(iri.starts_with(ENCODED_PREFIX));
                assert!(!iri.starts_with(LEGACY_PREFIX));
            }
        }
        let unsafe_iri = statement_reifier_id("opaque\nhandle");
        for lookalike in [unsafe_iri.as_str(), "6f70617175650a68616e646c65"] {
            assert_ne!(statement_reifier_id(lookalike), unsafe_iri);
            assert_eq!(
                statement_id_from_reifier(&statement_reifier_id(lookalike)),
                Ok(Some(lookalike.to_string()))
            );
        }
    }

    #[test]
    fn decoder_distinguishes_foreign_and_malformed_reserved_inputs() {
        assert_eq!(
            statement_id_from_reifier("https://foreign.test/reifier"),
            Ok(None)
        );
        assert_eq!(
            statement_id_from_reifier("urn:mere:statement-id:v2:20"),
            Ok(None)
        );
        assert_eq!(
            statement_id_from_reifier("urn:mere:statement-id:v1:20"),
            Ok(Some(" ".to_string()))
        );
        for suffix in ["0", "gg", "ff"] {
            assert!(statement_id_from_reifier(&format!("{ENCODED_PREFIX}{suffix}")).is_err());
        }
    }
}
