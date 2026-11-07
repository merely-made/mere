# Domain contract: what a domain supplies and what a client may do

**Status:** 2026-11-06: complete as a contract; nothing implemented. Mark
ruled one kind of store (O8) and C1 to C8 on 2026-11-06, and Cleromancy is
checked against it. Written for M0 of
[the domain adoption plan](../implementation_strategy/2026-11-05_domain_adoption_plan.md),
which made every application's authored store a reservoir mere (D8).
The last section, "Registration contract" (2026-11-06), is P1 of
[the cycle breaks plan](../implementation_strategy/2026-11-06_cycle_breaks_plan.md):
a design for review by Mark before P2 and P3, not a ruling.

**For:** one written contract that every domain and every client builds
against, so that Woodshed, Redshank, Hocket, Knot and the rest move into the
reservoir the same way. Mark asked on 2026-11-05 whether this contract was
prepared; it was not.

**Not here:** each application's own move (its plan); device pairing
(`../implementation_strategy/2026-10-28_device_pairing_plan.md`); the grant
format (adoption plan O7, Meadowcap model); the optional domain vault, an
opt-in feature built later.

## One kind of store (Mark, 2026-11-06)

Mark, on O8: "I'm ok with a second kind, but then are there gonna be other
kinds? Isn't a graph a log…?" It is. Every store in question is an
operation log folded into state; they differ in how the log is ordered, not
in being graphs or logs. Measured on 2026-11-06:

| | Reservoir graph mere | Knot's operation store | Graphshell's personal replica |
|---|---|---|---|
| Entry | `AttributedDelta` (graph-kernel `graph/journal.rs`) | signed `Operation<KnotSyncExt>` (knot `sync.rs`) | signed operation with `PersonalGraphExt` |
| Order | one linear `Seq` per session | causal DAG over `parents` | causal DAG over `parents` |
| Across devices | none | Stickleback log sync | Stickleback log sync |
| Fold | replay into `Graph` | `KnotDocumentProjection` | deterministic fold into `Graph` |
| Check | `GraphValidator`, `GraphAdmissionValidator` before append | `KnotSyncPolicy::admit` before insert | Stickleback operation policy |
| Sealing | none on the live path | each event body, and checkpoints | in the header extension |
| Backend | muniment `DirectoryBackend` | muniment `RedbBackend` | muniment `RedbBackend` |

The reservoir's linear journal is the odd one out. It cannot replicate,
because a per-session counter has no meaning on a second device, and it is
not sealed. Knot's store and Graphshell's personal replica already have what
D2 and D6 require: a signed causal log on Stickleback, sealed per operation,
checked before insert, and synced between devices. Graphshell's replica also
shows a graph kept as the fold of such a log.

**Ruled** ("Yes, one kind"): a mere is one Stickleback causal operation log. A domain
supplies its event grammar, its fold and its admission policy; the reservoir
supplies everything else. A graph is the fold most domains use, not a
different kind of store. Knot's store fits as it is, which answers D4
without a second kind and without projecting documents into a graph. There
would be no other kinds: new domains add grammars and folds, not stores.

This also makes M3 (sealing) and M4 (replication) properties of the one
store rather than separate work on the linear journal.

## What a domain supplies

1. **A domain id** (`DomainId`), named for the data, not the application
   (adoption plan D5).
2. **An event grammar:** typed, versioned operation bodies, closed to
   unknown variants unless the grammar says how to tolerate them. Knot's
   `KnotSyncEvent` is the model.
3. **A deterministic fold** from the causally ordered log to the domain's
   state. The fold never orders by wall-clock time or insertion order;
   concurrent writes to the same item surface as a conflict with each
   writer's version, as Knot's `KnotDocumentConflict` does, until a resolving
   operation supersedes them.
4. **An admission policy** over the folded candidate an operation would
   produce, run where the operation is authored and again where it is
   folded in (C3, C4). A graph domain's existing
   `GraphValidator` and `GraphAdmissionValidator` become this policy
   unchanged; Cleromancy's `validate_divination_graph` keeps working.
5. **A pure crate** holding 2 to 4, with no I/O, implementing Stickleback's
   domain interface (C5), that the resident composes
   in both the daemon and the embedded resident (adoption plan M1, M2).
6. **Its move/stay split:** what is authored and replicates, and what is a
   device fact kept in the device-local store (D7).
7. **Blob references** for large media, content-addressed, fetched on demand
   (M5). Operations carry the reference, never the bytes.

## What the reservoir supplies

- **Lifetimes and the single owner** (D1): opening, durability and closing
  of every mere; the reservoir index lock refuses a second owner.
- **Sealing** under the persona's key, always (D6), with the optional domain
  vault layered on top when it exists.
- **Writers and ordering:** each device is its own writer, derived from the
  persona and the device, as Knot does now; parents record the frontier each
  operation saw.
- **Replication** to the persona's paired devices (D2, M4).
- **The route** `mere/<domain>` with read, edit and notice operations, the
  same through a Djinn door or an embedded resident (reservoir plan V5).
- **A device-local store** per application for what stays (D7), outside any
  mere and never replicated.

## What a client may do

Cleromancy's path is the precedent
(`cleromancy/src/ui/worker.rs`, `ports/mere-resident/src/embedded_reservoir.rs`):

- **Open:** try the Djinn door's `reservoir` route, `ensure` the domain,
  then open `mere/<domain>`. Embed the resident only when no door answers
  (not found or connection refused). A door that refuses, is locked or times
  out never leads to embedding, so no second store opens.
- **Read** the folded state through the route's projection.
- **Edit** by invoking intents that carry the revision the client observed;
  the reservoir runs the admission policy and refuses stale or invalid edits.
  A client never writes the store directly.
- **Observe** through the route's notices (the revision bell), not by
  polling files.
- **When locked or unavailable:** show the domain as unavailable and keep
  device-local state working. Never fall back to a private copy of
  authored data.

## Cleromancy against the contract

Checked on 2026-11-06 against Cleromancy `813a0de` and Mere `c8f3131d`.
Mark chose a fresh start for its meres (C1): the new meres begin empty and
the old stores stay on disk untouched.

**Conforms as it is:**
- **Route only.** Production code reaches its meres only through
  `ReservoirAttachment::open_application` (`src/ui/worker.rs`) and
  `PersonalFactsClient`. `DirectoryBackend`, `ReservoirStore`,
  `GraphSession` and `MereSessions` appear only in
  `tests/granted_ambient.rs`. The `legacy-receipts` feature still writes a
  muniment slot (`cleromancy/mere-host/v1`); it is off in the resident path
  and stays off.
- **Grammar.** Edits are `CapturedDelta`s recorded on a candidate graph
  (`CleromancyHost::record_edits`, `src/host/operations.rs`) and sent as
  `ApplyEditsV1::checked`: add node, set title, insert tag, set facet, touch
  last visited, assert relation. These become operations in the graph
  grammar (C5) with no product change.
- **Policy.** `validate_divination_graph` (`src/host/domain.rs`) and
  `personal_facts::validate` take only the graph, do no I/O, and read no
  sequence number or clock. Both carry over as admission policies under C3.
- **Stale edits.** A refused revision surfaces as "The sessions changed;
  read them again." with no retry (`mere_overmap::route::declined`). Under
  C2 edits from other devices no longer conflict this way; the refusal
  remains for two clients on one device, and the behaviour is right.
- **Unavailable.** `WorkerUpdate::Unavailable` clears the graph, sessions
  and selection and shows the message (`src/ui/native/worker_update.rs`).
  Nothing keeps a private copy of authored data.
- **Device-local state.** Sync consent (`sync-settings.json`) is already
  local and says so; it belongs in the device-local store.
- **No blobs.** Cleromancy stores JSON facets only, so M5 does not block it.

**Changes, all on Mere's side unless marked:**
- **Session intents become operations (C2).** Cleromancy mints, attaches,
  forks, branches, merges, trashes and restores sessions through
  `mere_overmap::route`. Each except attach becomes an operation in the log;
  attach stays a per-connection choice of line. The route keeps its wire
  vocabulary, so Cleromancy does not change.
- **Codicils.** Cleromancy saves, opens and composes codicils (frozen,
  content-addressed graphs) through `archive_intent`. See C8.
- **View state.** Cleromancy keeps its reading workspace's selection, focus
  and layout in the mere through `SET_VIEW_INTENT` (view key
  `reading-workspace`). See C7.
- **The personal-facts admission check** asks the resident whether a
  profile exists (`validate_new_profile_subjects` with a lookup,
  `resident_mere.rs`). A receiving device may answer differently. See C6.
- **Appearance (Cleromancy's side).** `appearance.json` sits in
  Cleromancy's data root. Under D7 the theme choice follows the person, so
  it moves to the person's settings; the theme library `themes.json` has the
  same open question as Woodshed's.

## Rulings

- **C1. Existing meres: a fresh start (Mark, 2026-11-06).** Cleromancy's
  `divination` and `personal-facts` meres are not converted. The causal-log
  meres begin empty; the linear-journal stores stay on disk untouched and
  are no longer opened. Retiring them is Mark's call, later. Same choice as
  Woodshed's W5.
- **C2. Sessions are lines within the mere's one log (Mark, 2026-11-06).**
  Every operation names its session. Minting, forking and branching are
  operations that start a session from a frontier of the log; merging in is
  an operation that records the plan and each conflict choice, under the
  existing merge-in rules (`pandect/src/graph_session_merge.rs`,
  `merge_rules.rs`). Trash and restore are operations too. Concurrent edits
  from two devices within one session are concurrent heads that the fold
  joins or surfaces as a conflict; a deliberate branch is a session. The
  mere syncs and seals as one log.
- **C3. The full check runs where an edit is made, and again where it is
  received (Mark, 2026-11-06).** The authoring device runs the domain's full
  policy on the folded candidate before it signs the operation. Every
  receiving device runs it again when it folds the operation in, from a
  checkpoint of the fold rather than from the start of the log. An operation
  that fails on a receiver is kept as rejected and shown, never applied and
  never silently dropped, as Knot's `Tolerant` mode does with undecodable
  relations.
- **C4. Operations whose parents have not arrived wait (Mark, 2026-11-06).**
  They are stored as pending (Stickleback's `PendingCausalOperation`) and
  folded in, and checked under C3, once their parents arrive. Only the
  structural checks run before storing: signature, admitted writer, causal
  metadata bounds and a decodable body. An operation that fails those is
  refused.
- **C5. Code splits by layer (Mark, 2026-11-06).** The domain contract
  (grammar, fold, policy) is a Stickleback interface in Eidetic, so Knot and
  other non-graph domains depend on Stickleback, not on Mere's graph crates.
  The graph grammar and fold move from Graphshell's `personal_sync.rs` into
  pandect, where Mere's graph authority stays (resident split plan). The
  reservoir and the resident compose domains as they do now. M1's registry
  holds Stickleback domain implementations.
- **C6. Checks every device repeats read only the log (Mark, 2026-11-06).**
  A policy run on receivers (C3) is deterministic over the mere's log, so
  every device reaches the same verdict. A check that needs authority from
  outside the log, such as whether a profile exists in the persona's vault,
  runs only where the operation is authored, and the writer's signature
  vouches for it. Personal-facts' `validate_new_profile_subjects` lookup
  becomes an author-side check; `personal_facts::validate` stays the
  receiver policy.
- **C7. Arrangements follow; attention stays (Mark, 2026-11-06).** View
  state a person deliberately sets up (layouts, saved arrangements)
  replicates as operations. Selection, focus and camera position stay in
  the device-local store, keeping the rule that the attention axis is never
  synced. Cleromancy's `reading-workspace` view splits along that line.
- **C8. A codicil is a snapshot blob and an operation (Mark, 2026-11-06).**
  Saving a codicil writes the frozen graph as a content-addressed blob (M5)
  and records an operation in the log that references it, so it replicates
  like any other blob. Opening one starts a new session line from it (C2);
  composing is a merge.

## Done when

This document names what a domain supplies and what a client may do,
and Cleromancy is checked against it with any changes recorded. O8 and C1
to C8 are ruled, and Cleromancy is checked. M0 is done.

## Registration contract: how a domain reaches the resident (cycle breaks P1, 2026-11-06)

**Status:** design for Mark's review; nothing implemented, no manifest touched. Written
for P1 of [the cycle breaks plan](../implementation_strategy/2026-11-06_cycle_breaks_plan.md)
under its D1 ("Resident stops depending"). Read against Mere `1a26ec4e6`, Eidetic
`617f46a`, Knot and Cleromancy `main` of 2026-11-06. Line numbers are at those revisions
and are citations, not contracts. Illustrative code below is illustrative, not compile-ready.

**Not here:** the code moves (P2, P3), the Stickleback domain trait itself (C5, Eidetic's),
replication (M4), the wire protocol (the Graphshell layer).

### 1. What this builds on, and what does not exist yet

- **The domain interface (C5) is a ruling, not code.** Eidetic's `stickleback` has the
  admission half: `OperationPolicy<E>` (`eidetic/crates/stickleback/src/processor.rs:124`,
  "Domain-owned admission policy") and the causal helpers in `causal.rs` ("Domains still own
  their event grammar and fold"). No trait bundles grammar, fold and policy; Knot's
  `KnotSyncPolicy` (`knot-editor/crates/knot-editor/src/sync.rs:312`) is the one real
  implementation. So this design cannot wait on C5. It gives the registration a slot for
  it (`Authority::Log`, below) and registers what exists today: a graph authority (pandect
  validators) and a lane (Knot's resident composition).
- **M1's registry is not built, but half of it exists.** The adoption plan's M1
  ([domain adoption plan](../implementation_strategy/2026-11-05_domain_adoption_plan.md),
  "M1. A domain registry for both compositions") says both compositions are "hard-coded to
  install Cleromancy's authority", and that each domain names "its id, its validator and its
  owning application". Still true: `bin/djinn.rs:1408` and `embedded_reservoir.rs:250` both
  call `with_divination_authority()`. The existing seam: `MereRoutes::with_domain_validator`
  (`resident_mere.rs:335`, a `BTreeMap<String, GraphValidator>` in `Routes`), the embedded
  hook `ReservoirAttachment::open_with(.., configure)` (`embedded_reservoir.rs:260`, "the
  owner's domain registration hook"), and `ResidentEndpointCatalog::register_erased`. M1 is
  mostly generalising these. M2 (validators required) is the same code: `serve` refuses a
  domain without a validator only for the literal list `"divination" | "personal-facts"`
  (`resident_mere.rs:374`).
- **The presenting half is already registered at runtime, out of process.** README "App-owned
  projections" (`ports/mere-resident/README.md`): an application connects to the app door,
  offers a projection, and the owner's `projection_providers` setting permits it. Knot's
  `knot_endpoint projection-provider` does this with no Knot-specific registration in Djinn
  ("Djinn neither opens it nor creates a Knot-specific registration"). Per the
  [host/guest brief](../research/2026-11-06_host_guest_contract_brief.md) (c), that half
  rides the contract's `projection-provider`; the rest is in-process Rust. This design is
  about the rest.

### 2. The contract

**Registration is a value an app crate builds and the process that owns the store hands to
the resident at composition, before any route is served.** The resident already enforces
that order (`Arc::get_mut(..).expect("set validator before cloning or serving routes")`,
`resident_mere.rs:340`); the contract makes it a typed error. Registration is not hot-plug.
It confers no admission: the owner's grants and the app's declaration still decide who may
open `mere/<domain>` (adoption plan D24, `applications::grant_for`).

```rust
// Illustrative. Lives in Mere (mere-resident, or a small crate it re-exports).
pub struct DomainRegistration {
    pub domain: DomainId,            // named for the data: "divination", "notes"
    pub owner: AppId,                // owning application label (M1)
    pub version: &'static str,       // grammar version, shown in status
    pub authority: Authority,        // required: M2 refuses a registration without one
    pub lane: Option<Box<dyn LaneFactory>>,
    pub custody: CustodyNeeds,       // epoch derivations it asks for, from its own AppDeclaration
}

pub enum Authority {
    /// Today: pure, no I/O, no clock, deterministic over the graph (item 4 above).
    Graph { complete: pandect::GraphValidator,
            admission: Option<pandect::GraphAdmissionValidator> },
    /// When C5's Stickleback domain interface exists: grammar, fold, admission.
    Log(Arc<dyn StickDomain>),       // StickDomain: Eidetic's, not yet written
    /// A lane with no reservoir mere (the Knot document vault until `notes` exists).
    None,
}

pub trait LaneFactory: Send + Sync {
    fn label(&self) -> &'static str; // "knot": names the route and the lifecycle events
    fn open(&self, cx: LaneContext) -> BoxFuture<'_, Result<Box<dyn Lane>, String>>;
}
pub trait Lane: Send {
    fn route(&self) -> LaneRoute;    // route name, display label, notice cadence
    fn session(&self, cx: &AdmittedEndpointContext) -> Result<Box<dyn ResidentEndpoint>, String>;
    fn refresh(&mut self) -> BoxFuture<'_, Result<bool, String>>;  // pairing, readers, dial hints
    fn close(self: Box<Self>) -> BoxFuture<'static, Result<(), String>>;
    fn facts(&self) -> Vec<(&'static str, String)>;                // loggable, never keys
}
pub struct LaneContext {
    pub persona: PersonaId, pub data_root: PathBuf,
    pub settings: serde_json::Value, // owner's `lanes.<label>` entry, opaque to Mere
    pub epochs: EpochKeys,           // custody release, limited to `custody`
    pub blobs: BlobLease,            // the resident's one blob store, lent per scope
}
pub struct DomainRegistry;           // register(..) -> Result<Self, RegistryError>; mere_defaults()
```

**What a domain hands the resident:**
1. Identity: `domain`, `owner`, `version`.
2. Authority, the part Mere cannot supply: the complete-candidate validator (and the
   optional before/after admission check) for a graph domain. The resident calls it for
   "edits, archive save/open/compose and loaded sessions" (doc of `with_domain_validator`,
   `resident_mere.rs:333`), and on receivers under C3 and C6, so it must read only the graph.
3. A lane factory, only for a domain with a live component beside the store: a route, a sync
   host, a vault. Cleromancy has none; Knot has one today.
4. Its custody needs, taken from the app's own `AppDeclaration`, so the resident "asks for no
   more than Knot states it needs" (`resident.rs:92-97`) without Mere naming `knot_editor`.
5. A settings parser inside the lane (the lane receives opaque JSON and keeps its own
   `deny_unknown_fields`, as `KnotResidentSettings` does at `settings.rs:538-545`).

**What the resident hands back:**
1. A served route `mere/<domain>` under that authority, same in the daemon and the embedded
   resident, and a refusal for any product domain without a registration (M2, generalised
   from `resident_mere.rs:374`). The refusal is typed (`domain-not-registered`, with the
   registered ids) so a client shows "unavailable" and never embeds a second store (the
   client rule above: a door that refuses never leads to embedding).
2. For a lane, `LaneContext`, and a **gate owned by Mere**. Today `KnotGate` and
   `GatedSession` (`resident_knot.rs:61`, `:122`) cut live sessions and refuse opens while
   the vault is locked (vault lock ruling 48). Nothing in them is Knot's except `arm`
   (`:77`); they generalise to a `LaneGate` the resident wraps around every lane's
   `session`, so a domain cannot forget the lock rule. The resident calls `open` after
   unlock, `refresh` on its tick (`djinn.rs:1513`, `:1610`), `close` on lock (`djinn.rs:1588`)
   and `open` again on unlock (`:1597`).
3. Custody: epoch keys only for the derivations in `custody`; anything else is refused
   (the policy check in `resident.rs:101-103`).
4. Blobs: a lease on the resident's single blob store, scoped per lane (today
   `ResidentBlobCustody::authorizer`, `bind_scope`, `migrate_legacy_store`). Its type moves
   to Eidetic under the resident split plan's S4; the contract names the lease, not the
   crate.
5. Status and events: each registration appears in resident status (id, owner, version,
   lane open or closed). Lifecycle events keep their names, `"{label}-closed"` and
   `"{label}-reopened"` (`djinn.rs:1593`, `:1600`), so the event file stays byte-identical
   for the harness lifecycle, as the split plan's S6a requires.

**Rules.** Duplicate `domain` or lane `label` is refused. A registration after the first
`serve` is refused. A lane that fails `open` fails that domain only, loudly, as
`resident.rs:185-196` does for Knot (it closes blob custody first, then reports).
Registration comes from code linked into the process that owns the store. A connecting
application cannot install authority, the property the comment at `resident_mere.rs:147-149`
states ("applications cannot select a weaker callback"): an embedding application is its
own store's owner, and the daemon's builder chooses what the daemon carries.

### 3. What the resident needs from a domain today, by name

**Knot** (the only lane). Quoted from `ports/mere-resident` at `1a26ec4e6`.

| Need | Call site | Kind |
|---|---|---|
| Sync host over iroh and p2panda: `KnotSyncHost::open_with_scoped_evidence(&store, &signing_seed, host_config, blobs, readers, max_artifact_bytes)` | `resident_knot.rs:303-317`; config `sync_host_config` `:408`; `apply_authority` and `refresh_dial_hints` `:357-380`; `node_id`, `space_id` `:382-395`; `close` `:397`. Feature `peers`, `Cargo.toml:142-144` | lane |
| Vault open from custody keys: `PersonalVaultKeys::requests()`, `from_released`, `StartupUnlockedPersonalVault::open`, `into_resident_source` | `resident_knot.rs:238-255` | lane |
| Epoch derivation checked against Knot's declaration: `knot_editor::declaration::app_declaration().custody.epoch` | `resident.rs:95-97` | lane custody |
| Retained evidence on the shared blob store: `migrate_legacy_store(.., LEGACY_KNOT_LEASE)`, `bind_scope`, `KnotContentRetentionPort::borrow_scoped`, `readers.retain` | `resident_knot.rs:257-300`; const `resident_blobs.rs:29` | lane blobs |
| Route over a gate: `register_erased(RESIDENT_KNOT_ROUTE, "Knot", ..)`, `KnotGate::arm`, `close` | `resident_knot.rs:61-120`, `:335-343`; `resident.rs:466-475`; `djinn.rs:1359` | lane route (gate generic) |
| Owner selection: `OwnerSettings.knot: Option<KnotResidentSettings>` | `settings.rs:151-152`, `:538-560` | lane settings |
| Lifecycle: open, refresh, close on lock, reopen, log node and space | `resident.rs:41-44`, `:183-196`, `:259-262`, `:510-553`; `djinn.rs:1256`, `:1513`, `:1577-1614` | lane lifecycle |
| Relation intake: `KnotRelationProjection::from_store`, `import_knot_relations` into a pandect `GraphSession` | `knot_relation_intake.rs:72`, `:138`; exported `lib.rs:22`; no caller in the resident | importer, see 4 |
| `mere-overlay`: Knot's `MereOverlay` driving the route through the door | dev-dependency `Cargo.toml:295-296`; tests `resident_mere.rs:3691`, `:3789`, helper `knot_door` `:3625` | client test, see 4 |

**Cleromancy** (authority only).

| Need | Call site | Kind |
|---|---|---|
| `validate_divination_graph(graph).map_err(..)` installed for `"divination"` | `with_divination_authority`, `resident_mere.rs:150-158` (line 154 names the crate); `Cargo.toml:117` | graph authority |
| The same hard-coded in both compositions | `bin/djinn.rs:1408`; `embedded_reservoir.rs:250` (the default of `open`) | composition |
| Read-only replay check with the same validator | `bin/c1_receipt_host.rs:374`; install `:536` | receipt host, see 4 |
| `with_personal_facts_authority` (installed by the same call) | `resident_mere.rs:162-168`, over `mere-facts` | Mere's own crate, not Cleromancy |
| Cleromancy's own embedding call, the template for any app | `cleromancy/src/ui/worker.rs:165-167`: `open_application(options, &mut prompt, \|routes\| async { Ok(routes.with_divination_authority()) })` | app side |

Not named by a cargo row but part of the same coupling: 29 references to `cleromancy_domain`
in `resident_mere.rs` (one production, in `:154`; the rest in the four `natal_*` tests,
`:2502`, `:2568`, `:2649`, `:2730`), and 28 in `tests/embedded_reservoir_validation.rs`.

### 4. How each use maps, and what is not domain behaviour

| Use | Maps to | After |
|---|---|---|
| Knot sync host, vault open, epochs, blobs, route, settings, lifecycle | `Lane` registered by Knot (`label` `"knot"`, `Authority::None`) | `ResidentKnot::open_through` (`resident_knot.rs:209-330`) and its settings type move to Knot as a lane factory; `KnotGate` and `GatedSession` generalise into Mere's `LaneGate`; `OwnerSettings.knot` becomes `lanes.knot` |
| `knot_epochs` closure | the generic `EpochKeys` limited by the registration's `custody` | the Knot declaration stays in Knot; Mere's `resident.rs` loses its one Knot name |
| Relation intake | **not domain behaviour.** An importer from Knot's verified projection into a Mere graph session; nothing in the resident calls it | leaves the resident, app-owned: `knot_relation_intake.rs` (349 lines) to Knot with its test file (817 lines, 6 tests). Knot already consumes Mere's graph crates for other reasons |
| `mere-overlay` tests | **not domain behaviour.** Knot as a client of `mere/<domain>`; the route's archive and merge behaviour is covered at intent level in Mere (`resident_mere.rs:4966`, `:5213`) | the two tests and `knot_door` move to Knot as consumers of the resident; Mere loses the `mere-overlay` dev-dependency |
| `knot-document` workspace row (`Cargo.toml:279`) | not used by Mere's graph except through `knot-editor` | leaves with the rows |
| Cleromancy validator | `Authority::Graph { complete: validate_divination_graph, admission: None }` registered by Cleromancy | `with_divination_authority` leaves Mere; `djinn.rs:1408` and `embedded_reservoir.rs:250` read the registry |
| `personal-facts` authority | **Mere's own domain**: `mere-facts` is `crates/domain/personal-facts` in Mere | stays; the registry's `mere_defaults()` installs it unconditionally, so it no longer rides inside the Cleromancy call. Cleromancy still depends on `mere-facts` from `mere.git`, which is the legal direction |
| `c1_receipt_host` | **not resident behaviour.** "Explicit public synthetic resident for the Cleromancy C1 headed lane" (`:1`) | moves to Cleromancy as a binary consuming `mere-resident`; Mere keeps the `custody-receipt-host` feature for the other receipt hosts |
| Cleromancy natal tests in `resident_mere.rs` (4) | product conformance of Cleromancy's authority through the resident | move to Cleromancy; Mere keeps the generic `domain_authority_checks_route_batches_archive_compose_and_first_edit_forks` (`:4851`) over a fixture validator |
| `with_divination_authority()` used as a mere fixture | not Cleromancy: tests that need *some* validator for the domain id `divination` | switch to `with_domain_validator` with a fixture, at `tests/embedded_reservoir_two_process.rs:484`, `tests/mere_two_process.rs:162`, `tests/reservoir_access.rs:137`, `examples/knot_overmap_live_host.rs:169` (the example itself is Knot's), `resident_mere.rs:2392` |

Also check in P2: `tests/declared_applications.rs` and `tests/app_projection_offers.rs` use
`knot_editor::declaration` and `knot_editor::KnotEndpoint` as a guest fixture. They are
Mere-door tests with Knot as a convenient guest; P2 replaces the guest with a fixture
application in Mere (the repository has `tests/fixture_declarations`) and moves any
Knot-specific assertion to Knot. String app ids (`"knot-editor"`, `"cleromancy"` in grants,
`tests/fixtures/owner_applications.json`) are data, not dependencies, and stay.

### 5. Where each app's registration call lives, and the process shapes

The rule: **the registration call is in the binary or host crate that composes the
resident**, and that crate depends on Mere (an app consumes its host). Mere's manifests name
neither. Three launches exist today.

**Embedded resident (Cleromancy).** The app binary links `mere-resident`
(`cleromancy/Cargo.toml:44`) and passes the hook to `open_application`
(`cleromancy/src/ui/worker.rs:165`). After P3 the hook body is
`registry.register(cleromancy_core::registration())` instead of `with_divination_authority()`.
The only change is whose name the call carries. The embedded default
(`ReservoirAttachment::open`, `embedded_reservoir.rs:241-250`) stops installing anything.

**The daemon (`djinn`).** `ports/mere-resident/src/bin/djinn.rs` (1,712 lines) composes
everything in `run(args, events)` (`:1069`, about 580 lines), including the Knot lane
(`resident.rs`) and the routes (`:1397-1408`). It is built from Mere's manifests, so it
cannot link Knot or Cleromancy. Therefore:
- the composition body moves into `mere_resident` as a library entry taking a
  `DomainRegistry`;
- Mere's own `bin/djinn.rs` becomes a thin `main` passing `mere_defaults()` (the
  personal graph replica, personal facts): a resident with no product domains;
- **the resident an owner installs is a distribution binary** that passes
  `mere_defaults()` plus Knot's and Cleromancy's registrations. It lives where both domain
  crates are legal dependencies, which is above Mere, Knot and Cleromancy (decision (a)
  below). `install-linux.sh DJINN_BINARY VAULT_CLIENT_BINARY` already takes the binary as an
  argument, and `--installed`, `--launch <app>` (descriptor 3 handle) and `--resident-status`
  are unchanged.

**graphshell-mere** (`ports/graphshell/mere`, bin `graphshell_mere`). Door client only: its
manifest names `graphshell-door` (`projection-door`), `mere-overmap`, `mere-view`, not
`mere-resident`. It has nothing to register and nothing changes. It views whichever meres
the resident serves; a domain the resident does not carry shows as unavailable. The
Graphshell reference host's own personal replica (`mere.graphshell/personal-graph/v1`) is
one of Mere's defaults.

**Can an app in a separate process register?**
- **Presentation and declaration: yes, today.** The app offers a projection over the app
  door (the `projection-provider` path above), declares custody epochs and routes in its
  `AppDeclaration`, and the owner's settings permit it. It cancels when the app
  disconnects. No change.
- **Authority and lanes: no, by design.** The validator and the fold must run where the
  store lives, on receivers that may sync while the app is closed (C3, C6). A callback across
  a process boundary would make admission depend on a peer's liveness, and a lane is a
  vault and a socket. The only way for such code to run in the resident is to be linked into
  it.
- **What changes about launching.** (1) Stock `djinn` built from Mere carries no Knot or
  divination authority; the installed resident is the distribution binary. (2) A user who
  runs stock `djinn` and Cleromancy gets "unavailable" for `divination`, not an embedded
  fallback, because a door that answers never leads to embedding. This is a regression
  against today and must be a done-condition of P3: the installed binary carries the
  registration. (3) The two-process tests already show the pattern for apps' own tests:
  they re-spawn the test binary as the daemon and register in the child
  (`tests/embedded_reservoir_validation.rs:442-500`).

**Later, separately: out-of-process authority as a component.** A graph validator is a pure
function; it could ship as a WebAssembly component the owner installs, loaded sandboxed by
the resident, on the family's own WIT idiom (host/guest brief, standards survey). Knot's lane
cannot (it is iroh and a vault), and Cleromancy's validator would need its graph crates to
build for wasm (unmeasured). Not proposed now; it is decision (d).

### 6. What the shared dev and test fixtures become

**`tests/embedded_reservoir_validation.rs` (1,370 lines, 9 test attributes) is two files.**
It is not wholly Cleromancy's:

| Part | Lines | Tests | After |
|---|---|---|---|
| Generic registration conformance: `registered_domain_validator_survives_ownership_and_refuses_complete_batches` over the fixture domain `validated-fixture`, and the child entry `child_serves_the_registered_domain` (shared with the product test) | `1-257`, `319-575` | 2 | stays in Mere over the registry, as the conformance test of this contract |
| Product replay: `published_divination_authority_survives_three_real_resident_owner_processes` (`:258-318`) and `mod published_replay_regression` | `258-318`, `577-1370` | 1 + 6 | moves to Cleromancy as a consumer; needs the two-process child harness |

The child harness (spawn self as owner, endpoint, durable-bytes check) is shared by both, so
the split also moves it somewhere both can use: `crates/system/djinn-testkit`, which is "path
only, unpublished, so packaging drops it" (`Cargo.toml` dev-dependencies), becomes a git
dev-dependency of the app repositories (`branch = "main"`). A dev edge from an app to Mere is
the legal direction under D6.

**The Knot tests.** The plan says "five Knot tests"; the tree has four `tests/knot_*.rs`
files plus three inline tests, so P2 recounts before moving and carries the count:
`knot_app_route.rs` (152 lines, 1 test), `knot_checkpoint_receiving.rs` (330, 2),
`knot_relation_intake.rs` (817, 6), `knot_residue.rs` (370, `harness = false`, its own runs,
the `[[test]]` at `Cargo.toml:352`), the inline test in `resident_knot.rs`, the two
`knot_overlay_*` tests, and `examples/knot_overmap_live_host.rs` (350 lines). They move to
Knot as consumers of the resident, with `resident_knot.rs` going with the lane. Knot's lock
already names Mere, so a dev edge to `mere-resident` adds no cycle.

**What Mere keeps, and must gain (P1b's done-conditions):** conformance tests over a fixture
graph domain and a fixture lane that prove: duplicate and late registration refused; a
product domain without authority refused with the typed error; a lane's route refuses and its
live sessions are cut on lock and serve again on unlock; settings reach the lane opaque;
status lists registrations; events keep their names. Today only one inline test
(`resident_knot.rs`) covers the lock behaviour, and it is Knot's.

### 7. Against the resident split plan, the host/guest brief, and D7 to D11

**Resident split plan** ([2026-11-05](../implementation_strategy/2026-11-05_resident_split_plan.md)).
No slice is contradicted. Interactions:
- *Constraint wording.* It says "concrete producer registration" stays in Mere, and its D2
  outcome repeats it. Cycle breaks D1 (later, Mark's) narrows that: the registry
  mechanism and Mere's own domains' registrations stay; Knot's and Cleromancy's leave.
  Decision (e) asks Mark to amend the line.
- *S4* moves `resident_blobs.rs` to Eidetic. P2 touches it (the `LEGACY_KNOT_LEASE` const at
  `:29` and its test use at `:391`; the lane passes its lease name instead). Per the common
  rules this is a file the split is moving: P2 waits for S4 or lands the lease-name
  parameter first (decision (f)).
- *S6a* moves `resident_events.rs` and requires byte-identical event output. The event
  names stay `"{label}-closed"`; `diagnostics.rs` and the emit sites stay.
- *S6* keeps `personal_sync_host.rs`, graph keys and carriage in Mere: the Mere defaults.
  M4's replication of a `notes` mere will absorb the lane's sync host later; until then the
  lane carries it.

**Host/guest brief.** Consistent with its (c): the presenting half rides `projection-provider`
and needs no new design; this section is the in-process rest. It does not widen the
contract the brief wants narrow.

**D7 to D11: where the registration contract belongs.** In Mere, not in
`merely-made/graphshell`:
- D8/D9 make the protocol layer a repository whose crates "may depend on nothing in Mere".
  The registration's authority variant carries `pandect::GraphValidator`, the lane carries
  `ResidentEndpoint` and `AdmittedEndpointContext` (`graphshell-host`) and `ResidentEndpointCatalog`,
  all Mere types. It cannot move down without moving the resident's graph with it.
- The layer already holds the part that is protocol: the `projection-provider` world in the
  one WIT file (D9, D10) and the chirograph wire. Knot's `projection-provider` feature takes
  `graphshell-door` from Mere (D11), the app-consumes-host shape this design repeats.
- C5's data interface belongs to Stickleback in Eidetic, because domains must not depend on
  Mere's graph crates. `Authority::Log` takes that trait when it exists.
- If a wasm validator is ever chosen, its WIT world is the protocol layer's, and it joins the
  one world file under D10's open question about renaming `mere:script@0.1.0`.

**Where the plan's own citations drifted:** the root rows for Knot are now
`Cargo.toml:278-279` (plan: `:283-284`); the resident rows are `:144` and `:296` (plan:
`:143`, `:295`). The `[patch."https://github.com/merely-made/mere.git"]` section is at
`:664`. Its comment says it redirects what Knot names; Cleromancy also names `mere.git`, so
P2 and P3 decide what leaves by comparing the resolved graph, not by trusting the comment.
Also: `crates/inker/knot-editor-host` (`Cargo.toml:84`, `:355`) is Mere's own crate and not
the Knot repository; the check for "Mere names Knot" is the `knot-editor.git` source, not the
string.

### 8. Decisions for Mark

Answer before P2 or P3 starts. Each has a recommendation.

**(a) Where does the installed resident (the distribution binary) live?**
1. A new thin repository above Mere, Knot and Cleromancy that builds `djinn` with every
   first-party registration. One owner of the composition; apps release independently.
2. In one existing app repository (Knot or Cleromancy). Smaller, but that app then names
   the other app, an odd edge.
3. Keep the stock `djinn` only, and wait for wasm validators. No new repository, but Knot's
   lane cannot be loaded that way, so Knot would stay in Mere and D1 would not close.
- Recommend 1. It is the only option that closes D1 for both apps, and the installer already
  takes the binary as an argument.

**(b) Who builds the registry, and when?** The plan has P2 and P3 independent and both
reusing P1, but both edit `resident_mere.rs`, `resident.rs` and `djinn.rs`.
1. A phase P1b before them: the registry and the library entry, Mere's own domains only,
   plus the conformance tests of section 6. P2 and P3 then parallelise.
2. Fold it into P3 (the smaller, graph-only half) and run P2 after it.
3. Leave the order open and let two agents collide.
- Recommend 1. It is M1 and M2 anyway, and keeps P2 and P3 to deletions plus registrations
  in the app repositories.

**(c) Where does the registration type live?**
1. Mere (`mere-resident`), with C5's trait in Eidetic and the presenting half in the
   protocol layer, as in section 7.
2. Eidetic's Stickleback, including the lane.
3. `merely-made/graphshell`, the protocol layer.
- Recommend 1. Options 2 and 3 pull Mere's graph, door and endpoint types below Mere.

**(d) Out-of-process authority.**
1. Not now: apps register presentation out of process, authority only by linking.
2. A sandboxed WebAssembly validator component, for graph domains only, later.
3. A door callback to the app for validation.
- Recommend 1, noting 2 as a later plan. Option 3 makes admission depend on a peer being
  alive and breaks C3 and C6.

**(e) The split plan's "concrete producer registration stays in Mere".**
1. Amend the constraint and D2's outcome to: the registry mechanism and Mere-owned domains'
   registrations stay.
2. Leave it and record the exception only in this document.
- Recommend 1, so two active plans do not disagree (DOC_POLICY §2).

**(f) P2 against S4 (`resident_blobs.rs` moves to Eidetic).**
1. P2 after S4.
2. P2 first, with the lease name parameterised, so S4 moves a file that no longer names Knot.
3. In parallel, coordinated by the lease name.
- Recommend 2. It touches two lines, removes the Knot name from the file S4 moves, and does
  not wait on S4's schedule.

Settled in this design, not asked: `personal-facts` stays Mere's own domain and is installed
by `mere_defaults()`; owner settings keep reading the existing top-level `knot` key as
`lanes.knot` for one release because they are on users' disks (DOC_POLICY §3 allows the
shim for real user data); the lifecycle event names do not change.
