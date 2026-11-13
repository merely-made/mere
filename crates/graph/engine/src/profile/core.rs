// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The core pack (declared-lens D13, lessons D13): authored in
//! `core/pack.toml`, compiled by build.rs, embedded as its canonical bytes and
//! loaded through the same path as any pack, as a local, unsigned pack.

/// The core pack's id.
pub const CORE_PACK_ID: &str = "https://mere.computer/packs/core";

/// The core pack's canonical CBOR.
pub static CORE_PACK: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/core_pack.cbor"));

/// The core terms' dense ids (D8), generated from the core pack.
pub mod ids {
    use super::super::intern::TermId;

    include!(concat!(env!("OUT_DIR"), "/core_ids.rs"));
}
