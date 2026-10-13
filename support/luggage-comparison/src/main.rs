// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Disposable research fixtures. Never applies Luggage artifacts to an app.
use std::{fs, io::Cursor, path::Path, process::Command};
use luggage::{Config, Feed, StagedUpdate, UpdaterBuilder};
use serde_json::{json, Value};
use base64::Engine as _;

fn signature(keys: &minisign::KeyPair, bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(minisign::sign(None, &keys.sk, Cursor::new(bytes), None, None).unwrap().to_string())
}

fn publish(dir: &Path, keys: &minisign::KeyPair, version: &str) {
    let bytes = b"inert comparison artifact";
    let artifact = dir.join("artifact.bin");
    fs::write(&artifact, bytes).unwrap();
    let manifest = serde_json::to_vec(&json!({
        "version": version, "pub_date": "2000-01-01T00:00:00Z",
        "platforms": { "fixture": {
            "url": url::Url::from_file_path(&artifact).unwrap().to_string(),
            "signature": signature(keys, bytes),
            "blake3": blake3::hash(bytes).to_hex().to_string(), "format": "nsis"
        }}
    })).unwrap();
    fs::write(dir.join("luggage.json.sig"), signature(keys, &manifest)).unwrap();
    fs::write(dir.join("luggage.json"), manifest).unwrap();
}

fn luggage_probe() {
    let dir = tempfile::tempdir().unwrap();
    let keys = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
    publish(dir.path(), &keys, "0.2.0");
    let updater = UpdaterBuilder::new("0.1.0".parse().unwrap(), Config {
        feeds: vec![Feed::Directory(dir.path().to_path_buf())],
        pubkey: base64::engine::general_purpose::STANDARD.encode(keys.pk.to_box().unwrap().to_string()), ..Config::default()
    }).target("fixture").executable_path(dir.path().join("fake-exe")).build().unwrap();
    let update = updater.check().unwrap().unwrap();
    let bytes = update.download().unwrap();
    let stage = dir.path().join("stage");
    update.stage(&stage, &bytes).unwrap();
    let child = Command::new(std::env::current_exe().unwrap()).args(["stage-read", stage.to_str().unwrap()]).output().unwrap();
    let mut results = json!({"signed_download": true, "fresh_process_stage": serde_json::from_slice::<Value>(&child.stdout).unwrap()});

    fs::write(stage.join("staged-artifact"), b"changed inert fixture").unwrap();
    results["artifact_only_tamper_refused"] = json!(StagedUpdate::load(&stage).unwrap().unwrap().take_verified().is_err());
    let record_path = stage.join("staged.json");
    let mut record: Value = serde_json::from_slice(&fs::read(&record_path).unwrap()).unwrap();
    record["blake3"] = json!(blake3::hash(b"changed inert fixture").to_hex().to_string());
    fs::write(&record_path, serde_json::to_vec(&record).unwrap()).unwrap();
    let child = Command::new(std::env::current_exe().unwrap()).args(["stage-read", stage.to_str().unwrap()]).output().unwrap();
    results["artifact_and_digest_tamper"] = serde_json::from_slice::<Value>(&child.stdout).unwrap();
    record["format"] = json!("wix");
    record["version"] = json!("99.0.0");
    record["extract_path"] = json!(dir.path().join("other-app"));
    fs::write(&record_path, serde_json::to_vec(&record).unwrap()).unwrap();
    let found = StagedUpdate::load(&stage).unwrap().unwrap();
    results["unsigned_record_tamper_accepted"] = json!({"verified_bytes": found.take_verified().is_ok(), "version": found.version(), "format": format!("{:?}", found.format()), "destination_changed": found.extract_path() == dir.path().join("other-app")});
    publish(dir.path(), &keys, "0.5.0");
    results["highest_observed"] = json!(updater.check().unwrap().unwrap().version);
    publish(dir.path(), &keys, "0.3.0");
    results["older_signed_offer_after_higher_seen"] = json!(updater.check().unwrap().unwrap().version);
    results["old_publication_date_accepted"] = json!(true);
    fs::write(dir.path().join("luggage.json"), b"edited manifest").unwrap();
    results["edited_manifest_refused"] = json!(updater.check().is_err());
    fs::remove_file(dir.path().join("luggage.json.sig")).unwrap();
    results["unsigned_manifest_refused"] = json!(updater.check().is_err());
    fs::write(&record_path, b"partial record").unwrap();
    results["partial_record_ignored"] = json!(StagedUpdate::load(&stage).unwrap().is_none());
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}

fn main() {
    velopack::VelopackApp::build().set_auto_apply_on_startup(false).run();
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str).unwrap_or("info") {
        "luggage" => luggage_probe(),
        "stage-read" => {
            let found = StagedUpdate::load(&args[2]).unwrap().unwrap();
            println!("{}", json!({"accepted": found.take_verified().is_ok(), "version": found.version()}));
        }
        "check" | "download" | "apply" => {
            let manager = velopack::UpdateManager::new(velopack::sources::FileSource::new(&args[2]), None, None).unwrap();
            if args[1] == "apply" {
                let pending = manager.get_update_pending_restart().expect("pending local package");
                manager.wait_exit_then_apply_updates(&pending, true, false, Vec::<String>::new()).unwrap();
                return;
            }
            match manager.check_for_updates().unwrap() {
                velopack::UpdateCheck::UpdateAvailable(update) => {
                    if args[1] == "download" { manager.download_updates(&update, None).unwrap(); }
                    println!("{}", json!({"version": update.TargetFullRelease.Version.to_string(), "portable": manager.get_is_portable(), "pending": manager.get_update_pending_restart().is_some()}));
                }
                _ => println!("{}", json!({"update": false, "portable": manager.get_is_portable()})),
            }
        }
        _ => println!("{}", json!({"build_version": option_env!("LUGGAGE_COMPARISON_VERSION").unwrap_or("0.1.0")})),
    }
}
