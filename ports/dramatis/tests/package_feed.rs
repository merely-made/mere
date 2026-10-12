// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The real packager artifact and signatures must survive Luggage's gates.
use luggage::{Config, Feed, StagedUpdate};
use std::{fs, path::PathBuf};

struct Restore(PathBuf, Vec<u8>);
impl Restore {
    fn new(path: PathBuf) -> Self {
        let bytes = fs::read(&path).unwrap();
        Self(path, bytes)
    }
    fn restore(&self) {
        fs::write(&self.0, &self.1).unwrap();
    }
}
impl Drop for Restore {
    fn drop(&mut self) {
        let _ = fs::write(&self.0, &self.1);
    }
}

#[test]
#[ignore = "requires the marker-owned, test-signed package from packaging qualification"]
fn real_signed_package_and_tamper_controls() {
    let feed = PathBuf::from(std::env::var_os("DRAMATIS_PACKAGE_FEED").unwrap())
        .canonicalize()
        .unwrap();
    let root = feed.parent().unwrap().parent().unwrap();
    let marker: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(".dramatis-packaging.json")).unwrap()).unwrap();
    assert_eq!(marker["owner"], "dramatis-packaging");
    assert_eq!(
        fs::read_to_string(feed.join(".synthetic-package-receipt")).unwrap(),
        "test-signed Dramatis package fixture"
    );
    assert_eq!(
        feed.parent().unwrap().file_name().unwrap(),
        "dev",
        "controls cannot mutate a release feed"
    );
    let pubkey =
        fs::read_to_string(std::env::var_os("DRAMATIS_PACKAGE_PUBLIC_KEY_FILE").unwrap()).unwrap();
    let config = || Config {
        feeds: vec![Feed::Directory(feed.clone())],
        pubkey: pubkey.trim().into(),
        ..Default::default()
    };
    let check = || luggage::check_update("0.0.0".parse().unwrap(), config());
    let update = check()
        .unwrap()
        .expect("test package newer than the synthetic previous version");
    assert_eq!(update.version, env!("CARGO_PKG_VERSION"));
    assert!(update.blake3.is_some());
    let artifact = update
        .download_url
        .to_file_path()
        .unwrap()
        .canonicalize()
        .unwrap();
    assert_eq!(
        artifact.parent().unwrap(),
        feed,
        "only this generated package can be mutated"
    );
    let saved_artifact = Restore::new(artifact);
    let bytes = update.download().unwrap();
    assert_eq!(bytes, saved_artifact.1);

    let stage_dir = tempfile::tempdir_in(root).unwrap();
    let staged = update.stage(stage_dir.path(), &bytes).unwrap();
    assert_eq!(
        StagedUpdate::load(stage_dir.path())
            .unwrap()
            .unwrap()
            .take_verified()
            .unwrap(),
        bytes
    );
    let mut tampered = bytes.clone();
    tampered[0] ^= 1;
    fs::write(stage_dir.path().join("staged-artifact"), &tampered).unwrap();
    assert!(
        staged.take_verified().is_err(),
        "modified staged bytes must be refused"
    );
    staged.discard().unwrap();

    fs::write(&saved_artifact.0, &tampered).unwrap();
    assert!(
        update.download().is_err(),
        "modified signed artifact must be refused"
    );
    saved_artifact.restore();

    let manifest = Restore::new(feed.join("luggage.json"));
    let signature = Restore::new(feed.join("luggage.json.sig"));
    fs::remove_file(&signature.0).unwrap();
    assert!(check().is_err(), "unsigned manifest must be refused");
    signature.restore();

    let other_pubkey =
        fs::read_to_string(std::env::var_os("DRAMATIS_PACKAGE_OTHER_PUBLIC_KEY_FILE").unwrap())
            .unwrap();
    let mut other_publisher = config();
    other_publisher.pubkey = other_pubkey.trim().into();
    assert!(
        luggage::check_update("0.0.0".parse().unwrap(), other_publisher).is_err(),
        "another valid publisher key must be refused"
    );

    let mut edited: serde_json::Value = serde_json::from_slice(&manifest.1).unwrap();
    edited["version"] = "999.0.0".into();
    fs::write(&manifest.0, serde_json::to_vec(&edited).unwrap()).unwrap();
    assert!(
        check().is_err(),
        "tampered release metadata must be refused"
    );
    manifest.restore();

    // Artifact signature is independently checked after a correct digest.
    let saved_signature = update.signature.clone();
    let mut bad_signature = update.clone();
    bad_signature.signature = "AAAA".into();
    assert!(
        bad_signature.download().is_err(),
        "wrong artifact signature must be refused"
    );
    assert_eq!(update.signature, saved_signature);
    assert_eq!(
        check().unwrap().unwrap().download().unwrap(),
        bytes,
        "all feed bytes restored"
    );
    println!(
        "Signed package, durable staging, unsigned/edited manifests, artifact and staged-byte tampering qualified; feed restored."
    );
}
