//! D15: our generator writes wRPC's value encoding. The same values encode
//! to the same bytes through both routes, and each decodes the other's.

use c0_route_a::bindings as a;
use c0_route_a::rt::{Wire, finish};
use c0_route_b::consumer_bindings::mere::script::projection_session as b;
use wit_bindgen_wrpc::bytes::{Bytes, BytesMut};
use wit_bindgen_wrpc::tokio_util::codec::Decoder;
use wit_bindgen_wrpc::wrpc_transport::{Decode, Encode, ResourceOwn};

fn wrpc<T: Encode>(value: T) -> Vec<u8> {
    let mut out = BytesMut::new();
    let mut enc = T::Encoder::default();
    assert!(value.encode(&mut enc, &mut out).is_ok());
    out.to_vec()
}

fn unwrpc<T: Decode>(bytes: &[u8]) -> T {
    let mut buf = BytesMut::from(bytes);
    let Ok(Some(value)) = T::Decoder::default().decode(&mut buf) else {
        panic!("no whole value")
    };
    assert!(buf.is_empty(), "trailing bytes");
    value
}

fn ours<T: Wire>(value: &T) -> Vec<u8> {
    let mut out = Vec::new();
    value.put(&mut out);
    out
}

fn failures() -> Vec<(a::Failure, b::Failure)> {
    let long = "x".repeat(200);
    vec![
        (a::Failure::Revoked, b::Failure::Revoked),
        (a::Failure::Expired, b::Failure::Expired),
        (a::Failure::TooLarge(0), b::Failure::TooLarge(0)),
        (a::Failure::TooLarge(300), b::Failure::TooLarge(300)),
        (
            a::Failure::TooLarge(u64::MAX),
            b::Failure::TooLarge(u64::MAX),
        ),
        (
            a::Failure::Refused(String::new()),
            b::Failure::Refused(String::new()),
        ),
        (
            a::Failure::Refused("ä — ✓".into()),
            b::Failure::Refused("ä — ✓".into()),
        ),
        (
            a::Failure::Malformed(long.clone()),
            b::Failure::Malformed(long),
        ),
    ]
}

fn same_failure(x: &a::Failure, y: &b::Failure) -> bool {
    format!("{x:?}").replace("Failure::", "") == format!("{y:?}").replace("Failure::", "")
}

#[test]
fn failures_encode_alike() {
    for (x, y) in failures() {
        let bytes = ours(&x);
        assert_eq!(bytes, wrpc(y.clone()), "{x:?}");
        assert_eq!(finish::<a::Failure>(&wrpc(y.clone())).unwrap(), x);
        assert!(same_failure(&x, &unwrpc::<b::Failure>(&bytes)));
    }
}

#[test]
fn notices_encode_alike() {
    let cases = [(0, 0), (127, 128), (u64::MAX, 1 << 35)];
    assert_eq!(ours(&None::<a::Notice>), wrpc(None::<b::Notice>));
    for (epoch, revision) in cases {
        let x = Some(a::Notice { epoch, revision });
        let y = Some(b::Notice { epoch, revision });
        let bytes = ours(&x);
        assert_eq!(bytes, wrpc(y), "{x:?}");
        let back: Option<b::Notice> = unwrpc(&bytes);
        assert_eq!(back.map(|n| (n.epoch, n.revision)), Some((epoch, revision)));
    }
}

#[test]
fn replies_encode_alike() {
    for payload in [Vec::new(), vec![0xa5; 200], vec![1; 70_000]] {
        let x: Result<Vec<u8>, a::Failure> = Ok(payload.clone());
        let y: Result<Bytes, b::Failure> = Ok(Bytes::from(payload));
        assert_eq!(ours(&x), wrpc(y));
    }
    for (x, y) in failures() {
        assert_eq!(ours(&Err::<Vec<u8>, _>(x)), wrpc(Err::<Bytes, _>(y)));
    }
    assert_eq!(
        ours(&Ok::<(), a::Failure>(())),
        wrpc(Ok::<(), b::Failure>(()))
    );
}

#[test]
fn an_opened_session_encodes_alike() {
    let handle = 7u64.to_le_bytes().to_vec();
    let x: Result<(a::Session, Vec<u8>), a::Failure> =
        Ok((a::Session(handle.clone()), vec![1, 2, 3]));
    let y: Result<(ResourceOwn<b::Session>, Bytes), b::Failure> =
        Ok((ResourceOwn::from(handle), Bytes::from_static(&[1, 2, 3])));
    let bytes = ours(&x);
    assert_eq!(bytes, wrpc(y));
    assert_eq!(
        finish::<Result<(a::Session, Vec<u8>), a::Failure>>(&bytes).unwrap(),
        x
    );
}
