// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The revisioned settings contract over `mere:script`'s `settings`
//! interface (capability door K1, D22).
//!
//! [`bindings`] is script-witgen's output for the interface. [`SettingsServer`]
//! serves any [`RevisionedSettingsProvider`] behind the generated handler,
//! asking its [`SettingsGate`] before every call; [`serve`] runs it on one
//! channel. [`Remote`] is the consumer: a [`RevisionedSettingsProvider`] over
//! any transport, so a caller written against the owner works unchanged
//! across a channel. [`SettingsCalls`] is the typed call surface both the
//! generated client and a component's imports provide.

#[allow(clippy::all, dead_code, unused, missing_docs)]
pub mod bindings {
    use script_wire::*;
    include!(concat!(env!("OUT_DIR"), "/settings.rs"));
}

mod bridge;
mod convert;
#[cfg(feature = "testkit")]
pub mod testkit;

use std::cell::RefCell;

use mere_surface_api::settings::{
    RevisionedSettingsProvider, RevisionedSettingsSnapshot, RevisionedSettingsWriteRequest,
    RevisionedWriteOutcome, SettingsUnavailable,
};
use script_wire::{Serve, Transport};
use workbench::SettingsRef;

use bindings::{Client, Failure, Handler, Refusal, SettingsSnapshot, WriteOutcome, WriteRequest};
pub use convert::{
    native_outcome, native_request, native_snapshot, wire_outcome, wire_request, wire_snapshot,
};

/// The largest frame a settings channel carries. Settings are small; this
/// bounds a hostile peer, as the projection session's 1 MiB cap does (D13).
pub const MAX_FRAME: usize = 1 << 20;

/// What a call asks of a settings reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
}

/// The caller's authority over settings, asked before every call (D22).
pub trait SettingsGate {
    /// Admit `access` to `reference`, or say why not.
    fn admit(&mut self, reference: &SettingsRef, access: Access) -> Result<(), String>;
}

/// An owner and its gate behind the generated handler.
pub struct SettingsServer<P, G> {
    pub provider: P,
    pub gate: G,
}

impl<P, G> SettingsServer<P, G> {
    pub fn new(provider: P, gate: G) -> Self {
        Self { provider, gate }
    }
}

impl<P: RevisionedSettingsProvider, G: SettingsGate> Handler for SettingsServer<P, G> {
    fn snapshot(&mut self, reference: String) -> Result<SettingsSnapshot, Failure> {
        let reference = SettingsRef(reference);
        self.gate
            .admit(&reference, Access::Read)
            .map_err(Failure::Denied)?;
        self.provider
            .snapshot(&reference)
            .map(|s| wire_snapshot(&s))
            .map_err(|u| Failure::Unavailable(u.message))
    }

    fn write(&mut self, request: WriteRequest) -> WriteOutcome {
        let reference = SettingsRef(request.reference.clone());
        if let Err(message) = self.gate.admit(&reference, Access::Write) {
            return WriteOutcome::Denied(Refusal {
                request_id: request.request_id,
                message,
            });
        }
        wire_outcome(&self.provider.apply_revisioned(native_request(request)))
    }
}

/// Serve `server` on one channel until the consumer hangs up.
pub fn serve<P, G>(server: &mut SettingsServer<P, G>, channel: impl Serve) -> std::io::Result<()>
where
    P: RevisionedSettingsProvider,
    G: SettingsGate,
{
    channel.serve(|frame| bindings::dispatch(server, frame))
}

/// The typed calls, from the generated client or a component's imports.
/// `Err` is a channel that broke; the interface's own refusals are values.
pub trait SettingsCalls {
    fn snapshot(&mut self, reference: String) -> Result<Result<SettingsSnapshot, Failure>, String>;
    fn write(&mut self, request: WriteRequest) -> Result<WriteOutcome, String>;
}

impl<T: Transport> SettingsCalls for Client<T> {
    fn snapshot(&mut self, reference: String) -> Result<Result<SettingsSnapshot, Failure>, String> {
        Client::snapshot(self, reference).map_err(|e| e.to_string())
    }
    fn write(&mut self, request: WriteRequest) -> Result<WriteOutcome, String> {
        Client::write(self, request).map_err(|e| e.to_string())
    }
}

/// A settings owner across a channel.
///
/// The owner contract has no refusal for authority, so a `denied` reaches a
/// caller of [`RevisionedSettingsProvider`] as `Unavailable` with a message
/// starting `denied:`; a broken channel reaches it as `Unavailable` too,
/// which claims no change, so a write whose reply was lost is learned from
/// the next snapshot. [`Remote::calls`] keeps both typed.
pub struct Remote<C>(RefCell<C>);

impl<T: Transport> Remote<Client<T>> {
    pub fn new(transport: T) -> Self {
        Self(RefCell::new(Client(transport)))
    }
}

impl<C: SettingsCalls> Remote<C> {
    pub fn over(calls: C) -> Self {
        Self(RefCell::new(calls))
    }

    /// The typed calls underneath.
    pub fn calls(&mut self) -> &mut C {
        self.0.get_mut()
    }
}

impl<C: SettingsCalls> RevisionedSettingsProvider for Remote<C> {
    fn snapshot(
        &self,
        reference: &SettingsRef,
    ) -> Result<RevisionedSettingsSnapshot, SettingsUnavailable> {
        let message = match self.0.borrow_mut().snapshot(reference.0.clone()) {
            Ok(Ok(s)) => return Ok(native_snapshot(s)),
            Ok(Err(Failure::Denied(m))) => format!("denied: {m}"),
            Ok(Err(Failure::Unavailable(m))) => m,
            Err(e) => format!("disconnected: {e}"),
        };
        Err(SettingsUnavailable { message })
    }

    fn apply_revisioned(
        &mut self,
        request: RevisionedSettingsWriteRequest,
    ) -> RevisionedWriteOutcome {
        match self.0.get_mut().write(wire_request(&request)) {
            Ok(outcome) => native_outcome(outcome),
            Err(e) => RevisionedWriteOutcome::Unavailable {
                request_id: request.request_id,
                reason: SettingsUnavailable {
                    message: format!("disconnected: {e}"),
                },
            },
        }
    }
}
