# Reservoir plan: shared meres held by the device resident

**Date:** 2026-09-23
**Status:** in progress. V1 is complete and on main: the pandect index,
wallet-persona resolution, djinn's reservoir lane and route, and a real
two-process receipt. It reached origin with `5364dfa0` on 2026-09-24. V2's
shape was ruled on 2026-09-23 and 2026-09-24 (§7). Steps 1 to 3
(muniment, graph-kernel, pandect) landed on 2026-09-24 and reached origin on
2026-09-25; step 3b, undo with exact replay, landed on 2026-09-25 and reached
origin the same day. Step 4, `MereHost` on `GraphSession` in Graphshell, landed
and reached origin on 2026-09-25. Its browser runtime is unproven: updating
`graphshell-web`'s genet and netrender and a browser receipt come next (§7
item 29), then step 5, djinn's routes.
**Scope:** give each data domain one mere, and make every mere of an identity
openable by any of that identity's applications. The meres are held by the
device resident, with sessions, a graph journal and an Eidetic archive.
Cleromancy is the first consumer; its own plan
(`repos/cleromancy/design_docs/2026-09-23_divination_mere_plan.md`)
builds on this one.

**Related:**
- [Ambiance design](../design/2026-09-23_ambiance_design.md), which records
  the rulings this plan implements.
- [Device resident consolidation](2026-08-20_device_resident_consolidation_plan.md),
  which rules the ownership model: one owner per durable store, with apps as
  clients.
- [Graphshell reference host](2026-07-27_graphshell_reference_host_plan.md),
  the local session and admission layer.
- [Family-shared identity](2026-08-08_family_shared_identity_plan.md), which
  created the shared root.
- [Alembic memory and codicils](../technical_architecture/2026-06-09_alembic_memory_and_engrams.md)
  and its [implementation plan](2026-06-24_alembic_implementation_plan.md).
- [Browser multiplexer framing](../research/2026-05-11_browser_multiplexer_framing.md),
  the durable graph session.

## 1. Ruling

Mark, 2026-09-23 (quoted in full in the ambiance design, §2):
- "each data domain deserves its own mere. that isn't the same as every app
  getting its own mere, like two browsers should share a mere if you ask me";
- "all apps should be able to access any of an identity's meres by default (at
  least exceptions should have a really good reason)".

A mere is its domain's repository. It lives as sessions, with the full
lifecycle (mint, switch, fork, trash). Eidetic saves sessions and archives
them as codicils; a session can be opened from a codicil, and editing forks it.
All of an identity's meres together are its **reservoir**.

Asked where Cleromancy's mere should live, Mark chose "Mere change first": this
plan makes meres shared before any consumer moves in.

The ownership model is not new. The device resident plan rules that "One
logical device resident owns the selected persona's durable stores". It says
first-party applications "are clients of that resident when they use
persona-held state". A shared mere is persona-held state. So the reservoir
belongs to the resident, and applications reach it through Graphshell local
sessions.

## 2. Findings (verified 2026-09-23)

- **Sessions are app-private today.** The shared root's own rule, in
  `crates/system/pandect/src/shared_root.rs`, is: "**personas and the identity
  wallet are family-shared, and everything else stays app-private.** An app
  keeps its own root for sessions, graphs, and settings". The shared root is
  the platform data directory plus `mere`, or `MERE_ROOT`.
- **Turnstone is the working session precedent.** Each session owns
  `sessions/<id>/`, and "the manifest set is pandect's `ManifestStore`"
  (`repos/turnstone/src/session.rs`). The lifecycle is "boot, mint, adopt,
  fork, trash" (`repos/turnstone/src/app/session_lifecycle.rs`). At boot
  Turnstone installs the graph journal's capture hook: "every mutation that
  flows through apply_graph_delta records here under the current author".
- **The graph journal is Mere's edit spine.** "The materialized graph is the
  *replay* of the journal", a `muniment::Journal` of captured deltas with
  snapshot checkpoints (`crates/graph/graph-kernel/src/graph/journal.rs`;
  `journal_capture_hook`).
- **Codicil freeze and thaw exist.** "Save as graph codicil" and "Open as
  session", with read-only browsing, "editing forks a thaw" and codicil compose
  (`crates/system/pandect/src/graph_codicil.rs`, `snapshot_merge.rs`). Athanor
  already consolidates with `compose_graph_codicils`
  (`ports/distillery/athanor/src/lib.rs`). The Alembic plan records "slices A-C
  landed; merge, promotion, event-log, LoRA, and settings/Timeline follow-ons
  remain deferred."
- **The resident pattern is proven on Knot.** In the device resident plan:
  - R1 made the application door route-aware, with application-to-route
    grants;
  - R2 split a shared resident source from per-session endpoint adapters, so
    "one visitor could consume another visitor's revision bell" cannot happen;
  - R3 composed Knot into djinn;
  - R4 made standalone Knot "embed the same resident library when it owns the
    whole application process".
  The plan's status records R1 through R5 as complete. Its invariant 1: "A
  persistent redb or iroh blob store has one live process owner."
- **Knot already takes identity from the shared root.** Its sync host resolves
  `pandect::shared_root::shared_root` by default
  (`repos/knot-editor/crates/knot-editor/src/bin/knot_sync_host.rs`).
- **Cleromancy has none of this yet.**
  - `CleromancyHost` holds one Mere kernel `Graph` and saves it whole, as one
    snapshot document in one muniment slot (`repos/cleromancy/src/host/mod.rs`).
    It has no pandect sessions, no graph journal and no codicils.
  - Its data root is `%LOCALAPPDATA%\cleromancy` (`repos/cleromancy/src/main.rs`).
  - It runs its own resident authority, `CleromancySessionAuthority`
    (`repos/cleromancy/src/admitted.rs`).
  - No store exists at that default root on the primary development machine.

### V2 findings (verified 2026-09-24)

- **Nothing persists the graph journal yet, and its author is a free string.**
  `AttributedDelta` carries `author: String`: "`user` for the trusted UI path, a
  participant subject's hex for gated runs, `pre-gate` for entries migrated
  from bare logs" (`crates/graph/graph-kernel/src/graph/journal.rs`). No code
  outside that file calls `GraphJournal::save`, `load` or `migrate_bare_log`,
  so the stored form can still change freely. muniment writes a journal whole:
  "the whole log is written as a single slot on each `save`", and the
  append-friendly form "is the roadmap"
  (`crates/eidetic/muniment/src/journal/persist.rs`).
- **Capture is one hook per thread.** `set_captured_delta_hook` installs "the
  current thread's graph-delta capture hook"
  (`crates/graph/graph-kernel/src/graph/capture.rs`), and Turnstone installs
  it once at boot (`repos/turnstone/src/app/session_lifecycle.rs`). A resident
  holding many sessions cannot share one hook per thread. The event-log plan
  had already proposed recording as "a per-`Graph`-instance opt-in" (its E1).
- **No graph diff or inverse exists** in the kernel or pandect, so undo needs
  one.
- **Positions and view state bypass the journal.**
  - Node positions are written into `arrangement.*` facets at save time: "only
    the durable save-time position lands here"
    (`crates/system/pandect/src/arrangement_facets.rs`).
  - Per-pane view state is pandect's `ViewIntent`: hidden relations, folds,
    camera, focus, layout strategy and mirrored tiles, saved under
    `views/<frame>/<pane>.json`
    (`crates/system/pandect/src/view_intent_store.rs`).
- **Two storage styles.**
  - `MereHost` persists its graph, facets and projection epoch and revision as
    one muniment slot (`ports/graphshell/src/mere_host.rs`). Graphshell runs it
    on IndexedDB in the browser (`ports/graphshell/src/web.rs`) and on memory
    in native receipts. Cleromancy's host follows the same one-slot pattern
    (`repos/cleromancy/src/host/mod.rs`).
  - Turnstone's session files are loose JSON, written by pandect with plain
    `fs::write` (`crates/system/pandect/src/session_graph_store.rs`).
- **muniment has no directory backend.** It ships memory, redb, zip and
  IndexedDB backends. `Backend::apply` is atomic by contract: "every op lands,
  or none does" (`crates/eidetic/muniment/src/backend.rs`). The stack already
  keeps a log one entry per key: stickleback's `MunimentStore` stores
  `log/<author>/<log>/<seq>`, with "a zero-padded 16-hex sequence number so
  keys sort in log order and a single scan walks a log"
  (`crates/stickleback/src/store.rs`).
- **Earlier rulings already shape the log.**
  - Alembic decision #5 (2026-06-24) made view intent a "**parallel composed
    stream**" (`2026-06-24_alembic_implementation_plan.md`, §3). The event-log
    plan keeps it "**never** merged into the `GraphMutation` log — a
    structural undo and a camera pan must stay independent"
    (`2026-07-01_event_log_timeline_plan.md`, E4).
  - The graph view curation plan, which superseded that plan's substrate, gives
    durable graph history to `GraphJournal` and durable local curation to the
    view-intent store. It also makes "Restore from here … a separate,
    confirmable action using the existing delta/engram boundary"
    (`2026-08-03_graph_view_curation_and_interaction_plan.md`, Ownership and
    Time).
- **Turnstone's fork is a component tear-out.** `fork_session_from` copies the
  connected component around a seed node into a new session with
  `Graph::copy_component_from`, carries facets through the id remap with
  `copy_node_facets` and `copy_scene_facets`, and sets `parent_session`. It
  also carries Turnstone's resident admissions and nested participant worlds,
  which stay Turnstone's dressing (`repos/turnstone/src/app/session_lifecycle.rs`).

### Step 3b findings (verified 2026-09-25)

- **Replay was not exact.** Creating a node stamps its visit time from the
  clock (`Graph::add_node_with_id` in `crates/graph/graph-kernel/src/graph/mod.rs`),
  and every semantic relation mints its statement id from the time, a device
  salt and a counter (`mint_local_statement_id` in
  `crates/graph/graph-kernel/src/types.rs`). Neither was journaled, so a replay
  re-stamped and re-minted. The journal's own doc claimed replay "cannot
  diverge". Undo's first session test failed on it, since undo compares a
  replayed graph with the live one.
- **Loading a checkpoint was not exact either.** `Graph::from_snapshot` imports
  a snapshot's node columns as facets, default values included, and
  `overlay_facets` keeps them where `facets.json` has no key. A reloaded session
  held facets its live graph never had. The session core now replaces the facet
  store with the one it saved, which the kernel names "the single live
  authority".
- **graphshell's lib tests do not build from a worktree.** Two of them include
  `woodshed/scenarios/woodshed_musical_comparison.json` by a relative path
  that resolves only from `repos/mere`, not from `worktrees/`.

### Step 4 findings (verified 2026-09-25)

- **pandect did not build for the browser.** Graphshell's browser build
  (`ports/graphshell/web`, `wasm32-unknown-unknown`) runs `MereHost` over
  IndexedDB, but Graphshell declares pandect native-only, and pandect fails
  that target. getrandom 0.2 entered through two sources:
  - pandect's own `rand_core 0.6`, for three nonce and secret draws;
  - `p2panda-core`, whose `rand` dependency is unconditional. pandect used it
    only for two CBOR helpers, each one `ciborium` call.

  The crate doc's "compiles wasm32-clean" held for `wasm32-wasip2` only.
  Graphshell's join had already moved to getrandom 0.3 for the same reason
  (`ports/graphshell/src/webrtc_join.rs`).
- **The manifest read std's clock.** `GraphSessionManifest::new`, `touch` and
  `record_consolidation` called `std::time::SystemTime::now()`, which panics in
  a browser, and minting or forking a session calls `new`. A build cannot catch
  it. The kernel forbids that call for this reason and reads the clock through
  `web_time` (`crates/graph/graph-kernel/src/lib.rs`).

## 3. Target shape

```text
Cleromancy   Turnstone   Knot   Graphshell   other first-party clients
      \          |         |        |          /
          Graphshell local sessions (admitted, route-granted)
                              |
                    device resident (djinn, or embedded)
                    `- reservoir source (one per persona)
                         |- mere: divination, journaling, RNG
                         |    |- sessions (pandect manifests, full lifecycle)
                         |    |- graph journal per session (attributed)
                         |    `- Eidetic archive (graph codicils)
                         |- mere: browsing (shared by browsers)
                         `- ... one mere per data domain
```

- **One owner.** The reservoir source owns every mere's stores and serializes
  authoritative mutations per mere. Each attached application gets its own
  session adapter, with its own caches and notice cursor. That's R2's shape.
- **Access.** Any admitted first-party application of the identity may open
  any mere by default. An exception is an explicit, recorded denial. Ambient
  material crossing from one mere into another application's ambient tier
  needs a separate opt-in per mere and per app (ambiance design §9).
- **No daemon required.** A standalone application embeds the same resident
  library when it owns the whole process (R4's precedent). If djinn is running,
  the application is its client and never opens the stores itself.
- **App-private stays app-private.** Settings, caches and embedded local-only
  work remain under each application's own root, as invariant 9 allows.

## 4. Phases

### V1. A mere in the reservoir

Define the mere record: an id, the data domain it holds, and its session
manifest set. A data domain has a stable identifier, such as `divination`, so
every application finds the same mere (ruled, §7). The reservoir lives under
`<shared root>/personas/<persona>/reservoir/`, beside the identity its meres
belong to (ruled, §7). Add a reservoir source to the resident that lists,
creates and opens meres for the selected persona, and exposes those as
admitted routes.

**Done when:**
- the resident lists an identity's meres;
- creating a mere for a domain that already has one returns the existing mere;
- a mere survives a resident restart;
- a second process that tries to own the reservoir fails clearly instead of
  hanging on a lock;
- an admitted application opens a mere by id through a Graphshell local
  session and never touches its files.

### V2. Sessions with the full lifecycle and the full log

Move Turnstone's session lifecycle into pandect as product-neutral machinery,
and have every application adopt it (ruled, §7). Turnstone keeps only its own
dressing as hooks: windows, lens spaces, participant runtime, resident
admissions and nested worlds. It adopts the pandect lifecycle and deletes its
copy in its own plan. The reservoir source runs the lifecycle for every mere.
Every session keeps Alembic's full structural log: graph mutations, plus view
and projection changes, replayable for the Timeline and for undo (ruled, §7).

**Shape** (ruled 2026-09-23 and 2026-09-24; §7 records each ruling):
- **One schema over muniment.** A mere is one muniment store. Each session
  lives at keys under `sessions/<id>/`:
  - its manifest (pandect's `GraphSessionManifest`);
  - the baseline snapshot its journal starts from;
  - the latest checkpoint: graph, facets and the journal cursor they reflect;
  - one key per journal entry;
  - per application and view, the current view state and one key per entry of
    its change stream.

  Step 3 fixes the key spellings.
- **The directory backend.** muniment gains a native directory backend.
  - It stores each key as a file in Turnstone's layout (`graph.json`,
    `facets.json`, `manifest.json`).
  - It appends a log's per-entry keys as the lines of one file,
    `journal.jsonl`.
  - A small redo file keeps `Backend::apply` atomic across files. The batch is
    written first, then applied, and replayed on open if a crash cut it short.

  On redb and IndexedDB the same keys are rows. The per-entry journal form
  lives in muniment beside `Journal`, so any journal can use it.
- **The journal is the authority.** A session is its baseline plus its
  journal, and replaying from the baseline is always correct. A checkpoint,
  written in one batch, only makes loading cheap. The resident writes one when
  the last application detaches from a session, and whenever 1,000 entries
  have accrued since the last. The count is a setting, to be tuned by measured
  replay cost.
- **Two streams.** Graph mutations go to the journal. View changes go to each
  view's own stream and are never merged into the journal (Alembic decision
  #5). Each view entry records the journal cursor it was made at, so the
  Timeline composes the two streams by cursor.
- **Authors.** Every journal and view entry carries a structured author:
  - its kind: person, rule, script or engine;
  - its id and version;
  - the application it came through.

  The resident fills in the application from the admitted route and never
  takes a client's word for it. The structured author replaces the kernel's
  free string while nothing persists it yet.
- **Recording per graph.** Each session's graph records its own mutations, as
  the event-log plan proposed, instead of going through the per-thread hook.
- **Exact replay.** What an edit reads from the clock or mints is journaled
  beside it: a new node's visit stamp as a touch, and the exact edge after a
  statement id is minted (ruled, §7 item 19). Replaying a session from its
  baseline, or from its checkpoint and tail, yields the whole live graph,
  facets and statement ids included.
- **Undo.** Undo reverts the undoing author's own latest change by appending
  its inverse under their name.
  - Other authors' later edits stay, and nothing is truncated.
  - The Timeline shows the undo.
  - A change is one application call's batch of entries. Undo compares the
    graph just before the change, just after it, and now, field by field. It
    puts back each field nobody has changed since, and reports what it kept and
    who changed it (ruled, §7 item 17). Entries need not carry old values,
    since replaying the journal yields the graph before any change.
  - Undo writes ordinary edits (a retitle, a retracted relation), so the
    Timeline reads an undo like any other change.
  - Previews (node images) are experience, not truth, and undo leaves them.
  - Undo again steps further back; redo re-applies the latest undo; both are
    appended under the author's name (ruled, §7 item 18). The session keeps
    each author's undo and redo order, derived from its record of changes.
  - "Restore from here" stays a separate, confirmable action on the same
    mechanism.
- **View state lives in the session**, per application and view. It holds
  hidden relations, folds, camera, focus and layout strategy (pandect's
  `ViewIntent`). It travels with forks and codicils, and the resident writes it
  for the application.
- **Lifecycle.** Mint, list, open, fork, trash and restore:
  - opening a session is the switch;
  - a fork is taken either at a journal cursor or as a component tear-out
    (Turnstone's). Either way it records its parent and the cursor it left at,
    and its journal starts at the fork point;
  - trash marks the manifest with who trashed the session and when. Its keys
    stay where they are and the live list skips it;
  - restore clears the mark. Emptying the trash is a separate, deliberate
    delete.
- **`GraphSession`.** pandect gains `GraphSession`, in `pandect::graph_session`.
  It holds one session's graph, facets, journal, view states and revision
  counter, opened and persisted over any muniment backend. Graphshell's
  `MereHost` and Cleromancy's host wrap it. Graphshell's projection stays in
  Graphshell (ruled, §7).

**Steps, in order:**
1. muniment: the per-entry journal form over any backend, then the directory
   backend with its redo file.
2. graph-kernel: per-graph recording and the structured author.
3. pandect: the session schema, `GraphSession`, and the lifecycle over a mere's
   store.
3b. graph-kernel and pandect: undo's revert engine (the field-by-field
    comparison above) and the session's undo and redo order. It follows step 3
    so the session types reach their consumers first; step 2 had carried it as
    "the inverse of a captured delta" before the undo rulings.
4. Graphshell: `MereHost` on `GraphSession`, reading its stored slot once into
   the schema. It comes before djinn's routes, because a route projects a
   session's graph with `MereHost`'s projection (ruled, §7 item 21); this was
   step 5 until 2026-09-25.
   - pandect first becomes browser-safe, so the browser build can hold a
     `GraphSession` too (ruled, §7 item 26).
   - Edits are the selected persona's, via `graphshell` or via the browser
     extension that captured them (ruled, §7 item 23).
   - The host opens the live session updated last, and mints one if there is
     none (ruled, §7 item 24).
   - The old slot is read once into the first session's baseline and left in
     place (ruled, §7 item 25).
   - A codicil import is written as ordinary edits, and opening a codicil mints
     a new session from it (ruled, §7 item 27).
   - Every stored change advances the session manifest's `updated_at` (ruled,
     §7 item 28).
5. djinn:
   - each mere on its own route, registered at startup and whenever a mere is
     ensured; graphshell's door learns to add routes and grants at runtime
     (ruled, §7 item 20);
   - each route projects the mere's sessions, for the mere view, and the
     attached session's graph, for editing (ruled, §7 item 22);
   - mint, open, fork, trash, restore, undo and redo as intents;
   - attach to a session, apply edits and view changes with the route's
     application as author;
   - ring a revision bell for every attached session.

Turnstone adopts in its own plan, and Cleromancy in its C1.

**Done when:**
- two applications, running as two processes attached through djinn, each see
  the other's edits to one session through revision bells, with no stale
  copies;
- replaying a session's journal from its baseline reproduces its graph, and
  loading the latest checkpoint plus the journal tail gives the same graph;
- the journal and view streams survive a restart, and a crash inside a batch
  leaves all of the batch or none of it;
- every journal and view entry names its author and the application it came
  through, with the application supplied by the resident;
- undo reverts the author's own latest change and leaves another author's later
  edit in place;
- a fork is an independent session whose journal starts from the fork point and
  records its parent and cursor;
- trash removes a session from the mere's live set, and restore brings it back;
- a session can be scrubbed: the graph at any journal cursor, and each view's
  state at that cursor;
- Graphshell's reference host runs on `GraphSession`.

### V2b. The mere view

One Cambium component shows a mere's sessions and their lifecycle: mint,
switch, fork and trash. It is built over pandect's reservoir and session
types. Graphshell presents it as its mere surface, and every application
embeds the same component rather than building its own. Mark: "3 by way of 2.
cambium should be the solution for all" (§7).

**Requirements from Knot (2026-09-23).** The Knot session is Knot's first
consumer:
1. It embeds as a Workbench tile behind a keyed lens, with no host-state
   access, and sizes itself to its tile, from the full centre down to a ~280 px
   side stack.
2. Activation is the host's. Activating a node hands the host an id. Mint,
   switch, fork and trash are requests the host may decline, and a refusal is
   shown truthfully.
3. A host action slot beside the graph, for New, Open and recent, since the view
   takes over the start tile's job when the last document closes.
4. Host-supplied node states (available, unavailable, open, dirty), dimmed and
   labelled in words, never by colour alone.
5. Edge provenance stays visible: extracted links, authored relations and
   suggestions are distinct, and two relations with the same endpoints never
   collapse into one.
6. Layout is a host preference (a cartography `LayoutStrategy`; Knot defaults
   to Spectral), switchable without losing node identity.
7. Every node is a native, labelled hit target reachable by keyboard, with a
   stable data-key so an automation scenario can target it.
8. Plain empty and error states, each with a host-offered action: no mere or
   catalog configured, the reservoir locked or unavailable, a reading still
   building.
9. Theming through host tokens (CSS custom properties), with no hard-coded
   colours.

**Done when:**
- Graphshell, Knot and Cleromancy each show the same component over the same
  mere;
- Knot's nine requirements above hold in Knot's embedding;
- a lifecycle action taken from any of them is recorded in the session journal
  with the application that made it;
- no application keeps a mere view of its own.

### V3. The Eidetic archive

Wire pandect's graph codicils into the reservoir: save a session as a codicil,
open a codicil as a session, fork on edit, and compose.

**Done when:**
- a session saved as a codicil reopens byte-for-byte as a new session;
- browsing a codicil is read-only;
- editing a thawed codicil produces a fork;
- composing two codicils records both in the lineage.

### V4. Access and ambiance grants

Default-on application-to-route grants for the identity's first-party
applications, with explicit, recorded denials as the exception. Add a separate
per-mere, per-app grant for ambient crossing.

**Done when:**
- any admitted first-party application opens any mere with no prior grant;
- a recorded denial refuses the route before any product endpoint opens;
- material from mere X appears in application Y's ambient tier only after X is
  opted in for Y, while Y can still open X explicitly.

### V5. Embedded resident

A standalone application embeds the resident library when no daemon is running.

**Done when:**
- a standalone application opens its mere with djinn absent;
- with djinn running, the same application attaches as a client and never opens
  the reservoir itself;
- an attempt to own the reservoir from both fails clearly.

### Tracked, not scheduled

- Sync, branch and merge of meres across the identity's devices, over
  Stickleback personal sync and codicil lineage. Mark: "synced, branching,
  merging, and conflicts resolving going forward".
- Remote access or cloning of a mere by identifier and credentials, with clones
  travelling as codicils (ambiance design §6).
- The ambient engine registry, and a pack or mod form for engines.
- Moving existing app-private sessions into the reservoir. Each consumer plans
  its own move: Turnstone's browsing domain, Knot's notes, Woodshed.

## 5. Stop rules

- The reservoir source holds no product vocabulary. Domain meaning stays with
  the application that owns it, as "Djot … evidence meaning, merge" stays with
  Knot. When a domain's writes need validating, the owning application's
  domain authority is composed into the resident, as Knot's was in R3.
  Cleromancy's is the next (ruled 2026-09-23).
- Nothing here merges distinct network identities (resident invariant 7).
- No persistence format beyond the ruled ones. Sessions and their logs persist
  through muniment, and the directory backend lays them out as Turnstone's
  files plus one append-only file per log (ruled 2026-09-24, §7). Codicils are
  Eidetic's.
- The shared root's identity rules stay as they are; only meres move.

## 6. Verification wall

Per phase, as the done-conditions state. Each receipt names the Mere revision
it measured. Two-process receipts use real processes, not an in-memory
composition presented as two.

## 7. Decisions (ruled 2026-09-23 to 2026-09-25)

1. **Where the reservoir lives on disk.** Under the shared root, per persona:
   `<shared root>/personas/<persona>/reservoir/`.
2. **How a data domain is named.** By a stable identifier, such as
   `divination`. Letting the first creating application choose could not
   guarantee that two browsers find the same mere.
3. **Where the session lifecycle comes from.** Mark: "Why not move turnstone's
   into pandect? Then everyone adopt?" Turnstone's lifecycle moves into pandect,
   and every application adopts it.
4. **Validating a domain's writes.** The owning application's domain authority
   is composed into the resident (§5).
5. **The mere view.** Asked where a shared mere view lives and who builds it,
   Mark answered "3 by way of 2. cambium should be the solution for all". So:
   - Graphshell's surface is the view that applications embed;
   - it is a Cambium component, built in this plan beside V2 (V2b);
   - it serves every application.

V2's rulings. Items 6 to 8 were ruled on 2026-09-23 and the rest on
2026-09-24. Where Mark picked an offered option, the option is named:

6. **How a mere's sessions are stored.** "Turnstone's layout", over one redb
   per mere: per-session files that stay hand-inspectable, so Turnstone's
   adoption is a path change rather than a format migration.
7. **How much the log records.** "Full Alembic log": graph mutations plus view
   and projection changes, replayable for the Timeline and undo. The
   alternatives were the graph journal with checkpoints alone, or the journal
   kept in memory. View changes stay a parallel stream, per Alembic decision
   #5.
8. **What moves into pandect from `MereHost`.** "Graph + persistence core": the
   graph, its facets, open and persist over a muniment backend, and the revision
   counter. Graphshell's projection stays in Graphshell.
9. **How the journal is written.** "Append-only file": one attributed entry per
   line of `journal.jsonl`, appended as recorded, with the writer in muniment
   beside `Journal`. This relaxes §5's rule on new formats by this one file
   form.
10. **How an entry names its author.** "Structured record": kind (person,
    rule, script or engine), id, version, and the application it came through.
    The resident supplies the application.
11. **What undo does when two applications edit one session.** "Own last
    change": undo appends the inverse of the undoing author's own latest
    change, under their name.
12. **Where an application's view state lives.** "In the session", per
    application and view, travelling with forks and codicils.
13. **How the core persists.** Asked to choose between a two-store port, a
    directory backend that left the journal outside muniment, and IO in each
    host, Mark answered "There is no unification to be had?". There is:
    - one session schema over muniment's `Backend`, with the journal one entry
      per key, as stickleback already stores its logs;
    - a directory backend that lays those keys out as Turnstone's files and
      appends each log to one file.

    He chose "Directory backend" over one redb per mere, which would have
    reversed item 6.
14. **Trash.** "Manifest flag": trash marks the manifest, and the session's
    keys stay in place.
15. **The core's name.** `GraphSession`, pairing with `GraphSessionManifest`.
16. **Checkpoints.** "Detach + every 1,000": when the last application
    detaches from a session, and every 1,000 journal entries. The count is a
    setting.
17. **Undo against later edits** (2026-09-24). Asked what undo does when another
    author has since edited part of what the change touched, Mark chose "Undo
    the rest, say so": revert every part nobody touched since, leave the parts
    another author changed, and report what was kept and why. The alternative
    refused the whole undo.
18. **Repeated undo** (2026-09-24). "Steps back twice": each undo reverts the
    author's next-older change still in effect, and redo re-applies the most
    recent undo, both appended under the author's name. The alternative made a
    second undo revert the first.
19. **Exact replay** (2026-09-25). Asked how to make replay reproduce what
    an edit reads from the clock or mints, Mark chose "Journal what was
    minted": a touch after each new node carries its visit stamp, and the
    exact edge follows each minted statement id, using existing entry kinds.
    The alternatives were to extend the add-node and assert entries, or to
    have undo and the Timeline ignore those values.
20. **How an application reaches a mere** (2026-09-25). "A route per mere":
    each mere gets its own route, registered at startup and when a mere is
    ensured, and djinn's door learns to add routes and grants at runtime. This
    keeps V4's denial at the route. The alternative was one route with an
    attach-by-id intent.
21. **Step order** (2026-09-25). "Step 5, then 4": `MereHost` moves onto
    `GraphSession` first, so djinn serves `MereHost`'s projection instead of a
    projection of its own. The steps are renumbered to match.
22. **What a mere's route projects** (2026-09-25). "Sessions and graph": the
    mere's sessions, for the V2b mere view, and the attached session's graph,
    for editing. The alternative projected the graph only and left the session
    list on the reservoir route.
23. **Who Graphshell's edits are by** (2026-09-25). "Person via the channel":
    the selected persona as a person, via `graphshell` for intents and fixture
    edits, and via `browser.extension.<source>` for captured visits, so a
    capture reads as coming through the browser. The alternative named
    `graphshell` for everything.
24. **Which session the reference host opens** (2026-09-25). "Most recently
    updated": the live session whose manifest changed last, minting one if
    there are none, with no new record to keep. The alternative kept a
    current-session record, as Turnstone's `record_current_session` does.
25. **The old slot** (2026-09-25). "Leave it in place": once
    `graphshell/mere-host/v1` has been read into a new session's baseline it is
    never read again, but no code path deletes it. The alternative removed it
    in the batch that writes the new session.
26. **How the browser build reaches the session core** (2026-09-25). Step 4
    found that pandect could not build for the browser (§2, step 4 findings).
    Mark chose "Fix pandect":
    - nonces and secrets come from getrandom 0.3 directly, as Graphshell's
      join draws them;
    - the wallet's CBOR calls `ciborium` directly, the same two calls
      p2panda-core's helpers made, so the bytes are unchanged;
    - the manifest reads the kernel's `web_time` clock.

    The alternatives were to turn on getrandom 0.2's browser support (a third
    getrandom major in the browser build, which Graphshell avoids on purpose),
    to split the session core into a crate below pandect, or to keep the
    browser on the single slot.
27. **Codicils in a session** (2026-09-25). Graphshell swapped in a whole new
    graph for a codicil import (H6 transfer) and for "Open codicil" in the
    browser, which a journal cannot record. Mark chose "Import as edits": an
    import writes ordinary journaled edits, the new nodes rebuilt with the
    machinery undo uses, so it is one undoable change on the Timeline. Opening
    a codicil mints a new session from its graph and switches to it, leaving
    the previous session in the mere. The alternatives were a journal entry
    carrying the whole resulting graph, or a new session for both.
28. **What updates a session** (2026-09-25). Ruling 24 opens the live session
    updated last, but a manifest changed only at mint, trash and restore. Mark
    chose "Any stored change": every flush that stores new changes advances
    the manifest's `updated_at` and rewrites it in the same batch, as the
    manifest's own `touch` describes. The alternative counted lifecycle steps
    only.
29. **Proving step 4 in a browser** (2026-09-25). `graphshell-web` did not
    build here: its restated genet and netrender pins lagged the workspace's
    (§8). Mark chose "Update graphshell-web": move its pins to the
    workspace's, fix what the move changes, and record, in real Chromium, the
    seeded session reloading from IndexedDB. The alternatives were to defer
    the proof to whoever moves genet next, or to rebuild the machine-local
    path patch.

## 8. Progress

- 2026-09-23: Plan written from the ambiance rulings, the device resident plan,
  pandect, the graph journal and the Turnstone and Knot precedents. No code.
- 2026-09-23: §7 ruled the same day: reservoir under the shared root per
  persona, stable domain identifiers, Turnstone's lifecycle moved into pandect
  for every application, and domain authorities composed into the resident.
- 2026-09-23: V1's first piece landed in `fe5adc1a`: `pandect::reservoir`,
  containing
  - `DomainId`, `MereId` (a UUIDv5 of persona and domain), `MereRecord` and
    `ReservoirStore`;
  - `open_reservoir_backend`, over a redb file whose exclusive lock refuses a
    second owner.
  All 8 tests pass, the lock file is unchanged, and pandect builds for
  wasm32-wasip2. The resident composition and routes are next. The mere view
  was ruled the same day: a Cambium component built here as V2b, used by every
  application.
- 2026-09-23: the rest of V1 was built on branch `reservoir-v1`, in the
  worktree `worktrees/mere-reservoir`. A peer session's crate consolidation
  had left the shared checkout's `Cargo.toml` needing a lock update, so a
  `--locked` build could not pass there.
  - `pandect::wallet_store::resolve_persona` resolves the wallet persona by the
    2026-08-09 rule: an explicit persona, else the sole wallet; zero or several
    are refused with what exists.
  - djinn's `resident_reservoir` holds the reservoir and serves the
    `reservoir` route: a reservoir card and one card per mere, plus the
    `mere.reservoir.ensure` intent. It refuses a bad domain or a stale
    revision without changing anything.
  - `OwnerSettings.reservoir` is on when absent, as ruled; the owner can turn
    it off. A reservoir that cannot open leaves the lane unavailable with its
    reason while the other lanes run. `DjinnResident::open_reservoir` names the
    shared root, so tests never touch the owner's real one.
  - The binary grants the route to `turnstone` and `knot-editor`, and the
    ordered shutdown releases the lock.
  Done-conditions met:
  - the resident lists meres;
  - an existing domain returns its mere;
  - the reservoir reopens after its owner lets go;
  - a second owner is refused on a real redb file;
  - admitted sessions list and ensure meres through the catalog without
    touching the reservoir's files.
  Still owed:
  - a two-process receipt with real processes, since the second-owner check
    ran in one process;
  - opening a mere's own route by id, which belongs with V2, once a mere has
    sessions.
  Verified: pandect 284 of 284, djinn 78 of 78 library tests, and djinn's
  integration suites. After rebasing onto `1e1672bf`, the library suites and an
  all-targets djinn check were re-run and pass.
- 2026-09-23: V1's two-process receipt landed:
  `ports/djinn/tests/reservoir_two_process.rs`. The test re-runs its own
  binary as a separate OS process.
  - While the parent holds the reservoir, the child is refused at once: "could
    not open the reservoir at …/reservoir.redb: backend: Database already open.
    Cannot acquire lock." The whole run takes about 0.4 s.
  - After the parent lets go, a second child opens the reservoir and ensures
    `divination` through the admitted route, and the parent's next open finds
    it.
  V1 is complete. Opening a mere's own route by id moves to V2, with sessions.
- 2026-09-24: V1 rebased onto `c1e7ad7f` and landed on main.
  - Main had renamed `mere_resident` to `distillery::lifecycle`. weave's merge
    spliced the file header into `ports/djinn/src/resident.rs`'s imports, and
    the imports were repaired inside the commit that introduced them.
  - Re-verified: pandect 284 of 284, djinn 78 of 78 library tests plus every
    integration suite (the two-process receipt included), and
    `cargo check -p djinn --all-targets --locked`.
  - Main fast-forwarded to `0d1a83fb`. A peer session's push of
    `c1e7ad7f..5364dfa0` carried V1 to origin with its hashes unchanged.
- 2026-09-24: V2 assessed and ruled. The findings are in §2, the rulings are
  §7 items 6 to 16, and V2's phase text above is rewritten to match. V2
  continues on branch `reservoir-v2` in `worktrees/mere-reservoir`, from
  `bb709523`. No V2 code yet.
- 2026-09-24: V2 step 1 landed in muniment (`95bd3aca`).
  - `Journal::append_entries`, `entry_writes` and `load_entries` keep one
    entry per key, with its causes. `entry_writes` hands a caller the writes to
    commit beside a checkpoint. `Journal::starting_from` begins a fork empty,
    with its provenance pointing at the parent's cursor.
  - `DirectoryBackend` (feature `directory`) keeps one file per key, and one
    line per entry for keys under a `.jsonl` file. It refuses a hole in a log,
    lands batches whole through a checksummed redo file replayed on open,
    drops a torn last line, and refuses a second owner through an exclusive
    lock. The shared custody tests run against it.
  - Measured on the Windows development laptop, release build, entries of
    about 180 bytes: 100,000 entries written in batches of 1,000 take about
    0.95 s; one live append takes 1.5 to 4.6 ms, mostly the flush to disk; the
    17.5 MB journal reloads in 0.7 to 1.0 s.
  - Found on the way: a single file's multi-line append is not atomic on its
    own, since a crash can stop it after some lines. It goes through the redo
    file like a multi-file batch.
  - muniment: 73 of 73 tests with the directory and redb features. Clippy is
    clean for the crate, and it still builds for `wasm32-unknown-unknown`.
- 2026-09-24: V2 step 2 landed in the graph kernel (`c39d67a9`).
  - `Graph::set_recorder` gives a graph its own recorder; capture sites call
    `graph.record_delta`, which feeds that recorder and then the thread hook
    Turnstone still uses. A clone starts without a recorder.
  - Replay is now silent. Before, a host with the thread hook installed
    recorded every delta that `snapshot_at` or a catch-up replayed back into
    its live journal. The new test proves both instruments see a live edit in
    the same run.
  - `AttributedDelta.author` is an `Author`: kind, id, version, and the
    application it came through. `GraphJournal::starting_from` begins a fork
    empty at its parent's cursor. `migrate_bare_log` was removed, since nothing
    ever persisted a bare log.
  - A timing-bound kernel test, `capture_hook_receives_replayable_apply_events`,
    failed whenever it ran inside one millisecond (node creation stamps a visit
    time, and a touch records only on a new millisecond). It now waits 2 ms.
  - Turnstone pins an older mere. When it repins, it adapts to `Author`.
  - mere-kernel: 292 of 292, repeated; it builds for `wasm32-unknown-unknown`,
    and pictograph checks.
- 2026-09-24: undo ruled (§7 items 17 and 18). Undo's revert engine moves to
  step 3b, after the session types.
- 2026-09-24: V2 step 3 landed in pandect (`7bde1436`).
  - `pandect::graph_session` keeps a mere's sessions under `sessions/<id>/`:
    - `manifest.json` and `baseline.json`;
    - the latest checkpoint as `graph.json`, `facets.json` and
      `checkpoint.json`;
    - `journal.jsonl` and `changes.jsonl`, one entry per key;
    - per application view, `views/<app>/<view>.json` with its `.jsonl`
      stream.
  - `GraphSession` opens from the checkpoint plus the journal tail, or from
    the baseline plus the whole journal. It records its own graph's deltas
    and journals each `apply` as one change under its author, persisted with
    the change record in one batch. It checkpoints every 1,000 entries and on
    `checkpoint_if_behind`. It answers `graph_at` and `view_at` for the
    Timeline, keeps view changes in their own streams, and trashes or restores
    by marking its manifest.
  - `MereSessions` lists, mints, opens, forks at a cursor and forks a
    component. `fork_component_graph` is the product-neutral half of
    Turnstone's tear-out, native only as the kernel's component copy is. Each
    new session's first change says how it began.
  - `GraphSessionManifest` gains `forked_at` and `trashed` (who and when),
    both defaulted.
  - `pandect::reservoir` places each mere's store at
    `<reservoir>/meres/<mere id>/`, where the directory backend's lock refuses
    a second owner too.
  - pandect: 294 of 294 tests, ten of them new. It builds for
    `wasm32-wasip2`, and djinn checks.
- 2026-09-25: steps 1 to 3 rebased onto `b38afd96`, re-verified and pushed
  with Mark's approval: origin moved `b38afd96..d0cc3fba`. A peer session's full
  workspace gate (`check --workspace --all-targets --locked`) was green on
  that tree.
- 2026-09-25: step 3b, undo, landed.
  - The kernel's revert engine (`08e56181`): `revert_change` compares the graph
    before a change, after it and now, part by part. It puts back each part
    nobody touched since and names the rest as kept `Part`s; reverting an undo
    is the redo. Two new edits make it exact:
    - `ReplaySetEdgesByIds` sets every relation between two nodes from their
      persisted form, through `restore_persisted_edge` and `persisted_edge`,
      extracted from the snapshot code;
    - `ReplayRemoveFieldById` removes a field outright.
  - Exact replay and session undo (`ea8059b9`):
    - the kernel journals each new node's visit stamp and, after a minted
      statement id, the exact edge (ruled, §7 item 19);
    - `GraphSession::undo` and `redo` rebuild each author's order from the
      change log, apply the revert as an `Undo` or `Redo` change (recorded
      even when every part was kept), and name who changed each kept part;
    - a session's checkpoint load now replaces the facet store rather than
      overlaying it.
  - Tests:
    - a kernel test proves replay reproduces visit stamps and statement ids,
      and fails when either capture is disabled;
    - the session test compares whole graphs, facets included, for checkpoint
      plus tail and for baseline plus the whole journal;
    - three session tests cover undo and redo order per author, a kept part
      attributed to the author who changed it, and the order surviving a
      reopen;
    - mere-kernel 300 of 300 and pandect 298 of 298.
  - One kernel run straight after restoring the disabled captures failed one
    test; eleven runs since have passed, and the failure did not recur.
- 2026-09-25: step 4 found that pandect could not reach Graphshell's browser
  build (§2, step 4 findings), and Mark ruled "Fix pandect" (§7 item 26).
  Rulings 23 to 25 record his earlier answers for the host. pandect's session
  core is now browser-safe:
  - its three nonce and secret draws take getrandom 0.3 directly;
  - the wallet's CBOR calls `ciborium` directly, and pandect no longer depends
    on p2panda-core;
  - the kernel exposes its `web_time` clock as `time::wall_clock_now`, and the
    manifest reads it.

  Verified:
  - pandect and mere-kernel check for `wasm32-unknown-unknown` with the
    `wasm_js` getrandom backend, and pandect for `wasm32-wasip2`;
  - getrandom 0.2 has left pandect's browser graph, where it was before;
  - mere-kernel 301 of 301 and pandect 298 of 298;
  - pandect's six workspace dependents check;
  - clippy finds no new warning in pandect, which carries 48 older ones.

  Not yet verified: Graphshell's browser build with pandect in its cone, which
  comes with the host change.
- 2026-09-25: the kernel writes an import as edits (§7 item 27):
  `import_edits(live, incoming)` in `graph/merge.rs`.
  - A node the live graph lacks arrives through undo's node recreation, with
    its images.
  - A node both hold keeps its fields and takes the incoming facets.
  - The incoming relations join the live ones between the same two nodes,
    worked out on a scratch pair as a snapshot load restores them.

  Tests:
  - on a fixture with shared, new and linked nodes, an image, and three kinds
    of relation, the edits give the graph the old rebuild gave, apart from the
    rebuild's empty facets for legacy columns, which the test checks are all
    empty;
  - dropping the images, or the joined relations, fails the comparison;
  - importing what a graph already holds writes nothing;
  - mere-kernel 303 of 303.

  No codicil carries legacy column data: canonical saves have written the
  columns empty since 2026-07-27 (`62a3aff6`), and the first engram schema
  arrived on 2026-07-28 (`10d1b29b`). An import therefore loads a codicil's
  facet store whole, as a session loads its own.
- 2026-09-25: pandect's session core now writes the way a host needs:
  - `GraphSession::new` begins a session in memory, and its first flush
    stores the manifest and baseline with everything since;
  - `edit_now` and `apply_now` journal an edit synchronously, stored at the
    next flush;
  - `pending(at)` and `stored(pending)` let a host write through a batch of
    its own, and the session's cursors move only once that batch commits.
    `flush(at)` does both;
  - every store with new changes stamps the manifest's `updated_at` (§7 item
    28), and `MereSessions::latest_live` picks the live session changed last
    (§7 item 24);
  - a view change writes what the session has pending in the same batch, so
    no stored view names an unstored cursor;
  - a checkpoint lands in the batch whose entries reach the interval.

  Three new session tests: a session begun in memory writes nothing until its
  first flush, then reopens as the same whole graph; a batch that never
  commits stays pending, and the retry carries it; the latest live session
  follows stored edits and skips the trash. pandect 301 of 301; it still
  builds for the browser and `wasm32-wasip2`, its dependents check, and clippy
  finds nothing new.
- 2026-09-25: Graphshell's `MereHost` runs on a `GraphSession` (§7 items 23 to
  27).
  - The host's truth is one session in the store it is given, and `open`
    takes the live session changed last. A store from before sessions has its
    `graphshell/mere-host/v1` slot read into a first session's baseline,
    stored at once, and left in place. The slot is read only while the store
    holds no session at all, so a store whose sessions are all trashed begins
    an empty one.
  - Every edit is the selected persona's: via `graphshell` for intents,
    product edits, imports and the fixture, and via
    `browser.extension.<source>` for a captured visit.
  - Edits are journaled as they happen and stored by `persist`. Capture puts
    them into its own batch and marks them stored only once that batch
    commits, so a failed delivery leaves them pending for the retry.
  - An import is one change of `import_edits`. "Open codicil" begins a new
    session and keeps the old one until its last changes are stored.
  - Each open advances the projection epoch, kept in
    `graphshell/projection-epoch/v1`, so an intent observed before a restart
    reads as stale.
  - pandect joins Graphshell's portable dependencies and its `web` feature.

  Tests:
  - graphshell's lib tests with `personal-sync`, which carries the `web` cone:
    295 of 296. The failure, `distillery_w1`'s receipt comparison, is a
    checkout artifact: `core.autocrlf` turns the committed LF receipt into
    CRLF, and the test compares bytes.
  - The H1 test's byte-equivalence claim is restated for sessions: the
    reopened graph and facets encode to the same bytes as the live ones, and
    reopening leaves nothing to store.
  - New assertions: the old slot is read once, stored at once and left in
    place, and is not read again after it is rewritten or after every session
    is trashed; each open is a new epoch; the fixture's edits are the
    persona's via `graphshell`, and a captured visit's via the extension; an
    import is one change; opening a codicil begins a new session.
  - The capture batch test still passes: a rejected batch stores no session,
    and the retry stores the fixture's session with the visit.
  - Graphshell checks for `wasm32-unknown-unknown` with `web` and
    `webrtc-browser`, pandect in its tree and no getrandom 0.2.

  Not verified: the browser runtime. `graphshell-web` does not build in this
  environment, with or without this change. Its restated genet and netrender
  pins lag the workspace's, so a fresh resolution holds two revisions of
  each, reached through `cambium` and `mere`'s `pictograph`, and doubles
  `ScriptedDom` and `Scene`. The machine-local path patch that reconciled
  them points at a `worktrees/genet-head` that no longer exists. Inside that
  graph Graphshell, pandect included, compiles; all seven errors are in
  `graphshell-web`'s own files.

  Unchanged: the browser stores at open and at capture, as before, so an
  intent between them stays unsaved until the next capture batch.
- 2026-09-25: step 4 rebased onto origin's main, which had repinned genet to
  `5621ca05768`, re-verified with `--locked`, and pushed with Mark's approval.
  The gate ran on `6a206411`; main then moved one commit (`e8f83f89`, three
  cambium files), and `check --workspace --all-targets --locked` passed again
  on it. The gate, all `--locked`: `check --workspace --all-targets`; mere-kernel 303 of 303; pandect 301 of 301; graphshell's lib tests with `personal-sync` 295 of 296, the one failure being `distillery_w1`'s CRLF artifact; graphshell's every target with all features; pandect, the kernel and Graphshell's browser cone for `wasm32-unknown-unknown`; pandect for `wasm32-wasip2`.
