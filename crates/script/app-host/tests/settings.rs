// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Settings to a component (capability door K1, D22): the guest runs the
//! shared call script through its imports, served by the same
//! `SettingsServer` a native channel uses, and its transcript is the native
//! one. Values cross Wasmtime unchanged.

use std::path::PathBuf;

use settings_wit::bindings::{Client, Handler};
use settings_wit::testkit::{self, CALLS, FixtureGate, FixtureOwner, assert_outcomes, run};
use settings_wit::{MAX_FRAME, SettingsServer, serve};
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    path: ["../wit", "guest-settings/wit"],
    world: "mere:settings-test/settings-consumer",
});

use mere::script::settings as ws;

settings_wit::bridge!(crate::mere::script::settings);

type Server = SettingsServer<FixtureOwner, FixtureGate>;

fn server() -> Server {
    SettingsServer::new(FixtureOwner::default(), FixtureGate::default())
}

fn guest_component() -> PathBuf {
    if let Some(path) = std::env::var_os("APP_HOST_SETTINGS_GUEST_WASM") {
        return PathBuf::from(path);
    }
    let path = PathBuf::from("guest-settings/target/wasm32-wasip2/release/settings_guest.wasm");
    assert!(
        path.exists(),
        "guest component missing at {}. Build it with:\n  \
         cd crates/script/app-host/guest-settings && cargo build --target wasm32-wasip2 --release",
        path.display()
    );
    path
}

struct Host {
    wasi: WasiCtx,
    table: ResourceTable,
    server: Server,
}

impl WasiView for Host {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl ws::Host for Host {
    fn snapshot(&mut self, reference: String) -> Result<ws::SettingsSnapshot, ws::Failure> {
        Handler::snapshot(&mut self.server, reference)
            .map(bridge::theirs::snapshot)
            .map_err(bridge::theirs::failure)
    }
    fn write(&mut self, request: ws::WriteRequest) -> ws::WriteOutcome {
        let outcome = Handler::write(&mut self.server, bridge::wire::request(request));
        bridge::theirs::outcome(outcome)
    }
}

fn load() -> (Store<Host>, SettingsConsumer) {
    let engine = Engine::default();
    let component = Component::from_file(&engine, guest_component()).expect("guest loads");
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    ws::add_to_linker::<Host, HasSelf<Host>>(&mut linker, |h| h).unwrap();
    let mut store = Store::new(
        &engine,
        Host {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            server: server(),
        },
    );
    let bindings = SettingsConsumer::instantiate(&mut store, &component, &linker).unwrap();
    (store, bindings)
}

#[test]
fn the_guest_matches_the_native_transcript() {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    let native = std::thread::spawn(move || {
        let mut s = server();
        serve(&mut s, end).unwrap();
        s
    });
    let reference = run(&mut Client(client));
    let native = native.join().unwrap();
    assert_outcomes(&reference);

    let (mut store, guest) = load();
    let transcript = guest.call_run(&mut store).unwrap();
    assert_eq!(transcript, reference);
    // The same questions reached the gate, and the owner moved alike.
    let host = store.data();
    assert_eq!(host.server.gate.asked.len(), CALLS);
    assert_eq!(host.server.gate.asked, native.gate.asked);
    assert_eq!(
        format!("{:?}", host.server.provider),
        format!("{:?}", native.provider)
    );
}

#[test]
fn values_cross_wasmtime_unchanged() {
    let (mut store, guest) = load();

    let snapshot = bridge::theirs::snapshot(testkit::edge_snapshot());
    let back = guest.call_echo_snapshot(&mut store, &snapshot).unwrap();
    assert_eq!(
        format!("{:?}", bridge::wire::snapshot(back)),
        format!("{:?}", testkit::edge_snapshot())
    );

    for outcome in testkit::edge_outcomes() {
        let back = guest
            .call_echo_outcome(&mut store, &bridge::theirs::outcome(outcome.clone()))
            .unwrap();
        assert_eq!(
            format!("{:?}", bridge::wire::outcome(back)),
            format!("{outcome:?}")
        );
    }

    for failure in testkit::edge_failures() {
        let back = guest
            .call_echo_failure(&mut store, &bridge::theirs::failure(failure.clone()))
            .unwrap();
        assert_eq!(bridge::wire::failure(back), failure);
    }
}
