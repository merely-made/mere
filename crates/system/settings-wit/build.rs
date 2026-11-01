// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Binds `mere:script`'s settings interface (capability door K1, D22).

fn main() {
    let wit = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../script/wit/world.wit");
    println!("cargo::rerun-if-changed={}", wit.display());
    let src = script_witgen::generate(&wit, "settings").expect("settings binds");
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("settings.rs");
    std::fs::write(out, src).unwrap();
}
