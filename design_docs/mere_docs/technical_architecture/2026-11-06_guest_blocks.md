# Guest blocks: Woodshed's progression and notation as a guest of the block pane

**Status:** 2026-11-06: design; Mark ruled the eight first-round decisions the same day
(recorded as [cycle_breaks D13](../implementation_strategy/2026-11-06_cycle_breaks_plan.md), shown
under "Rulings" below), and the view spike was run. **The spike fell short for painting (Spike); Mark answered R1 to R3
(recorded as D16), and implementation follows the order of work.** Rulings 5 and 6 changed Delivery, What
moves, Native and the Cycle check; those sections are rewritten. Nothing implemented. Written first,
as P1 was, because [cycle_breaks D12](../implementation_strategy/2026-11-06_cycle_breaks_plan.md)
(Mark: "D: Woodshed as a guest") chose a new mechanism. Cited by the plan's P7.

**For:** letting Mere's block pane host Woodshed's progression editing and notation view without
Mere naming Woodshed in any manifest or lock (D5's end), while the pane keeps working with no
Woodshed installed.

**Not here:** the guest model for other apps (the host/guest brief's recursion is untouched); the
domain registration seam (`2026-11-06_domain_contract.md`, which is about stores, not views);
the host-neutral extraction of the pane is described under Native but is Mere work.

## What exists today

Read 2026-11-06 at Mere `1a26ec4e6`, `ports/graphshell`.

- **`block_editing`** (`src/block_editing.rs`, `rhyme.rs`, `store.rs`, library `graphshell`, feature
  `editable-blocks` = `web`, `knot-editor-host`, `illume`, `woodshedding`): `Document` (a Knot editor
  buffer with the undo journal), `Preview`, `Change`, `Div` (re-exported from `illume`),
  `MAX_DOCUMENT_BYTES`. A block is a Djot div; an adapter edits its attribute map and the
  document applies the whole desired map through the editor, so undo and save need nothing special.
  `rhyme.rs` is a non-Woodshed adapter with the same shape as `bars.rs`:
  `choices(body, attrs, selected, pattern)` and `attributes(body, attrs, changes)`.
- **`bars.rs`** (741 lines, 8 tests): the Woodshed adapter. Public functions `rows`, `choices`,
  `attributes`, `music`, `write_music`, `time_signatures`, `meter_attributes`, `scene`,
  `plaintext`, and `SOURCE = "woodshed"`. It reads `woodshedding::rehearsal::{Set, arrangement}`,
  produces `scenomise::backdrops::SceneInput`, and uses `Change` and `MAX_DOCUMENT_BYTES` from
  its parent. Nothing else in `block_editing` references it. Of its 8 tests, 3 use `Document`
  and `Preview` (write-notation atomicity, the saved-music reading, and grouping through one
  history); the other 5 are pure.
- **`web_tree/blocks.rs`** (1,315 lines, no tests): a private module of the `graphshell-web`
  cdylib (`ports/graphshell/web`, lib path `../src/web.rs`). It is the whole pane, written in
  the page's private types (`use super::*`: `TreePage`, `AppCtx`, `Logic`, `Child`,
  `ProbeSnapshot`, `custom_leaf`). Nine hooks in the page: the `blocks` field
  (`web_tree.rs:510`), `toolbar`, `workspace`, `active`, `control`, `busy`, `fields`,
  `focused_text`, `register`, and `SHEET`. It branches on the class by name (`has_class(div,
  "progression")` against `"metre"`) and on `proposals()` returns "This block has no registered
  editing adapter" for any other class.
  - Generic (stays): the document editor, save/undo/redo, IndexedDB reopen, block selection,
    command search, preview, apply, rhyme.
  - Woodshed (leaves; roughly 350 to 450 lines, the exact cut is made at the move): the
    bar-grouping branch, `insert_progression` and its fixture
    `support/woodshed-block-example.json`, `write_notation`, the `notation_*` state,
    `notation_leaf`, `notation_readings` and its `NotationLeaf` paint leaf (`NOTATION_KEY`, the
    `produce` and `render` of the leaf at about `:1110`), the bar-line plaintext export, and the
    backdrop `SceneInput` from `bars::scene`.
- **The applet mechanism** (`capsule_applet.rs`, `web_tree/applet.rs`, `web/applets/*.mjs`,
  feature `applets`, web only): a supplying member (a Moot collection) discloses signed capsule
  packs. `CapsuleMount::review` verifies the signed pack (`eidetic` pack signing), the single
  `WasmComponent` part's blob address and size (16 MiB cap), then `approve(grants)` accepts only
  the host dialect's powers (`power:capsule-view`, `power:navigate`) that the pack asked for in
  `requested_scopes`. A Worker (`worker.mjs`) re-verifies, runs the component through a locally
  generated jco adapter, and the guest talks back only through `app-core`'s `emit(action-envelope)`,
  which the host classifies by ring and may deny. `confirm-install-participant` is host-only.
- **The WIT** (`merely-made/graphshell`, `crates/graphshell-endpoint/wit/world.wit`, package
  `mere:script@0.1.0`): interfaces `log`, `caps`, `net`, `document`, `document-host`, `actions`,
  `projection-session`, `settings`; worlds `app-core`, `document-core`, `projection-provider`,
  `settings-provider`, `app-settings`. `app-core` is event-in, actions-out: it cannot return a
  value to a call. There is no world that takes a request and returns data.
- **No composition point.** Blocks are composed by Cargo feature inside the one crate; the web
  product is that crate; there is no host binary above it, and no auto-registration works on
  wasm. This is why a compiled-in Woodshed adapter cannot be "registered": there is nobody to
  register from.

## Interface

A guest block adapter is a typed request/response component. Proposed WIT, in a new interface
and world beside `projection-provider` (names are proposals):

```wit
interface block-adapter {
    record attr { key: string, value: string }

    /// What the guest claims. Called once at mount.
    record claim {
        /// The Djot div class it edits (`progression`). One class per claim; a pack may
        /// carry several claims.
        class: string,
        /// Attribute key that carries the guest's structured source (`woodshed`).
        source-key: string,
        /// Versions of that source it understands.
        source-versions: list<u32>,
        /// Commands it adds to the pane, each with an id, a label and a kind (choice,
        /// write, export).
        commands: list<command>,
        /// An example it can insert, if any.
        example: option<example>,
    }
    record command { id: string, label: string, kind: command-kind }
    enum command-kind { choice, write, export }
    record example { title: string, body: string, attrs: list<attr> }

    /// What the host sends with every call: a copied snapshot, never a handle.
    record block {
        class: string,
        body: string,          // the div's body text, at most 1 MiB with the attrs
        attrs: list<attr>,     // the div's attribute map, including `source-key`
        revision: string,      // content hash of the whole document source
        selected: list<u32>,   // selected rows or bars
        pattern: string,       // the pane's pattern field for this adapter
    }

    /// Value shape of the host's `Change`; the host keeps its own type.
    record change {
        index: u32, label: string, before: option<string>, after: string,
        conflict: bool, use-update: bool,
    }

    record region { bar: u32, event: u32, x: f32, y: f32, w: f32, h: f32, label: string }
    record view-result {
        /// A `sceno` scene, versioned CBOR (chirograph's payload form), at most 1 MiB.
        scene: list<u8>,
        width: f32, height: f32,
        /// Accessible, focusable regions over the scene; the host makes them buttons.
        regions: list<region>,
        /// A one-line description for the scene's accessible label.
        description: string,
    }
    record view-ask { mode: string, width: f32, theme: theme, tempo: option<u32>,
                      selected-region: option<tuple<u32, u32>> }

    describe:  func() -> claim;
    /// Proposed edits for the current selection (today `bars::choices`).
    choices:   func(b: block, command: string) -> result<list<change>, string>;
    /// The complete desired attribute map after the user's decisions (today
    /// `bars::attributes`); the host applies it through `Document::apply`.
    apply:     func(b: block, command: string, changes: list<change>)
                   -> result<list<attr>, string>;
    /// Explicit writes (`write_music`) and exports (`plaintext`): new attrs or text.
    run:       func(b: block, command: string, arg: string) -> result<run-result, string>;
    view:      func(b: block, ask: view-ask) -> result<view-result, string>;
}

world block-guest {
    import log;
    import caps;
    export block-adapter;
}
```

What the guest receives and returns, in the order asked:

- **The class it claims:** `claim.class`, with `source-key` and `source-versions`. The host
  looks adapters up by class. Two claims for one class are refused at review; the host's built-in
  rhyme adapter claims `metre`.
- **Block text and attributes:** the `block` record, a copy at one revision. Large bodies are
  refused by the host before the call (the existing `MAX_DOCUMENT_BYTES` and the 1 MiB payload
  cap of `projection-session`).
- **Edits back:** `choices` returns `change` records; `apply` returns the whole desired attribute
  map, exactly as `Document::apply(preview, attrs)` takes it today. The guest never edits the
  document: the host builds the `Preview` from the parser's div and applies the map through the
  editor and undo journal. A guest that returns a map the host's preview does not match cannot
  change anything but that block's attributes. (This is the property `bars.rs` already has, and
  the test `grouping_edits_only_group_metadata_and_uses_one_document_history` states it.)
- **Its view:** a **`sceno` scene** plus accessible regions, not an applet surface. Reasons: the
  guest then needs no UI toolkit in the component (today's `NotationLeaf` is a Sprigging paint
  leaf, which would pull Sprigging and a text shaper into a wasm component); the host already
  renders scenes (`BackdropSurface`, `FrozenScene`) and owns focus, reading order and the
  accessible tree; and `sceno` is in the protocol layer (D8, D9), so the guest depends on
  nothing in Mere to produce it. The regions replace today's `event_rects()` and `labels()`
  overlay of buttons. An applet surface (a retained Cambium view from the guest) would give
  the guest control of focus and layout inside Mere's tree, which is the OpenDoc direction the
  brief warns against.
- **Ruling 2 and the spike:** the ruling was this scene plus regions, spike first. The regions
  half holds; the painting half does not (Spike). Until Mark answers R1, `view-result.scene` and
  `regions` are provisional, and a second field for the drawing may be added.
- **Not in the interface:** network, storage, clipboard, time. The world imports `log` and `caps`
  only, so the component is a pure function of its inputs; "unimported means unreachable" does
  the sandboxing, and review has little to grant.

**Two things that must be checked before the first line is written** (not assumed here):
(1) whether `sceno::Scene` and `Region` can carry a chart, tablature and staff reading with
event-level hit regions, or whether `woodshed-notation` needs a scene output beside its paint
leaf (`bars::scene` already produces a scene-shaped `SceneInput` for the backdrop, but not for the
notation leaf); (2) the size of a component built from `woodshedding` and `woodshed-notation`
against the 16 MiB component cap.

## Spike: can a `sceno` scene plus regions replace `NotationLeaf`?

Ruling 2 required this before any move. Run 2026-11-06 in a scratch workspace
(`/Users/markik/Code/worktrees/cycles-20261106/p7/spike`, nothing committed to any repository);
screenshots and numbers in `/Users/markik/Code/testing/mere/guest_blocks_spike_20261106/`
(`report.txt`, 12 leaf/scene PNGs, one host-path PNG, the measured component).

**Method.** For `Score::progression_example` (4 bars) and `rhythm_example` (11 events), in Chart,
Tab and Score modes at 640 px: (A) paint the real `NotationLeaf` and rasterise the paint list
through Netrender and Vello CPU; (B) carry the same drawing as a `sceno::Scene` (every fill
as a `Backdrop` with a `Polygon` footprint and an open `kind` for the colour role, one `ProjectedItem`
with a hit footprint per authored event from `event_rects()`) and paint it again with a
spike-local painter, because the host has none for a generic scene; (C) freeze that scene with
`graphshell-client`'s `FrozenScene`; (D) draw Mere's `progression.json` through `BackdropSurface`,
the host's existing scene path.

| Case | Paint cmds | Polygons | Contours that lost a hole | Scene JSON | Pixels differing A vs B |
|---|---|---|---|---|---|
| progression Chart | 36 | 36 | 0 | 41 KB | 0 (0.00%) |
| progression Tab | 60 | 74 | 14 | 98 KB | 512 (0.17%) |
| progression Score | 60 | 96 | 36 | 138 KB | 2,348 (0.94%) |
| rhythm Chart | 41 | 41 | 0 | 57 KB | 0 (0.00%) |
| rhythm Tab | 61 | 65 | 4 | 102 KB | 88 (0.06%) |
| rhythm Score | 57 | 67 | 10 | 87 KB | 720 (0.52%) |

**What the scene carries.** Event-level hit regions, accessible names and order. Four events became
four items and `FrozenScene` listed them ("Bar 1, event 1" and so on; 11 for the rhythm score), so
`event_rects()` and the per-event button overlay are replaceable. Selection and the playhead are
presentation state the host applies over these items.

**What it does not carry: the drawing.**
1. `sceno` says it outright: "The scene never carries rendered content." A `Backdrop` has an open
   `kind` string, a footprint and two flags: no colour, no fill rule, no text.
2. The host's existing path cannot take an arbitrary scene. `BackdropSurface::new` accepts only a
   scenomise `SceneInput` (disclosed-data facts: fretboard, terrain, templated sequence), and its
   `paint` draws backdrops only as guide-coloured strokes plus marks and labels. `FrozenScene` is
   the semantic reading and paints nothing. `hostpath_progression_BackdropSurface.png` is what that
   path draws for a progression: spans, relations and text, no engraving.
3. Forcing the drawing into polygon backdrops works for single-contour shapes (the chord chart is
   pixel-identical) and fails visibly for anything with a counter: `progression_score_B_scene.png`
   shows the treble clef, whole and half note heads solid, and `progression_tab_B_scene.png` shows
   the fret digits 0, 6, 8 and 9 as filled blobs. `Footprint::Polygon` is one contour. Bezier
   outlines were also flattened (6 segments each) and the scenes are 40 to 140 KB for four bars.
4. Text (bar numbers, chord symbols) is overlay text in the leaf today and has no primitive in the
   scene.

**Verdict.** A `sceno` scene plus regions is enough for the regions and not enough to replace
`NotationLeaf`'s painting. Missing: compound (multi-contour) fills with a fill rule, fill versus
stroke, theme-role paint, text runs, and a host painter for a generic scene. Adding them to `sceno`
is a contract change in merely-made/graphshell, and it contradicts `sceno`'s stated stance that a
scene carries no rendered content, so it is Mark's to rule. A host painter is a Mere change. This
design therefore does not choose; R1 puts the options to him.

**Component size** (wasm32-wasip2, release, LTO, `opt-level = "s"`, stripped, a real component with
the draft `block-guest` world exporting `view`):

| Guest contents | Size | gzip -9 |
|---|---|---|
| `woodshedding` and `sceno` only | 151,052 B (0.14 MiB) | not measured |
| plus `woodshed-notation` (engraving, Bravura 889,228 B, and `sprigging`) | 1,208,190 B (1.15 MiB) | not measured |
| plus `scenomise` (the arrangement `SceneInput`) | 1,718,946 B (1.64 MiB) | 645,865 B |

Against `MAX_COMPONENT_BYTES` (16 MiB) the full guest is about 10%. Most of it is the font.
Dropping `sprigging` (if the view is not a paint list) shrinks it further; not measured.

## Layer

**The interface is a protocol item.** Both sides of it are the host/guest contract the brief and
D7 to D9 put in `merely-made/graphshell`: it is a WIT world with copied values and a chirograph
payload, depending on nothing in Mere, and it names no Mere type (`change` mirrors `Change`;
the `scene` is `sceno`). The one WIT file already holds `settings` and `document-core`, which
are also not projection, so the file is the right home for another world. The file is the
single source (DOC_POLICY §2); a Mere-local copy would be exactly the drift D10 set out to avoid.

**The host side is a Mere feature.** The pane, `block_editing::BlockAdapter` (the Rust trait),
the guest wrapper that calls a mounted component, review and grants (`capsule_applet`), and the
Worker plumbing stay in `ports/graphshell`, which D8 left above Mere.

**Ruled (decision 1): the interface goes in the graphshell protocol WIT as a new `block-guest`
world.** It is added in step 2 of the order of work, after Mark answers R1, because R1 may add a
drawing vocabulary to the same repository and the two should land together. This design adds
nothing there yet.

```
merely-made/graphshell   block-adapter WIT, sceno           (protocol, below Mere)
        ^                         ^
merely-made/mere     host: BlockAdapter trait, pane,        (host, above the protocol)
                     guest wrapper, review + grants
        ^
merely-made/woodshed guest: woodshedding/blocks +              (guest, above Mere and
                     woodshed-notation/block-guest component  the protocol)
```

No arrow points from Mere to Woodshed. Woodshed to Mere and to Graphshell are the legal edges
already in its lock.

## Delivery

**What Woodshed ships (ruling 5: a feature of `woodshedding`).** Taken literally the ruling
cannot hold for the whole guest, and the reason is a dependency direction, not a preference:
`woodshed-notation` depends on `woodshedding`, so the engraving and view code cannot live in
`woodshedding` without inverting that edge. The split that keeps the ruling where it can hold:

- **`woodshedding` feature `blocks`** (off by default; ruling 5): the moved `bars.rs` adapter logic,
  as plain Rust over `woodshedding` types (`rows`, `choices`, `attributes`, `music`, `write_music`,
  `time_signatures`, `meter_attributes`, `plaintext`), returning its own small structs. Its one new
  optional dependency is `serde_json` (the logic reads the `woodshed` source attribute as JSON). No
  WIT, no `sceno`, no `scenomise`: the portable core stays free of Mere and Graphshell. The 8 tests
  that are pure move here; the 3 that use `Document` and `Preview` are rewritten (ruling 7).
- **`woodshed-notation` feature `block-guest`** (enables `woodshedding/blocks`): the WIT bindings
  (`wit-bindgen`), the `change` and `attr` mapping, the view (engraving, `event_rects`, regions, and
  whatever R1 decides the drawing is), the arrangement backdrop scene (`bars::scene`, which needs
  `scenomise`, `sceno`, `mere-curation` and `scenograph`), the example, and the component entry. A
  cdylib is built only when its package is the build root, so declaring `crate-type = ["rlib",
  "cdylib"]` here does not change any dependent. The wasm build is `cargo build -p woodshed-notation
  --target wasm32-wasip2 --release --features block-guest`; the spike built exactly this shape.
- Alternatives to the second bullet, for Mark (R2): move the engraving into `woodshedding` under
  the feature, which makes everything literally a `woodshedding` feature but puts about 735 lines and
  the 889 KB Bravura font in the portable core; or a new crate, which ruling 5 declined.

The pack: `PackManifest { name: "woodshed-blocks", parts: [WasmComponent], requested_scopes:
["power:block-adapter"] }`, signed with Woodshed's author key through the same `eidetic`
pack-signing path the applets use, built by a Woodshed script. The native rlib build runs the unit
tests without wasm.

**How Graphshell discovers, reviews and loads it on the web.** The existing applet path, widened
by one power and one purpose:

1. Discovery: the supplying member discloses the pack as a capsule in a Moot collection, as for
   applets (`CapsuleDisclosure`, one revision per signed pack). A block adapter is a capsule whose
   component exports `block-adapter` rather than `app-core`; the pack declares it with its
   requested scope, which is what `CapsuleMount::review` reads today.
2. Review: `CapsuleMount::review` verifies signature, address, size and component inventory
   unchanged. `approve` learns `power:block-adapter` (a new constant beside `VIEW` and
   `NAVIGATE`), still refusing a grant the pack did not ask for. The review surface names the
   class the component claims and the author's key; a new grant needs a new review, as now.
3. Load: the Worker (`worker.mjs`) re-verifies the component hash and instantiates it through the
   locally generated jco adapter, as for applets. The instance exports `block-adapter` instead
   of calling `emit`. The host wrapper turns each call into a worker message and a result.
4. Registration: on `describe()`, the host adds a `GuestAdapter` for the claimed class to the
   pane's registry. Closing or revoking the grant removes it; blocks of that class return to
   the interim state below with their source untouched.

**A consequence of the worker.** Today's pane calls `proposals()` synchronously on every render
(`commands()` calls it to decide whether a command is disabled). A guest in a Worker answers
asynchronously, so the pane keeps a memo keyed by `(class, revision, selection, pattern,
command)`, shows the previous result marked stale, and re-asks on change. The pane's
`completed: Rc<RefCell<..>>` already does this for IndexedDB saves. This is the main cost of
D12 and it is in Mere's pane, not in Woodshed.

### Native (ruling 6: a native block pane now, over `app-host`)

Today there is none: `editable-blocks` requires `web`, which in `ports/graphshell` is a misnomer
for the portable host modules (they compile natively; the same crate already builds Cambium
desktop shells), and `applets` is the same. Ruling 6 adds one. What exists to build on, and what
does not:

- **Exists:** `capsule_applet` (review, signed-pack and component verification, grants) is
  portable Rust with no browser dependency; `muniment/directory` gives the document store a
  native `Backend` (the pane's `store::load` and `save` are generic over `DocumentStore`);
  `cambium-desktop` and `cambium_genet_winit_host::run` give native windows
  (`graphshell-alembic`, `graphshell-coop`, `goal-surface-host`); `crates/script/app-host`
  runs components under wasmtime 45 with a watchdog epoch and `StoreLimits`.
- **Does not exist:** a host for the `block-guest` world. `app-host` binds only `app-core`
  (`bindgen!` per world; "two worlds, two hosts" is the existing pattern with `document-host`).
  A native block pane therefore needs a third host crate, **`crates/script/block-host`**, binding
  `block-guest` over the same `mere:script` WIT (via the endpoint's `links` metadata, as D10
  describes), with `guarded_engine` and `Watchdog` shared or copied from `app-host`, exposing
  `BlockScript::attach_bytes`, `describe`, `choices`, `apply`, `run`, `view` as blocking calls.
- **The pane must leave `web_tree`.** `blocks.rs` is written against the web page's private types.
  The generic pane splits into a host-neutral `graphshell::block_pane` (the `Pane` state, the
  registry, the view builders, over a small trait giving size, appearance, storage, wake and the
  adapter calls) and two bindings: the existing `web_tree` one (storage `IndexedDbBackend`,
  adapters through the Worker, results memoised) and a native one. This is the largest single piece
  of work in D13 and should land before the guest moves (order of work, step 3).
- **Which native shell:** a new bin **`graphshell-blocks`** in `ports/graphshell`
  (feature `editable-blocks-native`), a Cambium desktop window in the style of `graphshell-alembic`:
  one window, the generic pane, a document directory under the user's data dir, and a review
  dialog. None of the three existing native shells is a document editor, and putting the pane in
  one of them would couple it to the coop session or the goal surface. Knot's desktop app is the
  natural document-editing shell and could host the same `block_pane` later, but it is another
  repository's application and is not part of this design (R3).
- **Same pack, same review.** The native shell loads the same signed pack as the web one:
  `CapsuleMount::review` verifies the signature, the component digest and size; `approve` grants
  only `power:block-adapter`, and only if the pack asked; the review dialog names the class the
  component claims and the author key, and a new grant needs a new review. The disclosure comes
  from a Moot collection through `ports/moot` when the shell has one, exactly as on the web; a
  directory of already-fetched packs is the offline form. Revocation drops the instance and its
  registry entry.
- **Calling model.** Native calls are in process and bounded by the watchdog, so they are
  synchronous and answer inside the frame; the pane's memo is still keyed the same way and is
  simply always fresh. The component never runs on the render thread unguarded: a turn that hits
  its epoch deadline traps, the host survives, the registry entry is dropped and the pane shows the
  interim state with the refusal text.
- **What native adds to the guarantees:** nothing new for Cargo. `block-host` and
  `graphshell-blocks` are Mere crates and name no Woodshed package.

**Where the pack comes from.** Woodshed's release publishes the signed pack and its disclosure
entry. Mere does not commit, vendor or fetch it at build time: a pack checked into Mere would be a
Woodshed artifact inside Mere's tree, and a build-time fetch would be a source edge. The product
site that serves the page may list it as a disclosed capsule (an operations choice; ruling 4 settles the mechanism),
and a user may add one by address.

## What moves and what stays

**To Woodshed** (receiver builds and tests first, blobs compared, then Mere deletes):

| From Mere | To Woodshed | Tests |
|---|---|---|
| `block_editing/bars.rs` (741 lines): the adapter logic | `woodshedding` feature `blocks` | the 5 pure `bars` tests move unchanged |
| `bars::scene` (the arrangement `SceneInput`) | `woodshed-notation` feature `block-guest` | covered by a view test |
| notation-view portion of `web_tree/blocks.rs` (about 350 to 450 lines: `insert_progression`, `write_notation`, `notation_*`, the `NotationLeaf` leaf, bar-line export, backdrop scene call) | `woodshed-notation` feature `block-guest`, as `view`, `run` and the claim's `example` | none exist; add component round-trip tests |
| `support/woodshed-block-example.json` | `woodshed-notation` (`block-guest`) | covered by the example test |

Tests (ruling 7): 5 of the 8 are pure and move unchanged to `woodshedding` (count 8 before; 5 plus
3 rewritten after). The 3 that use `Document` and `Preview` (write-notation atomicity, the
saved-music reading, grouping through one history) are rewritten against the adapter's own types
(attribute maps and `change` records), so Woodshed's lock gains no editor stack. The one thing
they proved through `Document::apply` (a returned map changes only that block's attributes, in one
undo step) is covered by **one conformance test in Woodshed** that loads the real pack through
Mere's host and applies a result through `Document`. That test is a Woodshed dev-dependency on Mere
(guest above host, legal under D6) and lives under `woodshed-notation`'s `block-guest` feature, so
it adds `graphshell` (with `editable-blocks`, `knot-editor-host`, `illume`) to Woodshed's root lock
only. The count is 8 before and 8 plus the conformance test after.

**Stays in Mere, generalised** (`ports/graphshell`):

- `block_editing`: `Document`, `Preview`, `Change`, `store`, `rhyme`, `MAX_DOCUMENT_BYTES`, and
  `Div`, unchanged. New: `trait BlockAdapter { claim; choices; apply; run; view }` and a
  `Registry` keyed by class.
- `web_tree/blocks.rs` shrinks to the generic pane: `has_class` branches become registry
  lookups; the command ids and labels come from claims; the one `woodshed:group-bars` command and
  "Insert Woodshed example" become claim-driven entries; the notation leaf is replaced by a
  generic "adapter view" that renders the returned scene and turns `regions` into buttons;
  `SHEET`'s `blocks-notation-*` classes are renamed `blocks-guest-*`.
- `rhyme.rs` becomes the first `BlockAdapter` implementation (built in). This is the evidence
  that the trait is not Woodshed-shaped: two implementations before the first guest exists.
- `block_pane` (new, host-neutral) and `crates/script/block-host` (new, native host); see Native.
- `capsule_applet.rs` gains `power:block-adapter`; `web/applets/worker.mjs` gains the
  request/response path next to the action path.
- A fixture block guest (a small component built in Mere, not Woodshed) for Mere's own tests of
  the loader, the registry and the stale memo.

## Interim behaviour

Until a block adapter claiming `progression` is installed and approved:

- The pane opens and edits Djot documents, applies rhyme, saves, undoes and reopens exactly as
  now. A saved document that contains `progression` blocks opens and saves byte for byte: the
  host only edits a block through an adapter, so the attribute map, including the `woodshed`
  source attribute, is never touched.
- A `progression` block is selectable as an ordinary div. Its command is disabled with the
  message the pane already has for an unclaimed class ("This block has no registered editing
  adapter"), extended with the class name and a "Review block adapters" entry that opens the
  review of any disclosed capsule that claims it, if the product site discloses one.
- Absent, not disabled: "Insert Woodshed example", bar grouping, "Write notation", the
  Chart/Tab/Score view, the bar-line plaintext export and the progression backdrop scene.
- Plainly lost in the meantime: everything in that last bullet, and nothing else. Rhyme editing,
  saving, undo and reopen are unaffected.
- A side effect worth having: the product wasm shrinks by `woodshed-notation` and
  `woodshedding`. The viewer cone had none of them already, so `build-viewer.py`'s delivery caps
  are unaffected; the full product build is rechecked at the move.

## Cycle check

Standard done conditions 1 and 3, with D5 and D6, applied to each piece:

- **Mere's locks.** `woodshedding` leaves `ports/graphshell/Cargo.toml` (`:56`, `:222`);
  `woodshed-notation` leaves `ports/graphshell/web/Cargo.toml` (`:40`, `:53`).
  `editable-blocks` keeps its name for the generic pane and loses `dep:woodshedding`. After
  relocking, `Cargo.lock` (now `woodshedding` at line 21327) and
  `ports/graphshell/web/Cargo.lock` (now `woodshed-notation` at 12296 and `woodshedding` at
  12306) hold no Woodshed package. The new `block-host` crate and `graphshell-blocks` bin add
  Mere-only edges.
- **No dev edge.** Mere's tests of the pane, loader, registry and `block-host` use a fixture guest
  built in Mere. No Mere test, dev-dependency or committed file names a Woodshed package or contains
  a Woodshed pack. The real pack is exercised only from Woodshed (the conformance test: a Woodshed
  dev-dependency on Mere, which D6 permits because Mere does not depend on Woodshed), or from
  `Code/testing/mere`.
- **The guest has no Cargo edge from Mere at all.** A signed pack is data. The cycle check reads
  locks, so a runtime-loaded component is invisible to it by construction; the guarantee is the
  rule above, not the check, and the plan's P7 done list carries it.
- **Graphshell (the protocol repo).** The WIT gains a world (and R1 may add a drawing vocabulary
  to `sceno`); it gains no dependency. Woodshed already takes `sceno` from it.
- **Woodshed's lock roots** (8 `Cargo.lock` files on main, 2026-11-06). A `Cargo.lock` records
  optional dependencies whether or not a feature is on (the plan's Findings), so what the features
  add to `woodshedding` and `woodshed-notation` reaches every lock that holds those packages.
  Counts are packages whose lock `source` is that repository's git URL; "holds" is checked in each
  lock.

  | Root | Mere pkgs | Graphshell pkgs | Holds `woodshedding` / `woodshed-notation` | At the move |
  |---|---|---|---|---|
  | `Cargo.lock` (workspace) | 32 | 4 | yes / yes | relock; gains `wit-bindgen` edges and the conformance test's `graphshell` editor stack |
  | `validation/relationship-recipe` | 26 | 4 | yes / yes | relock (the features' optional deps are recorded) |
  | `examples/musicxml-timing` | 0 | 0 | yes / no | relock; gains only `serde_json` and its registry dependencies, because `blocks` is the only feature on `woodshedding` and needs nothing else |
  | `ports/hocket` | 22 | 2 | no / no | untouched |
  | `ports/redshank` | 25 | 2 | no / no | untouched |
  | `ports/redshank/spikes/playback` | 0 | 0 | no / no | untouched |
  | `ports/ringdown` | 0 | 0 | no / no | untouched |
  | `testing/campaigns/audio-packs-ap0` | 0 | 0 | no / no | already broken (known), not chased |

  Three roots hold `woodshedding`, so three relock. This corrects my first draft, which expected
  Hocket and Redshank to churn: their locks hold neither package. The split in "What Woodshed
  ships" is what keeps `examples/musicxml-timing` from gaining Mere or Graphshell packages: a
  spike dependency closure for `woodshedding` plus `sceno`, `scenomise` and `wit-bindgen` is 4
  family packages (`sceno`; `mere-curation`, `scenograph`, `scenomise`) and 37 registry packages
  (`/Users/markik/Code/worktrees/cycles-20261106/p7/spike`, `cargo metadata`, normal edges,
  wasm32-wasip2). Putting those on `woodshedding` itself, as a literal reading of ruling 5 would,
  makes the portable core's example lock name Mere and Graphshell. Relock in dependency order
  (Graphshell, Mere, Woodshed) with `cargo +1.99.1 update -p` per the family repin loop; Woodshed's
  toolchain row is 1.97.1.

## Order of work (after review)

1. Mark answers R1 to R3.
2. In the graphshell repository: the `block-guest` world and `block-adapter` interface, and whatever
   R1 adds for the drawing (cut first: the receiver builds and tests; the protocol repo's
   conformance fixture gets a block-guest round trip).
3. Mere: `BlockAdapter`, the registry, rhyme as the first implementation; the generic pane
   extracted from `web_tree` into `block_pane` with the interim behaviour and the memo; the web
   binding (Worker path, `power:block-adapter`); `crates/script/block-host`; the `graphshell-blocks`
   native bin; a fixture block guest built in Mere. The Woodshed code stays in Mere and keeps
   working until step 5 is verified.
4. Woodshed: `woodshedding/blocks` and `woodshed-notation/block-guest`, the pack script, the
   rewritten tests and the conformance test; blob comparison against Mere's copies; relock the
   three roots.
5. Mere: delete `bars.rs` and the notation-view portion, the two manifest rows, relock the web
   lock and the main lock; build with `web/scripts/build-viewer.py` and check the digest; run
   `graphshell-blocks` against the real pack; run the cycle proof over both repositories' lock
   roots.
6. Docs: plan Progress, source homes map ("Graphshell names three things", "Repository cycles"),
   Woodshed's docs, this document's status.

## Rulings

Mark, 2026-11-06, on the eight decisions of the first draft (recorded as
[cycle_breaks D13](../implementation_strategy/2026-11-06_cycle_breaks_plan.md)); R1 to R3 are
new, from the spike and from rulings 5 and 6.

1. **Interface: ruled.** "Put `block-adapter` in the graphshell protocol WIT, as a new
   `block-guest` world in merely-made/graphshell." See Layer.
2. **View: ruled, with a condition that was triggered.** "A `sceno` scene plus regions, but run a
   spike first. If the spike falls short, come back to him before choosing differently." It fell
   short: see Spike and R1. Nothing different has been chosen.
3. **Calling: ruled.** "A Worker, asynchronous, with a memo and stale state." See Delivery; native
   is in process (Native).
4. **Discovery: ruled.** "A Moot capsule that is reviewed once." See Delivery and Native.
5. **Crate: ruled.** "A feature of `woodshedding`, not a new crate." Not the first draft's
   recommendation; Delivery and the Cycle check are rewritten around it. It cannot hold for the
   view without inverting an edge: R2.
6. **Native: ruled.** "Build a native block pane now, over `app-host`." Not the first draft's
   recommendation; see Native. `app-host` itself binds only `app-core`, so the host is a sibling
   crate, `block-host`.
7. **Tests: ruled.** "Rewrite them against the guest's own types, and add one conformance test
   through Mere's host." See What moves and what stays.
8. **Unclaimed class: ruled.** "Disable the command, name the class, and add a 'Review block
   adapters' entry." See Interim behaviour.

### R1 to R3 (ruled, D16)

- **R1: ruled.** "Scene + vector sheet." `view-result` carries the `sceno` scene (regions, names,
  order) and a protocol-owned, versioned **vector sheet**: compound fills with a fill rule,
  strokes, theme-role paint and text runs. The host paints the sheet; the guest never draws.
  Direct manipulation, if wanted later, gets its own design.
- **R2: ruled.** "Split across both." `woodshedding` feature `blocks`; `woodshed-notation` feature
  `block-guest`, as in Delivery.
- **R3: ruled.** "New graphshell-blocks bin." As in Native.

## Implementation status (2026-11-06)

Landed (SHAs in the cycle breaks plan's Progress): the `block-guest` world, `rows`, argument and
pattern fields, and the `vector-sheet` crate in Graphshell; in Mere the `BlockAdapter` trait,
registry, memo, rhyme as the built-in adapter, the vector sheet painter (`SheetLeaf`), the
`power:block-adapter` grant, `crates/script/block-host`, the `WorkerAdapter`, the block pane
dispatching by class (unclaimed class disabled and named, "Review block adapters" entry),
`graphshell-blocks` (native `GuestAdapter` and a headless render receipt), and the deletion of
`bars.rs` and both Woodshed rows; in Woodshed the `blocks` and `block-guest` features, the pack
builder and the conformance test. No Mere lock holds a Woodshed package.

Changes made on the way that the design did not state: `rows` is a sixth call; commands may declare
an argument field and the class a pattern field; the hidden `backdrop-scene` text command carries
the swatch scene; the progression's arrangement scene comes from `sceno-backdrops` (D17), so the
guest no longer depends on any Mere crate; the status text after an apply is generic.

Open: the host-neutral `block_pane` and a Cambium window for `graphshell-blocks` (ruling 6 asked for
a native pane; what exists is the native host and a headless receipt); the web Worker JavaScript
and the headed web check; the four notation scenarios; `power:block-adapter` review UI wiring in the
web host. Headless receipts: `/Users/markik/Code/testing/mere/guest_blocks_spike_20261106/real/`
(Chart, Tab and Score drawn by the real guest through the host's sheet painter, compound paths
intact, compared with the spike's leaf images).
