# Reservoir: shared meres held by the device resident

**Status:** 2026-10-28: cut from `2026-09-23_reservoir_plan.md`, archived in `archive_docs/2026-10-28/`. V1 and the native session contract of V2 are landed; V2b steps 1 to 3 are landed; V3 (archive), V4 (access and ambient route enforcement) and V5 (client-first embedded ownership) have integrated resident implementations qualified by route and real-process fixtures. Open: V2's headed browser scenario verdicts, V2b steps 4 and 5, the consumer gates (archive controls, trusted owner controls, daemon and embedded registration with actual startup), product ambient presentation, installed-daemon qualification, the Cleromancy C1 browser wall and the personal-facts domains.

**For:** giving each data domain one mere and making every mere of an identity openable by any of that identity's applications, held by the device resident as sessions with a graph journal and an Eidetic archive. Opened by Mark on 2026-09-23: "each data domain deserves its own mere. that isn't the same as every app getting its own mere, like two browsers should share a mere if you ask me" and "all apps should be able to access any of an identity's meres by default (at least exceptions should have a really good reason)". Asked where Cleromancy's mere should live, he chose "Mere change first".
Not here: V2b step 4, Graphshell's one-tree cutover, and the Graphshell-side page migration: `2026-10-28_graphshell_host_plan.md`; motion, channel and replay: `2026-10-28_dynamics_grammar_plan.md`; shared editing contracts: `2026-10-28_scenograph_editor_plan.md`; Cleromancy's own consumer plan (`repos/cleromancy/design_docs/2026-09-23_divination_mere_plan.md`), which owns the final C1 verdict and the later C4 card ambient presentation; sync, branch and merge of meres across the identity's devices, which rides Stickleback personal sync and codicil lineage (tracked, not scheduled); each consumer's own move of app-private sessions (Turnstone's browsing domain, Knot's notes, Woodshed), which its own plan carries.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **Shape.** One owner: the reservoir source owns every mere's stores and serializes authoritative mutations per mere, and each attached application gets its own session adapter with its own caches and notice cursor. Any admitted first-party application of the identity may open any mere by default; an exception is an explicit, recorded denial, and explicit denial wins over default and ambient grants. Ambient material crossing into another application's ambient level needs a separate opt-in per mere and per application. No daemon is required: a standalone application embeds the resident library when it owns the whole process, and with djinn running it is a client and never opens the stores itself. App-private settings and caches stay under each application's root. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:337-368`.
- **Disk and naming (rulings 1, 2, 30).** The reservoir lives under `<shared root>/personas/<persona>/reservoir/`; a data domain has a stable identifier such as `divination` and exactly one mere; its route is `mere/<domain>`, one route per mere (ruling 20). From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:727-736,800-806,859-862`.
- **Sessions and the log (rulings 3, 6 to 16).** Turnstone's lifecycle lives in pandect and every application adopts it; sessions persist in Turnstone's layout through muniment's directory backend, one entry per key, the journal as an append-only file; the full Alembic log records graph mutations plus view and projection changes as a parallel stream, never merged into the journal; an entry names a structured author (kind, id, version, application supplied by the resident); view state lives in the session; trash is a manifest flag; checkpoints are written on last detach and every 1,000 entries, the count a setting to be tuned by measured replay cost. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:426-430,731-784`.
- **Undo, replay and edits (rulings 11, 17 to 19, 31 to 33, 44 to 48).** Undo reverts the undoing author's own latest change, undoes the parts nobody touched since and reports what was kept, and steps back twice on repeat; replay journals what was minted so exact replay needs no clock; edits name nodes by stable id and are accepted at any revision in arrival order; lifecycle steps name the session by id and land only if the card offers them as it now stands; the replay check is clock-free through `Graph::to_snapshot_at`, and visit stamps and every other field keep their exact comparison. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:764-767,785-799,863-890,958-1023`.
- **Graphshell's mere surface (rulings 5, 34 to 43).** The mere view is a Cambium component in `crates/cambium/mere-view`, host-fed with one route adapter in `graphshell::mere_route` (behind the `mere-route` feature); Graphshell presents it beside the canvas, which stays its editing surface; a switch is not recorded; a relation shows its kernel family as its kind until a mere holds Knot's catalog. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:545-650,737-745,891-957`.
- **Stop rules.** The reservoir source holds no product vocabulary; a domain's writes are validated by the owning application's domain authority composed into the resident (Cleromancy's is the next). Nothing here merges distinct network identities. No persistence format beyond the ruled ones; the shared root's identity rules stay as they are. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:705-718`.
- **No bypass.** No application bypasses its domain validator, invents a second durable owner or calls a local snapshot reopen reservoir adoption. An admitted application label grants route access, not permission-management authority; `MereRoutes::set_access` is trusted owner configuration. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:155-179`.
- **Verification wall.** Each receipt names the Mere revision it measured; two-process receipts use real processes, not an in-memory composition presented as two. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:719-724`.
- **Dependency boundaries.** V3 to V5 do not wait for retirement of Graphshell's old page; when one is taken up it names its actual session, route, identity or owner dependency and the proof it adds. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:147-154`.

## Phases

### V2r. Sessions: the headed browser verdicts and browser persistence
V2's native and session contract is landed. The recorded browser run did not give a verdict on its three scenarios because the Browser pane was hidden and the frame pump never ticked; the verdicts wait for Mark's next visit with the pane open. The browser stores at open and at capture only, so an intent between them stays unsaved until the next capture batch, and no reservoir-backed browser persistence exists yet.
- Done when: the three headed scenario verdicts are recorded with the Browser pane visible.
- Needs: Mark's visit with the Browser pane open.
- Rulings: 26, 29
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:9,119-131,1359-1360,1385-1390`

### V2b-5. Graphshell's mere panel
The panel is the component in Graphshell's tree, fed from its `MereHost`: switching sessions reopens the canvas on the chosen one, and the panel can hide its own graph where the canvas shows it. In the browser it must mount the sessions, graph and archive projections from the same admitted resident route, synchronize its canvas and perform the advertised owner intents; a pacing verdict is part of the gate. A new native witness or retiring dead source does not establish the panel.
- Done when: the Browser pane's `read_page` lists the panel's sessions, steps and node targets; a step taken from the panel that changes the mere is recorded with `graphshell`; a headed Chromium capture shows the panel beside the canvas.
- Needs: V2b step 4 (`2026-10-28_graphshell_host_plan.md`, T4); V3 to V5 contracts.
- Rulings: 5, 38 to 41
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:132,631-637,1078-1080`; `archive_docs/2026-10-28/2026-09-25_graphshell_one_tree_plan.md:2250-2257`

### V2b-c. Consumers embed the view
No application keeps a mere view of its own. Knot's step 8 and Cleromancy's C1 embed the component and record the embedding against V2b's conditions; Turnstone adopts in its own plan and, pinning an older mere, adapts to `Author` when it repins.
- Done when: the consumer plans record the embedding against V2b's conditions.
- Needs: V2b-5.
- Rulings: 34
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:7-9,524,640-650,1186`

### V3. The Eidetic archive: consumer actions
Save a session as a codicil, open a codicil as a session, fork on edit, compose; the resident contract is qualified by route and real-process fixtures. Consumer UI must expose these actions, and installed or product acceptance is separate. The expanded personal-sync reopen test hung for over two minutes in the V3 lane and its cause is unresolved, so the expanded Graphshell gate is not complete.
- Done when: a session saved as a codicil reopens byte-for-byte as a new session; browsing a codicil is read-only; editing a thawed codicil produces a fork; composing two codicils records both in the lineage; a consumer UI exposes the actions.
- Needs: a consumer UI that exposes the actions.
- Rulings: 27
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:119-120,133,651-664,1057-1060`

### V4. Access and ambiance grants: owner controls and ambient presentation
Route enforcement is qualified. Trusted owner controls and actual opted-in material in a product's ambient tier remain consumer work.
- Done when: any admitted first-party application opens any mere with no prior grant; a recorded denial refuses the route before any product endpoint opens; material from mere X appears in application Y's ambient tier only after X is opted in for Y, while Y can still open X explicitly.
- Needs: a consumer with an ambient tier.
- Rulings: 20
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:134,665-679`

### V5. Embedded resident: consumer registration and installed startup
Library process fixtures qualify ownership, attachment and refusal. Each consumer must register the same domain in daemon and embedded composition and qualify its actual startup; installed-daemon and product qualification remain.
- Done when: a standalone application opens its mere with djinn absent; with djinn running, the same application attaches as a client and never opens the reservoir itself; an attempt to own the reservoir from both fails clearly.
- Needs: each consumer's registration; an installed Djinn.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:7-9,135,680-693`

### C1. Cleromancy coordinated cutover
Cleromancy attaches first and embeds on genuine endpoint absence, opens no reservoir files or private snapshot store, and registers its product-owned validator in daemon and embedded composition; bounded native and mounted-witness proofs pass. Open: cross-application renewal; the complete V2b and browser wall, which is the full 164-case canonical wall (21 closed, 143 open) plus the mandatory parameter, external, accessibility, renderer and negative supplements; native publication; the graphical unlock dialog (the first real graphical attempt failed, so graphical handling is unqualified); and the first cold original epoch 8 to 10 byte failure with the release and startup database changes, recorded separately. Cleromancy owns the final verdict; V2b steps 4 and 5 and full one-tree UI qualification require the complete wall.
- Done when: the complete canonical wall and supplements pass under Cleromancy's verdict; native publication is done; V1 to V5 hold for the consumer.
- Needs: V1 to V5; V2b-5; `2026-10-28_graphshell_host_plan.md` T4.
- Rulings: 4 (validator composed into the resident)
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:136,174-178,1694-1763`

### PF. Shared personal facts domains
The `personal-facts` domain consumes Dramatis plain birth and association contracts and scoped Gaz contact resources; Djinn installs its complete-candidate validator in daemon and embedded compositions, and its client uses the admitted mere session route with checked journal transactions and resource hashes, never an application-local store or custody fallback. Cleromancy native intake, product native evidence and full browser-wall qualification remain separate gates; no completion is inferred from the supplier types or the V1 to V5 fixtures.
- Done when: valid revisions and exact associations persist across owner reopening; tampering, wrong persona, stale candidate and access revocation refuse without installing durable edits; a second application reads the same admitted values; and the consumer retains an immutable authorized snapshot without moving Chatelaine autofill or credential custody.
- Needs: Dramatis subject reference plan (`dramatis/design_docs/2026-10-16_subject_reference_plan.md`); Cleromancy's plan.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:1765-1883`

## Open questions

- **Cause of the expanded personal-sync reopen test hang** in the V3 lane (over two minutes); the expanded gate is incomplete until resolved. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:119-120,1057-1060`
- **Telling extracted, authored and suggested relation kinds apart** waits until a mere holds Knot's catalog (ruling 43); moving Knot's notes into the reservoir is tracked, not scheduled. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:300-303,954-957`
- **The checkpoint interval** (1,000 entries) is a setting to be tuned by measured replay cost. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:426-427`
- **Remote access or cloning of a mere** by identifier and credentials, with clones as codicils (ambiance design section 6), and **the ambient engine registry** and a pack or mod form for engines: tracked, not scheduled. From: `archive_docs/2026-10-28/2026-09-23_reservoir_plan.md:696-701`

## Findings

- 2026-10-28: the shared root's rule is that personas and the identity wallet are family-shared and everything else is app-private; the reservoir moves only meres (`crates/system/pandect/src/shared_root.rs`).
- 2026-10-28: Eidetic's `manifest:<hash>` keys cannot use the directory backend's portable filenames, so the resident exclusively owns the Redb backend at each mere's `archive.redb`; the optional session payload extends graph codicil v2 and older payloads stay readable.
- 2026-10-28: an explicit session discloses graph and facets and only the admitted application's view; ambient routes offer graph reads only and do not mint a session in an empty mere. Complete candidate graphs are validated before installation or journal writes; rejection creates no fork.
- 2026-10-28: V1 and V2 proof is real two-process (`ports/djinn/tests/reservoir_two_process.rs`, `ports/djinn/tests/mere_two_process.rs`).

## Progress

- 2026-10-28: cut from the reservoir plan; nothing new landed.
