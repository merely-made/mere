# Cambium native hosts: incremental relayout adoption plan

**Date:** 2026-10-17. **Status:** proposal for owner ruling; no code changed.

Citations are `file:line`. Mere paths are relative to `crates/cambium/cambium-rootstock/src/`
(`mere:`), other Mere paths are repo-root-relative; Genet paths are relative to `components/genet-livery/src/` unless marked `genet:`.
Genet was read at `fa27ea5421c`; Mere pins `e37762d752e` (`Cargo.lock:6425`), which contains the
same retained path (`document/frame.rs` there references `layout_retained_formatting_root`).

## Verdict

1. Rootstock does a full restyle and a fresh layout for every drained mutation and never looks at
   the mutation list (`mere:frame.rs:335`, `:341`, `:359`; `mere:owned_layout.rs:156`, `:168`).
2. Genet has a complete retained pipeline, but only inside `LiveryDocument<D>`, which *owns* its
   DOM (`document.rs:185-190`). Rootstock's DOM is shared and borrowed through `WindowDom<'a>`
   (`mere:window_dom.rs:29-32`), so Rootstock cannot adopt it as is.
3. Even where the retained pipeline is admitted it is only about 2-3x faster, not the hundreds of
   times the owner wants, and it has one correctness bug and one coverage hole (measured below).
   Adoption therefore needs Genet work as well as Rootstock work.

## Answers

**1. Does any host drive retained layout? What would Rootstock call?**

- `genet-winit-host` does not. It is presentation plumbing only: surface, renderer, wheel and
  AccessKit (`genet:components/genet-winit-host/Cargo.toml:10`).
- The retained path is driven by `LiveryDocument<ScriptedDom>` in three Mere/Genet consumers:
  `genet:components/genet-documents/src/engines/livery.rs:434` and `:598`,
  `ports/pelt/desktop/frisket_surface.rs:741`, and
  `crates/canvas/pictograph/src/canvas.rs:407`.
- The entry points are `LiveryDocument::apply_dom_mutations(&[DomMutation])` (`document.rs:377`) or
  `mutate_dom` (`:399`), then `frame(w, h)` (`document/frame.rs:301`). `frame` tries, in order:
  repaint-only for positioned motion (`:354-360`), repaint-only for `background-color` (`:462`),
  one selected-root formatter (`:475-545`), and finally `layout_with_text_system` (`:553`).
- Rootstock cannot call `LiveryDocument` because of ownership (verdict 2). The recommended seam is
  to extract the retained state (`LayoutState`, damage, `IncrementalStyle`, identity source,
  transitions, text system; `document.rs:92-98`, `:185-221`) into a DOM-borrowing type whose
  `frame` takes `&D` per call, leaving `LiveryDocument` as a thin owner. Rootstock's
  `OwnedLayout` would hold that type and pass its `WindowDom` view (`mere:frame.rs:336-339`).

**2. Can the drained `DomMutation`s feed the damage record directly?**

Yes, with no translation. `record_dom_layout_damage` takes `&[DomMutation<NodeId>]`
(`document/frame.rs:46`), the same type Rootstock drains (`mere:frame.rs:334-335`), and maps each
mutation to its nearest formatting context via `formatting_damage_root` (`:140-165`). Caveats:

- The selected-root formatter runs only for a single damage root (`document/frame.rs:475-481`).
  Several changed rows in one frame fall back to full layout (a multi-root splice exists only as
  post-layout publication, `:566-577`).
- Multi-window: `MutationRouter` files each mutation under its window and sends unplaceable ones to
  every window (`mere:multi_host.rs:228-236`, `:326-327`). That is conservative and gives at worst
  a full-document damage.
- Interaction, stylesheet, resource and viewport events are full-document by design; their receipts
  are listed as missing in the K5h plan (`genet:docs/2026-08-11_buckram_k5h_dirty_root_execution_plan.md`,
  damage-class matrix). Rootstock's hover and focus path is one of them (`mere:owned_layout/interaction.rs:82-88`).

**3. Is style resolution incremental since the Stylo fork was deleted?**

Yes, in Livery's own `IncrementalStyle` (`invalidation.rs:65-299`). It snapshots changed
attributes, builds restyle hints, coalesces roots and re-cascades only those subtrees. It consumes
the same `DomMutation` batch (`invalidation.rs:116-124`), and it is public and DOM-borrowing, so
Rootstock could use it without the larger extraction. It falls back to full on a stylesheet or
device change (`invalidation.rs:136-153`). Differential tests exist
(`genet:components/genet-livery/tests/invalidation.rs`, 5 tests). The 2026-05-25 "fine-grained
Stylo restyle" plan (`genet:docs/2026-05-25_fine_grained_restyle_plan.md:1-8`) describes the
deleted Stylo machinery and is stale; `IncrementalStyle` is the plan of record.
Residual linear work remains (see below): `resolve_counters` over the whole plane
(`invalidation.rs:282`) and a whole-plane clone and `ch` pass in `frame` (`document/frame.rs:333-342`,
`layout.rs:1057-1059`).

## Measurements (2026-10-17, release, macOS, no fonts registered)

Keyed list, one row = row `div` (flex) + 4 cell `div`s + 3 text nodes = 8 nodes. One text node in
the middle row changes. "Rootstock-equivalent" is `resolve_styles` + `layout_with_text_system`
over the final DOM (what `OwnedLayout::rebuild` does). The retained numbers are
`LiveryDocument::frame` after `mutate_dom`. Source and raw output are in the session notes folder
(`livery_document_spike.rs.txt`, `spike_results.txt`); the probe was deleted from the repo.

| Rows | Rootstock-equivalent | Retained, block cells (not admitted) | Retained, flex cells (admitted) | Retained phases, flex (style / layout / paint) | Meristem diff |
|---|---|---|---|---|---|
| 100 | 69 ms | 57-61 ms | 26-28 ms | 4.7 / 9.1 / 13.6 ms | 0.15 ms |
| 1,000 | 685 ms | 668-697 ms | 300-311 ms | 52 / 124 / 126 ms | 1.6 ms |
| 5,000 | 3,363 ms | 4,546-4,620 ms | 2,342-2,388 ms | 450 / 1,268 / 618 ms | 10.5 ms |

Findings:

- **Coverage hole.** A plain text change under a block container (the common Cambium leaf) takes
  *no* retained path: 0 selected-root relayouts, and the timing equals a fresh layout. The text's
  nearest formatting context is `Inline`; `retained_root_box` accepts only Block, Flex and Grid
  (`layout/retained.rs:287-299`), and the frame loop breaks on `Unsupported` instead of promoting
  to the parent (`document/frame.rs:545`; promotion only happens from the formatter,
  `:539-544`). Admitted: text directly in a flex cell, or directly in the flex row. Rejected: text
  in a block cell, or in a block inside a flex cell. Confirmed with a temporary in-crate test using
  `retained_root_relayout_generation`, since reverted.
- **Correctness bug.** When the selected-root formatter *is* admitted, the retained paint list
  differs from a fresh document: the changed text is painted at the local origin (`y=0`) instead of
  its row (`y=1000` for row 50 of 100), at 100 and 1,000 rows. The formatter translates fragments
  to the retained origin (`layout/retained.rs:151-158`) but the paint shows the text frame was not
  translated. The existing receipts miss it because their changed text sits at the origin. The
  K5h reconciliation says `TextFrame::translate_subtree` exists for the positioned routes
  (`genet:docs/2026-08-24_k5h_retained_text_frame_reconciliation.md:17-21`); that it is the missing
  call here is my hypothesis, not verified.
- **Remaining linear cost.** Even admitted, the frame is O(N): the style phase does about 90 us per
  row with zero restyled elements, layout rebuilds the whole box tree (`layout/retained.rs:37`) and
  clones the previous styles and fragments (`document/frame.rs:487-488`), and paint re-emits the
  whole list (`document/frame.rs:599`). Incremental layout alone will not reach the diff's cost.
  Whether Rootstock's own paint (`mere:owned_layout.rs:649`) is also linear is unmeasured.
- The spike built Genet's working tree, which had another session's uncommitted
  `genet-scripted-dom` edits; they do not touch layout or style, but the numbers are not from a
  clean checkout.

## Adoption plan

Every phase lands only if its two gates pass. **Equivalence gate (E):** after each mutation the
retained result must equal a fresh layout of the same final DOM: paint-list commands, content
extent, hit-test of every row's rect, and text-find rects. This reuses Genet's pattern
(`retained_mutation_paints_like_a_fresh_final_document`, K5h plan) and compares an
`OwnedLayout` built by `new` against one updated incrementally. Cover: text change, attribute
change, row insert, row remove, row move, and two rows changed in one frame, at 100 and 1,000 rows,
with and without a scrolled viewport. **Timing gate (T):** rerun the 100/1,000/5,000-row keyed list
(recreate the probe under the scratchpad or as an untracked example, delete it afterwards), record
`stage_timings` (`mere:owned_layout.rs:190-196`) and total, one changed text node.

- **P0 (Genet, fix first): correctness and coverage.** Translate the local text frame in the
  selected-root path; add a failing receipt with the changed row away from the origin. Make an
  `Inline`-context damage root promote to its nearest Block/Flex/Grid ancestor instead of breaking.
  Gates: E on Genet's own suite plus the new receipts (not Rootstock yet); T on the
  `LiveryDocument` probe: block-cell 1,000 rows must beat the current 668 ms by a wide margin.
- **P1 (Rootstock only): incremental style.** Replace `resolve_styles` in `rebuild`
  (`mere:owned_layout.rs:156`) with `IncrementalStyle::update`, feeding the drained mutations
  through `relayout` instead of discarding them (`mere:frame.rs:335`). Full on stylesheet change,
  including the titlebar sheet (`mere:frame.rs:328-332`, `:340`). Gates: E on resolved styles
  (style plane equals `resolve_styles`) plus layout equality; T expects the style stage to fall
  from 164 ms to near the residual at 1,000 rows. Honest expectation: about 25 percent of the
  frame, because layout stays full.
- **P2 (Genet): borrowed-DOM retained session.** Extract the retained state machine from
  `LiveryDocument` so `frame` takes `&D`; `LiveryDocument` keeps its API. Moves with it: fonts and
  images (`document/resources.rs:22-76` versus `mere:owned_layout.rs:104-120` font registration),
  element and viewport scroll (`nested_scroll`, `scroll`, versus `mere:owned_layout.rs:240-262`),
  interaction states, and `caret_into_view`. Gate: zero behavior change in Genet's suite, Pelt,
  and Pictograph.
- **P3 (Rootstock): switch `OwnedLayout` to the P2 session.** Route `rebuild` through
  `apply_dom_mutations` and `frame`; keep `new` for size, sheet or font-ledger change. Gates: E
  as above across all of Rootstock's layout tests (`mere:owned_layout/tests.rs`,
  `hidden_panel_tests.rs`, `text_paint_tests.rs`); T must show the 1,000- and 5,000-row single-text
  case below the proposed bar. Proposed bar for the owner to set: 5,000 rows under 50 ms, and
  5,000-row time under 2x the 1,000-row time (sublinear).
- **P4 (Genet): remove residual O(N).** In priority order by measured share: whole-plane style
  passes in `frame` (450 ms at 5,000), box-tree regeneration and clones in the local formatter,
  and paint-list reuse for unchanged regions. A multi-root local formatter is needed for Meristem
  batches that change several rows. Hover/focus: map `set_interaction` to the existing
  `background-color`-only path (`document/frame.rs:462`) rather than `layout_resolved`
  (`mere:owned_layout/interaction.rs:82-88`); needs its own E receipt since K5h marks interaction
  events unproven.

## Decisions for the owner

1. Approve the Genet extraction (P2) as the route, versus owning a Rootstock-side copy of the
   retained logic. A copy would diverge from Pelt and Pictograph.
2. Set the T bar for P3 (my proposal above is a placeholder).
3. Order: P0 is independent and fixes a real divergence; P1 is the only phase that is Rootstock-only
   and can land now for a modest gain.
