# Batch 35 — S14 pass, phase A, first seven (records for documents added since 2026-09-05)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| eidetic_docs/implementation_strategy/2026-09-16_hagiograph_history_organ_plan.md | historical-unmarked | no | 16 | 13 | 2 | 1 |
| mere_docs/implementation_strategy/2026-09-05_projection_refresh_and_surface_reuse_plan.md | historical-marked | yes | 15 | 14 | 0 | 1 |
| mere_docs/implementation_strategy/2026-09-16_coop_lifecycle_parity_plan.md | historical-marked | yes | 23 | 22 | 0 | 1 |
| mere_docs/implementation_strategy/2026-09-16_lattice_sync_pass_plan.md | current | no | 20 | 15 | 4 | 1 |
| mere_docs/implementation_strategy/2026-09-16_lexical_capability_plan.md | current | no | 4 | 3 | 1 | 0 |
| mere_docs/implementation_strategy/2026-09-20_ranged_fetch_plan.md | current | no | 21 | 16 | 3 | 2 |
| mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md | current | no | 23 | 16 | 5 | 2 |
| **Totals** |  |  | **122** | **99** | **15** | **8** |

**Totals: 7 docs, 122 claims checked (99 holds, 15 stale, 8 unverifiable), 15 contradictions.**

Audit base: Mere `26060e88` (2026-10-05 21:58). All seven documents are
byte-identical between the working tree and the base. Sibling repositories
(isometry, turnstone, woodshed, knot-editor, genet) were read at their HEADs
on 2026-10-05, none of the cited evidence dated after that day. `archive_docs/`
is excluded. No cargo command was run, so build, test and gate claims count
as unverifiable.

This batch belongs to the stack seams plan's S14 pass (rulings S14 and S33),
phase A: a D2 record for each active document added since the 2026-09-05
snapshot without one. A read-only subagent (opus) drafted the seven records.
Every stale claim and contradiction below was then re-checked in this session
against the base and the sibling repositories: commit ancestry on main and
origin/main, the `DOC_README.md` lines, the root `Cargo.toml` patch rows,
`scripts/cargo_mode.py`, the portable workflow, the isometry and Knot pins, and
Turnstone's `src/shell/mod.rs`. The "holds" counts are the draft's. Nothing in
the seven documents changes here; corrections are phase C's.

## eidetic_docs/implementation_strategy/2026-09-16_hagiograph_history_organ_plan.md

- disposition: historical-unmarked
- status line: "Status, 2026-09-16: H1 to H4 landed. H5, Mesocosm pinning and adopting the crate, is next. No consumer yet." — accurate: no
- claims checked: 16 — holds: 13, stale: 2, unverifiable: 1

### Stale claims

- "H5, Mesocosm pinning and adopting the crate, is next." H5 landed in
  isometry the day the plan was written, both commits on isometry main:
  `8e41d90` ("Build Mesocosm's world record on the hagiograph's generic
  record": `WorldRecord` is a newtype over `hagiograph::Record`) and `490fb32`
  ("Run deep time before a generated world is entered": `impl
  hagiograph::Epochal for Unheld` and a call to `hagiograph::run`).
- "No consumer yet." `isometry/mesocosm/Cargo.toml:41` and
  `isometry/shared/isocosm/Cargo.toml:18` pin `hagiograph` at Mere
  `32edc2ad` (on main), and eponym-world uses it in `src/transitions.rs`.

### Contradictions

- `DOC_README.md:608` indexes the plan as "2026-09-16, plan" while the tree
  shows all five steps done.

### Recommended action

- Status to "H1 to H5 landed: H1 to H4 in Mere (`4a895c5e`, `53648d3a`), H5 in
  isometry (`8e41d90`, `490fb32`), 2026-09-16; consumers Mesocosm, isocosm and
  eponym", with an H5 Progress entry.
- Extract §8's "Later, not planned here" items (promotion, retelling, the
  attention fade, memorials) to a plan and archive this one (DOC_POLICY §8; a
  fork for Mark in this pass).
- `DOC_README.md:608` to match.

### Notes

The plan's source facts hold at the base: `record.rs` 155 lines,
`reckoning.rs` 61, `deep_time.rs` 114, each with a sibling test file; 12 + 4 +
5 = 21 tests; the §1 fixture bytes at `record_tests.rs:49`; `Record`'s
hand-written `Default` (`record.rs:72`); `Clone` required only on `merge` and
`reckon`; `DeepTimeError::Stalled { ticks }` counting advances; `postcard` a
dev-dependency only; the cited eidetic review brief lines; `WorldRecord` 405
lines at `8e41d90^`. §1's "26 lines, no consumers" describes the baseline
before the plan and is not counted stale. Unverifiable: the clippy, rustfmt
and wasm32 gate claims.

## mere_docs/implementation_strategy/2026-09-05_projection_refresh_and_surface_reuse_plan.md

- disposition: historical-marked
- status line: "Status: bounded implementation and focused validation complete, 2026-09-06." — accurate: yes
- claims checked: 15 — holds: 14, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- `DOC_README.md:28` presents the plan as "Active implementation"; its own
  status says complete as of 2026-09-06.

### Recommended action

- Extract §8's open points (persisted resource settings; background-work and
  texture-residency budgets; independent appearances for engines other than
  Reader; native frame performance and background CPU, unmeasured) and archive
  the plan (a fork for Mark in this pass).
- `DOC_README.md:28` to a completed or archived pointer.

### Notes

Checked: the URL-authority dependency selected only for by-site kanban
(`crates/canvas/pictograph/src/canvas/strategy.rs:67-71`); `placement_reused`
in `scenomise/src/projection.rs`, with label-only refreshes reusing placement
and moved inputs forcing a solve (`ports/graphshell/src/projection_compile.rs`);
both hosts on that path; Pelt's `with_surface_resource_policy` and
`SurfaceResourcePolicy` defaults 4 and 1, with no serde derive (not persisted);
the receipt `ports/graphshell/docs/receipts/projection_refresh_surface_reuse_receipt.json`
matching the doc's counts and medians; favicon `fc4bbc42` on main; Turnstone's
native receipt and favicon correlation tests. Unverifiable: that the recorded
runs passed.

## mere_docs/implementation_strategy/2026-09-16_coop_lifecycle_parity_plan.md

- disposition: historical-marked
- status line: "Status: slice 1 done 2026-09-17 (facts table below); slice 2, the contract, done 2026-09-18 (record at the end)." — accurate: yes
- claims checked: 23 — holds: 22, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- The "Stop rules" section (line 407 at the base) says "No shared contract
  type, crate or view is extracted in this slice." It sits after the slice 2
  section, whose K1 extracts `moot::coop` (`4837248d`). The rule was written
  for slice 1, but the section does not say so.

### Recommended action

- Scope the stop rules to slice 1.
- Extract the open follow-ons (the authoring-time judgement lane for revoked
  writers; membership-gated sync; a Turnstone revoke whose member's recipient
  is not registered locally; `turnstone_profile()` still declaring
  `receipts_duplicate_replay: false`; the fixture's browser receipt for the
  new controls; the empty `expires_at_ms` and `revocation.by`/`at_ms` in
  Turnstone's report) and archive (a fork for Mark in this pass).

### Notes

Checked: eight Mere commits (`71a767b8`, `af674f30`, `3d3ad81c`, `11f0d705`,
`41f35b4d`, `5e3850f7`, `4837248d`, `445c6ceb`) ancestors of the base; five
Turnstone commits on its main; the named stickleback, commons and moot symbols
at their cited lines; `projection_from_checkpoint` absent as the plan says;
the fixture capabilities and receipt; Turnstone's three conformance tests and
receipt. The plan has no `DOC_README.md` entry; the suite census links it.
Unverifiable: headed runs 29 and 30, drain latencies, `git diff --check`.

## mere_docs/implementation_strategy/2026-09-16_lattice_sync_pass_plan.md

- disposition: current
- status line: "Status: in progress, 2026-09-20. Mark authorized the reviewed sequence: establish local and portable resolution in Mere and Turnstone first, then move that pair onto a tested published Genet revision. Broader consumers follow the same procedure once qualified. The September 16 counts below remain historical." — accurate: no
- claims checked: 20 — holds: 15, stale: 4, unverifiable: 1

### Stale claims

- The status line, dated 2026-09-20 against Progress running to 2026-09-25,
  names as current a sequence whose two stages are met: Mere moved onto a
  published Genet in L2 pass B (`2a35077e`, 2026-09-22), and Turnstone moved
  too (next item); the consumer round ran 2026-09-23 (knot-editor `44f0519`,
  turnstone `ab6ff4e`).
- Turnstone's pins: the plan's last word is that Turnstone "retains its tested
  portable Mere `ca798151` / Genet `5ae30cad` set". turnstone `5d2c910`
  (2026-09-22) repinned Mere `0e031fa5`, and `ab6ff4e` (2026-09-23, on
  origin/main) aligned to Mere `250fd238`, Genet `532f1fad`, knot-editor
  `44f0519` and woodshed `95d1085`.
- CI: the plan's last word is "CI is not installed by these commits; publish
  the prepared workflows". `.github/workflows/portable.yml` ("Portable
  dependency graph") landed in `40670801` (2026-09-23), on origin/main.
- The unused-patch baseline: stop rule 3 compares against "the baseline three"
  (boa_engine, boa_gc, iroh-mdns-address-lookup). `a5543904` (2026-10-03)
  removed the iroh-mdns row; only `boa_engine` and `boa_gc` remain
  (`Cargo.toml:734-735`).

The second and third are dated Progress text, true on their dates; they count
as stale because they are the plan's last word on items it left open.

### Contradictions

- Done-condition 3 and the 2026-09-16 "Local locks" ruling have repos carry
  `resolver.lockfile-path` in config; the Execution amendment supersedes that,
  and the code follows the amendment: `scripts/cargo_mode.py` supplies the
  lock path as a command-line override (`:136`) and refuses, in portable mode,
  any config carrying `resolver.lockfile-path` (`:60-61`).
- The amendment's "Root portable baselines are being checked before
  repinning" is still present tense after L2 pass B landed.
- `DOC_README.md:64` repeats "in progress 2026-09-20".
- The ranged fetch plan has T2 waiting on "the next lattice round"; that round
  ran on 2026-09-23 and is not recorded here.

### Recommended action

- Status to record L1 and L2 passes A and B landed, the 2026-09-23 consumer
  round, CI live since `40670801`, and the two-row unused-patch baseline since
  `a5543904`; and name what remains (L5's knot-editor split between root
  `562353aa` and `ports/djinn` `ea3e99ef`; L3 and L6/L7 consumers beyond
  Turnstone; P2's cleanup, branch `burn-pre3-repin` still present and tag
  `archive/burn-pre3-repin-20260916` absent; P4).
- Rewrite done-condition 3 and the local-locks ruling's mechanism to the
  amendment (a dated annotation).
- Whether the pass continues or closes with its remainder extracted is a fork
  for Mark.
- `DOC_README.md:64` to match.

### Notes

Checked: Mere `0f5283d9`, `68f78873`, `2a35077e`, `250fd238`, `4b33a963`,
`bc7121ae`, `67ebef3a` on main; Genet `99769450`, `532f1fad`, `b7d56321` on
genet main; the launcher's setup, local and verify modes and four tests;
`.cargo/config.toml.example` without `servo-paint` or `genet-probe`;
`.gitignore` tracking the root and three guest locks; `rust-toolchain.toml`
at 1.98.1. Dated drift not counted: root genet is now `bd3e8861`, and
Turnstone now pins Mere `bd5912fb` and Genet `69a2383b`. Unverifiable: the
package counts, lock digests (the lock has changed since) and the 308/0 run.

## mere_docs/implementation_strategy/2026-09-16_lexical_capability_plan.md

- disposition: current
- status line: "Status (2026-09-16): accepted; not started. New objective, raised by Mark 2026-09-16. Nothing implemented. The verified survey behind it, with licences, sizes and formats per dataset, is the [lexical and grammar resources brief](../research/2026-09-16_lexical_grammar_resources_brief.md)." — accurate: no
- claims checked: 4 — holds: 3, stale: 1, unverifiable: 0

### Stale claims

- "Not started … Nothing implemented." `crates/intel/reference-data`
  (package `reference-data`) landed 2026-09-30 in `9310518b` ("Add opt-in
  validated local lexical reference packs"): a lookup from lemma, language and
  sources to language-tagged senses and relations with per-source provenance,
  packs installed opt-in and checked against a BLAKE3 digest, each manifest
  carrying licence and attribution. Knot consumes it (`knot-editor/Cargo.toml:29`
  pins it at Mere `9310518b`). That covers much of L2, Knot's half of L4 and
  part of L5; L1 ingestion, the morphology and etymology layers, Harper (L3)
  and the Turnstone consumer are absent.

### Contradictions

- Two Mere lexical surfaces now exist on paper, this plan's L2 crate and
  `reference-data`, and neither document names the other, against the plan's
  own "neither keeps its own copy".
- `DOC_README.md:65` repeats "not started".

### Recommended action

- A fork for Mark: is `reference-data` this plan's L2 foundation? If so, the
  status records L2 as partly built and L2 re-scopes onto that crate; if not,
  the record says why two lexical APIs are justified, or one retires.
- `DOC_README.md:65` to follow.

### Notes

Checked: the brief link resolves; no `harper`, `wordnet`, `morphynet` or
`etymdb` dependency or code in the tree; Turnstone has no dictionary-lookup
consumer; Mere's `esp.embed.lexical` and `crates/mesh/mesh/src/resources/lexical.rs`
are feature-hashed embeddings, not this capability. The survey's external
facts (dataset sizes, licences) are not claims about the tree and were not
checked.

## mere_docs/implementation_strategy/2026-09-20_ranged_fetch_plan.md

- disposition: current
- status line: "Status: plan accepted 2026-09-20 with D1 to D7 decided; lane F merged and pushed 2026-09-20; lane R pushed as Woodshed `bf5923d`; T1 and T3 pushed/built in Turnstone 2026-09-22; M1 built in Mere; T2 waits on the next lattice round." — accurate: no
- claims checked: 21 — holds: 16, stale: 3, unverifiable: 2

### Stale claims

- "T2 waits on the next lattice round" (status, and Progress line 209). The
  round ran: turnstone `ab6ff4e` (2026-09-23, origin/main) took Mere
  `250fd238`, which carries M1 `b3d52c74`. T2 is unblocked but undone:
  Turnstone's `src/shell/mod.rs` still builds one store set from
  `fetch::session_stores()` under the comment "Keying the set by persona is
  lane T2".
- "M1 built (Mere `b3d52c74`, local)" (Progress line 203): `b3d52c74` is on
  main and origin/main, and Turnstone consumes it.
- "T3 built (Turnstone, local)" (Progress line 216): T3 is turnstone `3671ad3`
  ("Keep the session cookie jar on disk again", 2026-09-22), on origin/main.

### Contradictions

- `DOC_README.md:169` indexes the plan as "accepted 2026-09-20, lane F
  starting"; the plan records F, R, T1, M1 and T3 landed.
- Line 151's "three unused-patch warnings" was true on its date; since
  `a5543904` the baseline is two (the same drift as the lattice record).

### Recommended action

- Status to "F, R, T1, M1 and T3 landed and pushed; Turnstone took M1 in
  `ab6ff4e` (2026-09-23); T2 open and unblocked".
- `DOC_README.md:169` to match.

### Notes

Checked: Mere `c0463e98`, `ba4f4951`, `e35898d1`, `68f78873`, `b45ecbbb`,
`50699731`, `b3d52c74`, `0e031fa5` on main; the fetch crate's handle types,
`spawn_fetcher_with`, `session_stores`, its default features and 20 tests;
all eight links; Woodshed `bf5923d`, `a1ebf6d`, `fdc5980`, `f5a6649` on
origin/main and no `ureq` in Redshank's lock; Turnstone `2c451d2` and
`77b7ece` on main and its session-reopen test. Unverifiable: `mere-fetch`
0.0.1 on crates.io, and the measured figures.

## mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md

- disposition: current
- status line: "Status: in progress, 2026-09-23. C1-C3's clear-cut moves are landed and pushed. Mark ruled every open question the same day (see Rulings): C4's folds are in progress in this session, insigne's delegation split is the dramatis session's (phase A landed 2026-09-24 as `5364dfa0`; phases B to D follow in the [insigne proofs plan](../../dramatis_docs/implementation_strategy/2026-09-23_insigne_proofs_plan.md)), and chatelaine waits on a CXF-shaped taxonomy." — accurate: no
- claims checked: 23 — holds: 16, stale: 5, unverifiable: 2

### Stale claims

- "C4's folds are in progress": every C4 fold landed by the plan's own
  Progress (titulus into chirograph `83feb122`, eidetic-fjall `3943874f`,
  graphshell carriers `0c65a9d6`, canvas and signals into pictograph
  `f590e45d`, `distillery::lifecycle` and `mesh_host`, `cambium::nematic`).
- "Phases B to D follow": phase B landed as `538226a3` (2026-09-26, Progress
  line 366); the insigne proofs plan records C in Mere and Knot and D in Gaz
  on 2026-09-29.
- "Chatelaine waits on a CXF-shaped taxonomy": chatelaine P1 to P3 landed on
  main (`da3c50bc`, `3e4992ec`, `ff68e86c`, Progress line 387).
- The C2 table's chatelaine row (line 50) still says "unbuilt".
- The C2 table's insigne row (line 51) still says "phases B to D open".

### Contradictions

- The status line and the C2 table contradict the plan's own Progress
  entries of 2026-09-24, 09-25, 09-26 and 10-02.
- They contradict the insigne proofs plan's status ("C landed in Mere and
  Knot on 2026-09-29 … D landed in Gaz").
- The status cites "(see Rulings)", but the plan has no Rulings section; the
  rulings sit under "The ruling this serves" and in the C4 table.
- `DOC_README.md:63` repeats "chatelaine waits on a CXF-shaped taxonomy".

### Recommended action

- Status to record C1 to C4 and C2a landed, insigne A to D landed, chatelaine
  P1 to P3 landed (meeting C5's conditions), and what remains open: C5's
  version baseline, C6's crates.io deletions (Mark's), and the dramatis
  facade.
- The chatelaine, insigne and tabard rows marked landed; "(see Rulings)"
  pointed at the real heading; `DOC_README.md:63` to match.

### Notes

Checked: 27 Mere commits ancestors of the base; woodshed `a57085b`, turnstone
`d6b62ad`, knot `5ad3f67` and `c6d5b9e`; C1's deletions (no `sibylla` or
`vates` outside docs bar the esp README's historical note, no `register-*`
packages); the folds' features and modules at their cited lines; tabard's
theme and smolweb modules and syntax properties; insigne's `TypedKey`,
`CheckFault` and `verify` feature; `dramatis` at 32 lines ("unbuilt" holds);
chatelaine at 10 source files. Unverifiable: test and package counts, and
C6's ten names on crates.io. Dated drift not counted: djinn's knot pin is now
`ea3e99ef`, and signalman joined the workspace in `a5543904`.
