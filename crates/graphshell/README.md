# Graphshell session stack

Mere's reusable remote-session machinery: a versioned session protocol, the
client state above it, the traits an authority implements, and the carriers,
one feature each on `graphshell-endpoint`.

| Crate | Contents |
|---|---|
| `chirograph` | Session messages (score, scene, presentation, resource, resume, status, intent); the `Carrier` trait; `CarrierError` |
| `graphshell-client` | `ClientState` (snapshots, diffs, resume, resource cache, cache policy), `RetainedEndpointSession`, `ActionDraft` |
| `graphshell-endpoint` | `ProjectionCatalog`, `ProjectionSource`, `PresentationSource`, `IntentSink`, `ResumableProjectionSource`, `ProjectionNoticeSource`, `LiveViewReferenceGate`, `dispatch_common`; and the carriers below |
| `graphshell-endpoint::stdio` (feature `stdio`) | `StdioCarrier` plus `serve_basic`, `serve_resumable`, `serve_resumable_notifying`: NDJSON over a child process's standard streams |
| `graphshell-endpoint::local` (feature `local`) | `LocalCarrier`: an endpoint hosted in this process, still round-tripping the wire encoding |
| `graphshell-endpoint::network` (feature `network`) | `NetworkCarrier`, `CarrierRuntime`: NDJSON over any `AsyncRead + AsyncWrite` |

The carriers were the separate `graphshell-stdio`, `graphshell-local` and
`graphshell-network` crates until 2026-09-24.

`Carrier::request` returns `Result<CarrierResponseBody, CarrierError>`.
`CarrierError` is `Refused` (the session is intact) or `Disconnected` (the
session is finished). `StdioCarrier`, `LocalCarrier`, and `NetworkCarrier` each
implement `Carrier`.

## Dependencies

| Crate | Depends on |
|---|---|
| `chirograph` | `sceno`, `scenotime`, `serde`, `serde_json`, `blake3`, `base64` |
| `graphshell-client` | `chirograph`, `sceno`, `scenotime`, `serde`, `serde_json` |
| `graphshell-endpoint` | `chirograph` |
| feature `stdio` | adds `serde_json`; uses `std::process` |
| feature `local` | adds `serde`, `serde_json` |
| feature `network` | adds `serde_json` and Tokio (`io-util`, `rt`, `rt-multi-thread`) |

`chirograph`, `-client`, and `-endpoint` with no carrier feature build for
`wasm32-unknown-unknown`. `NetworkCarrier`'s `Carrier` methods block, so they
must run off a runtime worker thread.

## Not in these crates

Admission: which peers may open a session, under what grant, and over which
ALPN. That lives in [`ports/graphshell`](../../ports/graphshell), along with the
serve loops for admitted sessions, `ResidentEndpointCatalog`, and
`ResidentProjectionHost`. That port is the reference application: it composes
these crates, and they do not depend on it.
