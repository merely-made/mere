# Cambium composition: retained components, bindings and selective updates

**Status:** 2026-10-30: P0–P5 implemented; qualification is limited to the recorded
supplier, receiving and native Settings scopes below. P4 adds coherent named
inputs, bounded coalescing and selective retained evaluation. P5 installs causal
decision evidence in Turnstone's existing diagnostics, including actual core
operation context, Settings/outer-frame renderer calls and native queue submission.
P3's arrangement and viewer-pin save/restart smoke pairs retain their recorded
qualification. P6 remains open: native IME candidate finalization/echo correlation,
broader movement/focus/AT and exact-frame host acceptance are not implied.
Published on Mere main and received by Turnstone main at Mere ccdfc8941,
adapted to Turnstone's deferred native presentation (Findings). Earlier
findings retain their dated baselines.

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

This is the inventory assessed before P0. The P0–P3 phase records below describe
the implemented source and its current qualification scope.

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
pointer capture is cancelled before transfer. In v1 a move commits tracked
Cambium preedit through the retained child and its old route, then moves (D4).
This synthetic Cambium commit updates the component's text state; it does not
prove that the native IME finalized its candidate or provide an OS session token
for matching a later native echo. That host/native qualification remains open
under P6. Before moving an active subtree, the host calls the move-preparation
hook; a move attempted while pointer capture or focused preedit remains active
is refused before DOM mutation. Focused retirement is likewise refused before
mutation. Before hiding or disposing a focused control, the host must send
focus loss; a visible move retains focus. Cross-window focus is restored only
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

P4 uses the flat owner table proved in P2. `composition_scheduler.rs` supplies
an opt-in complete input table; ordinary `RetainedComposition` construction
keeps its previous rebuild behavior.

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

The concrete dependency axis is source identity, owner incarnation and named
domain, with an owner-issued revision. Local input axes have named generations.
Vectors reject duplicate axes. Turnstone maps opaque provider incarnations to
checked per-pane tokens; it does not require UUID syntax or hash values. Its
appearance-choice/theme-catalog, shell-chrome and behaviors domains are sampled
from one cached snapshot per provider refresh. A session snapshot supplies both
the local generation and the factory's row props in the same flush.

The queue admits at most 256 components, one pending entry per key, 64 bytes
per axis name and 256 incoming causes per component; it retains eight causes
with explicit omission evidence. A full-table publication and its per-component
causes are atomic. Changed vectors and explicit same-vector notices invalidate;
identical publication without a notice does not. Capture freezes inputs and
pending generations before factories run. Acknowledgement removes only the
generation captured, so publication during a rebuild remains pending. Removing
a key prunes its pending work and mutation handoff.

One synchronous flush evaluates all dirty visible entries within the admitted
table. There is no idle-only debounce that continuous input can starve and no
partially evaluated shared revision. Hidden entries retain the latest pending
revision, including explicit same-vector notices, until reveal catches up.
Missing vectors are unknown and conservatively evaluate. Moves with unchanged
inputs refresh routes without invoking the factory or body. Messages still
dispatch individually and mark their owner dirty. Invalid input construction,
publication or exhausted adapter identities uses the legacy evaluation path
and records a coverage gap; it cannot freeze a row behind a clean fallback
vector. The retained table's normal visibility/input admission remains in force.

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

The first installed producer/collector path is Turnstone Settings, with renderer
evidence at its owned Genet adapters and the outer window frame. This is narrower
than complete renderer invalidation provenance across every pane.

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

Cambium's version-one payload uses only numeric identities and fixed enums.
Meristem reserves producer zero and emits bounded generation/path-depth facts;
Cambium producers receive checked nonzero identities. A retained operation span
supplies component/update identity to nested synchronous core calls. An actual
operation-start decision precedes each child call; a completion links back to
that start after the call returns. Recording a reference after a call cannot
retroactively give its nested events a cause. Core event
identity and ambient operation identity remain distinct. Unsupported atomic
targets and exhausted identities expose unavailable coverage rather than
silently reusing a sequence.

Turnstone installs a target-filtered tracing layer independently of its normal
log filter. Validated payloads pass through Registry's existing structured
Event ingress (4,096 records, 2 MiB, 300 seconds) into the existing Apparatus
observation history. Run and source-boot admission precede projection. Exact
producer-reference to admitted-record joins are bounded to 4,096 references;
256 unresolved edges can await reordered parents. Append-only resolution records
preserve exact record references. Missing/evicted parents, sequence holes and
late fills, malformed input, bounded cause omissions, ingress loss and store
loss remain distinguishable. Inspection and explicit export drain the same
history; export includes its typed coverage counters and renderer scope.
Owner-domain advance events alone populate the optional semantic-revision
metadata; scheduler generations and update identities remain in their typed
payloads. Event-supplied subjects take precedence over ambient subjects.

Nonempty accepted reconciliation emits a flush event and a MutationBatch with
the exact shared DOM mutation-cursor interval. This interval covers that flush,
not exclusive work attributable to one row. Visible Settings owners hand the
latest nonempty batch references to their receiving window's render attempt.
The actual `resolve_styles`, layout and paint-list call boundaries emit completed
work only when those calls ran. Separate work scopes prevent an unrelated DOM
root from acquiring a Settings cause by proximity. The final host fact is
HostFrameSubmitted after `queue().present` returns, with an opaque native-window
lifetime, surface generation and render-attempt identity. Its ScopeContains edge
names completed work in that submission; it is not physical presentation or
M7 exact-frame/readback evidence. Genet's internal invalidation decisions and
broader pane coverage need their own future producer instrumentation.

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

Tracing is the ruled path (D12). Turnstone's installed global subscriber now
has an independently filtered reactive layer for `meristem::reactive` and
`cambium::reactive`, alongside the existing formatted-log layer. The log filter
does not determine reactive collection. This introduces no public core trait,
scheduler, binding or renderer dependency in Meristem.

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

- **State:** implemented and locally qualified in the automated supplier and
  receiving scope below; headed host qualification remains open.
- Implemented in source: one flat retained owner table inside the existing
  runner, separate presentation containers, naturally sized rows and an owned
  hidden parking root. Table slots and route generations stay stable across
  placement moves; full placement generations change on moves and participate
  in message admission. Hidden rows retain state and stop rendering; hidden
  rebuilds skip their factory. Typed bounded lifecycle facts report creation,
  moves, hiding, reveal and disposal without raw binding/config values, with a
  dropped-event count and typed refusal reasons. The Turnstone settings
  consumer uses the real shared revisioned settings adapter from P1.
- Implemented in source: unprepared active-interaction moves and focused
  retirement fail closed before DOM mutation. The host can prepare a move to
  cancel captured pointer input and commit tracked Cambium preedit through the
  old route; hiding or disposing a focused row still requires host focus loss.
  Visible moves retain focus. Native IME candidate finalization and matching a
  delayed native Commit echo remain P6 host-qualification gaps.
- Ordinary retained-view publication sends `FocusLost` and pointer `Cancel`
  through still-live old routes before scrubbing handles when a rebuild
  hides/inerts the owner; callback state is published by one follow-up rebuild.
  New physical input remains blocked by hidden/inert/parked ancestors. An
  `aria-hidden` ancestor alone still permits physical clicks, while excluding
  the subtree from focus traversal and accessibility value actions. A live
  hidden file owner may receive completion of an already-started picker; P1
  settings binding outcomes are adjudicated independently.
- Locally qualified: full Cambium library suite 378 passed (0 failures),
  guarded Turnstone Settings filter 63 passed (0 failures), and frozen
  same-binary Arrange filter 9 passed (0 failures). Turnstone receiving used a
  temporary local source override with tracked dependency pins unchanged. The
  source snapshot is
  `/Users/markik/Code/output/cambium-composition-p2/source-before-r5.json`;
  frozen Cambium binary SHA-256 is
  `0de326e0ef348adfc7887f981875d9598ad3bee42f89b70c78b941d6952acd91`, and
  frozen Turnstone binary SHA-256 is
  `0afe3c34101791b28b23be92bbd94301b81cb7b2393c15400e27e5e5d9f6675e`.
  These results establish automated source/receiving behavior only; no headed
  or native IME host qualification is claimed.
- Done when: reorder and regroup keep owner incarnation, control state and DOM identity in both reconciliation orders, including adjacent settings with equal control/value and a nonempty draft; insertion/removal cannot transfer that draft to a neighbor. Replacing a group wrapper cannot remove retained descendants; parking plus DOM collection preserves the draft; duplicate placement and foreign-document moves are refused before mutation; final disposal tears down once and rejects late input. Tab traversal and accessibility activation use the current visible order and owner.
- Needs: P0 and P1; existing view/DOM contracts and pane runner. RetainedMount remains at the whole-pane boundary.

### P3. Editable composition and persistence

- Source implementation: Cambium's opt-in `setting_row_managed` keeps its
  typed text, slider, toggle or radio control inside the retained component;
  `SettingRowSession` supplies ephemeral value snapshots, dirty/reset/edit
  generations and pending/failed/conflicted admission metadata. Existing
  `setting_row` remains available with its earlier semantics. Managed rows
  delegate input to the existing controls. Move preparation's synthetic
  preedit commit reaches the session as an edit; slider cancellation preserves
  the last drag value.
- Turnstone now owns a validated accepted arrangement and bounded 64-entry
  model-only undo/redo history, with commands for adding, moving, grouping,
  hiding, duplicating, deleting and rebinding rows. Group/drawer collapse and
  reveal are named accessible controls available outside arrangement mode.
  Structural commands queue until the pane can prepare old input routes;
  retirement/rebinding rechecks dirty and pending work after preparation.
  Confirmed deletion disposes the session; undo creates a clean incarnation.
- Each realized component key has its own session and guarded binding, even
  for duplicate rows targeting the same setting. Dirty refresh preserves its
  captured write guard; only refused Apply raises a conflict. Use current
  explicitly discards, while review/retry checks the exact freshly reviewed
  owner guard before submitting. An adjusted reviewed edit remains unresolved
  through refresh, including when it matches the original baseline. A missing
  saved target remains inert and can be rebound to a registered target.
- Authored descriptors/placement persist in `turnstone.settings.config` and
  expansion/scroll in `turnstone.settings.view`. PaneWorkspace carries view
  records beside its descriptors through frame/lens persistence and transfers.
  Drafts, sessions, binding/request guards and history never enter either
  payload. Unknown authored kinds and unavailable source/setting references
  round-trip as inert descriptors; newly authored targets require registry
  admission. Invalid records leave the live arrangement and drafts intact.
- Save uses an exact pane admission and request id. The shell acknowledges
  only after the existing frame persistence write succeeds; an error retains
  unsaved state and an explicit retry path. Own accepted-record echoes cannot
  erase newer local changes or masquerade as an outside edit. No Meristem
  API or lifecycle change is needed for P3.
- Local automated qualification: the full Cambium library passed 401 tests;
  Turnstone's Settings filter passed 93, Arrange passed nine, and the workspace
  filter passed 13, all with zero failures. The actual Settings consumer tests
  include dirty group movement/collapse/reveal, parked refresh, destination-first
  whole-pane movement plus DOM collection, duplicate/conflict behavior,
  deletion/undo, unavailable-target rebind, disk restore, invalid-record refusal
  and failed/successful save-echo admission. The two native
  `settings_arrangement` scenarios passed using a fresh isolated profile and a
  new process for restore. Their headed scope proves group save/restore and
  rendered controls, not physical IME or every placement/scroll variant.
  Logs, frozen binaries, source hashes, temporary dependency closure and native
  captures are retained at `/Users/markik/Code/output/cambium-composition-p3`.
  The existing `rung4_settings` save/restart pair also passed after updating its
  retired `genet.web` default expectation and expanding Change viewer before
  choosing a radio. It exercises Auto-to-explicit `genet.livery` pinning,
  viewer-changed/respawn, title readback and applied restore; it does not claim
  switching to the retired engine. Failed first attempts remain in the evidence.
- Done when: a dirty row survives reorder, regroup, hide/reveal, whole-pane move and saving the arrangement; Apply reaches its original target exactly once; source replacement cannot retarget it; confirmed deletion and undo have the documented incarnation behavior; restart restores structure with clean drafts. Invalid configs leave the previous live composition intact; missing kinds/settings remain recoverable descriptors. Existing Turnstone settings scenarios (rung4_settings.scn and rung4_settings_verify.scn under turnstone/scenarios) continue to pass.
- Needs: P1–P2. Cross-pane row transfer remains outside this slice.

### P4. Selective updates with coherent flushes

Implemented source: `composition_scheduler.rs`, the retained composition rebuild
and Turnstone's Settings input adapter. Named dependencies, local session
generations, atomic publication/capture, coalescing, visible synchronous flushes,
hidden catch-up, unchanged factory/body skips and route-only moves are described
above. Queue failure falls back to ordinary evaluation rather than stale input.
Supplier tests cover independent and shared dependencies, exact-generation
acknowledgement, command delivery, same-vector notices, missing vectors,
visibility, removal and bounds. Receiving tests cover real Settings sessions,
opaque owner incarnations and independent source causes. Needs P2.

### P5. Connected diagnostic evidence

Implemented source: versioned Cambium producers, bounded Meristem tracing,
Turnstone's reactive layer and projection into its existing observation store,
and Settings/outer-frame Genet-call and native-submit evidence. The owner emits
request, persistence outcome and advanced-domain references at its actual
decision boundaries. The same payloads support regression assertions and
inspection/export. Receiving tests trace actual Apply through owner revision,
input evaluation, flush/batch, style/layout/paint and submission, and check
loss counters before an unchanged-row negative assertion. Reordered/missing/
evicted parents, disabled and disconnected collection remain explicit. Needs
P1–P4 and diagnostics M8/M11/M13; it cannot close M7. Native and automated
qualification results are recorded in Findings after the final source gates.

### P6. Turnstone receiving and supplier qualification

Outline (D14). Goal: qualify the settings pane in the real retained shell and
review the Cambium API before other panes adopt it. Done when the automated
lifecycle, binding and scheduling tests and a headed settings drive pass,
including window move, keyboard focus, IME commit on move and hidden
accessibility exclusion. Whole-pane move qualification is shared with
presentation PT1. Needs P1–P5.

## Remaining implementation questions

- P4 now supplies synchronous budgets from the bounded Settings consumer.
  A larger or asynchronous consumer needs a separately qualified scheduling policy.
- Inventory uncoordinated or cross-process settings writers in P1. The proposed
  single-owner guarantee must not be widened without storage-owner support.
- P5 now connects exact flush mutation intervals to actual adapter calls and
  native queue submission. Genet internal invalidation provenance, other panes
  and M7's exact-frame/readback links remain open.
- Row transfer across panes/documents and live deletion-undo retention remain
  later capabilities, not unresolved choices blocking this pilot.
- D4 needs the host to force-commit an active IME composition before a move.
  Genet's IME path has not been checked for this. Upstream Masonry clears the
  composition on IME disable today; linebender/xilem#1692 (open) proposes
  finishing it instead.
- D6 is the settings pilot's rule. A consumer whose source sees frequent
  outside edits, such as a shared document under Murm's live interaction,
  decides its own conflict timing rather than inheriting D6.

## Findings (2026-10-30)

- P4/P5 supplier and receiving source is implemented. The final supplier gate
  passes 420 Cambium and 9 Meristem tests; the host check with Meristem default
  features disabled passes. Receiving uses the existing Turnstone lane and a
  temporary 15-package local source closure, including Registry. Existing
  tracked pins remain unchanged; only the new direct Registry dependency is
  added at the existing Mere revision.
- Final receiving gates pass 103 Settings, 19 reactive diagnostics, 33 Arrange
  and 25 workspace tests from the same frozen consumer executable. These are
  selected overlapping test filters, not a sum of distinct cases. The owner
  conflict trace and real queue-merge payload regressions cover numeric domain/
  expected/current facts and keep notice counts separate from operation identity.
- The native Settings scenario applies Show shellbar through its actual toggle
  and Apply control in a private profile. Two captures are nonblank. Its export
  resolves the shell-chrome owner commit through invalidation, capture, component
  evaluation, flush, MutationBatch, actual style/layout/paint and native queue
  submission. Nested Meristem decisions carry the same component subject and
  resolve through the pre-call operation start to that evaluation's completion.
- The selected native chain is complete. Its mutation range is the exact shared
  flush interval [604, 605), and the native host fact names a window lifetime,
  surface generation and render attempt. The run reports zero malformed input,
  ingress/store loss, missing producer sequences and renderer coverage gaps.
  It also reports 35 bounded cause omissions and 2,022 prior join-index evictions;
  these remain visible and prevent claiming a complete graph for the whole run.
  The unchanged-row negative regression separately requires zero added loss over
  its measured interval, including explicit producer-unavailable records.
- Renderer scope is Settings plus the outer window frame, observed at actual
  Turnstone-owned Genet calls. This does not establish every pane's invalidation
  provenance, physical presentation, M7 exact-frame/readback acceptance, P6 host
  IME/AT/movement acceptance, publication or consumer repinning.
- Current Turnstone stages native frames and presents them later from a queue,
  and runs the window frame's style and layout while preparing geometry, before
  its scene paints. Merged as committed, the headed chain stopped at the
  mutation: preparation ran outside any work scope, and `completed()` drops
  such calls without counting a renderer gap, so coverage read clean while
  style and layout were missing. Receiving now hands the frame scope's outputs
  to the staged frame, records submission after the deferred queue present (or
  refusal if the staged frame is dropped), enters the window work scope during
  frame preparation and carries the window's last stage across work scopes.
- On the merged code the headed scenario reproduces both capture digests and
  resolves the twelve-stage chain over the same flush interval [604, 605), with
  zero cause omissions and 469 prior join evictions. The full Turnstone library
  suite passes (1355); Cambium 420, Meristem 9.
- Meristem's `no_std` trace sequence (producer 0) is process-wide while a test
  subscriber observes one thread, so the two gap-asserting Settings tests
  passed only when run serially; they now rerun alone in a child process. The
  receipt export test failed on the P5 commit because the export's coverage
  record takes a retention slot, which can evict the oldest app record at a
  full store. A renderer call outside a work scope still disappears from
  coverage rather than counting as a gap; the chain verifier, not the counters,
  caught the move.

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
- 2026-10-29 (P2 implementation and automated qualification):
  `crates/cambium/cambium/src/retained_composition/` implements the flat owner
  table, separate presentation containers, natural row sizing, hidden parking,
  and live-slot/route plus placement-generation admission. It emits typed,
  bounded lifecycle facts without raw binding/config payloads and exposes the
  dropped-event count and typed refusal. Hidden rebuilds skip their child
  factory. Unprepared moves with active capture/preedit and focused retirement
  refuse before DOM mutation; `prepare_subtree_move` dispatches pointer cancel
  and a synthetic tracked Cambium commit over the old route. A prepared move
  keeps visible focus. The host must send focus loss before hide/dispose. This
  does not qualify native IME candidate finalization or native echo matching;
  those remain P6. The ordinary runner publication path sends focus-loss and
  pointer-cancel to still-live hidden owners before scrubbing handles, then
  publishes callback state with one follow-up rebuild. Physical click/key
  admission uses hidden/inert/parked state; `aria-hidden` controls focus
  traversal and accessibility value input while leaving physical clicks
  available. Hidden live file owners still receive already-started chooser
  completions; P1 settings binding admission remains a separate check. The
  actual Turnstone settings consumer is wired to P1's shared revisioned adapter.
  Full Cambium passed 378/0, the guarded Turnstone Settings filter passed 63/0,
  and the frozen same-binary Arrange filter passed 9/0. Turnstone receiving
  used a temporary local source override with pins unchanged. Evidence is under
  `/Users/markik/Code/output/cambium-composition-p2`, including
  `source-before-r5.json`; frozen Cambium and Turnstone SHA-256 values are
  `0de326e0ef348adfc7887f981875d9598ad3bee42f89b70c78b941d6952acd91` and
  `0afe3c34101791b28b23be92bbd94301b81cb7b2393c15400e27e5e5d9f6675e`.
  This qualifies automated source/receiving behavior, not headed acceptance or
  native IME candidate finalization/echo correlation. P3–P6 remain open.

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
- 2026-10-29: P3 managed settings rows, arrangement commands and frame-owned
  config/view persistence implemented. Full Cambium library 401/0, Turnstone
  Settings 93/0, Arrange 9/0 and workspace 13/0 passed. A fresh-profile native
  arrangement pair and the updated viewer-pin pair passed (four native cases).
  Per-row conflict review, dirty deletion,
  clean undo, missing-target recovery and save acknowledgements are exercised
  through the installed consumer. No Meristem API changes, publication or
  dependency repin; full P6 host/native IME acceptance remains open.

- 2026-10-30: implemented P4 coherent selective evaluation and P5 connected
  decisions through the existing diagnostics stream. The first Settings scope
  includes actual retained-core operation context and native submit evidence;
  broader P6/M7 host qualification remains open.
- 2026-10-30: published (Mere 767c53a61, ccdfc8941) and received on Turnstone
  main 068f182e, including the deferred-presentation and frame-preparation
  adaptation recorded in Findings.
