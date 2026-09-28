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
