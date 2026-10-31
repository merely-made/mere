// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! NDJSON, the wire inside every session stream.
//!
//! Every carrier, whatever envelope it needs below admission, carries the same
//! lines once a session is admitted: one JSON value per line, a
//! [`crate::CarrierRequest`] one way and a [`crate::CarrierOutput`] the other
//! (transport agnosticism note, ruling T2). This is the codec every end
//! shares, with no I/O beneath it: `graphshell-endpoint`'s stdio and network
//! carriers, `graphshell-client`'s session driver, the browser's WebRTC
//! session, and `graphshell_host::line`, which runs it over Tokio streams. It
//! lives here, beside the types it frames, so none of them takes a dependency
//! on another to share it (Mark's ruling, 2026-10-15).
//!
//! A line here is the text without its newline. [`encode`] never produces a
//! newline inside a value (compact JSON escapes them), and [`decode`] treats a
//! blank line as nothing rather than an error, because a peer may send one to
//! keep a link warm.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// One value as one line, without the terminating newline.
pub fn encode<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// One value as the bytes of one line, newline included.
pub fn encode_terminated<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut line = serde_json::to_vec(value)?;
    line.push(b'\n');
    Ok(line)
}

/// One line read back. `Ok(None)` for a blank line; surrounding whitespace,
/// including a carriage return, is ignored.
pub fn decode<T: DeserializeOwned>(line: &str) -> Result<Option<T>, serde_json::Error> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    serde_json::from_str(trimmed).map(Some)
}

/// Bytes in, complete lines out, with the partial tail carried across calls.
///
/// For a carrier that delivers bytes in pieces of its own size rather than as
/// a stream: the WebRTC host writes NDJSON to a byte stream and its pump cuts
/// that stream at frame boundaries with no regard for lines, and this is the
/// browser's side of that bargain. Pure, so its cases are ordinary tests: a line split across
/// two pushes, two lines in one push, an empty line, and bytes with no newline
/// yet.
#[derive(Debug, Default)]
pub struct LineAssembler {
    buffer: Vec<u8>,
}

impl LineAssembler {
    /// Append bytes as they arrived.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// The next complete UTF-8 line, without its newline, if one is buffered.
    ///
    /// A trailing carriage return is dropped too, so a host writing CRLF is
    /// read the same as one writing LF. Invalid UTF-8 is returned as an error.
    pub fn next_line(&mut self) -> Result<Option<String>, std::string::FromUtf8Error> {
        let Some(end) = self.buffer.iter().position(|byte| *byte == b'\n') else {
            return Ok(None);
        };
        let mut line: Vec<u8> = self.buffer.drain(..=end).collect();
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        String::from_utf8(line).map(Some)
    }

    /// Whatever is buffered without a terminating newline, taken as UTF-8.
    ///
    /// For end-of-stream: a host that closed after a final line with no
    /// newline still said something, and this is how it is heard. Invalid
    /// UTF-8 is returned as an error.
    pub fn take_partial(&mut self) -> Result<Option<String>, std::string::FromUtf8Error> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        let tail = std::mem::take(&mut self.buffer);
        String::from_utf8(tail).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_split_across_two_pushes_arrives_once() {
        let mut lines = LineAssembler::default();
        lines.push(b"{\"id\":1,\"bo");
        assert_eq!(
            lines.next_line().unwrap(),
            None,
            "half a line is not a line"
        );
        lines.push(b"dy\":null}\n");
        assert_eq!(
            lines.next_line().unwrap().as_deref(),
            Some("{\"id\":1,\"body\":null}")
        );
        assert_eq!(lines.next_line().unwrap(), None);
    }

    #[test]
    fn two_lines_in_one_push_come_out_as_two() {
        let mut lines = LineAssembler::default();
        lines.push(b"a\nb\n");
        assert_eq!(lines.next_line().unwrap().as_deref(), Some("a"));
        assert_eq!(lines.next_line().unwrap().as_deref(), Some("b"));
        assert_eq!(lines.next_line().unwrap(), None);
    }

    #[test]
    fn crlf_reads_the_same_as_lf() {
        let mut lines = LineAssembler::default();
        lines.push(b"a\r\nb\n");
        assert_eq!(lines.next_line().unwrap().as_deref(), Some("a"));
        assert_eq!(lines.next_line().unwrap().as_deref(), Some("b"));
    }

    #[test]
    fn an_empty_line_is_a_line() {
        let mut lines = LineAssembler::default();
        lines.push(b"\n");
        assert_eq!(lines.next_line().unwrap().as_deref(), Some(""));
    }

    #[test]
    fn invalid_utf8_in_a_descriptor_label_is_not_rewritten_before_decode() {
        let mut lines = LineAssembler::default();
        lines.push(
            b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"A\xff\",\"projections\":[]}}}}\n",
        );

        assert!(
            lines.next_line().is_err(),
            "invalid UTF-8 in a completed line must be rejected before JSON decoding"
        );
    }

    #[test]
    fn invalid_utf8_in_an_eof_descriptor_tail_is_not_rewritten_before_decode() {
        let mut lines = LineAssembler::default();
        lines.push(
            b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"A\xff\",\"projections\":[]}}}}",
        );

        assert!(
            lines.take_partial().is_err(),
            "invalid UTF-8 in an EOF tail must be rejected before JSON decoding"
        );
    }

    #[test]
    fn a_multibyte_character_split_inside_a_codepoint_survives_reassembly() {
        let mut lines = LineAssembler::default();
        lines.push(b"{\"label\":\"");
        lines.push(&[0xe2, 0x82]);
        assert_eq!(
            lines.next_line().unwrap(),
            None,
            "a partial codepoint is not a line"
        );
        lines.push(&[0xac]);
        lines.push(b"\"}\n");

        assert_eq!(
            lines.next_line().unwrap().as_deref(),
            Some("{\"label\":\"€\"}")
        );
    }

    #[test]
    fn the_unterminated_tail_is_taken_only_on_request() {
        let mut lines = LineAssembler::default();
        lines.push(b"unfinished");
        assert_eq!(lines.next_line().unwrap(), None);
        assert_eq!(lines.take_partial().unwrap().as_deref(), Some("unfinished"));
        assert_eq!(lines.take_partial().unwrap(), None, "taken once");
    }

    #[test]
    fn a_value_round_trips_through_one_line() {
        let line = encode(&serde_json::json!({"id": 3, "body": "a\nb"})).unwrap();
        assert!(!line.contains('\n'), "a value never spans two lines");
        let back: serde_json::Value = decode(&line).unwrap().unwrap();
        assert_eq!(back["body"], "a\nb");
        let bytes = encode_terminated(&back).unwrap();
        assert_eq!(bytes.last(), Some(&b'\n'));
    }

    #[test]
    fn a_blank_line_decodes_to_nothing() {
        assert_eq!(decode::<serde_json::Value>("  \r").unwrap(), None);
        assert!(decode::<serde_json::Value>("{").is_err());
    }
}
