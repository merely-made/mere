// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::BTreeMap;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::process::{Child, Command, Stdio};
use std::path::{Path, PathBuf};

struct Host { child: Child, endpoint: PathBuf, inode: (u64, u64) }
impl Host {
    fn start(root: &Path) -> (Self, CustodyIdentity) {
        let endpoint = root.join("app.sock");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["retained_custody_receipt_host", "--ignored", "--exact", "--nocapture"])
            .env("EIDETIC_CUSTODY_FIXTURE_ROOT", root).env("EIDETIC_CUSTODY_FIXTURE_ENDPOINT", &endpoint)
            .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        let identity = loop {
            if let Ok(identity) = CustodyIdentity::connect_declared_at_timeout(endpoint.to_str().unwrap(), "turnstone", super::turnstone_declaration(), Duration::from_secs(1)) { break identity; }
            assert!(child.try_wait().unwrap().is_none(), "fixture process exited");
            assert!(std::time::Instant::now() < deadline, "fixture did not start");
            std::thread::sleep(Duration::from_millis(50));
        };
        let meta = std::fs::symlink_metadata(&endpoint).unwrap();
        (Self { child, endpoint, inode: (meta.dev(), meta.ino()) }, identity)
    }
    fn stop(mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        let meta = std::fs::symlink_metadata(&self.endpoint).unwrap();
        assert_eq!((meta.dev(), meta.ino()), self.inode, "endpoint was replaced");
        assert!(std::os::unix::net::UnixStream::connect(&self.endpoint).is_err(), "endpoint still has a listener");
        std::fs::remove_file(&self.endpoint).unwrap();
    }
}
impl Drop for Host { fn drop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); } }

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() { walk(base, &entry.path(), result); }
            else { result.insert(entry.path().strip_prefix(base).unwrap().to_owned(), std::fs::read(entry.path()).unwrap()); }
        }
    }
    let mut result = BTreeMap::new(); walk(root, root, &mut result); result
}

#[test]
fn real_receipt_process_reopens_wallet_and_fences_stale_epoch() {
    let root = tempfile::tempdir().unwrap();
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(root.path().join("fixture-marker"), b"eidetic-retention-20261019\n").unwrap();
    let (first, old) = Host::start(root.path());
    let old_status = old.connected_status().clone();
    let old_epoch = old_status.custody_epoch.clone().unwrap();
    let clear = b"SYNTHETIC-RECEIPT-PAYLOAD";
    let (sealed, marker) = old.seal(&Hash::of(clear), clear).unwrap();
    assert_ne!(sealed, clear);
    let profiles = [("family", "Family", [0x51; 32])];
    let spec = djinn_custodian::fixture::Spec { passphrase: "public synthetic receipt fixture", profiles: &profiles,
        grants: serde_json::json!([]), consent: "deny", mechanism: None, agent: false };
    assert!(djinn_custodian::fixture::receipt(root.path(), spec, false).is_err(), "live process owns root lease");
    drop(old);
    first.stop();
    let state = root.path().join("custodian");
    let vault = std::fs::read(state.join("vault/vault.json")).unwrap();
    let wallet = files(&state.join("data"));
    let (second, fresh) = Host::start(root.path());
    let fresh_status = fresh.connected_status().clone();
    let fresh_epoch = fresh_status.custody_epoch.clone().unwrap();
    assert_ne!(old_epoch.instance, fresh_epoch.instance);
    assert_eq!(old_status.profile, fresh_status.profile);
    assert_eq!(old_status.persona_public_key, fresh_status.persona_public_key);
    assert_eq!(old_status.wallet_public_key, fresh_status.wallet_public_key);
    assert_eq!(fresh.unseal(&Hash::of(clear), &marker, &sealed).unwrap(), clear);
    let endpoint = root.path().join("app.sock").to_string_lossy().into_owned();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut client = CustodyClient::open_declared_at(&endpoint, AppId::new("turnstone"), super::turnstone_declaration()).await.unwrap();
        let request = PayloadRequest { profile: old_status.profile.unwrap(), master: old_status.persona_public_key.unwrap(),
            content_hash: Hash::of(clear), operation: PayloadOperation::Seal { bytes: PayloadBytes(clear.to_vec()) }, custody_epoch: Some(old_epoch) };
        assert!(client.call(CustodyCall::Payload(request)).await.is_err(), "old instance epoch cannot write after restart");
    });
    drop(fresh);
    second.stop();
    assert_eq!(std::fs::read(state.join("vault/vault.json")).unwrap(), vault);
    assert_eq!(files(&state.join("data")), wallet, "reopen must not reseed or change wallet files");
}

#[path = "../custodian_fixture/mod.rs"]
mod custodian_fixture;

#[test]
fn profile_roundtrip_fences_a_stale_payload_write() {
    use custodian_fixture::{TestCustodian, manage_grants};
    use mere_resident::custody_host::ResidentCustody;
    use dramatis::intents::SwitchProfileIntentV1;
    let host = TestCustodian::start("profile fence fixture", &[("work", [0x31; 32]), ("home", [0x32; 32])], manage_grants());
    let persona = PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(&host.data_root, persona, "profile fence fixture").unwrap();
    let remote = host.remote();
    remote.unlock_with_passphrase(b"profile fence fixture").unwrap();
    let old = remote.custody_epoch().unwrap().unwrap();
    let clear = b"SYNTHETIC-PROFILE-FENCE";
    let request = PayloadRequest { profile: ProfileId("work".into()), master: remote.master_public_key().to_bytes(),
        content_hash: Hash::of(clear), operation: PayloadOperation::Seal { bytes: PayloadBytes(clear.to_vec()) }, custody_epoch: Some(old.clone()) };
    let scope = b"profile-fence".to_vec();
    assert!(remote.payload(host.dir.path(), persona, request.clone(), scope.clone()).is_ok());
    remote.switch_profile(SwitchProfileIntentV1 { profile: "home".into() }).unwrap();
    remote.switch_profile(SwitchProfileIntentV1 { profile: "work".into() }).unwrap();
    assert_ne!(remote.custody_epoch().unwrap().unwrap(), old);
    assert!(remote.payload(host.dir.path(), persona, request, scope).is_err(), "profile roundtrip must fence the old writer");
}

#[test]
fn incomplete_wallet_is_refused_without_reseeding() {
    let root = tempfile::tempdir().unwrap();
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(root.path().join("fixture-marker"), b"eidetic-retention-20261019\n").unwrap();
    let (host, identity) = Host::start(root.path());
    drop(identity);
    host.stop();
    let data = root.path().join("custodian/data");
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xe1de_2026_1019));
    let missing = pandect::wallet_store::persona_wallet_path(&data, persona);
    let saved = std::fs::read(&missing).unwrap();
    std::fs::remove_file(&missing).unwrap();
    let before = files(&data);
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["retained_custody_receipt_host", "--ignored", "--exact", "--nocapture"])
        .env("EIDETIC_CUSTODY_FIXTURE_ROOT", root.path()).env("EIDETIC_CUSTODY_FIXTURE_ENDPOINT", root.path().join("app.sock"))
        .stdout(Stdio::null()).stderr(Stdio::null()).status().unwrap();
    assert!(!status.success(), "an incomplete wallet must be refused");
    assert!(!missing.exists(), "reopen must not recreate the missing manifest");
    assert_eq!(files(&data), before, "failed reopen must not change wallet files");
    std::fs::write(missing, saved).unwrap();
    let vault = root.path().join("custodian/vault/vault.json");
    std::fs::write(&vault, b"corrupt synthetic vault").unwrap();
    let before = files(&data);
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["retained_custody_receipt_host", "--ignored", "--exact", "--nocapture"])
        .env("EIDETIC_CUSTODY_FIXTURE_ROOT", root.path()).env("EIDETIC_CUSTODY_FIXTURE_ENDPOINT", root.path().join("app.sock"))
        .stdout(Stdio::null()).stderr(Stdio::null()).status().unwrap();
    assert!(!status.success(), "a corrupt vault must be refused");
    assert_eq!(std::fs::read(vault).unwrap(), b"corrupt synthetic vault");
    assert_eq!(files(&data), before, "failed vault open must not change wallet files");
}
