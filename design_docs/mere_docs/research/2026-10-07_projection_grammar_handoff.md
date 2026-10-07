---
artifact_contract: "ce-handoff/v1"
created_at: "2026-10-07T05:30:00Z"
title: "Projection grammar: the catalog and adoption plan, handed off"
summary: "Who owns which parts of mere's projection grammar catalog and adoption plan, what is open or gated there (A3 stage two, A5, Track F), the rulings carried in from sibling plans, and the cross-session agreements around them."
keywords: ["projection grammar", "adoption plan", "catalog", "A5", "Track F", "A3", "arrangement roles", "stack seams", "dynamics grammar"]
cwd: "C:/Users/mark_/Code (machine-local)"
resume_focus: "Keep mere's projection grammar catalog and adoption plan current, and pick up whichever gated target a consumer opens first."
repository: "merely-made/mere"
branch: "main"
head: "ea9e6da7 when written"
---

# Projection grammar handoff (2026-10-07)

Mark asked for this handoff and chose its scope: the mere grammar docs only
(his answer: "mere grammar docs only"). mer3ly's site canvas plan is not
covered here. It has its own record and its own viewer-cone handoff in that
repository (`docs/handoffs/2026-10-07_viewer_cone/`).

## The two documents

- [Projection grammar catalog](2026-08-15_projection_grammar_catalog.md):
  the governing primitive vocabulary and promotion rules. Last reconciled
  2026-10-03, for arrangement roles.
- [Projection grammar adoption plan](../implementation_strategy/2026-08-15_projection_grammar_adoption_plan.md):
  gated targets carrying the projection grammar report's transfers into mere,
  genet and cambium. Its top status line (2026-10-06) is current and
  supersedes the two older status lines further down, which are kept as
  history.

Read `design_docs/DOC_POLICY.md` and `DOC_README.md` before editing either
one. Dated text keeps its words: change it with a dated annotation or a new
entry that says what it amends.

## Who owns which parts

This lane, the "Projection grammar" session [78751e], owns the catalog and the
adoption plan except for these sections, which other lanes wrote:
- **The adoption plan's "Cross-domain relationship recipe continuation
  (2026-10-05)"** and the qualification, host-continuation and Woodshed and
  Knot entries around it, plus `c79bb8c2`, `b2f67356` and `4a2560ed`. These
  came from a different session; Mark knows which. Another session carries
  the same name, "Projection grammar" [6b0934], which is how the stack seams
  session first mistook this lane for the recipe pass (corrected in the stack
  seams plan, `1ed54699`). Route recipe-pass questions to that lane, through
  Mark if needed.
- **The "P1 under way" note** (stack seams rulings S12, S15 to S20) is the
  stack seams session's ("Graph database concept for Mere"); it wrote it at
  `3b220f90`. P1 itself landed at `1633be0c`. This lane added only a dated
  "landed" line.

## Open and gated targets

Nothing in either document is in progress. Every remaining target waits on a
consumer.
- **A3 stage two** (adoption plan, "A3. LOD rungs as declarative conditions").
  Stage one landed 2026-08-19 through P3b. Stage two is portable only once a
  remote viewer re-selects rungs on a camera of its own. Its gate, sharpened
  2026-09-01: `Recency` and `Focused` are host facts the wire does not carry.
- **A5, gap proofs adopt named anatomies.** No proof is active. The
  [Distillery projection walk](../implementation_strategy/2026-09-02_distillery_projection_walk_plan.md)
  W3 is the prospective schematic consumer. At W3's entrance, check the
  existing routed polylines and endpoint semantics before claiming a portable
  port or routing addition. Its entrance gate and per-proof done-condition
  are in the plan.
- **Track F, the field receipts (gated; received 2026-10-06).** F1 is the old
  FT9, derived-mark integrity; F2 is FT10, field distinction. They came from
  the archived projection receipts plan under stack seams S61, and the tail is
  in the [archived plan tails plan](../implementation_strategy/2026-07-03_archived_plan_tails_plan.md),
  section "2026-10-06 archive pass". *Reading, not ruled* (written into Track
  F): both open with A5, because a field consumer would force A5's gap 1 too.
  F2's trigger candidates are host-side radio and mesh placement facts, and a
  simulator or radio model for Current.

## Rulings carried in from sibling plans

These are recorded in the two documents. The rulings themselves live in their
own plans.
- **Arrangement roles** (dynamics grammar plan F18-F30, `ce79a82f`). The
  catalog maps Free, Anchored and Pinned to the seeded, anchored and pinned
  roles, with a per-item choice and seeded as the default. The §6 "Force and
  constraint" row moved out, and "Settled" became an arrangement source.
  Adoption plan A1 carries a note that sceno's old `Hold::Anchored` named the
  seeded role. Score v5 then read old "Anchored" as Seeded (F44, `bd119a69`),
  and mer3ly reads its legacy "Anchored" links as Pinned (mer3ly Rulings
  103-104).
- **One arrangement catalog** (stack seams S1). The authoring proof's
  hand-mapped `grid.default` and `scatter.default` were superseded:
  `projection_compile.rs` re-exports `scenomise::projection`, and P1's
  built-in catalog with host `ItemSizes` landed at `1633be0c`.
- **Terminology** (stack seams S5 and S6). "Arrangement" means positions
  only, a workbench arrangement is a "forme", and a physics world is a
  "world". Neither document used those words in the other senses, so nothing
  changed.

## Working agreements

- **The conatus session** ("Conatus, physics, seiche status") owns the
  dynamics grammar plan, the sibling grammar for motion. A ruling there that
  touches arrangements arrives as a request to carry it into the catalog and
  the adoption plan. This lane carried F18-F30 and the hold fixes that way.
- **The stack seams session** ("Graph database concept for Mere") runs the S14
  doc audit. It records findings in `support/doc-audit/d2/` and hands each one
  to its owning lane. This lane applied batch 43 at `18f4cb8c`.
- **Merging and pushing.** mere's main checkout is shared and usually holds
  other sessions' unpushed commits. Work in a worktree off origin and push your
  own commits as a fast-forward of origin, so you never publish theirs. Mark
  ruled this way for this lane's genet repin (mer3ly Ruling 116), and his
  standing push rules still apply.

## Traps already hit

- The weave merge driver can drop lines. After any merge that touches these
  docs, check that every line either side added is still present. This lane's
  check: for each parent, every added line must appear in the merged file.
- `scripts/mere_doc_audit.py` run from a worktree reports sibling-repo paths
  as missing, because they resolve under `worktrees/`. Compare per-file
  against a run from the main checkout before treating a finding as new.
- Concurrency (Mark, 2026-10-06): at most three tasks at a time, counting
  lanes, subagents and background jobs.
