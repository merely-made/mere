// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The first-party application door, application side (`door` feature).
//!
//! What an application embeds to reach the resident (djinn) on its owner-only
//! local endpoint: the self-declared hello and the host's route table
//! ([`app_admission`], [`route`]), the envelope and frames both halves share
//! ([`wire`], [`frame`]), the client ([`app_client`]), the custody route's
//! wire and its clients ([`custody`], [`custody_client`], [`custody_identity`]),
//! quarantine review ([`quarantine`]), the local socket or named pipe
//! ([`local_endpoint`]), tracked tasks ([`tasks`]), the secret-free identity
//! views ([`identity`]) and the application's persona ([`profile`]).
//!
//! The serving half (the broker, the browser door, the session loops) stays
//! in Graphshell (`ports/graphshell`), and an application needs none of it.
//! Lifted from Graphshell's `native` module (Graphshell stack lift plan, L1,
//! ruling R3).

pub mod app_admission;
pub mod app_client;
pub mod custody;
pub mod custody_client;
pub mod custody_identity;
pub mod custody_payload;
pub mod frame;
pub mod identity;
pub mod local_endpoint;
pub mod profile;
pub mod quarantine;
pub mod resident_mere_proxy;
pub mod route;
pub mod wire;
