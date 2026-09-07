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

## Current Mere-owned compatibility (2026-09-06)

Cambium is now a Mere workspace family under `crates/cambium/`; it is not a
Genet workspace subtree. Mere owns the current `cambium`, `meristem`,
`sprigging`, and `workbench` paths, while pinning Genet seam packages at one
immutable `genet.git` revision (`221415af6643e7b31510547963217973ada6332b`).
Errand is likewise Mere-owned at workspace version 0.3.4.

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
