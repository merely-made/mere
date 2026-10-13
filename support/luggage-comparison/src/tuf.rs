// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Uses pinned upstream public fixtures, not a Merely release repository.
use std::{fs, path::Path};
use serde_json::json;
use tough::{ExpirationEnforcement, RepositoryLoader};
fn dir_url(path: &Path) -> url::Url { url::Url::from_directory_path(path).unwrap() }

async fn probe(base: &Path) {
    let expired = base.join("expired-repository");
    let root = fs::read(expired.join("metadata/1.root.json")).unwrap();
    let checked = RepositoryLoader::new(&root, dir_url(&expired.join("metadata")), dir_url(&expired.join("targets"))).load().await;
    let mut result = json!({"expired_repository_refused": checked.is_err(), "expiry_error": checked.err().map(|e| e.to_string())});
    result["expiry_bypass_negative_control_accepted"] = json!(RepositoryLoader::new(&root, dir_url(&expired.join("metadata")), dir_url(&expired.join("targets"))).expiration_enforcement(ExpirationEnforcement::Unsafe).load().await.is_ok());
    let rotated = base.join("rotated-root");
    let root = fs::read(rotated.join("1.root.json")).unwrap();
    let state = tempfile::tempdir().unwrap();
    let repo = RepositoryLoader::new(&root, dir_url(&rotated), dir_url(&rotated.join("targets"))).datastore(state.path()).load().await.unwrap();
    result["rotated_root_version"] = json!(u64::from(repo.root().signed.version));
    let copy = tempfile::tempdir().unwrap();
    for entry in fs::read_dir(&rotated).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() { fs::copy(entry.path(), copy.path().join(entry.file_name())).unwrap(); }
    }
    fs::create_dir(copy.path().join("targets")).unwrap();
    let timestamp = copy.path().join("timestamp.json");
    let original = fs::read(&timestamp).unwrap();
    let mut edited: serde_json::Value = serde_json::from_slice(&original).unwrap();
    edited["signed"]["version"] = json!(99);
    fs::write(&timestamp, serde_json::to_vec(&edited).unwrap()).unwrap();
    result["edited_timestamp_refused"] = json!(RepositoryLoader::new(&root, dir_url(copy.path()), dir_url(&copy.path().join("targets"))).load().await.is_err());
    fs::write(&timestamp, original).unwrap();
    fs::write(copy.path().join("2.root.json"), &root).unwrap();
    let replay = RepositoryLoader::new(&root, dir_url(copy.path()), dir_url(&copy.path().join("targets"))).load().await;
    result["old_root_at_next_version_refused"] = json!(replay.is_err());
    result["root_replay_error"] = json!(replay.err().map(|e| e.to_string()));
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
fn main() {
    let base = std::env::args().nth(1).expect("fixture directory");
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(probe(Path::new(&base)));
}
