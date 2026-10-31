// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The NDJSON line codec over an admitted Tokio stream.
//!
//! Inside every admitted stream the wire is one JSON value per line
//! (transport agnosticism note, ruling T2). The codec itself is
//! [`chirograph::line`], which every end shares; this module is
//! that codec over an async reader and writer, for both ends of a native
//! stream: the session loop reads requests and writes responses and notices,
//! and an in-process client (the local doors' duplex) writes requests and
//! reads outputs through the same two functions.
//!
//! [`LineReader::next`] is cancel safe, because the only await inside it is
//! `Lines::next_line`, which is: a read that a timeout abandons keeps its
//! partial line buffered and the next call resumes it. The session loop's
//! notice arm depends on that.

use chirograph::line;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines};

/// Reads one JSON value per line from an async stream.
pub struct LineReader<R> {
    lines: Lines<BufReader<R>>,
}

impl<R: AsyncRead + Unpin> LineReader<R> {
    /// Read lines from `reader`.
    pub fn new(reader: R) -> Self {
        Self {
            lines: BufReader::new(reader).lines(),
        }
    }

    /// The next non-blank line, decoded.
    ///
    /// `Ok(None)` is the end of the stream. A line that is not a `T` is
    /// `Ok(Some(Err(_)))`, so the caller decides whether a bad line ends the
    /// session (a client) or is answered and skipped (the session loop); only
    /// a failed read is an `Err`.
    pub async fn next<T: DeserializeOwned>(
        &mut self,
    ) -> std::io::Result<Option<Result<T, serde_json::Error>>> {
        loop {
            let Some(text) = self.lines.next_line().await? else {
                return Ok(None);
            };
            match line::decode::<T>(&text) {
                Ok(None) => continue,
                Ok(Some(value)) => return Ok(Some(Ok(value))),
                Err(error) => return Ok(Some(Err(error))),
            }
        }
    }
}

/// Write `value` as one line and flush it.
///
/// Flushed per line on purpose: a response or a notice the peer is waiting on
/// must not sit in a buffer until the next one pushes it out.
pub async fn write<W, T>(writer: &mut W, value: &T) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let bytes = line::encode_terminated(value).map_err(std::io::Error::other)?;
    writer.write_all(&bytes).await?;
    writer.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirograph::{CarrierOutput, CarrierRequest, CarrierRequestBody, CarrierResponse};

    #[tokio::test]
    async fn requests_and_outputs_cross_one_duplex_in_both_directions() {
        let (client, server) = tokio::io::duplex(1024);
        let (client_read, mut client_write) = tokio::io::split(client);
        let (server_read, mut server_write) = tokio::io::split(server);
        let mut client_lines = LineReader::new(client_read);
        let mut server_lines = LineReader::new(server_read);

        write(
            &mut client_write,
            &CarrierRequest {
                id: 7,
                body: CarrierRequestBody::Close,
            },
        )
        .await
        .unwrap();
        // A blank line between frames is skipped, not an error.
        client_write.write_all(b"\n").await.unwrap();
        client_write.write_all(b"not json\n").await.unwrap();

        let request: CarrierRequest = server_lines.next().await.unwrap().unwrap().unwrap();
        assert_eq!(request.id, 7);
        assert!(
            server_lines
                .next::<CarrierRequest>()
                .await
                .unwrap()
                .unwrap()
                .is_err(),
            "a bad line is reported, not swallowed"
        );

        write(
            &mut server_write,
            &CarrierOutput::Response(CarrierResponse {
                id: 7,
                body: Ok(chirograph::CarrierResponseBody::Closed),
            }),
        )
        .await
        .unwrap();
        drop(server_write);
        drop(server_lines);
        let output: CarrierOutput = client_lines.next().await.unwrap().unwrap().unwrap();
        assert!(matches!(output, CarrierOutput::Response(response) if response.id == 7));
        assert!(
            client_lines
                .next::<CarrierOutput>()
                .await
                .unwrap()
                .is_none(),
            "the end of the stream is None"
        );
    }
    #[tokio::test]
    async fn native_line_reader_rejects_invalid_utf8_in_complete_line() {
        let (mut writer, reader) = tokio::io::duplex(128);
        let mut lines = LineReader::new(reader);
        writer
            .write_all(b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"A")
            .await
            .unwrap();
        writer.write_all(&[0xff]).await.unwrap();
        writer
            .write_all(b"\",\"projections\":[]}}}}\n")
            .await
            .unwrap();

        let error = lines.next::<CarrierOutput>().await.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn native_line_reader_rejects_invalid_utf8_in_eof_tail() {
        let (mut writer, reader) = tokio::io::duplex(128);
        let mut lines = LineReader::new(reader);
        writer
            .write_all(b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"A")
            .await
            .unwrap();
        writer.write_all(&[0xff]).await.unwrap();
        writer
            .write_all(b"\",\"projections\":[]}}}}")
            .await
            .unwrap();
        drop(writer);

        let error = lines.next::<CarrierOutput>().await.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn cancelled_read_keeps_a_partial_multibyte_line() {
        use std::future::{Future, poll_fn};
        use std::task::Poll;

        let (mut writer, reader) = tokio::io::duplex(128);
        let mut lines = LineReader::new(reader);
        writer
            .write_all(b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"")
            .await
            .unwrap();
        writer.write_all(&[0xe2, 0x82]).await.unwrap();
        let mut pending = Box::pin(lines.next::<CarrierOutput>());
        poll_fn(|cx| {
            assert!(pending.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        drop(pending);
        writer.write_all(&[0xac]).await.unwrap();
        writer
            .write_all(b"\",\"projections\":[]}}}}\n")
            .await
            .unwrap();
        let output = lines
            .next::<CarrierOutput>()
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(output, CarrierOutput::Response(response)
            if matches!(&response.body, Ok(chirograph::CarrierResponseBody::Descriptor(descriptor))
                if descriptor.label == "€")));
    }

    #[tokio::test]
    async fn native_line_reader_preserves_valid_multibyte_utf8_across_writes() {
        let (mut writer, reader) = tokio::io::duplex(128);
        let mut lines = LineReader::new(reader);
        writer
            .write_all(b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"")
            .await
            .unwrap();
        writer.write_all(&[0xe2, 0x82]).await.unwrap();
        writer.write_all(&[0xac]).await.unwrap();
        writer
            .write_all(b"\",\"projections\":[]}}}}\n")
            .await
            .unwrap();

        let output = lines
            .next::<CarrierOutput>()
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(output, CarrierOutput::Response(response)
            if matches!(&response.body, Ok(chirograph::CarrierResponseBody::Descriptor(descriptor))
                if descriptor.label == "€")));
    }
}
