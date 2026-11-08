// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Materialises the `mere:script` WIT as `crates/script/wit/world.wit` (ignored by git) for
//! `wasmtime::component::bindgen!`, which takes only literal paths, and builds the fixture
//! guest the tests load. The same shape as block-host's build script: the WIT lives once, in
//! Graphshell's `graphshell-endpoint`, and this is a build product of that dependency.

use std::path::Path;
use std::process::Command;

fn materialize_wit() {
    let dir = std::env::var("DEP_GRAPHSHELL_ENDPOINT_WIT")
        .expect("graphshell-endpoint's build metadata names its wit directory");
    let source = Path::new(&dir).join("world.wit");
    println!("cargo::rerun-if-changed={}", source.display());
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let target_dir = Path::new(&root).join("../wit");
    let target = target_dir.join("world.wit");
    let wanted = std::fs::read(&source).expect("read graphshell-endpoint's WIT world");
    if std::fs::read(&target).ok().as_deref() == Some(wanted.as_slice()) {
        return;
    }
    std::fs::create_dir_all(&target_dir).expect("create crates/script/wit");
    // Several hosts materialise the same file in parallel: write beside it, then rename.
    let staged = target_dir.join(format!("world.wit.{}.tmp", std::process::id()));
    std::fs::write(&staged, &wanted).expect("stage the WIT world");
    std::fs::rename(&staged, &target).expect("place the WIT world");
}

fn main() {
    materialize_wit();
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let guest = Path::new(&root).join("guest");
    println!("cargo::rerun-if-changed={}/src", guest.display());
    println!("cargo::rerun-if-changed={}/Cargo.toml", guest.display());
    println!("cargo::rerun-if-changed={}/Cargo.lock", guest.display());
    println!("cargo::rerun-if-changed=../wit/world.wit");

    // A separate workspace with its own target dir and toolchain pin: strip the parent
    // cargo's environment so the child resolves its own. A failure warns and moves on; the
    // library needs no wasm and the tests that load the guest name the command to run.
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&guest)
        .args(["build", "--locked", "--target", "wasm32-wasip2", "--release"]);
    for (key, _) in std::env::vars() {
        if key == "CARGO_HOME" || key == "CARGO_BUILD_JOBS" {
            continue;
        }
        if key.starts_with("CARGO")
            || key.starts_with("RUSTC")
            || key.starts_with("RUSTUP")
            || key == "RUSTFLAGS"
            || key == "RUSTDOCFLAGS"
            || key == "TARGET"
        {
            cmd.env_remove(key);
        }
    }
    match cmd.status() {
        Ok(status) if status.success() => {},
        Ok(status) => println!("cargo::warning=authority fixture guest build failed ({status})"),
        Err(error) => println!("cargo::warning=could not run cargo for the authority fixture guest ({error})"),
    }
}
