// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! script-wire against wRPC, the reference encoder (D15). `generated/`
//! holds script-witgen's output for `wit/conformance.wit`; script-witgen's
//! golden test keeps it current. Random values must encode as wRPC encodes
//! them, mutated encodings must be accepted exactly when wRPC accepts them,
//! and each type must agree at its edges.

#[allow(clippy::all, dead_code, unused)]
mod ours {
    use script_wire::*;
    include!("generated/conformance.rs");
}

#[allow(clippy::all, dead_code, unused)]
mod wrpc {
    wit_bindgen_wrpc::generate!({
        path: "tests/wit",
        world: "conformance",
    });
}

use ours::test_conformance_shared::{Color, Numbers, Perms, Token};
use ours::{Bag, Handle, Point, Shape};
use proptest::prelude::*;
use script_wire::{Wire, finish};
use wit_bindgen_wrpc::bytes::BytesMut;
use wit_bindgen_wrpc::tokio_util::codec::Decoder;
use wit_bindgen_wrpc::wrpc_transport::{Decode, Encode, ResourceOwn};
use wrpc::test::conformance::values as w;

fn ours<T: Wire>(v: &T) -> Vec<u8> {
    let mut out = Vec::new();
    v.put(&mut out);
    out
}

/// wRPC decodes `bytes` and encodes what it got.
fn through_wrpc<T: Decode + Encode>(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut buf = BytesMut::from(bytes);
    let value = match T::Decoder::default().decode(&mut buf) {
        Ok(Some(v)) => v,
        Ok(None) => return Err("wRPC wanted more bytes".into()),
        Err(_) => return Err("wRPC refused the bytes".into()),
    };
    if !buf.is_empty() {
        return Err(format!("wRPC left {} bytes", buf.len()));
    }
    let mut out = BytesMut::new();
    let mut enc = T::Encoder::default();
    value
        .encode(&mut enc, &mut out)
        .map_err(|_| "wRPC could not encode".to_string())?;
    Ok(out.to_vec())
}

fn point() -> impl Strategy<Value = Point> {
    (any::<u32>(), any::<u64>(), ".{0,12}", any::<bool>()).prop_map(|(x, y, t, m)| Point {
        x,
        y,
        r#type: t,
        r#match: m,
    })
}

fn shape() -> impl Strategy<Value = Shape> {
    prop_oneof![
        Just(Shape::Empty),
        point().prop_map(Shape::Dot),
        proptest::collection::vec(any::<u8>(), 0..300).prop_map(Shape::Bytes),
        proptest::collection::vec(point(), 0..4).prop_map(Shape::Many),
        ".{0,40}".prop_map(Shape::Named),
        proptest::option::of(prop_oneof![
            any::<u64>().prop_map(Ok),
            ".{0,8}".prop_map(Err),
        ])
        .prop_map(Shape::Nested),
        color().prop_map(Shape::Painted),
    ]
}

fn color() -> impl Strategy<Value = Color> {
    prop_oneof![Just(Color::Red), Just(Color::Green), Just(Color::Blue)]
}

fn numbers() -> impl Strategy<Value = Numbers> {
    (
        any::<i8>(),
        any::<i16>(),
        any::<i32>(),
        any::<i64>(),
        any::<u16>(),
        proptest::num::f32::ANY,
        proptest::num::f64::ANY,
        any::<char>(),
    )
        .prop_map(|(a, b, c, d, e, f, g, h)| Numbers {
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h,
        })
}

type More = (
    Vec<Color>,
    Perms,
    Vec<Numbers>,
    (u8, u8, u8, u8, u8, u8, u8, u8, char),
    Option<Token>,
);

/// The rest of a bag, past proptest's twelve-element tuples.
fn more() -> impl Strategy<Value = More> {
    (
        proptest::collection::vec(color(), 0..6),
        (0u64..1024).prop_map(Perms),
        proptest::collection::vec(numbers(), 0..4),
        (any::<[u8; 8]>(), any::<char>())
            .prop_map(|(b, c)| (b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], c)),
        proptest::option::of(proptest::collection::vec(any::<u8>(), 0..8).prop_map(Token)),
    )
}

fn bag() -> impl Strategy<Value = Bag> {
    (
        any::<bool>(),
        any::<u8>(),
        proptest::collection::vec(shape(), 0..5),
        (any::<u32>(), ".{0,8}"),
        (
            any::<u8>(),
            proptest::collection::vec(any::<u8>(), 0..20),
            proptest::option::of(any::<u64>()),
        ),
        proptest::option::of(proptest::option::of(any::<u8>())),
        prop_oneof![Just(Ok(())), shape().prop_map(Err)],
        prop_oneof![point().prop_map(Ok), Just(Err(()))],
        more(),
    )
        .prop_map(
            |(flag, small, shapes, pair, triple, maybe, outcome, ok_only, more)| Bag {
                flag,
                small,
                shapes,
                pair,
                triple,
                maybe,
                outcome,
                ok_only,
                colors: more.0,
                perms: more.1,
                numbers: more.2,
                nine: more.3,
                token: more.4,
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20000))]

    #[test]
    fn bags_agree_with_wrpc(v in bag()) {
        let bytes = ours(&v);
        prop_assert_eq!(&through_wrpc::<w::Bag>(&bytes).unwrap(), &bytes);
        prop_assert_eq!(ours(&finish::<Bag>(&bytes).unwrap()), bytes);
    }

    #[test]
    fn replies_agree_with_wrpc(ok in any::<bool>(), handle in proptest::collection::vec(any::<u8>(), 0..16), s in shape()) {
        let v: Result<Handle, Shape> = if ok { Ok(Handle(handle)) } else { Err(s) };
        let bytes = ours(&v);
        prop_assert_eq!(&through_wrpc::<Result<ResourceOwn<w::Handle>, w::Shape>>(&bytes).unwrap(), &bytes);
        prop_assert_eq!(finish::<Result<Handle, Shape>>(&bytes).unwrap(), v);
    }

    /// A valid bag with one byte changed, inserted or removed: our decoder
    /// may refuse but never panics, and it accepts exactly what wRPC accepts,
    /// re-encoding it the same way.
    #[test]
    fn mutated_bags_agree_with_wrpc(v in bag(), at in any::<prop::sample::Index>(), byte in any::<u8>(), how in 0..3u8) {
        let mut bytes = ours(&v);
        let i = at.index(bytes.len());
        match how {
            0 => bytes[i] = byte,
            1 => bytes.insert(i, byte),
            _ => { bytes.remove(i); },
        }
        let mine = finish::<Bag>(&bytes).map(|v| ours(&v));
        let theirs = through_wrpc::<w::Bag>(&bytes);
        prop_assert_eq!(mine.is_ok(), theirs.is_ok(), "acceptance differs");
        if let (Ok(a), Ok(b)) = (mine, theirs) {
            prop_assert_eq!(a, b);
        }
    }
}

/// A claimed length far past the input is refused without allocating it.
#[test]
fn a_huge_claimed_length_is_refused() {
    // shapes: list<shape> claiming u32::MAX elements, after flag and small.
    let bytes = [0u8, 0, 0xff, 0xff, 0xff, 0xff, 0x0f];
    assert!(finish::<Bag>(&bytes).is_err());
    // An over-long or overflowing integer.
    assert!(finish::<u32>(&[0xff, 0xff, 0xff, 0xff, 0x1f]).is_err());
    assert!(finish::<u64>(&[0xff; 11]).is_err());
}

/// Positive control: the harness does catch a disagreement. A shape case
/// past the last one is refused by both decoders.
#[test]
fn a_corrupted_case_is_refused_by_both() {
    let v = Bag {
        flag: true,
        small: 1,
        shapes: vec![Shape::Empty],
        pair: (2, "p".into()),
        triple: (3, vec![4], None),
        maybe: None,
        outcome: Ok(()),
        ok_only: Err(()),
        colors: vec![],
        perms: Perms::default(),
        numbers: vec![],
        nine: (0, 0, 0, 0, 0, 0, 0, 0, 'x'),
        token: None,
    };
    let mut bytes = ours(&v);
    // flag, small, list length 1, then the shape's case byte.
    assert_eq!(bytes[3], 0);
    bytes[3] = 9;
    assert!(finish::<Bag>(&bytes).is_err());
    assert!(through_wrpc::<w::Bag>(&bytes).is_err());
}

/// How each side treats a non-minimal integer (0 written in two bytes).
#[test]
fn non_minimal_integers() {
    let ours_accepts = finish::<u32>(&[0x80, 0x00]).is_ok();
    let mut buf = BytesMut::from(&[0x80u8, 0x00][..]);
    let wrpc_accepts = matches!(
        <u32 as Decode>::Decoder::default().decode(&mut buf),
        Ok(Some(0))
    );
    println!("non-minimal u32: ours accepts {ours_accepts}, wRPC accepts {wrpc_accepts}");
}

/// How often random bytes decode to a whole bag, so the fuzz is not empty.
#[test]
fn random_bytes_reach_the_decoder() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let strategy = proptest::collection::vec(any::<u8>(), 0..200);
    let (mut whole, mut deep) = (0, 0);
    for _ in 0..20000 {
        let bytes = strategy.new_tree(&mut runner).unwrap().current();
        let mut input = &bytes[..];
        match Bag::take(&mut input) {
            Ok(_) => whole += 1,
            Err(_) if input.len() + 8 < bytes.len() => deep += 1,
            Err(_) => {},
        }
    }
    println!("random bytes: {whole} whole bags, {deep} refused past the eighth byte, of 20000");
}

/// Found by the mutation test: a ten-byte u64 whose last byte carries bits
/// past 64 was silently truncated; wRPC refuses it, and so do we now.
#[test]
fn an_overflowing_u64_is_refused() {
    let mut bytes = vec![0x80; 9];
    bytes.push(0x02);
    assert!(finish::<u64>(&bytes).is_err());
    assert!(through_wrpc::<u64>(&bytes).is_err());
    let mut max = vec![0xff; 9];
    max.push(0x01);
    assert_eq!(finish::<u64>(&max).unwrap(), u64::MAX);
    assert_eq!(
        finish::<u32>(&[0xff, 0xff, 0xff, 0xff, 0x0f]).unwrap(),
        u32::MAX
    );
}

/// Each new encoding against wRPC at its edges.
#[test]
fn new_types_at_their_edges() {
    fn same<T: Wire, U: Decode + Encode>(v: T) {
        let bytes = ours(&v);
        assert_eq!(through_wrpc::<U>(&bytes).unwrap(), bytes);
        assert_eq!(ours(&finish::<T>(&bytes).unwrap()), bytes);
    }
    for v in [i64::MIN, -65, -64, -1, 0, 63, 64, i64::MAX] {
        same::<i64, i64>(v);
    }
    for v in [i32::MIN, -1, 0, i32::MAX] {
        same::<i32, i32>(v);
    }
    for v in [i16::MIN, -1, 0, i16::MAX] {
        same::<i16, i16>(v);
    }
    for v in [i8::MIN, -1, 0, i8::MAX] {
        same::<i8, i8>(v);
    }
    for v in [0, 127, 128, u16::MAX] {
        same::<u16, u16>(v);
    }
    for v in ['\0', 'a', 'é', '✓', '\u{10FFFF}'] {
        same::<char, char>(v);
    }
    for v in [0.0f64, -0.0, f64::INFINITY, f64::NAN, f64::MIN_POSITIVE] {
        same::<f64, f64>(v);
    }
    // A non-canonical NaN comes out canonical, as wRPC writes it.
    let odd = f32::from_bits(0x7fc0_0001);
    assert_eq!(ours(&odd), 0x7fc0_0000u32.to_le_bytes());
    for v in [Color::Red, Color::Blue] {
        same::<Color, w::Color>(v);
    }
    for v in [Perms(0), Perms(1), Perms(0x3ff)] {
        same::<Perms, w::Perms>(v);
    }
    // A bit past the tenth flag is refused by both.
    let bad = [0x00, 0x04];
    assert!(finish::<Perms>(&bad).is_err());
    assert!(through_wrpc::<w::Perms>(&bad).is_err());
}
