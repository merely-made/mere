# Cycle breaks plan

**Status:** 2026-11-06: in progress. D1 to D5 settle the break for each open
family dependency cycle; the Genet/Murm cycle needs no break (Findings). Two forks remain
open (F1, F2). D6 (dev edges count) added the same day. D7 to D9 (Mark, 2026-11-06) ruled
the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md) and
unpaused D4 and P6: the contract leaves and the host-role kit left Mere for the revived
`merely-made/graphshell` the same day (Progress, Findings); `mere-comms` into Murm is still
to do, and F1 is half answered. D14 (Mark, 2026-11-06) added P1b; D15 answered the distribution binary (authority as guest or data, Knot's lane supervised after a blob door route). D2 landed as P4 and D3 as P5 (2026-11-06, Progress). D17 (Mark, 2026-11-06) closed F1 (`backdrops` to Graphshell) and P6 is done. D1 proceeds; D5 became D12 (Woodshed as a guest, design written, held for review). D26 (Mark, 2026-11-08) made Dramatis a producer and retired its window; D18 to D23 (Mark, 2026-11-08) ruled the P0 check's unlisted cycles: the resident seam binds wherever the resident lives, `inker -> mere` is allowlisted until engine-split P5, `murm-wire` and `murm-rtc` moved into Genet (D20, done), Dramatis's app may become a producer (tentative, still flagged), the viewer's one build, and the real Cleromancy guest.

**For:** breaking the family's git-dependency cycles. Mark, 2026-10-28: "what's important is
to untangle the repos and prevent cycles, dogfooding, and not drawing inefficient
boundaries" (source homes map, `mere_docs/technical_architecture/2026-10-28_source_homes_map.md`,
"Costs every move carries"). Doctrine Stack 3 (`../../doctrine.md`), as worded 2026-11-06: "Mere consumes
producers, never the reverse" (engine-split D1 reworded it on 2026-11-07 to "the host
consumes producers, never the reverse"). Until now the cycles were acknowledged as transitional and
paid for in two-phase repins (`../../2026-08-12_family_composition_thesis_brief.md`, the Knot,
Cleromancy and Murm repin sections); this plan removes the back edges instead.

Not here: the `branch = "main"` conversion of family rows (another lane); the resident split
(`2026-11-05_resident_split_plan.md`) and domain adoption (`2026-11-05_domain_adoption_plan.md`),
whose slices this plan cites but does not reschedule; publication of the producers' graph
cores (Stack 3 says it is still owed).

Receipts go to `archive_docs/receipts/2026-11-06_cycle_breaks_plan_receipts.md` (DOC_POLICY §8).

## Constraints

- **Family rows use the canonical URL** `https://github.com/merely-made/<repo>.git` with
  `branch = "main"`. The lock pins the tested revision. No `rev =` pins. A break lands its
  rows in that form; it does not add a pin to get around a window.
- **Cut first, not lockstep** (distillery-extraction D2): the receiving repository takes no
  `mere.git` dependency, and the donor deletes its copy only after the receiver builds and
  tests the moved code (compare blobs first, as the resident split plan's slices do).
- **Moved code keeps its tests.** A move carries the unit and integration tests that
  exercised the code; a test that needs the donor's graph stays with the donor as a consumer
  test of the receiver.
- **No cargo manifest churn outside a phase.** Each phase names the manifests it edits;
  repins follow the family repin loop (supplier first, one unused patch per root at most).
- **Docs live with the owner of the subject** (DOC_POLICY §4). Each phase updates the
  receiver's and donor's plans and the source homes map in the same session.

### Standard done-conditions

Every phase below ends on these four, then its own:

1. **The cycle is gone from every lock root.** For each lock root of the two repositories
   (`find . -name Cargo.lock` outside fixtures, receipts and `support/patches`), the resolved
   graph, `cargo metadata --format-version 1 --locked` with all features, has no package of
   repository A that depends on a package of repository B while a package of B depends on a
   package of A. The P0 check (`scripts/check_family_cycles.py`) reports it and the
   decision's allowlist entry is deleted in the same commit.
2. **Consumers still build and test.** Locked `cargo check` and the targeted tests of every
   actual consumer of the moved item, in the repositories that consume it.
3. **No new cycle.** The P0 check over the whole family passes with the allowlist one entry
   shorter, and the receiver's lock holds no `mere.git` source unless the phase says so.
4. **The moved code keeps its tests,** counted before and after, and the donor's gate passes
   without them.

## Phases

### P0. The cycle check

Design in "Keeping cycles out" below. Done when: `scripts/check_family_cycles.py` and its
tests exist; the check runs in the local gate; its allowlist names exactly the cycles open at
that date (C1 to C6 below, plus the permanent isolated Genet/Murm entry) with this plan cited
per entry; a synthetic two-repository fixture proves it fails on a new cycle and on a stale
allowlist entry.

**Built 2026-11-08** (see Progress and "The P0 result"); it is not yet wired into a gate (no
gate script exists in Mere to carry it), and its allowlist holds only the ruled, still-open D1
back edges and the D17 isolated root, because C3, C5 and C6 are already broken and the
checker found cycles this plan did not list.

### P1. D1 seam design (before any code moves)

**Partly superseded by D15 (2026-11-06):** the `Lane`, `LaneFactory`, `LaneContext` and `LaneGate`
parts of the domain contract's registration section and its distribution-binary section are
superseded. The registry, authority, M2 refusal, fixtures and layer analysis stand.

The registration contract between the resident and a domain, written and reviewed first.
Done when a dated section in `../technical_architecture/2026-11-06_domain_contract.md` (or a
sibling record it links) states:

- the registration contract: what a domain hands the resident at runtime and what the resident
  hands back, built on the Stickleback domain interface in Eidetic (domain contract C5) and the
  registry of the adoption plan's M1;
- what the resident needs from a domain, by name, from the current uses: Knot's `peers` sync
  host, relation intake and the `mere-overlay` test (`ports/mere-resident/src/resident_knot.rs`,
  `knot_relation_intake.rs`, `resident_mere.rs`, `tests/knot_*.rs`), and Cleromancy's
  divination validator (`with_divination_authority` in `resident_mere.rs` and
  `src/bin/c1_receipt_host.rs`);
- how each existing use maps onto the contract, and which are not domain behaviour at all
  (then they leave the resident or the app owns them);
- where each app's registration call lives (in the app's own binary or host crate), so no
  Mere `Cargo.toml` names Knot or Cleromancy;
- what the shared dev and test fixtures become (the 1,370-line `embedded_reservoir_validation`
  test and the five Knot tests move to the app repositories as consumers of the resident).

Reviewed by Mark before P2 starts. Written 2026-11-06; Mark answered it in D14, which leaves the
distribution-binary question open.

### P1b. The domain registry (M1 and M2), Mere's own domains only

Added by D14 (b). Before P2 and P3 both touch `resident_mere.rs`, `resident.rs` and `djinn.rs`,
build the registry the P1 section of the domain contract specifies: `DomainRegistration` and
`DomainRegistry` in Mere, `MereRoutes` and `Resident::open` reading it, the composition body of
`bin/djinn.rs` as a library entry taking a registry, and `mere_defaults()` installing Mere's own
domains (the personal replica, `personal-facts`). No Knot or Cleromancy change. Done when: both
compositions serve their domains from the registry, a second (fixture) domain is added without
editing either composition (M1), a product domain without authority is refused with the typed
error and the hard-coded `"divination" | "personal-facts"` list is gone (M2), the conformance
tests of the domain contract's section 6 pass over a fixture graph domain and a fixture lane,
and the standard conditions hold. D15 settled the authority forms: `Authority::{Graph closure, Guest,
Declared}`, the last two typed and refusing as "pending: no authority installed" until P2 and P3
implement them; no `Lane`.

**Interim (landed):** the registry's default for an unregistered, unreserved domain is
`Unregistered::ServeGeneric` (served under generic graph authority, as before), and only reserved
domains (`divination`) are refused with the typed `domain-not-registered` error when not
registered. Strict refusal of every unregistered domain, `Unregistered::Refuse`, is one switch;
flip it when the applications register, which flips the assertion of the
`unvalidated-control` case in `embedded_reservoir_validation.rs`.

### P2a. A door route to the resident's blob custody

Added by D15 (i): Knot's lane is not split off until this exists. Today the resident lends one
iroh blob store to personal-graph transfers and Knot evidence in process
(`resident_blobs.rs`: `ResidentBlobCustody::authorizer`, `bind_scope`, `migrate_legacy_store`,
`BlobLease`). Done when an admitted application can hold a scoped lease on that store through the
app door (route, grant and revocation shape stated first in the domain contract and reviewed),
with the lease's readers, scope binding and legacy-store migration behaving as the in-process
calls do; the in-process callers (personal replica) are unchanged; the resident split plan's S4
(the file moves to Eidetic) is sequenced against it, and decision (f)'s lease-name
parameterisation lands first. Tests keep their counts and the door route has its own.

**Status 2026-11-08: landed** (Progress; domain contract, "The blob door route").

### P2. D1 Mere stops naming Knot (reshaped by D14 and D15)

**Status 2026-11-08 (last): done for the resident; one edge remains, open.** `ports/mere-resident` names no Knot crate
(Mere `cacf7094f`, Knot `fc8ab8e41`); the supervised host, its door-backed evidence, the multi-process test and the
moves are built (Progress). `scripts/check_family_cycles.py --remote` exits 0. **Not done:** the allowlist entry
`mere -> knot-editor` stays, because `ports/graphshell`'s optional `knot-document` row (`editable-blocks`, `block_editing.rs`)
still names Knot; it leaves when D23 moves that port to Turnstone (Findings, "P2 remainder").

**Status 2026-11-08 (later, superseded above): step (1) done; step (2) settled by D24 (a pending slot until M4; nothing to build);
step (3) ruled by D25 and half built: Murm's source trait is on main, the door's verified read and `DoorBlobSource`
are built and tested on Mere branch `agent/p2-door-serving`, held for a relock blocker (Findings, "P2 relock
blocker"); Knot's side (door-backed evidence host, the supervisor, the multi-process test) and step (4) are not done.** The brief's repository name `merely-made/knot` does
not exist on GitHub; Knot is `merely-made/knot-editor`, which is also what Mere's manifests and lock name.

In this order. **(1)** P2a. **(2)** Knot's `DeclaredPolicy` (its `OperationPolicy`, a data value,
in Knot) registered as `Authority::Declared`, which P1b leaves as a typed variant that refuses
with "pending: no authority installed"; implementing the interpreter in Stickleback is part of
this step. **(3)** The supervised sync host: Knot's `knot_sync_host` (and `knot_endpoint` for the
route) run as an application the resident keeps running while the vault is unlocked, over the
door's custody and blob routes; a generic owner setting names the application; on lock the
resident refuses keys and ends the child; the lifecycle event names (`knot-closed`,
`knot-reopened`) are kept. **(4)** Drop the rows. Done when `ports/mere-resident` and every Mere
manifest drop `knot-editor` and `knot-document` (`Cargo.toml` rows, the `[patch]` redirect
section for `mere.git` packages Knot names, the `peers` and `mere-overlay` features), compared by
the resolved graph and not by the manifest comment; `resident_knot.rs`, `OwnerSettings.knot`
(read as the supervised-app setting for one release, since owner settings are on users' disks)
and the Knot tests listed in the domain contract move to Knot as consumers or are replaced by
fixtures; Knot's lock names Mere but Mere's resolves without `knot-editor`; the standard
conditions hold. No `Lane` trait and no distribution binary.

### P3. D1 Mere stops naming Cleromancy (reshaped by D14 and D15)

**Status 2026-11-08: steps (1) to (3) and (4) done on the Mere side; Cleromancy's own root lock could not follow
(Findings, "P3 guest").** Mark ruled "Build the real guest" (D23). The `domain-authority` world is in the protocol
layer (`authority-guest`), the resident hosts it (`authority-host`, `authority_guest`), `mere-resident` names no
Cleromancy crate, the `mere -> cleromancy` allowlist edge is gone, and the guest conforms: 577 of 577 corpus verdicts
and refusal texts equal native, and the seven multi-process owner tests plus the four natal tests pass through it. Held:
Cleromancy's root lock cannot take the new Mere (webcue and wgpu-cue are moving; Findings), so its conformance
tests wait on a branch.

In this order. **(1)** A validator-only crate split in Cleromancy: the authority's closure leaves
`muniment/redb` and the other native-only dependencies (the locked checks on
`wasm32-wasip2` and `wasm32-unknown-unknown` failed in `redb 2.6.4` and `errno 0.3.14`).
**(2)** The wasm feasibility spike: it builds as a component, its size against the 16 MiB cap, a
verdict-equal run native versus wasm over the corpus of the seven tests that move from
`embedded_reservoir_validation`, and per-call time on a realistic graph. If it fails, the
supervised native validator process is the fallback (D15 (h)). **(3)** The guest: the
`domain-authority` WIT world in the protocol layer, the resident's wasmtime host,
`Authority::Guest`, review binding the author's key to the domain and pinning the component hash
across paired devices, operations pending while no authority is installed. **(4)** Drop the
dependency. Done when `mere-resident` drops `cleromancy-core` (renamed `cleromancy-domain`);
`with_divination_authority` is gone and Mere registers no divination authority; `embedded_reservoir_validation`'s
product half (1 + 6 tests) runs in Cleromancy; Cleromancy's `core/Cargo.toml` stays an app-side
manifest; the standard conditions hold. P3 needs P1b; steps (1) and (2) can run beside P2.

### P4. D2 ESP adapter leaves Eidetic

`eidetic-search` keeps only the `GoalEmbedder` trait. Done when: `goal_model_esp.rs` (92 lines,
`#[cfg(any(feature = "bert", feature = "recall-example"))]` at `lib.rs:56-57`), the MiniLM
calibration tests and the `eidetic-recall` example move to ESP (Distillery), which already
depends on `eidetic-search` optionally; `eidetic-search`'s `bert` and `recall-example`
features, its optional and dev `esp` dependencies, and the workspace `esp` row leave Eidetic;
Eidetic's lock holds no `esp`; ESP's `persistence` feature (`dep:eidetic`) no longer needs a
second Eidetic source in either lock; the lexical fixture that used `esp` as a dev-dependency
is rewritten against the trait or moves with the tests; the moved tests run in ESP with the
same counts; Gemot's `work-host` edge to `eidetic-search` is checked for a new Gemot to
Distillery path.

### P5. D3 Murm stops depending on gemot-address

Done when `murm-resolve` defines the address trait (the four uses in `src/lib.rs`: the
`KnotAddress` host enum and parsing it needs) and carries no `gemot-address` dependency;
Gemot implements the trait (in `gemot-site-host` or beside `gemot-address`, the owner
chooses at implementation, since Gemot already depends on Murm); `murm-resolve`'s public
signature change is carried through its consumers; Murm's lock no longer names Gemot. If
Murm still reaches Gemot through another edge, the check names it and the phase is not done.

**Landed 2026-11-06** (Progress; Findings, P5 and P6 findings).

### P6. D4 mere-comms moves into Murm

**Paused 2026-11-06 (Mark) pending the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md); unpaused the same day by D7 to D9.** The scene vocabulary (`sceno`, `scenotime`) now lives in `merely-made/graphshell`, below Murm, so Signalman's `sceno` edge is legal. What is left of P6 is `mere-comms` and F1's `scenomise` edge.

Done when `crates/shell/comms` (package `mere-comms`, lib `comms`; `serde`, `async-trait`,
optional `murm` and `misfin`, dev `identity`, `transport`) lives in Murm; Mere's `ports/moot`
and `ports/graphshell` (`comms` feature) consume it from `murm.git`; Murm's `signalman` takes
`comms` from its own workspace. **Signalman's `portable-scenes` feature still names Mere
`sceno` and `scenomise`, so the cycle is not broken until F1 is closed;** P6 is done when
F1 is closed and the standard conditions hold with Murm's resolved graph free of `mere.git`.

**Done 2026-11-06.** `mere-comms` landed first; F1 closed by D17 (`sceno-backdrops`), and Murm's root lock holds no Mere package. The one exception is `apps/signalman-desktop`, its own isolated root (Findings, D17 findings).

### P7. D5 and D12: Woodshed becomes a guest of Graphshell's block pane

**Reshaped 2026-11-06 by D12** (Mark: "D: Woodshed as a guest"). D5's first wording moved the
two adapters into Woodshed behind its features. A read of the code showed that cannot be done
by moving files: `ports/graphshell/src/block_editing/bars.rs` (741 lines, 8 tests) is a data
adapter, but `ports/graphshell/src/web_tree/blocks.rs` (1,315 lines, no tests) is the whole
block pane, wired into the private web page state (`TreePage`, `AppCtx`, `Logic`, `Child`,
`ProbeSnapshot`) at nine call sites in `web_tree.rs`, `controls.rs` and `lane.rs`, and
`graphshell-web` is a standalone cdylib with no crate above it to register anything. Woodshed
therefore leaves as a runtime guest. The design is
[guest_blocks](../technical_architecture/2026-11-06_guest_blocks.md); **nothing moves until Mark
has reviewed it**, as with P1.

**The seam, stated first.** A guest block adapter is a typed request/response component, the
WIT interface `block-adapter` in a new world `block-guest`. It receives the block's Djot class,
body text, attribute map and the selected rows plus the pattern text; it returns choices as
copied `change` records (the value shape of `block_editing::Change`, not that type), the
complete desired attribute map, and a view as a `sceno` scene with accessible regions. On the
host side Mere gets a Rust trait `block_editing::BlockAdapter` that the built-in rhyme adapter
and the guest wrapper both implement, and the pane looks adapters up by claimed class. No Mere
type moves down, and no Woodshed type is named by any Mere manifest.

Done when (amended by D12):

- the design is reviewed by Mark and its decisions answered;
- `block_editing` carries the `BlockAdapter` trait and the pane (`web_tree/blocks.rs`, kept in
  Mere and generalised) dispatches through it, with rhyme as the built-in implementation;
- the `block-adapter` interface is in the one WIT world if Mark rules it a protocol item (the
  design's first decision), or Mere-local if he rules otherwise;
- Mere's applet host loads a reviewed, signed component pack as a block adapter (a new
  `power:block-adapter` grant in `capsule_applet`);
- `bars.rs` and the notation-view portion of `blocks.rs` live in Woodshed (D13): the adapter
  logic as `woodshedding`'s feature `blocks`, the view and the component entry as
  `woodshed-notation`'s feature `block-guest`, built as a signed wasm component pack; the 8
  `bars` tests count 8 before and 8 after (5 move unchanged, 3 rewritten against the adapter's own
  types) plus one conformance test in Woodshed that loads the real pack through Mere's host;
- Mere has a host-neutral `block_pane`, a `crates/script/block-host` for the `block-guest` world
  and a native `graphshell-blocks` shell that loads the same pack under the same review (D13,
  ruling 6);
- the guest-blocks open questions R1 to R3 are answered (D16);
- `editable-blocks` stays as the generic pane's feature but loses `dep:woodshedding`; the
  `woodshedding` row in `ports/graphshell/Cargo.toml` and the `woodshed-notation` row and its
  `product` entry in `ports/graphshell/web/Cargo.toml` are gone;
- `ports/graphshell/web/Cargo.lock` and Mere's main lock hold no Woodshed package, and no Mere
  test, dev-dependency or committed fixture names a Woodshed package or contains a Woodshed pack
  (D6);
- the web build (`ports/graphshell/web/scripts/build-viewer.py`, with its delivery budget)
  still builds and its digest is checked the way the repo does;
- the standard conditions hold.

### P8. Allowlist closed

Done when the P0 allowlist holds only the isolated Genet/Murm entry, and the source homes
map's "Repository cycles" bullet is rewritten from "already work this way" to what remains.

## Open forks

- **F1. Signalman's Mere scene crates. Closed 2026-11-06 by D17.** The measurement and options
  stay in Findings (P5 and P6 findings); Mark chose option 1, `backdrops` to Graphshell, which amends
  D9. `signalman-desktop`'s separate root is the one allowlisted isolated exception.
- **F2. Gemot's dev edge into Distillery.** `distillery` takes `gemot`, `mien` and `mooting`
  as dev-dependencies, and `gemot` takes `eidetic-search` optionally. After P4 the loop
  Gemot, Eidetic, ESP, Eidetic is gone; whether a Gemot to Distillery dev edge is a cycle the
  check should refuse is a decision for the check's rule (dev edges), put to Mark when P0
  runs. The rule is ruled by D6. Judged under it at P4 (2026-11-06): there is no Gemot to
  Distillery edge to refuse. No Gemot manifest names `distillery` or `esp`, `work-host`'s
  only route was `eidetic-search`'s optional `esp` (now gone from Eidetic main), and none of
  Gemot's five locks holds a `distillery.git` or `esp` package. Distillery to Gemot (dev) is
  the one direction left, which is legal. Closed.

## Decisions

- **D1. Mere↔Knot and Mere↔Cleromancy: the resident stops depending (Mark, 2026-11-06):**
  "Resident stops depending". The resident hosts domains through a seam only: domain
  contract C5 puts the domain interface in Stickleback, in Eidetic, so domains depend on
  Stickleback and not on Mere's graph crates, and the apps register themselves at runtime, so
  Mere never names Knot or Cleromancy in Cargo. Put to Mark on the cycle map's C1 and C2,
  against keeping the two-phase repin loop and against moving the domain crates to a lower
  repository. Outcome: applies Stack 3 ("Mere consumes producers, never the reverse") and
  refines the source homes map's "Mere↔Knot and Mere↔Cleromancy already work this way".

  Links: area stack · draws on domain-contract C5, stack.3 · amends source-homes-map "Repository cycles" · plans cycle-breaks P1, P2, P3 · summary The resident hosts domains through a Stickleback seam and apps register at runtime, so Mere's Cargo names neither Knot nor Cleromancy.

- **D2. Eidetic↔Distillery: the ESP adapter moves to ESP (Mark, 2026-11-06):** "Move adapter
  to ESP". `eidetic-search` keeps only the `GoalEmbedder` trait; the ESP adapter
  (`goal_model_esp`), the MiniLM calibration tests and the recall example move to ESP; Eidetic
  stops depending on ESP. Put to Mark against leaving it and recording that `persistence`
  stays off in Eidetic's root. Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle-breaks P4 · summary eidetic-search keeps the GoalEmbedder trait; the ESP adapter, calibration tests and recall example live in ESP.

- **D3. gemot↔Murm: an address trait (Mark, 2026-11-06):** "gemot↔Murm: trait". An address
  trait in `murm-resolve`, implemented by Gemot, so Murm stops depending on `gemot-address`.
  Put to Mark against Murm duplicating Gemot's grammar, which conflicts with Gemot owning it.
  Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle-breaks P5 · summary murm-resolve owns an address trait that Gemot implements; Murm no longer names gemot-address.

- **D4. Mere↔Murm: mere-comms goes down (Mark, 2026-11-06):** "Mere↔Murm: mere-comms down".
  `mere-comms` moves into Murm, so signalman stops depending on Mere. Outcome: applies the
  resident split plan's item that the comms contract may stay while its concrete adapters
  belong to Murm and Errand, now with the whole crate moving. F1 records that this alone
  does not clear Signalman's scene crates.

  **Paused 2026-11-06 (Mark), pending the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md); unpaused by D7 to D9, which moved the scene vocabulary below Murm.**

  Links: area stack · draws on resident-split S6, stack.3 · plans cycle-breaks P6 · summary mere-comms moves into Murm so signalman's comms feature no longer names Mere.

- **D5. Mere↔Woodshed: adapters go up (Mark, 2026-11-06):** "Mere↔Woodshed: adapters up".
  The Woodshed adapters move out of Graphshell into Woodshed behind its own features, so Mere
  stops naming Woodshed. Outcome: applies the source homes map's reading of Pandect/Graphshell
  adapters as "adapters for other owners".

  Links: area stack · draws on stack.3 · plans cycle-breaks P7 · summary Woodshed's block-editing adapters move out of Graphshell into Woodshed behind its features; Mere no longer names Woodshed.

- **D6. Dev-only edges count (Mark, 2026-11-06):** "Count them". Dev-dependency edges are cycles in
  the family check: tests use fixtures, or live in the higher repository. Settles F2's check rule
  and the dev-only fixture in P0's tests (a dev-only cycle fails). Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle-breaks P0 · summary Dev-only dependency edges count as cycles in the family check; tests use fixtures or live in the higher repository.

- **D7. Extract the contract leaves first (Mark, 2026-11-06):** "D: hybrid, leaves first", on
  `../research/2026-11-06_host_guest_contract_brief.md`. The clean contract crates move to a
  bottom layer now; the full host/guest split is decided later. Unpauses D4/P6 once the leaves are
  out. Outcome: new.

  Links: area stack · draws on cycle-breaks D4 · plans cycle-breaks P6 · summary The host/guest contract's leaf crates are extracted to a bottom layer first; the full split waits.

- **D8. The bottom layer is the revived Graphshell repository, named for the protocol (Mark,
  2026-11-06):** "we can use it as the bottom layer. unarchive, hard cut, i'm ok with that" and
  "graphshell = the protocol". `mark-ik/graphshell` was unarchived and transferred to
  `merely-made/graphshell`; it is cut over to hold only the contract, with no transitional copy
  left in Mere. The Graphshell reference host (`ports/graphshell`, `graphshell-mere`,
  `graphshell-host`) stays above Mere. Outcome: new.

  Links: area stack · draws on cycle-breaks D7 · plans cycle-breaks P6 · summary The revived merely-made/graphshell repository is the bottom protocol layer; the reference host stays above Mere.

- **D9. Contract plus host-role kit (Mark, 2026-11-06):** "Contract + kit". The bottom layer holds
  `chirograph`, one WIT world with `script/wire` and `witgen`, `sceno`, `scenotime`, `hest` (and
  `mere-curation` if guests need it), plus `graphshell-endpoint`'s default surface and
  `graphshell-client`; the endpoint's `door`, `profile` and projection-provider features, and
  `scenomise`, stay in Mere. Outcome: new.

  Links: area stack · draws on cycle-breaks D8 · plans cycle-breaks P6 · summary The bottom Graphshell layer holds the contract crates and the host-role kit, not the runtime-bound features.

- **D10. The WIT world reaches Mere's hosts through `links` metadata, for now (Mark,
  2026-11-06):** "Accept for now". `graphshell-endpoint` declares `links` metadata for its one
  WIT world; `document-host` and `app-host` copy it into their git-ignored manifest-relative
  path in `build.rs`, and `settings-wit` reads it directly, so these crates carry an otherwise
  unused normal dependency on `graphshell-endpoint`. Revisit together with the open question
  of renaming `mere:script@0.1.0`. Outcome: new.

  Links: area stack · draws on cycle-breaks D9 · summary Mere's WIT hosts take the one world from graphshell-endpoint's links metadata until the package rename is decided.

- **D11. The extraction's structural choices stand (Mark, 2026-11-06):** "Keep all three".
  The endpoint's door features are the Mere crate `graphshell-door`, beside `graphshell-host`;
  the frozen-scene tests that need Mere crates are the Mere crate `graphshell-client-tests`;
  the AccessKit conversion is `uxtree::frozen::FrozenUxTree` behind `uxtree`'s `frozen`
  feature. Outcome: new.

  Links: area stack · draws on cycle-breaks D9 · summary graphshell-door, graphshell-client-tests and uxtree's frozen feature are the accepted homes for what the extraction left in Mere.

- **D14. P1's decisions answered; the distribution binary reopened (Mark, 2026-11-06):** on the
  [domain contract](../technical_architecture/2026-11-06_domain_contract.md) section 8.
  (b) accepted as recommended: P1b runs first (the M1 registry and M2, Mere's own domains only).
  (c) accepted as recommended: the registration type lives in Mere, with C5's trait in Eidetic and
  the presenting half in the protocol layer. (d) accepted as recommended: out-of-process authority
  is not built now. (e) accepted as recommended: the resident split plan's "concrete producer
  registration stays in Mere" is amended (done in that plan). (f) accepted as recommended: P2
  parameterises the lease name before S4 moves `resident_blobs.rs`. (a), where the installed
  resident lives, Mark answered: "Hmmmm. Let's go back to the drawing board." He rejects the
  three options as framed (a new distribution repository, one app repository, stock `djinn` only).
  **(a) is open.** The drawing-board section of the domain contract questions the premise that a
  domain's authority is Rust code linked into the store-owning process; P2 and P3 stay held on
  it. Outcome: new; amends P1, adds P1b.

  Links: area stack · draws on cycle-breaks D1, domain-contract C5 · amends cycle-breaks P1 · plans cycle-breaks P1, P1b, P2, P3 · summary P1's decisions b to f accepted and P1b added; where the installed resident carries Knot's and Cleromancy's authority is open and back at the drawing board.

- **D15. The drawing-board answers (Mark, 2026-11-06):** on the domain contract's drawing-board
  section. (g) "Yes, split them": authority runs as a guest or as declared data, Knot's live sync
  lane becomes a supervised application until M3 to M5 retire it, with no distribution binary and no
  `Lane` trait. This closes D14's open (a). (h) "Wasm guest after a spike", with the supervised
  native validator process as the fallback; on both sub-points, "Yes to both": an authority guest
  gets stronger review than D12's view guests (the author's key bound to the domain, the component
  hash pinned across paired devices), and operations wait as pending while no authority is
  installed. (i) "Wait for a door route": against the recommendation, Knot's lane is not split
  off until a door route to the resident's blob custody exists, so that route is a prerequisite
  phase (P2a). Supersedes the Lane and distribution parts of P1; reshapes P2 and P3. Outcome: new.

  Links: area stack · draws on cycle-breaks D1, D12, D14 · amends cycle-breaks P1, P2, P3 · plans cycle-breaks P1b, P2a, P2, P3 · summary Authority is a guest or declared data, Knot's lane a supervised app after a blob door route exists, no Lane trait and no distribution binary.

- **D12. Woodshed is a guest, not a compiled-in adapter (Mark, 2026-11-06):** "D: Woodshed as a
  guest". The generic block pane (Djot documents, rhyme schemes, save, undo, reopen) stays in
  Mere. Woodshed's part (progression and bars, the example, the Chart/Tab/Score notation view,
  MusicXML export) leaves Mere and is loaded at runtime as a reviewed guest component, so no Mere
  manifest or lock names Woodshed. Chosen over moving both files into Woodshed (option A, which
  needs the private web page state made public and a Woodshed-built product), moving only
  `bars.rs` (B, which leaves the edge) and dropping editable blocks (C, which loses the pane).
  Amends D5's means, not its end: Mere stops naming Woodshed. Outcome: new design, written first
  and held for Mark's review.

  Links: area stack · draws on cycle-breaks D5, D8, D9, host_guest_contract_brief · amends cycle-breaks D5 · plans cycle-breaks P7 · summary The block pane stays in Mere and Woodshed's progression and notation parts load as a reviewed wasm guest, so Mere names no Woodshed package.

- **D13. The guest-blocks decisions (Mark, 2026-11-06):** the eight decisions of
  [guest_blocks](../technical_architecture/2026-11-06_guest_blocks.md), answered as relayed:
  1. Interface: "Put `block-adapter` in the graphshell protocol WIT, as a new `block-guest` world in
     merely-made/graphshell."
  2. View: "A `sceno` scene plus regions, but run a spike first. If the spike falls short, come back
     to him before choosing differently." The spike was run the same day and fell short for painting
     (guest_blocks, Spike); the view is back with Mark and nothing different is chosen.
  3. Calling: "A Worker, asynchronous, with a memo and stale state."
  4. Discovery: "A Moot capsule that is reviewed once."
  5. Crate: "A feature of `woodshedding`, not a new crate." Not the design's recommendation. It holds
     for the adapter logic; the view and component entry go in a `woodshed-notation` feature,
     because `woodshed-notation` depends on `woodshedding` (guest_blocks R2).
  6. Native: "Build a native block pane now, over `app-host`." Not the design's recommendation.
     `app-host` binds only `app-core`, so the host is a sibling `crates/script/block-host`, and the
     pane is extracted from `web_tree` into a host-neutral `block_pane` (guest_blocks, Native).
  7. Tests: "Rewrite them against the guest's own types, and add one conformance test through
     Mere's host."
  8. Unclaimed class: "Disable the command, name the class, and add a 'Review block adapters'
     entry."
  Outcome: settles D12's design except the view's drawing (guest_blocks R1), the home of the view
  (R2) and the native shell (R3), which are put to Mark.

  Links: area stack · draws on cycle-breaks D12, guest_blocks · plans cycle-breaks P7 · summary Mark's eight guest-blocks answers: the protocol WIT carries block-guest, Woodshed's adapter is a woodshedding feature, a native block pane is built, and the painting question is back with Mark after the spike.

- **D16. Guest blocks: the three open questions answered (Mark, 2026-11-06):**
  [guest_blocks](../technical_architecture/2026-11-06_guest_blocks.md) R1 to R3.
  1. R1, the drawing: "Scene + vector sheet". The guest returns the `sceno` scene for regions plus a
     protocol-owned, versioned vector sheet (compound fills with a fill rule, strokes, theme-role
     paint and text runs), and the host paints it. Direct manipulation, if wanted later, gets its
     own design.
  2. R2, the view's home: "Split across both". `woodshedding` gets feature `blocks`;
     `woodshed-notation` gets feature `block-guest`.
  3. R3, the native shell: "New graphshell-blocks bin".
  Outcome: D13's design is complete; implementation follows its order of work.

  Links: area stack · draws on cycle-breaks D13, guest_blocks · plans cycle-breaks P7 · summary The guest returns a sceno scene for regions plus a protocol vector sheet the host paints; the adapter splits across woodshedding and woodshed-notation features; the native shell is a new graphshell-blocks bin.

- **D17. `backdrops` goes to Graphshell; F1 closed (Mark, 2026-11-06):** "backdrops → graphshell",
  option 1 of F1. `scenomise::backdrops` and `template` (1,517 lines, 12 tests) move into
  `merely-made/graphshell` as a contract helper, the crate `sceno-backdrops`. Amends D9, which had
  said `scenomise` stays in Mere: the stay applies to the rest of `scenomise`, not to this slice.
  Signalman's `portable-scenes` takes the crate from Graphshell and drops `scenomise`,
  `scenograph` and `mere-curation`, so Murm's root lock holds no Mere package; `apps/signalman-desktop`
  stays an isolated root on Mere and is the one allowlisted isolated Murm-to-Mere exception (this
  plan, D17). Outcome: refines D9, closes F1, completes P6.

  Links: area stack · draws on cycle-breaks D9, stack.3 · amends cycle-breaks D9 · plans cycle-breaks P6 · summary The backdrops and template modules move into Graphshell's sceno-backdrops crate, so Signalman's portable-scenes no longer names Mere; signalman-desktop stays the one isolated exception.

  *D8 and D9 refined 2026-11-07 (engine-split D2): the repository keeps the contract as its
  bottom root and also holds the hosting above it; see `2026-11-07_engine_split_plan.md`.
  D9 refined again the same day by engine-split D19: the contract root gains `tinct` and is a
  leaf layer Inker may name, with the drift toward "the family's leaves" named as a cost.
  D8 refined by engine-split D23: the repository is the host layer as a library, contract root
  below and hosting above, with no application; the reference host and MCP server go to
  Turnstone.*

- **D18. The resident seam applies wherever `mere-resident` lives (Mark, 2026-11-08):** "(a) Seam
  goes wherever it lives". The P2 and P3 registration seam (D1, D14, D15) is not tied to the
  repository that holds the resident. Whether `mere-resident` stays in Mere or moves to Graphshell
  with the engine split (engine-split D23), the resident names neither Knot nor Cleromancy in its
  manifests, and Knot and Cleromancy register themselves at runtime through the Stickleback domain
  seam. The allowlisted `mere -> knot-editor` and `mere -> cleromancy` edges stay allowed until P2
  and P3 land and, if the resident moves first, become `graphshell -> knot-editor` and
  `graphshell -> cleromancy`, closed by the same steps. Outcome: refines D1 (where the seam lives
  does not change what it rules).

  Links: area stack · draws on cycle-breaks D1, cycle-breaks D14, stack.3 · amends cycle-breaks D1 · plans cycle-breaks P2, P3 · summary The registration seam binds the resident wherever it lives, Mere or Graphshell, so neither repository's Cargo names Knot or Cleromancy.

- **D19. Inker↔Mere: allowlisted until P5 (Mark, 2026-11-08):** "Allowlist until P5". Inker's
  optional `fleece` row on Mere's `linked-data` (`mere-fleece -> mere-rdf`, engine-split F3) is the
  only edge from Inker to Mere. Engine-split F3 records that `mere-rdf` depends on `inker`, so Inker
  cannot name the engine without a cycle until engine-split P5 folds `linked-data` and the resource
  half of `graph-kernel` into one `mere` crate with no host dependency. `inker -> mere` is allowlisted
  in `support/family-cycles/allowed.toml` with this decision and leaves when P5 lands (the entry on
  no cycle fails the check, so P5 deletes it). No code moves now. Outcome: new; the check's second
  unlisted finding (The P0 result).

  Links: area stack · draws on cycle-breaks D6, stack.3 · plans engine-split P5 · summary inker to mere is allowlisted as a temporary back edge until engine-split P5 separates the engine from inker; the entry leaves with P5.

- **D20. The ring: `murm-wire` and `murm-rtc` move into Genet (Mark, 2026-11-08):** Mark asked:
  "Why wouldn't rtc just be a genet capability then? Given it's w3c, wpt… http(s) isn't in murm
  either, but it's a protocol too." He then ruled "Move both into Genet". `murm-wire` (HTTP/1-3 and
  WebSocket carriage, originally split out of Genet's Netfetcher with its history kept) and
  `murm-rtc` (the pinned WebRTC protocol leaf) become Genet capabilities, in `components/`
  beside `netfetcher` and `genet-webrtc`. Genet is their only consumer: no Murm crate outside
  `murm/wire/` names them, and no manifest outside Genet names them, so they are renamed
  `genet-wire` and `genet-rtc`. The `murm/wire` workspace leaves Murm. This removes the
  `genet -> murm` edge, which the P0 check found closing the Genet, Murm, Eidetic and Dramatis ring
  (The P0 result, item 3). Reverses the Findings' reading that the Genet/Murm pair needed no break.
  Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle-breaks P0 · summary murm-wire and murm-rtc move into Genet as genet-wire and genet-rtc, since Genet is their only consumer; the Genet to Murm edge and the four-repository ring go with them.

- **D21. Dramatis's application may become a producer (Mark, 2026-11-08; tentative):** "Maybe those
  lib's apps should be producers if hosts already depend on them either way." A tentative direction,
  not a final ruling: `apps/dramatis` may become a producer that hosts present, instead of an
  application that names Mere's Cambium (`cambium`, `cambium-desktop`, `mandrake`). A separate
  agent is writing the design. The Dramatis to Mere cycle (The P0 result, item 1) stays flagged by
  the check, with no allowlist entry, until the design is ruled. Outcome: direction recorded, no
  decision taken; nothing moves. *Final ruling: D26 (2026-11-08).*

  Links: area stack · draws on cycle-breaks D6, stack.3 · plans cycle-breaks P0 · summary Dramatis's app may become a producer that hosts present rather than an app naming Cambium; tentative, design pending, and the Dramatis to Mere cycle stays flagged.

- **D22. The viewer ships one build, `webgl2/` (Mark, 2026-11-08):** "Keep one build". Recorded in
  the Graphshell host plan (T7) and in the
  [viewer build audit brief](../research/2026-11-07_viewer_build_audit_brief.md), Rulings 3. The
  viewer ships `webgl2/` only; `webgpu/` is an opt-in comparison build. Two findings travel with
  the ruling: WebGL2 uses 1.4 to 1.85 times WebGPU's page CPU time per frame (21 nodes 5.0 against
  2.7 ms paused and 17.4 against 9.8 live; 200 nodes 11.0 against 6.8 and 36.1 against 25.6), so a
  slower device has less headroom than the measured Mac; and ANGLE on Metal draws two overlapping
  diamonds in one pictograph glyph in the other order, a 10-pixel difference in the compare
  images, accepted and not fixed. Outcome: records an engine and host decision here so the
  cycle-breaks numbering carries all of Mark's 2026-11-08 rulings.

  Links: area stack · draws on stack.3 · summary The viewer ships the single webgl2 build; WebGL2 costs 1.4 to 1.85 times WebGPU's page CPU per frame and ANGLE on Metal differs by 10 pixels in the compare images, both recorded in T7 and the audit brief.

- **D23. P3: build the real Cleromancy guest (Mark, 2026-11-08):** after the P3 spike (Findings,
  "P3 spike"), Mark ruled to build the real guest, step (3). Another agent is doing that work.
  Recorded here only; the plan's P3 steps stand. Outcome: settles the spike's open question for P3.

  Links: area stack · draws on cycle-breaks D1, cycle-breaks D14, stack.3 · plans cycle-breaks P3 · summary Mark ruled to build the real Cleromancy guest (P3 step 3); another agent is doing it.

- **D24. Declared authority stays a pending slot until M4 (Mark, 2026-11-08):** "(A) Pending slot until M4",
  on the P2 findings' step (2) question. `Authority::Declared` stays a typed slot that refuses as "pending: no
  authority installed"; no interpreter, no extension trait and no delivery path are built before M4 (the
  replicated `notes` mere), its first resident-side consumer. Knot's `KnotSyncPolicy` keeps running inside
  Knot's own sync host. P2 step (2) is therefore done by this ruling: the slot and its test
  (`guest_and_declared_validators_refuse_every_candidate_as_pending`) exist, and its doc comment says so.
  Outcome: settles the P2 findings' step (2) question as option A.

  Links: area stack · draws on cycle-breaks D1, cycle-breaks D15 · plans cycle-breaks P2 · summary Authority::Declared stays a typed pending slot until M4; no interpreter is built.

- **D25. Serving Knot's evidence from a supervised child: change the transport (Mark, 2026-11-08):**
  "(2) Transport change", on the P2 findings' step (3) question. Murm's scoped blob provider takes a source
  trait (`BlobSource`) instead of a store; the blob door route gains a verified read so a child that holds no store
  serves iroh-blobs-compatible verified streams sourced from the resident. Knot's evidence reference and dial
  hints are unchanged. Design in the domain contract, "Serving through the door".
  Outcome: settles the P2 findings' step (3) question as option 2.

  Links: area stack · draws on cycle-breaks D15 · plans cycle-breaks P2 · summary The scoped blob provider reads from a BlobSource and the blob door adds verified reads, so a supervised Knot host serves evidence it does not store.

- **D26. Dramatis becomes a projection producer and its window retires (Mark, 2026-11-08):** after the
  design note (shapes A to D and the isolated-root fallback), Mark ruled four things. Shape A only, no
  interim: Dramatis serves its roster, quarantine review, vault lock and passphrase change as titulus cards
  plus `ActionFormV1` actions from a contract-only crate; no `[[isolated]]` allowlist entry is added, and the
  check kept flagging `dramatis -> mere` until the window left. Pane only: Dramatis exists as a pane in
  hosts; there is no standalone `Dramatis.app` or launcher, so `apps/dramatis` and its packaging are removed.
  Card contract: if quarantine review did not fit today's cards, extend the contract by the smallest
  compatible bump; it fits (labelled values and badges carry the store, details, credential kinds and the
  blocked reason), so no contract change is carried and nothing waits on a publish. Stdio carrier: until
  the peer-authentication work on the stdio carrier lands, a custody surface does not cross an unauthenticated
  pipe; the surface runs in the host's process and the stdio child form waits. Built: Dramatis
  `crates/dramatis-surface` (published contract crates only), Mere `ports/graphshell/dramatis-pane` and its
  mount in `graphshell-mere`. Outcome: settles D21's tentative direction as shape A; removes the Dramatis
  window, the `cambium`, `cambium-desktop`, `mandrake` and `taproot` rows and the dev `luggage` row.

  Links: area stack · draws on stack.3 · summary Dramatis serves its roster and quarantine review as portable cards plus bounded action forms that hosts present as a pane, in process until the stdio carrier authenticates its peer; the window application is removed.

## Findings

Verified 2026-11-06 against a fresh clone of Mere at `4e2cba4d9` and the other repositories'
`main` manifests (a read of the cycle map `family_cycles_map.md`, which scanned 24 shallow
clones; its raw edges are not committed).

- **Cycle list, with the closing edge:**
  - C1 Mere↔Knot: down `ports/mere-resident/Cargo.toml:143` (`knot-editor`, feature `peers`; dev
    `:295`, feature `mere-overlay`) with root rows `Cargo.toml:283-284`; up
    `knot-editor/crates/knot-editor/Cargo.toml` (optional `mere-overmap`, `mere-view`,
    `mere-fetch`) and `apps/desktop`. Held acyclic at the package level only by the
    `[patch]` section near `Cargo.toml:650`.
  - C2 Mere↔Cleromancy: down `ports/mere-resident/Cargo.toml:117`
    (`cleromancy-domain = { package = "cleromancy-core", ... }`); up `cleromancy/core/Cargo.toml`
    (`mere`, `graphshell-client`, `pandect`, `chirograph`, `sceno`, `seiche`, `mere-facts`).
  - C3 Mere↔Woodshed: down `ports/graphshell/Cargo.toml:56,221` (`editable-blocks`,
    `woodshedding`) and `ports/graphshell/web/Cargo.toml:40,53` (`woodshed-notation`); up
    Woodshed's `Cargo.toml` (`mere-curation`, `pictograph`, `sceno`, `scenograph`).
  - C4 Mere↔Murm: down Mere rows for `murm`, `mere-transport`, `murm-sync`, `murm-ocapn`,
    `retinue`, `murm-webrtc` (`Cargo.toml:177,511-519,634`); up `murm/apps/signalman/Cargo.toml`
    (`comms = { package = "mere-comms", git = mere.git }` at `:49`, `sceno` and `scenomise`
    at `:61-62`).
  - C5 Gemot↔Murm: `murm/native/crates/resolve/Cargo.toml:17` (`gemot-address`) against
    `gemot/crates/gemot/Cargo.toml` (`murm-sync`, optional `murm`, `transport`).
  - C6 Eidetic↔Distillery: `eidetic/crates/eidetic-search/Cargo.toml` (optional and dev
    `esp`; `bert` and `recall-example` enable `esp/persistence`) against
    `distillery/crates/esp/Cargo.toml` (`dep:eidetic` under `persistence`, dev `eidetic`).
- **Genet↔Murm is isolated.** The Murm side is `apps/signalman-desktop`, its own `[workspace]`
  root excluded from Retinue's, so neither lock contains the other. The source homes map
  records that pattern as the way to keep a lock free of Mere. No break is planned; the P0
  allowlist carries it as permanent.
- **The latent `esp/persistence` edge.** `esp` enables `eidetic` only under `persistence`, and
  `eidetic-search`'s `bert` and `recall-example` features turn that on. Eidetic's own lock
  holds `esp` and no Git `eidetic`, so the loop is dormant in the default build; turning
  `bert` on inside Eidetic's workspace would pull a second Eidetic source and a type
  mismatch, as in the 2026-10-20 Knot window. D2 removes it for good.
- **Optional edges count.** A `Cargo.lock` records optional dependencies whether or not a
  feature is on, so Signalman's `comms` and `portable-scenes`, Graphshell's `editable-blocks`
  and Eidetic's `bert` all put the other repository in the lock. The check must therefore
  read locks (or all-feature metadata), not default-feature trees.
- **D4 alone does not clear Murm** (F1): `signalman`'s `portable-scenes` feature still names
  Mere `sceno` and `scenomise`, which the cycle map's "Mere↔Murm" summary lists but its option
  text for `comms` treats as a single edge.
- **Non-canonical rows found in the sources this plan touches** (not touched here; the
  `branch = "main"` lane owns the Mere side): `rev =` pins and a `www.github.com` host in
  `ports/graphshell/Cargo.toml:221` and `ports/graphshell/web/Cargo.toml:53` (Woodshed,
  `a0ae58f`), in `murm-resolve` (`gemot-address`, `cd795b0`), in `knot-editor`
  (`gemot-site`, `204ee03`) and in `cleromancy/core/Cargo.toml` (`eidetic` at `86e78dd`).
  The check should treat `www.github.com` and `.git` suffix variants as one repository.
- **Where the cycle map and the docs differ:** the map names `ports/graphshell/web_tree/blocks.rs`;
  the file is `ports/graphshell/src/web_tree/blocks.rs` (a private module of the `graphshell-web`
  cdylib, not of the `graphshell` library; see P7 and D12). The map says `mere-comms` has "tiny
  blast radius: `mere-moot` in 2 files plus Signalman"; the consumers are the `moot` port
  (`ports/moot/Cargo.toml:60`) and Graphshell's `comms` and `workspace-editor` features
  (`ports/graphshell/Cargo.toml:76,235`). The map calls C4's `comms` half "Open" in one place
  and the resident split plan has it as an adapter question; D4 closes both.

### P6 extraction findings (D7 to D9), 2026-11-06

Verified against Mere `49571db83`, the Graphshell repository and the consumers.

- **Moved:** `chirograph`, `hest`, `sceno`, `scenotime`, `script-wire`, `script-witgen`,
  `graphshell-endpoint` (default surface, carriers and the WIT projection session) and
  `graphshell-client`, with the one WIT world at
  `graphshell/crates/graphshell-endpoint/wit/world.wit`. `crates/script/wit/world.wit`
  and the endpoint's copy were byte-identical (`cmp`); the history of the former is the
  one carried.
- **Split, `graphshell-endpoint`:** the `door` module (`projection`, `resident_mere_proxy`,
  `projection_offer`, `projection_provider`, `profile`) and the four features that pulled
  Ostiary, Castellan, Dramatis, Insigne and Personae became Mere's `graphshell-door`
  (`crates/graphshell/graphshell-door`, features `projection-door`, `projection-offer`,
  `projection-provider`, `profile`, names unchanged, empty by default). Named for the
  existing `graphshell-host` and `graphshell-mcp` siblings, which are the other
  `graphshell-*` crates that stay in Mere above the protocol.
- **Edges that needed a seam** (a moved crate may depend on nothing in Mere, dev edges
  included, D6): (1) `graphshell-client`'s `accesskit` feature took Mere's `uxtree`, which
  takes `inker`; `FrozenScene::to_ux_tree` became `uxtree::frozen::FrozenUxTree` behind
  `uxtree`'s `frozen` feature. (2) `graphshell-client`'s dev edges to `scenomise`,
  `scenograph`, `incipit`, `uxtree`, `genet-scripted-dom` and `taproot`: 28 of the frozen
  module's 32 unit tests and the facet and shelfmark tests moved to Mere as
  `graphshell-client-tests` (the four `sceno`-only tests stayed; the shelfmark test builds
  its own small capture because the site fixture stays with `scenotime`). (3)
  `scenotime`'s dev edge to `backstep`: its history test moved to
  `backstep/tests/scene_trace_history.rs`.
- **WIT consumers inside Mere** (`document-host`, `app-host`, `settings-wit` and the five
  guests) read the world by manifest-relative path. The bindgen macros take only literal
  paths, so `document-host` and `app-host` materialise
  `crates/script/wit/world.wit` (ignored by git) in `build.rs` from `graphshell-endpoint`'s
  Cargo `links` metadata (`DEP_GRAPHSHELL_ENDPOINT_WIT`, which Cargo passes from normal
  dependencies only), and `settings-wit` reads the metadata directly. A build product of
  the pinned dependency, not a second copy; the moot capsule guest relies on the same path.
- **`mere-curation` stays.** No moved crate depends on it; its dependents are `scenograph`,
  `mere-view`, `pandect`, `pictograph` and `forme` in Mere, and Knot, Turnstone and Woodshed,
  which depend on Mere crates for other reasons.
- **Versions:** none bumped. `graphshell-endpoint` 0.0.3 on crates.io still carries the
  door features, so it needs a version bump before its next publication; Distillery's
  registry rows resolve to the Graphshell git source through Mere's `[patch.crates-io]`.
- **Order forced by the cycle:** Mere's lock names Knot and Cleromancy, whose old revisions
  ask `mere.git` for `graphshell-endpoint` with `projection-door`, so Knot and Cleromancy
  took the source change first and Mere relocked onto them (Progress).

### P4 findings, 2026-11-06

- **Moved to ESP** (`esp::embed::goal_model`, feature `goal-model`, Distillery `e33bbaf`):
  `goal_model_esp.rs` (92 lines; only its `use` lines and one doc link changed), the MiniLM
  calibration test `goal_model_minilm.rs` (501 lines) with its `goal_calibration` corpus, the
  lexical n-gram recall receipt `lexical_ngram_recall.rs` and its `measurements.rs` (the
  "lexical fixture that used `esp` as a dev-dependency" had no trait form: it drives ESP's own
  `LexicalEmbeddingProvider`, so it moved whole), and the `eidetic-recall` example (577 lines)
  with its tests. Blobs compared with the donor's `HEAD`: corpus, lexical receipt, example and
  its tests byte-identical; the others differ only in imports, doc paths, feature gates and
  the `--lane`/`-p` hints in comments (and rustfmt's trailing-comma rule in Distillery).
- **Seam.** `eidetic-search` keeps `GoalEmbedder` and the `ModelGoalAssessor` machinery. ESP gained
  an optional `eidetic-search` dependency (the plan said ESP "already depends on
  `eidetic-search` optionally"; it did not: `crates/distillery` did, ESP did not) and a dev
  dependency for the moved tests, plus features `goal-model` (`persistence`,
  `dep:eidetic-search`) and `recall-example` (`goal-model`, `bert`, `bert-wgpu`). The example's
  `fjall` store needs `muniment/fjall` as an ESP dev feature (Fjall comes with Muniment's git
  source, so Distillery's lock grew 159 lines and no manifest gained a patch row).
- **Consumer.** Only Graphshell's `goal-surface-model` named `EspGoalEmbedder`
  (`ports/graphshell/src/goal_surface/model_path.rs`); it now imports
  `esp::embed::goal_model` and the feature reads `["goal-surface", "dep:esp", "esp/goal-model",
  "esp/bert", "dep:sha2"]`. Its `esp` row was already
  `git = distillery.git, branch = "main"`. A grep over Mere, Distillery, Gemot, Graphshell, Murm,
  Cleromancy, Woodshed, Turnstone, Isocosm, Knot, Bole, Dramatis, Armillary, Errand,
  merelyllc.com, Genet and netrender found no other user of `goal_model_esp`, the `bert` or
  `recall-example` features of `eidetic-search`, or `EspGoalEmbedder`.
- **Locks.** Eidetic's lock is 879 packages down to 411 (468 removed, none added) and holds no `esp`
  (nor does `responsa` or `line-ceiling`). Distillery's root lock names one Eidetic source; its
  probe locks resolve their own single source and none holds `esp` through Eidetic. Mere's root
  and `ports/graphshell/web` locks were relocked with `cargo update -p eidetic -p esp`, one
  Eidetic source each, `esp` no longer lists `eidetic-search`'s `esp` edge, 1 unused patch in web
  (as before), none in the root.
- **Where the plan was loose.** "A `Cargo.lock` records optional dependencies" is true for the
  workspace members of a lock root, not for an optional dependency of a non-member that no
  feature enables: Gemot's and Distillery's locks of `eidetic-search` never listed `esp`.
  The latent loop therefore showed only in Eidetic's own lock, where `esp` is a member's dev
  dependency.
- **Calibration test, pre-existing failure.** On Eidetic `main` before the move,
  `calibrate_match_similarity` already fails: the sweep picks 290, the recorded constant is 220
  (`CALIBRATED_MILLI` and `MINILM_L6_V2_MATCH_SIMILARITY_MILLI`), best F1 0.810 at 280 to 290 for
  the `Or` rule. It fails identically in ESP. Probably the stop-word S1 polish moved the S1
  side of the sweep after 220 was chosen; the threshold was left alone, since changing it is a
  calibration decision (Eidetic memory gates plan, Progress 2026-11-06). Not chased.

### P5 and P6 findings (D3, D4), 2026-11-06

Verified against Murm `45b1cf47`, Gemot `df6da4a`, Mere `ad63714ec` (before this change).

- **P5, the trait.** `murm-resolve` (`native/crates/resolve/src/address.rs`) defines
  `ReleaseAddress` with the four things `Resolver` read from an address (`host`,
  `collection`, `pin`, `path`), the `AddressHost` enum and `MootCollection` (owned page), plus a
  plain `LogAddress` (key, path, optional pin) for a caller that already holds a key.
  `Resolver::resolve` takes `&(impl ReleaseAddress + ?Sized)`. The `knot://` parser, `Display`
  and `KnotAddress::collection` stay in `gemot-address`; nothing in Murm parses an address.
- **P5, where Gemot implements it: `gemot-site-host`, feature `murm-resolve`
  (`release_address::ReleaseAddressRef`, a borrowing newtype).** Neither the trait nor
  `KnotAddress` is local to a third crate, so the orphan rule allows the implementation only in
  `gemot-address` or on a wrapper. `gemot-address` is documented as having no carrier or storage
  dependency and is a standalone root under `gemot-site`; an optional `murm-resolve` there would put
  Murm's whole native graph into its lock and make the grammar crate name Murm. `gemot-site-host`
  already names Murm (`retinue`) and carries the `[patch]` table that graph needs.
  Cost: a caller wraps (`ReleaseAddressRef(&address)`), once per call.
- **P5, consumers.** The only user of `murm-resolve` is `murm-holder` (`publish = false`); a code
  search over `merely-made` found no other. `Holder::acquire` now builds a `LogAddress`; no
  behaviour change.
- **P5, counts.** `murm-resolve` tests 7 before, 7 after (default features; 9 with `--all-features`),
  `murm-holder` 8 and 8. The governed-form test built a `KnotAddress` from text, so it now uses a
  fixture implementation in Murm and runs again against the real parser in Gemot, where four new
  `gemot-site-host` tests (`tests/release_address.rs`, feature `murm-resolve`) cover the log form,
  the governed form and the resolver's two refusals.
- **P5, the lock.** `murm/native/Cargo.lock` lost `gemot-address` (6 lines, nothing else changed);
  no other Murm lock names a Gemot package. Gemot's `crates/site-host/Cargo.lock` gained
  `murm-resolve` and its graph (relocked onto Murm, Eidetic and Dramatis mains).
- **P6, mere-comms.** Moved to `native/crates/comms` (package `mere-comms`, lib `comms`, version 0.0.1
  unchanged), a member of Murm's native workspace beside `murm`, whose adapter it wraps; Signalman
  takes it by path from its own repository. The root workspace could not hold it without putting
  `murm`'s iroh graph into the default lock, because a workspace member locks every optional
  dependency; as a path dependency of the root, only `serde` and `async-trait` enter the root lock.
  History: `git filter-branch --index-filter` on a fresh Mere clone at `016927d79` (`--prune-empty`,
  merges kept, the pre-dissolve path `crates/graphshell/shell/domain/comms` mapped too), 128 commits;
  the exported tree equals the donor's (`69dbd66fc`). Tests: 28 default, 38 with all features and one
  doctest each, in Mere before and in Murm after. `misfin` is taken from Errand's git source
  (Mere reached it through a `[patch.crates-io]` row); `murm`, `transport` and `identity` are the
  native workspace's.
- **P6, Mere's side.** `ports/moot` and `ports/graphshell` (`comms` feature) take `mere-comms` from
  `murm.git` (`branch = "main"`); `crates/shell/comms` is deleted. The only Mere user of the `murm`
  crate was `mere-comms`'s optional adapter, so the root lock no longer holds `murm`; its workspace row
  and the two `[patch]` rows for it left with it (otherwise two unused patches). `ports/graphshell/web`
  relocked. `mere-moot` lib 25 passed; `graphshell --features workspace-editor --all-targets` clean and
  the three `comms` lib tests pass; `ports/graphshell/web` check clean. Murm: `signalman --features
  comms` all-targets check clean, 75 lib tests pass; `signalman-desktop --locked --all-targets` check
  clean.
- **P6, mere.git edges left in Murm's resolved graph (F1's measurement).** Only the `portable-scenes`
  feature and `signalman-desktop` still name Mere. `murm/native` and every other Murm lock: none.
  - `murm/Cargo.lock` (root): three packages from Mere at `18c0f24a`: `scenomise` 0.0.7,
    `scenograph` 0.0.6, `mere-curation` 0.1.1. One direct edge, `apps/signalman/Cargo.toml:62`
    (`scenomise`, optional); `scenograph` and `mere-curation` are `scenomise`'s dependencies. `sceno`
    comes from `graphshell.git` and is legal.
  - `apps/signalman-desktop/Cargo.lock` (its own root): 24 Mere packages (the Cambium desktop host:
    `cambium`, `cambium-desktop`, `cambium-a11y`, `-rootstock`, `-winit`, `document-canvas`,
    `illume`, `inker`, `mandrake`, `mere-fleece`, `mere-lanes`, `mere-surface-api`, `meristem`,
    `numen`, `seiche`, `sprigging`, `tabard`, `tabard-workshop`, `tinct`, `workbench`, `backstep`,
    and `scenograph`, `scenomise`, `mere-curation`). Eight direct rows: `cambium`,
    `cambium-desktop`, `sprigging`, `scenomise`, `seiche`, `tabard`, `tabard-workshop`, `mandrake`.
    Its sources use `cambium` (19 files), `tabard` and `tabard-workshop` (4 each), `seiche` (3),
    `mandrake` (2), `sprigging` (1). Its `sceno` and `scenomise` rows are named by no source file
    (`scenomise` is also reached through the other Mere crates).
  - **What `portable-scenes` uses of `scenomise`:** one file, `apps/signalman/src/survey/scene.rs`
    (203 lines, plus the 77-line `signalman-survey-scene` binary calling it), and only
    `scenomise::backdrops::{BACKDROP_SCENE_VERSION, RadioDevice, SceneFacts, SceneInput,
    TerrainCoverage}`. `backdrops` is 1,311 lines in `backdrops.rs` and five submodules, and
    imports only `crate::template` (206 lines, no further crate-internal edge), `sceno` and `serde`.
    Inside Mere, `scenomise::backdrops` is also used by `cambium/src/backdrop_scene.rs`,
    `pictograph` (`backdrop_surface.rs`, its tests, the `backdrop_receipt` example),
    `ports/graphshell` (`block_editing/rhyme.rs`, `web_tree/backdrop.rs`, `web_tree/blocks.rs`) and by
    `scenomise`'s own `document.rs`, `document/djot.rs` and `span_edit.rs`; its tests are 8 in
    `tests/backdrop_scenes.rs` and 4 in `tests/sequence_templates.rs`.
- **P6, mere↔murm: exactly which edges remain** (cycle proof, all lock roots, dev edges counted).
  Murm to Mere: in `murm/Cargo.lock`, `signalman` to `scenomise`; in `apps/signalman-desktop`, the
  eight rows above. Mere to Murm (legal direction): `mere-comms`, `mere-transport`, `murm-sync`,
  `murm-webrtc`, `retinue`, `murm-ocapn`, `murm-ocapn-transport`, in the Mere root,
  `ports/graphshell/web` and `testing/moot-ocapn-coop`. The cycle is therefore not broken, by one
  optional feature and one isolated root.
- **F1 options, not ruled** (each leaves Murm's root lock free of `mere.git` except where noted):
  1. *The `backdrops` slice moves into `merely-made/graphshell` as a contract helper.* Moves
     `backdrops` and `template` (1,517 lines, 12 tests) with `sceno` and `serde` as its only
     dependencies, so Graphshell gains no Mere edge. Signalman then takes it from Graphshell and drops
     `scenomise`, `scenograph` and `mere-curation`; the root Murm lock holds no Mere package.
     Cost: about ten Mere files change an import path, or `scenomise` re-exports the module for
     one release; D9 said `scenomise` stays in Mere, so this amends D9; Graphshell needs the module's
     version policy. `signalman-desktop` still names Mere (isolated root, allowlisted).
  2. *The feature moves out of Murm.* `survey/scene.rs` and its binary (280 lines) go to
     `signalman-desktop`, whose root already names Mere, or to a Mere-side consumer of Murm's survey
     types. Cost: the headless `signalman-survey-scene` binary and its scene export leave the root
     workspace, so scene export builds only in the desktop's root (a heavy Genet graph); a Mere-side
     home would need Mere to depend on the `signalman` application crate, a legal direction but a
     large one. Cheapest in lines, largest in who can build it.
  3. *The separate root is the accepted answer, and the feature takes the same isolation.* Same
     file move as option 2, to a root of its own beside `signalman-desktop`, and the allowlist keeps
     two Murm-to-Mere isolated entries instead of one. Cost: a second lock to maintain, no Mere code
     changes. A fourth route, publishing `scenomise`, `scenograph`, `mere-curation` (and `sceno`)
     per the Cambium publish plan so Murm names the registry, is the largest: four crates through
     publication, patched back to path or git inside Mere.

### D17 findings (F1 closed), 2026-11-06

Verified against Graphshell `2f771813`, Murm `525d948b`, Mere `794a88c77` (the code commit).

- **The crate: `sceno-backdrops`** (`graphshell/crates/sceno-backdrops`, 0.0.1, lib `sceno_backdrops`).
  Named for what it is: the backdrop scene recipes over `sceno`, beside `sceno` and `scenotime`, which
  Graphshell already names with the `sceno` prefix; `scenomise` is the Mere crate it leaves, and
  `scene-` would break the prefix. The crate root is the old `backdrops` module and `template` is a
  public submodule, so a path `scenomise::backdrops::X` becomes `sceno_backdrops::X` and
  `scenomise::template::X` becomes `sceno_backdrops::template::X`.
- **Dependencies:** `sceno`, `serde` and `serde_json` (the code parses disclosed JSON; `sceno` already
  depends on `serde_json`), `scenotime` as a dev dependency. Nothing from Mere, dev edges included.
- **History:** `git filter-branch --index-filter` on a fresh Mere clone at `e19f47de2`, 19 commits kept
  of 7,730; the exported blobs equal the donor's (`backdrops.rs`, `template.rs` and both test files
  compared by hash). Merged into Graphshell with unrelated histories. Cut first: Graphshell built and
  ran the 12 tests before Mere deleted its copy.
- **Two seams that needed a change.** `SceneInput::span_target` was an inherent method defined in
  `scenomise::span_edit`; it cannot be implemented outside the crate that owns `SceneInput`, so it and
  its `SpanTarget` result moved too (`sceno_backdrops::SpanTarget`; callers unchanged). `scenomise`'s
  document reader needs `strict_core_records`, which became `pub`. The moved tests read three recipe
  fixtures from `support/design-studies/backdrops/`; Graphshell holds copies (`tests/fixtures/`), Mere
  keeps its own for the `scenomise` document and span tests.
- **Mere's side, direct imports, no re-export.** `scenomise` no longer has `backdrops` or `template`
  modules (D8's hard cut: no transitional copy). Files that changed an import: `cambium`
  (`backdrop_scene.rs`), `pictograph` (`backdrop_surface.rs`, its tests, the `backdrop_receipt`
  example), `ports/graphshell` (`block_editing/bars.rs`, `block_editing/rhyme.rs`,
  `web_tree/backdrop.rs`, `web_tree/blocks.rs`), and `scenomise` itself (`document.rs`,
  `document/djot.rs`, `span_edit.rs`, two tests). Manifests: the workspace row, and `scenomise`,
  `cambium`, `pictograph` and `ports/graphshell` take `sceno-backdrops`. Another consumer outside Mere:
  Woodshed (its Progress entry).
- **Tests, before and after.** Moved: `backdrop_scenes` 8 and `sequence_templates` 4 (12), now in
  Graphshell (`cargo test -p sceno-backdrops`: 8 and 4). `scenomise` stays: lib 199 and 199,
  `scene_documents` 3 and 3, `span_edits` 3 and 3. `cambium --lib backdrop` 1 passed; `pictograph
  --all-features --lib backdrop` 12 passed; `graphshell --features workspace-editor,editable-blocks
  --lib`: 398 passed, 1 failed, the same failure on Mere main without this change
  (`forme_workspace::lifecycle_tests::owner_lifecycle_successful_actual_metadata_completion_keeps_unpaused_physics_parked`).
- **Murm.** `apps/signalman`: `portable-scenes = ["dep:sceno", "dep:sceno-backdrops"]`; the `scenomise`
  row is gone; `survey/scene.rs` imports from `sceno_backdrops`. `murm/Cargo.lock` lost `scenomise`,
  `scenograph`, `mere-curation` and `libm`; no `mere.git` source remains in it or in any other Murm lock
  except `apps/signalman-desktop/Cargo.lock`. `signalman --features comms,portable-scenes`: 77 lib
  tests pass, all-targets check clean; `signalman-desktop --locked --all-targets` clean.
- **Cycle proof, mere↔murm** (every lock root of both repositories, `cargo metadata --locked
  --all-features`, dev edges counted, direct and transitive, excluding only the declared isolated root
  `murm/apps/signalman-desktop`): Murm to Mere: none, in any root. Mere to Murm: `mere-comms`,
  `mere-transport`, `murm-sync`, `murm-webrtc`, `retinue`, `murm-ocapn`, `murm-ocapn-transport` in the Mere
  root, `ports/graphshell/web` and `testing/moot-ocapn-coop`. No reverse pair: the cycle is gone.
  Including the isolated root, Murm to Mere remains there only (eight direct rows, 24 packages).
  Skipped as before (stale lock or network; no `murm` or `mere-comms` in them): Mere `guest-settings`,
  `native-id-consumer`, `luggage-comparison`, Murm `fuzz`.
- **The allowlist entry that remains** (P0 allowlist, not built yet): Mere↔Murm, isolated root
  `murm/apps/signalman-desktop`, reason "its own `[workspace]` root, excluded from Murm's, so neither
  lock holds the other; the source homes map's pattern for keeping a lock free of Mere", cites
  cycle-breaks D17. It is the Genet/Murm entry's sibling and stays permanent, so P8's allowlist ends
  with two isolated entries, not one.

### P2 findings (2026-11-08)

**Step (2), Knot's `DeclaredPolicy` as `Authority::Declared`, has no consumer and no delivery path today.**
Evidence:
- `KnotSyncPolicy` (`knot-editor/crates/knot-editor/src/sync.rs:307-366`) is private to Knot and is called by Knot's own
  `KnotSyncHost` when an operation arrives. No code in `mere-resident` takes an `OperationPolicy`: the registry's
  `Authority::validator()` returns a `pandect::GraphValidator` (a function of a complete graph), and `Declared` returns
  only the pending refusal. Admission of Knot operations happens in the process that runs the sync host, which after
  step (3) is the supervised child, not the resident.
- The interpreter the plan names (in Stickleback) cannot be written over `OperationPolicy<E>` alone: Knot's checks
  read `operation.header.extensions.space_id`, `.encryption` and `.parents`, which are Knot's `KnotSyncExt`. A generic
  interpreter needs a trait over the extension (space, encryption profile, causal parents) that Knot implements.
- A connecting application cannot install authority (`domain_registry`; the contract's rule), and the registry is built
  by the process that owns the store. So "Knot registers its policy value" needs a delivery path that does not exist:
  an owner setting, an owner-installed file, or a reviewed pack like D12's.

Options: (A) leave `Declared` a typed pending slot and skip step (2) until M4 (the replicated `notes` mere is the first
resident-side consumer of an operation policy), since Knot's policy keeps running inside Knot's own host. (B) Build the
interpreter and the extension trait in Eidetic now and have Knot replace `KnotSyncPolicy` with a `DeclaredPolicy` value
it interprets itself: the consumer is Knot's host, the resident only carries the slot. (C) Deliver the value as an
owner setting (`declared_policies.<domain>`), interpreted by the resident, with nothing consuming it until M4. Recommend
A: it deletes nothing Mere needs deleted and builds nothing with no caller; B is the right shape when M4 arrives.

**Step (3), the supervised sync host, is blocked on *serving*, which the door route does not carry.** P2a gives an
application custody administration (leases, the serving authorizer's retain and readers) and reads. Knot's host builds its
iroh endpoint with `P2pandaTransport::builder.scoped_blobs(&blobs, scope, readers)`
(`knot-editor/crates/knot-editor/src/resident/operations.rs:131-135`): the endpoint peers dial answers blob requests from
a `BlobStore` in its own process under the authorizer. A supervised child cannot hold the resident's store (iroh's file
store is single-process), so peers could not fetch Knot's evidence from the child. Options: (1) the child keeps its own
blob store for serving (the drawing board's old (i)1, two iroh endpoints and two stores until M5); the door route is then
only the one-time copy-out and the resident-side custody of what it already holds. (2) Make the transport's
`scoped_blobs` take a source trait so a door-backed reader can serve: a Murm change, and the provider needs verified
streaming (outboard data) that `read_range` bytes do not give, so the child would recompute it. (3) The resident serves
Knot's evidence from its own node: peers must be told to fetch evidence from the resident's node id, a change to Knot's
evidence reference and dial hints (the key stays in the child). (4) Keep Knot's lane compiled into the resident until M4
to M5 absorb it (drawing board (i)3); D1 stays open for Knot. Evidence not gathered: how often Knot evidence is fetched
across devices (decides whether two stores cost anything). Lean: (3) if the evidence reference can carry a node id,
otherwise (4); (1) is the cheapest and is the option (i)2 turned down.

**Independent of both:** `knot_relation_intake.rs` (349 lines, 6 tests in 817), the two `knot_overlay_*` tests and the
Knot-as-guest fixtures can move to Knot as consumers (domain contract, section 4). They were not moved: Knot would take
`graph-kernel`, a dev edge to `mere-resident` and a relock against Mere first, and none of it advances the manifests until
(3) is answered.

### P2 relock blocker (2026-11-08)

Mere cannot take a Murm revision past `58d068a04d4` (the `wire/` removal) without moving Genet, and cannot move
Genet while it pins `webcue = "=0.1.2"`. Evidence: `cargo update -p mere-transport` fails with "no matching package
named `murm-wire`" required by `netfetcher` at Genet `d634ee0c`; moving `netfetcher` too takes every Genet package to
`8f13bb7e`, where `document-session-api` is `0.1.2` (the selected-text query), while crates.io `webcue 0.1.2` requires
`document-session-api = "=0.1.1"`, so the lock holds both (the `[patch.crates-io]` entry for it no longer applies)
and `inker` stops compiling (`document_session_api::CapabilityStatus` against `webcue`'s). It is the wire-removal
lane's relock to settle (a webcue release against `0.1.2`, or Genet's version held at `0.1.1`). My Mere door code was
verified with a throwaway `[patch]` of Murm to a local checkout of the pre-removal Murm plus my two commits, and is
pushed on branch `agent/p2-door-serving` without a relock; it merges to main once Mere's lock can take Murm main.

### P3 spike (2026-11-08)

**Step (1), the split.** The blocker was one manifest line. `core/Cargo.toml` (the `cleromancy-core` package Mere names as
`cleromancy-domain`) listed `muniment` with feature `redb`; the only code that opens a redb store is the root package's
`cleromancy_receipt` binary, which keeps its own row. With the feature dropped, `cargo check --locked --target
wasm32-wasip2` passes for the whole portable core (`errno` went with `redb`). `wasm32-unknown-unknown` still fails, now in
`getrandom 0.2` ("enable the js feature"); a wasmtime guest does not need it. No code moved: a separate validator-only
crate is not needed for the guest, though it would shrink it (Cleromancy `046bd58`).

**Step (2), the spike** (Cleromancy `spike/wasm-authority`, throwaway, own lock root; corpus from
`tests/authority_corpus.rs`; reproduction in its README; toolchain 1.97.1, wasmtime 45, wit-bindgen 0.57.1):

| Measure | Result |
|---|---|
| Builds as a component | yes: `cargo build --release --target wasm32-wasip2` on a `cdylib` emits a component |
| Size | 7,816,625 bytes (7.45 MiB) against the 16 MiB cap (`opt-level = "s"`, LTO, strip, `panic = "abort"`, no wasm-opt) |
| Verdict-equal, native versus wasm | 577 of 577 cases, identical error text in 577 (51 refused, 526 admitted); 0 traps |
| Corpus | the six product graphs of the seven moving tests (published reading, typed decks, ambient, shuffled, situation, inference) and 571 single-leaf tampers of their captured batches, each with its native verdict |
| Per call, release, Apple silicon | native min 0.11 ms, p50 10.3 ms, p90 26.9 ms, max 34.1 ms; wasm min 0.21 ms, p50 20.4 ms, p90 55.6 ms, max 101 ms (about 2 times native); includes the guest's JSON decode and graph restore |
| Fuel per call | p50 3.9e8, max 1.74e9 (wasmtime fuel units) |
| Guest memory | 6.6 MiB peak requested |
| Cold start | compile 1.03 s (Cranelift on first load; precompile to avoid), instantiate 0.37 ms |

Candidate sizes 14 KB to 296 KB. Time follows the replayed content (ephemeris recomputation), not the byte size.

**Interface.** The drawing board's `domain-authority` world works unchanged (`describe`, `validate(list<u8>)`,
`admit`; `log` import only). Findings on the bytes:
- `GraphSnapshot` alone is not a complete candidate: it does not carry the facet store, and the validator over it fails
  ("must carry exactly one domain record"). The candidate the spike uses is one JSON object `{snapshot, facets}`
  (`Graph::to_snapshot_at`, `Graph::facets()`, `Graph::try_from_snapshot`, `Graph::install_facets`, all serde); the
  validator over the restored graph agrees with the native verdict in every case. The captured-delta batch also works but
  the resident holds no whole-graph delta export, so the declared format should be snapshot plus facets
  (`mere.graph-snapshot-and-facets/json`). The resident's own encode time per call is not in the numbers above.
- A fuel cut-off traps the guest and the instance then refuses re-entry ("cannot enter component instance"): the host
  instantiates afresh after a trap (0.4 ms).
- A guest panic is a trap (`panic = "abort"`); the kernel's delta replay panics on a malformed stable id, which the
  snapshot path reports as an error instead.

**Blockers.** None for `wasm32-wasip2`. Not measured: cross-architecture determinism (one aarch64 machine), the seven
tests' multi-process owner behaviour through a guest (step 3's conformance), the resident-side encode cost, and `admit`
(stubbed; divination registers none). The fallback (supervised native process, D15 (h)) is not needed on this evidence.
Mark's review of the spike gates step (3).

### P3 guest (2026-11-08)

**What was built.** Protocol layer (Graphshell `4d971f3`): `authority-guest` world and `domain-authority` interface in
`world.wit`, with a test beside `block_guest_world`. Mere: `crates/script/authority-host` (the wasmtime host, 9 tests
over a fixture guest), `ports/mere-resident/src/authority_guest.rs` (candidate encoding, `GuestRuntime`, owner review,
pin, `GuestStore`) and `Authority::Guest` carrying a running guest; `tests/authority_guest.rs` 6, a route test in
`resident_mere` 1. Cleromancy: `authority-guest/` (own lock root, toolchain pin, `cleromancy-core`'s
`validate_divination_graph` unchanged behind the WIT). Design is in the domain contract, "The authority guest".

**Choices this step made** (none contradicts D15; the plan did not rule them):
- A fresh instance per call, not a reused one with re-instantiation after a trap. A trapped instance cannot be re-entered
  and a reused one could carry state between verdicts; instantiating is under a millisecond.
- Fuel is the budget, not a wall clock. WASI is closed: clocks fixed at zero, fixed-seed random, no preopens or stdio.
- The claim gained `admission: bool`, so a guest with no before/after check costs no encoding of before and after.
- No on-disk precompiled artifact: `Component::deserialize` is `unsafe` and a stored file would bypass the pinned hash.
  The component compiles once per process (about a second, optimised).
- The review reuses D12's signed pack (`eidetic` pack signing) and the one power `power:domain-authority`, but not
  `CapsuleMount::review` itself: that lives in `ports/graphshell` and is tied to Moot capsule disclosures. The
  resident's `AuthorityReview::prepare`/`approve` does the same checks and adds the author-key binding and the pin.
- Guests install through a per-persona store under the shared root, read by `MereRoutes::with_resident_defaults`, so
  every composition picks them up and none names an application. `resident_defaults` now takes the store.
- `mere_defaults` still reserves the domain id `divination` (a name for data, not a dependency). Without it an
  unreviewed machine would serve the domain unchecked. A reserved domain with no pin is refused `domain-not-registered`;
  a pinned domain whose guest is revoked, missing, altered, or signed by another key is registered and refuses every
  candidate as `pending: no authority installed for <domain> (<reason>)`.

**Measurements** (Apple silicon, the spike's corpus of 577 cases: six product graphs and 571 tampers):

| Measure | Result |
|---|---|
| Component size | 5,396,056 bytes (5.15 MiB), against the 16 MiB cap; the spike's 7.45 MiB had more dependencies. Test ceiling 8 MiB |
| Verdicts | 577 of 577 equal native, 577 of 577 identical refusal text (526 admitted, 51 refused), 0 traps |
| Fuel per call | max 1.775e9 (spike 1.74e9) |
| Guest memory | max 6.4 MiB (spike 6.6) |
| Candidate | up to 296,077 bytes |
| Per call, release | native p50 9.6 ms, p90 25.8 ms, max 27.2 ms; resident total (encode + guest) p50 25.6 ms, p90 111 ms, max 121 ms |
| Encode (resident side) | mean 0.19 ms, max 0.47 ms release (mean 7.7 ms, max 46 ms in a debug build): about 1 percent of the call |
| Budget chosen | fuel 2e10 (11 times the corpus max, about 1 s of guest time), memory 128 MiB (19 times), candidate 16 MiB (54 times) |
| Cold compile | about 1 s once per process, optimised; tens of seconds in an unoptimised debug build, which is why Cleromancy's dev profile optimises Cranelift |

**Conformance.** The seven multi-process owner tests (embedded, daemon, embedded) pass through the guest: 7 of 7, 42 s.
The four natal tests that moved out of `resident_mere` pass: 4 of 4. The corpus test (guest versus native) passes.
The tests ran in a scratch member of Mere's workspace over Mere's lock (see below) with Cleromancy's sources.
`mere-resident` counts: lib 272 to 269 (four natal tests moved, one guest route test added), `embedded_reservoir_validation`
from 8 to 1 (the product half's 1 + 6 moved), `tests/authority_guest.rs` 6 new, `domain_registry` 8. Failures are the known
ones only (`the_dialable_host`, `custodian_client` 2, `cxf_import`, `quarantine_review`).

**Blocker: Cleromancy's root lock.** Moving Cleromancy's root lock to a Mere that has the guest host fails: Mere's
graphshell requires `wgpu-cue = "=0.1.3"` while wgpu-cue `main` names an unpublished 0.1.4 (and webcue 0.1.3 against
inker's `=0.1.2`), the same five-root blocker the P0 result lists. Moving a git source's revision always re-resolves its
dependencies, so no `cargo update -p` keeps `cue-netrender` where it was. Cleromancy's test and example changes, which need
the new resident API, wait on branch `agent/p3-guest-conformance` until that lane lands; the guest crate (its own lock root,
which resolves) is on main.

**Not done.** The pin across paired devices (`Pin::compare` exists, nothing carries a pin over the roster); a review
dialog; the cross-architecture determinism check (one aarch64 machine; fuel and the closed WASI are the means, not the
proof); `Component` caching across processes. `c1_receipt_host` no longer re-runs a product validator in its inspect
client (the resident validated before serving).

## Keeping cycles out: the check

`scripts/check_family_cycles.py`, with tests beside it in `scripts/test_check_family_cycles.py`
(`test_cargo_mode.py` style). The text below is the original design; **as built (2026-11-08)** it
differs in three ways, each because a lock cannot tell the kind of an edge:

- The family-wide scope reads **manifests**, not locks: every `git = ".../merely-made/<repo>"` row
  (dependencies, build-dependencies, dev-dependencies, target tables, and `workspace = true`
  inheritance; an unused `[workspace.dependencies]` row is no edge) is an edge tagged normal, build or
  dev, and optional rows count. Normal and build cycles fail; dev-only cycles fail too (D6), or warn
  with `--dev-warn`. A repository dependency on itself is ignored.
- The per-root scope still reads each `Cargo.lock` (one resolved graph, kind unknown). A cycle inside one
  lock is a **warning** by default, because a stale pin of a donor (a lock still holding Distillery at a
  revision that named Mere) shows as one; `--strict-locks` makes it fail.
- The allowlist is `[[edge]]` (a directed pair `from -> to` that a decision rules to break, left out of the
  graph, so what remains must be acyclic) and `[[isolated]]` (a manifest root left out of the family-wide
  graph), each with `decision` and `reason`. An entry on no cycle fails.

Run it: `python3 scripts/check_family_cycles.py` (checkouts beside Mere; each directory is named by its
`origin`, so worktree names do not matter), `--family-dir DIR`, or `--remote` (shallow-clones every
`merely-made` repository; about 80 s). It needs `tomllib` (Python 3.11) and falls back to `tomli` or
pip's vendored copy, which is what runs on this Mac's system Python 3.9. The original design follows.

- **Input.** Every lock root of every family repository. Family repositories are the clones
  named by `--family-dir` (default: sibling directories of the Mere checkout; in CI, shallow
  clones of `https://github.com/merely-made/<repo>.git` at `main`). A lock root is any
  `Cargo.lock` outside fixtures, receipts and `support/patches`.
- **Edges.** Parse each `Cargo.lock` (`tomllib`; no cargo run, no network). Each `[[package]]`
  has a `source`: `git+<url>#<rev>` or none (a path package, which belongs to the repository
  that owns the lock). Normalise a source URL to a repository name: strip scheme, `www.`,
  `.git`, and keep only the `merely-made/<repo>` form; other sources are external and ignored.
  For each package and each of its resolved dependencies in a different repository, add an
  edge `owner(package) -> owner(dependency)`. Because lock files list optional dependencies
  and dev dependencies alike, the graph is the all-feature graph, which is what a cycle
  costs (see Findings).
- **Two scopes.** Per root: a root whose repo-level graph has a cycle inside one lock (the
  Mere lock with Knot and Mere both present) is a cycle. Family-wide: the union of edges
  across all roots, strongly connected components of size above one.
- **Allowlist.** `support/family-cycles/allowed.toml`, one entry per edge pair, each citing
  this plan's decision (`cycle-breaks D1`) or an isolated-root reason (the signalman-desktop
  root). A cycle not on the list fails; an entry no cycle uses fails too, so the list only
  shrinks. The report prints each cycle with the closing edge, the package pair, the
  manifest path and, where known, the feature that enables it.
- **Gate.** Part of the local gate beside `scripts/check_port_boundaries.py`; the family-wide
  scope runs wherever the sibling clones exist and is skipped with a stated reason otherwise,
  and the per-root scope always runs. A pin-only repin triggers no new allowlist entry
  (hygiene plan).
- **Tests.** Synthetic locks for: a clean pair; a two-repository cycle; a cycle through an
  optional edge; an isolated root that must stay out of the union via the allowlist; a stale
  allowlist entry; `www.github.com` and `.git` spellings of one repository; a dev-only edge
  (F2 settles its rule).

### The P0 result, 2026-11-08

Run with `--remote` against the `main` of all 35 `merely-made` repositories (Mere `a2c41e933`,
Murm `df3e29399`, Graphshell `d80316db1`, Genet `fe7523697`, Eidetic `71188bb66`, Distillery
`94e4fb2d2`, Dramatis `772cce491`, Inker `174ae7b14`, Knot `afac0818f`, Cleromancy `164b4f0d2`,
Seiche `4109f0b01`, Gemot `9a5c7d405`; the full list is in the check's first line of output), 1,211
cross-repository manifest edges, 91 lock roots. Exit 1.

- **Broken as the plan says:** C3 Mere↔Woodshed (Woodshed names Mere only as dependent; no Mere row names
  Woodshed), C4 Mere↔Murm (Murm names Mere only in `apps/signalman-desktop`, the D17 isolated root), C5
  Gemot↔Murm (no Murm row names Gemot) and C6 Eidetic↔Distillery (no Eidetic row names ESP or Distillery).
  Nothing listed for them, as P0 and the done-conditions require.
- **Allowlisted, ruled, still open:** `mere -> knot-editor` and `mere -> cleromancy` (D1, P2 and P3), and
  the isolated root `murm/apps/signalman-desktop` (D17).
- **Not in the plan's C1 to C6 and not ruled (the check fails on them):**
  1. **Dramatis↔Mere.** `dramatis/apps/dramatis/Cargo.toml:55-58` takes Mere's `cambium`,
     `cambium-desktop` and `mandrake` (its own lock root, `dramatis/apps/dramatis/Cargo.lock`, resolved
     cycle `dramatis, mere`); Mere names Dramatis in about 50 normal rows (`castellan-client`, `insigne`, `personae`,
     `dramatis-resident`, `notochord`, `identity`).
  2. **Inker↔Mere.** `inker/crates/fleece/Cargo.toml:37` (optional `linked-data`, which is Mere's
     `crates/graph/linked-data`) against about 30 normal Mere rows naming Inker (`inker`, `fleece`, `illume`,
     `mere-lanes`, ...). Resolved inside `inker/Cargo.lock` (`mere-fleece -> mere-rdf -> inker`).
  3. **Genet/Murm/Eidetic/Dramatis ring (cleared 2026-11-08 by D20, below).** Cutting the two rows above and the D1 edges leaves one
     component: `genet -> murm` (`genet/components/genet-webrtc` `murm-rtc` and the optional
     `components/netfetcher` `murm-wire`, "independent wire leaf"), `murm -> eidetic` and `murm -> dramatis`
     (`murm/native/crates/holder`, `murm/crates/station-custody`), `eidetic -> genet`
     (`eidetic-search`'s `genet-text`), `eidetic -> dramatis` (`identity`, `insigne`) and `dramatis -> genet`
     (the Dramatis app's `taproot`). Removing any one of `genet -> murm`, `eidetic -> genet` or both `murm`
     rows clears it. This is the Genet/Murm pair the plan treated as isolated: the isolation holds only for
     `signalman-desktop`.
  4. **Dev-only, D6:** `dramatis -> eidetic` (`dramatis/apps/dramatis` dev `luggage`), which closes against
     Eidetic's normal rows to Dramatis.
- **After D18 to D20, 2026-11-08 (the same check, `--remote`, Genet `8f13bb7e6`, Murm `c6b31e702`, Inker
  `6643e7174`, Mere with this entry; exit 1):**
  - **The ring is gone.** `murm-wire` and `murm-rtc` moved into Genet as `genet-wire` and `genet-rtc` (D20),
    so no Genet row names Murm and Genet is on no cycle. Cleromancy is off the normal graph too (P3, Mere
    `0e33b02e4`); Knot stays held by its `mere -> knot-editor` entry (D1, D18).
  - **Inker↔Mere is allowlisted** (`inker -> mere`, D19, until engine-split P5).
  - **Still flagged, no allowlist entry (D21):** Dramatis↔Mere. Every remaining normal cycle in the report (seven
    repositories: Distillery, Dramatis, Eidetic, Gemot, Inker, Mere, Murm) passes through `dramatis -> mere`
    (`dramatis/apps/dramatis/Cargo.toml:55-58`); a throwaway allowlist with that one edge added leaves only
    the dev-only `dramatis -> eidetic -> dramatis` pair (`luggage`, item 4 above). The dev-only list, with
    Dramatis↔Mere open, adds `inker -> murm` (a `fleece` dev row on `transport`), `seiche -> eidetic` and
    `distillery -> gemot`; each closes only through `dramatis -> mere`.
  - **Downstream locks not relocked (blocked, not ruled out).** Mere, Knot, Cleromancy, Turnstone and
    Woodshed's `ports/redshank` still pin Genet `d634ee0c4`, which names `murm-wire` from `murm.git` at a revision
    that stays fetchable, so their `cargo check --locked` is unchanged. Relocking any of them to Genet `8f13bb7e6`
    (`cargo update -p netfetcher`: removes `murm-wire`, adds `genet-wire`, no other package moves but the Genet source
    and `bole`'s `rev` pin) fails to compile in all five: Genet `fe7523697` moved `document-session-api` to 0.1.2, so the
    lock holds two copies (registry 0.1.1 through `webcue =0.1.2`, and Genet's 0.1.2) and Inker's `cue.rs` mixes their
    types (E0308). Inker, which already has `webcue 0.1.3`, relocked and passes `cargo check --workspace --locked`
    (Inker `6643e7174`). The five roots need `webcue` 0.1.3 and `wgpu-cue` 0.1.4 first; `wgpu-cue` main
    requires `wgpu-cue =0.1.4`, which is not on crates.io (0.1.3 is the latest), and Mere's root pins
    `webcue =0.1.2` and `wgpu-cue =0.1.3`. That is the webcue lane's loop, not this decision's. Until it lands,
    the `--strict-locks` warnings for these roots remain.
- **After D26, 2026-11-08 (the same check, `--remote`, Dramatis `a96e22df8`, Mere `d5b9dcf4d` plus this entry,
  Graphshell `4d971f381`; exit 0, no allowlist entry added):** the check passes. The report lists 35 repositories,
  91 lock roots and 1,209 manifest edges; the normal, dev and strict lists are empty, and Dramatis has no
  outgoing cross-repository manifest edge at all. `dramatis -> mere` (the Cambium rows), `dramatis -> genet`
  (`taproot`) and the dev `dramatis -> eidetic` (`luggage`) are gone with `apps/dramatis`; its one remaining
  git row, `ostiary` from Armillary, went with the app as well, since `dramatis-surface` takes `ostiary` from
  crates.io like `castellan-client` does. The `--dev-warn` run is also empty. Ten resolved-lock warnings remain, all of the
  stale-pin kind below.
- **Resolved-lock warnings (stale pins, not ruled):** `cleromancy`, `mere`, `turnstone` (a Distillery
  revision that named Mere, and the loops above), `inker`, `dramatis/apps/dramatis` and seven more roots
  (twelve in all) holding `dramatis` and `eidetic` together. Re-run with `--strict-locks` after the family's relock loop.
- The plan's "Findings" cycle list C1 to C6 predates the Dramatis, Inker and Genet rows; whether these
  are new rows or were missed is not known (the cycle map's raw edges are not committed).

## Progress

- 2026-11-06: plan written from the cycle map; D1 to D5 recorded; nothing implemented.
- 2026-11-06: D6 recorded; D4/P6 paused pending the host/guest contract brief; nothing implemented.
- 2026-11-06: D7 to D9 / P6 extraction landed. History exported from Mere `49571db83` with
  `git filter-branch` (the filter-repo tool was not installed; `--prune-empty`, merges kept; 582
  commits; the exported tree equals the donor blobs, checked) and merged into
  `merely-made/graphshell` (old `main` `7676823c`, 1,263 commits and 8 branches left alone; the
  cut-over note is `docs/2026-11-06_cut_over.md`): graphshell `7cd9ca53` (tests 264 default, 291
  all-features, pass). Knot `2dd25442` and Cleromancy `afed1727` took the source change first,
  because Mere's lock names them; Mere `18c0f24a5` is the cut (door split into `graphshell-door`,
  seams, docs). Consumers: Knot `abffd2d5`, Cleromancy `671e9bc4`, Turnstone `53559826`, Woodshed
  `dbd0da9d`, Isocosm `a9a227cc`, merelyllc.com `c6150c12`, Murm `b80cf589`. Distillery names the
  crates through crates.io and needed no row. Receipts in
  `archive_docs/receipts/2026-11-06_cycle_breaks_plan_receipts.md`. P6 stays open for `mere-comms`
  and F1's `scenomise` edge.
- 2026-11-06: P1 written as a section of the domain contract
  ([registration contract](../technical_architecture/2026-11-06_domain_contract.md)): what a
  domain hands the resident and gets back, the call sites by name for Knot and Cleromancy, the
  uses that are not domain behaviour (relation intake, the `mere-overlay` tests, the C1 receipt
  host, `personal-facts`), where each app registers, the split of `embedded_reservoir_validation`
  (generic half stays, product half moves), and six decisions for Mark. Design only; for
  Mark's review before P2 and P3.
- 2026-11-06: D12 recorded and P7 reshaped (Mark chose D, Woodshed as a guest); the design is
  `technical_architecture/2026-11-06_guest_blocks.md`; nothing implemented, held for Mark's review.
- 2026-11-06: D14 recorded (Mark answered P1's decisions b to f as recommended; (a), the
  distribution binary, is open: "go back to the drawing board"); P1b added; the resident split
  plan's constraint amended. The drawing-board section is in the domain contract.
- 2026-11-06: D13 recorded (Mark's eight guest-blocks answers). The view spike was run and fell short
  for painting (a `sceno` scene carries the event regions and names but not the engraving: holes
  lost, no colour, text or fill rule); the full guest component is 1,718,946 B against the 16 MiB
  cap. R1 to R3 are with Mark; nothing moved.
- 2026-11-06: P4 / D2 landed. Distillery `e33bbaf` took the ESP adapter, the MiniLM calibration test, the
  lexical recall receipt and the `eidetic-recall` example (cut first; ESP built and ran them against
  the old Eidetic main); Eidetic `6b4ec49` dropped them, the `bert` and `recall-example` features, the
  `esp` dependencies and the workspace `esp` row (lock 879 to 411 packages, no `esp`); Distillery
  `ca9334d` relocked onto it (one Eidetic source); Mere (the commit carrying this entry) repointed
  Graphshell's `goal-surface-model` to `esp/goal-model` and relocked the root and
  `ports/graphshell/web`. Counts, before in Eidetic and after in ESP: lexical receipt 2 passed, 1
  ignored (default) and 4 passed with `bert --include-ignored` (before 4, after 4);
  `eidetic-recall` example tests 4 and 4; `goal_model_minilm` `adapter_end_to_end` ok and
  `calibrate_match_similarity` failing the same way in both (Findings). `eidetic-search` default
  tests unchanged after the move (74 lib passed, 1 ignored, and every integration binary that stayed).
  Graphshell `goal-surface-model`: `--all-targets` check clean, 29 `goal_surface` tests passed, 1
  ignored. The throwaway cycle proof is in the P4 receipts. The allowlist entry for C6 does not
  exist yet (P0 is not built); delete it with P0's allowlist.

- 2026-11-06: P5 / D3 landed. Murm `c0adc674` (`murm-resolve` owns `ReleaseAddress`; `murm-holder`
  and the resolve tests moved onto it; `native/Cargo.lock` lost `gemot-address`); Gemot `df6da4a`
  (`gemot-site-host` feature `murm-resolve`, `ReleaseAddressRef`, four tests, relocked). Murm's
  default-feature tests: `murm-resolve` 7 and `murm-holder` 8, unchanged. Cycle proof (throwaway script,
  every lock root of Gemot and Murm, `cargo metadata --locked --all-features`, dev edges counted):
  Gemot to Murm edges in the root, `crates/site-host`, `crates/site-model` and `probes/spaces-eval`
  (`murm-sync`, `mere-transport`, `murm`, `murm-resolve`, `retinue`); no Murm package depends on a
  Gemot package, directly or through any third repository, in any root. Murm's `fuzz` lock is stale
  against its manifest (`--locked` refuses; untouched, names no Gemot package); `webrtc-evaluation`
  was read from its lock text. Receipts record the output. The C5 allowlist entry does not exist
  yet (P0 is not built); delete it with P0's allowlist.
- 2026-11-06: P6 / D4, `mere-comms` half landed; F1 and P6 stay open. Murm `559470b6` (history merge,
  128 commits, tree equal to the donor's) and `45b1cf47` (workspace member, Signalman by path, both
  locks); this Mere commit deletes `crates/shell/comms`, points `ports/moot` and `ports/graphshell`
  at `murm.git`, drops the now-unused `murm` row and its two patch rows, and relocks the root and
  `ports/graphshell/web`. Mere-moot lib 25 passed, `graphshell --features workspace-editor` clean, the
  three `comms` tests pass; `mere-comms` 28 default and 38 all-features tests pass in Murm as in
  Mere. Cycle proof: Murm to Mere edges remain (Findings, P5 and P6 findings) and Mere to Murm
  unchanged in direction, so the cycle stands, held by `portable-scenes` and `signalman-desktop`.
  The F1 measurement and three options are in Findings; not ruled.
- 2026-11-06: SHAs for the two entries above: Mere `28a0be22a` (the commit that carries them);
  consumer Turnstone `f3a9e445` (relocked onto current mains, `mere-comms` now from `murm.git`; lib check
  clean with and without `peers`, 10 `captured_collection` tests pass). No other family lock names
  `mere-comms`, and `murm-resolve` has no consumer outside Murm.
- 2026-11-06: D15 recorded (authority and lane split; wasm guest after a spike; the blob door route first); P1's Lane and distribution parts superseded; P2 and P3 reshaped; P2a added.
- 2026-11-06: P1b landed: `mere_resident::domain_registry` (`DomainRegistry`, `DomainRegistration`, `Authority::{Graph, Guest, Declared}`, typed `domain-not-registered`), `MereRoutes::with_registry`, `bin/djinn.rs` and the embedded resident registering through it, `personal-facts` installed by `mere_defaults`, divination still registered by Mere until P3. mere-resident lib 259 to 261, new `tests/domain_registry.rs` 8, other targets unchanged; the same known failures; pandect 331. Interim: `ServeGeneric` plus reserved domains.
- 2026-11-06: D16 recorded (Mark answered guest_blocks R1 to R3); implementation of P7 begins.
- 2026-11-06: D17 / P6 closed. Graphshell `504097c1` and merge `5b25f351` (history import, crate
  `sceno-backdrops`, with a doc fix `e00f7edf`), `2f771813` (`SpanTarget`, `strict_core_records`); Murm
  `525d948b` (`portable-scenes` onto the crate, root lock free of Mere); Mere `794a88c77` (direct
  imports, `scenomise` loses the two modules, relocked root and `ports/graphshell/web`). Cycle proof and
  counts in Findings (D17 findings): no Murm package depends on a Mere package outside
  `signalman-desktop`, the declared isolated root. P8's allowlist therefore keeps that second isolated
  entry. Woodshed follows in the next entry.
- 2026-11-06: D17 follow-up: Woodshed `c35afb0e` (the one consumer outside Mere that used
  `scenomise::backdrops`/`template`: `woodshed-core`, `woodshed-views`, `audio-pack-views` import from
  `sceno-backdrops`; relocked onto current mains; workspace `--all-targets --all-features` check clean,
  `woodshed-core --features portable-scenes` 329 passed, `woodshed-views` 309 passed, `audio-pack-views` 5
  passed). Mere `f2af7218` is the commit that carries the D17 record. No other family repository names
  `scenomise::backdrops` or `::template`.
- 2026-11-06: P7 (D12, D13, D16) implemented except the pieces named below. Graphshell `40cadadf`
  (the `block-guest` world and `vector-sheet`; 291 to 299 tests), `d66d6495` (`rows`), `40e8c8df`
  (argument and pattern fields). Mere, in order: `deb7bc974` (Pandect `Backend` impl non-`Send` on wasm,
  the web wasm build was already broken there), `ea1103089` (BlockAdapter trait, registry, memo, rhyme as
  the built-in adapter, sheet painter; also relocks the web lock's `md-5` and `sha1` to 0.10 because
  `spareval` mixed digest versions on wasm), `88834c656` (`crates/script/block-host` with a Mere-built
  fixture guest, 5 tests), `a8ab87b15` (`power:block-adapter`), `9749d7cac` and `a321a2ac8` (rows and
  fields), `85a7d4f06` (WorkerAdapter and memo), `fc0278801` (the pane dispatches through the
  registry; guest views painted from the vector sheet; unclaimed class disabled, named, with
  "Review block adapters"), `261dcc689` (`graphshell-blocks`: `GuestAdapter` over block-host and a
  reviewed mount, 4 tests), `4cc999786`, merge `2654967a8`, and `4bba47f72` (delete `bars.rs`, drop the
  `woodshedding` and `woodshed-notation` rows, relock: neither Mere lock holds a Woodshed package).
  Woodshed: `9cb2c053` (`woodshedding` feature `blocks`), `a6c45fba`, `0e6a69dd` and `73f3c901`
  (`woodshed-notation` feature `block-guest`; the component is 1,652,723 bytes), `0e2f0450` (hocket
  relock), `44617484` (the pack builder example and the conformance test through Mere's host).
  Tests: `block_editing` 26 before, 37 after (8 `bars` tests left; adapter, worker, sheet leaf and
  rhyme-scene tests joined); the 8 `bars` tests are 8 in Woodshed (5 unchanged and 3 rewritten in
  `woodshedding`) plus 3 conformance tests and 9 guest tests. **Not done:** the host-neutral
  `block_pane` extraction and a native Cambium window for `graphshell-blocks` (the crate has the
  host and a headless render receipt, not a window); the web Worker's JavaScript side and the
  headed web check (no node, jco or wasm-bindgen CLI on this Mac; the web wasm still checks and the
  `WorkerAdapter` is tested against a scripted transport); the four notation scenarios under
  `ports/graphshell/web/scenarios/` still name the old Woodshed controls. Details in
  guest_blocks "Implementation status".
- 2026-11-07: P7's missing pieces, in order. (1) `1d6b2bc48` extracts `graphshell::block_pane` (host-neutral
  `Pane`, adapters and views over a `PaneHost` and `PanePorts`; the web page binds it thinly in
  `web_tree/blocks.rs`); web wasm checks, `block_editing` 37, lib 379 pass. (3) `ced77aa00` adds the
  native Cambium window `graphshell-blocks` with an in-window review panel (reads each
  `--adapter PACK COMPONENT`, verifies with `CapsuleMount::review`, loads only on "Approve block
  adapter"); logic tests 2 pass; it was compiled and unit-tested, not shown, because the Mac's
  console was locked. (4) `c4baa830b` moves the four notation scenarios to the guest flow
  (`blocks-insert-example`, `blocks-view-*`, `blocks-command-*`, `blocks-guest-*`, probes
  `blocks-view-mode`, `blocks-event-selected`, `blocks-adapters`) and adds 3 tests that they parse and
  name real controls. **(2) is not done**: the web Worker and the disclose, review and load host for
  block adapter packs need `@bytecodealliance/jco` 1.37.0, which the applet path takes from npm
  (`ports/moot/examples/capsule-library`, a pnpm lock) and which is neither vendored nor on this
  machine; the brief said to stop rather than fetch it. Web today: progression blocks open as text,
  the command disabled and the class named, with "Review block adapters" setting a flag and a
  status line.
- 2026-11-07: P7 step 2 done after Mark approved fetching jco (`17186947c` the Worker, the review host, the
  exports and the build script; `3c06dfda6` open-the-pane, swatch retry; Woodshed `backdrop-scene` command).
  Checked in the browser pane against the real signed Woodshed pack: unclaimed class disabled and named,
  "Review block adapters" opens a real review, approval loads the component into a Worker, the pane lists
  the guest's bars, offers its example and commands, and draws Chart, Tab and Score from the vector sheet
  (screenshots in `testing/mere/guest_blocks_spike_20261106/web_headed/`). The "Editable document"
  control is not reachable on `tree.html?app=local` at baseline either: it sits behind Panes, Graph
  controls, and enabling that crashes the page (genet-livery "C3 resolves every paint color") in a
  build of `ea1103089` as in this one, so it is not a regression; `window.blockAdapters.openPane()` and an
  "Open block pane" button open the same pane.
- 2026-11-07: P7 follow-ups (three items). **Chord-text contrast.** I could not reproduce "nearly invisible" at real size
  on the web page: the baseline build (colour fix removed) and the fixed build draw Dm7, G7, Cmaj7 and the bar
  numbers identically, about 16.7:1 on the dark pane (canvas pixel sample, text 224,241,252 on 4,15,22), and the
  light theme is legible too; the old 400-pixel screenshot was a downscale of an 1100-pixel page, which thins 12-pixel
  text. The guest is not the cause (it paints labels in `Role::Ink` at full alpha). The host had two real defects:
  `SheetLeaf::texts()` computed a colour per run and `block_pane` discarded it, leaving each span to inherit
  whatever its container's colour was; and the accent role (the theme's secondary colour) fails AA for text on some
  themes (2.63:1 on the default light theme), as would muted (a guide colour at 0.35 alpha). Fix in the host painter:
  the span now carries the run's colour (`TextRun::css_colour`), and a leaf told its ground (`SheetLeaf::on_ground`,
  given by both hosts) draws any text role that is under AA in ink; `SheetTheme::from_surface` is the one
  role mapping the web page and the native window share. Tests: `sheet_leaf` 4 to 9 (all built-in themes in all
  four modes, Ink and Accent, 192 runs, lowest 4.81:1; the default surface theme; the fallback; CSS colour; the
  contrast function), `graphshell-blocks` scenarios 3 to 4 (the pane must set the colour); `graphshell` lib with
  `editable-blocks` 372 pass. Screenshots in `testing/mere/guest_blocks_contrast_20261107/`. **Moot disclosure.**
  `ports/moot/examples/block_adapter_disclosure.rs` shares a signed block-adapter pack into a Moot, selects it into a
  collection, ships it as a plain drop to a retaining member that re-verifies it, and writes `moot-disclosure.json`;
  `build_block_adapters.mjs --moot DIR` builds the site from it with the real Moot, collection and revision in each
  entry's `CapsuleDisclosure`; the web review panel and receipt name them (checked headed in the browser pane with
  the throwaway-seed pack); the native window reads the same file with `--moot-disclosure FILE` (unit test; no live
  Moot connection, so the native shell has no Moot path of its own). `--pack`/`--component`, `adapters.json` and
  `--adapter` stay as development inputs. Not done: no network in the proof, no live fetch by either host.
  **Stale locks.** Distillery's `probe`, `remote-fixture` and `session-fixture` roots moved from Eidetic `2df00e0` to
  `71188bb` (remote-fixture's `insigne` and `personae` followed to Dramatis `696472da`); Mere's `ports/graphshell/web`
  no longer carries the unused `netrender-vello-shaders` patch (the graph takes shaders at the rev
  `netrender-vello-encoding` names), and its lock lost the matching `patch.unused` entry. Each touched root passes
  `metadata --locked` twice with no diff and `check --locked`. Distillery `94e4fb2`; Mere `b27e07466`, merged to main as `25b19570e`. Woodshed needed no change (the guest paints labels in ink; its conformance test builds `SheetTheme` literals, which this change leaves as they were).
- 2026-11-08: P0 built. `scripts/check_family_cycles.py`, `scripts/test_check_family_cycles.py` (15 tests on synthetic
  repositories: clean, two- and three-repository cycles, optional, build, dev (D6 and `--dev-warn`), target tables,
  workspace inheritance, spelling variants, skipped fixture directories, allowed and stale edge entries, an isolated root,
  a resolved-lock cycle) and `support/family-cycles/allowed.toml` (D1 `mere -> knot-editor`, `mere -> cleromancy`; D17
  isolated `murm/apps/signalman-desktop`). The result on the real family is under "The P0 result": three cycle groups this
  plan did not list, which the check reports and does not allowlist. Not wired into a gate yet.
- 2026-11-08: P2a landed (the blob door route). Decision (f) first: `LEGACY_KNOT_LEASE` left `resident_blobs.rs` for
  `resident_knot.rs` (the lane passes its own lease name; the shared file names no application). Then the route
  `blob-custody-v1`: shape stated first in the domain contract ("The blob door route", sections 1 to 5), built the same
  day (section 6). Server `ports/mere-resident/src/resident_blob_door.rs` (`BlobDoor`, registered by `djinn`); wire
  and client `graphshell-door` feature `blob-door` (`BlobDoorClient`); `ResidentBlobCustody` gained the durable scope
  owner record (`claim_scope`, `reserve_scope`, `withdraw_serving`). Grant: the route in the application's declaration
  and the owner's grant, plus first-binder scope ownership (the resident's own personal scope is reserved). Revocation:
  `BlobDoor::revoke` cuts live sessions and withdraws serving, keeps leases and ownership; `restore` plus re-bind
  rebuilds serving from the leases. Tests: `resident_blob_door::tests` 8 new, `tests/blob_door.rs` 1 new over the real
  door, `resident_blobs` 2 unchanged, `graphshell-door` 2 new wire tests; mere-resident lib 264 to 272 (the same
  `the_dialable_host` failure), and the same known failing binaries (`custodian_client`, `cxf_import`,
  `quarantine_review`); everything else in `mere-resident` passes. The in-process callers (personal replica, Knot lane)
  are unchanged except that `personal_sync` reserves its graph scope. Not wired: `djinn` has no live grant-removal
  path to call `revoke`. The route does not move serving; P2's blocker is in Findings.
- 2026-11-08: P2 stopped after step (1); two design questions (Declared authority has no consumer or delivery path; the
  supervised host cannot serve blobs through the door) are written up with options in Findings, "P2 findings". P3 steps (1)
  and (2) done and pushed (Cleromancy `046bd58`): `core/Cargo.toml` drops muniment's `redb` feature, the portable core checks
  on `wasm32-wasip2`; the component is 7.45 MiB of 16, verdict-equal over 577 of 577 corpus cases, about 2 times native per
  call (p50 20 ms, max 101 ms). The real guest is held for Mark's review (D15 (h)). Cleromancy's own plan has a short entry.
- 2026-11-08: P2 steps (2) and (3) rulings built as far as they go (D24, D25). **Declared:** nothing to build; the typed
  pending slot and its test exist (`domain_registry.rs`). **Transport:** Murm `c6b31e70` (main): `blob_source`
  (`BlobSource`, `BaoReader`, `StoreBlobSource`, chunk-interval helpers), `scoped_source`, `scoped_blobs` now the
  store-source wrapper; tests `mere-transport` lib `blobs::` and `blob_source` 20 pass (1 new interval test, 1 new
  `a_source_serves_verified_streams_without_a_store_in_the_provider`; the existing scoped-serving and revocation tests run
  through the unified provider); the whole-crate run had one failure,
  `p2panda_transport::tests::both_sides_dialling_at_once_stays_connected_while_the_link_delivers`, which hangs for over 150 s
  and does not touch blobs (not compared against a baseline). **Door:** Mere branch `agent/p2-door-serving`: ops `bao-open`
  and `bao-close`, `DoorBlobSource` (`graphshell-door` feature `blob-serve`), resident `BaoCursor`; tests `blob_door` 1 to 2
  (the new one: a provider with no store serves a 200 KB multi-leaf blob to a peer through the door, verified by the peer;
  ranges at the door; leased-but-unretained refused; non-reader refused; a scope the application does not own refused;
  denying the reader cuts serving; revocation cuts serving and keeps custody), `resident_blob_door` lib 8 and
  `graphshell-door` 4 unchanged and passing. **Not on main:** Mere cannot take Murm main yet (Findings, "P2 relock blocker").
  Not done: Knot's side (a door-backed evidence host is drafted on Knot branch `agent/p2-knot-door-evidence`, uncompiled
  against the relock), the supervisor, the multi-process Knot test, step (4), the independent moves, and the checker entry
  `mere -> knot-editor`, which stays.
||||||| 588fb8ea6
- 2026-11-08: D18 to D23 recorded (Mark's rulings after The P0 result); `inker -> mere` allowlisted (D19). D20 done:
  `murm-wire` and `murm-rtc` moved into Genet as `genet-wire` and `genet-rtc` (`components/genet-wire`,
  `components/genet-rtc`; nothing outside Genet named them, so they were renamed). History kept: 42 commits of Murm's
  `wire/crates/{wire,rtc}` (a tree-per-commit export, authors and dates kept, each message ends with its Murm SHA; Murm
  `df3e2939924`), merged as Genet `0f38f9d2c`; the rename and wiring are Genet `8f13bb7e6`. Murm `58d068a04` removed the `wire/`
  workspace and its README, DOC_README, gitignore, gitattributes and exclude rows; `merely-made/rtc` `789c434b6`
  repointed its README. Inker `6643e7174` relocked onto Genet. Tests, before in Murm and after in Genet: `murm-wire`
  7 and `genet-wire` 7; `murm-rtc` with `--features dns,udp` (after adding the `libc` dependency its `dns` feature
  already used) 22 passed and 6 failed in the lib, 1, 1, 1, 1 and 4 in the integration binaries, 3 ignored, and the
  same in `genet-rtc` (the six lib failures are the macOS mDNS-port ones, identical before and after);
  `netfetcher` 98 passed; `genet-webrtc` lib 6 passed and `browser` 26 passed, 12 failed on `AddrInUse` and
  `AddrNotAvailable` (mDNS port 5353 and loopback aliases; the baseline could not be built, `murm-rtc` lacked `libc`, so
  the same failure shape is shown only by the `genet-rtc` twins). Netfetcher's `hyper-transport`, `h3` and `websocket`
  features each check alone; `ortet` and `genet-wpt` check with `--all-targets`. Genet commits no `Cargo.lock`, so
  its checks are unlocked. Murm default and `native` check `--locked`; `native` lib tests pass (28, 97, 69, 15, 44 and
  small binaries). Five downstream roots could not relock (The P0 result). Receipts are in the P0 result section.
- 2026-11-08: P3 step 3 to 5 (Mark: "Build the real guest", D23). Graphshell `4d971f3`: the `authority-guest` world and
  `domain-authority` interface (claim has `admission`). Mere: `authority-host` (9 tests), `authority_guest` review, pin and store
  (6 tests plus a route test), `Authority::Guest` runs a reviewed component, `mere-resident` drops `cleromancy-core`
  (`divination_registration` and `with_resident_defaults`' divination are gone; `with_resident_defaults` reads the guest store),
  the `mere -> cleromancy` allowlist entry is removed, the product half of `embedded_reservoir_validation` and the four natal
  lib tests leave for Cleromancy. Cleromancy: `authority-guest/` (5.15 MiB), 577 of 577 verdicts and refusal texts equal native,
  the seven owner tests and four natal tests pass through it. Cleromancy's root lock is blocked on webcue and wgpu-cue, so its tests
  are on a branch. Measurements and choices: Findings, "P3 guest".
- 2026-11-08: Mere's pre-move `ports/dramatis` copy is deleted (Mere `028954140`): the workspace member, the
  `dramatis-app` lock entry, the four packages only it reached (`cargo update -w`), and the six
  `support/dramatis_*` receipt scripts and `scripts/test_dramatis_auth_tools.py` that hashed it (Dramatis holds
  its own copies under `support/`). No other family repo named the copy by path or by git package. Root
  `cargo check --workspace --locked` and `ports/graphshell/web` `cargo check --locked --target
  wasm32-unknown-unknown` pass. The stdio carrier's peer authentication is a design awaiting Mark's ruling R1:
  transport agnosticism note, section 10.
- 2026-11-08: D26 (Mark: shape A only, pane only, no interim, in process until the stdio carrier authenticates its
  peer). Dramatis `crates/dramatis-surface` (`f3ba597` to `0bef744`; the custody state machine and worker copied from
  the app, then served as titulus cards and `ActionFormV1` actions over `ostiary`, with 11 surface tests and the
  app's 5 state tests; no contract change) and `a96e22df8` (`apps/dramatis`, its packaging and its receipt drivers
  deleted, the `cambium`, `cambium-desktop`, `mandrake`, `taproot` and dev `luggage` rows with them). Mere
  `f957c7299`: `ports/graphshell/dramatis-pane` (`graphshell-dramatis-pane`, 12 tests rebuilt from the window's
  interaction tests: focus order, zoom and narrow layout, focus-follow against manual scroll, live region and
  heading stability, disabled-while-busy) and its mount in `graphshell-mere` (3 tests; 44 before, 47 after);
  Mere's lock moves `dramatis.git` to the surface commit. `scripts/check_family_cycles.py --remote` passes (The P0 result).
- 2026-11-08 (host placement): engine-split P3 had not landed when the Dramatis pane was placed (Mere `183acf839`, Graphshell
  `4d971f38`: the Graphshell repository still holds only the contract root, and Cambium is in Mere), so the pane is Mere's
  `ports/graphshell/dramatis-pane` plus a mount in `ports/graphshell/mere`. Both sit inside `ports/graphshell`, which D23
  moves to Turnstone, so the P3 lane carries them with it: the pane crate names only `cambium`, `cambium-desktop`,
  `cambium-rootstock`, the published projection contract and `dramatis-surface`, and needs no change but its path.
- 2026-11-08: P2 steps (3) and (4) built and pushed (Mark: no pins, no locks, track main). **Mere `cacf7094f` (main):**
  `ports/mere-resident` drops `knot-editor` (both rows, the `peers` and `mere-overlay` features, the `knot_residue` `[[test]]`),
  `resident_knot.rs`, `knot_relation_intake.rs` and `OwnerSettings.knot`. New: `supervisor.rs` (`AppSupervisor`: starts a
  named command with `GRAPHSHELL_APP_ENDPOINT` set to the door, kills it on lock, restarts it on unlock and after a crash
  with a 5 s back-off; `{data_root}` and `{endpoint}` placeholders; 3 tests) and `OwnerSettings.supervised`
  (`SupervisedAppSettings { app, command, args }`; the old `knot` key is read as it for one release by a `knot_sync_host
  --door-custody` command line and written back as `supervised`; 1 test). `djinn` starts the child once the door is up,
  ends it on lock and restarts it on unlock, keeping the events `knot-closed` and `knot-reopened`; the Knot arm of the
  select loop became the supervisor tick. `graphshell_host::canary::FixtureEndpoint` gained resume and notice impls so
  it serves as a provider fixture; `app_projection_offers` (4) and the provider half of `declared_applications` run on it,
  and `declared_applications`' Knot custody test is a vault-application test over literal contexts (held to Knot's real
  declaration by a Knot test). Counts: `mere-resident` lib 269 to 270 (3 supervisor and 1 settings new, the lane test and
  the 2 overlay tests gone); `knot_app_route` 1, `knot_checkpoint_receiving` 1 (+1 ignored helper), `knot_relation_intake`
  6 (+1 ignored helper) and `knot_residue` (harness off) left the crate. The same known failures as the baseline and
  nothing new: lib `the_dialable_host...`, `custodian_client` 2, `cxf_import`, `quarantine_review`; all else passes.
  **Knot `fc8ab8e41` (main):** `KnotContentRetentionPort::open_door` (a door-backed port holds no store;
  `blob_store()` is now an `Option`), `DoorEvidence` public with `retain_artifact` and `migrate_legacy`, `knot_sync_host
  --door-custody [--evidence-root] [--max-artifact-bytes] [--ticket-file]` (keys from custody, evidence in the resident's
  custody, old store migrated through the door, serving restored from the document's references), the declaration names
  `blob-custody-v1`. Tests: `tests/supervised_evidence.rs` 3 (a peer fetches evidence from the real supervised child
  through the door and revocation cuts it off, custody kept; the door-backed port retains into the resident under Knot's
  lease; the declaration against the owner grant and the Mere fixture literals) and `tests/relation_intake.rs` 6 + 1 ignored
  helper (moved with `knot_relation_intake.rs` as `tests/support/`; 6 before in Mere, 6 after). Knot's lock widened by the
  git dev-dependencies `mere-resident`, `djinn`, `castellan`, `dramatis-resident` (no `djinn-testkit`: the resident's door
  runs in the test process, only the child is spawned) and by Mere's git-sourced crates-io patch rows (the graph now
  carries Mere's graphics stack through `mere-resident` to `graphshell`). Knot lib 171 passed, all Knot test binaries pass.
  `check_family_cycles.py --remote`: exit 0.

- 2026-11-08 (later): P2 remainder, Mark's four rulings (no pins, no locks, one checkout, main). **Mere `2cdb1aa6c`, `65454be2b`
  (main); Knot `d3f0468` (main).** (2) **The supervised child offers the `knot` route.** `knot_sync_host --door-custody` keeps
  the `KnotResidentSource` beside the host (it is the one process that opens Knot's redb) and offers `knot` through the
  door's projection-provider route, a session per permitted viewer with the editor's write grant (`--offer-route <route>`,
  default `knot`; `--no-offer`). A refused or ended offer is logged and never stops sync. Mere un-reserved `knot` in the
  provider policy (the resident composes no lane of its own now; the reserved-route test lost its `knot` row and a new
  assertion accepts it). Ported into Knot: `tests/supervised_projection.rs` (from `knot_checkpoint_receiving`: the real
  child repairs a legacy checkpoint when it opens the store, a viewer reads and saves through the `knot` route, a restarted
  child continues the signed author; 1 test; Mere had 1 + 1 ignored helper) and `tests/knot_residue.rs` (harness off, with
  copies of Mere's `residue` and `residue_verdict` helpers; the source-and-session run is what the child holds and offers; 1
  before, 1 after). (3) **`test-fixtures` on `mere-resident`** exposes `resident_mere::fixture` (`Fixture`, `fixture`,
  `attach`, public `catalog`, `grants`, `route`; the rest `pub(super)` for the in-crate tests, which now import it; the
  module was cut out of `resident_mere::tests` with no test change). Knot's dev-dependency takes the feature and
  `tests/mere_overlay_door.rs` holds the two `knot_overlay_*` tests (2 before in Mere history, 2 after). `mere-resident` lib
  stays 270 (1 known failure). (4) **The live revoke path.** `applications::GrantWatch` compares the owner's settings
  `applications` with what it last saw: a grant that is gone is emptied in the door's grants and `BlobDoor::revoke(app)`
  runs (sessions cut, serving withdrawn, leases and scope ownership kept); a grant that comes back calls `restore`. `djinn`
  runs it every 2 s in the maintenance loop. Test `removing_a_grant_while_a_peer_is_mid_fetch_refuses_the_next_chunk`
  (`tests/blob_door.rs`, 2 to 3): the peer's verified stream has its first piece, the grant is removed, the next chunk is
  refused, serving authority is gone, the door refuses a new connection, the lease and bytes remain, and a re-grant rebuilds
  serving. `cargo check --workspace` clean in both repositories; checker exit 0. (1) **Not done: `block_editing`.** See
  P2 remainder, "Stopped on `block_editing`".

### P2 remainder (2026-11-08)

- **`mere -> knot-editor` is not gone.** The resident's rows are; `ports/graphshell/Cargo.toml:223` (`knot-document`,
  optional, `editable-blocks`, used by `src/block_editing.rs` for `KnotEditor`) remains, so the allowlist entry stays
  (its reason now names that row). Options: (A) leave it until D23 moves `ports/graphshell` to Turnstone (which already
  names `knot-document`), my recommendation since engine-split P3 owns that move; (B) move `block_editing.rs` and the
  `editable-blocks` feature to Knot, which would make Knot name Graphshell's block applet host.
- **The two `knot_overlay_*` tests were not ported.** They drive Knot's `MereOverlay` against `resident_mere`'s private test
  `Fixture` (`fixture`, `attach`, `knot_door`, ~300 lines), so they cannot be consumers of the public crate today. Removed
  from Mere without a Knot copy; the removed source is in Mere's history (`git show d3156d4e0:ports/mere-resident/src/resident_mere.rs`,
  lines 3392 to 3676). Options: (A) a `test-fixtures` feature on `mere-resident` exporting the fixture, then port;
  (B) a real `djinn` spawned from Knot by `djinn-testkit` (needs the binary built from Mere, which Knot cannot get as a
  Cargo artifact).
- **The `knot` route and the editor session no longer ride the resident.** The lane composed `KnotResidentSource` in the
  resident for Turnstone's `knot` route; the supervised child is the sync host only (it must be the one process that opens
  Knot's redb). Serving the editor through the app door is `knot_endpoint projection-provider` (route `knot` is reserved by
  it today); wiring the supervised child to offer it is not done. `knot_checkpoint_receiving` (route plus checkpoint
  receiving over the lane) and `knot_residue` (a residue run of the lane) were deleted from Mere and not ported for that
  reason; their old copies are in Mere's history. Needs Mark: should the child also offer the route, and should these two
  be rebuilt over it?
- **Not done:** pairing facts and `refresh_settings` (the host reconciles `knot-sync.json` itself every 5 s, as the
  standalone host always did); no live grant-removal path calls `BlobDoor::revoke` in `djinn` (still the API only);
  `resident_v1_fixture` (an example seeding the lane) was deleted; its code is in Mere's history and the Knot test seeds
  its own vault.

### `block_editing` (2026-11-08, ruling 1; done 2026-11-09 by Mark's choice A, "Small default in Graphshell")

Mark ruled that `block_editing.rs` moves into Knot so `ports/graphshell` stops naming `knot-document`. The move is not
mechanical, and the docs do not settle what replaces it, so it stopped here with nothing changed and the
`mere -> knot-editor` allowlist entry still in place.

- **What names Knot:** only `src/block_editing.rs` (`use knot_document::retained::KnotEditor`), in one struct,
  `Document { id, editor: KnotEditor }` (`new`, `blocks`, `wrap_selection`, `apply`, and the six tests). `Change`,
  `Preview`, `Div`, `wrap` and `MAX_DOCUMENT_BYTES` in the same file, and the five submodules (`adapter`, `rhyme`,
  `sheet_leaf`, `worker`, `store`), name no Knot.
- **What consumes it:** `block_pane.rs` holds a `Document` and reaches into `document.editor.source()` at some twenty sites
  (preview, apply, undo and save use its history); the `blocks` binary and `web`'s `product` feature
  (`graphshell/editable-blocks`) compose that pane inside Mere. A producer seam therefore needs a `BlockDocument` trait in
  Graphshell and a factory the host registers, and then every Mere build that links the pane needs an implementation
  without naming Knot.
- **The choice:** (A) give Graphshell its own small buffer-with-undo implementation of the trait (duplicating what
  `KnotEditor::scratch` supplies, so `editable-blocks` keeps working in Mere with no Knot); Knot registers its richer one
  where it composes the pane. (B) make the pane's host (`blocks`, the web product) live outside Mere, with the
  Turnstone/Graphshell move (engine-split P3 / D23), so Mere keeps only the trait and no default; the feature is dead in
  Mere builds until then. (C) leave it until D23 (the earlier recommendation, now overruled by ruling 1, so asking
  whether A or B is meant).

**Done (2026-11-09).** `ports/graphshell/src/block_editing.rs` now holds a `BlockEditor` trait (source, selection, apply, input,
input_mut, is_dirty, accept_saved_source), `BufferEditor` (a cambium `TextInput` with its undo journal, the default) and
`register_editor(factory)`, a once-only registration seam; `Document { id, editor: Box<dyn BlockEditor> }` takes the
registered editor or the default, so `block_pane`, the `blocks` binary and the web `product` feature were not touched.
`ports/graphshell` and the workspace drop `knot-document`; the `mere -> knot-editor` allowlist entry is deleted and
`scripts/check_family_cycles.py --remote` exits 0 without it. Knot (`block-editor` feature on `knot-editor`,
`src/block_editor.rs`) implements the trait over `KnotEditor` (`KnotBlockEditor`) and exposes `register()`; the host that
composes the pane with Knot calls it before the pane opens. Tests: Graphshell's six `block_editing` document tests stay and
run over the default buffer; the same six run in Knot over `KnotEditor` (6 before in Mere, 6 + 6 after). Mere `203a73abc`,
Knot `95467df`. Nothing in this repository registers Knot, so Mere builds edit through the default buffer.

**Amended (2026-11-09, Mark: "Switch").** The process-global `register_editor` is gone. `EditorFactory` (a trait, implemented
by closures and by `DefaultEditors`) is passed per pane: `Pane::load_with(storage, ports, editors)` (`Pane::load` keeps the
default) and `Document::open(id, source, &dyn EditorFactory)`; the `blocks` binary passes its factory explicitly. Knot's
`KnotBlockEditor::factory()` replaces `register()`. Tests: two panes in one process over different editors, and no editor
choice leaking between panes or documents (Mere, +2: 42 to 44 graphshell block tests); Knot's six over `KnotEditor` through the
factory plus one coexistence test (6 to 7). Mere `b68d6777d`, Knot `28c39c6`.

