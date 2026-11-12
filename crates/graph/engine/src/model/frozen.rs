// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Frozen selections and links: immutable readings a resource bears. The
//! types moved from the kernel (engine split P5 L3); their record validation
//! stays with the kernel's snapshot code.

use super::{CoverageNote, GraphStratum, PersistedEdge};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const FROZEN_SELECTION: &str = "graph.frozen-selection/v1";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenSelection {
    version: u32,
    spec: serde_json::Value,
    revision: u64,
    members: Vec<Uuid>,
    statements: Vec<PersistedEdge>,
}

impl FrozenSelection {
    /// A version-1 selection of `members` and the statements among them.
    pub fn v1(
        spec: serde_json::Value,
        revision: u64,
        members: Vec<Uuid>,
        statements: Vec<PersistedEdge>,
    ) -> Self {
        Self {
            version: 1,
            spec,
            revision,
            members,
            statements,
        }
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn spec(&self) -> &serde_json::Value {
        &self.spec
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn members(&self) -> &[Uuid] {
        &self.members
    }
    pub fn statements(&self) -> &[PersistedEdge] {
        &self.statements
    }
}

pub const FROZEN_LINK: &str = "graph.frozen-link/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenLinkEndpoint {
    pub id: Uuid,
    pub resource: Option<Uuid>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenSourceBinding {
    pub authority: String,
    pub domain: String,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrozenSourceRevision {
    Available { revision: String },
    Unavailable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrozenLinkSource {
    Available {
        binding: FrozenSourceBinding,
        revision: FrozenSourceRevision,
    },
    Unavailable {
        reason: String,
    },
}

impl FrozenLinkSource {
    /// Check the source fields are present and trimmed.
    pub fn validate(&self) -> Result<(), String> {
        let nonempty = |value: &str| !value.trim().is_empty() && value == value.trim();
        match self {
            Self::Available { binding, revision } => {
                if !nonempty(&binding.authority)
                    || !nonempty(&binding.domain)
                    || !nonempty(&binding.resource)
                {
                    return Err("frozen source binding fields must be nonempty and trimmed".into());
                }
                match revision {
                    FrozenSourceRevision::Available { revision } if nonempty(revision) => Ok(()),
                    FrozenSourceRevision::Available { .. } => {
                        Err("frozen source revision must be nonempty and trimmed".into())
                    },
                    FrozenSourceRevision::Unavailable { reason } if nonempty(reason) => Ok(()),
                    FrozenSourceRevision::Unavailable { .. } => {
                        Err("unavailable source revision requires a reason".into())
                    },
                }
            },
            Self::Unavailable { reason } if nonempty(reason) => Ok(()),
            Self::Unavailable { .. } => Err("unavailable frozen source requires a reason".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenLink {
    version: u32,
    stratum: GraphStratum,
    first: FrozenLinkEndpoint,
    second: FrozenLinkEndpoint,
    revision: u64,
    source: FrozenLinkSource,
    coverage: CoverageNote,
    statements: Vec<PersistedEdge>,
}

impl FrozenLink {
    /// A version-1 reading of the pair buckets between two endpoints.
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        stratum: GraphStratum,
        first: FrozenLinkEndpoint,
        second: FrozenLinkEndpoint,
        revision: u64,
        source: FrozenLinkSource,
        coverage: CoverageNote,
        statements: Vec<PersistedEdge>,
    ) -> Self {
        Self {
            version: 1,
            stratum,
            first,
            second,
            revision,
            source,
            coverage,
            statements,
        }
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn stratum(&self) -> GraphStratum {
        self.stratum
    }
    pub fn first(&self) -> &FrozenLinkEndpoint {
        &self.first
    }
    pub fn second(&self) -> &FrozenLinkEndpoint {
        &self.second
    }
    /// The local structural revision observed at freeze time; not a public source revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn source(&self) -> &FrozenLinkSource {
        &self.source
    }
    pub fn coverage(&self) -> &CoverageNote {
        &self.coverage
    }
    pub fn statements(&self) -> &[PersistedEdge] {
        &self.statements
    }
}
