# Cycle breaks plan

**Status:** 2026-11-06: in progress. D1 to D5 settle the break for each open
family dependency cycle; the Genet/Murm cycle needs no break (Findings). Two forks remain
open (F1, F2). D6 (dev edges count) added the same day. D7 to D9 (Mark, 2026-11-06) ruled
the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md) and
unpaused D4 and P6: the contract leaves and the host-role kit left Mere for the revived
`merely-made/graphshell` the same day (Progress, Findings); `mere-comms` into Murm is still
to do, and F1 is half answered. D1, D2, D3 proceed; D5 became D12 (Woodshed as a guest, design written, held for review).

**For:** breaking the family's git-dependency cycles. Mark, 2026-10-28: "what's important is
to untangle the repos and prevent cycles, dogfooding, and not drawing inefficient
boundaries" (source homes map, `mere_docs/technical_architecture/2026-10-28_source_homes_map.md`,
"Costs every move carries"). Doctrine Stack 3 (`../../doctrine.md`): "Mere consumes
producers, never the reverse". Until now the cycles were acknowledged as transitional and
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

### P1. D1 seam design (before any code moves)

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

Reviewed by Mark before P2 starts.

### P2. D1 Mere stops naming Knot

Done when `ports/mere-resident` and every Mere manifest drop `knot-editor` and `knot-document`
(`Cargo.toml` rows, the `[patch]` redirect section for `mere.git` packages Knot names, the
`peers` and `mere-overlay` features); Knot registers itself at runtime through P1's contract;
Knot's lock names Mere (an app consumes its host) but Mere's resolves without `knot-editor`;
Knot's resident tests run in Knot's repository; the standard conditions hold.

### P3. D1 Mere stops naming Cleromancy

Done when `mere-resident` drops `cleromancy-core` (renamed `cleromancy-domain`);
`validate_divination_graph` is installed by Cleromancy's registration, not by
`with_divination_authority` naming the crate; `embedded_reservoir_validation` runs in
Cleromancy; Cleromancy's `core/Cargo.toml` stays an app-side manifest; the standard
conditions hold. P3 may follow P2 or run beside it; both reuse P1 and neither depends on the
other.

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

### P6. D4 mere-comms moves into Murm

**Paused 2026-11-06 (Mark) pending the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md); unpaused the same day by D7 to D9.** The scene vocabulary (`sceno`, `scenotime`) now lives in `merely-made/graphshell`, below Murm, so Signalman's `sceno` edge is legal. What is left of P6 is `mere-comms` and F1's `scenomise` edge.

Done when `crates/shell/comms` (package `mere-comms`, lib `comms`; `serde`, `async-trait`,
optional `murm` and `misfin`, dev `identity`, `transport`) lives in Murm; Mere's `ports/moot`
and `ports/graphshell` (`comms` feature) consume it from `murm.git`; Murm's `signalman` takes
`comms` from its own workspace. **Signalman's `portable-scenes` feature still names Mere
`sceno` and `scenomise`, so the cycle is not broken until F1 is closed;** P6 is done when
F1 is closed and the standard conditions hold with Murm's resolved graph free of `mere.git`.

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
- `bars.rs` with its 8 tests and the notation-view portion of `blocks.rs` live in Woodshed,
  built there as a signed wasm component pack, with the 8 tests passing in Woodshed (count
  before and after), and a conformance test in Woodshed that loads the real pack through
  Mere's host;
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

- **F1. Signalman's Mere scene crates.** `signalman`'s `portable-scenes` feature and
  `signalman-desktop` (its own workspace root) name `sceno`, `scenomise` and the Cambium
  chain from `mere.git`. D4 moves only `mere-comms`. Whether the scene crates become
  upstream of Murm (published, per the Cambium publish plan), the feature moves out, or the
  desktop's separate root is the accepted answer for the optional feature too, is not ruled.
  The [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md) offers a rule (scene vocabulary below Murm) and leaves
  `scenomise::backdrops` open. Answered in part 2026-11-06 by D7 to D9: `sceno` and
  `scenotime` are in `merely-made/graphshell`; `scenomise` stays in Mere (D9), so
  `signalman`'s `portable-scenes` still names it through `mere.git` until its `backdrops`
  slice is ruled a contract helper or the feature moves out.
- **F2. Gemot's dev edge into Distillery.** `distillery` takes `gemot`, `mien` and `mooting`
  as dev-dependencies, and `gemot` takes `eidetic-search` optionally. After P4 the loop
  Gemot, Eidetic, ESP, Eidetic is gone; whether a Gemot to Distillery dev edge is a cycle the
  check should refuse is a decision for the check's rule (dev edges), put to Mark when P0
  runs. The rule is ruled by D6; the Gemot to Distillery edge itself is still to be judged
  under it.

## Decisions

- **D1. Mere↔Knot and Mere↔Cleromancy: the resident stops depending (Mark, 2026-11-06):**
  "Resident stops depending". The resident hosts domains through a seam only: domain
  contract C5 puts the domain interface in Stickleback, in Eidetic, so domains depend on
  Stickleback and not on Mere's graph crates, and the apps register themselves at runtime, so
  Mere never names Knot or Cleromancy in Cargo. Put to Mark on the cycle map's C1 and C2,
  against keeping the two-phase repin loop and against moving the domain crates to a lower
  repository. Outcome: applies Stack 3 ("Mere consumes producers, never the reverse") and
  refines the source homes map's "Mere↔Knot and Mere↔Cleromancy already work this way".

  Links: area stack · draws on domain-contract C5, stack.3 · amends source-homes-map "Repository cycles" · plans cycle_breaks P1, P2, P3 · summary The resident hosts domains through a Stickleback seam and apps register at runtime, so Mere's Cargo names neither Knot nor Cleromancy.

- **D2. Eidetic↔Distillery: the ESP adapter moves to ESP (Mark, 2026-11-06):** "Move adapter
  to ESP". `eidetic-search` keeps only the `GoalEmbedder` trait; the ESP adapter
  (`goal_model_esp`), the MiniLM calibration tests and the recall example move to ESP; Eidetic
  stops depending on ESP. Put to Mark against leaving it and recording that `persistence`
  stays off in Eidetic's root. Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle_breaks P4 · summary eidetic-search keeps the GoalEmbedder trait; the ESP adapter, calibration tests and recall example live in ESP.

- **D3. gemot↔Murm: an address trait (Mark, 2026-11-06):** "gemot↔Murm: trait". An address
  trait in `murm-resolve`, implemented by Gemot, so Murm stops depending on `gemot-address`.
  Put to Mark against Murm duplicating Gemot's grammar, which conflicts with Gemot owning it.
  Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle_breaks P5 · summary murm-resolve owns an address trait that Gemot implements; Murm no longer names gemot-address.

- **D4. Mere↔Murm: mere-comms goes down (Mark, 2026-11-06):** "Mere↔Murm: mere-comms down".
  `mere-comms` moves into Murm, so signalman stops depending on Mere. Outcome: applies the
  resident split plan's item that the comms contract may stay while its concrete adapters
  belong to Murm and Errand, now with the whole crate moving. F1 records that this alone
  does not clear Signalman's scene crates.

  **Paused 2026-11-06 (Mark), pending the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md); unpaused by D7 to D9, which moved the scene vocabulary below Murm.**

  Links: area stack · draws on resident-split S6, stack.3 · plans cycle_breaks P6 · summary mere-comms moves into Murm so signalman's comms feature no longer names Mere.

- **D5. Mere↔Woodshed: adapters go up (Mark, 2026-11-06):** "Mere↔Woodshed: adapters up".
  The Woodshed adapters move out of Graphshell into Woodshed behind its own features, so Mere
  stops naming Woodshed. Outcome: applies the source homes map's reading of Pandect/Graphshell
  adapters as "adapters for other owners".

  Links: area stack · draws on stack.3 · plans cycle_breaks P7 · summary Woodshed's block-editing adapters move out of Graphshell into Woodshed behind its features; Mere no longer names Woodshed.

- **D6. Dev-only edges count (Mark, 2026-11-06):** "Count them". Dev-dependency edges are cycles in
  the family check: tests use fixtures, or live in the higher repository. Settles F2's check rule
  and the dev-only fixture in P0's tests (a dev-only cycle fails). Outcome: new.

  Links: area stack · draws on stack.3 · plans cycle_breaks P0 · summary Dev-only dependency edges count as cycles in the family check; tests use fixtures or live in the higher repository.

- **D7. Extract the contract leaves first (Mark, 2026-11-06):** "D: hybrid, leaves first", on
  `../research/2026-11-06_host_guest_contract_brief.md`. The clean contract crates move to a
  bottom layer now; the full host/guest split is decided later. Unpauses D4/P6 once the leaves are
  out. Outcome: new.

  Links: area stack · draws on cycle_breaks D4 · plans cycle_breaks P6 · summary The host/guest contract's leaf crates are extracted to a bottom layer first; the full split waits.

- **D8. The bottom layer is the revived Graphshell repository, named for the protocol (Mark,
  2026-11-06):** "we can use it as the bottom layer. unarchive, hard cut, i'm ok with that" and
  "graphshell = the protocol". `mark-ik/graphshell` was unarchived and transferred to
  `merely-made/graphshell`; it is cut over to hold only the contract, with no transitional copy
  left in Mere. The Graphshell reference host (`ports/graphshell`, `graphshell-mere`,
  `graphshell-host`) stays above Mere. Outcome: new.

  Links: area stack · draws on cycle_breaks D7 · plans cycle_breaks P6 · summary The revived merely-made/graphshell repository is the bottom protocol layer; the reference host stays above Mere.

- **D9. Contract plus host-role kit (Mark, 2026-11-06):** "Contract + kit". The bottom layer holds
  `chirograph`, one WIT world with `script/wire` and `witgen`, `sceno`, `scenotime`, `hest` (and
  `mere-curation` if guests need it), plus `graphshell-endpoint`'s default surface and
  `graphshell-client`; the endpoint's `door`, `profile` and projection-provider features, and
  `scenomise`, stay in Mere. Outcome: new.

  Links: area stack · draws on cycle_breaks D8 · plans cycle_breaks P6 · summary The bottom Graphshell layer holds the contract crates and the host-role kit, not the runtime-bound features.

- **D10. The WIT world reaches Mere's hosts through `links` metadata, for now (Mark,
  2026-11-06):** "Accept for now". `graphshell-endpoint` declares `links` metadata for its one
  WIT world; `document-host` and `app-host` copy it into their git-ignored manifest-relative
  path in `build.rs`, and `settings-wit` reads it directly, so these crates carry an otherwise
  unused normal dependency on `graphshell-endpoint`. Revisit together with the open question
  of renaming `mere:script@0.1.0`. Outcome: new.

  Links: area stack · draws on cycle_breaks D9 · summary Mere's WIT hosts take the one world from graphshell-endpoint's links metadata until the package rename is decided.

- **D11. The extraction's structural choices stand (Mark, 2026-11-06):** "Keep all three".
  The endpoint's door features are the Mere crate `graphshell-door`, beside `graphshell-host`;
  the frozen-scene tests that need Mere crates are the Mere crate `graphshell-client-tests`;
  the AccessKit conversion is `uxtree::frozen::FrozenUxTree` behind `uxtree`'s `frozen`
  feature. Outcome: new.

  Links: area stack · draws on cycle_breaks D9 · summary graphshell-door, graphshell-client-tests and uxtree's frozen feature are the accepted homes for what the extraction left in Mere.

- **D12. Woodshed is a guest, not a compiled-in adapter (Mark, 2026-11-06):** "D: Woodshed as a
  guest". The generic block pane (Djot documents, rhyme schemes, save, undo, reopen) stays in
  Mere. Woodshed's part (progression and bars, the example, the Chart/Tab/Score notation view,
  MusicXML export) leaves Mere and is loaded at runtime as a reviewed guest component, so no Mere
  manifest or lock names Woodshed. Chosen over moving both files into Woodshed (option A, which
  needs the private web page state made public and a Woodshed-built product), moving only
  `bars.rs` (B, which leaves the edge) and dropping editable blocks (C, which loses the pane).
  Amends D5's means, not its end: Mere stops naming Woodshed. Outcome: new design, written first
  and held for Mark's review.

  Links: area stack · draws on cycle_breaks D5, D8, D9, host_guest_contract_brief · amends cycle_breaks D5 · plans cycle_breaks P7 · summary The block pane stays in Mere and Woodshed's progression and notation parts load as a reviewed wasm guest, so Mere names no Woodshed package.

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

## Keeping cycles out: the check

`scripts/check_family_cycles.py`, with tests beside it in the `test_cargo_mode.py` style.
Illustrative design, not compile-ready code.

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
  this plan's decision (`cycle_breaks D1`) or an isolated-root reason (the signalman-desktop
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
