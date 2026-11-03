// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The package's copy of `mere:script`'s world is the workspace's, byte for
//! byte. A published crate cannot read `../../script/wit/world.wit`, so the
//! build script reads `wit/world.wit`; this keeps the two from drifting.
//! Outside the Mere workspace the source is absent and the check has nothing
//! to compare.

#[test]
fn the_packaged_world_equals_the_workspaces() {
    let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Ok(source) = std::fs::read(package.join("../../script/wit/world.wit")) else {
        return;
    };
    let copy = std::fs::read(package.join("wit/world.wit")).expect("the packaged world");
    assert!(
        source == copy,
        "wit/world.wit differs from crates/script/wit/world.wit; copy the latter over it",
    );
}
