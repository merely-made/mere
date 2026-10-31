# Batch 124: Cambium composition P2 source status

## cambium_docs/implementation_strategy/2026-10-29_cambium_composition_plan.md

- disposition: current
- status line: "P0, P1 and P2 locally qualified in their recorded automated supplier/receiving scopes." - accurate: yes
- claims checked: 17 - holds: 17, stale: 0, unverifiable: 0

### Stale claims

None found in the updated status and P2 implementation record. P2's automated
qualification is distinct from headed host acceptance, which remains open. The
P1 offline supplier/consumer results remain separately recorded.

### Contradictions

None found within the recorded source scope. `RetainedComposition` has a flat
entry table and separate presentation containers, allocates natural row
wrappers, and uses an owned hidden parking root. Its route includes a stable
slot and route generation plus a placement generation that advances on moves.
Hidden entries retain their child state while skipping hidden child factory
rebuilds. Lifecycle events use a bounded typed queue, expose dropped-event
count and typed refusal reasons, and do not carry raw binding/config values.
Move preflight refuses active capture/preedit and focused retirement before
DOM mutation. The host move-preparation hook dispatches pointer cancellation
and tracked Cambium preedit commit through the old route. Visible moves preserve
focus; focused hide/disposal requires host focus loss. Ordinary view publication
sends focus-loss and pointer-cancel to still-live newly hidden owners before
scrubbing handles and rerenders callback state once. Hidden/inert/parked ancestry
blocks physical input; `aria-hidden` alone permits physical clicks but excludes
focus traversal and accessibility value actions. A live hidden file owner may
receive an already-started chooser completion; P1 settings binding result
admission is adjudicated separately. The Turnstone settings consumer is wired
to the P1 revisioned adapter.

Native IME candidate finalization and a native session token for delayed Commit
echo matching are not provided by this source and remain P6 host-qualification
gaps. Automated qualification passed: full Cambium library 378/0, guarded
Turnstone Settings filter 63/0, frozen same-binary Arrange filter 9/0. Turnstone
receiving used a temporary local source override with tracked pins unchanged.
This does not claim headed host acceptance.

### Recommended action

Keep P2 locally qualified within the recorded automated supplier/receiving
scope. Retain the explicit host/native IME boundary and do not claim P6, headed
acceptance, publication, pin updates or cross-process settings CAS.

### Notes

Source review covered `crates/cambium/cambium/src/retained_composition.rs`,
`retained_composition/view.rs`, `layout.rs`, `placement.rs`,
`retained_composition/tests.rs`, `retained_composition/tests/admission.rs`,
`runner.rs`, `context.rs` and Turnstone's settings consumer. Evidence location:
`/Users/markik/Code/output/cambium-composition-p2`; source snapshot
`source-before-r5.json`. Full Cambium library log:
`cambium-test-r5.log`; Turnstone Settings filter:
`turnstone-settings-test-r3.log`; frozen-binary Arrange filter:
`arrange-test.log`. Frozen Cambium SHA-256:
`0de326e0ef348adfc7887f981875d9598ad3bee42f89b70c78b941d6952acd91`;
frozen Turnstone SHA-256:
`0afe3c34101791b28b23be92bbd94301b81cb7b2393c15400e27e5e5d9f6675e`.
The Turnstone receiving run used a temporary local source override with tracked
pins unchanged. The receipts qualify automated source and receiving behavior,
not headed/native IME host acceptance.
