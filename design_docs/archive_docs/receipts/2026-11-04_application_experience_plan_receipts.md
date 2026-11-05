# Application experience plan receipts

## 2026-11-05 — Graphshell follower edit with a hidden writer

Built the full Graphshell WebAssembly product with the standalone locked
workspace, from Mere `0c46d0c73` (fix `8131e527a`). The repinned workspace
uses p2mere 0.9 and Eidetic `5182e4e2`. Both prepared extension packages
contained the same `graphshell_web_bg.wasm`, SHA-256
`ca4b8a96f3bb9321ed8105cefba8cbd63cbd139856bf33e6829919f5a7b7670a`.

The [headed harness](../../../ports/graphshell/web/extension/qualify-hidden-writer.py)
installed the unpacked extension into a disposable profile in each actual
browser on macOS 26.5.1 arm64. The two views used the browser's extension
origin, real Web Locks, BroadcastChannel and IndexedDB. The follower used
Panes → Document tools → Add an object. After submission the harness waited
for a matching acknowledgement before inspecting the writer.

| Browser | Writer at admission and acknowledgement | Follower reread | Fresh process, same profile |
| --- | --- | --- | --- |
| Chrome 156.0.8078.80 | Hidden throughout; one admission and one acknowledgement | Exactly one saved object | Exactly one saved object |
| Firefox 159.0.1 | Hidden throughout; one admission and one acknowledgement | Exactly one saved object | Exactly one saved object |

Both replies were admitted. Raw receipts and screenshots: `Code/testing/mere/
graphshell-hidden-writer-20261105/20261105T233459Z/` (Chrome and Firefox
subdirectories). The headed scenario covers adding an object. Metadata
edits, stale refusals and handover with in-flight edits retain library or
transport smoke coverage.

On the merged source, the locked Fleece and graph-kernel library runs
passed 20 and 535 tests; Graphshell passed 330 with seven ignored.
The writer-handover transport smoke passed.
