# Batch 118: Cambium composition and presentation reconciliation

## cambium_docs/implementation_strategy/2026-10-29_cambium_composition_plan.md

- disposition: current
- status line: "2026-10-29: reconciled proposal." - accurate: yes
- claims checked: 5 - holds: 5, stale: 0, unverifiable: 0

### Stale claims

The earlier draft's exclusion of selective scheduling and recommendations to
reset dirty drafts, retain deleted instances for undo, persist composition in
pane view-state, and require a context observer hook are replaced by the
explicit reconciled proposals. No implementation is claimed.

### Contradictions

The concurrent presentation draft proposed one runner per row; the reconciled
design retains this plan's flat instance table inside the pane runner. A
bounded owner table and selective factory/body evaluation share one input and
focus authority. The current schema distinguishes authored PaneConfig from
remembered PaneViewState; the proposal now specifies one home for each.

### Recommended action

Keep as the canonical composition contract and P0–P6 implementation sequence.
Start with the independent draft-binding correction. The proposed behavior
changes require the specified consumer tests before being reported as built.

### Notes

Five bounded source claims checked on 2026-10-29: Component stores local state
and dirties it on message receipt; the settings row reconciles only value and
control; Turnstone settings constructs positional rows; PaneConfig and
PaneViewState are separate schema-owned payloads; RetainedMount is an existing
whole-pane owner boundary. Sources: the files linked in the plan's capability
table and turnstone/src/panes/blueprint.rs. No runtime test was run. The
reconciled choices are design proposals, not new maintainer rulings. The prior
batch 117 remains historical evidence of the earlier draft.

## cambium_docs/implementation_strategy/2026-10-28_cambium_presentation_plan.md

- disposition: current
- status line: "2026-10-29: reconciled with the composition plan." - accurate: yes
- claims checked: 3 - holds: 3, stale: 0, unverifiable: 0

### Stale claims

The duplicate retained-component contract and RC phase sequence have moved to
the composition plan. Browser tenants, UA widgets, portable whole-pane/tile
movement and operator presence remain here.

### Contradictions

Removed competing row-lifetime and scheduling instructions. PT1 shares headed
move qualification with composition P6 and does not require rebuilding the
already existing whole-pane retention mechanism.

### Recommended action

Keep as the presentation/host plan; use the linked composition contract for
editable settings rows. Existing qualification limits remain unchanged.

### Notes

Checked three document claims: composition has one canonical home; this plan
contains no duplicate RC phase definitions; PT1 names the same retained-pane
boundary and shared qualification as composition P6. This is a scoped
reconciliation judgment, not a rerun of the historical browser/platform gates.

## mere_docs/implementation_strategy/2026-10-28_diagnostics_and_accessibility_plan.md

- disposition: current
- status line: "M13 adds the proposed reactive UI producers requested alongside retained editable components." - accurate: yes
- claims checked: 3 - holds: 3, stale: 0, unverifiable: 0

### Stale claims

M13's lifecycle/producer contract now points to the composition plan and its
P0–P6 sequence rather than the removed presentation-plan section.

### Contradictions

Collection remains with Registry/Apparatus product integration; composition
specifies the producer contract. A causal trace does not close M7's exact-frame
and two-consumer gates.

### Recommended action

Keep M13 as the diagnostics integration phase, with producer implementation
and collector qualification explicitly pending.

### Notes

Checked three document claims: M13 resolves to the canonical producer contract;
the stage dependencies use the reconciled sequence; exact-frame and collection
loss limits remain explicit. No diagnostics implementation or runtime evidence
was added by this reconciliation.
