// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Settings to an app participant (capability door K1, D22): an
//! `app-settings` component attached through `AppScript` runs the shared
//! call script on its imports, served by the same `SettingsServer` a native
//! channel uses, and its transcript is the native one. Each participant's
//! calls go to its own gate. Values cross Wasmtime unchanged.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use app_host::{ActionSink, AppScript, Refusal, Settings};
use settings_wit::bindings::{Client, Handler};
use settings_wit::testkit::{
    self, CALLS, FixtureGate, FixtureOwner, assert_outcomes, from_log, run,
};
use settings_wit::{Access, MAX_FRAME, SettingsGate, SettingsServer, serve};
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Engine, Store, StoreLimits};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use workbench::SettingsRef;

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

/// A gate the test can read after the participant has used it.
#[derive(Clone, Default)]
struct Shared(Arc<Mutex<FixtureGate>>);

impl SettingsGate for Shared {
    fn admit(&mut self, reference: &SettingsRef, access: Access) -> Result<(), String> {
        self.0.lock().unwrap().admit(reference, access)
    }
}

/// A participant whose grant covers no settings at all.
struct NoGrant;

impl SettingsGate for NoGrant {
    fn admit(&mut self, reference: &SettingsRef, _: Access) -> Result<(), String> {
        Err(format!("no settings grant for {}", reference.0))
    }
}

/// Counts emissions; the settings guest emits none.
#[derive(Default)]
struct Emits(usize);

impl ActionSink for Emits {
    fn emit(&mut self, _: &str, _: &str) -> Result<(), Refusal> {
        self.0 += 1;
        Ok(())
    }
}

fn participant(engine: &Engine, settings: Settings) -> AppScript<Emits> {
    let component = Component::from_file(engine, guest_component()).expect("guest loads");
    AppScript::attach_with_settings_blocking(
        engine,
        &component,
        Emits::default(),
        Vec::new(),
        settings,
        StoreLimits::default(),
        None,
    )
    .expect("app-settings participant attaches")
}

fn transcript(script: &mut AppScript<Emits>) -> Vec<(String, String)> {
    script.activate_blocking().unwrap();
    script.on_event_blocking("run", "").unwrap();
    script.deactivate_blocking().unwrap();
    assert_eq!(script.sink().0, 0);
    from_log(script.logs())
}

#[test]
fn the_participant_matches_the_native_transcript() {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    let native = std::thread::spawn(move || {
        let mut s = SettingsServer::new(FixtureOwner::default(), FixtureGate::default());
        serve(&mut s, end).unwrap();
        s
    });
    let reference = run(&mut Client(client));
    let native = native.join().unwrap();
    assert_outcomes(&reference);

    let gate = Shared::default();
    let engine = Engine::default();
    let mut script = participant(
        &engine,
        Box::new(SettingsServer::new(FixtureOwner::default(), gate.clone())),
    );
    assert_eq!(transcript(&mut script), reference);
    // The same questions reached the participant's gate.
    let asked = gate.0.lock().unwrap().asked.clone();
    assert_eq!(asked.len(), CALLS);
    assert_eq!(asked, native.gate.asked);
}

#[test]
fn each_participant_answers_to_its_own_gate() {
    let engine = Engine::default();
    let granted = Shared::default();
    let mut a = participant(
        &engine,
        Box::new(SettingsServer::new(
            FixtureOwner::default(),
            granted.clone(),
        )),
    );
    let mut b = participant(
        &engine,
        Box::new(SettingsServer::new(FixtureOwner::default(), NoGrant)),
    );
    assert_outcomes(&transcript(&mut a));
    // The script stops at its first snapshot when it is refused.
    let refused = transcript(&mut b);
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert!(
        refused[0]
            .1
            .contains("Denied(\"no settings grant for product/appearance\")"),
        "{refused:#?}"
    );
    assert_eq!(granted.0.lock().unwrap().asked.len(), CALLS);
}

mod echo {
    wasmtime::component::bindgen!({
        path: ["../wit", "guest-settings/wit"],
        world: "mere:settings-test/settings-participant",
    });
}

use echo::mere::script::{actions, caps, log, settings as ws};

settings_wit::bridge!(crate::echo::mere::script::settings);

struct Host {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for Host {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl log::Host for Host {
    fn log(&mut self, _: String) {}
}

impl caps::Host for Host {
    fn granted(&mut self) -> Vec<String> {
        Vec::new()
    }
}

impl actions::Host for Host {
    fn emit(&mut self, _: actions::ActionEnvelope) -> Result<(), actions::EmitError> {
        Ok(())
    }
}

impl ws::Host for Host {
    fn snapshot(&mut self, reference: String) -> Result<ws::SettingsSnapshot, ws::Failure> {
        let mut s = SettingsServer::new(FixtureOwner::default(), FixtureGate::default());
        Handler::snapshot(&mut s, reference)
            .map(bridge::theirs::snapshot)
            .map_err(bridge::theirs::failure)
    }
    fn write(&mut self, request: ws::WriteRequest) -> ws::WriteOutcome {
        let mut s = SettingsServer::new(FixtureOwner::default(), FixtureGate::default());
        bridge::theirs::outcome(Handler::write(&mut s, bridge::wire::request(request)))
    }
}

#[test]
fn values_cross_wasmtime_unchanged() {
    let engine = Engine::default();
    let component = Component::from_file(&engine, guest_component()).expect("guest loads");
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    echo::SettingsParticipant::add_to_linker::<Host, HasSelf<Host>>(&mut linker, |h| h).unwrap();
    let mut store = Store::new(
        &engine,
        Host {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
        },
    );
    let guest = echo::SettingsParticipant::instantiate(&mut store, &component, &linker).unwrap();

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
