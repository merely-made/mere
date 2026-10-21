# Hest: one command architecture

**Date:** 2026-10-20.

**Status (2026-10-20):** H4 is implemented and software-qualified through
Mesquite/Graphshell receiving and canonical Knot/Rhai command-producing fences.
Hest 0.1/0.2, command-menu 0.1.1 and Cambium 0.5 remain published; H4 uses their
existing command data contract and ships as Git adapters. Native acceptance
and backend package publication remain separate.

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

- **`hest::menu`** received `command-menu`'s content unchanged in H1:
  `Command`, `CommandSet`, `CommandChoices`, `MenuSession`. H2 extends declarations.
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
| H4 | Audit and script reuse through existing Hest data. | Mesquite invokes shared ids; an explicitly registered Rhai command evaluator produces the same requests. Graphshell receives them through its actual canvas reducer; Knot exercises the consented block-evaluation host. |

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
   The original H1 instruction held Git pushes and publication. Mark's H3
   authorization below lifted the Git hold. The later release authorization
   lifts the publication hold; Hest 0.1 precedes its shim, and Hest 0.2
   precedes Cambium 0.5.

## Risks

- **Scope creep.** Each phase has its second-host gate; without one, a piece does not
  move.
- **Import paths.** `command_menu::` appears in five dependent manifests and their
  sources; the re-export crate keeps them compiling during the move.
- **The Scenograph editor plan** (SE45 to SE50) names `command-menu` and the catalogue;
  those references update at H1, not before.

## Delivery history

The original qualification packets retain exact commands, tested sources,
failed attempts, reconciliations and host ownership. This summary replaces
the repeated chronological prose; those evidence files are unchanged.

| Checkpoint | Result and source basis | Owned evidence |
|---|---|---|
| H1 | Pure move and thin shim, frozen 0.1 API; Graphshell viewer and Turnstone palette exercises. | [H1 qualification](../../../support/receipts/hest-h1-20261020/qualification.json) |
| H2 | Effect/destructive declarations on prepared Hest 0.2; probe and isolated Turnstone gate. | [H2 qualification](../../../support/receipts/hest-h2-20261020/qualification.json) |
| Canonical facade adoption | Cambium 0.5 and canonical Turnstone/Knot intake, preserving unrelated supplier locks. | [Merged adoption](../../../support/receipts/hest-cambium-adoption-20261020/reconciled/qualification.json), [later supplier reconciliation](../../../support/receipts/hest-cambium-adoption-20261020/supplier-reconciliation-2/qualification.json) |
| H3 | Invocation/terminal outcome, Turnstone key reducer, Knot actual window queue, Chirograph wire. Supplier `9ef5e3026`; complete Turnstone rerun 1107 passed/15 ignored; Knot desktop/chrome/document 417 passed/2 ignored. | [H3 qualification](../../../support/receipts/hest-h3-20261020/qualification.json), [Git delivery and fresh remote source resolution](../../../support/receipts/hest-h3-20261020/delivery.json) |
| Publication | Mark authorized “Fix, proceed. Commit, push, merge, publish, prune.” Published Hest 0.1.0, command-menu 0.1.1, Hest 0.2.0, Cambium 0.5.0 in dependency order. Downloaded checksums and all 88 source/test files match release commits `561188d5a`/`615b35e2e`. | [Release qualification](../../../support/receipts/hest-release-20261020/qualification.json), [registry source proof](../../../support/receipts/hest-release-20261020/registry-publication-proof.json) |

H3 retains terminal-only semantics: Chirograph acceptance, window queue
admission, handled clicks and refreshed readings are not `Done`. The host
establishes completion. Native GUI/window-manager, fresh-process native reopen,
human AT acceptance and a fresh MSRV qualification were not part of publication.

The publication repair corrected rootstock Rustdoc links and made Cambium's
backdrop fixture package-local without changing runtime behavior. Registry-only
old/new facade checks passed. All four release trees stayed exact through
concurrent native Hest/material/receipt merge `9fe4a7ef0`; delivery reached
`4214af802`. Historical expanded-rootstock and package-test failures remain in
their packets with the passing replacements. No whole-tree doc-clean claim is made.

The excluded `crates/cambium/hest-0_1` package remains available to reproduce
original path-based receipts. The shim remains on Hest 0.1.x, preserving
published Cambium 0.4.0. H4 does not modify those source trees or published APIs.

## H4: bounded script adapters and audit

Mark authorized continuation and subagent orchestration, with deduplication
where it improves readability. The implementation remains inside existing
owners; it adds no Hest runtime, audit service, generic validator or repository.

### Mesquite and Graphshell

- `Product::invoke` accepts Hest `Invocation` and returns `Option<Outcome>`;
  the default refuses. A script's `invoke <JSON Invocation>` enters that hook
  before the existing product-specific fallback. Legacy `act` remains a
  handled/recognized Boolean, with its existing behavior.
- Optional `mesquite.commands/v1` receipts contain bounded id/schema/effect,
  sequence and outcome observations. `null` means pending. Arguments are
  omitted, fields are UTF-8 bounded, and truncation/eviction are explicit.
  Defaults retain 128 records and 256 bytes per textual field. Hosts may
  tighten those limits. Runs that never invoke or request audit keep the
  previous receipt shape. This is run-local observation, not durable history.
- Graphshell's actual `TreeLane` re-reads declarations and refuses remote or
  resident ownership. Its portable `graphshell.viewer/v1` adapter validates
  id/schema/effect/empty arguments and current disabled state. Only local zoom
  in/out and physics toggle use the synchronous receiver; `Done` requires an
  observed requested model change. Retired offers are stale. Zoom clamps
  refuse before dispatch, preserving layout following.
- The duplicate named canvas-command match now delegates to the existing
  viewer mapping. Legacy `act view:fit` remains `Fit`; a hosted menu remains
  `FitVisible`. The old scenario subset and 40/42 zoom/pan values are preserved.
  Parsing, bounded receipt mechanics and receiving tests live in small modules,
  with only narrow hooks added to the large orchestration files.

### Rhai command data

- `script_rhai::command::collect_invocations` uses the existing sandbox and
  host-supplied `CommandSet`/schema. `command(id, arguments)` accepts UTF-8
  strings or exact blobs and takes the effect from the declaration. Unknown,
  disabled or over-limit calls refuse the batch, even if that binding error is
  caught. Uncaught parse/runtime/budget failures expose no partial batch.
- Every collection uses fresh engine/scope/batch state. Defaults bound retained
  requests to 256 and aggregate argument data to 64 KiB; the operation budget
  is explicit. These limits describe retained request data, not the entire
  Rhai heap. Subsequent host dispatch is not a transaction over the whole batch.
- `RhaiCommandEvaluator` is a separately registered/allowlisted `rhai-command`
  block evaluator. It returns plain JSON invocation data and never executes it
  or invents completion. A zero block budget selects a finite 100,000-operation
  cap. Ordinary `RhaiEvaluator::new()` and `rhai eval` remain unchanged and have
  no command binding. Inker still owns policy, failed/denied source preservation,
  generated provenance and the non-recursive evaluation pass.

### Qualification and delivery

Supplier checks pass: 39 Mesquite, 33 Rhai, 12 Graphshell command/pipeline,
73 Hest/Chirograph, 7 Inker evaluation and 25 retained-host scenario cases;
standalone Graphshell Web Wasm and scoped strict Rustdoc also pass. The
Graphshell fixture uses the real graph-backed canvas reducer, including a
Rhai request batch serialized through the same Hest envelope as Mesquite.
The second receiving host is Knot's real consented block-evaluation endpoint,
pinned to canonical Mere `e94cb23e8`. Three new H4 cases, both existing
consent/freshness and runaway-budget cases, nine command-chrome cases,
workspace all-targets and locked metadata pass. The two host exercises have
different authority: Graphshell enacts the validated synchronous request;
Knot only derives request data after consent. Neither script success nor
Knot admission is command completion. Excluded languages have no advertised
run action; runtime/budget failures preserve the source fence; edited source
makes the old request stale.

The H4 delta leaves released Hest/Cambium and frozen compatibility sources
untouched. Concurrent main shell/canvas/co-op work is preserved separately;
its new Cambium shell is not represented as the published 0.5 package. H4
ships as Git adapters; no new Hest or Cambium publication is needed. Mesquite
is already Git-only (`publish = false`). No unrelated backend or family release
is included. Native presentation/human AT and MSRV remain separate evidence.


[H4 qualification](../../../support/receipts/hest-h4-20261020/qualification.json),
[supplier checks](../../../support/receipts/hest-h4-20261020/checks.json),
[owned receiver references](../../../support/receipts/hest-h4-20261020/receivers.json)
and [source preservation](../../../support/receipts/hest-h4-20261020/source-preservation.json)
record the exact software basis. The receiver owns its complete logs at
`knot-editor/docs/receipts/hest-h4-20261020/qualification.json`; root does not
duplicate them. Knot preserves 1,246 unrelated package blocks and 150 other
Rust files; endpoint production logic is exact. The initial Rhai fixture
syntax attempt and Knot's incorrect denied-language expectation are retained
as superseded evidence beside their passing replacements.

Subagent implementation and independent review found no remaining actionable
H4 findings. Source was reconciled with concurrent shell/canvas/co-op work in
`4927aae86`; command receiving and standalone Wasm passed again on that merge.
Git delivery uses immutable supplier `e94cb23e8` and Knot `2cfe1d4`. The actual
TreeLane hook is Wasm-compiled; the receiving exercises are software tests,
not a headed browser run, native window-manager or human AT acceptance.

A final concurrent owner/producer merge through `52d235927` was preserved in
`da805a080`. H4 adapters remained exact; Mesquite's 39 cases, Graphshell's
12 command/pipeline cases and standalone Wasm passed again on that runtime.
[Post-merge checks](../../../support/receipts/hest-h4-20261020/post-merge-checks.json)
and [delivery](../../../support/receipts/hest-h4-20261020/delivery.json) keep this
recheck distinct from the immutable supplier/receiver qualification. Scoped
reference and D2 audits have zero actionable Hest findings (20 claims hold).
Pruning stale refs and expired worktree metadata found no task artifacts to
remove; unrelated worktrees and branches are preserved.


## Qualification follow-through, 2026-10-21

Fresh Linux MSRV qualification is complete for the published Hest 0.1.0 and
0.2.0, command-menu 0.1.1 and Cambium 0.5.0 packages. The downloaded archives
match the publication checksums. Rust 1.97.1, shipped package locks and all
features pass **324 tests in aggregate**, including the old/current registry
compatibility fixtures, with one existing ignored Cambium doctest. The
[qualification packet](../../../support/receipts/hest-qualification-20261021/README.md)
retains commands, logs, exact locks and resource limits. This qualifies those
published package graphs on Linux; it does not establish whole-workspace MSRV
or native/browser/human AT acceptance. H4 remains the final defined phase.

The same packet adds a limited headed Linux WebGL2 viewer pass at frozen source
`c66620ca`: Hest scripting changes zoom, and the retained menu, disabled feedback,
synthetic keyboard, pan/wheel and focus paths pass. Browser H4 audit acceptance
remains open: Graphshell's completion report omits Mesquite's command audit
records. Full-product browser packaging stalled at the unchanged 3 GiB cap;
saved-choice fresh-process reopen, fresh native presentation and human AT remain
unqualified. Upstream was pulled again through `b4ae63d82` before publication;
the frozen browser result is not transferred to that newer host code.
