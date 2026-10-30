// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wallet at rest: its public records are disclosed over the wire, and an
//! existing wallet and catalog move into the custodian's data root with every
//! public key, epoch derivation, persona index and roster unchanged across a
//! cold reopen.

use std::path::{Path, PathBuf};

use custody_wire::contract::{Answer, EpochKeyRequest};
use custody_wire::Operation;
use personae::{IdentityProvider, InMemoryProvider, PersonaId};
use serde_json::json;

use super::h2_tests::{answer, base, config, data, vault};
use super::{Harness, code, djinn};
use crate::wallet::{adopt_files, carry_files, epoch_keys, public_records, retire_files, wallet_files};

const CONTEXTS: [(&str, bool, bool); 3] = [("mere.test.epoch.v1", false, false), ("mere.test.epoch.v1", true, false), ("mere.test.epoch.v1", false, true)];

/// Everything an owner could see change if a move altered the wallet.
#[derive(Debug, PartialEq)]
struct Golden {
    seed_public_key: [u8; 32],
    personas: Vec<String>,
    roster: Option<serde_json::Value>,
    epoch_keys: Vec<Vec<u8>>,
    secret_files: Vec<(PathBuf, Vec<u8>)>,
}

const SECRETS: [&str; 2] = ["identity/master.seed", "identity/local-device.json"];

fn golden(root: &Path, persona: PersonaId) -> Golden {
    let seed = castellan::custody::wallet::load_identity_seed_read_only(root).unwrap().unwrap();
    let requests: Vec<EpochKeyRequest> = CONTEXTS.iter().map(|(context, device_bound, public_only)| EpochKeyRequest { context: (*context).into(), device_bound: *device_bound, public_only: *public_only }).collect();
    let (personas, roster) = public_records(root).unwrap();
    let bridge = format!("personas/{}/private-epoch-bridge.json", persona.as_uuid());
    Golden {
        seed_public_key: InMemoryProvider::from_seed(seed).master_public_key().to_bytes(),
        personas,
        roster,
        epoch_keys: epoch_keys(root, persona, Some("this device"), &requests).unwrap().iter().map(|key| key.key.expose().to_vec()).collect(),
        secret_files: SECRETS.iter().map(|s| s.to_string()).chain([bridge]).map(|path| (PathBuf::from(&path), std::fs::read(root.join(path)).unwrap())).collect(),
    }
}

/// Djinn's data root as it stood: a two-persona wallet, a catalog file, and
/// files of Djinn's own beside them.
fn djinn_root() -> (tempfile::TempDir, PersonaId, PersonaId) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let (first, second) = (PersonaId::new(), PersonaId::new());
    castellan::custody::wallet::ensure_wallet_state(root, first, "fixture").unwrap();
    castellan::custody::wallet::ensure_wallet_state(root, second, "fixture").unwrap();
    let write = |relative: String, bytes: &[u8]| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    };
    write("castellan/profiles/work/store.json".into(), b"ciphertext the custodian derives keys for");
    write("identity/grants/device/scope.cert".into(), b"a granted certificate");
    for persona in [first, second] {
        write(format!("personas/{}/reservoir/store.redb", persona.as_uuid()), b"djinn's own reservoir");
    }
    write("settings/owner.json".into(), b"{}");
    (dir, first, second)
}

fn relative(root: &Path) -> Vec<String> {
    wallet_files(root).unwrap().iter().map(|p| p.to_string_lossy().replace('\\', "/")).collect()
}

#[test]
fn an_existing_wallet_moves_unchanged_and_nothing_secret_stays_in_djinns_root() {
    let (source, first, second) = djinn_root();
    let (before, before_second) = (golden(source.path(), first), golden(source.path(), second));
    let files = relative(source.path());
    assert!(files.contains(&"identity/master.seed".to_string()) && files.contains(&"castellan/profiles/work/store.json".to_string()));
    assert_eq!(before.personas.len(), 2);
    assert!(before.roster.is_some());

    let custodian = tempfile::tempdir().unwrap();
    let adopted = adopt_files(source.path(), custodian.path()).unwrap();
    assert!(adopted.left.is_empty(), "{:?}", adopted.left);
    assert_eq!(adopted.files.len(), files.len());

    // A cold reopen of the custodian's copy answers exactly as the original did.
    assert_eq!(golden(custodian.path(), first), before);
    assert_eq!(golden(custodian.path(), second), before_second);
    assert_eq!(relative(custodian.path()), files, "the same files at the same relative paths");

    // Nothing a wallet or catalog holds is left where Djinn reads, and Djinn's own files are.
    assert!(wallet_files(source.path()).unwrap().is_empty());
    assert!(!source.path().join("identity").exists() && !source.path().join("castellan").exists());
    assert!(public_records(source.path()).unwrap().0.is_empty());
    for persona in [first, second] {
        assert!(!source.path().join(format!("personas/{}/wallet.json", persona.as_uuid())).exists());
        assert_eq!(std::fs::read(source.path().join(format!("personas/{}/reservoir/store.redb", persona.as_uuid()))).unwrap(), b"djinn's own reservoir");
    }
    assert!(source.path().join("settings/owner.json").is_file());
}

#[cfg(unix)]
#[test]
fn the_files_in_the_custodians_root_are_private_to_it() {
    use std::os::unix::fs::PermissionsExt;
    let (source, _, _) = djinn_root();
    let custodian = tempfile::tempdir().unwrap();
    adopt_files(source.path(), custodian.path()).unwrap();
    let mode = |path: &str| std::fs::metadata(custodian.path().join(path)).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode("identity/master.seed"), 0o600);
    assert_eq!(mode("identity"), 0o700);
    assert_eq!(mode("castellan/profiles/work"), 0o700);
}

#[test]
fn an_adoption_that_cannot_proceed_changes_nothing() {
    let (source, first, _) = djinn_root();
    let before = golden(source.path(), first);
    let files = relative(source.path());

    // Nothing to adopt.
    let empty = tempfile::tempdir().unwrap();
    assert!(adopt_files(empty.path(), tempfile::tempdir().unwrap().path()).unwrap_err().contains("no wallet"));

    // A custodian root that already holds a wallet is never merged into.
    let occupied = tempfile::tempdir().unwrap();
    castellan::custody::wallet::ensure_wallet_state(occupied.path(), PersonaId::new(), "other").unwrap();
    let held = relative(occupied.path());
    assert!(adopt_files(source.path(), occupied.path()).unwrap_err().contains("already holds"));
    assert_eq!(relative(occupied.path()), held);

    // The two roots may not overlap.
    assert!(adopt_files(source.path(), &source.path().join("nested")).unwrap_err().contains("overlap"));
    assert!(adopt_files(source.path(), source.path()).unwrap_err().contains("already holds"));

    // A copy that cannot be made is removed again, and the source stays whole.
    let blocked = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(blocked.path().join("identity/master.seed")).unwrap();
    assert!(adopt_files(source.path(), blocked.path()).unwrap_err().contains("copy was not made"));
    assert!(wallet_files(blocked.path()).unwrap().is_empty());
    assert!(std::fs::read_dir(blocked.path().join("identity/master.seed")).unwrap().next().is_none());

    assert_eq!(relative(source.path()), files);
    assert_eq!(golden(source.path(), first), before);
}

#[cfg(unix)]
#[test]
fn a_symlink_among_the_wallet_files_is_refused() {
    let (source, first, _) = djinn_root();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("target"), b"elsewhere").unwrap();
    std::os::unix::fs::symlink(outside.path().join("target"), source.path().join("identity/grants/link")).unwrap();
    let custodian = tempfile::tempdir().unwrap();
    assert!(adopt_files(source.path(), custodian.path()).unwrap_err().contains("not a regular file"));
    assert!(wallet_files(custodian.path()).unwrap().is_empty());
    golden(source.path(), first);
}

#[test]
fn a_source_copy_that_changed_after_it_was_carried_is_not_removed() {
    let (source, first, _) = djinn_root();
    let custodian = tempfile::tempdir().unwrap();
    let files = carry_files(source.path(), custodian.path()).unwrap();
    assert_eq!(files, wallet_files(source.path()).unwrap(), "carrying only reads the source");
    let seed = source.path().join("identity/master.seed");
    let changed = [0x5a; 32];
    std::fs::write(&seed, changed).unwrap();
    let left = retire_files(source.path(), custodian.path(), &files);
    assert_eq!(left.len(), 1, "{left:?}");
    assert_eq!(left[0].0, Path::new("identity/master.seed"));
    assert_eq!(std::fs::read(&seed).unwrap(), changed, "the changed file was not deleted");
    assert_ne!(golden(custodian.path(), first).seed_public_key, InMemoryProvider::from_seed(changed).master_public_key().to_bytes());
}

#[test]
fn the_wallets_public_records_are_disclosed_while_locked_and_only_to_a_snapshot_grant() {
    let dir = vault();
    let persona = PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(&data(&dir), persona, "fixture").unwrap();
    let (personas, roster) = public_records(&data(&dir)).unwrap();
    assert_eq!(personas, vec![persona.as_uuid().to_string()]);

    let mut bare = Harness::new(config(&dir, "deny", base(json!([]))));
    let session = bare.open(djinn());
    assert_eq!(code(&bare.outcome(session, Operation::WalletPublic {})), "operation-denied");
    drop(bare);

    let mut h = Harness::new(config(&dir, "deny", base(json!([{"operation": "snapshot"}]))));
    let session = h.open(djinn());
    let Answer::WalletPublic { personas: disclosed, roster: disclosed_roster } = answer(h.outcome(session, Operation::WalletPublic {})) else { panic!() };
    assert_eq!((disclosed, disclosed_roster.clone()), (personas, roster), "answered while locked");
    let text = serde_json::to_string(&disclosed_roster).unwrap();
    let seed = castellan::custody::wallet::load_identity_seed_read_only(&data(&dir)).unwrap().unwrap();
    assert!(!text.contains(&hex(&seed)), "the roster holds no seed");
    for file in SECRETS {
        if let Ok(bytes) = std::fs::read(data(&dir).join(file)) {
            assert!(!text.contains(&hex(&bytes)), "{file} is not in the answer");
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
