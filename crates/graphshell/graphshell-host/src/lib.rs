// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The serving half of a Graphshell session.
//!
//! What a host needs to serve projections to an admitted peer, without the
//! `graphshell` port: Graphshell's admission (the action triple and its
//! check), the accept path over any [`transport`] stream, the NDJSON line
//! codec, the admitted session loop with its authority re-read and its
//! pushed notices, the resident endpoint catalog that names routes, the
//! projection host that runs accept loops over a catalog, and the WebRTC
//! door's sans-I/O core and join (Graphshell stack lift plan, L2, ruling R4;
//! transport agnosticism note, rulings T1, T2 and T6).
//!
//! The seam below this crate is an admitted byte stream plus the ingress
//! facts admission read (T1). Every carrier reaches it: a QUIC stream through
//! [`carrier::accept_projection_session`], a stream some other path accepted
//! through [`carrier::admit_accepted_session`], an in-process door's duplex
//! through [`carrier::admit_stream`], and a message channel that hands over
//! the hello as bytes through [`admission::admit_session`]. All four end in
//! the one action check, [`admission::serves_action`]. Inside every stream
//! the wire is NDJSON (T2), read and written by [`line`].
//!
//! The application side is elsewhere: the door an application dials is
//! `graphshell_endpoint::door`, and the client state and session drivers are
//! `graphshell_client`. The brokers a resident runs over this crate (the app
//! door's server, the browser host) are Djinn's `door` module.
//!
//! ## Features
//!
//! | Feature | Modules |
//! |---|---|
//! | none | [`resume`], [`live_endpoint`] and [`canary`] (fixture endpoints), [`view`] (the projection receipt views) and [`sessions`] (the session switcher's receipt; its endpoint spawning needs `native`) |
//! | `admission` | [`admission`] |
//! | `webrtc-join` (default) | [`webrtc_door`], [`webrtc_join`] |
//! | `native` | [`carrier`], [`line`], [`lifecycle`], [`session_loop`], [`session_notices`], [`endpoint_catalog`], [`projection_host`], [`network_carrier`], [`policy_projection`], and the host halves of [`webrtc_door`] |
//! | `webrtc-session` | `webrtc_session` and [`projection_host`]'s WebRTC door |
//!
//! The default build, `admission` and no features at all build for
//! `wasm32-unknown-unknown` (a browser build also needs
//! `--cfg getrandom_backend="wasm_js"`). `native` is for native targets: its
//! modules are compiled out on wasm32, but its dependencies are not meant to
//! build there.

#[cfg(feature = "admission")]
pub mod admission;
pub mod canary;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod carrier;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod endpoint_catalog;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod lifecycle;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod line;
pub mod live_endpoint;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod network_carrier;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod policy_projection;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod projection_host;
pub mod resume;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod session_loop;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod session_notices;
pub mod sessions;
pub mod view;
#[cfg(feature = "webrtc-join")]
pub mod webrtc_door;
#[cfg(feature = "webrtc-join")]
pub mod webrtc_join;
#[cfg(all(feature = "webrtc-session", not(target_arch = "wasm32")))]
pub mod webrtc_session;
