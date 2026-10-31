// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A replace never tears (ruling 48), metadata reads never open a payload
//! (ruling 39), and the Secret Service keeps no records of its own (ruling 20).

use std::path::{Path, PathBuf};

use super::*;
use crate::items::WriteStep;
use crate::resident::CastellanResident;
use tempfile::tempdir;

fn claim(root: &Path) -> CastellanResident {
    CastellanResident::claim(
        root.join("records"),
        [0xb1; 32],
        root.join("freshness"),
        [0xb2; 32],
    )
    .unwrap()
}

fn attributes() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("application".into(), "turnstone".into()),
        ("account".into(), "mark".into()),
    ])
}

fn files(directory: &Path) -> Vec<PathBuf> {
    if directory.is_file() {
        return vec![directory.to_path_buf()];
    }
    std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .flat_map(|entry| files(&entry.path()))
        .collect()
}

/// Live payload records on disk. A resident's delete leaves a sealed
/// tombstone, which does not count.
fn payloads(root: &Path, persona: PersonaId) -> usize {
    files(
        &root
            .join("records/castellan/items/v1")
            .join(persona.as_uuid().to_string())
            .join("payloads"),
    )
    .iter()
    .filter(|path| {
        let envelope: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        envelope["deleted"] == false
    })
    .count()
}

/// One item: "Old label", `text/old`, `old bytes`.
fn seed(store: &SecretServiceStore) -> SecretItem {
    let collection = store.ensure_default_collection("Castellan", 1).unwrap();
    store
        .create_item(NewSecretItem {
            collection: collection.id,
            label: "Old label".into(),
            attributes: attributes(),
            secret: b"old bytes".to_vec(),
            content_type: "text/old".into(),
            replace: false,
            unix_secs: 1,
        })
        .unwrap()
}

/// What a client sees of an item: label, content type and bytes.
fn seen(store: &SecretServiceStore, id: SecretItemId) -> (String, String, Vec<u8>) {
    let secret = store.secret(id).unwrap();
    (
        store.item(id).unwrap().label,
        secret.content_type.clone(),
        secret.bytes.to_vec(),
    )
}

fn view(label: &str, content_type: &str, bytes: &[u8]) -> (String, String, Vec<u8>) {
    (label.into(), content_type.into(), bytes.to_vec())
}

/// The two ways a payload is replaced: an exact-attribute replace, which also
/// relabels, and SetSecret, which keeps the label.
#[derive(Clone, Copy, Debug)]
enum Way {
    Replace,
    SetSecret,
}

fn replace(
    store: &SecretServiceStore,
    item: &SecretItem,
    way: Way,
) -> Result<(), SecretServiceError> {
    match way {
        Way::Replace => store
            .create_item(NewSecretItem {
                collection: item.collection,
                label: "New label".into(),
                attributes: attributes(),
                secret: b"new bytes".to_vec(),
                content_type: "text/new".into(),
                replace: true,
                unix_secs: 2,
            })
            .map(|_| ()),
        Way::SetSecret => store.set_secret(item.id, b"new bytes".to_vec(), "text/new", 2),
    }
}

fn after(way: Way) -> (String, String, Vec<u8>) {
    match way {
        Way::Replace => view("New label", "text/new", b"new bytes"),
        Way::SetSecret => view("Old label", "text/new", b"new bytes"),
    }
}

/// Run a replace that stops as a crash would after `step`; return what a
/// fresh view then sees, and how many payload records are on disk.
fn crashed(way: Way, step: WriteStep) -> (SecretItem, (String, String, Vec<u8>), usize) {
    let dir = tempdir().unwrap();
    let resident = claim(dir.path());
    let persona = PersonaId::new();
    let store = resident.secret_service(persona, SecretServiceLimits::default());
    let item = seed(&store);
    let crashing = SecretServiceStore::new(
        resident.items(persona).crashing_after(step),
        SecretServiceLimits::default(),
    );
    let error = replace(&crashing, &item, way).unwrap_err();
    assert!(error.to_string().contains("simulated crash"), "{error}");
    (
        item.clone(),
        seen(&store, item.id),
        payloads(dir.path(), persona),
    )
}

#[test]
fn a_replace_stopped_before_its_commit_leaves_the_old_item_whole() {
    for way in [Way::Replace, Way::SetSecret] {
        let (item, seen, payloads) = crashed(way, WriteStep::Payloads);
        assert_eq!(seen, view("Old label", "text/old", b"old bytes"), "{way:?}");
        // The new payload is an orphan under a credential id nothing names.
        assert_eq!(payloads, 2, "{way:?}");
        assert_eq!(seen.0, item.label);
    }
}

#[test]
fn a_replace_stopped_after_its_commit_shows_the_new_item_whole() {
    for way in [Way::Replace, Way::SetSecret] {
        let (_, seen, payloads) = crashed(way, WriteStep::Metadata);
        assert_eq!(seen, after(way), "{way:?}");
        // The old payload is the orphan now.
        assert_eq!(payloads, 2, "{way:?}");
    }
}

#[test]
fn a_whole_replace_keeps_the_item_and_leaves_one_payload() {
    for way in [Way::Replace, Way::SetSecret] {
        let dir = tempdir().unwrap();
        let resident = claim(dir.path());
        let persona = PersonaId::new();
        let store = resident.secret_service(persona, SecretServiceLimits::default());
        let item = seed(&store);
        replace(&store, &item, way).unwrap();
        assert_eq!(seen(&store, item.id), after(way), "{way:?}");
        assert_eq!(store.items(item.collection).unwrap().len(), 1, "{way:?}");
        assert_eq!(store.item(item.id).unwrap().created, 1, "{way:?}");
        assert_eq!(store.item(item.id).unwrap().modified, 2, "{way:?}");
        assert_eq!(payloads(dir.path(), persona), 1, "{way:?}");
    }
}

#[test]
fn listing_search_and_property_reads_never_open_a_payload() {
    let dir = tempdir().unwrap();
    let resident = claim(dir.path());
    let persona = PersonaId::new();
    let store = resident.secret_service(persona, SecretServiceLimits::default());
    let item = seed(&store);
    // A well-formed envelope whose ciphertext cannot be opened, over every
    // payload.
    let base = dir.path().join("records/castellan/items/v1");
    let sealed = files(&base.join(persona.as_uuid().to_string()).join("payloads"));
    assert_eq!(sealed.len(), 1);
    for payload in &sealed {
        std::fs::write(
            payload,
            r#"{"version":2,"generation":1,"deleted":false,"nonce":[0,0,0,0,0,0,0,0,0,0,0,0],"ciphertext":[1,2,3]}"#,
        )
        .unwrap();
    }

    let collection = store.collection(item.collection).unwrap();
    assert_eq!(store.collections().unwrap(), vec![collection]);
    assert_eq!(store.read_alias("default").unwrap(), Some(item.collection));
    assert_eq!(store.items(item.collection).unwrap(), vec![item.clone()]);
    assert_eq!(store.item(item.id).unwrap(), item);
    assert_eq!(store.search(&attributes()).unwrap(), vec![item.clone()]);
    store.set_item_label(item.id, "Relabelled", 3).unwrap();
    store.set_item_attributes(item.id, attributes(), 3).unwrap();
    assert_eq!(store.item(item.id).unwrap().label, "Relabelled");

    // Only releasing the secret opens its payload, and that fails.
    let error = match store.secret(item.id) {
        Err(error) => error,
        Ok(_) => panic!("an unopenable payload was released"),
    };
    assert!(
        error.to_string().contains("decrypt sealed record"),
        "{error}"
    );
}

#[test]
fn the_secret_service_keeps_only_item_store_records() {
    let dir = tempdir().unwrap();
    let resident = claim(dir.path());
    let persona = PersonaId::new();
    let store = resident.secret_service(persona, SecretServiceLimits::default());
    let item = seed(&store);
    store
        .create_collection("Second", Some("second"), 2)
        .unwrap();

    let records = dir.path().join("records").canonicalize().unwrap();
    let mut written: Vec<_> = files(&records)
        .into_iter()
        .map(|path| path.strip_prefix(&records).unwrap().to_path_buf())
        .filter(|path| path != Path::new(".personae-authority.lock"))
        .collect();
    written.sort();
    let base = Path::new("castellan/items/v1").join(persona.as_uuid().to_string());
    let credential = resident
        .items(persona)
        .get(item.id.0)
        .unwrap()
        .unwrap()
        .credentials[0]
        .id;
    assert_eq!(
        written,
        vec![
            base.join("index.json"),
            base.join("items").join(format!("{}.json", item.id.0)),
            base.join("payloads").join(format!("{credential}.json")),
        ]
    );
}
