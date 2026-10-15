// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native-messaging frames: one JSON message behind a native-endian 32-bit
//! byte length, the WebExtensions wire.
//!
//! Both local doors use it: the browser door, because a browser extension
//! speaks nothing else, and the first-party application door, so one
//! resident reads one framing. End-of-stream between messages is normal
//! completion; a partial prefix or body is an error.

use std::io::{Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// A tighter application bound than Chrome's 64 MiB extension-to-host limit.
pub const MAX_NATIVE_MESSAGE_BYTES: u32 = 1024 * 1024;

/// Why one frame could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("native messaging frame of {len} bytes exceeds the {max} byte bound")]
    TooLarge { len: u64, max: u32 },
    #[error("native messaging stream failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("native messaging JSON failed: {0}")]
    Json(#[from] serde_json::Error),
}

/// Read one native-messaging JSON frame.
pub fn read_native_message<R, T>(reader: &mut R) -> Result<Option<T>, FrameError>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut length = [0u8; 4];
    let mut read = 0usize;
    while read < length.len() {
        match reader.read(&mut length[read..])? {
            0 if read == 0 => return Ok(None),
            0 => return Err(truncated_prefix()),
            count => read += count,
        }
    }
    let length = bounded(u32::from_ne_bytes(length))?;
    let mut body = vec![0u8; length as usize];
    reader.read_exact(&mut body)?;
    Ok(Some(serde_json::from_slice(&body)?))
}

/// Write one native-messaging JSON frame.
pub fn write_native_message<W, T>(writer: &mut W, message: &T) -> Result<(), FrameError>
where
    W: Write,
    T: Serialize,
{
    let body = serde_json::to_vec(message)?;
    let length = body_length(&body)?;
    writer.write_all(&length.to_ne_bytes())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

/// Read one native-messaging JSON frame from an asynchronous stream, with the
/// same size and truncation behavior as [`read_native_message`].
pub async fn read_native_message_async<R, T>(reader: &mut R) -> Result<Option<T>, FrameError>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut length = [0u8; 4];
    let mut read = 0usize;
    while read < length.len() {
        match AsyncReadExt::read(reader, &mut length[read..]).await? {
            0 if read == 0 => return Ok(None),
            0 => return Err(truncated_prefix()),
            count => read += count,
        }
    }
    let length = bounded(u32::from_ne_bytes(length))?;
    let mut body = vec![0u8; length as usize];
    AsyncReadExt::read_exact(reader, &mut body).await?;
    Ok(Some(serde_json::from_slice(&body)?))
}

/// Write one native-messaging JSON frame to an asynchronous stream.
pub async fn write_native_message_async<W, T>(writer: &mut W, message: &T) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let body = serde_json::to_vec(message)?;
    let length = body_length(&body)?;
    AsyncWriteExt::write_all(writer, &length.to_ne_bytes()).await?;
    AsyncWriteExt::write_all(writer, &body).await?;
    AsyncWriteExt::flush(writer).await?;
    Ok(())
}

fn truncated_prefix() -> FrameError {
    std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        "native messaging length prefix was truncated",
    )
    .into()
}

fn bounded(length: u32) -> Result<u32, FrameError> {
    if length > MAX_NATIVE_MESSAGE_BYTES {
        return Err(FrameError::TooLarge {
            len: u64::from(length),
            max: MAX_NATIVE_MESSAGE_BYTES,
        });
    }
    Ok(length)
}

fn body_length(body: &[u8]) -> Result<u32, FrameError> {
    let length = u32::try_from(body.len()).map_err(|_| FrameError::TooLarge {
        len: body.len() as u64,
        max: MAX_NATIVE_MESSAGE_BYTES,
    })?;
    bounded(length)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn frames_json_with_a_native_endian_length() {
        let message = serde_json::json!({ "type": "take_notice" });
        let mut bytes = Vec::new();
        write_native_message(&mut bytes, &message).unwrap();
        let body = serde_json::to_vec(&message).unwrap();
        assert_eq!(&bytes[..4], &(body.len() as u32).to_ne_bytes());
        let decoded: serde_json::Value = read_native_message(&mut Cursor::new(bytes))
            .unwrap()
            .unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn end_of_stream_between_frames_is_none_and_mid_prefix_is_an_error() {
        let empty: Option<serde_json::Value> =
            read_native_message(&mut Cursor::new(Vec::new())).unwrap();
        assert!(empty.is_none());
        assert!(matches!(
            read_native_message::<_, serde_json::Value>(&mut Cursor::new(vec![1, 0])),
            Err(FrameError::Io(_))
        ));
    }

    #[test]
    fn an_oversized_length_is_refused_before_its_body_is_read() {
        let bytes = (MAX_NATIVE_MESSAGE_BYTES + 1).to_ne_bytes().to_vec();
        assert!(matches!(
            read_native_message::<_, serde_json::Value>(&mut Cursor::new(bytes)),
            Err(FrameError::TooLarge { .. })
        ));
    }

    #[tokio::test]
    async fn asynchronous_frames_match_the_synchronous_wire() {
        let message = serde_json::json!({ "type": "failure", "message": "no" });
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        write_native_message_async(&mut writer, &message)
            .await
            .unwrap();
        drop(writer);
        let decoded: serde_json::Value = read_native_message_async(&mut reader)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(decoded, message);
        let mut sync = Vec::new();
        write_native_message(&mut sync, &message).unwrap();
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        write_native_message_async(&mut writer, &message)
            .await
            .unwrap();
        drop(writer);
        let mut wire = Vec::new();
        reader.read_to_end(&mut wire).await.unwrap();
        assert_eq!(wire, sync);
    }
}
