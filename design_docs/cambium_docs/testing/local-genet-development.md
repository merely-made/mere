# Local Genet development

Cambium landed in mere on 2026-09-03 (the platform boundary plan's P2). Its
manifests now name the Genet seam crates through mere's root
`[workspace.dependencies]`, which pins `genet.git` at one revision for the
whole repository — not crates.io, and not a relative path into a sibling
checkout. The standalone Graphshell web host and Cambium web smoke also restate
these pins and the engine's vendored patches; update them in the same change.

To test unpublished Genet seam changes, redirect that git source to a local
Genet checkout in the uncommitted `.cargo/config.local.toml` at Mere's root;
copy `.cargo/config.toml.example` and edit from there. Invoke it explicitly
through `python scripts/cargo_mode.py local <cargo arguments>`, which uses a
separate `.cargo/local/Cargo.lock`. Ordinary Cargo must not inherit these
machine-local redirects. Two standing rules still apply:

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
ordinary Cargo from the owning workspace with redirect-free automatic configs
and inspect resolved package sources. `python scripts/cargo_mode.py verify --metadata-only`
checks that portable configuration and the tracked lock;
without `--metadata-only` it also checks all workspace targets. Running outside
the repository is not a substitute for inspecting inherited parent/Cargo-home
configuration. A local patch build cannot establish that committed Genet and
Netrender pins agree.

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

## 2026-09-28 published-source inline scroll adoption

The current primary worktree prepares Genet `7a60ad7965a1ae81292211b405a53210c554f70c`
in both the root and standalone Graphshell web manifests. Rootstock's new
formatting-line query consumption is covered by the bounded source controls
and status in [Genet compatibility](../technical_architecture/genet-compatibility.md).
The owning repair plan is `genet/design_docs/2026-09-25_line_box_model_plan.md`.
The revision is published by Genet; Mere's adoption remains uncommitted until
its remaining consumer checks and independent review complete.

Keep `RUSTUP_TOOLCHAIN=1.98.1`, `CARGO_TARGET_DIR=C:/t/cargo-targets/mere` and
`CARGO_BUILD_JOBS=4` explicit. The current bounded run also sets
`CARGO_INCREMENTAL=0` because disk headroom is limited. These environment
choices do not permit a different graph or weaker check. Root and web lock
candidates contain only the exact old-to-new Genet revision substitution;
locked metadata must accept them unchanged. The root graph retains the two
older Knot-owned Genet identities. Four copied-metadata controls verify that
both native and web source detectors reject unintended local paths and old
current-family revisions. Machine-local redirects remain outside the
published-source proof.

Do not confuse a long Git checkout preparation with failed resolution.
This revision contains 187,580 tracked files; Cargo's completed `.cargo-ok`
marker and subsequent locked metadata established completion. Retain attempts
and source provenance, and let a progressing preparation finish. Broader
compile gates are currently held for disk headroom; passing metadata and
Rootstock tests alone do not close them.

## 2026-09-29 continuation result and integration boundary

The five consumer checks previously held for disk space now pass with the
explicit environment above. The source and locks remain byte-identical to the
accepted restored Rootstock checkpoint. Exact cache restoration used a recorded
`cargo fetch --locked`; the failed initial offline attempt is preserved. The
stable target was recreated through ordinary builds after the user's cleanup.
Shared package-cache and target locks were allowed to finish normally.

A concurrent task briefly changed four shared source files during the first
workspace check, then restored them. That run remains qualified even though
its before/after hashes match and it exited successfully. Once it ended, the
restored files were checked against HEAD and the accepted hashes, and only
their mtimes were advanced. The accepted repeat is
`workspace-verify-restored-selector`; its log shows the affected crates being
checked again. This avoids treating final hashes alone as proof that source
was stable throughout a build. Details and the owner-reported interval are in
`restored-selector-invalidation.json`, including its provenance correction.

These receipts establish the bounded pre-integration adoption. Newer
`origin/main` contains separately owned web dependencies/features and runtime
changes. Keep this checkpoint intact, integrate those changes explicitly, and
establish fresh combined-graph and affected-test evidence before publication.
The earlier 527-package web graph cannot stand in for the combined graph.
Pre.4 and S13 remain gated independently.
