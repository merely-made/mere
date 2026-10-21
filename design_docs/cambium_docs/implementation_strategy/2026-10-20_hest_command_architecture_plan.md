# Hest: one command architecture

**Date:** 2026-10-20.

**Status (2026-10-20):** H3 implementation is in progress under Mark's explicit
commit-and-push authorization. Cambium 0.5.0 and canonical Turnstone H2
adoption retain their recorded qualification. Hest 0.2.0 carries declarations; the frozen
Hest 0.1.0 and `command-menu` 0.1.1 shim preserve the H1 compatibility line.
All commits remain local. Push, publication and fresh remote-only resolution
are deferred; native GUI and human AT acceptance remain separate.

**Owner direction (Mark, 2026-10-20):** subordinate `command-menu` to `hest` and
centralize some form of command architecture under it. The name `hest` was picked
because the alternatives are taken on crates.io (`beck`, `beckon`, `behest`, `bid`);
`bidding` was free but generic.

The census below records the pre-H1 source at Mere `2fead1143`; its citations
are historical locations. Turnstone's `action.rs` (in its `src` directory) was
read from a clone of its `main`. Current H1 locations and receipts are recorded
under Findings and Progress.

## Why: the pieces exist and live apart

| What | Where | Role |
|---|---|---|
| `Command`, `CommandChoices`, `CommandSet`, `MenuSession`, the shared id catalogue | `crates/cambium/command-menu/src/lib.rs:26`, `:70`, `:81`; `session.rs:13`; `catalogue.rs` | Declare a command and offer it: one order every surface shares, the person's kept and recent choices, a palette's open state. Serde is its only dependency. |
| `ActionId`, the typed form of the catalogue ids | `crates/graph/graph-kernel/src/actions.rs:21` | Graph hosts match on it instead of strings. |
| Key bindings keyed by catalogue ids | `crates/system/registry/src/input.rs:7` | Bind keys to shared commands without knowing the host. |
| The destructive-action gate probe | `crates/system/ux-events/src/ux_probes/destructive_action_gate.rs` | Reads a caller-supplied list of destructive `ActionId`s: the "destructive" fact lives in the probe's argument, not on the command. |
| Cambium's drawn commands | `crates/cambium/cambium/src/command_surface.rs:16` (`CommandItem`), `:84`, `:122`, `:131`; `command_menu_bar.rs:38` | Render and route a command's events. |
| Window verbs | `crates/cambium/cambium-rootstock/src/window_verbs.rs:51`, `:88` (`WindowCommand`, `WindowCommands`) | A queue the host drains after each dispatch. |
| The intent contract | `crates/chirograph/src/lib.rs:118` (`IntentReference`), `:123` (`IntentEffect`), `:1032` (`IntentInvocation`), `:1043` (`IntentResult`); `titulus.rs:277` (`ActionFormV1`) | An advertised action with an effect class, a payload schema, and a result that can be refused or stale. |
| Portable host intents | `crates/shell/chrome/src/host_intent.rs:47` (`HostIntent`) | What a host's chrome can express to the runtime. |
| Typed app intents | Turnstone `action.rs:116` (`Action`) | Keys, a later palette and automation produce them. Host-owned. |

The ids are already shared (`graph-kernel`, `registry`, Cambium and the graphshell port
all depend on `command-menu`). What is missing is the middle: nothing says that this
id is declared once, offered here, invoked with these arguments and this effect class,
and recorded as this outcome. The invocation edge exists in three shapes today: a typed
enum, a string plus payload bytes, and a queue of window verbs.

## Shape

One crate, `hest`, serde-only and kernel-free like `command-menu`. Arrows point into it:
Chirograph, Cambium, `graph-kernel` and hosts adapt to it, never the reverse.

- **`hest::menu`** is `command-menu`'s content, moved unchanged: `Command`,
  `CommandSet`, `CommandChoices`, `MenuSession`.
- **`hest::catalogue`** holds the shared ids. `ActionId` stays in `graph-kernel` as the
  typed form.
- **Declare (H2)** adds to `Command` an effect class (curation, domain truth, external
  effect, mirroring `IntentEffect`, duplicated rather than imported) and a `destructive`
  flag, so the gate probe reads the set instead of a list.
- **Invoke (H3)** adds `Invocation` (id, opaque arguments, schema id) and `Outcome`
  (done, refused with a reason, stale), shaped to what Chirograph already says.
  Chirograph converts to and from `IntentInvocation` and `IntentResult`; `hest` does not
  depend on it.
- **Stays out:** drawing (Cambium), binding storage (`registry`), enactment (a host's
  reducer or `Action` enum), permissions (Chirograph, Dramatis).

## Phases, each gated on a second host

The port law applies: a piece moves only when a second host exercises it. A pure move
(H1) needs no new behavior, only that every existing receipt still passes.

| Phase | Move | Gate (second host and receipt) |
|---|---|---|
| H0 | Reserve the name. | Done: `hest` 0.0.1 published 2026-10-20. |
| H1 | Pure move into `hest::menu` and `hest::catalogue`. Prepare `hest` 0.1.0 and the `command-menu` 0.1.1 re-export (no yank; published Cambium 0.4.0 keeps resolving). Repoint `graph-kernel`, `registry`, `pandect`, Cambium and the Graphshell port. Commit locally; do not push or publish without later authorization. | Existing menu/model, `command_menu_compact`, action and input tests pass. The shim preserves the old public types and catalogue, including through published Cambium 0.4.0. Identify two actual host exercises separately from library checks. |
| H2 | `hest` 0.2.0: effect class and `destructive` on `Command`. The `command-menu` 0.1.x shim stays on Hest 0.1.x. | `ux-events`' gate probe reads them from the `CommandSet`; Turnstone registers its `Action`s with them. |
| H3 | `Invocation` and `Outcome`; window verbs and Turnstone's `Action` get ids. | Two hosts invoke through it: Turnstone's keys and Knot's window verbs; a Chirograph round trip proves the remote path. |
| H4 | Audit and script reuse. | Mesquite's scripted verbs and the Rhai block evaluator name the same ids. |

## Accepted rulings (Mark, 2026-10-20)

1. **Catalogue scope:** keep the entire existing catalogue, including graph ids,
   unchanged for H1. Namespaces describe verbs without importing graph types.
   Hosts choose their offered subset and retain their own ids in the open set.
   Preserve the existing admission rule: a second host offers the verb **or
   ux-events reasons about it**. A later namespace split needs a concrete
   consumer need and its own compatibility decision.
2. **Effect class:** in H2, define `Curation`, `DomainTruth` and `ExternalEffect`
   in Hest, retain Chirograph's representation, and convert exhaustively on
   the Chirograph side with round-trip checks. These represent one agreed
   taxonomy. `destructive` is independent of effect class; hosts retain
   permission, confirmation and execution policy. Hest remains serde-only.
3. **Versions and compatibility:** H1 prepares `hest` 0.1.0 and `command-menu`
   0.1.1. The shim re-exports all four public types and `catalogue`, and depends
   on `hest = "0.1.0"` (the 0.1.x line). H2 adds fields to an all-public
   `Command`, which breaks struct literals, so H2 requires **0.2.0**. The shim
   must not follow that breaking release.
4. **Delivery:** implement H1 as a pure move, run existing receipts and a shim
   compatibility check, identify two host exercises, and commit locally.
   Do not push or publish yet. Future publication order is Hest first, shim
   second, after separate authorization.

## Risks

- **Scope creep.** Each phase has its second-host gate; without one, a piece does not
  move.
- **Import paths.** `command_menu::` appears in five dependent manifests and their
  sources; the re-export crate keeps them compiling during the move.
- **The Scenograph editor plan** (SE45 to SE50) names `command-menu` and the catalogue;
  those references update at H1, not before.

## Findings (2026-10-20)

### H1 historical qualification

- The four public types live in `crates/cambium/hest/src/menu/mod.rs` and
  `crates/cambium/hest/src/menu/session.rs`; the entire catalogue lives in
  `crates/cambium/hest/src/catalogue.rs`. Only module imports and one intra-doc
  link change in the moved implementation and existing tests.
- `command_menu_compact` constructs `CommandItem`s directly. It protects
  Cambium's drawing/input behavior, but does not by itself exercise the
  moved `CommandSet` or `MenuSession`. Kernel action and registry input tests
  are library receipts, not independent host exercises.
- The compatibility fixture uses immutable registry Cambium 0.4.0 and local
  candidate overrides for Hest and the shim only. This is a prepublication
  check; a registry-only resolution of the new versions waits for publication.
- The two independent host exercises are Graphshell's retained viewer command
  model (`viewer_commands::tests`, through the port's `web` feature) and
  Turnstone's actual palette (`app::palette::command_menu_tests`). The compact
  Cambium harness remains a separate rendering/input receipt.
- At the H1 start, Turnstone's `main` was fetched as
  `e94e7381b39ebc1927d4bf258d3c92787f9abeca` and exported into an isolated
  fixture. All 244 Rust files are byte-identical to that commit. Its locked
  Cambium 0.3.3 at Mere `48fd3385` exercises Hest through the shim; the separate
  registry fixture covers published Cambium 0.4.0. Cargo `--locked` metadata
  verifies one `command-menu`, one Hest, and the actual Cambium → shim → Hest
  dependency chain. Only the two Cargo-generated candidate package entries
  differ from the consumer's lock; every unrelated entry is byte-identical.
  The manifest/lock patch and source hashes are retained in the receipt.

### H2 declaration boundary

- `crates/cambium/hest/src/menu/mod.rs` requires an explicit `EffectClass`
  when constructing a command and carries an independent `destructive` flag.
  The 36 catalogue ids and labels remain unchanged. Their declared effects
  flow into offered commands, including disabled and searched rows. Storage
  remains `CommandChoices`; its field names and defaults are unchanged.
- `crates/chirograph/src/command_effect.rs` owns exhaustive conversions in
  both directions. All three effect variants round-trip with identical Serde
  representation; Hest imports no Chirograph or graph types.
- `crates/system/ux-events/src/ux_probes/destructive_action_gate.rs` now
  receives the offered `CommandSet`. It reads each dispatched id's destructive
  flag while retaining its existing one-use confirmation-grant semantics.
  Controls flip the flag on an unchanged id to prove the declaration supplies
  the fact. This probe is observational; it does not enact or authorize actions.
- The H2 Turnstone exercise uses consumer `c889038` and its locked Mere
  `dffcc9c8` cohort. It changes the actual palette to register every offered
  `Action` with host-owned metadata; an exhaustive match classifies all 164
  action variants. Delete node, empty recycle bin and close session are
  declared destructive. The reducer and ring/confirmation policies are unchanged.
- Turnstone's fixture adapts only Cambium's command dependency, re-export and
  `From<&Command>` path to Hest 0.2. All 74 other Cambium Rust files, 242 of
  Turnstone's 244 existing Rust files and all 1,670 unrelated consumer lock
  entries are unchanged. The two modified receiver files and new declaration
  module are retained as an applicable patch, not a canonical Turnstone commit.
- Hest 0.1.0 is frozen byte-for-byte in `crates/cambium/hest-0_1`; Cargo
  resolves the shim to it and current consumers to 0.2.0. The compatibility
  fixture still uses immutable registry Cambium 0.4.0 and only overrides the
  two H1 candidates. At the initial H2 fixture snapshot, the breaking Cambium version and
  canonical adoption were deferred. The canonical continuation below now
  prepares 0.5.0 and its direct requirement edges.

### Canonical Cambium 0.5 adoption (2026-10-20)

- `crates/cambium/cambium/Cargo.toml` prepares 0.5.0 because its public command
  alias now exposes Hest 0.2's breaking declaration API. The root requirement
  and six direct wrapper requirements follow 0.5.0. Registry Cambium 0.4.0
  remains immutable and continues to pass the separate H1 shim check.
- Embedded Knot `65a6632` permits the widget subset in Cambium 0.4/0.5.
  Its desktop requirement and root lock stay fixed at 0.4. In an exact source
  export, 53 default cases pass against the old 0.4 cohort and 66 engine cases
  against local Mere's 0.5 cohort, with one existing ignore per run.
- Canonical Turnstone `9df2f0007` requires Cambium 0.5 and locks the Mere
  branch source to supplier `0151950f8`, with Knot `65a6632`. Its actual palette
  declares the effect/destructive facts through `src/action/command_metadata.rs`;
  only two existing Rust files change plus that new module. 242 existing Rust
  files and 1,604 unrelated consumer lock blocks remain byte-identical.
- Locked metadata verifies one Cambium 0.5, one Hest 0.2, a direct dependency
  edge between them and no `command-menu`. No source path override is present.
  The pre-existing exact source-edit revision exception remains. Mere's 1,761
  unrelated lock blocks, including both distinct Murm cohorts, are unchanged.
- Local Git-object fetches make the unpublished supplier commits available
  under their canonical URL/branch identities. Fresh remote-only resolution
  waits for authorized delivery of Knot, Mere and then Turnstone. Crates.io
  release order is Hest 0.1 before the shim, and Hest 0.2 before Cambium 0.5;
  no push or publication is performed by this continuation.

### Reproduce the bounded checks

Commands below are ready to run from the Mere repository root:

```sh
cargo test --locked -p hest@0.2.0 -p command-menu
cargo test --locked --manifest-path crates/cambium/hest-0_1/Cargo.toml
cargo test --locked -p ux-events --lib
cargo test --locked -p chirograph --lib command_effect::
cargo test --locked -p mere-kernel --lib tests::actions
cargo test --locked -p mere-registry --no-default-features --features input --lib input::
cargo test --locked -p cambium-genet-winit-host --test command_menu_compact
cargo test --locked -p cambium-genet-winit-host --test accessibility
cargo test --locked -p graphshell --no-default-features --features web --lib viewer_commands::tests
cargo test --locked -p graphshell --no-default-features --features web --test command_declarations
cargo test --locked -p pandect --lib a_view_set_now_waits_for_the_next_batch
cargo test --locked --manifest-path support/fixtures/hest-h1-compatibility/Cargo.toml
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --locked -p hest@0.2.0 -p command-menu --no-deps
cargo check --locked --manifest-path ports/graphshell/web/Cargo.toml --target wasm32-unknown-unknown --no-default-features --features presentation-webgpu
```

For H2 Turnstone, export consumer `c889038` into an isolated directory and
export its Cambium crate at locked Mere `dffcc9c8` into a second directory.
Apply the [Cambium adapter patch](../../../support/receipts/hest-h2-20261020/cambium-adapter.patch)
there and the [Turnstone fixture patch](../../../support/receipts/hest-h2-20261020/turnstone-fixture.patch)
to the consumer, using `git apply --unidiff-zero` for both. Adjust the absolute
local paths in these prepublication patches for another workspace, including
the build cache path. Then run `cargo metadata --locked --offline` and
`cargo test --locked --offline --lib app::palette::command_menu_tests`.
The [source/lock proof](../../../support/receipts/hest-h2-20261020/turnstone-source.json)
records the exact dependency chain and finite source changes. H1's original
patch instead applies to its named `e94e738` consumer and the H1 supplier tree.
These receipts cover CPU model/integration and headless rendering;
desktop/browser presentation and native restart were not exercised.

## Progress (2026-10-20)

- Accepted all three design rulings plus the H2 0.2.0 compatibility boundary.
  Implemented the mechanical move, the thin shim and the five active consumer
  import/dependency changes. Existing test assertions are retained.
- **H1 local qualification:** 52 tests pass, zero fail: the original menu/model
  tests (13), kernel actions (3), registry input (14), compact menu (1),
  Graphshell viewer commands (5), Pandect stored choices (1), shim compatibility
  against published Cambium 0.4.0 (3), and Turnstone's existing palette tests
  (12). Both named host exercises pass. Strict Rustdoc link validation passes.
  The initial Pandect discovery filter matched no tests and is excluded; its
  exact stored-choices test supplies the admitted receipt.
- [Qualification and command outcomes](../../../support/receipts/hest-h1-20261020/qualification.json),
  [pure-move source hashes](../../../support/receipts/hest-h1-20261020/pure-move.json),
  and [Turnstone source/dependency proof](../../../support/receipts/hest-h1-20261020/turnstone-source.json)
  retain the finite evidence. Cargo emits the same nonfatal source-discovery
  diagnostic as the Turnstone baseline for Servo's deliberately malformed lint
  manifest; locked metadata and the final test command both exit zero.
- The Hest plan has no scoped mechanical doc-audit findings. The D2 parser's
  original batch-95 status line was repaired, and a new scoped H1 judgment
  records this continuation. Repository-wide documentation findings outside
  this work remain; no full-tree clean audit is claimed.
- H1 was committed locally, then rebased to `a68de7098` over fetched main
  `42baea78b`. Only the batch-95 D2 record needed conflict resolution; no product
  source conflicted. Fresh reconciliation checks pass: 45 model/integration
  tests, including eight existing accessibility checks, plus the three
  published-Cambium shim checks. The original Turnstone receipt retains its
  original consumer revision and is not represented as a fresh canonical repin.
- H2 began on the 0.2.0 line. The 0.1.0 source is frozen in
  `crates/cambium/hest-0_1`, an excluded standalone package, while the active
  workspace Hest advances to 0.2.0. The shim continues to depend on 0.1.x.
  This bounded compatibility source is needed until the two unpublished H1
  candidates can be delivered without breaking published Cambium 0.4.0.
  H2's two exercises are the declaration-driven `ux-events` probe and an
  isolated Turnstone receiver; canonical Turnstone changes remain separate.
- Push, publication, registry-only new-version resolution and native/browser
  presentation remain deferred.

- **Second reconciliation:** main advanced during qualification. Both local
  commits were rebased onto fetched Mere `b10ae5f88`; H1 is now `cea6d09b2`.
  The only conflicts were lockfile rows adding Hest beside upstream Illume;
  both dependencies were retained. Turnstone independently committed its
  AccessKit cohort update as `c889038`; the receiving export was refreshed to
  it before the final checks. No canonical Turnstone source was edited.
- **H2 local qualification:** 118 tests pass, zero fail: Hest 0.2 model (15),
  declaration-driven probe (40), Chirograph effect round trip (1), kernel
  actions (3), registry input (14), compact menu (1), accessibility (8),
  Graphshell retained viewer (5), actual browser-page declarations (1),
  Pandect stored choices (1), frozen Hest 0.1 model (13), published-Cambium
  shim compatibility (3), and actual Turnstone palette (13). The standalone
  WebGPU/Wasm source check and strict Rustdoc links pass. All final test and
  source checks use locked dependencies; the newly referenced upstream
  Woodshed revision was fetched before retrying the offline checks.
- [H2 qualification](../../../support/receipts/hest-h2-20261020/qualification.json),
  [command outcomes](../../../support/receipts/hest-h2-20261020/checks.json),
  [tested source](../../../support/receipts/hest-h2-20261020/tested-source.json),
  [frozen H1 source](../../../support/receipts/hest-h2-20261020/h1-frozen-source.json)
  and [receiving proof](../../../support/receipts/hest-h2-20261020/turnstone-source.json)
  retain the evidence. The original H1 receipts remain historical, with their
  original source basis; their 52-test total is not added to the H2 count.
- H3 invocation/outcome and H4 scripting are unstarted. Push, publication,
  facade release preparation, canonical Turnstone adoption and native/browser
  presentation remain explicit delivery or later-phase gates.

- **Canonical adoption continuation (Mark, 2026-10-20):** reconcile, prepare
  Cambium's breaking version and adopt the qualified declarations in canonical
  Turnstone. Commit locally, preserve unrelated work and do not push/publish.
  Mere reconciled cleanly onto fetched `c5db7d691`; H1/H2 are now `f9ff6156d`
  and `60b527b31`. Turnstone starts clean at fetched `0dd9b1b`.
- Cambium advances from 0.4.0 to 0.5.0, with the root requirement and six
  direct wrapper requirements following it. The immutable published-0.4.0
  compatibility fixture and Hest 0.1/shim versions stay fixed. Canonical
  Turnstone will keep the canonical Git URL/branch identity and lock to the
  local supplier commit; local cache seeding supplies the unpublished object
  for receiving checks. No local-path override becomes its portable contract.

- The embedded `knot-document` requirement was a necessary prerequisite:
  `^0.4.0` kept a second Cambium family in Mere and could not accompany the
  canonical 0.5 receiver. Knot `65a6632` permits `>=0.4.0, <0.6.0` for its
  unchanged widget subset. Its desktop requirement and root lock remain 0.4.
  Mere follows that small local prerequisite commit; all six Knot package
  source rows move together. A collateral Murm resolver refresh was
  discarded; all unrelated lock blocks retain their exact prior contents.

- **Canonical adoption qualified:** supplier `0151950f8`, embedded Knot
  `65a6632` and canonical Turnstone `c210fa05b` are committed locally.
  Supplier checks pass 405 tests; standalone Knot passes 119 across the two
  compatibility cohorts, with one existing ignore per run. The actual
  Turnstone full default library passes 1103 with 15 ignored,
  zero failures, run serially with `TMPDIR=/tmp`. Its 14 focused menu/sidecar
  cases are included in that full-library count. Workspace all-targets,
  portable metadata, standalone WebGPU/Wasm and strict Rustdoc links pass.
- [Canonical qualification](../../../support/receipts/hest-cambium-adoption-20261020/qualification.json),
  [consumer checks](../../../support/receipts/hest-cambium-adoption-20261020/turnstone-checks.json),
  [supplier source](../../../support/receipts/hest-cambium-adoption-20261020/supplier-source.json),
  [consumer source](../../../support/receipts/hest-cambium-adoption-20261020/turnstone-tested-source.json)
  and [dependency proof](../../../support/receipts/hest-cambium-adoption-20261020/turnstone-cohort.json)
  retain the actual canonical receiving evidence. The previous H1/H2 fixture
  receipts retain their original cohorts and are not represented as reruns.
- Push/publication, remote-only delivery qualification and native GUI/human AT
  remain held. H3/H4 remain unstarted; this adoption changes declarations and
  registration, without adding invocation/outcome or execution authority.

- **Final concurrent reconciliation:** preserved published Mere `d95e04bb4`
  (family promotion ownership notes), Knot `d30dff6` (separate supplier
  manifests) and Turnstone `1d13185` (retained-renderer pins and geometry
  fixtures) through merge commits. The qualified local supplier `0151950f8`
  and active embedded Knot `65a6632` remain unchanged; the unrelated newer
  Knot supplier-manifest cohort was not repinned into Turnstone.
- Turnstone merge `8e3e5cb` preserves NetRender `54d864c2` and Genet `6ef3cd37`.
  Fresh merged-tree checks pass: 14 menu/sidecar, 15 Inspector and 23 embedded
  Knot authoring cases (5 existing ignores); all-targets; portable metadata;
  and the complete library with 1103 passed, 15 existing ignores,
  zero failures. The focused cases overlap the full-library count.
  Consumer receipt commit `9df2f0007` changes documentation/evidence only
  after that tested merge. Source and lock preservation are measured against
  incoming Turnstone `1d13185`: 242 unrelated Rust files and 1,604 unrelated
  package blocks remain exact.
- [Final merged qualification](../../../support/receipts/hest-cambium-adoption-20261020/reconciled/qualification.json),
  [merged command outcomes](../../../support/receipts/hest-cambium-adoption-20261020/reconciled/checks.json)
  and [merged tested source](../../../support/receipts/hest-cambium-adoption-20261020/reconciled/tested-source.json)
  retain the final result separately from the first passing cohort. No push
  or publication occurred. Native GUI/human AT and H3/H4 remain separate.

- **Further supplier reconciliation:** merged incoming Mere `d4756e0fb` and
  Knot `3a7f9a9`, preserving the unrelated retained-rendering, site-model and
  embedding work. Before this evidence appendix, all 52 incoming-only files
  and 151 local-only files match their respective parents byte for byte.
  Hest, the frozen shim source, Cambium, Chirograph and ux-events code remain
  unchanged from the qualified local parent. The six Knot lock entries retain
  `65a6632`; Cargo consequently retains seven older embedding dependency
  records alongside the incoming cohort and qualifies ambiguous dependency
  names. Final locked offline metadata and 405 supplier test executions pass.
  Knot merge `2b118e3` preserves incoming root dependencies and Rust sources;
  its locked metadata passes. Turnstone `9df2f0007`, its source and its active
  supplier pins remain unchanged: the prior 1103-pass full-library result is
  retained, without representing it as a fresh rerun.
  [Supplier reconciliation receipt](../../../support/receipts/hest-cambium-adoption-20261020/supplier-reconciliation-2/qualification.json)
  records the exact tree, checks, preservation and dependency consequence.
  H3/H4 are unstarted. Push, publication, remote-only new-version receiving
  and native GUI/human AT remain held or separate.


## H3 implementation and delivery (2026-10-20)

Mark authorized H3, local commits and Git pushes. Crate publication remains
held. H3 adds API without changing the prepared Hest 0.2.0 declarations or the
frozen 0.1 compatibility line.

- Hest owns `Invocation` (id, schema id, opaque bytes and declared effect) and
  terminal `Outcome` (`Done`, `Refused`, `Stale`). An invocation grants no
  authority. Hosts decode arguments and recheck their current declaration.
- Chirograph owns the observation and routing context and checks the command
  against its advertised id, schema and effect. Its existing wire remains
  unchanged. `IntentResult::Accepted` has no terminal Hest outcome; terminal
  `Done` cannot be encoded as acceptance in that dispatch-only result. Stale
  epoch/revision context remains in Chirograph. Receipt admission or a handled
  click is not evidence of completed enactment.
- The portable window queue carries Hest invocations and drains through the
  existing `WindowCommand` event-source seam. Its adapter owns the window ids
  and versioned argument schema (resize carries two little-endian f64 values).
  These ids remain outside Hest's shared catalogue until its offer admission
  rule is met. Queue admission produces no terminal completion; close policy
  and actual window-manager handling retain authority.
- Turnstone binds static declared commands to a host-owned session/pane/graph/
  selection observation, validates it before entering the existing reducer,
  and returns stale/refusal without enactment. Its canvas key route uses this
  adapter; text input and contextual producer actions keep their dedicated
  paths. Only observed synchronous physics/isometric view changes report
  `Done`; deferred work such as saving remains without a terminal outcome.
- Receiving gates are Turnstone's actual canvas key binding through its app
  reducer, Knot's actual desktop caption/window queue harness, and a command
  round trip through Chirograph's real carrier request vocabulary. Existing
  model, probe, host, compatibility and consumer checks remain required.
- Delivery order is the compatible Knot prerequisite, Mere supplier, then
  final Knot and Turnstone consumers. Canonical URLs and committed locks are
  preserved. No release publication is included in the push authorization.
