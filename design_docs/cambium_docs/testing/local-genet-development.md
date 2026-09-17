# Local Genet development

Cambium landed in mere on 2026-09-03 (the platform boundary plan's P2). Its
manifests now name the Genet seam crates through mere's root
`[workspace.dependencies]`, which pins `genet.git` at one revision for the
whole repository — not crates.io, and not a relative path into a sibling
checkout. The standalone Graphshell web host and Cambium web smoke also restate
these pins and the engine's vendored patches; update them in the same change.

To test unpublished Genet seam changes, redirect that git source to a local
Genet checkout in the uncommitted `.cargo/config.toml` at mere's root; copy
`.cargo/config.toml.example` and edit from there. Two standing rules that file
records, both learned the hard way:

- a patch table that redirects a git source must name **every** package the
  graph pulls from it, or half the family resolves from the git checkout and
  the other half from the working copy, and one crate present twice is a type
  error rather than a resolution failure; and
- no workspace member may appear in it. Patching a git or crates.io source at
  a path that is also a workspace member is a hard lockfile collision, so
  `cambium`, `sprigging`, `workbench`, `mere-surface-api` and the rest of the
  family that landed with them are permanently out of that table.

A change that requires the redirect is ready to publish only after the matching
Genet seam release is on the pinned revision.

The crates.io `parley` patch must also point at the selected Genet checkout's
`support/patches/parley` during local engine work. A Git-source redirect alone
does not replace this separate patch entry. For shipping verification, invoke
Cargo from outside the repository with `--manifest-path` so its ignored local
config is not inherited, and inspect resolved package sources. A successful local
patch build cannot establish that the committed Genet and Netrender pins agree.

## 2026-09-16 — netrender T4 receipt: the retained-fragment path is dead code in Cambium

A local-patch build against netrender `aba7d837b` (T4 `06f3a12f4` plus the
filter-slot fix) and genet `f15547d8a34`, driven from a headless-GPU Cambium
host (`crates/cambium/cambium-genet-winit-host/examples/t4_receipt.rs`),
found that **`cambium-rootstock`'s retained-fragment placement is wired to
never fire.** `frame.rs::emit_scene` passes `|_| None` as the fragment
lookup into `layout.emit_paint_list_with_leaves`, so
`genet-livery`'s `PaintCmd::PlaceRetainedFragment` is never emitted for any
custom leaf, in any layer, on any frame — `self.s.leaf_fragments` (written by
`sync_leaf_fragments`) is read nowhere else in the crate. Confirmed
empirically: `fragment_lower_count()` stayed `Some(0)` across every captured
frame of the receipt despite `FrameProfile::leaf_repaints` confirming the
leaf's content really painted. This means netrender's T4 (retained
placements inside layer scopes) and its filter-texture-slot follow-up cannot
be exercised from a Cambium document today, independent of netrender's own
revision — the receipt is a Cambium-side gap, not a netrender finding, and
fixing `emit_scene`'s fragment closure is out of scope for a receipt (it is
a change to Rootstock's fragment logic).

Separately, genet has no CSS `filter` / `backdrop-filter` property at all
(`grep -rn FilterOp` across `genet` outside `paint_list_api` itself returns
nothing), so even with placement wired, the filter-layer case has no way to
be constructed from a Cambium document; only `overflow:hidden` (clip) and
`opacity` currently lower to a netrender layer.

Receipt directory: `C:\Users\mark_\Code\testing\mere\cambium_t4_receipt_20260916\`
(`results.md`, `results_raw.md`, readback PNGs). Full source citations:
netrender's `netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`,
"Retained placements inside layer scopes" and "Filter texture slots: key/handle
reuse".
