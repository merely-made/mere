# Graphshell stack lift plan

**Status (2026-10-14):** plan; nothing moved. The census below was read from
the source at `0aade8668` and corrects the first pass in several places
(§2). Five rulings from Mark are needed before the phases that name them
(§5); phases P1 to P3 need none. No code, manifest or other plan changed with
this document.

**Origin.** Mark noticed that the Graphshell reference host is about 100k
lines and asked what could be generalized into the stack. Knot, Cleromancy
and Woodshed are endpoints that want Graphshell's interactive capabilities
without taking the whole port. Woodshed's own survey
(`woodshed/design_docs/2026-10-14_dsp_graphshell_hocket_survey_brief.md`,
§2 and §3 item 3) asks the same question from its side: whether Mere extracts
Graphshell's canvas controls and editors into native-buildable crates, with
Woodshed as the second native consumer.

**What this plan decides and what it does not.** It names a home, a
prerequisite, a second-host receipt and a risk for each movable group, and
orders them. It moves nothing. Each phase's code change is its own pass
under the rulings in §5.

## 1. What 100k lines is

`ports/graphshell/src` holds 99,749 lines, and they are not one package:

| Part | Lines | Package |
|---|---:|---|
| Library modules listed in `src/lib.rs` | 70,559 | `graphshell` |
| `web.rs`, `web_tree/` (21,004) and the other `web_*` files | 24,852 | `graphshell-web` (cdylib, wasm32 only; `web/Cargo.toml` sets `[lib] path = "../src/web.rs"`) |
| `src/bin/` | 4,338 | `graphshell` binaries, mostly receipt fixtures |

Integration tests (3,188) and examples (1,118) sit beside them. Lines
include in-file tests: `forme_session` carries 1,326 lines of lifecycle
tests, `personal_sync` about 1,800.

Who actually compiles the `graphshell` package from outside the port
(imports counted from source, 2026-10-14):

| Consumer | What it names |
|---|---|
| Woodshed (`woodshed-genet`, Hocket) | `native::custody_client`, `native::custody`, `native::custody_identity`, `native::app_admission`, `identity::VaultLockView` |
| Knot (production source) | `native::app_admission`, `native::custody`, `native::custody_client`, `identity::VaultLockView` |
| Knot (tests and examples only) | the serving stack: `admission`, `carrier`, `network_carrier`, `session_notices`, `lifecycle`, `native::projection_host`, `native::endpoint_catalog`, `native::app_broker`, `native::local_session`, `sessions` |
| Cleromancy | `personal_sync` (`PersonalGraphReplica`, `PersonalGraphEvent`, `SyncProjection`, `SyncRoster`, `SyncSelection`), `native::endpoint_catalog`, `lifecycle::AdmittedEndpointContext`, `native::app_admission`, `view` |
| Signalman, `ports/dramatis` | `native::custody_client`, `native::app_admission`, `native::quarantine`, `native::custody` |
| Djinn | a dozen `native::*` modules, `session_item`, `mere_route_session`, `mere_route::archive_intent`, `browser_carrier`, `identity*`, `lifecycle` |

So the demand is concentrated: five external or sibling consumers want a
few thousand lines of custody and application-door code, and Djinn runs the
server halves. Nobody outside the port imports the canvas reducers, the
editors or the Forme session today. Woodshed names them as wanted.

## 2. Corrections to the first-pass census

The first pass was by imports. These are the places source disagreed with it.

1. **`forme_workspace` and `forme_session` cannot go to `crates/forme/forme`.**
   `forme` depends on `curation`, `taffy`, `uuid` and serde only. Both modules
   use `platen::TileLayout` and `platen::projection_geometry::TreeGeometry`,
   and `platen` depends on `forme`, so the move is a cycle. They also touch
   pictograph (`FormeRegion` and `FormeCell` in the workspace;
   `mere::canvas::Canvas` in `install_edge_preview`, `forme_session.rs:275`),
   `muniment` and `edit-history`. Only the `mere` facade already depends on all four of
   forme, platen, workbench and pictograph; `platen` sees forme and workbench
   but not pictograph (§5, R2).
2. **Projection authoring is thinner than 2.2k lines of shared code.**
   `projection_compile.rs` is mostly `pub use scenomise::…` plus Graphshell's
   card size (`PRACTICE_CARD`, 164 by 68) and `default_definition`, a recipe
   whose fields (`order`, `tempo_bpm`) and note ("Disclosed Woodshed Set") are
   Woodshed's. `projection_editor.rs` carries `chronicle_definition`, which is
   Distillery's recipe (`distillery.chronicle`). Those are host recipes, not
   shared authoring.
3. **Knot's composition drawer is not a second host of the projection editor.**
   No file in `knot-editor` mentions `ProjectionEditor` or `ProjectionDraft`.
   Knot (`apps/desktop/src/composition/recipe/`, 5,591 lines) and Woodshed
   (`woodshed-views/src/stage/relationship.rs`, 1,013) both edit
   `scenograph::relationship::RelationshipRecipeDraft`, which already lives in
   the stack, so the relationship recipe has already passed the anti-shell
   test. What Knot re-implements beside Graphshell is small: the option
   helpers (`option_default`, `declared_options`, `default_description` in
   `recipe/editor.rs`, against `arrangement_options`, `option_placeholder`,
   `with_kind` in `projection_editor.rs:463-530`) and an undo wrapper
   (`RecipeHistory`, 348 lines, over `edit-history`).
4. **`presentation_binding` is not clean.** It imports
   `crate::link_inspection::{LinkTarget, inspect_link}`, so the cluster is
   presentation plus `link_inspection` (332 code lines) plus
   `access_inspection` (267). `scenograph::presentation` already owns the
   rule model (scenograph editor plan R1); what the port holds is the host
   adapter, binding and per-occurrence hysteresis runtime.
5. **`canvas_physics` is not host-neutral.** Its production code imports
   `crate::product::ProjectionClock` (`canvas_physics.rs:22`). `canvas_reader`
   production code is clean; its tests import `app`, `mere_host` and
   `remote_board`. `canvas_controls`, `canvas_gestures` and `canvas_faces`
   import only `mere::canvas`. `viewer_commands` imports `canvas_controls` and
   `canvas_gestures::WheelMode`, so they move together.
6. **`coupling_edit` code is clean; its tests are not.**
   `coupling_edit_tests.rs` uses `app` and `mere_host`. The same holds for
   `forme_session`'s lifecycle tests, which use `access`, `capture`,
   `local_edit`, `local_intake`, `local_scene`, `mere_host` and `product`
   (1,326 lines). Tests either stay as host acceptance tests or are rewritten
   against the lifted crate's own fixtures; they do not travel as they are.
7. **`practice_*` is not Woodshed code in the sense the first pass assumed.**
   `practice_workspace` (783) is a host-neutral reducer that imports only std
   and serde. `practice_disclosure` (775) holds the one Woodshed-specific
   part: it parses Woodshed's comparison export and builds a host dataset
   from it. A Graphshell crate cannot import a Woodshed crate (the stack does
   not depend on applications), so "move it into Woodshed's embeddable half"
   reverses the direction. The right split is that Woodshed publishes the
   host dataset (`scenomise::host_dataset`, already a stack schema) and
   Graphshell consumes by schema. Woodshed's own plan calls the practice page
   "a wrapper around the same Graphshell component, not native Woodshed
   integration" (`woodshed/design_docs/2026-09-04_musical_projections_plan.md`,
   stop rules).
8. **The resident group is bigger and more entangled than 25k.** The named
   set (`native/` 12,144, `personal_sync` 3,693, `transfer*` 3,360,
   `carriage` 608, session stack 4,972) is 24.8k. `personal_sync` imports
   `crate::access` and `crate::product` (`personal_sync.rs:41-42`: saved-scene
   facet constants and `SavedSceneV2`), so it cannot lift before the facet
   shapes settle. `native/projection_host.rs` imports
   `crate::webrtc_session::serve_webrtc_join`, so the serving half drags a
   WebRTC door with it or gates it by feature. `admission`, `webrtc_door` and
   `webrtc_join` sit in the wasm `web` cone through the `webrtc-join` feature,
   so any crate that takes them must stay wasm32-clean.
9. **The identity door is already ruled.** `identity_endpoint` (1,170) and
   `identity_projection` (874) look like castellan's, but
   `identity_projection.rs`'s own header records that the cards moved back to
   Graphshell under the dramatis repo plan's DR-C (ruling D15). They are the
   generic endpoint adapter over the `ResidentIdentity` trait; they stay.
10. **Knot's production dependence is as the brief said; its tests are not.**
    Production source names only custody and admission types. The serving
    stack appears in `tests/place_projection.rs`, `tests/resident_app_route.rs`
    and `examples/k2_peer.rs`.
11. **`capture` is not Turnstone's today.** `capture.rs` (2,379) is consented
    browser-history intake into Mere and Eidetic browsing memory, built on
    `MereHost`, `access` and `product`. The Eidetic development plan's P0
    inventories "Graphshell capture, Turnstone navigation, portable import"
    and assigns owners; this plan leaves `capture` to that inventory.
12. **The `web_tree` views are page-shaped, not portable pieces.** Twelve of
    its files call no browser API (`query_view`, `remote`, `commands`,
    `field_inspection`, `speed`, `physics`, `link_inspection`, `history`,
    `dynamics_inspection`, `coupling`, `inspection`, `producer_scene`; about
    5.2k lines), but every one begins `use super::*;` and reads the `Page`
    state in `web_tree.rs`. They are in the wasm crate by placement, not by
    need, but moving them is a design pass over `Page`, not a file move.

## 3. What grounds the homes

- **Port law** ([composition thesis §3](../../2026-08-12_family_composition_thesis_brief.md)):
  the stack owns a capability; a port is its first-party embeddable
  embodiment; another application composes the relevant subset without
  cloning the reference application or inheriting its authority. Graphshell's
  `native/` is the clearest breach: it is a library for other ports.
- **Anti-shell test** (same section): a capability that works only with
  Graphshell outermost is a shell with plugins. The receipt is a second host
  that keeps its own identity, not a second view. Every lift below names one,
  and a lift with no second host today is marked as waiting, not scheduled.
- **Suite composition census** ([§3](../../2026-08-22_turnstone_suite_composition_and_capability_census.md#3-corrections-to-older-posture)):
  Graphshell stays the overmap, local Mere host, remote projection client,
  transfer surface and handler router; Castellan owns the identity product
  surface; Djinn owns the resident composition, application door, route
  catalog, store lifetimes and shutdown. §4's table sends a reusable control
  or layout pattern to Cambium, a background service lifetime to Djinn and a
  Turnstone-specific workflow to Turnstone.
- **`scripts/check_port_boundaries.py`:** no package under `crates/` may
  depend on a package under `ports/`; exactly one `graphshell` package exists
  and it is `ports/graphshell`; the `graphshell` web cone on wasm32
  (`--no-default-features --features web`) may contain none of the forbidden
  engine packages. Lifted code therefore lands in `crates/`, never in a port
  that a crate would then need, and every crate on the web cone must be
  wasm32-clean. A stack crate named `graphshell-*` is allowed; the check is on
  the exact package name `graphshell`.
- **Resident host design** ([2026-10-09](../technical_architecture/2026-10-09_resident_host_design.md)):
  the product operations in `product.rs` (create address, edit, set facet,
  assert relation, pin, file metadata) become typed Pandect live commands.
  The open resident-host port ([lane handoff](../../2026-10-14_mere_lane_handoff.md#open-resident-host-port-o-pc))
  rewrites the surface this plan would otherwise move.
- **DOC_POLICY §3, no legacy friction:** lifts leave no re-export shims in the
  port. In-tree consumers move in the same change; external consumers take
  the new paths at their next repin.
- **A recorded ruling is evidence, not a verdict** (working principles): where
  this plan's evidence cuts against an earlier ruling (finding 9) it says so
  and does not reverse it silently.

A separate custody-scoping task may be in flight. Nothing in the doc tree or
in this session's notes folder records one at this date; if it exists, its
rulings on what an application may ask custody for take precedence over §4
L1's wire contents, and L1 changes where the wire lives, not what it says.

## 4. The map

Sizes are lines including in-file tests unless noted. "Receipt" is the
second-host proof the lift must produce.

### Group 1: lifts that look clean, re-measured

| ID | Modules (lines) | Home | Prerequisite | Second-host receipt | Value / risk |
|---|---|---|---|---|---|
| L5 | `canvas_controls` (113), `canvas_gestures` (271), `canvas_faces` (141) | `pictograph` `canvas` | Reconcile `canvas_gestures` with `canvas/input.rs` (899 lines, already has `wheel`, zoom and pan) rather than copying; scenograph editor plan SE24 already ruled gestures belong in the pictograph canvas | Woodshed's native graph surfaces drive the same controls over `pictograph` with only the `canvas` feature | Medium / low |
| L6 | `viewer_commands` (232) | Waits | A second host mapping the shared `command-menu` verb ids to canvas operations; SE48 to SE51 say Graphshell and Turnstone both register them | Turnstone maps `view:fit` and its siblings through the same function | Low / low |
| L7 | `frame_budget` (940) | `pictograph` `canvas` beside `StepBudget` and `FALLBACK_DISPLAY_PERIOD` | A second browser host of the canvas | A second wasm host paces its canvas with `Period` | Low / low, waits |
| L8 | `coupling_edit` (102 code) | with L5 | Tests rewritten off `app` and `mere_host` | Woodshed or Turnstone edits one coupling's strength through the draft | Low / low |
| L9 | `appearance` (417) | Stays (§5, R6) | Tabard's division: Tabard owns definitions and derivation, the application owns its preference record and role mapping; `canvas_face_palette` also needs `mere::canvas`, which Tabard cannot depend on | none | n/a |

`canvas_physics` (1,104) stays until `ProjectionClock` leaves `product`; it is
the reducer Woodshed also wants, so cutting that one import is a P2 task.

### Group 1b: projection authoring and presentation rules

| ID | Modules | Home | Prerequisite | Second-host receipt | Value / risk |
|---|---|---|---|---|---|
| L4 | The option helpers in `projection_editor.rs:463-530` (`arrangement_options`, `arrangement_defaults`, `option_placeholder`, `with_option`, `with_kind`) | `scenomise` (it owns the solver registry and catalog the helpers read) | None | Knot's `recipe/editor.rs` deletes its three copies and calls the shared helpers | Medium / low |
| L4b | `ProjectionEditor` reducer, panels, `ProjectionDefinitionSink` (about 540 code lines) | Stays until a second host edits whole `ProjectionDefinition` drafts (§5, R1) | R1 | A host other than Graphshell opens and saves a whole-definition draft | Medium / medium, waits |
| L4c | Host recipes: `chronicle_definition`, `default_definition`, `PRACTICE_CARD` | The owning domain: Distillery (chronicle), the Woodshed export (practice) | Woodshed publishes the recipe in its dataset; Graphshell keeps its card size as host policy | Graphshell renders the Woodshed fixture without a Woodshed-named function | Low / low |
| L10 | `presentation_editor` (511 + 301 tests) | Waits: `scenograph::presentation` already holds the model; the port holds a Muniment-backed workspace record | A second host that stores rule sets | A second host loads and edits a rule set through the same record | Low / low |
| L11 | `presentation_binding` (306 + 328), `node_presentation` (204 + 216), with `link_inspection` and `access_inspection` | `crates/mere` (it already holds `roster`, `subgraph`, `query_view`, `gloss`), once a consumer exists | Move `link_inspection` and `access_inspection` first; they import only `mere::kernel` | Turnstone's or Woodshed's canvas resolves node faces from the same rule set; the `canvas::node_presentation` surface in pictograph (64 lines) is the paint end | Medium / medium, waits |

### Group 1c: forme

| ID | Modules | Home | Prerequisite | Second-host receipt | Value / risk |
|---|---|---|---|---|---|
| L12 | `forme_workspace` (400 + 281), `forme_session` (1,078 + 1,326 tests) | `platen` behind a feature, or `crates/mere` (§5, R2). Not `crates/forme/forme` | R2; cut the three canvas-touching pieces (`region`, `cells_of`, `install_edge_preview`) into a thin adapter; decide whether Muniment persistence is a feature of the lifted crate or an adapter | Turnstone's workbench, or Knot's desktop workspace (it already holds a `workbench` tile tree), unlocks, drafts, discards and applies a forme through the same session. The scenograph editor plan records "adoption by other hosts remains pending" | Medium / medium |

### Group 2: Woodshed

| ID | Modules | Home | Second-host receipt |
|---|---|---|---|
| W1 | `practice_disclosure`'s adapter (comparison export to host dataset) | Woodshed publishes the dataset; no stack crate imports Woodshed | Graphshell's practice page loads `fixtures/woodshed-comparison.host-dataset.json` produced by Woodshed, and `practice_disclosure.rs` loses the Woodshed parser |
| W2 | `practice_workspace` (783) | Stays: one consumer (`graphshell-web`), generic | none |
| W3 | Test include at `practice_workspace.rs:591` (`../../../../woodshed/scenarios/woodshed_musical_comparison.json`) | Replace with the vendored fixture that already exists in `web/fixtures/` | Test passes in a worktree with no sibling `woodshed` checkout |

`web/fixtures/woodshed-stage.json` is a vendored copy of a Woodshed export
(`web_tree/product/projection.rs:23` includes it). W1 and W3 retire the
cross-repo include and make the vendored fixture's provenance explicit (which
Woodshed commit produced it).

### Group 3: resident authority

The port is acting as a library for other ports. The seam is the application
door: a first-party app reaches djinn on a local socket, is admitted by a
self-declared hello that grants nothing by itself, and calls routes. The code
splits into what an application embeds, what an authority serves, and what
Djinn runs.

| ID | Modules (lines) | Home | Receipt |
|---|---|---|---|
| L1 | **App-facing half**: `native/app_admission` (586), `app_client` (452), `custody` (496), `custody_client` (558), `custody_identity` (449), `quarantine` (60), `local_endpoint` (277), `tasks` (103); the wire types in `app_broker` (`APP_CONNECT_SCHEMA`, `AppMessage`, `AppHostMessage`, `AppBrokerError`); the NDJSON frame functions in `browser_carrier` (`read_native_message_async`, `write_native_message_async`); `identity::*` re-exports from `dramatis::view`; `profile::env_profile` (240). About 3.6k lines | A new small crate under `crates/graphshell/` (working name `app-door`; Mark names it, R3) | Woodshed's `woodshed-genet` and Hocket, Knot's `startup.rs`, Signalman and `ports/dramatis` depend on it alone and build with no `graphshell` package in their cone. Today Woodshed's default features compile the whole retained-content cone for these imports |
| L2 | **Serving half**: `admission` (372), `lifecycle` (718, with `AdmittedEndpointContext`, `BindAdmittedSession`, `SessionAuthority`), `native/endpoint_catalog` (635), `carrier` (694), `network_carrier` (271), `session_loop` (943), `session_notices` (398), `resume` (447), `live_endpoint` (738), `native/projection_host` (836), `native/resident_identity` (136, the `ResidentIdentity` trait), `policy_projection` (391); `webrtc_door` and `webrtc_session` follow or are feature-gated | A second crate under `crates/graphshell/` beside `graphshell-endpoint`; that crate's README already says admission and the serve loops "live in `ports/graphshell`" and names this as the not-yet-extracted half | Cleromancy registers an endpoint with `ResidentEndpointCatalog` and `AdmittedEndpointContext` without `graphshell`; Knot's `place_projection` test serves a projection with it; both build for their target features |
| L3 | **Server halves Djinn runs**: `native/app_broker` (522), `device_broker` (285), `browser_host` (668), `local_session` (243), `resident_mere_proxy` (218), `personal_sync_host` (2,319), `carriage_host` (564 + 290), `transfer_staging` (673), `graph_keys` (840) | `ports/djinn`, which is the only caller of `serve_app_broker` (three call sites) | Knot's `resident_app_route` test and the `graphshell_native_host` binary run against Djinn's broker or a small fixture host; `graphshell` no longer holds a broker |
| L13 | `personal_sync` (3,693), `receipts` (1,368), `transfer_offer` (363) | Waits. A stack home (Pandect or a dedicated crate) for the replica, event and projection types Cleromancy imports | The facet shapes in `access` and `product` that `personal_sync.rs:41-42` reads must be settled (see the [facet dissolution brief](../research/2026-10-08_facet_dissolution_brief.md)) | Cleromancy builds `PersonalGraphReplica` with no `graphshell` package |

`identity_endpoint`, `identity_projection`, `browser_carrier`'s browser
bridge, `webrtc_join`, `webrtc_browser`, `sessions`, `canary` and `view` stay
(finding 9; they are the generic adapter and the G-series receipts).

### Group 4: product truth

`mere_host` (2,008), `product` (4,142), `local_edit` (871), `local_intake`
(416), `access` (580), `access_inspection` (267), `link_inspection` (701),
`field_inspection` (590), `session_item` (320), `mere_route` (452),
`mere_route_session` (715), `transfer` (2,512), `transfer_endpoint` (485),
`capture` (2,379). Total about 16k with the transfer and capture modules the
first pass had placed elsewhere. `product.rs` already imports
`projection_compile` and `projection_editor`, so it follows L4.

No lift is planned. The resident host design replaces `product.rs`'s
operations with Pandect `LiveCommand` values, and the open port will change
`mere_host`, `product` and the refresh path. The one exception is
`access_inspection` and `link_inspection`, which are readers over
`mere::kernel` only and move first if L11 proceeds. Djinn consumes
`session_item` and the `mere_route*` pair today (port to port, allowed).
`transfer` stays as the transfer surface the suite census gives Graphshell;
its staging and offer halves follow L3 and L13.

### Group 5: stays in the port

`web_tree` and the other `web_*` files (24,852), `canvas_physics`,
`canvas_reader`, `host_dataset_view`, `host_history_view`, `host_presenter`,
`local_scene`, `remote_board`, `speed_bar`, `retained_content` (1,877),
`retained_policy`, `query_views`, `moot_resources` (905), `app`, `view`,
`sessions`, `canary`, `capsule_applet`, `distillery_w1`, `handlers`,
`browser_storage`, `browser_carrier`'s browser bridge, the `webrtc_*` browser
half, `identity_endpoint`, `identity_projection` and the receipt binaries.
`capture` waits on Eidetic P0. `canvas_reader` is the one module on this list
Woodshed also names; its production code is clean (finding 5), so it can move
with L5 once a native second host needs a screen-reader producer.

Woodshed also asks for the Cambium views in `web_tree`. That is finding 12:
a design pass over `Page`, and it belongs behind L5 and L12, because those
views are the thin layer over the state those lifts expose.

## 5. Rulings needed

Each carries a recommendation and the evidence. None is needed for P1 to P3.

- **R1, where whole-definition authoring state lives.** Options: keep the
  reducer in Graphshell until a second host needs it (recommended: no second
  host exists, finding 3); a new crate under `crates/cambium/scenes/` (the
  reducer needs `scenograph`, `scenomise`, `workbench` and `edit-history`, so
  it cannot sit in `scenograph`, whose docs say hosts own editor UI and which
  scenomise depends on); `scenomise` itself. Moving it ahead of a consumer
  would add `workbench` and `edit-history` to a crate whose header says it has
  no runtime or widget dependency.
- **R2, where Forme workspace and session live.** Options: `platen` behind a
  feature (recommended once the three canvas-touching pieces are cut: it
  already holds `TileLayout`, the bridge and `TreeGeometry`; adding
  `muniment`, whose core dependencies are `async-trait`, `blake3` and serde,
  is small); the `mere` facade (the only crate that sees pictograph too, so
  it needs no cut, but its header excludes session-host concerns); stay until
  a second host. `crates/forme/forme` is excluded by the cycle.
- **R3, the app-facing crate's name and location.** Working name `app-door`
  under `crates/graphshell/`. Mark names crates. The custody-scoping task, if
  it exists, may already have a name.
- **R4, the serving half: stack crate or Djinn.** Recommended: a stack crate,
  because Knot and Cleromancy would otherwise depend on a port that links
  castellan. Ports may depend on ports (Djinn depends on `graphshell`); the
  cost is the external repos' dependency cone.
- **R5, whether `personal_sync` types move before the facet work settles.**
  Recommended: no.
- **R6, `appearance`.** Recommended: stays; it is the application's preference
  record, which Tabard's division gives to the application.

## 6. Phases

Ordered by value and risk. A phase that lands code ends with the second-host
receipt named for it; a receipt in this repository's tests does not stand in
for the other repository's build.

**P0. Rulings and ledger.** Settle R3 and R4, reconcile with any custody task,
and record each phase's baseline: the `graphshell` dependency cone per
consumer (`cargo tree` on each external consumer, not measured here because
the git dependencies need network access), and the per-module `crate::`
import graph in §2 (method in Findings).
*Done when:* R3 and R4 are ruled or explicitly deferred; the baseline numbers
are in Findings; the custody task is referenced or recorded as absent.

**P1. L1, the app-facing crate.** Highest value: five consumers outside the
port, and the most expensive current dependency. Split the wire types out of
`app_broker` and the framing out of `browser_carrier` first (the two real
tangles: `app_client` imports both), then move the files. Update Djinn,
Signalman, `ports/dramatis` and `graphshell_native_host`'s imports in the same
change. No re-export from `graphshell::native`.
*Done when:* the crate builds for native and has no `graphshell` dependency;
Signalman and `ports/dramatis` build against it alone; `custody_client` and
`app_client` tests pass in the crate; the boundary script passes; Woodshed's
and Knot's next repin names the crate and their cones lose the
retained-content cone, recorded in their plans.

**P2. L5, L8 and the `ProjectionClock` cut.** Canvas controls, faces and
gesture parity into pictograph; `coupling_edit`; move `ProjectionClock` out of
`product` so `canvas_physics` can follow. Woodshed's native surfaces are the
receipt.
*Done when:* pictograph holds the controls with `canvas::input` reconciled
(one wheel and zoom path); Graphshell's web page calls them from pictograph;
a Woodshed (or fixture) native surface drives them with only the `canvas`
feature; `web_tree/gestures.rs` and `controls.rs` call the pictograph functions.

**P3. L4 and L4c.** Option helpers into `scenomise`; Knot's copies deleted;
host recipes go to their domains.
*Done when:* Knot's `recipe/editor.rs` uses the shared helpers and its tests
pass; Graphshell's editor uses the same; no `chronicle_definition` or
`default_definition` remains in a shared path.

**P4. L2, the serving half.** Builds on P1 (the stack crate takes
`AdmittedEndpointContext`, which `endpoint_catalog` already names). Needs R4
and a decision on the WebRTC door's placement.
*Done when:* the crate is wasm32-clean for `admission`, `webrtc_door` and
`webrtc_join` and the web cone check passes; Cleromancy registers an endpoint
through it; Knot's `place_projection` test serves through it; the port keeps
only `identity_endpoint`, `identity_projection` and the generic adapters.

**P5. L3, Djinn takes the server halves.** After P4, because `app_broker` and
`browser_host` use `session_loop` and `session_notices`.
*Done when:* `graphshell` holds no broker; Djinn's tests and Knot's
`resident_app_route` pass; `graphshell_native_host` and the C4 host fixture
build against Djinn or a fixture host.

**P6. L12, forme.** Needs R2 and a second host (Turnstone's workbench or
Knot's workspace) that names its done condition.
*Done when:* that host unlocks, drafts, discards and applies a forme through
the lifted session; the lifted crate's tests carry the lifecycle cases that
today live in `forme_session_lifecycle_tests.rs` without importing the port.

**P7. L11, L10 and the inspection readers.** Waits for a consumer. If
Turnstone or Woodshed adopts presentation rules, `link_inspection` and
`access_inspection` move first, then binding and node presentation.
*Done when:* a second host resolves faces from the same rule set.

**P8. L13 and group 4.** Sequenced behind the resident-host port and the
facet dissolution brief. `personal_sync` types move once the facet constants
it reads have a stable owner; product operations follow the Pandect live
commands.
*Done when:* Cleromancy builds with no `graphshell` package; `product.rs`
holds no operation Pandect now owns.

**P9. The `web_tree` views.** A design pass over `Page`; start with the
least-coupled files (`speed.rs` names only `Shared`). Woodshed's view needs
(its `woodshed-views` already builds for wasm as `woodshed-web`) set the
constraint: model and reducer crates wasm-clean, native-only view pieces
behind a feature, and Mere's `mere-view` needing pictograph `signals`, which
Woodshed does not enable today, so adoption changes its feature set rather
than its revision.
*Done when:* a design note rules the `Page` split; no code moves before it.

## 7. Constraints from Woodshed

Relayed by the Woodshed session on 2026-10-14 and checked against source:

1. Consumers get client-side holders only. Woodshed never constructs
   `GraphshellApp`, `MereHost` or `MereSessions`; its stage plan
   (`woodshed/design_docs/2026-07-11_stage_set_tools_plan.md`, lines 411-414)
   says Graphshell is not imported as a second owner of Woodshed data. This
   is why L1 and L2 are the shapes Woodshed can take and L12 and L4b are not
   scheduled for it.
2. `woodshed-views` builds for wasm, so lifted model and reducer crates stay
   wasm-clean and native-only view pieces sit behind a feature.
3. The Woodshed survey read Mere HEAD, not its pin; its claim that
   `canvas_physics` is host-neutral is wrong (finding 5), and its list of
   editors and the Forme session is as §2 corrects.
4. Woodshed, Hocket and Redshank will repin to Mere main; the repin has not
   started and a separate session is narrowing Woodshed's `graphshell`
   features. This plan targets main and does not touch Woodshed's manifests.

## Findings

**2026-10-14 (source read at `0aade8668`, the tip of `origin/main` after a
fast-forward of local main; nothing built).**

- Package split: `graphshell` library 70,559 lines, `graphshell-web` 24,852,
  binaries 4,338. `web/Cargo.toml` stands on its own (`[workspace]`) and
  points its `[lib]` at `../src/web.rs`; `src/lib.rs` lists no `web_*` module.
- The `graphshell` default features are `native`, `image-decode`,
  `document-layout` and `retained-content` (`ports/graphshell/Cargo.toml`
  `[features]`); `retained-content` pulls `genet-render-host`, `cambium`,
  `fleece`, `wgpu`, `pelt-core`, `inker`, `netrender` and
  `mere-document-lanes`. Cleromancy and Signalman set
  `default-features = false, features = ["native"]`; Woodshed's root manifest
  and Knot's name `graphshell` at a pinned revision (`0b9b7a9`, Cleromancy at
  `a1309431`), so the consumers also disagree on revision.
- `native/` is 12,144 lines in 22 files. Import graph (production code): the
  app-facing leaves are `quarantine` (none), `tasks` (none), `custody`
  (`quarantine`), `custody_client` (`app_admission`, `app_client`, `custody`,
  `quarantine`), `app_client` (`browser_carrier` frame functions, `app_broker`
  wire types, `custody`, `custody_client`, `local_endpoint`), `app_admission`
  (`endpoint_catalog` through `super::`; its only `device_broker` use is a
  test at line 560), `endpoint_catalog` (`lifecycle::AdmittedEndpointContext`),
  `local_endpoint` (`tasks`). The server files (`app_broker`, `browser_host`,
  `device_broker`, `local_session`, `resident_identity`, `projection_host`)
  are tangled with `session_loop`, `session_notices`, `identity_endpoint`,
  `lifecycle` and `browser_carrier`.
- Callers of the broker: `serve_app_broker` is called from Djinn only
  (`djinn/src/bin/djinn.rs:55`, `embedded_reservoir.rs:28`, and
  `resident_mere.rs:44` for the catalog type); Knot's `resident_app_route`
  test also calls it.
- `identity_projection.rs`'s header (DR-C, ruling D15) records the identity
  cards' return from castellan to Graphshell.
- `personal_sync.rs` is 3,520 lines (about 1,735 production); it imports
  `crate::access` and `crate::product` at lines 41-42. `carriage.rs` imports
  no port module.
- `projection_compile.rs` re-exports `scenomise::host_dataset::*` and
  `scenomise::projection::*`; `PRACTICE_CARD` and `default_definition` are
  host recipes. `projection_editor.rs` holds `chronicle_definition` (line 142).
  `scenograph`'s crate header says hosts own editor UI; it depends on
  `mere-curation`, `sceno`, serde and serde_json only; `scenomise` depends on
  `sceno` and `scenograph`.
- Knot's recipe editor is 5,591 lines over `scenograph::relationship`; its
  `Cargo.toml` takes `edit-history` from Mere's git. Woodshed's
  `relationship.rs` is 1,013.
- Pictograph already carries `canvas/input.rs` (wheel, zoom, pan, pointer),
  `canvas/node_presentation.rs` (the face-layer end of presentation rules),
  `canvas/forme_region.rs` and `canvas/dynamics_inspection.rs`.
- `crates/mere` depends on `forme`, `platen`, `pictograph`, `workbench`,
  `kernel` and others, but not on `muniment` or `edit-history`.
- `scripts/check_port_boundaries.py` checks three things (§3); it does not
  check dependency direction between ports, so Djinn depending on `graphshell`
  passes.
- Method for the import graph: a script reading each top-level module's
  `crate::` paths, collapsing `#[path]` test files into their parent,
  counting production and test lines separately. It is saved with this
  session's notes, not in the repository (DOC_POLICY §1: no file for a
  one-time analysis); rerun it at P0.
- Not verified: dependency cones were not measured (`cargo tree --offline`
  fails on git dependencies); no consumer repository was built; Turnstone's
  source is not local, so Turnstone-side receipts are named but their
  readiness is unchecked; the custody-scoping task was not found.

## Progress

- **2026-10-14:** plan written from the census; indexed in `DOC_README.md`.
  No code moved. Open: R1 to R6 (§5), P0 baselines, whether a custody-scoping
  task exists.
