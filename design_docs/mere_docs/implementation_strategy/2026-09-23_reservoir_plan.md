# Reservoir plan: shared meres held by the device resident

**Date:** 2026-09-23
**Status:** in progress. V1's reservoir index landed in `fe5adc1a`; the
resident composition and routes are next. §7's decisions were ruled on
2026-09-23.
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

### V2. Sessions with the full lifecycle

Move Turnstone's session lifecycle (mint, switch, fork, trash) into pandect as
product-neutral machinery over its `ManifestStore`, and have every application
adopt it (ruled, §7). Turnstone keeps only its own dressing, such as windows,
lens spaces and participant runtime, as hooks. It adopts the pandect lifecycle
and deletes its copy in its own plan. The reservoir source runs the lifecycle
for every mere. Install the graph journal per session, so every move is
recorded with its author.

**Done when:**
- two applications attached to one session each see the other's edits through
  revision bells, with no stale copies;
- replaying a session's journal reproduces its graph;
- a fork is an independent session whose journal starts from the fork point;
- trash removes a session from the mere's manifest set;
- every journal entry names the person, rule, script or engine that made it.

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
- No new persistence format: sessions, the journal and codicils are the
  existing pandect, muniment and Eidetic forms.
- The shared root's identity rules stay as they are; only meres move.

## 6. Verification wall

Per phase, as the done-conditions state. Each receipt names the Mere revision
it measured. Two-process receipts use real processes, not an in-memory
composition presented as two.

## 7. Decisions (ruled 2026-09-23)

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
