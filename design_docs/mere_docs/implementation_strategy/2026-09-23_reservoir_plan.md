# Reservoir plan: shared meres held by the device resident

**Date:** 2026-09-23
**Status:** in progress. V1 is complete and on main: the pandect index,
wallet-persona resolution, djinn's reservoir lane and route, and a real
two-process receipt. It reached origin with `5364dfa0` on 2026-09-24. V2's
shape was ruled on 2026-09-23 and 2026-09-24 (§7). Step 1, in muniment,
landed on 2026-09-24; step 2, in graph-kernel, is next.
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
- **Undo.** Undo reverts the undoing author's own latest change by appending
  its inverse under their name.
  - Other authors' later edits stay, and nothing is truncated.
  - The Timeline shows the undo.
  - The inverse is derived by replaying the journal to just before the change,
    so entries need not carry old values.
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
2. graph-kernel: per-graph recording, the structured author, and the inverse of
   a captured delta against the graph it was applied to.
3. pandect: the session schema, `GraphSession`, and the lifecycle over a mere's
   store.
4. djinn:
   - open a mere by id on its own route;
   - list its sessions, with mint, open, fork, trash and restore as intents;
   - attach to a session;
   - apply edits and view changes with the route's application as author;
   - ring a revision bell for every attached session.
5. Graphshell: `MereHost` on `GraphSession`, reading its stored slot once into
   the schema.

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

## 7. Decisions (ruled 2026-09-23 and 2026-09-24)

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
- 2026-09-24: V2 step 1 landed in muniment (`80540b98`).
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
