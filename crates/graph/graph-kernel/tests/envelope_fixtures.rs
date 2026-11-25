// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The stored journal fixture (`tests/fixtures/wire/journal.json`, written by
//! the kernel before the engine split) read through the engine's journal
//! envelope (engine split D242): the JSON journal, muniment's v1 and v2
//! digests and the CBOR wire give the same bytes either way, and the two
//! resource writes route to the engine's half.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use kernel::graph::{AttributedDelta, CapturedDelta};
use mere::model::{Attributed, JournalDelta};
use muniment::Journal;
use muniment::journal::DigestChain;

type Entry = Attributed<JournalDelta<CapturedDelta>>;

fn read(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wire")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn kernel_entries() -> Vec<AttributedDelta> {
    serde_json::from_slice(&read("journal.json")).unwrap()
}

fn enveloped(entries: &[AttributedDelta]) -> Vec<Entry> {
    entries
        .iter()
        .map(|entry| Attributed {
            author: entry.author.clone(),
            delta: entry.delta.clone().into(),
        })
        .collect()
}

#[test]
fn journal_json_is_byte_identical_through_the_envelope() {
    let fixture = read("journal.json");
    let kernel = kernel_entries();
    let converted = enveloped(&kernel);
    assert_eq!(serde_json::to_vec(&converted).unwrap(), fixture);
    let decoded: Vec<Entry> = serde_json::from_slice(&fixture).unwrap();
    assert_eq!(decoded, converted);
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), fixture);
    // Back to the kernel enum, entry for entry.
    let back: Vec<CapturedDelta> = decoded.into_iter().map(|e| e.delta.into()).collect();
    let original: Vec<CapturedDelta> = kernel.into_iter().map(|e| e.delta).collect();
    assert_eq!(back, original);
}

#[test]
fn the_two_resource_writes_route_to_the_engine() {
    let decoded: Vec<Entry> = serde_json::from_slice(&read("journal.json")).unwrap();
    let resource = decoded
        .iter()
        .filter(|e| matches!(e.delta, JournalDelta::Resource(_)))
        .count();
    let shown = decoded
        .iter()
        .filter(|e| {
            matches!(
                e.delta,
                JournalDelta::Surface(CapturedDelta::ReplaySetShownResourceById { .. })
            )
        })
        .count();
    let misrouted = decoded.iter().any(|e| {
        matches!(
            e.delta,
            JournalDelta::Surface(
                CapturedDelta::ReplaySetResourceRecordById { .. }
                    | CapturedDelta::ReplaySetResourceEdgesByIds { .. }
            )
        )
    });
    // The fixture holds 7 record and 4 resource-edge writes, and 3 bindings.
    assert_eq!(
        (decoded.len(), resource, shown, misrouted),
        (24, 11, 3, false)
    );
}

#[test]
fn v1_and_v2_digests_are_identical_at_every_prefix() {
    let kernel = kernel_entries();
    let converted = enveloped(&kernel);
    let (mut old, mut new) = (DigestChain::new(), DigestChain::new());
    let (mut old_log, mut new_log) = (Journal::new(), Journal::new());
    for (a, b) in kernel.iter().zip(&converted) {
        assert_eq!(old.push(a).unwrap(), new.push(b).unwrap());
        old_log.append(a.clone());
        new_log.append(b.clone());
    }
    for len in 0..=kernel.len() {
        assert_eq!(
            old_log.prefix_digest(len).unwrap(),
            new_log.prefix_digest(len).unwrap()
        );
    }
    // Control: the digest sees a change of author.
    let mut moved = converted[0].clone();
    moved.author = kernel::graph::Author::person("someone-else");
    assert_ne!(
        DigestChain::new().push(&moved).unwrap(),
        DigestChain::new().push(&converted[0]).unwrap()
    );
}

#[test]
fn cbor_wire_is_byte_identical_and_reads_back() {
    let kernel = kernel_entries();
    let converted = enveloped(&kernel);
    let old = cbor4ii::serde::to_vec(Vec::new(), &kernel).unwrap();
    let new = cbor4ii::serde::to_vec(Vec::new(), &converted).unwrap();
    assert_eq!(old, new);
    let back: Vec<Entry> = cbor4ii::serde::from_slice(&new).unwrap();
    assert_eq!(back, converted);
}

#[test]
fn a_positional_journal_does_not_read_as_the_envelope() {
    let bytes = read("journal.postcard");
    assert!(postcard::from_bytes::<Vec<Entry>>(&bytes).is_err());
    // Control: the same bytes are the kernel's own journal.
    assert_eq!(
        postcard::from_bytes::<Vec<AttributedDelta>>(&bytes).unwrap(),
        kernel_entries()
    );
}
