# Batch 133 — engine workspace contract digest amendment

Written 2026-11-16 against Mere `origin/main` `440aa564b0ad05ed34b61894fa0995bbf4bccab5`, after the P8 contract clarified D154/D158 and the exact Resource-set comparison.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
| --- | --- | --- | --- | --- | --- | --- |
| engine_workspace_contract | current | yes | 5 | 4 | 0 | 1 |

## mere_docs/technical_architecture/2026-11-16_engine_workspace_contract.md

- disposition: current
- status line: "2026-11-16: draft for review. This records the P8 contract after rulings D147–D149, D158, D161, D163, D166 and D169. It describes current ownership and the intended boundary; it does not claim the Graphshell host integration is implemented." — accurate: yes
- claims checked: 5 — holds: 4, stale: 0, unverifiable: 1

### Stale claims

- none

### Contradictions

- none

### Recommended action

- Keep the portable Resource-quad digest encoding open until a cross-host consumer requires it.

### Notes

Confirmed D158's v2 chained digest rulings and current pin accessors in Mere `origin/main`: `PinnedView` exposes the session, cursor and digest, while the session holds a `DigestChain` rebuilt on open. The contract now plans to measure cursor-only identity lookup and cursor-plus-digest lookup at the headless probe. The exact full-set comparison is structural and does not choose a portable digest encoding. Transport schema and recovery remain outside this in-process probe.
