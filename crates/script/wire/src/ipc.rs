// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Frames over an ipc-channel bytes pair (D19). Both ends serialize, so one
//! can be handed to another process over a channel, as N1's introduction
//! does. Messages are whole, so there is no length prefix.

use std::io;

use ipc_channel::IpcError;
use ipc_channel::ipc::{IpcBytesReceiver, IpcBytesSender, bytes_channel};
use serde::{Deserialize, Serialize};

use crate::{Serve, Transport, bad};

/// The consumer's end.
#[derive(Serialize, Deserialize)]
pub struct Client {
    tx: IpcBytesSender,
    rx: IpcBytesReceiver,
    max: usize,
}

/// The provider's end.
#[derive(Serialize, Deserialize)]
pub struct Server {
    rx: IpcBytesReceiver,
    tx: IpcBytesSender,
    max: usize,
}

/// A connected pair; neither side accepts a message longer than `max` bytes.
pub fn pair(max: usize) -> io::Result<(Client, Server)> {
    let (client_tx, server_rx) = bytes_channel()?;
    let (server_tx, client_rx) = bytes_channel()?;
    Ok((
        Client {
            tx: client_tx,
            rx: client_rx,
            max,
        },
        Server {
            rx: server_rx,
            tx: server_tx,
            max,
        },
    ))
}

/// `None` when the other end has gone.
fn recv(rx: &IpcBytesReceiver, max: usize) -> io::Result<Option<Vec<u8>>> {
    match rx.recv() {
        Ok(frame) if frame.len() > max => {
            Err(bad(format!("message of {} bytes, over {max}", frame.len())))
        },
        Ok(frame) => Ok(Some(frame)),
        Err(IpcError::Disconnected) => Ok(None),
        Err(IpcError::Io(e)) => Err(e),
        Err(e) => Err(io::Error::other(e.to_string())),
    }
}

impl Transport for Client {
    fn call(&mut self, frame: Vec<u8>) -> io::Result<Vec<u8>> {
        self.tx.send(&frame)?;
        recv(&self.rx, self.max)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "provider hung up"))
    }
}

impl Serve for Server {
    fn serve(self, mut reply: impl FnMut(&[u8]) -> io::Result<Vec<u8>>) -> io::Result<()> {
        while let Some(frame) = recv(&self.rx, self.max)? {
            self.tx.send(&reply(&frame)?)?;
        }
        Ok(())
    }
}
