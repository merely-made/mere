# Batch 107: Native publication IDs and host ownership

## mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md

- disposition: current
- status line: "Bounded host-owner and disposal repairs; native ID remapping remains open" - accurate: yes
- claims checked: 10 - holds: 10, stale: 0, unverifiable: 0

### Stale claims

The shared publication binding now carries a weak host owner. Native/browser
publishers and the private queue consume that same binding, and the queue's
separate owner-identity field is removed. A published request cannot follow an
identical document/mount into a foreign host, including by re-enqueueing. A
disposed host now refuses direct raw/bound requests and stops publication while
retaining inspectable CPU state. Raw constructors still name current context.
Native attachment rejects another handle before changing the original; same-handle
attachment is idempotent. Its result is propagated by native host creation.

### Contradictions

Host-qualified requests do not prevent a late OS numeric ID from selecting a new
binding after native map replacement. Action maps exclude structural producer
nodes, and a children-only remapper excludes relation/text-selection references.
Neither bridge activation epochs nor a weak host marker proves the physical
window/canvas an adapter is installed against. Headless/compile gates do not
qualify browser events, native late events, installation or reattachment.

### Recommended action

Finish all-node authored publication identity before introducing a nonreused
native namespace and schema-owned complete reference remapper. Receive host/
guest TreeId and platform lifecycle together. Keep producer atomic activation,
raw canvas mutation visibility and full leaf raster/sizing work separately owned.

### Notes

Exact gate/source boundaries and pinned native reference inventory are recorded
in diagnostics-owner-qualification.json and diagnostics-owner-review.json. The
public contract now requires owner at Accessibility::sync and publication capture;
all four workspace implementations are migrated. The lane retains at most
10 MB, reuses the existing Mere Cargo target and removes its integration worktree
after receiving. Existing staged work and concurrent receipt edits are preserved.
The global D2 audit retains its 40 pre-existing errors; this is not a clean audit.
