// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Coverage notes: what a supplied graph is known not to hold. Moved from the kernel (engine split P5 L3).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLayer {
    Possession,
    Residency,
    Disclosure,
    Synchronization,
    Projection,
}

impl CoverageLayer {
    pub const ALL: [Self; 5] = [
        Self::Possession,
        Self::Residency,
        Self::Disclosure,
        Self::Synchronization,
        Self::Projection,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageLimit {
    pub layer: CoverageLayer,
    pub reason: String,
    /// Empty scope means the entire supplied graph. Hosts own these observations.
    #[serde(default)]
    pub resources: Vec<Uuid>,
    /// Surface scope; empty scopes together mean the supplied graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaces: Vec<Uuid>,
    /// None means the number beyond this boundary is unknown.
    #[serde(default)]
    pub count: Option<usize>,
}

impl CoverageLimit {
    pub fn new(layer: CoverageLayer, reason: impl Into<String>) -> Self {
        Self {
            layer,
            reason: reason.into(),
            resources: vec![],
            surfaces: vec![],
            count: None,
        }
    }
}

/// An empty note means no known limit within this supplied graph.
/// It never means all relevant data exists here or anywhere else.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageNote {
    pub limits: Vec<CoverageLimit>,
}

impl CoverageNote {
    pub fn push(&mut self, limit: CoverageLimit) {
        if !self.limits.contains(&limit) {
            self.limits.push(limit);
        }
    }
    pub fn add_count(&mut self, layer: CoverageLayer, reason: impl Into<String>, count: usize) {
        if count == 0 {
            return;
        }
        let mut limit = CoverageLimit::new(layer, reason);
        limit.count = Some(count);
        self.push(limit);
    }
}
