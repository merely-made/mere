// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The runtime `script-witgen`'s bindings call (capability door D17-D19):
//! wRPC's value encoding, the blocking [`Transport`] seam a client calls
//! through, and the transports. A request is the call's name and its
//! arguments; the reply is its result. One call is in flight per channel.
//!
//! [`pipe`] carries frames over OS pipes with a length prefix. [`ipc`],
//! behind the `ipc` feature, carries them over an ipc-channel bytes pair,
//! whose ends can themselves be sent over a channel; it never sets
//! ipc-channel's `force-inprocess` (D10).
//!
//! Generated code expects this crate's items in scope: include it under
//! `use script_wire::*;`.

#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(any(unix, windows))]
pub mod pipe;
mod value;

pub use value::*;

use std::io;

/// One blocking request and its reply.
pub trait Transport {
    fn call(&mut self, frame: Vec<u8>) -> io::Result<Vec<u8>>;
}

/// The provider's side of one channel: answer each request in turn until
/// the consumer hangs up, then return. A reply that fails to build ends the
/// channel with that error.
pub trait Serve {
    fn serve(self, reply: impl FnMut(&[u8]) -> io::Result<Vec<u8>>) -> io::Result<()>;
}
