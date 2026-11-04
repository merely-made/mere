// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Deserializer, Serializer, de::Error};

pub fn serialize<S: Serializer>(digest: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error> {
    use std::fmt::Write;
    let mut hex = String::with_capacity(64);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    serializer.serialize_str(&hex)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
    let hex = String::deserialize(deserializer)?;
    let bytes = hex.as_bytes();
    if bytes.len() != 64 {
        return Err(D::Error::invalid_length(bytes.len(), &"64 hex characters"));
    }
    let nibble = |b: u8| -> Result<u8, D::Error> {
        (b as char)
            .to_digit(16)
            .map(|d| d as u8)
            .ok_or_else(|| D::Error::custom("digest is not valid hex"))
    };
    let mut out = [0u8; 32];
    for (i, pair) in bytes.chunks_exact(2).enumerate() {
        out[i] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(out)
}
