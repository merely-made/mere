# Remote mere route: implementation spec

**Status (2026-10-15):** implementation-ready spec for the Codex lanes. Claude
designs; Codex implements (Mark's scope change, 2026-10-15). **Part of this is
already on Mere `main`** (commit `8b08c98b2`, pushed before the scope change
reached the Claude lane): §§2–6 say exactly what landed, so nothing there is
re-implemented. What remains is §7 (the one repin round), §8 (starting the
host in the Djinn binary), §9 (Turnstone's remote pane) and §10 (push
notices, a later phase). Questions for Mark are in §11; nothing else here
needs a decision.

Companion documents: the [transport agnosticism note](2026-10-15_transport_agnosticism_note.md)
(its §9 rulings) and the [remote mere route admission note](2026-10-15_remote_mere_route_admission.md)
(the grant shapes, in prose).

## 1. Rulings this spec implements

From the transport agnosticism note, §9 (Mark, 2026-10-15):

- **T1:** "an admitted byte stream or line channel plus ingress facts. No new
  carrier trait."
- **T2:** "NDJSON inside every stream. Length prefixes are used only as
  platform envelopes below admission."
- **T3:** "push on every carrier that can. Where push exists, `TakeNotice`,
  `WaitNotice` and Discover-as-poll retire. The native stream-to-lines pump
  lives in `graphshell-client`, so every remote session can use it, not only
  the overmap."
- **T4:** "same-persona devices by device grant, other people's Turnstone as
  place members by delegation, and `ambient/mere/<domain>` exposed remotely
  as well." The implementing pass "must set out [ambient's own admission and
  grant shape] before serving it."
- **T5:** "an application claim in the delegation. The persona's delegation
  names the application that may act."
- **T6:** "server side in `graphshell-host` (R4), client side in
  `graphshell-client`. No dedicated carrier crate."
- **T7:** Djinn's dialable projection host: "yes, on a dedicated resident
  key. It does not share personal sync's key or per-place keys."
- **T8:** "keep HTTP [signalling] for now."

From the coordinator relaying Mark: the Murm in-process transport kind is
"batched into the remote-route pass, which touches transport anyway, and do
one repin round then"; that round "also reaches Woodshed"; in it "Knot and
Turnstone drop `graphshell` from their manifests if nothing else needs it."

## 2. T4: the paths and grant shapes (landed)

Under Graphshell's action domain `mere.graphshell` and projection service
`/services/projection`:

| Route | Requested path | Action |
|---|---|---|
| `mere/<domain>` | `/services/projection/mere/<domain>/app/<app>` | `connect` |
| `ambient/mere/<domain>` | `/services/projection/ambient/<domain>/app/<app>` | `ambient` |

A grant is an ordinary `insigne::delegation::DelegationCertificate`:

- **Explicit:** `scope = CapabilityScope { domain: "mere.graphshell",
  resource: <dial host network id bytes>, path_prefix: <at or under
  "/services/projection/mere/<domain>/app/<app>">, actions: {"connect"} }`.
- **Ambient:** the same with `path_prefix` at or under
  `/services/projection/ambient/<domain>/app/<app>` and `actions:
  {"ambient"}`.

The two never confer each other: disjoint paths and different actions, and
`CapabilityScope::attenuates` narrows a path and takes a subset of actions,
so neither can be narrowed into the other. The dialable host's policy offers
exactly two `ServiceRule`s, `/services/projection/mere` (`connect`) and
`/services/projection/ambient` (`ambient`), both `ServiceAccess::MemberOnly`;
plain projection sessions are not served on it. Once admitted, a session
reaches the same `AccessEndpoint` the local door opens, so the owner's
per-application decisions (default grant, recorded denials, the ambient
opt-in, `MereRoutes::set_access`) apply to a remote application unchanged.

**Landed, Mere `8b08c98b2`:**

- `crates/cambium/mere-overmap/src/wire.rs`: `REMOTE_DOMAIN`,
  `REMOTE_MERE_SERVICE`, `REMOTE_AMBIENT_SERVICE`, `REMOTE_MERE_ACTION`,
  `REMOTE_AMBIENT_ACTION`; `enum RemoteRoute { Explicit, Ambient }`;
  `struct RemoteMereRequest { route, domain, application }` with
  `explicit(domain, app)`, `ambient(domain, app)`, `path()`, `action()`,
  `route_id()`, `parse(domain, path, action) -> Option<Self>`,
  `named_by(path_prefix) -> bool`. Tests: `remote_tests` (3).
- `crates/graphshell/graphshell-host/src/admission.rs`: `AMBIENT_ACTION`;
  `open_session_for(identity, network, profile, action: RequestedAction,
  class, nonce, binding, delegations) -> Result<SessionHello, HandshakeError>`.
  `serves_action` is unchanged (still exactly `connect` at
  `/services/projection`).

## 3. T5: the application claim (landed; no dramatis change)

Checked against dramatis `0234e7ee` (the revision Mere pins):
`insigne::delegation::CapabilityScope { domain, resource, path_prefix,
actions }` and its `attenuates` (path under the parent's, actions a subset);
Notochord's `LocalNetworkPolicy::service_rule` (the most specific rule that
covers the requested path at slash boundaries, so a base service admits its
children); the chain's leaf is its last certificate (`notochord::chain`).

**Dramatis changes: none.** The application is the `<app>` segment of the
requested path, and the dialable host accepts it only when the delegation's
leaf names it: `RemoteMereRequest::named_by(&leaf.certificate.scope.path_prefix)`,
that is, the leaf's `path_prefix` is at or under `…/<domain>/app/<app>`. A
broad grant (`/services/projection/mere`) is admitted by Notochord but served
nothing by this host. The admitted application is bound into the endpoint
context with `AdmittedEndpointContext::with_application`, as the local door
binds it.

**Landed, Mere `8b08c98b2`:**

- `graphshell-host/src/carrier.rs`: `type ActionCheck<'a> = &'a (dyn
  Fn(&AdmittedPrincipal, &[SignedDelegationCertificate]) -> bool + Send +
  Sync)`; `admit_stream_with(stream, facts, policy, ledger, now_ms,
  active_sessions, serves: ActionCheck)`; `accept_projection_session_with(…,
  serves)`. The plain `admit_stream`, `admit_accepted_session` and
  `accept_projection_session` keep `serves_action`.
- `graphshell-host/src/projection_host.rs`: `struct SelectedRoute { route:
  ResidentEndpointRoute, application: Option<String> }`; `type RouteSelector
  = Arc<dyn Fn(&AdmittedPrincipal, &[SignedDelegationCertificate]) ->
  Result<SelectedRoute, String> + Send + Sync>`; `type SharedCatalog =
  Arc<std::sync::Mutex<ResidentEndpointCatalog>>`;
  `ResidentProjectionHost::selecting(policy, catalog, selector)`;
  `ResidentProjectionHost::catalog() -> SharedCatalog`;
  `ResidentProjectionError::Route(String)`. `ResidentProjectionHost::new`
  (one fixed route) is unchanged for its callers.

## 4. T3's pump (landed)

**Landed, Mere `8b08c98b2`:** `crates/graphshell/graphshell-client`:

- feature `pump = ["dep:tokio"]` (Tokio `io-util` only; default build
  unchanged and wasm-clean);
- `driver::LineDriven` (always compiled, re-exported at the crate root):
  `fn take_outgoing(&mut self) -> Vec<String>; fn on_line(&mut self, line:
  &str) -> Result<(), String>`;
- `pump::LinePump<S: AsyncRead + AsyncWrite>`: `new(stream)`, `send(&str)`,
  `recv() -> Result<Option<String>, PumpError>`, `open(CapabilityProfile) ->
  Result<SessionOpened, PumpError>` (request id 0), `drive(&mut L, settled:
  Fn(&L) -> bool)`, `shutdown()`; `enum PumpError { Io, Closed, Session }`.
  Tests: `pump::tests` (2).
- `mere_overmap::route_session::MereRouteSession` implements `LineDriven`
  (a failure is the panel's `Unavailable` status message).

The codec underneath is `chirograph::line` (Mark's ruling after P5).

## 5. Murm's in-process transport kind (landed on a Murm branch)

**Landed:** Murm `38e9c9b8dffe19ff374bd226375e27eea2eb98f6` on branch
`agent/transport-in-process`, cut from `3d9e799fe06d2a58754569b8dc11b2dbc7d506e9`
(the revision every consumer pins, which is on
`codex/c1-webrtc-callback-lifetime-20261014`, not on Murm's `main`; see §11
Q1). `native/crates/transport/src/accepted.rs`: `TransportKind::InProcess`,
`IngressContext::in_process(shared_link: [u8; 16])`;
`src/notochord.rs`: `InProcess => CarrierKind::Other`. Tests: one in each
file; `cargo test -p mere-transport --features notochord --lib` 67 passed.

**Landed in Mere `8b08c98b2`:** Mere pins Murm `38e9c9b8` and names it
through the HTTPS alias `https://www.github.com/merely-made/murm.git`, with a
`[patch."https://github.com/merely-made/murm.git"]` table forwarding every
Murm package it uses (`embedded-graphics`, `embedded-graphics-core`,
`mere-transport`, `mere-webrtc-carrier`, `murm`, `outrider`, `postilion`,
`radio-face`, `radio-hand`, `retinue`, `selvage`, `seneschal`, `tulle`) to
the alias at the same revision. That is how Mere already handles its own
`mere.git` URL, and it is why Djinn still builds against Knot `b19eebf`,
whose plain-URL Murm refs at `3d9e799f` are forwarded. Djinn's
`door::browser_carrier::admit_local_session` now builds
`AcceptedSession::new(server, Alpn::from_bytes(PROJECTION_PROTOCOL), None,
IngressContext::in_process(link.shared_link))` and calls
`graphshell_host::carrier::admit_accepted_session`; the facts are unchanged
(carrier `Other`, no initiator, the derived link).

## 6. T7: the dialable host (landed; not started by the binary)

**Landed, Mere `8b08c98b2`:** `ports/mere-resident/src/resident_projection.rs`:

- `pub const RESIDENT_PROJECTION_SALT: &[u8] = b"mere.djinn/resident-projection/v1";`
- `pub fn dial_policy(network: NetworkId, roots: Vec<TrustedRoot>,
  accepted_profiles: Vec<ProfileRef>, max_sessions: Option<u32>) ->
  LocalNetworkPolicy` (closed; the two service rules of §2);
- `pub fn select_mere_route(principal, chain) -> Result<SelectedRoute,
  String>` (§3);
- `pub async fn bind_dial_transport<P: IdentityProvider + ?Sized>(identity:
  &P) -> Result<transport::P2pandaTransport, String>` (the key from
  `identity.derived_keypair(RESIDENT_PROJECTION_SALT)`, ALPN
  `mere/graphshell/v1` only);
- `pub struct ResidentDialHost` with `new(policy)`, `catalog() ->
  SharedCatalog`, `live_sessions()`, `accept_one(&T, now_ms)`, and
  `serve(self, Arc<T>, now_ms) -> ResidentProjectionError` (accepts until
  the transport fails; logs refusals);
- `ports/mere-resident/src/resident_mere.rs`: `MereRoutes::with_remote_catalog(SharedCatalog)`
  (set before cloning or serving) registers each served mere's two routes in
  the dial host's catalog too, through the shared `register_mere_routes`.

Receipts in `resident_mere::tests` (memory transport, a same-persona device
key holding a grant from the persona root):
`a_device_reads_a_mere_through_the_dialable_host_as_its_granted_application`
(admitted as `turnstone`, `MereRouteSession` over `LinePump` reads the
sessions and the graph) and
`the_dialable_host_serves_only_what_the_grant_and_the_owner_name` (another
application's grant refused at the handshake; a grant naming no application
serves nothing; an explicit grant does not reach ambient; ambient needs the
owner's opt-in; ambient opted in describes `djinn.mere/v1/graph` only).

Mere checks at `8b08c98b2` (`TMPDIR=/tmp`): graphshell-host (all features),
graphshell-client (`pump`), mere-overmap, djinn and graphshell
(`webrtc-session`, `personal-sync`, `distillery-chronicle-fixture`) pass, 840
tests; `cargo check --workspace --all-targets --keep-going` fails only at
`tabard-desktop`'s `native_shared_recovery` example and the `mere-mesh`
examples (both pre-existing); the shared crates and `graphshell-web` check
for wasm32; `scripts/check_port_boundaries.py` passes.

## 7. The one repin round (to do)

Pin order, one source throughout: **Murm (`main` `fb6360c12b`, §11 Q1) → Mere →
Knot → Mere's Knot pin → Turnstone → Woodshed.** Each consumer names Mere
through `https://www.github.com/merely-made/mere.git` and Murm through
`https://www.github.com/merely-made/murm.git`, both at the revisions below.
Before each push: `git fetch`, merge `origin/main` (never rebase or force),
check merged files, run the checks listed.

### 7.1 Murm

Per §11 Q1. If (a): merge `codex/c1-webrtc-callback-lifetime-20261014` and
`agent/transport-in-process` into Murm `main`, push, and use that `main`
commit as `M` below (rerun `cargo test -p mere-transport --features notochord
--lib` and webrtc-carrier's tests in `native/`). If (b): `M = 38e9c9b8`.

### 7.2 Mere

If `M ≠ 38e9c9b8`: replace `38e9c9b8dffe19ff374bd226375e27eea2eb98f6` with
`M` everywhere in the root `Cargo.toml` (workspace dependencies, every patch
table, the Murm patch table of §5). Then `cargo check --workspace
--all-targets --keep-going`, `TMPDIR=/tmp cargo test -p djinn -p
graphshell-host --all-features`, the boundary script. Call the pushed commit
`R`. If `M = 38e9c9b8`, `R` is Mere `main` at the time of the round
(`8b08c98b2` or later).

### 7.3 Knot (merely-made/knot-editor), on Mere `R`

Exact edits (a working copy of them, against Knot `b19eebf` and Mere
`8b08c98b2`, passed the checks below on 2026-10-15 but was not pushed):

1. Every `rev = "<old Mere>"` with `www.github.com/merely-made/mere.git` →
   `R`, in `Cargo.toml`, `apps/desktop/Cargo.toml`,
   `crates/knot-document/Cargo.toml`, `crates/knot-site/Cargo.toml`.
2. Every `git = "https://github.com/merely-made/murm.git", rev =
   "3d9e799fe06d2a58754569b8dc11b2dbc7d506e9"` → `git =
   "https://www.github.com/merely-made/murm.git", rev = "<M>"` (eight rows:
   the `transport` workspace row and the `mere-transport` /
   `mere-webrtc-carrier` rows in each patch table).
3. Drop `graphshell`: delete both `graphshell = { git = …mere.git, rev = … }`
   rows in the root `Cargo.toml` (the workspace row and the patch-table row);
   in `crates/knot-editor/Cargo.toml` `[dev-dependencies]` delete
   `graphshell.workspace = true` and keep `graphshell-host = { workspace =
   true, features = ["native"] }` and `graphshell-client.workspace = true`.
4. Source: `crates/knot-editor/tests/rosette_projection.rs` and
   `tests/revision_bell.rs`: `graphshell::view::` → `graphshell_host::view::`,
   `graphshell::sessions::` → `graphshell_host::sessions::`.
5. `crates/knot-editor/src/mere_overlay.rs` test
   `a_reading_adds_sessions_and_archive_and_keeps_the_catalog_graph`: the
   `MergeModel` literal gains `dropped: Vec::new()` (Lane B added the field).
6. Add a pin note under the existing ones in the root `Cargo.toml`.

Checks (`TMPDIR=/private/tmp`; knot-document and knot-site are their own
workspaces): `cargo check --workspace --all-targets`; `cargo check -p
knot-editor --features mere-overlay --all-targets`; `cargo check -p
knot-desktop --all-features --all-targets`; `cargo test -p knot-editor
--features mere-overlay`; `cargo test --manifest-path
crates/knot-document/Cargo.toml`; `cargo test -p knot-desktop --lib --bins
--test scenario_lane --test font_roles --test typography_fonts`; `cargo test
--manifest-path crates/knot-site/Cargo.toml`. Expected: 623 passed, 4
ignored. `cargo tree --workspace` names no `graphshell` package, one Mere
revision (`R`), and `mere-transport` / `mere-webrtc-carrier` at `M` only
(`murm-wire` at `80d99590` is a separate, pre-existing supplier pin). Call
the pushed commit `K`.

### 7.4 Mere's Knot pin

Root `Cargo.toml` `knot-editor` and `knot-document`, and
`ports/mere-resident/Cargo.toml` `knot-site`: rev → `K`; add a pin note. Once no
consumer in Mere's graph names `graphshell` through git, its rows in Mere's
two `mere.git` patch tables report "patch not used": remove the
`graphshell = { path = "ports/graphshell" }` rows from both tables in the
same commit if, and only if, `cargo check -p djinn` prints that warning.
Checks: `cargo check -p djinn --all-targets --all-features`; `cargo tree -p
djinn --all-features -e normal,dev` names no package from the plain
`mere.git` or plain `murm.git` URLs; `TMPDIR=/tmp cargo test -p djinn`
(expected 204 lib tests at `8b08c98b2`, plus its suites). Call it `R2`.

### 7.5 Turnstone (merely-made/turnstone), on Knot `K` and Mere `R`

1. Every Mere row → `R` (it names Mere through the alias already); Knot
   rows (`knot`, `knot-document`) → `K`.
2. Every Murm row → the alias at `M` (eight rows, including
   `transport` with `features = ["test-identity"]` in dev-dependencies and
   `retinue`).
3. Drop `graphshell`: remove its rows (dependency and patch table); add
   `graphshell-host = { git = …mere.git (alias), rev = R, features =
   ["native"] }` if not already present with `native`; in source,
   `graphshell::view::` → `graphshell_host::view::` and
   `graphshell::sessions::spawn_endpoint_session` →
   `graphshell_host::sessions::spawn_endpoint_session`.
4. If any `match` on `mere_view::MereViewRequest` or `SessionStep` appears
   by then (none at `79f5f87`), add a wildcard arm (both are
   `#[non_exhaustive]` on Mere `main`).
5. Pin note in the root `Cargo.toml`.

Checks: `cargo check --all-targets`; `TMPDIR=/tmp cargo test` (942 passed,
10 ignored at `79f5f87`); `cargo tree` names no `graphshell`, one Mere
revision and one Murm revision for the Murm packages. Call it `T`.

### 7.6 Woodshed (merely-made/woodshed), on Mere `R2`

Three roots, each with its own manifest and patch tables: the root,
`ports/hocket`, `validation/relationship-recipe`. In each:

1. Every Mere row (alias) → `R2`.
2. Every Murm row → the alias at `M` (at Woodshed `4005c5a` these are the
   `mere-transport` and `mere-webrtc-carrier` patch rows naming the plain URL
   at `3d9e799f`).
3. Woodshed names `graphshell-endpoint` (`door`) and `graphshell-client`, not
   `graphshell`; nothing in this round changes their APIs it uses, except
   that `graphshell_client::line` became `chirograph::line` (Mere
   `37fce431a`): there is no hit at Woodshed `4005c5a`; rerun `rg
   'graphshell_client::line'` at the repin and repoint any hit.

Checks: per root, `cargo check --all-targets` and the root's test suite as
Woodshed's own plan names it; `cargo tree` one Mere and one Murm revision.

### 7.7 Done-condition for the round

Murm `M`, Mere `R2`, Knot `K`, Turnstone `T` and Woodshed are pushed; Knot and
Turnstone name no `graphshell`; every graph resolves one Mere and one Murm
source; the lift plan's Progress records the five revisions and the checks.

## 8. Start the dialable host in the Djinn binary (to do, after §11 Q2 and Q3)

File: `ports/mere-resident/src/bin/djinn.rs`, where `MereRoutes::new(…)` is composed
(around the `first_party` list).

1. Build the host before the routes: `let dial = ResidentDialHost::new(
   dial_policy(network, roots, vec![ProfileRef { id: "mere.base".into(),
   revision: 1 }], Some(<max>)))`, with `network` and `roots` per Q2.
2. `MereRoutes::new(…).with_divination_authority().with_remote_catalog(dial.catalog())`
   (before cloning or serving).
3. After `routes.load_access` and the initial `serve` loop, if the host is
   enabled (Q3): `let transport = Arc::new(bind_dial_transport(&identity).await?);`
   log its peer id and ticket, then `tokio::spawn(dial.serve(transport,
   now_ms))` inside the resident's task scope so shutdown joins it.
4. Tests: an integration test in `ports/mere-resident/tests/` that composes the
   resident as the binary does (the existing harness in `tests/common`) with
   the host enabled on a memory transport, dials as a granted device and
   reads `mere/<domain>`; and one with the host disabled that finds no
   listener.

Done when the binary serves the route to a granted device on the dedicated
key and refuses everything §2 refuses, and the key differs from personal
sync's and every place's (assert the peer ids differ in the test).

## 9. Turnstone's remote overmap pane (to do; the L14 receipt)

Coordinate with the overmap lane, which is working in Turnstone's tear-out.
Turnstone gains `mere-overmap` (`view`) and `graphshell-client` (`pump`), and
a pane that, given a resident's peer id and a grant (an explicit grant of §2
naming `turnstone`): dials with
`graphshell_host::network_carrier::dial_projection_session` and a hello from
`graphshell_host::admission::open_session_for` with
`RemoteMereRequest::explicit(domain, "turnstone")`; opens with
`LinePump::open`; builds `MereRouteSession::from_descriptor(descriptor,
profile, "turnstone")`; drives it with `LinePump::drive` on a worker; and
renders `MereRouteSession::model()` as `mere-view`. Turnstone's local
`overmap.rs` keeps the local case. Receipt: a Turnstone test against a
fixture host (a `ResidentProjectionHost::selecting` over a fixture catalog
that serves a recorded mere route, since Turnstone cannot depend on Djinn),
and a native run against a real resident once §8 lands.

## 10. Push notices (later phase, T3)

Ordered; each step keeps every carrier working.

1. **Browser host relays pushes.** `ports/mere-resident/src/door/browser_carrier.rs`:
   add `BrowserHostMessage::Notice { notice: CarrierNotice }` (`type:
   "notice"`). `ports/mere-resident/src/door/browser_host.rs` `serve_native_messages`:
   while waiting for the next browser frame, also wait on the admitted
   session's notices (`BrowserSessionClient::wait_for_notice`) with
   `tokio::select!`, and write each as a `Notice` frame. The extension
   (`ports/graphshell/web/extension/bridge.js`, the `port.onMessage`
   listener) ignores unknown `type`s today, so this is safe before step 2.
2. **The extension consumes them.** `bridge.js`: on `type === "notice"`,
   resume the named session from the notice (re-snapshot it as it does after
   an intent) instead of polling; add a case to `smoke-native-host.mjs`.
3. **The app door pushes.** `crates/graphshell/graphshell-endpoint/src/door/wire.rs`:
   add `AppHostMessage::Notice { notice }`; `ports/mere-resident/src/door/app_broker.rs`:
   forward the session's notices as they arrive (the same `select!`), and
   keep answering `TakeNotice` / `WaitNotice` for one release; the client
   (`door/app_client.rs`) queues pushed notices so `take_notice` answers
   from the queue without a round trip.
4. **Retire the polls.** Once every first-party client reads pushed notices
   (Knot's overlay, Turnstone, Signalman, Graphshell's binaries), remove
   `AppMessage::TakeNotice` and `WaitNotice`, and `ResidentMereProxy`'s
   nested polling (`graphshell_endpoint::door::resident_mere_proxy`) in
   favour of forwarding the pushed notices; keep `SessionCore`'s
   Discover-as-poll only for carriers that cannot push (none of the above).

Tests: each step's carrier test asserts a notice reaches the client without
a request; a C4-chain test asserts one notice crosses Djinn, the door and the
proxy with no poll interval in the path.

## 11. Rulings (Mark, 2026-10-15)

1. **Q1, Murm's line. Ruled: one line, on `main`.** Done the same day:
   Murm `main` is `fb6360c12b`, a merge of both
   `codex/c1-webrtc-callback-lifetime-20261014` (`3d9e799f`) and
   `agent/transport-in-process` (`38e9c9b8`), and both branches are
   deleted. The 58,702-commit gap was upstream history imported through
   about 33 merges on main; the branch was main's `b49df894` plus one
   commit. Main had changed 2 lines in `transport` and `webrtc-carrier`
   since. `mere-transport`'s lib tests pass on the merge (57). The repin
   round (§7) pins Murm `main` at `fb6360c12b`, not a branch.
2. **Q2, trusted roots. Ruled: (a) for both.** Same-persona devices use
   a resident root derived from the persona (salt
   `mere.djinn/device-root/v1`). Pairing (`pandect::wallet_grant`
   enrollment) issues each paired device an explicit §2 grant per
   application the owner allows. Places: add a joined place's root when
   Turnstone's place host admits the owner, read from the place record.
3. **Q3, on by default. Ruled: (a).** The binary starts the host only
   when at least one trusted root besides the persona's exists, so a
   resident with no paired device opens no listener.
4. **Q4. Ruled: (a), keep `8b08c98b2`.** Codex starts from §7.

## Progress

- **2026-10-15:** written after the scope change; §§2–6 describe Mere
  `8b08c98b2` and Murm `38e9c9b8`; §§7–10 are the Codex lanes' work.
- **2026-10-15 (rulings):** Mark ruled Q1–Q4 (§11). Murm is merged to
  `main` `fb6360c12b`; the repin round starts there.
