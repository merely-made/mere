# Batch 112: Canvas same-call membership lifetimes

## mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md

- disposition: current
- status line: "implemented; the retained Pin failure now passes, with ordinary-edit identity controls and automated receiving qualified within the boundaries below" - accurate: yes
- claims checked: 14 - holds: 14, stale: 0, unverifiable: 0

### Stale claims

The same-call UUID/slot recreation gap is now received. Kernel Graph authors
runtime surface membership witnesses at one insertion boundary and removes them
on deletion. Ordinary edits, upserts and graph clones preserve live witnesses;
copy and snapshot creation receive the same insertion boundary. A snapshot restore
starts new runtime lives without persisting or replacing durable member UUIDs.
Canvas derives nonreused action keys from membership continuity; reads allocate
nothing. Recreated members retire their old Pin/Drag keys while unrelated members
retain keys. Fresh Pin requests remain usable.

### Contradictions

Endpoint UUID/slot equality is not proof of membership continuity. Metadata-only
refresh refuses recreated membership and uses structural ingest. A false closure
report cannot suppress a kernel-observed membership/structural change. Runtime
witnesses are neither durable identity nor authority leases or actor completion.
The original red patch/receipt remain historical evidence; the Pin test is now
permanent and passing, with old Drag and ordinary-edit controls.

### Recommended action

Receive lifetime boundaries in other retained Canvas caches/holds; then continue
custom producer/action-ID owner intake, supplier and external consumer promotion,
guest/selection/platform receiving and full-leaf/terminal-actor paths. Qualified
action routing and seed placement do not establish every physics/selection,
Forme, renderer-cache, disposal or rollback guarantee.

### Notes

Exact source, automated gates, red-to-green receiving, runtime/persistence limits
and retained byte budget are in
support/family-promotion/diagnostics-canvas-life-qualification.json.
The lane caps evidence at 10 MB, reuses the shared Mere target and preserves
pre-existing staged work and concurrent receipt changes. Global pre-existing D2
errors remain reported separately.
