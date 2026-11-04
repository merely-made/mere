# House Style Plan

**Date:** 2026-10-30
**Status (2026-11-04):** in progress. H1-H9 complete; H10 qualifies fifty additional source splits, awaiting publication. Seventeen rulings recorded (D1 to D17).
H1-H4 and Eidetic's E1-E4 are complete. All 25 owned repositories have
published local-check adoption, reviewed inventories and passing positive
and deliberate-violation gates. Qualification uses the native package and
feature scopes recorded in the receipt; broader product/platform acceptance
remains with its owning plans.
**Scope:** stack-wide house style for owned repositories: the source-size ceiling, comment
economy, reuse and consolidation obligations, and the shared check command that enforces the
measurable parts locally (D11). Doctrine lives in [doctrine.md](../../doctrine.md); this
plan carries the rollout.

## Origin

A drafted rule set (2026-10-30) proposed a 600-line ceiling, a comment budget, reuse and
consolidation obligations, ownership and failure principles, and five automated checks. The
review against recorded problems found it covered app-local copies, irreconcilable runs and
self-reported compliance, but missed crate inflation and stub homes (`mere-athanor` a 35-line
stub while its pass lived in `pandect`, 2026-09-23), missed the anti-hardcode and
design-for-isolation rules (2026-09-04), and its "concrete code until repetition earns an
abstraction" contradicted the known-futures ruling (2026-09-23).

## Decisions

- **D1. Abstraction timing (Mark, 2026-10-30):** "Named futures only", over keeping the
  drafted repetition rule or dropping it. A seam is reserved for a consumer that can be named
  today, never for a speculative one. Refines stack.1.

  Links: amends stack.1 · summary Reserve a seam for a consumer named today; never abstract for a speculative one.

- **D2. Crate inflation (Mark, 2026-10-30):** "Fold into Stack 1", over a separate statement
  or leaving it in agent memory. A component is a module, not a crate; a reserved home
  receives its capability. Refines stack.1.

  Links: amends stack.1 · summary A component is a module, not a crate; a reserved home receives its capability.

- **D3. First checks (Mark, 2026-10-30):** "Line ceiling + Responsa", over adding dependency
  boundaries now or building all five drafted checks. The shared command generalises
  isocosm's `scripts/line-ceiling.ps1` (ruling isocosm:592) and runs `responsa check`; the
  comment linter and component-access checks are not built.

  Links: draws on isocosm:592 · summary The shared check starts as the 600-line ceiling plus `responsa check`, locally and in CI.

- **D4. Where the principles live (Mark, 2026-10-30):** "Refine existing statements", over
  one new statement per principle or a separate house-style document. Adopted principles fold
  into stack.1, stack.3, stack.5 and stack.6 within the area budget (7, six used); a new
  statement only where nothing existing fits.

  Links: governs stack.1, stack.3, stack.5, stack.6 · summary Adopted principles refine existing stack statements within budget; new ones only where nothing fits.

- **D5. Patron style (Mark, 2026-10-30):** "rust-analyzer + TigerStyle parts", over
  rust-analyzer alone or no patron. rust-analyzer's `docs/book/src/contributing/style.md` is
  the base, cited by reference; from TigerStyle only "put a limit on everything" and
  units-last naming (`latency_ms_max`). Each divergence from either is recorded as a decision.

  Links: summary House style takes rust-analyzer's guide as its base and TigerStyle's bounds and units-last naming; divergences are recorded.

- **D6. Shared lint set (Mark, 2026-10-30):** "Contract + too_many_lines", over contract
  lints only or deferring lints. Owned Rust repositories share a `[workspace.lints]` set:
  `missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`,
  `undocumented_unsafe_blocks` and `too_many_lines` (advisory, default 100) as warnings.
  Boundary lints are added per repository as consolidations land. Extends D3.

  Links: amends house-style D3 · summary Owned Rust repositories share the API-contract lints and an advisory function-length lint.

- **D7. Existing systems first (Mark, 2026-10-30):** "Happy to use existing development
  systems that satisfy the requirements i've outlined, if possible". Each check runs on an
  existing tool where one meets the requirement; custom code covers only the remaining gap.
  Refines D3, which named generalising isocosm's script.

  Links: amends house-style D3 · summary Checks run on existing tools where one meets the requirement; custom code only for the gap.

- **D8. Hook runner (Mark, 2026-10-30):** "prek", over pre-commit, lefthook or plain git
  hooks with a CI script. prek runs pre-commit's config format as one Rust binary with no
  Python, locally and in CI; a repository pins shared hook repositories by rev.

  Links: draws on house-style D7 · summary prek runs every repository's checks, locally and in CI, from pre-commit's config format.

- **D9. Line-ceiling home (Mark, 2026-10-30):** "eidetic, as a Rust binary", over moving the
  PowerShell script or a `responsa` subcommand. A separate binary beside responsa, exposed as
  a hook from eidetic; repositories pin eidetic's rev.

  Links: draws on house-style D7 · summary The line ceiling is a separate Rust binary in eidetic, exposed as a pinned hook.

- **D10. The ceiling is stack doctrine (Mark, 2026-10-30):** "commit and add the 600 line
  ceiling to the doctrine", over keeping it a check-only rule citing isocosm:592. stack.7
  takes the area's last budget slot (7 of 7); isocosm's Method 3 says the same for that
  repository.

  Links: draws on isocosm:592 · summary The 600-line ceiling becomes stack.7, the area's last budget slot.

- **D11. Local only (Mark, 2026-10-30):** "Local only", over one prek-only workflow or CI as
  recorded. Put to Mark on finding that CI was removed from eidetic and isocosm on 2026-10-28
  ("Gates live in local scripts and dated receipts"), which contradicted D3, D8 and stack.7's
  "at commit and in CI". The checks run in the prek pre-commit hook and, over the whole tree,
  as `prek run --all-files` in each repository's local gate. Recorded with the rollout in
  eidetic's line-ceiling plan as D6.

  Links: amends stack.7, house-style D3, house-style D8 · summary The shared checks run at commit and in each repository's full local gate; no CI.

- **D12. Extraction timing (Mark, 2026-10-31):** "second host is planned", over waiting for a
  second host to show up: "shows up doesn't happen, you know, until it's planned. And shows up
  also doesn't happen until the extraction happens. But then the extraction is gated on a
  second consumer, right? So it's just circular." Put to Mark on finding the Graphshell
  stack lift's L4b, L6, L7 and P6 and the family promotion's source-home follow-up waiting
  on hosts that could only adopt after the move. Then "Refine stack.1"; planned is "Host's
  own plan" (a dated plan in the host's repository names the adoption with a
  done-condition); done is "Moved + first host" (the second host's adoption is a receipt in
  that host's plan); scope "Also isocosm Stack 7" (isocosm case 1045). The same change
  rewrites the gates in the Graphshell stack lift plan (§3, P6), the composition thesis's
  anti-shell test, the runtime composition ledger's rule 2 and the conatus plan's contract
  rule.

  Links: amends stack.1 · summary Extraction proceeds once a second host is planned in its own repository's plan; it is done when the code has moved and the first host builds on it.

- **D13. One source-size statement (Mark, 2026-11-02):** "Retire it and cite
  stack.7 (Recommended)", over keeping isocosm's Method 3 as a local
  restatement or deferring. Outcome: applies to stack.7; retire the duplicate
  Method 3 in isocosm and cite `mere:stack.7` there. Coordinate with the VTT
  session before changing that repository.

  Links: draws on stack.7 · governs isocosm:1160 · summary Isocosm retires its duplicate Method 3 source-size statement and cites Mere stack.7.

- **D14. Reuse order (Mark, 2026-11-02):** "Add that reuse order
  (Recommended)", over keeping stack.1's wording or deferring. Outcome:
  refines stack.1 with use, configure, compose, extend, then implement the
  gap, while retaining its test that a dependency must earn its cost.

  Links: amends stack.1 · summary Try use, configure, compose and extend before implementing the gap; a dependency still earns its cost.

- **D15. Consolidate before adding principles (Mark, 2026-11-02):**
  "Consolidate within the seven-statement budget (Recommended)", over
  refining existing statements only or deferring. Outcome: refines the Stack
  area within its seven-statement budget for bounded background work and
  distinguishable failure; ownership refines stack.6 and capability routing
  refines stack.3.

  Links: amends stack.3, stack.6 · governs H4 · summary Consolidate related Stack statements within seven slots for bounded work and distinguishable failure; refine ownership and capability routing in place.

- **D16. Preserve reference coefficients (Mark, 2026-11-03):** "Keep coefficients; allow only on the table module (Recommended)".
  Outcome: applies to stack.7 and H1's lint adoption. Turquet preserves
  the VSOP87D decimals and confines the coefficient-table
  `clippy::approx_constant` allowance to that module. This records the
  returned answer to the earlier card; it creates no stack-wide allowance.

  Links: draws on stack.7 · governs H1 · summary Preserve the VSOP87D coefficients with a module-only approximate-constant lint allowance.

- **D17. Repair receiving graphs (Mark, 2026-11-03):** "Repair the necessary receiving cohorts (Recommended)".
  Outcome: applies to stack.7 and H3. Include the receiving-graph repairs
  necessary for the new local lint gates, including Knot's related Mere,
  Genet, Murm and Dramatis source cohorts. Preserve unrelated lock selections
  and qualify the resulting receiving graph before publishing adoption.

  Links: draws on stack.7 · governs H3 · summary Repair the necessary receiving cohorts to qualify the local lint gates.

## Open forks

Each comes back as a question before anything is written.

1. Resolved by D10 and D13: source size lives in stack.7 and isocosm retires
   Method 3. D15 consolidates the Stack area before adding a statement.
2. Resolved by D15: extension rungs fold from stack.5 into stack.3;
   capability routing refines stack.3, ownership refines stack.6, and new
   stack.8 carries bounded background work and distinguishable outcomes.
   The retired stack.5 key is not reused; the area stays at seven statements.
3. Resolved by D14: stack.1 names the reuse order and retains dependency cost.
4. Resolved by D8 and D9.

## Phases

- **H1. Shared check command.** prek (D8) runs the line ceiling (eidetic, D9),
  `responsa check`, Clippy with the D6 set and, where configured, `cargo deny check bans`,
  identically in the pre-commit hook and the repository's local gate (D11). The line
  ceiling keeps per-repository scope and an exceptions file whose entries carry a reason.
  Done when isocosm runs it in place of `.githooks` and its own script, a planted 601-line
  file fails at commit and in the full local run, and the original script is deleted.
- **H2. Inventory.** Record every owned repository's existing oversize files as listed
  exceptions. Done when each owned repository passes with an explicit inventory.
- **H3. Adoption.** Pre-commit hook and local-gate step in every owned repository. Done when
  every owned repository runs the checks at commit and in its local gate, and a deliberate
  violation fails both.
- **H4. Doctrine pass.** Put forks 1 to 3 to Mark and fold the answers per D4. Done when the
  doctrine is within budget and `responsa check` is clean.

- **H5. Fifty additional source splits.** Mark requested a 50-file pass
  after the four Armillary and four Smolweb exceptions already retired.
  Those eight earlier files do not count toward this target. Choose existing
  oversize files with real responsibility or fixture-group boundaries;
  preserve public APIs, semantics, feature/cfg isolation and concurrent work.
  Count a file only when it and its replacement modules fit the ceiling,
  its exception is retired, the appropriate tests/full local gate pass,
  and the implementation is committed and pushed. Keep immutable historical
  receipts and fixture captures intact. Done when 50 distinct original paths
  are recorded as qualified in the appended H5 receipt. Retained cap stays
  100 MB across repositories, with compact evidence only.

- **H6. Second fifty-file source pass.** Mark's "Another batch?" continues
  the same qualification rule for fifty additional original paths. H5's
  fifty and the earlier eight are excluded. Selected: Woodshed 35 and
  Eidetic 15, with local plans, scoped tests, private/public/feature boundary
  preservation, exact exception retirement and pushed commits. Done when
  fifty new distinct originals and their replacement modules fit the ceiling
  and their owner receipts and full local gates qualify them. Total retained
  cap: 100 MB across H6; compact text only.

- **H7. Third fifty-file source pass.** Mark's "Next batch!" continues the
  qualification rule for fifty new originals: Knot 28, Mer3ly 12,
  Cleromancy 9 and Wavicle 1. Earlier qualified paths do not count again.
  Owner plans preserve APIs, data, feature isolation and concurrent work.
  Done when every original/replacement fits, fifty exceptions retire,
  baseline/post inventories agree, moved-surface feature tests and full
  local/commit gates pass, existing full-suite failures remain distinguished,
  and source and aggregate receipts are pushed. Total retained cap: 100 MB across H7;
  compact text only, stable approved Cargo targets, two jobs.

- **H8. Fourth fifty-file source pass.** Mark's next batch selects Eidetic
  five, Gemot thirteen and Woodshed thirty-two from clean main. Previously
  qualified paths are excluded. Done when all originals/replacements fit,
  fifty exact exceptions retire, baseline/post inventories agree, feature/docs
  and full local/actual gates pass, and source plus aggregate receipts are
  pushed. Existing failures and broader acceptance stay distinguished.
  Retained cap: 100 MB across H8, compact summaries, stable targets, two jobs.

- **H9. Fifth fifty-file source pass.** Mere graph (25) and canvas (25)
  originals qualify only after each original and child fits 600 lines, exact
  exceptions retire, baseline/post and relevant feature/docs gates agree,
  and configured local/actual hooks pass before publication. Preserve
  literal/fixture bytes, public facades, type fields, graph authority and
  canvas/runtime ownership. Retained cap: 100 MB across H9; compact summaries,
  stable target, two jobs and one own Cargo lane. Existing failures and
  physical/headed/platform qualification remain separately recorded.

- **H10. Sixth fifty-file source pass.** Cambium widgets, host/input, shared
  views, document layout and three UI support originals. Done when all
  originals/children fit 600 physical lines, exact exceptions retire,
  baseline/post and relevant feature/docs inventories agree, and configured
  full/actual hooks pass before publication. Keep field records, public
  facades, callback ownership, model/runtime identities, literals, fixture
  bytes and feature boundaries. Shared retained cap: 100 MB, compact
  summaries, stable Mere target, two jobs, one own Cargo lane. Frozen goldens
  and giant function factoring stay outside this pass.

## Findings

- 2026-11-04: H7 qualifies fifty additional originals across Knot 28, Mer3ly 12, Cleromancy 9 and Wavicle 1. Original/replacement files fit, fifty exact exceptions retire, and all four source commits are pushed. Baseline/post inventories agree; scoped feature/docs and local/actual commit hooks pass. Knot retains fourteen existing product/Windows full-suite failures and Cleromancy two product failures, carried into owner plans. The canonical LF fixture repair changes no golden Git bytes.

- 2026-11-03: H6 qualifies fifty more originals: Eidetic fifteen and Woodshed thirty-five. All original/replacement files fit, fifty exact exceptions retire, and code is pushed. Baseline/post tests match (746 and 967); optional feature checks, docs and full gates pass. The existing MIDI-disabled diagnostics target gains its missing feature guard.

- 2026-11-03: H5 qualifies fifty additional paths across Smolweb (9), Dramatis (20), Netrender (12), Turquet (6) and Emblem (3). Every replacement module fits the ceiling, fifty exact exceptions are retired, and all implementations are pushed. The Windows adoption flush and native fixture exhaustiveness repairs are recorded separately from source moves; Netrender's existing full-suite compose pixel failure remains open at its unchanged threshold.

- 2026-11-03: H2 records 900 individual oversize exceptions across 25
  owned repositories, each with a technical reason. Imported patches,
  vendored corpora and proven generated outputs have explicit exclusions.
  This supersedes the reconnaissance counts below. All 25 positive ceiling
  runs pass; a staged 601-line source file fails both the full local run and
  an actual commit, without advancing HEAD, in every repository.
- 2026-11-03: Smolweb, Wavicle and Mer3ly had existing CI Clippy commands
  that promoted warnings to errors. D11 moves those commands to their local
  hooks. Smolweb retains all-feature coverage; Mer3ly retains its root,
  repo-graph and radio-mirror packages. Other workflow gates remain.
- 2026-11-03: Mere's custody bans named the old `djinn` Cargo package. Its
  allowed host is now `mere-resident`; correcting that wrapper name makes
  the existing full-feature boundary pass without adding another host.
- 2026-11-03: Genet's duplicate policy has drifted behind its resolved graph.
  The refresh keeps `multiple-versions = "deny"`, exempts exact inspected
  versions with their immediate callers, and retains the independent local
  DOM patch and registry JavaScript-engine regex sources. It does not
  update the dependency graph or remove the `num` ban.
- 2026-11-03: D17's receiving repairs are qualified and published.
  Knot aligns the required Mere, Genet, Murm and Dramatis cohorts, including
  a stale AccessKit source record. Every registry package record remains
  unchanged, and actual resolved package records and dependency edges match
  the lock. Woodshed changes only gpu-allocator's Windows edge to HAL30's
  0.62.2 line; BLE keeps its required 0.57.0 line. A deliberately empty
  reversed-scope test has a function-scoped lint allowance, preserving its
  negative assertion. Neither repair claims new headed or device acceptance.
- 2026-11-02: H1-H3 receiving work has begun in the primary checkouts.
  The shared five Clippy lints are warnings in workspace tables and opted
  into by owned members; existing package-specific lint tables retain their
  local policy and receive the same five warnings. Standalone packages use
  their package lint table. Imported patches, generated sources and frozen
  fixture manifests keep their own inputs and are not rewritten.
- Patron references (D5): [rust-analyzer's style guide](https://rust-analyzer.github.io/book/contributing/style.html)
  and [TigerStyle](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md),
  from the latter only bounds and units-last naming. [Cargo's workspace lint
  inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table)
  requires package opt-in; adding only a root table would leave members unchecked.
- 2026-11-02: the native Clippy hook follows the repository's Cargo package
  selection and default features. Graft selects `grafting` and
  `grafting-frame`, Scry selects `scrying`, and Murm selects its portable
  root libraries to respect their existing platform and firmware boundaries.
  Other-platform demos, firmware and optional feature gates stay separately
  owned. Existing bans are configured in Dramatis, Genet and Mere; Murm's
  deny configuration is licenses-only, so it gets no invented bans policy.
- 2026-11-03: the first Turquet Clippy gate reports 55 `approx_constant`
  errors in the VSOP87D coefficient tables. A module-scoped allow preserves
  every coefficient decimal. Netrender's deliberately reversed selection
  range receives a test-scoped `reversed_empty_ranges` allow. These preserve
  intentional inputs, outside the shared five-warning set; they do not
  change a ruling. The initially requested Turquet card was unnecessary.
- 2026-11-03: all 393 owned package manifests in the rollout's active
  manifest scope resolve the five warnings, locally or through workspace
  inheritance. Generated/imported sources and frozen fixture manifests are
  outside that manifest rewrite; their source-size scope remains declared.
- 2026-11-03: prek discovers nested hook projects independently of its root
  filters. Genet's imported WPT dependencies therefore tried to install
  their historical hooks. `.prekignore` now excludes the declared imported
  and generated directories from project discovery. Genet's line hook also
  filters the same declared paths before file classification; the runner's
  pattern and the checker's path scope must change together.
- 2026-11-02: H2 reconnaissance over the current tool's default extensions
  finds 949 oversize candidates after skipping candidate imported/generated
  paths (`tests/wpt/`, `support/patches/`, `vendor/`, `third_party/`,
  `generated/`); this is not the adopted inventory. Mere has 389, Genet 188,
  Woodshed 105, Knot 39 and ESP 34. Candidate exclusions still need review:
  Genet's remaining count includes imported Blink performance fixtures;
  generated bindings and owned modifications must be classified by path.
  H2 is not done until every retained exception has its technical reason
  and the binary passes in each repository.
- 2026-11-02: the shared hook stanza and both adopters now use `always_run`
  so deletion-only commits still reject stale exceptions. Eidetic's
  line-ceiling receipt records the empty-effective-file-list control.
- 2026-10-30: isocosm's checker is 47 lines (`scripts/line-ceiling.ps1`), with `-Staged`
  for the pre-commit hook and `line-ceiling-exceptions.txt` requiring a reason per entry.
- 2026-10-30: [doctrine.md](../../doctrine.md) stack.5 already carries the WIT and native
  boundary rule the draft cited; stack.1 carried the crate-earning clause D2 extends.
- 2026-10-30: tracked source files over 600 lines (`.rs .ps1 .py .js .ts .wgsl .sh`, paths
  under `vendor/`, `third_party/`, `generated/` skipped, no other exclusions): 1,607 across
  20 repositories under `repos/`. genet 681, mere 407, turnstone 112, woodshed 104,
  wgpu-graft 88, knot-editor 39, wgpu-scry 22, eidetic 20, netrender 19, murm 19, mer3ly 18,
  turquet 15, smolweb 13, gemot 13, wgpu-weld 12, esp 9, cleromancy 9, emblem 3, dramatis 3,
  wavicle 1; isocosm 0. genet's count includes imported servo-stack sources, which H2 must
  classify as upstream before they count as debt.
- 2026-10-30, existing systems (D7): no repository under `repos/` uses a hook framework;
  only isocosm installs hooks (`core.hooksPath=.githooks`), so mere's DOC_POLICY line "the
  pre-commit hook runs it" holds nowhere else. `deny.toml` exists in dramatis, genet, mere
  and murm, and `cargo-deny` is installed; `[workspace.lints]` exists in armillary, eidetic,
  esp, gemot, knot-editor, mere and smolweb. No established multi-language file-length
  linter was found (Clippy, rustfmt and Ruff have none; pymaxlines is Python only).
  cargo-deny's `[bans]` `wrappers` field restricts which crates may depend on a crate,
  which covers layer-boundary checks.

## Progress

- 2026-10-30: D1 to D4 recorded; stack.1 refined.
- 2026-10-30: D5 to D10 recorded; stack.7 added.
- 2026-11-02: eidetic's E4 and Linux E2 landed; the tool rollout's E-phases
  are complete. See eidetic's line-ceiling plan and its compact receipt.
- 2026-11-02: D13-D15 recorded in answer order. The doctrine pass consolidates
  extension rungs into stack.3, adds stack.8 without reusing stack.5's key,
  and refines stack.1, stack.3 and stack.6. Isocosm's case 1160 carries the
  Method 3 retirement. **H4 landed**, with both repositories' manifests and
  doctrine checks passing after the retirement. Receipts live at
  `archive_docs/receipts/2026-10-30_house_style_plan_receipts.md` (100 MB cap;
  compact text only). H2-H3 and H1's remaining lint/deny wiring stay open.
- 2026-11-03: H2 is qualified across the 25-repository scope. Local
  controls pass in each repository. H1/H3 publication is continuing with
  platform/default-feature scope and any blocked receiving graphs recorded
  in the [rollout receipt](../../archive_docs/receipts/2026-10-30_house_style_plan_receipts.md).
- 2026-11-03: **H1 landed** with the five warning lints, applicable
  decision checks and existing bans in the same local/commit runner.
  Isocosm now checks its regenerated manifest as well as regenerating it.
  **H2 landed** with the reviewed 900-file inventory. H3 is qualified in
  23 repositories; Knot and Woodshed remain receiving blockers. Their new
  hook shims are uninstalled pending the scope answer, with reviewable
  configuration and lint changes preserved unstaged. Their lockfiles are
  unchanged. This does not mark H3 complete.
- 2026-11-03: returned cards recorded in answer order as D16 (Turquet's
  reference table) and D17 (necessary receiving repairs). H3 resumes in
  Knot and Woodshed under D17; their adoption is not yet qualified.
- 2026-11-03: **H3 landed.** Knot and Woodshed pass the final full local
  gates and actual commit hooks after the D17 receiving repairs. All 25
  adoption commits are pushed to main; the 25 earlier full/commit rejection
  controls remain valid for their unchanged ceiling hooks. H1-H4 and the
  tool plan's E1-E4 are complete. The receipt appends the final receiving
  hashes, lock audits, scope and cleanup; no remaining rollout phase is open.
- 2026-11-03: H5 started at **0/50** additional qualified paths. Mark's
  new scope is a 50-file pass; the first eight source splits remain previous
  work. The selected approach batches coherent changes per clean repository,
  preserving active WIP in Mere, Eidetic, Murm and Gemot. Each source owner
  keeps its local plan/receipt; the H5 receipt aggregates the published paths.

- 2026-11-03: H5 **12/50** published: Smolweb nine (`91ae0a67`) and Emblem three (`9bc1a1e`). Both reach zero exceptions, with matching baseline/post tests and full gates. Remaining candidates are Dramatis twenty, Netrender twelve and Turquet six; Cleromancy and Mer3ly now have concurrent manifest work and are left to their owners.

- 2026-11-03: H5 **44/50** published. Dramatis twenty (`e26b434`) and Netrender twelve (`2073c5050`) pass their recorded tests and full gates. Dramatis also repairs the baseline Windows adoption flush; Netrender retains its existing full-suite compose pixel failure at the original threshold. Emblem retains its extracted-file source notices at `0975e27`. Turquet six awaits its actual commit gate.

- 2026-11-03: H5 **50/50 landed**. Turquet six (`9819053`) completes the target. All five source checkouts are clean and match pushed main; the aggregate receipt lists every original path and owning receipt. Source notices, fixtures and concurrent WIP are preserved. Compact receipts stay below 100 KB; no isolated build area or worktree is retained.

- 2026-11-03: H6 starts at **0/50** additional qualified paths: Woodshed thirty-five and Eidetic fifteen, selected from clean main. The preceding fifty remain H5, and prior splits are excluded.

- 2026-11-03: H6 **15/50 published**. Eidetic fifteen (`5b805a1`): original/child files fit, exceptions retire, 746 baseline/post cases, optional recall-example type checks, docs and full local/commit gates pass. Five Eidetic exceptions remain.

- 2026-11-03: H6 **27/50 published**. Woodshed S1 twelve (`297b162`, integrated with concurrent Redshank work in `d2f990c`) joins Eidetic fifteen. All 967 software cases pass post-refactor with serial execution, literal bytes are audited, and full local/commit hooks pass.

- 2026-11-03: H6 **50/50 landed**. Woodshed S2/S3 twenty-three (`cf2b091`) completes the target with matching 967-case software tests, feature/docs checks and final full/commit gates. The aggregate receipt lists fifty unique additional paths and published commits. Five Eidetic and 69 Woodshed exceptions remain outside this pass. Concurrent Redshank and Mere plan work is preserved; compact receipts fit the 100 MB cap, with no isolated build area or worktree retained.

- 2026-11-03: H7 starts at **0/50**: Knot 28, Mer3ly 12, Cleromancy 9 and Wavicle 1, selected from clean main. Mere's concurrent stack-reduction plan work is preserved.

- 2026-11-03: H7 **1/50 published**: Wavicle's conformance source (`4c384df2`) fits with its encode child, reaches zero exceptions, and matches 21 default/16 reader-only cases with encode/docs and full local/commit gates.

- 2026-11-03: H7 **13/50 published**: Mer3ly twelve (`706014ff`) joins Wavicle one. Native site/standalone graph tests match baseline, Wasm compile, docs and all local/actual hooks pass; nine non-Rust exceptions remain.

- 2026-11-04: H7 **22/50 published**. Cleromancy nine (`d9de7f2d`) joins Mer3ly twelve and Wavicle one. All nine original/child sources fit with zero exceptions; 147 default passes, the same two existing failures and one ignore, matching baseline inventory except the LF fixture repair; analytic eighteen, narrow-feature/docs and full/actual hooks pass. Product defects carry forward in the divination plan O7. Knot 28 is prepared and in post-refactor qualification.

- 2026-11-04: H7 **50/50 landed**. Knot twenty-eight (`7e66a1df`) completes the source target, with 772 passes, the same fourteen failures and eight ignores, matching the complete baseline inventory. Optional CLI, offline/editor and retention-free/desktop checks, docs, explicit workspace Clippy and full/actual hooks pass. The signed-history fixture now follows its actual optional component without disabling other scenarios. Remaining exceptions in these owners: Knot eleven and Mer3ly nine; Cleromancy and Wavicle zero. All published original/new Rust files fit. The aggregate receipt lists all fifty paths and immutable source commits, preserving concurrent Mere work and owner evidence policies.

- 2026-11-04: H8 starts at **0/50**: Eidetic five, Gemot thirteen and Woodshed thirty-two root-workspace originals. Existing Mere plan WIP and excluded ports remain with their owners.

- 2026-11-04: H8 **5/50 published**. Eidetic's five remaining Stickleback originals (`171f7892`) reach zero owned exceptions, with matching 746-case native workspace inventories, public docs, structural review and full local/actual hooks. Gemot baseline passes; Woodshed baseline is running.

- 2026-11-04: H8 **18/50 published**: Gemot thirteen (`12ee625e`)
  joins Eidetic five. Zero Gemot exceptions remain; 242 native workspace
  passes and one ignore match baseline, and docs/full local/actual hooks pass.
  Concurrent relay custody ancestor `30be124` is preserved separately.

- 2026-11-04: H8 **50/50 landed**: Woodshed thirty-two (`a113b659`)
  completes the pass. Original/new Rust files fit, exact exceptions retire,
  and 1705 passes, one existing overview failure and eight ignores match
  baseline. The overview assertion carries forward in its Stage plan;
  docs and full local/actual hooks pass. Published source checkouts are
  clean. The aggregate records all fifty new originals, 122 children and
  immutable source commits. Eidetic/Gemot have zero exceptions; Woodshed
  retains 37. Existing Mere plan WIP, source fixtures and prior evidence
  remain intact; physical, headed and platform acceptance stays separate.

- 2026-11-04: H9 starts **0/50**: mere-kernel 19, mere-rdf 6, pictograph
  25, selected from unchanged sources. Existing device-pairing and stack-
  reduction plan work is preserved; baselines precede source edits.

- 2026-11-04: selection review replaces apply.rs and capture.rs with two
  canvas physics fixture files before source edits. Their 1,246-line mutation
  function and 753-line capture test need deliberate factoring in a later
  pass. H9 now covers graph 25 and canvas 25 through whole method and fixture
  groups; baseline package/feature scope is unchanged.

- The Canvas field record stays in its existing home; its selected slot is
  the PhysicsBoard implementation/fixtures, keeping field visibility intact.

- 2026-11-04: H9 untouched query baseline is blocked by spareval 0.2.9
  selecting MD5/digest 0.11 against SHA/digest 0.10. Under the receiving-
  cohort repair ruling, MD5 is locked to compatible 0.10.6. Cargo also moved
  an unrelated macro dependency; its exact original record was restored.
  Every retained package record is unchanged before repeating the baseline;
  no hash algorithm is replaced.

- 2026-11-04: H9 fifty originals and all children fit. Whole method and
  coherent fixture moves preserve bodies, signatures, type fields, included
  bytes and cfg boundaries. Extracted public test modules retain their
  original visibility after the path attribute. Feature-enabled native post
  results match the compatible baseline; narrower and final gates follow.

- 2026-11-04: H9 **50/50 landed**: nineteen mere-kernel, six mere-rdf
  and twenty-five Pictograph originals, plus all 98 children, fit 600 lines.
  Fifty exact exceptions retire. Compatible baseline/final feature-enabled
  tests and both narrow configurations agree; package Clippy/docs and all
  four configured full-repository hooks pass. A narrow MD5 lock repair
  makes the existing query feature buildable. Concurrent receipt/archive
  edits remain with their owner; this pass renews software qualification
  for the named packages, with physical/headed/platform gates separate.

- 2026-11-04: H10 starts **0/50**, selected from unchanged sources across
  eleven UI/library packages. Existing device-pairing and stack-reduction
  plan WIP is preserved; baselines precede selected Rust edits.

- 2026-11-04: H10 preparation fits all fifty originals and 129 children.
  Whole public data groups retain their exports, field visibility and
  serialization attributes; private host/context state remains in its
  original records. Bodies, impl bounds, signatures, types and included
  bytes are matched before final compiler qualification.

- 2026-11-04: H10 **50/50 qualified** across eleven UI/library packages.
  Fifty originals and 129 children fit (maximum 593 lines); fifty exact
  exceptions retire. Native inventories match 974 passes, zero failures and
  six ignores; both narrow inventories, embedding/all-feature checks,
  selected-package Clippy/docs, Responsa and all four full hooks pass.
  Source publication and its immutable receipt are the remaining checkpoint.
