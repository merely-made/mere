// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The projection session to a component (capability door C0, D18): the
//! guest runs the shared call script through its imports, served by the same
//! `WitServer` a native channel uses, and its transcript is the native one.
//! Values cross Wasmtime unchanged.

use std::path::PathBuf;

use graphshell_endpoint::wit::bindings::{Handler, Session};
use graphshell_endpoint::wit::testkit::{
    CALLS_BEFORE_REVOCATION, CountdownGate, FixtureEndpoint, assert_outcomes, run,
};
use graphshell_endpoint::wit::{self, Calls, MAX_FRAME, WitCarrier, WitServer, serve};
use wasmtime::component::{Component, HasSelf, Linker, Resource, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

/// The host's record of one opened session: the server's handle.
pub struct SessionRep(Session);

wasmtime::component::bindgen!({
    path: ["../wit", "guest-projection/wit"],
    world: "mere:projection-test/projection-consumer",
    with: { "mere:script/projection-session.session": SessionRep },
});

use mere::script::projection_session as ps;

type Server = WitServer<FixtureEndpoint, CountdownGate>;

fn server() -> Server {
    WitServer::new(
        FixtureEndpoint::default(),
        CountdownGate::new(CALLS_BEFORE_REVOCATION),
    )
}

fn guest_component() -> PathBuf {
    if let Some(path) = std::env::var_os("APP_HOST_PROJECTION_GUEST_WASM") {
        return PathBuf::from(path);
    }
    let path = PathBuf::from("guest-projection/target/wasm32-wasip2/release/projection_guest.wasm");
    assert!(
        path.exists(),
        "guest component missing at {}. Build it with:\n  \
         cd crates/script/app-host/guest-projection && cargo build --target wasm32-wasip2 --release",
        path.display()
    );
    path
}

fn wire(f: wit::Failure) -> ps::Failure {
    match f {
        wit::Failure::Revoked => ps::Failure::Revoked,
        wit::Failure::Expired => ps::Failure::Expired,
        wit::Failure::TooLarge(n) => ps::Failure::TooLarge(n),
        wit::Failure::Refused(s) => ps::Failure::Refused(s),
        wit::Failure::Malformed(s) => ps::Failure::Malformed(s),
    }
}

fn notice(n: wit::Notice) -> ps::Notice {
    ps::Notice {
        epoch: n.epoch,
        revision: n.revision,
    }
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

impl Host {
    fn session(&self, s: &Resource<SessionRep>) -> Session {
        self.table
            .get(s)
            .map(|r| r.0.clone())
            .unwrap_or(Session(Vec::new()))
    }
}

impl ps::Host for Host {
    fn discover(&mut self) -> Result<Vec<u8>, ps::Failure> {
        self.server.discover().map_err(wire)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(Resource<SessionRep>, Vec<u8>), ps::Failure> {
        let (session, opened) = self.server.open(request).map_err(wire)?;
        let res = self
            .table
            .push(SessionRep(session))
            .map_err(|e| ps::Failure::Refused(e.to_string()))?;
        Ok((res, opened))
    }
}

impl ps::HostSession for Host {
    fn snapshot(&mut self, s: Resource<SessionRep>, ask: Vec<u8>) -> Result<Vec<u8>, ps::Failure> {
        let s = self.session(&s);
        self.server.session_snapshot(&s, ask).map_err(wire)
    }
    fn fetch(&mut self, s: Resource<SessionRep>, address: Vec<u8>) -> Result<Vec<u8>, ps::Failure> {
        let s = self.session(&s);
        self.server.session_fetch(&s, address).map_err(wire)
    }
    fn chunk(
        &mut self,
        s: Resource<SessionRep>,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, ps::Failure> {
        let s = self.session(&s);
        self.server
            .session_chunk(&s, address, offset, length)
            .map_err(wire)
    }
    fn resume(
        &mut self,
        s: Resource<SessionRep>,
        epoch: u64,
        revision: u64,
    ) -> Result<Vec<u8>, ps::Failure> {
        let s = self.session(&s);
        self.server
            .session_resume(&s, epoch, revision)
            .map_err(wire)
    }
    fn intent(&mut self, s: Resource<SessionRep>, call: Vec<u8>) -> Result<Vec<u8>, ps::Failure> {
        let s = self.session(&s);
        self.server.session_intent(&s, call).map_err(wire)
    }
    fn suspend(&mut self, s: Resource<SessionRep>) -> Result<(), ps::Failure> {
        let s = self.session(&s);
        self.server.session_suspend(&s).map_err(wire)
    }
    fn take_notice(&mut self, s: Resource<SessionRep>) -> Option<ps::Notice> {
        let s = self.session(&s);
        self.server.session_take_notice(&s).map(notice)
    }
    fn wait_notice(&mut self, s: Resource<SessionRep>) -> Result<ps::Notice, ps::Failure> {
        let s = self.session(&s);
        self.server
            .session_wait_notice(&s)
            .map(notice)
            .map_err(wire)
    }
    fn drop(&mut self, s: Resource<SessionRep>) -> wasmtime::Result<()> {
        let rep = self.table.delete(s)?;
        self.server.drop_session(rep.0);
        Ok(())
    }
}

fn load() -> (Store<Host>, ProjectionConsumer) {
    let engine = Engine::default();
    let component = Component::from_file(&engine, guest_component()).expect("guest loads");
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    ps::add_to_linker::<Host, HasSelf<Host>>(&mut linker, |h| h).unwrap();
    let mut store = Store::new(
        &engine,
        Host {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            server: server(),
        },
    );
    let bindings = ProjectionConsumer::instantiate(&mut store, &component, &linker).unwrap();
    (store, bindings)
}

#[test]
fn the_guest_matches_the_native_transcript() {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    let native = std::thread::spawn(move || {
        let mut s = server();
        serve(&mut s, end).unwrap();
    });
    let reference = run(&mut WitCarrier::new(Calls::new(client)));
    native.join().unwrap();
    assert_outcomes(&reference);

    let (mut store, guest) = load();
    let transcript = guest.call_run(&mut store).unwrap();
    assert_eq!(transcript, reference);
    // The guest's Close dropped its resource, which reached the server.
    assert_eq!(store.data().server.gate.closed.len(), 1);
    assert_eq!(store.data().server.open_sessions(), 0);
}

#[test]
fn values_cross_wasmtime_unchanged() {
    let (mut store, guest) = load();
    let failures = [
        ps::Failure::Revoked,
        ps::Failure::Expired,
        ps::Failure::TooLarge(u64::MAX),
        ps::Failure::Refused("ä — ✓".into()),
        ps::Failure::Malformed("x".repeat(200)),
    ];
    for f in failures {
        let back = guest.call_echo_failure(&mut store, &f).unwrap();
        assert_eq!(format!("{back:?}"), format!("{f:?}"));
    }
    let notices = [
        None,
        Some(ps::Notice {
            epoch: u64::MAX,
            revision: 1 << 35,
        }),
    ];
    for n in notices {
        let back = guest.call_echo_notice(&mut store, n).unwrap();
        assert_eq!(
            back.map(|n| (n.epoch, n.revision)),
            n.map(|n| (n.epoch, n.revision))
        );
    }
}
