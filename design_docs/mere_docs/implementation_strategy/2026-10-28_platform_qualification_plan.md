# Platform qualification: native, assistive-technology, browser and hardware gates

**Status:** 2026-10-28: cut from `2026-10-23_contextual_navigation_plan.md`, `2026-10-20_hest_command_architecture_plan.md` (in `cambium_docs`) and `archive_docs/2026-10-28/2026-10-18_source_edit_proposal_plan.md` (all complete; residue only), with the Windows/Linux, assistive-technology, stock-client and VTT items of `2026-07-05_theme_modes_plan.md` and `2026-08-03_graph_view_curation_and_interaction_plan.md`, the human AT gate of `2026-10-13_browser_presentation_tenants_plan.md`, the Windows/Linux and headed items of `2026-08-09_burn_0_22_migration_plan.md`, and platform items of `2026-07-03_archived_plan_tails_plan.md`, all archived in `archive_docs/2026-10-28/`. Each source qualified its work on one platform, mostly macOS (contextual navigation on Windows, Hest's package graphs on Linux), by software tests or scripted runs. This plan owns the gates none of them claims; none has run.

**For:** the native presentation, lifecycle, assistive-technology, browser and hardware acceptance that other plans named "a separate gate" and left. Opened by the repeated limit in the sources: a software test is not a headed run, a macOS receipt is not a Windows or Linux one, and automated accessibility checks are not a person using assistive technology.
Not here: custody's receiving runs (Linux, signed-macOS and the second machine), which `../../dramatis_docs/implementation_strategy/2026-10-28_custody_plan.md` keeps; no other cut source leaves a signed-macOS gate. Multi-window pacing on X11, Wayland and macOS (stack seams F18 and F20) and the Safari and WKWebView runs of redb over OPFS, which are `2026-10-28_workspace_hygiene_plan.md` (X1 and X2). The Burn cohort's other residue (ESP package receipt, carried vendor patches, the Eponym probe, remote endpoints on two physical machines), which is `2026-10-28_distillery_models_plan.md`. Theme and typography adapters, font bounds and the rhai calculator, which are `2026-10-28_appearance_plan.md`. Folds, relation cells and share records, which are `2026-10-28_graph_view_curation_plan.md`. Viewer image loading, shadow masks and the unticked task boxes of the browser tenants, which are `2026-10-28_cambium_presentation_plan.md`. Retention and pruning of visit lineage (host-owned), which is `2026-10-28_graph_semantics_plan.md`. Reading chrome, which stays with the Turnstone GUI lane. Hest backend package publication, which has no target plan. Knot's own residue from the source-edit plan (strict all-test Clippy failures in unchanged files, the exact-revision supplier pin and when to return to main, multi-range applications refused until a batch undo boundary exists) and the Woodshed arrangement-draft candidate, which belong to the knot-editor repository. Document-audit baseline debt, which is a question in the hygiene plan.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- A software test, an automated ARIA-mirror or accessibility-projection check, or a fresh-process reopen is not a headed browser run, a native window-manager run, reviewed native GUI qualification or human assistive-technology acceptance (Hest 231-233, 255-256; tenants 252; source edit 84-86; contextual navigation 30-35).
- A receipt from one platform does not stand for another: macOS appearance, curation and Burn receipts say nothing of Windows or Linux, and Windows navigation receipts say nothing of macOS or Linux (theme 576-580, 667-668; Burn 5098-5099; contextual navigation 204-205).
- Font family availability does not certify glyph coverage, exact remote or frozen face identity, physical IME or OS accessibility (theme 1252-1256, 1400).
- A frozen result does not transfer to newer host code (Hest 264-265).

Sources are cited by short name with line numbers into the archived copies.

## Phases

### N1. Native presentation on the platforms no source qualified
Each source qualified native presentation on one platform. Open, by source:
- Contextual navigation: native macOS and Linux visual presentation of Woodshed's and Knot's chrome defaults (macOS keeps native traffic lights even for a custom content frame; Windows keeps the shared client captions and Snap), and full tile-layout persistence.
- Appearance adoption in Pelt, Graphshell, Knot, Turnstone, Woodshed, Redshank and Signalman: macOS receipts only, so Windows and Linux headed behavior and native OS decorations need their own evidence.
- Curation: headed acceptance of the production Canvas, the Swatch fixture and Knot's catalog passed on macOS only.
- Hest: native GUI and window-manager acceptance, a fresh native presentation, and a saved-choice fresh-process reopen.
- Source edits: reviewed native GUI qualification of the Knot and Isocosm adapters, with no native screenshot review claimed for Knot, and the broader compatibility claim.
- Burn 0.22.1: Windows and Linux builds, the complete headed product acceptance matrix, and process or driver VRAM telemetry, none of which is claimed.
- The scrying tile's X4 (macOS and Linux).
- Knot on Windows, noted and not ruled, predating the Burn repin: Windows checkouts get CRLF in `assets/oewn-notices.txt` and `tests/fixtures/wordnet.xml` through `.gitattributes`, which fails 7 `wordnet_import` tests (LF bytes pass), and Windows defaults to the title-bar chrome, which hides toolbar buttons Knot's headed scenarios press, so only a test can choose the plain toolbar.
- Done when: each item has a recorded native run on the platform it names, not inferred from the platform already qualified.
- Needs: Windows, Linux and macOS machines for the named hosts
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-23_contextual_navigation_plan.md:195-205,257-258`; `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:132-135,623-626,667-668`; `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:662-666,700-701`; `archive_docs/2026-10-28/2026-10-20_hest_command_architecture_plan.md:8-9,127-130,212-214,263-264`; `archive_docs/2026-10-28/2026-10-18_source_edit_proposal_plan.md:53-55,84-86,111-112,198-199`; `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:17,4995-5002,5098-5099,5217-5218`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:952`

### N2. Native lifecycle, startup and input cases
Cases the sources name as not exercised natively. Turnstone's dirty parent-quit Save, Discard and Cancel, and each OS focus transition. Knot's default 10-second startup stall guard and unattended presentation (the marked-fixture activation grace, capped at 60 seconds, is for synthetic runs only). Woodshed's recorded-loop missing-glyph gate, which needs actual fallback and paint proof, its fixed graph and fret geometry, and physical input. Woodshed's zoom-1.0 headed receipt. Contextual navigation's responsive-size matrix and physical audio quality. A visual check of the document-lane sheet arrows, deferred at closeout.
- Done when: each case has a native run that exercises it, with the glyph gate shown by actual fallback and paint.
- Needs: N1 for the platforms; the owning application's lane for each case
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:453-455,1209,1400-1402,1419-1424,1462-1463,1482-1486`; `archive_docs/2026-10-28/2026-10-23_contextual_navigation_plan.md:92-93`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:165,1239`

### A1. Human assistive-technology acceptance
A person using real assistive technology on a real platform, for flows the sources verified only through accessibility projections or automated mirrors. Hest's commands, whose human AT acceptance was part of neither publication nor the H4 receiving; the browser presentation tenants' ARIA mirror (picking, keyboard focus and actions, clipping); the appearance work's live screen-reader validation of custom calculator authoring, Turnstone activation, editable content and full graph workspaces, with physical AT, formal contrast, physical IME and formal OS accessibility; curation's keyboard gestures, live screen-reader interaction and recovery for folds; a VoiceOver check of the Graphshell window tree, where a new checklist would target `ports/graphshell` and adapter installation would be separated from OS traversal if VoiceOver cannot enter the tree; and a physical Narrator receipt for tear-out.
- Done when: each flow is completed by a person on the named screen reader or input method and recorded as a human run; automated checks do not count.
- Needs: N1 for the platforms; assistive-technology access
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_hest_command_architecture_plan.md:129-130,231-233,263-264`; `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:252,285`; `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:576-580,940-945,1252-1256`; `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:662-666,700-701`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:236-239,1221`

### B1. Browser and stock-client acceptance
Hest's browser H4 audit acceptance (Graphshell's completion report omits Mesquite's command audit records), a full-product browser package (it stalled at the unchanged 3 GiB cap), and a browser result on current host code (the limited headed Linux WebGL2 viewer pass was frozen at `c66620ca` and not transferred to `b4ae63d82`). The Lagrange stock client's visual load, restart and link-icon contrast for Tabard's exported theme artifact (Lagrange 1.21.1 ignores `yellow` and `magenta`, which the artifact reports as `StockVersionIgnored`).
- Done when: the audit records appear in the browser report on current host code, the package builds within its cap, and the stock client loads, restarts and shows the exported theme with legible link icons.
- Needs: the Graphshell completion report; a Lagrange install
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_hest_command_architecture_plan.md:257-265`; `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:8-9,716-720`

### T1. Whole-workspace MSRV
Fresh Linux qualification covers the published Hest, command-menu and Cambium package graphs only. Whole-workspace MSRV is not established.
- Done when: whole-workspace MSRV is established.
- Needs: the toolchain question in `2026-10-28_workspace_hygiene_plan.md`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_hest_command_architecture_plan.md:214,248-256`

### V1. The VTT campaign consumer of folds
Curation named Isocosm's VTT campaign as a consumer of folds; its Overmap has no fold controls, and Knot's catalog supplied the production proof instead without waiting for it.
- Done when: the VTT campaign consumer is qualified against the fold contract.
- Needs: `isocosm/design_docs/2026-10-28_vtt_overlay_plan.md`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:6-7,662-666,676-678`

## Open questions

- **Which host carries X4**: the scrying tile's home is gone with meerkat; the tails name Pelt (`ports/pelt/desktop/scrying_receipt.rs`) as the nearest live host. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:949-961`.

## Findings

- 2026-10-21: Rust 1.97.1 qualifies the published Hest 0.1.0 and 0.2.0, command-menu 0.1.1 and Cambium 0.5.0 package graphs on Linux with their shipped locks (324 tests in aggregate); this does not establish whole-workspace MSRV or native, browser or human AT acceptance (Hest 248-256).
- 2026-09-13: Lagrange 1.21.1's color loader ignores `yellow` and `magenta` although its help lists them (theme 716-720).

## Progress

- 2026-10-28: cut from the contextual navigation, Hest and source-edit plans and the platform items of the theme, curation, browser tenants and Burn plans and the archived plan tails; nothing new landed.
