# Batch 125: Cambium composition P3 source and local qualification

## cambium_docs/implementation_strategy/2026-10-29_cambium_composition_plan.md

- disposition: current
- status line: "P0–P3 implemented and locally qualified in their recorded automated supplier/receiving scopes." - accurate: yes
- claims checked: 17 - holds: 17, stale: 0, unverifiable: 0

### Stale claims

None in the updated status and P3 source record. The capability inventory is
explicitly marked as the pre-P0 assessment, and earlier phase findings retain
their dated qualification boundaries. P4–P6 remain open.

### Contradictions

None in this source scope. Cambium's opt-in managed row delegates to the existing
typed widgets and retains their local editor state. Its session carries only
ephemeral value snapshots and admission metadata. Async acknowledgement preserves
newer drafts; availability observations do not settle pending operations. Dirty
refresh keeps its baseline, conflict begins on Apply refusal, and explicit
review/retry rechecks the reviewed owner guard. Conflict/review remain unresolved
when an adjusted draft equals its original baseline. Per-row discard recovers
unavailable or incompatible editors.

Turnstone owns each component's binding and the accepted authored arrangement.
Structural commands prepare old input routes and recheck retirement/rebind
admission after tracked Cambium preedit commit. Duplicate rows have independent sessions;
delete resolves dirty/pending work, then undo creates a clean incarnation. The
model history and historical provenance are bounded. Inert saved descriptors
survive transport and can recover through registered-target rebinding.

The existing pane frame stores authored config and remembered view state
separately. Config/view payloads omit drafts, bindings, requests and history.
Exact pane admission and request identity gate saves; acknowledgements follow
fallible frame writes. Failed writes remain retryable and own in-memory echoes
do not erase newer local edits. No Meristem API or lifecycle change is present.

### Recommended action

Keep P3 locally qualified within its recorded automated and native smoke scopes.
Do not infer P4 scheduling, P5 connected diagnostics, full P6 host acceptance,
native IME candidate finalization/echo correlation, publication or consumer repins.

### Notes

Reviewed source: Cambium `setting_row_managed.rs`, its editor adapter and tests,
`setting_row_session.rs`, and the slider cancellation guard; Turnstone's Settings
arrangement, editor host, row commands, source synchronization, record host,
PaneWorkspace and App/Shell record persistence path. Seventeen checked claims
cover widget reuse, independent instances, draft/guard preservation, Apply-only
conflict, explicit review/retry, dirty rebind, pending deletion, clean undo,
retained movement, whole-pane movement, config schema, view schema, runtime-state
exclusion, persistence acknowledgement, inert descriptor recovery and unchanged
Meristem ownership.

Evidence is retained at `/Users/markik/Code/output/cambium-composition-p3`.
`cambium-test-r6.log` records 401/0; `turnstone-test-r7.log` records 93/0;
`arrange-test.log` records 9/0; `workspace-test.log` records 13/0. The frozen
binaries and SHA-256 manifests are `cambium-binary.json`,
`turnstone-binary.json` and `native-binary.json`. Native arrangement save and
restart results are in `native-arrangement-r2-results.json`; both passed with
nonblank captures. That headed scope proves group save/restore and rendered
controls, not every movement, scrolling, accessibility or native IME condition.
The existing settings viewer-pin pair also passed, recorded in
`native-existing-settings-r2-results.json`. Its first scenario was updated to
expect the live Auto/livery route and to expand the chooser before selecting a
radio. It retains explicit-pin, viewer-changed/respawn, title and restart
readback assertions; it does not qualify switching to retired `genet.web`.
The local receiving closure used a temporary 14-package source override;
tracked dependency pins were preserved and the override/lock were restored.
