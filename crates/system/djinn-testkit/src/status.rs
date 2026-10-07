// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a resident says about itself: its `resident-status-v1` route, read
//! through `djinn --resident-status`, and the lines of its `--events-file`.
//!
//! The testkit cannot depend on djinn (castellan and personae use it), so
//! these are readers of djinn's wire forms; djinn's own tests hold the two
//! together.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const STATUS_SCHEMA: &str = "djinn.resident-status/v1";
pub const EVENTS_SCHEMA: &str = "djinn.resident-events/v1";

/// The status route as the harness reads it. Unknown fields are kept, so a
/// newer djinn still reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResidentStatus {
    pub schema: String,
    pub pid: u32,
    pub started_ms: u64,
    pub installed: bool,
    pub ready: bool,
    pub startup_unlock: String,
    pub protection: String,
    pub lock: String,
    pub endpoints: StatusEndpoints,
    pub sync: Option<StatusSync>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEndpoints {
    pub agent: String,
    pub agent_listener: String,
    pub browser: String,
    pub app: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusSync {
    pub node_id: String,
    pub ticket: String,
}

/// One lifecycle line: `started`, `listening`, `ready`, `stopping`,
/// `stopped`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResidentEvent {
    pub schema: String,
    pub at_ms: u64,
    pub pid: u32,
    pub event: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// Every well-formed event line in `text`.
pub fn parse_events(text: &str) -> Vec<ResidentEvent> {
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}
