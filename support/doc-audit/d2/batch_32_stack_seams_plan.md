# Batch 32 — stack seams plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md | current | yes | 22 | 22 | 0 | 0 |
| **Totals** |  |  | **22** | **22** | **0** | **0** |

**Totals: 1 doc, 22 claims checked (22 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `c34449bd` (2026-10-04), with this pass's edits in the
working tree: the new plan, its `DOC_README.md` entry at the head of the
`mere_docs/implementation_strategy/` list, and TERMINOLOGY's sceno amendment
and new arrangement, forme and world entries. Sources read: the code paths
the plan's §1 cites, genet's `components/genet-winit-host/src/lib.rs`, branch
`grammar-g2` by diff statistics only, and the two probes' source and logs
under `Code/testing/mere/rapier-determinism/` and
`Code/testing/mere/seiche-repeat/`. No web sources.

This batch exists because the document is new.

## mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md

- disposition: current
- status line: "Status (2026-10-04): plan. Six rulings (S1 to S6); P1 and P2 for this plan's lane, P3 done as documents; S3 and S4 belong to the dynamics grammar lane, which carries them into its own plan (§3.2). No code." — accurate: yes
- claims checked: 22 — holds: 22, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none in the plan. It records two in the outside note (F1, F7) and one in
  TERMINOLOGY's sceno entry ("not as a crate"), which this pass corrects.

### Recommended action

- none for this record.

### Notes

Claims checked: F1 graphshell's re-export (`projection_editor.rs` 18),
`534ae1c6` as the bridge's first commit, `arrangement_for` (666-682) with its
written-in sizes, scenograph's `Arrangement` doc (147-160) and crate doc, and
scenomise's `SolverCapability` (50-90); F2 the three `Arrangement` types and
the kernel family (`edge_taxonomy.rs` 96-100); F3 `AdvertisedAction` and
`IntentEffect` (`chirograph/src/lib.rs` 105-130) and the dynamics plan's line
263; F4 `Term`'s fields (`terms.rs` 39-50), the absence of "determinism" from
the dynamics plan, both probes' figures as logged, the three `HashMap`
iteration sites, and the search for other nondeterminism; F5 the four scene
types and `BodyWorld` (`conatus/src/world.rs` 167); F6 the event loop (514),
`boot_with_transparency` (383), resume reboot (636-650), the module doc
(18-19), genet's `from_shared_core` (61), the spatial compute plan's lines 25
and 56, and the consumer list from `Cargo.toml` searches; F7 the `grammar-g2`
diff statistics against main.
