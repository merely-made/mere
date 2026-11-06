# Resident split plan

**Status:** 2026-11-05: plan. Nothing has moved. D1 to D4 settle the four graph-free pieces;
the sync and inference slices and F1 remain to be resolved before their source moves.

**For:** splitting `ports/mere-resident` (75 source files, 43,624 lines including inline tests
and the receipt-host binaries) by responsibility, so that each owner's reusable behaviour is
tested in its own repository and Mere supplies only concrete graph bindings and host
composition. Opened by the source homes map's resident review
(`../technical_architecture/2026-10-28_source_homes_map.md`, "Resident and inference adapter
review"), where Mark chose a dated plan over keeping the review in the map.
Not here: Djinn's own slimming (Armillary's `2026-10-31_djinn_slimming_plan.md`); the Dramatis
application move and the Distillery workshop cut (the map's ownership cuts 1 and 2); the
resident inference lane's own phases (`2026-10-28_resident_inference_plan.md`); pairing
semantics (`2026-10-28_device_pairing_plan.md`).

Receipts go to `archive_docs/receipts/2026-11-05_resident_split_plan_receipts.md` (DOC_POLICY §8).

## Constraints

- **Store lifetimes stay with the store** (distillery-extraction D19). Pandect's reservoir owns
  opening, durability and closing of every mere; Djinn and embedding hosts execute it. No slice
  assigns storage lifetime authority to a background process or to a receiving owner.
- **Mere keeps graph authority.** Graph replica operations, graph keys, graph-specific
  acceptance, route grants, the doors, the reservoir service and concrete producer
  registration stay here. A piece leaves only when it carries no graph type
  (`graphshell::`, `pandect::`, `kernel::`, graph ids, `MereRoutes`); a coupled piece is split
  at an explicit adapter first, never moved whole.
- **Security code follows standards.** Key groups, seals and custody use maintained
  libraries (Stickleback, Castellan) as today; no slice reimplements them at the receiver.
- **Licence gates travel.** Retinue and Tulle are Reticulum-licensed and stay behind
  off-by-default features (`moot-site-nomadnet`, `knot-site-nomadnet`) wherever they land.
- **The embedded resident composes the same pieces** (reservoir plan V5): every moved piece
  remains composable by an application that embeds the resident, not only by Djinn.

## Inventory

Measured 2026-11-05 at `44db9bccd`. "Graph refs" counts graph-type references in the file.

| Piece | Lines | Graph refs | Owner after the split |
|---|---|---|---|
| `conditions.rs` | 1,175 | 0 | Distillery `mesh-host` (D1) |
| `lock_triggers/` | 833 | 0 | Dramatis (D2) |
| `identity_ui/` | 569 | 0 | Dramatis (D2) |
| `moot_site.rs` | 1,179 | 0 | Gemot (D3) |
| `micron_site.rs` | 464 | 0 | Murm, beside Retinue (D3) |
| `resident_blobs.rs` | 400 | 0 | Eidetic (D4) |
| `resident_events/` | 505 | 0 | open (F1) |
| `resident_inference/` | 1,185 | 0 direct, 21 `crate::` | S5 |
| `sync/` | 5,583 | coupled | S6 |
| `resident_mere*`, reservoir, doors, sites, settings, status, devices, Knot, bins | remainder | coupled | Mere |

## Slices

Each slice updates members, paths, features, scripts, notices and documentation readers
together, compares donor and receiver blobs before deleting the donor, and runs targeted owner
tests plus locked checks of every actual consumer. Headed or process tests run only where
runtime behaviour changes.

### S1. Device conditions to mesh-host (D1)
Move `conditions.rs` (Windows Win32 sensing, stated thermal, `SignalProvenance`,
`validate_policy_coverage`) into Distillery's `mesh-host` as the real `ConditionSource` beside
its stub. Mere keeps the line that selects it.
- Done when: its 21 tests pass in Distillery on Windows and the non-Windows path builds; the
  resident's conditions behaviour is unchanged; `mesh-host` gains no graph or Mere dependency.

### S2. Vault-lock triggers and identity dialogs to Dramatis (D2)
Move `lock_triggers/` (policy, idle sampler, Linux logind/Mutter and Windows WTS/suspend
signals) and `identity_ui/` (platform dialogs, Windows Hello presence window) into Dramatis
beside `dramatis-resident`.
- Done when: their tests pass in Dramatis; Win+L, suspend and idle lock the vault in the
  resident as before (process check on Windows, logind check on Linux); Mere links them through
  Dramatis and holds no OS lock code.

### S3. Moot publication to Gemot, NomadNet carriage to Murm (D3)
Move `moot_site.rs` to Gemot beside `gemot-site-host`; move `micron_site.rs` to Murm beside
Retinue, keeping its feature gate off by default.
- Done when: their 10 tests pass at the receivers; `moot-site` and `moot-site-nomadnet` resident
  builds check against the published receivers; a default resident build links no Retinue.

### S4. Leased blob custody to Eidetic (D4)
Move `resident_blobs.rs` (one iroh store lent to personal-graph transfers and Knot evidence
under per-domain leases, with verified copy of old per-lane stores) into Eidetic.
`sync/transfer_staging.rs` stays: its three-hash agreement check is graph-transfer specific.
- Done when: the lease and migration tests pass in Eidetic; personal-graph transfer and Knot
  evidence still share one store in the resident; old stores are copied and never deleted.

### S5. Inference: protocol out, lane composition stays
`resident_inference.rs` composes Distillery's `InferenceLane` and stays. `mesh_discovery.rs`
exchanges attested compute endpoints only after proof of current graph authority through
`PersonalSyncHost`: the signed endpoint exchange is a candidate for Mesh, the authority check
stays. `personal.rs` binds inference's private log to the personal graph's key group and stays.
- Done when: the endpoint protocol, if separable, is tested in Distillery without a graph; the
  authority check remains in Mere; the inference plan's native gates still pass.
- Needs: a read of `mesh_discovery.rs` to confirm the protocol separates; a fork if it does not.

### S6. Sync: generic carriage to Murm, graph replica stays
`personal_sync_host.rs` (2,514), `graph_keys.rs`, `carriage_host*.rs` and `transfer_staging.rs`
own Graphshell's personal replica and stay. `personal_seals.rs` (125) borrows the key group to
seal sibling lanes. Review each for generic joining, discovery or live-sync mechanism that
Murm's `murm-sync` and `transport` should own, without copying Graphshell's operation grammar
into a transport library.
- Done when: every mechanism moved is tested in Murm with no graph type; lock, shutdown and
  restart behaviour matches the existing personal-sync receipts.
- Needs: a per-file read; each candidate comes back as a fork.

## Open forks

- **F1.** `resident_events/` (505, zero graph refs) writes Djinn's lifecycle event file and a
  local diagnostics copy under the Djinn test harness plan. Candidates: Armillary beside
  Djinn, or Mere as harness-specific composition.

## Decisions

- **D1. Device conditions go to mesh-host (Mark, 2026-11-05):** "Distillery mesh-host", over
  keeping them in Mere. Put to Mark on finding `conditions.rs` graph-free and the only real
  reading behind mesh-host's `ConditionSource` stub. Outcome: applies the source homes map's
  Distillery bullet ("device-condition sensing").

  Links: summary Device-condition sensing lives in Distillery's mesh-host; Mere selects it.

- **D2. Vault-lock triggers and identity dialogs go to Dramatis (Mark, 2026-11-05):** "Move
  both to Dramatis", over moving triggers only and over keeping both. Put to Mark on finding
  both graph-free and implementing Dramatis's vault-lock rulings and native identity callback.
  Outcome: refines the map's "Mere retains platform dialogs", now "Mere retains graph routes
  and concrete producer registration".

  Links: summary Dramatis owns its vault-lock triggers and native identity dialogs.

- **D3. Moot publication to Gemot, NomadNet carriage to Murm (Mark, 2026-11-05):**
  "Moot→Gemot, Micron→Murm", over both to Gemot and over keeping them. Outcome: applies the
  map's Gemot (publication policy) and Murm (generic carriage) bullets.

  Links: summary Moot publication goes to Gemot and its NomadNet carriage to Murm, licence gate kept.

- **D4. Leased blob custody goes to Eidetic (Mark, 2026-11-05):** "Eidetic", over Murm's
  transport and over keeping it. Outcome: applies the map's Eidetic bullet ("generic blob
  acquisition, retained claims").

  Links: summary The resident's leased iroh blob custody becomes Eidetic storage machinery.

## Findings

- 2026-11-05: seven pieces (12 files, 5,125 lines) carry no graph type; the rest of the
  resident is coupled to graph routes, the reservoir or the doors.

## Progress

- 2026-11-05: inventory and D1 to D4; no source moved.
