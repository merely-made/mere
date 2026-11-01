// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Binds `mere:script`'s projection-session interface when the `wit`
//! feature is on (capability door D18).

fn main() {
    #[cfg(feature = "wit")]
    {
        let wit =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../script/wit/world.wit");
        println!("cargo::rerun-if-changed={}", wit.display());
        let src =
            script_witgen::generate(&wit, "projection-session").expect("projection-session binds");
        let out =
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("projection_session.rs");
        std::fs::write(out, src).unwrap();
    }
}
