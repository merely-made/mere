# The mere view, proved headed

**Date:** 2026-09-25
**Result:** at mere `5c1905b3`, the mere view passes nine scenarios, one for
each of Knot's requirements, headed on the winit host. Each scenario runs at
the full centre (1,100 by 700) and in a 280 px side tile, and every frame was
reviewed whole. This is step 2 of V2b in the
[reservoir plan](../implementation_strategy/2026-09-23_reservoir_plan.md).

## How it runs

`crates/cambium/mere-view/examples/harness.rs` is a host of its own around the
view. It holds:
- a catalog like Knot's: twelve documents, with links read out of them, three
  authored relations and two suggestions;
- three sessions, one of them in the trash;
- New, Open and Recent in the action slot;
- a theme of custom properties.

It carries out what the view asks, declines what it cannot do and gives its
reason. It drives the scenario lane of `cambium-genet-winit-host`, and adds a
`key` verb that travels the runner's own keyboard path.

```text
bash Code/testing/mere/scripts/run-mere-view-scenarios.sh      # all nine
MERE_VIEW_SCENARIO=crates/cambium/mere-view/examples/scenarios/<name>.scn \
MERE_VIEW_CAPTURE_DIR=<dir> MERE_VIEW_RECEIPT=<dir>/receipt.txt \
  cargo run -p mere-view --example harness                     # one
```

The captures and receipts are under
`Code/testing/mere/scenarios/mere-view/<scenario>/`.

## Evidence

| Requirement | Scenario | What it shows | Captures |
|---|---|---|---|
| 1. Sizes to its tile | `r1_tile` | The graph takes 880 by 660 at the centre and 280 by 381 in the side tile, with all twelve node targets. | `r1_centre`, `r1_side` |
| 2. The host acts | `r2_host_acts` | A node press asks the host to open it. A declined trash and a declined fork show the host's reason in a banner over the graph. An unavailable document is refused in words. | `r2_centre_declined`, `r2_side_declined` |
| 3. Action slot | `r3_action_slot` | New, Open and Recent reach the host at both sizes. | `r3_centre`, `r3_side` |
| 4. States in words | `r4_node_states` | "unavailable", "unsaved" and "open" appear in labels. In the side tile, the label culling left out returns when its node is focused. | `r4_centre`, `r4_side` |
| 5. Provenance | `r5_provenance` | Fourteen relations make fourteen cells, two of them on one pair. Links are plain lines, authored relations heavy and suggestions dashed. | `r5_centre`, `r5_side` |
| 6. Layout preference | `r6_layout` | Spectral, then Grid, then Spiral move every node and keep every key. | `r6_centre_spectral`, `r6_centre_grid`, `r6_side_spiral` |
| 7. Keyboard targets | `r7_keyboard` | Tab reaches a node by its `data-key`, and Enter or Space opens it, with the focus ring shown. | `r7_centre_focused`, `r7_side_focused` |
| 8. Plain states | `r8_states` | Empty, unavailable and building each say so, and each carries the host's action. | `r8_centre_empty`, `r8_centre_unavailable`, `r8_side_building` |
| 9. Host theming | `r9_theming` | Swapping only the host's tokens changes the frame; the lane checks that the light and dark captures differ. | `r9_centre_light`, `r9_centre_dark`, `r9_side_dark` |

The run captured 21 frames, none blank, all distinct within each run. The
crate's thirteen unit tests cover the view tree. `tests/host_routing.rs`
presses nodes through the host's real routing, without a window.

## What the frames found

Each of these was fixed before this revision, by the commits named.

1. **A node press reached a relation cell.** A relation cell spans its whole
   segment, and its rotation makes it a stacking context, so it won the press
   at both of its endpoint nodes. `graph_canvas` now stacks node targets above
   the cells (`53f625fc`).
2. **Equal nodes landed on one point.** Spectral gives nodes with the same
   neighbours the same place. The view's layout now parts nodes to 44 px,
   the size of each node's target, clamping at the edges on every pass
   (`5c1905b3`).
3. **The narrow bar overflowed.** It now takes two rows, and minting moved to
   the head of the sessions list, beside the other lifecycle steps
   (`5c1905b3`).
4. **A refusal was half hidden.** The notice sat behind the minting button. It
   is now a banner over the top of the graph's area (`5c1905b3`).
5. **Provenance never showed.** Genet drew no content for the `::before` and
   `::after` rules on these cells. Relation kinds are now painted as lines:
   sprigging has solid, heavy, dashed and dotted, and `graph_canvas` maps kinds
   to lines (`53f625fc`).
6. **Labels drifted from their nodes.** Genet did not honour `text-align`
   inside the fixed-width box these absolutely placed spans had. A left label
   is now held by its right edge, and an above or below label is centred from
   its length. A label also takes the side with room for it (`53f625fc`).
7. **Labels crowded the side tile.** Opt-in culling in `graph_canvas` leaves
   out a label that would cover one already shown. The focused, hovered and
   selected nodes are placed first and always keep theirs (`53f625fc`).
8. **The dark theme left text black.** A Cambium tree has no `html` or
   `body`, so the harness's base colour never applied. A host themes through
   `:root`, which is also where the tokens live.
9. **A clipping wrapper blanked the whole frame.** A box with
   `overflow: hidden` held the graph, which clips itself, and the positioned
   notice. At the centre size the entire frame came out white. The wrapper now
   has no clip. This is a renderer fault at netrender `aba7d837`; it is
   avoided here, not fixed.

## Re-run on genet 18e41e44c36

At mere `065c2336`, after main's two genet repins were merged in
(`6afb472a0c6`, then `18e41e44c36`), all nine scenarios pass again with 21
captures. Every frame changed, and only where Genet now aligns a button on
its last line box. The bar and the sessions list are shorter by the strut's
descent, and in the side tile the graph sits about 9 px higher with the same
layout. In `r2_side_declined`, Session 4's steps, which the list's scroll
edge had cut off, now nearly all show. The frames at `5c1905b3` are kept
under `Code/testing/mere/scenarios/mere-view-at-5c1905b3/`.

## Boundary

- **Only the native winit host, on Windows at 2× scale.** The browser host
  projects no accessibility yet; that is phase 1 of the
  [one-tree plan](../implementation_strategy/2026-09-25_graphshell_one_tree_plan.md).
- **A catalog like Knot's, not Knot's.** Knot's step 8 re-proves the
  requirements in its own embedding.
- **Screen readers:** AccessKit projected the tree (65 nodes, from the host's
  log), but no reader was run.
- **Culling estimates widths:** 5.6 px per character at the labels' 10 px
  size.
- **A focused label can cover another node.** Culling weighs labels against
  labels, not against node dots. In `r4_side` the focused Archive 2025's label
  takes the left side and runs across Index's dot, at `5c1905b3` as on the
  re-run.
- **Two scenario lanes exist for one job:** `mesquite` and the winit host's.
  This uses the latter, as Knot does.
