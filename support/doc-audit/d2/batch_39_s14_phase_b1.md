# Batch 39 — S14 pass, phase B1 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| cambium_docs/implementation_strategy/2026-06-01_event_model_convergence_plan.md | historical-unmarked | no | 12 | 7 | 4 | 1 |
| cambium_docs/implementation_strategy/2026-09-06_fact_visualization_leaves_plan.md | historical-marked | yes | 12 | 11 | 0 | 1 |
| dramatis_docs/implementation_strategy/2026-08-08_gaz_founding_plan.md | current | yes | 13 | 11 | 1 | 1 |
| dramatis_docs/implementation_strategy/2026-08-12_ssh_ca_projection_plan.md | historical-marked | no | 12 | 10 | 1 | 1 |
| dramatis_docs/implementation_strategy/2026-09-23_insigne_proofs_plan.md | current | yes | 16 | 13 | 1 | 2 |
| eidetic_docs/implementation_strategy/2026-06-09_eidetic_deferred_phases_plan.md | current | yes | 11 | 9 | 1 | 1 |
| mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md | historical-marked | no | 10 | 5 | 4 | 1 |
| mere_docs/implementation_strategy/2026-05-14_engine_profile_boundary_plan.md | current | yes | 6 | 6 | 0 | 0 |
| mere_docs/implementation_strategy/2026-05-14_session_service_runner_plan.md | current | yes | 7 | 5 | 2 | 0 |
| mere_docs/implementation_strategy/2026-06-02_modular_integration_plan.md | historical-marked | yes | 6 | 5 | 1 | 0 |
| mere_docs/implementation_strategy/2026-06-09_multi_graph_activation_plan.md | historical-marked | no | 6 | 3 | 3 | 0 |
| mere_docs/implementation_strategy/2026-06-10_multi_window_plan.md | historical-marked | yes | 6 | 4 | 2 | 0 |
| **Totals** |  |  | **117** | **89** | **20** | **8** |

**Totals: 12 docs, 117 claims checked (89 holds, 20 stale, 8 unverifiable), 16 contradictions; 4 status lines wrong.**

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

Checked directly in this session: genet `0057a122779` and `9a994bb4193` are
ancestors of Mere's pin `bd3e8861` (event model); `mint_host_cert`'s only
callers are `ssh_ca.rs:646`, inside its tests, and `tests/ssh_ca_live.rs:146`
(SSH CA); `OpenGraphBeside` has no hit at the base, and `c5f01064` is
meerkat's removal (multi-graph). The verifier confirmed 28 of 31 items, found
3 true only in part (the protocol plan's §2, which still holds for iroh and
iroh-blobs; the multi-graph switcher builder, which survives in pandect; the
Woodshed pin, which differs for its `ports/hocket` workspace) and refuted
none; those records are corrected, with four citations and the SSH CA
restatement (T5 partial as well).

## cambium_docs/implementation_strategy/2026-06-01_event_model_convergence_plan.md

- disposition: historical-unmarked
- status line: "Status: core dispatcher convergence landed; `window`/`document` targeting and shadow-tree `composedPath` remain explicitly deferred." — accurate: no
- claims checked: 12 — holds: 7, stale: 4, unverifiable: 1

### Stale claims

- **Status says `window` targeting is deferred.** It is not. Genet `0057a122779` (2026-06-01, "unify window into the Node event-propagation model") added it, and that commit is in Mere's genet pin `bd3e8861`. At the pin, `components/script-runtime-api/dom/bootstrap.js:1014-1024` appends `terminalWindow` above the document.
- **Status says shadow-tree `composedPath` is deferred.** It is not. Genet `9a994bb4193` (2026-09-08, "Shadow DOM across both DOMs ... and retargeting") is in the pin. At the pin, `bootstrap.js:1006` stops a non-composed event at the shadow root, `:6235` defines `__composedPathVisible`, and `dom/shadow.rs` and `tests/shadow_dom.rs` exist.
- **"Still open: passive scroll-blocking" (lines 174-178) is overtaken.** At the pin, `bootstrap.js:967` flags a passive listener and `lib.rs:2777` makes `preventDefault` a no-op under that flag, and `bootstrap.js:5488-5498` implements WPT's `non-cancelable-when-passive` rule.
- **"Each test's doc comment names its twin and points here" (line 120) is half true.** The JS twin, genet `dom/tests.rs:1936-1940`, points at genet's own copy (`docs/2026-06-01_event_model_convergence_plan.md`) and names `xilem-serval`. Only the native side points at this file (`crates/cambium/cambium/src/tests.rs:3276`, `propagation.rs:20`).

### Contradictions

- Genet still tracks a second copy of this plan: `genet/docs/2026-06-01_event_model_convergence_plan.md`, 185 lines, no status line, last touched `3a82d43133a`. Genet's test cites that copy. DOC_POLICY §2 says shared material lives once.

### Recommended action

- Rewrite the status:
  - landed: window in the propagation path (`0057a122779`); shadow-including path with retargeting and `composedPath` (`9a994bb4193`); passive cancel-gating.
  - still open: per-interface event subclasses (`createEvent` still returns a base Event, pin `bootstrap.js:2819-2825`) and native `currentTarget`.
- Then decide on archiving (see Forks).

### Notes

Checked cambium `propagation.rs`, `event.rs:66`, `key.rs:179` and `tests.rs:3300,3339`. Checked genet at the pin taken from Mere `Cargo.toml` (28 genet rev entries): `bootstrap.js` and `dom_node_events_work` (`dom/tests.rs:1941`). Ran `merge-base --is-ancestor` for `0057a122779`, `9a994bb4193` and `741bf726eb4` against the pin. The 66→142 subtest count was not re-run.

## cambium_docs/implementation_strategy/2026-09-06_fact_visualization_leaves_plan.md

- disposition: historical-marked
- status line: "Status (2026-09-06): V0-V2 landed, including Cleromancy's first controlled range-scrubber adoption" — accurate: yes
- claims checked: 12 — holds: 11, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record. Every phase has landed (archive question in Forks).

### Notes

- Sprigging: `AngleStrip` (`sprigging/src/angle.rs:46`); `DimensionLine` with `Direct`/`Wrapped` (`dimension.rs:21-28`); `GraphCanvas`/`Meter`/`Knob` (`glyphs.rs:104,396,507`).
- Cambium: `RangeScrubber` (`cambium/src/range_scrubber.rs:38`) with `Preview`/`Commit` (`:148-151`); re-exports at `cambium/src/lib.rs:166,179-180`; 30 catalog hits in `component_catalog.rs`; `HostHooks::frame` (`cambium-rootstock/src/host.rs:823`).
- Cleromancy consumes all three leaves: `src/ui/native/mod.rs:15`, `src/ui/view/chart.rs:8`.
- Test counts (204/204 etc.) were not re-run.

## dramatis_docs/implementation_strategy/2026-08-08_gaz_founding_plan.md

- disposition: current
- status line: "Status: M0 landed 2026-08-08. On 2026-09-23 Mark ruled the anchor (a typed key, a `did:plc`, or a local id), reaffirmed it over the standards survey's first-pass DID grading (amended to match), and accepted the three M0.5 proposals from that day's critical pass: anchoring on a peer's root with attested keys held concurrently, a whole-identity Reticulum key (verified against Reticulum's reference implementation), and `Anchor::Local` for keyless contacts. JSContact is ruled *an* exchange format (M1). M0.5 landed the same day, and `TypedKey` then moved to insigne, as Mark ruled. M1 keeps sealing at the host and JSContact at the exchange boundary. M2 is drafted with done-conditions. The retained-proof slice of M2 landed on 2026-09-29 under Insigne phase D; M1's storage gate is implemented on 2026-09-29 and its host sealing gate on 2026-09-30; JSContact exchange landed 2026-09-30, completing the M1 library gates. The first M2 address-intake and WebFinger adapter slice landed 2026-09-30; checked key/PLC intake and application wiring remain open." — accurate: yes
- claims checked: 13 — holds: 11, stale: 1, unverifiable: 1

### Stale claims

- **M0.5 says no book was ever stored and nothing outside gaz consumes it.** The claims are at lines 194-196 ("Nothing stores a book yet and nothing outside gaz consumes it (checked 2026-09-23), so there is no migration and no legacy decoder") and §5 lines 475-476 ("no book was ever stored"). Both were false on their date:
  - Retinue's `apps/signalman-desktop/src/messages.rs:5,50,128` imports `gaz::{ContactBook, ContactKey, ...}` and loads and saves a `ContactBook` through Muniment slots.
  - It does this at Mere `d82afa17` (`apps/signalman-desktop/Cargo.toml:72`), and has since retinue `864645e` (2026-08-19).
  - M0.5 then changed the stored shape (a list instead of a map, hex map keys gone).

### Contradictions

- The insigne proofs plan's phase D (lines 487-489) records the same Signalman consumer still importing the pre-M0.5 `ContactKey`.

### Recommended action

- Correct M0.5 and §5 with the retinue finding, and record how Signalman's stored books are handled at its repin (fork).

### Notes

- Present at base: `lib.rs:109` re-exports, `anchor.rs:69` (`LocalId(Uuid)`), `persistence.rs:83,97`, `jscontact/exchange.rs:53-115`, `intake/apply.rs:23`, gazette `intake.rs:30,76`, castellan `tests/sealed_contacts.rs` and `tests/sealed_webfinger_intake.rs`, `sealed_storage.rs:24`, pandect `wallet_sealed_backend.rs:51`.
- `5364dfa0`, `cf901f3d`, `ca2351b3` and `8425cd73` are all ancestors of the base.
- The last gaz commit is `c388babb` (2026-09-30), so the open M2 gates and application wiring are still open. Gaz is only a castellan dev-dependency.
- Test counts were not re-run.

## dramatis_docs/implementation_strategy/2026-08-12_ssh_ca_projection_plan.md

- disposition: historical-marked
- status line: "Status: T1–T5 landed 2026-08-12, the same day this was drafted; see Progress for what the building of it corrected. Drafted the evening the wgpu-weld parity sweep ran its Intel-iMac leg over SSH and paid the bilateral toll three ways in one afternoon." — accurate: no
- claims checked: 12 — holds: 10, stale: 1, unverifiable: 1

### Stale claims

- **Status says T1–T5 landed, but T2's host certificates were never built** (the plan's own 2026-10-02 correction says so):
  - `SshCertAuthority::mint_host_cert` (`crates/dramatis/personae/src/ssh_ca.rs:276`) is called only from `ssh_ca.rs:646` (inside `mod tests` at line 410) and `tests/ssh_ca_live.rs:146`.
  - `enroll-host` prints "host-key prompts still apply" (`src/bin/personae-vault/certs.rs:185`).
  - The follow-on, device pairing D4, has all four boxes open (`2026-10-02_device_pairing_by_key_plan.md:765`).
  - Progress itself records T4 as one-third validated and no real machine enrolled.

### Contradictions

- The status line contradicts the "Corrected 2026-10-02" line directly below it.
- DOC_README's entry ("T1–T5 landed 2026-08-12") carries no correction.

### Recommended action

- Restate the status:
  - landed 2026-08-12: T1, T3;
  - partial: T2 (user-CA half only, no host certificates); T4 (validated on one OS); T5 (the KRL deploy stops at the file, lines 202-204, while T5's validation, lines 123-125, needs a deploy to enrolled hosts);
  - moved: host certificates to device pairing D4.
- Mirror this in DOC_README.

### Notes

The correction's own anchors hold: `certs.rs:34` derives the CA from the profile's master, and `bootstrap.rs:135` generates a fresh `Ed25519Keypair`. Also present: modules `ssh_ca`, `ssh_face`, `ssh_krl`, `enroll`; vault subcommands at `main.rs:129-134`; `install-agent-linux.sh`; `SSH_CA_MOD_ID` (`ssh_ca.rs:43`); `mint_user_cert` checks `request.ledger` (`ssh_ca.rs:202-215`). All five Related links resolve. The 121-test count was not re-run.

## dramatis_docs/implementation_strategy/2026-09-23_insigne_proofs_plan.md

- disposition: current
- status line: "Status (2026-09-29): phase A landed 2026-09-24 and phase B on 2026-09-26 (§3); C landed in Mere and Knot on 2026-09-29; sibling repins remain open. D landed in Gaz on 2026-09-29: stored artifacts reload and check again. Mark agreed the split and ruled how issuing is expressed (§2, option (a)) on 2026-09-23. The Mere 0.4 release baseline's Insigne prerequisite was phase B, met on 2026-09-26." — accurate: yes
- claims checked: 16 — holds: 13, stale: 1, unverifiable: 2

### Stale claims

- **The phase C handoff still lists Turnstone as unrepinned** (lines 383-387, reaffirmed 2026-10-01 at line 513: "the handoff stands as written"). Turnstone has repinned:
  - `Cargo.toml:28` pins `insigne` at Mere `bd5912fb`, which contains `7926d3a8`.
  - `src/denizen.rs:39` and `src/identity.rs:48` import from `insigne`, and no `personae::` proof imports remain.
  - Adopted in turnstone `d6b62ad` (2026-09-29) at Mere `ca2351b3`, since moved to `bd5912fb`; both contain `7926d3a8`.

### Contradictions

- none.

### Recommended action

- Mark Turnstone done (`d6b62ad`) and leave Hocket, Woodshed and mer3ly as the open repins:
  - Hocket and mer3ly pin `d82afa17`, which predates phase A.
  - Woodshed's root workspace pins `8106c7c2`, which lacks phase C; `ports/hocket`, a separate workspace excluded at its root `Cargo.toml:20` and where the handoff's two files live, pins `d82afa17`, which predates phase A, and its `crates/hocket-engine/src/handoff.rs:233` still calls `verify(&salt)`.

### Notes

- These Mere commits are all ancestors of the base: `5364dfa0`, `aacf39c7`, `b2677e15`, `0f9eaa52`, `bd6b8c4b`, `e4471b93`, `87aae7ab`, `538226a3`, `7926d3a8`, `a31b9a14`, `bc7121ae`, `4b33a963`, `67ebef3a`, `b7ed0bdc`, `3943874f`, `8fce5365`, `d82afa17`. Knot `c6d5b9e`, `a08c1f7`, `855cb75` and `5ad3f67` are all ancestors of knot HEAD.
- The `check` API exists (`attestation.rs:80`, `delegation.rs:243,386`, `check.rs:18`) and no `verify` remains. Personae re-exports none of the moved types, and no old import paths remain. Notochord `fold` takes `CheckedRevocation` (`chain.rs:97`). Gaz `contact.rs:46,60` holds `Option<KeyProof>`.
- The `dramatis` facade is still a reservation, and Retinue is still at `d82afa17` with `ContactKey`.
- Unverifiable: test counts, and the remote-fixture resolution failure.

## eidetic_docs/implementation_strategy/2026-06-09_eidetic_deferred_phases_plan.md

- disposition: current
- status line: "Status: Active (open tail spun out of the completed layered-stack plan)." — accurate: yes
- claims checked: 11 — holds: 9, stale: 1, unverifiable: 1

### Stale claims

- **The "Crate family" header lists `eidetic-fjall`, `eidetic-https-fetcher` and `eidetic-iroh-fetcher` as shipped crates.** All three were folded into `eidetic` as features on 2026-09-23: `fjall` in `3943874f`, `https-fetcher` and `iroh-fetcher` in `254b23b6` (`crates/eidetic/eidetic-core/Cargo.toml:58-60`). `crates/eidetic/` now holds chartulary, eidetic-core (package `eidetic`), hagiograph and muniment.

### Contradictions

- **Phase 7 and the newer browser-storage work ignore each other.** Phase 7 plans a hand-rolled `eidetic-opfs` Store. The tree has since gone another way:
  - Muniment's `IndexedDbBackend` (`crates/eidetic/muniment/src/indexeddb_backend.rs:46`) is "the browser store today" per `2026-08-22_redb_opfs_feasibility_plan.md:90-91`.
  - That redb-over-OPFS feasibility plan has its own probe, `ports/muniment-opfs-probe`.
- `2026-06-24_orrery_browser_lane_plan.md:109-110` says its lane "activates Phase 7", while this plan records Phase 7 only as "trigger emerging".

### Recommended action

- Fix the crate-family line and date the status (DOC_POLICY §8). Reconcile Phase 7 with Muniment and the redb-over-OPFS plan (fork).

### Notes

- Holds: no `OpfsStore` or `eidetic-opfs` in the tree (no code landed); no `EngramDirectory` (the Phase 9 consume half is still open); `BrowsingMemory` at `eidetic-core/src/browsing/mod.rs:211`; `crates/intel/eidetic-search`.
- The archived layered-stack plan, the design pass, the browsing derivation plan (Active) and genet's `docs/2026-06-24_nova_memory64_browser_lane_plan.md` all exist.
- Unverifiable: the `Code/.tantivy-probe` checkout cited for the 2026-06-11 analysis no longer exists.

## mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md

- disposition: historical-marked
- status line: "Status: Draft / canonical direction (architecture-level; per-protocol module plans branch off). Partially superseded by the 2026-05-07 briefs:" — accurate: no
- claims checked: 10 — holds: 5, stale: 4, unverifiable: 1

### Stale claims

- The status bullets (lines 6-7) say §2, §3 and §4 "remain authoritative"; each is at least partly overtaken.
- **"§2 (iroh layering) ... remain authoritative."** Partly: the plan's own 2026-06-09 banner says the `mere-transport` iroh path retired; `crates/murm/transport/src/` has `p2panda_transport.rs` and `reticulum_transport.rs` but no `iroh_transport.rs`, and gossip is `p2panda_net::gossip::GossipHandle` (`lib.rs:86`), so §2's iroh-gossip and iroh-docs rows, the Cable choice rule and the ALPN ledger are superseded. Its iroh and iroh-blobs rows still hold: iroh remains the QUIC byte plane under p2panda-net, and the transport uses iroh and iroh-blobs directly (`p2panda_transport.rs:56-59, 215-269`; transport `Cargo.toml:20`, "iroh stays the byte plane").
- **"§3 (identity vault) ... remain authoritative."** The vault lock plan (2026-10-05), ruling 12, rules `UnlockTier` to be consent, not custody. It states that §3.6/§3.7 (custody tiers, and line 349's swap-leakage defence) are not true today (`2026-10-05_vault_lock_plan.md:22-24, 83-87, 299-302`).
- **"§4 (self-host-with-fallback) ... remain authoritative," with Mode 1 served by verso.**
  - verso-tile was folded into `inker::flip` on 2026-09-05 (DOC_POLICY.md:248).
  - The dramatis tier architecture's rulings 3 and 6 put announcing in gazette, as a djinn service that exports static well-known files first (`2026-09-30_dramatis_tier_architecture.md:209-221, 241-259`).
- **The 2026-06-09 banner's rename key points at paths that no longer exist.** `persona/identity` is now `crates/dramatis/personae`, and `system/session-runtime` was renamed `pandect` (`441e70f0`, 2026-08-15).

### Contradictions

- The status bullet (§2 authoritative) contradicts the 2026-06-09 banner and DOC_README ("Substrate sections superseded by p2panda").
- §4 contradicts tier-architecture rulings 3 and 6. That document also says no earlier document named an owner for announcing.
- §3.6/§3.7 contradict vault-lock ruling 12.
- Two headings are numbered "3.7" (lines 347 and 361).

### Recommended action

- Narrow the status to what is still authoritative, fix the banner's targets, and point §3 at vault-lock ruling 12 and §4 at tier rulings 3 and 6. Archive question in Forks.

### Notes

- Holds: §3 types (`personae/src/vault.rs:96,141,203,260,369`; `passphrase_storage.rs:111`); `blobs.rs` over iroh-blobs `MemStore`/`FsStore`; no `PrimitiveMootProtocol` (§5 unbuilt); all nine linked docs resolve.
- Dated test counts were not re-run.

## mere_docs/implementation_strategy/2026-05-14_engine_profile_boundary_plan.md

- disposition: current
- status line: "Status: Implementation plan — v0a path-resolution primitive landed; v0b per-engine wiring pending" — accurate: yes
- claims checked: 6 — holds: 6, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- **Two types share the name `EngineProfileBinding`.** One is pandect's scope enum (`crates/system/pandect/src/manifest.rs:47`). The other is inker's `struct EngineProfileBinding { user_data_dir: String }` (`crates/inker/inker/src/surface_engine.rs:228-230`), which the surface engines consume and Pelt fills with the constant "pelt-surface-profile" (`ports/pelt/desktop/workspace_viewer.rs:799`). The plan names neither inker's type, which is where v0b would connect, nor the name clash.

### Recommended action

- Name the current homes (`crates/system/pandect/src/engine_profile_store.rs`, `crates/inker/engines/scrying-engine`) and inker's struct as v0b's seam, and date the status. Whether the plan is dormant is a fork.

### Notes

`engine_profile_path` and `_for_session` exist at `pandect/src/engine_profile_store.rs:75,122` with 9 tests, and nothing outside pandect calls them. `Engine::engine_id` is at `crates/inker/inker/src/engine.rs:79`. The plan's content has not changed since the 2026-07-03 reconcile.

## mere_docs/implementation_strategy/2026-05-14_session_service_runner_plan.md

- disposition: current
- status line: "Status: Implementation plan — v0a trait + null runner landed; v0b real workers pending" — accurate: yes
- claims checked: 7 — holds: 5, stale: 2, unverifiable: 0

### Stale claims

- **The 2026-07-03 reconcile note says the misfin server runs as a `SessionServiceRunner` worker and "v0b is at least partially real."**
  - Misfin left Mere in `f2e7825d` (2026-07-03).
  - Even at checkpoint `8dcaf441`, `crates/murm/misfin/src/server.rs:29-31` only said a host "or a daemon-side `SessionServiceRunner` worker" could spawn it.
  - At the base nothing outside `crates/system/pandect` names `SessionServiceRunner`, `NullRunner` or `InMemoryRunner`.
- **The v0a done-condition says "NullRunner ... lets `HostRoot` thread a runner reference through."** No `HostRoot` type exists, either at the base or at `8dcaf441`; the only mention is a doc comment at `pandect/src/session_service_runner.rs:105`.

### Contradictions

- The reconcile note says the code moved past the status line. The tree sides with the status line.

### Recommended action

- Correct or remove the reconcile note and name `crates/system/pandect`. Whether the plan is dormant is a fork.

### Notes

Present at base: trait at `session_service_runner.rs:93`, `NullRunner` at `:109`, `InMemoryRunner` at `:138`, `WorkerKind` variants at `manifest.rs:66-80`, `active_workers` at `manifest.rs:148`. The framing brief exists.

## mere_docs/implementation_strategy/2026-06-02_modular_integration_plan.md

- disposition: historical-marked
- status line: "Status: Historical browser/Meerkat integration plan; superseded as Mere-wide architecture by the [Platform Boundary and Repository Topology Plan](2026-09-02_platform_boundary_and_repository_topology_plan.md). The unifying sequence + architecture spine for integrating the browser product onto the single genet-as-host shell (`meerkat`). It does not replace the canonical docs it weaves: the [composition spine](../technical_architecture/2026-05-21_mere_composition_spine.md) (the model), the [genet-as-host flip plan](../../archive_docs/2026-06-10_completed_plans/2026-06-01_genet_host_flip_plan.md) (the host migration), and the [adoption roadmap](../../archive_docs/2026-06-09_completed_plans/2026-05-27_adoption_roadmap.md) (the R0–R5 wiring order). It sequences those three in-flight tracks into one build, fixes the architecture's root question, inventories the (large) already-built leverage surface, and schedules the cleanup." — accurate: yes
- claims checked: 6 — holds: 5, stale: 1, unverifiable: 0

### Stale claims

- **The line 3 banner says "the graphlets crate is crates/graph/subgraph (code renamed 2026-09-12)."** That path does not exist at the base. The crate was folded into mere by `61894570` (2026-09-23): `SessionSubgraphs` is at `crates/mere/src/subgraph.rs:39`, and `SubgraphId`/`SubgraphRef` at `crates/forme/forme/src/subgraph.rs:11,18`.

### Contradictions

- The named successor, `2026-09-02_platform_boundary_and_repository_topology_plan.md`, never mentions this plan (a grep for `modular_integration` finds nothing).

### Recommended action

- Fix the banner. Archive question in Forks.

### Notes

The Findings routing targets exist with consistent statuses (murm peer runtime "Active"; the moot M1 plan's community-publishing continuation). Pandect has `hidden_relations` in `view_intent_store.rs` (12 hits) and `live_view.rs` (2). The S1–S4 commits checked (`64ebe44` … `847ebfd`, 14 of them) are all ancestors.

## mere_docs/implementation_strategy/2026-06-09_multi_graph_activation_plan.md

- disposition: historical-marked
- status line: "Status: In progress. MG1–MG5 done plus the host text path — meerkat runs multi-graph with a window-scoped pane layout (near-Model-B): the shellbar switcher creates / switches / closes / renames graphs (labelled tiles), and switching keeps the panes while re-sourcing the graph-bound ones. MG6's far-B (different-graph leaves coexisting) and multi-window tear-out are now delivered / owned by the [tearout_composability_plan](../../archive_docs/2026-07-04_completed_plans/2026-06-19_tearout_composability_plan.md) (its P1 explicitly converges far-B / MG6, and `OpenGraphBeside` summons a second Orrery pane, `session_ops.rs:470`). What remains uniquely here: the persona chip (gated on multi-persona) and per-session engine-profile escalation (manifest field present, unwired)." — accurate: no
- claims checked: 6 — holds: 3, stale: 3, unverifiable: 0

### Stale claims

- **"In progress":** meerkat left the workspace in `c5f01064` (2026-07-18, "meerkat leaves the workspace; the pane model leaves with it"). The plan's own 2026-09-05 note calls it the Meerkat activation record.
- **"meerkat runs multi-graph ... switcher creates / switches / closes / renames":** `build_switcher_thumbnail`, `retag_graph_bound` and `follows_active_graph` are all absent at the base. Pandect keeps `ManifestStore` (`manifest_store.rs:70,260,294`) and the switcher-thumbnail builder, renamed `build_switcher_thumbnail_with` (`switcher_thumbnail.rs:97`, re-exported at `lib.rs:241`), with no consumer outside pandect.
- **"`OpenGraphBeside` ... `session_ops.rs:470`":** neither exists at the base (`session_ops` was split in `18b450ee` on 2026-06-27, then removed with meerkat).

### Contradictions

- The status names `tearout_composability_plan` as the owner of far-B and tear-out. MG6's 2026-06-15 reconcile note and DOC_README name `window_composition_plan`.
- DOC_README's entry repeats the dead `OpenGraphBeside` claim.

### Recommended action

- Change the status to historical (host removed in `c5f01064`). Extract the persona chip (`crates/dramatis/persona-picker` now exists) and the engine-profile escalation (already the engine profile plan's v0b) before archiving (fork).

### Notes

The engine-profile field exists (`pandect/src/manifest.rs:151`) and is unwired. All seven linked docs resolve.

## mere_docs/implementation_strategy/2026-06-10_multi_window_plan.md

- disposition: historical-marked
- status line: "Status: MW1–MW3 done (the per-window reshape, the `WindowId` registry, one-device/N-surfaces, spawn/close, slim leaf chrome — see Progress). MW4–MW6 are superseded by the [window composition plan (archived)](../../archive_docs/2026-06-19_completed_plans/2026-06-11_window_composition_plan.md), which reframes the second window as orrery (authority) vs panes (views that resolve to an orrery by `graph_id`) and pools the orrery off `Shell` by `GraphId` (the MW6 "IOU", brought forward and converged with far-B) after finding the shared constellation is UUID-keyed and graph-agnostic. The MW4–MW6 sections below are kept for history; read them through that plan. This plan carved the window seam and staged leaf → branch → fork." — accurate: yes
- claims checked: 6 — holds: 4, stale: 2, unverifiable: 0

### Stale claims

- **The "Reframed 2026-07-05" line says the N-runner/N-dom/N-ShellState shape "is now the *current* state."** Meerkat left in `c5f01064` (2026-07-18), and `WindowView`, `Shell` and `WindowKind` are absent at the base. The last Progress entry's "A leaf today still shows the shared orrery" also describes removed code.
- **The line 3 banner points at `crates/graph/subgraph`.** That path is gone (folded in `61894570`, as in the modular integration record).

### Contradictions

- DOC_README already calls the shape "the migration source, not the target" and links the archived one-state migration plan. The plan's Reframed line was never updated to match.

### Recommended action

- Mark the plan historical (host removed). Archive question in Forks.

### Notes

The window composition archive, the one-state design, the one-state migration archive and the tear-out brief all resolve.
