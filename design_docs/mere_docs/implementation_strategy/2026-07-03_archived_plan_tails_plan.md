# Archived-plan tails — deferred items spun out of the archive passes

**Date**: 2026-07-03, extended 2026-08-06, 2026-09-02, 2026-10-07 and 2026-10-09.
**Status (2026-10-09)**: backlog holder. Each item below was explicitly deferred by a plan that
is otherwise complete and now lives under
[`archive_docs/`](../../archive_docs/), in the checkpoint folder named by the
section it sits under. None of these gate anything today; pick up when the
relevant lane is quiet. Items already tracked for implementation by an active
plan are *not* repeated here. The dated RDF section below is an explicit scope
transfer from an active plan's received archive work; that plan retains a
pointer only.

This holds the tails from every pass rather than one per pass: a deferred item
is easier to find in one backlog than across a folder of dated stubs, and the
sections say which plan each came from.

## RDF archive cleanup deferred by graph semantics (2026-10-07)

Mark instructed **"Bound it."** after the graph-semantics lane accumulated
31 checkpoints. Its
[scope boundary](2026-10-04_graph_semantics_plan.md#scope-boundary-2026-10-07)
retains the accepted dual-stratum/RDF-profile work and transfers these two
independent tails here. They do not gate P2–P5 completion.

- **ExampleOf/Summarizes alignment review.** The received RDF archive proposes
  `schema:exampleOfWork` and `cito:describes` superproperties. These proposals
  remain unaccepted. Keep current predicate identity and vocabulary mappings
  during graph semantics. A future Mere vocabulary pass owns meaning review,
  consumer compatibility and any explicit ruling before a mapping changes.
- **Old Oxigraph oracle/dependency retirement.** The production borrowed
  `QueryableDataset` and the existing materialized spareval oracle already
  provide the required parity path. A future Mere dependency-cleanup pass may
  remove the independent test-only old Store path and its query dependency,
  with a narrow reviewed lock change if needed. It must preserve the active
  materialized parity control; it does not replace the backend or query model.

**Reviewed and locally completed 2026-10-09:** the
[follow-on lane](2026-10-09_graph_semantics_followons_plan.md) reviews both tails.
ExampleOf remains MereOnly: its generic meaning exceeds Schema.org's
creative-work instance relation. Summarizes retains the current approximate
`cito:cites` alignment; changing to `cito:describes` remains a separate explicit
vocabulary ruling because the exported alignment quad would change.

The old Store path, timing-only test and optional Oxigraph dependency are removed
at locally qualified source `297682178`. The active borrowed/materialized
spareval parity battery remains. Explicit features preserve SEP-0002, SEP-0006,
calendar support and directional JSON-LD behavior previously inherited through
Oxigraph. Six meaningful feature controls have observed negatives; the full
linked suite passes 91 tests and Mere passes 54, with feature-free/workspace and
both query wasm checks green. Fresh review's one Important direction-matching
regression is repaired and requalified. Only three unreachable packages and
unused canonicalization feature edges leave the lock; surviving package versions
and checksums are unchanged. Publication is pending user approval. Historical
receipts above remain historical; these counts describe the new source gates.

Term dictionaries/interned slotmaps remain separately gated in the existing
petgraph-RDF record. New CONSTRUCT/DESCRIBE capability is also outside the
bounded graph-semantics lane; its row-result saved-query requirement does not
create a general SPARQL feature expansion.

## From native_surface_compositing (complete 2026-06-21)

- **wgpu-scry non-blocking capture settle** — `start_capture`'s ~500ms settle blocks
  the UI thread on a (now rare, backed-off) stall-restart; make it non-blocking in
  wgpu-scry. Needs demo runtime verification, not a blind edit.
- **Cache-flush per-tile submit batching** (flagged cleanup).
- **Silent implicit-sync fallback** when the explicit D3D12 fence fails — should
  warn + fail the tile under D3D12.
- **`favicon_data_uri` misnomer** — it also encodes the snapshot peek; rename.

**Corrected 2026-10-06 (S14 pass):** `favicon_data_uri` no longer exists in
Mere, Turnstone or Genet's components at mere `535bca11`; it was meerkat render
code, removed with meerkat 2026-07-18 (`c5f01064`). The same holds for the
find-field paste tail (`handle_clipboard_shortcut`), the context-submenu tails
and the `ProducerSurface` rename, each corrected where it sits.

**Open, raised by the S14 pass (2026-10-06):** four tails in the 2026-07-03 and
2026-07-04 sections name meerkat code that no longer exists (above). How should
this backlog carry them? Options: prune them the way the 2026-09-02 pass
treated deleted subjects, keeping only any host-agnostic principle; keep them
in place marked as history.

## From documentscript_net_hardening (substantially complete 2026-06-24)

- **E1 refinement, uncredentialed same-origin fetch** — a same-origin `net.fetch`
  still carries that origin's own cookies; drop even those (Mark's session-store
  domain, A1 step 1).
- **E1 refinement, cross-origin `net` declarations** — a mod manifest declaring
  extra origins beyond its page, with broad-glob guards.
- **E2 — true non-blocking fiber suspension** for `net.fetch`: dispatch onto the
  off-thread fetch actor and resume the fiber, so the content actor keeps servicing
  commands during I/O (A2's 30s timeout mitigates today).
- **E3 — mod install/approval + optional signing** (the proper B1 fix; today any
  approved-capability `.wasm` in `mods/` auto-attaches).

  **Corrected 2026-10-06 (S14 pass):** nothing auto-attaches today.
  `discover_wasm_mods_in_dir`
  (`crates/system/registry/src/mod_loader/loader/free_fns.rs:149`) has no
  production caller in Mere or Turnstone; only its tests call it.
- **B4 — `.cwasm` AOT mods invisible to discovery** (loader supports them; discovery
  matches only `*.wasm`).
- **D2 — `Fetched` carries no HTTP status** (non-2xx collapses to `Err`; guests
  can't observe 404/3xx). Touches `fetch.rs`.
- **DNS-rebinding defence** — the SSRF guard is a literal-host deny-list;
  resolve-then-pin is the follow-on.

## From find_in_page_host_ui (feature complete 2026-06-16)

- **Gemtext/document-lane find — closed 2026-07-03.** The user-facing retained-text
  slice landed in the
  [retained_text_tiled_render_plan](../../archive_docs/2026-07-04_completed_plans/2026-06-15_retained_text_tiled_render_plan.md):
  Ctrl+F searches retained document blocks, paints block-scoped highlights, and
  page-text copy works from the retained packet. The precise glyph→char cluster
  map on `GlyphRun` remains only a fidelity follow-on, not an open backlog tail.
- **Paste into the find field** — `handle_clipboard_shortcut` routes to
  omnibar/palette only.

  **Corrected 2026-10-06 (S14 pass):** `handle_clipboard_shortcut` no longer
  exists in Mere, Turnstone or Genet's components; it left with meerkat
  (`c5f01064`, 2026-07-18). See the open question under native_surface_compositing.

## From context_submenus (implemented 2026-06-25)

- **GUI feel verify pass** — pixel placement of the flyout, live hover/keyboard
  feel (Mark-runs-it; logic + DOM are tested).
- **Hover-open with delay** (cursor-move handler + an `Instant`-timed
  `submenu_hover` field).
- **Submenu mis-anchor when the root menu is scrolled** (`row_y` ignores the root's
  scroll offset; narrow case).
- **Mouse hit-test after keyboard-scroll** uses unscrolled offsets (pre-existing for
  the flat menu).
- **Depth-N submenus** — model is depth-1 by convention; deeper needs a path, not an
  index.

**Corrected 2026-10-06 (S14 pass):** the context-submenu code these items
concern no longer exists in Mere, Turnstone or Genet's components; it left with
meerkat (`c5f01064`, 2026-07-18). `submenu_hover` was a proposed field and was
never added. See the open question under native_surface_compositing.

## From keyed_view_sequence (implemented 2026-07-02)

- **P4 — `ElementSplice` move primitive** for state-preserving arbitrary reorder.
  Gated on ui_polish finding-5 (paint-list emission cost) landing first, so
  delete+reinsert's real cost is separately visible. Plus OQ-1 (linear key lookup
  vs hash index — profile first) and OQ-2 (duplicate-key policy).

## From engram_compose_merge (P1–P3 done 2026-06-30)

- **rkyv compaction of graph engrams** (Alembic tail B6) — override
  `TypedPayload::serialize_to_bytes` for `GraphEngram` with rkyv (mind the
  read-alignment `AlignedVec` gotcha). Measure the size win first.

  **Corrected 2026-10-06 (S14 pass):** `GraphEngram` is now `GraphCodicil`
  (`c51b9704`, 2026-08-31; `crates/system/pandect/src/graph_codicil.rs:62`).
- **Promote `merge_snapshots` into the kernel** (`Graph::merge_from`, reusing the
  URL index + edge API) once the kernel is quiet — the snapshot-level merge was the
  non-colliding path, not the final shape.

## 2026-07-04 archive pass (12 more plans → `archive_docs/2026-07-04_completed_plans/`)

### From document_style_sheet (P0-P4 complete 2026-06-22, inker)

- **Container + non-text roles** (Quote / List) — own plan when consumer
  pressure appears; deliberately not built at the tail of P4.
- **Visual arrow check** on document-lane sheet arrows was deferred at closeout.

### From document_typography_surface (D1-D3 shipped 2026-06-22, inker)

- **D4 — per-role + per-engine typography overrides** (advanced section) and
  **full font enumeration**. The seed-palette plan's "surface per-role document
  knobs in settings" deferral is the same item; do them together.

### From surface_engine_contract_fold (complete 2026-07-04, inker)

- **`SecondaryForward` rename** — the flip shim kept the old `ProducerSurface`
  name while becoming generic over `WebSurface`; rename when touched (cosmetic).

  **Corrected 2026-10-06 (S14 pass):** `ProducerSurface` no longer exists in
  Mere, Turnstone or Genet's components; the shim left with meerkat
  (`c5f01064`, 2026-07-18). `SecondaryForward` was the proposed new name and
  never existed. See the open question under native_surface_compositing.

### From retained_text_tiled_render (acceptance met 2026-07-03)

- **Image-only inline links** — an `<a>` wrapping only a replaced element
  establishes no box and is not harvested.
- **Per-band link re-harvest caching** — links re-harvest on every band
  re-emit; cache once a tall many-link page measurably bites.
- **Glyph→char cluster map on `GlyphRun`** — upgrades document-lane find/select
  from block-scoped to exact intra-line geometry (fidelity, not function).

### From seed_palette_theme_system (complete 2026-06-22)

- **TOML swap** for the theme file format (kept format-agnostic); **rescan-on-
  demand** for theme packs (startup-only today).

### From gnode_pool (landed 2026-07-02, follow-ons resolved 2026-07-03)

- No open gnode-pool debt. The residual (loaded-session `chrome_us` /
  `chrome_raster_us` on legitimately-dirty frames) is owned by genet's
  `docs/2026-07-03_shell_paint_emission_raster_plan.md`; the like-for-like
  loaded-session capture is that plan's motivating measurement.

No tails: chrome_bar_refinement (its deferred switcher cleanup was completed
in-plan), ui_dpi_scaling, gloss_scene_to_dom (follow-ups owned by the active
gloss_outline_lens plan), graphlet_wiring (cross-plan leftovers owned by the
active relational_browse plan), tearout_composability (continuation is the
active tearout_gestures plan).

**Corrected 2026-10-06 (S14 pass):** the relational_browse plan is no longer
active. It was archived as
[2026-06-23_relational_browse_graphlet_plan.md](../../archive_docs/2026-08-06_completed_plans/2026-06-23_relational_browse_graphlet_plan.md),
so graphlet_wiring's cross-plan leftovers point at an archived plan.

## 2026-09-02 archive pass (17 retired plans → `archive_docs/2026-09-02_retired_plans/`)

Retired, not completed: each plan's subject was deleted — fifteen by the
meerkat removal of 2026-07-18 (`c5f01064`) or genet's Stylo and `genet-layout`
retirement of 2026-08-21 (`55c05d11759`) — or its status had read "in
progress" for two months with no commits. Ruled by Mark 2026-09-02 on the
active-tree audit's evidence (phase D of
[the doc policy consolidation plan](../../archive_docs/2026-10-06_completed_plans/2026-08-24_doc_policy_consolidation_plan.md)).
An evidence pass over turnstone, genet and mere found none of the work built
under another name. Items belonging to genet are marked; they want a home in
genet's `design_docs/`, not here.

### From short_term_memory_substrate (plan-only since 2026-05-14)

- **JSON-sidecar-over-fjall for short-term persistent state**, and the
  **`durable` flag for throwaway forks** — two unimplemented rulings. Their gate
  ("when branch operations land") never fired; the tear-out arc closed without
  a `branch_store`.

### From accesskit_screen_reader_verification (never run)

- **Separate adapter installation from OS traversal** when VoiceOver cannot
  enter the window tree (the macOS step). The checklist's purpose was met by
  Turnstone `648bf19`, the headed OS screen-reader receipt; a new checklist
  would target `ports/graphshell`.

### From workbench_staging (no code, 2026-06-09)

- **Where the latent staging relation lives** — gloss-owned subgraph store
  versus a kernel edge family flagged latent — explicitly Mark's call,
  unanswered; with it the chain-versus-bus default. The set primitive exists
  unconsumed (`platen/src/workbench.rs:182 open_split`, `:189 open_stack`);
  turnstone's workbench opens one node at a time.
- **Check the 2026-06-27 toolbar-clipping issue** against
  `turnstone:src/workbench_tiling.rs`; it may still reproduce.

### From lane0_sidequests (five of seven open)

- **Refreshed list** (evidence pass 2026-09-02): items 1 and 2 shipped; item 5,
  the Trail affordance, is met by the palette entry at
  `turnstone:src/panes/registry.rs:342-358`; **item 6, Barnes-Hut, is one
  `add_force` line from live** — `BarnesHutRepulsion` is built and exported in
  seiche, unwired at `crates/canvas/pictograph/src/canvas/seiche_bridge.rs:58` (check
  whether "tuning" is the real blocker); items 3 (relation-kind picker — every
  `assert_selected_relation` caller passes `UserGrouped`), 4 (tessera score on
  the chip — `Ledger::score` exists, no consumer) and 7 (Steward per-row
  controls — the pane is a read-only downloads projection) are unbuilt with
  their substrate present.

  **Corrected 2026-10-06 (S14 pass):** item 6 is done. Barnes-Hut is live as
  `PhysicsLaw::Charge` ("charge.barnes-hut",
  `crates/canvas/pictograph/src/canvas/physics_catalog.rs:149` and
  `:1110-1114`), landed in `37477457` on 2026-09-02 and recorded in the
  [physics catalog plan](./2026-09-02_physics_catalog_plan.md).
- **Forme dead-submodule cleanup, never done**: `subgraph` (renamed from `graphlet` 2026-09-12), `lens`, `parity`,
  `pressure`, `reconciliation` still exist at `crates/forme/forme/src/`. A
  deletion; wants a yes.

### From command_registry_configurable_menus (P1–P5 landed, deleted with meerkat)

- **The registry-as-one-seam thesis and the S1–S3 searchable-menu design**,
  with three corrections from the evidence pass: turnstone re-decided P2-rest
  as flat palette rows, not a picker (`action.rs:554-566`); P4's configurable
  menu is absent (the catalog is hardcoded, `palette.rs:1-6`); the thesis is
  partly realized in turnstone's single `Action` catalog read by palette,
  snapshot and automation alike.
- **Orphaned persisted fields**: `PersonaSettings.menu_actions` and
  `command_usage` at `crates/system/pandect/src/persona_settings_store.rs:39`
  survive with a round-trip test and zero consumers in mere or turnstone.
  Reconsume in turnstone or drop from the schema — a code ruling.

### From object_card (P0 done, host deleted)

- **The Widget / Preset / type-scoped-card model and P1–P4** — unimplemented
  design. Widget 1's logic lives on in canvas (`SIZE_TIERS` at
  `crates/canvas/pictograph/src/canvas.rs:182`, `node_size_tier`).

### From layout_phase_split_probe (never built; genet)

- **The parallel-cascade thesis has no mechanism left.** It rode Stylo's rayon
  traversal (`genet:docs/2026-06-13_parallel_cascade_scope.md`, deferred), and
  Stylo and `genet-layout` were retired in genet `55c05d11759`; buckram,
  genet-livery and cambium carry no `rayon` and no timing. The root
  `2026-06-21_substrate_parallelism_composition_brief.md` still presents the
  thesis as live and needs a note. A measurement of the new stack is a new
  genet plan.

### From meerkat_render_perf (meerkat only)

- **Three host-agnostic principles**: M2's dirty-gate rule, M3's settled-scene
  per-key caching, M4's per-`(member, viewport)` scene set. M2 speaks directly
  to the live turnstone perf item, the command-palette lag recorded in the
  device resident consolidation plan on 2026-08-22.

### From host_scroll_engine_adoption (never started; genet)

- **P3's nested scroll-into-view gap** and **P4's "engine owns the scrollbar
  thumb, host does not paint one"** — engine-side asks for genet's
  `design_docs/`.

### From tracing_reach_and_quality (T1, T1.5, T2-substrate landed; spine deleted)

- **Lesson worth a workspace note**: a tracing layer must never panic, and
  per-span extension writes must be idempotent.
- **T3 in-tree engine and graph-kernel spans, T4 correlation, T5 registry
  sampling plus error-chain capture** — re-scopable against turnstone and
  djinn; the armillary and register-diagnostics halves survive (`spawn_named`
  at `crates/armillary/src/actor.rs:128`).

### From notification_subsystem (Phase 0 deleted; Phases 1–4 never built)

- **The model**: the log is the Steward's, the toast is the chrome's, actions
  ride the toast, continuous chips are not notifications; dedupe and
  rate-limit before rollout. Turnstone has no notification concept (its
  `AppEvent` stream is telemetry for the a11y and app arms).
- **Dead code**: `ToastSpec` / `ToastSeverity` / `FrameViewModel.toasts` at
  `crates/shell/chrome/src/frame_model.rs:404-416` *(historical citation)* <!-- doc-audit: historical-path -->, egui/iced-era, no producer,
  no consumer. A code ruling.

### From graph_object_roster_detail_cards (model migrated, views deleted)

- The model lives on verbatim — `RosterTab`/`RosterSubject`/`SubgraphSpec` in
  `mere::roster` (`crates/mere/src/roster.rs`), `EdgeCell`/`EdgeFamily` and `visible_relation_edges`
  in canvas, the selectors in `mere::subgraph` (`crates/mere/src/subgraph.rs`). Open: **sub-kind
  selector editing**, the **P5 `GraphDefault < GraphViewOverride <
  SelectionOverride` stack**, **true parallel edge instances**; the plan's
  §Contradictions and §Pitfalls are durable design notes.

  **Corrected 2026-10-06 (S14 pass):** two locations are inexact. `SubgraphSpec`
  is `forme::SubgraphSpec` (`crates/forme/forme/src/subgraph.rs:69`), not part
  of `mere::roster`, and `EdgeFamily` lives in graph-kernel
  (`crates/graph/graph-kernel/src/graph/edge_taxonomy.rs:33`), not canvas.

### From kith_capability_sharing (gate cleared 2026-08-09, never started)

- **Owed by name**: `crates/mesh/mesh/src/lease.rs:22-26` *(historical citation)* <!-- doc-audit: historical-path --> — "the kith plan,
  which widens the ring beyond one owner, has to revisit it"; gemot is still
  at the ring rule with capability gating a later milestone. **Re-scope onto
  the shipped vocabulary** — `servitor::cap` (`crates/servitor/src/cap.rs`, `Cap::{Power,Scope,Facet}`),
  gemot `typed_authorization.rs` (ruled 2026-07-24), personae delegation
  certificates — keeping only the mesh-specific parts: claim validation in the
  board fold, epoch revocation, the six done-conditions. Meadowcap-shaped
  grants exist in `crates/servitor/src/grant.rs`; notochord may already
  answer "was this chain valid at T".

### From ui_polish (S1–S4 executed 2026-07-05, status never advanced)

- **P5 canvas-text-scaling policy** — fixed labels versus zoom-scaled versus
  hybrid, as a setting with an LOD floor — and **OQ-1**, whether Ctrl+zoom
  reaches content. Turnstone-relevant.
- **Finding 5's retained/fragment-keyed paint-list ask** belongs to genet.

### From comms_gating_and_key_addressing (never built; successor differently shaped)

- **Default-off comms as a product requirement** — belongs in the djinn family
  resident services plan, since djinn owns network runtimes. Turnstone's
  `src/place/` already delivers G1/G2's outcome by construction: every network
  action is a user verb.
- **G3–G6 untouched**: the refresh-ticket versus rotate-identity verb split;
  the Windows/firewalld bind posture; the `mere/misfin/v1` ALPN; the
  blueprint-over-protocol ruling and the LXMF finding (store-and-forward tier).

### From signet_trust_plane (S0, S1 and the grant half landed elsewhere; premise reversed)

- **OQ1** the trust-plane umbrella name; **OQ4** meadowcap versus Biscuit grant
  format (with kith above); **S3** a first non-mere consumer, the one open
  rung. OQ2 and OQ5 are answered in code (`personae::carry`,
  `wallet_grant/epochs.rs`).

### From irc_mod (moothold; design step only, 2026-05-05)

- No open tail. Its shape remains the template for future T1 protocol mods
  (Nostr, Matrix, ATproto).

## Progress

- **2026-07-03** — created during the archive/reconcile pass; 11 completed plans
  moved to `archive_docs/2026-07-03_completed_plans/`, their deferred items
  collected here.
- **2026-07-04** — reconciled the find-in-page carry-over: the document-lane
  retained-text find/copy acceptance slice is now closed in the retained-text
  plan, while paste into the find field remains open.
- **2026-07-04** — second archive pass: 11 more completed plans moved to
  `archive_docs/2026-07-04_completed_plans/` (joining the concurrently-archived
  misfin promotion plan); their tails added in the 2026-07-04 section above.
- **2026-09-02** — third archive pass, the first for *retired* rather than
  completed plans: 17 moved to `archive_docs/2026-09-02_retired_plans/` on
  Mark's ruling over the active-tree audit; tails in the 2026-09-02 section
  above. Four of them are code rulings rather than doc tails: the orphaned
  `PersonaSettings` fields, the forme submodules, `ToastSpec`, and the one-line
  Barnes-Hut wire.
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_49_s14_phase_b11.md: the Barnes-Hut tail marked
  done (physics catalog P1, `37477457`), the E3 auto-attach claim and the
  GraphEngram and session-runtime names corrected, the relational_browse owner
  noted as archived, the roster model locations fixed, and the four
  meerkat-subject tails marked with an open question on pruning them.
- **2026-10-06** — fourth archive pass, the S14 pass's (stack seams plan,
  rulings S36 and S48 to S66): 88 plans moved to
  `archive_docs/2026-10-06_completed_plans/` (77),
  `archive_docs/2026-10-06_superseded_plans/` (7) and
  `archive_docs/2026-10-06_retired_plans/` (4). Their tails are in the
  2026-10-06 section at the end of this file, each with its destination;
  those held here are marked "here", and the S59 critical pass has its own
  subsection.

## From stickleback_replication_promotion (complete 2026-07-27, archived 2026-08-06)

- **A sibling repository for `stickleback`, gated on a real external consumer.**
  The promotion deliberately stopped inside Mere: `stickleback` 0.1.0 is
  published from this repository under MIT OR Apache-2.0, and S3 passed the
  publishable-boundary review, so nothing technical blocks the move. What is
  missing is a reason. The plan's own rule was that a domain-neutral crate earns
  a sibling repo when someone outside this workspace depends on it, not when it
  merely could. Revisit when an external consumer appears, and not before.

## From forest_dom (landed 2026-07-18, archived 2026-08-06)

- **F4, per-window multi-DPI.** Per-window DPI, viewport, and cascade, deferred
  on purpose rather than missed: the plan's own instruction was not to
  gold-plate the per-window cascade before F3 had banked the topology, and F3
  has. Revisit when a real multi-monitor case wants it, which is also the only
  situation that can say whether the cascade needs to be per-window at all.

## From eidetic_on_muniment (landed 2026-07-12, archived 2026-08-06)

- **mooting adopting `eidetic-fjall`.** It already rides muniment, so this is a
  manifest change rather than a port. The plan's third done-condition also
  named meerkat's suite as pending the genet ring-3 rename. That suite has no
  standing home to be pending *in*: meerkat was **decomposed**, not discarded,
  so the check it described now belongs to whichever of mere's crates or
  turnstone inherited the code it covered. The fourth, a correction owed to the boundary-pass plan's
  point 5, was already recorded there and needs nothing.

## From mere_turnstone_boundary_pass (landed 2026-07-09, archived 2026-08-06)

- **Splitting `session-runtime`'s mixed concerns.** The plan named the whole
  list: graph engram and session stores, wallet and identity, frame layout and
  tearout, browser content, engine profile and image stores, settings, and
  scripts. Only the settings slice is tracked today, by the
  [configuration ownership plan](../../archive_docs/2026-10-06_completed_plans/2026-08-06_configuration_ownership_settings_projection_plan.md);
  the rest is unowned. Worth naming before the crate is treated as settled.

  **Corrected 2026-10-06 (S14 pass):** `session-runtime` is now `pandect`
  (`441e70f0`, 2026-08-15).
- **Production journal persistence** (the G5 follow-on list). Until it lands,
  the delta vocabulary can still change without forcing a durable-log
  migration, which is the reason it was safe to defer and the reason it stops
  being safe once a real log exists.

## From host_wiring_grabbag (genet side complete 2026-06-12, archived 2026-08-06)

- **Four genet seams with no caller yet: G1.1 `on_wheel`, G1.2 transform
  hit-test, G1.3 pointer cancel, G2.3 keyboard escapes.** All eight seams
  landed genet-side; four were runway whose done-condition was adoption by
  meerkat. That condition did not evaporate when meerkat was decomposed, it
  moved: the adopter is now whichever surface inherited those callers, which is
  turnstone for the app-shell ones. Re-read against turnstone before assuming
  any of the four is still unadopted.


## 2026-10-06 archive pass (S14 pass, rulings S36 and S48 to S66)

Eighty-eight plans moved on 2026-10-06: 77 to
`archive_docs/2026-10-06_completed_plans/`, 7 to
`archive_docs/2026-10-06_superseded_plans/` and 4 to
`archive_docs/2026-10-06_retired_plans/`. Forty-six were plainly complete
(ruling S36 of the [stack seams plan](./2026-10-04_stack_seams_plan.md)); the
other forty-two were ruled in rounds 18 to 22 (S49 to S66). Each plan's open
items come from its phase C lane report, its ruling and any "Open, raised by
the S14 pass" note in its text, and were extracted before the move (S36, S48).

Unlike the sections above, this one lists every tail with its destination, so
the extraction can be checked against the archived plans:

- **here**: this backlog holds it (S48: no plain live owner).
- **owner: X**: a live lane's plan or a plan in another repository carries
  it. That plan was not edited here; its lane is told separately.
- **received by X**: the named Mere plan now carries a dated "Received
  2026-10-06" note for it.
- **carried by X**: the named plan already records it.
- **done** or **settled**: closed since the plan's status was written, with
  the evidence.

### Plainly complete (ruling S36)

#### From hagiograph_history_organ (H1 to H5 landed 2026-09-16)

Archived as [2026-10-06_completed_plans/2026-09-16_hagiograph_history_organ_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-16_hagiograph_history_organ_plan.md).

- **Promotion and retelling, the attention that lets an untold legend fade,
  and memorials handed to the voxel lane** (§3, "later, not planned here") —
  here. The design record is Isometry's
  `isometry/mesocosm/design_docs/2026-09-16_isoscape_family_plan.md`, which
  names them as hagiograph's later work.

#### From projection_refresh_and_surface_reuse (complete 2026-09-06)

Archived as [2026-10-06_completed_plans/2026-09-05_projection_refresh_and_surface_reuse_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-05_projection_refresh_and_surface_reuse_plan.md).

- **Persisted resource settings** (Pelt's `SurfaceResourcePolicy` has no
  serde) — here.
- **Background-work and texture-residency budgets** — here.
- **Independent appearances for engines other than Reader** — here.
- **Native frame performance and background CPU**, unmeasured — here.

#### From coop_lifecycle_parity (slices 1 and 2 done 2026-09-17 and 2026-09-18)

Archived as [2026-10-06_completed_plans/2026-09-16_coop_lifecycle_parity_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-16_coop_lifecycle_parity_plan.md).

- **The authoring-time judgement lane for revoked writers, and
  membership-gated sync** — carried by the I3 entry of the
  [suite census](../../2026-08-22_turnstone_suite_composition_and_capability_census.md),
  the plan's named lane owner, which records both as "recorded and not
  started".
- **A Turnstone revoke whose member's recipient is not registered locally** —
  here.
- **`coop`'s reference driver still declaring
  `receipts_duplicate_replay: false` for `turnstone_profile()`** — here.
- **The fixture's browser receipt for the new controls** — here.
- **Turnstone's empty `expires_at_ms` and `revocation.by` / `at_ms`** — here.

#### From fact_visualization_leaves (V0 to V2 landed 2026-09-06)

Archived as [2026-10-06_completed_plans/2026-09-06_fact_visualization_leaves_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-06_fact_visualization_leaves_plan.md).

- No open tail.

#### From multi_window (landed in meerkat, retired with it)

Archived as [2026-10-06_completed_plans/2026-06-10_multi_window_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-10_multi_window_plan.md).

- No open tail: MW4 to MW6 were superseded by the window composition plan,
  and MW3's step-4 part 2 retired with meerkat.

#### From modular_integration (superseded by the platform boundary plan)

Archived as [2026-10-06_superseded_plans/2026-06-02_modular_integration_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-06-02_modular_integration_plan.md).

- **S5's comms and sync** — carried by the
  [Murm peer runtime and Moot domain plan](./2026-07-12_murm_peer_runtime_and_moot_domain_plan.md)
  and the
  [Moot collections plan](../../moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md),
  as the plan's Findings route them.
- **S6's external content re-home** — routed to the scrying tile plan,
  itself archived in this pass; see its tails below.
- **S7's retirement and documentation reconciliation** — done under the
  platform boundary plan's P7 (closed 2026-09-06), also archived in this
  pass.
- **Per-family edge toggles and a per-family visibility UI** — here, after
  the consumer audit the plan asks for; pandect already persists
  `hidden_relations` (`crates/system/pandect/src/view_intent_store.rs`).

#### From multi_graph_activation (landed in meerkat, retired with it)

Archived as [2026-10-06_completed_plans/2026-06-09_multi_graph_activation_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-09_multi_graph_activation_plan.md).

- **Per-session engine-profile escalation** — here, with the engine profile
  boundary plan's v0b, which it waited on (that plan is retired in this pass,
  ruling S54).
- **The persona chip** — here; it is the same chip as the shellbar plan's
  F2.3 below.

#### From unified_document_host (closed 2026-06-23)

Archived as [2026-10-06_completed_plans/2026-06-17_unified_document_host_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-17_unified_document_host_plan.md).

- **The JSON-LD broadcast follow-on** — here.
- **The sighted-keyboard orrery focus-stop** — here.
- **Tiles to `platen-view`** — here.
- **N secondary orreries** — here.
- **Condition 1**, which lived in the orrery custom layout element plan —
  here, with that plan's tail below (it is retired in this pass, ruling S52).

#### From node_representation_arrangement (superseded 2026-06-23)

Archived as [2026-10-06_superseded_plans/2026-06-18_node_representation_arrangement_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-06-18_node_representation_arrangement_plan.md).

- **Representation work** — carried by the
  [node body face model plan](./2026-06-23_node_body_face_model_plan.md).
- **Who owns the arrangement axis** — here; the owner the status names, the
  graph signals layer plan, was archived on 2026-08-20.

#### From xilem_serval_control_adoption (landed in genet and meerkat)

Archived as [2026-10-06_completed_plans/2026-06-25_xilem_serval_control_adoption_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-25_xilem_serval_control_adoption_plan.md).

- No open tail: P3 is moot, and the deferred `role="radiogroup"` container
  was a `PaneItem` refinement that left with meerkat.

#### From athanor_steady_heat_actor (P1 retired with meerkat; P2 and the forgetting pass survive in mere-athanor)

Archived as [2026-10-06_completed_plans/2026-06-25_athanor_steady_heat_actor_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-25_athanor_steady_heat_actor_plan.md).

- **Scheduling the consolidation and forgetting passes** — carried by the
  [djinn family resident services plan](./2026-08-22_djinn_family_resident_services_plan.md),
  which names Athanor as a scheduled service (the crate's 2026-09-02 ruling:
  "scheduled, not resident").
- **The facet pass** — carried by the
  [alembic implementation plan](./2026-06-24_alembic_implementation_plan.md)'s
  slice D, subject to that plan's open question on who owns slice D's
  remainder.
- **Thresholds as settings** — here.

#### From burn_wgpu_flip (complete 2026-07-05)

Archived as [2026-10-06_completed_plans/2026-07-04_burn_wgpu_flip_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-04_burn_wgpu_flip_plan.md).

- **The browser model-ceiling headed receipt** — D2a to D2c passed on
  2026-08-22; the remaining rows are carried by the
  [distillery models plan](./2026-10-06_distillery_models_plan.md) (M1, M5).

#### From overmap_sessions_graph (rungs complete 2026-07-20)

Archived as [2026-10-06_completed_plans/2026-07-20_overmap_sessions_graph_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-20_overmap_sessions_graph_plan.md).

- **Stored-overmap promotion** — here.
- **Cross-session edges** (Murm and Moot's seam) — here.

#### From smolweb_host_integration (landed in meerkat, retired with it)

Archived as [2026-10-06_completed_plans/2026-06-28_smolweb_host_integration_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-28_smolweb_host_integration_plan.md).

- No open tail. The trust-posture gap is carried by the
  [smolweb fidelity plan](../../nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md)'s
  Workstream 2.

#### From event_log_timeline (superseded for implementation)

Archived as [2026-10-06_superseded_plans/2026-07-01_event_log_timeline_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-07-01_event_log_timeline_plan.md).

- No open tail; the
  [graph view curation plan](./2026-08-03_graph_view_curation_and_interaction_plan.md)
  is the implementation authority.

#### From family_repo_merges (superseded by the repo consolidation plan)

Archived as [2026-10-06_superseded_plans/2026-07-21_family_repo_merges_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-07-21_family_repo_merges_plan.md).

- **Archiving the seven donor GitHub repositories** — here, with its S14
  question: is it still a task, now that their crates live in Mere
  (`6a37de6b`)? Options: keep it as an open tail; record it as done or
  dropped. The repo consolidation plan, its other possible home, is archived
  in this pass.
- **The crates.io `eidetic` description reword** (P4's last bullet) — here.

#### From projection_proofs (P1 to P5 complete)

Archived as [2026-10-06_completed_plans/2026-07-21_projection_proofs_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-21_projection_proofs_plan.md).

- **The arrangement pull**, removed in `270172db` (2026-10-03) — owner: the
  [dynamics grammar plan](./2026-10-02_dynamics_grammar_plan.md), as the
  status already says.
- **P5's live radio facts** — here, with its S14 question: do retinue's
  position disclosure PD1 and PD2 count as the radio fact surface? Options:
  yes, and P5's live half becomes adapting it; no, it waits for a product to
  expose location facts.
- **The display-name drift** between the scenomise catalog ("L-system",
  "Kanban") and the canvas's `CANVAS_LAYOUT_STRATEGIES` (Fractal, Columns) —
  here, with its S14 question: which table is authoritative? Options: the
  canvas table, with the scenomise catalog renamed; the scenomise catalog,
  with the canvas table derived from it.

#### From capability_model (completed round)

Archived as [2026-10-06_completed_plans/2026-07-23_capability_model_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-23_capability_model_plan.md).

- **Follow-on 2, sub-delegation** (the endpoint's certificate at depth 1
  issuing a narrower one to the viewer) — here. It was gated on and tracked
  in the Graphshell remote projection host plan, archived in this pass, whose
  gate for it (G5b and G5c) has been met.

#### From scenograph_0_0_3_release (completed release plan)

Archived as [2026-10-06_completed_plans/2026-07-24_scenograph_0_0_3_release_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-24_scenograph_0_0_3_release_plan.md).

- No open tail; the published 0.0.3 artifacts stay the release baseline.

#### From knot_port (K0 to K7 complete)

Archived as [2026-10-06_completed_plans/2026-07-25_knot_port_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-25_knot_port_plan.md).

- **Whether "one immutable revision" is the rule for Knot's consumers** (its
  S14 question: one shared knot-editor revision across Turnstone, Mere and
  djinn's `knot-site`, or each consumer pins its own) — owner: knot-editor's
  copy of this plan (`knot-editor/design_docs/2026-07-25_knot_port_plan.md`).
  At this pass Mere's root manifest and djinn's `knot-site` both pin
  knot-editor `a5888dc6`.

#### From knot_authoring_consumer (Knot-owned work complete)

Archived as [2026-10-06_completed_plans/2026-07-27_knot_authoring_consumer_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-27_knot_authoring_consumer_plan.md).

- **An optional headed Turnstone receipt of selected clips from Genet's Livery
  and scripted producers** — owner: knot-editor's copy
  (`knot-editor/design_docs/2026-07-27_knot_authoring_consumer_plan.md`).
- **The "one immutable revision" pin rule** — owner: knot-editor's copy, as
  under the knot port plan above.

#### From knot_in_graphshell (K0 to K3 complete)

Archived as [2026-10-06_completed_plans/2026-08-02_knot_in_graphshell_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-02_knot_in_graphshell_plan.md).

- **Knot search S0 and S1** — owner: knot-editor's copy
  (`knot-editor/design_docs/2026-08-02_knot_in_graphshell_plan.md`), which
  carries it.
- **The "one immutable revision" pin rule** — owner: knot-editor's copy, as
  above.

#### From configuration_ownership_settings_projection (C0 to C6 complete; C7 deferred by design)

Archived as [2026-10-06_completed_plans/2026-08-06_configuration_ownership_settings_projection_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-06_configuration_ownership_settings_projection_plan.md).

- **Knot's move to pandect's `write_bytes_with_backup`** (the 2026-08-26
  follow-up under C3; knot-editor's settings write still removes, then
  renames) — owner: knot-editor, where Knot's settings code lives (no plan
  named).
- **C7's latecomers** (Isometry, Cleromancy) — here.

#### From esp_consolidation (E0 to E4 complete 2026-08-09)

Archived as [2026-10-06_completed_plans/2026-08-08_esp_consolidation_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-08_esp_consolidation_plan.md).

- **D2's tail** (physical GPU-allocation telemetry, larger capability bounds)
  — carried by the distillery models plan (M4, M5).
- **A real stacked-adapter row, and portable remote checkpoints** — carried by
  the distillery models plan (M4).
- **Lane 4, training** — spun out to the distillery v0 plan (carried by the
  distillery models plan) and the autodiff LoRA trainer plan (its tails
  below).
- **The stable Burn repin** — done: stable Burn 0.22.0 reached main on
  2026-10-06 (the
  [burn 0.22 migration plan](./2026-08-09_burn_0_22_migration_plan.md),
  §13.47).
- **Lane 3's `endpoint` backend** (external OpenAI-compatible endpoints) —
  here.
- **D1's render-versus-compute device policy**, open "until the
  resident-data consumer forces it" — here.
- **The communal lanes** (geist, shared adapter engrams, group-scale
  compute), entrance-gated behind the local trainer receipt and portable
  checkpoints — here; the
  [geist models brief](../research/2026-05-10_geist_models_brief.md) and the
  [communal compute tiers brief](../research/2026-06-10_communal_compute_tiers_brief.md)
  hold their design.

#### From family_shared_identity (adoption landed in five products)

Archived as [2026-10-06_completed_plans/2026-08-08_family_shared_identity_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-08_family_shared_identity_plan.md).

- **Hocket's return-to-own-identity path**, intentionally unbuilt (no caller
  has asked; Hocket now lives in woodshed) — here.

#### From wallet_carry_foldin (complete 2026-08-10)

Archived as [2026-10-06_completed_plans/2026-08-10_wallet_carry_foldin_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-10_wallet_carry_foldin_plan.md).

- **Pandect's `graph_codicil.rs` (838 lines) and `manifest_store.rs` (658
  lines), over the 600-line ceiling** — here.

#### From castellan_keeper_founding (complete)

Archived as [2026-10-06_completed_plans/2026-08-14_castellan_keeper_founding_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-14_castellan_keeper_founding_plan.md).

- **Splitting `keeper` into two features**, once a consumer wants views
  without the agent stack — here. The dramatis repo plan's cut through
  castellan may take it, but does not name it.
  **Amended 2026-10-06:** owner: the
  dramatis repo plan (`repos/dramatis/design_docs/2026-10-06_dramatis_repo_plan.md`),
  which took it in `73812e3a` and records, as a reading not ruled, that its
  rulings D14 and D15 make the split unnecessary.

#### From scenograph_absorption (S1 to S7 landed)

Archived as [2026-10-06_completed_plans/2026-08-22_scenograph_absorption_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-22_scenograph_absorption_plan.md).

- **Deliberate pin bumps for the scenograph-only consumers** (signalman-desktop,
  mesocosm, woodshed-core, turnstone, cleromancy), now against score 5 —
  here.
- **Whether seiche and `scenomise::relax` converge** — here.
- **Cambium's `graph_canvas` as a third node-link renderer** — here.

#### From terminology_and_crate_folds (landed 2026-09-02)

Archived as [2026-10-06_completed_plans/2026-08-31_terminology_and_crate_folds_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-31_terminology_and_crate_folds_plan.md).

- **Which seven consumer commits the status means** — here, with its S14
  question: correct the count to the four that Progress names, or identify
  the other three.

#### From autodiff_lora_trainer (complete 2026-09-03)

Archived as [2026-10-06_completed_plans/2026-09-02_autodiff_lora_trainer_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-02_autodiff_lora_trainer_plan.md).

- **The padding-with-mask follow-on** — here.
- **The burn `LoraAdapter` question** — here. The distillery models plan,
  which carries its parent distillery v0 plan's items, names neither.

#### From platform_boundary_and_repository_topology (P0 to P7 complete 2026-09-06)

Archived as [2026-10-06_completed_plans/2026-09-02_platform_boundary_and_repository_topology_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-02_platform_boundary_and_repository_topology_plan.md).

- **The 2026-09-09 games-wing renderer adoption follow-up** — here; no live
  renderer or games plan names it. Vello V1 to V4 stay owned by Netrender's
  upstream-ask note, and the 11 unverifiable claims stay with the
  [2026-09-06 remediation receipt](../../../support/doc-audit/d2/remediation_2026-09-06.md).

#### From recursive_query_experiments (E1 to E3 merged 2026-09-12)

Archived as [2026-10-06_completed_plans/2026-09-11_recursive_query_experiments_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-11_recursive_query_experiments_plan.md).

- **E2 phase 2, the relation delta journal** (gated, not started) — here.

#### From headed_automation (landed in meerkat, retired with it)

Archived as [2026-10-06_completed_plans/2026-07-07_headed_automation_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-07_headed_automation_plan.md).

- **`Code/testing/mere/scripts/mk-harness.ps1`, which still launches
  `meerkat.exe`** — here, with its S14 question: retire it beside the other
  retired drivers in `Code/testing/_archive/scripts/`, or repoint it at a
  current host's scenario runner.

#### From flora_tulpa_standing (landed 2026-09-02)

Archived as [2026-10-06_completed_plans/2026-08-31_flora_tulpa_standing_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-31_flora_tulpa_standing_plan.md).

- No open tail.

#### From apparatus_pane_and_theme_switcher (landed in meerkat, retired with it)

Archived as [2026-10-06_completed_plans/2026-06-08_apparatus_pane_and_theme_switcher_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-08_apparatus_pane_and_theme_switcher_plan.md).

- **A3, folding the settings overlay into apparatus** — here, or dropped; no
  host has asked.
- **Per-theme high-contrast node fills** — here. The
  [theme modes plan](./2026-07-05_theme_modes_plan.md) owns high-contrast mode
  and may take them.

#### From shellbar (landed in meerkat, retired with it)

Archived as [2026-10-06_completed_plans/2026-06-09_shellbar_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-09_shellbar_plan.md).

- **The ruling that the shellbar sits outside the frame tree, its edge a
  window preference** — here, as a host-agnostic design note.
- **F2.3's persona chip** — here; the same chip as the multi-graph plan's.

#### From repo_consolidation (C0 to C6 done; topology superseded 2026-09-02)

Archived as [2026-10-06_completed_plans/2026-07-23_repo_consolidation_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-23_repo_consolidation_plan.md).

- **Hocket's toolchain pin**, still 1.96.0 (a sibling repository) — here.
- **The four unpublished `graphshell-*` crates** — here. Their gate was the
  remote projection host's G5 to G7; G6 and G7 are now received by the
  Graphshell reference host plan (below).

#### From serval_as_host_xilem_serval (historical extraction plan; Stages 0 to 7 landed)

Archived as [2026-10-06_completed_plans/2026-05-27_serval_as_host_xilem_serval_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-05-27_serval_as_host_xilem_serval_plan.md).

- No open tail.

#### From host_p2p_wiring (superseded by the Murm/Moot plan)

Archived as [2026-10-06_superseded_plans/2026-06-03_host_p2p_wiring_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-06-03_host_p2p_wiring_plan.md).

- No open tail of its own. S5.3, the real comms surface, had moved to the
  comms shell plan, archived in this pass; it is that plan's P6 host below.

#### From index_burn_lift (P1 to P3 complete for the flat index)

Archived as [2026-10-06_completed_plans/2026-07-08_index_burn_lift_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-08_index_burn_lift_plan.md).

- **HNSW, Path B** — with the mere-side twin's P4 in the S59 subsection
  below.
- **GPU top-k returning `[Q,k]`** — here.
- **A per-shape warm-on-first-use pass** — here.

#### From isometric_orrery_camera (complete 2026-06-24)

Archived as [2026-10-06_completed_plans/2026-06-22_isometric_orrery_camera_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-22_isometric_orrery_camera_plan.md).

- **Yaw and tilt persistence**, which no host now writes — here.

#### From physics_scenes_and_tangibility (complete 2026-06-24)

Archived as [2026-10-06_completed_plans/2026-06-22_physics_scenes_and_tangibility_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-22_physics_scenes_and_tangibility_plan.md).

- **A Mere or Turnstone host scene picker and tangibility binding** — here.
- **A bounds-drain** — here.
- **A fluid emitter** — here.
- **Reusing aether's `CouplingForce`** (aether is now numen) — here.
- **Re-applying tangibility on node add** — here.

The physics catalog plan is not this plan's successor, so none of these is
carried there.

#### From browser_extension_companion (superseded 2026-07-27)

Archived as [2026-10-06_superseded_plans/2026-06-23_browser_extension_companion_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-06-23_browser_extension_companion_plan.md).

- **The extension product** — carried by the
  [Graphshell reference host plan](./2026-07-27_graphshell_reference_host_plan.md)
  (H4, H5).
- **The companion, smolweb, p2p and federation forward arc** — here, with its
  S14 question: does it live in the Graphshell reference host plan, or in a
  home of its own?

#### From orrery_browser_lane (superseded 2026-07-27)

Archived as [2026-10-06_superseded_plans/2026-06-24_orrery_browser_lane_plan.md](../../archive_docs/2026-10-06_superseded_plans/2026-06-24_orrery_browser_lane_plan.md).

- No open tail; the v1 extension shipped as Graphshell H5.

#### From meerkat_promotion_pass (P1 to P7 and the P8 slices landed)

Archived as [2026-10-06_completed_plans/2026-07-02_meerkat_promotion_pass_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-02_meerkat_promotion_pass_plan.md).

- No open tail; the P8 remainder is moot without meerkat.

#### From eidetic_reorg (landed 2026-08-12)

Archived as [2026-10-06_completed_plans/2026-08-12_eidetic_reorg_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-12_eidetic_reorg_plan.md).

- **Done-condition 4, the leverage census's `mere-embed` row** — done in this
  pass: the row in the
  [leverage census](../../2026-08-10_leverage_census_brief.md) now carries a
  dated note that the reorg landed (`8595cd38`).

### Ruled in rounds 18 to 22 (rulings S50 to S66)

#### From scrying_tile (S50)

Archived as [2026-10-06_completed_plans/2026-06-10_scrying_tile_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-10_scrying_tile_plan.md).

- **X2's chrome round-trip** — here.
- **X3's durable pin** (pandect's `compat_mode` as the source of truth) —
  here.
- **X4, macOS and Linux** — here.
- **The registry fold-in**, shared with the engine picker plan's Phase 5 —
  here; the engine picker stays active (S50) and may take it.
- **Overlay scrollbars, the frame-arrival wake, and `content_generation`** —
  here.
- **The modular integration plan's S6, web content through scrying** — here,
  with the above.

The nearest live host is Pelt, which hosts scrying surfaces
(`ports/pelt/desktop/scrying_receipt.rs`).

#### From render_ladder_and_extraction (S50)

Archived as [2026-10-06_completed_plans/2026-06-23_render_ladder_and_extraction_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-23_render_ladder_and_extraction_plan.md).

Mark, archiving it (S50): **"render ladder needs rethinking."** These tails
wait on that rethink, not only on a host.

- **Keyboard dispatch** — here.
- **Interactive-region refinement** — here.
- **Routing article `main_text` into the eidetic corpus** — here.
- **A `pump()` before the scripted extract** — here.

#### From comms_shell (S51)

Archived as [2026-10-06_completed_plans/2026-06-05_comms_shell_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-05_comms_shell_plan.md).

- **A host for the P6 docked comms pane** — here, with its S14 question:
  `crates/shell/comms` has had no consumer since meerkat left; does it stay,
  as the comms domain a future host's pane consumes, or retire? The host p2p
  wiring plan's S5.3 is this item.
- **P6c's live pieces**: the misfin server socket, networked murm cabals and
  the misfin send path — here.
- **P3b′**: status 63, the from-line, the redirect and rate-limit codes, and
  the `MisfinServer` worker — here.
- **Mooting protocol adapters** (Matrix, Nostr) as those backends land —
  here.
- **The contact model**, deferred 2026-06-15 — here; the
  gaz founding plan (`repos/dramatis/design_docs/2026-08-08_gaz_founding_plan.md`)
  may own it, but not plainly.

#### From tearout_gestures (S51)

Archived as [2026-10-06_completed_plans/2026-06-24_tearout_gestures_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-24_tearout_gestures_plan.md).

- **G5 copy and move on Turnstone** — here.
- **G6 cascade on Turnstone** — here.
- **The ambiguous no-modifier drag**, whose routed home (the notification
  subsystem plan) was retired on 2026-09-02 — here.

The portable tiles plan, which stays active (S51), names its P2 against these
gestures.

#### From orrery_graph_intelligence (S52)

Archived as [2026-10-06_completed_plans/2026-07-06_orrery_graph_intelligence_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-06_orrery_graph_intelligence_plan.md).

- **The BERT `semantic-embeddings` provider upgrade** — here.
- **The off-thread embedding actor** (raw-body text) — here.
- **A caller for pictograph's `set_content_affinity`**, which has none in Mere
  or Turnstone — here.

#### From orrery_custom_layout_element (S52; retired, parked with no code)

Archived as [2026-10-06_retired_plans/2026-06-23_orrery_custom_layout_element_plan.md](../../archive_docs/2026-10-06_retired_plans/2026-06-23_orrery_custom_layout_element_plan.md).

- **Mechanism A**, the genet-side ask for host-driven transform setting, to
  un-park only if that becomes a performance or correctness problem — here,
  genet's.
- **The unified document host plan's condition 1**, which this plan held —
  here, with it.

#### From graph_query_layer (S53)

Archived as [2026-10-06_completed_plans/2026-06-18_graph_query_layer_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-18_graph_query_layer_plan.md).

- **#3, `CONSTRUCT` / `DESCRIBE` into a subgraph, and the
  `QueryResults::Graph` error** — received by the
  [graph semantics plan](./2026-10-04_graph_semantics_plan.md) at P4, where
  its ruling 3 had already put #3.
- **#1, a shared `mapping` module** — here.
- **The rest of #2, RDF-star edge metadata** (the statement-metadata reifiers
  landed) — here.
- **#4, a results pane** — here.
- **#6, semantic-surface JSON-LD in view** — here; its gate, the unified
  document host plan's Phase 2, is closed.
- **#7, a synced-mirror store**, only if query frequency demands it — here.
- **#8, federation, `UPDATE` and HDT** — here.

#### From petgraph_rdf (S53)

Archived as [2026-10-06_completed_plans/2026-06-18_petgraph_rdf_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-18_petgraph_rdf_plan.md).

- **The JSON-LD shaper gaps, the `skip_serializing_if` follow-up, the
  `ExampleOf` and `Summarizes` vocabulary rows, the raw-IRI `Semantic` edge
  path, and dropping `dep:oxigraph`** — received by the graph semantics plan.
- **Whether Phase 3 is done** (its S14 question: done on the shipped
  spareval path, with ruling 6's adapter a new P2 requirement; or reopened,
  carried by P2) — received by the graph semantics plan, whose P2 carries
  the adapter either way.
- **Phase 4, the interned slotmap kernel**, gated on the 2026-07-06 footprint
  measurement — here. The status lists it as gated rather than open, so it is
  not among the open items S53 sent on.

#### From graph_write_path_migration (S53)

Archived as [2026-10-06_completed_plans/2026-07-01_graph_write_path_migration_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-01_graph_write_path_migration_plan.md).

- **The `apply.rs` split** (1938 lines; there is no `delta.rs`) — here, with
  its S14 question: still wanted, or closed with the file staying whole?

#### From graph_delta_capture_apparatus_stats (S53)

Archived as [2026-10-06_completed_plans/2026-07-02_graph_delta_capture_apparatus_stats_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-02_graph_delta_capture_apparatus_stats_plan.md).

- **A canned regression log** — here.
- **Per-table stats** — here, with its S14 question: re-scope them into
  `crates/domain/apparatus`, or leave them retired with meerkat, with
  `GraphJournal` and the graph view curation plan as the successors?

#### From engine_profile_boundary (S54; retired)

Archived as [2026-10-06_retired_plans/2026-05-14_engine_profile_boundary_plan.md](../../archive_docs/2026-10-06_retired_plans/2026-05-14_engine_profile_boundary_plan.md).

- **v0b, per-engine profile wiring** — here. v0a's path-resolution primitive
  is `crates/system/pandect/src/engine_profile_store.rs`, uncalled outside
  pandect; the multi-graph plan's per-session escalation (above) waited on
  this.
- **The two `EngineProfileBinding` types** (pandect's scope enum, inker's
  `user_data_dir` struct) — here, with its S14 question: rename one, declare
  inker's struct the v0b seam, or merge them.

#### From session_service_runner (S54; retired)

Archived as [2026-10-06_retired_plans/2026-05-14_session_service_runner_plan.md](../../archive_docs/2026-10-06_retired_plans/2026-05-14_session_service_runner_plan.md).

- **v0b, real session workers** — here. The v0a trait, `NullRunner` and
  `InMemoryRunner` are in `crates/system/pandect/src/session_service_runner.rs`,
  unused outside pandect.
- **How `SessionServiceRunner` relates to djinn's resident scheduler** (the
  dramatis tier architecture's ruling 6 makes feed polling a djinn job) —
  here, with its S14 question: the runner stays the session-worker contract;
  djinn's scheduler supersedes it; or both, scoped apart.

#### From bounty_verification_economy (S54; retired, nothing built)

Archived as [2026-10-06_retired_plans/2026-06-30_bounty_verification_economy_plan.md](../../archive_docs/2026-10-06_retired_plans/2026-06-30_bounty_verification_economy_plan.md).

- **The bounty design itself** (`BountyEvent`, `ResultSpec`,
  `MootEpochHeader`, `WorkReceipt`, `RoleWitness`, over the existing
  `proofs::Commitment` and `proofs::Digest`; its standing ledger is now mien's
  Standing) — here.

#### From capture_provenance_consent (S55)

Archived as [2026-10-06_completed_plans/2026-06-26_capture_provenance_consent_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-26_capture_provenance_consent_plan.md).

- **A C4 consent gate in a live host** — owner: Turnstone's page capture plan
  (`turnstone/design_docs/2026-08-28_page_capture_plan.md`), for its trail
  memory, whose C4 gate is a no-op stub; and received by the Graphshell
  reference host plan at H5, whose `HistoryCapturePolicy` is disabled by
  default.
- **C4's default** (its S14 question: `Full`, as this plan's C4 shipped, or
  disabled and opt-in, as Graphshell's policy is) — with the gate, to both
  owners.
- **Phase 9 federation promotion and consumption** — carried by the
  [eidetic deferred phases plan](../../eidetic_docs/implementation_strategy/2026-06-09_eidetic_deferred_phases_plan.md).
- **Forget's provenance cleanup** — here.
- **C3's excerpt, summarize and generated-node cases** — here.
- **The plan's other open questions**: the legality policy for federating a
  distillation of crawled text, candidate-set observability, and recorder
  granularity against `node-lineage` — here.

#### From insigne_proofs (S56)

Archived as 2026-10-06_completed_plans/2026-09-23_insigne_proofs_plan.md (`repos/dramatis/design_docs/archive_docs/2026-10-06_completed_plans/2026-09-23_insigne_proofs_plan.md`).

- **The remaining sibling repins: Hocket, Woodshed and mer3ly** (Turnstone
  repinned in `d6b62ad`) — owner: the
  dramatis repo plan (`repos/dramatis/design_docs/2026-10-06_dramatis_repo_plan.md`).

#### From identity-vault-ssh-agent (S56)

Archived as [2026-10-06_completed_plans/2026-07-22_identity-vault-ssh-agent_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-22_identity-vault-ssh-agent_plan.md).

- **V4, broader item types** — owner: the
  chatelaine and CXF import plan (`repos/dramatis/design_docs/2026-10-01_chatelaine_cxf_plan.md`).
  S56 answers the plan's S14 question: yes, V4 moves to chatelaine.
- **V5, sync** (no replicated `IdentityStorage` exists) — here.
- **The V3 follow-ons**: a passphrase prompt, `generate`, and the ShortTtl
  relock — here.
- **The interim install scripts** (`install-agent-windows.ps1`,
  `install-agent-linux.sh`, `install-agent-macos.sh` in personae; the Windows
  header still says "Until then") — here, with its S14 question: retire them
  now that castellan hosts the agent, or keep them as the standalone install
  path with the header reworded.

#### From castellan_otp (S56)

Archived as [2026-10-06_completed_plans/2026-08-10_castellan_otp_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-10_castellan_otp_plan.md).

- **Code presentation, the admitted approval surface, and the Secret Service
  serve policy** — here. The plan names the device resident consolidation
  plan; the dramatis repo plan moves `otp/`'s plain types (DR-A) but does not
  take these slices.
- **Credential replication between persona devices**, with per-device
  freshness evidence — here; the dramatis repo plan does not take it.
- **CXF import** — owner: the chatelaine and CXF import plan (its P4).

#### From dramatis_tier (S57)

Archived as 2026-10-06_completed_plans/2026-08-10_dramatis_tier_plan.md (`repos/dramatis/design_docs/archive_docs/2026-10-06_completed_plans/2026-08-10_dramatis_tier_plan.md`).

- **The `dramatis` facade**, ruled real on 2026-10-01 for repos outside Mere
  and unbuilt — owner: the dramatis repo plan, whose ruling D1 makes dramatis
  the product.
- **Checking chatelaine's 0.0.2 republish**, which could not be checked
  offline — here.

#### From persona_wallet_carry_layer (S57)

Archived as [2026-10-06_completed_plans/2026-06-25_persona_wallet_carry_layer_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-25_persona_wallet_carry_layer_plan.md).

- **Lock and unlock follow-through, and non-Windows startup unlock backends**
  — owner: the
  [vault lock plan](../../dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md),
  as the status already says.
- **The PAKE/QR chrome and transport UI, copy-mode export and import,
  epoch-history usage beyond the current epoch, and a migration pass for
  cleartext private blobs** — owner: the dramatis repo plan (S57: the carry
  gaps).

#### From participant_gate_packs (S57)

Archived as [2026-10-06_completed_plans/2026-07-17_participant_gate_packs_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-17_participant_gate_packs_plan.md).

- **B5's tessera receipt on a live moot** — here.
- **The mere-native meadowcap structural-cap layer** — here.
- **"Broader consumer admission"** — here, with its S14 question: define it
  as a phase with a done-condition, or drop it as met by the three actor
  kinds (script, wasm component, moot peer) that now petition through the one
  gate?

#### From the four ML plans (S58, consolidated under S63)

- **distillery_v0** — Archived as [2026-10-06_completed_plans/2026-08-12_distillery_v0_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-12_distillery_v0_plan.md). Open items carried by the
  [distillery models plan](./2026-10-06_distillery_models_plan.md).
- **inference_provider** — Archived as [2026-10-06_completed_plans/2026-07-05_inference_provider_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-05_inference_provider_plan.md). Open items carried by the distillery models
  plan.
- **browser_model_ceiling_probe** — Archived as [2026-10-06_completed_plans/2026-08-09_browser_model_ceiling_probe_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-09_browser_model_ceiling_probe_plan.md). Open items carried by the
  distillery models plan.
- **mesh_host_lanes** — Archived as [2026-10-06_completed_plans/2026-08-09_mesh_host_lanes_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-09_mesh_host_lanes_plan.md). Open items carried by the distillery models
  plan, except **reliability and reputation accounting**, which that plan
  leaves out of scope "until a lane gives standing a purpose" — here. The kith
  lane the mesh plan gave it retired on 2026-09-02.

#### From workbench_component (S60)

Archived as [2026-10-06_completed_plans/2026-08-31_workbench_component_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-31_workbench_component_plan.md).

- **S3, Turnstone's compositor walking the shared tree** (its pin,
  `bd5912fb`, is satisfied; adoption has not started) — owner: Turnstone's
  pane registry plan
  (`turnstone/design_docs/2026-08-08_pane_registry_and_graph_panes_plan.md`,
  A4).
- **Pelt onto the shared bar names, retiring the `frisket-*` aliases and
  `TabBarNames`** — here.
- **Frisket's names made `pub(crate)`** — here.
- **The float view child through `PortableKeyed`** — here.
- **A physical screen-reader (Narrator) tear-out receipt** — here.

#### From knot_shared_surface_and_port_contribution (S60)

Archived as [2026-10-06_completed_plans/2026-08-24_knot_shared_surface_and_port_contribution_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-24_knot_shared_surface_and_port_contribution_plan.md).

- **F0** — owner: knot-editor's application workspace plan
  (`knot-editor/design_docs/2026-09-05_knot_application_workspace_plan.md`),
  which knot-editor's copy of this plan names as F0's authority.
- **The pointer-capture gap** — owner: Turnstone (its T lane).

#### From host_ui_zoom (S60)

Archived as [2026-10-06_completed_plans/2026-09-03_host_ui_zoom_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-09-03_host_ui_zoom_plan.md).

- **Z5's 820-versus-1040 design figure** — owner: Isometry's genet host
  migration plan (`isometry/design_docs/2026-09-02_genet_host_migration_plan.md`),
  which records it as Mark's call.
- **Woodshed's zoom-1.0 headed receipt** — here.
- **Consumer adoption** (Turnstone uses neither the Cambium host nor
  `HostOptions`) — here.

#### From component_catalog_growth (S60)

Archived as [2026-10-06_completed_plans/2026-07-15_component_catalog_growth_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-15_component_catalog_growth_plan.md).

- **C5's second-application reuse of `summary_body`** (only Isometry reuses
  it) — here.
- **The catalog's Decisions and promotion bar**, which need a standing home
  now that this plan is history — here.

#### From projection_receipts (S61)

Archived as [2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md).

- **Wave 3: FT9 (derived-mark integrity, receipt 3) and FT10 (field
  distinction, receipt 6)**, gated on a field-data consumer that does not
  exist — owner: the
  [projection grammar adoption plan](./2026-08-15_projection_grammar_adoption_plan.md).

#### From native_smolweb_rendering (S61, S66)

Archived as [2026-10-06_completed_plans/2026-06-27_native_smolweb_rendering_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-27_native_smolweb_rendering_plan.md).

- **Mere host integration and interactive eyeballing** — received by the
  smolweb fidelity plan, under S66: the EngineDocument lane is the single
  smolweb render path; the fidelity plan enriches the parse ASTs and adds the
  block kind EngineDocument lacks (a typed-column block first); and the
  per-format views in `cambium::nematic` retire, as a follow-on code task.
- **Which render design stands** (its S14 question) — settled by S66, after
  S62: this plan's two-family view model is historical.

#### From graphshell_remote_projection_host (S61)

Archived as [2026-10-06_completed_plans/2026-07-22_graphshell_remote_projection_host_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-22_graphshell_remote_projection_host_plan.md).

- **G6, composing several applications, and G7, product pulls and
  constrained management** — received by the Graphshell reference host plan,
  into its H-series.
- The capability model's sub-delegation and the repo consolidation plan's
  `graphshell-*` publication, which this plan gated, are listed under those
  plans above.

#### From knot_publishing_protocol (S61)

Archived as [2026-10-06_completed_plans/2026-08-07_knot_publishing_protocol_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-07_knot_publishing_protocol_plan.md).

- **Phase B: its specification and the `knot-protocol` crate** — owner:
  knot-editor's copy
  (`knot-editor/design_docs/2026-08-07_knot_publishing_protocol_plan.md`),
  canonical under S61.
- **Whether the Mark read adapter counts as Phase B's entry** (its S14
  question) — owner: knot-editor's copy.
- **Which copy is canonical** (its S14 question) — settled by S61: Knot's.

#### From spatial_compute (S64)

Archived as [2026-10-06_completed_plans/2026-08-13_spatial_compute_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-13_spatial_compute_plan.md).

- **The Nexus adoption watch** (tactile body count outgrowing CPU rapier) —
  owner: the [conatus engine plan](./2026-08-22_conatus_engine_plan.md), which
  holds the Nexus-decomposition ruling.
- **The lease's promotion trigger** (a shipped producer and consumer) — here.
- **Retiring the local rust-gpu fork by re-checking plain upstream** — here.

#### From tactile_tier (S64)

Archived as [2026-10-06_completed_plans/2026-08-14_tactile_tier_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-14_tactile_tier_plan.md).

- **T4, piles** — here.
- **T5, the mere profile** — here. It overlaps the physics catalog's named
  profiles and the dynamics grammar plan's G4 `DynamicsSpec` saved in
  `SavedSceneV1`, neither of which cites it; its S14 question asked whether
  to fold it into G4, fold it into the catalog's profiles, or keep it
  separate.

#### From derived_faces (S64)

Archived as [2026-10-06_completed_plans/2026-08-28_derived_faces_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-28_derived_faces_plan.md).

- **§7, editing**: tier 1 parameter editing and tier 2 vector editing — here.
- **§8's branding gap**: three apps ship no icon, and genet still ships
  Servo's — here.
- **Pictograph's manifest version**, which names source that differs from
  the published 0.2.0 — here, with its S14 question: bump it now, or leave it
  until the next publish?

#### From event_model_convergence (S64)

Archived as [2026-10-06_completed_plans/2026-06-01_event_model_convergence_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-01_event_model_convergence_plan.md).

- **Per-interface event subclasses** (`createEvent` still returns a base
  `Event`) — here, genet's.
- **`currentTarget` on the native side** — here, genet's.
- **Which copy is canonical** (its S14 question) — settled by S64: genet's
  copy stands as the live one.

#### From reticulum_transport (S65)

Archived as [2026-10-06_completed_plans/2026-06-29_reticulum_transport_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-29_reticulum_transport_plan.md).

- **A forged legacy-binding test** — here.
- **A README section on ALPN mapping, announce discovery and binding** —
  here.
- **The Phase 3 decision** (the Direction trigger has fired; the feature is
  default-off) — here. Retinue's own plan covers the backend.
- **Whether the reference-discipline section restates its rule or points at
  retinue's notice** (its S14 question) — here.

#### From doc_policy_consolidation (S65)

Archived as [2026-10-06_completed_plans/2026-08-24_doc_policy_consolidation_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-24_doc_policy_consolidation_plan.md).

- **Merging genet's `docs/` into its doc tree**, never scheduled — here,
  genet's.
- **Where the 2026-09-03 return of the three area roots is tracked** (its S14
  question) — settled by S65: DOC_POLICY records it, and its pointer to this
  plan carries a dated amendment.

### For the S59 critical pass (Mark: these need a good critical pass and to be turned into a plan if needed; Eidetic needs development now that identity has a clear path)

Ruling S59 archived these four with their open items held for one critical
pass, which turns them into a plan if warranted; its reading is that the pass
is a new objective, with its own assessment, after phase C.

#### From eidetic_browsing_derivation

Archived as [2026-10-06_completed_plans/2026-06-12_eidetic_browsing_derivation_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-06-12_eidetic_browsing_derivation_plan.md).

- **E2's done-condition**: Mark's run over a real exported history — here,
  for the pass.
- **E2's three open questions**: per-persona traces, the home of
  co-occurrence, and the owner of the geist's fused query (the deferred phases
  plan or the search surface wiring plan) — here, for the pass.
- **E5's remainder**: W3's reports and the corridor strip; the
  [search surface wiring plan](./2026-08-12_search_surface_wiring_plan.md)
  owns W3 — here, for the pass.

#### From redb_opfs_feasibility

Archived as [2026-10-06_completed_plans/2026-08-22_redb_opfs_feasibility_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-22_redb_opfs_feasibility_plan.md).

- **Adoption and crash-atomic creation**, both open; Safari and WKWebView
  were not run — here, for the pass.
- **§7.3 items 3, 5, 6 and 7, the Web Locks posture, and the transact note**
  — here, for the pass.
- **Whether adoption requires the adapter to implement `Backend::transact`**
  (muniment gained it in `9ba9f790`, defaulting to `NotTransactional`), its
  S14 question: yes, adoption requires it; or the default is acceptable for
  this backend — here, for the pass. The eidetic deferred phases plan's Phase 7 touches this area
  but does not plainly own it.

#### From intel_vector_index_burn_lift (the mere-side twin)

Archived as [2026-10-06_completed_plans/2026-07-06_intel_vector_index_burn_lift_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-07-06_intel_vector_index_burn_lift_plan.md).

- **Consumer routing**: no crate enables `index-burn`, so affinity, recall
  and canvas search do not route to the accelerators. Its S14 question: is
  Path A closed (routing is each caller's one-line check), or does P3 stay
  open until they route? Here, for the pass.
- **P4, HNSW (Path B)**, deferred, with the intel-side index burn lift plan's
  Path B above — here, for the pass.

#### From receipt_artifacts_replication

Archived as [2026-10-06_completed_plans/2026-08-10_receipt_artifacts_replication_plan.md](../../archive_docs/2026-10-06_completed_plans/2026-08-10_receipt_artifacts_replication_plan.md).

- **R1's done-condition**: a receipt browsable on a second paired device;
  §6.5's remote runs were ingested on one machine — here, for the pass.
