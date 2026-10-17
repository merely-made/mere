// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The device-agent hosts behind personal graph synchronization.
//!
//! The personal sync host (`personal_sync_host`), the carriage host over its
//! endpoint (`carriage_host`), the graph's transport and agreement keys
//! (`graph_keys`) and transfer staging (`transfer_staging`). They were
//! Graphshell's `native` module, under its `personal-sync` feature, until the
//! stack lift plan's P5 (L3). The replica, event and offer types they serve
//! are still Graphshell's (`graphshell::personal_sync`, `transfer`,
//! `transfer_offer`, `receipts`), which L13 moves later.

pub mod carriage_host;
pub mod graph_keys;
pub mod personal_seals;
pub mod personal_sync_host;
pub mod transfer_staging;
