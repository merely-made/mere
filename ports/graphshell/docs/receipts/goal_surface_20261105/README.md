# Goal surface, headed native receipt (2026-11-05)

Eidetic plan P4 S5: a goal given by a person, run over retained page text and
the graph's links, with cited results and traversal explanations, in
Graphshell's native host. Cited results and explanations only; no proposals.

## Source cohort
- Mere base: 3209349e23ee328bd9d97e3964d0fe35cdc49650 (main); code commit: ea317d523ac73ec5b9ee149e0409e0615081ad2c (branch agent/s5-goal)
- Eidetic: rev 5182e4e244c85b1008182a1be6b822f65a8efe50 (Cargo.lock; eidetic 0.0.3, eidetic-search 0.0.1). This rev predates
  `PassageSelector::to_byte_range`; the view converts scalar offsets itself.
- Fleece: `retained-traversal` (via graphshell feature `goal-surface`); provider = lexical `GoalAssessor`, no model.
- Features: `--no-default-features --features goal-surface-proof`, bin `goal-surface-host`.

## Host
Apple M4 (Metal 4), macOS 26.5.1, headed shared Cambium winit host, 1280x920 logical (2560x1840 captured, scale 2).

## Run
`cargo build --offline -p graphshell --no-default-features --features goal-surface-proof --bin goal-surface-host`
`target/debug/goal-surface-host --receipt DIR --size 1280x920` (about 9 s, closes itself).

## Limits
Visits 50, depth 2, lookups 50, bytes 4 MiB (defaults); start page: Bread notes index.

## Result (qualification.json)
Goal "keep a starter alive while travelling": 6 lookups, stop Exhausted, cited Baking while travelling and Starter care
(each with version, matched/missing terms, highlighted passages); Hotel kitchens (linked, not retained) reported as not read,
not fetched. Three captures reviewed by eye: stage-0 empty surface; stage-1 goal typed, results and start picker; stage-2
explanation (scroll is clamped to the bottom, so it equals the lower view of stage-1's layout).

## Captures (large; not committed)
/Users/markik/Code/testing/eidetic/goal_surface_20261105/run2/stage-{0,1,2}.png, sha256
75b21a72713fe8d34500191a524cd2b8ad626c7ca9c0154363323a4785bc4fb7,
8b13f1dafc825fa4b67e2c5737d2ec26649b1901d31c4f39a6ef975b1e06726d,
0094d7b234d0df4797335001e05001a70ef4faa962ec9a320e91bec7b3c551b6.

## Not qualified
Physical pointer/keyboard (goal buffer filled and Find clicked via DOM callbacks); other platforms; model provider path;
real user content (fixture is synthetic); the graphshell-mere resident-door viewer (it reads projections, not retained text).
