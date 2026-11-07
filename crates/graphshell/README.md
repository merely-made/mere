# Graphshell, Mere's side

The Graphshell protocol layer lives in
[merely-made/graphshell](https://github.com/merely-made/graphshell) (cycle breaks
plan, D7 to D9, 2026-11-06): `chirograph`, `sceno`, `scenotime`, `hest`,
`graphshell-endpoint` (the traits an authority implements, the `stdio`, `local`
and `network` carriers, the one `mere:script` WIT world), `graphshell-client`,
`script-wire` and `script-witgen`. It is the bottom layer and depends on nothing
in Mere. This directory holds what stays above it.

| Crate | Contents |
|---|---|
| `graphshell-door` | The application side of Djinn's first-party door, split out of `graphshell-endpoint` on 2026-11-06: `projection` (`AppBrokerClient`, `AppRouteCarrier`; feature `projection-door`), `resident_mere_proxy` (an endpoint that forwards over the door; `projection-door`), `projection_offer` (the portable provider messages; `projection-offer`), `projection_provider` (the native provider runner; `projection-provider`) and `profile` (`GraphshellIdentity`, `env_profile`; `profile`). Native only; empty by default. Depends on `graphshell-endpoint` and Ostiary, Castellan, Dramatis, Personae and Insigne |
| `graphshell-client-tests` | Consumer tests of `graphshell-client` that need Mere (`scenomise`, `scenograph`, `incipit`, `uxtree`) or Genet: the frozen realization's coastal-map, matrix, DOM, probe and AccessKit tests, the facet test and the shelfmark test. Unpublished |
| `graphshell-host` | The serving half: `admission` (the action triple, the hello builders, the sans-I/O admission and its action check), `carrier` (accept, `admit_accepted_session`, `admit_stream`), `line` (the codec over Tokio streams), `lifecycle` (`SessionAuthority`, `AdmittedEndpointContext`, `BindAdmittedSession`), `session_loop`, `session_notices`, `endpoint_catalog` (`ResidentEndpointCatalog`), `projection_host` (`ResidentProjectionHost`), `network_carrier` (dialling), `policy_projection`, `canary`, `resume` and `live_endpoint` (fixture endpoints), `view` and `sessions` (the projection receipt views and session switcher), and the WebRTC door (`webrtc_door`, `webrtc_join`, `webrtc_session`) |
| [`graphshell-mcp`](graphshell-mcp) | A stdio MCP server over one admitted route: projections as resources, advertised intents through one invoke tool. Library over any `Carrier`; the `mere-mcp` binary (feature `door`) dials djinn's application door as a named app, holds no key and mints no grant |

The carriers were the separate `graphshell-stdio`, `graphshell-local` and
`graphshell-network` crates until 2026-09-24. The door modules were the
app-facing half of Graphshell's `native` module until 2026-10-15 (Graphshell
stack lift plan, L1), the `door` module of `graphshell-endpoint` until
2026-11-06, and are now `graphshell-door`; an application reaches djinn's
custody and routes without the `graphshell` package. The WIT world is
materialised at `crates/script/wit/world.wit` (ignored by git) by
`document-host` and `app-host`, from `graphshell-endpoint`'s build metadata.

## Dependencies

| Crate | Depends on |
|---|---|
| `graphshell-door` | `graphshell-endpoint` (`projection`), `chirograph`; per feature `ostiary`, `serde`, `serde_json`, Tokio, `castellan-client`, `dramatis`, `insigne`, `personae` |
| `graphshell-host` | `chirograph`, `graphshell-client`, `graphshell-endpoint`, `sceno`, `scenotime`, `serde_json` |
| feature `admission` | adds `notochord`, `personae`, `insigne` |
| feature `webrtc-join` (default) | adds `admission`, `murm-webrtc` (its wasm-clean core), `serde`, `thiserror`, `getrandom` |
| feature `native` | adds `webrtc-join`, `graphshell-endpoint` (`network`), `mere-transport` (`notochord`), `pandect`, `base64`, `rand_core`, Tokio |
| feature `webrtc-session` | adds `native` and `mere-webrtc-carrier/native` (str0m) |

`graphshell-host` builds for `wasm32-unknown-unknown` with its default features
(admission and the WebRTC join), with `admission` alone and with none; `native`
is native only. `graphshell-door`'s modules are compiled out on that target.
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
host and its native-messaging envelope, the device broker, the local session,
the identity endpoint, `ResidentIdentity` and the sync hosts) are Djinn's
`door` and `sync` modules since the stack lift plan's P5.
[`ports/graphshell`](../../ports/graphshell), the reference application,
composes these crates, and they do not depend on it.
