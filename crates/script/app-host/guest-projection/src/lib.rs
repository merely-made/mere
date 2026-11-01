// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The projection-session test guest: the shared call script through the
//! WIT carrier, over this component's imports.

wit_bindgen::generate!({
    path: ["../../wit", "wit"],
    world: "mere:projection-test/projection-consumer",
    // The imported interface lives in another package, mere:script.
    generate_all,
});

use graphshell_endpoint::wit::{self, Fault, SessionCalls, WitCarrier, testkit};
use mere::script::projection_session as ps;

fn native(f: ps::Failure) -> wit::Failure {
    match f {
        ps::Failure::Revoked => wit::Failure::Revoked,
        ps::Failure::Expired => wit::Failure::Expired,
        ps::Failure::TooLarge(n) => wit::Failure::TooLarge(n),
        ps::Failure::Refused(s) => wit::Failure::Refused(s),
        ps::Failure::Malformed(s) => wit::Failure::Malformed(s),
    }
}

fn notice(n: ps::Notice) -> wit::Notice {
    wit::Notice {
        epoch: n.epoch,
        revision: n.revision,
    }
}

/// The imports as [`SessionCalls`].
struct Imports;

impl SessionCalls for Imports {
    type Session = ps::Session;
    fn discover(&mut self) -> Result<Vec<u8>, Fault> {
        ps::discover().map_err(|f| native(f).into())
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(ps::Session, Vec<u8>), Fault> {
        ps::open(&request).map_err(|f| native(f).into())
    }
    fn snapshot(&mut self, s: &ps::Session, ask: Vec<u8>) -> Result<Vec<u8>, Fault> {
        s.snapshot(&ask).map_err(|f| native(f).into())
    }
    fn fetch(&mut self, s: &ps::Session, address: Vec<u8>) -> Result<Vec<u8>, Fault> {
        s.fetch(&address).map_err(|f| native(f).into())
    }
    fn chunk(
        &mut self,
        s: &ps::Session,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, Fault> {
        s.chunk(&address, offset, length)
            .map_err(|f| native(f).into())
    }
    fn resume(&mut self, s: &ps::Session, epoch: u64, revision: u64) -> Result<Vec<u8>, Fault> {
        s.resume(epoch, revision).map_err(|f| native(f).into())
    }
    fn intent(&mut self, s: &ps::Session, call: Vec<u8>) -> Result<Vec<u8>, Fault> {
        s.intent(&call).map_err(|f| native(f).into())
    }
    fn suspend(&mut self, s: &ps::Session) -> Result<(), Fault> {
        s.suspend().map_err(|f| native(f).into())
    }
    fn take_notice(&mut self, s: &ps::Session) -> Result<Option<wit::Notice>, Fault> {
        Ok(s.take_notice().map(notice))
    }
    fn wait_notice(&mut self, s: &ps::Session) -> Result<wit::Notice, Fault> {
        s.wait_notice().map(notice).map_err(|f| native(f).into())
    }
    fn close(&mut self, s: ps::Session) -> Result<(), Fault> {
        drop(s);
        Ok(())
    }
}

struct Component;

impl Guest for Component {
    fn run() -> Vec<(String, String)> {
        testkit::run(&mut WitCarrier::new(Imports))
    }
    fn echo_failure(value: ps::Failure) -> ps::Failure {
        value
    }
    fn echo_notice(value: Option<ps::Notice>) -> Option<ps::Notice> {
        value
    }
}

export!(Component);
