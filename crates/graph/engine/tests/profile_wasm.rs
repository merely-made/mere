// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The core pack loads through `load` on wasm32 as on native (declared-lens
//! L2). Native runs it as a plain test; wasm32 under node:
//!
//! ```text
//! CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner \
//!   cargo test -p mere --target wasm32-unknown-unknown --test profile_wasm
//! ```

use eidetic::pack::{NoReview, TrustPolicy};
use identity::Ed25519Keypair;
use mere::model::GraphStratum;
use mere::profile::core::{CORE_PACK_ID, ids};
use mere::profile::lens::AdmittedAs;
use mere::profile::load::{Limits, NoPacks, load};
use mere::profile::resolve::PackRef;

fn core_loads() {
    // No release key exists to build in; any key will do for an unsigned local pack.
    let policy = TrustPolicy {
        release_key: Ed25519Keypair::from_seed([1; 32]).public_key(),
        review: NoReview,
    };
    let p =
        load(&PackRef::Core, &NoPacks, &policy, &Limits::default()).expect("the core pack loads");
    assert_eq!(p.selection.lens.id, CORE_PACK_ID);
    assert_eq!(p.selection.pins[0].admitted, AdmittedAs::Local);
    assert_eq!(p.terms.len(), ids::TERMS.len());
    let grouped = "https://mere.computer/ns/rel#user-grouped";
    assert_eq!(p.terms.stratum(grouped), GraphStratum::Surface);
    assert_eq!(
        p.terms.stratum("https://foreign.test/p"),
        GraphStratum::Resource
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_core_pack_loads() {
    core_loads();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn the_core_pack_loads() {
    core_loads();
}
