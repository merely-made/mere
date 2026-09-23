// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The two text codecs keys travel in: base58btc (inside `did:key`) and hex
//! (a Reticulum identity, and UUIDs).
//!
//! Hand-rolled rather than a dependency, as castellan's base32 is: each is a
//! few dozen lines, and gaz keeps its dependency list to serde.

/// The Bitcoin base58 alphabet, which multibase names `base58btc`.
const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Encode bytes as base58btc. Each leading zero byte becomes a leading `1`.
pub(crate) fn base58_encode(bytes: &[u8]) -> String {
    let zeros = bytes.iter().take_while(|&&byte| byte == 0).count();
    // Base-58 digits, least significant first.
    let mut digits: Vec<u8> = Vec::with_capacity(bytes.len() * 138 / 100 + 1);
    for &byte in &bytes[zeros..] {
        let mut carry = u32::from(byte);
        for digit in &mut digits {
            carry += u32::from(*digit) << 8;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut out = "1".repeat(zeros);
    out.extend(
        digits
            .iter()
            .rev()
            .map(|&digit| char::from(BASE58_ALPHABET[usize::from(digit)])),
    );
    out
}

/// Decode base58btc, or `None` on a character outside the alphabet.
pub(crate) fn base58_decode(text: &str) -> Option<Vec<u8>> {
    let zeros = text.bytes().take_while(|&c| c == b'1').count();
    // Bytes, least significant first.
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() * 733 / 1000 + 1);
    for c in text.bytes().skip(zeros) {
        let mut carry = u32::from(base58_value(c)?);
        for byte in &mut bytes {
            carry += u32::from(*byte) * 58;
            *byte = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    let mut out = vec![0u8; zeros];
    out.extend(bytes.iter().rev());
    Some(out)
}

fn base58_value(c: u8) -> Option<u8> {
    BASE58_ALPHABET
        .iter()
        .position(|&symbol| symbol == c)
        .map(|index| index as u8)
}

/// Lowercase hex, two characters per byte.
pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

/// Decode exactly `N` bytes of hex, either case, or `None`.
pub(crate) fn hex_decode<const N: usize>(text: &str) -> Option<[u8; N]> {
    let digits = text.as_bytes();
    if digits.len() != N * 2 {
        return None;
    }
    let (pairs, _) = digits.as_chunks::<2>();
    let mut out = [0u8; N];
    for (slot, [high, low]) in out.iter_mut().zip(pairs) {
        *slot = (hex_value(*high)? << 4) | hex_value(*low)?;
    }
    Some(out)
}

const fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The examples published in the IETF base58 draft (draft-msporny-base58).
    const DRAFT_VECTORS: [(&[u8], &str); 3] = [
        (b"Hello World!", "2NEpo7TZRRrLZSi2U"),
        (
            b"The quick brown fox jumps over the lazy dog.",
            "USm3fpXnKG5EUBx2ndxBDMPVciP5hGey2Jh4NDv6gmeo1LkMeiKrLJUUBk6Z",
        ),
        (&[0x00, 0x00, 0x28, 0x7f, 0xb4, 0xcd], "11233QC4"),
    ];

    #[test]
    fn base58_matches_the_draft_vectors_both_ways() {
        for (bytes, text) in DRAFT_VECTORS {
            assert_eq!(base58_encode(bytes), text);
            assert_eq!(base58_decode(text).as_deref(), Some(bytes));
        }
    }

    #[test]
    fn base58_keeps_leading_zeros_and_the_empty_string() {
        assert_eq!(base58_encode(&[]), "");
        assert_eq!(base58_encode(&[0]), "1");
        assert_eq!(base58_decode("11"), Some(vec![0, 0]));
        assert_eq!(base58_decode(""), Some(vec![]));
    }

    #[test]
    fn base58_refuses_characters_outside_the_alphabet() {
        for bad in ["0", "O", "I", "l", "+", "abc!"] {
            assert_eq!(base58_decode(bad), None, "{bad} is not base58btc");
        }
    }

    #[test]
    fn hex_round_trips_and_reads_either_case() {
        let bytes = [0x00, 0x7f, 0xab, 0xff];
        assert_eq!(hex_encode(&bytes), "007fabff");
        assert_eq!(hex_decode::<4>("007FABff"), Some(bytes));
    }

    #[test]
    fn hex_refuses_wrong_length_and_bad_digits() {
        assert_eq!(hex_decode::<2>("abc"), None);
        assert_eq!(hex_decode::<2>("abcdef"), None);
        assert_eq!(hex_decode::<2>("zz00"), None);
    }
}
