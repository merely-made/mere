# Batch 56: the app composition brief (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| cambium_docs/research/2026-10-06_app_composition_brief.md | current | yes | 18 | 16 | 0 | 2 |
| **Totals** |  |  | **18** | **16** | **0** | **2** |

**Totals: 1 doc, 18 claims checked (16 holds, 0 stale, 2 unverifiable), 0 contradictions; 0 status lines wrong.**

Audit base: Mere `362c5d5a` with the brief in the working tree (2026-10-06),
Turnstone's committed tree at `032463ae`, Knot `210ee64b`, Woodshed
`cefc903d`, wgpu-graft `95bab7e5`, wgpu-weld `b5cf0435`, netrender `0edc6077`
and Genet `90c5ef50`. The read-only lane that wrote the brief judged it the
same day; an independent re-judgment can supersede this record under ruling
S34.

## cambium_docs/research/2026-10-06_app_composition_brief.md

- disposition: current
- status line: "**Status (2026-10-06):** research. Nothing in it is authorised to build. A read-only lane wrote it under fork U8 of Turnstone's unusual-protocols browser plan; no code changed and nothing was built or run. The forks in §9 are with Mark." — accurate: yes
- claims checked: 18 — holds: 16, stale: 0, unverifiable: 2

### Stale claims

None.

### Contradictions

None. The brief places a Cambium guest in tree-level composition, which agrees
with the 2026-06-19 native-surface compositing plan's split of host surfaces by
nature (genet-rendered content as a DOM subtree, external content as a
texture). It records that Turnstone's §4 table names "only D3D12 shared handles
exist, from Weld and Scry" and adds the Linux and macOS import halves in
wgpu-graft, which narrows rather than contradicts that line.

### Recommended action

Re-judge when Mark rules AC1 to AC6, or when E1 runs, since either changes §7
to §9. If AccessKit is repinned, re-check F9's adapter-to-consumer versions.

### Notes

Each claim below was checked against the source in brackets, and holds:
- Turnstone does not depend on `cambium-rootstock` or
  `cambium-genet-winit-host`, and places one composited surface per pane
  [Turnstone `Cargo.toml` at `032463ae`; `src/panes/mod.rs` line 17;
  `src/shell/render.rs` lines 855 to 993];
- the contributed-surface seam, its four providers, and Knot's and Redshank's
  sessions [`src/contributed_surface.rs`; `src/knot_document_surface.rs`;
  `knot-editor/crates/knot-document/src/document_view.rs` lines 179 to 195;
  `woodshed/ports/redshank/surfaces/src/surface_api.rs` lines 1 to 15];
- Turnstone's chrome as a forest [`src/chrome_view.rs` lines 7 to 19 and 649
  to 651];
- `RetainedSurfaceSession`'s v1 freeze, its excluded duties and its event set
  [`crates/cambium/cambium/src/surface.rs` lines 7 to 75];
- traversal wrapping inside a runner [`crates/cambium/cambium/src/runner.rs`
  around line 567];
- the forest as one state over one view type, with rebuilds of every other
  projection [`crates/cambium/cambium/src/multi.rs` lines 78 to 95 and 212 to
  226]; `MultiHost`, its router and `WindowDom`
  [`crates/cambium/cambium-rootstock/src/multi_host.rs`;
  `window_dom.rs` lines 20 to 27]; P2 on main at `40d7ae5e` and rulings S24,
  S28, S29 [stack seams plan];
- Knot and Woodshed on the single-window `run`
  [`knot-editor/apps/desktop/src/lib.rs` line 32;
  `woodshed/crates/woodshed-genet/src/main.rs`];
- Meristem's `lens` and `map_state` [`crates/cambium/meristem/src/views/`];
- the producer's same-device context and flat semantics
  [`crates/cambium/cambium-rootstock/src/producer.rs` lines 68 to 203;
  `crates/cambium/cambium-winit-a11y/src/lib.rs` lines 290 to 351];
- the per-OS GPU-sharing parts and the absence of a handle broker between two
  of the family's own processes [wgpu-graft `grafting/src/` files named in
  F6; `wgpu-weld/welding/src/windows_cef/mod.rs` line 12;
  `genet/components/genet-compositor/interop/windows_dx12.rs`; a grep for
  `DuplicateHandle`, `SCM_RIGHTS` and Mach-port calls];
- the Scry receipt's stale and blank captures and supplier defect
  [`turnstone/docs/receipts/browser_scry_windows_20261005/README.md`];
- `PaintEnvelope` as an IPC wire form [`netrender/paint_list_api/src/lib.rs`
  lines 307 to 341];
- Knot authoring's split of presentation and authority
  [`turnstone/src/knot_authoring.rs` lines 7 to 12];
- AccessKit 0.24.1's subtrees, and graft support in the consumers under each
  pinned adapter [`accesskit-0.24.1/src/lib.rs`; the `Cargo.toml` of
  `accesskit_windows` 0.32.1, `accesskit_atspi_common` 0.18.1 and
  `accesskit_macos` 0.26.3; `accesskit_consumer` 0.35.0, 0.36.0 and 0.38.0
  `tree.rs` and `node.rs`];
- no adapter accepting another process's tree, and the stack publishing only
  the root tree [`accesskit_windows-0.32.1/src/node.rs` line 1062;
  `accesskit_unix-0.21.1/src/atspi/bus.rs` line 81; a grep for `TreeId` in
  Mere's crates, Genet's components and Turnstone's source];
- Narrator's one walk and the open gate for contributed panes
  [`turnstone/design_docs/2026-08-20_screen_reader_pass_receipt.md`;
  `turnstone/design_docs/2026-07-17_genet_probe_automatability_plan.md` around
  line 310];
- the five Mere revisions pinned across the heads [each repository's
  `Cargo.toml` at the commits above, Redshank's nested one included];
- Mark's rulings U8 and U2 (second round), quoted verbatim [Turnstone plan at
  `032463ae`].

Unverifiable:
- that the pinned adapters apply a graft node's geometry and focus correctly
  on Windows, macOS and Linux. The brief says this was read from source and
  never run; E1 is the check.
- the size estimates for B3 and A1 (§5), which the brief marks as readings.
  No document records a per-process memory figure, as F12 says.
