//! The runtime the generated code calls: wRPC's value encoding (D15) for the
//! types the projection session uses, and the blocking transport seam.

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

/// One blocking request and its reply.
pub trait Transport {
    fn call(&mut self, frame: Vec<u8>) -> io::Result<Vec<u8>>;
}
