# Shared link attachment — 2026-10-18

Compiled and tested source: `0b7454123426578a22d02a8e530b67c0e6f237b0`.
Work began on refreshed Mere `908283728`, incorporated `9b8c161a8`, then merged
published consumer/trace updates through `15b9b6648` before final qualification.
The [owning plan](../../../design_docs/mere_docs/implementation_strategy/2026-10-15_link_model_plan.md#shared-billboard-attachment--2026-10-18)
records the findings, completion criteria and remaining backlog.

| Claim | Actual evidence |
| --- | --- |
| Shared canvas | [Process](canvas-final-6.json), [log](canvas-final-6.log): 573 library and five integration passes, 18 library and one doctest ignores. The run uses `--test-threads=1`; the unchanged 2ms wall-clock gate passed. |
| Graphshell library | [Process](graphshell-final-3.json), [log](graphshell-final-3.log): 312 passed, six ignored. |
| Product package | [Wasm build](wasm-final-3.json), [log](wasm-final-3.log), [bindgen](bindgen-final-2.json), [log](bindgen-final-2.log), [artifact/source hashes](build.json). |
| Viewer and boundaries | [Viewer process](viewer-final-2.json), [log](viewer-final-2.log), [boundaries](boundaries-final-3.json), [log](boundaries-final-3.log): passed. |
| Native seed | [Scenario](browser-qualified/p4_tree_link_attachments/result.json): eight captures, result `ok`, no scenario errors or gate failures. |
| Fresh native process | [Reopen](browser-qualified/p4_tree_link_attachments_reopen/result.json), [process observations](browser-processes.json): seed PID 10950 exited; final reopen PID 12379 reopened the stored IndexedDB graph and exited. |
| Image review | [Ten image observations/hashes](browser-review.json): primary and independent subagent review, with reviewer ownership per image. |
| Source continuity | [Comparison](source-continuity.json): equal 15,699-byte source readings and graph revision 21. This runtime revision is not an immutable public revision claim. |
| Evidence verification | [Thin verifier](verify.py) reuses the [existing checks](../link-model-ordinary-paths-20261017/verify.py). [Process](verification-final-2.json), [log](verification-final-2.log): passed exits, actual Cargo summaries, hashes, PNG dimensions, scenario results, source equality, stored reopening and distinct-process observations. |

`NodeBillboard` resolves the upright outer body through the existing camera,
node size, grammar scale, outline and lift. Body layout, inset icon placement,
visual picking, marquee selection, self arches and responsive clipping consume
it. Borders fit that size. Link picking measures tolerance in screen pixels;
paint/pick/report still share one segment list. Self-loop route construction
has one implementation. Shape intervals and precision-aware strict-interior
queries serve all consumers, replacing repeated crossing predicates. Physics
colliders and force calculations retain their separate authority.

Scripted shared-API profiles cover flat, tilted/yawed, rounded, circular,
scaled Ring suggestion and lifted bodies on the normal product graph route.
Every profile passed independent Livery body-layout readback against model
bounds within 0.05px. Responsive endpoints passed the same 0.05px attachment
limit, with zero strict crossings and presentation fallbacks. Source-backed
projected Resource picking also passed. Profiles are qualification inputs,
not new ordinary user controls. [Capability observations](browser-qualified/probe/environment.json)
record Chrome 155, Apple Metal/WebGPU, DPR 2 and an initially empty database
inventory at the isolated origin `127.0.0.1:38129`.

The native lifted fixture rises 16px; its ground foot remains inside its 36px
body. Larger exposed stems and distinct self-loop arches have CPU regression
coverage. The existing quiet grammar lowers path contrast and the territory
underlay is strong; neither is claimed solved by attachment work. Conservative
bounding-box overlap refusal remains safe resting geometry. Wider cohorts,
central crossing density, new silhouettes, other receiving hosts and physical
qualification are separate bounds. No frame-time gain is claimed.

Failed runs are preserved under [iterations](iterations/index.json): invalid
new test inputs, a strict float comparison, a lifted fixture lacking a held
Surface relation, product-only compile omissions, contention-sensitive timing
failures, and the first native circular check. A matching CPU cohort measured
0.000019073px inward round-off against the former fixed 0.00001px predicate.
The shared coordinate-aware rule now distinguishes it from a real 0.1px
intrusion. The original 0.05px visible attachment/readback gates are unchanged.
The first reopen correctly restored Graphshell's enabled grammar; its script
incorrectly expected `false`. Only the reopen input was corrected after build;
[build.json](build.json) records executed input hashes and the override.
The attachment implementation and recorded source files are unchanged after
qualification. Publication also merges upstream standards documentation and
the unrelated `mere-view-tier-rule` divider from dashed to solid. Its
[product compile check](publication-check.json) and [log](publication-check.log)
are separate from the source-specific native image qualification.

The full package, frozen site and raw audit inventories remain at
`/Users/markik/Code/testing/mere/shared-link-attachments-20261018`.
Generated Wasm is excluded from Git. Run:

```sh
python3 support/receipts/link-model-shared-attachments-20261018/verify.py \
  /Users/markik/Code/testing/mere/shared-link-attachments-20261018
```

The archive lacks `pkg/`; verification without the manifest-matching package
must fail. Chrome was quit normally after both accepted runs. The loopback
server was intentionally interrupted afterward ([observation](server-process.json)).
The [documentation comparison](doc-audit/comparison.json) records new findings
separately from existing global audit debt (including 20 judgment errors). Knot checkpoint confidentiality
remains in its own storage lane; this feature changes no Knot source.
