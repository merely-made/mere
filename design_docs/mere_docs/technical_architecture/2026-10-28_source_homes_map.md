# Source homes map: which Mere packages fit another repository

**Date:** 2026-10-28
**Status (2026-11-03):** the remaining extraction and directory scope is refreshed
below. The original findings were read at Mere `d4d5b0c32` (`cargo metadata` over the
workspace) and at the 2026-10-28 `main` of Murm, Gemot, Eidetic, ESP, Isocosm,
Knot, Turnstone, Cleromancy and Woodshed (their manifests' Mere dependencies).
The inventory below records that original survey; the dated ownership update records later moves. This is the dependency evidence that the family promotion
plan's F3 asks for before it chooses a grouping
([family promotion plan](../implementation_strategy/2026-10-28_family_promotion_plan.md)).
[What the code adds](#what-the-code-adds) comes from reading every package's
code at Mere `0740ea10a` and the other repositories' `main`. Its load-bearing
claims were checked again at `45492c644`.

**Origin.** Mark, 2026-10-28: "consider what should be moved from mere to the
various repos." Earlier that day he described Mere's scope, in conversation
rather than as a ruling, as the GUI toolkit, the cross-platform application
framework, and the interfaces, utilities and integrations for the stack's
capabilities, to be reviewed system by system. On 2026-10-27 he brought in a
grouping proposal: Djinn in Armillary's repository, and Distillery, Mesh,
Alembic and Athanor beside ESP. The review that followed agreed on three
points: the custody packages are siblings in the resident unit, Pandect takes
domain settings in rather than importing them, and Graphshell's fixture edges
move to an integration unit. Neither home was ruled.

Later on 2026-10-28 he added, again in conversation:
- "redefine existing names instead of minting new ones."
- Graphshell may become a repository again. That would mean unarchiving the old
  repository and replacing its contents, leaving its history in the past.
  "what's important is to untangle the repos and prevent cycles, dogfooding,
  and not drawing inefficient boundaries."
- On Djinn: "some of djinn's capabilities have to be distributed throughout the
  stack."
- On which apps project: "we are _not_ treating the current situation as the
  intended one". The design is that every app projects through Graphshell's
  protocol.

## The scope test

A package stays when it is one of these:
- **toolkit:** Cambium and its hosts, the scene crates, `tinct`, `illume`, and
  Tabard's theme library;
- **application framework:** Inker and its engines, `document-canvas`,
  `nematic`, document lanes, `fetch`, `page-load`, `framing`, `surface-api`,
  `workbench`, the registry, Pandect's session authority, `chrome`, `uxtree`,
  `platen`, Forme, and the script hosts;
- **the graph:** the kernel, `linked-data`, `cartography`, Pictograph, `mere`,
  `mere-view`, `mere-overmap` and `incipit`;
- **a capability contract:** Chirograph, the four `graphshell-*` crates, and
  `state-witness`.

Products under `ports/`, and engine families that Mere uses but does not define,
are the candidates to leave. The table below is the residue of that test, with
each package's coupling measured in files.

## Candidates

| Packages | Used inside Mere by | Used outside Mere by | Coupling to Mere | Home that fits | Standing |
| --- | --- | --- | --- | --- | --- |
| `station-custody` | nothing | Murm `apps/signalman-desktop` | `pandect` in 9 files, `graphshell_endpoint` in 6 | Murm, beside `signalman-desktop` and outside Murm's root workspace (Murm already takes Pandect from Mere there) | No ruling found. It is the only route by which Mere's graph resolves `retinue`, `outrider`, `radio-hand` and `tulle` (Reticulum License); without it, 20 more packages leave Mere's resolution, among them `postilion`, `seneschal`, `selvage`, `radio-face` and the serial stack. |
| `mere-comms` | `mere-moot` (2 files) | Murm `apps/signalman` (`comms` feature) | none: its dependencies are `serde`, `async-trait`, and optional `murm` and `misfin` | Murm, or Mere as a capability contract | Open. The murm runtime plan (M7) puts the Sennet, Tucket and Signalman adapters at the Comms consumer boundary, and all three live in Murm. Mark's scope sentence keeps capability contracts in Mere. |
| `distillery`, `mere-athanor`, `mere-alembic`, `mere-mesh` | Djinn and Graphshell (Distillery, Mesh); Pandect (Alembic) | Turnstone (Athanor, Distillery), Cleromancy and merelyllc.com (Alembic) | Mesh: none. Distillery uses Mere in five adapter files (2026-10-27 count). Two edges point back: Pandect imports Alembic's `EvictionPolicy` (`persona_settings_store.rs`), and Graphshell depends on Distillery for a test and the optional `distillery-chronicle-fixture` feature. | ESP's repository, as Distillery's (the 2026-10-27 proposal) | Proposal, not ruled. Family promotion F3. |
| `djinn`, `djinn-testkit`, `castellan`, `djinn-custodian`, `custody-wire`, `graphshell-mcp` | Djinn is the only user of the other five | Cleromancy (Djinn, Castellan) | Inherent: `graphshell_endpoint` in 45 Djinn files, `graphshell_host` in 23, `chirograph` in 23, `pandect` in 18 (2026-10-27) | A resident repository, or Armillary's (the 2026-10-27 proposal) | F3 and the custody plan. The custody packages move together. |
| `mere-moot` (`ports/moot`), `mere-site` | `mere-site`: Djinn | Turnstone (`moot`) | Moot: `app_host` in 1 file, `comms` in 2, `fleece` in 1. Site: none (`mien`, `muniment`, `mere-proofs`, `servitor`) | Gemot, the governed-space family | No ruling. The murm runtime plan withdrew a separate Moot repository (its Phase G) but kept its two requirements: Mere builds offline, and no manifest carries a local path. Moot in Gemot would give Gemot a Mere edge for `app-host` and `fleece`. |
| `dramatis-app` (`ports/dramatis`), `gazette`, `mere-facts` | `gazette`: Castellan; `mere-facts`: Djinn | Cleromancy (`mere-facts`) | App: `graphshell_endpoint` in 4 files, `mesquite` in 2, `cambium` and `persona_picker` in 1 each. Gazette: `chirograph`, `incipit` and `sceno` in 2 files each. Facts: `mere-kernel` in 2. | Dramatis; the app as a separate root, as Murm keeps `signalman-desktop` | No ruling. The Dramatis repository plan (DR-D, 2026-10-09) moved the identity libraries and left these three. |
| `conatus`, `modulus`, `nisus`, `tenant` | nothing | Isocosm (all four) | `conatus` uses `seiche` in 5 files and `nisus` in 2. `tenant` brings the kiss3d fork into Mere's graph. | Isocosm, or a Conatus repository. `numen` and `seiche` stay, because the kernel and Pictograph use them. | The workspace manifest says "Cambium and Conatus remain platform components" (2026-07-23 consolidation, restated at the 2026-10-19 promotion). |
| `mere-edit` (`source-edit`), `mora-cmudict`, `reference-data` | nothing | Knot | none | Knot; `mora-cmudict` could go beside `mora` instead | No ruling found. |
| `document-host`, `ux-events` | nothing | nothing in the nine repositories read | — | — | Workspace members with no consumer. |
| `support/graphshell-coop/relay_server`, `support/patches/burn-remote`, `crates/probes/murm-direct-phy` | the co-op harness; Distillery `remote` | — | — | The relay and the Burn patch: murm `design_docs/2026-10-28_transport_consolidation_plan.md`, T5 and T4. The probe: Murm. | Transport plan. |

**2026-11-03 ownership update:** the table retains its original audit basis.
Castellan, Sporran and Gazette's domain now live in Dramatis (D11-D13), and
D14 moves the custodian and custody-wire there too. The resident composition
remains in Mere. D15 also moves the reusable resident identity service into `dramatis-resident`;
Mere retains its graph-door and platform-dialog adapters. The grants template is now
`repos/dramatis/crates/djinn-custodian/packaging/djinn-grants.json.in`.
Distillery and its compute mesh now live in the renamed Distillery repository,
with the model probes, lexical wasm probe and their inference patches. Their
core uses the published producer contracts and has no Mere source dependency.
Mere composes those Git dependencies and keeps concrete graph/host adapters.
Athanor and Alembic remain held by extraction-plan D16. The earlier candidate
row and source paths below retain their original audit basis; the current
cutover and remaining work are in the Distillery extraction plan.

## Remaining extraction and directory scope, 2026-11-03

**Station Custody cutover, 2026-11-04:** its standalone home is
`murm/crates/station-custody`; Signalman's desktop consumes it by path. All
twelve Rust files retain their donor blobs. Dramatis/Personae and Djinn retain
credential and grant authority. The receiver's tests, MSRV and Signalman gates
are in `Code/testing/murm/station-custody-extraction-20261104`; the owning plan
is `murm/design_docs/2026-11-02_murm_family_move_plan.md`. Mere now has 87
workspace members. Its default native normal/build graph contains none of the
station/radio family below. The optional resident NomadNet feature still names
Retinue, so Retinue remains in the lockfile; the earlier blanket exclusivity
claim applies to the default graph, not every declared optional edge.

Mark requested this scope after the Stickleback network move. The current
tracked inventory contains 104 Rust packages, 88 in the main workspace.
`Code/testing/murm/stickleback-extraction-20261103/mere-package-scope.json`
records paths, physical Rust source lines including tests, normal/build
dependency declarations, and external package consumers across twelve sibling
repositories. These are declared edges; optional features and process entry
points still need their receiving gate. The historical candidate table above
must not substitute for this current inventory.

Stickleback 0.2 retains signed history, admission and group encryption in
Eidetic. Murm's `murm-sync` now owns its address book and live joined/synced
sessions. The same admission receiver feeds both local and network arrivals.
Domain grammar, graph projection and application authority remain with their
owners. Qualification is Murm's transport consolidation plan, T7.

| Slice | Current evidence | Next scope and done-condition |
| --- | --- | --- |
| `station-custody`, moved 2026-11-04 | Twelve unchanged Rust files now in `murm/crates/station-custody`, outside the default radio workspace. Signalman consumes that path; Mere's donor member, source files and three source-size exceptions are removed. | Murm owns this Retinue-specific custody adapter; Dramatis/Personae and Djinn retain credentials and grant policy. Scoped derivation, expiry, revocation, first-owner control, licenses and fixtures are unchanged. The standalone adapter and Signalman receiving pass; Mere's remaining workspace is checked separately. Physical qualification retains its prior scope. |
| `crates/domain/site` and Knot's `knot-site` | The portable model is already editor-free. Knot still houses filesystem handling, immutable publication, protocol servers, resident handoff and submission adapters used beyond its editor. | Generalize the shared site domain from both repositories. Preferred repository home: Gemot, as an independent library with ordinary-site serving available without Moot membership. Knot keeps authoring UI and its application integration. Preserve V1 bytes, identities, submission consent and runtime boundaries; qualify Knot, Djinn and a Turnstone consumer. See the scope refinement below. |
| Mora repository, `crates/intel/mora-cmudict`, `reference-data` | Mora's application callers are Knot's editor and Readings. CMUdict's corpus/license and the validated lexical packs accompany them. Isocosm's CMUdict row is a patch-table entry, not an application dependency or call site. | Consolidate Mora and CMUdict as separate crates with Knot's lexical providers; include Reference Data in that leaf move. Preserve the no_std engine, corpus/license hashes and portable pack schema. A future second editor can motivate broader packaging. `mere-edit` already serves Knot and Isocosm and remains Mere's format-neutral application contract. |
| Pipit repository | Actual voice-codec callers are Murm's Outrider (`voice`) and Signalman; the published package is `pipit` 0.2.0. | Recommended consolidation into Murm as a separate codec crate. Preserve frame/clip identities, malformed-input behavior, licenses and no_std/alloc profiles; qualify both consumers. Repository grouping adds no Retinue or GUI dependency to the codec. |
| Turquet repository | Cleromancy uses 0.17.0; Turnstone's optional `sky` feature uses 0.13.0 in real surface/timeline code. The embedded geometry helper also lives in Turquet. | Consolidation beside Cleromancy is a repository-grouping option, retaining Turquet as an independent astronomy library directly usable by Turnstone. This is already a shared engine, so moving it cannot fold astronomy into Cleromancy's application/core API. Qualify both pinned API cohorts or explicitly migrate them together; preserve provenance, coefficient tables and verification fixtures. |
| `modulus`, `tenant`, `conatus`, `nisus` | Modulus and Tenant have only Isocosm consumers. Seiche now optionally imports Conatus for GPU evaluation; Numen and Seiche still serve the graph toolkit. | Review Modulus/Tenant as a bounded games-wing slice. A bulk four-crate games-only move is stale: first qualify Seiche's GPU path and choose a portable engine owner for Conatus/Nisus. Preserve the host's shared device. Numen/Seiche remain platform components under this scope. |
| `ports/dramatis` | A real packaged `dramatis-ui` binary, not an unused library; 1,464 Rust lines. It uses Cambium/Desktop, Mesquite, Taproot and Persona Picker. | Candidate Dramatis application root with a standalone producer/SDK gate and preserved packaging/profile behavior. Core Dramatis must remain free of GUI dependencies. Persona Picker stays reusable GUI code; Woodshed also consumes it. |
| `ports/distillery/athanor`, `crates/distillery/alembic` | 5,222 and 738 Rust lines. The `workshop-destination` feature still imports Kernel/Pandect; graph destinations call Athanor validation and share a durable operation transaction. | D16 still applies. Separate Mere's graph-destination adapter from the graph-free workshop core, then move that core to Distillery after the W3 recovery/two-host gate. A Pandect feature must take admitted effects/injected checks rather than import agent-domain authority. Preserve operation ids, sealed contexts, transactions and recovery semantics. |

`mere-moot` and `mere-facts` contain graph/application adapters, so a Gemot or
Dramatis name alone is insufficient reason to move them. The Comms contract
is reusable; provider implementations belong with their protocol owners.
`document-host` and `ux-events` have no declared production consumer in this
census, but Document Host has its own component tests/probes and implements
Registry's runtime trait. Review integration or retirement in the scripting
and diagnostics plans before deleting or relocating either.

### Site and small-repository scope refinement, 2026-11-04

Mark's direction: serving sites is broader than Knot, and should move out of
Knot as well as Mere; Mora can live with its only expected editor consumer and
be generalized again if another editor needs it. Pipit and Turquet are also
repository-consolidation candidates. These are scopes, not completed moves.

The previous assertion that Knot owns the shared snapshot/schema is withdrawn.
Knot is its originating authoring consumer. Native-site manifests, immutable
snapshots, publication selection, serving and resident handoff are reusable
site-domain responsibilities. Turnstone must consume that domain directly,
without acquiring an editor dependency. Knot retains draft editing, authoring
controls and calls into the shared domain; editor origin does not establish
ownership of the general service.

Gemot is the recommended repository home for this domain, with its ordinary-site
library independent of the community-authority crate. Preserve the release-log
plan's D2/D6: a personal site log works without Gemot membership or its optional
Moot adapter. Repository location must not make every site a community. P2mere
supplies operation/sync/group primitives below the domain; Murm supplies carriage
and protocol adapters; Eidetic supplies retained content/history; Personae and
Dramatis supply identity/custody. Mere keeps browser/graph/host integration.
The existing site-publication and release-log plans own behavior and receiving
gates; this map refines their source-home scope rather than inventing another
publication system.

Separate library crates can share a consumer repository while retaining their
public packages, licenses, feature profiles and dependency direction. Mora and
Pipit fit that consolidation now. Turquet has a second real consumer, so a
Cleromancy home remains a grouping recommendation requiring Turnstone receiving,
not evidence that the engine belongs inside Cleromancy's application logic.
Existing releases, immutable Git pins and historical evidence stay available
through any later source move.

Directory cleanup follows these cuts:

1. Move ready owner-specific leaves first, updating workspace members, scripts,
   fixtures, notices, source pins and actual consumers together. Remove a donor
   only after the receiving source is pushed and its blobs are verified.
2. The remaining `ports/distillery/athanor` and `crates/distillery/alembic`
   directories are live held code. Remove those misleading parent groupings
   when the workshop split lands; an interim cosmetic move adds another path
   migration without settling ownership.
3. Keep standalone applications in `ports/` and reusable graph/host adapters
   in the existing `crates/` groups. Classify `ports/moot` by its library role
   before a mechanical relocation; preserve public crate and protocol names.
4. `crates/probes/murm-direct-phy` belongs with Murm's physical harnesses.
   The co-op relay belongs beside Murm's transport pin, but its frozen input
   hashes require a new harness cohort. `testing/moot-ocapn-coop` needs its
   Gemot/Murm consumer boundary recorded before relocation. Keep historical
   receipts at their cited locations and retain their original input scope.

Station Custody is the first completed source cut under this scope. The remaining rows are
reviewable scopes with named receiving gates, not completed moves or authority
to replace the existing policies.

## What the code adds

On 2026-10-28, eleven readers went through the code of the whole stack: every
package's code, not just its manifest. The table above says where packages could
go. The findings below say where the lines between homes fall, and why several
of those lines can't be drawn yet: they follow contracts that the stack doesn't
have.

### Capabilities with no contract

Without a shared contract, each app writes its own version of a capability, and
every copy is one more edge for a move to carry.

- **Persona.**
  - The word names six unrelated things:
    - Castellan's `ProfileId`, a vault master key;
    - `PersonaId`, a wallet persona's UUID and a directory name;
    - `gaz::PersonaScope`;
    - Gemot's `mien::PersonaKey`;
    - Hocket's `IdentityHome`;
    - notochord's `ProfileRef`.
  - About ten pieces of code decide which persona an app runs under. Only hand
    configuration links a `ProfileId` to a `PersonaId`, and only tests call
    `PersonaId::new()`.
  - There is no type for use that leaves no trace. The journal's `AuthorKind`
    is Person, Rule, Script or Engine (`graph-kernel/src/graph/journal.rs:66-99`),
    and `GraphSessionManifest.persona_id` is required
    (`pandect/src/manifest.rs:124`).
  - What already fits:
    - An app gets keys through `IdentityProvider`, and a writer record may
      carry a `DerivedKeyAttestation` (Eidetic `stickleback/src/writer.rs:43`).
      A signer without one is its own root and has no standing.
    - Woodshed's persona gate is the one anonymous mode in code: escape keeps
      "no key, no store, nothing kept".
  - A persona contract would let an app declare its persona use and salt
    namespace itself. It would also decide which package is the persona
    client. Today that is the `door`, described below.
- **Settings.**
  - There are seven or more stores, each with its own file layout:
    - Djinn's `OwnerSettings` and `lock.toml`;
    - Pandect's `ApplicationSettings`, `DeviceSettings` and `PersonaSettings`;
    - Knot's;
    - Distillery's `InstalledSettings`;
    - the custodian's per-user file.
  - Two `SettingScope` types define different scopes: kernel `permissions.rs`
    and `surface-api` `settings.rs`.
  - About 51 non-test files across the apps write JSON through a temp file and
    a rename. At least seven of them hand-roll it.
  - Pandect holds other owners' shapes:
    - `ApplicationSettings` is Turnstone's (`tab_cap`, `shellbar_edge`);
    - `DeviceSettings` holds Djinn's unlock mode and mesh lending;
    - `PersonaSettings` stores Alembic's `EvictionPolicy`. This is the
      Pandect→Alembic edge in the table.
  - The 08-06 ruling already fits a settings capability:
    - the configured thing owns its typed setting;
    - providers describe it, Cambium renders it and hosts apply it;
    - mechanism may be shared, schema may not.

    What's missing is the shared mechanism and one scope vocabulary. With
    those, each of the shapes above would live with its owner, and Pandect's
    Alembic edge would go.
- **Custody and admission.**
  - One custody API exists in three shapes:
    - the door's `CustodyCall`, with 21 variants;
    - Djinn's `ResidentCustody`, with 29 methods;
    - `custody-wire`'s `Operation`, with 22.
  - `CustodyCall` mixes app calls (Release, Attest, Sign, Status) with owner
    and admin calls (ChooseProfile, CreateProfile, RevokeDevice,
    IssueStationGrant).
  - Each of three doors has its own admission path: the app door, Djinn's
    browser door, and `graphshell-host`'s WebRTC door. Projection admission
    (notochord triples, `graphshell-host/src/admission.rs`) is a fourth model.
  - The custody plan says a forwarded label never grants scope, and that
    releasable-salt permission becomes per-principal grants. The code differs:
    - Every installer defines one principal, `djinn`.
    - Its grants (`repos/dramatis/crates/djinn-custodian/packaging/djinn-grants.json.in`) are the union of
      every app's salts.
    - They repeat Djinn's `ReleasePolicy` (`ports/mere-resident/src/custody.rs`). One
      test is the only check that the two agree.
- **Transport.**
  - Consumers call `P2pandaTransport`'s own p2panda and iroh methods rather
    than the `Transport` trait. Its `builder_for` has 23 call sites outside
    Murm.
  - Distillery and Djinn bind at least five separate transports per resident.
    Djinn's inference mesh defines its own ALPN
    (`resident_inference/mesh_discovery.rs`).
  - Isocosm opens a raw iroh endpoint (`isonetry/src/iroh_link.rs`). Knot runs
    a bare quinn server (`knot-editor/src/mark.rs`), which Murm's consolidation
    survey doesn't list.

### Djinn holds what apps can't declare

- Djinn has 46k lines in `src`. Its resident core is about 6k:
  - the composition in `bin/djinn.rs`;
  - the lane container (`resident.rs`);
  - status and events;
  - blobs;
  - the custody client;
  - the generic app door.
- By purpose, the rest belongs to other owners:
  - Distillery: about 5k lines (`resident_distillery`, `resident_inference`);
  - Murm and sync: about 7.9k, whose replica and offer vocabulary is still
    Graphshell's;
  - Knot;
  - publication: a hand-built Gemini server (`resident_site`), Gemot's
    `moot_site` and Retinue's `micron_site`;
  - Dramatis: the identity endpoint and projection, enrollment and the keeper;
  - Graphshell: the browser carrier;
  - Mere: the graph routes (`resident_mere`);
  - an OS adapter: `conditions` and `lock_triggers`;
  - settings: `settings.rs`, one closed struct that holds every lane's
    settings.
- Apps can't declare themselves, so Djinn names them:
  - The app list is hard-coded in three places: `custody.rs:46`
    (`CUSTODY_APPS`, ten labels), `bin/djinn.rs:1284` and
    `embedded_reservoir.rs`.
  - Routes are registered and granted by label (`bin/djinn.rs:1243-1335`).
  - The door's `AppMessage` (`graphshell-endpoint/src/door/wire.rs:62`) is
    Connect, Request, TakeNotice, WaitNotice or Call. No message lets an app
    offer a projection.
- **2026-10-30 follow-up:** external apps can now offer a projection route through
  the existing typed Call/Answer envelope. `door::projection_offer` owns its
  portable messages, `door::projection_provider` the native runner, and Djinn's
  `door::projection_offers` owns permission and lease handling. The standalone
  Knot directory endpoint is the first consumer. This addresses projection
  registration, not the full proposed app manifest, custody salt grants or
  extraction of embedded resident lanes. See the [implementation plan](../implementation_strategy/2026-10-30_app_projection_offers.md)
  and [qualification](../testing/receipts/2026-10-30_app_projection_offers.md).
- Djinn and Cleromancy form a repository cycle:
  - Djinn installs Cleromancy's divination validator in the mere route
    (`resident_mere.rs:131-138`).
  - Djinn depends on `cleromancy-core` by git.
  - Cleromancy links `djinn`.
- The door could carry an app manifest: routes, projections, salt namespaces
  and persona use. With one, each piece above could live with its owner, and
  Djinn's home would be a question about its 6k-line core rather than all
  46k lines.

### "Graphshell" names three things

- **The projection protocol:** Chirograph, the endpoint traits and
  `graphshell-client`. Since 2026-08-01, `crates/graphshell` has had 79
  commits and Chirograph 20.
- **The graph product:** `ports/graphshell`, with 93.6k lines in `src` and 925
  commits over the same period.
  - About 9.1k of its lines are adapters for other owners: Woodshed practice,
    Knot blocks, Pelt retained content, Moot resources, the Alembic workshop
    and Distillery.
  - `ports/graphshell/docs` holds about 269 MB of committed receipts.
- **Djinn's client library:** the `door` module in `graphshell-endpoint`,
  about 4.5k lines, which handles app admission and acts as the custody client.
  - The socket (`graphshell-app.sock`), the environment variables
    (`GRAPHSHELL_APP_ENDPOINT`, `GRAPHSHELL_PROFILE`), the hello schema and
    Djinn's app directory all carry Graphshell's name.
  - Ruling R3 of the stack lift plan put the door in the endpoint crate,
    because apps already depended on that crate.
  - As a result, Woodshed and Hocket link Pandect, the kernel and the wallet
    crypto through the door for two types, `DeviceId` and
    `RemoteAuthRevocationOutcome` (`door/custody_client.rs:28`).

The scope test files the four `graphshell-*` crates together as a capability
contract. The code divides them:
- Chirograph, the endpoint traits and `graphshell-client` are the projection
  contract.
- The `door` is a custody and admission client.
- `graphshell-host` (`publish = false`) holds admission and lifecycle. Beside
  them sit the WebRTC door (2.8k lines) and about 2.7k lines of fixtures and
  receipt views.

A persona and custody client under its own existing name would let custody and
projection change apart, and Graphshell's name would mean one thing.

### Two graph edit spines

- The kernel wraps `chartulary::Graph` (`graph-kernel/src/graph/mod.rs:352`)
  but writes through its own muniment `GraphJournal`. Six consumers write
  through Chartulary's revision-checked `GraphLog` instead: Servitor,
  commons-spine, Turnstone, Woodshed, Knot and Cleromancy.
  - In Mere, `GraphLog` appears only in tests and probes.
  - Turnstone keeps each participant in both graphs, under different authors.
  - The G5 note that justified a separate journal calls `GraphLog`
    "topology-only". That was before `EditSpec` gained `SetFacet`, `Assert`
    and `Derive`.
- Servitor's gate is typed to `GraphLog`, `Container` and `Relation` (Eidetic
  `servitor/src/gate.rs`), so it can't petition the main graph.
- Pandect has two session-persistence stacks:
  - files (`manifest_store`, `session_graph_store`, `facet_store`), which
    Turnstone uses;
  - muniment (`graph_session`), which Djinn and Graphshell use.
- Merge is written four times: the kernel's `merge.rs`, and Pandect's
  `snapshot_merge`, `graph_session_merge` and `merge_rules`.
- With one edit spine, Servitor's `Cap`, which Gemot already maps onto, would
  govern the main graph. It would also be clear which half of Pandect is graph
  sessions.

### Code with no consumer

Every move carries this weight, and it counts toward each layout's costs below:
- `chrome`, apart from about 380 lines (`nav::History`, `classify`);
- six of the registry's modules;
- about 4.8k parked Forme lines, which `forme/src/lib.rs` says "will be
  removed";
- about 1.7k lines of donor-era kernel host vocabulary (`pane`, `host_event`,
  `overlay` and others), used only by `chrome`, `ux-events` and the registry;
- Castellan's one-time-password and Secret Service code, about 7k lines;
- `gazette`.

Outside Mere, five Errand crates and Gemot's `moothold` have no consumer.

### What already fits the design

- **Already projecting:**
  - Knot;
  - Isocosm: its world, six game-table panels, Mesocosm and Eponym;
  - Turnstone;
  - Cleromancy;
  - Distillery's Chronicle, which implements all six traits
    (`ports/distillery/src/chronicle.rs`);
  - Djinn's own endpoints.

  Woodshed and Hocket don't project.
- **G6, one workspace composing several apps' projections, isn't met in
  code:**
  - `ClientState` is a mount cache keyed by a `ProjectionSession` that isn't
    qualified by endpoint (`graphshell-client/src/lib.rs`).
    `graphshell-host/src/lifecycle.rs` warns that a second mount replaces the
    first unless the caller mints per-session randomness.
  - sceno has no notion of a mount or a fragment.
  - A saved workspace (`pandect/src/saved_scene.rs`) has no list of endpoints.
- **The plan record lags.** The Graphshell host plan carries G6 and G7 without
  an owner. Its line that they gate publishing four `graphshell-*` crates is
  stale: Chirograph, `graphshell-client`, `graphshell-endpoint` and
  `graphshell` were published at 0.0.2 on 2026-08-23 and 24.
- **Clean upstream:** Gemot needs nothing from Mere. Eidetic, ESP and
  Armillary also sit cleanly upstream of it.

### What this means for the homes

- The lines between homes follow four contracts that the stack doesn't have
  yet:
  - a persona and custody client that isn't part of the projection protocol;
  - an app manifest that the door carries (since 2026-10-30, M1 of the
    [capability door plan](../implementation_strategy/2026-10-30_capability_door_plan.md));
  - a settings mechanism with one scope vocabulary;
  - one graph edit spine.

  A package that moves before its contract exists takes today's copies with
  it, and each copy becomes an edge between repositories.
- Armillary, Personae, Eidetic and Servitor each left Mere, came back, and left
  again within about three months.
- This map records Mere's scope from conversation, not from a ruling. Mere's
  README still calls Graphshell "Mere's reference host".

## Costs every move carries

- **Repository cycles.** Gemot, Dramatis, ESP, Armillary and Eidetic sit
  upstream of Mere. A product or adapter that moves into one of them and keeps
  a Mere dependency creates a repository cycle, even when the package graph has
  none. Mere↔Knot and Mere↔Cleromancy already work this way, and
  their renames on 2026-10-25 showed the price: lockfiles regenerated before a
  push, and consumers updated in supplier-first order.
  - A separate workspace root inside the receiving repository keeps that
    repository's lockfile and CI scope free of Mere. Murm's
    `apps/signalman-desktop` already does this.
  - The source-identity seam remains: one side reaches the other through a
    pinned git source and a `[patch]` row.
  - "A repository hosts only what is upstream of it" is a policy choice, not a
    requirement.
- **Identity preservation.** The 2026-10-19 promotion moved history, licenses,
  stored formats and default features unchanged and left immutable consumer pins
  alone. The house naming rule allows an unpublished `mere-` package to drop the
  prefix when it leaves Mere.
- **A test that compares layouts.** For each candidate layout, check two things
  (proposed 2026-10-27):
  - Turnstone and Knot still resolve one source per shared package, with no
    `[patch]` rows beyond today's.
  - Count the repositories that must be repinned for a change in the moved
    package to reach Turnstone.
