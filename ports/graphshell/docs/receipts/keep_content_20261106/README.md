# Keep page text, headed Chrome receipt (2026-11-06)

Eidetic plan P5 (memory_gates D19): the extension's optional "keep page text"
control, recorded into Eidetic's `PageTextStore` / `ResourceCaptureStore`, read
back by the browser mere's goal surface. Fixture profile, invented pages on a
local server (127.0.0.1); no real data.

## Source cohort
Mere branch `agent/keep-content`; Eidetic pin 86e78dd; Chrome 156.0.8078.80
(headed, `Extensions.loadUnpacked` over the CDP pipe, dev-profile wasm); Apple
M4, macOS 26.5.1. Harness: `web/extension/qualify-keep-content.py`
(`--package` an unpacked `prepare-extension.sh chromium` output).

## Run (`chrome_receipt.json`, result pass)
Live remembering stays off throughout; three graph objects are seeded through
the page's forms (titles and addresses hold none of the goal's words). Control
off by default. Goal "vermicast red wigglers seedlings": 0 of 3 matched. Turn
on **Keep page text** (32 KiB limit), load five pages in tabs (three pages, one
on an excluded origin, one `text/plain`): panel shows 3 kept, 3 skipped
(excluded origin, non-HTML, the extension page). Reloading one page: 1
unchanged, nothing new written. IndexedDB holds 3 `page-text/`, 3
`resource-capture/` and 3 `blob/` keys. The same goal: 1 of 3 matched, citing
`/notes/1` with body passages and version fb8c0937be5f. The excluded origin's
word matches nothing; the long page's first word matches and a marker past its
32 KiB cut does not. Chrome closed and relaunched on the same profile: the same
goal cites the same version; the panel keeps its counts. Server log: only the
loaded pages and the browser's own favicon requests; the link in the pages was
never requested.

## Captures (large; not committed)
`/Users/markik/Code/testing/eidetic/keep_content_20261106/run10/`, sha256 in
`captures.sha256`; reviewed by eye: control off, goal before, control on, pages
kept, unchanged counted, goal with passages, cold-reopen goal and panel.

## Not qualified
- **The browser's access prompt.** The harness cannot press the native
  permission bubble (osascript has no accessibility rights here). The run used a
  copy of the package whose manifest declares `scripting` and `<all_urls>` as
  installed rather than optional, so `permissions.request` resolves without a
  prompt. The prompt, refusal and later removal are covered only by
  `smoke-keep-text.mjs` and the code path, not headed.
- Firefox; Mark's real data; private windows (smoke only: the extension is not
  active in them by default); quota failure (Rust test with a write-refusing
  store; the panel's failure line was not driven in the browser).
- The writer view drains only while it is the visible tab (hidden pages'
  frames are throttled); text loaded meanwhile waits in the queue.
- Some highlight spans render empty in the web presenter (pre-existing).
- Dev (unoptimised) wasm only.
