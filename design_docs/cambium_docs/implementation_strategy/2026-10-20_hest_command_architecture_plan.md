# Hest: one command architecture

**Date:** 2026-10-20.

**Status (2026-10-20):** H1 implemented and qualified locally; publication and
push are explicitly deferred. `hest` 0.1.0 and the `command-menu` 0.1.1 shim are
local candidates. Published releases remain `hest` 0.0.1, `command-menu` 0.1.0,
and Cambium 0.4.0.

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
- Turnstone's current `main` was fetched as
  `e94e7381b39ebc1927d4bf258d3c92787f9abeca` and exported into an isolated
  fixture. All 244 Rust files are byte-identical to that commit. Its locked
  Cambium 0.3.3 at Mere `48fd3385` exercises Hest through the shim; the separate
  registry fixture covers published Cambium 0.4.0. Cargo `--locked` metadata
  verifies one `command-menu`, one Hest, and the actual Cambium → shim → Hest
  dependency chain. Only the two Cargo-generated candidate package entries
  differ from the consumer's lock; every unrelated entry is byte-identical.
  The manifest/lock patch and source hashes are retained in the receipt.

### Reproduce the bounded checks

Commands below are ready to run from the Mere repository root:

```sh
cargo test --locked -p hest -p command-menu
cargo test --locked -p mere-kernel --lib tests::actions
cargo test --locked -p mere-registry --no-default-features --features input --lib input::
cargo test --locked -p cambium-genet-winit-host --test command_menu_compact
cargo test --locked -p graphshell --no-default-features --features web --lib viewer_commands::tests
cargo test --locked -p pandect --lib a_view_set_now_waits_for_the_next_batch
cargo test --locked --manifest-path support/fixtures/hest-h1-compatibility/Cargo.toml
RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --locked -p hest -p command-menu --no-deps
```

For Turnstone, export the named consumer commit, apply the retained fixture
patch there with `git apply --unidiff-zero`, and run `cargo test --locked --offline --lib app::palette::command_menu_tests`.
The fixture patch uses this workspace's absolute shim path; adjust it when
reproducing elsewhere. Keep the consumer Rust source and unrelated locked
packages unchanged. These receipts cover CPU model/integration and headless
rendering; desktop/browser presentation and native restart were not exercised.

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
- H1 is ready as a local commit. Push, publication, registry-only new-version
  resolution and native/browser presentation remain deferred. H2 has not begun.
