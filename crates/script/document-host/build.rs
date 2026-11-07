// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Builds the two WASM guest components the integration tests load.
//!
//! The guests (`guest/`, `guest-bomb/`) are standalone workspaces compiled to
//! `wasm32-wasip2`. Before this script existed they had to be built by hand, so a
//! fresh checkout or a CI runner failed four tests until someone read the panic
//! message and ran two commands. Nothing about that was machine-specific, it was
//! just manual.
//!
//! The tests resolve the artifacts by a path relative to the package root (cargo
//! runs tests with the CWD set there), overridable via `DOC_HOST_GUEST_WASM` /
//! `DOC_HOST_BOMB_WASM`. This script only has to make sure the artifacts exist at
//! those paths, so it needs no cooperation from the tests.
//!
//! If the `wasm32-wasip2` target is missing, this warns and moves on rather than
//! failing the build: the library itself does not need wasm, and the tests already
//! fail loudly, naming the exact command to run. A missing guest must never become
//! a silently-skipped test.

use std::path::Path;
use std::process::Command;

const GUESTS: [(&str, &str); 2] = [
    ("guest", "document_core_guest.wasm"),
    ("guest-bomb", "document_core_guest_bomb.wasm"),
];

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
    let root = Path::new(&root);

    for (dir, artifact) in GUESTS {
        let guest = root.join(dir);
        if !guest.join("Cargo.toml").exists() {
            println!("cargo::warning={dir}: no Cargo.toml; skipping guest build");
            continue;
        }

        // Rebuild when the guest's own sources move, not on every touch of the host.
        println!("cargo::rerun-if-changed={}/src", guest.display());
        println!("cargo::rerun-if-changed={}/Cargo.toml", guest.display());
        println!("cargo::rerun-if-changed={}/Cargo.lock", guest.display());
        println!(
            "cargo::rerun-if-changed={}/rust-toolchain.toml",
            guest.display()
        );

        // The guest is a separate workspace with its own target dir and toolchain
        // pin. Strip the parent cargo's environment so the child resolves its own:
        // inherited RUSTFLAGS / CARGO_TARGET_DIR / toolchain vars would otherwise
        // leak the host's native build into a wasm one.
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
            // Clearing CARGO_HOME would escape a portable-check isolation home.
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
                continue;
            },
            Err(err) => {
                println!("cargo::warning={dir}: could not run cargo ({err})");
                continue;
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
}
