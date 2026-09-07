# Batch 17 — 2026-09-07 eidetic brainstorm brief

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| eidetic_docs/research/2026-09-07_lighter_recall_and_standards_ledger_brief.md | current | yes | 9 | 9 | 0 | 0 |
| **Totals** |  |  | **9** | **9** | **0** | **0** |

**Totals: 1 doc, 9 claims checked (9 holds, 0 stale, 0 unverifiable), 0 contradictions within the doc; 2 contradictions it records against other docs.**

Audit base: Mere `0a8198ba29`, genet `5af76a0cb8`, turnstone `c6ee31ed92`
(2026-09-07). Recorded at authoring, in the same session, by the author;
a later batch should re-verify independently.

## eidetic_docs/research/2026-09-07_lighter_recall_and_standards_ledger_brief.md

- disposition: current
- status line: "Brainstorm brief with Mark. Research, contradictions, ideas and a proposed ledger; no code changed." — accurate: yes
- claims checked: 9 — holds: 9, stale: 0, unverifiable: 0

### Claims verified

- `TraceEvent`/`PageRef` carry `url` and `title` only: `crates/eidetic/eidetic-core/src/browsing/mod.rs:92-93`.
- turnstone extracts only the title via `fleece::extract`: `turnstone/src/browse.rs:517-518`; no `main_text` / `rebuild_with_text` / `text_for` caller in turnstone or mere outside `eidetic-search` itself.
- Capture plan C5 status "built + verified 2026-06-28 (`8b8b039`)": `mere_docs/implementation_strategy/2026-06-26_capture_provenance_consent_plan.md:279-307`.
- Meerkat obviated by turnstone 2026-07-18: `turnstone/README.md:31-32`.
- Wiring plan W3 note of 2026-08-26 ("supplies extracted page text to the trace corpus"): `mere_docs/implementation_strategy/2026-08-12_search_surface_wiring_plan.md:55-58`.
- "re-minted from the corpus, never repaired": `turnstone/src/trail_memory.rs:157`; fusion at `385-391`.
- `content_report`: `genet/components/genet-render/src/inspect.rs:27`; `extract_text` / `extract_main_text`: `genet/components/fleece/src/lib.rs:558, 821`.
- `unicode-segmentation` in fleece and genet-documents manifests: `genet/components/fleece/Cargo.toml:22`, `genet/components/genet-documents/Cargo.toml:52`.
- Census counts copied from `genet/design_docs/2026-09-06_web_platform_wpt_census.md` results table.

### Stale claims

- None.

### Contradictions

- None internal. The doc itself records two cross-doc contradictions (review brief §2 vs capture plan C5; wiring plan W3 note vs the trace type) for their owners.

### Recommended action

- none pending Mark's answers to its §5.

### Notes

- External crate facts (`bm25`, `probly-search`) are registry snapshots dated 2026-09-07.
