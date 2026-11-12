// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A resource's persisted identity and metadata. Moved from the kernel's
//! `persistence` (engine split P5 L3); the encoding is unchanged.

use rkyv::{Archive, Deserialize, Serialize};

/// Ordinary metadata on a resource, encoded without a JSON codec assumption.
#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct PersistedResourceFacet {
    pub facet: String,
    pub value_json: String,
}

/// One resource's identity and metadata; its edges are stored separately.
#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct PersistedResourceRecord {
    pub canonical_iri: String,
    pub facets: Vec<PersistedResourceFacet>,
}
