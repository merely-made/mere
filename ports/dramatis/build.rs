// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Package freshness: the app reports its compiled source and locked graph.
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

fn main() {
    let package = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = package.parent().unwrap().parent().unwrap();
    let mut names = vec!["Cargo.lock".to_string(), "Cargo.toml".into()];
    names.extend(["Cargo.toml", "build.rs"].map(|s| format!("ports/dramatis/{s}")));
    for entry in fs::read_dir(package.join("src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|s| s == "rs") {
            names.push(format!(
                "ports/dramatis/src/{}",
                path.file_name().unwrap().to_str().unwrap()
            ));
        }
    }
    names.sort();
    let mut hash = Sha256::new();
    for name in names {
        let path = root.join(&name);
        println!("cargo:rerun-if-changed={}", path.display());
        let source = fs::read_to_string(path).unwrap().replace("\r\n", "\n");
        hash.update(name.as_bytes());
        hash.update([0]);
        hash.update(source.as_bytes());
        hash.update([0]);
    }
    println!("cargo:rerun-if-changed={}", package.join("src").display());
    let digest = hash
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("");
    println!("cargo:rustc-env=DRAMATIS_SOURCE_SHA256={digest}");
    println!(
        "cargo:rustc-env=DRAMATIS_BUILD_TARGET={}",
        env::var("TARGET").unwrap()
    );
}
