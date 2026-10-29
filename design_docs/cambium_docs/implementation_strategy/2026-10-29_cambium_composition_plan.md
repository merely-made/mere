# Cambium composition: live component instances inside an editable composition

**Status:** 2026-10-29: plan. Nothing is built. Findings were read against
Mere `9f8618e90` and Turnstone `a1a52f3`; no test was run for them.

**For:** compositions of reusable controls that a user can rearrange,
regroup, hide and save while each control's live state (an unapplied draft, a
selection) survives, together with the diagnostic facts that explain and test
that behaviour. The first consumer is Turnstone's Settings pane: reorder rows,
group them, move a group into a drawer, hide a group, save the arrangement, all
with an unapplied edit intact. Opened by Mark's 2026-10-29 request: "turn this
into a concrete design plan for Cambium, using Turnstone's settings pane as the
first consumer. spell out the lifecycle and binding rules, and which changes
actually need to reach Meristem", with diagnostics added to the same brief.

Not here:
- Moving a whole pane between tabs, floats and windows. `RetainedMount` does
  that and Turnstone uses it (F3).
- One Cambium application inside another:
  [`2026-10-06_app_composition_brief.md`](../research/2026-10-06_app_composition_brief.md).
- A visual authoring tool. This plan builds the model and edit operations it
  would drive; the editing UI in P3 is a minimal one.
- Selective update scheduling (per-component dependencies, deferral). P4's
  events are shaped so that work can report through them.
- Upstream Xilem: `5d72ad41..9f504ae7` changes nothing functional in
  `xilem_core` or `xilem_web` (checked 2026-10-29; see
  [`upstream-xilem.md`](../technical_architecture/upstream-xilem.md)).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8).

## Constraints

- No speculative cross-tree key registry; one is built only for a named
  consumer whose state must survive a tear-out
  ([presentation plan](2026-10-28_cambium_presentation_plan.md), portable tiles
  constraint).
- Components stay in Cambium; Meristem stays the renderer-neutral diff and
  message core (`crates/cambium/cambium/src/component.rs:5-14`).
- Surfaces Meristem cut for lack of a family consumer (environment, proxy,
  side effects) come back only with a consumer
  ([`upstream-xilem.md`](../technical_architecture/upstream-xilem.md)).
- Settings truth and validation stay with the provider. A draft is
  component-local and never enters the caller's state; only an applied
  `SettingValue` crosses (`crates/cambium/cambium/src/setting_row.rs:5-16`).
- Diagnostic payloads use fixed vocabulary. Setting values, draft text and
  labels never enter an observation (Turnstone
  `src/diagnostic_observations.rs:18-21`).

## Findings (2026-10-29)

- **F1. Settings rows are positional.** Turnstone builds the rows as a
  `Vec<Box<dyn AnyView>>` inside `el("div", rows)` (`src/settings_pane.rs:288-300`).
  Meristem's `Vec` sequence is index-based with a generation per index
  (`meristem/src/view_sequences/impl_vec.rs`), and every row has the same
  concrete type, so a row's state follows its index, not its setting.
- **F2. A draft can move to another setting.** `setting_row` re-derives the
  draft only when the spec's `value` or `control` changes
  (`setting_row.rs:274-278`); a changed `spec.id` keeps it. The apply closure
  captures the current id (`settings_pane.rs:273-285`). With F1, inserting or
  reordering rows moves an unapplied draft onto a neighbouring setting with the
  same control and value, and Apply writes it there. Latent today: as read,
  the provider's list has fixed membership and order
  (`src/settings_provider.rs:353`). Any reorderable composition makes it live.
- **F3. Retained owners are proven.** `RetainedMount` parks a withdrawn runner
  in a hidden container rather than removing it, and only the owner dispatches
  input; hosts validate custody and use window-scoped layout
  (`retained_mount.rs:7-13, 32-53`). Tests cover destination-first mounting,
  source-first withdrawal with GC and remount, and release after runner state
  (`retained_mount.rs:185, 256, 313`). The Settings pane is built on one
  (`settings_pane.rs:355, 372-412`).
- **F4. Portable children depend on traversal order.** `PortableKeyed`
  preserves a cross-parent move only when the source sequence rebuilds before
  the target (`portable.rs:24-28`). The nursery lives in `GenetCtx` and is
  drained at the end of a rebuild (`context.rs:224-260`), so postponing cleanup
  alone does not help a destination visited first.
- **F5. Erased replacement is intentional.** `AnyView` tears down and bumps a
  generation when the concrete type changes, so old messages go stale
  (`meristem/src/views/any_view.rs:100-130`). `ViewId` is meaningful only to
  the parent that issued it (`meristem/src/view_ctx.rs:9-23`).
- **F6. Component memoization exists.** A memoized component skips its body
  while props are equal and no message has dirtied local state
  (`component.rs:102-110, 183, 222`).
- **F7. The action-silencing adapter is written twice.** `setting_row`'s
  `silence` and Turnstone's title bar (`src/chrome_view.rs:1072-1078`) both map
  `MessageResult<()>` to another action type by hand.
- **F8. Multi-window updates rebuild every projection.** `update` rebuilds all
  projections; only `update_local` rebuilds one (`multi.rs:177-200`).
- **F9. The diagnostics store already has causal fields.** Armillary
  `apparatus-core`'s `ObservationMetadata` carries `operation`, `cause`
  (a `RecordRef`), `subject`, `semantic_revision` and `presented_frame`
  (armillary `crates/apparatus-core/src/observation.rs:49-61` at `06340b0`).
  Turnstone copies `AppEvent` categories into it and states it has no
  operation correlation (`src/diagnostic_observations.rs:1-5`). Mesquite
  exposes the batch type (`crates/cambium/mesquite/src/lib.rs:94`). Neither
  Cambium nor Meristem produces observations.
- **F10. Turnstone persists pane view-state.** Every registered pane, Settings
  included, has a view schema (`turnstone.settings.view`) and `Blueprint`
  persistence (`src/panes/registry.rs:124-127, 238, 466-480`).

## Model

Four things that the current code partly conflates:

| Term | Meaning | Settings example |
|---|---|---|
| Source | What is presented, owned by its provider | `SettingsRef` + setting id `ui.zoom` |
| Instance | One live control and its interaction state | the row editing `ui.zoom`, with its draft |
| Placement | Where the instance is shown now | second row of the "Display" group |
| Kind + config | What the instance is and how it presents its source | `setting.row`, compact label |

A **composition** is the saved description: a table of instances (each with
kind, config and binding) and a placement tree that names instance ids. The
first placement tree is small: ordered groups, one drawer, and a hidden set.
Reusing workbench's split/tab tree is deferred (Q4).

### Lifecycle rules

An instance is in one of four states: **placed** (attached to one placement,
may receive input), **withdrawn** (retained, not presented), **closing**
(held for undo of a deletion) or **closed**.

- **L1.** Only a composition edit or the owner creates or closes an instance.
  Rendering never creates one; a placement that names an unknown instance is a
  rejected composition.
- **L2.** An instance has at most one placement. Showing a source twice means
  two instances bound to it, each with its own state.
- **L3.** A move is withdraw-and-attach inside one transaction. State survives
  whichever placement is visited first (P2 makes traversal order irrelevant).
- **L4.** A withdrawn instance receives no pointer or key input, cannot hold
  focus, and has no IME session. Withdrawal blurs it first, with focus going
  where the host's existing focus rules send it.
- **L5.** A withdrawn instance keeps its state until it is closed (Q2).
- **L6.** Deleting an instance moves it to closing; undo restores the same
  instance. When it leaves the undo horizon it closes (Q3).
- **L7.** Closing discards any outstanding draft and reports it (P4).
  Teardown releases view state before its DOM, as `RetainedMount` already
  does (F3).
- **L8.** Changing an instance's kind replaces it: close, then create, with
  stale messages for the old one (as F5). A kind may declare a migration from
  a named earlier kind; nothing else carries state across kinds.
- **L9.** Saving persists the composition only. Live state (drafts,
  selections, scroll) is restored only from a snapshot its kind explicitly
  supports. Settings rows support none. Meristem's view state is never a
  persistence format.

### Binding rules

- **B1.** A binding names a source by provider identity. Instance identity is
  separate, so retargeting keeps the instance and moving keeps the binding.
- **B2.** A draft belongs to one instance under one binding. Retargeting an
  instance resolves the draft by its kind's policy; for `setting.row` the draft
  is discarded and re-derived from the new spec. A draft never carries to
  another source (fixes F2).
- **B3.** Apply writes to the binding the draft was made under. If the binding
  changed in between, B2 has already discarded the draft.
- **B4.** An external write to the bound value (another instance, another
  window, a provider reload) re-derives the draft, as today, and reports the
  discard (Q1).
- **B5.** A binding whose source stops being described leaves the instance
  placed but unavailable: shown as such, its draft inert and not appliable,
  until it is rebound or closed.
- **B6.** The provider validates bindings. A composition binding to a source
  the provider does not describe is rejected, and the accepted composition
  stays.
- **B7.** Two instances bound to one source keep separate drafts; applying one
  is an external write to the other (B4).

### Edits and their treatment

| Edit | Treatment |
|---|---|
| Move, regroup, or change surrounding layout | Same instance, state kept (L3) |
| Change a compatible display option | Update config as props; memoized body reruns |
| Hide a group | Withdraw its instances (L4, L5) |
| Change the bound source | Same instance, draft resolved by B2 |
| Change kind | Replace (L8) |
| Duplicate | New instance; binding copied, state not (L2) |
| Delete | Closing, then closed (L6, L7) |
| Invalid composition | Rejected with a reason; accepted composition stays |

## Design

### Cambium: a composition view that owns its instances

The central choice is that the composition view keeps a **flat instance
table** keyed by instance id, rather than nesting each instance's state inside
the group that currently shows it. Placement is then a DOM move of an element
the table already owns, and no group's rebuild decides whether an instance
lives. This is what removes the traversal-order dependence of F4 for authored
compositions, inside Cambium, without a general transfer protocol in Meristem.

Each rebuild of the composition view:
1. Diffs the previous composition against the new one into the edit set above.
   Validation runs first; a rejected composition leaves everything as it was.
2. Builds, rebuilds or tears down instance views from the table. Each instance
   routes under a `ViewId` minted from its table slot and a generation, so
   moves do not change its message path and a closed or replaced instance's
   messages go stale (F5 semantics).
3. Builds the placement containers, then attaches each placed instance's
   element under its container as one atomic move, as `PortableKeyed` already
   does with `ElementSplice::adopt_pending`
   (`meristem/src/element_splice.rs:61`). Withdrawn and closing instances go to
   a hidden parking container, as `RetainedMount` does (F3).
4. Publishes input routes and focus changes after placement, so a withdrawn
   instance is never a routing target (L4).

Instances are ordinary views, usually `Component`s, inside the pane's one
runner. A `RetainedMount` owner per instance is not needed at this
granularity; it remains the tool for whole panes across windows.

Composition types (instance table, binding, placement tree, edit set,
rejection) are generic in Cambium. Kinds, their configs and their bindings are
registered by the application, which also supplies the provider check for B6.

### Turnstone: the Settings pane as a composition

- Rows become instances of kind `setting.row`, bound to `(SettingsRef,
  setting id)`; the default composition is today's order in one group.
- Edits in the pane: move a row up or down, move it between groups, create a
  group, move a group into the drawer, hide or show a group, reset to default.
- The composition saves in the pane's view-state (`turnstone.settings.view`,
  F10). Drafts never persist (L9).
- The provider and `apply_value` path are unchanged.

### What reaches Meristem

Nothing in P0–P3 changes Meristem. The composition view uses `View`, `AnyView`,
`ViewId`, message generations and `ElementSplice` as they are; the `State`
and `Action` generics stay.

Changes that could reach Meristem, each only on its own case:
- **An observer hook** (P4). Two facts happen inside Meristem's own code:
  `AnyView` replacing state, and a message turning stale. Reporting them needs
  a no-op-by-default method on the context trait that Cambium's `GenetCtx`
  implements. The alternative is to report only what Cambium decides (Q5).
- **A silencing combinator.** F7's adapter is renderer-neutral and has two
  call sites; a named combinator beside `map_message_result` would replace
  both. Optional and small.
- **A general transfer protocol** for arbitrary nested retained children,
  destination first. Not needed for compositions, which own their instances.
  The no-speculative-registry constraint applies until a consumer outside a
  composition needs it.

### Diagnostics

Each layer reports the decisions only it can explain, into the existing
`apparatus-core` store (F9), with shared identity:

| Field | Carries |
|---|---|
| `operation` | one composition transaction or one input dispatch |
| `cause` | the record that triggered this one |
| `subject` | `cambium.instance` / instance id |
| `semantic_revision` | composition revision |
| `presented_frame` | the host frame that showed the result |

| Producer | Facts |
|---|---|
| Meristem (Q5) | stale message; `AnyView` replacement |
| Cambium | transaction accepted or rejected (with reason); instance created, attached, withdrawn, retargeted, replaced, closed; draft discarded (with reason); memoized body skipped or rebuilt |
| Genet | mutation classes that needed style, layout or paint work (whether Genet can supply these per operation is unverified) |
| Host | when and in which window the result was presented |

Cambium emits into a sink the host provides; Turnstone's
`diagnostic_observations` collects. The same stream is the test surface:
"moving the group kept instance 7" is no created or closed record for 7 and
one withdrawn plus one attached record under one operation.

## Phases

### P0. Bind drafts to their setting

- `setting_row` re-derives its draft when `spec.id` changes, not only value or
  control.
- Turnstone keys settings rows by setting id (`Keyed`) instead of position.
- **Done:** a test inserts and reorders specs while a draft is open on a row
  whose neighbour has the same control and value; no draft appears on another
  setting, and Apply writes only the setting the draft was made for. Holds for
  the Arrange pane's `setting_row` uses too.

### P1. Composition model and transaction

- Generic types and validation in Cambium; the diff from one composition to
  the next produces the edit set.
- **Done:** a unit test per row of the edit table; an invalid composition
  returns a rejection and an empty edit set, and the accepted composition is
  unchanged.

### P2. Composition view

- The flat-table view of the Design section, with parking and per-instance
  routing.
- **Done:** tests show that a move preserves view state and DOM node with the
  destination group built before and after the source; a withdrawn instance
  gets no pointer or key event and loses focus; messages to a closed or
  replaced instance are stale; a kind change replaces state.

### P3. Settings pane adoption

- The Turnstone section above, with a minimal editing UI.
- **Done:** Turnstone tests show that, with an unapplied draft open: reorder,
  regroup, move to drawer, hide and show keep the draft; retargeting a row
  discards it; an invalid composition leaves the pane usable; saving and
  reopening restores the arrangement and no draft. Turnstone's
  `scenarios/rung4_settings.scn` and `rung4_settings_verify.scn` still pass.

### P4. Diagnostics

- Cambium's events and sink; Turnstone collection with the metadata fields
  above; the Meristem hook if Q5 says so.
- **Done:** tests assert causal chains from the store: an apply in one window
  re-deriving a draft in another is one operation with a cause link; a hidden
  group's instances report withdrawn and receive no dispatch; a move reports no
  close. An inspection line renders through `inspect_batch`.

## Open questions

- **Q1.** On an external write, keep today's behaviour (re-derive and report)
  or keep the draft and mark the conflict? Recommendation: re-derive and report.
- **Q2.** Withdrawn retention: until close, or release when hidden?
  Recommendation: until close.
- **Q3.** Undo of deletion: hold closing instances for the composition's undo
  depth, or offer no undo of deletion? Recommendation: hold for the undo depth.
- **Q4.** Placement vocabulary: a small group/drawer tree first, or workbench's
  split/tab tree from the start? Recommendation: the small tree.
- **Q5.** Add the Meristem observer hook, or report only Cambium's decisions?
  Recommendation: add it in P4, no-op by default.

## Decisions

None yet.

## Progress

- 2026-10-29: plan written.
