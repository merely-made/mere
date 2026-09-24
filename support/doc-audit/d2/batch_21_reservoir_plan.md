# Batch 21 — reservoir plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md | current | yes | 25 | 25 | 0 | 0 |
| **Totals** |  |  | **25** | **25** | **0** | **0** |

**Totals: 1 doc, 25 claims checked (25 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `03c05dbd` (2026-09-23), with this pass's documentation edits
in the working tree. Turnstone, Knot and Cleromancy sources were read from the
local checkouts on the same day. `archive_docs/` is excluded.

This batch exists because the plan is new. It implements the ambiance design's
rulings. The 2026-09-23 rulings it quotes are recorded first in that design
document and are not counted again here.

## mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md

- disposition: current
- status line: "Status: plan. Nothing implemented. §7's decisions were ruled on 2026-09-23." — accurate: yes
- claims checked: 25 — holds: 25, stale: 0, unverifiable: 0

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
