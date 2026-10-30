# Independent door consumers

Each directory is its own Cargo workspace. These fixtures exercise the endpoint
without the main workspace or endpoint dev dependencies unifying its features.

- `custody`: native custody client, with a legacy locked refusal test.
- `custody-types`: native custody DTOs and serializers without a connection.
- `projection`: native projection client, with a legacy hello test.
- `portable`: labels, typed envelopes and synchronous framing, with an installed
  v2 hello fixture. Also checked for `wasm32-unknown-unknown`.
- `host`: the portable Graphshell host with default features disabled. This is
  a compile gate, including Wasm, for its unconditional projection contracts.

Run `python3 support/fixtures/custody-projection/verify.py OUTPUT_DIRECTORY`
through the machine's bounded build runner. The Wasm target must be installed.
The script retains target-filtered metadata, normal/build dependency edges,
and expected missing-API compiler diagnostics. Each positive test runs before
its negative check; an unrelated compiler failure cannot satisfy a negative
check. `forbidden-api` is intentionally uncompilable.

The four door fixtures run one behavioral test each. They supplement the
endpoint and Djinn suites; an empty host test suite is not behavior evidence.
Wasm checks establish compilation, not browser IPC. Native custody and socket
clients are unavailable on Wasm. Windows requires a separate executor receipt.
