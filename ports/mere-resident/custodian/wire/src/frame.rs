// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Graphshell door's frame (one JSON message behind a native-endian
//! 32-bit length), with clearing buffers. Reads land in a zeroizing buffer
//! of the exact announced size; writes measure first and serialize into one
//! zeroizing buffer of exact capacity, so no reallocation leaves a copy of a
//! released seed or passphrase behind. End of stream between frames is a
//! clean close; a partial prefix or body is an error.

use std::io::{Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use zeroize::Zeroizing;

use crate::limits::MAX_FRAME_BYTES;

/// Why a frame could not be read, written or decoded.
#[derive(Debug)]
pub enum FrameError {
    TooLarge(u64),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge(len) => write!(f, "frame of {len} bytes exceeds {MAX_FRAME_BYTES}"),
            Self::Io(error) => write!(f, "frame stream: {error}"),
            Self::Json(error) => write!(f, "frame JSON: {error}"),
        }
    }
}

impl std::error::Error for FrameError {}

impl From<std::io::Error> for FrameError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Read one frame body. `None` on a clean end of stream.
pub fn read_frame<R: Read + ?Sized>(reader: &mut R) -> Result<Option<Zeroizing<Vec<u8>>>, FrameError> {
    let mut prefix = [0u8; 4];
    let mut read = 0;
    while read < prefix.len() {
        match reader.read(&mut prefix[read..]) {
            Ok(0) if read == 0 => return Ok(None),
            Ok(0) => {
                return Err(FrameError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "frame length prefix was truncated",
                )));
            },
            Ok(count) => read += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {},
            Err(error) => return Err(error.into()),
        }
    }
    let length = u32::from_ne_bytes(prefix);
    if length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge(u64::from(length)));
    }
    let mut body = Zeroizing::new(vec![0u8; length as usize]);
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

/// Decode one frame body.
pub fn decode<T: DeserializeOwned>(body: &[u8]) -> Result<T, FrameError> {
    serde_json::from_slice(body).map_err(FrameError::Json)
}

/// Read and decode one frame.
pub fn read_message<R: Read + ?Sized, T: DeserializeOwned>(
    reader: &mut R,
) -> Result<Option<T>, FrameError> {
    read_frame(reader)?.map(|body| decode(&body)).transpose()
}

/// Serialize and write one frame.
pub fn write_message<W: Write + ?Sized, T: Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<(), FrameError> {
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, message).map_err(FrameError::Json)?;
    if counter.0 > MAX_FRAME_BYTES as usize {
        return Err(FrameError::TooLarge(counter.0 as u64));
    }
    let mut frame = Zeroizing::new(Vec::with_capacity(4 + counter.0));
    frame.extend_from_slice(&(counter.0 as u32).to_ne_bytes());
    serde_json::to_writer(&mut *frame, message).map_err(FrameError::Json)?;
    debug_assert_eq!(frame.len(), 4 + counter.0, "a frame never reallocates");
    writer.write_all(&frame)?;
    writer.flush()?;
    Ok(())
}

struct Counter(usize);

impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
