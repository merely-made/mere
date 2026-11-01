// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! wRPC's value encoding (capability door D15) for the synchronous Component
//! Model types.

use std::io;

pub trait Wire: Sized {
    fn put(&self, out: &mut Vec<u8>);
    fn take(input: &mut &[u8]) -> io::Result<Self>;
}

pub fn bad(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn put_leb(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn take_leb(input: &mut &[u8], bits: u32) -> io::Result<u64> {
    let mut n = 0u64;
    for shift in (0..bits).step_by(7) {
        let (&byte, rest) = input
            .split_first()
            .ok_or_else(|| bad("short integer".into()))?;
        *input = rest;
        let part = u64::from(byte & 0x7f);
        // The last byte may carry only the bits the type has left.
        if shift + 7 > bits && part >> (bits - shift) != 0 {
            return Err(bad("integer overflows its type".into()));
        }
        n |= part << shift;
        if byte & 0x80 == 0 {
            return Ok(n);
        }
    }
    Err(bad("integer too long".into()))
}

/// Signed LEB128.
fn put_sleb(out: &mut Vec<u8>, mut n: i64) {
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        let done = (n == 0 && byte & 0x40 == 0) || (n == -1 && byte & 0x40 != 0);
        if done {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// Signed LEB128 of at most `bits`, refusing a value outside the type.
fn take_sleb(input: &mut &[u8], bits: u32) -> io::Result<i64> {
    let mut n = 0i128;
    let mut shift = 0;
    while shift < bits {
        let (&byte, rest) = input
            .split_first()
            .ok_or_else(|| bad("short integer".into()))?;
        *input = rest;
        n |= i128::from(byte & 0x7f) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                n |= -1i128 << shift;
            }
            let (min, max) = (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1);
            if n < min || n > max {
                return Err(bad("integer overflows its type".into()));
            }
            return Ok(n as i64);
        }
    }
    Err(bad("integer too long".into()))
}

fn take_bytes<'a>(input: &mut &'a [u8], n: usize) -> io::Result<&'a [u8]> {
    if input.len() < n {
        return Err(bad("short value".into()));
    }
    let (head, rest) = input.split_at(n);
    *input = rest;
    Ok(head)
}

/// A variant or function discriminant: LEB128 u32.
pub fn put_disc(out: &mut Vec<u8>, d: u32) {
    put_leb(out, d.into())
}

pub fn take_disc(input: &mut &[u8]) -> io::Result<u32> {
    take_leb(input, 32).map(|n| n as u32)
}

/// Decode a whole reply, refusing trailing bytes.
pub fn finish<T: Wire>(mut reply: &[u8]) -> io::Result<T> {
    let value = T::take(&mut reply)?;
    if !reply.is_empty() {
        return Err(bad("trailing bytes".into()));
    }
    Ok(value)
}

impl Wire for () {
    fn put(&self, _: &mut Vec<u8>) {}
    fn take(_: &mut &[u8]) -> io::Result<Self> {
        Ok(())
    }
}

impl Wire for bool {
    fn put(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self))
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        match take_bytes(input, 1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(bad(format!("bool byte {b}"))),
        }
    }
}

impl Wire for u8 {
    fn put(&self, out: &mut Vec<u8>) {
        out.push(*self)
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        Ok(take_bytes(input, 1)?[0])
    }
}

impl Wire for i8 {
    fn put(&self, out: &mut Vec<u8>) {
        out.push(*self as u8)
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        Ok(take_bytes(input, 1)?[0] as i8)
    }
}

impl Wire for u16 {
    fn put(&self, out: &mut Vec<u8>) {
        put_leb(out, (*self).into())
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        take_leb(input, 16).map(|n| n as u16)
    }
}

macro_rules! signed {
    ($t:ty, $bits:literal) => {
        impl Wire for $t {
            fn put(&self, out: &mut Vec<u8>) {
                put_sleb(out, (*self).into())
            }
            fn take(input: &mut &[u8]) -> io::Result<Self> {
                take_sleb(input, $bits).map(|n| n as $t)
            }
        }
    };
}

signed!(i16, 16);
signed!(i32, 32);
signed!(i64, 64);

/// Floats are little-endian, with every NaN written as the canonical one.
impl Wire for f32 {
    fn put(&self, out: &mut Vec<u8>) {
        let bits = if self.is_nan() {
            0x7fc0_0000
        } else {
            self.to_bits()
        };
        out.extend_from_slice(&bits.to_le_bytes())
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        Ok(f32::from_le_bytes(
            take_bytes(input, 4)?.try_into().unwrap(),
        ))
    }
}

impl Wire for f64 {
    fn put(&self, out: &mut Vec<u8>) {
        let bits = if self.is_nan() {
            0x7ff8_0000_0000_0000
        } else {
            self.to_bits()
        };
        out.extend_from_slice(&bits.to_le_bytes())
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        Ok(f64::from_le_bytes(
            take_bytes(input, 8)?.try_into().unwrap(),
        ))
    }
}

/// A char is its UTF-8 bytes, with no length: the first byte says how many.
impl Wire for char {
    fn put(&self, out: &mut Vec<u8>) {
        let mut buf = [0; 4];
        out.extend_from_slice(self.encode_utf8(&mut buf).as_bytes())
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        let first = *input.first().ok_or_else(|| bad("short char".into()))?;
        let len = match first {
            0x00..=0x7f => 1,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            b => return Err(bad(format!("char lead byte {b:#x}"))),
        };
        let bytes = take_bytes(input, len)?;
        let s = std::str::from_utf8(bytes).map_err(|e| bad(e.to_string()))?;
        Ok(s.chars().next().unwrap())
    }
}

/// Flags: a little-endian bitset of `bytes` bytes.
pub fn put_flags(out: &mut Vec<u8>, bits: u64, bytes: usize) {
    out.extend_from_slice(&bits.to_le_bytes()[..bytes])
}

/// Flags of `count` flags, refusing a bit past the last.
pub fn take_flags(input: &mut &[u8], count: usize) -> io::Result<u64> {
    let mut buf = [0; 8];
    buf[..count.div_ceil(8)].copy_from_slice(take_bytes(input, count.div_ceil(8))?);
    let bits = u64::from_le_bytes(buf);
    if count < 64 && bits >> count != 0 {
        return Err(bad("unknown flag".into()));
    }
    Ok(bits)
}

impl Wire for u32 {
    fn put(&self, out: &mut Vec<u8>) {
        put_leb(out, (*self).into())
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        take_leb(input, 32).map(|n| n as u32)
    }
}

impl Wire for u64 {
    fn put(&self, out: &mut Vec<u8>) {
        put_leb(out, *self)
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        take_leb(input, 64)
    }
}

/// A list: its length, then each element. A `u8` element is its byte, so
/// `list<u8>` comes out as wRPC's length-prefixed bytes.
impl<T: Wire> Wire for Vec<T> {
    fn put(&self, out: &mut Vec<u8>) {
        put_leb(out, self.len() as u64);
        for v in self {
            v.put(out);
        }
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        let n = take_leb(input, 32)? as usize;
        // Every element takes at least a byte, except `()`: never trust a
        // claimed length past what the input could hold.
        let mut items = Vec::with_capacity(n.min(input.len()));
        for _ in 0..n {
            items.push(T::take(input)?);
        }
        Ok(items)
    }
}

impl Wire for String {
    fn put(&self, out: &mut Vec<u8>) {
        put_leb(out, self.len() as u64);
        out.extend_from_slice(self.as_bytes());
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        let n = take_leb(input, 32)? as usize;
        String::from_utf8(take_bytes(input, n)?.to_vec()).map_err(|e| bad(e.to_string()))
    }
}

impl<T: Wire> Wire for Option<T> {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            None => out.push(0),
            Some(v) => {
                out.push(1);
                v.put(out)
            },
        }
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        match take_bytes(input, 1)?[0] {
            0 => Ok(None),
            1 => Ok(Some(T::take(input)?)),
            b => Err(bad(format!("option byte {b}"))),
        }
    }
}

impl<T: Wire, E: Wire> Wire for Result<T, E> {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Ok(v) => {
                out.push(0);
                v.put(out)
            },
            Err(e) => {
                out.push(1);
                e.put(out)
            },
        }
    }
    fn take(input: &mut &[u8]) -> io::Result<Self> {
        match take_bytes(input, 1)?[0] {
            0 => Ok(Ok(T::take(input)?)),
            1 => Ok(Err(E::take(input)?)),
            b => Err(bad(format!("result byte {b}"))),
        }
    }
}

/// A tuple: its elements in order.
macro_rules! tuple {
    ($($t:ident $i:tt),+) => {
        impl<$($t: Wire),+> Wire for ($($t,)+) {
            fn put(&self, out: &mut Vec<u8>) {
                $(self.$i.put(out);)+
            }
            fn take(input: &mut &[u8]) -> io::Result<Self> {
                Ok(($($t::take(input)?,)+))
            }
        }
    };
}

tuple!(A 0);
tuple!(A 0, B 1);
tuple!(A 0, B 1, C 2);
tuple!(A 0, B 1, C 2, D 3);
tuple!(A 0, B 1, C 2, D 3, E 4);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14);
tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15);
