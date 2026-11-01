// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The projection session over `mere:script`'s WIT interface (capability
//! door C0, D13-D19). This is what an N1 channel carries for every provider,
//! native or component.
//!
//! [`bindings`] is script-witgen's output for the interface. [`WitCarrier`]
//! is a chirograph `Carrier` over any [`SessionCalls`]: the generated client
//! natively ([`Calls`]), or a component's imports. [`WitServer`] serves a
//! complete endpoint through the generated handler, rechecking the session's
//! authority on every call through its [`SessionGate`]; [`serve`] runs it on
//! one channel and closes every session that channel opened when it ends.

#[allow(clippy::all, dead_code, unused, missing_docs)]
pub mod bindings {
    use script_wire::*;
    include!(concat!(env!("OUT_DIR"), "/projection_session.rs"));
}

mod carrier;
mod client;
pub mod payload;
mod server;
#[cfg(feature = "wit-testkit")]
pub mod testkit;

pub use bindings::{Failure, Notice};
pub use carrier::{Fault, SessionCalls, WitCarrier};
pub use client::Calls;
pub use server::{SessionGate, WitServer, serve};

use chirograph::{ProtocolVersion, Revision, SceneEpoch};
use sceno::{InstanceId, Score};
use serde::{Deserialize, Serialize};

/// The largest frame a channel carries: one capped payload and its call.
pub const MAX_FRAME: usize = payload::MAX_PAYLOAD + 4096;

/// A snapshot request without its session, which the resource holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionAsk {
    pub version: ProtocolVersion,
    pub score: Score,
}

/// An intent without its session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntentCall {
    pub target: InstanceId,
    pub observed_epoch: SceneEpoch,
    pub observed_revision: Revision,
    pub intent: String,
    pub payload: Vec<u8>,
}
