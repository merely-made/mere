# Batch 82: Link model

Audit date: 2026-10-15. Scoped judgment of a new design doc against Mere `99ecfdeeb`:
`crates/graph/graph-kernel` (edge_data.rs, edge_payload.rs, edge_ops.rs,
predicate_registry.rs, edge_taxonomy.rs), `crates/cambium/cambium/src/graph_canvas.rs`,
`crates/canvas/pictograph/src/canvas/{edge_cells,dynamics_inspection}.rs`,
`crates/system/pandect/src/view_intent_store.rs` and the dependency manifests of
pictograph, mere, scenomise, sceno, cambium and graphshell. The remaining code
pointers come from the verified 2026-10-15 link/edge survey
(`testing/mere/link-edge-survey-20261015/survey.json`, outside the repo).

## mere_docs/technical_architecture/2026-10-15_link_model.md

- disposition: current
- status line: "design, no code changed. Mark ruled L1–L4 on 2026-10-15" - accurate: yes
- claims checked: 40 - holds: 38, stale: 0, unverifiable: 2

### Stale claims

None. Three earlier documents gained one-paragraph notes pointing here: the edge
system audit (thread 2), swatch design §7 and graph semantics plan §7's candidates.
TERMINOLOGY gained the assertion entry and an L1 note on link.

### Contradictions

None with current rulings. L1 keeps ruling 1's separate assertions grouped by the view
and the catalog's rule that multiple reasons remain multiple relations. L2 follows the
2026-07-12 withdrawal and checkpoint law. Link keeps the 2026-10-13 within-stratum
sense, and edge stays the shown binding.

### Recommended action

Take O3 (where the shared link shape lives) to Mark before P2, and O1/O2 before P1's
RDF terms.

### Notes

Unverifiable: Woodshed and Knot pointers were taken from the survey rather than
re-read; `graph_canvas.rs` line numbers have drifted since the survey (`FAN_GAP_PX` is
now at 844), so the doc cites symbols rather than lines there.
