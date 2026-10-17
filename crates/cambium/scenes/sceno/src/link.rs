// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Source link identity carried through scene routing. Readings and source
//! authority remain in scenograph; these values never mint a pair entity.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStratum {
    Resource,
    Surface,
}

/// Canonical source endpoints within one stratum. Resource endpoints use
/// canonical IRIs; Surface endpoints use stable source identities.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LinkKey {
    pub stratum: LinkStratum,
    pub first: String,
    pub second: String,
}

impl LinkKey {
    pub fn new(stratum: LinkStratum, a: impl Into<String>, b: impl Into<String>) -> Self {
        let (a, b) = (a.into(), b.into());
        let (first, second) = if a <= b { (a, b) } else { (b, a) };
        Self {
            stratum,
            first,
            second,
        }
    }

    /// Deterministic view token, not a stored entity or assertion handle.
    /// Version 1 encodes stratum and UTF-8 byte-length-prefixed endpoints.
    pub fn stable_id(&self) -> String {
        let stratum = match self.stratum {
            LinkStratum::Resource => "resource",
            LinkStratum::Surface => "surface",
        };
        format!(
            "link:v1:{stratum}:{}:{}{}:{}",
            self.first.len(),
            self.first,
            self.second.len(),
            self.second
        )
    }
}

/// Marks relative to the routed appearance's from-to orientation. Several
/// assertions can contribute marks to one resting stroke.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkMarks {
    pub forward: bool,
    pub reverse: bool,
    pub opposed: bool,
}

impl LinkMarks {
    pub fn reversed(self) -> Self {
        Self {
            forward: self.reverse,
            reverse: self.forward,
            opposed: self.opposed,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.forward |= other.forward;
        self.reverse |= other.reverse;
        self.opposed |= other.opposed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_view_tokens_separate_strata_and_ambiguous_delimiters() {
        let key = LinkKey::new(LinkStratum::Surface, "b:é", "a");
        assert_eq!(key, LinkKey::new(LinkStratum::Surface, "a", "b:é"));
        assert_eq!(key.stable_id(), "link:v1:surface:1:a4:b:é");
        assert_ne!(
            key.stable_id(),
            LinkKey::new(LinkStratum::Resource, "a", "b:é").stable_id()
        );
        assert_ne!(
            LinkKey::new(LinkStratum::Surface, "a", "bc").stable_id(),
            LinkKey::new(LinkStratum::Surface, "ab", "c").stable_id()
        );
    }
}
