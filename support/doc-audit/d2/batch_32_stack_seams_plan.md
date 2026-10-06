# Batch 32 — stack seams plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md | current | yes | 45 | 45 | 0 | 0 |
| **Totals** |  |  | **45** | **45** | **0** | **0** |

**Totals: 1 doc, 45 claims checked (45 holds, 0 stale, 0 unverifiable), 0 contradictions.**

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
- status line: "Status (2026-10-06): in progress. Seventy rulings in twenty-three rounds (S1 to S70); P1 landed on main (`1633be0c`); P2 staged (S27 to S31, four stages in §3.1) and landed on main (`40d7ae5e`); S32 (F19) landed on main (`48c08dee`); the S14 pass (S33) done (`8d7702d2`, receipt `support/doc-audit/d2/remediation_2026-10-06.md`); P3 and S7 done as documents; S3 to S6 carried into the dynamics grammar plan (G8, G9); S9 done by the identity lane (`b52edea7`)." — accurate: yes
- claims checked: 45 — holds: 45, stale: 0, unverifiable: 0

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
diff statistics against main. Added with the second pass: the census
behind F8 (each benign pair read at its definition, inker's `NodeKey` alias at
`routing/ids.rs` 20, the workbench plan's lines 35-36, document-host's README
on `Grant::from_authority`, platen's alias at `workbench.rs` 410-415); F9's two
`ViewIntent`s and the absence of any conversion between them; F10's two
`PersonaId`s; F11's README claims against `cargo metadata` (100 members, 13 under
`ports/`), the ports directory, and pandect's `Cargo.toml`; and the dynamics
plan's `6c3dca60` and `e932d526` as the carrying commits in §3.2. Added
2026-10-05: P1's owner, checked against the adoption plan's lines added at
`1a2e71db` (Mark's answer quoted there) and `projection_compile.rs`'s last
commit, `e387df16`. S9's completion checked against `b52edea7` on main and a
search of mien and the workspace for the old name. Round 3's S12 evidence
checked: the comment at `projection_compile.rs` 135 and its commit `534ae1c6`,
the remote projection host plan's lines 81, 389 and 579, the dataset's serde
derives, and its only JSON load at `web_projection.rs` 65 (a fixture and the
editor preview). F12 checked against `registry.rs` (`solve_via`'s doc and the
absence of any built-in registration), scenograph's `Arrangement::default`, and
graphshell's two id constants (`projection_compile.rs` 75-76). F13 checked
against `c79bb8c2`'s file list and the adoption plan lines it added, the
moved constants and mapping in `scenomise/src/projection.rs`, and a scan of
every local branch for unmerged compiler edits (two found, both merges of
main only). F13's correction checked against the three commits' authorship
and file lists, the adoption plan's host continuation entry (lines 88-97),
and the pitch arithmetic in `solve.rs` 312-322 with the footprint at
`projection.rs` 697. S19's premise checked against the graph-semantics
branch's record of the relay (`5666943e`) and the absence of compiler
commits on any ref since `c79bb8c2`. F14 and the P1 progress entry checked
against branch commit `dd2cb6fd` (its diff and the gate logs) and sceno's
`Stack` and `Geographic` docs. Round 10's evidence checked against the tree at `b6a50a4d`: `HostState`'s
60 fields (`cambium-rootstock/src/host.rs` 877-1007), genet's private
`ScopedDom` (`genet-scripted/livery.rs` 1611, genet `bf723d5d`), no host
constructing `GenetMultiRunner` outside cambium's own tests, and the winit
host's resume path booting a fresh core through `boot_surface`. Round 11's
evidence checked: `push_forest_projection` and `window_root` in `multi.rs`,
the owned layout's generic `LayoutDom` bounds, the four whole-document reads
in `cambium-winit-a11y/src/lib.rs` (212, 324, 369, 384), the pipeline's line
counts, and Woodshed's sources (no `HostState` field reached). F15 checked against Knot `54bb8cd` (`git grep` of
`compile_relationship_snapshot`; recounted with `git grep -c`: twelve mentions in
`apps/desktop/src/composition/recipe.rs`, one of them the import, one in `retention.rs`,
none in `tests/support/recipe.rs`),
mere's root `Cargo.toml` (no `scenomise` row in the mere.git patch table),
`dd2cb6fd` on `origin/main` through `19e6dc9f`, and the burn plan's §13.46
entry at `ac7f906d`. F16 checked against wgpu 30.0.1's
`src/backend/wgpu_core.rs` 752 and `src/api/device.rs` 26, and netrender
`9607d16f`'s `boot_async_shared` (a new instance per boot); F17 against genet
`bd3e8861`'s `genet-render-host` `rasterize_for` doc and rootstock's
`frame.rs` call; stage 1's entry against branch commit `76aa1bee` and its gate
and receipt logs. Stage 2's entry checked against branch
commit `03f8fe74`, its gate logs, and the review subagent's report. Stage 3's entry checked against branch
commit `21b0057f`, its gate logs, and the two control runs. F18 checked against winit
0.30.13's `platform_impl/windows/window.rs` 152 and `event_loop.rs` 1276-1298
and the stalled and passing headed logs; F19 against the Knot lane's report
(not rerun here); F20 against the review subagent's report and `windows.rs`;
stage 4's and the landing's entries against `7ae0e36e`, `e131273e`,
`40d7ae5e` and their gate logs; the F15 addition against `102aa548`. S32's entry and F21 checked against `3ecba102`, `48c08dee`, the gate
logs, the control runs with `card_issue` removed, and the nine runs of
graphshell's `projection` filter with and without S32.
