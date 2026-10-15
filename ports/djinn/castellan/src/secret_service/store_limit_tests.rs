// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One refusal per `SecretServiceLimits` field the store enforces, each
//! beside its control: the value at the limit is accepted, one past it is
//! refused, and a refused value leaves nothing stored.

use super::*;
use crate::resident::CastellanResident;
use tempfile::tempdir;

/// Every limit at its default except the one under test.
fn limits(tight: impl FnOnce(&mut SecretServiceLimits)) -> SecretServiceLimits {
    let mut limits = SecretServiceLimits::default();
    tight(&mut limits);
    limits
}

fn open(root: &std::path::Path, limits: SecretServiceLimits) -> SecretServiceStore {
    CastellanResident::claim(
        root.join("records"),
        [0xa1; 32],
        root.join("freshness"),
        [0xa2; 32],
    )
    .unwrap()
    .secret_service(PersonaId::new(), limits)
}

fn one(key: &str, value: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(key.to_string(), value.to_string())])
}

/// A small valid request, then `shape` applied to it.
fn request(
    collection: SecretCollectionId,
    id: &str,
    shape: impl FnOnce(&mut NewSecretItem),
) -> NewSecretItem {
    let mut request = NewSecretItem {
        collection,
        label: "Label".into(),
        attributes: one("id", id),
        secret: b"secret".to_vec(),
        content_type: "text/plain".into(),
        replace: false,
        unix_secs: 1,
    };
    shape(&mut request);
    request
}

fn plain(collection: SecretCollectionId, id: &str) -> NewSecretItem {
    request(collection, id, |_| {})
}

fn refused<T>(result: Result<T, SecretServiceError>, limit: &str) {
    match result {
        Err(SecretServiceError::Limit(found)) => assert_eq!(found, limit),
        Err(other) => panic!("expected the {limit} limit, got {other}"),
        Ok(_) => panic!("a value past the {limit} limit was accepted"),
    }
}

#[test]
fn the_collection_limit_refuses_one_past_it() {
    let dir = tempdir().unwrap();
    let store = open(dir.path(), limits(|l| l.max_collections = 2));
    store.ensure_default_collection("Default", 1).unwrap();
    // Control: the second collection is at the limit.
    store.create_collection("Second", None, 1).unwrap();
    refused(
        store.create_collection("Third", None, 1),
        "collections per persona",
    );
    assert_eq!(store.collections().unwrap().len(), 2);
    // An existing alias names a collection rather than adding one.
    store
        .create_collection("Relabelled", Some("default"), 2)
        .unwrap();
    assert_eq!(store.collections().unwrap().len(), 2);
}

#[test]
fn the_item_limit_refuses_one_past_it_but_still_replaces() {
    let dir = tempdir().unwrap();
    let store = open(dir.path(), limits(|l| l.max_items_per_collection = 2));
    let full = store.ensure_default_collection("Default", 1).unwrap().id;
    store.create_item(plain(full, "a")).unwrap();
    // Control: the second item is at the limit.
    store.create_item(plain(full, "b")).unwrap();
    refused(store.create_item(plain(full, "c")), "items per collection");
    assert_eq!(store.items(full).unwrap().len(), 2);
    // A replace adds nothing, so a full collection still takes one.
    let replaced = store
        .create_item(request(full, "b", |r| r.replace = true))
        .unwrap();
    assert_eq!(store.items(full).unwrap().len(), 2);
    assert!(store.items(full).unwrap().contains(&replaced));
    // The limit is per collection.
    let other = store.create_collection("Other", None, 1).unwrap().id;
    store.create_item(plain(other, "c")).unwrap();
}

#[test]
fn the_attribute_count_limit_refuses_one_past_it() {
    let dir = tempdir().unwrap();
    let store = open(dir.path(), limits(|l| l.max_attributes = 2));
    let collection = store.ensure_default_collection("Default", 1).unwrap().id;
    let two = BTreeMap::from([("a".to_string(), "1".to_string()), ("b".into(), "2".into())]);
    let mut three = two.clone();
    three.insert("c".into(), "3".into());
    // Control: two attributes are at the limit, by every way in.
    let item = store
        .create_item(request(collection, "", |r| r.attributes = two.clone()))
        .unwrap();
    store.set_item_attributes(item.id, two.clone(), 2).unwrap();
    assert_eq!(
        store.search(&two).unwrap(),
        vec![store.item(item.id).unwrap()]
    );

    refused(
        store.create_item(request(collection, "", |r| r.attributes = three.clone())),
        "attributes per item",
    );
    refused(
        store.set_item_attributes(item.id, three.clone(), 3),
        "attributes per item",
    );
    refused(store.search(&three), "attributes per item");
    assert_eq!(store.items(collection).unwrap().len(), 1);
    assert_eq!(store.item(item.id).unwrap().attributes, two);
}

#[test]
fn the_name_limit_counts_bytes_for_every_name() {
    let dir = tempdir().unwrap();
    let store = open(dir.path(), limits(|l| l.max_name_bytes = 10));
    // Five two-byte characters are ten bytes; six characters, one more byte.
    let (at, past) = ("ééééé", "éééééa");
    let collection = store.ensure_default_collection(at, 1).unwrap().id;
    let item = store
        .create_item(request(collection, "", |r| {
            r.label = at.into();
            r.content_type = at.into();
            r.attributes = one(at, "v");
        }))
        .unwrap();
    store.set_item_label(item.id, at, 2).unwrap();
    store.set_collection_label(collection, at, 2).unwrap();
    store.set_alias("abcdefghij", Some(collection)).unwrap();

    refused(store.ensure_default_collection(past, 1), "collection label");
    refused(store.create_collection(past, None, 1), "collection label");
    refused(
        store.set_collection_label(collection, past, 3),
        "collection label",
    );
    refused(
        store.set_alias("abcdefghijk", Some(collection)),
        "collection alias",
    );
    refused(
        store.create_item(request(collection, "label", |r| r.label = past.into())),
        "item label",
    );
    refused(store.set_item_label(item.id, past, 3), "item label");
    refused(
        store.create_item(request(collection, "type", |r| {
            r.content_type = past.into();
        })),
        "content type",
    );
    refused(
        store.create_item(request(collection, "", |r| r.attributes = one(past, "v"))),
        "attribute key",
    );
    assert_eq!(
        store.items(collection).unwrap(),
        vec![store.item(item.id).unwrap()]
    );
    assert_eq!(store.item(item.id).unwrap().label, at);
    assert_eq!(store.read_alias("abcdefghijk").unwrap(), None);
}

#[test]
fn the_attribute_value_limit_refuses_one_past_it() {
    let dir = tempdir().unwrap();
    let store = open(dir.path(), limits(|l| l.max_attribute_value_bytes = 8));
    let collection = store.ensure_default_collection("Default", 1).unwrap().id;
    let (at, past) = ("éééé", "éééée");
    // Control: eight bytes are at the limit; nine are one past.
    let item = store.create_item(plain(collection, at)).unwrap();
    refused(
        store.create_item(plain(collection, past)),
        "attribute value",
    );
    refused(
        store.set_item_attributes(item.id, one("id", past), 2),
        "attribute value",
    );
    refused(store.search(&one("id", past)), "attribute value");
    assert_eq!(store.items(collection).unwrap(), vec![item]);
}

#[test]
fn the_secret_limit_refuses_one_byte_past_it() {
    let dir = tempdir().unwrap();
    let store = open(dir.path(), limits(|l| l.max_secret_bytes = 8));
    let collection = store.ensure_default_collection("Default", 1).unwrap().id;
    // Control: eight bytes are at the limit, created and set.
    let item = store
        .create_item(request(collection, "a", |r| {
            r.secret = b"12345678".to_vec()
        }))
        .unwrap();
    store
        .set_secret(item.id, b"87654321".to_vec(), "text/plain", 2)
        .unwrap();

    refused(
        store.create_item(request(collection, "b", |r| {
            r.secret = b"123456789".to_vec();
        })),
        "secret bytes",
    );
    refused(
        store.create_item(request(collection, "a", |r| {
            r.secret = b"123456789".to_vec();
            r.replace = true;
        })),
        "secret bytes",
    );
    refused(
        store.set_secret(item.id, b"123456789".to_vec(), "text/plain", 3),
        "secret bytes",
    );
    assert_eq!(store.items(collection).unwrap().len(), 1);
    assert_eq!(store.secret(item.id).unwrap().bytes.as_slice(), b"87654321");
}
