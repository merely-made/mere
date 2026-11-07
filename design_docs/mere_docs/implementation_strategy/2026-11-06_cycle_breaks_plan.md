# Cycle breaks plan

**Status:** 2026-11-06: in progress. D1 to D5 settle the break for each open
family dependency cycle; the Genet/Murm cycle needs no break (Findings). Two forks remain
open (F1, F2). D6 (dev edges count) added the same day. D7 to D9 (Mark, 2026-11-06) ruled
the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md) and
unpaused D4 and P6: the contract leaves and the host-role kit left Mere for the revived
`merely-made/graphshell` the same day (Progress, Findings); `mere-comms` into Murm is still
to do, and F1 is half answered. D14 (Mark, 2026-11-06) added P1b and left the distribution binary open. D2 landed as P4 and D3 as P5 (2026-11-06, Progress); P6's `mere-comms` half landed and F1's measurement is in Findings, the phase stays open. D1 proceeds; D5 became D12 (Woodshed as a guest, design written, held for review).

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
and the standard conditions hold. Which authority forms the registration carries (linked
closure, guest, declared policy) waits on D14 (a).

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
conditions hold. P3 may follow P2 or run beside it; both need P1b and neither depends on the
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

**Landed 2026-11-06** (Progress; Findings, P5 and P6 findings).

### P6. D4 mere-comms moves into Murm

**Paused 2026-11-06 (Mark) pending the [host/guest contract brief](../research/2026-11-06_host_guest_contract_brief.md); unpaused the same day by D7 to D9.** The scene vocabulary (`sceno`, `scenotime`) now lives in `merely-made/graphshell`, below Murm, so Signalman's `sceno` edge is legal. What is left of P6 is `mere-comms` and F1's `scenomise` edge.

Done when `crates/shell/comms` (package `mere-comms`, lib `comms`; `serde`, `async-trait`,
optional `murm` and `misfin`, dev `identity`, `transport`) lives in Murm; Mere's `ports/moot`
and `ports/graphshell` (`comms` feature) consume it from `murm.git`; Murm's `signalman` takes
`comms` from its own workspace. **Signalman's `portable-scenes` feature still names Mere
`sceno` and `scenomise`, so the cycle is not broken until F1 is closed;** P6 is done when
F1 is closed and the standard conditions hold with Murm's resolved graph free of `mere.git`.

**`mere-comms` half landed 2026-11-06; open for F1** (Progress; the F1 measurement and options are in Findings, P5 and P6 findings).

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
- the guest-blocks open questions R1 (the drawing), R2 (the view's home) and R3 (the native shell)
  are answered;
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

  Links: area stack · draws on cycle_breaks D1, domain-contract C5 · amends cycle_breaks P1 · plans cycle_breaks P1, P1b, P2, P3 · summary P1's decisions b to f accepted and P1b added; where the installed resident carries Knot's and Cleromancy's authority is open and back at the drawing board.

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

  Links: area stack · draws on cycle_breaks D12, guest_blocks · plans cycle_breaks P7 · summary Mark's eight guest-blocks answers: the protocol WIT carries block-guest, Woodshed's adapter is a woodshedding feature, a native block pane is built, and the painting question is back with Mark after the spike.

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

