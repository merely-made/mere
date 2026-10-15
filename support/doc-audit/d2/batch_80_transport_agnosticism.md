# Batch 80: Transport agnosticism note and the stack lift plan's P2a

Audit date: 2026-10-15. Scoped judgment of a new design note against `ports/graphshell`,
`ports/djinn`, `crates/graphshell`, `crates/chirograph`, `crates/mesh` and the new
`crates/cambium/mere-overmap` at `9cc711fbc` plus this change; Murm's `mere-transport`
and `mere-webrtc-carrier` at `b49df894` and `3d9e799f` from the cargo cache; Turnstone
`de56f958` read from GitHub; Knot's local checkout. The lift plan is re-judged for the
P2a progress entry it gained.

## mere_docs/technical_architecture/2026-10-15_transport_agnosticism_note.md

- disposition: current
- status line: "design note for Mark's review; nothing implemented" - accurate: yes
- claims checked: 64 - holds: 61, stale: 0, unverifiable: 3

### Stale claims

None. The note proposes; it amends no earlier document.

### Contradictions

None with current rulings. It places the shared server layers in `graphshell-host` (R4)
and keeps the doors under `graphshell-endpoint`'s `door` feature (R3).

### Recommended action

Take rulings T1 to T8 to Mark before the P2a receipt or any carrier consolidation.

### Notes

Unverifiable: the browser extension's JavaScript (whether it expects notices over native
messaging), the external crates' QUIC stream mapping and handshake limits beyond their
call sites, and whether a WebRTC rejoin's preserved projection session matches the
transcript-derived id after re-admission. Line pointers were taken from source at the
revisions above.

## mere_docs/implementation_strategy/2026-10-14_graphshell_stack_lift_plan.md

- disposition: current
- status line: "in progress. P2a's code landed" - accurate: yes
- claims checked: 12 - holds: 12, stale: 0, unverifiable: 0

### Stale claims

None. The status line, the P2a phase and the Progress log were updated with the change
that landed `mere-overmap`.

### Contradictions

None. R7 named the dependency set `mere-view`, `graphshell-client`, `chirograph`,
`pandect` and `kernel`; the crate also takes `cartography`, `sceno`, `scenotime`,
`serde`, `serde_json` and `uuid`, which the Progress entry records.

### Recommended action

Keep P2a open until Turnstone's overmap pane runs the receipt.

### Notes

Supersedes batch 79's record for this document.
