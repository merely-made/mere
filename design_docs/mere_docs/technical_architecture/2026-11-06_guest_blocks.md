# Guest blocks: Woodshed's progression and notation as a guest of the block pane

**Status:** 2026-11-06: design, unreviewed; nothing implemented, nothing moved. Written first,
as P1 was, because [cycle_breaks D12](../implementation_strategy/2026-11-06_cycle_breaks_plan.md)
(Mark: "D: Woodshed as a guest") chose a new mechanism. Mark reviews it before any code or file
moves. Cited by the plan's P7.

**For:** letting Mere's block pane host Woodshed's progression editing and notation view without
Mere naming Woodshed in any manifest or lock (D5's end), while the pane keeps working with no
Woodshed installed.

**Not here:** the guest model for other apps (the host/guest brief's recursion is untouched); the
domain registration seam (`2026-11-06_domain_contract.md`, which is about stores, not views);
a native block pane, which does not exist today (Delivery).

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
- **Not in the interface:** network, storage, clipboard, time. The world imports `log` and `caps`
  only, so the component is a pure function of its inputs; "unimported means unreachable" does
  the sandboxing, and review has little to grant.

**Two things that must be checked before the first line is written** (not assumed here):
(1) whether `sceno::Scene` and `Region` can carry a chart, tablature and staff reading with
event-level hit regions, or whether `woodshed-notation` needs a scene output beside its paint
leaf (`bars::scene` already produces a scene-shaped `SceneInput` for the backdrop, but not for the
notation leaf); (2) the size of a component built from `woodshedding` and `woodshed-notation`
against the 16 MiB component cap.

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

**This design adds nothing to `merely-made/graphshell`.** Putting the interface there is a
ruling for Mark (Decision 1): the common rules say a contract item is reported, not added.
Until he rules, the WIT lands nowhere.

```
merely-made/graphshell   block-adapter WIT, sceno           (protocol, below Mere)
        ^                         ^
merely-made/mere     host: BlockAdapter trait, pane,        (host, above the protocol)
                     guest wrapper, review + grants
        ^
merely-made/woodshed guest: woodshed-blocks (component),    (guest, above Mere and
                     built against the WIT; tests use Mere   the protocol)
```

No arrow points from Mere to Woodshed. Woodshed to Mere and to Graphshell are the legal edges
already in its lock.

## Delivery

**What Woodshed ships.** A new crate (proposed `crates/woodshed-blocks`, a workspace member
beside `woodshedding`, not a feature of it, so `woodshedding`'s dependencies and the other
Woodshed roots do not move). It holds the moved `bars.rs` logic against the WIT's `block`,
`change` and `attr` types, and the notation-view portion, producing `view-result`. It builds for
`wasm32` as a component with `world block-guest`, exporting one `claim` (`progression`). A
Woodshed script packages it as a signed pack: `PackManifest { name: "woodshed-blocks", parts:
[WasmComponent], requested_scopes: ["power:block-adapter"] }`, signed with Woodshed's author key
through the same `eidetic` pack-signing path the applets use. The same crate builds natively as an
rlib, so its unit tests (including the 8) run without wasm.

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

**Native.** `editable-blocks` requires `web` and `applets` is web only, so there is no native
block pane and no native applet host in `ports/graphshell` today. Nothing in D12 needs one. The
pack format is identical for the wasmtime-based `crates/script/app-host`, so a native pane would
load the same signed pack under the same review; building that is outside this design and listed
as Decision 6.

**Where the pack comes from.** Woodshed's release publishes the signed pack and its disclosure
entry. Mere does not commit, vendor or fetch it at build time: a pack checked into Mere would be a
Woodshed artifact inside Mere's tree, and a build-time fetch would be a source edge. The product
site that serves the page may list it as a disclosed capsule (an operations choice, Decision 4),
and a user may add one by address.

## What moves and what stays

**To Woodshed** (receiver builds and tests first, blobs compared, then Mere deletes):

| From Mere | To Woodshed | Tests |
|---|---|---|
| `block_editing/bars.rs` (741 lines) | `woodshed-blocks`, as the guest's adapter logic | the 8 `bars` tests, count 8 before and after |
| notation-view portion of `web_tree/blocks.rs` (about 350 to 450 lines: `insert_progression`, `write_notation`, `notation_*`, the `NotationLeaf` leaf, bar-line export, backdrop scene call) | `woodshed-blocks`, as `view`, `run` and `describe`'s `example` | none exist; add component round-trip tests |
| `support/woodshed-block-example.json` | `woodshed-blocks` | covered by the example test |

Tests: 3 of the 8 use `Document` and `Preview`. Two ways to keep them (Decision 7): a Woodshed
dev-dependency on Mere's `graphshell` library with `editable-blocks` (legal, guest above host,
but it puts Mere's `graphshell`, `knot-editor-host` and `illume` in Woodshed's lock), or rewrite
the three against the guest's own `change` and `attr` types and have one conformance test in
Woodshed load the real pack through Mere's host, which exercises `Document::apply` once.

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
  12306) hold no Woodshed package.
- **No dev edge.** Mere's tests of the pane, loader and registry use the Mere-built fixture
  guest. No Mere test, dev-dependency or committed file names a Woodshed package or contains a
  Woodshed pack. The real pack is exercised only from Woodshed (guest above host: a Woodshed
  dev-dependency on Mere, which D6 permits because Mere does not depend on Woodshed), or from
  `Code/testing/mere`.
- **The guest has no Cargo edge from Mere at all.** A signed pack is data. The cycle check reads
  locks, so a runtime-loaded component is invisible to it by construction; the guarantee is the
  rule above, not the check, and the plan's P7 done list carries it.
- **Graphshell (the protocol repo).** If Decision 1 puts the WIT there, `graphshell` gains a
  world and no dependency; Woodshed already takes `sceno` from it.
- **Woodshed's lock roots** (8 `Cargo.lock` files on main, 2026-11-06):

  | Root | Mere packages | Graphshell packages | At the move |
  |---|---|---|---|
  | `Cargo.lock` (workspace) | 32 | 4 | takes `woodshed-blocks`; relock; gains the WIT source |
  | `validation/relationship-recipe` | 26 | 4 | path-depends on `woodshedding` and `woodshed-notation`; relock only if either's dependencies change (a new crate leaves them unchanged) |
  | `ports/hocket` | 22 | 2 | relock only if it takes a changed crate |
  | `ports/redshank` | 25 | 2 | relock only if it takes a changed crate |
  | `examples/musicxml-timing` | 0 | 0 | names `woodshedding` or the notation crate; no relock expected |
  | `ports/ringdown` | 0 | 0 | none |
  | `ports/redshank/spikes/playback` | 0 | 0 | none |
  | `testing/campaigns/audio-packs-ap0` | 0 | 0 | already broken (known), not chased |

  Counts are packages whose lock `source` is that repository's git URL. The four roots that
  name Mere and Graphshell are the ones the repin loop touches. Keeping the guest in a new
  crate keeps `woodshedding`'s own graph unchanged, so the neighbouring roots do not churn.
  Relock in dependency order (Graphshell, Mere, Woodshed) with `cargo +1.97.1 update -p`.

## Order of work (after review)

1. Mark rules the decisions below.
2. If the WIT is a protocol item: add `block-adapter` and `block-guest` to the graphshell
   repository (cut first: receiver builds and tests).
3. Mere: `BlockAdapter`, the registry, rhyme as the first implementation, the pane rewritten over
   the registry with the interim behaviour, `power:block-adapter`, the worker path, the fixture
   guest. The Woodshed code stays in Mere and keeps working until step 4 is verified.
4. Woodshed: `woodshed-blocks` with the moved code and tests, the pack script, the conformance
   test; tests and blob comparison against Mere's copies.
5. Mere: delete `bars.rs` and the notation-view portion, the two manifest rows, relock the web
   lock and the main lock; build with `web/scripts/build-viewer.py` and check the digest; run
   the cycle proof over both repositories' lock roots.
6. Docs: plan Progress, source homes map ("Graphshell names three things", "Repository cycles"),
   Woodshed's docs, this document's status.

## Decisions for Mark

1. **Where does the `block-adapter` interface live?**
   - (a) The WIT file in `merely-made/graphshell`, as a new world. One source, usable by any host
     above the protocol; the file already holds non-projection worlds. Recommended.
   - (b) A Mere-local WIT beside `app-host`. Smaller blast radius, but it is a second copy of a
     wire name, and D10's links-metadata rule would apply to it too.
   - (c) No WIT: a Rust trait object loaded only by a native host. Does not reach the web
     product, which is the only block pane that exists. Not recommended.
2. **What does a guest's view return?**
   - (a) A `sceno` scene plus accessible regions, drawn by the host. Recommended: no toolkit in
     the guest, the host keeps focus and accessibility.
   - (b) A retained applet surface (a Cambium view tree). More expressive; the guest then drives
     layout and focus inside Mere's tree, and the guest needs the toolkit.
   - (c) Paint commands (Sprigging). Cheapest port of `NotationLeaf`, but it names Sprigging in the
     contract, and Sprigging is a Mere crate.
3. **How does the web host call a guest, given the worker is asynchronous?**
   - (a) Worker, asynchronous, with the pane memoising results and showing stale state.
     Recommended: same isolation as applets, and the cost is in one place.
   - (b) Instantiate the jco adapter on the main thread and call synchronously. Simplest for the
     pane, but guest CPU runs on the render thread, and review would have to bound it.
   - (c) Worker, and block the pane's disabled/enabled state until the first answer. Simple, but
     the command bar flickers on every selection.
4. **How does the product find Woodshed's pack?**
   - (a) Woodshed publishes the signed pack; the product site discloses it as a capsule in a Moot
     collection and the user reviews it once. Recommended: no Cargo edge, no vendored artifact,
     and the existing review UI is reused.
   - (b) A "Add block adapter" control that takes an address or file. Works with no site
     support; the first-run experience is a user chore.
   - (c) The product ships the pack as a static asset next to the wasm. Best first run, but the
     bytes are a Woodshed artifact inside the product's release; acceptable only if it is built
     in Woodshed's release and copied by the site's deployment, never committed in Mere.
5. **Where does the guest live in Woodshed?**
   - (a) A new crate `crates/woodshed-blocks`. Recommended: `woodshedding`'s dependencies stay
     the same, so the other Woodshed roots do not churn.
   - (b) A feature of `woodshedding`. Fewer crates, but it adds the WIT and wasm toolchain to the
     portable core that Hocket and Redshank use.
6. **Is a native block pane in scope?**
   - (a) No, as today. Recommended: D12 loses nothing the user has natively.
   - (b) Build it now over `app-host` (wasmtime) with the same pack and review.
   - (c) Decide when a second host is planned (Stack 1's rule for leaving a port).
7. **The 3 `Document`-dependent `bars` tests.**
   - (a) Rewrite them against the guest's own types and add one conformance test that loads the
     real pack through Mere's host. Recommended: keeps Mere's editor stack out of Woodshed's lock.
   - (b) Keep them unchanged with a Woodshed dev-dependency on Mere's `graphshell`
     (`editable-blocks`). Same tests and same count, but the lock gains `knot-editor-host` and
     `illume`.
8. **The interim text.**
   - (a) Disable the command with the class name and a review entry when a capsule is disclosed.
     Recommended.
   - (b) Silent: the block is an ordinary div with no explanation.
   - (c) Hide progression blocks' source behind a read-only card until an adapter is installed.
     Safer against hand-edits of the structured source, but it hides the user's own text.
