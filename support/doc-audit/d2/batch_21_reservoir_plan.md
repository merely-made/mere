# Batch 21 — reservoir plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md | current | yes | 43 | 43 | 0 | 0 |
| **Totals** |  |  | **43** | **43** | **0** | **0** |

**Totals: 1 doc, 43 claims checked (43 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `03c05dbd` (2026-09-23), with this pass's documentation edits
in the working tree. Turnstone, Knot and Cleromancy sources were read from the
local checkouts on the same day. `archive_docs/` is excluded. The V2 findings
added on 2026-09-24 were checked against Mere `bb709523` and the same
checkouts that day.

This batch exists because the plan is new. It implements the ambiance design's
rulings. The 2026-09-23 rulings it quotes are recorded first in that design
document and are not counted again here.

## mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md

- disposition: current
- status line: "Status: in progress. V1 is complete and on main … V2's shape was ruled on 2026-09-23 and 2026-09-24 (§7). Steps 1 to 3 (muniment, graph-kernel, pandect) landed on 2026-09-24 and reached origin on 2026-09-25; step 3b, undo with exact replay, landed on 2026-09-25 and reached origin the same day. Step 4, `MereHost` on `GraphSession` in Graphshell, is next." — accurate: yes
- claims checked: 43 — holds: 43, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The plan changes one rule on purpose: pandect's shared root says
  "everything else stays app-private". The plan cites that rule and moves only
  meres, by Mark's 2026-09-23 ruling.

### Recommended action

- none for this record. §7's decisions were ruled on 2026-09-23 and are
  recorded in the plan.

### Notes

The claims checked:

- **Device resident consolidation plan**, 10 claims:
  - its ruling quote, and its line that applications are clients for
    persona-held state;
  - R1, route-aware door and application-to-route grants;
  - R2, a shared resident source with per-session adapters, and the
    revision-bell quote;
  - R3, Knot composed into djinn;
  - R4, the embedded resident library quote;
  - the status line (R1 through R5 complete);
  - invariants 1, 7 and 9;
  - the ownership table giving Knot "Djot … evidence meaning, merge".
- **Pandect**, 2 claims:
  - the shared-root rule and its resolution to the platform data directory plus
    `mere` or `MERE_ROOT` (`crates/system/pandect/src/shared_root.rs`);
  - the graph codicil freeze, thaw, fork and compose.
- **Graph journal** (`crates/graph/graph-kernel/src/graph/journal.rs`), 1
  claim: "The materialized graph is the *replay* of the journal", snapshot
  checkpoints, and `journal_capture_hook`.
- **Athanor** (`ports/distillery/athanor/src/lib.rs`), 1 claim: it uses
  `compose_graph_codicils`.
- **The Alembic implementation plan**, 1 claim: its status line.
- **Turnstone**, 3 claims:
  - `repos/turnstone/src/session.rs`: `sessions/<id>/` and "the manifest set is
    pandect's `ManifestStore`";
  - `repos/turnstone/src/app/session_lifecycle.rs`: "boot, mint, adopt, fork,
    trash";
  - the capture-hook comment at boot.
- **Knot**, 1 claim: `repos/knot-editor/crates/knot-editor/src/bin/knot_sync_host.rs`
  defaults to `pandect::shared_root::shared_root`.
- **Cleromancy**, 4 claims:
  - `repos/cleromancy/src/host/mod.rs` saves one snapshot document in one
    muniment slot;
  - `repos/cleromancy/src/main.rs` puts the data root at
    `%LOCALAPPDATA%\cleromancy`;
  - `repos/cleromancy/src/admitted.rs` defines `CleromancySessionAuthority`;
  - no store exists at the default root on the primary development machine. A
    search of `%LOCALAPPDATA%` to depth three found nothing; the same listing
    found other directories, which serves as the positive control.
- **The ambiance design's cross-references**, 1 claim: §2 records the
  rulings, §9 the per-mere, per-app ambient opt-in, and §6 clones travelling as
  codicils.
- **The related-document links**, 1 claim: all resolve.

The V2 findings (§2, added 2026-09-24), 15 claims:

- **Graph journal and muniment**, 5 claims:
  - `AttributedDelta`'s author string and its quoted meanings
    (`crates/graph/graph-kernel/src/graph/journal.rs`);
  - no caller of `GraphJournal::save`, `load` or `migrate_bare_log` outside
    that file;
  - muniment's founding granularity and its append-form roadmap
    (`crates/eidetic/muniment/src/journal/persist.rs`);
  - its four shipped backends, and `Backend::apply`'s atomicity quote
    (`crates/eidetic/muniment/src/backend.rs`);
  - stickleback's per-entry log keys (`crates/stickleback/src/store.rs`).
- **Capture and undo**, 4 claims:
  - the per-thread capture hook quote
    (`crates/graph/graph-kernel/src/graph/capture.rs`);
  - Turnstone installing it at boot;
  - the event-log plan's per-`Graph` recording quote (E1);
  - no graph diff or inverse in the kernel or pandect, found by a search for
    diff, restore, invert and inverse functions whose only hit was
    `ManifestStore::restore_from_trash`.
- **Positions, view state and storage**, 4 claims:
  - the save-time position quote
    (`crates/system/pandect/src/arrangement_facets.rs`);
  - `ViewIntent`'s fields and its `views/<frame>/<pane>.json` layout
    (`crates/system/pandect/src/view_intent_store.rs`);
  - `MereHost`'s one-slot document, Graphshell's IndexedDB use in
    `ports/graphshell/src/web.rs` and memory in native receipts, and
    Cleromancy's one-slot host;
  - `session_graph_store::save` writing with plain `fs::write`.
- **Earlier rulings**, 1 claim: Alembic decision #5's parallel stream, the
  event-log plan's E4 quote, and the curation plan's ownership table and
  "Restore from here" quote.
- **Turnstone's fork**, 1 claim: `fork_session_from`'s component copy, facet
  carry, `parent_session`, admissions and nested worlds.

The step 3b findings (§2, added 2026-09-25), 3 claims, checked against the
branch that became `ea8059b9`:

- node creation stamps its visit time from the clock
  (`crates/graph/graph-kernel/src/graph/mod.rs`), and statement ids are
  minted from time, salt and counter (`crates/graph/graph-kernel/src/types.rs`);
  both were unjournaled until `ea8059b9`, shown by a kernel test that fails
  when either capture is disabled;
- `Graph::from_snapshot` plus `overlay_facets` kept default-valued facets
  imported from snapshot columns, shown by the session checkpoint test's
  whole-graph comparison before the fix;
- graphshell's `practice_disclosure.rs` and `practice_workspace.rs` include a
  woodshed file by a path that resolves from `repos/mere` only.
