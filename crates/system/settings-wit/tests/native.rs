// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The settings interface natively (capability door K1, D22): the shared
//! call script over OS pipes matches the server called in process, the
//! gate is asked on every call, and the owner contract holds through
//! [`Remote`].

use mere_surface_api::settings::{
    RevisionedSettingsProvider, RevisionedWriteOutcome, SettingValue, SettingsUnavailable,
};
use settings_wit::bindings::{
    Client, Failure, Handler, SettingsSnapshot, WriteOutcome, WriteRequest,
};
use settings_wit::testkit::{
    self, ABOUT, APPEARANCE, CALLS, FixtureGate, FixtureOwner, PRIVATE, assert_outcomes, run,
};
use settings_wit::{
    Access, MAX_FRAME, Remote, SettingsCalls, SettingsServer, native_snapshot, serve,
};
use workbench::SettingsRef;

type Server = SettingsServer<FixtureOwner, FixtureGate>;

fn server() -> Server {
    SettingsServer::new(FixtureOwner::default(), FixtureGate::default())
}

/// The server called in process, with no channel.
struct Direct(Server);

impl SettingsCalls for Direct {
    fn snapshot(&mut self, reference: String) -> Result<Result<SettingsSnapshot, Failure>, String> {
        Ok(Handler::snapshot(&mut self.0, reference))
    }
    fn write(&mut self, request: WriteRequest) -> Result<WriteOutcome, String> {
        Ok(Handler::write(&mut self.0, request))
    }
}

/// Run `f` against a client whose server is on a pipe, and return the
/// server when the client hangs up.
fn over_pipe<R>(f: impl FnOnce(Client<script_wire::pipe::Client>) -> R) -> (R, Server) {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    let provider = std::thread::spawn(move || {
        let mut s = server();
        serve(&mut s, end).unwrap();
        s
    });
    let r = f(Client(client));
    (r, provider.join().unwrap())
}

#[test]
fn the_pipe_matches_the_server_in_process() {
    let mut direct = Direct(server());
    let reference = run(&mut direct);
    assert_outcomes(&reference);

    let (transcript, served) = over_pipe(|mut c| run(&mut c));
    assert_eq!(transcript, reference);
    assert_eq!(served.gate.asked.len(), CALLS);
    assert_eq!(served.gate.asked, direct.0.gate.asked);
    // Only the writes that committed moved the owner.
    assert!(served.provider.chrome);
    assert_eq!(served.provider.history, i64::MIN);
    assert_eq!(served.provider.appearance_rev, 6);
}

#[test]
fn the_gate_is_asked_before_the_owner() {
    let mut s = server();
    let denied = Handler::snapshot(&mut s, PRIVATE.into());
    assert_eq!(
        denied,
        Err(Failure::Denied("no grant for product/private".into()))
    );
    let guard =
        settings_wit::wire_snapshot(&s.provider.snapshot(&SettingsRef(ABOUT.into())).unwrap())
            .specs[0]
            .guard
            .clone();
    let outcome = Handler::write(
        &mut s,
        WriteRequest {
            request_id: "w".into(),
            reference: ABOUT.into(),
            setting_id: "about.version".into(),
            expected: guard,
            value: settings_wit::bindings::Value::Text("2".into()),
        },
    );
    assert!(matches!(outcome, WriteOutcome::Denied(r) if r.request_id == "w"));
    assert_eq!(
        s.gate.asked,
        vec![
            (PRIVATE.into(), Access::Read),
            (ABOUT.into(), Access::Write)
        ]
    );
}

#[test]
fn remote_is_an_owner() {
    let ((), served) = over_pipe(|client| {
        let mut remote = Remote::over(client);
        let page = SettingsRef(APPEARANCE.into());
        let snapshot = remote.snapshot(&page).unwrap();
        let local = FixtureOwner::default().snapshot(&page).unwrap();
        assert_eq!(snapshot, local);

        let request =
            |id: &str, guard| mere_surface_api::settings::RevisionedSettingsWriteRequest {
                request_id: id.into(),
                reference: page.clone(),
                setting_id: "ui.zoom".into(),
                expected: guard,
                value: SettingValue::Number(1.1),
            };
        let guard = snapshot.specs[1].guard.clone();
        let RevisionedWriteOutcome::Committed(c) =
            remote.apply_revisioned(request("a", guard.clone()))
        else {
            panic!("current guard commits");
        };
        // The owner's canonical value comes back, not the requested one.
        assert_eq!(c.value, SettingValue::Number(f64::from(1.1f32)));
        assert!(matches!(
            remote.apply_revisioned(request("b", guard)),
            RevisionedWriteOutcome::Conflict { request_id, .. } if request_id == "b"
        ));

        // Denied has no case in the owner contract: it reads as unavailable.
        assert_eq!(
            remote.snapshot(&SettingsRef(PRIVATE.into())),
            Err(SettingsUnavailable {
                message: "denied: no grant for product/private".into()
            })
        );
        // The typed calls keep it.
        assert!(matches!(
            remote.calls().snapshot(PRIVATE.into()),
            Ok(Err(Failure::Denied(_)))
        ));
    });
    assert_eq!(served.provider.zoom, f64::from(1.1f32));
}

#[test]
fn a_vanished_owner_reads_as_unavailable() {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    drop(end);
    let mut remote = Remote::new(client);
    let page = SettingsRef(APPEARANCE.into());
    let err = remote.snapshot(&page).unwrap_err();
    assert!(err.message.starts_with("disconnected"), "{err:?}");
    let guard = FixtureOwner::default().guard("appearance");
    let outcome =
        remote.apply_revisioned(mere_surface_api::settings::RevisionedSettingsWriteRequest {
            request_id: "lost".into(),
            reference: page,
            setting_id: "chrome.enabled".into(),
            expected: guard,
            value: SettingValue::Boolean(true),
        });
    assert!(matches!(
        outcome,
        RevisionedWriteOutcome::Unavailable { request_id, .. } if request_id == "lost"
    ));
}

#[test]
fn conversions_round_trip() {
    let wire = testkit::edge_snapshot();
    assert_eq!(
        settings_wit::wire_snapshot(&native_snapshot(wire.clone())),
        wire
    );
    for o in testkit::edge_outcomes() {
        if matches!(o, WriteOutcome::Denied(_)) {
            continue; // Denied folds into Unavailable natively, by design.
        }
        let back = settings_wit::wire_outcome(&settings_wit::native_outcome(o.clone()));
        assert_eq!(format!("{back:?}"), format!("{o:?}"));
    }
}

#[test]
fn the_client_type_is_the_generated_one() {
    // Compiles only if the generated client is a SettingsCalls.
    fn calls<C: SettingsCalls>() {}
    calls::<Client<script_wire::pipe::Client>>();
}
