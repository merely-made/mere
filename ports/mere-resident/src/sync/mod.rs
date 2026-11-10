// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The device-agent hosts behind personal graph synchronization.
//!
//! The personal sync host (`personal_sync_host`), the carriage host over its
//! endpoint (`carriage_host`), the graph's transport and agreement keys
//! (`graph_keys`), transfer staging (`transfer_staging`) and receipt ingest
//! (`receipts`). They were
//! Graphshell's `native` module, under its `personal-sync` feature, until the
//! stack lift plan's P5 (L3). The replica, event and offer types they serve
//! are pandect's: `pandect::personal_sync` (engine split D69) and
//! `pandect::transfer` and `pandect::transfer_offer` (D70).

pub mod carriage;
pub mod carriage_host;
pub mod graph_keys;
pub mod personal_seals;
pub mod personal_sync_host;
pub mod receipts;
pub mod transfer_staging;
