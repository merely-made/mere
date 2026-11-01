# Capability door: app identity, manifests, introduction and WIT contracts

**Status:** 2026-11-01: plan, revised on 2026-10-30 (D4). Most of S1 landed
outside this plan in Mere `c3567c5ab` (application projection offers), and the
door library moved to Armillary's `djinn` crate (`721e17d`). I1 is built
(Armillary `b6578ee`, its Windows remainder `104b8d4`) and M1's Knot and
Turnstone cases have landed; M1's
remaining entries follow. C0 is built (Mere `32be2f0b0`): script-witgen and
script-wire bind WIT natively, and graphshell-endpoint's `wit` carrier is what
N1's channels will carry (D19).
`96e8843e814`, the revision's against Mere `94c1bd1a3` and Armillary
`a333f51`, and the upstream findings of 2026-10-31 on GitHub and crates.io
that day; no test was run for them.

**For:** letting applications and components share capabilities with each
other, on one device and beyond it, without each one carrying the stack.
Mark, 2026-10-30, on "embedding apps, sharing capabilities/data of the apps to
each other without making each app carry the stack's whole weight", and of
Wasm components: "i originally thought of this as the way we would share app
capabilities throughout and outside the stack in an embeddable,
non-burdensome way. that's why i thought wasi, mods, etc. were interesting."
He asked to examine the Fuchsia pattern (manifests declare what a component
uses and offers, a router assigns identity and routes, a capability is a
channel), then chose a Mere plan "but have a look at the graphshell host plan
too". The first consumer is that plan's G6: one workspace composing two
applications' projections.

Not here:
- G6's host-side items (`ClientState` keyed per endpoint, sceno mounts, a
  saved workspace's endpoint list) and G7:
  [`2026-10-28_graphshell_host_plan.md`](2026-10-28_graphshell_host_plan.md).
  S1 here is G6's need.
- Extracting and publishing Graphshell's components, including the `door`
  feature split: [`2026-10-28_stack_lift_plan.md`](2026-10-28_stack_lift_plan.md).
- Live references between devices: Murm's
  `murm/design_docs/2026-10-29_secure_sessions_and_live_interaction_plan.md`
  (O1 to O4, `murm-ocapn`) and
  [`2026-10-28_murm_runtime_plan.md`](2026-10-28_murm_runtime_plan.md). G1
  here is the gateway to it at the device edge.
- Custody policy: the custodian stays the only holder of secrets. This plan
  reuses its peer facts, not its grants.
- UI across processes, and components that draw their own pixels: the
  [app composition brief](../../cambium_docs/research/2026-10-06_app_composition_brief.md)'s
  AC6 (paint lists and accessibility trees, research once AC1's path is
  proven). Under D3 a paint list is a serde payload.
- The graph edit spine and the settings scope vocabulary, the
  [source homes map](../technical_architecture/2026-10-28_source_homes_map.md)'s
  other missing contracts.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **The door structures; it does not confine.** On macOS, Windows and Linux a
  process running as the user can read the user's files and reach the same
  sockets. Identity, manifests and introduction give honest applications least
  authority, consent, revocation and true attribution. They do not stop
  malware running as the user, and no surface claims they do; the host plan
  already says the GUI must not claim to identify every local caller.
  Confinement comes from the custodian (another OS principal), Wasm components
  and other devices.
- **Projections are not authorities** (host plan). Introduction hands over
  access to a service; source truth stays with its owner, and an intent still
  passes the owner's checks.
- **A contract promotes with a heterogeneous second consumer**, the
  implementation-ready gate of the
  [composition thesis](../../2026-08-12_family_composition_thesis_brief.md) §7.
  A WIT interface with one consumer stays with that consumer.
- **Cold and narrow goes behind a component; hot or wide stays native**
  (doctrine Stack 5). Components carry contracts, readers, validators and
  applets, not renderers, editors or the graph kernel.
- **The contract stays narrow** (composition thesis §5). A capability is a
  service: a projection session, a settings domain, a custody facet.
  Fine-grained objects stay data inside its protocol.
- **Optional, with no C by default.** Each layer sits behind a consumer
  feature, and a default build gains no C toolchain requirement.
- **Between devices, through Murm.** Nothing here binds another endpoint or
  carries another device's traffic.
- **Secrets never cross.** Browser Wasm never receives a seed, private key or
  vault root (host plan); custody keys leave Djinn only under its release
  rules.

## Model

| Fuchsia | Here |
|---|---|
| A manifest declares what a component uses, offers and exposes | A door app's manifest (M1). A component's uses are its WIT world's imports |
| The component manager routes and assigns identity | Djinn's route grants and endpoint catalog, fed by manifests; identity per D2 |
| A capability is a channel in the component's namespace | D1: one OS channel per granted service, with handles carried beside the bytes |
| No consent step; routing is system policy | Djinn asks, under castellan's rule that only the resident asks (Sandstorm's Powerbox) |
| — | Durable delegation stays insigne certificates and Servitor grants, unchanged |

Inside one process, a host's linker binds a component's imports to an
in-process implementation or to a door channel: the same routing decision at
a different boundary. A proxied grant is revoked by Djinn closing it; a direct
pair is revoked by its provider on Djinn's notice. Which one a service gets is
part of its manifest entry.

## Decisions

- **D1, OS channels on one device.** Mark chose "OS channels
  (Recommended)": Djinn introduces after consent by passing a channel end (fd
  passing, Mach ports, `DuplicateHandle`); revocable grants stay proxied
  through Djinn; a gateway maps services to `murm-ocapn` at the device edge. A
  software CapTP on one device was not chosen: the kernel makes channels
  unforgeable, introducing a third party costs nothing, and pipelining saves
  little locally.

  Links: area Stack · draws on stack.2, stack.3 · summary On one device a capability is an OS channel that Djinn introduces after consent; revocable grants stay proxied; a gateway meets murm-ocapn at the device edge.

- **D2, OS identity and launch handles.** Mark chose "Both": OS identity
  where the platform gives one, launch-time handles where it doesn't (Linux).
  The door reads its peer as `custody-wire`'s `Peer` does (a macOS code
  signature, a Windows executable image, a Unix uid), and Djinn starts Linux
  applications with a handle. A self-declared label is accepted only in
  development mode, which status reports, as the custodian already does.

  Links: area Stack · draws on stack.2 · summary A door app's identity is what the OS reports, or a handle Djinn gave it at launch; labels only in development mode.

- **D3, WIT interfaces with serde payloads.** Mark asked to "compare 1 and
  2" (WIT; Rust with serde), then chose "WIT + serde payloads (Recommended)":
  interfaces and handles in WIT; scenes, paint lists, documents and
  certificates stay Rust types carried as named serde bytes. Chirograph keeps
  its data model, and its control surface becomes a WIT resource.

  Links: area Stack · draws on stack.3, stack.5 · summary Capability interfaces and handles are WIT; bulk data stays Rust types carried as named serde bytes.

- **D4, record what landed; Knot's case, then I1.** `c3567c5ab` built most
  of S1 (policy offers with leases) seven minutes after this plan. Mark chose
  "Revise plan, then I1 (Recommended)": the plan records what landed and
  fixes its stale findings, and C0 waits for a component provider someone
  wants, since G6's two native applications are served by the offers path
  without WIT. The same day, in another session, he ruled "Application
  registration is next, with Knot as the first complete case" (under M1).
  Asked to reconcile the two, he chose "Knot case stands, then I1
  (Recommended)": Knot's landed declaration case stays, and I1 comes next,
  before M1's remaining entries, so later declarations bind to OS identity
  rather than labels. I1 closes the gap the offers record names ("Same-user
  AppId labels are not executable authentication").

  Links: area Stack · draws on stack.2 · summary Most of S1 landed as policy offers; Knot's declaration case stands, I1 comes before M1's remaining entries, and C0 waits for a wanted component provider.

- **D5, OCapN is an optional edge profile.** Mark, on the stack as a whole:
  "Idk if OCapN is ready, or if this proposition coheres in the way we'd
  hope". He chose "Optional edge profile (Recommended)": `murm-ocapn` stays a
  pinned, optional profile for interop at the device edge; nothing local
  depends on it; G1 waits for a named remote consumer and for OCapN's
  encoding and error types to settle. The Murm runtime stays as built.

  Links: area Stack · draws on stack.1 · summary murm-ocapn is a pinned optional edge profile; nothing local depends on it; G1 waits for a remote consumer and a settled encoding.

- **D6, versioned CBOR for payloads between applications.** Mark chose
  "CBOR, versioned (Recommended)": projection payloads that cross an
  application boundary through Djinn are CBOR behind a version, following the
  data formats brief's line for bytes that cross an ownership boundary. The
  existing door path keeps JSON. Postcard was rejected because chirograph's
  internally tagged enums and skipped fields need a self-describing format.

  Links: area Stack · draws on stack.3 · summary Projection payloads crossing an application boundary are versioned CBOR; the existing door path keeps JSON.

- **D7, one home for OS peer facts.** Armillary's door cannot depend on
  `custody-wire`, which lives in Mere, and the two already duplicate the
  Windows token reading. Mark chose "Small crate in Armillary
  (Recommended)": `Peer` and its OS readers move into one small Armillary
  crate that both the door and `custody-wire` depend on; `custody-wire` keeps
  its own policy (`ServerExpectation`, consent). The crate's name is open and
  goes through the naming ledger.

  Links: area Stack · draws on stack.1 · summary OS peer facts move into one small Armillary crate shared by Djinn's door and custody-wire; custody policy stays in custody-wire.

- **D8, Djinn computes effective authority; services recheck.** Three
  checks exist and each service ANDs them by hand: subject admission
  (Notochord's `RetainedAuthority`), the application grant (Djinn's
  `AppAuthority`) and domain authority (Gemot, Moot, Servitor). No type holds
  the first two together, and Djinn's proxied projections hand providers a
  `ViewerContext` documented as "not credentials". Mark chose "Djinn
  computes, services recheck (Recommended)": at admission Djinn builds one
  value holding the application's `AppAuthority` and the subject's
  `RetainedAuthority`, and passes it on in place of `ViewerContext`; a service
  re-asks lapse and coverage on each call and keeps its own domain checks. A
  deputy (Djinn's proxy, G1's gateway) passes the narrower party's authority,
  never its own. The shape is recorded now and built with N1 or G1.

  Links: area Stack · draws on stack.2 · summary Djinn builds one effective-authority value (app grant and subject's retained authority) at admission; services recheck lapse and coverage per call and keep domain checks; a deputy never lends its own authority.

- **D9, an application's grant names its personas.** The persona in use is
  vault-wide today, and `AppGrant` has no persona term. Mark chose "Yes, in
  AppGrant (Recommended)": the owner's grant lists the personas an
  application may act as, empty by default, so linkage across applications
  stays opt-in and the gateway has a persona term to intersect.

  Links: area Stack · draws on stack.2 · summary An application's grant lists the personas it may act as, empty by default.

- **D10, one upstream ipc-channel, no in-process flag in libraries
  (2026-10-31).** Genet's vendored fork existed only because ipc-channel 0.22
  pulled `windows` 0.61 while wgpu-hal needs 0.62; 0.23.0's only change is
  `windows` 0.62, and 0.23.1 fixes the macOS advisory. Mark chose "Upstream
  0.23.1, no flag (Recommended)": the fork goes, Genet's media takes crates.io
  0.23.1 on its default OS backend, and Turnstone drops its patch row so its
  Servo copy and Genet's are one 0.23.1. A library never sets
  `force-inprocess`: Cargo unifies features, so the flag would put N1's
  channels in-process in any application that links Genet's media. Only an
  application root may set it.

  Links: area Stack · draws on stack.1 · summary Genet and Turnstone take one crates.io ipc-channel 0.23.1; libraries never set force-inprocess, which only an application root may.

- **D11, Windows identity rests on launch (2026-10-31).** Mark chose "Launch
  + hardened pin (Recommended)": Djinn launches a Windows application and
  holds its process handle, so the pipe client's process id cannot be reused
  while it is compared, with a job object covering the application's own
  children; an application Djinn did not launch gets a hardened image pin,
  both facts read from one process handle and refused if the process was
  created after the connect, reported as same-user and read at connect; the
  token's user, session, integrity level and AppContainer are read through the
  pipe; and the door's pipe gets an explicit access list and refuses remote
  clients, as Mere's custodian pipe already does. Package identity and
  Authenticode stay out of admission until an installer exists.

  Links: area Stack · draws on stack.2 · summary On Windows Djinn launches and holds the child handle; a hardened, same-user image pin covers the rest; token facts and a strict pipe access list apply to every connection.

- **D12, C0 runs now and probes two binding routes (2026-10-31).** Mark
  chose "Probe both first" for C0's native binding and "Lift the hold now
  (Recommended)" for its timing, amending D4's hold: the projection session
  is bound both through a small generator of our own over an exactly pinned
  `wit-parser` and through `wit-bindgen-wrpc` at a pinned revision with our
  own transport over OS channels; effort, size and fit with the blocking,
  single-thread `Carrier` are measured, then one route is chosen. Wasmtime's
  `bindgen!` stays on the component side, since its generated types bring
  Wasmtime and its C helpers into every native process. The component
  consumer is a test guest under `app-host` until a real provider is wanted.

  Links: area Stack · amends capability-door D4 · draws on stack.3, stack.5 · summary C0 runs now: the projection session is bound through our own wit-parser generator and through pinned wRPC, measured, then one route is chosen.

- **D13, the projection session's contract (2026-10-31).** Mark chose
  "Free calls, then a resource (Recommended)": `discover` and `open` are free
  functions and `open` returns a `projection-session` resource whose methods
  are snapshot, resource, chunk, resume, intent, close and suspend, so the
  session id no longer rides in each payload. "Every method rechecks
  (Recommended)": the provider rechecks the session's retained authority on
  every call, resume included; a lapse comes back as Revoked or Expired and
  nothing is disclosed under it. "Status stays, drop is close
  (Recommended)": revocation stays a status in replies plus a typed
  `revoked` failure after a lapse, dropping the resource is Close, and
  cancelling a call in flight is left to N1. "Cap every payload at 1 MiB
  (Recommended)": each CBOR payload is capped at 1 MiB, the browser
  carrier's limit, checked on both sides with a typed `too-large` failure;
  chunks stay at 512 KiB.

  Links: area Stack · draws on stack.2, stack.3 · summary Discover and open are free calls returning a session resource; every method rechecks authority; revocation is a status and a typed failure, drop is close; payloads cap at 1 MiB.

- **D14, where C0's pieces live (2026-10-31).** Mark chose "Interface in
  mere:script (Recommended)": the projection-session interface joins
  `mere:script` with its own provider world, and no package name is minted.
  "Probe dir, then promote (Recommended)": both routes are built under
  `crates/probes/projection-session-c0/`, and only the chosen one is promoted
  into a shipping crate. "ciborium (Recommended)": payloads are CBOR through
  `ciborium`, already in graph-kernel and pandect.

  Links: area Stack · draws on stack.1, stack.3 · summary The interface joins mere:script; both routes are probed under crates/probes and only the winner is promoted; CBOR through ciborium.

- **D15, notices and the wire encoding for C0 (2026-10-31).** Mark chose
  "Opened session holds the id (Recommended)": one resource per Open holds
  the session id derived at admission, method arguments carry none, and the
  binding fills chirograph's field and refuses a reply naming another session;
  D13 stands as written, on the finding below. "Both, mirroring Carrier
  (Recommended)": the resource has `take-notice` and `wait-notice`, notices
  as typed records. "wRPC's value encoding (Recommended)": our generator
  writes WIT values in wRPC's value encoding, and the test asserts byte
  equality with wRPC for a set of values and equal values back through a
  Wasmtime guest.

  Links: area Stack · draws on stack.3 · summary The session resource holds the admission-derived id; notices are taken or awaited as typed records; our generator writes wRPC's value encoding.

- **D16, a launched Windows application is the child alone (2026-10-31).**
  Asked what D11's job object should do, since Linux's `claim` (Armillary
  `7eebeeb`) keeps a launched application's children from speaking as it,
  Mark chose "Child only, no job (Recommended)": the door admits only the
  process Djinn launched, by id and creation time while Djinn holds its
  handle, as on Linux, and no job is made. Nothing would use one, and
  starting a child inside a job without a race needs `CreateProcessW` by
  hand, since std's attribute list and main-thread handle are unstable.

  Links: area Stack · amends capability-door D11 · draws on stack.2 · summary A launched Windows application is the process Djinn started, known by id and creation time; its children are not the application, and no job is made.

- **D17, C0 binds native code with our own generator (2026-10-31).** Mark
  chose "Route A, our generator (Recommended)" after the probe: our
  generator over `wit-parser` =0.263.0 and its blocking runtime are promoted
  into a shipping crate, and `wit-bindgen-wrpc` 0.12.0 is rejected. The
  measurements are in the 2026-10-31 C0 finding. Because our encoding equals
  wRPC's byte for byte, wRPC could be taken up later without changing the
  wire.

  Links: area Stack · amends capability-door D12 · draws on stack.1, stack.3 · summary C0 chose our own wit-parser generator with a blocking runtime; wRPC 0.12.0 is rejected for its runtime, Send bounds and missing resource drop.

- **D18, promoting C0's generator (2026-11-01).** Mark chose "Two crates in
  crates/script (Recommended)" and "script-wire, script-witgen
  (Recommended)": `script-wire` is the runtime a consumer links (the value
  encoding, the transport seam, the transports) and `script-witgen` the
  generator, only ever a build-dependency, so `wit-parser` reaches no shipped
  binary. For what moves he chose "Also a Carrier now", adding "but have a
  look at n1… compatibility is critical": the projection-session interface
  and its provider world join `crates/script/wit`, the test guest and
  Wasmtime host test go under app-host's tests, the test provider, call
  script and conformance checks become the runtime's tests, and a `Carrier`
  over the generated client ships now. For its home he chose
  "graphshell-endpoint, `wit` feature (Recommended)", beside the stdio,
  local, network and door carriers. "Delete it once promoted
  (Recommended)": the probe goes in the promotion commit; its commits and
  the findings keep the comparison.

  Links: area Stack · draws on stack.1, stack.3, stack.5 · summary script-wire and script-witgen in crates/script; the interface joins mere:script; a Carrier over the generated client ships in graphshell-endpoint's wit feature; the probe is deleted.

- **D19, what an N1 channel carries (2026-11-01).** Mark chose "The WIT
  calls, in wRPC encoding (Recommended)": every projection provider on an
  N1 channel, native or component, speaks the generated calls, so a consumer
  cannot tell which it reached; app-host serves a component by passing
  frames to its guest, and the JSON-lines wire stays on the existing
  carriers. D4's "native channels need no WIT" now reads "need no Wasm".
  "Pipes + ipc-channel, feature off (Recommended)": `script-wire` ships an
  OS-pipe transport and an ipc-channel 0.23.1 bytes-channel transport behind
  an `ipc` feature off by default, never setting `force-inprocess` (D10),
  and the transcript runs over both, once with the provider's ends passed
  over a channel first. "Disconnect closes all (Recommended)": when a
  channel ends the provider closes every session opened on it, the call in
  flight returns `CarrierError::Disconnected`, and a revoked grant shows as
  `revoked` on the next call (D13).

  Links: area Stack · amends capability-door D4 · draws on stack.1, stack.3 · summary N1 channels carry the generated WIT calls for every provider; script-wire ships pipe and ipc-channel transports; a channel's end closes its sessions.

- **D20, a lapse reaches the client as its status (2026-11-01).** Asked
  for the drawbacks of turning the WIT carrier's `revoked` failure into the
  status the client acts on, Mark chose "New CarrierError::Lapsed(status)
  (Recommended)": chirograph's `CarrierError` gains `Lapsed(SessionStatus)`
  for `Revoked` and `Expired`; `WitCarrier` raises it for those typed
  failures and keeps one that `take_notice` would swallow for the next
  call; graphshell-client marks the session revoked or expired from it,
  purging per the cache policy. The JSON carriers keep reporting status
  inside snapshots. Knot and Turnstone do not use `CarrierError` and
  Cleromancy only constructs it, so no other repository needs a repin.

  Links: area Stack · amends capability-door D13 · draws on stack.2 · summary CarrierError gains Lapsed(status); the WIT carrier raises it for revoked and expired, and the client marks the session from it.

- **D21, the WIT carrier takes the local wires first (2026-11-01).** Asked
  whether the WIT carrier should supplant the JSON one, Mark chose "Local
  first, remote later (Recommended)": the generated calls carry every local
  channel, N1 (D19) and stdio, local and Djinn's door path as each is
  touched; the remote carriers (QUIC through iroh, the WebRTC door, network
  NDJSON) move once script-wire has an async transport, pushed notices and
  more than one call in flight, each measured against the JSON carrier
  first; browser native messaging stays JSON, which Chrome's protocol
  requires. Phase W1 carries it.

  Links: area Stack · draws on stack.1, stack.3 · summary The WIT calls replace JSON lines on local channels now and on remote ones once script-wire is async, pushes notices and pipelines; browser native messaging stays JSON.

## Phases

### C0. One contract, bound both ways
Built (Mere `32be2f0b0`, D17-D19). G6 does not need it.

Write the projection session's control surface (`CarrierRequestBody`'s
Discover, Open, Resume, Intent, Close and Suspend) as a WIT interface with a
`projection-session` resource. Snapshots and resource chunks return bytes in
chirograph's serde form as versioned CBOR (D6). Bind it from a native provider
and consumer, and from a component guest under `app-host`. D13 settles
what the existing contract left open: discovery and opening apart from
methods on an opened session, Resume rechecking authority, revocation and
cancellation apart from Close and Suspend, and size limits. D14 places it. Flow
control is already settled: a chunk request acknowledges the previous chunk,
and notices carry no payload (`crates/chirograph/src/lib.rs:1131-1205`).
- Done when: the native pair and the component exchange the same calls and
  payloads; the session is bound through both D12 routes, a generator of our
  own over `wit-parser` and `wit-bindgen-wrpc` with our transport, and their
  effort, generated size, dependency closure and fit with the blocking
  `Carrier` are compared before one is chosen and the other recorded as
  rejected; a test shows our generated types and Wasmtime's encode the same
  values alike; the default build needs no C toolchain; the existing JSON door
  path still works unchanged.
- Needs: nothing. This phase is in-process; carriage is N1.

### W1. One wire for the projection session
The generated WIT calls (D19) replace JSON lines carrier by carrier (D21).
Local first: N1's channels carry them from the start, and the stdio, local
and Djinn door carriers move as each is next touched. Remote next: QUIC,
the WebRTC door and network NDJSON move once script-wire has an async
transport, notices pushed rather than pulled, and more than one call in
flight. Browser native messaging stays JSON, as Chrome requires.
- Done when: stdio, local and the door path carry the generated calls and
  their JSON form is gone; script-wire's async transport, pushed notices
  and pipelining are measured against the JSON carrier over QUIC and
  WebRTC before those move; afterwards JSON lines remain only on browser
  native messaging.
- Needs: C0 (built). The remote half needs script-wire's async transport.

### I1. Identity at the door
Next (D4). The door now lives in Armillary's `djinn` crate, so this phase
works in two repositories, Armillary first. Its first step extracts the OS
peer facts into one small Armillary crate (D7): `custody-wire`'s `Peer`
(`ports/mere-resident/custodian/wire/src/peer.rs`, 246 lines) and the door's
same-user check (`crates/djinn/src/door/local_endpoint.rs:300-345`).
- Done when: the door records each connection's OS peer facts; outside
  development mode it refuses a label that disagrees with them, on macOS
  (signature) and Windows (image); Djinn launches Linux applications with a
  handle and refuses a Linux label without one; status shows each
  connection's identity basis, including that a Windows image match is a
  same-user posture.
- Windows remainder (D11, D16; built 2026-10-31): Djinn launches Windows
  applications, holding the child's process handle, and admits a pinned
  label when the pipe recorded that child, by id and creation time; the image
  pin reads the image and creation time from one process handle and refuses
  a process created after the connect; token facts are read through the pipe
  with nothing awaited while impersonating; the door's pipe gets an explicit
  access list and refuses remote clients. Run on Windows, not only
  cross-checked.
- Needs: nothing.

### M1. Manifests for door apps
Each application declares what it offers and uses: routes, mere routes,
custody salt namespaces and persona use. Djinn routes from the declarations.
- Done when: `CUSTODY_APPS`, the `first_party` list and the per-app grants in
  `bin/djinn.rs` are gone; a request outside an application's manifest is
  refused with a typed reason; the MCP door's ban on Retinue routes is a
  manifest check; Woodshed and Hocket declare theirs.
- Needs: I1. A manifest is matched to an identity, not a label.
- Knot first, ahead of I1 (Mark, 2026-10-30: "Application registration is
  next, with Knot as the first complete case"; declaration on the hello,
  namespace grants with Mere defaults, Knot's custody limited to what it
  declares). Declarations stay statements under a label until I1; they give
  honest applications least authority, not authentication.

### S1. Applications serve routes
The door's missing message: an application offers a route (a projection or
another service) into Djinn's catalog. Mostly built for projections in
`c3567c5ab`
([application projection offers](2026-10-30_app_projection_offers.md)):
`ProviderCall` Offer, Next and Withdraw over the reserved
`mere.projection-provider` route, owner-set provider permissions apart from
viewer grants, and generation-bound leases withdrawn on disconnect, revocation
or timeout. What remains is driving those permissions from M1's manifests and
serving routes other than projections.
- Done when: a route one application serves is reachable by a second
  admitted application through Djinn, and is withdrawn when the serving
  application disconnects.
- Needs: M1.

### N1. Introduction
Djinn hands a consumer a channel to a provider's served route after consent:
a direct pair where the manifest marks the route direct, a proxied channel
otherwise. A passed channel's peer facts say nothing trustworthy about who
holds it, so the consumer's authority travels in D8's value, never in the
socket's identity.
- Done when: on Linux, macOS and Windows a consumer gets a channel only after
  consent; revoking a proxied grant refuses new calls and settles in-flight
  ones by a stated rule; a direct grant ends when its provider is told; the
  carrier is ipc-channel's OS backend at 0.23.1 or later (D10) or a small
  layer of our own, with the reasons recorded.
- Needs: S1 and C0's promotion: every N1 channel carries the generated WIT
  calls (D19); native channels need no Wasm.

### K1. The capability contracts in WIT
Persona, settings, custody facets and projection-session control, each
promoted only with a native and a component consumer. Settings follow the
revisioned contract in `surface-api` that Turnstone's settings owner uses.
- Done when: each promoted interface has both consumers, and `mere:script`'s
  string `caps` and JSON `actions` are expressed through them or kept with a
  stated reason.
- Needs: C0.

### G1. The gateway to Murm
Deferred (D5) until a remote consumer is named and OCapN's encoding and
error types settle.
- Done when: a route served on one device is invoked from a peer through
  `murm-ocapn`, and a peer's object reaches a local application as a channel;
  both sides agree on session scope, breaking on disconnect, deliver versus
  deliver-only and no automatic replay, with domain-owned idempotency for
  writes. Third-party handoffs between devices stay Murm's O4.
- Needs: N1 and Murm's O3.

## Open questions

- **Where D8's value lives.** Djinn (Armillary) holds `AppAuthority` and
  Notochord (Dramatis) holds `RetainedAuthority`; the type needs a home both
  directions of dependency allow.
- **How native code binds WIT.** C0 decides.
- **Where consent is asked, and how it reads.** Djinn asks under castellan's
  rule; the surface's owner is not named.
- **Linux identity beyond launch.** Flatpak application ids and desktop
  portals are the platform's route; not surveyed.
- **Whether introduction may cross personas.** The host plan keeps
  cross-application persona linkage opt-in; D9 gives each grant its
  personas, and an introduction's rule across them is open.
- **Which routes are direct and which proxied by default.**
- **Custom clipboard formats as a capability.** Mark, 2026-10-30, choosing
  upstream arboard over the deleted fork: custom OS clipboard formats
  (Hocket's `audio/wav`) "should be provided through the new boundaries",
  with arboard used as a library rather than forked. Which facet carries them,
  and on which platforms, is open.

## Findings

- 2026-10-30: the door admits one route per connection under an `AppId` its
  client declares. The code calls it "a *label*, not a credential", and the
  endpoint's permissions are the only proof
  (`crates/graphshell/graphshell-endpoint/src/door/app_admission.rs:22-28`).
  `AllowedAppRoutes::default` admits only Turnstone (`:255`), and
  `AppMessage` has no message by which an application offers a route
  (`door/wire.rs:62`). Superseded the same day: see the revision's findings
  below.
- 2026-10-30: Djinn grants routes by label in code: identity, Knot, site,
  devices, status, control, custody, reservoir, chronicle and inference
  (`ports/mere-resident/src/bin/djinn.rs:1245-1339`). `CUSTODY_APPS` lists ten labels
  (`ports/mere-resident/src/custody.rs:46`) and a `first_party` list four
  (`bin/djinn.rs:1284`). Djinn launches no applications; its only child
  processes are identity dialogs (`ports/mere-resident/src/identity_ui.rs`).
- 2026-10-30: `custody-wire`'s `Peer` already reads OS peer facts:
  `SO_PEERCRED`, `getpeereid` and `LOCAL_PEERPID`, a macOS audit token checked
  against the code signature's team and identifier, and a Windows pipe
  client's process and image (`ports/mere-resident/custodian/wire/src/peer.rs`). The
  custodian's policy records a Windows image match as "the same-user posture
  of a polkit authentication agent, not an OS-held identity"
  (`ports/mere-resident/custodian/src/policy.rs:96-108`). Its Linux platform hands a
  child it forks one end of a socket pair (`platform/linux.rs:268`), the
  launch-handle shape D2 uses.
- 2026-10-30: `mere:script@0.1.0` has two worlds, `app-core` and
  `document-core` (`crates/script/wit/world.wit:168,183`), hosted on Wasmtime
  45 by `crates/script/app-host` and `document-host`. Its `caps` is a list of
  strings and its `actions` a name with a JSON payload; it declares no
  resources. The Moot capsule-library guest uses `app-core`, and its native
  host checks each action against signed Servitor delegations
  (`ports/moot/examples/capsule-library/native.rs`).
- 2026-10-30: chirograph is 2,019 lines with 63 serde types, using
  `BTreeMap` fields and boxed variants (`crates/chirograph/src/lib.rs`). WIT
  has no recursive types (component-model `WIT.md`: "Types, however, cannot
  be recursive"). The map half needs a version: WASI 0.3.1 (2026-08-11)
  adopted the component model's `map<K, V>`
  ([WASI roadmap](https://wasi.dev/roadmap)); WASI 0.2 and Wasmtime 45's
  hosts here predate it.
- 2026-10-30: Genet vendors ipc-channel 0.22 with its in-process backend as
  the default, because the OS backend's `windows` 0.61 pin collided with
  wgpu-hal's 0.62 (Genet `Cargo.toml`, `support/patches/ipc-channel`).
  Nothing in Mere passes a descriptor or handle to another running process.
- 2026-10-30, upstream: wRPC 0.18.0 (2026-10-23) carries WIT calls over Unix
  sockets, TCP, QUIC and WebTransport and encodes resources as opaque bytes
  (its `SPEC.md`, Resources), so it does not keep handle semantics across
  processes. WASI 0.2 guests build on stable Rust, and `wasm32-wasip3` reaches
  stable in Rust 1.100. There is no WASI GPU standard: `wasi:surface` left
  WASI on 2026-08-24 and `wasi:webgpu` is at Phase 2. OCapN's "a finish line"
  milestone has ten issues open and none closed, among them its encoding,
  ordering guarantees and error types.
- 2026-10-30: the `mark-ik/arboard` custom-formats fork no longer exists, so
  any graph resolving Genet's clipboard failed (Turnstone, Hocket). Genet
  `b7c0bef` returns `genet-clipboard` to arboard 3.7: one primary
  representation per write, custom formats reported `Unsupported`. Hocket
  (Woodshed `e1d93b0`) keeps copied loop audio host-locally, so audio paste
  works only inside one running Hocket.
- 2026-10-30, revision: `c3567c5ab` ("Admit application projection offers
  through leased resident routes", 01:52, seven minutes after this plan)
  added the provider route, owner policy and leases S1 describes, for
  projections (`ports/mere-resident/src/door/projection_offers.rs`,
  `crates/graphshell/graphshell-endpoint/src/door/projection_offer.rs`).
  `675540865` then moved the neutral door modules (`app_admission`, `wire`,
  `frame`, `local_endpoint`, `app_client`, the custody modules) to Armillary's
  `djinn` 0.0.3, pinned at `721e17d`
  (`armillary/design_docs/armillary_docs/implementation_strategy/2026-10-30_djinn_ownership.md`);
  Mere's resident composition is `ports/mere-resident` (renamed from
  `ports/djinn` in `28a39cef2`).
- 2026-10-30, revision: the Armillary door checks only that a peer is the
  same user (a 0600 socket on Unix, SID equality on Windows,
  `crates/djinn/src/door/local_endpoint.rs:300-345`) and keeps no peer
  facts; `door/server.rs` admits by the `AppId` and declaration in the
  hello. Unchanged at `a333f51`, the revision Mere now pins.
- 2026-10-30, revision: the door's wire is JSON lines (`CarrierOutput` is
  `#[serde(untagged)]`), and chirograph uses internally tagged enums
  (`#[serde(tag = "type")]`, `lib.rs:927`) and `skip_serializing_if`.
  Postcard cannot decode either; CBOR can (D6).
- 2026-10-30, revision, upstream: OCapN agreed on 2026-08-11 to replace
  Syrup with a new binary encoding
  ([ocapn#275](https://github.com/ocapn/ocapn/issues/275), draft PR #284);
  its varint, tag layout and in-band promise and error forms are open. The
  "first consensus draft" milestone shows one issue closed; "a finish line"
  shows ten open. Murm's plan already records this and keeps its pinned
  `ocapn-suite-31f0b806` Syrup profile (D5).
- 2026-10-30, effective authority (D8): the remote Gemot path checks
  subject admission and domain authority per method, with no deputy
  (`MootSessionAuthority::from_admitted`, then lapse, coverage, Moot
  membership and `authorize_current_capability`,
  `ports/moot/src/ocapn_coop.rs:65-360`). Graphshell's viewer sessions keep
  `RetainedAuthority` in `SessionAuthority` and name the application in
  `AdmittedEndpointContext` as attribution only
  (`crates/graphshell/graphshell-host/src/lifecycle.rs:105-150`). Djinn's
  `AppAuthority` is declaration and owner grant with no persona term
  (Armillary `d0c79f5`, `crates/djinn/src/door/declaration.rs:244-340`).
- 2026-10-31, review of `d0c79f5` and `b6578ee` against D7 and I1: D7 holds
  (one crate, both consumers, the moved Windows functions unchanged, custody
  policy left in custody-wire), and `b6578ee` closes the review's macOS gap
  (the audit token kept as `Connected`), refuses a pinned label with no peer,
  and Mere pins `djinn` and `peer-facts` at one revision (`73968e2`). Three
  points remain. The Windows image is read by process id after the pipe
  connects (`peer-facts` `pipe_client_peer`), so a reused id reports another
  process's image; status should say the match was read at connect. On
  Linux, `DJINN_LAUNCH_FD` stays in the environment and descriptor 3 is
  inherited by any child spawned before the application's first route hello
  calls `launch::inherited()`, so such a child can present the handle;
  reading it at startup, or clearing the variable on read, closes that.
  `admit` and `serve_on` remain as label-only paths beside `admit_from`;
  they refuse under a pin, so this is cleanup, not a hole. `peer-facts` is
  not in the naming ledger, which the tier rule allows while it is
  unpublished.
- 2026-10-31, upstream, for N1's carrier: std's ancillary data is still
  unstable ([rust#76915](https://github.com/rust-lang/rust/issues/76915))
  and leaves macOS out; rustix 1.1.6 (2026-10-27) sends `ScmRights` through
  `sendmsg` and compiles no C, as does nix 0.31. `CommandExt::fd`, the
  launch-handle API, is accepted but unstable
  ([rust#144989](https://github.com/rust-lang/rust/issues/144989)).
  ipc-channel 0.23 passes channels on all three systems (SCM_RIGHTS, Mach
  ports, named pipes with `DuplicateHandle`) and now takes `windows` 0.62,
  the line Genet's fork collided with; wgpu-hal's current pin is unchecked.
  [GHSA-fj85-w6rj-rxrc](https://github.com/servo/ipc-channel/security/advisories/GHSA-fj85-w6rj-rxrc)
  (high, 2026-10-24): on macOS a sender could make a receiver read and free
  arbitrary memory; fixed in 0.23.1, so N1 on ipc-channel takes 0.23.1 or
  later. Genet's vendored 0.22 predates the fix, which matters only if its OS
  backend is enabled.
- 2026-10-31, upstream, for D2: Linux's `SO_PEERPIDFD` (kernel 6.5) gives a
  pidfd for the connecting process; nix exposes it and rustix does not yet
  ([rustix#1474](https://github.com/bytecodealliance/rustix/issues/1474)).
  xdg-desktop-portal's host app registry
  ([#1521](https://github.com/flatpak/xdg-desktop-portal/pull/1521), in
  1.19.4) records an id the application declares itself, a label rather
  than identity, and nothing upstream verifies unsandboxed applications;
  the launch handle stays Linux's identity. On macOS, `LOCAL_PEERTOKEN` with
  the code signature is supported from macOS 13, while Apple steers toward
  XPC peer code-signing requirements
  (`xpc_connection_set_peer_code_signing_requirement`, macOS 12;
  `XPCPeerRequirement`, macOS 26), and Apple's developer support notes that a
  socket's peer identity is not trustworthy once its descriptor is passed on.
  Windows was not surveyed.
- 2026-10-31, upstream, for C0: Wasmtime's `component::bindgen!` output
  needs a `Store`, with no documented store-free use, and wit-bindgen targets
  wasm guests. `wit-bindgen-wrpc` 0.12.0 generates native clients and
  servers over a pluggable transport (`Invoke`, `Serve`), but encodes
  resources as opaque bytes, lacks `map`
  ([wrpc#1151](https://github.com/bytecodealliance/wrpc/issues/1151)), and
  changed its transport API on 2026-10-30. No upstream tool binds WIT
  natively with handles carried as OS channels. jco 1.40.0 (2026-10-28) and
  its WASI 0.3 shim claim browser support on the 0.2 model.
- 2026-10-31, upstream, for G1: OCapN has had no commits since 2026-10-13;
  the encoding draft
  ([#284](https://github.com/ocapn/ocapn/pull/284)) is open and the "a
  finish line" milestone has ten issues open and none closed. D5's deferral
  stands.
- 2026-10-31, Windows survey for D2: `peer-facts` reads the pipe client's
  process id after the connect and opens that id twice, once for the SID and
  once for the image, so a reused id can mix two processes; its comment says
  so (Armillary `7eebeeb`, `crates/peer-facts/src/lib.rs:358-372`). The
  door's pipe is created with default security
  (`crates/djinn/src/door/local_endpoint.rs:57-59,177`), unlike Mere's
  custodian pipe, which sets a protected access list and rejects remote
  clients; `door/launch.rs` launches on Unix only. Windows returns no process
  handle for a pipe client, only the id the pipe recorded at connect; a held
  handle keeps that id from being reused
  ([Old New Thing](https://devblogs.microsoft.com/oldnewthing/20110107-00/?p=11803)),
  and Project Zero describes a deliberate same-user reuse attack against the
  id alone
  ([2019](https://projectzero.google/2019/09/windows-exploitation-tricks-spoofing.html)).
  Sparse MSIX packages are the only OS-held program identity for unpackaged
  applications and need a certificate, a manifest and installer registration;
  Authenticode proves a file's signer, not the running process. Win32-OpenSSH's
  agent reads its client the way the door does today; Chromium's Mojo does not
  identify by process id at all and relies on access lists and inherited
  endpoints. No approach stops a same-user process injecting into another.
- 2026-10-31, for C0: Wasmtime's imported-interface `Host` traits work
  without a `Store`, but its generated types derive Wasmtime's own traits,
  and its `runtime` feature compiles `helpers.c` through `cc`, so native use
  brings Wasmtime and C into every process. `wit-parser` 0.263.0 is pure Rust
  and states no API stability, with a new 0.X each release.
  `wit-bindgen-wrpc` 0.12.0 generates native clients and `Handler` traits over
  a pluggable `Invoke`/`Serve` transport and any async reader and writer; its
  spec is a draft, one maintainer wrote nearly all recent commits, its main
  branch carries unreleased breaking changes, it needs tokio and `Send`, opens
  one Unix-socket connection per call and has no `map`; `wrpc-wasmtime` needs
  Wasmtime 50, and Mere's hosts are on 45.
- 2026-10-31, for D10: Genet's ipc-channel users are its media components
  only, sending player events and seek acknowledgements
  (`components/media/player/lib.rs:72-81`, `backends/gstreamer/player.rs`).
  Turnstone's lock holds both Genet's fork 0.22.0 (for `servo-media-player`)
  and crates.io 0.23.0 (for Servo 0.7), with one `windows` 0.62.2, so the
  0.61 clash is already absent there.
- 2026-10-31, for C0: one admission mints one projection session. The id is
  derived from the admission transcript (graphshell-host
  `lifecycle.rs:23-29,204-207`), a client cannot name another, and
  graphshell-client keeps one mounted projection per id; several ids under
  one carrier occur only on stdio and local, which refuse Open. So a resource
  per Open is a resource per projection session, and no plan needs one
  resource per offer.
- 2026-10-31, C0 probe (`crates/probes/projection-session-c0/`, Mere
  `e5cf4cbd5`): one script of 19 calls (a refused
  open, both caps, a stale intent, both notice calls, suspend, and three calls
  after revocation) gives the same transcript through route A (our generator,
  OS pipes), route B (`wit-bindgen-wrpc` 0.12.0, Windows named pipes), the
  Wasmtime 45 guest and a direct call. Route A's encoding equals wRPC's byte
  for byte on failures, notices, replies and an opened session, and each
  decodes the other's; values cross Wasmtime unchanged. Hand-written code:
  A 277 (generator) + 193 (runtime) + 214 (adapter) lines, B 321. Generated:
  A 214 lines, B 3,246. Normal dependencies: A 37 crates, B 81 (tokio,
  futures, anyhow, bytes, tracing). B needs a runtime per consumer, a
  `Send + Sync + Clone` handler (the provider behind a mutex), a connection
  per call and a transport per platform, and it has no resource-drop hook,
  so D13's drop-is-close cannot be expressed: its provider never saw Close.
  A is blocking, `&mut self`, single-threaded, and its drop reaches the
  provider. Neither route adds a C build, but chirograph already compiles
  blake3's C and assembly through `cc` (no `pure` feature), so no native
  path that carries chirograph builds without a C toolchain today;
  Wasmtime's `cc` users (zstd, ittapi, its fiber and helpers) stay on the
  component side. `resource` is a WIT keyword, so the fetch method is named
  `fetch`. Run on Windows only; not timed.
- 2026-10-31, C0 generator measured (Mere `54ac51415`): calls now travel under
  wRPC's names rather than their position in the WIT, and Rust keywords are
  escaped. Against wRPC, 20,000 random values per property on a conformance
  interface (every supported type nested, keyword names) encode to the same
  bytes and decode back unchanged; 20,000 mutated encodings are accepted by
  ours exactly when wRPC accepts them. That test found a u64 decode that
  truncated a ten-byte integer instead of refusing it; fixed, with a
  regression test. A corrupted case is refused by both (the positive
  control); both accept non-minimal integers, so the wire is not canonical.
  Pure random bytes reached past the eighth byte only 10 times in 20,000 and
  were replaced by mutation. Coverage, counting an interface bound when code
  is generated and its first refusal otherwise: 25 of 57 interfaces across
  WASI 0.2.6, WASI 0.3.0-rc-2026-03-15 and `mere:script` (29 of the 32
  refusals are a type `use`d from another interface, 2 enums, 1 async
  function), and 2,123 of 3,430 interfaces in 1,999 random `wit-smith`
  packages (use 308, enum 144, flags 125, then the signed, float and `char`
  types, static functions, constructors and tuples over six). A Wasmtime
  differential adds nothing to wRPC's for our wire, since Wasmtime decodes
  the canonical ABI, not this encoding; values crossing Wasmtime unchanged
  stays the existing check.
- 2026-11-01, C0 generator's gaps closed (Mere `62b3884de`, Mark: "All
  synchronous gaps"): every primitive, enums, flags, tuples to sixteen,
  static functions, constructors and types `use`d from other interfaces
  (emitted as sibling modules) are bound in wRPC's encoding; async
  functions, streams, futures, maps and fixed-length lists are refused.
  The conformance interface now carries them all and agrees with wRPC on
  20,000 random and 20,000 mutated values, and at each type's edges
  (signed extremes, NaN written canonically, the last `char`, a flag bit
  past the last refused by both). Coverage: 51 of 59 real interfaces (all
  of WASI 0.2.6 and `mere:script`; the 8 refused are WASI 0.3's async
  functions, streams and futures) and all 3,430 interfaces in 1,999 random
  `wit-smith` packages, whose default configuration generates none of the
  refused features. `c0-corpus` compiles the 551 emitted interfaces (the
  real ones and the first 500 random) with no errors; a planted error is
  caught. Run on Windows only.
- 2026-11-01, C0 promoted (Mere `32be2f0b0`, D18, D19): `script-witgen` (the
  generator, a build-dependency only) and `script-wire` (value encoding,
  `Transport`, OS-pipe and, behind `ipc`, ipc-channel 0.23.1 transports) in
  `crates/script`; the projection-session interface and a
  `projection-provider` world in `mere:script`; graphshell-endpoint's `wit`
  feature with `WitCarrier` (a chirograph `Carrier` over any
  `SessionCalls`), `WitServer` (a complete endpoint behind the generated
  handler, authority rechecked through a `SessionGate` on every call) and
  `serve`, which closes a channel's sessions when it ends. One call script,
  run through `Carrier`, gives the same transcript over pipes, ipc-channel,
  an ipc end passed over a channel, and a Wasmtime 45 guest under app-host
  served by the same `WitServer`; a vanished provider reads as
  `Disconnected`. Tests: script-wire 14 (9 against wRPC, 5 transport),
  script-witgen's golden test, graphshell-endpoint 22 with its stdio, local
  and network tests unchanged, app-host's two component tests. Coverage from
  its new home: 49 of 57 real interfaces (WASI 0.2.6, WASI 0.3 rc and
  `mere:script`; the 8 refused are async, streams and futures) and all of
  1,999 random packages; the 549 emitted interfaces compile. The `wit`
  feature takes graphshell-endpoint's dependencies from 29 crates to 37 and
  adds no C build beyond chirograph's blake3. `wait-notice` polls the
  endpoint every 5 ms, since `ProjectionNoticeSource` offers no blocking
  wait. Run on Windows only; macOS and Linux not built.
- 2026-11-01, D20 built (Mere `1dd59c48a`): chirograph's `CarrierError` gains
  `Lapsed(SessionStatus)`; `WitCarrier` raises it for `revoked` and
  `expired` and keeps one met in `take_notice` for the next call;
  graphshell-client's session marks its scenes from it (revoked purges per
  the cache policy, expired keeps the bytes); graphshell-mcp treats it as
  the session's end. The only exhaustive match outside chirograph was
  graphshell-mcp's; Knot and Turnstone do not use `CarrierError` and
  Cleromancy only constructs it. 217 tests pass across chirograph,
  graphshell-client, graphshell-mcp, graphshell-endpoint and app-host, the
  component test on a freshly built guest among them. Windows only.

## Progress

- 2026-10-30: plan opened; nothing built.
- 2026-10-30: M1's first case, Knot. Djinn's `door::declaration` carries an
  application's routes, offers, custody operations and epoch derivations on
  the route hello; the owner's `AppGrant` names what it may receive, by
  namespace, and a connection gets both or a typed `Undeclared` /
  `NotGranted` refusal. Mere supplies grants from owner settings
  (`applications`) or `ports/mere-resident/policy/default_applications.json`.
  Knot left `CUSTODY_APPS`, the `first_party` list, the published-site grant
  and `ReleasePolicy::default()`'s epoch rules; its site handoff client moved
  into Knot (`knot_site`, contract in `knot_site::handoff`) and Mere's
  `djinn-site` is gone. Remaining for M1: the other applications' entries,
  salt namespaces in declarations, `reservoir_applications()` for the
  embedded reservoir owner, the MCP door's Retinue ban, and I1.
- 2026-10-30: revised after `c3567c5ab` and the Armillary move (D4 to D7):
  S1 marked mostly built, I1 next before M1's remaining entries, C0 held,
  G1 deferred, payloads CBOR, and the OS peer facts homed in one Armillary
  crate.
- 2026-10-30: Turnstone's declaration built and parked behind I1 (D4) on
  `agent/turnstone-declaration` in Armillary (`b2388f4`: salt namespaces as
  `exact`, `prefix` or `delegation_scope`, declarations through
  `CustodyIdentity`, an `AllowedAppRoutes::default()` that admits no one),
  Mere, Knot and Turnstone. Armillary `main` reverted `b2388f4` (`1cb9518`);
  landing the branches repins the cohort to it.
- 2026-10-30: I1's first step, D7. Armillary `d0c79f5` adds `peer-facts`
  (Unix credentials and pid, Windows pipe token SID and image, macOS code
  signature against the audit token), and Djinn's listener and admission
  record each connection's peer (`serve_with_peer`, `admit_from`,
  `AdmittedApp::peer`). custody-wire reads peers through it and keeps its
  custodian policy. Mere adopts the recorded peers when it repins Djinn.
- 2026-10-30: effective authority mapped; D8 (Djinn computes, services
  recheck) and D9 (grants name personas) recorded, to be built with N1 or G1.
- 2026-10-30: I1 built. Grants pin a label per platform (macOS signature,
  Windows image, Linux launch); unpinned is development mode (Mark: "Per grant:
  no identity pinned", "Include a minimal launcher"). Armillary `b6578ee`
  checks the pin at admission against `peer-facts::Connected` (the macOS audit
  token kept from accept) and launches with a one-time handle on descriptor 3
  that the door client carries on route hellos. Mere reports each connection's
  basis and peer account in status and launches by `djinn --launch <app>`.
  Not run here: the Linux launch-pin test (cross-checked only) and Windows
  image matching on Windows.


- 2026-10-30: M1's second case, Turnstone. Declarations and grants name salt
  namespaces (`exact`, `prefix`, `delegation_scope`), released or acts only;
  `CustodyIdentity` and the blocking custody calls carry declarations. Turnstone
  left `CUSTODY_APPS`, the first-party list, the binary's identity, Knot,
  chronicle and inference grants, and the label release policy's Turnstone
  namespaces; Djinn's `AllowedAppRoutes::default()` admits no one. Remaining
  label tables: `CUSTODY_APPS` for the other applications,
  `reservoir_applications()`, and the payload scope chosen by label.
- 2026-10-30: Turnstone's declaration landed after I1: Armillary `73968e2`
  reapplies `b2388f4` on I1, and Mere, Knot and Turnstone merged their
  `agent/turnstone-declaration` branches. Turnstone's label is admitted by
  its declaration within Mere's default grant, in development mode until
  its identity is pinned.
- 2026-10-31: the review's three findings closed. Armillary `7eebeeb` adds
  `launch::claim`, which Knot's binaries call first in `main` (Knot
  `5e47782`); the Windows image basis is described as read at connect;
  `admit` and `serve_on` are removed. Mere `81c83c5`, Cleromancy `4a09fc9`.
  Turnstone stays on Djinn `73968e2` with Mere `68608e3`: every Mere commit
  carrying `7eebeeb` follows `081c650ae`, which removed athanor's `furnace`
  feature Turnstone enables. When Turnstone receives that change, it repins
  Djinn and calls `djinn::door::launch::claim()` first in `main`.
- 2026-10-31: D10 to D12 ruled: one upstream ipc-channel, Windows identity
  through launch, and C0 lifted to probe two binding routes. Nothing built.
- 2026-10-31: D10 landed. Genet `a3ea104bb9a` drops the vendored fork for
  crates.io ipc-channel 0.23.1 with no flag; `30e4c190ca3` before it lets
  `servo-media-derive` enable syn's `proc-macro` feature itself, which it had
  only through other crates' features. Turnstone `317f0bf7` removes its patch
  row and locks one ipc-channel 0.23.1 and one `windows` 0.62.2. Because
  0.23.1 caps uuid below 1.27.0 for Servo's Rust 1.88, uuid moves from 1.27.0
  to 1.26.1, the pair Servo's own main locks; 1.27.0 changed no public API.
  Checked on this Mac: Genet's media crates and Turnstone with `audio`. Not
  run: Turnstone's `servo` feature and a Windows build on o-pc.
- 2026-10-31: D13 and D14 ruled: the projection session's contract (free
  discover and open, a session resource, authority rechecked on every call,
  revocation as status and a typed failure, 1 MiB payloads) and where C0's
  pieces live. The two-route probe starts under
  `crates/probes/projection-session-c0/`.
- 2026-10-31: D10 checked on Windows. On o-pc (Rust 1.99.0) Turnstone
  `317f0bf7` checks with `audio,scry`: one ipc-channel 0.23.1 and one
  `windows` 0.62.2 build together with Genet's media. It first needed
  Turnstone `73565e33`: `forward_native!` refused the trailing commas in
  three calls, which broke the Windows `scry` build before D10 as well
  (`7d72d9e8` failed the same way).
- 2026-10-31: D11 built, with D16. Armillary `104b8d4`: Windows
  `launch::launch` registers the launched process by id and creation time,
  and a Windows pin admits the label when the pipe recorded that process, or
  else the pinned image. `peer-facts` reads a pipe client's image and
  creation time from one process handle and refuses a process created after
  the connect (`pipe_client_peer` too, which custody-wire's custodian uses),
  and `Connected::pipe_client` reads the client's token through the pipe.
  The door's pipe instances carry a protected access list for SYSTEM and the
  user and refuse remote clients, and the door refuses another user, an
  AppContainer and anything below medium integrity by that token. Mere
  `c92368402` repins Armillary, adds `djinn` and `peer-facts` to the
  armillary patch so Knot's `7eebeeb` pin resolves to the same packages, and
  launches a Windows grant's pinned image. Run on o-pc: Djinn's 52 tests and
  peer-facts' pass, among them the token read before any byte, the refusal
  of a later process, the access list, launched-process admission and the
  image fallback. Mere's launcher was cross-checked for Windows, not run.
- 2026-10-31: C0's two-route probe built and run on Windows; D17 chose our
  own generator. Promoting it into a shipping crate and moving the interface
  into `mere:script` remain.
- 2026-10-31: the C0 generator measured against wRPC, mutated input and
  real WIT; one decode bug found and fixed. Promotion remains.
- 2026-11-01: the C0 generator covers the synchronous Component Model and
  compiles what it binds. Promotion remains.
- 2026-11-01: D18 and D19 ruled: C0's promotion into script-wire,
  script-witgen and graphshell-endpoint's `wit` carrier, and what N1's
  channels carry.
- 2026-11-01: C0 built and promoted (Mere `32be2f0b0`); the probe is deleted.
- 2026-11-01: D20 ruled: a lapse reaches the client as its status.
- 2026-11-01: D20 built (Mere `1dd59c48a`); D21 ruled and phase W1 added.
