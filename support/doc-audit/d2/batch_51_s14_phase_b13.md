# Batch 51 — S14 pass, phase B13 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md | current | no | 44 | 26 | 17 | 1 |
| **Totals** |  |  | **44** | **26** | **17** | **1** |

**Totals: 1 docs, 44 claims checked (26 holds, 17 stale, 1 unverifiable), 4 contradictions; 1 status lines wrong.**

Audit base: Mere `535bca11` (2026-10-05). Sibling repositories were read at
their HEADs that day, genet also at Mere's pin `bd3e8861`. `archive_docs/` is
excluded. No cargo command was run.

This batch belongs to the stack seams plan's S14 pass (rulings S14 and S33),
phase B: the plans the early-September snapshot judged, re-judged against the
tree. Each record here supersedes the snapshot's record for its plan (ruling
S34). A read-only subagent (opus) drafted the records; a second, independent
read-only subagent (opus) then tried to refute every stale claim and checked
every status quote, and its corrections are applied below; this session
re-checked a sample directly. The "holds" counts are the draft's. Nothing in
the plans changes here; corrections are phase C's, and plans found complete
are extracted and archived there (ruling S36).

Checked directly in this session: errand's `FeedEntry` carries `guid` and
`enclosures` at the base (smolweb fidelity, the §1 feed rows). The verifier
refuted nothing, found the "always Unknown" evidence pointed at a test fixture
(now the Nematic engine sites), corrected four citations and added three stale
items the draft had missed (plan:77's styling cells, plan:79's inert forms,
plan:241-242's Knot override); the record and its counts are corrected.

## nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md

- disposition: current
- status line: "Status: planning (with Mark). Extends the [native smolweb rendering plan](2026-06-27_native_smolweb_rendering_plan.md) (that effort shipped: transport → parse → native themed render → scene → window, with scroll, link nav, per-site/app theming). This one recovers the spec-faithfulness the flavour-neutral pipeline collapses, and closes the security posture the native lane currently drops." — accurate: no
- claims checked: 44 — holds: 26, stale: 17, unverifiable: 1

### Stale claims

- Status line, plan:4: says "planning (with Mark)", with no date. The tree shows work has landed. The Micron reading slice and both consumer integrations were implemented 2026-09-13, and plan:106 says so itself. Lane 3 (forms) met its done-condition 2026-09-13. Part of WS1 has landed too: the Spartan prompt is typed (080a2141, 2026-08-20) and feeds now keep guid and enclosures (5630e256, 2026-09-04). Both commits are ancestors of 535bca11.
- §1 table, plan:343, says `<enclosure>` has "no field on `FeedEntry`". The tree has `FeedEntry.enclosures: Vec<FeedEnclosure>` (crates/system/errand/src/parse/feed.rs:43, :71), added in 5630e256.
- plan:344 says `<guid>`/`<id>` is dropped. The tree has `FeedEntry.guid` (feed.rs:66), filled from `"guid" | "id"` (feed.rs:308).
- plan:345 says "first-wins, `rel` ignored". In the tree, `rel="enclosure"` now goes to enclosures (feed.rs:236). Only alternate or no-rel links are still first-wins (feed.rs:203-213).
- plan:348 says Spartan `=:` "becomes body text". The tree parses it to `SpartanLine::Prompt { target, label }` (errand/src/parse/spartan.rs), and Nematic lowers it with `push_submit` (crates/nematic/nematic/src/spartan.rs:71-73). It is not `GemLine::Prompt`: gemini-protocol 0.1.7's GemLine has no such variant.
- The source line references are dead:
  - feed.rs:212, :207 and :174 now point at unrelated lines. The summary merge is at :330, the date merge at :325, and link handling at :203.
  - errand/src/parse/gopher.rs is now a 20-line re-export, so gopher.rs:112-119 does not exist. The gopher grammar lives in gopher-protocol 0.1.1 at src/menu.rs:166-173, pinned at mere Cargo.toml:361.
- Trust gap, plan:363, and Findings, plan:574, say the native lane goes "errand-parse → `SmolwebDocument`, bypassing `Block`". The tree shows `SmolwebDocument` holding `document: Arc<EngineDocument>`, which is Nematic output (crates/system/document-lanes/src/smolweb.rs:7-10, :73-74). It held an EngineDocument by value from its first commit, 8460ed46 (2026-09-02). So both lanes already carry `EngineDocument.trust`. The Nematic engines always set it `Unknown` (gemtext.rs:68, gopher.rs:99, feed.rs:76, micron.rs:71, nex.rs:112, spartan.rs:90, :108); document-lanes never sets it.
- plan:514 says "`genet-documents` installs an `InMemoryTofu`". At base that install is in mere-document-lanes (crates/system/document-lanes/src/remote.rs:226). Genet's components/genet-documents has no InMemoryTofu reference.
- plan:515 says "no durable `TofuStore` exists anywhere in the workspace". This was false even on its 2026-09-02 date. Turnstone has had a file-backed `GeminiTrustStore` (`gemini_trust.json`, turnstone src/gemini_trust.rs:7-23) since dca3207 (2026-08-18). It implements `fetch::SmolwebTofuStore` (:120) and is installed via `fetch::install_smolweb_tofu` (turnstone src/shell/mod.rs:648).
- WS2 "Surface in the host", plan:494, and the cross-references at plan:613 name the meerkat `ensure_smolweb` in `content/handlers.rs`, and the host integration plan's P3/P4, as the trust-surfacing touchpoint. No `ensure_smolweb` exists at base. Meerkat is gone, and the plan itself says so at plan:513. The host integration plan carries a 2026-09-05 historical note calling itself a Meerkat receipt.
- The errand cross-repo model is out of date: plan:616 "sibling repo `mark-ik/errand`", plan:458-461 path override plus "errand push", and plan:433-439 lockstep "errand → genet → mere" plus "settle before any crates.io publish". In the tree, errand is a Mere workspace member (crates/system/errand/Cargo.toml: "Landed in mere 2026-09-03"; version 0.3.4, publish = true). The gemtext and gopher grammars are crates.io pins of the smolweb crates gemini-protocol =0.1.7 and gopher-protocol =0.1.1 (mere Cargo.toml:359-362). DOC_POLICY.md:248 records errand 0.1.0 published 2026-07-04.
- Lane 2, plan:164-172, and the Micron table's "still open" cell, plan:77 ("folding, anchor scrolling … require additional presentation or interaction support"), are out of date. The navigation plan, 2026-09-15_micron_navigation_plan.md:3, has C1, C1b, N1, P1 and P1b landed with A1 next. `SmolwebDocument` has `folds: FoldState` and `in_page` (smolweb.rs:87-93). Navigation receipts exist under tests/fixtures/micron/nomadnet-1.4.2/navigation/. This plan never links its lane-2 plan.
- Lane 3 follow-ups, plan:257-267, list three Knot items as open (Progress 2026-09-13, plan:593, names the response cap). All three are fixed on knot-editor main:
  - Response cap compared after unpacking: fixed by 6d6e7eb (2026-09-13).
  - Reply text surviving a form close: fixed by b88440b (2026-09-13).
  - Scroll offset lost during submission redraws: fixed by 6f0a3b3 (2026-09-14).
  - Separately, d460670 added the `KNOT_NOMADNET_*` env overrides.
- The 2026-08-04 carrier correction, plan:490, says TCP/TLS is "the only one wired today". The plan's own 2026-09-13 entries record NomadNet/Micron fetched over Reticulum through Retinue, in Turnstone a596f18 and Knot 4d88091 (turnstone Cargo.toml:165). WS2's posture mapping has no entry for that carrier.

- plan:77's still-open cell also names color, underline, alignment and indentation, which plan:113-115 records the reading slice landing.
- plan:79 says "Consumer field state, submission … remain inert"; lane 3 landed them (Knot fae329c, Turnstone d710af9).
- plan:241-242 says Knot has no environment override; d460670 added the `KNOT_NOMADNET_*` overrides.

### Contradictions

- The top status line ("planning") disagrees with the 2026-09-13 Micron section status (plan:106, "implemented and tested") and with lane 3's "done-condition … was met".
- WS2 still names meerkat's `ensure_smolweb` as the host touchpoint (plan:494) while the same section's 2026-09-02 re-check says meerkat was deleted 2026-07-18 (plan:512-514).
- The 2026-08-03 home-refinement note (plan:11-15) says grammars follow the wire crates into smolweb. WS1 (plan:446-450 for gopher, :452-456 for gemtext) and the cross-references (plan:616) still direct gopher and gemtext enrichment at errand files.
- The DOC_README.md entry (DOC_README.md:553-563) repeats "planning (with Mark)" and "the native lane, which currently drops `DocumentTrustState`". It does not mention the Micron conformance lanes that now make up most of the plan.

### Recommended action

- Replace the status line with a dated one. Suggested wording: in progress; Micron reading slice and both consumers landed 2026-09-13; lane 3 closed 2026-09-13; lane 2 runs in micron_navigation_plan (C1–P1b landed, A1 next); lane 1 headed qualification and lane 4 open; WS1 partial (Spartan prompt 080a2141, feed guid and enclosures 5630e256; date/content split, channel ttl and gopher raw_type, 8/T fix and CSO open in gopher-protocol); WS2 and WS3 not started.
- Add a dated annotation to the §1 table rows at plan:343-345 and :348 citing those commits. Repoint the line references to feed.rs:325, :330 and :203, and to gopher-protocol menu.rs:166-173.
- Annotate the trust gap (plan:361-367), Findings bullet 2 and the "Carry through the view" bullet: the native lane now carries EngineDocument.trust, so WS2 comes down to producing the transport descriptor, populating `trust`, and surfacing it in host chrome.
- Correct the 2026-09-02 re-check:
  - Name mere-document-lanes remote.rs:226 as the InMemoryTofu install.
  - Record Turnstone's file-backed GeminiTrustStore (dca3207).
- Replace the meerkat touchpoint with Turnstone and Mere hosts, and mark the host integration cross-reference as a historical receipt.
- Rewrite the errand cross-repo, lockstep and publish notes for errand as a Mere member, with grammar changes going through gemini-protocol and gopher-protocol releases plus a Mere repin.
- Link micron_navigation_plan from lane 2 and record the phases that landed there. Strike "folding, anchor scrolling", and the color, underline, alignment and indentation the reading slice landed (plan:113-115), from plan:77's still-open cell with a dated note.
- Record Knot 6d6e7eb, b88440b, 6f0a3b3 and d460670 against the lane 3 follow-ups.
- Note that the Reticulum carrier is now wired for NomadNet, and extend the WS2 mapping to cover it.
- Update the DOC_README entry to match.
- The plan is not complete: WS2 and WS3 are unstarted, WS1 is partial, and Micron lanes 1, 2 and 4 are open. It is not an archive candidate.

### Notes

Judged at 535bca11; the plan file is unchanged from base to HEAD.
- Mere: merge-base for 91c6238d, dce5cc97, 080a2141 and 5630e256; git show of errand parse/{feed,gemtext,gopher,spartan}.rs, nematic {gopher,gemtext,spartan,micron*,file}.rs, document-lanes smolweb.rs and remote.rs, fetch lib.rs:761-776, document-canvas text.rs:218-223 (SoftBreak renders as a space, with no setting, so WS3's hard-break item holds open).
- Siblings: the registry copies of gopher-protocol 0.1.1 and gemini-protocol 0.1.7; the Retinue receipt with its `## Typed form request receipt` heading at :355 and `StringMapRequest`.
- All cited Knot, Turnstone, Retinue, Genet and Netrender SHAs resolve.
- The lane 3 headed acceptance is unverifiable: its artifacts are gone (C:/t/micron-headed-20260913 is absent) and cargo was not run.
- Cambium's `gopher_view`/`feed_view` (crates/cambium/cambium/src/nematic/views.rs:204-328) still exist but have no consumers in Mere, Turnstone or Knot.
- In passing, beyond this record: DOC_README's entries for native_smolweb_rendering_plan ("planning") and micron_navigation_plan ("C1 landed, N1 next") lag those plans' own status lines. Knot already consumes Micron navigation (4680eb5, ba25e1a, 2026-09-16) while the navigation plan says A1 is next.
