// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `block-guest` world end to end over a real component built in Mere: the five calls,
//! the refusals a guest returns as values, the `caps` window, and the watchdog trapping a
//! turn that spins.

use std::path::PathBuf;
use std::time::Duration;

use block_host::{BlockGuestHost, Watchdog, guarded_engine, wit};
use vector_sheet::{FillRule, Op, Seg, VectorSheet};
use wasmtime::StoreLimits;

fn guest_component() -> PathBuf {
    if let Some(path) = std::env::var_os("BLOCK_HOST_GUEST_WASM") {
        return PathBuf::from(path);
    }
    let path = PathBuf::from("guest/target/wasm32-wasip2/release/block_fixture_guest.wasm");
    assert!(
        path.exists(),
        "guest component missing at {}. Build it with:\n  \
         cd crates/script/block-host/guest && cargo build --locked --target wasm32-wasip2 --release",
        path.display()
    );
    path
}

fn block(selected: Vec<u32>) -> wit::Block {
    wit::Block {
        class: "fixture".into(),
        body: "one\ntwo\n".into(),
        attrs: vec![wit::Attr { key: "class".into(), value: "fixture".into() }],
        revision: "r1".into(),
        selected,
        pattern: String::new(),
    }
}

fn host(deadline: Option<u64>) -> (BlockGuestHost, Watchdog) {
    let engine = guarded_engine().unwrap();
    let dog = Watchdog::start(engine.clone(), Duration::from_millis(10));
    let host = BlockGuestHost::attach_path_blocking(
        &engine,
        &guest_component(),
        vec!["power:block-adapter".into()],
        StoreLimits::default(),
        deadline,
    )
    .unwrap();
    (host, dog)
}

#[test]
fn describe_claims_one_class_with_its_commands_and_views() {
    let (mut host, _dog) = host(Some(100));
    let claim = host.describe().unwrap();
    assert_eq!(claim.class, "fixture");
    assert_eq!(claim.source_key, "fixture");
    assert_eq!(claim.commands.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["mark", "stamp", "echo", "spin"]);
    assert_eq!(claim.views.len(), 1);
    assert_eq!(claim.example.unwrap().title, "Fixture block");
}

#[test]
fn choices_apply_and_run_cross_as_copied_values_and_refusals_are_values() {
    let (mut host, _dog) = host(Some(100));
    let b = block(vec![0, 1]);
    let rows = host.rows(&b).unwrap().unwrap();
    assert_eq!(rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["one", "two"]);
    let changes = host.choices(&b, "mark").unwrap().unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[1].label, "Row 2");
    let attrs = host.apply(&b, "mark", &changes).unwrap().unwrap();
    assert!(attrs.iter().any(|a| a.key == "fixture" && a.value == "marked:2"));
    assert!(attrs.iter().any(|a| a.key == "class"), "the whole map comes back");
    let stamped = host.run(&b, "stamp", "s1").unwrap().unwrap();
    assert!(matches!(&stamped, wit::RunResult::Attrs(a) if a.iter().any(|x| x.key == "stamp" && x.value == "s1")));
    let echoed = host.run(&b, "echo", "e").unwrap().unwrap();
    assert!(matches!(echoed, wit::RunResult::Text(t) if t == "8:e"));
    assert_eq!(host.choices(&b, "nope").unwrap().unwrap_err(), "Unknown command nope");
}

#[test]
fn view_returns_a_scene_naming_regions_and_a_sheet_with_a_compound_path() {
    let (mut host, _dog) = host(Some(100));
    let view = host
        .view(&block(vec![]), &wit::ViewAsk { mode: "plain".into(), width: 200.0, selected: Some((0, 1)) })
        .unwrap()
        .unwrap();
    assert_eq!(view.scene[0], 1, "version byte");
    let scene: sceno::Scene = ciborium_from(&view.scene[1..]);
    assert_eq!(scene.regions.len(), 2);
    assert_eq!(scene.regions[0].label.as_deref(), Some("Event 1 of fixture"));
    let sheet = VectorSheet::decode(&view.sheet).unwrap();
    assert_eq!(view.width, sheet.width);
    let Op::Fill { path, rule, .. } = &sheet.ops[0] else { panic!("compound fill first") };
    assert_eq!(*rule, FillRule::NonZero);
    assert_eq!(path.segs.iter().filter(|s| matches!(s, Seg::Move(..))).count(), 2);
    assert_eq!(view.description, "fixture view plain");
    assert!(host
        .view(&block(vec![]), &wit::ViewAsk { mode: "bogus".into(), width: 1.0, selected: None })
        .unwrap()
        .is_err());
}

fn ciborium_from<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    // sceno derives serde; the guest wrote CBOR with ciborium. serde_json is not involved.
    let value: serde_cbor_shim::Value = serde_cbor_shim::decode(bytes);
    serde_cbor_shim::into(value)
}

mod serde_cbor_shim {
    //! A test-local bridge: decode CBOR through ciborium's `Value`, then into `T`.
    pub use ciborium::value::Value;
    pub fn decode(bytes: &[u8]) -> Value {
        ciborium::from_reader(bytes).expect("CBOR scene")
    }
    pub fn into<T: serde::de::DeserializeOwned>(value: Value) -> T {
        value.deserialized().expect("a sceno scene")
    }
}

#[test]
fn a_spinning_turn_hits_its_deadline_and_traps_without_taking_the_host() {
    let (mut host, _dog) = host(Some(5));
    let started = std::time::Instant::now();
    let error = host.run(&block(vec![]), "spin", "").unwrap_err();
    assert!(started.elapsed() < Duration::from_secs(5), "the watchdog ended it");
    assert!(
        format!("{error:?}").to_lowercase().contains("interrupt") || format!("{error:?}").contains("epoch"),
        "{error:?}"
    );
}

#[test]
fn an_oversize_component_is_refused() {
    let engine = guarded_engine().unwrap();
    let too_big = vec![0u8; block_host::MAX_COMPONENT_BYTES + 1];
    let error = BlockGuestHost::attach_bytes_blocking(&engine, &too_big, vec![], StoreLimits::default(), None)
        .err()
        .expect("refused");
    assert!(error.to_string().contains("exceeds"));
}
