# Family update resident: Luggage and the Djinn resident

**Status:** 2026-10-28: cut from `2026-08-22_djinn_family_resident_services_plan.md` (the entry gate, phases A to G and
the section 11 release gates), `2026-08-20_device_resident_consolidation_plan.md` and the H4 tail of
`2026-07-27_graphshell_reference_host_plan.md`, archived in `archive_docs/2026-10-28/`. The device resident is built
(R1 to R5, C1 to C4, V1 automated, Turnstone's headed edit, close and restart slice) and Djinn replaced Graphshell's
device host as the desktop composition root. A4 is partly built (`ReleaseRefV1` exists). Open: the `djinn 0.0.2` entry
gate, A1 to A4, B to G, the release gates, the physical two-device and remaining headed receipts, the command-palette
frame-time item, and the resident's real sign-out or reboot recovery.

**For:** one product-neutral desktop resident for the Merely family that carries device services while no application is
running: Luggage's update policy, verified staging and apply boundary; a Djinn family update resident that polls,
stages and projects status; Hocket as its first migration; Djinn updating itself through the platform launcher; a
scheduler extracted from real jobs; notifications, the authenticated agent door and DNS-SD advertisement; firmware
artifacts without resident flashing. Beside it, the device-resident topology's remaining acceptance (one logical
resident owns the selected persona's durable stores, approved by Mark on 2026-08-20) and the H4 recovery tail that
moved here with the resident host.
Not here:
- ordinary site serving and governed hosting (the old section 13, its "settle and verify" list, the headed Knot handoff
  and the further slices): `2026-10-28_site_publication_plan.md`;
- Tucket and Sennet custody, consumer nonce reservations and the radio gates: Murm's family move plan (R6 to R8,
  `murm/design_docs/2026-10-13_murm_family_move_plan.md`) and `2026-10-28_murm_runtime_plan.md`;
- observation and action gates: `2026-10-28_family_promotion_plan.md`; the custody process boundary and the resident's
  three authority tiers: `2026-10-28_custody_plan.md`;
- the fleet rollout of the resident, its Linux launcher and the Windows switch from the legacy host:
  `2026-10-28_device_pairing_plan.md`;
- the rest of H4's follow-ons (native Firefox key import, macOS and Linux dialog providers, a configured carry root,
  broader carry management): `2026-10-28_graphshell_host_plan.md`;
- the WebRTC carrier's use of `ReleaseRefV1`: `2026-10-28_browser_carrier_plan.md`; firmware flashing is Linkboy's.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **Update invariants.** An application never needs to be running for checks, downloads or honest update status to
  advance. Policy is per installed application (off, notify, download then ask, automatic) with channel, cadence and
  metered behaviour explicit. Store, distro or enterprise-managed installs are reported as externally managed and never
  overwritten; Djinn never crosses an installation privilege boundary. Release keys are offline product keys, separate
  from Personae identity. Manifest and artifact are both verified before staging; apply revalidates the whole persisted
  offer; a staged offer is bound to application id, target, version, format, canonical install root, digest and signed
  manifest. `Applied` means the newly launched binary reported its actual version, never an installer spawn or exit.
  A release reference is a manifest hash plus publisher-key identity and carries neither a feed nor trust; advertising,
  resolving or joining a release never changes an installed application's trusted publisher, feed, channel or policy.
  DNS-SD advertises only enabled service records and discloses no persona, vault, place, document or release key.
  Source: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:94-119`.
- **Ownership.** Djinn: lifetime, scheduling, installed-app inventory, local projections (it contains the scheduler).
  Luggage: policy grammar, signed feeds, verified staging, platform apply. Athanor: one scheduled service, a proposal
  emitter, never the generic scheduler or a graph-truth authority. The notification domain: records and delivery rules.
  Murm: authenticated streams and DNS-SD mechanics. Personae and Castellan: caller identity and secret authority.
  Gazette: directory resolution, feed polling, contact intake (ruled 2026-10-01). Linkboy: firmware compatibility,
  consent, transfer, apply; Djinn never flashes. Source: same file `:39-68,112-115`.
- **Non-goals.** A universal daemon framework; a public MCP, Knot, Misfin, Gemot or relay service without a forcing
  consumer and admission model; silent firmware flashing; replacing distro, store or enterprise package managers;
  treating DNS-SD reachability as authorization; staged percentage rollout before a release operator needs it. Source:
  same file `:487-496`.
- **Default-off comms is a product requirement of the resident** (Djinn owns network runtimes); Turnstone's `src/place/`
  already delivers it by construction, every network action being a user verb. Source:
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:369-372`.
- **One logical device resident** owns the selected persona's durable stores and network runtimes: a user-scoped process
  on desktop, embedded in a mobile or sandboxed application, in-memory in tests. The invariant is one owner per durable
  resource, not one executable shape. Djinn is the desktop composition root; Graphshell stays the local-session and
  admission layer. Source: `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:23-43`.
- **Resident invariants the open receipts test.** One live process owner per persistent redb or iroh store. A shared
  physical store authorizes `(scope, peer, hash, mode)`. An artifact is exposed only after byte count, transport BLAKE3
  hash and portable digest verify. Consolidating process lifetime does not consolidate network identities. Personal
  authority comes from Personae pairing, communal from a Gemot projection; route hints are reachability only. Arbitrary
  directory editing may stay embedded in Knot or Turnstone. The stdio carrier stays a conformance deployment, not the
  production owner of a persona vault. HTTP(S) fetching stays per host process. Source: same file `:111-116,123-143`.
- **Stop line.** No `.knot` format; no artifact bytes in p2panda operations; no GraphQL or HTTP replacing Graphshell; no
  unreviewed adoption of aquadoggo or `p2panda::Node`; no single network identity; Gemot is not a process supervisor;
  directories need not join a persona vault. Source: same file `:494-506`.

## Phases

### Entry gate. Publish the resident baseline
Phases A to G start after `djinn 0.0.2` and its public dependency chain are published and pinned by a clean consumer.
At the 2026-10-06 status djinn is version 0.0.2 but its `knot-site` dependency is Git-only, so `cargo package` refuses
it; packaging stops at workspace dependencies without registry version requirements, beginning with `cambium` through
`knot-editor`. This plan targets the next source version and folds no new service into 0.0.2.
- Done when: `cargo package -p djinn --locked` succeeds from a clean checkout; `djinn 0.0.2` is published with the
  resident positioning landed; a clean external probe resolves `djinn = "0.0.2"` without path or Git substitutions and
  starts the resident fixture; the release commit and package contents are recorded before any work below changes the
  public API.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:20-27,70-87`

### A1. Extract the proven policy grammar into Luggage
Hocket owns the first `UpdatePolicy`, `UpdateChannel`, `UpdateStatus`, `decide` and update worker. Move the
product-neutral types and pure decisions into Luggage and add metered-network behaviour, an explicit externally-managed
state, durable timestamps (last attempt, last success, next eligible check) and failure phase and reason with no generic
working state. Hocket's settings projection and rendering stay in Hocket until C.
- Done when: Hocket and a Luggage-only test consumer use the same public policy and status types; the four policies
  have table tests for check, download, consent and apply decisions on metered and unmetered networks; unknown
  persisted policy values fail visibly; no UI, Genet, Graphshell or Djinn dependency enters Luggage.
- Needs: entry gate
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:123-148`

### A2. Persist a self-sufficient verified offer
`Update::install_staged` still needs the ephemeral `Update` from the original feed check, and the stage record keeps
mutable format and extraction-path fields while rechecking only the artifact BLAKE3 digest. Replace it with a staged
offer that supports `StagedUpdate::apply(expected_app, current_executable)`: it persists the signed manifest bytes and
signature, the artifact signature and the fields that rebuild the verified release, and rechecks signatures, digest,
application identity, target, version ordering, format and permitted install root. Apply returns an explicit
disposition (for example `RelaunchStarted`, `AppliedExitRequired`, `Deferred`), never inferred from `Ok(())`.
- Done when: a fresh process applies a stage without the network or an `Update` object; tampering with any persisted
  offer field is refused; a stage for one application or target cannot be applied to another; interrupted writes
  recover as absent or resumable, never as a valid partial stage; Windows, macOS and Linux tests cover each installer
  format's process disposition.
- Needs: entry gate
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:150-184`

### A3. Preserve rollback and channel history
Luggage persists the highest accepted release per application and channel. Older signed releases stay authentic but are
refused as automatic updates. An explicit rollback, if later wanted, is a separate user-authorized operation with its
own receipt.
- Done when: the source sets none; the rule above is the content.
- Needs: entry gate
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:186-191`

### A4. Name a release independently of its feed and target
Partly built: `ReleaseRefV1` (manifest BLAKE3 plus a domain-separated publisher key id, a lookup identity and not a trust
grant) exists in Luggage and the WebRTC carrier's invite uses it. Open: `ReleaseOfferV1` (a reference plus disposable
per-artifact locators from a feed, host, mirror or peer, never trusted) and the versioned signed manifest: stable
application id and release version, source repository and revision (a signed claim, not reproducible-build proof),
supported invitation and protocol versions, and artifacts keyed by kind and target (native installer, browser bundle,
later firmware) each with BLAKE3 digest, minisign signature and format. The v1 `luggage.json` embeds artifact URLs, so v2
needs a compatibility reader and a split writer. The envelope verifier stays free of filesystem, HTTP, installer, Djinn,
Graphshell and Genet dependencies so the `mer3ly.net` bootstrap loader can verify a browser bundle before executing it.
- Done when: native and `wasm32-unknown-unknown` consumers derive the same `ReleaseRefV1` from frozen signed-manifest
  vectors; changing application id, source revision, compatibility or artifact digest invalidates the manifest signature
  and reference; changing only `ReleaseOfferV1` locators leaves the reference unchanged and cannot make mismatching
  bytes verify; an unknown publisher id stays resolvable but is never described as trusted; the same verified artifact
  bytes are accepted from HTTP, a directory and an injected in-memory source; selecting a browser artifact cannot select
  a native installer or firmware entry with the same target spelling.
- Needs: entry gate; its consumer is `2026-10-28_browser_carrier_plan.md`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:193-264`

### B1. Installed-app inventory
A versioned, device-local registry of installed family applications. An installer writes or updates its own record;
arbitrary processes cannot choose another application's executable or release key. A record carries application id and
label, install owner (self or externally managed), canonical executable and allowed installation root, target and
formats, stable release-key identity, a feed per channel, per-app policy and stage directory. An offered `ReleaseRefV1`
may use a record only when verified application id and publisher key match; adopting a fork is an explicit change to
trusted publisher key and feed, and accepting its live session is not that change. A record cannot hold a command line.
- Done when: registration, replacement, removal, malformed records, path escapes and externally managed installs have
  focused tests; registration is atomic and idempotent; uninstall removes the registration and stage without touching
  another app; a copied development binary is not guessed to be an installed app.
- Needs: entry gate; A1 for the per-app policy fields
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:268-298`

### B2. Resident polling and staging
`FamilyUpdateResident` loads the inventory, evaluates each policy, schedules eligible checks, invokes Luggage and persists
status transitions; cadence, concurrency, retry, jitter, channel and metered behaviour are policy inputs. It
distinguishes disabled, externally managed, waiting, checking, available, awaiting download consent, downloading,
staged, awaiting install consent, apply on next launch, applied and confirmed, and failed (phase, reason, retry
eligibility). Djinn attempts every service shutdown and reports failures through the `distillery::lifecycle` close report.
- Done when: two fake apps on different policies advance independently under a deterministic clock; closing every app
  does not stop checks or downloads; metered refusal and later resumption are chronological tests; a restart restores
  the exact durable status and never repeats an apply the new binary acknowledged; one broken feed does not block
  another application's schedule; shutdown cancels network work, preserves valid stages and leaks no task.
- Needs: B1; Luggage's policy and staged offer (A1, A2)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:300-334`

### B3. Local status projection and intents
Project the durable statuses through Djinn's admitted local surface; hosts render it and submit typed intents (check now,
download, approve install, defer, discard, policy changes). The projection exposes no feed credentials, release keys,
installation paths or staged bytes, and caller admission precedes every mutation.
- Done when: the source sets none; the rules above are the content.
- Needs: B2
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:336-343`

### C. Migrate Hocket and prove the second consumer
Move Hocket's device-local update settings into the Djinn app record by an idempotent migration; remove its check and
download worker and in-memory Luggage transport; keep the status renderer, explicit intents and the early startup hook
that asks Luggage to apply an approved stage before Genet, winit or the audio engine start. If Djinn is unavailable
Hocket reports the resident updater unavailable and never resurrects a polling loop; an explicit release-test command
exercises the same resident service.
- Done when: a signed Hocket update is discovered and staged while Hocket is closed; notify does not download,
  download-then-ask does not apply without approval, automatic applies at the next natural launch; Hocket reports its
  launched version to Djinn, which alone advances the state to applied; the Windows, macOS and Linux installed-update
  receipts pass through the resident path; Hocket has no periodic feed poll or background update worker afterward.
- Needs: B2, B3
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:345-372`

### D. Djinn updates itself
The platform launcher owns the final handoff (Windows Task Scheduler, a launchd user agent, a systemd user unit or the
selected equivalent). Djinn stages and records intent, then exits cleanly; the launcher runs the verified Luggage apply
path before restarting the resident; store locks and network endpoints are released before apply; the launcher keeps a
recoverable previous installation until the new resident's startup receipt succeeds.
- Done when: the resident updates itself from N to N+1 without a second live owner; failure before replacement
  restarts N, failure after replacement restores or explicitly offers the working previous version; the scheduled-task
  identity and profile selection survive the update; no general elevated update service is introduced.
- Needs: A2, B2; the Linux launcher is `2026-10-28_device_pairing_plan.md` D2
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:374-395`

### E. Extract the resident scheduler through real jobs
Only after the update resident works. The first two consumers are family updates and one Athanor maintenance pass (the
consolidation and forgetting passes, recorded as carried here; the crate's ruling is "scheduled, not resident").
Gazette's feed polling is a later consumer (ruled 2026-10-01) and does not change the first two. Athanor still emits
proposals; the authority that owns the affected store reviews or applies them.
- Done when: the common contract is limited to due-time calculation under an injected clock, cancellation and orderly
  shutdown, configurable jitter, retry and missed-run behaviour, durable last-run and next-eligible facts, and
  structured outcome reporting.
- Needs: B2
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:397-413`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:610-613`

### F. Notifications, the agent door and DNS-SD
F1: update availability, consent requests, staged updates and failures are the first notification consumer. A
product-neutral domain (`notifications` is a working package name) owns `NoticeRecord`, `DeliveryRule`, deduplication,
acknowledgement, expiry and delivery; Djinn owns lifetime and local projection; hosts choose presentation. The earlier
model still binds: the log is the Steward's, the toast is the chrome's, actions ride the toast, continuous chips are not
notifications, dedupe and rate-limit come before rollout. F2: one authenticated local socket for agent clients; Personae
and Castellan decide caller and secret authority; MCP is an optional adapter; remote listeners stay out of scope until a
consumer supplies an admission and disclosure model. F3: Murm implements DNS-SD browsing and advertisement; Djinn supplies
the registry of enabled services and their disclosure-safe records; LAN discovery is configurable, creates no
authorization and does not replace owner-selected relays (its substrate is R0 of the reachability rungs, carried in
`2026-10-28_browser_carrier_plan.md`).
- Done when: an update notification appears once, survives a host restart and can be acknowledged from two hosts without
  duplicate delivery; an admitted local agent can enumerate only its allowed tools and receives no durable secret
  material; two LAN devices discover an enabled service without a relay while unconfigured services and all persona
  identifiers stay absent from DNS-SD; disabling discovery withdraws the advertisement and leaves authorization unchanged.
- Needs: B2 for F1's first consumer; Murm for F3
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:415-458`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:326-329`

### G. Firmware artifacts without resident flashing
Extend Luggage's signed manifest envelope with an explicit artifact kind and signed firmware facts: board, hardware
revision, region, minimum bootloader, image digest. Djinn may check, download and stage a compatible-looking offer for a
registered device family; Linkboy independently validates device and offer, obtains consent, transfers, flashes and
reports the version. An automatic desktop policy never implies automatic flashing.
- Done when: the source sets none; the rules above are the content.
- Needs: A4
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:460-469`

### Release and review gates
Each phase lands as an independently buildable slice. Before a new public release: Luggage and Djinn focused suites with
a deterministic clock and fake feeds; Hocket's installed update cycle on Windows, macOS and Linux; review of
canonical-path handling, signed-offer reconstruction, anti-rollback, release-reference resolution, publisher-key
changes, caller admission, stage permissions and launcher privilege; packaged contents inspected and every public
dependency resolving from the registry; app-authored and resident-authored receipts for applied version and schedule.
- Done when: the five checks are met for the release in question.
- Needs: the phase being released
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:471-485`

### V1-2. Physical two-device receipt
- Done when: Device A runs the desktop resident and authors a Djot clip through Turnstone; the resident retains the
  artifact and writes its portable reference; Device B receives the signed document operation before the artifact, asks
  the authorized Device A identity for the transport hash, verifies BLAKE3, byte count and portable SHA-256 and then
  exposes the artifact; both devices restart with the artifact still available offline; unpairing Device B prevents a
  fresh fetch and leaves Device A's retained bytes intact.
- Needs: two paired devices (`2026-10-28_device_pairing_plan.md` D2)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:4-7,419-430`

### V1-3. Standalone and directory-only headed receipts
Turnstone's edit, close and restart through the resident route is done. Open: the standalone sync-status part, standalone
Knot embedding the resident composition without Turnstone, a Turnstone evidence-open action, and a directory-only
document editable with the resident stopped. The 2026-08-22 blocker "no standalone Knot executable" is gone (Knot's
sources and its `knot` binary live in `knot-editor`), though the receipts it blocked are not recorded as run. Automation
targeting should move onto the retained painted layout (the receipt used a fixed point because Probe's re-derived
selector layout disagrees with Turnstone's painted textarea); that is scoped to the Genet and Turnstone automation contract.
- Done when: Turnstone edits the resident Knot route while the standalone sync status is visible and remains live after
  the pane closes; standalone Knot embeds the same resident composition and performs the same edit, restart and
  evidence-open flow without Turnstone; a directory-only document stays editable with the desktop resident stopped.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:432-465`

### Command-palette frame time
Reported 2026-08-22: interaction lags while Turnstone's command palette is open. Scope it as a Turnstone chrome
regression, not resident or Knot work. The cause is a code-reading hypothesis (a larger chrome subtree re-synchronised
before the scene is produced); an unoptimised build showed roughly ten times the whole-frame time with the palette
open. The item does not widen this plan's scope or count as a V1 failure until the comparison below is recorded. The meerkat render-performance principles are host-agnostic and speak to it: a dirty-gate rule (M2),
settled-scene per-key caching (M3), a per-(member, viewport) scene set (M4).
- Done when: the same release-build scene and profile are compared with the palette closed and open, publishing median
  and p95 frame time plus input-to-present latency; suggestion recomputation is separated from chrome cascade, layout,
  paint and background resident activity; repeated work caused merely by the palette being open is removed, with a
  regression check at the stage that caused the lag.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:467-492`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:302-306`

### H4. Resident recovery: real sign-out or reboot, and retiring the Personae task
Arrives from the Graphshell reference host plan, whose other H4 work is done. Two operational items remain: the resident
proved to start at login and recover from a crash on the same Windows machine through a real sign-out or reboot, and the
retained, now disabled, interim `personae-agent` logon task finally retired. The resident host moved from Graphshell to
Djinn, so both concern Djinn's launcher (the Windows installer under `ports/mere-resident`).
- Done when: the interim resident task is removed only after launch-at-login and crash recovery are proved for the
  resident on the same machine, by a real sign-out or reboot receipt; the task is retired in the same change that
  replaces it, never before, and the standalone agent is kept until the replacement passes the real SSH and recovery receipts.
- Needs: Djinn's Windows installer; see the first question on the Windows switch below
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-27_graphshell_reference_host_plan.md:636-640,705-718,728-731,806-810,839-841,848-856`

## Open questions

- Does the entry gate bind the publication service? The old section 13's Gemini service shipped on 2026-09-13 without
  passing it; the cut puts section 13 in `2026-10-28_site_publication_plan.md`, and whether the gate then scopes to
  A to G only is not confirmed. From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:89-93`
- Where is `ReleaseRefV1` credited? It landed for the WebRTC carrier, not through A4. Options: credit A4 as partial
  here, or leave it to the browser WebRTC carrier plan. From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:255-264`
- When does Djinn compose Gazette? Gazette is built (`ports/gazette`) but Djinn has no gazette dependency, while the
  ownership table gives Gazette directory resolution, feed polling and contact intake (feed polling is a later scheduler
  consumer, E). From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:57-64,400-402`
- Is H4's Windows sign-out or reboot receipt the same as D2's Windows switch from the legacy `graphshell-device-host` in
  the pairing plan (ruling 15: side by side until the resident signs a real login, then Mark switches)? The 2026-10-06
  question whether H4's tail moves to the Djinn plan is answered by this cut; the overlap is not. From:
  `archive_docs/2026-10-28/2026-07-27_graphshell_reference_host_plan.md:852-856`; `archive_docs/2026-10-28/2026-10-02_device_pairing_by_key_plan.md:535-542,1106-1108`
- Where are the remaining standalone-Knot headed receipts produced: here, or in a `knot-editor` plan? From:
  `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:461-465`
- Does the Turnstone source-evidence receipt run still fail? On 2026-08-21 Turnstone's lock and retired Genet path
  patches rejected the local Mere metadata; the run was not recorded closed. From: `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:526-532`
- Should `p2panda::Node` replace the current implementation? Undecided: the p2panda reference is a topology donor, not
  a dependency ruling; the shared lifecycle crate grows only when a third user adds another observed lifecycle rule. From:
  `archive_docs/2026-10-28/2026-08-20_device_resident_consolidation_plan.md:45-55`
- How does pandect's `SessionServiceRunner` relate to Djinn's scheduler (E)? The tier architecture's ruling 6 makes feed
  polling a Djinn job. Options: the runner stays the session-worker contract; the scheduler supersedes it; both, scoped
  apart. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1091-1094`
- The dead `ToastSpec`, `ToastSeverity` and `FrameViewModel.toasts` code ruling is the workspace hygiene plan's (`2026-10-28_workspace_hygiene_plan.md`); F1 picks its presentation seam after it.
- Strict Clippy on the Djinn port is blocked by existing sync and door diagnostics, and the all-tests graph shows
  unrelated `resident_tasks` and `passphrase_change` fixture failures; the sources name no repair. From:
  `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:14-15`; `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:76-81,238-239`

## Findings

- 2026-10-06: Gazette is built (`ports/gazette`, manifest line 7: WebFinger resolution and unverified Gaz address intake);
  the Djinn composition is what is unbuilt. `ReleaseRefV1` is at `crates/system/luggage/src/release.rs:62` and the
  WebRTC carrier's `invite.rs` uses it; `ReleaseOfferV1` and the versioned manifest are absent. Source:
  `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:60-64,255-259`.
- 2026-10-06: the desktop resident is `ports/mere-resident` (`src/bin/djinn.rs`, `install-windows.ps1`); Graphshell ships no
  device-host binary. The Windows installer disables the retained Personae task after cutover. Source: `archive_docs/2026-10-28/2026-07-27_graphshell_reference_host_plan.md:81-85,828-833`.

## Progress

- 2026-10-28: cut from the Djinn family resident services plan (entry gate, A to G, release gates), the device resident
  consolidation plan (remaining acceptance receipts and the palette item) and the Graphshell reference host plan's H4
  tail; nothing new landed.
