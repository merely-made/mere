# House Style Plan

**Date:** 2026-10-30
**Status (2026-11-03):** in progress. Fifteen rulings recorded (D1 to D15).
Eidetic's E1-E4 and H1, H2 and H4 landed. H3 is qualified in 23 of 25
owned repositories; Knot and Woodshed await the receiving-graph scope
answer below. The adoption ledger and remaining failures are in the receipt.
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

## Findings

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
- 2026-11-03: receiving scope is an open question, not a ruling: whether
  this rollout includes necessary first-party graph repairs, or records
  inventory/lints while deferring blocked hooks. Knot's targeted Dramatis
  dry-run also moves Mere, Genet and Murm cohorts. Woodshed's Windows HAL30
  path has gpu-allocator 0.28 on windows 0.57 while HAL uses windows 0.62.2.
  Neither lockfile has been changed by this rollout. Do not treat an
  unreturned card as approval for those repairs.
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
