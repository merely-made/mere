// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bindings script-wire's conformance tests compile are this generator's
//! output for the conformance interface. Run with `SCRIPT_WITGEN_BLESS=1` to
//! rewrite them after a deliberate change.

use std::path::Path;

#[test]
fn conformance_bindings_are_current() {
    let wire = Path::new(env!("CARGO_MANIFEST_DIR")).join("../wire/tests");
    let generated = script_witgen::generate(&wire.join("wit/conformance.wit"), "values")
        .expect("the conformance interface binds");
    let golden = wire.join("generated/conformance.rs");
    if std::env::var_os("SCRIPT_WITGEN_BLESS").is_some() {
        std::fs::write(&golden, &generated).unwrap();
        return;
    }
    let current = std::fs::read_to_string(&golden).unwrap_or_default();
    assert!(
        current.replace("\r\n", "\n") == generated,
        "{} is stale; rerun with SCRIPT_WITGEN_BLESS=1",
        golden.display()
    );
}
