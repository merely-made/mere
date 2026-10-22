# Graphshell stack lift plan

**Status (2026-10-15):** in progress. Castellan has moved (R8); P1 landed:
the app door is the `door` feature on `graphshell-endpoint` (L1), and
Signalman and `ports/dramatis` build without the `graphshell` package; P2a's
code landed: `mere-overmap` holds the remote overmap client (L14); its
Turnstone receipt and the Djinn route registration it needs are open, with
the transport design in the
[transport agnosticism note](../technical_architecture/2026-10-15_transport_agnosticism_note.md).
Nothing else has moved. The census below was read from the source at
`0aade8668` and corrects the first pass in several places (§2). Mark ruled R1
to R8 and the `Page` split on 2026-10-15 (§5).

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
| Turnstone (`f14d2091`, pins Mere `77fd235f`; read from GitHub) | Production source: `native::app_admission`, `native::app_client`, `native::custody_client`, `native::custody_identity`, `admission`, `carrier`, `network_carrier` (`dial_projection_session`), `native::endpoint_catalog`, `native::projection_host`, `sessions`, `view`, `identity::VaultLockView`. `place/projection_host.rs` (482 lines) serves projections with the serving stack. Its `graphshell::projection_editor` imports are the scenograph types the module re-exports (`SourceBinding`, `AuthoredProjectionDefinition` and others), not the reducer |
| Signalman, `ports/dramatis` | `native::custody_client`, `native::app_admission`, `native::quarantine`, `native::custody` |
| Djinn | a dozen `native::*` modules, `session_item`, `mere_route_session`, `mere_route::archive_intent`, `browser_carrier`, `identity*`, `lifecycle` |

So the demand is concentrated: six external or sibling consumers want a
few thousand lines of custody and application-door code, two of them
(Turnstone, and Knot in tests) also serve projections with the serving half,
and Djinn runs the server halves. Nobody outside the port imports the canvas reducers, the
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
| L8 | `coupling_edit` (102 code) | `mere` facade, `graph` feature (landed 2026-10-15; the draft touches no canvas, so pictograph was the wrong home) | Tests rewritten off `app` and `mere_host` | Woodshed or Turnstone edits one coupling's strength through the draft | Low / low |
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
| L1 | **App-facing half**: `native/app_admission` (586), `app_client` (452), `custody` (496), `custody_client` (558), `custody_identity` (449), `quarantine` (60), `local_endpoint` (277), `tasks` (103); the wire types in `app_broker` (`APP_CONNECT_SCHEMA`, `AppMessage`, `AppHostMessage`, `AppBrokerError`); the NDJSON frame functions in `browser_carrier` (`read_native_message_async`, `write_native_message_async`); `identity::*` re-exports from `dramatis::view`; `profile::env_profile` (240). About 3.6k lines | A `door` feature on `graphshell-endpoint` (ruled, R3) | Woodshed's `woodshed-genet` and Hocket, Knot's `startup.rs`, Signalman and `ports/dramatis` depend on it alone and build with no `graphshell` package in their cone. Woodshed narrowed `graphshell` to `default-features = false, features = ["native"]` on its main (`c11571b`, relayed by its session), which already drops the retained-content cone; the crate then replaces the `graphshell` dependency itself in its three manifests |
| L2 | **Serving half**: `admission` (372), `lifecycle` (718, with `AdmittedEndpointContext`, `BindAdmittedSession`, `SessionAuthority`), `native/endpoint_catalog` (635), `carrier` (694), `network_carrier` (271), `session_loop` (943), `session_notices` (398), `resume` (447), `live_endpoint` (738), `native/projection_host` (836), `native/resident_identity` (136, the `ResidentIdentity` trait), `policy_projection` (391); `webrtc_door` and `webrtc_session` follow or are feature-gated | `graphshell-host`, a new crate under `crates/graphshell/` beside `graphshell-endpoint` (ruled, R4); that crate's README already says admission and the serve loops "live in `ports/graphshell`" and names this as the not-yet-extracted half | Cleromancy registers an endpoint with `ResidentEndpointCatalog` and `AdmittedEndpointContext` without `graphshell`; Knot's `place_projection` test serves a projection with it; both build for their target features |
| L3 | **Server halves Djinn runs**: `native/app_broker` (522), `device_broker` (285), `browser_host` (668), `local_session` (243), `resident_mere_proxy` (218), `personal_sync_host` (2,319), `carriage_host` (564 + 290), `transfer_staging` (673), `graph_keys` (840) | `ports/djinn`, which is the only caller of `serve_app_broker` (three call sites) | Knot's `resident_app_route` test and the `graphshell_native_host` binary run against Djinn's broker or a small fixture host; `graphshell` no longer holds a broker |
| L13 | `personal_sync` (3,693), `receipts` (1,368), `transfer_offer` (363) | Waits. A stack home (Pandect or a dedicated crate) for the replica, event and projection types Cleromancy imports | The facet shapes in `access` and `product` that `personal_sync.rs:41-42` reads must be settled (see the [facet dissolution brief](../research/2026-10-08_facet_dissolution_brief.md)) | Cleromancy builds `PersonalGraphReplica` with no `graphshell` package |

`identity_endpoint`, `identity_projection`, `browser_carrier`'s browser
bridge, `webrtc_join`, `webrtc_browser`, `sessions`, `canary` and `view` stay
(finding 9; they are the generic adapter and the G-series receipts).

### Group 3b: remote overmap (requested by Mark, 2026-10-14)

Mark: "it would be nice to have a remote overmap capability, especially for
the consumers that would have the required dependencies anyway. like
turnstone probably." Relayed by the session that started this one; the pieces
below were read from source.

| ID | Modules (lines) | Home | Receipt |
|---|---|---|---|
| L14 | `mere_route` (452), `mere_route_session` (715), and the wire half of `session_item` (about 250 of 320 lines: the `djinn.mere/v1/*` route names, intent and schema constants, `SessionGraphV1`, `SetViewV1`, `ApplyEditsV1`, `SessionsActionV1`, `ArchiveActionV1`) | `mere-overmap`, a sibling crate to `mere-view` under `crates/cambium/` (ruled, R7), depending on `mere-view`, `graphshell-client` and `chirograph`; not a feature of `mere-view` | Turnstone's overmap pane mounts a remote mere's graph as `mere-view` over an admitted session and shows its sessions, with Turnstone's own identity and window. Single route only |

What source shows: `mere_route` fills `MereViewModel` from a route's two
projections (sessions and the attached graph) and turns `MereViewRequest`s
into route intents; `mere_route_session` is the event-driven panel over one
admitted endpoint on `graphshell_client::SessionDriver`. Their imports are
stack crates (`chirograph`, `mere::canvas::MERE_GRAPH_ADAPTER`, `mere_view`,
`pandect::SessionId`, `sceno`, `graphshell_client`, `uuid`) plus
`session_item`. `session_item` is the one tangle: its wire constants and
types are what the client needs, but it also holds server-side builders
(`card<B: Backend>(&GraphSession<B>)`, `actions()`), and `mere_host` imports
it (group 4). The wire half moves; the builders stay with group 4. The wire
types name `pandect::SessionId`, `pandect::ViewIntent`, and
`kernel::CapturedDelta`, so the new crate depends on `pandect` and `kernel`,
both stack crates.

Why a sibling and not a `mere-view` feature: `mere-view` depends on
`cambium`, `cartography`, `pictograph` (`signals`), `incipit`, `kernel`,
`sprigging` and `uuid`, and not on `chirograph` or `graphshell-client`. Its
header says every application embeds it, including hosts with a local mere
and no session. A feature would put the session client on every embedder's
feature list.

Why Turnstone: it already depends on `graphshell`, `graphshell-client`
(with `accesskit`), `graphshell-endpoint`, `notochord`, `personae`,
`transport` and `pandect`, and already dials projection sessions
(`network_carrier::dial_projection_session`) and opens the app door
(`app_client::AppRouteCarrier`). Its `src/overmap.rs` (189 lines) is local
only: it projects its own `ManifestStore` into a kernel `Graph`. The suite
census (§2) says Turnstone may mount granted projections through Graphshell.

What is not here: nothing composes several remote routes into one overmap
graph. `sessions.rs` is a switcher proof harness. Multi-route composition is
a design (identity of nodes across meres, whose authority a merged view
shows), not a lift, and no code moves for it. Not confirmed: whether a
mere route is registered on a network-carrier catalog. Source shows Djinn
registering `mere/<domain>` routes on the local application door
(`djinn/src/resident_mere.rs`) and the C4 fixture serving a divination route
through `native::resident_mere_proxy` over the WebRTC door
(`bin/c4_webrtc_host.rs`); `djinn/src/bin/djinn.rs` registers Knot, site,
device directory, status and reservoir routes in its shared catalog, and
this plan found no mere-route registration on a carrier a remote device can
dial. That registration is Djinn-side work that the receipt needs, and it is
a P5-adjacent step, not part of L14. The [transport agnosticism note](../technical_architecture/2026-10-15_transport_agnosticism_note.md)
designs it: besides the registration, a dialled session has no application
identity today, and `AccessEndpoint::open` refuses one without it.

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

## 5. Rulings

Mark ruled R1 to R8 and the `Page` split on 2026-10-15. The options and
evidence put to him are kept under each ruling.

- **R1, where whole-definition authoring state lives. Ruled: `scenomise`.**
  Mark: "it seems obvious this will end up in turnstone, knot, woodshed,
  anything with an engine and ambient data." The reducer lifts ahead of a
  second host; its first adopter (Knot's recipe editor or Turnstone) is the
  receipt. It cannot go in `scenograph`: the reducer compiles through
  `scenomise`, which depends on `scenograph`, so that would be a cycle.
  `workbench` (serde optional) and `edit-history` carry no runtime, so adding
  them does not break scenomise's no-runtime header. The reducer does carry
  state between calls, unlike the closed-form solvers, so it goes behind a
  feature (working name `authoring`), and solver-only consumers stay as they
  are. Options considered: keep it in Graphshell until a second host; a new
  crate under `crates/cambium/scenes/`.
- **R2, where Forme workspace and session live. Ruled: `platen`**, behind a
  feature, once the three canvas-touching pieces are cut. Platen already
  depends on `workbench`, which is a stack crate (split-and-tab workspace
  organization; Platen, Cambium, Pelt and the `mere` facade use it).
  `crates/forme/forme` is excluded by the cycle.
- **R3, the app-facing half (L1). Ruled: a `door` feature on
  `graphshell-endpoint`; no new crate.** Apps already depend on the
  endpoint crate, and its carriers are already features (folded in
  2026-09-24), so the default build stays wasm-clean. The dramatis repository
  cannot host it: L1 names `pandect` and `chirograph` types, and Mere depends
  on dramatis. `custodian` was considered and set aside: the family
  composition thesis brief already uses it for a delegated holder of bytes,
  and the door carries admission, route ids and quarantine as well as
  custody.
- **R4, the serving half (L2). Ruled: a new stack crate, `graphshell-host`,
  under `crates/graphshell/`.** Mark: "Clear boundaries and efficient use of
  components." Knot, Cleromancy and Turnstone serve through it without
  Djinn. Djinn keeps only the brokers (L3), and its `graphshell` dependency
  goes once they move. Djinn stays where it is (`ports/djinn`). Options considered: a `serve` feature on
  `graphshell-endpoint` (one crate spanning wasm-clean app and Tokio host
  sides); Djinn owning it.
- **R8, Castellan's place. Ruled: a component crate under Djinn,
  `ports/djinn/castellan`**, laid out like `ports/distillery/athanor`.
  Mark: "castellan does not need to keep credential manager screens when
  dramatis exists." Castellan has no screens. Its secret-free views moved to
  `dramatis::view` (dramatis repo plan D24), `dramatis-app` is the persona
  and quarantine UI, and `deny.toml` lets only Djinn link `castellan` (D4,
  D16). It stays its own crate, so the secret-custody code keeps its
  dependency boundary and the `deny.toml` wrapper check; only its location
  changes. Its `lib.rs` header still describes an "embeddable half" and is
  corrected with the move. This supersedes the 2026-08-22 census line giving
  Castellan the identity and credential product surface.
- **R5, `personal_sync` and the facets. Ruled: move, and adapt when the
  facet brief rules.** The [facet dissolution brief](../research/2026-10-08_facet_dissolution_brief.md)
  is still open (waiting on graph semantics' owner since 2026-10-08), and no
  node-facet change has landed since except float preservation in replay
  (`a1309431f`). Mark asked for a nudge to that owner if one could reach it.
  None can from this lane, because the owner is the ThinkPad Codex lane, so
  the nudge is left to Mark. L13 no longer waits on the brief.
- **R6, `appearance`. Stays** (recommendation stood; Mark asked what it is).
  It is Graphshell's preference record: which Tabard theme and mode
  Graphshell uses and how its visual roles map. Tabard owns theme
  definitions, validation and derivation; each application owns its own
  record, as Knot does for its font roles.
- **R7, the remote overmap crate (L14). Ruled: `mere-overmap`, under
  `crates/cambium/` beside `mere-view`.** Mark named it. The dependency set
  (`mere-view`, `graphshell-client`, `chirograph`, `pandect`, `kernel`) is
  the same in either location; it sits with the view it fills.
- **The `Page` split (P9). Ruled: views become components.** Each
  `web_tree` view becomes a Cambium component with narrow typed inputs and
  moves with the lift that owns its state (L5, L12, L11). No separate
  sub-state pass happens first. Options considered: sub-states inside the
  port first; leaving `Page` until a native host asks.

## 6. Phases

Ordered by value and risk. A phase that lands code ends with the second-host
receipt named for it; a receipt in this repository's tests does not stand in
for the other repository's build.

**P0. Rulings and ledger.** R3 and R4 are ruled (§5); the custody-scoping
task was proposed but never started, and L1 under R3 supersedes it. Remaining:
and record each phase's baseline: the `graphshell` dependency cone per
consumer (`cargo tree` on each external consumer, not measured here because
the git dependencies need network access), and the per-module `crate::`
import graph in §2 (method in Findings).
*Done when:* R3 and R4 are ruled or explicitly deferred; the baseline numbers
are in Findings; the custody task is referenced or recorded as absent.

**P1. L1, the `door` feature on `graphshell-endpoint`.** Highest value: five consumers outside the
port, and the most expensive current dependency. Split the wire types out of
`app_broker` and the framing out of `browser_carrier` first (the two real
tangles: `app_client` imports both), then move the files. Update Djinn,
Signalman, `ports/dramatis` and `graphshell_native_host`'s imports in the same
change. No re-export from `graphshell::native`.
*Done when:* `graphshell-endpoint` with `door` builds for native, the
default feature set stays wasm-clean, and neither pulls `graphshell`;
Signalman and `ports/dramatis` build against it alone; `custody_client` and
`app_client` tests pass in the crate; the boundary script passes; Woodshed's
(root, `ports/hocket`, `validation/relationship-recipe`) and Knot's next repin
name `graphshell-endpoint/door` in place of `graphshell`, recorded in their
plans.

**P2. L5, L8 and the `ProjectionClock` cut.** Canvas controls, faces and
gesture parity into pictograph; `coupling_edit`; move `ProjectionClock` out of
`product` so `canvas_physics` can follow. Woodshed's native surfaces are the
receipt.
*Done when:* pictograph holds the controls with `canvas::input` reconciled
(one wheel and zoom path); Graphshell's web page calls them from pictograph;
a Woodshed (or fixture) native surface drives them with only the `canvas`
feature; `web_tree/gestures.rs` and `controls.rs` call the pictograph functions.
*Landed 2026-10-15:* the controls, gestures and face alignment are
`pictograph::canvas::{controls, gestures, face_alignment}`; `ProjectionClock`
is `scenotime::ProjectionClock`; `StrengthDraft` is `mere::coupling_edit`
(Progress). *Receipt:* Woodshed, native, 2026-10-15 (Progress);
touch stays blocked on host input. `canvas_physics` and `canvas_reader` have not
moved (Progress, "Follows").

**P2a. L14, the remote overmap client.** Independent of P1 and P4: it needs
only `graphshell-client`, which is already a stack crate, and an admitted
session the host obtains itself. R7 ruled: `mere-overmap`. First split `session_item`'s wire
half from its server builders, then move `mere_route`, `mere_route_session`
and the wire half.
*Done when:* the new crate builds with no `graphshell` dependency and its
tests (today's `mere_route` and `mere_route_session` tests) pass there;
`graphshell_mere` and Djinn import it; Turnstone's overmap pane mounts a
remote mere through it, over a session Turnstone admitted itself, and shows
sessions and graph. Turnstone's own `overmap.rs` keeps the local case.
Registering a mere route on a carrier a remote device can dial is the Djinn
step the receipt needs, and is recorded here if it lands first.
*Landed 2026-10-15:* the crate, the split, the moved tests and both
importers (Progress). *Open:* the Turnstone receipt, which waits on the
transport note's rulings T4, T5 and T7 and a native stream-to-lines pump.

**P3. L4 and L4c.** Option helpers into `scenomise`; Knot's copies deleted;
host recipes go to their domains.
*Done when:* Knot's `recipe/editor.rs` uses the shared helpers and its tests
pass; Graphshell's editor uses the same; no `chronicle_definition` or
`default_definition` remains in a shared path.
*Landed 2026-10-15 on the Mere side:* `scenomise::arrangement` (always on),
`scenomise::authoring` (feature `authoring`, R1), and Chronicle's recipe in
`distillery::chronicle_recipe` (Progress). *Held:* Knot's side. Reading
Knot's `recipe/editor.rs` against the shared helpers showed they are siblings,
not copies (Progress), so "delete Knot's copies" is a copy decision for Knot's
owner, not a mechanical step.

**P4. L2, the serving half.** Builds on P1 (the stack crate takes
`AdmittedEndpointContext`, which `endpoint_catalog` already names). R4 ruled: `graphshell-host`. Needs
a decision on the WebRTC door's placement.
*Done when:* the crate is wasm32-clean for `admission`, `webrtc_door` and
`webrtc_join` and the web cone check passes; Cleromancy registers an endpoint
through it; Knot's `place_projection` test serves through it; the port keeps
only `identity_endpoint`, `identity_projection` and the generic adapters.
*Landed 2026-10-15 on the Mere side:* `crates/graphshell/graphshell-host`,
with the WebRTC door and join in the crate (default feature `webrtc-join`,
wasm-clean) and the live lane behind `webrtc-session`; the transport note's
shared admission entry and line codec came with it (Progress). *Kept in the
port:* `native::resident_identity`, whose trait names the browser carrier's
native-identity vocabulary and `identity_projection`'s unlock policy (Progress).
*Cleromancy consumer receipt (2026-10-15):* `9aae258` uses
`graphshell_host::endpoint_catalog::ResidentEndpointCatalog` and
`graphshell_host::lifecycle::AdmittedEndpointContext`; its A18 admitted-context
and A19 catalog-registration tests pass against locked Mere `59f88e63`.
See Progress for scope.

**P5. L3, Djinn takes the server halves.** After P4, because `app_broker` and
`browser_host` use `session_loop` and `session_notices`.
*Done when:* `graphshell` holds no broker; Djinn's tests and Knot's
`resident_app_route` pass; `graphshell_native_host` and the C4 host fixture
build against Djinn or a fixture host.
*Landed 2026-10-15 in Mere:* Djinn's `door` and `sync` modules, with
`ResidentIdentity` (Mark's ruling), the native host relay, the C4 fixture and
Knot's app-route receipt (Progress).

**P6. L12, forme.** R2 ruled: `platen`. Needs a second host (Turnstone's workbench or
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

**P9. The `web_tree` views.** Ruled: views become components (§5), each
moving with the lift that owns its state; start with the
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
4. Woodshed, Hocket and Redshank will repin to Mere main; the repin is
   proposed but not started. Woodshed main `c11571b` has already narrowed
   `graphshell` to `native` only in its root, `ports/hocket` and
   `validation/relationship-recipe`. The repin is the natural point to adopt
   P1 if P1 is on main by then. This plan targets main and does not touch
   Woodshed's manifests.

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
- Turnstone (`f14d2091`, read from GitHub, 185 `.rs` files under `src/`):
  production imports of `graphshell` are the L1 and L2 sets and
  `view::{ProjectionLayoutView, ProjectionReceiptView, render_projection_receipt}`.
  Its `graphshell::projection_editor` imports in `knot_authoring.rs` are
  `ProjectionInputBinding`, `PublicSourceRevision`, `RevisionEvidence`,
  `RuntimeSourceBinding`, `SourceBinding` and the `AuthoredProjectionDefinition`
  family, all scenograph types the module re-exports; none is the reducer, so
  finding 3 stands and Turnstone's side of L4b is a one-line repoint to
  `scenograph`. It pins Mere `77fd235f`.
- `session_item.rs` (320 lines) holds route names, intent and schema constants
  and wire types, and two server-side builders (`card`, `actions`);
  `mere_host` imports it. `mere-view` has no `chirograph` or
  `graphshell-client` dependency.
- Not verified: dependency cones were not measured (`cargo tree --offline`
  fails on git dependencies); no consumer repository was built; Turnstone's
  receipts (L12, L14) are named but its readiness to run them is unchecked,
  though its imports were read; the custody-scoping task was not found.

## Progress

- **2026-10-14:** plan written from the census; indexed in `DOC_README.md`.
  No code moved. Open: R1 to R7 (§5), P0 baselines, whether a custody-scoping
  task exists.
- **2026-10-14 (later):** Mark's remote-overmap request added as L14, R7 and
  P2a; Woodshed's interim narrowing (`c11571b`) recorded; Turnstone read from
  GitHub, adding it as a production consumer of L1 and L2 and confirming it is
  not a reducer consumer of the projection editor.
- **2026-10-15:** Mark ruled R1 to R7 and the `Page` split (§5): the
  authoring reducer goes to `scenomise` behind a feature; Forme goes to
  `platen`; the app door becomes a `door` feature on `graphshell-endpoint`;
  the serving half becomes `graphshell-host`; `personal_sync` moves and
  adapts later; `appearance` stays; the remote overmap is `mere-overmap`
  beside `mere-view`; `web_tree` views become components. The custody-scoping
  task is withdrawn as superseded by L1.
- **2026-10-15 (later):** R8: Castellan moves to `ports/djinn/castellan` as
  a component crate. The R4 note that Castellan keeps a credential UI is
  withdrawn; Castellan has none.
- **2026-10-15 (R8 done):** Castellan moved from its own port directory to
  `ports/djinn/castellan`; the package stays `castellan`. The workspace
  member and dependency path, `.gitattributes`, the install scripts' repo
  root, the DR-C and Dramatis review receipt scripts, and the live docs that
  cited the old path followed. Its `lib.rs` header, README and manifest
  description now say it is Djinn's secret-custody component with no
  screens: the views are `dramatis::view` (D24), the UI is `dramatis-app`.
  The `deny.toml` wrapper rule keys on the crate name and still holds.
- **2026-10-15 (P2a, L14):** `crates/cambium/mere-overmap` landed. `wire`
  holds `session_item`'s wire half (route names, intents, schemas,
  `SessionGraphV1`, `SetViewV1`, `ApplyEditsV1`, `StepV1`, `SessionsActionV1`,
  `ArchiveActionV1`, `archive_action`, and `advertised`, the action shape the
  builders share); `route` and `route_session` are `mere_route` and
  `mere_route_session`, behind the default `view` feature. Without `view` the
  crate is the wire alone (chirograph, kernel, pandect, scenotime, serde), so
  Graphshell's `web` feature takes it without the view's Cambium cone; with
  it, it adds `mere-view`, `graphshell-client`, `cartography` (the graph
  adapter id, which `mere::canvas` re-exported), `sceno` and `uuid`.
  `session_item` keeps `card`, `actions` and its docs, for `mere_host`.
  Graphshell's `mere-route` feature is removed: `mere-view-host` names
  `mere-overmap/view`, `graphshell-web`'s `remote` takes `mere-overmap`
  directly, and Djinn's `custody-receipt-host` needs nothing extra. One change
  beyond the move: `MereRouteSession::from_descriptor` takes the application
  it reads as, because the panel had `graphshell` and `graphshell.mere`
  hard-coded and a second host must read its own view state; Graphshell's
  callers pass `"graphshell"`. Checks: `mere-overmap` builds with and without
  `view` and has no `graphshell` in its tree; its two `route_session` tests
  pass; Djinn's `resident_mere` (16), `embedded_reservoir_validation`,
  `embedded_reservoir_two_process` and `mere_two_process` pass;
  `graphshell` `mere_host` tests (12) pass; `graphshell_mere` and
  `graphshell-web` (wasm32) build; the boundary script passes.
  `mere_two_process` fails on macOS when `TMPDIR` makes its socket path longer
  than 104 bytes (it passes with a short `TMPDIR`); that predates this change.
  Open: the Turnstone receipt and the Djinn registration (P2a).
- **2026-10-18 (`graphshell_mere`, its own package):** the native viewer
  imported only stack crates, but as a `graphshell` binary it still built the
  whole Graphshell library with `native`, `web` and the package defaults,
  whose features reached crates it does use (ICU's dictionaries got into it
  that way). It is now `ports/graphshell/mere` (package `graphshell-mere`,
  binary still `graphshell_mere`), depending on exactly what it uses; its
  graph drops from about 1,032 packages to 376 (no document canvas, lanes,
  Pelt, iroh, reqwest, ring, rustls or ssh-key). Graphshell's `mere-view-host`
  feature and its `mere-view`, `mesquite`, `sprigging` and `taproot` rows are
  removed. Build it with `cargo build -p graphshell-mere`; its 15 tests pass.
- **2026-10-15 (P1, L1 landed):** the app door is
  `graphshell_endpoint::door`, feature `door` (native only; the module is
  compiled out on wasm32). Moved from `ports/graphshell`: `native/`
  `app_admission`, `app_client`, `custody`, `custody_client`,
  `custody_identity`, `quarantine`, `local_endpoint` and `tasks`; `identity`
  (the `dramatis::view` re-export) and `profile` (`GraphshellIdentity`,
  `env_profile`, `session_salt`). Old path `graphshell::native::X` (or
  `graphshell::identity`, `graphshell::profile`) is now
  `graphshell_endpoint::door::X`. The two tangles were split first:
  - `door::wire` holds `APP_CONNECT_SCHEMA`, `AppMessage`, `AppHostMessage`
    and the host challenge (`BrowserChallenge`, which `AppHostMessage`
    carries; the name and its schema string are unchanged, so both doors'
    wire is byte-identical). `browser_carrier` keeps its own nonce check.
  - `door::frame` holds the frame functions (sync and async) and
    `MAX_NATIVE_MESSAGE_BYTES`, with their own `FrameError`.
    `browser_carrier`'s four frame functions are now one-line wrappers that
    keep `BrowserCarrierError`. The frames are length-prefixed
    native-messaging JSON, not NDJSON as §4 says.
  - `door::route` holds `ResidentEndpointRoute` and its error, moved out of
    `native::endpoint_catalog`, because `app_admission`'s grants map to it.
  - `AppBrokerError` did **not** move: it wraps server errors
    (`BrowserHostError`, `ResidentEndpointCatalogError`) and stays with
    `app_broker`. `AppClientError::Broker`, never constructed, is removed;
    `AppClientError::Carrier(BrowserCarrierError)` is now
    `AppClientError::Frame(FrameError)`.
  - The app-endpoint-differs-from-browser-endpoint test moved to
    `native::device_broker`, since it compares the two doors.

  Djinn (`graphshell-endpoint` with `door`), Signalman and `ports/dramatis`
  (both now `graphshell-endpoint` with `door` in place of `graphshell`) and
  Graphshell's binaries import the new paths; `scripts/dr_c_receipts.py`
  hashes the moved files at their new paths. `cargo tree` shows no
  `graphshell` package for `graphshell-endpoint` (default or `door`),
  `mere-signalman` or `dramatis-app`.

  **One transitional re-export remains**, against the plan's "no re-export
  from `graphshell::native`": `native/mod.rs` re-exports `app_admission`,
  `custody` and `custody_client` from the door, `#[doc(hidden)]`, because
  Djinn depends on `knot-editor` at `8ed49d4`, whose `src/startup.rs` imports
  those three paths and resolves `graphshell` to this port through the
  workspace patch. Without it Djinn, and so the workspace, does not build.
  Nothing in this repository uses it. It goes when Djinn's Knot pin moves.

  **What the external repins need** (out of scope here):
  - *Knot* (`knot-editor`): `startup.rs`, `bin/knot_sync_host.rs`,
    `tests/resident_ownership.rs` and `tests/resident_app_route.rs` import
    `graphshell::native::{app_admission, app_client, custody,
    custody_client}`, `graphshell::identity` and
    `graphshell::native::endpoint_catalog::ResidentEndpointRoute`; each
    becomes `graphshell_endpoint::door::...` (`ResidentEndpointRoute` is
    `door::route`). Its `graphshell-endpoint` dependency gains `door`; the
    library then needs `graphshell` only for its tests and examples
    (`place_projection`, `revision_bell`, `rosette_projection`, `k2_peer`,
    and `resident_app_route`'s `serve_app_broker`, which is L2 and L3). Then
    Mere repins `knot-editor` and deletes the re-export above.
  - *Woodshed* (root, `ports/hocket`, `validation/relationship-recipe`): the
    same path change; `graphshell = { default-features = false, features =
    ["native"] }` becomes `graphshell-endpoint = { features = ["door"] }`
    wherever only L1 is named.
  - *Turnstone*: its L1 imports take the same rename; its
    `view::ProjectionReceiptView` imports still need `graphshell` until L2.
  - Any `AppClientError::Carrier` match becomes `AppClientError::Frame`.

  Checks: `graphshell-endpoint --features door` builds and its 41 tests pass
  (the `custody_client` and `app_client` ones among them); the default
  feature set checks for wasm32; `cargo test` on `graphshell-endpoint`,
  `graphshell`, `djinn`, `mere-signalman`, `dramatis-app` and `mere-overmap`
  passes (840 passed, 22 ignored) with a short `TMPDIR`, since Djinn's socket
  receipts exceed macOS's 104-byte socket path under the default one (as the
  P2a entry notes); `cargo build --workspace` and the boundary script pass.
  `cargo check --workspace --all-targets` fails only where this change does
  not reach: `tabard-desktop`'s `native_shared_recovery` example on main, and
  `mere-mesh` examples and Graphshell's `practice_*` tests that `include!`
  sibling checkouts (`murm`, `woodshed`) by a path a worktree does not have.
- **2026-10-15 (P1 closed, shim gone):** Knot repinned (knot-editor
  `a27911ad`, on Mere `19573b51`): its library imports custody, admission and
  the identity views from `graphshell_endpoint::door`, and `graphshell` is a
  Knot dev-dependency only. Mere's workspace and Djinn's `knot-site` move to
  that revision, and the transitional `graphshell::native` re-export is
  deleted, so P1 now holds without exception. Knot's repin also carried a
  `mere-view` change (archive controls) under Mark's ruling "catalog, route as
  overmap": Knot's catalog stays the graph, and knot-editor's new
  `mere_overlay` (feature `mere-overlay`) reads a resident mere's `mere/<domain>`
  route as `knot-editor` through the door, using `mere-overmap`'s mapping, for
  the sessions and archive above it (knot-editor
  `design_docs/2026-10-15_knot_mere_overmap_layer.md`). Mere's patch tables gain
  `mere-overmap`, which Knot now names, and Djinn's tests take knot-editor
  with `mere-overlay`. New receipt: `resident_mere`
  `knot_overlay_saves_and_opens_an_archive_through_the_door` (Save, read back,
  Open, unadvertised Compose declined). Checks: `TMPDIR=/tmp cargo test` on
  `graphshell-endpoint` (`door`), `graphshell`, `djinn`, `mere-signalman`,
  `dramatis-app` and `mere-overmap` passes (841 passed, 22 ignored);
  `cargo build --workspace` and the boundary script pass; `cargo check
  --workspace --all-targets` fails only at the pre-existing `tabard-desktop`
  example and the `mere-mesh` examples noted above. Still open: Woodshed's and
  Turnstone's repins (the P1 entry lists what they need).
- **2026-10-15 (P2, L5 and L8 landed):** the first three came out of the
  port as thin policy over the canvas's public API, so there was no second
  wheel or zoom path to reconcile with `canvas/input.rs`: they call
  `Canvas::wheel`, `cursor_moved`, `set_ctrl` and `set_camera`, and now sit
  beside it.
  - `pictograph::canvas::controls` (`CanvasCommand`), `gestures` (`WheelMode`,
    `WheelUse`, `wheel_use`, `wheel_zoom`, `TouchUse`, `touch_use`,
    `two_finger`, `pan_at_once`, `PAN_GLIDE`) and `face_alignment`
    (`FaceAlignment`). Their eight tests moved with them and pass under
    pictograph's `canvas` feature alone, which is what Woodshed enables.
  - `scenotime::ProjectionClock`, out of `product.rs`: the host's elapsed
    clock for a transition, which `scenotime::transition`'s header already
    says the host owns. Two new unit tests (pause not charged, time never runs
    backward). This cuts the one production import `canvas_physics` had of
    `product`.
  - `mere::coupling_edit::StrengthDraft`, under the `graph` feature: it reads
    only `kernel::graph`, so pictograph was the wrong home. `graph_id` is now a
    public field; the journal channel string (`graphshell.coupling/<id>`) is
    Graphshell's and moved to `mere_host_coupling.rs` with the host tests
    (`coupling_edit_tests.rs` became `mere_host_coupling_tests.rs`), since
    every one of them drives `GraphshellApp`. Four new kernel-only unit tests
    cover the draft itself.
  - Graphshell: the three modules and `coupling_edit` are deleted with no
    re-export; `canvas_physics`, `viewer_commands`, `web_tree`, `web_tree/gestures`,
    `web_tree/lane`, `web_tree/coupling` and `web_tree/product` import the new
    paths. Checks: `scenotime` and the pictograph and `mere` tests above pass;
    `TMPDIR=/tmp cargo test -p graphshell --lib` is 475 passed, 0 failed, 6
    ignored; `cargo check` of `graphshell-web` for wasm32 passes; the boundary
    script passes. Not run: the integration tests under `ports/graphshell/tests`
    and Djinn's, which do not import these modules.
  - Follows: `canvas_physics` (production part about 490 lines) can now leave
    `product` behind, but it also names `scenotime::TransitionSchedule`,
    `SceneOp` and `SceneSnapshot` ten times, and pictograph does not depend on
    `scenotime`; moving it means pictograph gains that dependency (`sceno` and
    serde only), which this pass did not take. `canvas_reader`'s production
    code is clean and can move on its own. `viewer_commands` (L6) and
    `frame_budget` (L7) still wait for a second host. The receipt is open: a
    Woodshed native surface driving the controls with only `canvas` enabled,
    after its repin onto a Mere revision that has them.
- **2026-10-15 (P3, Mere side, L4 and L4b landed; Knot held):**
  - `scenomise::arrangement` holds the five option helpers
    (`arrangement_options`, `arrangement_defaults`, `option_placeholder`,
    `with_option`, `with_kind`), always compiled, with their four tests.
  - `scenomise::authoring`, behind the new `authoring` feature (optional
    `workbench` and `edit-history`), holds the reducer: `ProjectionEditor`,
    `EditorAction`, `ReduceResult`, `ProjectionPanel`, `COALESCE_WINDOW_MS`,
    `ProjectionDefinitionSink`, `SaveError`, and the dotted-name `edit_field`
    adapter, with its sixteen tests. Two deliberate changes: `edit_field` takes
    the host's `&SolverRegistry` (it had reached for Graphshell's own
    `practice_compiler`, which a stack crate cannot name), and the panel tile
    kind is the public `PROJECTION_EDITOR_PANEL_KIND`, now
    `scenomise.projection-editor.panel` (it was a private
    `graphshell.projection-editor.panel`; nothing else in the tree, scenarios
    or receipts names it, and the workbench layout is not persisted). The module
    re-exports nothing from `scenograph`.
  - `distillery::chronicle_recipe` holds `chronicle_definition`,
    `chronicle_era_bands_off_variant` and their constants and three tests;
    `ports/djinn`'s chronicle receipt host imports it from there.
  - `ports/graphshell`: `projection_editor.rs` is deleted with no re-export.
    `initial_projection_draft` (the reference host's starting draft, with its
    "phyllotaxis.default" recipe) moved to `projection_compile.rs` beside
    `default_definition`; the other importers (`local_projection`,
    `host_dataset_view`, `projection_compare`, `projection_dynamics_*`,
    `practice_disclosure`, `web_practice`, `web_tree/product/projection*`, the
    `projection_compile_perf` example, `tests/local_projection.rs`) import
    `scenograph`, `scenomise::arrangement` and `scenomise::authoring` directly.
    `graphshell-web`'s manifest names `scenograph` and `scenomise` (with
    `authoring`). `default_definition` (the Woodshed-flavoured practice recipe)
    stays in `projection_compile.rs`: moving it to Woodshed needs Woodshed to
    publish the recipe with its dataset, which this pass did not ask of it.
  - Checks: `graphshell` lib 452 passed (23 fewer than before: the moved
    tests), `local_projection` and `projection_round_trip` pass,
    `scenomise` 187 passed with `authoring` and 171 without, `distillery` lib 27
    passed, `mere` lib 53 passed, the `projection_compile_perf` example and the
    Djinn chronicle receipt host build, `graphshell-web` checks for wasm32, the
    boundary script passes.
  - Knot's helpers are not copies of the shared ones. Its `option_default`
    says "automatic" for an automatic default and nothing for an empty one
    where `option_placeholder` says "auto" and "none", and writes
    `{number} ({words})` differently; `default_description` has no
    counterpart; `declared_options` over `Family` and `Measure` overlaps
    `arrangement_options` plus `arrangement_defaults`. Adopting the shared
    ones changes Knot's copy, so Knot's owner decides it. The earlier claim
    that Knot duplicates the helpers (finding 3, L4's receipt) was too strong.
  - Open for Mark: `scenomise` is published (0.0.4) and its `authoring` feature
    names `edit-history`, which is `publish = false` ("publishing is a separate
    decision" in its manifest). The next `scenomise` publish needs `edit-history`
    published, or the feature dropped from the published manifest, the way
    `mere-curation` was published for forme (SE78).
  - Turnstone, read from GitHub earlier, imports scenograph types through
    `graphshell::projection_editor` in `knot_authoring.rs`; at its repin those
    lines become `scenograph::...`. No other repository on this machine names
    the module.
- **2026-10-15 (P2 and P3 on main; receipts and Knot):** P2 and the Mere side
  of P3 are in Mere `e92572a4e` on `origin/main`. Woodshed (Hocket Looper
  integration session) reports it has no code using `CanvasCommand`, the
  gesture helpers or `FaceAlignment`: its Session Mere overview drives
  pictograph's canvas directly, so the P2 receipt is new adoption work in
  `woodshed-views`, plus a repin of its roots from `90ce8f12d`, plus a native
  foreground run. That is a scope change for Woodshed and is with Mark; until
  then Woodshed is the planned second consumer without a receipt. Knot was
  repinned locally to `e92572a4e` (knot-editor `c487a7c`, rows only, no source
  change): `cargo check --workspace --all-targets` and the document, editor
  (`mere-overlay`), desktop and site suites pass (knot-site's export tests
  refuse a temp directory reached through a symlink, so run them with a
  symlink-free `TMPDIR`; macOS's `/tmp` and `/var` are symlinks). Knot's option
  helpers were left alone (see the P3 entry). Not done: pushing Knot, the
  Mere-side `knot-editor`, `knot-document` and `knot-site` pin bump, and
  Turnstone's move to the same Mere and Knot revisions (which also needs its
  `graphshell::projection_editor` imports changed to `scenograph`).
- **2026-10-15 (Mark's rulings on P2's receipt and option wording; option
  helpers return values):** Mark ruled that Woodshed adopts the P2 controls as
  that phase's receipt (the Hocket Looper session has been told by the Knot
  adoption session), and that option labels are per host. So
  `scenomise::arrangement::option_placeholder`, which returned Graphshell's
  words, is replaced by `option_fallback` returning `OptionFallback`
  (`Value`, `Measured { resolved, words }`, `Auto { resolved }`, `Empty`): the
  facts a label is made from. Graphshell's words ("auto", "none", "n (words)")
  moved to `graphshell::projection_compile::option_placeholder` with the old
  test; Knot's own labels ("automatic", the empty label, "Default: ...") become
  a mapping over the same enum, and its `declared_options` becomes
  `arrangement_options` plus `arrangement_defaults`. Both hosts show the same
  text as before: the catalog resolves every option for a built-in family
  (`Auto` resolves to `auto`, `Empty` to the empty string), which is why the
  enum carries `resolved` for `Auto` as well as `Measured`.
- **2026-10-15 (P2, generic over the host, at Woodshed's request; Mark chose
  the route):** Woodshed's three hosting surfaces (Session Mere overview,
  Related, relationship recipe editor) render through `cambium::graph_canvas`
  with a `GraphViewport`, not a `Canvas`, so the lifted helpers could not serve
  them. `pictograph::canvas::camera_host` adds three small traits, portable and
  with no runtime: `CameraHost` (`camera`, `set_camera`, `zoom_range`,
  `set_view_follow`, plus `wheel_zoom_px` and `pan_step` with generic defaults),
  `FitHost: CameraHost` (the fit and physics verbs, each defaulting to nothing)
  and `FaceGeometry` (`zoom`, `faces`). `Canvas` implements all three, so
  existing callers change nothing. `CanvasCommand::apply`, `wheel_zoom`,
  `two_finger`, `pan_at_once` and `FaceAlignment::measure` are generic over
  them. `Canvas` keeps its own paths for the two behaviors that differ from the
  generic arithmetic: it zooms through `cursor_moved`, `set_ctrl` and `wheel`
  (projection-aware, the canvas's per-notch factor) and it pans as an impulse
  into its inertia, so Knot and Graphshell behave as before. A host that takes
  the defaults zooms by the same per-notch factor, keeps the world point under
  the pointer and clamps to its `zoom_range`, and pans at once by the distance
  the canvas's glide covers (`PAN_GLIDE`). Tests: a plain host agrees with the
  canvas on wheel zoom and two-finger gestures, stays in its range, reaches
  commands through its own verbs (absent verbs do nothing) and feeds
  `FaceAlignment`. A trait defined here cannot be implemented for a foreign
  camera type, so Woodshed wraps its `GraphViewport` in a newtype per surface.
  Checks: pictograph (`canvas`) lib 435 passed, 1 failed, 18 ignored; the one
  failure, `arrangement_goldens::every_arrangement_matches_its_golden_on_both_fixtures`
  ("topic radial weighted focus 0 moved from its golden"), fails the same way at
  `e92572a4e` without these changes (run in a clean worktree), so it predates
  them and is not touched here. `graphshell` lib 453 passed; `graphshell-web`
  checks for wasm32 with default features and with none; pictograph's `canvas`
  checks for wasm32; the boundary script passes. Graphshell's page code now
  dereferences its `RefMut<Canvas>` (`&mut *canvas`) at ten call sites, because
  generic parameters do not auto-deref the way a `&mut Canvas` parameter did.
- **2026-10-15 (pins synced after P2 and P3):** Mere `8357b00d0` moves the
  `knot-editor`, `knot-document` and `knot-site` pins to Knot `c487a7c`
  (Knot's repin to Mere `e92572a4e`). Turnstone `c191a56` is on Knot `c487a7c`
  and Mere `e92572a4e`, so it resolves one Mere source; its `projection_editor`
  imports in `knot_authoring.rs` now come from `scenograph`, and its 940 tests
  pass (reported by the Knot adoption session, not rerun here). Woodshed is on
  Mere `8357b00d0` (woodshed `25acaa0`).
- **2026-10-15 (P3, Knot side landed; helpers on main):** Mere `a26ed1caf` on
  `origin/main` carries `282413cd0` (option values), `2cc10a674` (the P2
  helpers generic over the host) and the pin-sync note. Knot `601f99b` (on
  `origin/main`) repins Mere to `a26ed1caf` and takes the arrangement option
  facts from `scenomise::arrangement` (`option_fallback`, `arrangement_options`,
  `arrangement_defaults`): `recipe/editor.rs` no longer has its own
  `option_default`, `declared_options` or `default_description` logic, and keeps
  only its words ("automatic", the empty label, "Default: ...", "Measured
  default: ..."). Its rows show the same text as before. Checks: workspace
  `cargo check --all-targets`; knot-document, knot-editor (`mere-overlay`),
  knot-desktop (lib, bins, `scenario_lane`, `font_roles`, `typography_fonts`)
  and knot-site pass, 624 tests, with a symlink-free `TMPDIR`. Graphshell
  already used the shared helpers through its own labels
  (`projection_compile::option_placeholder`). P3 is done on both sides. Still
  open: the P2 receipt (Woodshed adopting `CameraHost`, `FitHost` and
  `FaceGeometry` on its three surfaces, after its repin to a revision with them);
  `canvas_physics` and `canvas_reader` have not moved; Knot's pin bumps now
  belong to the overmap agent.
- **2026-10-15 (the arrangement goldens):** the golden failure noted under P2
  was never a regression: aarch64 macOS prints the same 22 hashes at
  `5d04c2d60` (where the goldens arrived), `e5a24a4af`, `768a2a94f` and
  `a26ed1caf`, and they match neither the historical table nor `768a2a94f`'s
  Linux one. The Spiral's and Radial's angles went through the platform's
  `sinf` and `cosf`. scenomise's solvers (and the spectral producer's start
  vector) now take their transcendentals from `libm`, so the positions have
  the same bits on every target, and one table replaces the two. Against the
  platform maths, 71 of 500 Spiral positions and 8 of 500 Radial ones move,
  by at most two ulps (0.0002 px).
- **2026-10-15 (P2, Mark's rulings: the wheel reaches graph_canvas consumers;
  touch is deferred):**
  - `cambium::graph_canvas` turned a wheel notch into
    `GraphCanvasEvent::Zoom { factor: 1.2 or 1/1.2 }`, which drops the
    pointer position, the pixel delta, the second axis and the modifiers, so no
    graph-canvas host could apply `wheel_use` or the anchored `wheel_zoom`.
    `graph_canvas_with_wheel` and `GraphCanvasInput` (`Event(GraphCanvasEvent)`
    or `Wheel(WheelEvent)`) now carry the whole notch to a consumer that opts
    in; the consumer calls `WheelEvent::prevent_default` when its policy takes
    the notch, and a notch it does not take scrolls the host as before. It is a
    separate type, as `GraphAtlasEvent` is, because Knot's recipe scene and
    Woodshed's stage match `GraphCanvasEvent` exhaustively, so a new variant (or
    new fields on `Zoom`) would have broken both. `graph_canvas` itself is
    unchanged in behavior: a fixed zoom factor, the host scroll default
    suppressed, a zero notch ignored; both entry points share one private core.
    Two new tests (the opt-in sees both axes, `local`, `size` and the modifiers
    and decides the default; the established zoom is unchanged); cambium lib 271
    passed; `mere-view`, `graphshell` and `cambium-genet-winit-host` build;
    `graphshell-web` checks for wasm32.
  - **Touch is blocked on host input.** Neither Cambium nor the Genet desktop
    host delivers touch or multi-finger events, so `touch_use` and `two_finger`
    have no native input in Woodshed. Mark chose to defer touch rather than wire
    it to synthetic events. The helpers stay generic and tested; the receipt for
    them waits for host touch input.
- **2026-10-15 (P2 receipt support: wheel input drivable in native scenarios):**
  Woodshed now takes the whole notch through `graph_canvas_with_wheel`, but the
  native host's scripted input had no wheel, so a wheel could only be checked by
  hand. `cambium_rootstock::HostPointer::Wheel { x, y, dx, dy, unit, modifiers }`
  (with `WheelUnit::{Pixels, Lines}` and `HOST_WHEEL_LINE_PX`, 48, a restatement
  of `genet_winit_host::WHEEL_LINE_PX` that a test holds equal) is drained by
  `Host::drain_pointer` through `Host::wheel`, so the nearest `on_wheel`
  ancestor of the hit node gets a genuine `WheelEvent` and what no handler takes
  falls to the host's scroll and Ctrl-zoom defaults. The modifiers are held for
  that notch only. Mesquite's `Probe::app_step` gains the verb
  `wheel <dx> <dy> [lines|px] [shift] [ctrl] [alt] [meta] (at <x> <y> | on <selector>)`;
  `dy > 0` advances toward the end of the content, as the host's `wheel` takes
  it, and `on` uses `click`'s selector grammar. Taproot parses `click`'s selector
  privately and unknown verbs pass to `app_step` as raw lines, so `mesquite::wheel`
  restates that grammar (five parse tests hold it). End to end, headless, in
  `cambium-genet-winit-host/tests/wheel_scenario.rs`: a `graph_canvas_with_wheel`
  view receives the notch with both axes, the cursor-local position at the
  swatch centre, and the modifiers; lines convert to 48 px each; modifiers do not
  leak into the next notch; Shift turns a vertical notch sideways as a real wheel
  does; a point off the canvas reaches nobody; a missing target or a typo fails
  the receipt. Checks: mesquite 34 passed, rootstock lib 103, the winit-host
  `wheel_scenario`, `scenario` and `input_routing` suites pass, `graphshell`,
  `mere-view`, `mere-overmap` and `djinn` build with all targets, `graphshell-web`
  checks for wasm32, the boundary script passes. Touch remains deferred.
- **2026-10-15 (P2's second-host receipt: Woodshed, native):** Woodshed `4005c5a`
  (checked in its repository: pinned to Mere `39c3c955d`, which holds
  `a26ed1caf`, `1b4172b25` and `bdef17239`; its receipt reads `RESULT ok` over 241
  frames). It runs the lifted helpers on three surfaces that render through
  `cambium::graph_canvas` over a `GraphViewport`, not a `Canvas`:
  `woodshed-views::stage::canvas_camera`'s `SurfaceCamera` implements
  `CameraHost` and `FitHost` over the viewport for the Session Mere overview,
  Related and the relationship editor. `CanvasCommand` drives the camera buttons
  and the motion toggle; `wheel_use` and the anchored `wheel_zoom` take the notch
  through `graph_canvas_with_wheel` (and through the lower-level builder's wheel
  hook on Related); `FaceAlignment` runs through a `FaceGeometry` adapter that
  compares view-layer hit targets with the bodies of Sprigging's painted leaf.
  Evidence: 8 adapter tests on the CPU (a cross-surface command and face test, a
  wheel-anchor test); native `validation/canvas-p2_20261015/receipt.json` with 8
  distinct captures, buttons clicked through real host hit-testing and scripted
  wheel notches, plain and Ctrl, on the relationship editor and Related; zoom,
  face-missing and face-offset asserted per surface plus eleven paint-current
  assertions. Findings it carries: (1) `FaceAlignment` cannot see stale paint,
  because it measures a freshly painted leaf, not the host's registered one;
  Woodshed's relationship and Related leaves did not repaint on camera-only
  changes, so hit targets moved while the paint stayed, and it now asserts the
  painted camera against the live one, which another host should also do; (2) the
  overview's wheel was not driven natively (its canvas is below the fold at the
  receipt window size), so it has a CPU test only; (3) touch stays deferred. P2's
  done-condition (a second host's native surface drives the controls with only
  `canvas` enabled) holds, through the host traits and not through a `Canvas`.
- **2026-10-15 (the `edit-history` question ruled: renamed `backstep`):** Mark
  renamed the crate to `backstep` (free on crates.io) rather than folding it into
  Cambium (the scene crates would then depend on the UI toolkit) or into
  scenotime (the wrong layer for a text field). It stays standalone and
  `publish = false`, now publishable: a description, license, repository and no
  path-only dependencies (`cargo package --list` accepts it; `cargo publish
  --dry-run` refuses only because of `publish = false`). Renamed: the directory
  (`crates/cambium/backstep`), the package and `[lib]` name, the workspace member,
  dependency and two patch rows, and every dependent: `cambium` (still
  re-exporting `History`; `EditHistory` is unchanged), `scenotime` (its test
  dev-dependency, now with a version), `scenomise` (`authoring`), `graphshell`, and
  the six receipt harness manifests that build live under
  `ports/graphshell/docs/receipts/*/harness`. Recorded evidence (receipt logs and
  JSON, the dated plan sections) keeps the old name. TERMINOLOGY gains **backstep**;
  `crates/cambium/CHANGELOG.md` notes the rename; the scenograph editor plan's SE13
  and the two path citations were updated. This closes the open `scenomise`
  publish question above: its `authoring` feature no longer names an unpublishable
  crate, though `workbench` and `backstep` still have to be published before
  `scenomise` can be. Knot and Turnstone follow at their next repin through Mere.
  Checks: `backstep` 8 tests, `scenotime`, `scenomise` (`authoring`) and cambium's
  editor tests pass; `graphshell`, `cambium`, `mere-view` and `djinn` build with all
  targets; `graphshell-web` checks for wasm32; the boundary script passes.
- **2026-10-15 (P4, L2 landed in Mere):** the serving half is
  `crates/graphshell/graphshell-host` (R4), with no re-export left in the port.
  Old path `graphshell::X` is `graphshell_host::X` for `admission`,
  `lifecycle`, `carrier`, `network_carrier`, `session_loop`,
  `session_notices`, `resume`, `live_endpoint`, `policy_projection`,
  `webrtc_door`, `webrtc_join` and `webrtc_session`; `graphshell::native::
  endpoint_catalog` and `native::projection_host` are
  `graphshell_host::endpoint_catalog` and `graphshell_host::projection_host`.
  - Features: none (the `resume` and `live_endpoint` fixture endpoints), then
    `admission`, `webrtc-join` (default: the door's sans-I/O core and the join,
    the WebRTC decision P4 asked for), `native` (accept, the line codec,
    lifecycle, the loop and notices, the catalog, the projection host, dialling,
    the policy file, the door's host halves) and `webrtc-session` (str0m). The
    default build, `admission` alone and no features check for wasm32;
    `native` is native only. The port names `graphshell-host` with default
    features off and forwards `native`, `webrtc-join` and `webrtc-session`, so
    the read-only viewer's cone gains the crate and nothing under it.
  - Transport note recommendation 3, one admission entry:
    `carrier::admit_stream(stream, facts, policy, ledger, now, active)` is
    what every stream-carried admission runs, ending in
    `admission::serves_action`. `admit_accepted_session` is
    `AcceptedSession::into_session` plus that entry; the port's local doors
    (`browser_carrier::admit_local_session`, which the app door's
    `admit_local_client` also takes) state their own facts and call it, so the
    in-process duplex now gets the action check it lacked; the WebRTC door's
    message path keeps the sans-I/O twin, `admission::admit_session`.
    `mere-transport` has no `TransportKind` for an in-process door, so the
    local doors build `SessionFacts` directly rather than an `AcceptedSession`
    (as before).
  - Recommendation 4, the line codec (T2, T6): `graphshell_client::line`
    (`encode`, `encode_terminated`, `decode`, and `LineAssembler`, moved from
    `webrtc_join`) is the sans-I/O codec both ends share;
    `graphshell_host::line` (`LineReader`, `write`) runs it over Tokio
    streams. `SessionDriver`, the session loop and `BrowserSessionClient`
    use it. Not yet on it: `graphshell-endpoint`'s `stdio` and `network`
    carriers, which cannot depend on `graphshell-host` and would need
    `graphshell-client`; the client correlator and the native
    stream-to-lines pump (T3) are not started.
  - Kept in the port: `native::resident_identity`. Its trait names
    `browser_carrier`'s `NativeIdentityAction`, `NativeIdentityResult` and
    `NativeIdentityFailure` and, through them, `identity_projection`'s
    `SshUnlockPolicyIntentV1`, and nothing in L2 uses it in production; its
    consumers are `identity_endpoint` (stays) and the brokers (P5). `DoorIdentity`
    stays in `native::local_session` with them.
  - Tests that compose port pieces stayed: the browser identity card test that
    was in `session_loop` is `identity_session_tests` in the port, and the
    committed N4 receipt check is the port's `tests/n4_owner_policy_receipt.rs`
    (the receipt and its binary are the port's). `webrtc_door_loopback`,
    `webrtc_join_loopback`, `projection_round_trip` and `remote_session_live`
    moved to the crate.
  - Djinn names `graphshell-host` with `native` (and `webrtc-session` for the
    chronicle receipt host); its `graphshell` dependency stays for the brokers
    until P5. Graphshell's binaries import the new paths.
  - Checks (`TMPDIR=/tmp`): `graphshell-host` (`webrtc-session`) 88 lib tests
    and its four integration suites pass; `graphshell-client` 78 lib tests and
    its suites pass; `graphshell` (`webrtc-session`) 365 lib tests and its
    integration suites pass; `djinn` 180 passed, 11 ignored; `mere-overmap`
    and `graphshell-endpoint` (all features) pass; `cargo check --workspace
    --all-targets --keep-going` fails only at the `tabard-desktop` example and
    the `mere-mesh` examples noted under P1; `graphshell-web` checks for wasm32
    with and without default features; the boundary script passes.
- **2026-10-15 (P4, Knot's receipt and pins):** Knot `b0feec9` (on
  `origin/main`) repins Mere to `71761a553`, the P4 merge: knot-editor's
  `place_projection` test and `k2_peer` example take admission, the carrier,
  the catalog, the projection host and the notifying loop from
  `graphshell-host` (`native`), so Knot's P4 receipt holds;
  `resident_app_route` takes the catalog from it and keeps `graphshell` for
  `serve_app_broker`, `DoorIdentity` and `ResidentIdentity` (P5). The same
  repin takes `backstep` for `edit-history`. Checks (`TMPDIR=/private/tmp`):
  Knot `cargo check --workspace --all-targets`; knot-editor (`mere-overlay`,
  `place_projection` 2 and `resident_app_route` 1 among them), knot-document,
  knot-desktop (lib, bins, `scenario_lane`, `font_roles`, `typography_fonts`)
  and knot-site pass, 624 tests. Mere's `knot-editor`, `knot-document` and
  Djinn's `knot-site` rows move to `b0feec9`; `cargo tree -p djinn` names no
  package from `mere.git`, and Djinn's tests pass (180 passed, 11 ignored).
- **2026-10-15 (P4, Turnstone):** Turnstone `667ba6a` (on `origin/main`) is
  on Knot `b0feec9` and Mere `71761a553`, one Mere source (all 300 Mere
  packages in its tree at that revision). Its place projection host and place
  lanes take admission, the carrier, dialling, the catalog and the projection
  host from `graphshell-host` (`native`); `graphshell` stays for `view` and
  `sessions`. Its `edit-history` patch row is `backstep`. `cargo check
  --all-targets` and `cargo test` pass (942 passed, 10 ignored,
  `TMPDIR=/tmp`). Still open for P4: Cleromancy's receipt (it pins
  `graphshell` at `a1309431` and was not touched here).
- **2026-10-15 (P2 leftover, L5: `canvas_physics` into pictograph):** the
  arrangement and physics actions Graphshell's old page and tree panel share
  are `pictograph::canvas::physics_controls` (`mere::canvas::physics_controls`):
  `apply_physics`, `apply_profile`, `apply_arrangement`,
  `advance_arrangement`, `ArrangementTransition`, `LawStart`,
  `layout_line`, `framing_fields`, the picker catalogs and constants. Their
  code is unchanged apart from paths. pictograph's `canvas` feature now
  depends on `scenotime` (`sceno`, serde and serde_json only) for
  `ProjectionClock`, `TransitionSchedule`, `SceneOp` and `SceneSnapshot`.
  Seven tests that need only `Canvas::with_sample_graph` moved with the
  module. The six that open `GraphshellApp`'s reference fixture (the P2
  fixture's Kinds and Orbit bars and two diagnostics) stay with the host as
  `ports/graphshell/src/canvas_physics_tests.rs`. Graphshell's module is
  deleted with no re-export; `local_scene`, `forme_session_lifecycle_tests`
  and `web_tree` (`lane`, `physics`, `product/scenes`) import the new path.
  `canvas_reader` stays in the port until a host needs it. Checks
  (`TMPDIR=/tmp`): pictograph (`canvas`) lib 446 passed, 18 ignored, and it
  checks for wasm32; `graphshell` lib 365 passed, 6 ignored, the moved host
  tests among them (2 passed, 4 ignored); `graphshell-web` checks for wasm32 (its lockfile, like the root's, gains only pictograph's `scenotime` row); the boundary
  script passes.
- **2026-10-15 (Mark's rulings after P4):** `ResidentIdentity` moves to Djinn
  with the brokers in P5; the I/O-free line codec and `LineAssembler` move into
  `chirograph`, so `graphshell-endpoint`'s carriers and `graphshell-client`
  share it with no new dependency edge; the in-process transport kind goes to
  `mere-transport` in Murm, batched into the remote-route pass with one repin
  round.
- **2026-10-15 (P5, L3 landed in Mere):** `graphshell` holds no broker and no
  `native` module. Old path `graphshell::native::X` is `djinn::door::X` for
  `app_broker` (with its tests), `browser_host`, `device_broker`,
  `local_session` (with `DoorIdentity`), `resident_identity` and
  `resident_mere_proxy`, and `djinn::sync::X` for `personal_sync_host`,
  `carriage_host` (with its commissioning half and tests), `graph_keys` and
  `transfer_staging`.
  - `ResidentIdentity` is Djinn's and extends the port's new
    `identity_endpoint::IdentityAuthority` (`snapshot`, `apply_intent`), the
    half the identity endpoint reads, so the endpoint stays in the port without
    naming Djinn (Djinn depends on the port; the reverse would be a cycle).
    Djinn's keeper implements both. The port keeps a test-only fixed authority;
    Djinn keeps its own for the broker tests.
  - Binaries: `graphshell_native_host` is Djinn's (name unchanged; the
    extension README now builds it with `-p djinn`). `c4_webrtc_host` and its
    `c4_host_signaling` test are Djinn's under the new feature `c4-host`,
    because its resident-mere endpoint is `door::resident_mere_proxy`; Djinn
    always carries Distillery, so the port's `distillery-chronicle-fixture`
    feature and optional `distillery` and `mesh` dependencies are gone.
  - Knot's `resident_app_route` receipt is Djinn's `tests/knot_app_route.rs`,
    against `knot_editor::KnotEndpoint::fixture()`: Knot cannot depend on
    Djinn, which embeds it. Knot deletes its copy at its next repin.
  - Codec: `chirograph::line` (`encode`, `encode_terminated`, `decode`,
    `LineAssembler`); `graphshell-client`'s `line` module is gone. The stdio and
    network carriers in `graphshell-endpoint` now frame through it (they
    already depended on `chirograph`), as do `SessionDriver`,
    `graphshell_host::line` and the browser's WebRTC session. Two behavior
    notes: `StdioCarrier` and `NetworkCarrier` now skip a blank output line
    rather than failing on it (`NetworkCarrier` already did), and stdio's
    request parser reports a blank line it is handed as malformed (its callers
    skip blanks first, as before).
  - What still needs `graphshell` outside the port: Djinn, for
    `browser_carrier` (the browser's envelope and `BrowserSessionClient`),
    `identity_endpoint` and `identity_projection`, and the replica, transfer
    and receipt types the sync hosts serve (L13). Knot and Turnstone need it for
    `view` and `sessions` only, once their repins land.
  - Checks (`TMPDIR=/tmp`): chirograph 52, graphshell-client 77,
    graphshell-endpoint 45 and graphshell-host 88 lib tests and all their
    suites pass (all features); `graphshell` (`webrtc-session`,
    `personal-sync`) 419 lib tests and its suites pass; `djinn --features
    c4-host` 229 passed, 11 ignored (lib 170, `c4_host_signaling` 2,
    `knot_app_route` 1); `cargo check --workspace --all-targets --keep-going`
    fails only at the `tabard-desktop` and `mere-mesh` examples noted under P1;
    the shared crates and `graphshell-web` (with and without defaults) check for
    wasm32; the boundary script passes.
- **2026-10-15 (P5, Knot):** Knot `b19eebf` (on `origin/main`) is on Mere
  `0c03dff3d`. It deletes its `resident_app_route` test (Djinn's
  `knot_app_route` is the receipt now) and takes its client state from
  `graphshell-client`, so `graphshell` is a Knot dev-dependency for `view`
  (`rosette_projection`) and `sessions` (`revision_bell`) only; no Knot
  library code names it. The same repin carries the overmap's merge-in:
  `is_overlay_request` and the desktop's `carry_graph_request` route
  `MereViewRequest::Merge` to the route, and `MereOverlay` reads the pending
  merge through its card's material, keeps each conflict's choice until
  Apply and sends Apply or Cancel with `mere-overmap`'s `merge_intent`; the
  desktop's request match has a wildcard arm ahead of `#[non_exhaustive]`.
  Djinn's `knot_overlay_merges_a_branch_through_the_door` drives it end to
  end (merge into source, the pending card read back, Apply declined until
  chosen, an unknown conflict declined, Choose, Apply, the branch's title
  wins). Mere's Knot rows move to `b19eebf`; Djinn names no `mere.git`
  package. Checks: Knot workspace `cargo check --all-targets` (and
  knot-editor with `mere-overlay`, knot-desktop with all features); knot-editor
  (`mere-overlay`), knot-document, knot-desktop (lib, bins, `scenario_lane`,
  `font_roles`, `typography_fonts`) and knot-site pass, 623 tests
  (`TMPDIR=/private/tmp`); `djinn --features c4-host` 230 passed, 11 ignored.
- **2026-10-15 (P5, Turnstone):** Turnstone `79f5f87` (on `origin/main`) is
  on Knot `b19eebf` and Mere `0c03dff3d`, one source (301 Mere packages at
  that revision). It names `graphshell-client` and `chirograph` directly in
  place of `graphshell::client` and `graphshell::protocol`, so `graphshell`
  is left for `view` (the projection receipt views) and `sessions`
  (`spawn_endpoint_session`). It matches neither `MereViewRequest` nor
  `SessionStep`, so `#[non_exhaustive]` asks nothing of it. `cargo check
  --all-targets` and `cargo test` pass (942 passed, 10 ignored). P5 is done:
  Knot and Turnstone need `graphshell` for `view` and `sessions` only (Knot
  as a dev-dependency); moving those two (Group 5, "stays") would let both
  drop it.
- **2026-10-15 (Mark's rulings after P5):** `view` and `sessions` lift into
  `graphshell-host`, so Knot and Turnstone can drop the port; the browser
  carrier and the identity endpoint pair (`identity_endpoint`,
  `identity_projection`) move into Djinn; L13's replica, transfer and receipt
  types wait for the facet brief; `resident_mere_proxy` moves into
  `graphshell-endpoint`'s `door`, and the C4 fixture returns to the port.
  Knot's and Turnstone's repins wait for pass (b)'s single round.
- **2026-10-15 (those rulings landed in Mere):**
  - `graphshell_host::{view, sessions, canary}`: `canary` came too, because
    `view::render_g1_receipt` runs its loopback canary and Djinn's broker tests
    and `GraphshellApp`'s fixture remote use its `FixtureEndpoint`. All three
    are in the crate's base build (wasm-clean; `base64` is now a plain
    dependency); `sessions`' endpoint spawning needs `native`, which now adds
    `graphshell-endpoint/stdio`. The two receipt checks that read this port's
    committed HTML (`g1_loopback.html`, `g4_session_switch.html`) are the
    port's `tests/g1_g4_receipts.rs`.
  - `djinn::door::{browser_carrier, identity_endpoint, identity_projection}`,
    with the browser identity session test. `IdentityAuthority` stays the
    endpoint's half of `ResidentIdentity`. `GraphshellApp`'s
    `h4_exportable_identity_cards_and_access_survive_scene_reopen_as_projections`
    needed a live identity endpoint; the port cannot depend on Djinn, so it
    now reads the projection (snapshot and card resources, after the
    revocation) from `ports/graphshell/tests/fixtures/identity_cards.json`, and
    Djinn's `graphshells_identity_cards_fixture_is_this_endpoints_projection`
    runs the revocation and fails if the endpoint's output drifts from that
    file (`DJINN_WRITE_IDENTITY_CARDS_FIXTURE=1` rewrites it). The fixture's
    signing request id is pinned so it is the same bytes every run.
  - `graphshell_endpoint::door::resident_mere_proxy`; `c4_webrtc_host`, its
    `c4_host_signaling` test and the `distillery-chronicle-fixture` feature are
    the port's again, and Djinn's `c4-host` feature is gone.
  - What `graphshell` is still needed for outside the port: Djinn only, for the
    sync hosts' replica, transfer, offer and receipt types (L13). Knot and
    Turnstone need nothing from it once they repin.
- **2026-10-15 (pass (b), the remote mere route, in Mere):** the
  [remote mere route admission note](../technical_architecture/2026-10-15_remote_mere_route_admission.md)
  sets out the ambient and explicit grant shapes first (T4), then:
  - T5 needs no dramatis change. The application rides in the requested
    path under the projection service (`/services/projection/mere/<domain>/app/<app>`,
    `connect`; `/services/projection/ambient/<domain>/app/<app>`, the new
    `ambient` action), which Notochord's structural service rules and
    Insigne's attenuation already govern, and the host accepts it only when
    the delegation's leaf names it. `mere_overmap::wire::RemoteMereRequest`
    is the one spelling.
  - `graphshell_host`: `admission::AMBIENT_ACTION` and `open_session_for`;
    `carrier::{admit_stream_with, accept_projection_session_with}` take a
    host's own `ActionCheck` (the default is still `serves_action`, unchanged);
    `projection_host::ResidentProjectionHost::selecting` with a
    `RouteSelector` chooses each session's route and binds its application
    into the endpoint context; the host's catalog is a shared handle
    (`catalog()`).
  - T3's pump: `graphshell_client::pump::LinePump` (feature `pump`, Tokio's
    `io-util` only): open a session over an admitted stream and drive any
    `LineDriven` session (now implemented by `MereRouteSession`) until it
    settles.
  - T7: `djinn::resident_projection` (`ResidentDialHost`, `dial_policy`,
    `select_mere_route`, `bind_dial_transport` on the key derived with
    `mere.djinn/resident-projection/v1`); `MereRoutes::with_remote_catalog`
    registers each mere's two routes in the dial host's catalog. The Djinn
    binary does not start it yet (below).
  - The Murm in-process kind: `TransportKind::InProcess` and
    `IngressContext::in_process` on Murm `38e9c9b8` (branch
    `agent/transport-in-process`, cut from `3d9e799f`, the revision every
    consumer pinned, which is on `codex/c1-webrtc-callback-lifetime-20261014`
    and not on Murm's main). Djinn's local doors now admit through
    `admit_accepted_session` with it. Mere names Murm through the
    `www.github.com` alias and forwards the plain URL to it, the way it
    handles Mere itself, so a consumer still on the old pin resolves to one
    Murm source; consumers name the alias at their repin.
  - Receipt (in Mere): Djinn's
    `a_device_reads_a_mere_through_the_dialable_host_as_its_granted_application`
    dials the host over a memory transport as a same-persona device whose
    grant names `turnstone`, opens `mere/divination`, and reads the sessions
    and graph with `MereRouteSession` over the pump, as `turnstone`;
    `the_dialable_host_serves_only_what_the_grant_and_the_owner_name` refuses
    a grant naming another application, a grant naming none, an explicit
    grant used for ambient, and ambient without the owner's opt-in, and reads
    the graph alone once opted in. Checks (`TMPDIR=/tmp`): graphshell-host
    (all features), graphshell-client (`pump`), mere-overmap, djinn and
    graphshell (`webrtc-session`, `personal-sync`,
    `distillery-chronicle-fixture`) pass, 840 tests; mere-transport's lib tests
    pass in Murm (67); the workspace check fails only at the known examples;
    the shared crates and `graphshell-web` check for wasm32; the boundary
    script passes.
  - Open: the Djinn binary starting `ResidentDialHost` (it needs the trusted
    roots: how a device grant becomes a delegation from the persona root, and
    how a joined place's root is configured; note §7); Turnstone's overmap
    pane mounting a remote mere over `LinePump` (the receipt above uses the
    same panel and pump in Djinn's tests); the Knot, Turnstone and Woodshed
    repins onto this revision and Murm `38e9c9b8`.
- **2026-10-15 (`backstep` cleared for publishing):** Mark said `backstep` can be
  published. `publish = false` is removed from its manifest (crates.io had no
  `backstep` at that date, HTTP 404 on the crate page) and `cargo publish
  --dry-run -p backstep` passes (5 files, 3.7 KiB compressed, verified build). It
  is not on crates.io: the actual `cargo publish` needs Mark's crates.io token and
  cannot be undone, so it waits for his go. Publish order for the crates that name
  it: `backstep`, then `workbench` (already `publish = true`), then `scenomise`,
  then `cambium`.
- **2026-10-15 (`backstep` 0.1.0 published):** Mark ran `cargo login` and approved
  the publish; `cargo publish -p backstep` uploaded `backstep` 0.1.0 to crates.io
  (verified build, 5 files). Its packaged `.cargo_vcs_info.json` names the Mere
  commit it was cut from, a local commit when published and not yet on
  `origin/main`. `cambium` and `scenomise` depend on it, so it no longer
  blocks either; `scenomise` also needs `workbench` published, and `cambium`
  has other dependencies this entry did not check. Publishing either is Mark's
  call.
- **2026-10-15 (publishing scope for `scenomise` and `cambium`; read from crates.io
  and the published tarballs, nothing published or changed):** `workbench` is on
  crates.io only as 0.0.1; the local 0.1.0 has byte-identical `src`, so it is a
  plain publish of 0.1.0. More generally, a crate's local version often equals the
  published one while its source differs, so each of those needs a version bump
  before it can go up (a published version cannot be replaced). Drift of source
  from the published tarball, counting changed lines ignoring whitespace:
  `sceno` 0.0.4 (328), `scenomise` 0.0.4 (406), `mere-curation` 0.1.0 (19: a
  `SubgraphKind::Sparql` variant, so a 0.x minor bump), `tinct` 0.1.2 (280),
  `sprigging` 0.2.1 (291), `illume` 0.0.2 (74), `cambium` 0.3.3 (about 6,000).
  Local versions never published: `scenograph` 0.0.4 (crates.io has 0.0.3),
  `meristem` 0.2.0 (has 0.1.1), `mere-surface-api` 0.1.0 (absent), `workbench`
  0.1.0. Order for `scenomise` with `authoring`: `workbench` 0.1.0; `mere-curation`
  0.2.0; `sceno` 0.0.5 (its dependents' version requirements in the workspace move
  with it); `scenograph` 0.0.4; `scenomise` 0.0.5; `backstep` is already up. For
  `cambium`, past that: `meristem` 0.2.0, `tinct`, `sprigging`, `illume` and
  `mere-surface-api` (publishable once `workbench` is up), and two blockers that are
  not version bumps: `command-menu` is still `publish = false` (unclaimed name,
  SE13 and SE46, so it wants Mark's naming call) and `genet-text`, which current
  Cambium needs and which is not on crates.io (the published Cambium 0.3.3 did not
  use it; the Genet repository owns that decision). `genet-scripted-dom` `=0.1.2`
  and `layout-dom-api` `=0.1.1` are published at the versions Cambium names.
- **2026-10-15 (`workbench` 0.1.0 published; the rest of the chain held):** Mark
  approved publishing the `scenomise` chain. `workbench` 0.1.0 is on crates.io (its
  source equals the published 0.0.1; dry run and publish passed). `sceno`,
  `scenograph` and `scenomise` are held: `sceno` and `scenomise` must bump (0.0.5
  each) because crates.io's 0.0.4 sources differ, and a version bump in this
  workspace stops Mere's `[patch]` rows applying to anything pinned to an older
  Mere (a `^0.0.4` requirement does not accept 0.0.5), which breaks Djinn's
  dev-dependency on Knot until Knot repins onto the bumped revision and Mere bumps
  its Knot pins. The bump is saved as a patch, not committed, and the Knot repin
  agents were asked how to sequence it. `mere-curation` does not need a bump for this
  chain (`scenograph` uses only `SubgraphSpec`, which 0.1.0 has). More `cambium`
  blockers found: current `sprigging` needs `netrender-vello` `^0.11` (crates.io has
  0.10.1; the published `sprigging` 0.2.1 had no such dependency), and `genet-text`
  is publishable as it stands (no `publish = false`, one registry dependency, name
  free; `cargo publish --dry-run -p genet-text` passes in the Genet repository).
  `command-menu` is the model behind menus, not the drawn context menu: the
  commands a host registers, a person's added, removed and recent choices, the
  order a surface shows them in, the palette's session state, and the shared
  command ids.

- **2026-10-15 (publishing chain done; the one repin round; then the branch
  convention):** published to crates.io: `workbench` 0.1.0, `backstep` 0.1.0,
  `sceno` 0.0.5, `scenograph` 0.0.4, `scenomise` 0.0.5, and, from the Genet
  repository at Mark's direction, `script-engine-api` 0.1.1 (ValueEngine, so
  `scenograph`'s `^0.1.0` requirement resolves) and `genet-text` 0.1.0.
  `scenograph` needed `version = "0.1.0"` on its optional git row. The repin round
  ran in the order the remote mere route spec §7 sets: Murm `main` `fb6360c12b`;
  Mere `R` `75a2e6ae7` (Murm rows, `www` alias); Knot `K` `fe9d98c` (Mere `R`, Murm
  `M`, `sceno` and `scenomise` 0.0.5 rows, no `graphshell`); Mere's Knot pin `R2`
  `b2c8dcc28`; Turnstone `T` `6439fa88` (Mere `R`, Knot `K`, Murm `M`, `graphshell`
  dropped). Receipts: Knot 623-plus tests across the four Knot workspaces;
  Mere `cargo test -p djinn -p graphshell-host --all-features` 204 passed; port
  boundary script green; Turnstone `cargo check --all-targets` and `cargo test`
  (942 passed, 10 ignored). Pre-existing failures that this round did not touch:
  `tabard-desktop`'s `native_shared_recovery` example (`PendingPresentationCapture`
  fields) and `mere-mesh`'s `inventory_ingress` and `retained_receive` examples
  (`include!` of `../../../../murm/...`, a path that no longer exists).
  Mark then ruled the family onto `branch = "main"` (one canonical URL per
  repository, committed lock, repin = `cargo update -p`; written up in the family
  composition brief, "Family source convention"). Landed: Genet `7af8de32`
  (netrender and murm-wire rows), Mere `4a74b1cf` (genet, murm, netrender,
  knot-editor rows; the Genet, Murm and netrender bridge tables and every
  same-repository patch row deleted), Murm `a3621dc8` (the native workspace's Mere
  rows, which kept a second Mere source in every consumer), Knot `3fc16df`. The
  lesson is that Cargo cannot bridge two spellings of one repository, so the switch
  is a flag day across the supplier chain, not a per-repo edit: one repository left
  on `rev` keeps an extra source in every consumer. Each graph now resolves one
  source for Mere, Murm, Genet, netrender and Knot. Not switched, and why:
  `dramatis`, `cleromancy`, `wgpu-weld` and `smolweb` (their `main` is ahead of the
  pin and not qualified against the consumers), `vano` (Genet pins a specific
  `nova_vm`), the `mark-ik` forks (pinned on purpose), and Woodshed and Turnstone
  (Woodshed's redshank rows still name Mere, Genet and netrender by `rev`, so
  Turnstone cannot flip until Woodshed does; the Woodshed lane has been asked to).
  Mere keeps one bridge: `[patch."https://www.github.com/merely-made/mere.git"]`,
  because Cleromancy's pinned revision still names Mere through the `www` alias; it
  goes when Cleromancy switches (its handoff says so). Woodshed `cbf95e4b` switched
  at the Woodshed lane's hand, and Turnstone follows it (below).
  One local environment note: `TMPDIR` under a long path makes
  `ports/djinn/tests/device_directory.rs` fail to bind its socket (it passed with
  `TMPDIR=/tmp`), while knot-site's export tests need a symlink-free `TMPDIR`
  (`/private/tmp`).
- **2026-10-15 (convention extended to wgpu-weld, smolweb and vano; dramatis
  blocked):** none of those four repositories names a family repository in its own
  manifests, so the switch is in their consumers' rows. Landed: Genet `e37762d`
  (vano; `script-engine-nova` checks and 43 tests pass), Mere `4e2828c`
  (wgpu-weld; `weld-engine` 11 tests, djinn, graphshell-host, cambium, scenomise
  and inker check clean), Knot `8196f98` (smolweb; knot-site, knot-editor and
  knot-desktop tests pass), Turnstone `efdc67c` (wgpu-weld and smolweb, Mere
  `4e2828ce`; 942 passed, 10 ignored). Dramatis is not switched: Cleromancy, which
  Mere's Djinn pins at `2851e6b0` and which stays with its own lane, names Dramatis
  by `rev` at the plain URL, so any consumer that flips keeps a second Dramatis
  source through it, and Cargo cannot bridge that. It flips when Cleromancy does
  (its handoff, task 2), together with Mere, Murm `native/`, Knot, Turnstone and
  Woodshed in one pass.
- **2026-10-15 (the dramatis flip, one pass):** with Cleromancy in the pass,
  dramatis now follows the convention everywhere. Manifest commits first (each
  graph needed the others' flipped manifests before it could lock): Murm
  `742d2e8`, Knot `11b0736`, Cleromancy `a4364de`, Mere `59f88e6` (dramatis and
  the Djinn Cleromancy pin; the `www` Mere patch table deleted), Turnstone
  `bda84ca`, Woodshed `0a0d67d`. Then lock commits: Murm `cba46d8`, Knot
  `d2cb090`, Mere `82e7fd7`, Turnstone `15362e3`, Woodshed `4415b71`, Cleromancy
  `22df4c9`. Every graph resolves one source for Mere (`59f88e63`), Murm
  (`cba46d85`), Genet (`e37762d7`), netrender (`f4f9c0d8`), Knot (`11b07360`),
  Cleromancy (`a4364de9`) and dramatis (`5c89001a`). Checks: Mere's djinn,
  graphshell-host, graphshell-client, cambium and scenomise (all targets, all
  features), djinn and graphshell-host tests, port boundaries and the
  graphshell-web wasm32 check; Knot's full round; Turnstone 942 passed; Woodshed's
  root, hocket and redshank checks and tests; Murm's native workspace; Cleromancy's
  default, graphshell-admission and core checks and `cargo test`. Not green, and
  not fixed because it is product code: Cleromancy's `personal-sync` feature
  fails with 15 `PersonalGraphEvent.basis` missing-field errors against the
  current Mere (its handoff says so, task 2), and Woodshed's
  `validation/relationship-recipe` bin test target lacks a `tempfile`
  dev-dependency, which its manifest never declared. Still on `rev`, on purpose:
  `wavicle` (Woodshed), `turquet`, `wgpu-graft`, the `mark-ik` forks.

- **2026-10-15 (P4, Cleromancy's second-host receipt):** Cleromancy `9aae258`
  retains the extracted endpoint registration/admission path in
  `cleromancy/src/admitted.rs`. `resident_catalog_opens_cleromancy_under_the_admitted_session`
  and `admitted_context_scopes_cleromancy_to_the_browser_session_and_subject`
  pass in the explicit `legacy-receipts,personal-sync,graphshell-admission`
  harness against locked Mere `59f88e63`. Registration and admitted context
  name `graphshell-host`, with local carrier/client types from their stack
  crates. The only direct port import in the adapter is feature-gated
  `graphshell::personal_sync::SyncProjection`; the Graphshell package also
  remains in the resolved dependency graph. This is the P4 endpoint-path
  consumer proof, not L13/P8 removal of the port or new native acceptance.
  `cleromancy/receipts/family-intake_20261015.json` records the software intake
  and preserves the original C1/C2 qualification identities.
- **2026-10-19 (the Cambium publish chain, done):** Mark chose to keep the name
  `command-menu` (the name `alembic` is held on crates.io by a third party, a
  placeholder for the Alembic VFX bindings; `mere-alembic` 0.0.1 stays ours) and to
  publish the vello fork's pieces under our names. Published, in order:
  `netrender-vello-encoding` 0.11.0, `netrender-vello-shaders` 0.11.0 and
  `netrender-vello` 0.11.0 (from `mark-ik/vello` branch `mark-ik/all-vellos`,
  `ada4ed86`; the fork's `vello_encoding` and `vello_shaders` research crates
  renamed by package, library names unchanged, because upstream's 0.11.0 lacks
  `MAX_FLATTEN_LINES` and `FLATTEN_QUALITY_FAILED`, which the dry run showed);
  `mere-surface-api` 0.1.0, `command-menu` 0.1.0, `meristem` 0.2.0, `tinct` 0.2.0,
  `illume` 0.0.3, `sprigging` 0.2.2, `mere-curation` 0.1.1, `sceno` 0.0.6,
  `scenograph` 0.0.5, `scenomise` 0.0.6 and `cambium` 0.4.0 (minor, because the
  `nematic` feature removal breaks 0.3.3). Every crate differed from its published
  version, which is why the published three and the scene crates bumped; the first
  cambium dry run failed against crates.io's scene crates (`sceno::LinkKey`,
  `scenograph::link`, `scenomise::backdrops` were not published), so they went
  first. Verified from crates.io alone: a project depending on `cambium` 0.4.0 with
  the `highlight` feature builds with no patches. The family's version requirements
  moved with the bumps (cambium 0.4.0, illume 0.0.3, sceno 0.0.6, scenograph 0.0.5,
  scenomise 0.0.6: Mere, Knot, Woodshed hocket, Cleromancy, Murm signalman) and
  each lock was refreshed: Knot `bde6f94`, Woodshed `18ef512`, Cleromancy `d932acc`
  (built from a fresh clone; its `ResidentTasks` import moved to `armillary`),
  Murm `50c2e56b`, Genet `4987cdf` and Mere `4d5632de` (netrender back on
  `branch = "main"`; the Genet lane had moved those rows to `rev`, which kept a
  second netrender source in every consumer). Knot's and signalman's tests moved off
  `taproot::matching` (Genet removed it), and Knot's metre scene sets
  `stress_known`. Known red, not caused by the chain: Mere's `djinn` integration
  tests read files under `crates/intel/esp` that the ESP extraction removed; Knot's
  `command_chrome` `title_is_draggable_but_menu_and_document_are_not` (the harness
  finds no `menubar` role) and `scroll_site`'s
  `a_micron_submission_redraw_keeps_the_scrolled_preview...`. The fork keeps rev
  `276b4664` in the family's `[patch]` rows (same code, old package names); they
  move to the renamed fork revision at the next vello repin.
- 2026-10-22 (scene shapes): the saved-scene envelope (`SavedSceneV3`, the
  version 2 reader shape, `SavedDynamics`, the `graphshell.saved-scene/v4` facet
  constants) and the cartography geometry left the port for `pandect`
  (`saved_scene`, `cartography_geometry`), and capture, restore, validation, the
  version 2/3 conversion and the member remap for copies left it for
  pictograph's `canvas::saved_scene`, so Turnstone holds the same scene on its
  container node. `personal_sync` now imports the facet constants from
  `pandect`, which removes the scene types from what keeps it in the port; what
  still keeps it there is the rest of its product-facet reading. Its
  facet-name check (`validate_facet_name`) refuses any name containing "seed",
  so a facet named like `arrangement.seed` cannot sync; the scene facet holds its
  seed inside the value and is unaffected.
