// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [`SessionCalls`] through the generated client, over any transport.

use std::io;

use script_wire::Transport;

use super::bindings::{Client, Session};
use super::{Fault, Notice, SessionCalls};

/// The generated client as [`SessionCalls`].
pub struct Calls<T: Transport>(pub Client<T>);

impl<T: Transport> Calls<T> {
    pub fn new(transport: T) -> Self {
        Self(Client(transport))
    }
}

fn gone(e: io::Error) -> Fault {
    Fault::Disconnected(e.to_string())
}

impl<T: Transport> SessionCalls for Calls<T> {
    type Session = Session;
    fn discover(&mut self) -> Result<Vec<u8>, Fault> {
        Ok(self.0.discover().map_err(gone)??)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(Session, Vec<u8>), Fault> {
        Ok(self.0.open(request).map_err(gone)??)
    }
    fn snapshot(&mut self, s: &Session, ask: Vec<u8>) -> Result<Vec<u8>, Fault> {
        Ok(self.0.session_snapshot(s, ask).map_err(gone)??)
    }
    fn fetch(&mut self, s: &Session, address: Vec<u8>) -> Result<Vec<u8>, Fault> {
        Ok(self.0.session_fetch(s, address).map_err(gone)??)
    }
    fn chunk(
        &mut self,
        s: &Session,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, Fault> {
        Ok(self
            .0
            .session_chunk(s, address, offset, length)
            .map_err(gone)??)
    }
    fn resume(&mut self, s: &Session, epoch: u64, revision: u64) -> Result<Vec<u8>, Fault> {
        Ok(self.0.session_resume(s, epoch, revision).map_err(gone)??)
    }
    fn intent(&mut self, s: &Session, call: Vec<u8>) -> Result<Vec<u8>, Fault> {
        Ok(self.0.session_intent(s, call).map_err(gone)??)
    }
    fn suspend(&mut self, s: &Session) -> Result<(), Fault> {
        Ok(self.0.session_suspend(s).map_err(gone)??)
    }
    fn take_notice(&mut self, s: &Session) -> Result<Option<Notice>, Fault> {
        self.0.session_take_notice(s).map_err(gone)
    }
    fn wait_notice(&mut self, s: &Session) -> Result<Notice, Fault> {
        Ok(self.0.session_wait_notice(s).map_err(gone)??)
    }
    fn close(&mut self, s: Session) -> Result<(), Fault> {
        self.0.drop_session(s).map_err(gone)
    }
}
