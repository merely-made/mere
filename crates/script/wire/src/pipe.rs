// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Frames over two OS pipes, each with a little-endian `u32` length.

use std::io::{self, PipeReader, PipeWriter, Read, Write};

use crate::{Serve, Transport, bad};

/// The consumer's end.
pub struct Client {
    tx: PipeWriter,
    rx: PipeReader,
    max: usize,
}

/// The provider's end.
pub struct Server {
    rx: PipeReader,
    tx: PipeWriter,
    max: usize,
}

/// A connected pair; neither side reads a frame longer than `max` bytes.
pub fn pair(max: usize) -> io::Result<(Client, Server)> {
    let (server_rx, client_tx) = io::pipe()?;
    let (client_rx, server_tx) = io::pipe()?;
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

fn write_frame(w: &mut impl Write, frame: &[u8]) -> io::Result<()> {
    let len = u32::try_from(frame.len()).map_err(|_| bad("frame over 4 GiB".into()))?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(frame)?;
    w.flush()
}

/// `None` at a clean end of stream.
fn read_frame(r: &mut impl Read, max: usize) -> io::Result<Option<Vec<u8>>> {
    let mut len = [0; 4];
    match r.read_exact(&mut len) {
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        other => other?,
    }
    let len = u32::from_le_bytes(len) as usize;
    if len > max {
        return Err(bad(format!("frame of {len} bytes, over {max}")));
    }
    let mut frame = vec![0; len];
    r.read_exact(&mut frame)?;
    Ok(Some(frame))
}

impl Transport for Client {
    fn call(&mut self, frame: Vec<u8>) -> io::Result<Vec<u8>> {
        write_frame(&mut self.tx, &frame)?;
        read_frame(&mut self.rx, self.max)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "provider hung up"))
    }
}

impl Serve for Server {
    fn serve(mut self, mut reply: impl FnMut(&[u8]) -> io::Result<Vec<u8>>) -> io::Result<()> {
        while let Some(frame) = read_frame(&mut self.rx, self.max)? {
            write_frame(&mut self.tx, &reply(&frame)?)?;
        }
        Ok(())
    }
}
