# Batch 57: the Scenograph editor plan and the projection grammar handoff (new documents)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md | current | yes | 16 | 14 | 0 | 2 |
| mere_docs/research/2026-10-07_projection_grammar_handoff.md | current | yes | 6 | 5 | 0 | 1 |
| **Totals** |  |  | **22** | **19** | **0** | **3** |

**Totals: 2 docs, 22 claims checked (19 holds, 0 stale, 3 unverifiable), 0 contradictions; 0 status lines wrong.**

Audit base: Mere origin `f1d169c7` with the plan in the working tree (2026-10-07),
Genet origin and Isometry, Turnstone, Woodshed, Knot and mer3ly origins as
fetched the same day. The lane that wrote both documents judged them the same
day; an independent re-judgment can supersede this record under ruling S34.
The handoff (`973a7fc1`) was indexed without a record; this batch supplies it.

## mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md

- disposition: current
- status line: "**Status (2026-10-07):** plan. Rulings SE1 to SE9 recorded (§1). Track E1, the generic history, is next; nothing is built yet." — accurate: yes
- claims checked: 16 — holds: 14, stale: 0, unverifiable: 2

### Stale claims

None.

### Contradictions

None. Rulings A and B are quoted as the balaur brief's §1 records them; SE5
and SE7 narrow how, and the brief carries a dated pointer saying so.

### Recommended action

Re-judge when E1 lands, and when the scripting comparison (SE3) returns.

### Notes

Each claim below was checked against the source in brackets, and holds:
- `ProjectionEditor`'s fields, its ten actions, its sink, and no undo
  [`ports/graphshell/src/projection_editor.rs`]; Graphshell's web app as its
  only host [a grep for `ProjectionEditor` across mere];
- `EditHistory` text-only, cap 200, insert-run coalescing, no clock or saved
  marker [`crates/cambium/cambium/src/editor.rs`]; its only users
  [`controls/text_input/core.rs`, `command.rs`, `lib.rs`; a grep of mere,
  Turnstone, Knot, Woodshed, Isometry, Genet and mer3ly]; no clock type in
  Cambium [a grep of `crates/cambium`]; graph-truth undo
  [`crates/system/pandect/src/graph_session.rs`, `ports/graphshell/src/mere_host.rs`];
- the authored surface [`crates/cambium/scenes/scenograph/src/lib.rs`];
- eleven families, the seven readers, `finish`, custom solvers' JSON,
  `SolverCapability` without options [`scenomise/src/catalog.rs`,
  `projection.rs`, `registry.rs`]; test fixtures as the only `Solver`
  implementations [a grep of six repositories];
- `script-engine-api`'s shape and backends, piccolo's documented deviations
  [Genet `components/script-engine-*`]; rhai's quoted line
  [`crates/script/rhai/src/lib.rs`]; the WIT world [`crates/script/wit/world.wit`];
  Isometry's direct piccolo and no Rune manifest [manifests at origin];
- the rulings, quoted from the chat answers.

Unverifiable here:
- F5's wing rulings and F7's wing stagecraft, relayed by the games wing
  session and not re-read in Isometry's tree.

## mere_docs/research/2026-10-07_projection_grammar_handoff.md

- disposition: current
- status line: "n/a: a handoff carries no status line; its date and scope are in its first section." — accurate: n/a
- claims checked: 6 — holds: 5, stale: 0, unverifiable: 1

### Stale claims

None.

### Contradictions

None.

### Recommended action

Re-judge when any open target it names (A3 stage two, A5, Track F) opens.

### Notes

Holds: every cited commit is an ancestor of origin; the adoption plan's top
status line is the 2026-10-06 one; nothing in either grammar document is in
progress; the arrangement-role and stack-seams rulings it lists are recorded
in the two documents; the 2026-10-07 note naming the Scenograph editor plan.
Unverifiable: the weave line-dropping trap, which is a report of a past merge.
