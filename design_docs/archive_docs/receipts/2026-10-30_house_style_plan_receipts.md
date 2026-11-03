# House-style rollout receipts

## 2026-11-02: doctrine pass

- Read current doctrine, house-style D1-D12 and eidetic's line-ceiling E1-E4.
- Mark answered the three doctrine cards; recorded verbatim as house-style
  D13 (retire isocosm Method 3), D14 (reuse order), D15 (consolidate within
  seven Stack statements).
- VTT chat `Plan VTT Simulation Development` reported no pending edits or
  collision with the doctrine, case record or generated decision files.
- After staging source records, `responsa regen --stage` regenerated the
  manifest; `responsa check` reported a current manifest and doctrine within
  budget. Stack has seven statements; its new key is stack.8, with stack.5
  retired into stack.3 rather than reused. Existing historical links can
  follow the typed replacement; active doctrine links use stack.3.
- Isocosm's corresponding case is 1160; it retires method.3 and points to
  `mere:stack.7`. Generated records are regenerated, never hand-merged.

The shared lint set and remaining repository inventory/adoption are not
qualified by this doctrine-only gate. Retained receipt cap: 100 MB.

### Final cross-repository gate

Mere's `responsa check` and isocosm's `responsa check` both pass after
isocosm's Method 3 retirement. Cross-repository links target its live case
1160; the retired method.3 key remains a local replacement target.
Isocosm's full `prek run --all-files` passes. The approved CLAUDE.md edit
changes only the source-size rule's citation.

## 2026-11-03: inventory and local adoption

Owner: chat `01a19f86-cb71-7820-a6ac-aaec39d8bfd4`. Retained-byte cap:
100 MB across this lane; compact text, no full compiler captures.

### Scope and controls

The reviewed source ceiling is 600 lines. TOML audit: 25 configurations,
900 individual exceptions, every exception/exclusion has a reason. Genet
declares 167 exceptions and five imported/generated exclusions; Mere 389
and two. ESP was checked and committed on main (nine exceptions), then its
original `distillery-receive` branch was restored. Its separate receiving
work was preserved.

All 25 staged 601-line controls failed both `prek run --all-files` and an
actual `git commit`. Every control kept HEAD unchanged; the temporary source
was removed from the index and disk. All positive ceiling runs passed.
The Genet runner-filter audit covered 187,905 tracked paths with zero
mismatches against its declared checker exclusions.

393 owned package manifests resolve the five D6 warnings through their
package table or workspace inheritance. Imported patches, generated source
and frozen fixture manifests retain their own inputs. Standalone suppliers
also passed `cargo clippy --manifest-path crates/{line-ceiling,responsa}/Cargo.toml
--locked --all-targets` independently.

### Adoption ledger

Full gate: `prek run --all-files`, Windows native Cargo package/default
feature scope except the explicitly recorded selections below. Every
qualified row also has the real commit-rejection control above. Hashes
name pushed main, except Mere, whose changes are in this receipt’s commit.

| Repository | Oversize exceptions | Full gate / publication |
|---|---:|---|
| armillary | 4 | pass; `22ce3a215d6d` |
| cleromancy | 9 | pass; `135f73108416` |
| dramatis | 9 | pass; `260a431b34e8` |
| eidetic | 20 | pass; `663dc40a30a4` |
| emblem | 3 | pass; `1ce843c13957` |
| gemot | 13 | pass; `91113d25deeb` |
| genet | 167 | pass; `70a54f6f43cb` |
| isocosm | 0 | pass; `b43efefe8e97` |
| knot-editor | 39 | ceiling passes; receiving gate fails; draft preserved |
| mer3ly | 21 | pass; `e22fd460bf27` |
| mere | 389 | pass; this receipt’s commit |
| merely-made.github | 0 | pass; `5f4910c73fda` |
| mora | 0 | pass; `42c232c3ac60` |
| murm | 22 | pass; `b0e31de8b581` |
| netrender | 19 | pass; `58d80d8aee04` |
| pipit | 0 | pass; `1281de0901bf` |
| smolweb | 13 | pass; `0b22d00e2249` |
| turnstone | 0 | pass; `57bd730b3961` |
| turquet | 15 | pass; `84d4089cf2a3` |
| wavicle | 1 | pass; `60bc853ebfb0` |
| wgpu-graft | 9 | pass; `dea7c5f793cc` |
| wgpu-scry | 22 | pass; `967a0fa353d4` |
| wgpu-weld | 12 | pass; `f2c1779b98db` |
| woodshed | 104 | ceiling passes; receiving gate fails; draft preserved |
| esp | 9 | pass; `a55709273168` |

### Preserved coverage and repairs

- Graft selects grafting/grafting-frame; Scry selects scrying; Murm selects
  its portable root libraries. Optional/platform/firmware work remains under
  its own gates. Smolweb’s local Clippy gate covers all features. Mer3ly’s
  local gate covers root, repo-graph and radio-mirror. Existing CI Clippy
  invocations in Smolweb, Wavicle and Mer3ly were moved local per D11;
  publication, test, format and other workflow checks were retained.
- Mere, Dramatis and Genet run their existing dependency bans. Genet keeps
  `multiple-versions = "deny"` and the `num` ban. Forty exact inspected
  duplicate-version exemptions name their callers; newer duplicate versions
  still fail. Its patched DOM regex and independent registry engine regex
  sources remain separate. Existing cross-profile unnecessary-skip warnings
  are advisory and were not removed indiscriminately.
- Mere’s allowed custody host package name changed from stale `djinn` to
  actual `mere-resident`; the custody boundary is otherwise unchanged. Its
  full gate, including all-feature bans, passed.
- Turquet’s VSOP87D module and one rounded-coordinate test permit
  `approx_constant` locally; the coefficient decimals were untouched.
  Netrender’s deliberately reversed-range test permits that lint locally
  while preserving the malformed-input assertion.
- Isocosm passed canonical `cargo check --workspace --all-features
  --all-targets --locked` as well as the full local gate. Its ignored
  `.cargo/tabletop-local.toml` still points at the retired Mere armillary
  path; that optional local wrapper failed and was preserved. The committed
  graph passed through the direct command expressly allowed by CLAUDE.md.
  VTT reported no collision before the scoped manifest/hook edits.

### Remaining receiving scope

Knot’s Clippy gate fails at resolution: Dramatis is locked at 0.0.3 while
the resolved main workspace offers 0.0.4. Both the targeted offline dry-run
and the precise-Dramatis dry-run also move Mere, Genet and Murm cohorts.
No real update was applied.

Woodshed’s Clippy gate fails compiling HAL30’s Windows backend:
gpu-allocator 0.28.0 has a windows 0.57.0 edge while wgpu-hal 30.0.1 uses
windows/windows-core 0.62.2. This matches the alignment invariant already
documented by Graft. No lockfile repair was applied.

The scope card asks whether the rollout includes the necessary receiving
cohort repairs, or keeps inventory/lints while deferring blocked hooks.
There is no answer recorded yet. These two repositories retain only the
reviewable unstaged draft; our newly installed pre-commit shims were
uninstalled so the unresolved gate does not block their other work. H3
remains open.

### Workspace hygiene

Each Cargo invocation, including commits that invoke hooks, used the stable
`C:	\cargo-targets\<repo>` target, its `build` directory and two jobs.
No isolated Cargo home, target or worktree was created. Existing generated
build/cache output was retained for reuse; no cleanup of other lanes’
evidence or source was performed. The original handoff’s temporary location
is not needed to understand this receipt and the current plan.

## 2026-11-03: receiving repairs and H3 closeout

The returned cards are recorded verbatim in the house-style plan, in answer
order: D16 preserves Turquet's reference coefficients with the module-only
allowance; D17 includes necessary receiving-cohort repairs. The rulings and
regenerated manifest were pushed in Mere `83fc7e84a5b3`.

### Final receiving results

| Repository | Pushed main | Final full local gate |
|---|---|---|
| Knot | `8148b669bb2264058ba29331638dfceedf877d43` | pass, 43.25 s |
| Woodshed | `445b7deca620f598b9ff3141d65c5549ba544d22` | pass, 15.53 s |

Both ran `prek run --all-files` on their final source/configuration/lock
inputs. Their pre-commit shims were reinstalled, and both real adoption
commits passed their installed hooks. Their earlier 601-line full-run and
real-commit rejection controls use the same ceiling configuration and tool.
All 25 repositories now have pushed adoption and passing full local gates.
The earlier 23-repository checkpoint above is preserved as run history.
H1-H4 and the linked tool plan's E1-E4 are complete.

Knot's receiving graph uses these coherent source cohorts:

- Mere `fc0e92d0132f6cc44ba9e6c53410b8ced594f166`.
- Genet `ad362723109b7e1738dfce13a625cf1a1b5e9ed1`.
- Murm transport/WebRTC `c332e9c528d8505564ba54b2453463cf062ae9f9`.
- Dramatis `260a431b34e8e9e888fc3b57c8db5a8483793949`.

The disabled Retinue selection remains at its prior Murm revision. The
initial offline precise-Dramatis update was followed by the targeted
AccessKit alignment; the Dramatis packages were then aligned together.
Cargo's updates also rewrote 15 unrelated registry dependency records.
Those complete records were restored from the original lock, preserving
all registry versions, checksums and dependency selections. Final
`cargo metadata --locked --offline --format-version 1` passes: zero
resolved-package gaps, zero resolved dependency edges absent from the
lock, and all 12 first-party checkout heads match their reported sources.
Every registry package record is identical to the pre-repair lock.
Existing unused patch/URL-alias warnings were preserved; the rollout does
not normalize unrelated source identities.

Woodshed changes exactly the gpu-allocator 0.28.0 dependency edge from
windows 0.57.0 to windows 0.62.2, alongside wgpu-hal 30.0.1. Locked metadata
confirms BLE still resolves windows 0.57.0. Its remaining Clippy error was
the intentionally empty `1..=0` scope in
`wide_necks_wrap_readably_and_scopes_stay_bounded`; a function-only
`reversed_empty_ranges` allowance keeps the existing assertion intact.
The final gate passes without changing playback or coefficient data.

### Qualification and retained output

These gates qualify the local house-style adoption on the documented native
Cargo scopes. They do not requalify historical headed, audio, physical-peer,
firmware or other-platform receipts on new dependencies. Those remain tied
to their original inputs and owner plans. The prior Linux tagged-tool
installation remains the tool plan's cross-platform receipt.

The stable Cargo paths are `C:/t/cargo-targets/<repo>` and their `build`
directories, with two jobs per invocation. This forward-slash spelling also
clarifies the path in the earlier hygiene paragraph. No isolated target,
Cargo home or worktree was created or retained. Rollout helper/state files
in system Temp are removed after final publication; the source records,
configs and this appended receipt are the durable handoff. Receipt cap
remains 100 MB; only compact text is retained.
