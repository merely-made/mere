// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The doors this resident serves, and the identity authority behind them.
//!
//! The first-party application door's server (`app_broker`), the browser
//! door's host (`browser_host`) and the device broker that relays a browser's
//! native-messaging frames to it (`device_broker`), the local session both
//! doors admit through (`local_session`), the resident identity authority
//! they serve (`resident_identity`), and the browser's native-messaging
//! envelope (`browser_carrier`) with the identity endpoint and its cards
//! (`identity_endpoint`, `identity_projection`). They were Graphshell's until
//! the stack lift plan's P5 (L3; the browser carrier and identity pair by
//! Mark's ruling after it); Djinn was their only product caller. The
//! application side of the door, with the endpoint that forwards over it
//! (`resident_mere_proxy`), is `graphshell_endpoint::door`; the serving half
//! they compose is `graphshell_host`.

pub mod app_broker;
#[cfg(test)]
mod app_broker_tests;
pub mod browser_carrier;
pub mod browser_host;
pub mod device_broker;
pub mod identity_endpoint;
pub mod identity_projection;
#[cfg(test)]
mod identity_session_tests;
pub mod local_session;
pub mod projection_offers;
pub mod resident_identity;
