// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The §3.5 golden selection records, reopen by pin, and the proptest that
//! resolution ignores the order the source offers packs in (D13).

use eidetic::schema::{Hash, HashFn};
use proptest::prelude::*;

use super::support::*;
use crate::profile::core::CORE_PACK_ID;
use crate::profile::lens::AdmittedAs;
use crate::profile::resolve::PackRef;

fn record_digest(profile: &crate::profile::load::Profile) -> String {
    let bytes = profile.selection.to_canonical().unwrap();
    Hash::digest(HashFn::Blake3, &bytes).to_hex()
}

// Recorded 2026-11-13 from the core pack and fixtures at this commit; a change
// to either moves them, and that is the point.
const CORE_RECORD: &str = "43b6017241b18dfe94fcec83f1332025047544f7e38b8f418d70fc8bdbc9d5f6";
const WORKSPACE_RECORD: &str = "93d5e397d55682d091bb7045d696d10a41932453eec2a91d9a6fa7672d996fe0";

#[test]
fn the_core_pack_alone_is_a_golden_record() {
    let p = load_with(&PackRef::Core, &Source::default()).unwrap();
    assert_eq!(p.selection.lens.id, CORE_PACK_ID);
    assert_eq!(p.selection.pins.len(), 1);
    assert_eq!(p.selection.pins[0].pin, p.selection.lens);
    assert_eq!(
        p.selection.pins[0].admitted,
        AdmittedAs::Local,
        "local and unsigned (D13)"
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(record_digest(&p), CORE_RECORD);
}

#[test]
fn the_workspace_lens_is_a_golden_record() {
    let p = workspace_profile();
    let ids: Vec<&str> = p.selection.pins.iter().map(|r| r.pin.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            CORE_PACK_ID,
            WORKSPACE_ID,
            "https://mere.computer/packs/test/workspace-lens"
        ],
        "core first, then dependencies before dependents"
    );
    assert_eq!(p.selection.lens, p.selection.pins[2].pin);
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(record_digest(&p), WORKSPACE_RECORD);
}

#[test]
fn a_pinned_reopen_reproduces_the_record() {
    let first = workspace_profile();
    let again = load_with(
        &PackRef::Pinned(first.selection.lens.clone()),
        &workspace_source(),
    )
    .unwrap();
    assert_eq!(again.selection, first.selection);
    assert_eq!(again.terms, first.terms);
    let mut wrong = first.selection.lens.clone();
    wrong.version = "0.1.1".into();
    assert!(load_with(&PackRef::Pinned(wrong), &workspace_source()).is_err());
}

/// Several packs at several versions, some of them not chosen; compiled once.
fn offers() -> &'static [crate::profile::resolve::Offered] {
    static OFFERS: std::sync::OnceLock<Vec<crate::profile::resolve::Offered>> =
        std::sync::OnceLock::new();
    OFFERS.get_or_init(|| {
        vec![
            local(lens("l", &[("a", "1.0.0"), ("c", "0.2.0")], &[("t", "b")])),
            local(pack("a", "1.0.0", &[("b", "2.0.0")], &[("t", "resource")])),
            local(pack("a", "1.3.0", &[("b", "2.0.0")], &[("t", "resource")])),
            local(pack("a", "2.0.0", &[], &[])),
            local(pack("b", "2.0.0", &[], &[("t", "surface")])),
            local(pack(
                "b",
                "2.2.0",
                &[],
                &[("t", "surface"), ("u", "resource")],
            )),
            local(pack("c", "0.2.4", &[("b", "2.1.0")], &[("v", "surface")])),
            local(pack("c", "0.3.0", &[], &[])),
        ]
    })
}

proptest! {
    #[test]
    fn resolution_ignores_offer_order(order in Just((0..8).collect::<Vec<usize>>()).prop_shuffle()) {
        let baseline = load_with(&select("l"), &Source::of(offers().to_vec())).unwrap();
        let all = offers();
        let shuffled = Source::of(order.iter().map(|&i| all[i].clone()).collect());
        let p = load_with(&select("l"), &shuffled).unwrap();
        prop_assert_eq!(&p.selection, &baseline.selection);
        prop_assert_eq!(&p.terms, &baseline.terms);
        let versions: Vec<_> = p.selection.pins.iter().map(|r| r.pin.version.as_str()).collect();
        prop_assert_eq!(versions, vec!["0.1.0", "2.2.0", "1.3.0", "0.2.4", "1.0.0"]);
    }
}
