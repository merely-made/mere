# Transport agnosticism and composition note

**Status (2026-10-15):** design note for Mark's review; nothing implemented.
Source was read at `9cc711fbc` (Mere main), Knot at its local checkout, the
external `mere-transport` and `mere-webrtc-carrier` crates at Murm `b49df894`
and `3d9e799f` (their `transport` sources are identical; `webrtc-carrier`
differs only in `browser/initiator.rs`), and Turnstone at `de56f958` on
GitHub. Line numbers are as read at those revisions. Nothing was built for
this note.

**Origin.** P2a of the
[Graphshell stack lift plan](../../archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md)
moved the remote overmap client into `mere-overmap`. Its receipt needs a mere
route that a remote device can dial, and Djinn registers `mere/<domain>` only
on the local application door. Mark chose to design that first and widened
the question twice: "look at the transports across the stack. consider how
we keep it agnostic", then "and consider composition...? lot of those have
similar profiles/methods...". He doubts one transport is enough.

**What this note decides and what it does not.** It surveys every carrier
the stack has, tabulates what each implements itself, names the duplication,
and proposes layers that carriers assemble instead of re-implementing. The
remote mere route is the worked example. It ends with ranked
recommendations and Mark's rulings (§9). It moves no code.

## 1. The worked example: a remote mere route

Turnstone's overmap pane should mount another device's mere as `mere-view`
and show its sessions, with Turnstone's own identity and window (L14's
receipt). `mere_overmap::route_session::MereRouteSession` is already
transport-blind: it takes lines (`on_line`) and gives lines
(`take_outgoing`), over `graphshell_client::SessionDriver`. What the receipt
still needs is underneath it:

1. **A route on a carrier a remote peer can reach.** `MereRoutes::serve`
   (`ports/mere-resident/src/resident_mere.rs:263-332`) registers `mere/<domain>` and
   `ambient/mere/<domain>` on the `AppEndpointCatalog`, which wraps the same
   `ResidentEndpointCatalog` that `ResidentProjectionHost` serves
   (`ports/graphshell/src/native/app_broker.rs:134`,
   `native/projection_host.rs:203-206`). The registration is reusable; no
   dialable accept loop exposes it. The Djinn product binary runs no
   projection host over QUIC; the only Djinn projection host is Distillery's
   chronicle over WebRTC (`ports/mere-resident/src/resident_distillery.rs:459-474`).
2. **An application identity for the dialer.** `AccessEndpoint::open`
   refuses a context with no application: "mere access requires an admitted
   application" (`resident_mere.rs:359-361`). Only the app door binds one
   (`app_broker.rs:326`, `with_application`); a session admitted over QUIC or
   WebRTC carries a subject and no application
   (`ports/graphshell/src/lifecycle.rs:121-144`). The disclosed view state is
   keyed by application too (`SessionGraphV1.application`), which is why the
   lifted panel now takes the application it reads as.
3. **A native line pump.** The browser has one (`web_rtc_link.rs` feeds
   `RemoteSession`); natively, `dial_projection_session`
   (`network_carrier.rs:68-79`) returns an admitted byte stream, and
   `NetworkCarrier` wraps that as a blocking `chirograph::Carrier`, which the
   line-driven panel does not take. A stream-to-lines pump for
   `SessionDriver` is the missing piece.
4. **Notices pushed end to end**, so the pane updates without polling across
   hops (§4 item 11).

Turnstone already has the dial side's ingredients: it serves place
projections with `accept_projection_session`, `ResidentEndpointCatalog` and
`serve_admitted_session_notifying` over a per-place `P2pandaTransport`
(Turnstone `src/place/projection_host.rs:7-49`), and its delegation chain
already spends one hop at dial time so the hello's subject equals the
carrier-proven peer (same file, module docs).

## 2. The carriers

| Carrier | Carries | Admission and identity bind | Session layer above it |
|---|---|---|---|
| **stdio** (`graphshell-endpoint` `stdio`) | Projection sessions to a child process | None: Open is refused as "does not authenticate" (`stdio.rs:199-242`); trust is the parent spawning the child | Transport-blind (`dispatch_common`) |
| **local** (`graphshell-endpoint` `local`) | In-process projection sessions, JSON round-tripped on purpose (`local.rs:15-26`) | None; session plane refused (`local.rs:101-110`) | Transport-blind |
| **network NDJSON** (`graphshell-endpoint` `network`) | Projection sessions over any `AsyncRead + AsyncWrite` | Admission-ignorant by design (`network.rs:7-19`); the stream arrives admitted | Transport-blind |
| **QUIC projection** (`ports/graphshell` `carrier`, `network_carrier`, `session_loop`, K2 `native/projection_host`) | Projection sessions over iroh QUIC through `mere-transport`'s `P2pandaTransport` | `notochord::admit_session` over the stream after ALPN `mere/graphshell/v1` (`admission.rs:51`, `carrier.rs:168-199`); the hello's subject must equal the carrier-proven peer; session id from the transcript (`lifecycle.rs:23-29`); authority re-read per request (`session_loop.rs:217-235`) | Transport-blind after admission: the serve loop takes any stream |
| **WebRTC door** (`webrtc_door`, `webrtc_join`, `webrtc_session`, `webrtc_browser`; frames from `mere-webrtc-carrier`) | Projection sessions, browser to host | Invite redemption, host-signed DTLS-fingerprint challenge, then sans-I/O `notochord::admit` with a link-only binding (`webrtc_door.rs:519-552`, `604-606`); no peer id | Transport-blind after `stream_over_frames` turns frames into a `DuplexStream` carrying NDJSON (`webrtc_session.rs:18-29`) |
| **Browser native messaging** (`browser_carrier`, `native/browser_host`) | Projection sessions, extension to native host | Extension-id allow-list, `BrowserChallenge` nonce, `LocalLink` shared link; the host then runs notochord for the browser over an in-process `tokio::io::duplex` (`browser_carrier.rs:584-631`) | Blind inside the duplex; the envelope (`BrowserMessage`, `BrowserHostMessage`) is not |
| **App door** (UDS / named pipe; `native/app_broker`, `app_client`, `local_endpoint`, `local_session`) | Routes for first-party apps, plus custody calls | `AppHello` v1/v2, the same `BrowserChallenge`, `LocalLink`, `admit_local_client` over a duplex; the only place an application id is bound (`app_broker.rs:326`) | Blind inside the duplex; the envelope (`AppMessage`) is door-specific |
| **Device broker relay** (`native/device_broker`) | Native-messaging bytes copied onto an owner-only socket (`device_broker.rs:148-168`) | `DeviceBrokerHello` and the allow-list (`device_broker.rs:204-247`), then as native messaging | Inherits the browser carrier |
| **`ResidentMereProxy`** (`native/resident_mere_proxy`) | An endpoint whose verbs forward over `AppBrokerClient` (`resident_mere_proxy.rs:157-218`) | Inherits the app door's admission | Composes one carrier into another's catalog |
| **Sync lanes** (`personal_sync`, `carriage`, Djinn's Knot and Distillery lanes) | Signed p2panda operations (CBOR), gossip and log sync, iroh blobs | Per-graph transport key (`native/personal_sync_host.rs:195`), policy-before-insert per operation (`personal_sync.rs:931`); no session | Not chirograph; shares only the byte plane |
| **Mesh admitted ingress** (`crates/mesh/mesh`) | Retained mesh content | ALPN `mere/mesh-retained-admitted/v1` (`ingress/mod.rs:33`), notochord frames, sans-I/O `notochord::admit` (`ingress/session.rs:80-110`) | Own wire |
| **Knot publish** (Knot `publish_carrier`, `publish_host`, `publish_client`) | Publication over ALPN `mere/knot-publish/v1` | Noise responder, then notochord, then an action check (`publish_carrier.rs:89-129`); "intentionally parallel" to Graphshell's carrier (`:7-12`) | Own wire (postcard over notochord frames) |

Below all of these, `mere-transport` (Murm `native/crates/transport`) already
abstracts the byte plane: `Transport` gives ALPN-keyed `AsyncRead +
AsyncWrite` streams (`transport.rs:52-88`), with `P2pandaTransport` (iroh
QUIC, relays, hole-punching), `MemoryTransport`, Reticulum, and Noise as a
session layer "that composes over any byte stream" (`noise.rs` module docs).
`AcceptedSession<S>` and `IngressContext` (`accepted.rs:37-136`) carry the
facts admission needs, with `TransportKind` naming Memory, P2panda,
Reticulum, Noise and WebRtc. The byte plane is agnostic already; what is not
is everything between it and `SessionCore`.

## 3. What each carrier implements itself

| Carrier | Framing | Hello and admission | Correlation | Notices | Resume | Chunking | Backpressure and lifetime |
|---|---|---|---|---|---|---|---|
| stdio | NDJSON (`stdio.rs:244-256`) | none | own `next_id` matcher, notice queue (`stdio.rs:295-333`) | server polls `poll_notice` on a helper thread with `recv_timeout` (`stdio.rs:73-142`) | `serve_resumable` | `dispatch_common` | `sync_channel(1)`; child reaped on drop (`stdio.rs:369-422`) |
| local | none (serde round trip) | none | direct call | client pulls the endpoint (`local.rs:118-120`); `wait_for_notice` refused | n/a | `dispatch_common` | none |
| network NDJSON | NDJSON | none (stream pre-admitted) | own matcher (`network.rs:154-184`, `241-253`) | interleaved `CarrierOutput` | via `SessionCore` | `dispatch_common` | owns or borrows a tokio runtime (`network.rs:63-85`) |
| QUIC projection | NDJSON over a QUIC stream (`session_loop.rs:9-13`) | notochord over the stream | client: `NetworkCarrier` or `SessionDriver` | server: poller drained before each read plus a timed read arm (`session_loop.rs:177-191`, `session_notices.rs:46-80`) | new admission, then `Resume` (`lifecycle.rs:23-29`) | `dispatch_common` | `max_sessions` live count, `LiveSession` guard (`projection_host.rs:187-270`); Suspend and Close end the loop |
| WebRTC | join JSON, binary hello frames, then 4-byte big-endian frames of at most 64 KiB (`webrtc-carrier` `frame.rs:30-48`), pumped to NDJSON | invite, challenge, sans-I/O notochord | `SessionDriver` / `RemoteSession` | same server loop | `peer_rejoin` re-admits (`webrtc_join.rs:210-231`) | chirograph chunks; lines cut at frame size (`webrtc_browser.rs:386-397`), reassembled by `LineAssembler` (`webrtc_join.rs:363-399`) | high/low water marks (`webrtc-carrier` `backpressure.rs:28-38`); ordered close (`webrtc_session.rs:296-320`) |
| native messaging | 4-byte native-endian prefix, 1 MiB cap (`browser_carrier.rs:49`, `449`, `478`); sync and async copies (`:429-546`) | allow-list, challenge, `LocalLink`, notochord on a duplex | own matcher, `BrowserSessionClient` (`:634-719`) | **not forwarded**: `serve_native_messages` relays responses only (`native/browser_host.rs:250-267`) | none; ends on Close or Suspend | 512 KiB chunks sized to fit the 1 MiB cap (`chirograph/src/lib.rs:626-634`) | duplex 256 KiB (`browser_carrier.rs:594`) |
| app door | the native-messaging prefix again (`app_broker.rs:36-37`) | `AppHello`, the same challenge, `LocalLink`, notochord on a duplex | `AppBrokerClient`, one `next_id` for routes and custody (`app_client.rs:237-283`) | **pulled**: `TakeNotice` polls for 1 ms, `WaitNotice` blocks (`app_broker.rs:394-411`) | a new open | chirograph chunks | socket mode 0600, pipe token check, 2 s busy retry (`local_endpoint.rs:36-64`, `131-210`) |
| `ResidentMereProxy` | inherits the app door | inherits | per-verb forwarding | **polled again** (nested) | inherits | inherits | own thread, mpsc, fixed 5 s timeout (`resident_mere_proxy.rs:26-136`) |
| sync lanes | CBOR p2panda operations | per-operation policy | none (gossip) | gossip push | log sync re-offers refused operations (`personal_sync_host.rs:344-346`) | 256 events, 1 MiB per operation (`personal_sync.rs:44-49`); iroh blobs | shutdown waits (`personal_sync_host.rs:1072-1083`) |
| mesh, Knot publish | notochord length frames (+ postcard for Knot) | own accept path each | own | n/a | n/a | n/a | own |

The chirograph contract under all the projection carriers is already shared:
`CarrierRequestBody` (Discover, Snapshot, Resource, ResourceChunk, Resume,
Intent, Open, Close, Suspend; `chirograph/src/lib.rs:1117-1133`),
`CarrierRequest { id }`, `CarrierNotice { session, epoch, revision }`,
`CarrierOutput` (response or notice, untagged; `:1177-1183`), the blocking
`Carrier` trait (`:1200-1216`) and the two-variant `CarrierError`. The
duplication is in how each carrier gets bytes to and from that contract.

## 4. Duplication

1. **NDJSON server loops.** `stdio::serve_with` and
   `serve_resumable_notifying` (`graphshell-endpoint/src/stdio.rs:73-180`)
   against `serve_admitted_session_with` (`ports/graphshell/src/session_loop.rs:151-282`).
   Line writers twice (`stdio.rs:244-256`, `session_loop.rs:302-329`). Notice
   polling three ways: a thread with `recv_timeout`, `tokio::time::timeout`,
   and a direct pull (`local.rs:118`).
2. **NDJSON clients with id matching and a notice queue.** `StdioCarrier`
   (`stdio.rs:295-367`), `NetworkCarrier` (`network.rs:115-253`),
   `BrowserSessionClient` (`browser_carrier.rs:650-718`), `SessionDriver`
   (`graphshell-client/src/driver.rs:153-229`), and receipt copies in
   `g5_peer/connect.rs` and `h6_transfer_peer/destination.rs:41-80`.
3. **Dial then `initiate_session`.** `network_carrier.rs:68-79`, the g5 and
   h6 receipt peers (`connect.rs:168-205`, `destination.rs:344-370`), and
   Knot's `publish_client.rs:121-145` and `publish_host.rs:534-571`.
4. **Admission entry** (facts, notochord, action check): `carrier.rs:168-199`
   (stream), `admission.rs:134-154` (sans-I/O, from `webrtc_door.rs:536`),
   `browser_carrier.rs:584-631` (in-process duplex), mesh
   `ingress/session.rs:80-110`, Knot `publish_carrier.rs:89-129`. The action
   check is `serves_action` (`carrier.rs:188`, `admission.rs:143`) and Knot's
   `serves_publish_action`.
5. **Length-prefixed framing, three conventions.** Native messaging is
   native-endian (`browser_carrier.rs:449`), WebRTC frames are big-endian
   (`webrtc-carrier` `frame.rs:88`), notochord frames are a third
   implementation (mesh, Knot).
6. **Challenge-nonce hellos.** `BrowserChallenge` with `BrowserMessage::Connect`
   (`browser_carrier.rs:164-204`); the app door's `AppMessage::Connect` with
   the same three fields (`app_broker.rs:85-91`); `DeviceBrokerHello`
   (`device_broker.rs:33`); WebRTC's Open, Challenge, Redeem, Grant
   (`webrtc_join.rs:56-100`).
7. **Door relay loops.** Request, `client.request`, Close/Suspend check,
   write: `browser_host.rs:250-267` and `app_broker.rs:377-392`.
8. **`resource_chunk` defaults twice.** `graphshell-endpoint/src/lib.rs:75-88`
   and `ResidentEndpoint` (`native/endpoint_catalog.rs:43-55`).
9. **Endpoint-over-carrier by hand.** `ResidentMereProxy` matches each verb
   (`resident_mere_proxy.rs:157-218`); its `unexpected` helper is copied from
   `graphshell-client/src/session.rs:386-400` into `app_client.rs:400-410`.
10. **Blocking over async, three ways.** `CarrierRuntime`
    (`network.rs:63-85`), `AppRouteCarrier` owning a `Runtime`
    (`app_client.rs:311-372`), `ResidentMereProxy`'s thread and mpsc.
11. **Ways to ring the bell.** Discover sent only to pump notices
    (`graphshell-client/src/core.rs:470`, `session.rs:212-230`), `TakeNotice`
    and `WaitNotice`, `BrowserSessionClient::poll_notice` (`:683-694`). In the
    C4 chain a notice crosses a 50 ms poll in Djinn, a 1 ms `TakeNotice` on
    the app door, the proxy, and a 250 ms poll in the C4 host before it
    reaches the browser.
12. **p2panda wiring.** `personal_sync_host.rs:195-311` and
    `carriage_host.rs:154-219` (`open`), and Distillery's own transport
    (`resident_distillery.rs:44`). `carriage_host::attach` already shares the
    sync endpoint (`:233-265`), so the composition exists for one of three.
13. **Session start** (admit, `SessionAuthority::retain_admitted`, catalog
    open, spawn the notifying loop): `projection_host.rs:292-338`,
    `browser_host.rs:190-247`, `app_broker.rs` around `300-366`.

## 5. Where transport knowledge leaks upward

- **chirograph**: `MAX_RESOURCE_CHUNK_BYTES` is sized against the
  native-messaging cap and a test pins it to `MAX_NATIVE_MESSAGE_BYTES`
  (`chirograph/src/lib.rs:626-634`, `1925-1942`); the `SessionOpen` docs name
  notochord (`:1063-1077`). The constant is defensible as protocol; its
  justification names one carrier.
- **graphshell-client**: `RemoteSession`'s lifecycle (`channel_closed`,
  `nudging`, `nudged`, `remote-nudge` events; `remote.rs:394-471`) is the
  WebRTC fixture's vocabulary in a crate whose description is
  "transport-independent".
- **Protocol id as everyone's transcript.** `PROJECTION_PROTOCOL`
  (`mere/graphshell/v1`) is the QUIC ALPN and also the transcript protocol for
  native messaging and WebRTC (`browser_carrier.rs:595`, `webrtc_door.rs:522`).
  It works, but the name says QUIC.
- **Browser-named types in the app door.** `BrowserChallenge`,
  `BrowserSessionClient`, `BrowserHostError` and `read_native_message_async`
  serve first-party apps that have nothing to do with a browser
  (`app_broker.rs:36-37`, `68`; `local_session.rs:32`, `46`).
- **Schemas naming carriers**: `browser_carrier.rs:51-53`, `app_broker.rs:60`,
  `app_admission.rs:46-50`, `device_broker.rs:33`, `local_session.rs:39-40`,
  `webrtc_door.rs:107`.
- **Feature flags per carrier**: `graphshell` `native`, `webrtc-join`,
  `webrtc-session`, `webrtc-browser`; `graphshell-endpoint` `stdio`, `local`,
  `network`. These are proper as build gates; they leak only when a consumer
  must pick a carrier to get a session layer.
- **Application identity is a door property.** Only the app door can say which
  application is calling (§1 item 2). That is the one leak that blocks the
  worked example.

What does not leak: `SessionCore` is sans-I/O (`core.rs:7-43`),
`dispatch_common` answers every data verb for every carrier
(`graphshell-endpoint/src/lib.rs:128-185`), the catalog chooses notice
intervals rather than the client (`endpoint_catalog.rs:91-127`), and
`mere-overmap`'s panel reads lines.

## 6. Proposal: keep it agnostic by composing it

The coordinator's prompt offered one candidate, a single carrier trait at the
chirograph and `graphshell-endpoint` seam with admission above it. The source
argues for a slightly different seam. `chirograph::Carrier` already exists
and is the blocking client view; a second trait at the same place would add a
fourth way to wrap a stream. The seam that every carrier already reaches is
lower and plainer: **an admitted byte stream, or an admitted line channel
where the platform gives no stream** (a browser data channel, a JavaScript
port), plus the facts admission produced. Everything above that is the same
for every carrier, and should be written once.

The layers, bottom up:

| Layer | Contract | Exists today | Change |
|---|---|---|---|
| **0. Byte plane** | `mere-transport::Transport` streams by ALPN; UDS and pipes; stdio; an in-process duplex; a WebRTC data channel via `stream_over_frames` | all of these | none |
| **1. Envelope framing** | Only where the platform forces a message boundary: native messaging (Chrome's native-endian prefix), WebRTC frames, notochord frames. Each turns messages into a byte stream or line channel | three implementations | one module per platform envelope, each exposing a stream or line channel; never used above admission |
| **2. Admission** | `admit(stream, IngressContext, policy) -> AdmittedEndpointContext`, with a sans-I/O twin for carriers that hand over messages (WebRTC) and a local-hello front for doors (challenge, `LocalLink`, then the same notochord exchange) | `admit_accepted_session`, `admission::admit_session`, `admit_local_client` | one entry in `graphshell-host` (R4) taking `AcceptedSession<S>`; the local doors produce `IngressContext` and call it; the action check is one function; the application id becomes a field the policy can bind on any carrier (§7) |
| **3. Line codec** | NDJSON of `CarrierRequest` / `CarrierOutput`, the same on every stream | `BufReader::lines` everywhere, `LineAssembler` for frames | one reader and one writer pair, shared by server and client |
| **4. Session plane** | Server: one loop (`serve_admitted_session_with`) with push notices and authority re-read. Client: `SessionCore` with one correlator; `SessionDriver` for line channels; one blocking adapter for `Carrier` | both exist; stdio and the browser client duplicate them | stdio's server becomes a sync adapter over the shared loop; `StdioCarrier`, `NetworkCarrier`, `BrowserSessionClient` and `AppBrokerClient`'s route half share the correlator; add the native stream-to-lines pump |
| **5. Notices** | Pushed as `CarrierOutput::Notice` on every carrier that can push; Discover-as-poll kept only for carriers that cannot | push on QUIC and WebRTC; pull on the app door; dropped on native messaging | the app door and the browser host relay pushes; nested polls in proxies go away |
| **6. Route catalog** | `ResidentEndpointCatalog` routes; the host decides which accept loops expose which routes | the catalog is already shared by the app door and the projection host | an exposure policy per accept loop (local door, QUIC host, WebRTC door) instead of one catalog per carrier |

**Composition between carriers** follows from layers 0 to 2 being separate:

- **Relays** are already inside layer 0: `P2pandaTransport` uses iroh relays,
  and Noise can wrap any stream for zero trust through them.
- **WebRTC signalled over an existing carrier.** Today signalling is plain HTTP
  (`bin/c4_webrtc_host.rs:316-331`, `451-460`). Signalling is bytes; an ALPN
  stream on an already-admitted QUIC connection, or the app door, could carry
  the offer and answer, so a browser paired once can rejoin without the HTTP
  surface. This is a later step and needs its own threat note.
- **Proxies** become one generic `CarrierEndpoint<C>`: an endpoint whose
  verbs forward over any client carrier and whose notices are forwarded
  pushes, replacing `ResidentMereProxy`'s per-verb code, timeouts and nested
  polls.
- **The browser door and the app door** become the same door with different
  allow-lists and hello fronts, because after the challenge they already run
  the same `LocalLink` and duplex notochord.

**What to leave alone.** The chirograph wire and its versions; notochord's
handshake; `mere-transport`'s `Transport` trait; the WebRTC join wire and its
invite model; the sync lanes, which are not sessions (they share only layer 0,
and `carriage_host::attach` already shows the right composition); Knot's
publish carrier until Knot asks (it is parallel on purpose and lives in
another repository). The `graphshell-endpoint` carrier features stay as build
gates.

## 7. The remote mere route as a composed carrier

Assembled from the layers above, with each step marked by what exists:

```
Turnstone overmap pane                       Djinn on the other device
MereRouteSession (lines)          [exists]   AccessEndpoint (mere/<domain>)   [exists]
  ↕ stream-to-lines pump          [missing]    ↕ ResidentEndpointCatalog      [exists]
dial_projection_session           [exists]   ResidentProjectionHost           [exists]
  notochord hello, subject = peer [exists]     admit: policy + application    [partly]
P2pandaTransport (iroh QUIC)      [exists]   P2pandaTransport                 [not run by djinn]
```

**Admission and who can dial.** Two populations are plausible, and they need
different policies on the same route:

- **The owner's own devices.** A device of the same persona, holding a device
  grant (`pandect::wallet_grant`), dials the resident directly. This is the
  "another device's sessions and graph" case.
- **Another person's Turnstone**, as a place member, presenting a delegation
  chain the way Turnstone's place host already admits members (Turnstone
  `src/place/projection_host.rs`, module docs). This is "another app's" case
  and needs per-mere disclosure rules the reservoir's access model already
  has for local apps (`MereApplicationAccess`, `resident_mere.rs:111`).

**What the receipt needs** (in order):

1. Djinn runs a `ResidentProjectionHost` on a transport key it chooses, and
   exposes `mere/<domain>` (not `ambient/`) on it (§9, T6 and T7).
2. The admitted context names an application for a remote dialer. The
   smallest change is that the session policy binds an application claim
   carried in the hello's delegation (or a route-scoped grant naming the
   app), and the endpoint context gets it through `with_application` exactly
   as the door does. `AccessEndpoint::open` and the per-application view
   state then work unchanged (§9, T5).
3. A native pump from the dialled stream to `SessionDriver` lines, in
   `graphshell-client` or `mere-overmap` (§9, T3).
4. The pane constructs `MereRouteSession::from_descriptor(descriptor,
   profile, "turnstone")` and reads sessions and graph; Turnstone's
   `overmap.rs` keeps the local case.

Mark's doubt that one transport is enough holds. A browser cannot dial iroh
QUIC; a first-party app on the same machine should not need a network key; a
remote native peer has no local socket. Three ingress kinds are the minimum
(local door, QUIC, WebRTC), with Reticulum available underneath when a mesh
is the only link. Agnosticism belongs above the stream, not in a single
transport.

## 8. Ranked recommendations

1. **Bind the application on every carrier, not only the door.** It blocks the
   worked example and every remote use of an application-scoped route. Small
   in code, large in policy (T5).
2. **Push notices on every carrier.** The native-messaging host drops them
   and the app door makes clients poll; fixing both removes the nested polls
   in proxies (§4 items 11 and 7). Needs confirming against the extension's
   JavaScript, which this note did not read.
3. **One admission entry over `AcceptedSession<S>`** in `graphshell-host`
   (P4), with the sans-I/O twin and the local-hello front calling it, and one
   action check (§4 item 4).
4. **One line codec and one client correlator** (§4 items 1 and 2), with the
   native stream-to-lines pump the overmap pane needs.
5. **Expose catalog routes per accept loop by host policy**, then register
   the mere route on a Djinn projection host: the Djinn step the plan's P2a
   receipt names.
6. **A generic endpoint-over-carrier adapter** replacing `ResidentMereProxy`'s
   hand-written forwarding (§4 item 9).
7. **Neutral names for the shared local door types** when L1 moves them
   (`BrowserChallenge` and its siblings serve apps too), and move the
   chunk-size justification in chirograph off the native-messaging cap.
8. **Later:** WebRTC signalling over an admitted carrier; sharing one
   p2panda endpoint across sync lanes; Knot's publish carrier on the shared
   admission entry if Knot wants it.

## 9. Rulings

Mark ruled T1 to T8 on 2026-10-15. Every recommendation stood except T4,
where he chose the widest option.

- **T1, the seam. Ruled: an admitted byte stream or line channel plus
  ingress facts.** No new carrier trait.
- **T2, framing. Ruled: NDJSON inside every stream.** Length prefixes are
  used only as platform envelopes below admission.
- **T3, notices. Ruled: push on every carrier that can.** Where push exists,
  `TakeNotice`, `WaitNotice` and Discover-as-poll retire. The native
  stream-to-lines pump lives in `graphshell-client`, so every remote session
  can use it, not only the overmap.
- **T4, who can dial a remote mere route. Ruled: same-persona devices by
  device grant, other people's Turnstone as place members by delegation,
  and `ambient/mere/<domain>` exposed remotely as well.** This was the widest
  option offered. Exposing ambient remotely needs its own admission and
  grant shape, which the implementing pass must set out before serving it.
- **T5, the remote application identity. Ruled: an application claim in the
  delegation.** The persona's delegation names the application that may act.
- **T6, where the shared layers live. Ruled: server side in
  `graphshell-host` (R4), client side in `graphshell-client`.** No dedicated
  carrier crate.
- **T7, Djinn's dialable projection host. Ruled: yes, on a dedicated
  resident key.** It does not share personal sync's key or per-place keys.
- **T8, WebRTC signalling. Ruled: keep HTTP for now.** Moving it onto an
  admitted carrier stays in the later tier.

## Findings

**2026-10-15** (Mere `9cc711fbc`; Murm `b49df894` and `3d9e799f` from the
cargo cache; Turnstone `de56f958` from GitHub; Knot local checkout; nothing
built):

- Production code binds an endpoint context's application in one place,
  `app_broker.rs:326`; `AccessEndpoint::open` requires one
  (`resident_mere.rs:359-361`).
- `serve_native_messages` (`native/browser_host.rs:162-299`) relays responses
  only; no notice reaches the browser over native messaging.
- Native-messaging length prefixes are native-endian
  (`browser_carrier.rs:449`, `478`, `510`, `542`); WebRTC frames are
  big-endian (`webrtc-carrier` `frame.rs:88`).
- The Djinn product binary serves the app door and the browser broker
  (`bin/djinn.rs:1248-1286`) and p2panda lanes; no QUIC projection accept
  loop.
- `mere-transport`'s `transport` sources are identical at `b49df894` (Knot's
  and Turnstone's pin) and `3d9e799f` (Mere's).
- Turnstone's place host serves projections over a per-place
  `P2pandaTransport` with Graphshell's accept, catalog and notifying loop;
  its pinned Mere is `77fd235f`.
- Not verified: the extension's JavaScript; the external crates' QUIC stream
  mapping and handshake limits beyond their call sites; whether a WebRTC
  rejoin's preserved `ProjectionSession` (`graphshell-client/src/remote.rs:441-451`)
  is the same session the transcript-derived id names after re-admission.

## Progress

- **2026-10-15 (with the stack lift plan's P4):** recommendation 3 and the
  codec half of recommendation 4 landed in `graphshell-host` and
  `graphshell-client` (T6). One admission entry:
  `graphshell_host::carrier::admit_stream` (stream plus facts, T1), which
  `admit_accepted_session` and the port's local doors call; the WebRTC message
  path keeps the sans-I/O `admission::admit_session`; both end in
  `admission::serves_action`. One codec (T2): `graphshell_client::line`
  (sans-I/O, with `LineAssembler`) and `graphshell_host::line` (Tokio), used
  by `SessionDriver`, the session loop and `BrowserSessionClient`.
  Still open, in order of the recommendations:
  1. The application claim in the delegation (T5, recommendation 1): nothing
     binds an application outside the app door yet.
  2. Push notices on every carrier (T3, recommendation 2): the
     native-messaging host still drops notices and the app door still serves
     `TakeNotice` and `WaitNotice`; the extension's JavaScript is still unread.
  3. The rest of recommendation 4: `graphshell-endpoint`'s `stdio` and
     `network` carriers still frame and correlate by hand (they cannot depend
     on `graphshell-host`; moving them onto `graphshell_client::line` is a new
     `graphshell-client` edge for `graphshell-endpoint`); one client correlator
     for `StdioCarrier`, `NetworkCarrier`, `BrowserSessionClient` and
     `AppBrokerClient`; the native stream-to-lines pump for `SessionDriver` in
     `graphshell-client` (T3), which the overmap pane needs.
  4. Djinn's dialable projection host on a dedicated resident key with
     `mere/<domain>` (and, per T4, `ambient/mere/<domain>` with its own
     admission shape) exposed on it (T7, recommendation 5); the brokers
     moving to Djinn is the lift plan's P5.
- **2026-10-15 (with the lift plan's P5):** Mark ruled the codec into
  `chirograph`. `chirograph::line` (with `LineAssembler`) is now the one codec:
  `graphshell-endpoint`'s stdio and network carriers frame through it, so item
  3's codec half is closed. Still open from item 3: one client correlator and
  the native stream-to-lines pump. The brokers are Djinn's `door` module, so
  push notices (item 2) and the dialable host (item 4) are Djinn work.
- **2026-10-15 (pass (b)):** T5, T7, the native pump (T3) and the Murm
  in-process kind landed; the remote route's admission is the
  [remote mere route admission note](2026-10-15_remote_mere_route_admission.md).
  Item 1 (the application claim) holds on the dialable host by the
  requested path and the delegation's leaf, with no dramatis change; item 3's
  pump is `graphshell_client::pump`; item 4's host is
  `djinn::resident_projection`, not yet started by the binary. Still open:
  push notices (item 2), one client correlator (item 3), and the binary's
  trusted roots.

## Authority audit checkpoint, 2026-10-16

At Mere `b567660d67c4d900f1d638345c89715e1f36c575`, all 24 files in
`crates/graphshell/graphshell-host` have been read at recorded content hashes.
This is a source-review checkpoint, not completion of the Mere audit or a
headed/remote acceptance gate. No production code changed in this pass.
The detailed findings, authority map, coverage and native diagnostics are in
[`support/receipts/authority-success-audit`](../../../support/receipts/authority-success-audit/).

- **Q056 / A052, message admission can contradict its reply.** The
  sans-I/O helper keeps Notochord's reply, then can replace an accepted
  principal with `ActionNotOffered` when `serves_action` refuses it. A signed
  hello for an owner-allowed but unimplemented action returns a decoded
  `Accept` alongside that local error. The WebRTC host sends this reply
  before testing the error; the peer interprets `Accept` as a successful
  join. The host still refuses service. Both paths really share
  `serves_action`; the coordination around it differs. The native stream
  adapter explicitly documents policy admission followed by a possible
  service refusal and close, so its two-stage contract needs separate
  treatment. Done: the message helper's final outcome and peer-visible
  result agree for supported, policy-refused and service-refused actions.
- **Q057 / A053, reconnect uniqueness is conditional.** The lifecycle
  module's unconditional "different id" and "by construction" claims exceed
  its function-level contract, which correctly assigns nonce freshness to
  the caller. Two public admissions of the same signed hello produce the
  same session id; a fresh nonce changes it. The reconnect unit test supplies
  two different ids rather than proving freshness. Current WebRTC peers do
  generate fresh random nonces. This is a qualified claim/test defect, not
  evidence of an ordinary WebRTC reconnect failure or cross-subject access.
  Done: claims and tests name the actual freshness owner.
- **Q058 / A054, the N4 policy receipt repeats its own wrong domain.** One
  helper sets `mere.network` for both Murm and Graphshell, while Graphshell
  asks for `mere.graphshell`. Its synthetic claims helper repeats the wrong
  domain, so the test can pass while the rendered stage says Graphshell is
  open. The exact extracted helper marked `Public` refuses a real Graphshell
  hello in current public admission; the correct-domain control accepts.
  The full persistence/headed scenario was not run. Done: service-specific
  rules are checked against production hello builders, not a repeated local
  string.
- **Q059 / A055, default host unit tests do not compile.** A standalone
  default-feature `cargo test -p graphshell-host --lib --no-run` reaches the
  package and exits 101 with 36 errors. The unconditional WebRTC door test
  module imports native-only host definitions and a transport adapter gated
  by `notochord`; default features enable `webrtc-join`, not `native`.
  Admission-only library compilation succeeds. No tests ran in the failed
  gate. Done: fixture gates/imports match their feature owners, with default
  and native gates qualified independently.

The five bounded admission diagnostic tests passed while establishing the
stated defects and controls. They use synthetic signed identities and supplied
authenticated memory-carrier facts, with no network or DTLS join. Existing
loopback integration sources do exercise ICE/DTLS/SCTP, but reading them is not
an execution receipt.

Larger follow-ups remain documented rather than expanded into this audit:
`admit_webrtc_catalog` snapshots a finite session allowance before an awaited
join, then `serve_admitted` increments the live count without another ceiling
check. Concurrent joins need reservation/rollback ownership. The same join
also uses a captured time and revocation snapshot across awaited exchanges;
carrier lifetime and consumer adoption need tracing before asserting a
reachable service bypass. Shared catalog factories, common dispatch, the
codec and the session loop are real reuse; they do not by themselves prove
every authority boundary is complete.

The same pass subsequently confirmed two more boundaries:

- **Q060 / A056, pushes outlive authority until a request arrives.** The
  notifying loop polls and writes notices before its authority check; idle
  timeouts repeat that path. A native public-loop diagnostic starts live,
  then expires or revokes its signed grant, independently verifies the lapse,
  and still receives a newly queued notice. The next request is correctly
  refused and terminates with `Lapsed`. The exposed data is notice
  session/epoch/revision metadata, not a demonstrated snapshot/resource or
  intent bypass. Three duplex diagnostics, including a live control, passed.
  They construct an admitted session; no network admission or installed
  resident ran. Done: retained-authority checks govern autonomous output and
  idle polling as well as requests, with a defined quiet-peer lapse signal.
- **Q061 / A057, the client has two pending-state owners.** Reading the
  client's core and driver shows `SessionCore.pending` can change before
  `SessionDriver.inflight` refuses a second send. Start a Close, call
  `discover` again and receive the expected busy error, then supply the
  original valid Closed response: it is rejected because the core now
  expects a descriptor. The existing busy test stops after checking the
  error. Two public-API diagnostics establish this difference against a
  successful close control. Done: a refused operation leaves the original
  operation intact; core mutation and request-slot admission are coordinated
  for both driver helpers and callers using `core_mut` plus `begin`.

The client package is still being read. These diagnostics used the libraries
from a successful native host build, recorded in `host_review_receipt.json`.

### Client source checkpoint, 2026-10-16

The next pass completed all 15 files in `graphshell-client` (331,603 source
characters, including the manifest and tests). The shared endpoint, host and
client packages are now fully source-read. The rest of Mere remains under
review. No production code changed; the earlier repair files were preserved.

Eight bounded diagnostics using a freshly built client library passed while
establishing five findings and three controls. Evidence is recorded in
`support/receipts/authority-success-audit/client_review_receipt.json` and its
referenced native probe receipt. These are public-API tests with generated,
validated snapshots and encoded replies. They do not exercise a browser,
network, installed resident or OS accessibility bridge. Existing package
unit/integration tests were read, not executed in this pass.

- **Q062 / A058, cache admission does not correlate a resource reply.**
  `Pending::Resolve` retains the session/instance but loses the requested hash.
  Another valid resource advertised in that session is cached and the missing
  resource requested again. Six repeated wrong-resource replies made no
  progress; the correct-resource control completed. The blocking adapter adds
  no progress bound. This does not bypass advertisement or content-address
  checks. Done: retain and compare the outstanding request before caching or
  advancing, with the same rule in both adapters. The media-resource path
  already performs this comparison separately.
- **Q063 / A059, frozen reading order comes from table slots.** A valid
  snapshot with explicit item order `[1, 0]` reads `[1, 0]` through remote
  accessibility and `[0, 1]` through the frozen view. Dense-order control
  agrees. Sparse identity preservation is real; stable ids and explicit order
  are compatible. Done: share ordered active-item traversal while retaining
  instance ids, including tombstones and reordered items.
- **Q064 / A060, remote accessibility ignores an item's own visibility.**
  It filters fold hiding but omits the `visible` check. A valid invisible item
  is absent according to `SceneSnapshot::is_shown` and the frozen view, yet
  remains in remote accessible children, from which remote actions are also
  derived. The data was already disclosed, so this is a presentation/action
  mismatch, not an endpoint authorization bypass. Done: consistent flat-item
  visibility across readers, with folded-member disclosure precedence stated
  separately. Shared `FoldReading` is real reuse, but does not supply this
  final visibility decision.
- **Q065 / A061, an unexpected close does not perform the disconnect
  transition.** `channel_closed` updates outer strings and `link_busy`, while
  intentional `disconnect` also clears pending wire state and marks mounts
  disconnected. An idle mount remains `Live` after an unexpected close. A
  lost discovery remains awaited and rejects rediscovery after `rejoined`.
  Intentional-disconnect control succeeds. Targeted port source routes stream
  termination to these APIs; actual browser channel teardown was not run.
  Done: common terminal-link state handling for idle and mid-request close or
  failure, preserving cached display while retiring operations and traffic
  from the lost link. This needs coordinated lifecycle ownership, not a status
  string patch.
- **Q066 / A063, a snapshot recovery is labelled a diff.** The core reduces
  successful resume replies to `Changed(true)`; the remote receipt spells
  this `diff`. A full `ResumeReply::Snapshot` is applied correctly and reports
  `diff · 1 → 2`. The scripted board test makes the same substitution while
  naming/asserting diff recovery. Done: preserve the actual recovery mode or
  use an honest generic update label; test snapshot and diff independently.

**A062 tracks a remaining manual implementation:** `resume_after_notice` and
`resume_request_for_notice` retain resume/retry logic beside `SessionCore`.
The compatibility role is explicit; both paths reuse `ClientState.apply_resume`
and the retry constant. Current repository search found no direct caller of
`resume_after_notice`, while the request helper remains used by a host test.
This is a maintenance surface to account for, not proof that external callers
do not exist or that its current behavior differs.

Further source candidates are preserved in the ledger: delayed resources can
be admitted after client-side revocation; resume diffs can overwrite a terminal
status with `Live`; failure paths retain some pending/outbox state; an older
action draft can survive a failed attempt to open a new one; sparse object
counts include tombstones; and frozen fold-member order/visibility differs
from its broader prose. Composing authority and consumer contracts still need
tracing before promoting these to broader product or security claims.

### Protocol and scene-contract checkpoint, 2026-10-16

All five Chirograph files and all nine Sceno files are now source-read at
recorded hashes. The package gates passed: 52 Chirograph unit tests and 37
Sceno unit tests. Five additional public-API diagnostics passed while
establishing two findings and qualifying chunk-check ownership. No production
code changed. Evidence and preservation checks are in
`support/receipts/authority-success-audit/chirograph_review_receipt.json`.

- **Q067 / A064, capture and live presentation admission disagree.** Valid
  scene tables with absent-item bindings, duplicate bindings or a
  codec/capability conflict pass both capture versions' validation,
  encode/decode and resource-availability checks. The same sidecar is refused
  by the live client. A valid control agrees. `CaptureWire` really shares
  version/scene/resource plumbing, but the live client's private presentation
  validator is another invariant owner. Done: shared carrier-independent
  presentation validation for captures and live snapshots, with session/cache
  policy handled explicitly at its composing boundary. Current repository
  searches found these capture APIs in tests, not a production capture
  consumer; no user-artifact or renderer failure was executed.
- **Q068 / A065, byte admission differs before shared JSON decoding.** The
  frame assembler silently replaces invalid UTF-8 in complete lines and EOF
  tails; the native `LineReader` rejects it with `InvalidData`. A typed
  descriptor response containing an invalid label byte becomes a successfully
  decoded response with a replacement character through the assembler. Valid
  UTF-8 split inside a multibyte character agrees across both readers. Done:
  fallible, consistent UTF-8 admission, including browser caller error
  propagation. This was tested through the public assembler and native reader
  over a duplex; the browser caller was source-traced, not run.

**A066 qualifies chunk authority.** The shared assembler owns offsets and
hash integrity; it does not own the requested session/hash, stable total size,
or progress bounds. The native app client already adds all of those checks,
with explicit fault tests in its source. Raw assembly accepts a validly hashed
resource switch and non-final empty progress, but that is not evidence of a
bypass through this stricter app wrapper. Its fault tests were read, not rerun
in this pass. **A067 qualifies acknowledgments:** `ProjectionAck` describes
in-memory applied progress; its "durably applied" wording is not a store
receipt, particularly under default memory-only retention.

Sceno supplies a useful positive boundary: unique, live, disjoint fold
membership is validated centrally; fold member order is explicit; own
visibility still counts. Invalid overlapping folds are therefore not evidence
of a valid-input reader conflict. Score migration is implemented: old
`Anchored` reads as `Seeded` and is written with the reader's version, while
newer scores are refused. Source identity and instance identity remain
separate, and `LinkKey` is explicitly a derived view token, not graph truth.

The ledger retains the remaining questions without inventing policy: default
scene construction versus validated world spaces; constructor-only ordering
of deserialized link keys; nested score migration versus capture byte identity;
version/capability negotiation at `Open`; and message assembly budgets. The
test named `every_arrangement_round_trips` omits the three current subdivision
families, so its name is broader than its coverage. Solver behavior and sparse
scene validation are still ahead in the review.


### Audit wording repair, 2026-10-16

Q066's remote resume receipt now reports `updated` rather than inferring `diff`
from a changed-result boolean. That outcome can represent a complete replacement
snapshot. The two scripted-board tests now name the Snapshot replies they
actually supply. This bounded repair makes the label truthful; it does not add
wire-mode provenance to Outcome or repair the separate unexpected-close and
resource-correlation findings. The all-feature client gate passes 91 tests,
including these remote tests, and fails the existing exact-byte trace test
(Q073), whose default-field reserialization changes are documented separately.
Native results are recorded in
the [audit receipt](../../../support/receipts/authority-success-audit/scenograph_review_receipt.json).
