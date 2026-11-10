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


## 2026-11-05 — Title, tags and stale edits with a hidden writer

Chrome 156.0.8078.80 and Firefox 159.0.1 passed the title and tag edit and the
stale object refusal in their installed unpacked extensions. The packages
and Wasm hash match the preceding receipt. The follower selected the saved
object, entered a new title and two tags, and used **Send changes**. Its
refresh notification was held for one commit so a second submission from
the same form would still carry the previous title and tags. Both requests
used the same session, epoch and member.

In both browsers the writer stayed hidden through both replies. It admitted
the first edit once and refused the second because the object had changed.
The follower reread the accepted title, and a fresh process reopened the
same extension profile with that title and both tags. The stale draft
did not replace either field.

Raw receipts and screenshots are under
`Code/testing/mere/graphshell-hidden-writer-20261105/`, in
`metadata-run1/chrome/` and `metadata-run2/firefox/`.


## 2026-11-05 — Handover during a queued edit

The [headed handover harness](../../../ports/graphshell/web/extension/qualify_handover.py)
ran the same packages in Chrome 156.0.8078.80 and Firefox 159.0.1.
A native IndexedDB transaction in the follower held the overlapping store
scope with a read pump. The harness observed the hidden writer start a real
save, with the write pending behind that transaction, before it clicked
**Edit here** in the follower. The writer refused the handover as busy and
kept the sole exclusive Web Lock. The waiting handover request was canceled.

Releasing the transaction let the writer commit and acknowledge one edit.
The next **Edit here** transferred the sole lock to the follower. The old
writer reloaded as a hidden follower and displayed the same single saved
object when shown. Showing it did not move the lock. A fresh browser
process reopened the same profile with exactly one saved object.

Raw receipts and screenshots are under
`Code/testing/mere/graphshell-handover-20261105/`, in
`20261106T005455Z/chrome/` and `20261106T005538Z/firefox/`.
The harness controls the overlap without replacing IndexedDB or the
application save path. This scenario covers voluntary handover.

## 2026-11-10 — N2 Woodshed on desktop and web

Rebuilt the N2 fixture at Mere `fe7c4905168e90e578895bcdb6c6e811492c70de`
with Rust 1.99.1. The published fixture commit `942d8a6c478d5fa04b632bd12eb15df6e466af14`
has identical source and dependency files. The native Graphshell Blocks window
and headed Chrome 152
with WebGPU ran on Intel macOS 15.8.1. Both used the same signed component
built from Woodshed `c9b461da0c846e3a08a237d70acc7a16eb03de4e` and the
[shared passage scenario](../../../ports/graphshell/web/scenarios/woodshed_passage_return.scn).

Before approval, the passage refused to open the tune. After approval, both
hosts opened `graphshell:fixture/woodshed-tune` from the Bridge passage in
Rehearsal notes, showed the guest example and passage as provenance, and ran
**Write notation**. Both rendered the chart and selected a chart event. The
source fingerprint changed from `19910c964cbf8bd1` to `8b0d3ab47ae1b581` on
both hosts. **Return to passage** restored focus to the originating link.
Pressing Enter opened the same edited chart in the current session.

Native approval used the window's review control. The web run used the
existing browser approval harness. The browser refusal control made zero
guest calls. The approved browser run made ten guest calls with zero
refusals or errors.

Raw receipts, screenshots, guarded build logs and frozen inputs are under
`Code/output/mere-n2-woodshed/published/`. Android and iOS device evidence
is pending. Live peer access and offline reopen have separate gates.
