# Cambium composition: retained components, bindings and selective updates

**Status:** 2026-10-29: P0 and P1 locally qualified. The shared composition
model/transaction, revision-aware settings contract, and Turnstone's shared
settings owner, guarded adapter and binding/request admission passed focused
offline qualification. P2–P6 remain open. P0/P1 supplier/consumer receiving used a
temporary local source override; publication and consumer repinning remain
separate from these source changes. Earlier findings retain the pre-P0 baseline.

**For:** reusable controls that retain local state while the user rearranges,
regroups, hides and saves them. Turnstone's settings pane is the first consumer.
The requested selective-update and causal-diagnostics work is included.
Mark requested the concrete design on 2026-10-29 and then asked to reconcile
this plan with the presentation plan. He ruled on fourteen of its choices the
same day (Decisions, D1–D14); the other technical choices remain proposed
defaults.

The [presentation plan](2026-10-28_cambium_presentation_plan.md) owns browser
tenants, UA widgets, whole-pane/portable-tile movement and host qualification.
The [diagnostics plan](../../mere_docs/implementation_strategy/2026-10-28_diagnostics_and_accessibility_plan.md#m13-reactive-ui-decisions-and-causal-references)
owns admission, collection and presentation of producer evidence. This plan
defines what composition must emit and how its behavior is tested.

Not here: arbitrary scripting/code hot reload, a general visual authoring tool,
cross-pane row transfer or a replacement workbench/window model. Application
embedding stays in the [app composition brief](../research/2026-10-06_app_composition_brief.md);
upstream maintenance stays in [upstream-xilem.md](../technical_architecture/upstream-xilem.md).

Run evidence stays outside the plan. Local runs under Code use the workspace
guarded launcher, stable lane and retained Code/output evidence policy.

## Reconciled choices

The overlapping lifecycle and scheduling material has moved here from the
presentation plan; its former RC stages are replaced by this plan's P0–P6.
The original Q1–Q5 alternatives are folded into these proposed contracts.

| Disagreement | Reconciled proposal and reason |
|---|---|
| Flat instance table or runner/mount per row | One flat retained table in the existing pane runner. Placement changes do not own lifetime; ordinary rows share focus, capture and dispatch authority. Whole panes keep RetainedMount. |
| External writes and rebinding discard drafts or preserve them | Preserve dirty drafts. An outside change shows no live conflict; Apply's revision guard refuses a stale write and offers theirs or yours (D6). A dirty row cannot be rebound until its edit is applied or discarded (D7). |
| Hold deleted instances for undo or destroy on confirmed deletion | Resolve dirty/pending work before confirmed deletion, then dispose. Undo recreates a clean incarnation. Live tombstone retention is deferred; hide and move still retain the same instance. |
| Composition in pane config or pane view-state | Authored structure/bindings in turnstone.settings.config; remembered expansion/scroll under the pane's view-state policy. Drafts and runtime machinery never persist. |
| Selective scheduling excluded or included | Included as P4, using explicit dependencies and bounded dirty entries. Commands remain individually delivered. |
| Meristem context hook or existing tracing | Start with structured events through existing tracing. A new observer API needs demonstrated necessity; composition itself requires no new core lifecycle or transfer protocol. |
| Cause/frame fields already sufficient or missing producer links | Existing facilities are reused, but producer references, bounded multi-parent joins and exact host evidence must be added and tested. |

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
- Settings truth and validation stay with the provider. Draft storage remains
  component-local; an explicit Apply sends the value and its binding/revision
  guards as an intent. This extends the current value-only boundary in
  `crates/cambium/cambium/src/setting_row.rs:5-16` without moving drafts into
  application truth.
- Diagnostic payloads use fixed vocabulary. Setting values, draft text and
  labels never enter an observation
  (`turnstone/src/diagnostic_observations.rs:18-21`).

## Design

### First consumer and scope

Turnstone's settings pane should allow a person to start editing a setting,
reorder the row, move it into a group or drawer, hide and reveal that group,
move the containing pane between windows, and apply the original draft to the
original setting. Saving the arrangement must not apply or erase that draft.
Another committed settings change refreshes a clean row and leaves a dirty
row's edit in place; the conflict surfaces when Apply's guard is refused (D6).
It must never silently replace the edit.

The first composition vocabulary is rows, ordered groups and collapsible
groups/drawers within the settings pane. Whole-pane docking, floating and
window assignment continue to use Turnstone's existing frame model. A row
does not become a catalogued product surface merely because it has a component
identity. Cross-pane row transfer, arbitrary scripting, a visual Rust editor,
code hot reload and a general plugin marketplace are outside this pilot; rows
move only within one pane (D5).

The implementation has three authorities:

- Turnstone owns settings truth, write validation, persistence, the settings
  source and the authored pane configuration. A component submits an intent.
- Cambium owns the retained UI instance, local draft, mounting, subscriptions
  and the choice of which component bodies to evaluate. Its scheduler asks
  the host for a wake/frame; the host remains the presentation authority.
- Meristem owns view reconciliation and message routing within each retained
  tree. Genet owns DOM and resulting style/layout/paint work. Existing
  diagnostics collect the facts each owner emits.

### What already exists, and what is new

| Capability | Current source | Work needed for this consumer |
|---|---|---|
| Component-local state and equal-props memoization | [component.rs](../../../crates/cambium/cambium/src/component.rs) | Keep independently editable components in a flat owner table outside the placement tree; expose lifecycle and skip evidence. Memoization currently skips a body only after its parent has produced the next view. |
| Whole-pane retention and same-document moves | [retained_mount.rs](../../../crates/cambium/cambium/src/retained_mount.rs), `turnstone/src/settings_pane.rs`, `turnstone/src/shell/renderers.rs` | Keep this whole-pane boundary. Inside it, the composition view owns ordinary row elements and naturally sized placement wrappers; pane-filling retained mounts are not allocated per row. |
| Typed runner plus erased host integration | [runner.rs](../../../crates/cambium/cambium/src/runner.rs), [surface.rs](../../../crates/cambium/cambium/src/surface.rs) | Keep one runner for the settings pane and route its composition children by stable table slot/generation. RetainedSurfaceSession remains the whole-pane host boundary. |
| Keys and move operations | [portable.rs](../../../crates/cambium/cambium/src/portable.rs), [keyed.rs](../../../crates/cambium/cambium/src/keyed.rs), [element_splice.rs](../../../crates/cambium/meristem/src/element_splice.rs) | Keep ordinary keyed children for ordinary local lists. `PortableKeyed` uses a per-context nursery and pass lifetime; it does not establish cross-runner owner retention or hidden lifetime by itself. |
| Pane identities, binding vocabulary and saved configuration | `turnstone/src/panes/blueprint.rs`, `turnstone/src/panes/registry.rs`; [workbench](../../../crates/cambium/workbench/lib.rs) | Add the settings composition payload inside the existing settings `PaneConfig` schema. Do not replace the frame/space model or create another settings store. |
| Typed settings controls and provider projection | [setting_row.rs](../../../crates/cambium/cambium/src/setting_row.rs), [settings contract](../../../crates/system/surface-api/settings.rs) | Add guarded binding/commit and dirty/conflicted draft behavior. P0 resets a replacement id as well as changed value/control. `SettingsProvider::apply` still has no expected revision; conflict-preserving reconciliation remains P1/P3 work. |
| Scoped updates and observations | [multi.rs](../../../crates/cambium/cambium/src/multi.rs), [Registry collector](../../../crates/system/registry/src/observations.rs) | Named dependency revisions, a bounded dirty-component queue, causal producer events and a product-installed bridge. Existing `update_local` is manual scoping, not dependency tracking. |

One correctness prerequisite precedes exposing reorder: today's settings rows
are a positional `Vec` of the same view type, and draft reconciliation checks
value/control, not binding identity. Swapping two rows with equal control/value
can therefore retain the old draft at an index while installing the new row's
Apply closure. P0 fixes this within the current list; P2 must preserve the
same guarantee through group moves in the composition view.

### Identity, ownership and runtime shape

The following names describe proposed contracts, not existing Rust APIs.

| Identity | Lifetime and meaning |
|---|---|
| Component key | Durable key in a pane's authored composition, scoped by its `PaneId`. Survives reorder and save/load. Never derived from row position, label, current value or DOM address. |
| Live instance | Component key plus a runtime owner incarnation. Stable through moves and hiding; changes after disposal/recreation, including restart. A duplicate receives a new key and instance. |
| Binding | Source owner and owner incarnation, `SettingsRef`, setting id, schema and a binding generation. Identifies what the instance edits. A data revision advancing is not a rebind. |
| Placement | Desired group/slot plus current window/document custody and a placement generation. Identifies where the instance can receive input. Changes independently of binding. |
| Dependency revision | Named domain, source incarnation and monotonic owner-issued revision. Used for freshness; not a view path or a polling counter. |
| Update identity | One scheduler evaluation of a live instance at a captured dependency vector and local/config generations. Connects invalidation to reconstruction and mutations. |

`ViewId` remains a local routing-path segment. `NodeId`, a setting id, a
`probe_id` attribute and a chrome `SurfaceId` are not interchangeable with a
live component identity. Diagnostic subjects include their owner namespace.

The composition view owns a flat table keyed by durable component keys,
inside the pane's existing runner (D1). Each entry retains its current view, opaque
view state (including typed component-local state), element, subscriptions and
pending freshness independently of the group that presents it. The placement
tree names these entries; group reconstruction does not recreate their bodies.
Duplicate active references to one instance are rejected before applying a composition;
two appearances editing the same setting use separate instances and drafts.
Changing the registered component kind is an explicit replacement, with the
same dirty-state resolution as removal and a new live incarnation. It cannot
reuse opaque local state just because the authored key is unchanged.

The proposed implementation is one composition view with two separate structures:
its logical owner table and its presentation containers. A composition
transaction validates the complete next descriptor set before changing the
accepted model. The composition's build/rebuild realizes admitted instances;
visiting a placement never implicitly creates or destroys one.

1. Validate instance keys, kinds, bindings, limits and unique placements;
   compute the accepted edit set. A refusal leaves the current model intact.
2. Reconcile only new, explicitly replaced or invalidated owner-table entries.
   Keep their view state even if their presentation group is replaced.
3. Give each entry a table slot plus generation for Meristem routing. Local
   moves keep that path; slot reuse or kind replacement advances its generation.
   This route is internal and is never persisted as the component key.
4. Create/reconcile presentation containers, then move owned elements to their
   current containers or an owned hidden parking root. Withdraw retained
   descendants before removing an obsolete group; its teardown must not
   recursively delete elements the table still owns.
5. Commit placement/input admission and restore eligible focus after the DOM
   transaction. A failed attachment must roll back or explicitly park the
   owner without losing its state.

Use Cambium's existing DOM move operations and element contracts where they
apply; a splice's optional adoption method is not a blanket guarantee for
arbitrary placement trees. P2 proves both traversal orders, wrapper replacement,
natural row sizing, scroll and DOM collection. A generic Meristem transfer
registry is not needed: this composition already owns every moved child.

All row input travels through the existing pane runner. The composition checks
live slot/generation, binding and current placement admission before forwarding
to a row. Tab order follows the presented DOM order, not table allocation
order. Hidden live nodes must be excluded from focus traversal, keyboard,
pointer and accessibility action admission even though they pass a liveness
check. Normal event propagation inside the pane remains intact; no second
row-runner dispatcher claims the same listeners.

The whole pane continues to use RetainedMount and its existing host session.
A runner/mount per row is a future alternative only if a demonstrated
independent-host boundary needs it. The flat table avoids multiplying runner
focus, capture and scheduling authorities for ordinary controls.

### Lifecycle and movement rules

Visibility, freshness and draft status are separate state dimensions. Hiding
is not destruction; being stale is not a new instance.

| Transition | Required behavior |
|---|---|
| Absent → allocated/parked | Validate kind/config/binding, allocate one live incarnation, realize its retained view/state in the owner table, establish dependencies and keep its DOM under owned parking. Do not grant input custody yet. |
| Parked → mounted | Catch up against a coherent owner snapshot, attach once, publish the new placement generation, then admit input and current semantics. On failure show an explicit unavailable state, not stale actionable controls. |
| Mounted A → mounted B | Validate the complete target first. Transfer custody under the same live instance, binding, draft and subscriptions. Source teardown cannot withdraw a node already adopted by B. Reject queued input addressed to A's old placement. |
| Mounted → hidden/parked | Remove hit testing, keyboard/AT action admission and active focus; retain local state and latest pending dependency revisions. Continue adjudicating already-started operation completions. Ordinary hidden rendering stops. |
| Bound → rebound | Use the binding transaction below. Old subscriptions and requests lose admission when the binding generation changes. Moving alone never invokes this transition. |
| Live → retiring → disposed | Resolve dirty/pending-edit policy first; revoke input and async admission, unsubscribe, remove queue entries, teardown the view/resources once, then release mount custody. Disposal is terminal for that incarnation. |

Moves are composition transactions: validate uniqueness, destination and
document identity before detaching anything, then commit placement atomically
from the input coordinator's perspective. Both source-first and
destination-first reconciliation must preserve the same owner. A refused
move leaves the old placement live. Partial attachment failure returns the
owner to a valid old station or explicit parking; it cannot drop the draft.

Same-document moves preserve DOM identity, local selection and scroll state.
The target host recomputes bounds, clips and accessibility mappings. Active
pointer capture is cancelled before transfer. In v1 a move commits an active
IME composition, as a focus loss would, and then moves (D4); native
composition is not assumed transferable. The host must be able to force that
commit, which nobody has checked for Genet. Cross-window focus is restored only
after target custody and native focus admission. Hidden controls are absent
from the active accessibility projection even while their DOM is retained.

`RetainedMount` requires the same shared document. Moving a live owner to a
different document or process is refused in this pilot. A future transfer
would need an explicit state export/import contract and a new live incarnation;
serializing Meristem's opaque `ViewState` is not that contract. Whole-pane
window custody and its headed qualification remain PT1 in the
[presentation plan](2026-10-28_cambium_presentation_plan.md#pt1-retained-dom-lane-owners-portable-tiles-p1-re-grounded-2026-10-29).

Collapsing a group or switching an inactive tab hides its owners; they keep
their state and stop rendering (D2). Removing a
row or closing/destroying its pane retires them. Proposed pilot deletion policy:
a dirty row offers Apply, Discard or Cancel; a pending Apply must settle or be
cancelled at its actual owner before deletion. Undoing a move/group edit keeps
the live instance. Undoing a confirmed deletion recreates a clean instance from
the saved descriptor, with a new incarnation (D3); v1 promises no deleted-draft
resurrection and retains no unbounded tombstone cache.

### Binding and draft rules

A row holds committed value/spec, the source revision guard, local draft and
local edit generation. Draft states are clean, dirty, applying, conflicted or
unavailable; a row becomes conflicted only when Apply is refused (D6). These
are UI states; the provider alone declares write success.

| Event | Draft and binding outcome |
|---|---|
| Edit | Change only local draft, increment local generation and request an interactive refresh. No settings write. |
| Equal inputs / placement change | Retain draft, binding and local control state. No source read or body evaluation merely to move. Layout can still change. |
| New source revision while clean | Adopt the coherent current value and guard. Update the control only where its inputs changed. |
| New source revision while dirty | Preserve the draft and the guard it was edited against; show no live conflict (D6). A changed write scope surfaces when Apply is refused; changed control semantics follow the incompatible-schema row below. Refreshing a label alone does not reset the editor. |
| Apply | Capture instance, binding generation, target owner incarnation, expected write revision, local edit generation and a unique request id. Validate and commit at the owner. Keep the draft until an acknowledged result. |
| Successful Apply | Publish only after persistence succeeds. Acknowledge the exact committed value/revision. Mark clean only if the current draft still matches the captured edit generation; edits made during an asynchronous Apply remain dirty. |
| Conflict / validation / storage failure | Preserve the draft and show the typed reason. Conflict offers theirs (discard and adopt the current value) or yours (review, then retry against a fresh guard). No automatic last-writer-wins retry. |
| Source disappears / becomes read-only | Preserve the draft as unavailable, disable Apply, revoke old admission and show the reason. A returning source with a new owner incarnation requires revalidation. |
| Schema/control becomes incompatible | Do not reinterpret old editor state as the new type. Preserve the old draft for resolution; rebuild a clean control only after discard or a declared, tested migration. |

Rebinding is explicit. Resolve and validate the proposed target first. A clean
row can adopt it with a new binding generation. A dirty row cannot be rebound:
the rebind control is disabled, with its reason, until the edit is applied or
discarded (D7). V1 has no implicit "carry this draft to the new setting"
operation. Replace subscriptions as one
binding transaction and mark the new target stale. A failed resolution leaves
the old binding intact. Late responses must match instance, binding, owner and
request generations before installation; movement does not invalidate a
logical completion whose binding is still current.

The first UI binds only to the application settings source. Generic rebind
rules still need a second fake provider in tests, so a captured closure cannot
silently keep applying to `APPLICATION_REFERENCE` after a future retarget.

The settings contract needs an opt-in revision-aware companion to
`SettingsProvider`: read one coherent snapshot with owner incarnation and
domain revisions, and apply with an expected guard returning committed,
conflict, unavailable or validation/storage failure. Names/signatures are to
be finalized in P1. An adapter without this capability remains usable through
the existing API but cannot claim guarded writes; no default method may
pretend a read followed by `apply` is atomic.

Turnstone must make all participating settings panes and theme-choice writes
use one application-owned authority (D8), rather than independent cached providers.
Begin with named revision domains for appearance choice, shell chrome,
behavior settings and the theme catalog. A row subscribes only to its relevant
domains. The owner returns the write guard for the domain being edited; this
may conservatively conflict on another edit in that domain. Revisions advance
on committed changes, not on `describe` calls, polls or layout work.

Check the guard, validate against the current spec, persist the candidate,
advance revisions and publish as one serialized owner operation. The current
provider reloads the durable base before saving, which protects unrelated
fields from an older pane snapshot, but it is not compare-and-set protection
for a competing edit to the same field. The pilot's guarantee covers writers
routed through that authority. Audit other writers in P1: if another process
can write the same file concurrently, a storage-owner transaction/locking
protocol is required before claiming that scope. A pre-write file read or
atomic rename alone is insufficient. Persistence stays with the existing
Pandect application-settings path/API, not a Cambium side store.

A remote edit proposal through Murm's optional live interaction layer (Murm's
`design_docs/2026-10-29_secure_sessions_and_live_interaction_plan.md`, O2–O3)
would be one more writer: its exported object submits through this authority,
never the provider. Request ids are correlation only today, and a redelivered
Apply is refused by its stale guard. Before remote writers are admitted, a
redelivery must return its original outcome, which is the domain-owned
idempotency that plan's O3 asks for.

Keep theme-library editing distinct from selecting/applying a theme. A catalog
refresh updates available definitions; it does not silently adopt saved bytes
under the currently selected theme id. Preserve Turnstone's existing explicit
Apply behavior and its live appearance snapshot.

### Composition editing and persistence

Store the authored tree and component descriptors in the existing
`turnstone.settings.config` pane payload: versioned row/group kinds, stable
keys, ordered placements, setting bindings and supported presentation options.
Use existing pane/view persistence policy for remembered expansion and scroll
(D9).
Runtime instances, DOM ids, callbacks, subscriptions, pending requests and
unsaved setting drafts are not serialized. Saving the composition leaves live
drafts untouched; restart restores the composition with fresh owner snapshots
and clean drafts.

The product registers the allowed component kinds and validates their
configuration. V1 needs a settings row and group/drawer (D10), not a global registry
of all application objects. Reject duplicate placements, cycles, unsupported
versions and malformed bindings before replacing the live composition. Unknown
kinds or temporarily missing settings retain their authored descriptor and
show an unavailable placeholder; they are not silently discarded on save.
Put explicit row/depth/config-size limits on loading and editing. For a newly
authored binding, reject a target the provider does not admit. A previously
saved or live binding that becomes unavailable retains its descriptor and
placeholder; this does not authorize Apply to an unknown target.

Reorder, regroup, collapse, insert and remove are typed composition commands.
They edit the authored model and invalidate its structural projection without
reconstructing unchanged row bodies. Apply remains a separate settings command.
Offer keyboard move/group controls in addition to drag, with accessible
announcements of the new position. The first implementation preserves source
order when deriving defaults and deterministically adds newly available settings
without rekeying existing rows. The empty existing config remains a supported
input; introducing authored contents needs a schema-owned migration, not a
change to the generic `PaneConfig` container.

### Selective updates and scheduling

P4 stays in this plan (D11) and is an outline until P2 lands (D14). Its
detailed design is written from P2's result; the 2026-10-29 draft is in this
file at Mere `01fce8a78`, a starting point rather than a commitment.

- Dependencies are declared per owner as named domain revisions. No global
  application-change counter and no automatic per-field signal.
- At most one queued evaluation per live instance; repeated invalidations
  merge. An unchanged-input skip avoids both the view factory and the
  component body; a memoized child skip is a different outcome, counted
  separately.
- Commands are never coalesced: two clicks dispatch twice.
- Hidden owners defer and catch up before they become actionable.
  Continuous input cannot starve an ordinary update.
- Each flush presents one coherent owner snapshot. `update_deferred` stays a
  paint-only escape hatch, not this queue.

### Diagnostics are part of the contract

The collection/presentation owner is the existing
[diagnostics plan](../../mere_docs/implementation_strategy/2026-10-28_diagnostics_and_accessibility_plan.md),
especially M7, M8, M11 and the UI-producer slice M13. A component inspector is
one consumer of those observations, not a new event store or the prerequisite
for emitting them. Record causes where decisions happen.

| Producer | Required facts for this slice |
|---|---|
| Settings owner | Request accepted/refused, target and expected/current revision, committed/failed outcome, advanced domains. Never draft or secret values. |
| Meristem | Message path and match/stale outcome; retained state reused, replaced or torn down at the core boundaries that actually make those decisions. |
| Cambium | Instance created/moved/hidden/rebound/disposed; dependency changed; invalidation merged; update deferred, unchanged-input skipped, started/completed; child memo skip; resulting mutation-batch reference. |
| Genet | Mutation batch admitted, which work it required (style/layout/paint or none), produced layout/render revision and failure. This is renderer evidence, not a Cambium guess from DOM-mutation counts. |
| Host | Window/surface incarnation, committed presentation revision and actual frame/submission/presentation outcome at the boundary available on that host. |

The rest of this section is an outline until P2 lands (D14); the 2026-10-29
detail is in this file at Mere `01fce8a78`.

- Events are versioned, with fixed reason enums. They carry the identities
  above, a producer sequence, an operation/request id and explicit causal
  references. Labels, setting values, drafts, bearer capabilities and
  invitation secrets never enter an event.
- Causes use producer references plus bounded cause-edge records for
  coalesced parents (D13), over apparatus-core's single `cause`. An
  unresolved reference stays an explicit gap; no parent is inferred from a
  timestamp.
- Turnstone extends `turnstone/src/diagnostic_observations.rs` with the
  producer stream through Registry's structured `Event` path, not a second
  inspector store. Collection loss never changes scheduling or command
  outcomes, and a test claiming that something did not rebuild proves zero
  loss over its interval.
- The chain is owner revision → invalidations → update → mutation batch →
  Genet work → named host frame, ending explicitly at the last proven link.
  These producers do not close M7's exact-frame gate.
- A cause arriving from a remote peer, if Murm's live interaction lands, is
  referenced by its session epoch and request id, never by a capability.

### Exactly what reaches Meristem

No lifecycle registry, settings binding, domain revision service, scheduler,
layout vocabulary or serialization belongs in Meristem. Existing `View`,
`ViewSequence`, `ViewState` and splice contracts suffice for the retained-owner
pilot; do not reintroduce unrelated upstream environment/effect machinery or
fork another Xilem backend for it.

The required core change for the diagnostic part is bounded structured
instrumentation at actual decisions: `AnyView` same-type reuse, type replacement
and teardown; its generation match/refusal; and message consumption/routing
boundaries in `MessageCtx` and relevant sequence implementations. Start through
Meristem's existing `tracing` dependency. Cambium enters an explicit
instance/update or message-operation span around each synchronous core call;
the installed bridge attaches its identity to nested core events. Asynchronous
work carries references explicitly rather than relying on an ambient span.

Core events contain only bounded routing/generation facts and outcomes, never
the message's `Debug` payload. Preserve `no_std` plus `alloc`, behavior with no
subscriber, and existing trait signatures. Truncating a deep path is explicit;
it must not be advertised as a unique identity. The trace bridge must be
nonblocking, nonreentrant and nonpanicking under normal sink failure.

Do not claim automatic observation of every arbitrary `View` implementation:
core adapters report their own decisions; Cambium's component/runner boundary
reports its lifecycle and full route outcome. A renderer-neutral optional
observer hook is a later alternative only if tracing cannot meet a demonstrated
coverage/overhead requirement. It is not a prerequisite to retained components.

Tracing is the ruled path (D12). Turnstone installs one global subscriber whose
default filter is `turnstone=info` (`turnstone/src/launch.rs:27-30`), which
drops Meristem's and Cambium's events. The bridge needs its own targets in
that filter, or its own layer.

### Change ownership and compatibility

| Owner / source | Planned change |
|---|---|
| Cambium `component.rs`, `setting_row.rs` | Retained component adapter, explicit draft status/commit acknowledgement, conflict-preserving reconciliation, lifecycle and memo evidence. Ordinary tree components remain supported. |
| Cambium `runner.rs`, `events.rs`; existing `retained_mount.rs` for the whole pane | Scheduled dispatch/flush, visibility-aware input/focus admission and update diagnostics. Row retention and natural sizing live in the composition view. `events.rs` is input routing, not the diagnostic store. |
| Cambium composition support | Flat retained instance table, stable route generations, bounded subscriptions/jobs, transaction validation and a small row/group/drawer placement tree. Workbench keeps whole-pane layout; no second split/tab model or domain store. Module split follows the pilot implementation. |
| Meristem `views/any_view.rs`, `message_context.rs`, relevant sequences | Structured reuse/replacement/teardown and route/match observations; no required public `View` redesign. |
| Mere `crates/system/surface-api/settings.rs` | Optional revision-aware snapshot/guarded-apply companion and typed result vocabulary. Binding semantics belong here and in Cambium, not in Meristem. |
| Turnstone `turnstone/src/settings_provider.rs`, `turnstone/src/settings_pane.rs` | One authoritative writer handle, coherent snapshots/revisions, guarded commit, retained row owners and settings composition commands. Preserve current storage and explicit theme-Apply semantics. |
| Turnstone `turnstone/src/panes/blueprint.rs`, `turnstone/src/panes/registry.rs`, `turnstone/src/shell/renderers.rs`, `turnstone/src/frame_controls.rs`, `turnstone/src/shell/appearance_editor.rs` | Wire the existing settings config/view schemas, keep owners across placement edits, route all participating writes and input, save/load composition and install diagnostics. Extend schema-owned payload handling rather than generic blueprint fields unless a real need is found. |
| Turnstone `turnstone/src/diagnostic_observations.rs`, `turnstone/src/shell/effects.rs` | Extend installed observation/inspection/export integration with the producer stream while preserving the existing redacted app-event source and receipt semantics. |
| Registry / Apparatus integration | Versioned event descriptors/projection and bounded causal joins over the current ingress/history. No universal app-event enum or new inspector-specific store. |
| Genet / host render boundaries | Receive mutation/update references and emit work/frame evidence. Exact locations and existing tokens must be inventoried at P5; do not declare physical presentation from Cambium or require renderer changes to prove draft retention. |

The optional renderer-neutral action-silencing combinator identified in F7
can be a separate small cleanup beside map_message_result; it is not a
prerequisite for composition, guarded bindings or scheduling. A general
arbitrary-child transfer protocol remains deferred until a consumer outside
this owning composition needs it.

The main new implementation is in Cambium and its Turnstone consumer. Meristem
changes are a diagnostic supplier slice. Source intake, lockfile updates,
external consumers and any publication are separately qualified when code
lands; this design does not authorize a release or claim external compatibility.

## Phases

### P0. Bind drafts to their setting

**State:** implemented; local machine qualification passed 2026-10-29.

- Key Turnstone's initial settings rows by setting id rather than position.
  Make the existing setting_row reconciliation include binding identity; an
  unexpected id change must never retain an actionable draft for the new id.
  This small correction may discard the old draft rather than silently retarget
  it; it does not implement or authorize user-facing rebind/external-conflict
  behavior, which is specified above and lands in P1/P3.
- Done when: insertion and reorder beside a setting with equal control/value
  preserve the draft on its original setting, and Apply targets only that
  setting. A direct unexpected identity replacement cannot apply the old draft
  to the replacement. Cover Arrange's setting_row consumer as well.
- Needs: existing keyed/component contracts only; can land independently.

### P1. Composition model, guarded binding and event vocabulary

**State:** locally qualified for the stated supplier and consumer test scope;
composition editing and P2–P6 remain open.

- Implemented: public `cambium::composition` descriptors, stable keys,
  group/drawer placement, explicit unplaced components, application validation,
  authored-unavailable restore and deterministic transaction edits. Defaults:
  256 components, 64 groups/drawers, placement depth 12 (hard cap), payload depth
  64, encoded model 1 MiB. Loading/editing must use admission, not unchecked
  deserialization as authorization. Edit records contain application data and
  are declarative; they are not a renderer execution order or diagnostic payload.
- Implemented: the opt-in `RevisionedSettingsProvider` contract in surface-api,
  with owner incarnations, named-domain revision vectors, per-setting guards,
  request correlation and typed outcomes. The old `SettingsProvider` remains
  unguarded. Fake-owner tests qualify the protocol; they do not qualify a real
  product's storage transaction.
- Implemented in Turnstone: one shared in-process settings owner for shell and
  Settings-pane writes, a revisioned adapter, and per-setting bindings with a
  maximum of eight pending requests, client UUID, generation and opaque
  single-use tickets. Four owner-issued domains are `appearance-choice`,
  `shell-chrome` (including zoom), `behaviors` and `theme-catalog`. A write
  validates its owner/domain/revision guard and persists under the shared
  owner lock; the owner publishes a new target-domain revision only on success.
  Committed replies contain the canonical persisted value, so normalized
  values such as zoom become the displayed baseline. Older acknowledgements
  cannot regress a binding baseline. Guards remain sticky when only a revision
  changes, including same-value edits; an explicit discard/reload adopts a new
  guard. Validation, storage and unavailable failures do not publish the
  attempted value or advance its revision.
- Theme-library refresh advances only `theme-catalog` when its semantic
  contents/status change. Applying a theme is a separate explicit choice write;
  even a successful same-value Apply advances the appearance-choice revision.
- Locally qualified: the guarded offline Turnstone settings filter passed 61
  tests (0 failures), including all 13 binding tests, real-owner/receiving
  coverage, all five command tests and existing theme/UI receiving. The frozen
  binary Arrange filter passed nine tests (0 failures). Source-size and diff
  checks passed. Evidence under
  `/Users/markik/Code/output/cambium-composition-p1-owner` includes the source
  manifest, `settings-test-r2.log`, `arrange-test.log`, and frozen
  `turnstone-lib-tests` (SHA-256
  `2aef06464467c3b79ebb55c3380b3a49e62713207d3b58359a556c6fa820e89c`).
- On successful Apply, the binding accepts a canonical persisted value. The
  pane may adopt same-value/control sibling guards from a coherent post-commit
  snapshot only when each sibling still has the exact guard captured for that
  request. This lets successive fields in the same pane proceed without
  refreshing an older draft from another pane. The owner serializes participating
  in-process callers only; file reload/replacement is not cross-process
  compare-and-swap, and detached provider snapshots are not coordinated writers.
  The historical writer inventory below remains the dated baseline.

- Define the generic descriptor/table/placement model, allowed edit set and validation. Build the revision-aware settings companion, serialize participating Turnstone writes through one authority and define the scoped diagnostic identities/events. Inventory the theme-workshop write route and other writers before claiming a write scope.
- Done when: each supported edit validates and produces the intended edit set; invalid or duplicate compositions leave the accepted model unchanged. A stale guarded write fails without overwriting a newer value; unrelated domains can refresh without erasing a draft; persistence failure publishes no committed revision; binding/request generations reject old results; same-id theme-library saves retain explicit Apply behavior. These pass with a fake delayed provider and Turnstone's real provider adapter.
- Needs: existing provider/storage and diagnostics contracts; no Meristem API change.

### P2. Retained settings rows and custody

- Build the composition view with a flat retained instance table inside the existing pane runner, naturally sized row containers, owned parking and stable route generations. Instrument creation, movement, hiding and disposal from the start.
- Done when: reorder and regroup keep owner incarnation, control state and DOM identity in both reconciliation orders, including adjacent settings with equal control/value and a nonempty draft; insertion/removal cannot transfer that draft to a neighbor. Replacing a group wrapper cannot remove retained descendants; parking plus DOM collection preserves the draft; duplicate placement and foreign-document moves are refused before mutation; final disposal tears down once and rejects late input. Tab traversal and accessibility activation use the current visible order and owner.
- Needs: P0 and P1; existing view/DOM contracts and pane runner. RetainedMount remains at the whole-pane boundary.

### P3. Editable composition and persistence

- Add row/group/drawer edits and accessible commands to Turnstone settings; save the model through its existing pane-config owner. Implement dirty-delete, conflict and rebind behavior. P1's guarded owner rejects stale writes, but the current `setting_row` still resets its raw draft when an external snapshot changes that same setting's value or control; full dirty/conflicted reconciliation is deferred to this phase.
- Done when: a dirty row survives reorder, regroup, hide/reveal, whole-pane move and saving the arrangement; Apply reaches its original target exactly once; source replacement cannot retarget it; confirmed deletion and undo have the documented incarnation behavior; restart restores structure with clean drafts. Invalid configs leave the previous live composition intact; missing kinds/settings remain recoverable descriptors. Existing Turnstone settings scenarios (rung4_settings.scn and rung4_settings_verify.scn under turnstone/scenarios) continue to pass.
- Needs: P1–P2. Cross-pane row transfer remains outside this slice.

### P4. Selective updates with coherent flushes

Outline (D14). Goal: rebuild only the rows whose declared inputs changed.
Done when a two-panel fixture with one shared and two independent
dependencies shows untouched panels skipping their factory and body, shared
changes landing coherently, and commands still dispatching once each.
Needs P2.

### P5. Connected diagnostic evidence

Outline (D14). Goal: the installed settings consumer traces a real settings
write from owner revision to the renderer work it caused, through the event
schemas the tests use. Done when move, hide, skip and coalescing are visible,
and loss, eviction and missing parents show as explicit gaps. Needs P1–P4 and
diagnostics M8/M11/M13; it cannot close M7.

### P6. Turnstone receiving and supplier qualification

Outline (D14). Goal: qualify the settings pane in the real retained shell and
review the Cambium API before other panes adopt it. Done when the automated
lifecycle, binding and scheduling tests and a headed settings drive pass,
including window move, keyboard focus, IME commit on move and hidden
accessibility exclusion. Whole-pane move qualification is shared with
presentation PT1. Needs P1–P5.

## Remaining implementation questions

- P1 now supplies concrete model admission defaults above. Select the P4
  scheduling budgets from the actual consumer and qualify nonstarvation.
- Inventory uncoordinated or cross-process settings writers in P1. The proposed
  single-owner guarantee must not be widened without storage-owner support.
- Inventory the actual Genet mutation and host frame tokens in P5. Missing
  causal links remain explicit rather than inferred from timing.
- Row transfer across panes/documents and live deletion-undo retention remain
  later capabilities, not unresolved choices blocking this pilot.
- D4 needs the host to force-commit an active IME composition before a move.
  Genet's IME path has not been checked for this. Upstream Masonry clears the
  composition on IME disable today; linebender/xilem#1692 (open) proposes
  finishing it instead.
- D6 is the settings pilot's rule. A consumer whose source sees frequent
  outside edits, such as a shared document under Murm's live interaction,
  decides its own conflict timing rather than inheriting D6.

## Findings (2026-10-29)

F1–F10 record the source baseline before P0; the P0 finding after them records
the implementation and qualification. Baseline line citations refer to those
earlier source revisions.

- **F1. Settings rows were positional before P0.** Turnstone builds the rows as a
  `Vec<Box<dyn AnyView>>` inside `el("div", rows)` (`turnstone/src/settings_pane.rs:288-300`).
  Meristem's `Vec` sequence is index-based with a generation per index
  (`crates/cambium/meristem/src/view_sequences/impl_vec.rs`), and every row has the same
  concrete type, so a row's state follows its index, not its setting.
- **F2. Pre-P0 wrong-binding draft reuse.** `setting_row` re-derives the
  draft only when the spec's `value` or `control` changes
  (`crates/cambium/cambium/src/setting_row.rs:274-278`); a changed `spec.id` keeps it. The apply closure
  captures the current id (`turnstone/src/settings_pane.rs:273-285`). With F1, inserting or
  reordering rows moves an unapplied draft onto a neighbouring setting with the
  same control and value, and Apply writes it there. This is a source-level risk that P0 must reproduce and prevent before
  reorder is exposed. P0 subsequently reproduced the direct replacement case and repaired it, as recorded below.
- **F3. Retained owners exist with focused tests.** `RetainedMount` parks a withdrawn runner
  in a hidden container rather than removing it, and only the owner dispatches
  input; hosts validate custody and use window-scoped layout
  (`crates/cambium/cambium/src/retained_mount.rs:7-13, 32-53`). Tests cover destination-first mounting,
  source-first withdrawal with GC and remount, and release after runner state
  (`crates/cambium/cambium/src/retained_mount.rs:185, 256, 313`). The Settings pane is built on one
  (`turnstone/src/settings_pane.rs:355, 372-412`).
- **F4. Portable children depend on traversal order.** `PortableKeyed`
  preserves a cross-parent move only when the source sequence rebuilds before
  the target (`crates/cambium/cambium/src/portable.rs:24-28`). The nursery lives in `GenetCtx` and is
  drained at the end of a rebuild (`crates/cambium/cambium/src/context.rs:224-260`), so postponing cleanup
  alone does not help a destination visited first.
- **F5. Erased replacement is intentional.** `AnyView` tears down and bumps a
  generation when the concrete type changes, so old messages go stale
  (`crates/cambium/meristem/src/views/any_view.rs:100-130`). `ViewId` is meaningful only to
  the parent that issued it (`crates/cambium/meristem/src/view_ctx.rs:9-23`).
- **F6. Component memoization exists.** A memoized component skips its body
  while props are equal and no message has dirtied local state
  (`crates/cambium/cambium/src/component.rs:102-110, 183, 222`).
- **F7. The action-silencing adapter is written twice.** `setting_row`'s
  `silence` and Turnstone's title bar (`turnstone/src/chrome_view.rs:1072-1078`) both map
  `MessageResult<()>` to another action type by hand.
- **F8. Multi-window updates rebuild every projection.** `update` rebuilds all
  projections; only `update_local` rebuilds one (`crates/cambium/cambium/src/multi.rs:177-200`).
- **F9. The diagnostics store already has causal fields.** Armillary
  `apparatus-core`'s `ObservationMetadata` carries `operation`, `cause`
  (a `RecordRef`), `subject`, `semantic_revision` and `presented_frame`
  (armillary `armillary/crates/apparatus-core/src/observation.rs:49-61` at `06340b0`).
  Turnstone copies `AppEvent` categories into it and states it has no
  operation correlation (`turnstone/src/diagnostic_observations.rs:1-5`). Mesquite
  exposes the batch type (`crates/cambium/mesquite/src/lib.rs:94`). Neither supplies the complete reactive decision/causal stream required
  here; existing store metadata does not establish that stream.
- **F10. Turnstone persists pane view-state.** Every registered pane, Settings
  included, has a view schema (`turnstone.settings.view`) and `Blueprint`
  persistence (`turnstone/src/panes/registry.rs:124-127, 238, 466-480`).

- 2026-10-29 (source inspection): `crates/system/surface-api/settings.rs` exposes describe/apply without revision guards. `turnstone/src/settings_provider.rs` reloads the current durable base and saves before updating its cached value; `turnstone/src/settings_pane.rs` publishes the live snapshot only after successful apply. P1 preserves that ordering and adds owner serialization/guards; current behavior is not atomic conflict detection.
- 2026-10-29 (source inspection): Registry's `DiagnosticCorrelation` carries run/source/operation; its optional collector projects into Apparatus history rather than deriving causal `RecordRef`s automatically. `armillary/crates/apparatus-core/src/observation.rs` has one primary `cause`, subject, semantic revision and presented-frame fields. Multi-parent UI links and product installation remain work, tracked jointly with diagnostics M13.
- 2026-10-29 (source inspection): `turnstone/src/diagnostic_observations.rs` already stores redacted app-event categories in Apparatus with default metadata; `turnstone/src/shell/effects.rs` records those categories at fanout. This is installed diagnostic history, but not the Registry reactive-UI bridge or causal revision/frame evidence proposed here.

- 2026-10-29 (P0 implemented and locally machine-qualified): the shared
  `crates/cambium/cambium/src/setting_row.rs` now re-derives draft state when
  `spec.id` changes. Its negative control reproduced an old Boolean draft
  reaching the replacement id before the fix. Turnstone's
  `turnstone/src/settings_pane.rs` uses a keyed production row helper; its
  insertion/reorder tests verify each row's draft and the original provider
  write. `turnstone/src/arrange_pane.rs` verifies replacement reset and
  same-binding draft preservation through its real choice-row helper. All 348
  Cambium library tests and all 21 Settings/Arrange tests passed. The test
  fixture selects Apply by exact id because Taproot attribute selectors match
  substrings. No Meristem change was needed; provider guards, editable
  composition and scheduling remain P1–P6. Retained evidence follows the local
  Code/output policy.

- 2026-10-29 (P1 shared foundations implemented):
  `crates/cambium/cambium/src/composition.rs` and its tests implement bounded
  whole-candidate admission and declarative edit classification without view
  state. `crates/system/surface-api/settings/revisioned.rs` defines the separate
  object-safe guarded interface. The full Cambium library suite (352 tests)
  and surface-api suite (10 tests) pass. Rejection tests preserve the accepted
  model; delayed stale requests, owner replacement, storage failure and
  unrelated-domain commits are covered by a fake owner. No real Turnstone
  guarded-write guarantee follows from those tests.
- 2026-10-29 (P1 writer inventory, Turnstone source at 571c2f3c):
  `turnstone/src/settings_provider.rs` centralizes production application-file
  writes in `save_candidate`, but each settings pane loads its own provider.
  `current_write_base` reloads and then replaces storage without a revision
  guard. `LiveSettingsHandle` in `turnstone/src/settings_pane.rs` shares chrome
  snapshots/actions/notices, not the write owner. Shell creation moved to
  `turnstone/src/shell/shell_new.rs`. `turnstone/src/shell/appearance_editor.rs`
  edits the authored theme library and refreshes catalogs on close; it protects
  the application-settings path and does not itself select the saved theme.
  No production caller of `apply_theme_choice` or separate full-settings reset
  writer was found. Pandect's recoverable file replacement is not compare-and-set
  protection. Consolidation must cover the actual pane writers and retain this
  explicit library-versus-selection boundary; cross-process safety needs its
  own storage-owner protocol.
- 2026-10-29 (upstream Xilem): Meristem's audit base is linebender/xilem
  `5d72ad41e`. Since then two upstream commits touched `xilem_core`:
  `31a9ccf26` (tuple sequences, for Grid track sizing) and `ce7b04d2b` (MSRV
  and README). Neither changes a contract this plan relies on. Open #1766
  (`AppCtx`, app-wide services) is Masonry-side and does not reach
  `xilem_core`. `mark-ik/xilem` main mirrors upstream, six commits behind; its
  branches hold May–July Masonry theme work, not Meristem.
- 2026-10-29 (Murm secure sessions and live interaction, `31281d62514`, N1–N3
  then optional OCapN O1–O4): no ruling here depends on it landing. If it
  lands, a remote proposal is one more writer that must enter through D8's
  authority. A session epoch maps onto this plan's owner incarnation, so
  reconnect and revocation already fall under the "source disappears" and
  "returning source requires revalidation" rows. Remote causes need a
  redacted reference form (D13). The request id is correlation only
  (`crates/system/surface-api/settings/revisioned.rs:71-79,138-149`): a
  redelivered Apply carries its old guard and is refused as stale rather than
  returning its original outcome, short of the domain-owned idempotency O3
  asks for.

- 2026-10-29 (P1 implementation, locally qualified; Turnstone composition
  worktree): `turnstone/src/settings_provider/owner.rs` wraps the existing
  persistence provider in `SharedApplicationSettingsOwner`; shell startup
  constructs the owner in `turnstone/src/shell/shell_new.rs`, and the shell
  passes a clone through `turnstone/src/shell/render/shell_answer_permission_request.rs`
  to Settings panes. `turnstone/src/settings_pane/commands.rs` routes Apply
  through the revisioned binding and shared owner. The shared handle lives in
  `turnstone/src/settings_pane/live_settings.rs`; the appearance editor keeps
  library edits/refresh separate from explicit choice Apply. The four domains
  and guard, acknowledgement, and failure behavior are summarized in P1 above.
  The guarded offline Settings filter passed 61 tests and the frozen-binary
  Arrange filter passed nine; source-size and diff checks passed. Receipts and
  the frozen binary hash are recorded under
  `/Users/markik/Code/output/cambium-composition-p1-owner`. This is an
  in-process writer boundary only; it establishes no cross-process CAS,
  publication/pin status, or headed acceptance.

## Decisions

Mark answered fourteen question cards on 2026-10-29. Each entry quotes the
option he chose. D4 replaced the proposed default; the other thirteen
confirmed it.

- **D1, one flat owner table in the pane runner.** Mark chose "Flat table in pane runner". Rows live in one retained table inside the settings pane's existing runner;
  RetainedMount stays at the whole-pane boundary.

  Links: summary Live settings rows live in one flat owner table inside the pane's existing runner, not a runner and mount per row.

- **D2, hidden rows keep state and stop rendering.** Mark chose "Keep state, stop rendering". A collapsed group or inactive tab keeps its rows' drafts and subscriptions,
  removes them from focus and accessibility, and stops rendering them.

  Links: summary Hidden rows keep their state and subscriptions but leave focus and accessibility and stop rendering.

- **D3, undoing a deletion recreates a clean control.** Mark chose "A clean new control". Edits are resolved before deletion; undo rebuilds the row from its saved
  descriptor as a new incarnation. No tombstone cache.

  Links: summary Undoing a confirmed deletion recreates a clean row from its descriptor; no tombstone cache.

- **D4, a move commits an active IME composition, then moves.** Mark chose "Commit the text, then move". This replaces the proposed default of waiting for the composition to end.
  The host must be able to force the commit; Genet has not been checked.

  Links: summary Moving a row mid-composition commits the IME text as a focus loss would, then moves.

- **D5, moves stay within one pane.** Mark chose "Within one pane". Moves to another pane or document are refused in the pilot.

  Links: summary The pilot moves rows only within one settings pane; cross-pane and cross-document moves are refused.

- **D6, outside changes are caught at Apply.** Mark chose "Keep edit, check at Apply". Chosen over flagging conflicts live. A dirty row keeps its edit and the
  guard it was edited against; a stale Apply is refused and offers theirs or
  yours.

  Links: summary A dirty row keeps its edit through outside changes; Apply's revision guard catches the conflict.

- **D7, a dirty row cannot be rebound.** Mark chose "Block until resolved". Rebinding stays disabled, with its reason, until the edit is applied or
  discarded.

  Links: summary Rebinding a row with an unapplied edit is blocked until the edit is applied or discarded.

- **D8, one write authority with revisions.** Mark chose "One authority with revisions". Participating panes and theme-choice writes go through one
  application-owned writer with domain revisions and guarded Apply.

  Links: summary Turnstone's settings writes go through one application-owned authority with revision-guarded Apply.

- **D9, arrangement in pane config, view state apart.** Mark chose "Pane config + view-state". Rows, groups and bindings go in `turnstone.settings.config`; expansion and
  scroll follow the pane's view-state policy.

  Links: summary The authored arrangement is saved in the settings pane config; expansion and scroll in view-state.

- **D10, a small rows, groups and drawers tree.** Mark chose "Rows, groups, drawers". A product-registered tree with only the kinds settings needs, not the
  workbench split/tab tree.

  Links: summary The authored composition is a small rows, groups and drawers tree.

- **D11, selective updates stay as P4.** Mark chose "P4 of this plan". Not split into a plan of its own.

  Links: summary Selective updates remain this plan's P4.

- **D12, Meristem reports through existing tracing.** Mark chose "Existing tracing events". No observer hook API. Turnstone's subscriber filter must admit the new
  targets.

  Links: summary Meristem reports its decisions through its existing tracing dependency, with no observer hook.

- **D13, producer references with multi-cause edges.** Mark chose "Producer refs + multi-cause". Observations carry producer references, and bounded edge records keep
  every cause of a join.

  Links: summary Observations carry producer references, and bounded edge records keep every cause of a join.

- **D14, P4–P6 stay outlines until P2 lands.** Mark chose "Outlines until P2 lands". Each keeps a goal and a done-condition; the detail is written from P2's
  result.

  Links: summary P4–P6 stay outlines with a goal and done-condition until P2 proves the retained table.

## Progress

- 2026-10-29: initial composition plan written.
- 2026-10-29: reconciled with the presentation plan. Consolidated lifecycle,
  binding, scheduling and producer contracts here; retained the flat-table
  implementation and independent P0 binding correction; resolved the competing
  draft, deletion, persistence and instrumentation recommendations as proposed
  defaults. P0–P6 is the sole composition sequence; no code landed.
- 2026-10-29: P0 implemented in Cambium and Turnstone, with six new regression
  tests. Local supplier and consumer suites passed; source changes are in the
  primary checkouts. Receiving used a temporary worktree-only source override;
  tracked consumer dependency pins were preserved.
- 2026-10-29: P1 shared model and guarded-settings contract implemented and
  unit-qualified. At that point the real Turnstone write-owner/adapter handoff
  remained open; the later P1 receiving and qualification entry below records
  its completion. P0 source commits were retained while reconciling with the
  latest Turnstone module split.
- 2026-10-29: Mark ruled D1–D14 by question card. Lifecycle, binding and
  persistence text now cites them; P4–P6 and their design sections are
  outlines (D14). Findings added for upstream Xilem and Murm's secure
  sessions and live interaction plan.
- 2026-10-29: Turnstone shared owner, guarded Apply route and binding admission
  passed focused local offline qualification (61 settings-filter tests and
  nine frozen-binary Arrange tests; zero failures). The consumer used a local
  source override with dependency pins unchanged. No Meristem API change was
  made; P2–P6 remain open. No headed or cross-process qualification is claimed.
