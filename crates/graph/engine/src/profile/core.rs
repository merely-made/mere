// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The core pack (declared-lens D13, lessons D13): authored in
//! `core/pack.toml`, compiled by build.rs, embedded as its canonical bytes and
//! loaded through the same path as any pack, as a local, unsigned pack. One
//! core profile serves the process (D23).

use std::sync::LazyLock;

use super::intern::{TermId, TermTable};
use super::load::{Limits, NoPacks, Profile, load_local_lens};
use super::resolve::PackRef;

/// The core pack's id.
pub const CORE_PACK_ID: &str = "https://mere.computer/packs/core";

/// The id of the local pack a mere's predicate declarations compile into (D24).
pub const DECLARATIONS_PACK_ID: &str = "https://mere.computer/packs/declarations";

/// The core pack's canonical CBOR.
pub static CORE_PACK: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/core_pack.cbor"));

/// The core terms' dense ids (D8), generated from the core pack.
pub mod ids {
    use super::super::intern::TermId;

    include!(concat!(env!("OUT_DIR"), "/core_ids.rs"));
}

static CORE: LazyLock<Profile> = LazyLock::new(|| {
    load_local_lens(&PackRef::Core, &NoPacks, &Limits::default())
        .expect("the embedded core pack loads")
});

/// The process's core profile, loaded once (D23).
pub fn profile() -> &'static Profile {
    &CORE
}

/// The core profile's terms: the flat table built-in relations read (D15).
pub fn terms() -> &'static TermTable {
    &CORE.terms
}

/// A core term's IRI.
///
/// # Panics
///
/// When `id` is not a core id.
pub fn iri(id: TermId) -> &'static str {
    ids::TERMS[id.0 as usize].0
}
