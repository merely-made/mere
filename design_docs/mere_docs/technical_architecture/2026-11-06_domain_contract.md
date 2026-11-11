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

**Superseded in part (cycle breaks D15, 2026-11-06):** the lane parts of this section (`LaneFactory`,
`Lane`, `LaneContext`, `LaneGate`, the lane rows of sections 3 to 6) and the distribution binary of
section 5 are superseded by the drawing-board section: Knot's lane becomes a supervised
application, authority is a guest or declared data, and there is no `Lane` trait and no
distribution binary. The registry, `Authority`, the M2 refusal and the rest stand.

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
build for wasm (unmeasured). Not proposed now; it is decision (d). Cycle breaks D12's
guest blocks design (`graphshell/design_docs/graphshell_docs/technical_architecture/2026-11-06_guest_blocks.md`) already chooses a reviewed, signed wasm guest
for Woodshed's parts; a validator component would share that delivery path, and (d) should be
answered with it in view.

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

**Resident split plan** (2026-11-05 (`graphshell/design_docs/graphshell_docs/implementation_strategy/2026-11-05_resident_split_plan.md`)).
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
Also: `crates/inker/knot-editor-host` *(historical citation)* <!-- doc-audit: historical-path --> (`Cargo.toml:84`, `:355`) is Mere's own crate and not
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

## Drawing board: where a domain's authority runs (cycle breaks D14 (a), 2026-11-06)

**Status:** design for Mark's review; nothing implemented. Mark answered the registration
section's decision (a), where the installed resident lives, with "Hmmmm. Let's go back to the
drawing board." (cycle breaks D14), rejecting all three options as framed. Decisions (b) to (f)
stand, so P1b (the registry, without any Knot or Cleromancy change) can proceed. P2 and P3 wait
on this section. Read against the same revisions as the section above, plus a locked
`cargo check` of `cleromancy-core` on two wasm targets (section 2). Illustrative code is
illustrative, not compile-ready. The WIT lines are a proposal.

### 1. The premise all three options shared

Every option in decision (a) assumed that a domain's authority is Rust code linked into the
process that owns the store, so someone has to build a binary that names every domain: a
distribution repository, one app's repository, or nothing (which fails D1). The code suggests
that bundles two different things that want different answers:

- **Authority:** what the resident asks a domain, *is this candidate admissible?* It is a
  function of a graph or an operation, called at author and receiver (C3, C6).
- **Lane:** a live component beside the store: a vault, a socket, a route. Only Knot has one.

And the second turns out to be transitional (section 2). The registration section above
specified both together, and `Lane`, `LaneContext` and `LaneGate` are the part of it this
section puts in question.

### 2. What the code says

**Cleromancy's validator is a pure function over a graph, and mostly not data.**
`src/host/domain.rs:46`:

```rust
pub fn validate_divination_graph(graph: &Graph) -> Result<(), HostError> {
    // Build the replay reader without opening or writing a private backend.
    let mut reader = CleromancyHost::empty(muniment::MemoryBackend::new());
    reader.graph = graph.clone();
    reader.validate_domain()
}
```

It opens nothing and reads no clock. Its first block is structural and would fit a table: every
`cleromancy.*` facet must be in `DOMAIN_FACETS` and sit on a real node; a node carries "exactly
one domain record"; a record's node id must equal `Graph::node_namespace_id(node.url())`
(`:62-96`). The rest is replay, not structure:

```rust
for reading in readings {
    if self.replay_reading(&reading)? != reading {
        return Err(ReadingError::ReceiptMismatch("sealed reading".to_string()).into());
    }
}
...
spread.validate()?;
self.replay_three_card_spread(&spread)?;
```

It recomputes each reading from its recorded entropy, re-derives content addresses with
per-record digest functions (`canonical_facet_values(.., AstrologyChart::digest, ..)`), checks
spreads against sessions, and, per the resident's own tests, recomputes natal houses, aspects and
motion (`natal_houses_authority_admits_both_systems_and_refuses_cusp_tamper`,
`resident_mere.rs:2568`). `src/host` is 4,163 lines in 16 files. The input type is Mere's
`kernel::graph::Graph`.

**It does not build for wasm today.** Measured 2026-11-06 in the Cleromancy clone, locked,
toolchain 1.97.1: `cargo check -p cleromancy-core --target wasm32-wasip2` fails in
`redb 2.6.4` (`E0554`, `#![cfg_attr(target_os = "wasi", feature(wasi_ext))]`), reached through
`muniment` with its `redb` feature, which `core/Cargo.toml` names. `--target
wasm32-unknown-unknown` fails in `errno 0.3.14` (`compile_error!`, "The target OS is unknown or
none"). Neither is Cleromancy's code. It shows that the crate holding the validator also holds
native-only dependencies, so a guest build needs a validator-only crate or feature split first.
That is not measured further; the size of the split is unknown.

**Knot's authority is data.** `KnotSyncPolicy::admit`
(`knot-editor/crates/knot-editor/src/sync.rs:312-366`) checks, in order: the operation's
`space_id` equals the vault's; the encryption profile equals the vault's; the author is in
`writers` (an `RwLock<BTreeSet<[u8; 32]>>` that pairing updates); a sealed body is present; under
`CommonsDataV1` the body decodes strictly as Stickleback's `GroupCiphertext`;
`validate_causal_metadata(.., KNOT_CAUSAL_LIMITS)`; then
`Admission::keep(StoreTarget::new(Topic::from(self.space_id), LOG_ID))`. Five parameters (space,
profile, writer set, limits, log id) and Stickleback's own checks. It is `OperationPolicy`-shaped
because it is one.

**Knot's lane is a standalone program the resident happens to host.** `resident_knot.rs:209-330`
opens a persona vault from custody-released epoch keys, lends it the resident's blob store,
starts an iroh and p2panda sync host, and registers a route over a gate. Knot already ships the
same thing as separate programs:
- `knot_sync_host` ("Resident Knot sync for one persona ... stays up and keeps the persona's
  vault space joined, which is what makes an edit on one device reach the others without either
  being opened at the time", `knot-editor/crates/knot-editor/src/bin/knot_sync_host.rs`);
- `knot_endpoint projection-provider`, which offers the route through the app door under the
  owner's `projection_providers` setting (README "App-owned projections", which also says
  "Djinn's embedded Knot service remain separate compositions");
- custody over the door: `resident_knot.rs`'s own header says the epoch release is "the same
  release its custody route answers Knot's own processes with (dramatis DR-C)", and
  `tests/declared_applications.rs:131-174` drives Knot as a declared application asking for it.

**Knot's vault is not a reservoir mere.** The lane keeps Knot's own `KnotSyncFileStore` under
`persona_vault_root(data_root, persona)`; the resident holds no Mere store for Knot. It hosts
Knot for three conveniences: one shared blob store (split plan S4), one lock gate (ruling 48),
one daemon. And the end state removes the lane: adoption plan D4 and the one-kind-of-store
ruling make Knot's operation store the `notes` mere, whose sync is M4, blobs M5 and sealing M3.
The lane exists until those land.

### 3. The options

Common to all: the registry of P1b and decisions (b), (c) and (f) stand. "D1 closed" means no
Mere manifest names Knot or Cleromancy and both apps' behaviour is intact.

**Baseline, for comparison: the registration section's linked distribution** (option 1 of the
old (a)). Moves: nothing; a new repository links Mere, Knot and Cleromancy. Cost: a new
repository, a `Lane` trait for something that is deleted when M3 to M5 land, release lockstep.
Closes D1 for both. C3/C6, custody: unchanged from today. Synergy with D12: none.

#### A. Authority as a guest

- **Idea.** A graph domain's validator ships as a signed wasm component that the resident loads
  at runtime. Proposal, beside `block-adapter` in the protocol layer's one WIT file (D10):

  ```wit
  interface domain-authority {
      record claim { domain: string, version: string, graph-format: string }
      describe: func() -> claim;
      /// The complete candidate graph, copied, in the declared format.
      validate: func(candidate: list<u8>) -> result<_, string>;
      /// Optional before/after check (today's `GraphAdmissionValidator`).
      admit: func(before: list<u8>, after: list<u8>) -> result<_, string>;
  }
  world authority-guest { import log; export domain-authority; }
  ```

  The protocol layer names no Mere type: the graph travels as bytes with a format tag, and the
  guest links Mere's graph crates itself, which a guest above Mere may.
- **What moves.** In Cleromancy: a thin component crate over `validate_divination_graph` (the
  validator, which is already pure), a signed pack (the D12 path: `PackManifest`, `eidetic` pack
  signing). In Mere: a wasmtime host in the resident (`crates/script/app-host` already carries
  `wasmtime = "45"`, shared with `document-host`; `mere-resident` links none today), a loader,
  and a native review and pin store (today's `CapsuleMount::review` is web only). Nothing in
  Mere names Cleromancy; `Authority::Guest(ComponentRef)` joins `Authority::Graph(closure)`.
- **Cost.** (1) The wasm port is blocked as measured in section 2. (2) The validator runs on a
  *complete candidate* at every edit and load, so each call marshals the whole graph across the
  boundary; per-call cost is unmeasured and must be, with a fuel limit, because the replay
  recomputes ephemerides. (3) The 16 MiB component cap of D12 against this dependency closure
  is unmeasured. (4) A new native host surface in the resident.
- **C3/C6.** Intact and stronger. The resident runs the same component at author and receiver.
  Wasm execution is deterministic where native floating point on x86 and arm is not; the replay
  is an ephemeris recomputation, so C6's "every device reaches the same verdict" is better
  served by a pinned component than by two native builds. Two new rules are needed: the
  component's hash is pinned per domain in the reservoir index and must match across paired
  devices, and an operation arriving where no authority is installed waits as pending (the C4
  mechanism), never applies.
- **Custody.** Better than today. The guest imports `log` only, so the validator cannot touch
  keys or the vault; today linked code shares an address space with the custody client.
- **Embedded and daemon.** One path for both: the daemon loads installed packs, so stock
  `djinn` from Mere needs no distribution binary. An embedding app may still pass a native
  closure built from the same source (same trust as today); a conformance test runs native and
  wasm over the replay corpus (the seven tests that move from `embedded_reservoir_validation`)
  and demands the same verdicts.
- **D1.** Closes it for Cleromancy. Knot's authority also fits (B below), its lane does not.
- **D12 and the protocol layer.** The same pack, review, grant and signing machinery and the same
  WIT file. One asymmetry to rule on: a D12 guest is a view, and the host re-applies its result
  so a lying guest can change only its block's attributes; an authority guest is the gate, and a
  lying validator admits anything. Review must therefore bind the author's key to the domain and
  pin the hash, and revoking it refuses the domain.

#### B. Declarative authority

- **Idea.** The domain registers data that the resident interprets: a Stickleback
  `OperationPolicy` implementation parameterised, not coded.
- **How much fits.** Knot's `admit` entirely: `DeclaredPolicy { space, encryption profile,
  writers, causal limits, log id }`, with the writer set owned by pairing (a roster the
  resident already reconciles, `key_paired_devices`). That is the C5 interface for log domains
  in its smallest form. For Cleromancy, the structural prefix fits (facet allow-list, one record
  per node, canonical node id from the url, refuse unknown versions, about 60 of the 380
  lines of `domain.rs`) and none of the replay does: it is `replay_reading`, address
  recomputation and cross-record checks, 4,163 lines of domain behaviour in the validator's
  crate. A policy language that expressed it would be a second Cleromancy.
- **What moves.** Knot: nothing but a policy value. Mere: a generic interpreter in Stickleback
  (Eidetic), which C5 wants anyway.
- **Cost.** Small for log domains; a language for graph domains is the cost that cannot be paid.
- **C3/C6.** Perfect determinism, no code run. **Custody:** none needed. **Embedded and daemon:**
  identical. **D1:** closes Knot's authority only. **D12:** none; it is the Stickleback side.
- **Use.** As the cheap first stage in front of A: reject structurally invalid candidates without
  a component call.

#### C. App-owned stores

- **Idea.** Knot and Cleromancy own their domains' stores; Mere's resident hosts only generic
  domains.
- **What breaks.**
  - *The reservoir rulings.* Adoption plan D1 (the store owns its lifetimes, one owner under the
    reservoir index lock), D8 (every application's authored store is held by the reservoir) and
    the 2026-09-23 ruling ("two browsers should share a mere"). `music` shared by Woodshed and
    Hocket (D5) cannot be owned by both.
  - *Custody.* M3 seals every mere under the persona key through the resident; app-owned
    stores each need their own custody path and sealing, uniform nowhere.
  - *The door.* `mere/<domain>` would be served by the app's process, so it is absent when the
    app is closed, and M4 replication would have to be built per app.
  - *Cross-domain reads.* Cleromancy's facts live in Mere's `personal-facts` domain and its
    profile-subject check reads the persona vault (`with_profile_subject_authority`,
    `resident_mere.rs:171-179`); in an app-owned store both must be rebuilt app-side.
- **D1.** Closes it by retreating from the architecture Cleromancy already uses as a client of
  `mere/divination`. **D12:** none. Not recommended; it contradicts D8.

#### D. Split authority from lane

- **Idea.** Authority is A or B (guest or data). Knot's live sync lane stops being a resident
  component and becomes an application the resident supervises.
- **What moves.** `ResidentKnot::open_through` and its settings leave the resident. What replaces
  them already exists in Knot: `knot_sync_host` (sync, no projection) and `knot_endpoint`
  (the route, offered through the door). Mere gains a generic, small supervisor on machinery it
  has: `application_launcher` (`bin/djinn.rs:412`), `djinn --launch` and the descriptor-3 handle.
  An owner setting names an application to keep running while the vault is unlocked.
- **Custody and lock.** The child asks custody over the door under its own `AppDeclaration`
  (exists). On lock the resident refuses keys and ends the child. Process exit is a stronger cut
  than `LaneGate` plus zeroizing (ruling 49 exists because seed residue in a live process is the
  worry), and `knot_residue.rs` has no process left to judge.
- **Cost.** (1) Blobs. Knot loses the resident's single blob store (split plan S4, "personal-graph
  transfer and Knot evidence still share one store"): either the child keeps its own iroh blob
  store, which is the state before S4 (two endpoints on one machine), or blob custody gains a door
  route (Eidetic's S4 work, later). Users who already migrated need a copy out; S4 says legacy
  stores are "copied and never deleted", so it is verifiable. (2) The resident's `refresh` tick
  and `facts` logging become the child's own. (3) The supervisor is new generic machinery.
- **C3/C6.** Untouched: the lane was never authority.
- **Embedded.** The Knot desktop links `knot-editor` and runs `KnotSyncHost` itself; no lane is
  needed in an embedded resident at all.
- **Daemon.** Stock `djinn` from Mere, no distribution binary.
- **D1.** With A or B, closes it for both apps. **D12:** with A, the same path for Cleromancy;
  Knot's side stands alone.
- **Is it the honest boundary?** Yes, on the evidence. Authority is the part Mere must control,
  because it is the gate to Mere's store, and it is pure. The lane is the app's own live
  behaviour on its own vault; Mere never owned that store, and the reservoir end state (M3 to M5)
  deletes the lane. Building a `Lane` trait and a distribution repository for it would be
  throwaway work.

#### E. A supervised validator process (not asked, from the code)

- **Idea.** A graph validator ships as a native executable the resident spawns and talks to over
  stdio, pinned by hash. Precedent: the custodian is a separate process (`djinn-custodian`,
  custody isolation plan).
- **Cost.** A binary per platform per domain; an IPC round trip per edit with the whole candidate
  graph; OS-level sandboxing only; native floating point (no determinism gain). Liveness is the
  resident's (it spawns), so it avoids the objection to a callback to an app.
- **C3/C6.** Intact, determinism no better than today. **D1:** closes it. **D12:** none.
- **Use.** The fallback if the wasm port in A proves too costly.

#### F. Trust the author (rejected)

The resident runs only generic checks and relies on the app having validated before signing.
C3 requires the full check at the receiver and the resident split plan's constraint is "no
bypass": a buggy or hostile client could write invalid domain data. Not an option.

#### Side by side

| | Closes D1 | C3/C6 | New machinery | D12 synergy |
|---|---|---|---|---|
| Baseline (distribution repo) | both | unchanged | repo, `Lane` trait | none |
| A. Guest authority | Cleromancy; Knot's policy | stronger (determinism) | wasmtime host, pin and review | full |
| B. Declared authority | Knot's policy | exact | interpreter in Stickleback | none (C5) |
| C. App-owned stores | both, by retreat | breaks D8 | per-app everything | none |
| D. Authority split from lane | both (with A or B) | unchanged | small supervisor | with A |
| E. Authority sidecar | both | unchanged | per-OS executables | none |

### 4. Recommendation

**D, with B in front of A for authority and E held as the fallback.** In order:
1. **P1b now,** as D14 ruled, with `Authority` carrying `Graph(closure)` for embedding and Mere's
   own domains, `Guest` and `Declared` as slots, and **no `Lane` trait**.
2. **Spike before committing P3:** make a validator-only build of Cleromancy's authority for
   `wasm32-wasip2` (split off `muniment/redb` and the other native dependencies) and measure
   (i) it builds, (ii) component size against 16 MiB, (iii) a verdict-equal run over the
   seven moving tests' corpus, (iv) per-call time on a realistic graph. If it fails, E.
3. **P2 as a supervisor, not a lane:** Knot's authority is a `DeclaredPolicy`; its sync host
   runs as the supervised `knot_sync_host`, deleting `resident_knot.rs` from Mere. It is
   transitional by design.
4. The distribution repository of the old (a) is not built. If the spike fails and Mark prefers
   linking to E, the old (a) option 1 is the fallback for Cleromancy alone.

What this changes in the registration section: sections 2 (`LaneFactory`, `Lane`, `LaneContext`,
`LaneGate`) and 5 (the distribution binary) are superseded if Mark accepts D; sections 1, 3, 4,
6 and 7 stand, the fixtures table with one change (Knot's resident tests move with the sync host
into Knot as tests of a supervised process, not of a lane).

### 5. Decisions for Mark

**(g) Is the authority/lane split the boundary?**
1. Yes: authority as guest or data, Knot's lane as a supervised app, until M3 to M5 retire it.
2. Authority as guest or data, but keep a `Lane` trait and a distribution binary for Knot only.
3. Keep the linked distribution of the old (a) after all.
- Recommend 1: the lane is transitional and Knot already ships it as a program.

**(h) The form of Cleromancy's authority.**
1. A wasm guest, after the spike in the recommendation.
2. A supervised native validator process (E).
3. Linked code in a distribution binary.
- Recommend 1 with 2 as the fallback. Rule at the same time whether an authority guest needs
  stronger review than D12's views (author key bound to the domain, hash pinned across paired
  devices) and whether operations awaiting an absent authority wait as pending: recommend yes to both.

**(i) Knot's blobs while its sync host is supervised.**
1. The child keeps its own blob store now (two iroh endpoints until M5), with a copy out for
   users already migrated.
2. Wait for the door route to the resident's blob custody (S4's follow-on) before moving Knot.
3. Keep Knot's lane compiled into the resident until M4 absorbs it (leaves Knot named in Mere's
   manifest, so D1 stays open for Knot).
- Recommend 1: the cost is bounded and temporary, and it is the only option that closes D1 for
  Knot without waiting on S4.

## The blob door route (cycle breaks P2a, 2026-11-08)

**Status:** route, grant and revocation shape stated first, as P2a requires; built the same day
(section 6, "As built"). For Mark's review alongside D15 (i).
Read against Mere `a2c41e933`. Mark's D15 (i) ("Wait for a door route") made this the
prerequisite for moving Knot's lane out of the resident.

### 1. What exists, and what is asked of the route

`ResidentBlobCustody` (`ports/mere-resident/src/resident_blobs.rs`) owns the resident's one iroh
store and one `BlobReadAuthorizer`. Two in-process lanes borrow it today: personal-graph transfers
(`personal_sync.rs`, scope = the graph) and Knot's evidence (`resident_knot.rs`, scope = the
Knot space id). A lane uses six things: `blobs()` (put under a `BlobLease`, lease and release an
existing hash, read bytes), `authorizer()` (retain and release a hash for serving, add, remove and
replace a scope's readers), `bind_scope` (rebuild serving custody from durable leases after a
restart), and `migrate_legacy_store` (copy an old per-lane store under verified leases, with a
marker, never deleting the source). The route must give an admitted application the same six,
over the app door, and nothing else.

### 2. The route

- **Name:** `blob-custody-v1`, a resident endpoint route in the app catalog, like
  `device-directory-v1`. It rides the existing projection carrier; no wire in Ostiary changes.
- **Calls** are intents: one intent, `blob.call`, whose payload is a JSON `BlobRequest { call, op }`.
  `op` is one of `bind-scope`, `put-begin`, `put-part`, `put-commit`, `put-abort`, `lease`,
  `release`, `retain`, `unretain`, `allow-reader`, `deny-reader`, `replace-readers`,
  `migrate-legacy`. Every op that names a scope names it explicitly.
- **Replies** are the route's snapshot: its one resource is a `BlobLedger` JSON document holding the
  outcomes of the last 64 calls by `call` number plus, for each bound scope, its leased hashes and
  its readers. The intent answers `Accepted` or `Rejected { reason }`; the outcome (a hash count, a
  `LegacyBlobMigration`, the committed hash) is read from the ledger. This is the device
  directory's pattern, and it keeps the wire as it is.
- **Bytes out** are the carrier's resource chunks. The resource address is the BLAKE3 hash,
  which is also the iroh `BlobHash`, so `resource_chunk(hash)` is `BlobStore::read_range`. A read
  is answered only for a hash leased in a scope the application owns.
- **Bytes in** are staged in parts of at most 64 KiB (a JSON byte array inside a 1 MiB frame),
  declared up front with the length and the BLAKE3 hash, verified at `put-commit` before the
  store sees them, then installed under the named lease. One staged upload at a time per session,
  at most 256 MiB.

### 3. Grant

Two gates, both already the family's idiom, and no default grant:
1. **Route:** the application declares `blob-custody-v1` in its `AppDeclaration.routes` and the
   owner's grant names it (`applications::grant_for`, the owner dialog lists it as a route). An
   ungranted application never opens the endpoint (the `an_ungranted_route_never_opens...` test
   pins that for the catalog).
2. **Scope:** a scope is claimed by the first admitted application that binds it, recorded in a
   durable ledger (`content/scope-owners.json`, written atomically). Another application naming
   that scope is refused with the owner named. The resident's own in-process scopes (personal
   transfers) are reserved to `resident` when they bind, so no door application can name them. An
   application may therefore use the Knot space id as its scope, as the in-process lane does, so
   leases already in the store (`knot.evidence`, `legacy.knot`) are the same leases the supervised
   child will use; nothing is re-leased or re-keyed.

The lease name stays the lane's: `namespace` is the caller's (`BlobLease::new` still validates its
characters), and the migration `namespace` must begin `legacy.` as the in-process calls' names do.
A migration `source` is a directory path the application names. The door is owner-only and the
same user could open that directory, so this widens nothing, but the resident opens it read-only
and copies only blobs the legacy store retains.

### 4. Revocation

- **The grant** is revoked the way every route grant is: the owner removes it, and the door
  refuses the next open.
- **A live session** is cut by `BlobDoor::revoke(app)`: the endpoint's `is_live` turns false, its
  calls are refused, and the resident releases the application's *serving* authority (every hash
  retained in its scopes, every reader) from the in-memory authorizer. Leases and the owner ledger
  are kept: custody data is never deleted by revocation (S4 says legacy stores are "copied and
  never deleted"; leases follow the same rule). `BlobDoor::restore(app)` reinstates the
  application, which then re-binds its scopes and so rebuilds serving custody from the durable
  leases, exactly as after a resident restart.
- **On lock** the resident refuses new sessions and ends live ones, as for custody (a locked
  resident serves no route).
- **Shutdown** is `ResidentBlobCustody::shutdown`'s existing check: the route holds a custody
  handle, so its catalog is dropped inside the task scope before the shutdown that refuses live
  borrowers.

### 5. What is not in it, and the S4 sequence

- No new serving path. Peers fetch blobs from the resident's iroh endpoint under the shared
  authorizer, as now; the route only changes who may retain and release. What an out-of-process
  sync host does about *serving* (it has no iroh store) is not answered by this route; see the open
  question in the cycle breaks plan, P2.
- `resident_blobs.rs` moves to Eidetic under S4. The route's server side is in its own file
  (`resident_blob_door.rs`) and reaches `ResidentBlobCustody` only through its public methods plus
  the two scope-owner methods this change adds, so S4 moves one file and the route follows by import. Decision
  (f) is done first: `LEGACY_KNOT_LEASE` left that file and is now `resident_knot`'s.
- The client and the wire types are in `graphshell-door` (feature `blob-door`), so a supervised
  application links a door client and not the resident.

### 6. As built (2026-11-08)

- **Server:** `ports/mere-resident/src/resident_blob_door.rs` (`BlobDoor`, registered by `bin/djinn.rs`
  beside the published-site route; `BlobDoor::revoke` and `restore`). **Wire and client:**
  `crates/graphshell/graphshell-door/src/blob_door.rs`, feature `blob-door`
  (`BlobDoorClient`: `bind_scope`, `put`, `lease`, `release`, `lease_hash`, `retain`, `unretain`,
  `allow_reader`, `deny_reader`, `replace_readers`, `allows`, `migrate_legacy`, `read`, `ledger`).
  **Custody:** `ResidentBlobCustody` gained `claim_scope`, `reserve_scope`, `scope_owner`,
  `scopes_of` and `withdraw_serving`, and the durable owner record `content/scope-owners.json`;
  `personal_sync` reserves the personal graph's scope as `resident`. The Knot lane claims nothing:
  it is the in-process caller until P2 removes it, and the supervised application then claims the
  same scope.
- **The one narrowing against the in-process calls** (beyond the namespace and path rules in 3):
  `retain` is refused unless the scope holds a lease on the hash, because the door otherwise lets an
  application authorize serving of bytes it does not hold by naming another scope's hash.
- **Not wired:** nothing in `djinn` calls `BlobDoor::revoke` yet, since the daemon has no live
  grant-removal path (owner grants are read at start and on the consent dialog's yes). Revoking by
  removing the grant therefore takes effect at the next connection and the next restart; the
  live cut is the API, tested.
- **Open for P2 (not this route's):** the route moves custody administration and reads. It does not
  move *serving*. Knot's sync host builds its iroh endpoint with
  `P2pandaTransport::builder.scoped_blobs(&blobs, scope, readers)`
  (`knot-editor/crates/knot-editor/src/resident/operations.rs:131-135`), so the endpoint that
  peers dial serves bytes out of the `BlobStore` in its own process. A supervised child cannot hold
  the resident's store, so P2 step 3 needs a ruling; the options are in the cycle breaks plan's
  Findings.
- **Tests:** `resident_blob_door::tests` 8 (the lib's `resident_blobs` 2 unchanged), `tests/blob_door.rs`
  1 (the real door: put in parts, read in chunks, lease, release, serving authority, migration,
  a second application refused, an ungranted application refused, revocation, restore), and
  `graphshell-door` 2 wire tests (4 total in the crate with the feature).

## Serving through the door: the source trait and the verified read (cycle breaks P2 step 3, 2026-11-08)

**Status:** designed first, then built the same day (section 7). Mark's ruling on the cycle breaks plan's
"P2 findings" serving question: "(2) Transport change". The blob door route above moves custody
administration and reads; this section moves *serving*, so a supervised application that does not hold
the resident's store still answers peers' iroh-blobs requests with verified streams.

### 1. The problem, with the code

Knot's host binds its endpoint with `P2pandaTransport::builder.scoped_blobs(&blobs, scope, readers)`
(`knot-editor/.../resident/operations.rs`). In Murm (`native/crates/transport/src/p2panda_transport.rs`)
that registered a `ScopedBlobsProtocol` holding the iroh `Store` and the in-process `BlobReadAuthorizer`
and delegated each allowed request to iroh-blobs' `handle_get`, which calls `store.export_bao(hash, ranges)`:
the BLAKE3 outboard (hash pairs) and the data, verified as they are read, written to the QUIC stream as the
iroh-blobs response (an 8-byte little-endian size, hash pairs in pre-order, leaves). iroh's file store is
single-process, and the door's reads (`read_range` over carrier chunks) carry data but not the pairs.

### 2. The source trait (Murm, crate `mere-transport`, module `blob_source`)

`BlobSource` is what a scoped provider reads from, in three questions:

- `allows(scope, peer, hash) -> bool`: may this peer have this hash from this scope now.
- `children(hash) -> Vec<BlobHash>`: the hashes a complete hash sequence names.
- `export_bao(hash, ranges) -> BaoReader`: the response bytes for those chunk ranges of that blob, in pieces.

`P2pandaTransportBuilder::scoped_source(Arc<dyn BlobSource>, scope)` registers the provider over any source.
`scoped_blobs(&store, scope, authorizer)` keeps its signature and is now a one-line wrapper that builds a
`StoreBlobSource`, so there is one provider, not two: every in-process caller (the personal replica, the
Moot example, the old Knot lane) runs through the code the child uses. The provider's rules are the
store-backed one's: read-only; every named hash, and every child a hash-sequence request reaches, authorised
before any byte is written; pushes and unknown requests refused. **One narrowing:** `Observe` requests are
refused (the store-backed provider answered them; no caller in the family issues one, and `fetch` observes
the local store, not the remote).

### 3. What the door adds: the verified read

Two ops on `blob-custody-v1`, in the existing intent-then-ledger-then-resource pattern, no change to the carrier:

- `bao-open { scope, peer, hash, ranges }`: `ranges` is a list of `(start, end)` chunk intervals (`end`
  absent for open-ended), the wire form of iroh's `ChunkRanges`. The resident checks the policy below, then
  makes a stream cursor and answers `bao { resource, total_len }` in the ledger. `resource` is a content-address
  label (BLAKE3 of the route name, scope, peer, hash and ranges), unique per request.
- `bao-close { resource }`: drop a cursor.
- **Bytes out** are the carrier's resource chunks at that `resource`, read in order from offset 0. The resident
  produces them from its own store's `export_bao` (so they are verified by the store as they are read, not
  recomputed anywhere), the child writes each chunk straight to the QUIC stream, and the fetching peer verifies the
  stream against the hash it asked for. A read out of order is refused (streams are sequential; a restart is a
  new `bao-open`). `total_len` is computed at open by counting one pass of the export, so the client can use the
  ordinary chunk assembly without a cap beyond a stream's length.

Child side: `graphshell-door` (feature `blob-door`) carries the wire and the client methods (`bao_open`,
`bao_chunk`, `bao_close`) and no iroh type. The `BlobSource` implementation over a `BlobDoorClient` is in
the application that serves (Knot), on its own door session (`DoorBlobSource`): `allows` is the route's
existing `allows` call, `children` reads the sequence through the verified `read`, and `export_bao` is
`bao-open` plus chunks.

### 4. Security properties

1. **Only leased or retained hashes.** `bao-open` is refused unless the scope holds a lease on the hash (the
   rule `retain` already has) **and** the resident's authorizer allows `(scope, peer, hash)`: retained by the
   scope and the peer a reader. A hash the store holds for another scope, or leased but not retained, cannot be
   streamed.
2. **Scope ownership.** `scope` must be owned by the calling application (`owned`, the durable owner ledger). The
   resident's own scopes (the personal graph) are reserved, so no door application can name them.
3. **Revocation.** Every chunk read re-checks that the application is not revoked and that the authorizer still
   allows `(scope, peer, hash)`. `BlobDoor::revoke` cuts the session and withdraws retention and readers, so an
   open cursor stops at its next chunk, an unretained hash or a removed reader stops the same way, and a new
   `bao-open` is refused. Leases are kept (custody data is never deleted by revocation).
4. **The child is trusted for availability only.** It names the peer (it holds the transport and authenticates
   it), so a compromised child can stream any of *its own scope's* retained hashes to a key it chooses; it
   cannot reach another scope's, a leased-only hash, or the resident's. The bytes it forwards are verified by the
   store that read them and by the peer that receives them.
5. **Bounded.** At most 4 open cursors per session, each a streaming read (no whole-blob buffer in the resident);
   chunks are at most the carrier's `MAX_RESOURCE_CHUNK_BYTES`.
6. **Lock.** On lock the resident refuses new sessions and ends live ones, as for custody; with the session gone the
   child has no source and serves nothing, and the supervisor ends the child (cycle breaks P2 step 3).

### 5. What stays as it is

Knot's evidence reference (`KnotClipEvidenceRef`, whose hash is the blob hash) and its dial hints are unchanged:
the peer still dials the Knot device's node id and asks for the hash, which is now answered by the child. The
resident's own iroh endpoint is not asked to serve Knot's evidence, so no new node id enters the references.

### 6. Rejected

- A copy of the store in the child (the state before S4: two stores and two endpoints).
- Recomputing the outboard in the child from `read_range` bytes (the child reads the whole blob to serve any range).
- The resident serving Knot's evidence from its own node (changes evidence references and dial hints).

### 7. As built (2026-11-08)

- **Murm** (`merely-made/murm` `c6b31e70`, on main): `mere-transport` module `blob_source` (`BlobSource`,
  `BaoReader`, `StoreBlobSource`, interval helpers), `P2pandaTransportBuilder::scoped_source`, and `scoped_blobs`
  rewritten as the store-source wrapper. Tests: `blob_source` 1 new, and
  `a_source_serves_verified_streams_without_a_store_in_the_provider` (a provider with no store, a forwarding
  source, a 200 KB multi-leaf blob verified by the fetching peer, a leased-but-unretained hash refused, a
  non-reader refused, a source cut-off refusing without a stream request); the existing scoped-serving and
  revocation tests now run through the same provider.
- **Mere**, branch `agent/p2-door-serving` (not on main, see the plan's Findings, "P2 relock blocker"):
  `graphshell-door` feature `blob-serve` (`DoorBlobSource`, `BlobDoorClient::bao_open`, `bao_next`, `bao_close`,
  wire `BaoOpen`, `BaoClose`, `Bao`, `ChunkInterval`); `resident_blob_door.rs` (`BaoCursor`, `bao_chunk`, the
  per-chunk re-check); `tests/blob_door.rs` gains
  `an_application_without_the_store_serves_verified_streams_through_the_door`.

||||||| 24f8bbf5a

## The authority guest (cycle breaks P3, 2026-11-08)

Mark ruled "Build the real guest" after the spike (cycle breaks D15 (h)). This section records the
form as built. The measurements are in the plan's Findings, "P3 guest".

### 1. The world

`authority-guest` in the protocol layer's one WIT file (`graphshell-endpoint/wit/world.wit`, next to
`block-guest`): imports `log` only, exports `domain-authority` with `describe`, `validate(list<u8>)`
and `admit(list<u8>, list<u8>)`. The claim is `{domain, version, graph-format, admission}`; the one
addition to the drawing board's proposal is `admission`, which says whether `admit` is a real check so
the host never calls it, and never pays to encode before and after, when it is not (divination has
none). The WIT names no Mere type: the world is the interface, the format is a tag.

**The candidate format** is `mere.graph-snapshot-and-facets/json`: one JSON object `{snapshot, facets}`,
the kernel's `Graph::to_snapshot_at(0)` and `Graph::facets()`; the guest restores it with
`try_from_snapshot` and `install_facets`. A snapshot alone is not a complete candidate. A guest that
claims another format is refused at review. The snapshot's serde shape belongs to the kernel at the
guest's pinned Mere revision: a resident whose kernel changed the shape sends candidates an older guest
cannot decode, and the guest refuses them. A change to that shape therefore needs a new format tag.

### 2. The host

`crates/script/authority-host` (wasmtime 45; the sibling of `block-host`) runs one component:

- **A fresh instance per call**, from the once-compiled, pre-linked component. A fuel cut-off or a panic
  traps the instance and a trapped instance cannot be re-entered, and an instance that kept state
  could let one candidate shape the next verdict; instantiating costs under a millisecond, so the host
  re-instantiates by never reusing. A trap is reported as a refusal. An authority never fails open.
- **A budget per call** (`Budget::default()`): fuel 2e10, memory 128 MiB, candidate 16 MiB. They are the
  spike's corpus maxima (1.74e9 fuel, 6.6 MiB, 296 KB) with headroom: 11, 19 and 54 times. Fuel and not
  a wall clock, because fuel is the same on every device.
- **A closed WASI.** Rust's standard library on `wasm32-wasip2` imports WASI, so "log only" is a
  statement about the world, not about the binary. The host links WASI with no preopens, environment,
  arguments, network or stdio, wall and monotonic clocks fixed at zero, and fixed-seed random, so the
  guest can read neither the time nor entropy and hashed-collection order is the same everywhere.
- **Compiled once per load, never deserialised from disk** (`Component::deserialize` is `unsafe` and
  trusts its bytes, so a stored artifact would let anyone who can write the data directory bypass the
  pinned hash). About a second per process.

### 3. Review, pin and pending (`mere_resident::authority_guest`)

The pack is the D12 signed pack and asks for `power:domain-authority` and nothing else. The review
(`AuthorityReview::prepare`) verifies the pack's address and signature, that it holds exactly one
component matching its inventory, loads the component and reads its claim; it installs nothing. The
owner's `approve` binds the **author's key to the domain** and **pins the component hash**, writing
`<shared root>/authority-guests/<persona>/<domain>/{pin.json, pack.json, component.wasm}`, the pin
last. A later review for the domain must come from the bound key or the owner must rebind on purpose
(`rebind`); each new component is a new review. Every load re-checks the signature, the key against
the pin, the inventory, the hash against the pin and the claim's domain and format.

`DomainRegistry::resident_defaults(persona, store)` is Mere's defaults plus one `Authority::Guest`
registration per pin, and `MereRoutes::with_resident_defaults` builds it from the store under its
shared root, so every composition (daemon, embedded, an application's) picks up approved guests
without naming an application. A guest cannot take a domain Mere registers itself.

**Pending while absent.** A pinned domain whose guest is revoked (`GuestStore::revoke`), missing,
altered, signed by another key or unloadable is still registered and refuses every candidate with
`pending: no authority installed for <domain> (<reason>)`. A domain with no pin that is reserved
(`divination`) is refused at `serve` as `domain-not-registered`, as P1b specified.

### 4. Not built

- **The pin across paired devices.** `Pin::compare` states the agreement (agree, component differs,
  author differs); nothing carries a pin over the device roster yet, so two devices agree only if
  their owners approved the same pack.
- **A review surface.** `AuthorityReview` carries what a review shows (domain, version, format, author
  key, component address, size, the one power asked); no dialog presents it. Tests approve in code.
- **The reserved-domain string.** `mere_defaults` still reserves the id `divination`, a name for data
  and no dependency, so an unreviewed machine refuses the domain instead of serving it unchecked.
  Flipping `Unregistered::Refuse` (P1b's switch) would retire the string.
