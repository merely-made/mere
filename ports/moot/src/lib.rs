// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Gemot**, the community surface for murmurs, moots, and coop.
//!
//! The first-party community surface composes independently usable activities:
//!
//! - **murmurs** — secret conversations. Direct and invitation-scoped
//!   conversations, store-and-forward mail, history, drafts, delivery,
//!   refusal and retry, attachments through shared content custody, and
//!   calls once the transport and media receipts support them. Its model is
//!   `mere-comms` (the WASM-clean inbox: `Conversation`, `Message`, `Draft`,
//!   and the `ProtocolAdapter` seam).
//! - **moots** — spaces of agreement. Find, preview, join, leave and
//!   reconnect ceremony; membership and role inspection; proposals,
//!   decisions, moderation and appeals; storage and compute contributions;
//!   space health, replication and reachability.
//!
//! **coop** is shared activity among peers; application domains own its state.
//!
//! **murmur must mount alone.** Signalman wants messages and voice drops
//! without governance UI, and that constraint is what keeps the two surfaces
//! honestly separable rather than one screen with tabs.
//!
//! The boundaries are the point:
//!
//! - **Not the governance authority.** That is `gemot`, which owns a moot's
//!   lifecycle, membership, constitution, and trust facts. Tier 3 federation
//!   is `moothold`.
//! - **Not the exchange.** That is `murm`, which owns the post grammar, the
//!   signed per-author log, admission, and the sync lanes.
//! - **Not the shared graph.** That is the commons spine over `chartulary`,
//!   and not the replication mechanics either, which are `stickleback`'s.
//! - **Not a Turnstone feature.** The 2026-07-28 place-port plan ruled a
//!   place a Turnstone composition; the 2026-08-22 suite census reversed the
//!   application half of that ruling and left the authority half standing.
//!   Turnstone composes this port like any other host.
//!
//! The technical package remains `mere-moot` (library `moot`) at `ports/moot`.
//! Gemot is the product name; the `gemot` dependency remains the governance owner.
#![doc(html_no_source)]

#[cfg(feature = "captured-web")]
pub mod captured_web;
