# Batch 132 — engine workspace contract

Written 2026-11-16 against Mere `origin/main` `440aa564b0ad05ed34b61894fa0995bbf4bccab5`, for the draft document added in the P8 contract lane.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
| --- | --- | --- | --- | --- | --- | --- |
| engine_workspace_contract | current | yes | 6 | 4 | 0 | 2 |

## mere_docs/technical_architecture/2026-11-16_engine_workspace_contract.md

- disposition: current
- status line: "2026-11-16: draft for review. This records the P8 contract after rulings D147–D149, D161, D163, D166 and D169. It describes current ownership and the intended boundary; it does not claim the Graphshell host integration is implemented." — accurate: yes
- claims checked: 6 — holds: 4, stale: 0, unverifiable: 2

### Stale claims

- none

### Contradictions

- none

### Recommended action

- Review the explicitly marked open questions before treating this contract as an implementation acceptance record.

### Notes

The cited rulings were checked in the Mere engine split plan at the source commit. `GraphSession::pin`, `PinnedView::dataset`, `graph_at`, and `view_at` exist in Mere `origin/main`; Graphshell `origin/main` builds its scene from its live graph and stamps it with its own `projection_revision`, with no `PinnedView` reference in `graphshell-host`. D161's 8 ms refresh bound and D163/D166 pin semantics are stated as recorded rulings, not remeasured here. The portable Resource-quad digest and cross-process recovery behavior remain unverified by implementation and are marked open in the document.
