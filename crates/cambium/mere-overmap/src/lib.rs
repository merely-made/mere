// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The remote overmap: another device's or application's mere, read through
//! its mere route and shown as the mere view (Graphshell stack lift plan, L14
//! and R7).
//!
//! A resident serves a mere as a route with two projections, the mere's
//! sessions and the attached session's graph, and optionally its archive.
//! This crate is the reading side:
//!
//! - [`wire`]: the route's vocabulary, its projection names, intents, schemas
//!   and payloads. Always built; it carries no view and no connection.
//! - `route` (feature `view`): fills a [`mere_view::MereViewModel`] from the
//!   route's projection snapshots and turns the view's requests into the
//!   route's intents. It holds no connection.
//! - `route_session` (feature `view`): the event-driven panel over one
//!   admitted endpoint, on `graphshell_client::SessionDriver`. It never owns a
//!   store.
//!
//! The host obtains and admits the session itself, over whatever carrier it
//! has, and keeps its own identity and window. It is a sibling of `mere-view`
//! rather than a feature of it because every application embeds the view,
//! including hosts with a local mere and no session; this crate adds the
//! session client and chirograph only for hosts that read a route.

pub mod wire;

#[cfg(feature = "view")]
pub mod route;
#[cfg(feature = "view")]
pub mod route_session;
