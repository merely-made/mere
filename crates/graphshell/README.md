# Graphshell session stack

Mere's reusable remote-session machinery: a versioned session protocol, the
client state above it, the traits an authority implements, and the carriers,
one feature each on `graphshell-endpoint`.

| Crate | Contents |
|---|---|
| `chirograph` | Session messages (score, scene, presentation, resource, resume, status, intent); the `Carrier` trait; `CarrierError`; `line`, the NDJSON codec and `LineAssembler` every end shares |
| `graphshell-client` | `ClientState` (snapshots, diffs, resume, resource cache, cache policy), `RetainedEndpointSession`, `ActionDraft`, `SessionDriver` |
| `graphshell-endpoint` | `ProjectionCatalog`, `ProjectionSource`, `PresentationSource`, `IntentSink`, `ResumableProjectionSource`, `ProjectionNoticeSource`, `LiveViewReferenceGate`, `dispatch_common`; and the carriers below |
| `graphshell-endpoint::stdio` (feature `stdio`) | `StdioCarrier` plus `serve_basic`, `serve_resumable`, `serve_resumable_notifying`: NDJSON over a child process's standard streams |
| `graphshell-endpoint::local` (feature `local`) | `LocalCarrier`: an endpoint hosted in this process, still round-tripping the wire encoding |
| `graphshell-endpoint::network` (feature `network`) | `NetworkCarrier`, `CarrierRuntime`: NDJSON over any `AsyncRead + AsyncWrite` |
| `graphshell-host` | The serving half: `admission` (the action triple, the hello builders, the sans-I/O admission and its action check), `carrier` (accept, `admit_accepted_session`, `admit_stream`), `line` (the codec over Tokio streams), `lifecycle` (`SessionAuthority`, `AdmittedEndpointContext`, `BindAdmittedSession`), `session_loop`, `session_notices`, `endpoint_catalog` (`ResidentEndpointCatalog`), `projection_host` (`ResidentProjectionHost`), `network_carrier` (dialling), `policy_projection`, `resume` and `live_endpoint` (fixture endpoints), and the WebRTC door (`webrtc_door`, `webrtc_join`, `webrtc_session`) |
| `graphshell-endpoint::door` (feature `door`, native only) | The application side of the first-party door: `app_admission` (hello, `AppId`, route grants), `route` (`ResidentEndpointRoute`), `wire` and `frame` (the envelope and native-messaging frames both halves speak), `app_client` (`AppBrokerClient`, `AppRouteCarrier`), `custody`, `custody_client`, `custody_identity`, `quarantine`, `local_endpoint`, `tasks`, `identity` (the `dramatis::view` types) and `profile` (`GraphshellIdentity`, `env_profile`) |

The carriers were the separate `graphshell-stdio`, `graphshell-local` and
`graphshell-network` crates until 2026-09-24. The `door` modules were the
app-facing half of Graphshell's `native` module until 2026-10-15 (Graphshell
stack lift plan, L1); an application now reaches djinn's custody and routes
without the `graphshell` package.

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
| feature `door` | adds `dramatis`, `personae`, `insigne`, `chatelaine`, `pandect`, `serde`, `serde_json`, `base64`, `rand_core`, `thiserror`, `tracing`, `uuid`, `zeroize`, Tokio (adds `net`, `sync`, `time`) and, on Windows, `windows-sys` |
| `graphshell-host` | `chirograph`, `graphshell-client`, `graphshell-endpoint`, `sceno`, `scenotime`, `serde_json` |
| feature `admission` | adds `notochord`, `personae`, `insigne` |
| feature `webrtc-join` (default) | adds `admission`, `mere-webrtc-carrier` (its wasm-clean core), `serde`, `thiserror`, `getrandom` |
| feature `native` | adds `webrtc-join`, `graphshell-endpoint` (`door`, `network`), `mere-transport` (`notochord`), `pandect`, `base64`, `rand_core`, Tokio |
| feature `webrtc-session` | adds `native` and `mere-webrtc-carrier/native` (str0m) |

`chirograph`, `-client`, and `-endpoint` with no carrier feature build for
`wasm32-unknown-unknown`; the `door` module is compiled out on that target.
`graphshell-host` builds for it with its default features (admission and the
WebRTC join), with `admission` alone and with none; `native` is native only.
`NetworkCarrier`'s `Carrier` methods block, so they must run off a runtime
worker thread.

## Layers

Every carrier ends at the same seam: an admitted byte stream, or a line
channel where the platform gives no stream, plus the ingress facts admission
read (transport agnosticism note, ruling T1). Above it the wire is NDJSON on
every carrier (T2): `chirograph::line` is the codec, which the stdio and
network carriers and `SessionDriver` use directly, and `graphshell_host::line`
runs it over Tokio streams. Admission has one entry,
`graphshell_host::carrier::admit_stream`; a transport's accepted session
reaches it through `admit_accepted_session`, an in-process door states its
own facts, and a carrier that hands over the hello as bytes uses the sans-I/O
`admission::admit_session`. All of them end in `admission::serves_action`.

## Not in these crates

The servers Djinn runs (the application door's broker, the browser door's
host, the device broker, the local session, `ResidentIdentity` and the sync
hosts) are Djinn's `door` and `sync` modules since the stack lift plan's P5.
The browser carrier and the identity endpoint stay in
[`ports/graphshell`](../../ports/graphshell), the reference application: it
composes these crates, and they do not depend on it.
