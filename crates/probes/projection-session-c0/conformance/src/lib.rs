//! Measures our generator against outside references: wRPC's encoder on
//! random values, random bytes into our decoders, and WIT coverage.

#[allow(clippy::all, dead_code, unused)]
pub mod ours {
    use c0_route_a::rt::*;
    include!(concat!(env!("OUT_DIR"), "/ours.rs"));
}

#[allow(clippy::all, dead_code, unused)]
pub mod wrpc {
    include!(concat!(env!("OUT_DIR"), "/wrpc.rs"));
}
