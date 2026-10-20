# Retained document receipts, 2026-10-13 to 2026-10-19

Evidence behind the
[retained document resource brief](../../../../2026-10-14_retained_document_resource_brief.md).
The [evidence log](evidence_log.md) records each experiment in date order.

On 2026-10-19 the small reports and receipts were copied here from the
M4-local `~/Code/testing/` directories, one folder each, keeping their
relative layout. Each copied Markdown report starts with a note naming its
source and any linked files that stayed local.

**What was not copied:** `target/` directories, probe and vendored source
trees (`src/`, `vendor/`, `upstream/`), runners (`run.py`, `qualify.py` and
similar), lockfiles, fixtures and transcripts over about 100 KB (the monster
catalogues, workflow transcripts, Cargo metadata dumps), raw per-sample timing
files from 2026-10-15 onward, and most raw logs. Those stay on the M4 host only.

**Edits to the copies:** each Markdown report gains a one-line provenance note.
Links to files that were not copied become plain paths marked "local only",
in three reports: the October 13 probe, the lowering profile and the workflow
report. The lowering profile's two `baseline/` links point at the
retained-document copies, which are byte-identical. Four `environment.json`
files have the host name in their `uname` string replaced with `<host>`. All
other bytes match the originals. JSON receipts still contain absolute
`/Users/markik/...` and `~/.cargo/git/...` paths. Those are recorded
provenance, not links.

| Folder | Date | What it shows | Main files |
|---|---|---|---|
| [redox-document-20261013](redox-document-20261013/REPORT.md) | 2026-10-13 | Precursor probe: Sonic, `serde_json`, `toml_edit` and a source overlay on real JSON/TOML | report, summary, verification, timings, allocations, fresh reopen |
| [retained-parser-20261015](retained-parser-20261015/REPORT.md) | 2026-10-15 | taplo 0.14 on Rowan 0.15.19 compared with a manifest-only Rowan 0.17 port | report, summary, port patch, releases, memory, verification |
| [retained-document-20261016](retained-document-20261016/REPORT.md) | 2026-10-16 | Isolated retained-document resource and bounded pack consumer | report, [SYNTAX.md](retained-document-20261016/SYNTAX.md), summary, verification, memory, fixtures |
| [retained-lowering-profile-20261016](retained-lowering-profile-20261016/REPORT.md) | 2026-10-16 | Stage profile of wide-table lowering; triangular DOM clone counts | report, summary, clone counts, instrumentation patch |
| [retained-dom-batch-20261016](retained-dom-batch-20261016/REPORT.md) | 2026-10-16 | Batched DOM construction against an unchanged control | report, summary, comparison, optimization patch, clone counts |
| [pack-author-workflow-20261017](pack-author-workflow-20261017/REPORT.md) | 2026-10-17 | Real-pack workflow through the private Isocosm CLI at `285821ad` | report, both qualification receipts, final manifest |
| [shared-source-edit-20261018](shared-source-edit-20261018/REPORT.md) | 2026-10-18 | `mere-source-edit` core plus Isocosm and Knot adapters, local override | report, core review/checks, per-consumer reports and receipts, lock audit |
| [source-edit-dedup-20261018](source-edit-dedup-20261018/REPORT.md) | 2026-10-18 | Shared `SourceSnapshot` removes duplicated Knot state and Isocosm undo ranges | report, core/consumer receipts, Knot report |
| [source-edit-publication-20261018](source-edit-publication-20261018/REPORT.md) | 2026-10-18 | Supplier published at Mere `70deef2c8`; Git-only receiving checks | report, publication receipt, per-consumer reports and receipts |

Isocosm's own receipt for the private CLI is in that repository:
[`design_docs/receipts/2026-10-17_pack_authoring/qualification.json`](https://github.com/merely-made/isocosm/blob/ecff57627cb8340cd1514432f84cda47de3d0c16/design_docs/receipts/2026-10-17_pack_authoring/qualification.json).
