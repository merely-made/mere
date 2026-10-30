# Bounded Gemot live-interaction source consumer

This isolated workspace compiles the existing `ports/moot/src/coop.rs` and the
optional `ocapn_coop` adapter directly. It avoids Mere's browser/rendering and
other port dev dependencies while exercising the same source. It adds no
alternate governance implementation or persistent activity store.

Run from the repository root with the committed supplier sources:

```sh
cargo test --manifest-path testing/moot-ocapn-coop/Cargo.toml --locked
cargo clippy --manifest-path testing/moot-ocapn-coop/Cargo.toml --all-targets --no-deps --locked -- -D warnings
```

Tests use synthetic identities and real signed Gemot operations. They cover
current membership and typed grant checks, service revocation/expiry, read-only
membership, exact-head stale requests, host-versus-submitter attribution,
independent replica intake of the normal outbound operation, immutable session
context, bounded callbacks, disconnect cleanup and queue backpressure.
A native test uses actual `P2pandaTransport::builder_for` endpoints with scoped
keys distinct from the stable application roots, native secure admission, and
then the real CapTP adapter view/proposal path. The fixture owns these endpoints;
the adapter creates none.

The admitted-channel input in the small in-memory tests is an explicit local
fixture, not physical device or browser qualification. The native composition
is a loopback software acceptance case; wider hosts, browser UI, radio behavior,
and product integration retain their separate gates.

During supplier development, local checkout overrides may be supplied on the
Cargo command line. They are not part of this workspace's committed manifest.
