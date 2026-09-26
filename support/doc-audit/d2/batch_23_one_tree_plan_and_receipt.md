# Batch 23 — one-tree plan and the mere view's headed receipt (new documents)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-09-25_graphshell_one_tree_plan.md | current | yes | 52 | 47 | 0 | 5 |
| mere_docs/testing/2026-09-25_mere_view_headed_receipt.md | historical-marked | yes | 35 | 31 | 0 | 4 |
| **Totals** |  |  | **87** | **78** | **0** | **9** |

**Totals: 2 docs, 87 claims checked (78 holds, 0 stale, 9 unverifiable), 0 contradictions. Five claims were stale at the audit base and are corrected in this pass.**

Audit base: Mere `1da2a226` (2026-09-26), with this pass's corrections in the
working tree. The plan's findings were checked at the commits that wrote them:
`a064e25f` for those of 2026-09-25, and `cde489c2` and `1da2a226` for later
ones. The receipt was checked at the commits it names (`5c1905b3`, `53f625fc`,
`065c2336`) and against the captures, receipts and host logs under
`Code/testing/mere/scenarios/mere-view/` and `mere-view-at-5c1905b3/`. Woodshed
(`d721a80`), genet (`1b62fd0b218`), knot-editor (`5ad3f67`), isometry
(`d2f2061`) and netrender (`aba7d837`) were read in their local checkouts the
same day. `archive_docs/` is excluded.

This batch exists because both documents are new and had no record. They come
from the reservoir plan's V2b work, and Mark asked on 2026-09-26 for this
workstream's two to be covered.

## mere_docs/implementation_strategy/2026-09-25_graphshell_one_tree_plan.md

- disposition: current
- status line: "Status: in progress, ruled 2026-09-25 (reservoir plan §7 items 39 and 40). Phases 1 and 2, accessibility in the browser and the file seam, were done on 2026-09-26; phase 3, the canvas as a producer, is next." — accurate: yes
- claims checked: 52 — holds: 47, stale: 0, unverifiable: 5

### Stale claims

- none remaining. Four were stale at `1da2a226` and are corrected in this pass:
  - Related called `a11y.rs` "the module doc that records the web host's
    accessibility gap"; phase 1 rewrote that doc to describe the mirror —
    evidence: `crates/cambium/cambium-genet-web-host/src/a11y.rs:1`.
  - "holds about 100 controls": `component.html` held 89 form controls (45
    buttons, 28 inputs, 13 selects, 3 textareas) and one link at `a064e25f` —
    evidence: a tag count over
    `git show a064e25f:ports/graphshell/web/component.html`.
  - "an aside of six sections … and the node detail": the aside
    (`gs-product-panel`) holds five sections, and the node detail is
    `gs-detail-surface`, after the aside closes — evidence: `component.html`
    lines 39, 262 and 265 at `a064e25f`. §3's "the six sections" is reworded
    with it.
  - "Pelt's desktop workspace viewer and the scrying engine use"
    `TextureProducer`: both implement inker's `SurfaceProducer`
    (`crates/inker/inker/src/surface_engine.rs:970`); at `a064e25f` no code
    outside rootstock and the winit host names `TextureProducer`, and
    `git log -S TextureProducer -- ports/pelt crates/inker` finds nothing. Its
    users are isometry's `shared/isomere/src/host.rs`,
    `shared/isometer/src/producer.rs` and eponym's client.

### Contradictions

- none. Two findings of 2026-09-25, that the web host projects no
  accessibility and has no file seam, describe the state phases 1 and 2 then
  changed; §6 records both changes.

### Recommended action

- none after this pass's corrections.

### Notes

The claims checked:

- **Status, links and rulings**, 10 claims:
  - the status line against `c19e1120`, `685c830e`, `721441ff` and
    `1da2a226`, and phase 3's not having started (no `TextureProducer` in
    `ports/graphshell`);
  - the reservoir plan's V2b step 4 waiting on this plan, and §7 items 39 and
    40 recording "One tree first" and "Enablers first";
  - the `a11y.rs` entry (corrected), and the platform boundary plan's
    `p2_cambium_h3_boot` receipt showing the chrome and a scene through Genet
    on the web target;
  - the phase 1 rulings: `document_a11y_projection`
    (`components/genet-render/src/a11y.rs:787` at genet `1b62fd0b218`)
    lowered to ARIA, with leaves bridged to the Graphics Module roles
    (`mirror.rs` lines 61 to 63); focus following the tree; the pin bumps; the
    text-field fix and its test (`a11y.rs:919` at genet `1b62fd0b218`);
  - phase 2's "Both hosts now", met by the winit host installing
    `DialogFileChooser`.
- **Findings of 2026-09-25**, 21 claims, at `a064e25f`:
  - the line counts: `web_gpu.rs` 300, `web_view.rs` 353, `component.html`
    337, `web.rs` 2,248 and `web_scenario.rs` 768, all exact;
  - `GpuPresenter`; the chrome's `ChromeModel` and `ScriptedDom`; the content
    scene in `web.rs`;
  - the page's controls and aside (both corrected), its mounted-sessions
    header, the graph bar's select, edit, pan and zoom, and the projection
    editor's seven tabs;
  - `mount` and its callers, `crates/woodshed-web/src/lib.rs:78` and
    `ports/redshank/web/src/lib.rs:633` in woodshed;
  - the accessibility gap as `a11y.rs`'s module doc then recorded it;
    `cambium-winit-a11y`'s single 336-line file with `project_tree` and
    `TreeUpdate`; rootstock's `Accessibility` trait (`lib.rs:243`);
  - `#gs-file-input`;
  - `TextureProducer` (`cambium-rootstock/src/producer.rs:86`) and its users
    (corrected); rootstock's `redraw` (`frame.rs:492`), called by the web
    host's frame loop (`mount.rs:174`);
  - the lane's `dom click`, `type`, `assert dom` and `assert attr`, and its
    module doc on the generic `click` missing (`web_scenario.rs:39`);
  - 35 `.scn` files under `ports/graphshell/web/scenarios/`, and 29 run
    directories under `testing/mere/scenarios/graphshell-web/`;
  - the three Genet traits and the `:root` rule, as the receipt records them.
- **Later findings**, 7 claims: the mirror written at mount and on a reader's
  action (`mount.rs`); Cambium's checkbox named "Checkbox" by default
  (`cambium/src/controls/toggle.rs:36` at `cde489c2`); woodshed-web's
  three-field `Init` at its mere `691f9a0b` pin (woodshed `a0910d5^`); all five
  pages loading `loader.js`; the web host's frame loop on
  `requestAnimationFrame` (`schedule_frames`); and two unverifiable runtime
  observations, the hidden Browser pane's WebGPU, frame and focus behaviour
  and the hidden Chrome tab's frames and capture timeout.
- **Progress**, 14 claims:
  - phase 1's commits, all ancestors of `1da2a226`; `document_projection`,
    the mirror, one element per node, the `aria-hidden` canvas and the named
    region (`a11y.rs` lines 128, 129 and 145); the `a11y_page` example; the
    mirror's six tests;
  - woodshed `a0910d5` and `d721a80` moving to mere `149b8053` and genet
    `1b62fd0b218`; Redshank's bump touching only its manifests and lock;
    woodshed's four changes (the `=0.1.1` requirements, the Parley row, the
    `taproot` rename, `Init`'s `fonts` and `images`); the pages under
    `Code/testing/cambium/`;
  - phase 2's `open_file`, `FileRequest` and `FileEvent`; rootstock's
    `FileChooser`, delivery at the start of a frame, the empty answer with no
    chooser, and `AppCtx.files`; `WebFileChooser`; `DialogFileChooser` on
    `light-file-dialog`, which `ports/graphshell/Cargo.toml` already uses; the
    lane's `file` step; four `files` tests and three new scenario tests;
  - three unverifiable runtime results: phase 1's Browser pane session, the
    bumped pages in the pane, and phase 2's Chrome run.

The five unverifiable claims are observations of running pages. The code each
depends on was checked.

## mere_docs/testing/2026-09-25_mere_view_headed_receipt.md

- disposition: historical-marked
- status line: "Result: at mere `5c1905b3`, the mere view passes nine scenarios, one for each of Knot's requirements, headed on the winit host. Each scenario runs at the full centre (1,100 by 700) and in a 280 px side tile, and every frame was reviewed whole. This is step 2 of V2b in the reservoir plan." — accurate: yes
- claims checked: 35 — holds: 31, stale: 0, unverifiable: 4

### Stale claims

- none remaining. One was stale and is corrected in this pass: "AccessKit
  projected the tree (65 nodes, from the host's log)". Every `stdout.txt` of
  both runs, under `Code/testing/mere/scenarios/mere-view/` and
  `mere-view-at-5c1905b3/`, reads "accessibility Installed, 67 nodes
  projected".

### Contradictions

- none. The Boundary's "the browser host projects no accessibility yet" was
  true on the receipt's date; the one-tree plan records phase 1 building it on
  2026-09-26.

### Recommended action

- none after this pass's correction.

### Notes

The claims checked:

- **The run**, 14 claims:
  - the result: 9 `RESULT ok` receipts at each commit; the centre and side
    sizes, captured at twice scale as 2,200 by 1,400 and 560 by 1,400;
  - the harness (`crates/cambium/mere-view/examples/harness.rs` at
    `5c1905b3`): twelve documents, fourteen relations with three authored and
    two suggested, three sessions with one trashed, New, Open and Recent, a
    theme of custom properties, the winit host's lane through
    `LaneConfig::from_env("MERE_VIEW")`, and the `key` verb;
  - the run script, the capture folders, the nine scenario files and 21
    named captures; 21 frames, none blank and all distinct in each run;
    thirteen unit tests in `src/tests.rs`; `tests/host_routing.rs`.
- **The fixes**, 9 claims:
  - at `53f625fc`: node targets above relation cells (`graph_canvas.rs:86`);
    sprigging's `RelationLine` with solid, heavy, dashed and dotted
    (`glyphs.rs:42`), and kinds mapped to lines (`graph_canvas.rs:359`);
    labels anchored and placed on the side with room (`graph_canvas.rs:928`,
    `:658`); opt-in culling that places the selected, focused and hovered
    labels first (`graph_canvas.rs:246`, `:587`);
  - at `5c1905b3`: `SEPARATION` of 44 (`layout.rs:28`); the narrow bar's two
    rows and minting at the head of the sessions list (`view.rs:29`, `:407`);
    the notice as a banner over the graph (`view.rs:55`); the wrapper without
    a clip (`view.rs:42`); theming through `:root`; netrender `aba7d837`
    exists.
- **The re-run**, 2 claims: `065c2336` after genet `6afb472a0c6` and
  `18e41e44c36`, with nine passes and 21 captures; the frames at `5c1905b3`
  kept in their own folder.
- **The boundary**, 6 claims: the one-tree plan link; Knot's step 8
  re-scoped to embed the view
  (`knot-editor/design_docs/2026-09-23_knot_workspace_slice1_plan.md`); the
  node count (corrected); 5.6 px per character at the labels' 10 px
  (`graph_canvas.rs:906`, `:72`); the focused Archive 2025 label crossing
  Index's dot, seen in `r4_side.png`; mesquite's lane beside the winit
  host's, which Knot uses (`apps/desktop/src/lib.rs:80`).
- **Unverifiable**, 4 claims: Genet drawing nothing for `::before` and
  `::after`, Genet ignoring `text-align` in the fixed-width box, the clipping
  wrapper blanking the frame at netrender `aba7d837`, and how each frame
  changed on the re-run. These are renderer behaviour seen in captures; the
  fixes were checked.
