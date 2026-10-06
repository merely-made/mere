# D2 source-document remediation receipt: the S14 pass, phase C

**Date:** 2026-10-06

**Base:** the records were cut at Mere `535bca11` (phase B, batches 39 to
51) and at phase A's bases (batches 35 to 37). The corrections landed on main
between `3e107e32` and `8d7702d2`.

**Scope:** phase C of the S14 status-versus-code pass, carried out under
rulings S33 to S66 of the
[stack seams plan](../../../design_docs/mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md).
Phase C applied every recommended action in batches 35 to 37 and 39 to 51 to
its document. It also archived the plans the pass found complete, extracting
their open items first, and corrected DOC_README.

The batch files remain the immutable judgment record for the trees they
audited. Their stale, contradiction and unverifiable counts are not rewritten
after remediation; the errors found in them since are listed under Record
errata.

## Corrections by lane

Fourteen opus write lanes did the corrections (S47). Each lane worked in its
own worktree and edited only its assigned documents. Before each `--no-ff`
merge, its diff was reviewed and both audits were run.

The conventions:

- a corrected status line reads `**Status (2026-10-06):**`;
- stale text keeps its words beside a `**Corrected 2026-10-06 (S14 pass):**`
  note;
- a question that has more than one defensible answer becomes an
  `**Open, raised by the S14 pass (2026-10-06):**` note naming its options;
- meerkat-era work uses S39's wording ("landed in meerkat on <date>, retired
  with it 2026-07-18 (`c5f01064`)").

| lane | records | assigned | files changed | corrections | Open notes | merge |
|---|---|---:|---:|---:|---:|---|
| L1 | batches 35, 36 | 13 | 9 | 23 | 0 | `3e107e32` |
| L2 | batches 37, 51 | 8 | 6 | 25 | 4 | `8dcba9da` |
| L3 | batch 39 | 12 | 11 | 18 | 7 | `c5a500c1` |
| L4 | batch 40 | 12 | 12 | 36 | 3 | `e5700649` |
| L5 | batch 41 | 12 | 11 | 45 | 8 | `62a6f5c6` |
| L6 | batch 42 | 12 | 11 | 37 | 12 | `2751449c` |
| L7 | batch 43 | 11 | 11 | 32 | 2 | `0563951d` |
| L8 | batch 44 | 10 | 10 | 19 | 5 | `b4cf963e` |
| L9 | batch 45 | 11 | 11 | 28 | 5 | `63041dd0` |
| L10 | batch 46 | 9 | 9 | 30 | 8 | `9387aa1d` |
| L11 | batch 47 | 11 | 10 | 30 | 2 | `3ba4252f` |
| L12 | batch 48 | 12 | 12 | 49 | 6 | `1113bd25` |
| L13 | batch 49 | 12 | 12 | 34 | 7 | `476b6f4c` |
| L14 | batch 50 | 10 | 10 | 22 | 6 | `a3b69537` |
| **total** |  | **155** | **145** | **428** | **75** |  |

The 428 corrections sit in 134 documents and the 75 Open notes in 61. Ten
assigned documents needed no edit, either because the record asked for none
or because the lane found nothing stale. They are four L1 documents (the
projection refresh plan, the lexical grammar resources brief, and the
graphshell producer and apparatus diagnostics receipts), two L2 receipts
(R2_A and R2_B_C), the fact visualization leaves plan (L3), the overmap
sessions graph plan (L5), the scenograph 0.0.3 release plan (L6) and the net
media plan (L11).

Three more Open notes came from outside the fourteen lanes, one of them the
lexical plan's (S38, `eefe52e3`). That makes 78 tree-wide in 64 documents. Of
those, 29 now sit in 26 archived plans and are carried as tails (below); 49
remain in 38 active documents, each a fork for its lane or for Mark.

Ruling S46 kept eleven live plans out of the lanes. Their own lanes corrected
them from the same records:

- the identity lane in `8fa9bf26`;
- the projection grammar lane in `89ec26b6`;
- the Conatus lane in `a469ed28` through `a6f57003`, merged in `12220b8f`.

## DOC_README

Lane D (`d2184520`) applied the lanes' proposed DOC_README edits: 99 entries
corrected and 9 orphans indexed. Proposals for live plans were skipped
(S46). DOC_README lines 106, 362 and 436 stay with the Conatus lane.

## Archive

Lane E (`9967596a`, merged in `8d7702d2`) archived 88 plans under S36 and
S48 to S66:

- 77 to `archive_docs/2026-10-06_completed_plans/`;
- 7 to `archive_docs/2026-10-06_superseded_plans/`;
- 4 to `archive_docs/2026-10-06_retired_plans/`.

46 of the 88 were plainly complete and archived under S36 without a round
(S49); the other 42 were ticked in rounds 18 to 21 (S50 to S61, S64, S65). The move tool
rewrote 650 relative links and 24 more were repointed by hand. The D2 records
keep the old paths as history (210 citations).

Each plan's open items were extracted before it moved. About 200 tails went
into the section "2026-10-06 archive pass" of the
[archived plan tails plan](../../../design_docs/mere_docs/implementation_strategy/2026-07-03_archived_plan_tails_plan.md)
(S48). Of these, 139 stay in that backlog; nine of the 139 are in the
subsection kept for the S59 critical pass. The section also holds 22
`owner:` tails, 12 tails already carried by a named plan, 5 Received notes,
4 tails carried by the distillery models plan, 7 items already done or
settled, and 11 plans with no open tail. The render ladder's entry carries
Mark's note that it "needs rethinking" (S50).

The five Received notes went into three plans:

- the graph semantics plan: two, at P4 and after P5 (S53);
- the graphshell reference host plan: two, G6/G7 and the C4 consent gate at
  H5 (S55, S61);
- the smolweb fidelity plan: one, host integration plus S66.

S58's four ML plans are carried by the new
[distillery models plan](../../../design_docs/mere_docs/implementation_strategy/2026-10-06_distillery_models_plan.md)
(S63, `77137426`, record batch 53).

The live lanes took their `owner:` tails into their own plans the same day:

- identity (`73812e3a`): insigne's repins, the dramatis facade and the
  wallet carry gaps into the dramatis repo plan; V4 and CXF import into the
  chatelaine plan; the startup unlock backends into the vault lock plan,
  already decided by its rulings 22 and 42. It also took the castellan keeper
  split, which the tails section had kept in the backlog; that entry carries
  a dated amendment;
- Conatus (`a0cfa30a`): the Nexus adoption watch into the conatus engine
  plan; the arrangement pull into the dynamics grammar plan, recorded there
  as retired by G7;
- projection grammar: FT9/FT10 were sent to its lane and await uptake.

The tails owned in other repositories are recorded and were not edited:

- Turnstone: the page capture C4 gate, the pane registry's A4 and the T
  lane's pointer-capture gap;
- knot-editor: the copies of its port, authoring consumer, knot-in-graphshell
  and publishing protocol plans, its application workspace plan's F0, and
  the `write_bytes_with_backup` settings migration;
- Isometry: the genet host migration plan's Z5 figure.

## Rulings applied

- **S35:** three receipts renamed and S10 flattened (`c8ca1a63`, with
  digests updated in `286c1a4c`).
- **S38 to S43:**
  - forks routed to their lanes (S38);
  - meerkat-era wording (S39);
  - inker and tinct shown as MPL-2.0 in `LICENSES.md` (S40, `fd4a599d`);
  - S37's question ruled directly (S41), and the lexical plan's L2 recorded
    as partly built in `reference-data` (S42);
  - the lattice sync pass's status rewritten to what landed (S43).
- **S46 to S48:** live plans to their lanes, reviewed write lanes, ownerless
  tails to the backlog.
- **S49 to S65:** archive selection.
  - S63 founded the distillery models plan.
  - S65 records the doc policy work in DOC_POLICY itself, as a dated
    amendment that keeps the old words.
- **S66:** written into the smolweb fidelity plan as its Received note.

## Readings, not ruled

- **Batch supersession and archived records.** A later batch record
  supersedes an earlier one for the same document, extending S34
  (`bb8a3e8a`). Records of documents moved into `archive_docs` count as
  inactive supplemental history (`38c4812a`). Both changes to
  `scripts/mere_doc_judgment_audit.py` came with controls.
- **knot-editor's copies as owners.** Lane E treated knot-editor's copies as
  the owners of the knot port, authoring consumer and knot-in-graphshell
  tails (the pin rule, the optional headed selected-clip receipt, Knot search
  S0/S1). S61 rules only the publishing protocol's copy canonical, and lane
  L6 had made ownership conditional. Those three plans were archived under
  S49's plainly complete clause. **Ruled 2026-10-06 (S67):** knot-editor's
  copies own them.
- **Petgraph RDF Phase 4.** It went to the backlog, not the graph semantics
  plan. It is gated and was not in the plan's Open list, so it was not
  counted among the "open items" S53 sends to graph semantics.
  **Ruled 2026-10-06 (S68):** it stays in the backlog.
- **The rest:** the capability model's sub-delegation went to the backlog
  because its tracking plan is archived and its gate has been met; castellan
  OTP's follow-ons went to the backlog because S56 allows either home.

## Record errata

Found during phase C. The batch files are not edited.

- **batch_35, lattice:** S43's evidence of "three knot-editor revisions"
  counts Turnstone's pin; Mere's own manifests carry two (`562353aa`,
  `ea3e99ef`). Noted by L1.
- **batch_40 header:** "`JarCookieProvider` appears only in documents, never
  in code" holds at the base only. The type existed in meerkat (added
  `435985d5` on 2026-06-23, removed with `c5f01064`). Found by L4.
- **batch_46, conatus engine notes:** "The pre.4 migration is still pending
  at base" is wrong: `cec0b3a4` is an ancestor of `535bca11`, as batch_50
  records.
- **batch_46, one-tree plan:** the 2026-09-30 collision worktree was
  integrated but not retired. `Code/worktrees/mere-canvas-elapsed` still
  exists.
- **batch_46, minor:**
  - `mesquite/src/lib.rs:55` is `mod scenario;` and `Lane` is re-exported
    at :74;
  - `eae87153` carries author date 2026-08-31 and commit date 2026-09-02,
    and batch_50 uses the author date.
- **batch_50, overtaken after the base:** S17's Knot repin is on Knot's
  origin/main (`c966e31`), and djinn's repin merged on Mere main as
  `840c543d`.

The Conatus lane found the batch_46 and batch_50 items.

## Follow-ons

- **S44:** repin the three `sha2` 0.10 manifests. A separate task, for Mark
  to start.
- **S45:** re-run the MPL header tool over the 61 sources and gate it. A
  separate task, for Mark to start.
- **S59:** a critical pass over the four search and memory plans' tails,
  turned into a plan if needed, with Eidetic's development.
- **S66, code:** add a typed-column block to EngineDocument and retire the
  per-format `cambium::nematic` views.
- **S50:** rethink the render ladder.
- **Projection grammar:** take FT9/FT10 into the adoption plan.
- **Other repositories:** the Turnstone, knot-editor and Isometry tails,
  which go to Mark. **Ruled 2026-10-06 (S69):** a dated note in each owning
  plan.
- **The 49 active Open notes.**
- **A pre-existing broken link, left alone:**
  `archive_docs/2026-06-09_pivot_superseded/2026-05-15_browser_taxonomy_translation_brief.md`
  links to `../implementation_strategy/...`.

## Verification

Run from the repository root:

```text
python scripts/mere_doc_audit.py
python scripts/mere_doc_judgment_audit.py
```

At `8d7702d2` both exit 0.

The doc audit reports:

- 253 active documents and 0 index orphans;
- 0 index ghosts and 0 broken relative links;
- 0 invalid historical annotations and 7 stale ones.

The 7 stale annotations are local-environment artifacts. Six point at
untracked, gitignored files present only on this machine: four probe
directories under `crates/probes` and the h4f receipt under
`ports/graphshell/docs`. The seventh, `components/genet-layout`, resolves in
the genet checkout. A clean clone would not resolve any of them.

The judgment audit reports 253/253 active coverage: 191 legacy and 197
supplemental records. Of these, 47 supersede legacy records and 26 supersede
earlier batches. Inactive history holds 90 legacy and 88 supplemental
records.
