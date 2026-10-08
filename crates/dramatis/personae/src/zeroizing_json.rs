// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! serde_json for secret plaintext, without freed copies.
//!
//! `serde_json::to_vec` and serde's `Vec<u8>` visitor both grow by
//! reallocating, and every reallocation frees the outgrown buffer uncleared.
//! [`to_vec`] sizes its buffer before writing; [`bytes`] zeroizes each buffer
//! it outgrows. The wire format is unchanged. Vault lock ruling 6; the
//! no-residue test (`tests/no_residue.rs`) is what holds this to account.

use std::fmt;
use std::io;

use serde::Serialize;
use serde::de::{Deserializer, SeqAccess, Visitor};
use zeroize::Zeroizing;

/// `serde_json::to_vec` into one exactly-sized buffer that zeroizes on drop.
pub(crate) fn to_vec<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<Zeroizing<Vec<u8>>> {
    let mut counted = Counter(0);
    serde_json::to_writer(&mut counted, value)?;
    let mut out = Zeroizing::new(Vec::with_capacity(counted.0));
    serde_json::to_writer(&mut *out, value)?;
    Ok(out)
}

struct Counter(usize);

impl io::Write for Counter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// `#[serde(deserialize_with)]` for secret bytes held as `Vec<u8>`.
pub(crate) fn bytes<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    deserializer.deserialize_seq(BytesVisitor)
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an array of bytes")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<u8>, A::Error> {
        let start = seq.size_hint().unwrap_or(0).clamp(32, 4096);
        let mut out = Zeroizing::new(Vec::with_capacity(start));
        while let Some(byte) = seq.next_element::<u8>()? {
            if out.len() == out.capacity() {
                let mut grown = Zeroizing::new(Vec::with_capacity(out.capacity() * 2));
                grown.extend_from_slice(&out);
                // The outgrown buffer zeroizes as it drops here.
                out = grown;
            }
            out.push(byte);
        }
        Ok(std::mem::take(&mut *out))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Holder {
        #[serde(deserialize_with = "super::bytes")]
        payload: Vec<u8>,
    }

    #[test]
    fn the_wire_format_is_serde_json_s_own() {
        for len in [0usize, 1, 31, 32, 33, 64, 65, 1000] {
            let held = Holder {
                payload: (0..len).map(|i| i as u8).collect(),
            };
            let ours = super::to_vec(&held).unwrap();
            assert_eq!(*ours, serde_json::to_vec(&held).unwrap());
            assert_eq!(ours.len(), ours.capacity());
            let back: Holder = serde_json::from_slice(&ours).unwrap();
            assert_eq!(back, held);
        }
    }

    #[test]
    fn non_bytes_are_refused_as_before() {
        assert!(serde_json::from_slice::<Holder>(br#"{"payload":[256]}"#).is_err());
        assert!(serde_json::from_slice::<Holder>(br#"{"payload":"abc"}"#).is_err());
    }
}
