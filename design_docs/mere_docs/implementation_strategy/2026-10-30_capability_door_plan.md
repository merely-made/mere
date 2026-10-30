# Capability door: app identity, manifests, introduction and WIT contracts

**Status:** 2026-10-30: plan. Nothing is built. Findings were read against
Mere `56fbdc2e4`, Murm `96e8843e814` and upstream on 2026-10-30; no test was
run for them.

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

## Phases

### C0. One contract, bound both ways
Write the projection session's control surface (`CarrierRequestBody`'s
Discover, Open, Resume, Intent, Close and Suspend) as a WIT interface with a
`projection-session` resource. Snapshots and resource chunks return bytes in
chirograph's serde form under a named format. Bind it from a native provider
and consumer, and from a component guest under `app-host`.
- Done when: the native pair and the component exchange the same calls and
  payloads; the binding route for native code is chosen and the rejected ones
  recorded (Wasmtime's `bindgen!` types, `wit-bindgen-wrpc` over a channel
  transport, a small generator of our own); the default build needs no C
  toolchain; the existing JSON door path still works unchanged.
- Needs: nothing. This phase is in-process; carriage is N1.

### I1. Identity at the door
- Done when: the door records each connection's OS peer facts; outside
  development mode it refuses a label that disagrees with them, on macOS
  (signature) and Windows (image); Djinn launches Linux applications with a
  handle and refuses a Linux label without one; status shows each
  connection's identity basis, including that a Windows image match is a
  same-user posture.
- Needs: nothing.

### M1. Manifests for door apps
Each application declares what it offers and uses: routes, mere routes,
custody salt namespaces and persona use. Djinn routes from the declarations.
- Done when: `CUSTODY_APPS`, the `first_party` list and the per-app grants in
  `bin/djinn.rs` are gone; a request outside an application's manifest is
  refused with a typed reason; the MCP door's ban on Retinue routes is a
  manifest check; Woodshed and Hocket declare theirs.
- Needs: I1. A manifest is matched to an identity, not a label.

### S1. Applications serve routes
The door's missing message: an application offers a route (a projection or
another service) into Djinn's catalog.
- Done when: a route one application serves is reachable by a second
  admitted application through Djinn, and is withdrawn when the serving
  application disconnects.
- Needs: M1.

### N1. Introduction
Djinn hands a consumer a channel to a provider's served route after consent:
a direct pair where the manifest marks the route direct, a proxied channel
otherwise.
- Done when: on Linux, macOS and Windows a consumer gets a channel only after
  consent; revoking a proxied grant refuses new calls and settles in-flight
  ones by a stated rule; a direct grant ends when its provider is told; the
  carrier is ipc-channel's OS backend or a small layer of our own, with the
  reasons recorded.
- Needs: S1 and C0.

### K1. The capability contracts in WIT
Persona, settings, custody facets and projection-session control, each
promoted only with a native and a component consumer. Settings follow the
revisioned contract in `surface-api` that Turnstone's settings owner uses.
- Done when: each promoted interface has both consumers, and `mere:script`'s
  string `caps` and JSON `actions` are expressed through them or kept with a
  stated reason.
- Needs: C0.

### G1. The gateway to Murm
- Done when: a route served on one device is invoked from a peer through
  `murm-ocapn`, and a peer's object reaches a local application as a channel;
  both sides agree on session scope, breaking on disconnect, deliver versus
  deliver-only and no automatic replay, with domain-owned idempotency for
  writes. Third-party handoffs between devices stay Murm's O4.
- Needs: N1 and Murm's O3.

## Open questions

- **How native code binds WIT.** C0 decides.
- **The WIT package:** extend `mere:script` or open a sibling package. Mark's
  2026-10-28 preference applies: "redefine existing names instead of minting
  new ones."
- **Where consent is asked, and how it reads.** Djinn asks under castellan's
  rule; the surface's owner is not named.
- **Linux identity beyond launch.** Flatpak application ids and desktop
  portals are the platform's route; not surveyed.
- **Whether introduction may cross personas.** The host plan keeps
  cross-application persona linkage opt-in.
- **Which routes are direct and which proxied by default.**

## Findings

- 2026-10-30: the door admits one route per connection under an `AppId` its
  client declares. The code calls it "a *label*, not a credential", and the
  endpoint's permissions are the only proof
  (`crates/graphshell/graphshell-endpoint/src/door/app_admission.rs:22-28`).
  `AllowedAppRoutes::default` admits only Turnstone (`:255`), and
  `AppMessage` has no message by which an application offers a route
  (`door/wire.rs:62`).
- 2026-10-30: Djinn grants routes by label in code: identity, Knot, site,
  devices, status, control, custody, reservoir, chronicle and inference
  (`ports/djinn/src/bin/djinn.rs:1245-1339`). `CUSTODY_APPS` lists ten labels
  (`ports/djinn/src/custody.rs:46`) and a `first_party` list four
  (`bin/djinn.rs:1284`). Djinn launches no applications; its only child
  processes are identity dialogs (`ports/djinn/src/identity_ui.rs`).
- 2026-10-30: `custody-wire`'s `Peer` already reads OS peer facts:
  `SO_PEERCRED`, `getpeereid` and `LOCAL_PEERPID`, a macOS audit token checked
  against the code signature's team and identifier, and a Windows pipe
  client's process and image (`ports/djinn/custodian/wire/src/peer.rs`). The
  custodian's policy records a Windows image match as "the same-user posture
  of a polkit authentication agent, not an OS-held identity"
  (`ports/djinn/custodian/src/policy.rs:96-108`). Its Linux platform hands a
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
  has no map type and no recursive types (component-model `WIT.md`: "Types,
  however, cannot be recursive").
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

## Progress

- 2026-10-30: plan opened; nothing built.
