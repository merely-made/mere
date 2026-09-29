# Genet compatibility

Cambium consumes Genet through published seam packages. `cambium-nematic`
adds reactive projections over Errand's portable smolweb ASTs. Genet's
engine-side Livery and Scripted document runtime remains in `genet-documents`;
Mere's application-facing reader and smolweb lanes live in
`mere-document-lanes`.

## Historical verified Genet seam set (2026-07-22)

Verified on 2026-07-22:

- `genet-scripted-dom = 0.1.0`
- `layout-dom-api = 0.1.0`
- `errand = 0.1.3`
- core provider release commit:
  `2e462fe8975`

| Package | Version | Current source |
| --- | --- | --- |
| `layout-dom-api` | 0.1.0 | crates.io and sibling path |
| `errand` | 0.1.3 | crates.io and sibling path |
| `genet-paint-types` | 0.1.0 | crates.io |
| `engine-observables-api` | 0.1.1 | crates.io |
| `genet-static-dom` | 0.1.0 | crates.io |
| `genet-scripted-dom` | 0.1.0 | crates.io and sibling path |

## Historical Cambium stack: source vs registry (updated 2026-08-20)

| Package | Workspace | crates.io | State |
| --- | --- | --- | --- |
| `meristem` | 0.2.0 | 0.1.1 | 0.2.0 pending publication; breaking scope cut |
| `sprigging` | 0.2.1 | 0.2.1 | current |
| `cambium` | 0.3.3 | 0.3.3 | current (published 2026-08-09; 0.3.2 lacked the IME composition surface) |
| `cambium-nematic` | 0.3.1 | 0.3.1 | current |
| `cambium-winit` | 0.3.0 | 0.3.0 | current (published 2026-08-09; 0.2.0 remains yanked) |
| `cambium-winit-a11y` | 0.3.0 | never published | `publish = false` by design |

The registry graph resolves as of 2026-08-09. The history, kept because it
explains the yank and the split: `cambium-winit` 0.3.0 became publishable on
2026-07-26, when the accessibility host moved out to `cambium-winit-a11y`
exactly because `genet-layout` and `genet-winit-host` inherit Genet's
`publish = false`. Publishing it then surfaced a second defect: the local
`cambium` had grown the IME composition surface after 0.3.2 shipped, so
package verification failed against the registry; `cambium` 0.3.3 publishes
that state under its own number and `cambium-winit` 0.3.0 requires it.
`cambium-winit-a11y` can never publish, and that is the reason it exists:
holding the genet-coupled half keeps `cambium-winit` down to `cambium` +
`winit`.

At that table's date, consumers followed a git-first family rule through
`genet.git`; the registry served external consumers. Cambium Nematic's release
boundary was Cambium plus the protocol AST package, without Genet's layout or
rendering engine.

## Current Mere-owned compatibility (2026-09-13)

Cambium is now a Mere workspace family under `crates/cambium/`; it is not a
Genet workspace subtree. Mere owns the current `cambium`, `meristem`,
`sprigging`, and `workbench` paths, while pinning Genet seam packages at one
immutable `genet.git` revision (`7baa554c66f966295ee945d908738b635afe136a`).
Errand is likewise Mere-owned at workspace version 0.3.4.

That pin moved from `101d9e9ade8671564e723443d9f0498e899a33f1` on 2026-09-14,
eight commits forward, to take `genet-probe`'s new name `taproot`. The
compatibility table below was not re-validated at the new revision; the
2026-09-13 receipt in the architecture doc still names the old one, which is
what it measured.

The external viewport bridge uses that revision's content-box geometry and typed
used-color queries. Genet's vendored Parley and Taffy patch entries carry the same
revision; upstream Parley 0.10 has the same version but lacks the engine's local
extensions. Netrender and the paint-list family align to
`3961aca919f707ab09a786379eb4ce8bb121258e`, matching Genet's own dependencies.
Mixing that revision with Mere's former Netrender pin produced distinct Rust
types when ignored sibling patches were absent. Root and standalone host manifests
must keep these families aligned.

| Boundary | Current source in this workspace | State |
| --- | --- | --- |
| Cambium family | `crates/cambium/` workspace members | Mere-owned |
| Genet DOM seams | `genet-scripted-dom`, `layout-dom-api`, and `genet-static-dom` pinned to the revision above | Genet dependency boundary |
| Errand | `crates/system/errand` 0.3.4 | Mere-owned protocol AST dependency |

Downstream consumers' pins and registry publication state need verification in
their own repositories or registries; this document records only the current
Mere workspace boundary.

## Custom-leaf protocol

Cambium emits Genet's neutral `<custom-leaf>` element and related attribute
vocabulary. Genet temporarily accepts `<chisel-leaf>` as a read-side
compatibility alias for older documents.

## Direction rule

Cambium may depend on Genet seam crates. Genet engine crates must remain free
of Cambium, Meristem, and Sprigging dependencies. Reference applications such as
Pelt may depend on all three.

## 2026-09-27 accepted text bounds publication

The owning Genet fix under Isocosm rulings 329/379 is accepted at exact commit
`7b48f94d7a742840b527205d82a37da958240d73`. It separates font-content bounds
from formatting line boxes and anchors wrapped inline decorations to their
own font/baseline. Main's retained motion and restyle preparation are included;
the combined owner receipt passes 876 affected tests (6 existing ignored),
200 boundary tests and a current-family local Isometry detector pair of
187/0 short rows against the published baseline's 187/39. That local pair was
not a committed portable consumer repin. Source and evidence qualifications
live in `genet/design_docs/2026-09-25_line_box_model_plan.md`.

Mere is preparing publication of that exact Genet commit in its 27 root and
7 standalone Graphshell web manifest rows. NetRender remains `9607d16`,
netrender-vello 0.10.1, wgpu 30 and the accepted pre.2 compute patches remain
unchanged. Direct Cargo uses published sources; inactive machine-local configs
are not supplied. Portable graph/lock classification and the bounded host,
Cambium, Sprigging and standalone-web checks are pending. The separately
pinned old Fleece/layout-dom-api lineage through Knot is pre-existing and
outside this pin's acceptance claim. Downstream Isometry publication follows
only after Mere's reviewed commit is pushed.

**Preparation note, 2026-09-28:** a long first metadata invocation was initially
misidentified as dependency-solving cost. The trace had reached patch
registration while the new Git checkout still lacked Cargo's `.cargo-ok`
marker. Its files carried the current attempt's timestamps; the prior cached
revision's manifest-to-marker interval was eight minutes and forty-five seconds.
That supports unfinished checkout preparation, not a dependency conflict.
Interrupted attempts remain recorded as interrupted preparation. Compare this
phase/progress before diagnosing a resolver failure; acceptance still requires
completed locked metadata and the consumer checks. The external candidate lock
is an unvalidated, source-backed input until Cargo accepts it unchanged.

**Publication gates, 2026-09-28:** Cargo 1.98.1 completed `metadata --offline
--locked` and accepted the external source-backed lock unchanged; that exact
lock is now the primary lock. The graph has 1,524 packages (previously 1,523),
including 24 current Genet packages. After the exact revision substitution,
the only node/feature/dependency-edge changes are the new `genet-text` 0.1.0
package and the source-declared `genet-documents` edge through it. The two
old `34626a6` Fleece/layout-dom-api packages remain. Both copied-metadata
controls reject an outside path and an old current-family Genet source.

The all-target checks for `cambium-genet-winit-host`, `cambium` and `sprigging`
pass using the primary published lock. Standalone Graphshell web's Wasm check
also passes with its separately validated ignored lock and only the required
`getrandom_backend="wasm_js"` target cfg. That lock has exactly the revision
substitution from the preserved prior portable lock; current unfiltered
metadata has 527 nodes and 18 Genet packages, with no documents/text package.
The historical provenance's 432-package count has no recorded filter/command
and is not reused as a current count. Native wgpu remains 30.0.1; the independent
web lock retains 30.0.0. Both retain NetRender `9607d16` and netrender-vello
0.10.1. Web source-detector controls also reject both deliberate faults.

Raw commands, compiler identity, original and accepted locks, source audits,
controls and logs are in `testing/mere/receipts/2026-09-27/genet-text-publication`
under the shared Code root. An initial direct-Cargo compile invocation selected
rustc 1.97.1 for dependencies and failed on existing 1.98.1 artifacts; the
retained successful commands explicitly select the repository's 1.98.1
toolchain throughout. No target cleanup or source adjustment was needed.
The pre-existing `reader.rs` bytes remain unchanged. These are native/Wasm
compile and dependency-source gates, not new headed, browser or WPT receipts.
Isometry's published-pin text-row gate remains downstream work.

## 2026-09-28 inline scroll bounds adoption (verification in progress)

Primary Mere is preparing exact Genet
`7a60ad7965a1ae81292211b405a53210c554f70c`, whose retained formatting-line
query repairs the scroll-range gap exposed after the text-bounds publication.
The owner design and repair evidence live in
`genet/design_docs/2026-09-25_line_box_model_plan.md`. This update is not yet
committed or accepted for publication. The separate pre.4 lane remains frozen
at held checkpoint `387a8dd2`; this work retains primary's pre.2 compute patches.

Rootstock now consumes `LiveryLayout::inline_scroll_bounds`: the container's
own line bounds, every retained fragment for a descendant, and line bounds
owned by unclipped descendants all contribute in layout coordinates. The
existing end padding/border arithmetic is preserved. A descendant's border
fragments contribute before its clipping boundary; its lines and deeper
descendants do not escape that boundary. The existing non-visible overflow
predicate is shared with `content_clip`, without mixing its painted rectangle
coordinates into this layout-space calculation.

The original hover test still requires a 12-pixel offset, now also asserted
before hover alongside a nonzero initial range. New tests cover own and
descendant line bounds, hidden/clip/auto/scroll descendants, and later font
fragments extending beyond short line boxes. The full Rootstock suite passes
44 tests. Four deliberate source controls separately omit container lines,
omit descendant lines, retain only the first fragment, and omit the clipping
prune; each compiles and fails its expected assertion. Exact source restoration
is followed by another 44-test pass. These are correctness results, not timing
or exclusive-GPU measurements.

The 27 root and seven standalone web manifest rows move together. Both locked
graphs accept exact source-substituted candidates without resolver changes:
native metadata remains 1,524 packages with 24 current Genet packages and two
old Knot-owned identities; web remains 527 packages with 18 Genet packages.
Node features and dependency edges are unchanged after mapping the revision.
Both detectors reject an outside path and an old current-family source.
Root/web locks retain 1,651/527 packages, WGPU 30.0.1/30.0.0, NetRender
`9607d16f` and netrender-vello 0.10.1. All 385 recorded pre.2 patch files retain
their bytes. The initial offline cache miss and completed pinned Git checkout
preparation are preserved as separate attempts.

The new optional accessibility description field requires two Reader
initializers and the web-host blank test initializer to specify `None`.
Reader's former dirty state had no normalized content delta: its archived raw
preimage contains 956 CRLF lines and four LF-only lines. The adoption preserves
those bytes and inserts exactly two LF-terminated lines; it does not normalize
the file or absorb unrelated edits.

Full affected-host tests, workspace verification and the native/web consumer
compile gates remain pending. After the focused controls, C: had about
1.54 GiB free; existing remaining host debug artifacts alone were about
1 GiB before new libraries and link temporaries. Further builds are held for
adequate headroom, without reducing proof scope or deleting evidence.
Commands explicitly use Rust 1.98.1, four jobs and the stable Mere target;
incremental output is disabled for this bounded run. Receipts are under
`testing/mere/receipts/2026-09-28/inline-scroll-adoption` in the shared Code
root. Main publication, pre.4 reconciliation and S13 remain separate gates.

## 2026-09-29 resumed verification before main integration

Disk headroom is restored. The five checks held in the September 28 entry now
pass at the same runtime source and locked Genet `7a60ad79` graph: affected
host/Mesquite tests, Reader all-target check, portable workspace verification,
native Cambium/Sprigging/host all-target check, and standalone Graphshell web
Wasm check. The host/Mesquite run has 156 passing tests, zero failures and three
existing ignored tests across 25 result blocks. Each accepted command records
1,996 unchanged source/lock hashes, matching the earlier restored Rootstock
proof. The original 44-test passes and four deliberate assertion failures
remain the focused repair evidence; they were not needlessly repeated.

The first resumed host attempt stopped before compilation because the offline
registry cache lacked `petgraph`. A separate `cargo fetch --locked` restored
the exact cache without changing locks or sources, and the preserved offline
retry passed. Commands continue to use Rust 1.98.1, four jobs, the stable Mere
target and disabled incremental output. The web check additionally uses only
`--cfg getrandom_backend="wasm_js"`.

The first full workspace check exited successfully, but another task briefly
edited and restored four shared source files while it ran. That attempt is
retained as qualified evidence, not the accepted gate. The other owner reports
the interval as 2026-09-29 04:09:16.765 through 04:10:39.248 UTC; root separately
verified the restored content. After the first check ended, all four files
matched HEAD and the accepted runtime hashes. Advancing only their mtimes
forced Cargo to recheck Rootstock, Mesquite and the winit host. The separate
`workspace-verify-restored-selector` run passes. Host tests and the Reader check preceded
the interval; native and web checks followed restoration.

This is a verification checkpoint before integration with newer
`origin/main`, not permission to publish an older main. The remote now carries
separately approved canvas, physics and web work, including additional web
packages, features and a newer Wasm binding version. The recorded 1,524/527-node
graph comparison belongs to this bounded source snapshot. Integration must
classify those upstream changes alongside the exact Genet revision substitution
and rerun the affected tests and native/web graph checks on the combined tree.
Pre.4 reconciliation, S13, headed/browser receipts and downstream Isometry
publication remain separate gates. `resumed-checkpoint.json` in the same receipt
directory seals this outcome without altering `blocked-checkpoint.json`.

## 2026-09-29 combined checkpoint with a31b9a14

The pending merge combines local scroll adoption `8eca3e4c` with incoming main
`a31b9a14`, preserving upstream runtime work and the tested Genet `7a60ad79` pin.
Native metadata remains 1,524 packages with 24 current Genet packages and the
two existing Knot-owned identities; its only added edge is web-host to Mesquite
under `cfg(target_arch = "wasm32")`. The root lock retains 1,651 identities and
all 385 recorded pre.2 patch files retain their bytes.

Web metadata/lock grows from 527 to 636 packages: seven required Wasm-family
updates and 109 additions reachable from the three approved local host roots.
Nineteen existing nodes gain features/edges without losing prior ones, including
the requested `Location` and `UrlSearchParams` features. The filtered Wasm graph
has 445 reachable nodes; these are graph counts, not payload measurements.
An incidental offline `libredox` downgrade was rejected and prior 0.1.25 restored.
The accepted ignored web lock and all attempts are preserved. Native/web WGPU
stay 30.0.1/30.0.0, NetRender stays `9607d16f`, and netrender-vello stays 0.10.1.
Both fresh source detectors reject an outside path and an old Genet revision.

Tests pass: 202 Rootstock/host/Mesquite tests with four ignored doctests;
261 Pictograph unit plus two headless Vello tests; 88 Seiche default-library
and eight no-default runtime tests. Both retained-layout cases and the GPU
lost-crossing-edge control pass. Workspace verification, Reader all-target,
native and standalone Wasm checks pass. These are correctness results on a
shared device; pre.4 tensor/release and S13 acceptance remain separate.
All commands preserve 3,034 tracked-file/ignored-lock hashes. The earlier host
wrapper has hashes only; seven later runtime/check runs also preserve mtimes.

Evidence: `testing/mere/receipts/2026-09-29/scroll-main-integration/combined-checkpoint.json`
under the shared Code root. This accepts only the a31b9a14 checkpoint, pending
independent review and local commit. Remote main advanced to `ee77bf23` during
verification; its Personae/Insigne migration must be integrated and the affected
dependency checks refreshed before publication or downstream adoption.

## 2026-09-29 ee77bf23 follow-on verification

The verified merge of `ee77bf23` into `14f5f9c9` retains tested Genet `7a60ad79`
and upstream's Insigne migration. Default native metadata keeps 1,524 packages:
four Knot identities move to `855cb75d` and eight nodes gain Insigne edges.
Web keeps 636 packages (445 in the filtered Wasm graph), adding only the
Pandect-to-Insigne edge. Registry identities/checksums and all 385 pre.2 patch
files remain unchanged. Existing legacy Genet and Knot-site identities remain
explicitly classified. Five default-graph fault controls reject the wrong source
or an outside local path.

Exact package, feature, dependency and local-file comparisons preserve the five
Rootstock/host/Mesquite/Seiche runtime cones, carrying their a31b9a14 receipts.
Pictograph reaches changed Personae files, so its 263 tests were rerun and pass,
including retained-layout cases and the headless crossing-edge control. The
Insigne/Personae/Notochord all-feature suite passes 233 tests. Native workspace
and standalone Wasm all-target/all-feature checks pass; only the existing
`mere-linked-data` native failure is excluded. All-feature metadata selects
1,651 native and 636 web packages within the unchanged locks; four additional
source controls reject both faults. Metadata includes `mere-linked-data` and
does not establish compilation of that excluded package.

Every follow-on gate retains 3,034 source/lock hashes and unchanged mtimes.
Shared-device rendering establishes correctness, not performance. Evidence is
`testing/mere/receipts/2026-09-29/scroll-insigne-integration/final-checkpoint.json`
under the shared Code root. Pre.4, S13 and downstream adoption retain their
separate gates.
