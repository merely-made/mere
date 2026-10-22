# Batch 104: Child and leaf lifetime continuation

## mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md

- disposition: current
- status line: "Leaf insertion life shared by cache and queue; canvas child keys authored at model transitions; generic and platform target-life migrations remain open" - accurate: yes
- claims checked: 12 - holds: 12, stale: 0, unverifiable: 0
- qualification: 317 native/canvas/product passes, seven existing ignores; Wasm and embedding compile pass; final 30 Sprigging passes repeat a subset. See diagnostics-child-leaf-qualification.json.

### Stale claims

The earlier leaf-cache lead is now implemented: one LeafRegistry insertion
marker governs cache reuse and queued leaf actions. Same-key replacements drop
former commands/scenes before the size/dirty gate, including while the old
registry remains alive. Mutable updates preserve the registration. Paint epochs
also remain distinct after cache removal, so retained host artifacts cannot
mistake replacement output for a prior generation.

Canvas::describe_items formerly allocated and pruned action keys under a mutex.
The canvas now authors them during construction and model reconciliation. A
removal/restoration between reads, graph switch with overlapping UUIDs, and
replacement canvas cannot inherit the former child key. Metadata refresh and
ordinary publication preserve live keys. Graphshell retained-content keys keep
their existing model owner.

### Contradictions

These changes do not supply a generic publication lease or atomic callback
transaction. ProducerNode key nonreuse is an authored contract, not host proof
that arbitrary producers comply. Raw native/browser maps can lose originating
publication provenance before enqueue. Canvas ingest_graph still trusts the
mutation callback and observes its final state; a same-UUID removal/recreation
inside one callback is a separate kernel/admission migration.

### Recommended action

Continue the shared target-life publication/request migration through native,
browser and automation. Keep native install/rebinding recovery as a separate
headed gate. Receive custom-leaf intrinsic sizing and scene preparation through
maintained supplier contracts: a source scan finds measure/layout_dirty without
Rootstock receiving, and RenderedLeaves::scenes consumers only in tests. Cache
unit tests do not establish actual Path-B rasterization or four-pass integration.

### Notes

The canonical plan and main ownership ledger already have concurrent staged
edits. Their new continuation section/property preserves those existing bytes
and index changes. This record and diagnostics-child-leaf-review.json track
the current slice and remaining ownership gates.
No earlier receipt or sealed register was rewritten. The retained receipt cap
is 10 MB; existing Mere Cargo output is reused.
