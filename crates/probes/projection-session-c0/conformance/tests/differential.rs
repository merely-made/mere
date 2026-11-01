//! Random values through both encoders. Our bytes must decode under wRPC
//! and re-encode to the same bytes, and decode back under ours to the
//! value we started with. Random bytes must never panic our decoders.

use c0_conformance::ours::{Bag, Handle, Point, Shape};
use c0_conformance::wrpc::c0::conformance::values as w;
use c0_route_a::rt::{Wire, finish};
use proptest::prelude::*;
use wit_bindgen_wrpc::bytes::BytesMut;
use wit_bindgen_wrpc::tokio_util::codec::Decoder;
use wit_bindgen_wrpc::wrpc_transport::{Decode, Encode, ResourceOwn};

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
    ]
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
    )
        .prop_map(
            |(flag, small, shapes, pair, triple, maybe, outcome, ok_only)| Bag {
                flag,
                small,
                shapes,
                pair,
                triple,
                maybe,
                outcome,
                ok_only,
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20000))]

    #[test]
    fn bags_agree_with_wrpc(v in bag()) {
        let bytes = ours(&v);
        prop_assert_eq!(&through_wrpc::<w::Bag>(&bytes).unwrap(), &bytes);
        prop_assert_eq!(finish::<Bag>(&bytes).unwrap(), v);
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
