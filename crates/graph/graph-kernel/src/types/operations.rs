// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl AssertionBasis {
    /// L13 reason identity: method IRI without version plus sorted evidence
    /// references. A plain explicit claim has the empty key.
    pub fn same_reason_as(&self, other: &Self) -> bool {
        if self.method.as_ref().map(|method| method.iri.as_str())
            != other.method.as_ref().map(|method| method.iri.as_str())
        {
            return false;
        }
        let mut left = self.evidence.iter().map(String::as_str).collect::<Vec<_>>();
        let mut right = other
            .evidence
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        left.sort_unstable();
        right.sort_unstable();
        left.dedup();
        right.dedup();
        left == right
    }

    pub fn is_empty(&self) -> bool {
        self.note.is_none() && self.method.is_none() && self.evidence.is_empty()
    }
}

impl ClassificationStatus {
    /// Classifications used by ordinary selection, display and affirmative export.
    pub fn is_affirmative(&self) -> bool {
        matches!(self, Self::Accepted | Self::Verified | Self::Imported)
    }
}

impl ImageRef {
    pub fn new(digest: [u8; 32], width: u32, height: u32) -> Self {
        Self {
            digest,
            width,
            height,
        }
    }

    /// Lowercase hex of the digest — the `content/image/<hex>` blob-key suffix.
    /// Matches `eidetic::Hash::to_hex` so a ref built from a saved blob's hash
    /// reads that same blob back.
    pub fn hex(&self) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(64);
        for byte in &self.digest {
            let _ = write!(out, "{byte:02x}");
        }
        out
    }
}
