# Projection Grammar Adoption Plan

**Status (2026-10-06):** active. Track F (the field receipts FT9 and FT10, from the archived projection receipts plan, ruling S61) was received gated.
- The 2026-10-05 cross-domain relationship recipe continuation is published and
  qualified (its own status line, below).
- The executable compiler lives in `scenomise::projection` since `c79bb8c2`.
- Stack seams P1's arrangement catalog landed on main at `1633be0c`: built-in
  families, `ProjectionCompiler` with host `ItemSizes`.
- A5 and the remaining portable-grammar questions stay open.

This line supersedes the two older status lines in the document, the
2026-09-04 one below and "Status (reconciled 2026-09-01)" further down. Both
keep their words as history (stack seams S14 pass, batch 43).

Earlier status (2026-09-04): active: the executable Graphshell authoring proof landed 2026-09-04; A5 and the remaining portable-grammar questions stay open.

## Cross-domain relationship recipe continuation (2026-10-05)

Status: shared implementation and Knot adoption published and qualified;
visible Woodshed host adoption is published and qualified on committed source
`c55461cc`, with acceptance documentation on main at `dad3f624`.
The existing Scenograph authoring definitions are the source contract. The
executable compiler has moved from Graphshell to `scenomise::projection`,
preserving the authoring crate's independence from solvers and widgets.
Graphshell continues to compose that capability into its reference host.

Feature target: one editable relationship recipe over Knot sound selections and
Woodshed Working Set occurrences. Start with authored order, occurrence labels,
and disclosed relationships with explanations and method provenance. Duplicate
occurrences may refer to one source; selection and relationship endpoints remain
occurrence-specific. Domain calculations and owner actions stay in the adapters.

Done conditions:

- Both domains use the same shared recipe/compiler rather than copied layout
  implementations, with explicit refusal for missing semantic roles.
- A spacing/arrangement edit, occurrence selection, and explained relationship
  survive save/reopen; compatible rebinding requires an explicit new source.
- Knot embeds the capability in its reading workflow and retains validated
  recipe versions in its own mere. Woodshed retains its own source custody.
- Shared tests cover source freshness, duplicate occurrences, relation endpoints,
  schema bounds and compatible/incompatible bindings; consumer tests exercise
  actual domain results and owner actions. Native captures qualify only the
  paths actually observed.

This slice does not require the completion of dynamics G1–G6. Existing physics,
pairing and dependency-migration lanes keep their owners. No recipe conveys a
credential, data-access grant, or script execution authority.

Findings: inspection of Mere `bd119a69d` confirms existing durable Scenograph
definitions and the executable compiler's node/grid/scatter subset. The two
consumer repositories remain separate; the local integration worktree is the
single owner of shared compiler changes for this pass.

Progress: Scenograph supplies the typed relationship recipe, shared draft edits
and saved occurrence/relationship selection. Scenomise supplies bounded
disclosure validation, source-revision checking, compilation and explained edge
endpoints; Graphshell reexports the compiler instead of copying it. Named
`authored_order`, `occurrence_labels` and `explained_relationships` facets are
mandatory. Ordinal relationship recipes use a spaced grid; scatter coordinates
are not inferred from occurrence order. The generic coordinate compiler retains
its previous scatter support.

Knot integrates actual Mora relationships into its reading UI and typed retained
collection material. Woodshed supplies actual Working Set/catalog disclosure and
retains shared recipe state through its existing session model. Woodshed's
compiler adoption is exercised by a standalone compatibility instrument and a
visible host recipe editor. Production uses one coherent Mere `5011e2f9` /
Genet `bd3e8861` pair, without a local path override or copied compiler.

Shared qualification (2026-10-05): `cargo test --locked -p scenograph -p
scenomise` passes 8 authoring tests, 88 choreography tests (including nine new
relationship regressions), and the authoring compile-fail doctest. The two
crates also pass `cargo check --locked --target wasm32-unknown-unknown`.
Graphshell's native `--features web` compiler compatibility suite passes 12
tests; the generic compiler's prior grid/scatter semantics remain intact.
Shared Clippy passes with only the explicit pre-existing `manual_contains`
solver-lint allowance; unconditional strict Clippy is not claimed.

Historical disclosure checkpoint: shared code is on Mere main at
`c79bb8c203b52817991e0d7ba1c4ce3c3a2aac34`. Woodshed's disclosed source and
session adoption are on main at `c92e7c96779ef316f5d8244a59de7cbc84f42a0b`.
Its receipt reports core 189, views 97, desktop session 16, shared adoption 5,
exporters 10 and integration 1 tests passing, plus separate-process restoration
using the production session backend on an isolated unsealed fixture. It does
not qualify a visible Woodshed editor or native rendering.

Knot's actual Mora-derived recipe also rebinds through the common draft to the
generated Woodshed disclosure: edited label/spacing, repeated-source identity,
occurrence selection and exact musical explanation survive; neither dataset
changes, missing semantic facets refuse, and Knot document anchors cannot be
repurposed as music owner actions. Focused adapter test passes. Knot's native
recipe scenario has four reviewed captures at 1280×1100 (bind, spacing, explained
relation, authority refusal). Knot subsequently adopts shared horizontal reveal
`5011e2f9` on implementation `e4cf739c`, published with acceptance docs at
`9d0b955f`. Its compact recipe scenario passes four reviewed native captures at
420×900 and four at 1280×900 / 400% UI zoom: real second occurrence selection
reveals the full card, method/limits and source anchor remain readable across
scroll positions, and unavailable-authority refusal is visible. Updated-pin
qualification passes desktop 306 tests / one existing ignore, composition/readings
61 / one existing ignore, locked build and metadata. Successful native personal-wallet
retention remains unrun. Knot's canonical collection plan
records its complete suite and encrypted-retention receipts. The visible
Woodshed editor/renderer was qualified in the host continuation below; Knot's
unrun native personal-wallet gate remains separate. The compact native receipt
is `/Users/markik/Code/testing/knot-editor/recipe-responsive-20261005/receipt.json`;
no retained collection reopen, browser runtime or other-platform claim is added.

### Knot authored-scene adoption in Mere (2026-10-11)

Status: consumer qualification is published; Mere's embedded adoption is
qualified. Knot `19b99042d6c6708854f64e6187429da839a1c82c` adds bounded,
source-bound occurrence translations through the shared `SceneTrace`. Its own
mere retains immutable signed/encrypted versions; copied analysis, occurrence
identity, source anchors and source bytes keep their original authority. Drag
release and keyboard controls use Knot's existing undo/redo history.

Knot's canonical receipt is
`knot-editor/docs/receipts/2026-10-11_authored_scene/README.md`. It records 647
workspace tests, 45 default and 61 engine/highlight document tests, four native
save/fresh-process-reopen runs at 1100×900 and 420×900, and 28 inspected captures.
That native evidence uses disposable synthetic identities. Personal-custody
native acceptance, unattended background presentation and live dynamics remain
separate gates.

Mere moves workspace `knot-editor`, workspace `knot-document` and Djinn's
`knot-site` together. The regenerated root lock contains all six Knot packages
at that one revision and removes the old Genet `7422e906` Fleece/layout copies;
no registry version changes occur. Djinn's resolved closure redirects the
shared scene, host and storage contracts to this Mere workspace, with no second
Mere git source. The existing `scenotime` patch supplies the new retained trace
dependency; no desktop-only contract is added to the embedded patch table.

The supplier family remains Genet `4bc75583`, Dramatis `0234e7ee`, Netrender
`ef5412d7` and maintained Vello `5862bba2` (Classic/encoding/shaders 0.11.0).
Standalone Knot and Woodshed's published `8846b13` adoption qualify Mere
`0e78fcff`; Mere's embedded contracts use its own current source. Woodshed's Hocket/Redshank appearance reopen continuation is published at
`0032f35`: two fresh processes, 33 presentations and four reviewed captures,
with both saved profile identities preserved. Knot's next live-dynamics work
retains its owner.
Native execution stays serialized, Woodshed before Knot.

Qualification: isolated `cargo check --locked -p djinn --all-targets -j 2` passes.
The pinned `knot-composition` recipe suite passes 17 tests against Mere's local
contracts; shared Scenograph/Scenomise/Scenotime suites pass 243 tests, including
the authoring compile-fail doctest. Documentation findings and D2 judgment
coverage match pre-edit HEAD exactly; the existing snapshot-digest and producer
receipt coverage defects remain baseline debt. All three pins and the root lock
publish together. These checks establish embedded source/type and tested scene
contract compatibility; no new Mere native launch or personal-custody acceptance
is claimed. Exact results and source bounds belong in the
[embedded qualification record](../testing/receipts/2026-10-11_knot_repin/qualification.json).

Host continuation starting point (2026-10-05): refreshed Woodshed main `c92e7c9` matches the
published disclosure/session proof. Its production Mere `8106c7c` / Genet
`34626a6` pair predates the shared compiler. The next bounded pass integrates
the real shared draft/compiler into a visible Woodshed reading, preserving
Working Set/Card identity and source custody. It includes explicit source
navigation and rebind/refusal, durable edited state, actual host controls,
separate-process reopen, and default/narrow native captures. Production pins
must remain coherent; no local path override or copied layout/compiler counts
as adoption. The Woodshed checkout lane owns implementation and its canonical
plan; this coordinating lane owns capture review and shared status reconciliation.

Shared host contribution `5011e2f9` is published: Woodshed's 420px native
graph-node acceptance exposed that Rootstock's generic `scroll_into_view`
handled only the vertical axis. The independent semantic-click fixture failed
before the fix. The host now reveals horizontal targets through ancestor
scrollports and the document, respects clipping, keeps oversized targets stable,
and reports each moved plane once. Vertical alignment and caret-follow behavior
remain unchanged. Qualification: 56 Rootstock unit tests and 157 native-host
tests across 25 suites pass; the focused 18-test click/scroll gate is included
in that host total. No solver, shader, domain calculation, or Genet pin change
was required. Woodshed's production repin and final native matrix now pass.

Visible host qualification (2026-10-05): Woodshed implementation source
`c55461cc97bb29e9e78ecb2bb310bb4bda5accdf` uses the shared draft and compiler
for editable labels/spacing, graph occurrence selection, and method-disclosed
pitch-class relationships. A retained reading appears in Woodshed's own Mere
view, linked to its exact captured Working Set. Navigation checks owner Set,
Card identity, ordered content revision and full owner-derived disclosure;
changed source instructions refuse until explicit rebind. Navigation does not
start playback or transfer a background rehearsal owner. Invalid optional
retained payloads are preserved, not silently overwritten.

The final automated gate passes 558 tests (core 189, integration 1, desktop 63,
graph 14, views 106, theory 181, doctests 4); the standalone compatibility
instrument separately passes 5. Locked desktop build and native host check of
the web crate pass. This is not a browser/wasm runtime acceptance claim.

Committed-source native acceptance passes 8 captures at 1280×900, 8 at 420×900,
and 2 after a separate-process reopen. All 18 captures were visually reviewed,
including real last/middle graph-node selection at narrow width, readable
explanation/method/limits, visible stale-source refusal, explicit rebind, retained
owner disclosure, restored spacing/selection, and exact source Card navigation.
The first final narrow attempt failed before any capture landed because the
surface was occluded; its logs remain preserved. An unchanged-source replay in
a fresh profile passed after explicit foreground Raise. External scenario copies
only extend initial settling to allow native launch and Raise; assertions remain
those of the committed scenarios.

Evidence: `/Users/markik/Code/testing/woodshed/relationship-host-20261005/receipt.json`
records source/binary/scenario hashes, automated logs, preserved failed attempts,
and the committed baseline, narrow retry and reopen captures. Retention here
uses the production session FsBackend in an isolated explicitly unsealed profile.
It proves durable host state, not successful personal-wallet encryption. No
high-zoom, other-platform, release-signing or browser-runtime qualification is
claimed. A multi-reading library and explicit recovery UI for invalid payloads
remain follow-ons; this bounded host slice does not complete A5 or dynamics G1–G6.

Publication verified: Woodshed main and origin/main match
`dad3f6240bef8339432effa2d9fe7840d5035612`, with a clean checkout. That
acceptance-only commit follows the native-qualified implementation `c55461cc`.
The existing Woodshed `repos/woodshed/design_docs/2026-07-11_stage_set_tools_plan.md` and
document index record the final gates and their limits; no parallel plan was
introduced. Mere's shared behavior fix is on main at
`5011e2f90e988775410fe4b4213e9895fe64538e`.

## Executable authoring proof (2026-09-04, verified in current working tree)

Mark requested Luna/Terra subagents to try the review's bounded proof. Graphshell's
editor previously saved definitions while its preview only formatted a sentence. This
slice resolves a supported definition against disclosed Woodshed Set facts and
compiles it through existing Score/Scene machinery. Woodshed owns the export;
Graphshell has no dependency on Woodshed's product crates. This does not promote
new portable grammar contracts or close A5.

Done when a generated export from actual Woodshed Set/catalog types, including
two occurrences of one source, drives two arrangements in the existing browser
host; spatial and list selection name the same occurrence; saving and reopening
restores the definition and selection. Unsupported registrations, fields, and
stale source revisions must fail explicitly. Validate the exporter, focused
compiler tests, wasm build, and a headed browser interaction with a captured
frame. Preserve concurrent Cambium and Woodshed release work.

Findings (2026-09-04): the executable subset resolves `nodes`, numeric x/y
fields, a text label field, `grid.default` (dense numeric ranks), and
`scatter.default` (numeric values). Both compile through `scenomise::solve`;
Graphshell realizes the result through Cambium, Genet, and Netrender. Single
occurrence selection is coordinated between spatial cards and an occurrence
list; the saved payload holds the definition and occurrence id. Unsupported
registrations, channels, realization policies, and stale source revisions are
explicit failures. Source ownership remains in Woodshed; the included export
is a reproducible three-card Set made from its real musical catalog, not a live
connection to a user's saved practice library.

*2026-10-05, one arrangement catalog.* The
[stack seams plan](2026-10-04_stack_seams_plan.md) ruled S1: "Resolve through
scenomise (Recommended)". Graphshell maps the two families above by hand:
- `arrangement_for` and `placement_for` in `ports/graphshell/src/projection_compile.rs`
  (lines 666-691);
- `grid.default` uses a fixed 184 by 84 cell and 8 columns;
- `scatter.default` maps to `Geographic`;
- every other id resolves to nothing.

Both functions retire into one shared scenomise compile step. It resolves an
id through the registry, takes its parameters from the definition, and makes
every registered family authorable. That plan's P1 carries it out. Asked who
takes P1, Mark answered: "Seams lane keeps P1 (Recommended)". This plan
records the change and does not build it.
*Superseded 2026-10-06* (stack seams S14 pass, batch 43). When this note was
written, local main did not yet hold `c79bb8c2`, which had already moved the
compiler into `scenomise::projection`. `ports/graphshell/src/projection_compile.rs`
is now 432 lines and re-exports `scenomise::projection::*`, so the hand map
above is gone. P1 then replaced it with the built-in catalog (next note).

*2026-10-05, P1 under way (stack seams rulings S12, S15 to S20).* A
cross-reference for the relationship-recipe pass, which moved the compiler
into `scenomise::projection` at `c79bb8c2`. One correction to the note
above: scenomise's registry holds only custom solvers (stack seams F12), so
ids resolve through a built-in catalog of sceno's eleven named families beside
it, and an unknown id falls through to the registry (S15). P1 changes the
compiler where it now lives: the dataset types move to scenograph and stay
re-exported from `scenomise::projection` (S12); `grid.default` and
`scatter.default` resolve as aliases, so saved recipes load (S15); parameters a
recipe leaves unset are measured from the items (S16); and item sizes come from
the host instead of the 164 by 68 written into the compiler (S20), so the
compile entry points take the host's sizes. A consumer sees two changes when it
repins past P1: that signature, and grid pitch (cell plus gap), which becomes
the host's card size plus the spacing (180 by 84 for a 164 by 68 card at
spacing 16, against 200 by 100 today). Built on branch `stack-seams-p1`;
nothing merges before Mark's review.
*2026-10-06:* P1 landed on main at `1633be0c`, recorded in the stack seams plan
at `add54925`. The compile entry points take a `ProjectionCompiler` built from
host `ItemSizes` (`scenomise/src/projection.rs:520`, `:530`).

Progress (2026-09-04): Luna produced the Woodshed exporter and Terra the
compiler skeleton before both hit the account usage limit. Root completed
integration. An isolated source-path harness passed 14 compiler/editor tests,
15 existing Stage/arrangement tests, and 3 exporter tests. This harness points
at the actual Rust source modules and excludes unrelated application hosts;
it is not a full Woodshed workspace test. The exporter replay SHA-256 is
`E23353CD22FA22B309FF4E46E1941429525B3B20C7A5F832FBF66E3011F32B7F`.
The fixture is `ports/graphshell/web/fixtures/woodshed-stage.json`, produced by
`woodshed/crates/woodshed-core/examples/stage_projection_export.rs`.
The browser scenarios are `projection_authoring.scn` and
`projection_reopen.scn` under `ports/graphshell/web/scenarios`.
The actual standalone Graphshell wasm build passed offline. Terra also ran
`cargo test -p graphshell projection_compile --lib -j 1` against the native
workspace: 7 tests passed (overlaps the harness coverage above).

The final Chromium/WebGPU run at 1280x720 passed 58 authoring steps and 13
fresh-page reopen steps, with four GPU captures and no page errors. Additional
real browser input verified Enter selection from the list and pointer selection
from the spatial view. A full-page screenshot exposed a CSS specificity bug in
the semantic overlay; the final run verifies transparent, absolutely positioned
buttons aligned with the painted card rectangles. Invalid numeric spacing now
invalidates the typed draft and blocks saving instead of saving its old value.

Evidence is in `ports/graphshell/docs/receipts/projection_authoring_receipt.json`
and the four `projection_*.png` files beside it. The receipt records source and
wasm hashes, exact scenario results, and manual browser checks. These are current
dirty-tree receipts, not a committed release baseline. To replay, build the
standalone web package, run wasm-bindgen with `--target web`, serve the web assets
over loopback HTTP, and open `index.html?scenario=scenarios/projection_authoring.scn`,
then `index.html?scenario=scenarios/projection_reopen.scn` on the same origin.
The loader exposes scenario results and captures on the page; an optional `sink`
query parameter receives the existing JSON receipt protocol.

The Woodshed musical-projections plan is the next product-directed refinement.
Its review addendum records metric, complete-cost, history-provenance, and voicing
resolver corrections. This proof does not implement those slices or settle the
finished product appearance, narrow layouts, arbitrary data loading, or release
packaging.

## Shared practice proof (2026-09-04, verified in current working tree)

Mark authorized a second Luna/Terra proof: two people join a Woodshed practice
space, contribute one musical comparison, disconnect, and reopen retained work.
Woodshed's `musical_comparison_export` example owns the keyed C Major/A Minor
disclosure and versioned singleton-difference method. It does not implement a
general voice-leading metric or publish personal browsing history.

`commons_practice_peer` is a proof-only native consumer of real Commons redb
replicas, Personae writer attestations, and Gemot authority-filtered projections.
Each peer owns an independent store. Fixed identities, a fixed evaluation clock,
and an exactly pinned fixture constitution establish the test boundary; signed
delegations admit the member. This is not a constitutional lifecycle or identity
onboarding implementation. The gateway carries canonical signed plaintext graph
operations over loopback HTTP. It does not establish Internet P2P discovery,
encryption, or a production invitation protocol.

Graphshell's `co_op.html` hosts membership, contribution, sync and offline-reopen
controls alongside the existing Genet/Netrender projection. Only effective shared
records enter the contributed rows. Local source material is separately marked
as a preview. `co_op_receipt.py` reproduces the process/HTTP receipt, including
an unauthorized signed record retained but excluded from the effective view;
headed browser verification is a separate gate. Disconnect closes the native
process, and offline reopen starts a fresh process over the same local store.

Both gates passed. The process/HTTP receipt proves pre-invitation join and
pre-admission contribution rejection, successful signed admission, byte-equivalent
comparison retention on both peers, distinct process IDs after offline reopen,
unchanged network traffic during that reopen, and duplicate-free reconciliation.
A third fixture writer's signed operation remains retained but pending authority:
both peers retain two operations and expose only the one admitted comparison.
Six native checks also pass: invitation codec roundtrip, tampered delegation,
unsigned rule mutation, wrong-space invitation, unauthorized author, and
wrong-container operation.

Two browser surfaces completed found/join/contribute/disconnect/reopen. Fresh
pages then each passed the seven-step retained-projection capture scenario at
1280x720 with no page errors. Keyboard selection of the shared comparison maps
to its spatial occurrence. Full-page visual inspection corrected a sidebar
overlap, stale host status, and an unavailable arrow glyph. The executable wasm
build passed, as did Woodshed's seven exporter tests. The first authoring proof
also passed its 58-step authoring and 13-step reopen regression after adding the
external-source seam (before the final cosmetic fixes).

`ports/graphshell/docs/receipts/co_op_process_receipt.json` records the process
states and negative checks. `co_op_browser_receipt.json` records browser actions,
source/wasm hashes, offline peer states and the two `co_op_*.png` GPU captures.
The PNGs exclude HTML controls; those were separately inspected in full-page
browser screenshots. Receipts describe a dirty working tree, not a published
feature. Membership changes over time, public hosting, encryption, discovery,
arbitrary shared Set editing and production identity onboarding remain unproven.

Reproduce from Mere after building the standalone Graphshell web package and
running wasm-bindgen into its `pkg` directory (an isolated copied asset directory
also works):

```powershell
cargo build -p mere-moot --example commons_practice_peer -j 1 --target-dir <isolated-target>
python ports/graphshell/web/co_op_receipt.py --binary <isolated-target>/debug/examples/commons_practice_peer.exe --comparison ../woodshed/scenarios/woodshed_musical_comparison.json --assets ports/graphshell/web --output <new-receipt-directory>
```

For interactive reproduction, run `co_op_gateway.py` twice with separate `--store`
directories, `--role founder` / `--role member`, ports 8860 / 8861, and opposite
`--peer-port` values. Both require the same `--binary`, `--comparison`, `--assets`
and a `--receipts` output directory. Open each server's `co_op.html`. After offline
reopen, `?scenario=scenarios/co_op_retained.scn&sink=http://127.0.0.1:<port>/scenario-receipt`
captures the retained GPU projection on that origin.

**Date**: 2026-08-15
*Superseded by the top status line (2026-10-06); kept as written.*
**Status (reconciled 2026-09-01)**: A0, A6, A1, C1, B1, B2, B3, C3, A3 stage
one, A4+C2, and A2 are closed. Turnstone `648bf19` is B1's definitive close,
including routed screen-reader interaction. The Projection Receipts Plan's
coordinated spatial/Matrix views close A2's resolution half. A3 stage two and
A5 retain their entrance gates. A3 stage two's gate was sharpened 2026-09-01
against C4a: it waits for a remote viewer with a local camera over a served
scene, which C4b is not; its fact-transport shape is ruled (evaluation must
work without the host facts; carrying them is the stretch). A5 waits for a named product consumer to open
its first proof; the promotion suite begins with that proof and is an evidence
harness rather than a consumer. FT7 is closed through the local, admitted
remote, and frozen Matrix parity receipt. FT8 is closed through Retinue
Signalman's mixed-realization receipt at `8cea8f9`.
The Scenograph family is now on an unpublished 0.0.4 development line with
Score v4. Mer3ly ruled an authority-grade consumer 2026-08-16. This plan turns the projection grammar report's
findings (the claude.ai design artifact "Projection Grammar Report", two
passes, sources verified 2026-08-15) into gated feature targets across mere,
genet, and cambium. Sequenced against the
[scenograph expansion brief](../research/2026-08-10_scenograph_expansion_brief.md)
lanes L1-L5 and governed by the
[projection grammar catalog](../research/2026-08-15_projection_grammar_catalog.md)
promotion rules.
**Related**:
[scenograph_0_0_3_release_plan](../../archive_docs/2026-10-06_completed_plans/2026-07-24_scenograph_0_0_3_release_plan.md)
(historical 0.0.3 release; rulings D1-D4),
[projection_proofs_plan](../../archive_docs/2026-10-06_completed_plans/2026-07-21_projection_proofs_plan.md) (P1-P5 landed),
scene contract note
(`design_docs/scenograph_docs/technical_architecture/2026-07-22_scene_contract_note.md`),
[multi_window_plan](../../archive_docs/2026-10-06_completed_plans/2026-06-10_multi_window_plan.md),
[graph_signals_layer_plan](../../archive_docs/2026-08-20_completed_plans/2026-06-22_graph_signals_layer_plan.md),
[accesskit_screen_reader_verification](../../archive_docs/2026-09-02_retired_plans/2026-06-09_accesskit_screen_reader_verification.md).

## Ruling context


### Workbench component adoption (2026-08-31)

Platen now names its graph-specific split/tab cache `TileLayout`; `Workbench` is a compatibility
alias. Its structural AccessKit projection lives at
`platen::accessibility::project_tile_layout`, beside the layout it reads. The old Mere
`workbench` package is retired. Mere and Graphshell consume Genet's reusable `workbench` package
at immutable revision `eff0cb6df4834ecce9ac552a055c1c459befa7c3`. The component boundary and
remaining headed-host receipts are in
`genet/design_docs/2026-08-31_workbench_component_plan.md` *(historical citation)* <!-- doc-audit: historical-path -->.

The report reviewed the catalog against eleven external systems (Vega-Lite,
Draco, SetCoLa, Gemini, GoTree, ATOM; then Mosaic, Gosling, Penrose, Bluefish,
GoFish) and returned three results this plan acts on: the catalog's stack
holds and its authority layer is genuinely novel; every named contract gap has
a shipped anatomy to lift rather than invent; and the eleven systems are prior
art for the *compiler* (what a projection spec means), not for drawing scenes.

Two disciplines from the report govern every target here:

1. **Solver proposes, the score records.** Draco-style search and
   Penrose-style global optimization live above the score, never inside it.
   The score stays a deterministic record of what was chosen; determinism
   receipts stay meaningful. (Report caution 4.)
2. **A layer enters the portable contract only when a promotion proof fails
   without it.** GoTree's complexity cliff is the cost of adopting
    speculatively; the catalog's promotion rules are the defense. Published
    0.0.3 remains an historical artifact; `main` evolves through explicit
    score and crate versions behind forcing consumers. (Report caution 2.)

### Consumer ruling (2026-08-16)

Mer3ly is authority-grade: its asks open gates, and it is not a donor in the
way graphshell is. It is live, public, pinned to a real rev, and consumes six
Mere crates plus published cambium and genet-scripted-dom. The
[mer3ly stack consumer survey](../research/2026-08-16_mer3ly_stack_consumer_survey.md)
is the evidence. The practical effect is that "the first consumer that asks"
has a real answer today, so the targets below name mer3ly where they used to
name a hypothetical host, and expansion lane L3's entrance gate is already met
in production.

## Findings

What transfers, from where, to where. Landing sites verified against the tree
2026-08-15 (score shape read from `crates/scenograph/sceno/src/score.rs` *(historical citation)* <!-- doc-audit: historical-path -->;
0.0.3 release rulings from the release plan; genet components listed from
`repos/genet/components/`).

| Transfer | Source system | Landing site |
| --- | --- | --- |
| Placement satisfaction state; pin = ensure (fails loudly), anchored home = encourage *(2026-10-03: the pinned and anchored roles, beside seeded; see A1)* | WebCoLa silent-soft caution + Penrose `ensure`/`encourage` | sceno scene surface + scenomise solvers (A1), cambium chrome (C1) |
| Selection clauses with declared resolution (single / union / intersect / crossfilter) | Mosaic selections + Vega-Lite selections | chirograph intent plane / mere host coordination (A2) |
| LOD rungs as declarative conditions (measure, operation, threshold, hysteresis) | Gosling `visibility` | cartography representation profiles, then `ScoreItem.representation` selection (A3) |
| Transition specs between epochs, host-owned clock | Gemini | scenotime, expansion lane L5 (A4, C2) |
| Set-scoped constraints over predicate-defined member sets | SetCoLa | future score constraint records, gap 2 proof (A5) |
| Factored arrangement dimensions instead of new monolithic variants | GoTree + ATOM | `score::Arrangement` evolution at the hierarchy/tiling proofs (A5) |
| Rule-filled scales and guides; unit/aggregate distinction | Vega-Lite + ATOM | gap 1 proof (A5) |
| Per-channel shared/independent scale resolution | Vega-Lite `resolve` | gap 4 proof (A5) |
| Backdrop property classing (visible / collidable / hit / provenance; layout vs paint) | Mapbox Style Spec | expansion lane L2 backdrops (C3) |
| Accessible static realization as a standing receipt | none external; catalog's W3C citations stand | promotion rules + Graphshell realization targeting Genet surfaces (B1) |
| Compound scenegraph shape (hierarchy + adjacency together) | Bluefish | sceno `Scene` already carries spaces, regions, layers; hold the shape deliberately, add nothing now |
| Effectiveness knowledge versions beside the grammar, never inside it | Draco 1 vs Draco 2 | wherever defaults/effectiveness land; `SCORE_VERSION` covers the wire, not the knowledge |

Current portable baseline the targets extend: `Score` v3 with
`Arrangement::{Spiral, Grid, Geographic, Hulls}`,
`Placement::{Ordinal, Cell, Coordinate}`, per-item footprint, pre-selected
`Representation` rung, layer, visibility, authored holds, and honored/unmet
placement truth. Intents live in chirograph (release ruling D1); picking in
scenotime (D4); emphasis channels open (D3).

## Plan

Three tracks. Every target names its forcing consumer; a target with no
consumer yet states its entrance gate and waits.

### Track A: mere (contract and compiler)

**A0. Catalog second shelf (docs only) — landed 2026-08-15.**
Context: the catalog's external-systems section reads as one shelf of
renderers and toolkits; the report's central structural finding is that
specification languages and design solvers are a different kind of prior art.
Tasks: split "What the external systems teach us" into two shelves; add the
eleven report systems with one-line transfers; add the Penrose name-collision
note (CMU's Penrose diagram language is unrelated to the
`graph_layout:penrose` tiling arrangement); cite the report in-repo rather
than by artifact URL, since mere is a public repo.
Validation: catalog cites all eleven; DOC_README updated.
Done when: a reader of the catalog can find every system the report verified
without leaving the repo.

**A6. The placement seam (numbered late, sequenced first) - CLOSED 2026-08-16.**
Context: mer3ly consumes the stack along two disjoint paths. The portable path
builds a `Score` with `Placement::Ordinal` and solves it; the live path runs a
seiche simulation with `pin_node`, three-way mobility, and backdrops. `Score`
and `Placement` appear only in the first, pins and mobility only in the second,
and nothing joins them. A visitor who pins three repositories and shares the
result is sharing state the portable contract cannot express, which is why the
site invented its own wire.
The seam is the precondition for three targets that otherwise each wait on a
consumer that does not exist: A1 has no satisfaction to record while pins never
reach a score, A2 has no coordinated view to serialize, and C3 has no portable
backdrop.
Tasks: let a live arrangement's placement reach the score.
`Placement::Coordinate` carries a visitor-placed position; the mobility class
(`anchored` and `free` since mer3ly `145eb41` retired `frozen` from the
interactive path) is recorded rather than inferred from force configuration; the solver's proposal and the recorded outcome stay
distinguishable, per the solver-proposes discipline. Decide with mer3ly whether
the seam is a write-back into the score or a sidecar the score references.
Ruling 2026-08-16 constrains the decision: whatever A6 produces is also the
shelfmark's `placement` delta section, one serialization, never a parallel
one (the shelfmark format note owns the envelope; A6 owns this record).
Forcing consumer: mer3ly, authority-grade.
Validation: a pinned arrangement in the sandbox survives a round trip as a
portable artifact; the reconstructed scene places pinned items where the
visitor put them; the determinism receipt still holds for the unpinned case.
Done when: the two paths meet, and a shared scene is a portable artifact rather
than a site-local JSON blob.

**A1. Placement satisfaction state - CLOSED 2026-08-16.**
Context: the catalog's free/anchored/pinned policies currently have no
satisfaction surface. Authored coordinates exist today (isometry adapts
authored pins to a geographic score via `Placement::Coordinate`), and two
mechanisms may displace placed items after the arrangement speaks
(`scenomise::relax`, viewport fits). WebCoLa's silent-soft failure is the
warned outcome; Penrose's ensure/encourage is the vocabulary: a pin is
ensure-class (satisfied or reported), an anchored home is encourage-class
(best effort by design).
Tasks: first, audit where a `Coordinate`-placed item can move after solving
and whether anything records it. Then, with the forcing consumer, give the
realized scene a satisfaction record for ensure-class placements (shape
decided by the consumer: a typed field or a recognized channel). An unmet pin
is reported, never silently repositioned.
Forcing consumer: mer3ly, once A6 lands. Its sandbox already ships the
ensure/encourage split (an `anchored` mobility installs a soft `AnchorSpring`,
a manual pin hard-pins) with no satisfaction record anywhere, and it already
shares pins to a second device. Until A6, those pins never reach a score, so
there is nothing for a satisfaction field to attach to. Isometry's VTT pins and
the canvas pin/unpin intent remain alternates.
Validation: a score with an unsatisfiable pin produces a scene that carries
the violation; a test asserts the violation is present rather than the pin
silently best-efforted; the record crosses the graphshell wire.
Done when: a remote viewer can distinguish "placed as pinned" from "pin
unmet" without source access.

**2026-10-03, the roles.** The
[dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md) retires ensure
and encourage for three roles: seeded, anchored and pinned (F26). Read this
target's ensure-class as pinned. Its encourage-class covered two different
behaviours:
- the anchored home, which returns to the arrangement (F19);
- sceno's best-effort `Hold::Anchored` ("relaxation may carry it away",
  `crates/cambium/scenes/sceno/src/score.rs`, lines 86–88), which nothing
  returns. F19 and F25 name it seeded.

F25 gives `Hold` all three: `Seeded` (the old `Anchored`'s meaning, read from
old saves by a serde alias), `Anchored` (returns) and `Pinned`. The dynamics
plan's G7 builds it. The Progress entries below keep their words: their
`Hold::Anchored` and "encourage" are the seeded role, and their
"ensure-class" is pinned.
- A1's record is unchanged: `honored_holds` carries pinned holds only.
  *Reading, not ruled:* anchored holds stay out of it, as seeded ones did, and
  report a residual through the dynamics plan's G6 instead.
- **A finding for mer3ly.** `PlacementDelta::holds`
  (`mer3ly/crates/repo-graph/src/lib.rs`, line 1878) records a pin placed
  under mer3ly's `anchored` motion as `Hold::Anchored`, "best effort by the
  visitor's own choice", and a test at line 3034 asserts it. The sandbox's
  `shelfmarkPlacement` (`mer3ly/assets/graph-sandbox.js`, line 1521) makes
  the same choice in JS. The live path itself hard-pins a manual pin in
  either motion (`pinNode`, `mer3ly/crates/repo-graph/src/lib.rs`, line 399),
  so the recorded class already differs from the live behaviour under
  `anchored`. Under F25 the Rust line keeps compiling but changes meaning,
  from best effort to returning, the next time mer3ly repins mere. Scores mer3ly has already shared read
  back as seeded, through the alias. Which role those pins should take
  returns to Mark with mer3ly's mobility move, which G7 lists outside its
  track.
  *2026-10-03, ruled:* a manual pin records as `Hold::Pinned` in either
  motion, once mer3ly takes up the roles. Mark: "A pin is a pin
  (Recommended)" (mer3ly's site canvas plan, Ruling 103). Both mapping sites
  drop the motion check. *Landed the same day,* ahead of mer3ly's repin
  (mer3ly `f0fc678`), at Mark's "make the move". mer3ly no longer writes
  `Hold::Anchored` anywhere, so G7 changes no meaning for it.
- **The alias, as F25 is written.** A serde alias `"Anchored"` on `Seeded`
  also catches the new `Anchored` on read. In a scratch probe,
  `Hold::Anchored` serialized as `"Anchored"` and read back as `Seeded`, with
  only an `unreachable_patterns` warning. How old saves are read is G7's
  question, and it has gone to the dynamics grammar session.

**A2. Selection clauses: coordination as data — CLOSED 2026-08-23.**
Context: release ruling D1 stands (sceno ships no intent vocabulary; the
protocol owns the triple). What the report adds is the *coordination* record
between views: Mosaic's clause (source, client set, predicate, value) with
declared resolution (single, union, intersect, crossfilter), where crossfilter
means a view is filtered by every brush but its own. Mere's one-app-state,
N-windows posture (multi_window_plan, expansion lane L3) is the shipped
precedent's exact use case; a Mere clause carries a reading-parameter delta
against graph authority, never SQL.
Tasks: with the forcing consumer, define the clause record and resolution
declaration; decide its home with evidence (chirograph beside the intent
triple, or mere host state that graphshell serializes); wire two views over
one authority through it.
Forcing consumer: L3's entrance gate is met. Mer3ly shares a scene by URL hash,
and Wave 1 of the [projection receipts plan](../../archive_docs/2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md)
supplies the genuine two-view ask: its spatial view and two-reading Matrix
contribute named clauses over the same source identities. The proof forced
crossfilter only; union and intersection remain absent until their behavior is
distinctly required.
Validation: brush in view one filters view two; crossfilter resolution
honored (the brushing view is unfiltered by its own clause); serialized round
trip is deterministic; clause removal restores the unfiltered reading.
Done when: brush, filter, and focus are named, serialized citizens rather
than host-only state.

**A3. LOD rungs as declarative conditions - STAGE ONE LANDED 2026-08-19 (via P3b); stage two stays gated, gate sharpened 2026-09-01.**
Context: `ScoreItem.representation` is a pre-selected rung; the conditions
that select it live in host code, so a remote client cannot re-select on its
own zoom and a static realization cannot state why a rung was chosen.
Gosling ships the missing form: target, measure (screen-space width/height
vs data-space zoomLevel), operation, threshold, and hysteresis padding as
data. The measure split mirrors Mapbox's layout/paint classing: screen-space
conditions are realization-dependent, data-space conditions are
reading-dependent.
Tasks: stage one, conditions as data in cartography's representation
profiles (host-side registry; the P3b card-to-glyph traversal is the named
consumer). Stage two, portable only when a remote viewer re-selects on a
camera of its own: rung conditions travel beside the score, and a static
realization evaluates them at its declared zoom deterministically. Stage two
is larger than the ladder alone: `ScreenWidth`, `ScreenHeight` and `ZoomLevel`
are viewer facts, but `Recency` and `Focused` are host facts the wire does not
carry (recency derives from `graph.node_last_visited` in
`cartography/src/spiral_score.rs`), so either they travel per item or the
ladder is split-evaluated. Ruled 2026-09-01: both, in that order of
obligation; see
[Stage-two fact transport](#resolved-ruling-and-open-question).
Forcing consumer: P3b (the recorded remaining half of P3: representation
degrades card to glyph with recency and zoom, focus stays live). Mer3ly does
not force this one: it exports `representation_registry()` to JavaScript for
the live path's own UI, while its portable path hardcodes
`Representation::Glyph`, and its fold/expand is a manual toggle over `visible`
plus an untyped fold channel rather than a measure and threshold.
Validation: same graph and placement inputs, two declared zooms, different
selected rungs, deterministic; a hysteresis test shows a rung boundary does
not flicker under small zoom oscillation.
Stage-one done when: the representation ladder is registry data and one live
host selects from it using declared view facts rather than a hardcoded branch.
Entrance gate (sharpened 2026-09-01): a remote viewer with a local camera
over a served scene. The earlier wording, "a remote re-selection consumer",
made C4a's browser peer look like a candidate; it is not. The browser's
remote view has no camera (`ports/graphshell/src/web.rs` `zoom` and `pan`
return unless the session is local; `remote_scene` fits the served bounds at
scale <= 1.0), the served rung is fixed by the endpoint
(`MereHost::build_snapshot` hardcodes `Representation::Card` at
`ports/graphshell/src/mere_host.rs`), and the client's own resolution is
capability-based offer choice within that rung, the presentation plane the
remote projection host plan ruled in §4.3 and landed at G1. C4b's
done-conditions are keyboard and accessibility-tree checks on the embedded
and full-page surfaces; they add no camera. The gate opens the moment a
served scene is composed on the local canvas's camera, which the remote
projection host plan §1 permits as curation truth.
Stage-two done when: the ladder travels beside the score because a second host
needs to re-select locally, and that host honors it without porting Mere's
selection code.

**A4. Transition specs between epochs - CLOSED 2026-08-19.**
Context: expansion lane L5 named motion the deepest of the missing 90%;
`scenotime::diff` computes what changed and every consumer snaps. Gemini's
result: transitions specified relative to explicit start and end states, over
component classes, with sync/concat composition, and the authors rejected
extending the visualization spec itself. Scenotime epoch pairs are the
natural start/end states; the host owns the clock (the same seam discipline
the mesh host, `distillery::mesh_host`, uses for `Clock`).
Tasks: a transition spec over diff output (which item classes, what staging,
duration ratios); pure evaluation against host-supplied time in scenotime;
default staging so consumers get respectable motion for free; playback in the
first continuous consumer.
Forcing consumer: L5's entrance gate verbatim: the first continuous
re-projection a consumer ships (woodshed's rehearsal filmstrip or a canvas
projection switch, whichever lands first). C2 is the cambium half. Mer3ly
already ships most of the record this needs, a labeled revision-chained diff
sequence it steps through with a source-time control, so it is the cheapest
place to prove a spec once one exists; what it lacks is the authored half,
duration, easing, and staging.
Validation: identical scene pair plus identical spec yields an identical
schedule; a static realization is unaffected by transition specs; motion
stays out of arrangement types (the catalog's motion taxonomy holds).
Done when: a projection switch reads as a staged transition, specified as
data, on at least one consumer, with snapping still the default elsewhere.

**A5. Gap proofs adopt named anatomies (gated on a named first proof consumer).**
Context: the catalog's promotion suite is an evidence harness, not a consumer.
It begins when a named product needs one of these proofs. Reusing one
heterogeneous fixture as orrery, matrix, Cartesian chart, hierarchy, and
schematic then tests whether the resulting contract material composes beyond
that first ask. The report's job was to make sure none of that material is
invented fresh.
Tasks, each strictly behind its proof:
- Chart proof (gap 1): scales, axes, legends filled by rule with derivation
  recoverable, per Vega-Lite; the unit/aggregate distinction per ATOM ("a bar
  over twenty nodes is not a twenty-first node" is already catalog law).
- Facet proof (gap 4): per-channel shared/independent resolution, lifted from
  Vega-Lite `resolve` with its default resolutions.
- Schematic proof (gap 2): ELK's port and label anatomy for endpoints;
  SetCoLa's set-scoped constraint form (constraints over predicate-defined
  sets, instance generation deferred to the runtime) so a saved layout
  reapplies to a second graph, which is the catalog's second-dataset receipt.
- Hierarchy proof: before adding tree/treemap as new monolithic
  `Arrangement` variants, evaluate GoTree's factoring (element x placement
  rule x coordinate transform) and ATOM's recursive partition operators so
  node-link and space-filling are parameter settings of one family.
- Every proof: an accessible static realization in the receipt (B1 defines
  the shape). Read GoFish in full before the facet and flow proofs; it is the
  chart-side proof of the catalog's central bet.
Forcing consumer: no A5 proof is activated. The
[Distillery projection walk](2026-09-02_distillery_projection_walk_plan.md)
W3 is the prospective schematic consumer: training/provenance records and the
workspace dependency graph, manifest/evaluation interactions, and a frozen
navigable table. At W3's entrance, check existing routed polylines and endpoint
semantics before claiming a portable port/routing addition is needed. Naming
this candidate does not close its entrance check or open the other A5 families.
Entrance gate: a named product consumer needs one proof strongly enough to
state its task, source facts, interaction, and accessible output. Found the
suite around that proof. Any portable addition
still needs the catalog's second heterogeneous consumer; the other suite views
do not count as substitutes merely because they share a fixture.
Validation: per the catalog's promotion checklist, unchanged.
Per-proof done when: the contract addition cites the anatomy it lifted, the
first proof that forced it, and the second heterogeneous consumer that proves
it portable. A5 stays open until every A5 proof family is either promoted
through that evidence or ruled unnecessary by a forcing proof.

### Track B: genet (realization receipts)

Genet is the realization layer of the projection stack: the accessible static
form and the drivable interactive form both *target* its surfaces. They do not
live in it. **Corrected 2026-08-16 while opening B1**: genet has no dependency
on scenograph and mere depends on genet, so a converter from `Scene` to a
semantic form cannot sit in genet without inverting the stack. The static
realization lands in mere and renders into genet's lane, which is the same
direction mer3ly already proves by serializing cambium views to static HTML.
Pointer docs in `genet/docs/` are founded when a slice genuinely lands there.

**B1. Accessible static realization - CLOSED 2026-08-20. All legs, including the manual pass.**
Context: no surveyed grammar treats the accessible form as a first-class
realization target; the catalog's W3C citations (WAI complex images, Graphics
ARIA, SVG structure) are the anatomy. Retrofitting accessibility contracts is
the expensive order, so the receipt shape should exist before the promotion
suite starts producing receipts.
Tasks: define the receipt shape (navigable structure, names, descriptions,
values, relations, and a tabular or long-form alternate where the visual
alone is insufficient); realize one existing scene (the P3 spiral score or
the P5 `coastal_map.json` fixture) as a static semantic form through genet's
DOM lane; verify with the AccessKit lane precedent
(accesskit_screen_reader_verification, 2026-06-09) and a genet-probe scenario
asserting the semantic tree (apps self-drive via genet-probe; never synthetic
OS input).
Forcing consumers: Mer3ly's shipping accessibility gap opened the static form,
and Turnstone `648bf19` supplied the definitive headed traversal and routed
interaction close. The promotion suite reuses this receipt shape; it is the
harness that made the need visible, not a substitute consumer. Mer3ly's sandbox
was JavaScript-only, shipping `data-graph-interface` hidden with a runtime-built
node list and a no-script status reading "Graphshell sandbox not initialized",
so the site served accessibility by rendering an entirely separate
authority-derived static index instead. That workaround is what B1 retires.
Validation: a screen-reader traversal of the static projection enumerates
instances and relations with names; a probe scenario asserts structure
deterministically; the same scene still produces its interactive realization.
Done when: "static with navigable semantics" is a checklist item the first
promotion proof can satisfy by following an existing worked example.

**B2. Probe-drivable projections - CLOSED 2026-08-20.**
Context: genet-probe drives applications through DOM-carried identity, and
chirograph intents target an `InstanceId`. The two identities should meet:
a scenario should pick a projected instance and invoke an intent on it by
stable identity, not by coordinates.
Tasks: projected instances expose stable identity to the probe resolver at
whichever host surface renders them; a scenario verb resolves instance to
intent invocation.
Forcing consumer: the first headed receipt that asserts an intent against a
projected instance (the promotion suite's interaction receipts, or the A1
pin receipt, whichever runs first).
Validation: a scenario addresses an instance by identity, invokes an intent,
and asserts the `IntentResult`; renaming or moving the instance on screen
does not break the scenario.
Done when: projection receipts are driven by identity end to end.
**Landed.** Turnstone's retained Knot toolbar carries the projected
`InstanceId` and advertised intent in DOM data attributes on Save, Resolve,
and Run. The Knot session retains the endpoint's exact `IntentResult` as a
visible receipt carrying the same target and intent. `knot_authoring.scn`
selects Resolve as `.knot-resolve @data-projection-instance=1`, independent of
its label and screen position, then asserts
`Projection intent 1: knot.transclusion.resolve accepted` and the derived
content. A focused unit proof resolves a second fixture at `InstanceId(17)`
through genet-probe. The headed scenario completed with `RESULT ok`. The
boundary stayed narrow: Turnstone carries the host identity, genet-probe uses
its existing generic class-plus-attribute resolver, and chirograph continues
to own `InstanceId`, invocation, and result.

**B3. Livery property-classing check - CLOSED 2026-08-16, recorded.**
Context: Mapbox's layout/paint split is prior art for classing every visual
property by what it invalidates. Livery's TOML property DB likely already
carries an invalidation class per property for its own engine needs.
Tasks: read livery's property DB schema; if classing exists, record the
rhyme in livery's docs and close this item; if it does not, weigh adding the
class with livery's own consumers, not on the report's authority.
Validation: one paragraph of recorded evidence either way.
Done when: the question is answered from the tree, not the report.
**Answered.** Livery's DB carries no invalidation class: 123 property entries
with name, value type, inheritance, initial, grammar, seed values, animation
behaviour, and source, and nothing saying what a change invalidates. But the
rhyme is not missing from genet, it is *borrowed*: `LayoutDamageClass` resolves
None / PaintOnly / Relayout and a color swap is asserted repaint-only, with the
damage computed inside Stylo's `compute_style_difference` from the incumbent's
own property metadata. The adapter's `compute_layout_damage` hook is a stub.
So the classing leaves with the fork at retirement unless Livery's DB grows
somewhere to put it. Recorded in the cutover plan's deferral register as a
compounding item, on that plan's authority and with its retirement stage as the
forcing consumer, which is what this target asked for.

### Track C: cambium (host consumption)

Cambium is where scenes meet users: `cambium-genet-winit-host` is the
single-root host, woodshed then signalman its consumers, and swatches are the
agreed cross-product graph-view contract. Cambium doc updates land in
`genet/components/cambium/docs/` *(historical citation)* <!-- doc-audit: historical-path --> when a slice opens.

**C1. Satisfaction state in host chrome - LANDED 2026-08-16.**
Context: A1's scene-side record is only honest if a user can see it.
Tasks: with A1's consumer, surface pin state in the host's widget chrome
(a pinned badge; an unmet-pin state visibly distinct); keep the vocabulary
plain (pinned, held, unmet), not solver jargon.
Validation: the headed receipt shows a pinned item, a displaced anchored
item returning home, and an unmet pin visibly reported.
Done when: no best-effort placement is presented as satisfied truth in any
cambium-hosted view.

**C2. Transition playback with the host clock (consumer half of A4) - CLOSED
2026-08-19.**
Context: scenotime evaluates transitions purely; a cambium host owns time.
Tasks: the host drives A4's schedule with its frame clock; woodshed's
rehearsal filmstrip is the named first consumer (L5); the canvas projection
switch is the alternate.
Validation: pausing the host clock pauses the transition; identical inputs
replay identically; a consumer that never adopts transitions still snaps.
Done when: one shipping cambium consumer plays a staged epoch transition.

**C3. Backdrop realization (consumer half of expansion lane L2) - CLOSED
2026-08-20.**
Context: L2's two waiting consumers (isometry's map, woodshed's stage floor)
force the backdrop contract; the report adds the vocabulary: class backdrop
properties explicitly (visible, collidable, hit-transparent, provenance;
which properties are placement-class vs paint-class), per Mapbox's decade of
production answers, so "a backdrop may be visible, collidable, both, or
neither" is declared data rather than host convention.
Tasks: prototype against both consumers per L2's entrance gate; carry the
property classing into whatever shape both force; a remote graphshell viewer
renders the backdrop from scene data alone. Mer3ly supplies a third,
already-shipping data point for the minimum property set: `set_backdrop(kind,
tangible)` in the live path, and a backdrop carrying kind and collidable on its
shared wire. Identity plus collidable is what a real consumer reached for
first, ahead of visible, hit-transparent, and provenance.
Validation: L2's own gate, plus: the same backdrop crosses the wire and
renders identically remote; a hit-transparent backdrop never picks; a
collidable one participates in placement.
Done when: environment is scene data with declared properties on two
consumers.

### Track F: field receipts (gated; received 2026-10-06)

The projection receipts plan's wave 3 came here when the stack seams S14
archive pass retired that plan (stack seams ruling S61). Its tail is recorded
in the [archived plan tails plan](2026-07-03_archived_plan_tails_plan.md),
section "2026-10-06 archive pass". The archived plan is at
[archive_docs/2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md),
§5. The targets keep that plan's words. No work opens under either one until
its gate's trigger is met: scheduling them here asserts order, not readiness.

**F1 (was FT9). Derived-mark integrity (receipt 3) — gated.**
A Distribution or Contour scene emits derived marks that remain selectable,
name their values and contributors, and never masquerade as source nodes.
Gate: a named statistical or field consumer.

**F2 (was FT10). Field distinction (receipt 6) — gated.**
One sample dataset produces Territory and Contour scenes whose categorical
and scalar meanings remain distinct through rendering, picking, legends, and
accessible output.
Gate: a field-data consumer. The trigger candidates on record:
- host-side radio and mesh placement facts. This is the sited-device brief's
  fact surface: the node does not know where it is, the host knows where it
  put it.
- a simulator or radio model for Current.

*Reading, not ruled:* both receipts are A5's territory, the remaining
portable-grammar questions, which already wait on a forcing consumer. A field
consumer that opens F2 would also force A5's gap 1, derived marks, scales and
guides. So F1 and F2 open with A5 rather than ahead of it.

### Sequence

Closed: **A0**, **A6**, **A1**, **C1**, **B1**, **B2**, **B3**, **C3**, **A2**,
**A3 stage one**, **A4 + C2**, and P3b's local selection-to-renderer proof.
Turnstone `648bf19` closes B1's readable and routed-interaction receipts.

The remaining implementation targets keep their entrance gates. **A3 stage
two** waits for a remote re-selection consumer. **A5** waits for a named first
proof consumer; the promotion suite is founded with that proof.

Non-goals, restated from the governing docs and the report: no intent
vocabulary in sceno (D1 stands); no global nonconvex solver
in scenomise (deterministic solving is the receipt currency); no speculative
adoption of the report's six-layer spec stack (each layer arrives only with
its proof); no new grammar DSL (the score is the spec; the report's
what-a-spec-means table describes meanings the score may adopt, not syntaxes
to build).

## Resolved ruling and open question

**The site's wire as an index - ruled 2026-08-16.** Mark's framing, 2026-08-16: the wire has its
own utility as an index, and that may be a better way to think about it than
promotion. The argument in its favor is already in the code. `score.generation`
is derived from the authority's SHA-256, so a score is a pure function of its
authority. That makes the site's scene state a citation rather than a
transport: dataset, revision cursor, reading, arrangement, and a small delta of
pins, selection, motion, and backdrop. The scene is reconstituted
deterministically rather than shipped, which is why the wire is small. If the
framing holds, the five fields it carries beyond the contract do not all need
to enter the score, and the index becomes its own layer above the contract with
its own promotion question.
Ruled 2026-08-16, all five questions, recorded in the
[shelfmark format note](../technical_architecture/2026-08-16_shelfmark_format_note.md):
the citation is **shelfmark**, a Mere-level format held as a note until the second
shipping consumer; the citation delta and A6's sidecar are one record (A6's
`placement` section is the first standardized member); home is incipit, scoped,
with chirograph the fallback; checkability (`expects.generation`) is required in
v1. The [scene citation index brief](../research/2026-08-16_scene_citation_index_brief.md)
holds the reasoning.

**Apps embedded in the site - ruled 2026-09-01, two sites, two answers.** Mark, 2026-08-16: if he could embed full
versions of all his apps in the site, he would. That is a larger scope question
than this plan carries, but it bears on the index ruling directly. An index is
exactly the addressing an embedded app needs for a deep link, and under that
reading authority-grade consumption stops being one site consuming one stack
and becomes the site hosting the family. Recorded here so the index question is
not settled without it.
Ruled 2026-09-01. There are two sites and the answer differs. For
**merelyllc.com**, the company site: it hosts Graphshell embeds, and every
other application reaches it as a served projection through Graphshell, which
is the remote projection host plan's product ruling (§1) applied; shelfmark is
the deep link, C4b of the browser WebRTC carrier plan is the embed component,
and C5 public rendezvous is what lets a visitor reach a device. For
**mer3ly.net**: full versions of the applications embedded in the site as
their own browser builds, the site hosting the family. Forced to one answer
for both, the second wins. Refined the same day: the first of those
applications is Graphshell, and its job on mer3ly.net is to **supersede the
site's own canvas** (the seiche sandbox the consumer survey found consuming
the stack along its second, non-portable path) and to **serve as the site
index** beside everything else it does. That reading joins the two rulings
above: shelfmark is already the index layer, and Graphshell as the site's
canvas is the thing that reads it. It also answers the prover-or-demand
question raised in Progress for mer3ly's own case: once Graphshell is the
site's index, the graph view stops being a stress-test toy and becomes the
site's navigation, which is a product pull. The mer3ly.net direction is a
new objective, not a target in this plan: it needs its own assessment,
Graphshell-supersedes-the-canvas first and one per further application,
since each has its own browser story. Nothing in the ruled scope of this
plan starts it.

**Stage-two fact transport - ruled 2026-09-01.** Found 2026-09-01 while testing C4b
against A3's gate. `RepresentationLadder::select` evaluates five facts;
three are the viewer's (`ScreenWidth`, `ScreenHeight`, `ZoomLevel`) and two
are the host's (`Recency`, `Focused`), and neither the score nor the scene
carries the host pair. When stage two opens, one of two shapes has to be
chosen: the host pair travels per item beside the ladder, or the ladder is
split-evaluated, host clauses resolved before serving and viewer clauses at
the viewer. The choice fixes what a static realization can claim about why a
rung was chosen, so it is ruled before the consumer exists, not under it.
Ruled: not a choice between the two. **Evaluation without the host facts is
the baseline obligation**: a viewer given a ladder and no `Recency` or
`Focused` must still select a rung, so the host's clauses resolve to a
per-item ceiling before serving and the viewer evaluates the rest. **Carrying
the facts is the stretch**: a host may disclose them per item, and a viewer
that receives them evaluates the full ladder. Disclosure is therefore never
required for the ladder to work, which is what makes it a host's choice
rather than a wire obligation. Design consequence for stage two: a condition
whose fact is absent must have a defined result (the ceiling stands in for
the host clauses; an absent viewer fact is the existing non-finite path,
which never selects), and a static realization states whichever half it had.
The ladder types move out of cartography (they depend only on
`sceno::Representation`) when stage two opens. `ProjectedItem.channels` is
the natural carrier for the disclosed pair if the stretch is taken; its doc
already reserves it for host-side signals a remote viewer cannot read.

**Served endpoint selects at a declared zoom - ruled and landed 2026-09-01.** Found the
same day: `MereHost::local_request` builds the offer's score through the
strategy at 1280 x 720, zoom 1.0, no extents, so the ladder's Card rung fails
`ScreenWidth > 0` and every item scores Glyph, while `build_snapshot`
hardcodes every item as Card at 240 x 112. The offer and the snapshot
disagree on every item; nothing breaks because endpoints ignore the request
score for shape, but the wire carries two answers. Ruled: the served snapshot
selects from the ladder at declared zoom 1.0 with the served footprint as the
measured extent, and the hardcode goes. That makes stage one's "one live host
selects from it" true of the product endpoint, and the offer and snapshot
agree. A3 stage-one task; the landing is recorded in Progress.

**What mer3ly is, clear-eyed - recorded 2026-09-01.** Mark's framing when
ruling the two 2026-08-16 leftovers below: the graph view of an index of his
own site is a toy, and a fine one, whose purpose is stress-testing an embedded
canvas. There are zero consumers for sharing how people arrange it, and no
need for any. The feature would have real utility over a real dataset, which
needs other embeds and other data than descriptions of his GitHub repositories.
Recorded beside the 2026-08-16 consumer ruling above because the two must be
read together: mer3ly is live, public, pinned, and the shipping prover of the
receipts plan, and that is what makes it authority-grade; what it is not is
evidence of product demand for the features its receipts prove. Whether that
qualifies the consumer ruling's "its asks open gates" is not settled here; it
is raised in Progress for Mark.
*Settled 2026-09-30.* Mark, when opening the mer3ly.net canvas assessment:
"my feeling is, the site counts as a consumer, but it should be consuming or
creating stack capabilities, not special exceptions." mer3ly's asks open gates,
but only for capabilities that another consumer could use unchanged. Recorded
as Ruling 1 of mer3ly's `docs/2026-09-30_graphshell_site_canvas_plan.md`, which
is that objective's assessment.
*Added 2026-10-01.* Rulings in that assessment that land in mere:
- A typed fold is promoted onto sceno's 0.0.4 line, with mer3ly as the forcing
  consumer. Mere's native canvas, through pictograph's existing
  `forme::FoldRecord` projection plus a product trigger and folds in
  `MereHost` scenes, is the second (Ruling 11).
- *Ruled 2026-10-08, the fold fact's promotion (site S5, Rulings 149 to 153
  there).* Two questions put to Mark when the site began S5, the fold fact's
  data layer, with the native canvas left to a later phase (its Ruling 152).
  **A pending second consumer.** The catalog's promotion rule says every
  promoted addition "still requires the second heterogeneous consumer"; the
  native canvas, named by Ruling 11, comes in the native phase. Options:
  promote, the record naming the native canvas and marking it pending until
  the native phase (recommended); land it provisional, promoted only when the
  native canvas consumes it; hold promotion until then. Mark: **"Promote,
  marked pending (Recommended)"**. *Follows:* the catalog's addition record
  names the native canvas as pending and is checked again when the native
  phase lands; the rule carries a dated note.
  **Whether `forme::FoldRecord` is a duplicate.** The rule removes "the old
  duplicated implementation" on promotion; FoldRecord stays, moving to a
  view-curation leaf crate in the native phase (Ruling 153). Its members are
  kernel graph-member UUIDs and it persists as view curation (pandect
  `ViewIntent.folds`); the fact's members are scene `InstanceId`s and it
  belongs to a served scene and its trace; a host lowers the one into the
  other (Ruling 149). Options: not a duplicate, the authored record and its
  realization (recommended); a duplicate whose retirement the native phase
  plans. Mark: **"Not a duplicate (Recommended)"**. *Follows:* the record
  names mer3ly's copies as the duplicates that retire (`visibility_diff` in
  `repos/mer3ly/crates/repo-graph`, the `["fold",1]` channel, the `+N` count in
  `assets/projection-proof.js`), in the site's follow-up to S5. *Open:*
  pictograph's `FoldProjection` summary logic overlaps the fact's summary
  stand-in; whether it retires is native-phase work, not yet ruled.
- The two-reading matrix's contract types go into cartography, and its
  derivation into a cartography sibling crate. Both mer3ly's copy and
  `ports/gazette/src/ledger.rs` retire, as the catalog's
  removal-on-promotion rule requires (Rulings 10 and 12).
- *Amended 2026-10-01 (Rulings 19-20 there):* there is no sibling crate. The
  matrix's types and a thin adapter go in cartography beside `src/adapters/`,
  and the cell derivation goes in the scenes family, after the adapters' own
  pattern. Cartography's "stays contract-only" comment (`Cargo.toml:18`)
  predates `graph-layout`'s retirement at `739c87f4` and is stale. The fold
  fact names its stand-in: either a member or a synthetic summary.
- *Added 2026-10-08 (site Ruling 142):* the types move too. The matrix's
  types and its pure derivation are `scenomise::matrix`; cartography's
  `adapters/matrix.rs` re-exports them and adds a `two-reading-matrix`
  profile beside the unchanged one-authority `matrix` profile.
  graphshell-client's `frozen.rs` gains `FrozenGrid`, the two-axis table.
  Gazette's `ledger.rs` stays until the dramatis split (the dramatis repo
  plan's D7), held to the shared derivation by `tests/ledger_parity.rs`; at
  the split it retires onto `scenomise::matrix` and `FrozenGrid` (site
  Ruling 143). Landed by the mer3ly site
  session as its S2, `96fbe064` to `3620cd3b`; its Rulings 142 to 147 are in
  mer3ly's `docs/2026-09-30_graphshell_site_canvas_plan.md`.
- Separately, A4's text here runs together two histories that the assessment
  separates: authority-revision history, the site's checkpoint slider, and
  scene-edit history, `projection-proof`'s chained trace.
- *Added 2026-10-08:* the scene-edit history now has a portable form.
  scenotime's `SceneTrace` (`src/trace.rs`, `SCENE_TRACE_VERSION = 1`) is a
  pure record of a base and steps (a label, an optional `SceneDiff`, an opaque
  host note), validated as a chain and replayed by `snapshot_at(n)`; moving
  through it and truncating on commit is `edit_history::History<SceneTrace>`
  (scenotime `tests/history.rs`), so the Scenograph editor plan's one history
  serves it. A capture of it travels beside chirograph's `ProjectionCaptureV2`
  (`capture.rs`: an optional `score` and a `CaptureAuthorityV1`). Landed by the
  mer3ly site session as its S4 and S6, ending at `a6014bcb`; its Rulings 130
  to 133 are in mer3ly's `docs/2026-09-30_graphshell_site_canvas_plan.md`. The
  authority-revision history stays apart from it.
- *Added 2026-10-08:* the authority-revision history has its form too:
  `scenomise.host-dataset/v2` carries a dataset per revision (v1 stays
  readable), `scenomise::history` classes each item as added, updated, stable
  or removed between revisions, and cartography's `adapters/changes.rs`
  reads it. It stays apart from `SceneTrace`. Landed by the mer3ly site
  session as its S3, `1a53836b` to `1adedb50`.

## Progress

- **2026-09-05:** cross-plan review names Distillery W3 as the prospective
  schematic proof consumer. W1's cleared repository-move prerequisite is not a
  W3 or A5 receipt. Existing `sceno::RoutedRelation` already carries full
  polylines; the W3 entrance review must distinguish route realization from
  any missing reusable port-anchor semantics. Every portable addition still
  requires the catalog's independent heterogeneous consumer proof.

- 2026-08-15: plan founded from the projection grammar report (pass one: six
  specification grammars and the five gap anatomies; pass two: the
  Mosaic/Gosling/Penrose/Bluefish/GoFish tail survey and the
  what-a-spec-means typology, the latter arriving from Mark's notes and
  verified against the papers). The report artifact was updated the same day
  with both passes. Landing sites verified against the tree: score shape read
  from `sceno/src/score.rs`, release rulings D1-D4 from the 0.0.3 plan,
  expansion lanes L1-L5 from the brief, isometry's authored pins and P3b's
  open LOD half from the proofs plan, genet component inventory
  (cambium family, genet-probe, livery) from `repos/genet/components/`. No
  code target started.
- 2026-08-15: **A0 landed.** Catalog's external-systems section split into
  two shelves (renderers/toolkits/foundations, then specification languages and
  design solvers) with all eleven report systems cited and one-line transfers.
  Source URLs checked before writing: two guesses were 404s (`uwdata.github.io/draco2`,
  `uwdata.github.io/gemini`) and were replaced with the live repos; GoTree, ATOM, and
  GoFish resolved to an ACM DOI, the MSR publication page, and the MIT VIS page.
  The report artifact is cited by name and date, not linked: mere is a public repo and
  a claude.ai artifact URL is private, so A0's done-condition (findable without leaving
  the repo) is met by pointing at this plan instead.
  The collision note covers **Mosaic** as well as Penrose: the catalog's own tiling row
  names Mosaic as a packing variant, and it has no implementation in the tree.
  DOC_README: founded the missing catalog entry (the catalog had no index line at all)
  and recorded A0 on this plan's entry.
- 2026-08-16: correction. A0's first pass claimed `graph_layout:penrose` was absent
  from the tree and rewrote the note around `PenroseAdapter` / `penrose.default`. That
  was wrong: the claim came from a grep truncated by `head -20`. Both ids are real and
  distinct, `graph_layout:penrose` being the `LayoutCapability` registry id
   (`crates/canvas/arrangements/src/registry.rs:358` *(historical citation)* <!-- doc-audit: historical-path -->) and `penrose.default` the cartography
  adapter's projection id. Plan text and catalog note restored to the registry id.
- 2026-08-16: **mer3ly ruled authority-grade; A6 founded.** Survey first
  (`../research/2026-08-16_mer3ly_stack_consumer_survey.md`), then Mark's
  ruling: mer3ly's asks open gates. A6, the placement seam, is the new target
  and the new opener, because the site's two paths never meet and that single
  fact is what leaves A1, A2, and C3 each waiting on a consumer that does not
  exist. Re-gated A1 (follows A6; mer3ly ships ensure/encourage with no
  satisfaction record), A2 (L3's gate met by URL-hash scene sharing; resolution
  strategies still unforced), A3 (mer3ly does not force it; the plan's premise
  stands), A4 (mer3ly has the record, not the spec), B1 (mer3ly is the standing
  argument, not a counterexample), C3 (identity plus collidable is the shipped
  minimum). Two open rulings recorded: whether the site's wire is better
  understood as an index than a promotion candidate, and the app-embedding
  ambition that bears on it.
- 2026-08-16: **index ruled; shelfmark founded as a format note.** Mark ruled
  the brief's five questions: note now scoped Mere-level; one record shared
  between A6's sidecar and the citation delta, grounded in the stack; incipit
  as home (scoped: 127 lines, serde + uuid only, register pairs with the
  shelfmark; chirograph fallback if the opaque-sections indirection fights
  the one-record rule); checkability in v1; the name is shelfmark. The
  format note (`../technical_architecture/2026-08-16_shelfmark_format_note.md`)
  is the authority; the founding in code waits for the second shipping
  consumer. A6's task text now carries the one-record constraint.
- 2026-08-16: **A6 first slice landed: the score can hold a placement, and
  every solver honors it.** The audit found three silent displacement paths,
  not one. `Placement::Coordinate` was honored by `Geographic` and `Hulls` and
  silently discarded by `Spiral` (ordinal wins) and `Board` (rank wins), and
  `relax` moved every item with no notion of a pin at all. All three were the
  same bug wearing different clothes: the person said where, the layout
  answered somewhere else, and nothing said so.
  Decision, write-back versus sidecar: **sidecar, keyed by `SourceRef`**. The
  stack argued it, per the one-record ruling. A shelfmark cites by source
  identity, so a per-item field would mean shipping a whole score to cite one
  pin, defeating the index; item indices move when the authority changes and
  source ids do not; and pins are sparse, so absence is the free common case.
  `Score.holds: Vec<HeldPlacement>` is therefore the shelfmark's `placement`
  delta section verbatim, one serialization of a pin.
  Landed: `sceno` gains `Hold` (`Anchored` = encourage, `Pinned` = ensure),
  `HeldPlacement { source, at, hold }`, `Score.holds` (serde-default) and
  `Score::hold_for`; `SCORE_VERSION` 1 -> 2, because an adapter that only knows
  v1 must reject rather than silently drop the one field that carries what
  someone asked for. `scenomise::solve` consults `hold_for` ahead of all four
  arrangement families; `relax_holding` leaves held instances where they stand
  while they still push their neighbours (the asymmetry is the meaning of a
  hard hold), with `relax` preserved as the nothing-held wrapper;
  `pinned_instances` resolves holds to instances through interned sources, so
  a source appearing twice pins both.
  Receipts: 8 new tests, `sceno` 19 -> 21 and `scenomise` 15 -> 21, all green,
  plus `chirograph` (21) and `mere-cartography` (25) unaffected. A v1 score
  still loads and loads as "nothing held"; an unheld score places exactly as
  before, so no existing determinism receipt moves.
  At this point satisfaction reporting was still A1 and crate versions stayed
  at 0.0.3. Later entries close A1. The 2026-08-19 reconciliation opens the
  0.0.4 development line and Score v3; nothing was republished. Mer3ly's own
  seam (its live path still never builds a score) was the next slice.
- 2026-08-16: **A6 mer3ly half written and verified, parked on a push.** The
  seam now exists in code: `PlacementDelta` deserializes the sandbox's own
  scene state (extra fields ignored, so a caller hands over the whole shared
  record), and its `holds()` maps that placement onto `Score.holds`. A manual
  pin is hard in the live path, so it records as `Hold::Pinned`; under
  `anchored` motion the visitor has already said best-effort, so it records as
  `Hold::Anchored`. The class is recorded rather than left to be re-inferred
  from a spring stiffness, which was the point. `portable_projection_holding`
  and `portable_projection_with_placement_json` carry it, with a
  `portable_projection_with_placement` wasm export for the sandbox, and a pin
  naming a node the authority lacks is refused as a broken citation rather
  than solved into a scene that quietly omits it.
  The receipt half matters as much: `consume_portable_projection` now checks
  every hold against the scene **as solved** (not after the trace, which
  deliberately moves Turnstone) and reports `honored_holds`. A forged artifact
  claiming a pin the scene does not satisfy is rejected. That makes a cited
  pin checkable, which is the shelfmark's `expects` property arriving one
  layer down.
  Receipts: mer3ly-repo-graph 14 -> 19 tests, site workspace 49 green,
  including a round trip that pins a node under `Spiral`, the exact family
  that used to discard an authored coordinate in silence.
  The version bump proved itself immediately: `tests/m7_showcase.rs` asserted
  `score.version == 1` and failed loudly the moment the contract moved, which
  is a consumer noticing rather than silently dropping a field.
  **Blocker, and it is only this.** Mer3ly resolves mere by git rev, pinned at
  `8a7ede70`, and the contract half sits in unpushed `3716c601`. A local
  `[patch]` does not substitute for a rev-pinned git dep (it resolved the old
  rev with no warning), so verification ran against temporarily repointed path
  deps, now reverted; no local path is left in a manifest. Landing sequence:
  push mere, bump mer3ly's rev, reapply the parked diff, rebuild the committed
  wasm, then wire the sandbox's share control to the new export. The last two
  are deliberately not done, because a wasm rebuilt against a local path is
  not a thing to commit.
- 2026-08-16: **A6 closed; A1's first slice landed.** A6's mer3ly half shipped
  after the rev bump: `PlacementDelta` maps the sandbox's pins onto
  `Score.holds`, `consume_portable_projection` checks each hold against the
  scene as solved and reports `honored_holds`, and an `export projection`
  control sits beside `share scene` rather than replacing it, because a share
  is a citation and a score plus snapshot has no business in a URL. Verified in
  a browser, not only in tests: pin a node, press export, "1 pin honored"; and
  driving the export directly returns score version 2, a hold recording
  `Pinned` at (-137.5, 88.25), and that node placed at exactly (-137.5, 88.25)
  under `Spiral`. The Wasm ceiling was raised deliberately, with all three
  measurements recorded beside it (858,444 baseline / 932,204 stripped /
  958,016 full) and the finding that no variant fit the old bound.
  A1 then opened on a gap A6 itself created. A hold naming a source the score
  never placed was dropped in silence, which is the same failure as moving a
  pin and saying nothing; a probe test confirmed it before anything was built.
  `Scene.unmet_holds` now carries those violations as full `HeldPlacement`
  records, so a viewer without the score can say what was asked and where, and
  `SceneTables.unmet_holds` carries them through `from_dense` onto the wire.
  Encourage-class holds are excluded on purpose: an anchored home that goes
  unplaced is best effort working as designed.
  Receipts: sceno 21, scenomise 24, scenotime 19, with chirograph (21) and
  mere-cartography (25) unaffected. Two of the new tests serialize the snapshot
  and read it back, because A1's done-condition names a *remote* viewer, and
  one strips the field to prove an older wire still loads.
  Still open in A1: the only unsatisfiable case today is a hold on an absent
  source. Displacement by a consumer's own physics or a viewport fit is not yet
  reported, and mer3ly's `honored_holds` remains a site-local receipt rather
  than a promoted one. C1 (satisfaction in host chrome) now has something real
  to render.
- 2026-08-16: **B3 closed, and the answer was better than either branch the
  target anticipated.** The question was whether Livery's property DB classes
  properties by what they invalidate. It does not: 123 entries, eight keys,
  none of them an invalidation class, and Livery's own invalidation machinery
  is selector-scoped, a different axis. The target then expected either "close
  it" or "weigh adding it". Neither fit, because genet already has the
  layout/paint rhyme working: `LayoutDamageClass` resolves None / PaintOnly /
  Relayout, `RestyleOutcome::needs_relayout` reads `RestyleDamage::RELAYOUT`,
  and a test asserts a color swap is repaint-only. It is borrowed, not owned.
  Stylo computes the damage in `compute_style_difference` from its own
  per-property metadata, and genet's `compute_layout_damage` hook is a stub
  returning default. A first read of that stub suggested the classing was
  unimplemented; checking where damage actually comes from corrected it.
  So the finding is a dependency, not a gap: every incremental-layout receipt
  that skips layout for a paint-only change is currently a receipt for Stylo's
  property table, and the classing leaves with the fork unless the DB grows a
  home for it. Recorded in genet's cutover-plan deferral register as a
  compounding item with the retirement stage as its forcing consumer, on that
  plan's authority rather than the report's, which is exactly the bar this
  target set. Mapbox's layout/paint split is named there as the shape to
  borrow when it is built.
- 2026-08-16: **B1's first slice landed: the receipt shape exists, with two
  corrections it forced.** `graphshell_client::frozen` turns a `Scene` into
  navigable semantics: a document name, a generated summary, one entry per
  visible instance with a coarse role (symbol / object / live content, mapped
  down from `Representation` because a reader cares about kind, not rung),
  relations resolved to names at both ends, and a `rows()` tabular alternate.
  It produces a *structure*, not markup, so a host renders it into a genet DOM
  tree, an AccessKit node tree, or an HTML table, and the receipt is assertable
  in a test with no browser. That also keeps a DOM engine out of the client.
  **Correction one: B1's home is mere, not genet.** Genet has no dependency on
  scenograph and mere depends on genet, so a `Scene`-to-semantics converter
  cannot live in genet without inverting the stack. Track B's preamble is
  corrected: the realizations *target* genet's surfaces rather than living
  there, which is the direction mer3ly already proves.
  **Correction two: a scene carries no names.** `ProjectedItem` identifies
  itself with a `SourceRef`, an address rather than a label, so an accessible
  form cannot be derived from a scene alone. The names already exist one layer
  out, on `chirograph::PresentationSemantics.label`, so `freeze` takes a
  supplied lookup and falls back to the source id, and `unnamed` counts the
  fallbacks so a receipt can state how legible the scene actually was. This is
  the report's claim landing concretely: no surveyed grammar treats the
  accessible form as first-class, and the contract shows it.
  A1's violations are carried into the frozen form deliberately. A sighted
  reader can see a pin sitting in the wrong place; without this the frozen
  realization would be the one form that cannot say so.
  Receipts: six tests, run against the real P5 `coastal_map.json` fixture
  solved through `scenomise` (added test-only) rather than a hand-built
  stand-in, covering enumeration, fallback naming, summary-matches-listing,
  invisible-item omission, relation naming, unmet placements reaching the
  tabular alternate, and determinism. graphshell-client 13 -> 19.
  Still open in B1: rendering the structure into an actual DOM tree, the
  AccessKit screen-reader traversal, and the genet-probe scenario asserting the
  semantic tree. The shape they all need now exists and is stable.
- 2026-08-16: **B1 carried to its buildable edge; A1 closed; A2 half-open.**
  B1 gained a renderer: `FrozenScene::to_html` emits the graphics-document
  anatomy with WAI's long-form alternate as a real table, and the tests parse it
  back through genet-scripted-dom rather than string-matching, so a malformed
  tag fails as a missing tree node. One of them is a hostile name, because a
  projection whose accessible form can be broken by a source called `<script>`
  is not accessible and is a hole besides. It also gained an AccessKit
  projection into uxtree, behind an off-by-default feature so a client that only
  reads scenes carries no tree builder, and probe-resolution proof that every
  instance is reachable by carried id and by announced name.
  What B1 still lacks needs a host, not effort: no OS screen reader has been
  driven, and no probe scenario can run until something renders this realization
  inside a frame pump. The precedent this target named is a manual checklist
  written for meerkat, an app that no longer exists, so it could not be followed
  literally.
  A1 closed by merging its two remaining questions into one field.
  `Scene.honored_holds` records the positive half bound to instances, which both
  promotes the receipt mer3ly was recomputing and closes a real footgun: plain
  `relax` used to drag an ensure-class placement away in silence and now cannot,
  whatever entry point a caller reaches for. Anchored holds stay out, because
  recording a suggestion as honored would invite a later pass to treat it as
  binding.
  A2's serialization half landed as `chirograph::Selection`, carrying the
  producing view and its targets, with the resolution strategy deliberately
  absent until two coordinated views exist. The `source` field is mandatory now
  rather than later because crossfilter is defined as "every brush but this
  view's own", which is unanswerable without it, and a field added later would
  be missing from every link already in circulation. The shelfmark note's
  reserved `selection` section is now defined against that record.
- 2026-08-16: **C1 investigated and blocked, on the same wall B1 hit.** A1 has
  landed, so there is finally something real to render: `Scene.unmet_holds`
  crosses the wire, `Scene.honored_holds` names the satisfied pins, and the
  frozen realization already lists violations in its tabular alternate and
  announces them in its AccessKit tree. The target says surface the same
  distinction in cambium host chrome. Cambium cannot: it has no `sceno`
  dependency, so it cannot see a `Scene` at all, and nothing in mere consumes
  cambium either. Verified from the tree in both directions, not assumed.
  This is Track C meeting the correction Track B already took. A host renders
  *into* cambium's surfaces; the code that knows what a hold is lives mere-side.
  So C1's home is a mere-side host built on cambium, and no such host exists
  today, which is the same missing piece that blocks the driven probe scenario.
  One consumer could take it now without a new host: mer3ly already reports
  "1 pin honored" on its export control, and could report the unmet count
  beside it rather than only failing the whole artifact, since the snapshot now
  carries the violations. That is C1's substance in a web page rather than
  native chrome, and it costs another rev bump, wasm rebuild, and public deploy,
  so it is Mark's call rather than a quiet extension of this target.
  Not built: a speculative cambium surface with no consumer. That is exactly
  the work the five gated targets are being held back from, and C1 does not get
  an exemption for being adjacent to something that just landed.
- 2026-08-16: **C1 landed, after a wrong "blocked" call that Mark caught.**
  The earlier entry claimed C1 could not be built because nothing in mere
  consumes cambium. That was false. `ports/graphshell/web` depends on cambium,
  sceno, and graphshell together, and `ports/graphshell/src/web.rs` already
  renders a mounted `SceneSnapshot` in `remote_scene`. The claim came from a
  grep with relative paths run after the shell cwd had been reset, so it
  searched a directory with no `ports/` in it and returned nothing; an empty
  result was read as an answer. That is the wrong-cwd trap, hit one turn after
  citing it about a Cargo patch table.
  Chasing the real host then exposed a second, worse gap: `SceneTables` carried
  `unmet_holds` but not `honored_holds`, because the honored half was added to
  `Scene` after the unmet half had already been wired through `from_dense`. So
  A1's done-condition was only half met on the wire. A remote viewer could see
  that a pin failed but not that a pin succeeded, leaving every unremarked item
  ambiguous between unpinned and pinned-and-honored. Both halves now cross, with
  one test asserting them together rather than separately, since testing each
  alone is what let the gap open.
  C1 itself: `graphshell_client::frozen::Satisfaction` reads both halves off a
  snapshot and answers the two questions a host has, whether a given instance is
  holding an authored position and what the chrome line should say. It lives in
  the client rather than the host because `web.rs` is
  `#![cfg(target_arch = "wasm32")]` and no native test can reach it; a summary
  nobody can test is one that quietly goes wrong. The host draws a held edge
  outside a pinned card, so an item sitting where a person put it no longer
  looks identical to one the arrangement happened to place there, and the chrome
  carries the line. An unpinned scene says nothing at all, because zero of zero
  is noise.
  Receipts: three native tests on the summary, one wire test on the pair, and
  `cargo check -p graphshell-web --target wasm32-unknown-unknown` for the host,
  which is the only check that covers a cfg-gated file at all.
- 2026-08-19: **protocol and catalogs reconciled.** The Scenograph family
  moves to the unpublished 0.0.4 development line; Score v3 renames portable
  regular-cell Board to Grid, while the local categorical layout is Columns.
  Chirograph 0.0.2 renames the retained wire artifact to
  `ProjectionCaptureV1`. Snapshot validation now rejects an honored-pin claim
  whose instance is absent, names another source, is not ensure-class, or is
  not at the claimed position. The projection grammar governs primitives;
  the content catalog depends on it and collects complete scene recipes.
  Rosette is recorded as landed through Knot over poem and lyric datasets.
- 2026-08-19: **B1 probe scenario ran for real; the no-host premise died the
  way C1 death did.** Mark asked whether graphshell was built on cambium;
  re-checking from the tree found what the wrong-cwd grep had hidden, and the
  right host was turnstone: it implements genet-probe Automatable with real
  ProbeSurfaces, carries a .scn scenario lane whose assert-a11y reads a
  stitched AccessKit tree, and already depends on graphshell-client, sceno,
  and cambium.
  Landed in turnstone (49e179e): disclose_scene extracted from the endpoint
  snapshot so the frozen form freezes exactly what a remote peer is served,
  one recipe rather than two that drift; a Frozen Projection pane in cambium
  chrome with instances by name and carried data-source-id, relations named
  at both ends, the WAI summary, unmet placements, and C1 satisfaction line;
  and an a11y arm built by FrozenScene::to_ux_tree itself, so assert-a11y
  reads the realization actual tree rather than a summary of one.
  Receipt: scenarios/frozen_projection.scn, RESULT ok, self-driven, offline
  mere:// nodes, captures in Code/testing/turnstone/frozen_projection. The
  capture shows the pane beside the live canvas: 14 items, 17 relationships,
  harbor-notes and beacon-notes by name.
  All three automated verification legs are now done: renderer proven by
  parsing, AccessKit tree driven through a real host, probe scenario driven.
  At this checkpoint the manual OS screen-reader pass was still open. Turnstone
  `648bf19` later supplied the definitive routed receipt and supersedes this
  historical remainder.
- 2026-08-19: **A3 stage one landed; P3b selection is declarative.**
  `mere.graph-representation-registry/v2` puts ordered representation rungs
  on each class profile. A rung names its selected `sceno::Representation`
  and a conjunction of measure, operation, threshold, and hysteresis
  conditions. The initial policy is concrete: focus selects `LivePane`; a
  measured, recent item at zoom 1 or closer selects `Card`; `Glyph` is the
  fallback. Cartography resolves graph classes in registry order, normalizes
  durable visit recency, and consults the prior score only for hysteresis.
  Graphshell's live web host now supplies its camera zoom, measured node
  extents, focus, and prior score, and refreshes the score after local view
  commands. `sceno` is unchanged: it still records only the chosen rung.
  Deterministic receipts prove identical placement at two zooms with different
  rungs, focus overriding distance, and a card holding through the declared
  0.1 hysteresis band. This closes A3's host-side grammar, not P3b's visible
  rendering: the canvas still stores the selected score without painting
  distinct glyph/card/live-pane realizations from it. Portable conditions stay
  gated on a remote client that needs to re-select.
- 2026-08-19: **P3b's local renderer half landed.** Canvas rebinds the score's
  opaque `mere.graph` sources to node keys and its real frame assigns distinct
  `Glyph`, `Card`, and `LivePane` face classes. The glyph hides the caption,
  the card retains the established labelled node, and the live-capable rung
  receives a frame plus caption emphasis. The measured footprint still owns
  all geometry, and the live-capable treatment does not claim embedded browser
  content. Tests drive score → Canvas frame → DOM class and computed style,
  and prove a whole-graph swap clears the rebound score state. This closes the
  local forcing consumer for A3 stage one; A3 stage two remains gated on a
  remote client that must perform its own re-selection.
- 2026-08-19: **A4 + C2 landed through Graphshell's Canvas arrangement
  switch.** `scenotime::TransitionSpec` declares duration, easing, class
  windows, and stable stagger ratios over a validated `SceneDiff`.
  `TransitionSchedule::sample_at` is a pure function of that data and elapsed
  host time. The default stages exits, updates, and entrances without adding
  motion to `Arrangement` or changing direct diff application.
  Graphshell builds the arrangement change as a real scene diff, advances it
  from browser frame timestamps, and feeds sampled positions through Canvas's
  preview buffer. Completion returns to the ordinary strategy apply, which is
  still the authority that seeds physics. The host clock explicitly excludes
  paused intervals; identical timestamp sequences replay identically.
  Receipts: 27 scenotime tests, 179 Canvas tests, the focused Graphshell host
  clock test, and a wasm32 build. A headed browser drive reported
  `Arrangement changing to grid.default` before
  `Arrangement set to grid.default`; the start and final canvas captures were
  distinct. Consumers that do not construct a schedule continue to snap.
- 2026-08-19: **A3 stage one arrived under P3b's flag, landed by a sibling
  session and verified here against the tree.** The P3b check Mark ordered
  found the gate had fallen the same day: cartography's v2 representation
  profiles now declare `RepresentationCondition { measure, operation,
  threshold, hysteresis }`, which is Gosling's anatomy verbatim, serde-derived
  so the conditions are data, with the measure split A3 asked for
  (ScreenWidth/ScreenHeight are realization-space; ZoomLevel, Recency, Focused
  are view facts). Hysteresis relaxes the threshold only for the rung already
  selected, so a card holds to zoom 0.9 while a glyph still needs 1.0, and
  `profile_ladder_selects_from_data_and_retains_through_hysteresis` is the
  no-flicker receipt A3's validation named. The renderer half consumes the
  selected rung in the real canvas frame. What was the hint: the frozen-pane
  captures showing plain nodes flip object to symbol between runs, which was
  rung selection moving with recency and focus.
  Stage two, portable rung conditions traveling beside the score, remains
  gated exactly as written: the P3b entry itself says remote re-selection
  waits on a real remote consumer, and nothing about stage one changes that.
- 2026-08-20: **C3 and L2 landed through Isometry and Woodshed.** The two
  consumers forced a separate backdrop table rather than an item kind or a
  generalized Region. Each backdrop carries a source reference, coordinate
  space, transform, footprint, open appearance kind, visibility, and collision
  participation. Table order is back-to-front and the whole table paints before
  graph content. Pointer transparency is structural: selecting part of a map
  remains an ordinary item over the environment, so intent routing keeps one
  `InstanceId` path.
  Isometry's authored board now supplies `isometry:tile-board` behind its
  selectable tile items. Woodshed's Stage supplies `woodshed:stage-floor`
  around its arranged cards. Both cross `SceneSnapshot`; stable backdrop slots
  also cross `SceneDiff`. Graphshell's remote canvas derives transformed
  backdrop bounds and deterministic fallback paint from the snapshot before it
  draws items. Scenomise treats collidable backdrop footprints as static
  placement obstacles while visibility remains independent.
  Receipts: 22 sceno tests, 31 scenomise tests, 30 scenotime tests, 116
  Graphshell tests, Isometry's views and Graphshell endpoint tests, Woodshed's
  `stage_backdrop` wire test, and the Graphshell wasm32 build.
- 2026-08-20: **B2 landed through Turnstone Knot authoring.** Save, Resolve,
  and Run expose the projection instance and intent in the retained DOM. The
  session records the endpoint's typed intent result as visible scene text.
  The shared genet-probe scenario selects Resolve by `InstanceId(1)`, asserts
  the accepted result, and verifies the resulting transclusion before running
  the document. Receipts: the focused `InstanceId(17)` resolver unit test and
  the headed `knot_authoring.scn` run, `RESULT ok`.
- 2026-08-20: **B1 closed. Mark ran the manual screen-reader pass, and it
  earned its place twice before it passed.** Preflight found the OS bridge
  had never existed: accesskit was in-process only, so assert-a11y read a
  complete tree while Narrator saw a bare window; turnstone gained
  shell/a11y_bridge.rs (accesskit_winit installed before first show, shared
  slot for cross-thread activation, thirty-frame cadence, ActionRequests
  dropped honestly rather than routed nowhere). Attempt one then dead-ended
  at the omnibar: a live UIA dump showed every projected node boundless
  (rect -Infinity), which Narrator treats as off-screen and stops at, while
  the in-process integrity test written on the spot passed - a structurally
  sound tree that no reader could walk, the exact gap between iterating a
  node list and being a UIA client with geometry. Window-extent bounds fixed
  it. Attempt two: the full walk - window, chrome, canvas group, the
  Disclosed projection document with its summary spoken, twelve items by
  name, all twenty-seven relationships to the last entry, in the operator's
  words "made it to 27 of 27". Receipt at turnstone
  design_docs/2026-08-20_screen_reader_pass_receipt.md (05a2ec8), replacing
  the meerkat-era checklist as the worked example. Recorded follow-ups, none
  speculative: route ActionRequests when a consumer asks for operable rather
  than readable, per-node rects from the surface plan, and a label for the
  frisket root group that announces as blank.
- 2026-08-20: **The routed half landed: a screen reader's Enter reaches the
  app.** Mark asked for it, which is the forcing consumer B1's follow-up was
  written to wait for. Turnstone e117f27: a route table built beside every
  pushed tree (ids are one-way path hashes, so routing cannot be parsed back
  out of the tree), a frozen-projection instance selecting its member in the
  graph, the omnibar opening, both through the same update spine a keypress
  uses; unrouted requests land as interaction-missed a11y-action, the pointer
  miss's exact vocabulary. Eight tests. Linking them exposed two real defects:
  the livery cutover had shipped a second AccessKit platform stack beside the
  bridge's (two UIA providers in one process, deduped), and the grown graph
  crossed the preview MSVC linker's PDB limit. Both fixed and recorded in the
  turnstone receipt (`648bf19`). **Reconciled 2026-08-23:** that clean Turnstone
  state is the definitive routed screen-reader receipt; B1 has no remaining
  manual leg.
- 2026-08-23: **A2 resolution closed through the first projection-receipts
  wave.** `chirograph::CoordinatedSelection` carries named focus/filter/brush
  clauses and explicit `single` or `crossfilter` resolution. The mer3ly
  spatial and Matrix views prove both clauses in a headed browser: each view
  applies the foreign clause and excludes its own; deterministic Shelfmark
  serialization restores both clauses; removing the Matrix clause restores all
  18 spatial actors. Gazette's Ledger independently replays the same record.
- 2026-08-25: **FT7 closed.** The FT1 Matrix preserves its exact
  `InstanceId -> SourceRef` mapping, including repeated source instances,
  through `LocalCarrier`, an admitted Graphshell `MemoryTransport` session
  using Notochord/personae policy to a source-free viewer, and the
  `FrozenScene` semantic-table realization. The Mere platform implementation
  is `302bbe72d7597b7573e14199ce926bd3b03eea7f`; the Mer3ly consumer is
  `4c42847272489c41a20dc515884e83f3b413059a`.
- 2026-08-25: **FT8 closed.** Retinue Signalman's Network face derives its
  Sprigging marks and Cambium retained semantic targets from one
  `GraphCanvasSwatch`. Its focused receipt proves those marks and ordinary DOM
  controls share one focus order, pointer-capture route, AccessKit tree,
  genet-probe address space, and `DesktopState` action model. The full locked,
  offline Signalman suite passes 50 tests at Retinue `8cea8f9`. The receipt
  reused Cambium's existing `graph_canvas`; it did not force a generic
  scene-hosting abstraction or keyboard-navigable Cambium Matrix. A3 stage two
  and A5 remain gated by their own consumer rules.
- 2026-08-29: **governing documents reconciled against landed code and the
  promotion rule** at Mere `caf19a014766ae08e3231d9963d5edfaec945dd7`. The
  catalog now describes Score v4's actual arrangement surface, distinguishes
  C3's closed minimum backdrop contract from richer raster and scalar field
  work, and states that the promotion suite is an evidence harness rather than
  a forcing consumer. A5 now waits for a named first proof consumer and still
  requires a second heterogeneous consumer for every portable addition. The
  shelfmark/index ruling is labeled resolved; the embedded-app question
  remains open.
- 2026-08-31: **Graphshell Projection Editor component and first authoring
  loop landed through Mere `77369f6f0c03b301c398ad65107114080c4ba630`.** The
  host-neutral `ports/graphshell/src/projection_editor.rs` now models source
  and domain binding, reading, encoding, arrangement, interaction,
  appearance/realization, and provenance as an editable definition with
  field-level validation, panel taxonomy, reducer actions, deterministic JSON,
  and a sink-only save boundary. Its seven tools are `workbench::Tile`s in an
  open Graphshell content lane, so selection and typed tearout use the shared
  reducer without granting graph or endpoint authority. A focused standalone
  cross-repo harness passed eight tests, including Platen projection, editor
  activation, tearout custody, validation, provenance, serialization, and sink
  refusal. The standalone web manifest now restates its inherited immutable
  Genet patches, and the host `x86_64-pc-windows-msvc` cargo check passes
  offline with the immutable G1 Genet revision. G1's native-only `fontsan`
  policy closes the prior C++/WASI sysroot blocker for the wasm dependency
  cone. The standalone wasm build now passes with an isolated target and the
  matching `wasm-bindgen 0.2.126` package is generated under the ignored web
  package directory. The headed Graphshell browser receipt is now green:
  localhost reported title `GRAPHSHELL H3 READY` and `ready=true`; the editor
  opened the source panel, authored `local-workbench-receipt`,
  `woodshed.practice`, `fixture.projection/workbench`, `practice_title`,
  `tempo`/`difficulty`, `grid.default`/vertical/24, `canvas.points`, title
  `Workbench projection receipt`, and provenance
  `Projection Editor receipt` / `workbench-w4` / `Saved and reloaded in the Graphshell browser receipt`. The deterministic preview was `Workbench projection receipt · read nodes by practice_title · grid.default · x=tempo y=difficulty · canvas.points`. Save reported `saveCount=1`,
  `validation=valid`, `errors=0`, and `Saved · workbench-w4 · 1 save(s)`;
  after mutation, reload restored every authored field and reported
  `Reloaded · graphshell-reference · workbench-w4`. The bound stripped artifact
  is 34,679,440 bytes with SHA256
  `A6A43EA0D1FB510E9EEF897B6C9232BB66F5D1F5927DF54DADE3463F55D3DB85`.
  This closes the browser authoring receipt, while broader headed verification
  and A5 remain open. Woodshed is Workbench's second
  heterogeneous consumer, but it is not itself A5's named portable projection
  proof; A5 still needs a named proof consumer and its per-proof evidence. The
  component follows the component seam in
   `genet/design_docs/2026-08-31_workbench_component_plan.md` *(historical citation)* <!-- doc-audit: historical-path -->.
- 2026-09-01: **C4a landed; A3 stage two tested against it and the gate
  sharpened.** Mere `d10e04da` serves a live Graphshell projection to a
  browser-shaped WebRTC peer (admitted by Notochord, sans-I/O `SessionCore`
  with blocking and event-driven adapters, `LiveEndpoint` moving only for
  admitted intent), tracked in the browser WebRTC carrier plan; C4b, the
  embedded and full-page surfaces, was split out 2026-08-31. Checked whether
  C4b is A3 stage two's remote re-selection consumer: it is not. The served
  rung is endpoint-fixed, the browser's remote view has no camera, and the
  client's only resolution is capability-based offer choice within the rung.
  The gate now reads "a remote viewer with a local camera over a served
  scene". Opened the stage-two fact transport ruling, since `Recency` and
  `Focused` are host facts the wire does not carry. No code changed.
- 2026-09-01: **the open rulings and two deferred questions scoped and
  ruled by Mark.** Stage-two fact transport: evaluation without host facts
  is the baseline, carrying them the stretch; disclosure is a host's choice.
  Apps embedded: two sites, two answers; merelyllc.com hosts Graphshell
  embeds with other applications as served projections, mer3ly.net embeds
  the full applications, and the latter is a new objective needing its own
  assessment. Served endpoint: select from the ladder at declared zoom 1.0
  rather than hardcode Card; code to follow. Two 2026-08-16 leftovers: A1's
  `honored_holds` half is closed, since `Scene.honored_holds` was promoted
  the same day, while displacement by a consumer's own physics or viewport
  fit stays gated on a consumer asking; C1's mer3ly unmet-count beside
  "1 pin honored" is approved and folds into the next mer3ly wasm rebuild
  rather than earning a deploy. Both carry Mark's framing, recorded in the
  rulings section: mer3ly's graph view is a stress-test toy for an embedded
  canvas, not a product pull.
  Raised the same day and answered by Mark's refinement of the embedded-app
  ruling: mer3ly's asks are a prover's until Graphshell supersedes the site
  canvas and becomes the site index, at which point the graph view is the
  site's navigation and its asks are product demand. The 2026-08-16
  consumer ruling stands unchanged; this records what would change its
  character and when.
- 2026-09-01: **served endpoint selects from the ladder: code written,
  receipt written, receipt not yet run.** `MereHost` now builds the offer's
  score and the snapshot from one `served_layout` (phyllotaxis at 1280 x 720,
  every node measured at the served 240 x 112 footprint, ladder evaluated at
  `SERVED_ZOOM = 1.0`), and each served item carries the rung the registry
  selected, Glyph when the score does not name it. The receipt
  `served_snapshot_selects_rungs_from_the_ladder_at_declared_zoom` asserts
  the fixture's visit spread as its control, then web (10 ms) as Glyph,
  receipt (110 ms) as Card, and offer/snapshot agreement on every item.
  Verification so far: the change compiles for its real target
  (`cargo check -p graphshell --target wasm32-unknown-unknown
  --no-default-features --features web`, clean). It could not be *run*:
  `mere_host` is `web`-gated, the default `native` suite never compiles it
  (149 tests pass and prove nothing about this file), and the native
  `--no-default-features --features web` test build fails at link with
  unresolved `canvas` symbols even after `cargo clean -p mere-canvas
  -p mere-cartography -p graphshell`. That failure predates this work: it is
  the invocation the 2026-07-27 H1 receipt ran, and every MereHost-testing
  module (`app`, `capture`, `product`, `mere_host`) is `web`-gated, so the
  whole MereHost test surface has been unrunnable natively on this machine
  for some time without anything noticing. Swept in on the way: the
  `sessions.rs` test module used `native`-only imports under a bare
  `cfg(test)`, which was the first compile error the web build hit; it is
  now gated like its imports. Visible effect of the change, for the record:
  none on the rich viewer, which picks offers by capability and never
  branches on the rung; the frozen realization now calls stale served items
  Symbol and recent ones Object, and the offer's score stops contradicting
  the snapshot.
- 2026-09-01: **the link break chased to its cause; the receipt ran.** After
  a real clean the 103 unresolved externals fell to one symbol,
  `drop_glue<canvas::geometry::CartographyGeometry>`, advertised in
  mere-canvas's metadata and absent from its incremental object cache: the
  shape of rust-lang/rust#86049 (open; windows-msvc; incremental;
  LNK2019/2001 on allocator and drop glue; no cause identified upstream;
  "disable incremental" the standing workaround). Turning incremental off
  for `mere-canvas` alone links: the receipt passes and the full web-feature
  suite passes, 39 of 39, with graphshell still incremental. The fix is
  scoped to the invocation rather than the workspace because a canvas
  edit-loop rebuild measures ~9 s incremental against ~2.5 min without, a
  penalty the Mac and Linux boxes have no reason to pay: `cargo test-web`,
  an alias in `.cargo/config.toml.example`, carries
  `--config profile.dev.package.mere-canvas.incremental=false`, and the H1
  receipt doc points at it. Retire condition: run the bare invocation on
  each toolchain bump; if it links, drop the override. The served-endpoint
  ruling is closed by this receipt; A3 stage one now holds for the product
  endpoint.
- 2026-10-03: **arrangement roles carried in.** The dynamics grammar plan's
  F18–F30 are recorded in the projection grammar catalog:
  - the Free, Anchored and Pinned policies are now the seeded, anchored and
    pinned roles, chosen per item over a recipe default, with seeded as the
    default and an encoded axis pinned;
  - the §6 "Force and constraint" row moved out, and "Settled" joined as an
    arrangement source;
  - §7 reads drag, displacement, pick and stop by role;
  - a scene recipe names an arrangement and, optionally, a dynamics recipe,
    under one binding.

  This plan gains A1's note on the old `Hold::Anchored`, which named the
  seeded role, and the mer3ly finding. Doc-only; no code moved.

- 2026-10-07: **handed off.** Asked what a projection grammar handoff should
  cover (the whole lane; mer3ly's site canvas plan only; the mere grammar docs
  only), Mark answered "mere grammar docs only". The
  [projection grammar handoff](../research/2026-10-07_projection_grammar_handoff.md)
  covers this plan and the catalog: who owns which sections, the open and gated
  targets, the rulings carried in, and the cross-session agreements.

## 2026-09-05 practice workspace browser proof

The current-tree Graphshell practice proof gives one disclosed Woodshed
comparison a small browser workspace: Relations, a pitch-membership Compare
view, and History share typed occurrence/comparison selection. Save and reopen
retain the bounded workspace location and runtime settings only when the
current source identity, revision, occurrence IDs, and complete disclosed
comparison evidence still validate. This is a Graphshell component proof, not
native Woodshed: the same WASM component runs in `web/practice.html` and in
the `web/practice-embed.html` wrapper.

Relations use real Seiche dragging around projection slots; Grid and Scatter
remain recipes, physics runs at a fixed 60 Hz with at most three substeps, and
redraw is demand-driven. Cambium/Genet and Netrender retain faces across
frames. Projection fields are not wired into this proof yet.

The current-tree
`ports/graphshell/docs/receipts/practice_workspace_receipt.json` records the
current-tree checks: 14 reducer/disclosure tests, 15 overlapping
compiler/editor tests, four physics tests, and a passing wasm build. The
browser scenarios passed 45 standalone, 11 fresh-reopen, 14 embedded, and
58 existing-authoring regression steps. At 360px, Compare remains readable
and Relations stacks its cards; keyboard Enter saves successfully. Idle
counters remain unchanged. A real drag added 367 physics steps with zero
projection compiles or text layouts. These CPU counters exclude GPU present.

Grid compilation previously rebuilt and sorted channel ranks for every card.
Precomputing ranks once changes that path from quadratic sorting work to
O(n log n). Sequential single-sample release runs at 10,000 occurrences measured
14,328,279us before and 19,618us after, including Scenomise solving. The receipt
preserves raw output and binary hashes; the baseline source snapshot was not
archived. The reproducible example is `ports/graphshell/examples/projection_compile_perf.rs`.
This is not a large-graph interactive frame budget or a release benchmark.

To reproduce, build the current Graphshell web package, serve
`ports/graphshell/web` over loopback HTTP, then open
`practice.html?scenario=scenarios/practice_workspace.scn`. Reopen on the same
origin with `practice.html?scenario=scenarios/practice_reopen.scn`. For the
component-in-wrapper check, open
`practice-embed.html?scenario=scenarios/practice_embedded.scn` on that origin.

### Template scene vertical framing (2026-10-16)

**Status:** shared fitting and native Woodshed adoption qualified.

Long template timelines currently shrink both axes through BoardFit, leaving
labels and role alignments in a narrow band even when the host lends height.
Pictograph's shared BackdropSurface will fit template Y coordinates to the
available height independently, using the same transform for paint and inverse
picking. Other scene kinds retain uniform instrument/map geometry. Source
facts, musical spans, selection identities and document serialization stay
authoritative. Done when the shared regression verifies row clearance and span
edge picking, and Woodshed presents the revised scene in its native panel.

Findings (2026-10-16): the 580 × 300 timeline now separates supplied and
expected rows by about 49 pixels. The same two-axis transform drives inverse
picking, including the lower edge of a stretched Card span. Fretboard fitting
remains uniform. Five surface tests pass against the current source; see
`validation/template-scene-height_20261016/tests.log`.

Native adoption (2026-10-16): Woodshed `f683bbe` adopts shared source
`52fb4900e`. Its 300px primary canvas passes 214 view cases, two native editor
cases and the locked build. Three reviewed native captures show the separated
rows and compact scrolling; the actual sealed Set and complete portable
document equal the earlier fixture. Consumer evidence lives in
`woodshed/validation/native-arrangement-spacing_20261016/receipt.json`.


### Native arrangement interaction geometry (2026-10-16)

**Status:** shared geometry and native Woodshed interaction adoption qualified.

BackdropSurface now exposes forward/inverse viewport transforms and rectangular
occurrence bounds through its existing fit. A host can opt into selected-span
edge affordances. These queries carry no mutation authority or domain edit
semantics. Woodshed owns beat snapping, draft preview, validation and Save.
The shared regression exercises both progression and metre template specimens
across three viewports, verifies round trips and visible-edge picking, and rejects
invalid queries. Six surface tests pass (`validation/template-scene-interaction_20261016/tests.log`).
A shared editing-intent API still requires a second concrete editing consumer;
metre geometry evidence alone does not qualify source editing in Knot/Graphshell.


Woodshed adoption (2026-10-16): source `b3afe76` pins shared `d1e1d7aae`.
Fourteen focused consumer tests and the locked desktop build pass. Native body
and edge gestures, keyboard movement, Escape and compact rest editing pass in
229 frames; a 94-frame fresh process reopens the exact sealed Set and complete
portable document. Five captures were reviewed. The consumer keeps beat
snapping and draft/Save semantics; the surface still supplies geometry only.
Evidence: `woodshed/validation/native-arrangement-spans_20261016/receipt.json`.


### Metre editing and shared span intents (2026-10-17)

Prove native authored metre alignment in Knot before extracting editing overlap
with Woodshed. Each source syllable has an optional explicit positive span (or
unplaced value); omitted records retain legacy ordinal placement. Stress heights,
pronunciation evidence, exact source identities and rhyme arcs stay independent
of authored alignment. Metre positions are not playback durations.

Done conditions: native pointer and keyboard edit/cancel/reopen over the shared
surface; source/pronunciation preservation and stale proposal refusal; portable
Graphshell rendering and local-copy editing; one bounded revision-qualified
intent contract used by both real owners, with domain validation and persistence
remaining in those owners. Read-only imports never gain source mutation authority.

Implementation: `scenomise::span_edit` now supplies checked move/start/end math,
placed-span targets and source/revision/occurrence/lane/before-qualified intents.
Metre and Woodshed adapters accept the same contract; they retain their own
history, source checks and durable writes. Zero movement preserves off-grid
source spans. Graphshell offers explicit local-copy editing, preview/cancel and
portable export; source imports retain read-only authority. The standalone
product WebAssembly check and dependency boundaries pass. At that initial compile milestone, native and browser acceptance receipts were
still pending; the final qualification below closes those bounded gates.
The web lockfile reconciles two missing `zeroize` edges from the fetched
owner-admission supplier update; no web Git source pins change.

Bounded continuation complete: Knot `fdd6940` and Woodshed `28a0741` adopt the
canonical shared `4c4ea11d8` source. Shared validation passes three span-intent,
four sequence-template and six surface cases plus the dependency boundaries.
Woodshed passes nine locked arrangement-editor route/model cases. Knot passes
72 focused cases and two final macOS native processes (237 presented frames, six
reviewed captures), including preview/Escape, body and edges, keyboard, history,
explicit sealed collection and exact fresh-process SceneInput reopening.
The lexical source, candidate stress and rhyme facts remain unchanged.

Graphshell's actual WebGPU product host passes the committed copy-edit scenario
for metre and progression inputs, with three functional captures and no page
gate failures. The scrolled captures do not claim every viewport is visually
qualified. Source imports remain read-only; explicit local copies receive a new
Graphshell identity and exported copied facts. No generic command bus, history,
playback clock or source-write capability was moved into the shared intent type.
Evidence: [shared/browser receipt](../../../validation/shared-span-intents_20261017/receipt.json),
`knot-editor/docs/receipts/2026-10-17_metre_editing/`, and
`woodshed/validation/shared-span-intents_20261017/`. The older A5 and unrelated
portable-grammar gates in this plan remain open.

### Shared scene prototype design review (2026-10-17)

**Status: initial audit answered; prototype refinement in progress.** Mark explicitly identified
these implementations as prototypes and requested a prompted review of each
consequential choice, followed by refinement. The qualification above records
observed implementation behavior, not agreement on every product choice. No
previous receipt is withdrawn or expanded by this review.

Keep this review here as the cross-repository record; Knot and Woodshed refer
to it from their existing consumer plans. Ask in small rounds, record explicit
answers, and distinguish decisions, recommendations and current behavior. An
unanswered prompt stays open. Routine implementation corrections do not need
to become product questions.

| Topic | Prototype behavior or proposal to review | Decision |
| --- | --- | --- |
| Metre authoring scope (prompt 1) | Edits syllable spans; supplied stress/rhyme are preserved. Authored scansion and expected-form editing are absent. | Answered: alignment, scansion, metrical unit and editable poetic-form templates in composition lenses; see verbatim answer below. |
| Incomplete analysis (prompt 2) | Starting native metre requires a complete current CMUdict reading. | Answered: manual work with unknown words/incomplete analysis; disclose unknowns. |
| Vocabulary (prompts 3, 21) | Technical occurrence/placement/slot/lane terms have appeared in product descriptions; universal versus domain vocabulary is unsettled. | Answered: domain language in each app; prefer commonly accepted technical terms. Lane is not a queue here, and no new shared naming scheme is adopted. |
| Attention and durable keeping (prompt 16) | Some UI uses retained for a saved copy, while the supplied brief uses Retained for attention. | Answered: reserve Retained for attention; Save is ordinary saving and Collect makes explicit snapshots. |
| Source relationship (prompt 4) | Knot captures a reading; live source changes block edits, while reopened saved copies are independent. | Answered: preserve authorship where the match is clear; flag affected passages for review and offer fresh analysis. |
| Rearrangement and document order (prompt 5) | Current alignment edits leave source bytes unchanged. | Answered: offer document rearrangement and alternative arrangements, with an explicit distinction. |
| Saving and document lifecycle (prompt 6) | Knot collects immutable typed scenes; Graphshell edits/exports an explicit local copy; Woodshed saves Set-owned drafts. | Answered for Knot: ordinary composition save/reopen includes authored information; collecting a snapshot stays separate. Storage format and other hosts remain open. |
| Editor location and switching | Active metre takes over Sound and must be closed/discarded to return to its controls. | Answered in prompt 1: lenses within composition editing, not a special editor within the editor. Detailed interaction remains open. |
| Template configuration (prompts 7, 19) | Metre setup precedes capture, defaults to five binary feet, and offers none/paired expected rhyme only. | Answered: choose/edit a form while composing, preserve work and flag conflicts; start without an imposed form, offer templates and manual units/structure. |
| Small-window presentation (prompt 8) | Native metre uses a 600px logical scene with horizontal scroll; template scene fitting stretches axes independently. | Answered for metre: follow document wrapping, preserve readable structure across wraps, offer a wider overview. Other scene fitting remains open. |
| General template scope (prompt 10) | Implemented sequences use lanes, integer spans, expected slots, and same-lane nested groups with a three-level limit. | Answered: option 1, test contextual pieces against both consumers; nesting also has lateral/in-plane presentations, not just depth. See full answer below. |
| Meaning of span/group edits (prompt 11) | One already-placed span changes within its lane/extent; expected structure and group edits are excluded. Metre spans are positions, not measured performance. | Answered: move meaningful groups preserving internal relationships; distinguish content moves from editing expected structure. Mark identifies this grouping as nesting. |
| Woodshed arrangement and practice (prompt 9) | Scene movement does not reorder Cards, infer harmonic roles, or alter their playback instructions. | Answered: an explicitly chosen arrangement can drive practice; retain the Set-order path. Playback implementation remains future work. |
| History experience (prompt 12) | Knot has bounded local history; Graphshell copy editing lacks undo/redo. Shared snapshot history already exists in cambium/backstep; review reuse as an engineering follow-up. | Answered: one chronological history per composition or arrangement; one gesture per step; analysis refresh preserves authorship. |
| Djot block and Swatch/Canvas integration (prompt 13) | Domain block parsing and portable scene presentation currently lack a general editable round-trip connection. | Answered: both insert composed content as an editable block and apply reusable rules/templates to existing content. |
| Template update relationship (prompt 14) | The prototype captures template facts; it does not establish reusable-template update behavior. | Answered: linked template with explicit updates, preserving local edits and flagging conflicts; related instances can be selected for chosen changes. |
| Inserted block identity (prompts 15, 17) | Local copied scenes and reflected views have different relationships to their sources. | Answered: independent by default; source/provenance link does not create live shared content. |
| Placement interaction (prompt 18) | An unplaced-items tray and select/preview/place workflow were proposed in conversation, not implemented or agreed. | Answered: keep unresolved pieces in context plus an optional holding utility; pocket/minimap Swatch appearance is exploratory. |
| Holding utility lifecycle (prompt 20) | A workspace/document holding area has no established scope or persistence contract. | Answered: workspace utility shared across open documents, saved across reopen until items are removed. |
| Next implementation priority (prompt 22) | Placement/unplacement ahead of template or group editing was a recommendation, not an agreed sequence. | Answered: editable document-block path first, normal save/reopen, shared Swatch/Canvas view and metre lens; validate shared pieces with Woodshed. |

#### First-round answers and their limits

The following are Mark's answers, verbatim and contiguous within each answer.

**Prompt 1 — metre authoring scope:**

> alignment, scansion, metrical unit, poetic form (analysis says to me offer as templates for now, let people edit them, and later analyze and identify when people stumble on structures (poetic form autocomplete?) or are close to/could follow possible forms. as a suggestion). clearly rhyme, synonym, grammar, word parts definition, etymology, and a dictionary are valuable here too, and the pieces exist. i think metrical editing should be part of the rest of the editing, not a special editor within the editor. more like lenses on composition lines. the shared bit is the arrangement of contextualized pieces relative to a context (bars in music, lines, sentences, paragraphs, extracted web content?), i'd think. although it's clear that music and language grammars share common constraints, here... perhaps i should give this more thought...

**Prompt 2 — incomplete analysis:**

> No. Allow manual work with unknown words or incomplete analysis; show what is unknown.

**Prompt 3 — vocabulary:**

> Use domain language in each app; review shared technical names separately.

Implementation consequences: replace the analysis-completeness entry gate and
separate-editor assumption; make room for authored scansion, units and editable
form templates without conflating them with source analysis. Related lexical
tools are relevant composition context, not a mandate to add every tool in this
pass. Form recognition/autocomplete is a later suggestion idea, not an immediate
implementation requirement. The proposed commonality of contextualized pieces
and contexts, and a broader music/language grammar, remain explicitly exploratory;
this answer does not approve a universal template schema or its technical names.
Detailed interaction and scope sequencing remain open and are prompted
separately. Source reconciliation and saving direction are answered below;
their implementation mechanisms remain to be assessed.

#### Second-round answers and their limits

**Prompt 4 — source changes:**

> Keep authorship attached where the match is clear; flag affected passages for review and offer fresh analysis.

**Prompt 5 — document order and alternatives:**

> Offer both: distinguish rearranging the document from trying an alternative arrangement.

**Prompt 6 — ordinary saving:**

> As part of the composition’s ordinary save/reopen workflow; collecting a snapshot remains a separate action.

Implementation consequences: the captured-reading global edit freeze is not
the intended composition-lens lifecycle. Preserve clearly matched authored
work after text edits, expose affected/uncertain passages and offer reanalysis;
do not silently transfer authorship across ambiguous matches. Distinguish
document-changing arrangement actions from experimental alternatives. Ordinary
composition save/reopen must include authored scansion, metrical units and
chosen/edited forms; snapshot collection remains a separate capability. This
does not select a file encoding, sidecar, journal format, attachment scheme,
matching algorithm or conflict interface, and does not change the established
ownership of collected copies in Knot's mere. Assess existing document/source
anchor mechanisms before choosing the implementation.

#### Third-round answers and their limits

**Prompt 7 — changing forms:**

> Let me choose or edit a form while composing; preserve my work and flag conflicts with the changed form.

**Prompt 8 — wrapping:**

> Follow the document’s wrapping and keep metrical structure understandable across wraps; offer a wider overview when useful.

**Prompt 9 — Woodshed practice:**

> An arrangement can drive playback when I explicitly choose to practice it; keep the existing Set-order path available.

Implementation consequences: allow form changes within the composition flow;
retain authored work and report conflicts instead of forcing a fresh captured
draft or silently coercing it to the changed form. A wrapped visual continuation
must remain part of the same authored verse line, with understandable metrical
structure; the fixed-width scrolling prototype is not the default lens target.
A wider overview is complementary. Woodshed needs an explicit arrangement
practice choice alongside existing Set-order practice; editing or saving an
arrangement does not by itself choose that playback path. This answer does not
settle tempo/duration conversion, overlapping spans, repetitions, missing Cards,
live edits during playback, or how alternative arrangements are saved/selected.
Those semantics need inspection and focused prompts before implementation.

#### Fourth-round answers and their limits

**Prompt 10 — shared model and nesting:**

> 1, but nesting is a property that extends laterally as well as depth. words are nested in a sentence. notes are nested in a chord. and if you want to represent nesting within a plane's scope, you basically use a field. think an ios home screen folder: that's nested, when you click it, it arranges the members within that scope. it could also expand the folder within the homescreen's original scope, pushing other icons out of the way; that's representing a nest too, just without depth, no?

The selected option 1 was: "Use the contextual-pieces idea as a working
hypothesis; refine metre and progressions, then extract only what both
demonstrate."

**Prompt 11 — group movement:**

> yes, 1, that is the nesting.

The selected option 1 was: "Yes; preserve internal relationships, and make
moving the contents distinct from editing the expected form."

**Prompt 12 — history:**

> One chronological undo history per edited composition or arrangement; one gesture is one step, and analysis refresh does not erase authorship.

Interpretation to test in the prototypes: membership/nesting is independent
of its presentation. A field enclosing members in the current plane and a
focused view arranging members within a nested scope can express the same
grouping. Do not equate the existing three-level sequence rendering bound with
a universal nesting law, and do not infer that every expansion must displace
neighbors. Preserve source distinctions between structural membership, a selected
subgraph, and a temporary display fold. Group movement preserves internal
relationships; changing expectations is a distinct action. Reuse existing shared
snapshot history where appropriate rather than maintaining avoidable parallel
mechanisms; ownership stays with the edited composition/arrangement.

#### Djot and block integration question

Mark then asked, verbatim:

> feels like, wasn't the point of djot that we could say, this block has these rules we can define per domain/protocol/format? wouldn't that be useful for different domains? and then, so like can't we compose that in a swatch/canvas and then apply that as a block to an editor?

This is an architectural question and proposed connection, not approval of new
syntax or a universal domain grammar. Existing grounding:

- The [polyglot Knot design](../../nematic_docs/implementation_strategy/2026-05-08_polyglot_knot_design.md),
  section 10, explicitly chose Djot divs/attributes for meaningful typed blocks
  and preserved protocol content in fences.
- `crates/nematic/nematic/src/knot/djot.rs` currently lowers a fixed recognized
  div vocabulary; unknown classes become generic quotes. It does not preserve
  arbitrary domain-rule bindings and nested containers as a generic editable
  block model. `knot/expand.rs` separately dispatches known protocol fences.
- The [polyglot block resolver plan](../../nematic_docs/implementation_strategy/2026-06-13_polyglot_block_resolver_plan.md)
  still records its unified resolver as planned. Current source inspection
  confirms separate fence expansion, transclusion and evaluation mechanisms.
- `scenograph::swatch` already declares scoped projection/reflection views;
  shared BackdropSurface hosts the metre/progression scene prototypes. Neither
  by itself completes a Djot document block's editable save/reopen round trip.

Prompt 13 asked whether the operations should include both inserting composed
content as an editable block and applying its reusable rules/template to existing
content. Mark answered, verbatim:

> Both

Direction: support both operations, retaining content, domain rules/template
bindings and authored structure through the document path, with Swatch/Canvas
and editor views operating on the same domain-owned model. Applying rules to
existing content and inserting composed content are distinct operations.
This does not settle new syntax, encoding, reference-versus-copy semantics,
template-update behavior or the domain adapters. Prompts 14 and 15 address
template updates and inserted content identity separately; answers follow below.
Inspect and extend the existing seams before creating another block system.

#### Template relationships, insertion and terminology answers

**Prompt 14 — template updates and applying selected structure:**

> 1, because all the instances of the template are themselves related as effectively nodes in the document, no? so selecting them all and applying changes to them should be simple.
>
> for the previous question I responded both to, i think of it as, if i'm editing a sentence, and I choose to arrange the sentence relative to some poetic form, then that should affect what i arranged. however, if i take an arrangement of words from elsewhere, then putting it in there would require insertion, not application. if i wanted to apply some structural aspect of the arrangement of words from elsewhere, i'd choose that aspect and apply it to the words i'm editing. now there's probably some simplification to work out there, but that feels consistent to me; keeping the distinction between things accurate

The selected option 1 was: "Keep a link to the template and offer updates
explicitly; preserve local edits and flag conflicts."

**Prompt 15 — inserted block identity:**

> default independent, offer link to source

**Prompt 16 — attention versus saving:**

> Reserve Retained for attention; use Save for ordinary saving and Collect for explicit snapshots.

Implementation consequences: preserve the relationship from a template instance
to its template so related instances can be found/selected for an explicitly
chosen update. Do not infer that every template update immediately changes all
instances or selects them automatically. Applying a form or a selected structural
aspect acts on the target content; importing external words/content is insertion.
This does not prescribe a new graph identity scheme or a universal structural
transfer algorithm. Inserted content is independent by default. "Offer link to
source" does not yet settle provenance/navigation versus live shared-content
insertion; prompt 17 asks that distinction. User-facing durable actions use
Save/Collect, while Retained remains attention. Existing lower-level storage
API names are not renamed by this vocabulary decision.

#### Source links, optional holding and initial form answers

**Prompt 17 — source link:**

> A source/provenance link; the inserted content remains independent.

**Prompt 18 — unresolved pieces and holding utility:**

> 2; an arboard/clipboard utility, didn't we think about that before? idk, like a lil minimap pocket swatch you can throw stuff in maybe.

The selected option 2 was: "Keep them in context and also offer an optional
holding area for arranging pieces."

**Prompt 19 — initial form:**

> No imposed poetic form; offer templates and let me define units or structure manually.

Implementation consequences: a source link records provenance/navigation and
does not make inserted content follow its source. Unresolved alignment must
remain inspectable in its original context; an optional holding utility
complements that experience. Its name, exact appearance, item semantics, scope
and persistence have not been decided. A pocket/minimap Swatch is a suggested
presentation, not a new component specification. Start composition without
silently imposing five binary feet or another form; templates and manual
units/structure remain available.

Precedent check: the historical
[workbench staging plan](../../archive_docs/2026-09-02_retired_plans/2026-06-09_workbench_staging_plan.md)
described host-owned staging into workbench tiles, and current
`scenograph::swatch` supplies scoped views. These are adjacent precedents, not
proof that the proposed pocket utility was previously specified or implemented.
The targeted memory and Mere/Knot/Genet documentation searches did not locate
the exact earlier pocket/clipboard discussion. Prompt 20 asks its scope/lifetime;
inspect existing clipboard/transfer capabilities before choosing implementation.

Historical checkpoint after prompt 19 (superseded by the answers below): remaining prompts were 20 (holding lifecycle), 21 (provisional internal
vocabulary), 22 (implementation priority). Detailed Woodshed playback semantics
and any ambiguity in these answers require a focused follow-up before their
corresponding behavior is implemented; they are not implied by closing the
initial prototype audit.

Refinement criteria: record the intended behavior and vocabulary before changing
the corresponding experience; inspect existing shared mechanisms before adding
new ones; update the affected consumers together where a shared contract changes.
Verification must cover the revised behavior and retain source-bound historical
evidence without presenting it as acceptance of the new design.

#### Final first-pass answers and selected implementation

**Prompt 20 — holding utility:**

> A workspace utility shared across open documents, saved across reopen until I remove things.

**Prompt 21 — internal terminology:**

> a lane feels like a queue to me. is it that? or is it a line? eh. it's not a huge deal either way. tend towards what's commonly accepted

**Prompt 22 — implementation priority:**

> The editable document-block path: Djot structure/rules, normal save/reopen, shared Swatch/Canvas view, and a metre lens; validate the shared pieces with Woodshed.

The initial prompted audit is answered. Internal terms remain provisional;
prefer established format/domain terminology. A sequence lane currently indexes
a parallel row, without queue semantics. The workspace holding utility is
persistent across reopen; its implementation follows the selected document-block
path rather than becoming a prerequisite for ordinary composition.

### Editable domain blocks and composition lenses

**Status (2026-10-17): first local prototype slice qualified; broader refinement remains open.** Selected by Mark in prompt 22. The prior
prototype receipts remain historical evidence. No user-visible vocabulary or
universal template grammar is frozen by this implementation.

Feature targets and checks:

1. Preserve Djot div attributes and nested content through the portable document
   model and round-trip writer. Opaque domain declarations remain data; unknown
   domains render their content. Verify nesting, escaped attributes, source
   preservation, protocol traversal and ordinary fallback rendering.
2. Bind block edits to the existing document source and one chronological text
   history; normal Save/reopen retains authored block data without collection.
   Verify one-step undo/redo, stale/read-only refusal, exact unaffected source,
   and actual temporary-file fresh-session reopening.
3. Connect the same authored block to a shared scene view and a composition lens.
   Support manual/incomplete metre without imposing a form. Preserve authored
   facts separately from analysis; source changes preserve clear matches and
   disclose uncertain ones. Verify views agree and preview/cancel is reversible.
4. Exercise the shared document-block behavior with Woodshed progression facts;
   distinguish applying selected rules from inserting independent content with
   provenance. Keep explicit arrangement playback and workspace holding as
   subsequent features with their own focused semantics/acceptance.

Inspect existing document, Djot, source-anchor, Swatch and history seams before
adding mechanisms. Use existing domain facts for the first two consumers and
keep serialization choices identified as prototype details. CPU source proof,
normal-save file proof and actual host interaction are separate checks.


Implementation slice (2026-10-17): generic `Block::Div` retains opaque attributes
and nested children. The Djot parser preserves quote/list parents and existing
semantic containers; fallback readers show the contents, and protocol/inline
expansion traverses the container. This is not a claim of full Djot conformance:
the existing inline/table lowering limits remain.

Knot now applies declarations to existing source or inserts independent block
content through source-bound document operations. Declaration edits preserve
nested bodies and quote/list markers. Both operations use the normal text undo
and file Save/reopen path, with stale/read-only refusal. The composition lens
supports manually supplied units, strong/weak/unknown scansion, optional form,
metrical-unit and rhyme expectations, with no dictionary or default form.
Unique unchanged line text retains authorship across unrelated edits/moves;
changed or ambiguous lines remain in context and retain their old authored
records for review. This is conservative matching, not a settled line identity
scheme. The wrapping lens and optional shared overview are both attached to the
ordinary source document. The source editor is still visible: this slice proves
the data/history connection rather than a finished rich-text block interface.

The optional scene-document Djot codec wraps the existing versioned JSON facts;
it introduces no second music grammar. Woodshed's real arrangement export uses
that codec with its Card/rest spans and linked voicings. Graphshell admits the
same Djot payload through its existing surface/import path; bundled domain
exports exercise this reference host. Reading/inserting a progression block
never implicitly changes a live Set or selects arrangement playback.

Qualification is recorded in
`validation/editable-document-blocks_20261017/receipt.json`. Consumer checks use
a temporary local Mere source cohort; canonical app lockfile integration is a
separate unqualified step. The reusable-template relationship/bulk update UI,
source provenance insertion UI, meaningful group moves, alternate arrangements,
explicit arrangement practice and lexical/form
suggestion integration remain open. Attribute names and encoded authorship are
prototype serialization details, not a frozen shared grammar.


### Pool workspace utility

Mark selected the provisional name and authorized implementation on 2026-10-17:

> Pool it is for now. Let’s build the workspace utility, and extract a crate if the shared use calls for it.

Pool holds independent material across the workspace's open documents and across
reopen until explicitly removed. Adding copies content; insertion does not consume
it. A source address is provenance/navigation, not a live content binding. Pool
is distinct from ordinary document Save, explicit Collect snapshots and the
Retained attention state. Finished and unresolved material have the same lifetime.

The first implementation adds a small `inker::pool` data module, consumed by
Graphshell and native Knot. No separate crate is justified yet: text/material
identity and validation are shared, while storage and editing remain host-owned.
The versioned UTF-8 payload envelope is provisional; this slice supports plain
text and attributed Djot, not a general binary asset or template-update protocol.

Graphshell stores Pool in an origin-wide IndexedDB key independent of its current
graph session. It can add the current portable scene (including Woodshed's real
progression export), add text, open an independent scene copy, export held bytes,
and remove material. Commits compare the expected stored value within the
existing backend transaction. Browser origin storage policy still applies.

Knot stores Pool beside workspace preferences, outside document tabs. It copies
a selection or an authored metre block, inserts at the destination selection
through normal document history, and offers provenance navigation. Inserted metre
blocks receive fresh block IDs; no universal reference-remapping grammar is
introduced. Pool writes use a private temporary file and atomic replacement,
refuse changed/unsupported/corrupt storage, and leave visible state unchanged on
failure. The stale-value check is not a cross-process filesystem lock.

The expandable bounded panel is an initial utility presentation. The pocket/
minimap swatch appearance, drag/drop, richer item previews, grouping and native
Woodshed integration remain subsequent refinements. Pool is workspace-local to
each host, not a cross-application synchronized library. Template-rule application
remains separate from inserting held content.

Qualification and source binding: `validation/pool_20261017/receipt.json`.
One shared model test, three native integration tests, three actual macOS
processes (six inspected captures) and five browser page loads (eight inspected
captures) pass. The browser export callback is exercised; downloaded bytes are
not qualified. Expanded Pool plus existing Graphshell chrome still crowds the
scene stage, so final utility placement/pocket presentation remains open.
Canonical app supplier integration remains a separate step.


#### Pool review: scope, presentation and editing (prompts 23–25)

These answers refine the intended product beyond the qualified local prototype
above. They do not claim that cross-application storage, the pocket presentation
or editing held material has been implemented.

**Prompt 23 — sharing across apps and workspaces:**

> i was thinking 2, but 1 and 3 have merit as scope options

The referenced choices were (1) same Pool for the same workspace, (2) one
personal Pool across all workspaces and apps, and (3) separate per-app Pools with
explicit transfer. Personal scope is the primary direction; workspace and app
scope options have merit. Prompt 26 below resolves these as distinct scoped Pools. Cross-device synchronization,
workspace identity and transfer behavior are not settled by this answer.

**Prompt 24 — presentation while composing:**

> A small pocket that expands into a Swatch/Canvas (Recommended)

**Prompt 25 — editing held material:**

> 1 if available, 2 as fallback

The referenced choices were (1) open the held item in its usual editor and save
changes back to Pool, and (2) edit the held item directly on Pool's Swatch/Canvas.
Prefer the usual editor when available, with the canvas as fallback. Inserted
copies remain independent; revising the held item does not update them. Both
paths should reuse existing domain editing and shared scene surfaces rather
than introducing a separate authoring grammar. That reuse is an implementation
direction; editor dispatch and save/session ownership still need definition.

Prompts 26–28 below resolve scoped-Pool ownership versus views, removal/recovery
and the default way to inspect a nested group. Moving group contents together
was already agreed. Other remaining topics include alternate arrangements,
selected structural aspects and template updates, cross-app workspace identity
and the held-item editing lifecycle.


#### Pool review: scoped collections, undo and nesting (prompts 26–28)

**Prompt 26 — distinct Pools versus filtered views:**

> 1; it's basically an embedded minimere, no?

Option 1 was "Distinct scoped Pools, with personal as the default (Recommended)".
Personal, workspace and app scopes therefore select distinct Pools. The personal
Pool is the default. Mark's embedded-minimere framing points toward reusing Mere
collection, presentation and session mechanisms for a compact embedded utility.
This is product direction, not a claim that the current `inker::pool` vector is
already a Mere or that a specific Mere API satisfies the new requirements.
Inspect existing Mere/Swatch/Canvas and history seams before extending the
prototype; do not extract an independent Pool crate merely to bypass that review.
Prompt 30 below selects Move as the default cross-Pool transfer. Workspace
identity, non-default transfer actions and cross-device availability remain open.

**Prompt 27 — removal and recovery:**

> Remove it with Pool undo available across reopen

Removal must be undoable after reopening Pool. This replaces the prototype's
immediate permanent removal. A separate Trash is not the selected recovery UI.
Inserted independent copies remain unaffected. Prompt 29 below defines Pool history as adding/removing and arranging items,
with content edits in the normal editor history. This answer does not impose a
history limit, expiry period or automatic purge.

**Prompt 28 — inspecting nested groups:**

> Expand in place, with entering its own scope also available (Recommended)

In-place expansion is the usual interaction; entering the group's own scope is
also available. Expanding may rearrange the surrounding presentation while
preserving authored text order and musical timing. Group members remain members
when their presentation changes; moving a group together was already agreed.
Focus/selection on collapse, reflow details and nested-scope navigation remain
interaction refinements, not authority to change authored arrangement silently.

These decisions have not yet changed implementation or expanded the historical
qualification claims. Validation for a future slice must include distinct scope
selection, removal/undo across a fresh process, shared held-item editing ownership
and both nested views preserving the same authored facts in metre and Woodshed.


#### Pool review: history, transfer and alternatives (prompts 29–31)

**Prompt 29 — undo ownership:**

> Pool history covers adding/removing and arranging items; content edits use the item’s normal editor history (Recommended)

Pool's collection/arrangement operations have their own history; editing held
content uses that item's normal editor history, including when its shared
editor is exposed on the canvas. The earlier requirement for removal undo to
survive reopen still applies. Durable history policy for other operations is
not additionally specified by this answer. Do not duplicate a content edit into
an independently competing Pool content history.

**Prompt 30 — default transfer between distinct scoped Pools:**

> Move the held item to the other Pool

Default cross-Pool transfer is Move. This is distinct from inserting independent
content into a composition, which remains the agreed behavior. Non-default copy
or shared-reference actions are not settled here. Multi-Pool operation/undo
coordination remains an implementation and interaction question; do not treat
an incomplete destination write as a completed move.

**Prompt 31 — where alternative arrangements live:**

> Save alternatives within the same composition or arrangement, with switching/comparison (Recommended)

Alternatives belong to the same composition/arrangement and can be switched or
compared. This does not yet settle whether their content pieces are shared or
independent, how edits diverge, or how adopting an alternative affects the main
arrangement. Ordinary document rearrangement remains a separate explicit action.

#### Sidecar framing

Mark added this framing while reviewing the decisions above:

> ahhh. perhaps the best way to think about it is, this is what would be the browser extension, as a sidecar.

This suggests a compact Mere surface alongside a host's ordinary activity, with
a browser extension as a concrete host integration. It is a design framing, not
a request to ship a browser extension now, a settled extension permission model,
or a claim of existing cross-app support. Prompt 32 below refines the collection
and view relationship. The sidecar's attachment and context switching remain
open; material enters through deliberate action rather than passive browsing
capture. No implementation qualification is expanded by these notes.


#### Pool review: an editable nested-graph Swatch utility (prompt 32)

**Prompt 32 — any Mere in a Pool role versus a dedicated utility:**

> 2, but i guess it's a type of swatch; take a bit of canvas from a mere, embed it. in much of the cases we've seen before, swatches are pretty much scoped to their function and aren't manipulable. this is a utility of a mere that is like a swatch that's like a clipboard for a mere, almost like a nested mere... so probably the relation to pool and the mere it's embedded in is similar to a mere and the browser. but that of course doesn't mean we can't use it outside the mere as a lil catchall. where a mere would passively collect browsing history as you use your browser, the pool wouldn't; what you put in would form a subgraph nested within a node in the mere (with that node then serving as the pool container). idk, i'm just trying to work this out in my head. it's like a mere but it's not. it's like a swatch using more capabilities than they have before. swatches were meant to be embedded meres anyway. so idk, makes sense to me

The referenced option 2 was "Pool is a dedicated collection type with its own
rules, using shared Mere components". Mark refines that binary choice into a
working design: Pool is a dedicated clipboard/catchall utility of a Mere,
presented through a more capable, manipulable Swatch. Its material forms a
subgraph nested within a container node in the Mere. The compact pocket and
expanded Swatch/Canvas expose that material; the utility can also be presented
outside its enclosing Mere, such as in a browser sidecar. Material is added
deliberately; passive browsing-history collection is not Pool behavior. Source
provenance attached to deliberately added material remains consistent with the
previous answers.

Treat this as exploratory product direction, preserving Mark's uncertainty and
analogy rather than promoting it into a frozen type/schema ruling. It replaces
the earlier assistant recommendation that any arbitrary Mere simply take a Pool
role. A dedicated utility does not by itself require a separate crate, graph
engine or storage system. Distinct scope choices, the personal default, Pool
versus item-editor history, default Move between Pools and independent insertion
into compositions remain applicable.

The existing `scenograph::swatch` model describes a Swatch as a Mere seen at a
scope, small enough to embed, and includes `Scope::NestedGraph(String)` for the
graph nested within a node. This is a concrete representation seam to investigate,
not proof of complete embedded editing, persistence, history or host integration.
The current `inker::pool` material list and host-specific stores remain the
qualified prototype; they have not been converted to the nested-graph utility.

Remaining boundaries include the container's creation/selection and lifecycle
when the sidecar runs outside the Mere, allowed manipulation through an embedded
Swatch, transfer/undo coordination across scoped Pools, and the concrete content
and membership mapping. Verify reuse against Graphshell plus a real composition
consumer before defining shared APIs. No browser-extension implementation or
new automatic capture behavior is authorized by the analogy alone.
