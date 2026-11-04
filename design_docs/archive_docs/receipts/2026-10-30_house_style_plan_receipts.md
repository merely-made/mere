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

## 2026-11-03: H5 fifty-file continuation

Target: 50 additional existing oversize paths. Excludes the four Armillary
and four Smolweb paths retired before this request. Owner: chat
01a19f86-cb71-7820-a6ac-aaec39d8bfd4. Total retained cap: 100 MB across all
source-owner receipts; compact text only. Historical captures remain intact.

Qualification requires each original path and its replacements to fit the
ceiling, retirement of its exception, scoped tests/full gate and a pushed
implementation commit. Initial tally: 0/50. Published batches append their
original paths, owning receipt and commit below.

### smolweb — 9 additional paths

Published: `91ae0a6731c1cf1059099908f566f82354a47bc4`. Owner plan: `smolweb/design_docs/implementation_strategy/2026-11-03_source_splits_plan.md`;
its archived receipt records line counts, commands and limits.

- `crates/errand/src/lib.rs`
- `crates/errand/src/parse/feed.rs`
- `crates/gopher-protocol/src/client.rs`
- `crates/guppy-protocol/src/server.rs`
- `crates/misfin/src/server.rs`
- `crates/misfin/src/client_deadline_tests.rs`
- `crates/misfin/src/server_deadline_tests.rs`
- `crates/errand/tests/client_limits.rs`
- `crates/text-protocol/src/lib.rs`

### emblem — 3 additional paths

Published: `9bc1a1e004bdfda69ad6c10584fc14a38307967a`. Owner plan: `emblem/design_docs/2026-11-03_source_splits_plan.md`;
its archived receipt records line counts, commands and limits.

- `src/encode/style.rs`
- `src/encode/writer.rs`
- `src/vm.rs`

### dramatis — 20 additional paths

Published: `e26b434a1011b2ff509d0d6a1553cbbb03902669`. Evidence: `dramatis/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`.

- `crates/castellan-client/src/custody.rs`
- `crates/castellan-client/src/custody_client.rs`
- `crates/castellan-client/src/custody_identity.rs`
- `crates/notochord/src/handshake.rs`
- `crates/notochord/tests/secure_channel.rs`
- `crates/personae/src/carry/mod.rs`
- `crates/personae/src/sealed_record_storage.rs`
- `crates/personae/src/ssh_ca.rs`
- `crates/castellan/src/authority.rs`
- `crates/castellan/src/authority_lock_tests.rs`
- `crates/castellan/src/custody/agent.rs`
- `crates/castellan/src/custody/passphrase_storage.rs`
- `crates/castellan/src/custody/vault.rs`
- `crates/castellan/src/cxf/parse.rs`
- `crates/castellan/src/cxf/tests.rs`
- `crates/sporran/src/wallet_grant/certificate.rs`
- `crates/djinn-custodian/src/core.rs`
- `crates/dramatis-resident/src/custody_host/remote.rs`
- `crates/dramatis-resident/src/custody.rs`
- `crates/dramatis-resident/src/startup_vault.rs`

### netrender — 12 additional paths

Published: `2073c50503bc2a4706f17640feac1d08df3841da`. Evidence: `netrender/netrender-notes/receipts/2026-11-03_source_splits_plan_receipts.md`.

- `netrender/src/external_texture.rs`
- `netrender/src/renderer/rg2c.rs`
- `netrender/src/scene/build.rs`
- `netrender/src/tile_cache/index.rs`
- `netrender/src/vello_backends.rs`
- `netrender/src/vello_backends/sessions.rs`
- `netrender/tests/rg2a_scene_corpus.rs`
- `netrender/tests/vb3_sparse_text.rs`
- `paint_list_api/src/lib.rs`
- `paint_list_render/src/lib.rs`
- `paint_list_render/src/tests.rs`
- `paint_list_render/tests/resource_bundle.rs`

Emblem license-header follow-through is published at `0975e274eec7`; the
original three-file qualification count is unchanged. Tally: **44/50**.

### turquet — six additional paths

Published: `98190539e2406ec8b1d2eb24051c0b7dc5e78ab4`. Evidence:
`turquet/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`.

- `src/foundation.rs`
- `src/lunar.rs`
- `src/planet/mod.rs`
- `tests/altitude_crossings.rs`
- `tests/events.rs`
- `tests/houses.rs`

### Final H5 audit: 50/50

| Source owner | Additional original files | Remaining exceptions | Published main |
|---|---:|---:|---|
| smolweb | 9 | 0 | `91ae0a6731c1cf1059099908f566f82354a47bc4` |
| dramatis | 20 | 1 | `e26b434a1011b2ff509d0d6a1553cbbb03902669` |
| netrender | 12 | 7 | `2073c50503bc2a4706f17640feac1d08df3841da` |
| turquet | 6 | 9 | `98190539e2406ec8b1d2eb24051c0b7dc5e78ab4` |
| emblem | 3 | 0 | `0975e274eec7a73858c6b1c262f9f9b3d60329ee` |

The audit checks fifty distinct original paths against each owner's baseline:
each began above 600 physical lines, each now fits, its precise exception is
removed, and every new Rust module fits. All five source checkouts are clean
and match their pushed `origin/main`. The earlier eight splits are excluded.

All five actual implementation commit hooks pass. Source-owner receipts
record tests and limits: Smolweb 650 cases (one unchanged ignored), Emblem
157, Dramatis 833 plus 21 all-feature resident cases, Netrender 156 scoped
cases (three existing physical-receipt cases ignored), and Turquet 163.
Dramatis repairs Windows adoption's write access before flushing; Netrender
repairs only its native fixture's enum exhaustiveness. Its pre-existing full
workspace compose pixel failure remains open at the original ±4/16 threshold.
Optional-feature compilation and public docs are recorded per owner; this
is not a new macOS/Linux, headed, live-SSH or physical acceptance receipt.

The aggregate documentation hook invokes the same configured hooks with
explicit documentation paths through a temporary per-command hook directory.
This avoids automatic stashing of other sessions' live source WIP. Ceiling,
Responsa and dependency bans run; Clippy's normal source selector skips docs.
The installed hook configuration is unchanged. Responsa reads the staged
snapshot, preserving unrelated unstaged plan and receiving work.

Cleanup: only compact text receipts are retained, below 100 KB across this
aggregate and the five owner receipts, within the 100 MB total cap. This
pass's Temp helper, state, logs and temporary hook are removed after final
publication. Builds reused stable approved `C:/t/cargo-targets/<repo>`
directories with two jobs. No isolated target, Cargo home or worktree was
created or retained; existing source, fixtures and historical evidence stay.

## 2026-11-03: H6 second fifty-file continuation

Target: fifty additional originals, excluding H5's fifty and the earlier
eight. Owner: chat 01a19f86-cb71-7820-a6ac-aaec39d8bfd4. Selected: Woodshed
35 and Eidetic 15. Initial count: 0/50. Same qualification rule and shared
100 MB retained receipt cap; compact summaries, preserving original captures.
Owner plans and receipts record each phase; published paths append here.

### H6 Eidetic: fifteen paths

Published: `5b805a1eb808b32f890feb2c7464ef6b2471a9dc`. Evidence:
`eidetic/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`.

- `crates/chartulary/src/commit.rs`
- `crates/chartulary/src/spine.rs`
- `crates/eidetic-core/src/browsing/page.rs`
- `crates/eidetic-core/src/models/answer.rs`
- `crates/eidetic-search/examples/eidetic-recall.rs`
- `crates/eidetic-search/src/index.rs`
- `crates/eidetic-search/tests/lexical_ngram_recall.rs`
- `crates/luggage/src/install.rs`
- `crates/luggage/src/updater.rs`
- `crates/muniment/src/directory_backend.rs`
- `crates/servitor/src/delegation.rs`
- `crates/servitor/src/gate.rs`
- `crates/servitor/src/run.rs`
- `crates/stickleback/examples/inventory_compare/model.rs`
- `crates/stickleback/src/drop.rs`

Tally: **15/50** for H6.

### H6 Woodshed S1: twelve paths

Code: `297b162d888c3590e99bdaec6be21fc2487354d9`; published through clean integration `d2f990cf770a6fff2ccd321954010dd9d34c8b98`.
Evidence: `woodshed/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`.

- `crates/audio-primitives/src/alignment.rs`
- `crates/audio-primitives/src/onset.rs`
- `crates/hocket-model/src/history.rs`
- `crates/hocket-project/src/handoff.rs`
- `crates/hocket-recorder/src/host_policy.rs`
- `crates/woodshedding/src/chord.rs`
- `crates/woodshedding/src/exercise.rs`
- `crates/woodshedding/src/fretboard.rs`
- `crates/woodshedding/src/notation.rs`
- `crates/woodshedding/src/progression.rs`
- `crates/woodshedding/src/rehearsal.rs`
- `crates/woodshedding/src/scale.rs`

Tally: **27/50**. The concurrent Redshank note-layout change remains its
owner's work. Root theory/DSP/history qualification preserves the original
16 ms assertion; the recorded initial timing miss passes in the isolated
and full serial reruns.


### H6 Woodshed S2/S3: twenty-three paths

Published: `cf2b09196adce374b65d401dfa7a2d0461e94b9c`. Evidence continues in the same Woodshed owner receipt.

- `crates/woodshed-core/src/chord_approach.rs`
- `crates/woodshed-core/src/comparison_disclosure.rs`
- `crates/woodshed-core/src/history.rs`
- `crates/woodshed-core/src/lib.rs`
- `crates/woodshed-core/src/loop_capture.rs`
- `crates/woodshed-core/src/loop_plan.rs`
- `crates/woodshed-core/src/practice_clock.rs`
- `crates/woodshed-core/src/session_overview.rs`
- `crates/woodshed-core/src/stage_context.rs`
- `crates/woodshed-core/src/storage.rs`
- `crates/woodshed-graph/src/lib.rs`
- `crates/woodshed-notation/src/lib.rs`
- `crates/hocket-recorder/src/lib.rs`
- `crates/hocket-views/src/leaves.rs`
- `crates/hocket-views/src/surface.rs`
- `crates/woodshed-audio/src/input.rs`
- `crates/woodshed-audio/src/offline/song_export/tests.rs`
- `crates/woodshed-audio/src/output_runtime.rs`
- `crates/woodshed-audio/src/output_runtime/processor.rs`
- `crates/woodshed-audio/src/output_runtime/tests.rs`
- `crates/woodshed-audio/src/song.rs`
- `crates/woodshed-instrument/src/midi.rs`
- `crates/woodshed-instrument/src/midi/ble.rs`

Tally: **50/50 for H6**, distinct from H5's fifty and the earlier eight.
Woodshed's final 35/35 and Eidetic's 15/15 are pushed to main. Every original
was over 600 lines at its recorded baseline; it and every replacement now
fit. Fifty exact exceptions retire, leaving five in Eidetic and 69 in
Woodshed. Root manifests/locks and source notices remain intact.

Woodshed's 967-case baseline/post suite, default Windows BLE compilation,
notation/audio/instrument no-default checks, public docs and full local/commit
hooks pass. The initial MIDI-disabled integration-test failure is retained
in its owner receipt alongside the missing guard repair and successful
reruns. Operation/literal, function-signature, type-layout and facade-export
review covers the final moves. Eidetic's 746 cases, optional recall-example
type check, docs and full hooks are recorded in its owning receipt.
These are software checks; physical-device, audio-quality, headed and
other-platform acceptance remain under their existing owners.

The aggregate docs commit scopes its per-command hook to its three changed
docs, using the configured prek runner with ceiling, Responsa and dependency
bans. The final run supplies cargo-deny with a freshly generated
`cargo metadata --all-features --frozen` snapshot through `--metadata-path`,
in an otherwise identical temporary runner config. This avoids cargo-deny's
separate fetch while preserving the locked, all-feature graph and every bans
check. Metadata bytes, HEAD and Cargo/config input hashes are verified again
inside the hook. Clippy's normal source selector skips docs. This preserves concurrent
unstaged Mere plan work without automatic stashing; installed hook config
is unchanged. Responsa regeneration/check uses the staged snapshot.

Only compact Markdown receipts remain, within H6's shared 100 MB cap.
This batch's owned Temp helper, state, audit script, logs and temporary hook
are removed after publication. Stable approved repository Cargo targets
were reused with two jobs. No isolated target, Cargo home or worktree was
created or retained. Existing source and historical evidence are preserved.


### H6 final-run clarification

The aggregate is published at `16dd9d438`. The snapshot supplied to the
checker contains 1,857 packages (9,954,469 bytes), generated with frozen,
all-feature metadata; HEAD and Cargo/config inputs were verified.

Correction to the optimization description above: cargo-deny 0.20.2 still
prefetches unconditionally when given metadata. The supplied snapshot did
not eliminate that phase. Its queued prefetch eventually completed after
the shared cache lock cleared, and ceiling, Responsa and dependency bans
all passed; Clippy correctly skipped the doc-only selection. The attempted
frozen/snapshot changes did not bypass a check or establish a faster path.
This addendum corrects the run description. Its commit uses the ordinary
configured runner, with the same three-doc selection and installed hooks
unchanged.


## 2026-11-03: H7 third fifty-file continuation

Target: fifty additional originals, excluding H5, H6 and the earlier eight.
Selected: Knot 28, Mer3ly 12, Cleromancy 9 and Wavicle 1. Initial count: 0/50.
Owner: chat 01a19f86-cb71-7820-a6ac-aaec39d8bfd4. Owning plans carry software
scopes and compact evidence under the shared 100 MB retained cap.

### H7 Wavicle: one path

Published: `4c384df20837ca5f098eba2cb337c03d4a3c1220`. Original: `tests/conformance.rs`. Evidence:
`wavicle/design_docs/archive_docs/receipts/2026-07-15_wavpack_codec_plan_receipts.md`.
Default/reader-only inventories match at 21/16 passed, encode-only/docs and
full local/commit hooks pass. Zero codec exceptions remain. Tally: **1/50**.

### 2026-11-03: H7 Mer3ly twelve published

Commit `706014ffab08b1cbf37bc2a42c52a04d1154adf3` is pushed. Owner evidence: `mer3ly/docs/receipts/2026-11-03_source_splits_plan_receipts.md`. Site 110 and standalone repo-graph 43 tests match baseline; both native roots, graph Wasm compilation, docs and full local/actual commit hooks pass. Twelve distinct original exceptions retire.

### 2026-11-04: H7 Cleromancy default comparison

The nine prepared originals compile. Untouched default baseline: 146 passed,
three failed, one ignored. Post-split exact Cargo-built executables: 147 passed,
two failed, one ignored, with matching complete leaf-name inventory. The LF
checkout repair makes shuffled-deck pass without changing its Git golden.
Existing authoring-DOM and declared broker-route failures remain unchanged;
no assertions are relaxed. Feature/docs/full-hook qualification is pending.

### 2026-11-04: H7 Knot baseline and prepared moves

Untouched workspace/peers baseline: 772 passed, 14 failed, eight ignored.
Eight desktop cases cannot reach Readings/recipe controls; six legacy
checkpoint/collection cases reach an explicit Unix-only storage replacement
refusal on Windows. No selected Rust file changed during that run. All 28
prepared originals/replacements now fit; staged semantic review matches 428
changed bodies and all signatures/type fields, with only reviewed Rustfmt
layout/import ordering differences. No paths qualify before post tests and
full/actual hooks. Owner pointer: `knot-editor/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`; full compact evidence is outside Git under Code/testing/knot-editor/receipts.

### 2026-11-04: H7 Cleromancy nine published

Source `d9de7f2d3e239b86f4855c0aac1a551acfb20ef0` is pushed, with actual
configured commit hooks passing and a clean source checkout. Nine original
exceptions retire; every original/new module fits. Default leaf-name inventory
matches, 147 pass/two pre-existing failures/one ignore; analytic natal 18/18,
no-default-feature library check, doctests, docs and full local hooks pass.
The expected JSON Git blob is unchanged, and .gitattributes fixes its Windows
checkout newline policy. Product issues are carried once in divination O7.
Owner pointer: `cleromancy/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`; complete compact evidence: `Code/testing/cleromancy/receipts/2026-11-03_source_splits_plan_receipts.md`, outside Git per divination D3.

### 2026-11-04: H7 fifty additional originals published

All fifty originals and every new Rust module fit 600 physical lines at the
source commits below. Exactly fifty named exceptions retire. Publication
ancestry is verified against origin/main; all four source checkouts are clean.
Cargo manifests/locks, installed hook definitions, CI and CLAUDE.md are unchanged
across each owner source pass. Concurrent Mere plan edits are preserved.

| Owner | Published source | Originals | New Rust modules | Remaining exceptions |
|---|---|---:|---:|---:|
| wavicle | `4c384df20837ca5f098eba2cb337c03d4a3c1220` | 1 | 1 | 0 |
| mer3ly | `706014ffab08b1cbf37bc2a42c52a04d1154adf3` | 12 | 35 | 9 |
| cleromancy | `d9de7f2d3e239b86f4855c0aac1a551acfb20ef0` | 9 | 16 | 0 |
| knot-editor | `7e66a1dfd644e74fea775d5782a4436ebc09ff95` | 28 | 47 | 11 |

| Original | Baseline lines | Published lines |
|---|---:|---:|
| `wavicle/tests/conformance.rs` | 882 | 383 |
| `mer3ly/crates/repo-graph/src/lib.rs` | 3664 | 427 |
| `mer3ly/crates/repo-graph/src/portable.rs` | 622 | 420 |
| `mer3ly/src/artifact.rs` | 2070 | 314 |
| `mer3ly/src/devices.rs` | 624 | 529 |
| `mer3ly/src/pages/devices.rs` | 1173 | 525 |
| `mer3ly/src/pages/projects.rs` | 922 | 462 |
| `mer3ly/src/pages/radio.rs` | 1028 | 579 |
| `mer3ly/src/pages/repositories.rs` | 989 | 578 |
| `mer3ly/src/repositories.rs` | 1574 | 568 |
| `mer3ly/src/repository_history.rs` | 666 | 520 |
| `mer3ly/src/retinue_traces.rs` | 680 | 425 |
| `mer3ly/src/site.rs` | 716 | 477 |
| `cleromancy/src/consultation/mod.rs` | 765 | 178 |
| `cleromancy/src/host/mod.rs` | 662 | 245 |
| `cleromancy/src/ui/native/appearance_tests.rs` | 1256 | 567 |
| `cleromancy/src/ui/native/mod.rs` | 792 | 516 |
| `cleromancy/src/ui/native/scenario_driver.rs` | 1611 | 544 |
| `cleromancy/src/ui/resident.rs` | 1192 | 200 |
| `cleromancy/src/ui/state.rs` | 1033 | 599 |
| `cleromancy/src/ui/view/chart/natal.rs` | 806 | 220 |
| `cleromancy/tests/natal.rs` | 763 | 499 |
| `knot-editor/apps/desktop/src/composition/metre.rs` | 1124 | 511 |
| `knot-editor/apps/desktop/src/composition/recipe/dynamics.rs` | 1145 | 595 |
| `knot-editor/apps/desktop/src/documents.rs` | 1462 | 349 |
| `knot-editor/apps/desktop/src/graph.rs` | 1214 | 309 |
| `knot-editor/apps/desktop/src/link_workflow.rs` | 828 | 484 |
| `knot-editor/apps/desktop/src/main.rs` | 768 | 393 |
| `knot-editor/apps/desktop/src/preferences.rs` | 649 | 357 |
| `knot-editor/apps/desktop/src/scenario.rs` | 637 | 267 |
| `knot-editor/apps/desktop/src/theme_session.rs` | 607 | 529 |
| `knot-editor/apps/desktop/src/workspace_commands.rs` | 738 | 490 |
| `knot-editor/apps/desktop/tests/font_roles.rs` | 628 | 508 |
| `knot-editor/apps/desktop/tests/scenario_lane.rs` | 707 | 489 |
| `knot-editor/apps/desktop/tests/tabard_appearance.rs` | 883 | 563 |
| `knot-editor/crates/knot-composition/src/wordnet_import.rs` | 946 | 336 |
| `knot-editor/crates/knot-composition/tests/recipes.rs` | 721 | 538 |
| `knot-editor/crates/knot-editor/src/bin/knot_endpoint.rs` | 953 | 581 |
| `knot-editor/crates/knot-editor/src/clip_evidence.rs` | 615 | 505 |
| `knot-editor/crates/knot-editor/src/embedding.rs` | 745 | 468 |
| `knot-editor/crates/knot-editor/src/mark.rs` | 1126 | 428 |
| `knot-editor/crates/knot-editor/src/mere_overlay.rs` | 655 | 281 |
| `knot-editor/crates/knot-editor/src/publish.rs` | 965 | 575 |
| `knot-editor/crates/knot-editor/src/publish_host.rs` | 1152 | 427 |
| `knot-editor/crates/knot-editor/src/resident.rs` | 1224 | 154 |
| `knot-editor/crates/knot-editor/src/rosette.rs` | 671 | 589 |
| `knot-editor/crates/knot-editor/src/search.rs` | 1042 | 448 |
| `knot-editor/crates/knot-editor/src/vault.rs` | 619 | 480 |
| `knot-editor/crates/knot-editor/tests/checkpoint_confidentiality.rs` | 858 | 563 |
| `knot-editor/crates/knot-file-catalog/src/lib.rs` | 907 | 440 |

Owner evidence:

- `wavicle/design_docs/archive_docs/receipts/2026-07-15_wavpack_codec_plan_receipts.md`: default 21 and reader-only 16 match baseline; encode/docs and full local/actual hooks pass. Pinned reference-decoder branches execute; fixtures are unchanged.
- `mer3ly/docs/receipts/2026-11-03_source_splits_plan_receipts.md`: native site 110 and graph 43 match baseline; native roots, graph Wasm compile, docs and all configured local/actual hooks pass. Deployed/browser qualification stays separate.
- `cleromancy/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md` points to `Code/testing/cleromancy/receipts/2026-11-03_source_splits_plan_receipts.md`, outside Git per divination D3. Default 147 pass/two existing failures/one ignore; baseline 146/three/one differs only by canonical LF fixture repair, with identical complete inventory. Analytic natal 18, narrow-feature/docs and full local/actual hooks pass. Product residuals carry once in divination O7.
- `knot-editor/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md` points to `Code/testing/knot-editor/receipts/2026-11-03_source_splits_plan_receipts.md`, outside Git per its policy. Full workspace/peers baseline and post both have 772 pass/14 fail/eight ignore, matching inventories and failed identities. Eight cases concern unavailable desktop controls; six hit explicit Unix-only legacy checkpoint replacement refusal on Windows. Provider CLI 8, minimal editor and retention-free desktop checks, workspace docs, explicit workspace/all-target peer Clippy and full local/actual hooks pass. After the optional fixture guard repair, focused default scenarios preserve ten passes and the same two failures, with all twelve cases present. Warning-level lint diagnostics remain. O4 remains open for excluded standalone packages and its named receiving-round obligations.

Source notices, literals, type fields, signatures, public facades, numerical
values and fixture Git bytes are preserved. Reviewed responsibility factoring
is recorded in owner receipts; no assertion is relaxed, and no lint allow or
CI change is introduced. Source moves account for path-loaded parents and
optional fixture cohorts. Headed, model, device, standalone-package, deployed
site and other-platform qualification keep their previous boundaries.

The retained cap is 100 MB across H7. Compact summaries carry required evidence;
owned transient scripts/logs are cleaned after publication. No isolated target,
Cargo home, branch or worktree was required.

### 2026-11-04: H8 selection

Fifty new originals selected: Eidetic 5, Gemot 13, Woodshed 32. Baselines
precede source edits. Compact summaries retained under the shared 100 MB cap;
prior evidence and concurrent work remain intact.

### 2026-11-04: H8 Eidetic five published

`171f7892024c4af3897f5eb9e5e92f1fa1c5e930` is pushed. All originals and
new modules fit (maximum 557), five exact exceptions retire and zero remain;
imported patches stay excluded. Workspace baseline/post 746 pass/five ignore
with identical inventory; Stickleback docs and full local/actual hooks pass.
Review matches 194 bodies plus signatures/type fields, with no semantic or
format-only difference after scope normalization. Owner evidence appends to
`eidetic/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`.

### 2026-11-04: H8 fifty additional originals published

| Owner | Originals | New Rust files | Source commit | Exceptions left |
|---|---:|---:|---|---:|
| eidetic | 5 | 15 | `171f7892024c4af3897f5eb9e5e92f1fa1c5e930` | 0 |
| gemot | 13 | 42 | `12ee625e11f8ac25322c1486144ec5a116b6c3f1` | 0 |
| woodshed | 32 | 65 | `a113b659b0977a6eae9d5f25a2a07c1dc68cdef6` | 37 |

All source commits are pushed main; each source checkout is clean and equals
its remote-tracking main. Fifty distinct originals exceed 600 lines at their
untouched baselines and do not repeat prior qualified originals. All fifty
originals and all 122 extracted Rust files now fit 600 physical lines; every
selected exception retires exactly once. No new exception or lint allow.

Qualified originals (per-file before/after counts live in each owner receipt):

**eidetic (5):**

- `crates/stickleback/src/drop_io.rs`
- `crates/stickleback/src/group_lane.rs`
- `crates/stickleback/src/group_session.rs`
- `crates/stickleback/src/processor.rs`
- `crates/stickleback/src/store.rs`

**gemot (13):**

- `crates/commons/src/call.rs`
- `crates/commons/src/chat.rs`
- `crates/commons/src/encrypted.rs`
- `crates/commons/src/lib.rs`
- `crates/gemot/examples/author-offline-publication.rs`
- `crates/gemot/src/moot/flora.rs`
- `crates/gemot/src/moot/records/collection.rs`
- `crates/gemot/src/moot/records/roster.rs`
- `crates/gemot/src/moot/records/store.rs`
- `crates/gemot/src/moot/service.rs`
- `crates/gemot/src/moot/tulpa.rs`
- `crates/gemot/src/moot/typed_authorization.rs`
- `crates/mien/src/ledger.rs`

**woodshed (32):**

- `crates/woodshed-genet/src/audio/input_admission/tests.rs`
- `crates/woodshed-genet/src/audio/loop_projection.rs`
- `crates/woodshed-genet/src/audio/practice_clock.rs`
- `crates/woodshed-genet/src/drive.rs`
- `crates/woodshed-genet/src/leaves.rs`
- `crates/woodshed-genet/src/main.rs`
- `crates/woodshed-genet/src/midi.rs`
- `crates/woodshed-genet/src/persona.rs`
- `crates/woodshed-genet/src/persona_preservation_tests.rs`
- `crates/woodshed-genet/src/scenario/mixed_looper.rs`
- `crates/woodshed-genet/src/scenario/physical_audio.rs`
- `crates/woodshed-genet/src/scenario/practice_clock.rs`
- `crates/woodshed-genet/src/session.rs`
- `crates/woodshed-genet/src/storage.rs`
- `crates/woodshed-genet/src/sync.rs`
- `crates/woodshed-genet/src/wav_export.rs`
- `crates/woodshed-views/src/appearance.rs`
- `crates/woodshed-views/src/fretboard_leaf.rs`
- `crates/woodshed-views/src/stage/canvas_camera.rs`
- `crates/woodshed-views/src/stage/context.rs`
- `crates/woodshed-views/src/stage/contextual_navigation.rs`
- `crates/woodshed-views/src/stage/looper.rs`
- `crates/woodshed-views/src/stage/looper/recorder.rs`
- `crates/woodshed-views/src/stage/music_editor.rs`
- `crates/woodshed-views/src/stage/musical_context.rs`
- `crates/woodshed-views/src/stage/practice_clock.rs`
- `crates/woodshed-views/src/stage/rehearsal.rs`
- `crates/woodshed-views/src/stage/related.rs`
- `crates/woodshed-views/src/stage/relationship.rs`
- `crates/woodshed-views/src/stage/settings.rs`
- `crates/woodshed-views/src/theme.rs`
- `crates/woodshed-views/src/workspace.rs`

Owner evidence:

- `eidetic/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`: native workspace baseline/post 746 pass, zero fail, five ignore; identical emitted inventories, Stickleback docs and full local/actual hooks pass. Imported patches remain excluded.
- `gemot/design_docs/archive_docs/receipts/2026-11-04_source_splits_plan_receipts.md`: native workspace baseline/post 242 pass, zero fail, one ignore; identical emitted inventories, workspace docs and full local/actual hooks pass. Separate relay-worker custody commit `30be124` is preserved as an ancestor, changes none of the selected sources or Cargo inputs, and gains no new operator/relay qualification here.
- `woodshed/design_docs/archive_docs/receipts/2026-11-03_source_splits_plan_receipts.md`: full root workspace baseline/post 1705 pass, one fail, eight ignore; matching emitted inventories and the same overview navigation/process-count assertion, actual 2 versus expected 3. That source is unselected; the assertion stays unchanged and the owning Stage plan carries the residual. Native/view docs and full local/actual hooks pass. Independent ports remain unselected.

Move audits match 1,100 old/new function/constant bodies across the three
owners, all signatures and type fields after scope/visibility normalization.
Woodshed has six Rustfmt-only brace/semicolon differences; the other two have
none. Literal/fixture bytes, generic bounds, public facades and cfg variants
remain matched. Session worker test names stay reachable through narrow
same-module includes, preserving their existing `--exact` launch strings.
No test assertion is weakened and no ignore is added. Existing warning-level
diagnostics remain; docs/compiler passes do not claim warning-free graphs.

Each source commit contains only its recorded owned paths. H8 changes no
Cargo manifest/lock, CI, tool release, hook configuration, CLAUDE, doctrine or
product/platform qualification. Existing Mere device-pairing and stack-
reduction WIP remains unstaged. Responsibility/fixture groups stay with their
owners; headed, physical audio/MIDI, real-network, deployed and other-platform
acceptance is not renewed.

Retained cap: 100 MB across H8. Compact summaries and owning receipt history
fit below 1 MB; transient logs/helpers stay below 1 MB before cleanup. Builds
reuse approved repository targets with two jobs and one own Cargo lane. No
isolated target, Cargo home, branch or worktree was created. Aggregate
metadata and scoped configured hooks are checked on the three changed docs;
actual commit repeats ceiling, Responsa and always-run dependency bans.
