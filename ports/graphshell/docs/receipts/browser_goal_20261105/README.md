# Goal surface and durable tag suggestions, headed browser receipt (2026-11-05)

Eidetic plan P4 (memory_gates D15, D11): the goal surface in Graphshell's
browser mere over the extension's IndexedDB, with tag suggestions that survive
a cold reopen. Fixture profile only (five invented objects); no proposals
through Alembic (D11).

## What the browser retains
Browser capture (`web_capture.rs`, `BrowserVisit`) keeps no page body:
visits, and the graph objects they create (title, address, tags, links).
Nothing in the browser writes `PageTextStore` or page-text captures. So the
provider reads, per graph object, a record of its title and address
(`Origin::Record`, text "title. address", version = BLAKE3 of that text), and
page text instead for any page where something has stored it. A goal can match
only those words. Real page-text retention in the browser is a separate piece
of work and is not built here.

## Source cohort
Mere branch `agent/browser-goal` on `b38abf4e7`; Eidetic pin 86e78dd; Chrome
156.0.8078.80 (headed, real Google Chrome, unpacked dev-profile build of
`graphshell-web`, `Extensions.loadUnpacked` over the CDP pipe, as the hidden
writer qualification does). Apple M4, macOS 26.5.1.

## Run
`ports/graphshell/web/extension/qualify-goal-surface.py --out DIR --package dist/chromium`
(needs `websocket-client`). Seeds five objects through the page's own forms
(real mouse and key events), then: goal "keep a sourdough starter alive while
away" -> 1 of 5 read sources matched; Suggest tags for all retained pages ->
4 suggestions (two concepts, `sourdough` and `baking`, from the one tagged
object); accept `sourdough` on the "Keeping a sourdough starter..." object;
reject `baking` on "Oven steam..."; regenerate (new run, 7 rows: the accepted
tag is held so it is not proposed again, earlier reviews unchanged); close
Chrome; relaunch on the same profile directory (cold): 7 rows with Accepted,
Rejected and five Not reviewed intact, regenerating again adds 3 and still
does not re-propose the accepted tag; a second tab is a follower: same rows,
no Accept, Reject or Suggest controls, goal run works. Result: pass
(`chrome_receipt.json`).

## Captures (large; not committed)
`/Users/markik/Code/testing/eidetic/browser_goal_20261105/run1/`, sha256 in
`captures.sha256`; reviewed by eye: seeded graph, goal results with
highlighted passages and the walk's reasons, suggestions with
producer/run/version/evidence, accepted and rejected states, the regenerated
list, the cold reopen, the follower (with its "Viewing only" bar).

## Not qualified
Firefox (the hidden-writer harness's Firefox plumbing was not run for this
panel); Mark's real data (a later step); page-text retention in the browser;
the follower queueing accept/reject (a follower shows suggestions read-only);
a tag read back through the details form after accept (selection by canvas
pointer was unreliable in the harness; the held tag is shown by regeneration
not re-proposing it, and by the lib test reopening the saved graph); some
highlight spans render empty in the web presenter (text drawn only for the
first line of a wrapped mark); dev (unoptimised) wasm only; real clicks needed
the wheel and a hover to scroll the Graph tools dock.
