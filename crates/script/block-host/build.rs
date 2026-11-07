// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Builds the `block-guest` fixture component the integration tests load.
//!
//! Same shape as document-host's build script: the guest is a standalone
//! workspace compiled to `wasm32-wasip2`, and the tests resolve the artifact
//! by a path relative to the package root (overridable via
//! `BLOCK_HOST_GUEST_WASM`). A missing `wasm32-wasip2` target warns and moves on
//! rather than failing the build — the library itself needs no wasm, and the
//! tests fail loudly naming the command to run, so a missing guest can never
//! become a silently-skipped test.

use std::path::Path;
use std::process::Command;

/// Each guest: its directory, its artifact, and what else it is built from.
const GUESTS: &[(&str, &str, &[&str])] = &[("guest", "block_fixture_guest.wasm", &[])];

/// The `mere:script` WIT lives once, in the Graphshell repository
/// (`graphshell-endpoint/wit/world.wit`). The bindgen macros and the guests'
/// `generate!` take only literal paths relative to the crate, so the hosts
/// materialise that file as `crates/script/wit/world.wit` (ignored by git) from
/// the Cargo `links` metadata `graphshell-endpoint` publishes, before this crate
/// or its guests compile. It is a build product of the pinned dependency, not a
/// second copy to keep in step.
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
    // Two hosts materialise the same file in parallel: write beside it, then rename.
    let staged = target_dir.join(format!("world.wit.{}.tmp", std::process::id()));
    std::fs::write(&staged, &wanted).expect("stage the WIT world");
    std::fs::rename(&staged, &target).expect("place the WIT world");
}

fn main() {
    materialize_wit();
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    for &(dir, artifact, watch) in GUESTS {
        build(Path::new(&root), dir, artifact, watch);
    }
}

fn build(root: &Path, dir: &str, artifact: &str, watch: &[&str]) {
    let guest = root.join(dir);
    if !guest.join("Cargo.toml").exists() {
        println!("cargo::warning={dir}: no Cargo.toml; skipping guest build");
        return;
    }

    println!("cargo::rerun-if-changed={}/src", guest.display());
    println!("cargo::rerun-if-changed={}/Cargo.toml", guest.display());
    println!("cargo::rerun-if-changed={}/Cargo.lock", guest.display());
    println!(
        "cargo::rerun-if-changed={}/rust-toolchain.toml",
        guest.display()
    );
    println!("cargo::rerun-if-changed=../wit/world.wit");
    for path in watch {
        println!("cargo::rerun-if-changed={path}");
    }

    // The guest is a separate workspace with its own target dir and toolchain
    // pin. Strip the parent cargo's environment so the child resolves its own.
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&guest).args([
        "build",
        "--locked",
        "--target",
        "wasm32-wasip2",
        "--release",
    ]);
    for (key, _) in std::env::vars() {
        // Keep the caller's cache/config boundary and resource limit.
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
        Ok(status) => {
            println!(
                "cargo::warning={dir}: guest build failed ({status}). The tests that load \
                 it will fail and name the command to run by hand."
            );
            return;
        },
        Err(err) => {
            println!("cargo::warning={dir}: could not run cargo ({err})");
            return;
        },
    }

    let built = guest.join("target/wasm32-wasip2/release").join(artifact);
    if !built.exists() {
        println!(
            "cargo::warning={dir}: build reported success but {} is missing",
            built.display()
        );
    }
}
