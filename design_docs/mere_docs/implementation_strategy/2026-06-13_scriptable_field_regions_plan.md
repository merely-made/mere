# Scriptable Field Regions Plan

*Written before the 2026-09-05 retirement of graphlet (TERMINOLOGY.md): read graphlet as subgraph. Identifiers such as GraphletId, GraphletRef, and SessionGraphlets are now SubgraphId, SubgraphRef, and SessionSubgraphs, and the graphlets crate was crates/graph/subgraph (code renamed 2026-09-12), now `crates/mere/src/subgraph.rs` (folded in `61894570`, 2026-09-23).*

**Status (2026-10-06):** partially implemented. Landed: placement, move and resize,
the force well and rebuild-on-mutation (P0, P1 and the follow-ups, `3c62b15` to
`7445e70`), and the physics settings on 2026-06-14: runtime damping (`540935f2`,
`50c23945`), now `Canvas::set_physics_damping`, driven live by Turnstone and
Graphshell; per-field strength (`af775f24`), whose library half survives as
`set_field_strength` with no host driver since meerkat's removal (2026-07-18,
`c5f01064`). No host calls `add_field_at` (only a pictograph test does), so a user
cannot place a field today. Open: per-field response (gather / repel / wall /
dampen), and P2-P4.

Earlier status (undated, 2026-09-05 reconcile): partially implemented: movable and resizable field regions landed; physics-setting and further scripted-region work remains deferred.

A **field region** is a spatial area you place on the graph that carries a rule
set — scriptable in rhai — governing the graph's characteristics inside it:
forces (how nodes are pulled), edge visibility (which relations show), and node
layout (how the contained nodes arrange). The third graph element beside nodes
and edges, but a *rule-bearing* one: place a region, write its rules, and the
graph within it behaves accordingly.

**Spine**: per the [interaction model spine](../technical_architecture/2026-06-18_interaction_model_spine.md),
this plan owns the **localized / scripted arrangement** half of the *arrange* stage (the
scene-wide arrangement choice is the node-representation plan's), plus the placed rule region
(forces, edge-visibility) and its rhai surface. Field regions are already moveable + resizable
(Progress, `7445e70`).

**Corrected 2026-10-06 (S14 pass):** the node-representation plan was superseded on
2026-06-23 and its arrangement axis went to the graph signals layer plan, archived
complete on 2026-08-20, so the scene-wide arrangement choice has no active owner.

This is the "field" the user means — not a node attribute, a **spatial rule
region**. It unifies three subsystems that already exist separately (forces via
couplings, edge visibility via projection, layout via arrangements) under one
placeable, scriptable spatial primitive, and it is the third rhai lane after the
knot note-blocks and the omnibar command shell: one scripting language, now
governing a region of space.

## Expanded field direction (2026-10-09; research, not implemented)

The [design language §9.8](../../2026-08-23_projection_scenes_and_graph_native_platform.md#98-voice-clarifications-and-research-boundaries-2026-10-09)
records Mark's clarification: fields participate in a scene and can apply an
arrangement, dynamics or a projection to affected material. Scripting includes
general-purpose behavior and invoking a graph-held applet on entry. The forme
is a particular field representing the recursive workbench arrangement,
existing alongside other arrangements rather than controlling the whole scene.
Earlier force/visibility/layout scope is incomplete as a description of the
intended capability.

This direction does not establish that the current scalar/vector field AST
implements region events or applet execution, or select Rhai as the universal
backend. Scenograph's scripting comparison and application execution owners
retain those decisions. The partial implementation status above stands.

Research must distinguish spatial inclusion from declared membership and
define how overlapping or nested regions compose placement, projection and
motion without creating a second workbench layout authority. Entry/exit needs
defined behavior when nodes, regions or arrangements move, including avoiding
repeated actions caused only by boundary jitter. Foreground pin, position
anchor, position pin and layout lock remain separate controls; whether a lock
affects local dynamics remains open.

Mark's feed-overflow example is exploratory: excess material enters a field
and dissolves or moves to background. *Reading, not ruled:* an action must
identify whether it changes an appearance, attention, residency, keeping or
source truth; these effects cannot be inferred from the word "dissolve".

*Candidate proof, not opened:* a forme region and another field overlap;
their contributions and constraints are explainable, unrelated scene items
continue their own behavior, and one deliberate entry invokes the chosen
action through its existing authority. Overlap policy, event cadence and
execution grants must be ruled before that proof is implemented.

The [primitive planning direction, §9.9](../../2026-08-23_projection_scenes_and_graph_native_platform.md#99-plan-through-graph-primitives-2026-10-09)
also asks to represent fields alongside nodes and links in the graph, then
connect them to dynamics. Research each field's address, owned attributes,
membership, behavior and scene representation; a painted region alone does
not establish that authoring loop. Representation does not by itself choose
a new node encoding or replace the existing field/coupling stores.

**Source refresh (2026-10-09, Mere `b4e818b4`):** the
[primitive and dynamics map](2026-10-07_scenograph_editor_plan.md#primitive-and-dynamics-map-2026-10-09-source-backed-planning)
records the current authoring seams. `NodeSelector` offers All, Tagged, Kind
and NotTagged, not a spatial-membership selector. Pictograph's coupling bridge
captures matching node keys and the field definition; it does not pass the
extent into that conversion. A spatially shaped definition can produce a
localized force, but this is different from enforcing membership in the
authored extent. The earlier Findings shorthand equating a region coupling
with "nodes in the field's extent" describes intended behavior, not this
implementation. Extent tests already exist in numen; entry/exit tracking,
composition and response consumers require their own host integration.

*Reading, not ruled:* research placement, projection and action composition
separately. Additive motion alone cannot decide which of two incompatible
arrangements or projections wins. Likewise, changing a selector or moving a
field may change membership without a node crossing a stationary boundary.
The entry-action decision must cover those causes and initial scene loading,
alongside jitter and re-entry; the current force evaluator is not an event log.

**Host inventory (2026-10-10, Mere `6183006b`):** the editor plan's
[Graphshell control inventory](2026-10-07_scenograph_editor_plan.md#graphshell-control-inventory-2026-10-10-planning-step-1)
confirms that shared field cards and Canvas field methods have no Graphshell
authoring caller. Visibility is presentation-only; hiding a field does not
disable its coupling. Strength/placement mutate Canvas's graph, so wiring a
button straight to those methods would not establish MereHost's session edit,
undo and persistence path. A host adapter must distinguish that recorded write
from the refreshed simulation and the visibility intent. No new placement or
script UI is installed by this inventory.

## Findings (code-verified substrate)

The pieces exist; what is missing is **placement, rendering, and the unifying
rule surface**.

- **The kernel `Field` truth primitive** —
   [`graph-kernel/.../field.rs`](../../../crates/graph/graph-kernel/src/graph/field.rs) *(historical citation)* <!-- doc-audit: historical-link -->:
  `Field { id: FieldId, name, definition: FieldDefinition, extent: FieldExtent,
  lifecycle }`. `FieldDefinition = Scalar(ScalarField) | Vector(VectorField)`.
  `FieldExtent = Global | Region{min_x,min_y,max_x,max_y} | AttachedToNode(Uuid)`
  — **`Region` is a world-space box, so a field already carries a spatial
  placement**. The AST (`field_ast.rs`) has spatially-anchored constructors:
  `ScalarField::disk_at(cx,cy,radius,Falloff)`, `gaussian_at(cx,cy,sigma)`,
  `Falloff::{Hard,Linear,Smoothstep,Quadratic}`. Fields live in
  `Graph.fields: HashMap<FieldId,Field>` + `couplings: HashMap<CouplingId,Coupling>`
  (a parallel keyed store, not petgraph weights), mutated via `field_ops.rs`
  (`add_field`, `add_coupling`, `retire_field`, `field(id)`, `fields()`).
- **Forces** — a `Field` does nothing alone; a `Coupling` (field → `NodeSelector`
  → `CouplingResponse` → strength) is resolved by gyre
   [`CouplingForce::from_coupling`](../../../crates/orrery/gyre/src/coupling_force.rs) *(historical citation)* <!-- doc-audit: historical-link -->
  into a rapier force the layout tick runs. So "forces inside the region" = a
  coupling whose selector is "nodes in the field's extent".
- **The rhai authoring path already exists** — aether
   [`FieldProjection`](../../../crates/orrery/aether/src/projection.rs) *(historical citation)* <!-- doc-audit: historical-link --> + its rhai
  bindings (`rhai_bindings.rs`: `gaussian`, `couple_attract`, …), committed to the
  graph via `FieldProjection::commit_to_graph`. **This is the seam the region's
  rule script extends**: today it is registry-id / global authoring; a field
  region scopes it to a placed extent.

  **Corrected 2026-10-06 (S14 pass):** both seams moved. `commit_to_graph` was
  removed; numen stays graph-kernel-free and a graph write is the host's job
  (`crates/conatus/numen/src/projection.rs`). `CouplingForce::from_coupling` became
  pictograph's `coupling_force_from_graph` bridge
  (`crates/canvas/pictograph/src/canvas/seiche_bridge.rs`).
- **Edge visibility** — the graphlet
  [`EdgeProjectionSpec`](../research/2026-06-13_edge_system_audit.md) (the design
  in [subgraph derivation](../design/2026-06-13_subgraph_derivation_from_selection.md))
  already models "which edge families count" for a node subset. A field region's
  edge-visibility rule is an `EdgeProjectionSpec` scoped to the nodes in its
  extent.
- **Layout** — the `orrery/arrangements` family (layout strategies) already
  arranges node subsets. A region's layout rule selects an arrangement applied to
  its contained nodes.

  **Corrected 2026-10-06 (S14 pass):** no `orrery/arrangements` package exists; the
  layout adapters are `crates/canvas/cartography/src/adapters/`.
- **The gaps**: (1) no gesture to *place* a field at a world point (the just-shipped
  `add_node_at` is the exact pattern to mirror — `Orrery::add_field_at`); (2) the
  orrery does not *render* fields (placed fields are invisible — grep finds no
  `FieldExtent`/`fields()` use in `orrery/orrery/src`); (3) no *unified rule
  surface* binding forces + edge-visibility + layout to one region script.

## Design

A field region is a placed `Field` (with a `Region` extent) plus a **rule
script** the region evaluates over the nodes and edges within its extent. The
script is rhai, with privileged bindings in three domains:

- **Forces** — `couple(response, strength)` (attract/repel toward the field's
  min/max), realized as a `Coupling` over a `NodeSelector` resolving to the
  region's contained nodes. (Substrate: gyre `CouplingForce`.)
- **Edge visibility** — `show_edges(families)` / `hide_edges(families)`, an
  `EdgeProjectionSpec` scoped to the region's nodes (relations among contained
  nodes show/hide per the spec). (Substrate: the graphlet projection.)
- **Layout** — `arrange(strategy)`, applying an arrangement to the contained
  nodes within the region's box. (Substrate: `orrery/arrangements`.)

**Corrected 2026-10-06 (S14 pass):** the substrate names above are pre-fold. The
coupling force resolves through pictograph's seiche bridge, the arrangements are
cartography's adapters, and `EdgeProjectionSpec` is forme's
(`crates/forme/forme/src/subgraph.rs`). No `FieldContext`, `show_edges` or `arrange`
binding exists yet.

The region is a first-class visible object: a translucent outline (disk radius /
box) painted in the orrery, **selectable and movable like a node** (drag to
reposition the extent; the rules re-evaluate over the new contents).

**Trust** mirrors the omnibar command shell's privileged tier: the region script
is user-authored (you place and script your own region), so it gets the bound
`couple` / `show_edges` / `arrange` surface — the same two-tier model as the rhai
lanes (sandboxed note-blocks vs the privileged omnibar/region authoring), privilege
= the binding set.

## Phases

- **P0 — Place + render + move** (the foundation). `Orrery::add_field_at(content_band_xy)`
  mirrors `add_node_at`: mint a default disk `Field` with a `Region` extent at the
  cursor world point; render its outline in the orrery scene; make it selectable +
  draggable (move re-anchors the extent). A `ContextAction::AddField` row on the
  no-selection context menu (and an `Add field` row on the add-pill menu). Done
  when you can place a visible field at the cursor, see it, and drag it.
- **P1 — Forces**. The placed region gets a default `Coupling` (so it immediately
  bends the layout — the no-placebo gesture), then a `couple(...)` rule. Done when
  nodes in the region drift per its force rule and a force tick reflects it.
- **P2 — Edge visibility**. A `show_edges`/`hide_edges` rule scopes an
  `EdgeProjectionSpec` to the region's nodes; the orrery edge pass respects it.
  Done when a region can reveal/hide relation families for the nodes it contains.
- **P3 — Layout**. An `arrange(strategy)` rule applies an arrangement to the
  contained nodes within the region. Done when a region re-lays-out its contents.
- **P4 — The rhai rule surface** (the unifier). A region carries a rhai script;
  evaluating it (over the region's contained nodes/edges, via a `FieldContext`
  snapshot like the omnibar shell's `ShellContext`) emits the force/edge/layout
  effects. Built on aether's `FieldProjection` + the existing rhai bindings,
  extended with the placed-extent scope. Done when one region script sets all
  three characteristics over its contents.

## Open decisions

- **Default field shape**: *resolved (2026-06-14)* — disk-in-a-box. The soft disk
  well is the persistent visual; the box is the extent, shown **only on
  interaction** (hover / select / drag), both draggable/resizable.
- **Rule editing surface**: where the region's rhai script is written — an
  inspector pane (the field editor the configurability preference wants), the
  omnibar (`>` over a selected region), or a knot-style block. Likely the inspector
  pane, reusing the rhai infrastructure.
- **Selector semantics**: "nodes in the region" = strictly inside the extent box,
  or weighted by the scalar field's falloff? Falloff is the richer (and already
  modeled) answer.
- **Recompute trigger**: when does a region re-evaluate — on move, on graph
  mutation, every tick? gyre warns couplings snapshot targets at build time
  (rebuild on mutation); the region needs a defined recompute cadence.
- **Persistence**: fields already persist (`PersistedField*` in the kernel); the
  region's rhai *script* needs a persisted home alongside the field.

## Field as a first-class object (2026-06-14 user direction)

On first contact with a placed field, the user reframed it from "a thing on the
canvas" to a **manipulable, listable object**, and surfaced the core gap: *"I have
no idea what to do with the field."* The durable requirements:

- **Manipulation** — a field is moved, **resized**, and otherwise handled like any
  object (select, drag, resize handles). Move/resize re-anchor the `Region` extent
  (and the disk center/radius), and the rules re-evaluate over the new contents.
- **Box-on-interaction, not persistent chrome** — the dashed extent box should
  appear only while you are *interacting* with the field (hover / select / drag);
  the soft disk well is the persistent at-rest visual. An always-on box reads as
  clutter. (Supersedes P0b's always-drawn box.)
- **Hideable but findable in the roster** — a field can be **hidden** from the
  canvas yet remain listed in the **roster** (a third member kind beside nodes and
  edge-rows), so it stays findable and re-showable. The roster is the field's
  index + visibility control.
- **Purpose must be tangible** — the "no idea what to do with it" gap is the real
  one: an inert translucent disk has no evident point. The fix is **P1 first** — a
  placed field must *immediately do something visible* (the no-placebo gesture):
  its default coupling gathers / repels the nodes in its extent, so placing and
  dragging a field visibly moves the graph. Forces make the field self-explanatory
  before any rhai rule surface exists. **Gyre wiring note:** `CouplingForce` (gyre,
  `impl Force`) exists but is **not yet added into the orrery's live physics tick**
  (`physics.advance_frame`) — wiring couplings (rebuilt on graph mutation) into the
  sim is the load-bearing P1 work, not a flip.

## Physics tuning — deferred to a settings menu (post-window-composition)

After P1 shipped, testing the force well surfaced three physics issues. The user's
call: these belong in a **physics settings menu**, **deferred until after the
window-composition plan** (which unblocks pelt in meerkat) — not hardcoded guesses
now. Captured here so the menu's scope is ready:

- **Field strength is configurable, and the default is probably too weak.** Nodes
  already inside a field's radius do gather, but the pull wants more force, and the
  right strength varies — so it is a **per-field setting**, not one global constant
  (`configurability over opinionated defaults`). The disk's small gradient
  (~slope/radius) means the strength number is large; the menu exposes it (plus
  field radius / falloff) per field.
- **A coupling does not pick up nodes added after the field** *(correctness, not
  just tuning)*. `CouplingForce::from_coupling` snapshots its target set
  (`NodeSelector::All`) at build time, and the orrery only *adds* the force on
  placement — so a node minted *later* is in no field's target set and feels no
  pull (it drifts on the default forces, which looked like "the new node went to
  the old field"). The fix is the **rebuild-on-mutation** the P1 note already flags:
  on a node/field change, re-resolve every coupling's targets (a gyre force-replace
  API, or a position-preserving sim rebuild — the live sim is add-only today). This
  is the load-bearing gap; arguably worth fixing before the menu since it makes
  fields feel broken.
- **Inertia/damping is not preserved across pause/resume.** Settling damps momentum
  to rest (the right *default*), but when scrubbing with pause/play the user wants
  the momentum preserved so the motion continues from where it froze — a **damping
  toggle** (or a "resume with inertia" mode) in the menu. Today resume kicks a fresh
  settle budget; the body velocities survive a halt, but the settle damping bleeds
  them off.

Also deferred into the same surface: per-field **response** (gather / repel / wall /
dampen), the **move/resize re-aims the well** behavior (today the force snapshots
the field definition at placement, so dragging the field doesn't move its pull
until rebuild), and field **removal** dropping its force.

**Corrected 2026-10-06 (S14 pass):** part of this section, and the "still deferred"
list in the last 2026-06-14 Progress entry, landed later that day, after that entry
(`9d46d934`). Runtime damping (`540935f2`, `50c23945`) is now
`Canvas::set_physics_damping` in `crates/canvas/pictograph/src/canvas/input.rs`,
called by Turnstone's session lifecycle and by `ports/graphshell/src/canvas_physics.rs`.
Per-field strength (`af775f24`) is now `set_field_strength` in the same file, over the
kernel's `set_field_coupling_strength` (`crates/graph/graph-kernel/src/graph/field_ops.rs`);
its control was meerkat's roster, so no host drives it today. Per-field response is
still absent.

## Progress

- 2026-06-13: Plan written from the field-system scout (kernel `Field`/`Coupling`,
  aether `FieldProjection` + rhai, gyre `CouplingForce`, `EdgeProjectionSpec`,
  arrangements). User confirmed the vision: a placed spatial region whose rhai
  rules govern forces + edge visibility + node layout — "do it right" (place +
  render + couple, then the full rule surface). No code yet; P0 (place + render +
  move) is the foundation, mirroring the shipped `add_node_at`.
- 2026-06-14: P0a + P0b shipped. `Orrery::add_field_at` places a disk-in-box
  `Region` field at the cursor (`3c62b15`); the orrery renders it as a soft
  radial-gradient well inside a faint dashed square, spliced *under* the edges via
  `CanvasPaintList::splice_world_underlay`, placeable from the empty-space
  right-click and the add-pill (`80c05fb`). User feedback on first contact (above):
  fields want manipulation (move/**resize**), box-on-interaction only, roster
  listing + hide/show, and — the priority — a **tangible purpose**, which moves P1
  (forces, the no-placebo coupling) ahead of further P0 polish. Open question put
  back to the user: which effect to make tangible first (forces / edge-visibility /
  layout).
- 2026-06-14: **P1 shipped — the force well** (`48067be`). User chose "force well"
  as the field's first job. `add_field_at` attaches a default `RepelFromMax`
  coupling over the disk peak (nodes pulled up the gradient toward the center) and
  pushes the resolved gyre `CouplingForce` into the live sim via a new
  `Physics::add_coupling_force` / `PhysicsCommand::AddCouplingForce` (no
  position-losing rebuild); `build_simulation` also folds field couplings in for a
  session reload. Default strength `5000` (the disk gradient is small). Also shipped
  **physics pause/resume** (`b2ecaaf`): Space + a toolbar pause/play button, with a
  gated `settle_physics` so a paused graph stays frozen through mutations. Testing
  surfaced the three physics issues above (weak/strength-per-field, new-node not
  captured, inertia-on-pause) → **deferred to a physics settings menu, gated on the
  window-composition plan** (the section above). No further field code until the
  user resumes the follow-ups (move/resize, box-on-interaction, roster + hide).
- 2026-06-14: **Follow-ups shipped** (user said "knock out each", ultracode on).
  Scouted the seams (4 parallel agents), implemented in the main loop, adversarial
  review (8 agents). **Box-on-interaction** (`d43f3c1`): the dashed extent box draws
  only on hover (new `fields.rs` module; `active_field`). **Roster listing + hide +
  locate** (`bfd753a`): a Fields section with per-field hide/show toggle and
  click-to-center (`hidden_fields` mirroring `hidden_edges`, `center_on_field`).
  **Play runs continuously** (`fada862`): resume settles ~forever so a well can be
  watched. **Move + resize + rebuild-on-mutation** (`7445e70`): grab a field's box
  edge (move) / corner (resize); the disk + extent follow and the **well re-aims
  live**. This required the rebuild-on-mutation fix — gyre was add-only, so it now
  keeps a replaceable `coupling_forces` list (`set_coupling_forces`) and the orrery
  re-resolves all couplings on place / move / resize **and in `reconcile_derived`,
  which also fixes the new-node-capture bug** (a node added after a field is now
  pulled). Review fixes folded in (hover-clear on viewport exit, roster sort, z-order
  comment). **Still deferred to the physics menu (post-window-composition):**
  per-field strength, response (gather/repel/wall/dampen), and the inertia/damping
  toggle — the *tuning* surface, distinct from the now-shipped *mechanics*.
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere
  535bca11, from the D2 record in support/doc-audit/d2/batch_40_s14_phase_b2.md: a
  dated status recording the 2026-06-14 physics settings and the missing host driver
  for placement and per-field strength, the subgraph banner path, and the moved
  numen / seiche / cartography substrate names.
