//! The component guest exchanges the same calls and payloads as the native
//! routes, and values cross Wasmtime unchanged. Build the guest first:
//! `cd guest && cargo build --release --target wasm32-wasip2`.

use c0_contract::provider::TestProvider;
use c0_contract::script::{CALLS_BEFORE_REVOCATION, Direct, run};
use c0_host::Guest;
use c0_host::mere::script::projection_session::{Failure, Notice};

fn guest() -> Guest<TestProvider> {
    let wasm = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../guest/target/wasm32-wasip2/release/c0_guest.wasm");
    Guest::load(&wasm, TestProvider::new(CALLS_BEFORE_REVOCATION)).expect("guest loads")
}

#[test]
fn the_guest_matches_the_reference_transcript() {
    let reference = run(&mut Direct(TestProvider::new(CALLS_BEFORE_REVOCATION)));
    let mut g = guest();
    let transcript = g.bindings.call_run(&mut g.store).unwrap();
    assert_eq!(transcript, reference);
    // The guest's drop reached the provider as Close.
    assert_eq!(g.store.data().provider.closed, vec![1]);
}

#[test]
fn values_cross_wasmtime_unchanged() {
    let mut g = guest();
    let failures = [
        Failure::Revoked,
        Failure::Expired,
        Failure::TooLarge(u64::MAX),
        Failure::Refused("ä — ✓".into()),
        Failure::Malformed("x".repeat(200)),
    ];
    for f in failures {
        let back = g.bindings.call_echo_failure(&mut g.store, &f).unwrap();
        assert_eq!(format!("{back:?}"), format!("{f:?}"));
    }
    for n in [
        None,
        Some(Notice {
            epoch: 0,
            revision: 0,
        }),
        Some(Notice {
            epoch: u64::MAX,
            revision: 1 << 35,
        }),
    ] {
        let back = g.bindings.call_echo_notice(&mut g.store, n).unwrap();
        assert_eq!(
            back.map(|n| (n.epoch, n.revision)),
            n.map(|n| (n.epoch, n.revision))
        );
    }
}
