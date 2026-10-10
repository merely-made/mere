# Moot Object M1 — a moot you can declare, join, and share into

**Date**: 2026-06-12
**Status (2026-10-06)**: Historical M1 landed as recorded below. The active
continuation is [Community collections and author-offline publishing](#community-collections-and-author-offline-publishing-2026-09-04). Its same-machine live-peer process proof passed with stable-Persona binding and current Gemot command authority. P3b (contribution withdrawal, `b99e532a`) and P3d (collection lineage, `6f1f73bf`) landed in Gemot on 2026-09-09. Turnstone has landed P3a's source capture (`b4e69ce`), the P3c collection consumer (`6b77e34`) and collection-scoped search (`aa515bd`) on 2026-09-09 and 2026-09-10; its page capture plan (`turnstone/design_docs/2026-08-28_page_capture_plan.md`) calls P3a in progress. Production publication/hosting records, historical authority proof, the Persona-to-device adapter, and a two-machine receipt remain open.
The original M1 body preserves its dated vocabulary and ownership. Current owners
are Gemot for community authority and recognition, Commons for shared graph
operations, and Stickleback for accepted-operation replication. Historical
references to flora as the artifact catalog mean fauna; FLORA is the separate
federated adaptation lane. Possession of a moot id does not replace current
admission and delegation checks.
**What this is**: the moot tier's missing product object. The reputation
lane is proven (`moothold::tessera`: signed per-moot ops, LogSync, two-peer
convergence) but nothing yet *is* a moot — no declaration, no visible
membership, no flora. M1 makes the smallest honest moot: **declare** it
(name + charter), **join** it (announce yourself), **share** into its flora
(engram references), all converging deterministically on every member.
**Naming correction recorded**: this was provisionally called "mooting M1"
in conversation, but `mooting`'s charter (its own crate docs) is the
*protocol-adapter selection* layer (Matrix / Nostr / IRC / ATproto /
ActivityPub adapters over a unified social-primitive API) — not the moot
object. The object lane lands in **`moothold::moot`**, beside the tessera
lane it composes with; `mooting` keeps its adapter charter untouched.
**Related**: the [mesh M1 plan](../../archive_docs/2026-06-15_completed_plans/2026-06-12_mesh_m1_plan.md) *(archived — M1 done)*
(this is the third lap of the proven wire/state/sync recipe);
the [eidetic browsing derivation plan](../../archive_docs/2026-10-06_completed_plans/2026-06-12_eidetic_browsing_derivation_plan.md)
(the flora is where a shared `SearchIndex` reference would land — the
federation demo seed, and the consume half's eventual trigger);
the communal-compute tiers brief (a moot is ring 2's container).
**Conflict posture**: pure mere lane (moothold + docs); no genet, no
meerkat. Shell adoption is post-reshape, as everywhere today.

---

## Design (the third lap of the proven recipe)

- **Wire** (`moot/wire.rs`): signed `Operation<MootExt>`; `MootExt
  { moot_id: [u8; 32] }` is the signed addressing extension (cross-moot
  replay fails verification). Events, plain words:
  - `Declared { name, charter, at_ms }` — the founding statement.
  - `Joined { name, at_ms }` — a member announcing themselves (their key is
    the operation's author; `name` is just a display label).
  - `Shared { manifest_id: [u8; 32], schema_id, title, at_ms }` — an engram
    **reference** into the flora (the CID + what it claims to be). Blob
    transfer is deliberately out (M2 rides iroh-blobs; eidetic's
    consume-half work picks up from exactly this reference).
- **State** (`moot/roster.rs`): a deterministic, order-independent fold:
  - Competing declarations resolve by **lowest declaring-op hash** (the
    claim-race rule from the mesh board) — every member sees the same
    founding.
  - Membership: first `Joined` per author wins; members are keys, labels
    are decoration.
  - Flora: entries ordered by `(at_ms, op_hash)` — stable everywhere.
- **Sync** (`moot/sync.rs`): `SyncedMootSpace`, the `SyncedMesh` shape
  (which improved on tessera's receive-only session): LogSync catch-up +
  live lane, an `author()` path (sign at next seq/backlink, persist,
  publish), `roster()` folding the store, real `SyncStatus`, settle-watch
  `resync`.
- **Store**: p2panda-store's SQLite backend behind a `MootStore` mirror of
  `MeshStore` (one transactional insert that persists + sync-indexes).
  The tessera lane keeps its proven redb store; convergence of the two is
  the unification step below, not M1 churn.
- **The peer bin** (`examples/moot-peer.rs`): `declare <name> <charter>` /
  `join <name>` / `share <manifest-hex> <schema-id> <title>` / `show`,
  with the mesh-peer transport shape (env-derived identity + space,
  tickets on stdout/stdin, real status). The two-machine run mirrors the
  mesh milestone: declare on one device, join + share from the other,
  both rosters agree.

## The one-endpoint composition finding (recorded, deferred)

p2panda-net 0.6.1 registers LogSync under a **constant**
`LOG_SYNC_PROTOCOL_ID` — two LogSync instances cannot share one endpoint,
and one instance is monomorphic in its extension type. So when meerkat
eventually runs tessera + mesh + moot lanes on one transport, the shape is
**one LogSync, one shared extension type, lanes separated by topic** —
and the lanes are already structurally ready for it (`TesseraExt`,
`MeshExt`, `MootExt` are all `{ 32-byte id }`). The natural end-state for
a moot specifically: **one moot = one topic**, its log carrying tessera
receipts *and* object events as one vocabulary, the folds separating
concerns. That unification (or an upstream patch making the protocol id
configurable) is its own slice with its own plan; M1 builds standalone
exactly as mesh M1 did.

## Tests

1. Wire: round-trip, signature verification, cross-moot replay fails.
2. Roster fold: declaration race identical in both fold orders; duplicate
   joins collapse; flora order stable; foreign-moot ops skipped.
3. Two-peer convergence: A declares + shares, B joins live; both rosters
   agree (declaration, two members, one flora entry); status counters real.
4. The two-*machine* run is Mark's verification, via `moot-peer`.

## Done conditions

- `cargo test -p moothold` green including the new `moot` module's 1-3.
- `moot-peer` round-trips declare/join/share between two in-process peers.
- Workspace untouched beyond moothold (the crate already exists and is a
  member); cross-repo smoke stays green.
- No blob fetching, no tessera coupling, no protocol adapters (that is
  `mooting`'s charter, untouched), no economy.

## Out of scope (named)

M2: flora blob transfer over iroh-blobs + the eidetic consume-half
hand-off; invitation/capability gating (M1 is the trust-ring rule: holding
the moot id is membership eligibility — the kith ring's definition);
moderation/removal events; the one-endpoint unification above; shell
adoption (post-reshape).

## Progress

- **2026-06-12** — Plan written after the survey that redirected it:
  `mooting` is 27 lines because its charter is the adapter layer, not the
  object; tessera is per-moot but receipt-shaped; nothing declares or
  joins a moot today. Recipe and rules lifted from the mesh M1 lap
  (deterministic races, one write path, author+publish, real status).
- **2026-06-12** — **M1 landed: `moothold::moot` with the full suite green
  (moothold 72 tests; the module's 11 across wire/roster/store/sync,
  including both two-peer convergence lanes) and the `moot-peer` rehearsal
  run end to end.** `wire.rs` (signed `Operation<MootExt>`; cross-moot
  replay fails; p2panda validator compatibility), `roster.rs` (the
  order-independent fold: declaration race resolves by lowest op hash in
  both fold orders, duplicate joins collapse to the earliest, flora stable
  by `(at_ms, op_hash)`, foreign-moot ops skipped), `store.rs` (`MootStore`
  over p2panda-store sqlite, one transactional persist+index write path),
  `sync.rs` (`SyncedMootSpace`: catch-up + live lanes, `author()`,
  `roster()`, real `SyncStatus`, settle-watch `resync`). The rehearsal
  (durable sqlite stores so one identity authors across invocations):
  founder declared `printing-circle`; the friend synced the declaration
  (real status: 1 round, 1 op), joined as `alex`, and shared a
  `eidetic.SearchIndexSpec/v1` reference into the flora; the founder's
  final roster converged on all three — declaration, member, flora entry.
  The flora reference is the literal hand-off eidetic's deferred consume
  half picks up from. **Remaining**: Mark's two-machine run (`moot-peer`,
  the mesh-peer recipe: same `MOOT_SPACE`, distinct `MOOT_SEED`s, tickets
  both ways); then M2's named scope.

## Community collections and author-offline publishing (2026-09-04)

**Status:** active. This continuation follows Mark's Eidetic/Fleece, application
co-op, community lineage and voluntary distribution discussion. It replaces the
old M2 implementation assumptions above; it does not reopen completed M1 work.

### Capsule-library composition proof (2026-10-09)

**Status (2026-10-09):** bounded example and opt-in Graphshell browser mount
implemented and exercised; broader product adoption remains open. Mark authorized the
capsule-library proof following the Moot prior-art discussion. This does not
adopt a new publication wire grammar or make the experimental host a Graphshell
product surface.

**Phases and done-conditions:**

1. Independently signed capsule packs and a signed applet pack enter an existing
   Gemot collection through authorized contributions. The selected collection
   exposes author identities and immutable content references; old revisions
   remain retained. Tampered packs, payload substitution, an unauthorized
   contribution and a foreign author's replacement are refused.
2. A member receives a NativeDrop over scoped iroh, retains it in Muniment,
   and reopens it after the publisher process exits. A separate member accepts
   only graph records, without retaining capsule or applet payloads. The proof
   records separate process identities and storage facts.
3. The same `app-core` component drives browsing/search in the native host and
   a browser host using jco's generated Component Model bindings. Each host
   checks a separate local execution grant, refuses self-escalation and
   out-of-scope navigation, and reports missing capabilities. Browser rendering
   and retention remain host responsibilities. This does not prove Graphshell
   mounting, arbitrary applet support, cross-machine delivery or live coediting.

**Findings (2026-10-09):** `crates/moot/gemot/examples/author-offline-publication.rs`
already proves same-machine scoped transfer, author exit and durable reopening.
`crates/eidetic/eidetic-core/src/pack.rs` already signs `WasmComponent` and
`Asset` inventories; signature verification alone grants no execution power.
`crates/script/app-host/src/lib.rs` hosts the existing envelope WIT contract and
accepts exact verified component bytes. The existing production collection
commands preserve original signed contribution identities. This proof composes
those owners and keeps its catalogue/event vocabulary inside the example.

**Progress:** 2026-10-09 — isolated checkout started at current remote Mere
`db24dbe96612a6bd47fcd79625668dbc713fa575`; no production runtime wiring changed.

2026-10-09 — `crates/moot/gemot/examples/capsule-library.rs` composes two
independently signed gemtext capsule packs, a later Alice revision and a signed
WIT applet pack. Existing Gemot grants authorize contribution, collection
curation and a distinct Standing hosting commitment. The collection selects
Alice's new reference and Bob's reference while the old Alice pack, payload
and signed contribution remain retained. A publisher, a retaining member, an
index-only member and the reopened native host use separate processes; the two
authors use independent roots inside the publisher fixture.

The final index-only member received 27,180 bytes of signed community/lane evidence,
retained no capsule or applet payloads, and was refused the 285,851-byte full
bundle. These are fixture sizes, not performance measurements. After publisher
exit, the native host reopened the member's durable Moot and ingress stores,
rechecked current hosting authority and Standing, and recovered its retention
lease. Its applet used signed Servitor grants for a separate local participant
key, independently of pack signatures and community membership.

The same component, BLAKE3
`56aaec6106a0d4a4d82a2ac20b6ae666919b1081d164b256c13d807beac5fe5f`,
produced equal browsing and title/author search results in Wasmtime and Chromium
151.0.7922.34 via jco 1.37.0. Real browser checks covered explicit execution
review, reading without saving capsule files, address-only retention, explicit
revision retention, page reopen and offline IndexedDB reading, missing
capabilities, grant narrowing, self-escalation/outside-address refusals,
malformed/unknown actions, changed component bytes and host recovery after
terminating a runaway worker. Native grant revocation and epoch interruption
also passed. The native receipt matches all four compiled fixture source hashes;
the browser build records its host-source and generated JS/core-Wasm hashes.

**Additional findings (2026-10-09):** a JSON value projection reorders the
signed pack wrapper's fields. Re-serializing that projection cannot recover its
original content address. The browser therefore fetches, hashes and retains
exact original pack bytes before parsing and checking their signatures. The
guest receives copied disclosed metadata and emits proposals through the
existing envelope. Its example projection action does not extend Turnstone's
production classifier.

**Qualification boundary:** the native host is headless; the standalone browser
host consumes the member's disclosed snapshot without replaying Gemot operations.
Browser retention uses real IndexedDB directly, not Muniment's browser adapter.
jco bindings are generated locally from the checked component. The small WASI
adapter supplies only this guest's CLI/logging imports, without ambient files,
network or environment; it is not a general browser WASI host. Native memory
limits and browser worker interruption are distinct containment mechanisms.
The native fixture uses short-lived instances: `AppScript` currently sets its
epoch deadline at attach rather than renewing it for each lifecycle call.
Sustained native sessions need an explicit budget/renewal policy at host adoption.
Public fixtures use encrypted authenticated iroh carriage and unsealed local
stores. Private publication/key epochs, full hosting bounds, historical authority
proofs, two-machine delivery, Turnstone/native product mounting and live coediting
remain open. Pack versions do not establish production publication lineage.

**Graphshell adoption continuation (2026-10-09):** implemented and qualified on
one machine, authorized by
Mark's next "Go ahead". The isolated checkout has been advanced to fetched
`b6b551d44`; the prior proof's changes were preserved. This pass mounts the
verified capsule component in Graphshell's existing retained tree. Its bounded
host dialect is the capsule catalogue and addressed reader, through the existing
`app-core` envelope; it does not freeze a universal applet surface ABI.

1. An opt-in portable host model verifies exact signed pack/component bytes and
   each selected capsule's signed metadata. A host-disclosed collection binds
   the scope; local review grants execution independently. Changed signatures,
   content, authors and out-of-scope or stale proposals must be refused.
2. Graphshell's retained Cambium tree presents the review, catalogue and reader.
   A worker runs the original component; Graphshell lowers its proposals after
   the turn, rechecking the current grant. Replacing or closing a mount invalidates
   its outstanding work. Runaway execution must leave the host responsive.
3. The fixture exercises the actual Graphshell browser bundle and records
   browse/search parity, reading, revocation, replacement, containment and visible
   captures. A baseline tree without the feature remains buildable. Any retention
   adopted here uses Muniment's browser backend. Graphshell mounting is qualified
   only after these checks pass; native product mounting and remote projection
   fallback remain separate.

**Adoption finding (2026-10-09):** fetched `b6b551d44` lacks the locally
qualified renderer pins from Mere `27a39780b`. Both initial captures were black;
headless Chromium additionally lost its GPU device during buffer creation.
This continuation restates the existing immutable Classic/encoding/shader
repair `865cbf419668a6fb5b5e96ea54eaaf9be160fd92` in the root and standalone
browser manifests. It does not change shader source or claim software rendering
as hardware acceptance. Pointer checks use the accessibility mirror's retained
rectangles to click the actual canvas; the mirror deliberately is not the
pointer target. The accepted receipt must record the actual adapter and visible
capture after this adoption.

**Graphshell progress and acceptance (2026-10-09):**

- `ports/graphshell/src/capsule_applet.rs` is the opt-in portable host gate. It
  verifies exact signed applet/asset pack bytes, signed contributor metadata and
  the component inventory; review itself grants nothing. Proposals bind the
  pack, Moot, collection, disclosure revision, local instance and execution
  generation. Lowering and body arrival recheck the current grant. This is a
  bounded reader dialect over the fixture's pack wrapper, not a publication or
  universal surface schema. Limits are 1 MiB of disclosure, 1,024 selected
  capsules, and 16 MiB per component or UTF-8 capsule body.
- `ports/graphshell/src/web_tree/applet.rs` mounts review, search, catalogue and
  reader in Graphshell's existing retained Cambium tree. Exact capsule bytes
  enter Muniment's IndexedDB backend only on the person's explicit retention
  command. Reopening rechecks the exact retained pack and body. Closing or
  replacing a mount invalidates pending turns/responses; a failed replacement
  preserves the current verified mount. This pass supports one active capsule
  mount in the one-tree host.
- The proof's browser worker, verifier and narrow WASI adapter now live once at
  `ports/graphshell/web/applets/`, shared by the earlier standalone browser and
  Graphshell. jco derivation is still performed by trusted local build tooling.
  The supplying host pins the derivation manifest's address before the worker
  imports any generated JavaScript or compiles a core module. The signature on
  the original component alone does not authenticate JavaScript claimed to be
  its translation. A remote peer's self-asserted derivation hash is insufficient;
  deployment must establish that pin from a trusted build. The worker has a
  2.5-second turn deadline; browser memory is not qualified as a per-instance
  hard limit. Native AppScript's earlier deadline-renewal finding remains open.
- The final fixture completed scoped peer delivery, index-only refusal,
  publisher exit and durable native reopening again, then used the same
  `56aaec6106a0d4a4d82a2ac20b6ae666919b1081d164b256c13d807beac5fe5f`
  component in the Graphshell browser mount. Actual pointer/keyboard interaction
  passed native browse/search parity, verified reading without saving, explicit
  Muniment retention, fresh-page reopen and offline capsule reading, typed
  refusals, queued-action and late-body revocation, rejected/stale replacement,
  worker interruption and a freshly reviewed instance after interruption. An
  unpinned browser derivation was refused before executing generated code.
- Headed Google Chrome 152.0.7977.83 requested the default high-performance
  WebGPU adapter: AMD `gcn-5`, no fallback. The machine reports Radeon Pro Vega
  56. Review, reader and reopened captures passed visible-pixel gates and were
  inspected. There were no page or console errors in the accepted run. Earlier
  failed captures remain separate. The bindings receipt names exact host source
  and bundle hashes; this is a debug build, not a payload/performance result.
- Six portable gate tests, the original standalone Chromium proof, the opt-in
  browser build and the baseline viewer check passed. The applet browser cone
  includes dev/test edges and excludes the forbidden native packages; native
  AccessKit fixture dependencies/tests were target-scoped in Graphshell. The
  general port wall still fails on the existing `crates/mere` to `ports/tabard`
  edge. Documentation audits retain the starting commit's six D2 errors and
  add no broken-link or missing-path subjects. Graphshell Clippy completed with
  existing host warnings; no new gate warning was reported.

**Graphshell reproduction (after the earlier peer proof and browser build):**

```sh
# From ports/graphshell/web; its standalone lock is generated on first build.
cargo build --target wasm32-unknown-unknown --no-default-features --features applets
# Return to crates/moot/gemot/examples/capsule-library:
node graphshell-build.mjs /absolute/path/to/proof-root/site /absolute/path/to/graphshell_web.wasm
python3 -m http.server 8774 --bind 127.0.0.1 --directory /absolute/path/to/proof-root/site
# In a second terminal; headed installed Chrome and Python with Pillow are required:
node graphshell-check.mjs http://127.0.0.1:8774/graphshell.html /absolute/path/to/proof-root
```

The accepted local run is `targets/moot-graphshell-run-20261009-c` in the
workspace umbrella, outside this repository. `graphshell.json`, the three
`graphshell-*.png` captures, `site/graphshell-bindings.json` and the preserved
`graphshell-web.Cargo.lock` supplement the peer/native/browser receipts.
`CAPSULE_PYTHON` selects the Pillow interpreter when the default Python lacks it.
Browser Gemot replay, host-page offline boot, live collection subscription,
generic/concurrent applets, native/Turnstone mounting, remote projection
fallback, private publication epochs and live coediting remain separate work.

**Main integration (2026-10-09):** the qualified capsule source is committed at
`56879c7cd`. Mark authorized merging and pushing it, then continuing against the
recent plans. The integration starts from fetched `4d8bd7037`, preserving the
primary checkout's unpublished design commits. Consolidating the overlapping
native test tables and resolving the combined lock are integration changes.
Current Tabard placement repairs the earlier global port-wall failure. The
rebuilt browser passed all thirteen checks on AMD `gcn-5` with no fallback;
captures and matching source/bundle hashes are in the local workspace artifact
`targets/moot-main-integration-20261009`. Its peer/native fixture is reused from
the unchanged capsule proof; the integrated native suite passed 187 tests with
five existing ignored tests. Current
documentation comparison adds no audit subjects; existing D2 gaps remain.

The final integration also includes fetched `e5818742b`: resolved face palettes,
protected shared theme exports and native launcher cleanup. Six capsule-gate
tests and four native reader tests pass against that tree; the ordinary viewer
check and locked applet build pass. Its actual browser run repeats all thirteen
checks and the three inspected Radeon captures. The distinct accepted artifact
is `targets/moot-main-published-20261009`; earlier receipts remain preserved.
The first push raced additive Tabard strict-choice loading at `0669a9192`.
That change is included; all 36 Tabard library tests and the six capsule-gate
tests pass. The captured capsule execution paths and owned host sources are
unchanged by this appearance-store API addition.
Publication completed at `3055ae5af`; the remote main hash was checked after
the successful fast-forward push.

### Independent capsule readings continuation (2026-10-09)

**Status (2026-10-10):** source and CPU checks passed; final narrow scroll
qualification is held during the concurrent GPU/WindowServer investigation.
The continuation starts from published `3055ae5af`. The
[design language §9.2–9.3](../../2026-08-23_projection_scenes_and_graph_native_platform.md#92-selection-foreground-and-ambient-context)
separates attention, activity, presentation and keeping, and distinguishes two
accesses to the same resource. This capsule slice exercises that boundary with
instance-local readings; it neither authors graph Surfaces nor introduces a
general applet/tile ABI. The existing
[app composition owners](../../cambium_docs/research/2026-10-06_app_composition_brief.md#11-current-consumers-and-design-direction-2026-10-09),
[Tabard adoption](../../mere_docs/implementation_strategy/2026-07-05_theme_modes_plan.md#application-adoption-2026-10-09)
and resident graph work keep their current lanes.

1. Give each open reading a distinct local identity while sharing one verified
   body per addressed revision. Selection and catalogue filters must not close
   readings or keep files. Closing one must preserve the others and stored data.
2. Render readings through Cambium's keyed sequence, with local selection,
   explicit close and per-reading Keep controls. Limit this mount to eight
   readings; exceeding the bound must preserve the running host and prior views.
3. Confirm retention feedback only for the requesting generation, reading and
   revision. Exercise closed/stale acknowledgment controls, repeated access,
   deselection, catalogue filtering, distinct revision retention and wide/narrow
   actual Graphshell presentation. The existing grants, signature, replacement,
   interruption and offline-read checks must still pass.

**Findings (2026-10-09):** the instance-local resource cache in
`ports/graphshell/src/capsule_applet/readings.rs` shares one verified body across
distinct, never-reused reading identities. Cambium's keyed sequence presents
them independently. A confirmed Keep updates the observation of the exact
revision; a delayed acknowledgment cannot attach to a closed or replaced
reading. A failed IndexedDB write preserves execution and permits retry. The
ten capsule-gate/resource tests pass; the applet Wasm build and ordinary viewer
check pass. Existing warnings remain.

The headed Chrome run at `targets/moot-capsule-readings-20261009-b` passed all
nineteen checks on AMD `gcn-5` with no fallback or page/console errors. Its five
captures were inspected. The narrow capture prompted an additional actual
scroll-and-select check for controls below the initial viewport; its final
receipt will be recorded here. The failed first run is preserved separately:
its new test wait omitted a page argument; no product source changed to fix it.
The peer/native fixture and signed component are reused unchanged, with explicit
fixture origins beside each rebuilt browser host.

Fetched design clarifications at `421818710` add §9.8's rule that deselection
preserves edits and §9.9's source-attribute/per-view distinction. The bounded
reading model follows those rules without changing the canonical node/link/
field ownership or the reserved Scenograph viewer work.

The additional narrow check exposed a shared browser input fault: DOM wheel
deltas were negated a second time, so a downward gesture at the top could not
advance the host's scroll offset. The correction keeps DOM signs and resolves
pixel/line/page units; all eleven browser-host CPU tests pass. The corrected
applet Wasm build and ordinary viewer Wasm check pass. No renderer, shader or
dependency changes belong to this slice. The last recorded browser diagnostic
launch was 00:12 Eastern on October 10; the corrected source has not been
browser-qualified. The nineteen-check receipt remains earlier evidence, not
acceptance of the corrected source. The twentieth check and publication remain
pending while headed/GPU runs are held. The activity audit is preserved at
`targets/moot-gpu-activity-audit-20261010.json`; failed captures remain beside
their original receipts.

### Moot conversation and coop continuation (2026-10-10)

**Status:** implementation and CPU receipt complete; awaiting publication alongside
the held readings qualification. Browser and
native GPU presentation remain held during the concurrent system incident.
This slice composes `ports/moot`'s existing coop contract with Comms' portable
pane and Commons' actual encrypted chat owner. The retired Meerkat host does
not make a second message store necessary. This consumer earns retaining the
small Comms model; it does not revive that host or move exchange into Cambium.

1. Project the existing authorized Commons chat read into independently
   mountable conversation state. Preserve the space/channel address, stable
   Personae author, original operation, reply/edit facts and causal order.
   Withheld content must remain absent and retained operation counts unchanged.
2. Apply design language §9.8–9.9 to local attention and editing. Deselecting,
   switching channels and refreshing owner state preserve drafts. Delayed
   thread loads cannot attach after switching or reloading. Successful send
   acknowledgment clears only the submitted content; owner refusal keeps it.
3. Join a consumer's existing coop report to presentation without converting
   membership into presence or report data into write authority. Confirm two
   encrypted stores under actual signed Gemot delegation/revocation facts,
   expiry, retained-data survival and independent pane state. The default port
   must still build on Wasm without the optional Commons/native dependency cone.

**Findings (2026-10-10):** `ports/moot/src/conversation.rs` is a derived view.
Its optional `commons-chat` read bridge always calls
`ChatReplica::projection_with_authority`, never the unguarded compatibility
read. The exact channel address supplies a fallback title when the title's
author is withheld, so permitted messages remain visible without leaking a
withdrawn title. Comms names this record grammar `CommonsChat`, independently
of carriage; bilateral Murm remains distinct. Authors do not populate the
membership roster, presence or unread state.

`crates/shell/comms/src/pane.rs` now parks drafts per conversation and binds
thread completions to the latest local request. Explicit identity unbinding
forgets that identity's transient work; a transport outage alone need not.
`prepare_send` creates addressed content only. The actual application must
recheck membership, current Gemot authority and keys at dispatch. Host controls,
real transport sessions, live presence and call/media composition remain the
next receipts; the coop lifecycle contract and durable authorities are unchanged.

**Progress (2026-10-10):** the locked CPU receipt passes nineteen Comms tests,
ten existing coop contract tests and three conversation tests over two encrypted
stores with independently attested Personae writers and signed Gemot authority.
It confirms causal order despite opposing wall clocks, edit/reply/retraction
identity, title fallback after revocation, expiry withholding, unchanged store
counts and independent drafts. The default Moot port builds for Wasm; its normal
dependency cone contains Comms and serde without Commons, Iroh, redb or tokio.
The port/default-Graphshell boundaries pass. Evidence is preserved in
`targets/moot-conversation-tests.log`, `targets/moot-conversation-wasm-check.log`
and `targets/moot-conversation-wasm-cone.txt`. No GPU or browser run belongs to
this receipt.

The integration includes published `7bfb293df`, Graphshell's Tabard appearance
and the shared asynchronous capture helper/Genet pin from `b513994ba`. Its new
Graphshell control inventory confirms that source attributes and recorded edits
still belong to the authoritative session; local attention is a separate concern.
The earlier nineteen browser checks predate this appearance integration as well
as the wheel correction. Fresh CPU checks are recorded separately from them;
final browser qualification and publication remain held.

The integrated CPU rerun passes all ten capsule tests, eleven browser-host
tests and the thirty-two Comms/coop/conversation tests; strict consumer Clippy
and the ordinary viewer check pass. The prepared capsule bundle is
`targets/moot-capsule-readings-20261010-f`, with byte/source verification and
an explicit held status. It has no browser check or capture receipt.

### Capsule peer and standalone-browser reproduction

The following commands execute the earlier capsule-library proof. Start at the
repository root; browser commands remain subject to the current GPU hold.

```sh
cargo build --manifest-path crates/moot/gemot/examples/capsule-library/guest/Cargo.toml --locked --target wasm32-wasip2 --release
cargo run -p gemot --example capsule-library --locked -- run crates/moot/gemot/examples/capsule-library/guest/target/wasm32-wasip2/release/capsule_library_guest.wasm /absolute/path/to/new-proof-root
cd crates/moot/gemot/examples/capsule-library
pnpm install --frozen-lockfile
node build.mjs /absolute/path/to/new-proof-root/site
python3 -m http.server 8770 --bind 127.0.0.1 --directory /absolute/path/to/new-proof-root/site
# In a second terminal in the same example directory:
node browser-check.mjs http://127.0.0.1:8770/ /absolute/path/to/new-proof-root
```

Use `pnpm exec playwright install chromium` if the pinned browser is absent.
Every peer-proof run requires a new directory and refuses to overwrite earlier
work. Local artifacts are `proof.json`, `native.json`, `browser.json`,
`browser.png` and `site/` under that directory, outside tracked source.

**Checks (2026-10-09):** the peer/native run, actual Chromium browser check,
strict example Clippy, Rust/JS formatting or syntax checks, source-hash comparison
and diff check passed. The D2 judgment audit has the same six errors on the clean
starting commit and this tree. The general documentation audit also fails on the
starting commit; comparison adds no broken-link or missing-path subjects. Existing
ledger/index and external-path findings are not repaired by this proof.

### First product proof

An author publishes a small gemtext page into a moot. A different member accepts
a bounded hosting commitment, receives and verifies the bytes, and retains them.
The author process is stopped. The host is restarted from its durable store. A
third reader, with an empty content cache and no access to the author's storage,
retrieves the exact published revision from that host. The reader can inspect
authorship, the moot submission, and the host's distinct commitment.

This is the first execution target. It makes community distribution useful without
waiting for new ranking, reputation economics, FLORA training or a universal graph
editor. The three roles use independent Personae roots and distinct stores.

### Shared model and ownership

The following are semantic requirements, not new wire types. Before adding a type,
map it to the existing owner and record any extension needed there.

| Fact | Meaning and owner |
|---|---|
| Space identity and accepted history | A mere is share-ready; adding participants does not migrate its format. Gemot owns community admission and authority; Commons retains graph edits; Stickleback accepts and replicates canonical operations. |
| Publication and revision | A continuing publication names immutable content revisions and their parents. Eidetic stores typed manifests/payloads; an existing domain authorizes updates. Current host location is separate from publication identity. |
| Contribution | A signed submission to a moot references a revision and carries its own context. Republishing preserves the origin and adds a contribution. It does not replace authorship with the host or moot identity. |
| Hosting commitment | A host names the content, audience, byte limit, retention bound and applicable policy revision it accepts. Inspect Mesh availability/lease contracts for reuse; their existing job scope is not automatically a publication-hosting implementation. |
| Availability observation | Retrieval and integrity checks establish observed availability. A promise, an accepted transfer and demonstrated service are separate facts. |
| Extraction and search | Fleece extracts supplied DOMs; the host owns acquisition and snapshot custody. Eidetic retains the extraction contract; search is a derived projection of admitted, selected content. |
| Application interaction | Woodshed and other apps own domain facts/actions. Graphshell exposes granted views and intents. Live presence and playback coordination have distinct lifetimes from retained material. |

Retain separate content identity, capture identity, publication identity and
contribution identity. Identical payload bytes may be shared by different captures
or contributions. A snapshot's source URL, time and extraction contract are not
deducible from its payload hash. The earlier experiment's HTML-hash snapshot key is
a fixture simplification, not the production identity contract.

Independent personae may build independent histories without publishing links to
other roots. Device identities name offers and observations, with owner/community
authority checked separately. Reputation and rewards are community-issued
assessments over particular acts. Recognition between moots is explicit policy;
ancestry does not transfer membership, obligations or governing power. Fili remains
the named community-descent lane, not a new storage engine for this work.

Regional interest, optional geolocation and measured radio reach can inform a moot.
RF reception alone grants no membership. Personal compute can be offered outward
through successive trust rings under explicit scope, quotas and owner ceilings.
This continuation consumes those contracts; it does not implement the economy.

### Phases and done-conditions

**P1: author-offline publication.** Inventory existing publication addressing,
Gemot contribution records, Eidetic blob resolution and host-serving adapters.
Write down the exact stable publication/update authority and how advertised hosts
are resolved before wiring them. Start with a native peer receipt; then expose the
same accepted bytes through the existing Gemini-facing host adapter. If a serving
adapter is absent, record and implement that bounded adapter in its protocol owner.

Done when the three-role sequence above passes, including host close/reopen;
changed/corrupt bytes are refused; a foreign-moot or unauthorized hosting/update
operation is refused; and an ordinary Gemini client retrieves the public page from
the volunteer host while the author is stopped. A signed revision and raw Gemini
response have separate checks: conventional clients do not inherently verify our
publication signatures. Record process identities, store paths, content hashes,
operation references and actual retrieval source. Distinguish same-machine process
proof from a later two-machine receipt. A self-issued fixture secret on both peers
does not satisfy independent-persona admission.

The 2026-09-05 proof now satisfies the same-machine form of this gate. One
NativeDrop bootstraps constitution, membership, contribution, revision, and content
into a cold host. Attested protocol-derived keys resolve contribution and Standing
events to stable Persona roots. Current constitutional or delegated capability,
write membership, and admission policy authorize the local contribution and hosting
commands; refused commands leave their stores unchanged. The restarted host rebuilds
that authority before serving. The production publication and full hosting record
owners, authority-at-publication frontier, atomic cross-lane import, device adapter,
and two-machine transport receipt remain open.

Production ownership is now ruled. Eidetic owns the signed immutable publication
revision as a typed artifact; the existing Gemot `Shared` event contributes its
manifest without duplicating authorship. Gemot owns the full hosting promise as a
Standing fact. Mesh's compute leases and Stickleback's carrier records remain
unchanged. The production hosting command derives `moot/hosting/<audience>` from
the requested audience itself, intersects community bounds with the device's
local ceilings, and binds the observed authority frontier before signing. The
generic Standing command remains a lower-level primitive and does not choose
hosting authority on the caller's behalf.

Historical authority is a signed causal cut, not a wall-clock claim. Contribution
and hosting records bind the constitution revision plus observed constitution,
membership, and delegation heads. Resolution reports `proven`, `denied`,
`pending missing evidence`, or `legacy unbound`; an incomplete imported prefix is
never treated as denial. Current authority remains the moderation and serving
view, so later revocation can hide or stop serving an historically valid record
without rewriting its publication-time status.

**P2: updates and honest retention.** Publish a second revision under the same
publication identity. Show exact-version retrieval and a policy-selected current
revision; detect competing heads rather than silently treating arrival order as
authority. Expose requested/accepted/available/expired or withdrawn state in the
existing Graphshell surface. Re-evaluate visibility and serving authority when
relevant grants or policies change, including after restart.

Done when an old exact reference remains identifiable, accepted update authority
is checked, an expired promise is not reported as live hosting, an offline host is
not reported as measured availability, and withdrawn content leaves the current
search/serving projection. Withdrawal does not claim to erase copies already held
by recipients. Availability floors, erasure policy and checkpoint pruning remain
separate under the existing deletion/retention plan.

**P3: captured collections.** Move the useful behavior from the isolated Fleece
experiment into the retained consumer: explicit capture, separate contributions,
collection versions/forks, and searchable body text. Retention settings distinguish
records, canonical/reader extraction plus anchors, and replay resources. Preserve
Fleece version and normalization; selector positions refer to canonical DOM text,
not the shorter reader text. Any missing anchor remains explicitly missing.

Fleece 0.5 supplies canonical DOM text, paired quote/position selectors, reader
structure, structured data, metadata links, and semantic tables. Its versioned
preservation record carries the text and reader profile, quote context, and
implementation version; validates anchors on decode; supports pure range-to-anchor
and anchor-resolution operations for human selections; and projects RFC 5147
Fragment, Text Quote, and Text Position selectors over the same immutable
canonical-text resource. Eidetic owns the complete Annotation JSON-LD
envelope and wraps the payload with source URL, capture time, available response
type and DOM-mode facts, plus raw or replay blob identities. Fleece retains no
fetch, storage, replication, or Moot policy. The complete cross-standard ledger
lives in Genet's `genet/design_docs/2026-09-05_fleece_preservation_contract_plan.md`.

The capture host remains the standards boundary. A fuller archival response can
map its record id, request/effective URI, response status and headers, capture
time, payload/block digests and truncation state to
[WARC 1.1](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1-annotated/),
[HTTP semantics](https://www.rfc-editor.org/rfc/rfc9110.html), and
[HTTP digest fields](https://www.rfc-editor.org/rfc/rfc9530.html). Memento fields
apply when datetime negotiation actually occurred; WACZ is a package for carrying
captures and indexes. Fleece consumes the resulting capture reference and scoped
digest as extraction input evidence rather than owning those protocols.

Done when real selected pages survive peer transfer/reopen, a body-only query finds
them, duplicate submissions preserve both contributors while results group content,
and a changed page cannot silently retarget an old annotation. Record unique
content/capture counts, extraction and replay bytes, index build/update cost, heap
use and query latency. Search-engine selection additionally needs held-out relevance
cases and browser validation; the four-fixture receipt does not admit a replacement
for Tantivy. The unchanged consumer seam remains the migration boundary.

**P4: application co-op and lineage.** Reuse the active Woodshed/Graphshell co-op
work, after checking its final implementing receipt. A friend joins a selected Set
or comparison space; independent local views issue granted domain intents. Retain
material, edits and chosen analysis records after both applications close. Record
analysis input revisions, relation selection/depth, musical constraints, method or
model version and accepted result. Personal chronological trails remain separately
selected for retention and sharing.

Done when a second participant reopens the shared work without the original host,
a fork retains ancestry without inheriting ungranted membership/resources, and
Woodshed continues to own musical meaning. A published edition can use P1's hosting
path. Verify actual live coordination separately from retained edit exchange.

**P5: addressed delivery through a mesh.** Add bounded acceptance of addressed
content for later delivery using the existing Stickleback/native-drop path. Keep
receive-for-delivery, retain-for-a-period and publish-to-an-audience explicit.

Done when an intermediary restarts while the recipient is absent, then delivers
the same canonical addressed record once the recipient returns; wrong-recipient,
duplicate and over-budget handling is demonstrated. The intermediary gains no
right to publish or edit by carrying bytes. LoRa airtime/range and Signalman device
management require their own subsequent consumer receipts.

### Findings (2026-09-04)

- `crates/moot/commons/src/lib.rs` exposes `Replica::accept` and
  `projection_with_authority`; use the retained/effective split rather than adding
  an arrival-ordered shared graph.
- `crates/eidetic/eidetic-core/src/manifest.rs` (`BlobManifest`) already separates
  content hash, sources, privacy, provenance and trust. Source advertising and
  community admission still need the P1 consumer proof.
- `crates/mesh/mesh/src/retention.rs` (`AvailabilityPolicy`) distinguishes a hosting
  promise from erasure. `lease.rs` describes author-offline job grants; inspect its
  job-specific scope before claiming publication hosting is implemented.
- The local `Code/experiments/eidetic-moot-20260904/RESULTS.md` receipt exercised
  real Fleece/Personae/Muniment on four synthetic captures and five submissions:
  16 anchors, a changed paragraph, independent journal forks and body-text queries.
  It exercised neither p2panda transport nor service commitments. Its sources and
  reproducible command are local artifacts, not checked-in shipping evidence.
- Concurrent untracked `ports/moot/examples/commons_practice_peer.rs`
  describes a line-JSON retained Woodshed space with one redb store per process and
  explicitly no transport implementation. `ports/graphshell/web/co_op.*` and
  related projection work are another active lane. Their presence is not a landed
  co-op receipt and this planning pass does not modify or absorb them.
  **Corrected 2026-10-06 (S14 pass):** the example is no longer untracked; it was
  committed in `534ae1c6` (2026-09-06) and is present at mere `535bca11`.

### Progress

- **2026-09-04:** scoped the continuation and selected P1 as the next implementation
  target. Reconciled the smolweb publishing brief: a moot can coordinate hosting
  for a single author. All P1-P5 implementation and acceptance gates remain open.
- **2026-09-05:** the separate-process P1 rehearsal passed over the live scoped
  iroh blob path. An admitted host fetched the exact Stickleback NativeDrop; an
  unadmitted peer was refused and retained no bytes. The author then exited; a new
  host process reopened the durable ingress, Moot and Standing stores; and an
  isolated fresh reader retrieved the exact page through an ordinary Gemini TLS
  exchange. Distinct Personae roots signed the publication/contribution and
  hosting facts. Corrupt carrier bytes and a foreign-Moot operation were refused;
  proof-local policy rejected unauthorized publication and hosting candidates;
  an unpublished path returned Gemini `51`. The concurrent proof artifact is
  not yet committed; its target and exact ids are:
  [author-offline community publication proof](../research/2026-09-05_author_offline_publication_proof.md)
  P1 remains partial: candidate publication/hosting records and explicit fixture
  policy must become production authority. Current `Shared` and Standing folds
  also need attested outer-signer binding to stable Personae roots. The
  Persona-to-device-key adapter and a two-machine receipt remain open.
  **Corrected 2026-10-06 (S14 pass):** the proof artifact is committed: the linked
  proof document and its receipt
  `design_docs/moothold_docs/research/receipts/2026-09-05_author_offline_publication.json`
  landed in `b9e9078f` (2026-09-06) and are present at mere `535bca11`.

- **2026-09-06:** the first P3 preservation slice passed and its cross-repository
  adoption landed. Genet Fleece 0.5 at `9e8f9dc2f3ddc0af1658580bb51964462a03923f`
  carries document-level canonical-text identity, arbitrary range mint/resolve
  operations, the RFC 5147/quote/position selector triple, ordered mixed-language
  and direction evidence, lossless embedded JSON-LD blocks, and validated optional
  wire records. Quote resolution now checks every Unicode code-point start, so
  overlapping occurrences remain distinct matches, then applies Fleece's stricter
  grapheme-boundary rule. Its focused gate passed 69 tests, strict Clippy, and
  `wasm32-unknown-unknown`. This is a tested extraction and selector profile, not a
  claim of general HTML, JSON-LD, RDF, Web Annotation, or browser conformance.
  Mere's opt-in `mere-document-lanes/eidetic-bridge` binds the selected immutable
  `text/plain; charset=utf-8` resource to caller-supplied capture evidence and
  canonical-page scope, saves it as an Eidetic typed payload, closes Fjall, and
  reopens the same validated Annotation envelope. An independent offline
  `oxjsonld`/`oxrdf` oracle expands the envelope with the official W3C context and
  verifies the expected dataset up to blank-node identity. All 28 root-workspace
  Mere Genet dependency pins move together to the hardened revision.

- **2026-09-06:** Fleece's preserved JSON-LD blocks now feed linked-data through a
  document-lanes adapter, and linked-data's duplicate HTML string scanner is
  retired. The detailed adapter retains document order, element id, declared media
  type, source text, and either the contribution or its parse or expansion failure;
  a convenience projection keeps the former best-effort successful-results
  behavior. The host can supply its resolved document IRI for JSON-LD expansion
  without giving Fleece source, fetch, or custody authority. W3C JSON-LD ToRDF
  `t0017` proves relative IRI expansion against that caller-owned base; absent and
  invalid bases retain distinct outcomes. Existing callers of linked-data's removed
  `from_html*` helpers must extract with Fleece and call this adapter. Re-ingesting
  an old HTML fixture can also change blank-node skolem IRIs where DOM text
  normalization changes the exact JSON-LD bytes; migrations must treat those as
  derived identities and regroup by durable source facts. `knot-editor`
  `7da29a6bd75112e4d28abb8066490cb9e514d543` widens its direct dependency to
  Fleece 0.5 and carries Cargo's corrected lock resolution. Its 94 locked library
  tests pass. Whole-crate strict Clippy reaches four existing lints in endpoint,
  publishing, wire validation, and vault code outside this manifest-only adoption;
  the exact annotation seam passes strict Clippy. Mere's generated locked graph now
  contains exactly one Fleece, 0.5 at the hardened Genet revision. The combined
  adapter and Eidetic gate passed 47 focused native tests, strict Clippy for both
  changed crates, and a
  `wasm32-unknown-unknown` check using the workspace's established `wasm_js`
  getrandom backend. Full official JSON-LD and Web Annotation test suites remain
  open.

- **2026-09-07:** the next P3 capture proof binds host-observed acquisition facts
  into the durable extraction at Mere `ee533436`. `CaptureEvidenceV1` retains the
  final absolute source, BLAKE3 identity of the exact acquired bytes, capture time,
  available response content type, source/rendered/caller-supplied DOM mode, and optional
  raw/replay manifest identities. The raw manifest, when present, must name the
  same bytes as the capture hash. A hash of the complete evidence record is also
  bound into the Web Annotation target; mutation tests cover every retained field,
  while records serialized before the extension remain readable with those facts
  explicitly absent.

  A native receipt sends a `TrustedPeersOnly` annotation manifest and exact HTML
  capture over authenticated p2panda QUIC, validates peer/protocol/privacy/schema
  and both content hashes before storage, then closes and reopens a receiver-only
  Fjall store. The reopened annotation and raw page bytes are exact. This proves
  transport of the document evidence and durable receiver reopen. Commons' existing
  LogSync test separately proves operation convergence; this receipt does not prove
  that manifest operation sync, authenticated carrier identity grants Moot
  membership, or sharing consent. Genet's current `ResourceResponse` exposes final
  URL, content type and body bytes, but not requested URL, status, lossless headers
  or truncation state. WARC-grade full-response capture therefore remains a host
  contract follow-on. Body query, contributor-preserving deduplication, corpus and
  cost measurements, and the held-out search-engine decision also remain open.
  The same gate exposed UUID 1.25's new wasm RNG-selection requirement; Inker
  `aac6e5df` now selects its JavaScript-backed provider only on wasm targets.

- **2026-09-08:** the P3 search projection now has a production-shaped seam.
  `eidetic-search::DocumentIndex<K>` is a transient in-tree BM25 over
  caller-owned documents with opaque keys, primary and alias addresses, title
  and body fields. It does not accept `BrowsingTrace`, own a store or write an
  index directory. The existing `TrailIndex` adapter and API are unchanged.

  Moot's new opt-in `captured-web` surface builds from
  `MootRoster::authorized_fauna`, then validates caller-resolved
  `FleeceAnnotationRecord`s. It groups only the exact canonical-text hash under
  the same Fleece extraction schema, normalization and reader profile. URL,
  raw-capture identity, manifest identity and near similarity do not merge
  records. Each group retains every signed `FaunaEntry` and `CaptureIdentity`;
  missing, unsupported and invalid manifests remain explicit rejections.

  The focused fixture has seven fauna entries: three authorized valid shares,
  two over identical text from distinct contributors and source URLs, plus one
  changed revision; one unauthorized decoy; and three authorized rejected
  references (missing, unsupported schema and integrity-invalid). The projection
  yields two searchable documents, retains all three valid contribution records,
  returns each body-only term once, returns both revisions for their shared exact
  source URL, and excludes the unauthorized decoy. `mere-eidetic-search` has 37
  passing library tests with one timing test ignored; `mere-moot --features
  captured-web` passes its focused test and doc tests. Both changed production
  crates pass strict no-deps Clippy, the search crate passes
  `wasm32-unknown-unknown`, and the port remains dependency-free with the feature
  off.

  This composes with the 2026-09-07 authenticated transfer and receiver-reopen
  receipt, but is not yet one headed Turnstone receipt. P3 still needs the
  Turnstone place worker to resolve locally replicated fauna manifests into this
  projection, explicit capture consent and withdrawal/remint behavior, collection
  version/fork records, shared-corpus cost measurements and held-out relevance.

- **2026-09-09:** P3 was split into four ordered implementation lanes after a
  Turnstone, consent/withdrawal and collection-lineage review. The captured-web
  consumer is not the next patch. It has no production Fleece record resolver to
  consume yet, and Turnstone's current hosted capture work produces a viewport
  PNG rather than canonical DOM text.

  **P3a: explicit source-document capture.** Turnstone owns this application
  action. Its fetched-page path already receives exact response bytes, decoded
  HTML and content type, parses a `StaticDocument`, and calls Fleece for reader
  text. Retain the exact bytes, final URL, content type and acquisition time in
  the node-and-URL-scoped capture candidate long enough for an explicit action to
  deposit the representation, run `fleece::extract_document`, and save a
  `FleeceAnnotationRecord` as `LocalOnly`. A whole-document capture uses an
  explicit whole-text anchor; later human annotations use their selected ranges.
  The action is separate from sharing. It may share Keep, envelope, tombstone and
  garbage-collection mechanics with visual capture, but a PNG hash and a source
  document hash remain distinct artifacts.

  Hosted Weld is unsupported in this lane: it fetches internally and currently
  returns only correlated pixels. A later engine seam must expose a correlated
  source or named serialized-DOM snapshot with final URL, content type,
  acquisition time and privacy semantics. Accessibility text, OCR and pixels do
  not substitute for Fleece's canonical DOM text. Turnstone must also replace its
  unconditional page-text retention hook with an actual local setting or stop
  retaining those bodies. Incidental trail text is never capture evidence and is
  never eligible for Moot sharing.

  **P3b: contribution withdrawal.** Gemot's records lane should add one signed
  `Withdrawn { target_share, at_ms }` event, targeting the original `Shared`
  operation hash. Only that share's stable author may withdraw it. The roster
  retains the positive and withdrawal facts while `authorized_fauna` excludes an
  effectively withdrawn contribution. Delegation revocation remains the broader
  operation that removes all of a sharer's current contributions. Withdrawal
  changes current search and serving after remint; it does not erase Fleece
  records, local blobs, audit history or copies already held elsewhere. This is a
  records-wire change and therefore requires upgraded peers before use.

  **P3c: Turnstone collection consumption.** Once P3a and P3b exist, update all
  of Turnstone's Mere pins together and compose `mere-moot` with `captured-web` in
  the existing place worker. The worker resolves locally held annotation records
  by manifest id, supplies its current `GemotAuthorityView`, and emits an
  app-owned collection summary plus a locally reminted `DocumentIndex`. The
  render-free two-peer fixture has two authorized identical captures at distinct
  URLs and one missing manifest; it proves one grouped result with both signed
  contributions, body and alias recall, explicit rejection, then withdrawal or
  capability revocation removing the result after resync and cold reopen while
  retained facts remain inspectable. Index bytes and scores never replicate.

  **P3d: collection lineage.** Extend Gemot's existing signed records lane rather
  than creating another journal or transport. A collection has a caller-minted
  stable id; changes name their exact observed collection-event heads; a derived
  version is the collection reference, sorted causal frontier and canonical
  membership commitment. Membership references the signed `Shared` operation,
  preserving its contributor rather than naming only content. A fork root names
  the parent version and copies the selected contribution references so it can be
  read after source-history pruning. The first receipt is same-Moot only. A fork
  carries neither community membership, delegations, group keys, Standing facts,
  leases nor hosting promises.

  P3a and P3b are independent first gates and may land in parallel. P3c waits for
  both; P3d follows the working consumer. Cross-Moot lineage verification,
  collection merge UI, private collection encryption, hosted Weld DOM capture,
  WARC-grade custody, near-duplicate grouping, distributed indexes, corpus-cost
  measurement and relevance admission remain later gates.

  **Corrected 2026-10-06 (S14 pass):** Turnstone has since landed P3a's source
  capture (`b4e69ce`, "capture explicit page sources with Fleece", 2026-09-09), the
  P3c collection consumer (`6b77e34`, "project local Fleece captures into places",
  2026-09-09) and collection-scoped search (`aa515bd`, 2026-09-10). They are recorded
  in Turnstone's page capture plan (`turnstone/design_docs/2026-08-28_page_capture_plan.md`,
  which calls P3a in progress), not in this plan, so this plan records P3d (below)
  without a P3c consumer; Turnstone's consumer landed the same day as P3d.

  **Open, raised by the S14 pass (2026-10-06):** where are P3a and P3c tracked from
  here? Options: record Turnstone's P3a and P3c progress in this plan; point to
  Turnstone's page capture plan as the authority for those lanes.

  **2026-09-09: P3b landed in the Gemot records lane.** `MootEvent::Withdrawn`
  is a signed, upgraded-peer wire event carrying the original `Shared`
  operation hash and withdrawal time. The roster retains every distinct signed
  withdrawal fact (duplicate copies of one operation collapse by operation
  hash), resolves the stable withdrawing root, and filters only the targeted
  share when that root matches the share's stable contributor. Unknown targets
  and mismatched identities are refused by `Moot::withdraw_share_for_identity`
  before authoring; there is no seed-only withdrawal command. Checkpoint
  snapshots carry the withdrawal facts, so pruning and late replay cannot
  resurrect the contribution. The captured-web projection inherits the
  exclusion through `authorized_fauna` and remints without withdrawn records.
  Focused Gemot withdrawal tests pass 5/5 and the captured-web feature tests
  pass 2/2. This remains a local implementation receipt; peer interoperability
  requires the upgraded records wire to be deployed on both sides.

  **2026-09-09: P3d collection lineage implemented locally.** The records wire
  now carries caller-minted same-Moot collection ids, contribution references
  to original signed `Shared` operations, fork roots, and per-contribution
  membership decisions over exact observed heads. Collection versions commit
  to the sorted causal frontier and curated membership, independent of current
  fauna authority. Read-time projection separately reports curated and
  effective selections, so withdrawal or capability revocation cannot rewrite
  historical version identity. Causally later edits dominate ancestors;
  concurrent edits to distinct contributions commute and concurrent decisions
  about one contribution use the operation hash as stable tiebreak.

  Fork roots reproduce the named parent membership commitment and materialize
  those contribution references. Foreign Moot references, false fork
  commitments, missing parents, and currently ineffective contributions remain
  explicit diagnostics while raw signed facts remain retained. Retention
  checkpoints carry the collection facts inside their roster snapshot: a
  focused prune test removes the source operations, resolves the same version,
  authors a fork, and accepts a later child change against the retained head.
  The service exposes stable-identity authoring only, checks current
  collection-scoped authority and membership, requires exact current heads,
  and refuses additions outside current effective fauna.

  The complete Gemot library gate passes 145/145 tests, including the four
  collection-fold cases, signed wire and stable-author round trip, and
  checkpoint/prune/fork/child case; strict no-deps Clippy and diff-check also
  pass. Cross-Moot
  lineage, collection merge UI, private collection encryption, and a headed
  Turnstone collection editor remain later gates. A collection fork copies no
  Moot membership, capability grants, delegations, key epochs, Standing facts,
  leases, hosting promises, or capture payloads; its wire type contains only
  the parent version and contribution references.

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere
  535bca11, from the D2 record in support/doc-audit/d2/batch_45_s14_phase_b7.md. The
  status records P3b and P3d as landed and Turnstone's P3a and P3c work; the two stale
  planned-target markers are removed with annotations; where P3a and P3c are tracked is
  left open.
