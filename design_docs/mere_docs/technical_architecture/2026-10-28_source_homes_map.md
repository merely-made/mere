# Source homes map: which Mere packages fit another repository

**Date:** 2026-10-28
**Status:** findings, read at Mere `d4d5b0c32` (`cargo metadata` over the
workspace) and at the 2026-10-28 `main` of Murm, Gemot, Eidetic, ESP, Isocosm,
Knot, Turnstone, Cleromancy and Woodshed (their manifests' Mere dependencies).
Nothing has moved yet. This is the dependency evidence that the family promotion
plan's F3 asks for before it chooses a grouping
([family promotion plan](../implementation_strategy/2026-10-28_family_promotion_plan.md)).

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
