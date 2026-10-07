# Batch 42 — S14 pass, phase B4 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-07-23_capability_model_plan.md | historical-marked | no | 12 | 9 | 3 | 0 |
| mere_docs/implementation_strategy/2026-07-24_low_power_managed_network_plan.md | current | no | 12 | 9 | 3 | 0 |
| mere_docs/implementation_strategy/2026-07-24_scenograph_0_0_3_release_plan.md | historical-marked | yes | 9 | 8 | 0 | 1 |
| mere_docs/implementation_strategy/2026-07-25_knot_port_plan.md | historical-marked | no | 8 | 4 | 4 | 0 |
| mere_docs/implementation_strategy/2026-07-27_commons_calls_plan.md | current | no | 8 | 7 | 1 | 0 |
| mere_docs/implementation_strategy/2026-07-27_graphshell_reference_host_plan.md | current | yes | 10 | 8 | 2 | 0 |
| mere_docs/implementation_strategy/2026-07-27_knot_authoring_consumer_plan.md | historical-marked | no | 8 | 6 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-02_knot_in_graphshell_plan.md | historical-marked | yes | 11 | 8 | 3 | 0 |
| mere_docs/implementation_strategy/2026-08-03_graph_view_curation_and_interaction_plan.md | current | yes | 8 | 6 | 1 | 1 |
| mere_docs/implementation_strategy/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md | current | no | 12 | 10 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-06_configuration_ownership_settings_projection_plan.md | current | yes | 10 | 6 | 4 | 0 |
| mere_docs/implementation_strategy/2026-08-08_esp_consolidation_plan.md | historical-marked | no | 11 | 8 | 2 | 1 |
| **Totals** |  |  | **119** | **89** | **27** | **3** |

**Totals: 12 docs, 119 claims checked (89 holds, 27 stale, 3 unverifiable), 24 contradictions; 7 status lines wrong.**

Audit base: Mere `535bca11` (2026-10-05). Sibling repositories were read at
their HEADs that day, genet also at Mere's pin `bd3e8861`. `archive_docs/` is
excluded. No cargo command was run.

This batch belongs to the stack seams plan's S14 pass (rulings S14 and S33),
phase B: the plans the early-September snapshot judged, re-judged against the
tree. Each record here supersedes the snapshot's record for its plan (ruling
S34). A read-only subagent (opus) drafted the records; a second, independent
read-only subagent (opus) then tried to refute every stale claim and checked
every status quote, and its corrections are applied below; this session
re-checked a sample directly. The "holds" counts are the draft's. Nothing in
the plans changes here; corrections are phase C's, and plans found complete
are extracted and archived there (ruling S36).

Checked directly in this session: `5335a869` deleted `crates/capability`,
absent at the base (capability model); genet `44e291afe8b` is an ancestor of
Mere's pin `bd3e8861` (knot port); `d69be378` deleted the sibylla and vates
shims, and `crates/intel` holds only eidetic-search, esp, mora-cmudict and
reference-data (ESP). The verifier confirmed the rest, found 5 items true
only in part (the capability record's attribution to the status line, the
low-power plan's last word on wake, the knot port's merge date, the
reference host's decision 3, the config plan's genet `tile.rs`) and refuted
none; those records are corrected.

## mere_docs/implementation_strategy/2026-07-23_capability_model_plan.md

- disposition: historical-marked
- status line: "Status: completed round: typed delegation, revocation, Moot authorization, peer use, and the D3b extraction landed; later consumer-driven extensions remain separate work." — accurate: no
- claims checked: 12 — holds: 9, stale: 3, unverifiable: 0

### Stale claims

- **D3b extraction.** The 2026-08-18 Progress entry (lines 621-627) says the dependency-free `mere-capability` crate owns `Capability`/`Cap`/`ScopePath`/`Mode`/`FacetNamespace`, that `servitor::cap` and `servitor::grant::Mode` are compatibility re-exports, and that gemot depends on the leaf directly; the status line's "the D3b extraction landed" is true as history but omits the fold.
  - In the tree, commit `5335a869` (2026-09-23, an ancestor of the base) folded the crate back into servitor. It deleted `crates/capability` and removed the gemot dependency.
  - The algebra is now defined in `crates/servitor/src/cap.rs` (the `Cap` enum and `Cap::Facet` at line 159; `FacetNamespace` at 115).
  - `crates/servitor/src/grant.rs:20` re-exports `Mode` from `crate::cap`.
  - Gemot imports `servitor::cap::{Cap, Mode}` (`typed_authorization.rs:46`); `gemot/Cargo.toml:33` has `servitor.workspace = true`.
  - The plan was last edited at 0e031fa5 (2026-09-22), the day before the fold.
- **"personae owns the delegation machinery"** (D3b, F5). Since `5364dfa0` (2026-09-24, ancestor), the statement and verification types live in insigne: `crates/dramatis/insigne/src/delegation.rs:211`, `:300`. Personae keeps only issuing (`personae/src/delegation.rs` header). Servitor depends on `insigne` with the `verify` feature (`servitor/Cargo.toml:12`).

### Contradictions

- DOC_README.md:196 says "D3b's algebra now lives in the dependency-free `mere-capability` leaf". That contradicts the tree and DOC_README.md:63, the crate consolidation entry, which says "`mere-capability` into `servitor::cap`".
- Other active docs still name the leaf as current: `technical_architecture/2026-07-18_one_node_facets_layer_map.md:151,157` and `2026-08-16_facet_signaling_and_control_loops.md:209,278,283`.

### Recommended action

- Add a dated 2026-09-23 annotation to the 2026-08-18 entry and the status line recording the fold (5335a869).
- Amend D3b's ownership sentence to name insigne for statements and checks.
- Fix DOC_README:196.
- The round is complete and follow-on 2 (sub-delegation) is tracked in `2026-07-22_graphshell_remote_projection_host_plan.md:951`, so this is an archive candidate under DOC_POLICY §8.

### Notes

- Checked: `git merge-base --is-ancestor` for 04529a59, 5335a869 and 5364dfa0; `crates/servitor/src/{cap,grant,lib,gate,delegation}.rs` (`UnauthorizedFacet` at gate.rs:147, `revoke_root_grants` at delegation.rs:245); `crates/moot/gemot/src/moot/typed_authorization.rs:63,112,125`.
- In turnstone: `src/identity.rs:64` (`RootIdentity`), `src/action.rs:540` (`UninstallDenizen`), `src/denizen.rs:452,787`. `repos/servitor` is absent.

## mere_docs/implementation_strategy/2026-07-24_low_power_managed_network_plan.md

- disposition: current
- status line: "Status: in execution (2026-07-27). V1-V8 have landed; V7's carrier matrix passes over Memory, real p2panda/Iroh, Reticulum/TCP, and headed Reticulum/direct-PHY RF. Murm's accept path consumes the carrier's accepted session directly. The V0/V2 power and sleep bench remains open." — accurate: no
- claims checked: 12 — holds: 9, stale: 3, unverifiable: 0

### Stale claims

- **The status line presents the V0/V2 bench as the only open work.** V9 (`NodeOffer`), V10 (inventory-only replication) and V11 (`InterfaceObservation`, failover, bonding) have no code at base: zero grep hits in Mere's `crates/` and `ports/`, nor in Retinue (V11's repository) for `NodeOffer`, `InterfaceObservation`, bonding or failover. Completion conditions 5–7 are unmet and not mentioned.
- **"Every power and wake claim remains UNPROVEN pending the V0 bench"** (2026-07-25 entry, lines 897-898; the 2026-07-27 lines 1008-1010 and 1027-1029, "remains a V0/V2 power and wake proof", are the plan's last word and equally overtaken). Retinue's `design_docs/2026-07-29_v4_light_sleep_rf_wake_acceptance.md` reads "partially accepted on hardware": Light-sleep resumes without reset, and one direct RF wake was proven end to end. Current and energy remain unmeasured, so V0/V2 is still open.
- **D5/Ownership: "Personae supplies delegation and revocation proof primitives".** Those types moved to insigne in 5364dfa0. `crates/system/notochord/Cargo.toml:24` depends on `insigne` with `features = ["verify"]`.

### Contradictions

- The status line says "V1-V8 have landed". The body (2026-07-25) says V1 and V2 are only "compile-verified", and their headed proofs are deferred.
- DOC_README.md:177 cites "the Commons direct-PHY receipt … 1,177-byte encrypted operation", which this plan does not record. The plan cites the Murm direct-PHY receipt.

### Recommended action

- Re-date the status line.
- State V9–V11 explicitly as not started, or extract them to their own plan.
- Record retinue's 2026-07-29 partial Light-sleep/RF-wake receipt under V2.
- Amend D5/Ownership to name insigne.

### Notes

- Mere: 166b68ba is an ancestor; `crates/system/notochord/src/{handshake.rs:47,72,412; io.rs:163,211,249; types.rs:132,140}`; `crates/murm/transport/tests/notochord.rs`; `src/notochord.rs:50,66`; `reticulum_transport/stream.rs:41`; `crates/probes/murm-direct-phy`; `crates/murm/murm/src/session_lane.rs:156`.
- Retinue: f02f572, 1f767b8, 5b2dbbc, a67472a and 88c55cc are all in its HEAD.
- The archived V7 receipts exist at base.

## mere_docs/implementation_strategy/2026-07-24_scenograph_0_0_3_release_plan.md

- disposition: historical-marked
- status line: "Status: completed historical release plan (reconciled 2026-08-19). The published 0.0.3 artifacts remain a release baseline. Development on `main` continues through explicit score and crate versions." — accurate: yes
- claims checked: 9 — holds: 8, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- Complete with nothing open, so this is an archive candidate under DOC_POLICY §8.
- The crates.io publish of 0.0.3 is unverifiable offline (no network).

### Notes

- Checked: 4b8d875f and 6d2014b3 are ancestors; `sceno/src` has no `measure.rs`; `scene.rs:142` has `channels`; `cartography/src/scene_out.rs:44,47`; `sceno` `footprint.rs:76` and `geometry.rs:123`; `scenotime/src/pick.rs`.
- The scene contract note has a "Rulings (2026-07-24 release)" section.
- Woodshed's stage-set plan line 26 carries the 0.0.3 banner.
- The crates are now at 0.0.4, consistent with the status line.
- Every link resolves at base.

## mere_docs/implementation_strategy/2026-07-25_knot_port_plan.md

- disposition: historical-marked
- status line: "Status: implementation complete locally 2026-07-27. K0 through K7 are executable. Knot has now pulled Stickleback's causal projection seam: concurrent writers for one document are reported as a visible conflict while unrelated documents remain available, and an explicit resolution names the exact causal versions it replaces. Personal vault sync and communal multi-member Knot are separate signed encryption profiles; communal documents use retained Commons data-key epochs. Graphshell protocol 1.1 now carries the payload-free revision bell Knot's native watcher needed. K7's Cambium text primitive is committed on local Genet `main` at `44e291afe8b`; publishing that commit remains the clean remote-checkout gate." — accurate: no
- claims checked: 8 — holds: 4, stale: 4, unverifiable: 0

### Stale claims

- **"publishing that commit remains the clean remote-checkout gate."** Genet `44e291afe8b` is an ancestor of `origin/main` (local tracking ref) and of mere's genet pin `bd3e8861`.
- **§3: "knot-editor-host … now lives in Genet".** It is now a mere workspace member at `crates/inker/knot-editor-host` (root Cargo.toml:310 and the members list).
- **§5: "Automatic same-document text merge remains a Knot-owned product decision".** `automatic_text_merge` landed 2026-07-27 in Mere `259f4e3b` (`ports/knot/src/sync.rs`), about seven hours after §5's sentence was written (`e06f5a78`), so §5 was stale the day it was written; knot-editor `078fead` (2026-08-20) added Djot-block merge in `djot_merge.rs`, and `eee326f` (2026-08-25) moved `automatic_text_merge` there.
- **Repository note: "consumed by Djinn and Turnstone from one immutable revision".**
  - Turnstone `Cargo.toml:102,105` pins rev `3dfb70b0`.
  - Mere's root `Cargo.toml:243-244` (used by djinn through `knot-editor.workspace = true`) pins `562353aa`.
  - djinn's `knot-site` pins `ea3e99ef`.
  - This is stale if the note means one shared revision; the reading is ambiguous.

### Contradictions

- The repository note marks only the paths as historical. §1's ruling ("Not a standalone repository") and the §6 non-goal ("Not a new repository") stand unannotated, although Knot is now its own repository.
- DOC_README.md:175 says Knot "automatically merges exactly two compatible concurrent UTF-8 text versions"; §5 says that remains a product decision.
- A near-identical copy exists at `knot-editor/design_docs/2026-07-25_knot_port_plan.md` (14 differing lines).

### Recommended action

- Annotate the status gate as cleared.
- Annotate §1/§6 as reversed by the extraction.
- Annotate §5 with the merge landing.
- Decide whether mere keeps this copy at all, given knot-editor's.

### Notes

- Checked: genet `git merge-base --is-ancestor 44e291afe8b origin/main` and `... bd3e8861`; `crates/chirograph/src/lib.rs:1286` (`CarrierNotice`); `ports/knot` absent at base; knot-editor extraction plan present.
- Companion links resolve at base.

## mere_docs/implementation_strategy/2026-07-27_commons_calls_plan.md

- disposition: current
- status line: "Status: A0 complete and promoted at `crates/moot/commons` 2026-07-28; A1-A6 not started. Turnstone is the fixed first consumer for A1, gated on T5 of the [place-port plan](../../../../turnstone/design_docs/2026-07-28_turnstone_place_port_plan.md): the headed two-window receipt, not merely the existence of a place port or a `place.json`. Named as a rung rather than a capability because "can open a shared place" was already true at T2, when nothing was live." — accurate: no
- claims checked: 8 — holds: 7, stale: 1, unverifiable: 0

### Stale claims

- **"A1 … gated on T5 … the headed two-window receipt."** Turnstone's `design_docs/2026-07-28_turnstone_place_port_plan.md` records T5a (2026-09-13, line 1014), T5b (2026-09-14, line 1086) and T5c (2026-09-15, line 1233) receipts. Its words: "Reframe steps 1 through 7 are now each receipted; step 5 at the strength of a founder-held document on a loopback place." Line 941 names T5 as the gate A1 waits on.

### Contradictions

- DOC_README.md:185 repeats "A1 gated on Turnstone place-port T5".
- The place-port plan's 2026-09-13 product correction (Gemot hosts murmurs, moots and coop) bears on this plan's unresolved "is a call owned by the right noun?" question. That question is still unanswered anywhere.

### Recommended action

- Record T5's receipts and their loopback qualifier.
- Restate A1's remaining precondition as the open noun question rather than T5.
- Update DOC_README:185 to match.

### Notes

- Checked: `crates/moot/commons/src/call.rs` (package `commons-spine`); `crates/murm/transport/src/accepted.rs`; `notochord/src/io.rs:211` (`admit_session`); `AdmittedSession` in `notochord/src/authority.rs`.
- No `turnstone/src/call.rs` and no `commons-call` ALPN use anywhere.
- All four dependency links resolve.

## mere_docs/implementation_strategy/2026-07-27_graphshell_reference_host_plan.md

- disposition: current
- status line: "Status: product boundary ruled with Mark; H0-H3 complete; H4 operational follow-ons remain; H5-H7 complete; H8 not started; H9's first-party application door and Turnstone receipts client are complete, while its AI and MCP surfaces remain open." — accurate: yes
- claims checked: 10 — holds: 8, stale: 2, unverifiable: 0

### Stale claims

- **Graphshell as the resident.** §1 says Graphshell is "the permanent resident host and GUI for Personae's identity vault, SSH agent…". The 2026-08-20 topology addendum says "the installed Graphshell device host becomes the first desktop composition". In the tree:
  - Commit 1a3dcf6f (2026-08-22, "make resident composition the desktop owner") moved the device host to `ports/djinn` (`src/bin/djinn.rs`).
  - `ports/graphshell/src/bin/` has no `graphshell_device_host.rs`.
  - `ports/graphshell/README.md:142`: "The desktop resident is djinn, not a Graphshell binary."
- **§12 open decisions 2 (OPFS vs IndexedDB) and 3 (who owns the presenter surface) are still listed open.** Decision 2 is closed by H5a, which chose a full IndexedDB `muniment::Backend` (now muniment's `indexeddb` backend). Decision 3 is closed by the current tree rather than by H2 (which only made `graphshell-web` a separate package): `genet_render_host::RenderCore` boots wgpu and configures the surface and Graphshell keeps only composition (`ports/graphshell/src/web_gpu.rs:10-14`), the plan's "thin Genet host adapter" side.

### Contradictions

- "H5-H7 complete" while the H6 addendum's S3 is "Still open": recovery pins and the blob-reference-set intent pair.
- The status line omits H10, whose landed half moved to the reachability plan's R0 and whose DNS-SD half stays here.
- DOC_README.md:70 omits the H9 AI/MCP and H10 open items.

### Recommended action

- Annotate §1 and the 2026-08-20 addendum with the Djinn move.
- Say where H4's remaining operational items (sign-out/reboot recovery, retiring the disabled Personae task) now live.
- Close decisions 2 and 3.
- Name H6 S3 and H10 in the status line.

### Notes

- Checked: H0/H1/H4i/H5b/H6d/H7b receipts exist at base; the h4f receipt is gitignored and correctly marked historical.
- `crates/mere/Cargo.toml` has the graph/linked-data/canvas/workbench features; `ports/graphshell` has the native/web profiles.
- Retinue has no `ports/retinue-agent`, consistent with "H8 not started".
- Turnstone has `src/device_receipts_service.rs`; the application door is described in the resident plan's R1.

## mere_docs/implementation_strategy/2026-07-27_knot_authoring_consumer_plan.md

- disposition: historical-marked
- status line: "Status: all Knot-owned work in the reconciled sequence is complete locally: A1 through A4, typed Inspector clip insertion, production Resolve/Run providers, sanitized HTML lowering, and the sealed attributable resolve cache. Deterministic and real-process receipts are green, including the OS-headed Genet Probe drive. Exact selected-range clipping is complete for Genet's static retained document producer. Livery and scripted documents remain producer-specific selection seams; Knot already accepts and preserves an explicit selector without inventing one." — accurate: no
- claims checked: 8 — holds: 6, stale: 2, unverifiable: 0

### Stale claims

- **"Livery and scripted documents remain producer-specific selection seams"** and "the scripted lane supplies no clip yet".
  - Genet `50c1fd7d40e` (2026-07-28) is "Retain exact Livery text selection".
  - `components/genet-documents/src/engines/livery.rs:2150` and `scripted.rs:415` both emit selection-scoped clips with DOM ranges (`semantic_clip_from_selection_with_links`).
  - Both are present at mere's genet pin `bd3e8861`.
- **Repository note "one immutable revision"**: the same pin divergence as in the knot port record.

### Contradictions

- DOC_README.md:117 says selected-range clipping is "gated on Genet retaining selection", while the status line says static clipping is complete.
- A near-identical copy exists at `knot-editor/design_docs/2026-07-27_knot_authoring_consumer_plan.md` (12 differing lines).

### Recommended action

- Record the Livery and scripted producers as landed.
- Fix DOC_README:117.
- Archive candidate: nothing Knot-owned is open, and the producer seams have now landed.

### Notes

- Checked: `crates/chirograph/src/lib.rs:954,988,1000` (`EditableTextV1`, `DerivedCacheInfoV1`, `SaveTextV1`).
- Turnstone: `scenarios/knot_authoring.scn` and `knot_selected_clip.scn`; `src/knot_authoring.rs:800,817` (the env settings).
- Genet `docs/2026-07-25_text_editing_primitive_plan.md` is present.

## mere_docs/implementation_strategy/2026-08-02_knot_in_graphshell_plan.md

- disposition: historical-marked
- status line: "Status: K0-K3 complete. K1 chose Option A (Mark): shared documents are projected, personal documents replicate, and T4's done condition is replaced accordingly, closing the shared-Knot authority question as dissolved rather than answered. K2's three clauses are proven against the real resident host, and its physical two-machine receipt passed on 2026-08-08. K3 kept the spawn path deliberately." — accurate: yes
- claims checked: 11 — holds: 8, stale: 3, unverifiable: 0

### Stale claims

- **"`BlockEvaluator: Send` is now stated in genet … the trait is genet's".** The trait is now in mere at `crates/inker/inker/src/document/evaluate.rs:86`; inker landed from genet on 2026-09-03.
- **Knot search, "the actual defect": `search.rs` calls `sibylla::SemanticSearch`; "Sibylla has a BERT provider".**
  - Sibylla is deleted. knot-editor `src/search.rs:18` uses `esp::embed::SemanticSearch` with a `KnotEmbeddingPreference` (BERT via `esp/bert`).
  - S0 and S1 are still unbuilt: no tantivy and no `fuse` in `search.rs`.
- **Repository note "one immutable revision"**: the same pin divergence as in the knot port record.

### Contradictions

- K0 (2026-08-02) says "Still spawning: the persona-vault modes"; K3 (2026-08-06) says "Turnstone hosts every Knot mode in-process".
- "Not in scope" lists the "Wasm" bullet twice.
- A near-identical copy exists at `knot-editor/design_docs/2026-08-02_knot_in_graphshell_plan.md` (26 differing lines). That copy marks itself "historical integration record"; mere's copy does not.

### Recommended action

- Repoint the `BlockEvaluator` home.
- Update the search-defect wording to esp.
- Dedupe the bullet.
- The plan itself says the hosting work can be archived. Knot search S0/S1 is Knot-owned and is carried in knot-editor's copy.

### Notes

- Checked: the knot-editor K2 receipt, `examples/k2_peer.rs` and `Cargo.toml:96` (graphshell is a dev-dependency only).
- Mere: `ports/graphshell/src/native/endpoint_catalog.rs:217`; `crates/chirograph/src/lib.rs:1341` (`CarrierError`); `ports/graphshell/src/sessions.rs:52`.
- The device resident plan and the archived carrier seam plan resolve.

## mere_docs/implementation_strategy/2026-08-03_graph_view_curation_and_interaction_plan.md

- disposition: current
- status line: "Status: C3's root-Canvas fold is landed; its Swatch proof remains pending. C4 is complete: the source-time contract has journal-prefix and Git-authority adapters, plus a real second source in Isometry's pre-log `GameSnapshot` and authority `GameEvent` Codicil. Its Overmap Swatch has a Cambium slider that selects disposable historical snapshots, preserves local curation, disables world actions while historical, and returns to untouched live truth. The native root Canvas has a painted source rail with pointer scrubbing plus Home/End and Page Up/Page Down, rather than a fictional Cambium widget. Historical Canvas and Swatch headed captures exist. The receipt matrix verifies every current Canvas arrangement against a journal prefix and all seven public repository arrangements against an available Git checkpoint, then verifies return to the retained live source. Joined clients and pre-origin checkpoints remain live-only until the session handshake carries a verified origin plus public log. C5 is complete: Mer3ly publishes reduced, commit-pinned public history from Graphshell through the former WebRender-wgpu fork and the current authority. Its native headed smoke records ready desktop and mobile source playback, arrangement changes at a past cursor, keyboard stepping, archive appearance and closure, and Return to live. C6 is complete: Mer3ly's versioned public fragment reopens the same historical source, arrangement, and selection on desktop and mobile without private references; Mere persists a versioned, content-addressed local live-view recipe and lets its source owner explicitly refuse missing, stale, denied, or unsupported requests; Graphshell carries an opaque record reference through its participant gate; and a projection capture serializes validated Scenotime tables plus each presentation-resource address, so a replay verifies identical tables and visual bounds before rendering." — accurate: yes
- claims checked: 8 — holds: 6, stale: 1, unverifiable: 1

### Stale claims

- **The ownership table and "names that must stay distinct" assign durable curation to `session-runtime::view_intent_store` / `session_runtime::ViewIntent`.** session-runtime is gone; the store is `crates/system/pandect/src/view_intent_store.rs` (pandect `lib.rs:141,243`).

### Contradictions

- DOC_README.md:252 says "planned with Mark; shared slices not started", but the status line says C3 (root), C4, C5 and C6 have landed.
- The plan's own sequencing (C3 → C4 → C5 → C6, "C6 remains last", "Stop after each slice's receipt wall") conflicts with C6 being complete while C3's Swatch proof is pending.
- The plan has no Findings or Progress section, which DOC_POLICY §8 requires. The status line is undated and silent on C0–C2.

### Recommended action

- Repoint the view-state owner to pandect.
- Add dated Progress entries for C0–C6.
- Date the status line.
- Fix DOC_README:252.

### Notes

- Checked in mere: `crates/graph/graph-kernel/src/graph/source_time.rs:29`; `journal.rs:379` (`impl SourceTime for GraphJournal`); `crates/canvas/pictograph/src/canvas.rs:732` (`SourceTimeCanvas`) and its `tests/fold_and_source_time.rs`; `crates/chirograph/src/lib.rs:442,564`; `crates/cambium/cambium/src/graph_canvas.rs:182` (`GraphCanvasRelation`).
- Checked elsewhere: isometry `crates/isonetry/src/source_time.rs`; mer3ly `src/repository_history.rs`.
- The C3 Swatch proof was not checked.

## mere_docs/implementation_strategy/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md

- disposition: current
- status line: "Status: R0 landed, recorded 2026-09-01; R1 landed 2026-08-03 (Graphshell) and 2026-08-06 (Knot), the genuinely remote receipt still open; R2 scoped; R3 scoped, gated on emissary entering the tree. Veilid retired 2026-09-01." — accurate: no
- claims checked: 12 — holds: 10, stale: 2, unverifiable: 0

### Stale claims

- **The status line says "R3 scoped, gated on emissary"**, but R3's own body records "Noise (the in-stream plane, landed 2026-08-06)". The code is `crates/murm/transport/src/noise.rs`, with the test `noise_over_an_iroh_stream_layers_a_second_identity` at `noise/tests.rs:246`.
- **R0: "Two forks pinned by branch (root `Cargo.toml`, the 'LAN peer discovery (H10)' block)".** Commit a5543904 (2026-10-03, after this plan's last edit 08ec084a on 2026-10-02) removed the `iroh-mdns-address-lookup` pin. Root Cargo.toml:710-726 now reads "one dead theory" and pins only `swarm-discovery`. The p2panda tag is `mere-p2panda-net-0.7.5`, not the 0.7.4 the correction cites.

### Contradictions

- R2's body ("The retinue announce already binds authenticated app data") contradicts its own 2026-10-02 correction and `reticulum_transport/announce.rs:38-51`, which returns empty app data.
- DOC_README.md:188 still says "the two branch-pinned mDNS forks".

### Recommended action

- Add the Noise landing to the status line.
- Update R0's fork list to one fork, citing a5543904.
- Restate R2's premise against empty announce app data.

### Notes

- Checked: 0f0a8006 and dfe95f3e are ancestors; `crates/murm/transport/src/p2panda_host.rs:19,76`; `ports/djinn/src/settings.rs:468,511` (`PairedDevice.last_endpoint`).
- knot-editor: `src/settings.rs:129,202` and `src/resident.rs:577`.
- The emissary fork is local at `Code/crates/emissary` and absent from mere's dependency graph.
- Links resolve: device pairing plan, Djinn plan, event DAG brief.

## mere_docs/implementation_strategy/2026-08-06_configuration_ownership_settings_projection_plan.md

- disposition: current
- status line: "Status: implementation complete through C6; C7 deferred by design" — accurate: yes
- claims checked: 10 — holds: 6, stale: 4, unverifiable: 0

### Stale claims

- **The Code line cites `mere/crates/system/session-runtime/src/{application_settings_store.rs,…}` and `turnstone/src/apparatus_pane.rs`, neither marked historical.**
  - The stores are now `crates/system/pandect/src/{application_settings_store,device_settings_store,persona_settings_store,settings_store}.rs`.
  - Turnstone has no `apparatus_pane.rs`.
- **The Code line and ledger cite `genet/components/config` (opts/prefs, "correct as-is").** Genet deleted it in `5af76a0cb8c` (2026-09-07).
- **The C3/C4 contract is described as a "module in genet-host-api beside the existing `SettingsRef` lane (tile.rs:144)" and `genet-host-api::settings::SettingsProjection`.**
  - It now lives in mere's `mere-surface-api`: `crates/system/surface-api/settings.rs:84,102,134`.
  - `SettingsRef` is at `crates/cambium/workbench/lib.rs:167`.
  - Genet `d25ef444d21` renamed `tile.rs` to `components/workbench/lib.rs`, since moved into Mere's `crates/cambium/workbench`.
- **The ledger row for graphshell `OwnerSettings` cites `owner_settings.rs:104`.** It moved to `ports/djinn/src/settings.rs` in 1a3dcf6f.

### Contradictions

- none inside the plan beyond the moved homes above. The status line is undated.

### Recommended action

- Repoint the Code line, the ledger and §C3 to pandect, mere-surface-api/workbench and djinn.
- Mark the genet config row historical.
- With C0–C6 done, C7 deferred by design and the open Knot write-path migration owned by knot-editor, this is an archive candidate once those two are extracted.

### Notes

- Checked: `pandect/src/atomic_file.rs:24` (`write_bytes_with_backup`), used in `notochord_policy_store.rs` and `ports/distillery/src/installed.rs`.
- knot-editor `src/settings.rs:115-117` still does remove-then-rename, so that migration is open.
- `ports/graphshell/src/lib.rs:36,65` gates `personal_sync` behind the `personal-sync` feature.
- Turnstone `settings_provider.rs` and `settings_pane.rs` are present; the archived predecessor resolves.

## mere_docs/implementation_strategy/2026-08-08_esp_consolidation_plan.md

- disposition: historical-marked
- status line: "Status: E0-E4 complete 2026-08-09. ESP 0.1.0, Vates 0.1.2, and Sibylla 0.1.2 are published on crates.io in that order. The 2026-08-08 adversarial amendments remain authoritative: corrected knot consumer graph, restored mesh/scheduler boundaries, host-side device policy, narrowed servitor language, and separate portability and repository-promotion gates. Supersedes the first draft written in `repos/esp/design_docs/` *(historical citation)* <!-- doc-audit: historical-path -->; that file is now a pointer here. D2's configured embedding matrix and first exact browser decoder row now pass, including cooperative token-boundary cancellation, explicit browser device teardown, and exact recovery in a fresh worker. Physical GPU-allocation release remains unobservable in Chromium. Immutable ModelSession plus the real PEFT LoRA row are complete, and Eidetic's training/evaluation artifact boundary has landed. The next model execution gate is one deterministic local trainer fixture; communal compute remains later." — accurate: no
- claims checked: 11 — holds: 8, stale: 2, unverifiable: 1

### Stale claims

- **"The next model execution gate is one deterministic local trainer fixture"** (status line, 2026-08-26 Progress).
  - `2026-08-12_distillery_v0_plan.md`'s status (lines 19-25) records the first local trainer receipt and the `esp.train.peft-lora/v1` mesh job, with the adapter strictly beating the unchanged baseline.
  - `2026-09-02_autodiff_lora_trainer_plan.md` is "complete on `main` (2026-09-03)".
  - esp's Cargo.toml has `decoder-autodiff`, added in 5214dc7f.
- **E3/E4 and the 2026-08-09 entry: Vates and Sibylla retained as deprecated shims that "ride along on future bumps".** Commit d69be378 (2026-09-23) deleted the shims and their names. `crates/intel` now holds only esp, eidetic-search, mora-cmudict and reference-data.

### Contradictions

- The plan says "kept this consolidation plan closed" (2026-08-09 spin-out), yet its status line keeps carrying progress for the spun-out lanes: D2, ModelSession, Lane 4.
- DOC_README.md:365 repeats the stale trainer gate: "only then should Distillery add a trainer resource".

### Recommended action

- Record the trainer landing and the shim deletion.
- Trim the status line to E0–E4.
- Fix DOC_README:365.
- Archive candidate: E0–E4 are complete and the ready work is spun out.

### Notes

- Checked: 1283b4a8 is an ancestor; `crates/intel/esp/src/embed/lexical.rs:79`; esp's burn pin `=0.22.0-pre.4` (so the stable-repin gate holds).
- The graph behaviors plan header confirms R1a/R1b are still pending validation.
- The testing receipts and the feature target matrix exist at base.
- The crates.io publishes are unverifiable offline.
