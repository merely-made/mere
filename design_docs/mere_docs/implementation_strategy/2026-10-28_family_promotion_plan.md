# Foundation family promotion: observations, actions and source homes

**Status:** 2026-10-28: cut from `2026-10-19_family_promotion_plan.md`, archived in `archive_docs/2026-10-28/`. Done: source promotion and consumer cutover (five supplier repositories published, seven consumer adoptions pushed), the execution and truthful-shutdown slice, pool admission and worker retirement (Q112, Q113), the apparatus portable-core extraction, the embedding compatibility cohort and the coordinated pool rollout. Open: the observations and semantic action evidence slice (a contained set of pieces is qualified; the broader migrations are not), and the source-home follow-up with repository grouping.

**For:** finishing the boundary consolidation Mark scoped on 2026-10-19 after the extraction: one implementation per shared responsibility, with explicit ownership of state, permissions, execution, lifetime and reported outcomes; names and umbrella repositories are secondary. Execution and shutdown are done; this plan carries the observation and action-result responsibilities and the final source homes.
Not here: the Murm transport shutdown gate (the unfiltered Murm run hung on `a_killed_peer_off_the_overlay_reads_not_connected_when_its_connection_closes`, the follow-up filtered that one case, and no unfiltered Murm transport-suite or physical-radio pass is claimed) and reusable carriage factoring into Murm (consumer-driven), which are `2026-10-28_murm_runtime_plan.md`; custody process isolation (the boundary audit, operation, caller and consent design, direct vault-opening migration and cross-process acceptance, all unimplemented in production), which is `repos/dramatis/design_docs/2026-10-28_custody_plan.md`; the detailed implementation and acceptance of observations (target lifetimes, collector and Registry publication, the Apparatus library, Rootstock accessibility publication, human screen-reader acceptance), which is `2026-10-28_diagnostics_and_accessibility_plan.md`; the shared command contract (declaration, invocation, outcome vocabulary), which was the Hest plan, complete, with its native residue in `2026-10-28_platform_qualification_plan.md`; the 658 missing Exhibit A headers that moved with the promoted trees and the deferred standalone renderer root `crates/cambium/examples/genet_web_smoke`, which are `2026-10-28_workspace_hygiene_plan.md`; Mere's graph ownership migration and the wider protocol and admission consolidation, which are `2026-10-28_graph_semantics_plan.md` (ownership repair lane L1) and the other ownership lanes' plans.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Cleanup direction, 2026-11-03

Mark's goal is to implement the p2panda modules under developed subsystem
owners and reduce Mere to its graph/UI semantics and composition. The working
boundary map, checked against current sources, is:

| Owner | Responsibility | Caller supplies |
| --- | --- | --- |
| Dramatis | identities, credential custody, authentication and identity service | application declarations and operation policy |
| Distillery | inference, compute jobs, leases, mesh and producer surfaces | selected models, budgets and owner decisions |
| Eidetic | stores, blobs, signed durable history, validation and stored-content encryption mechanisms | domain grammar, keys/authority and merge semantics |
| Murm | transports, carrier/session security, discovery, exchange and sync sessions | admitted peer/domain scopes and durable storage adapters |
| Gemot | governed spaces, membership, roles, sharing and group-key policy | identity/custody and store/transport suppliers |
| Armillary/Djinn | execution, task lifetime and supervision | domain services and shutdown/resource ordering |
| Mere | graph semantics, projections, editing and host composition | developed subsystem capabilities |

p2panda-auth is group access control and therefore a Gemot integration, while
Dramatis supplies identities and credentials. Group encryption mechanisms can
live below policy in Eidetic, with Gemot deciding membership and rotation.
Stickleback 0.2 keeps storage and admission in Eidetic; Murm's `murm-sync`
owns the extracted address book, join ceremony and receive drain. The network
receiver reuses the existing policy-before-insert path.
The Distillery extraction plan already owns ESP, mesh and port migration,
including the committed receiving branch and history-preserving cutover; its
older pause is re-evaluated against completed Djinn slimming and publication.
These are implementation boundaries, not a claim that all moves have landed.
Upstream module descriptions: [p2panda](https://p2panda.org/) and
[access-control CRDT](https://p2panda.org/2025/08/27/notes-convergent-access-control-crdt.html).

Current code inventory (2026-11-03): Gemot already has the p2panda-auth group
CRDT adapter in `crates/gemot/src/moot/group.rs` and its wire/store modules.
Eidetic's Stickleback already owns the signed-operation store and carrier-neutral
p2panda group-encryption engine (`group_crypto.rs`); Gemot supplies membership
and ordered control facts. Stickleback's network feature is removed, and its
normal graph contains neither p2panda-net nor p2panda-sync. Murm's T7 records
the source move, receiver boundary, live convergence/rotation/shutdown tests
and actual consumer receiving cohorts. A manifest entry alone is not module
acceptance.

Current execution: Dramatis's D15 has extracted the reusable resident identity
service. Distillery, compute mesh, probes and inference patches now live in
the renamed Distillery repository, with seven consumer repositories qualified
by the Distillery extraction plan. Turnstone calls Pandect's graph retention
API and the direct Dramatis/Ostiary clients. The remaining domain moves and
each p2panda module integration keep their owner plans and must carry
source-qualified receiving evidence. The refreshed
[source homes map](../technical_architecture/2026-10-28_source_homes_map.md#remaining-extraction-and-directory-scope-2026-11-03)
scopes the remaining leaves, adapters and directories. Station Custody has moved.
The accepted 2026-11-04 direction is Gemot's shared site generator and serving
domain out of Mere/Knot, then Mora/CMUdict into Knot, Pipit into Murm, and Turquet
into Cleromancy. Libraries keep independent boundaries. Turnstone's richer daily
sky implementation supplies Cleromancy's shared embeddable sky capability; both
applications consume it while retaining their own presentation/interpretation.
The source homes map and site-publication P12 own the detailed gates.
Workshop destinations retain D16's separate checkpoint.

## Constraints

- One implementation per shared responsibility. Execution owns actor and thread completion, typed messaging, task scopes, cancellation and shutdown mechanics, with the caller supplying resource order, policy, deadlines and domain meaning, and optional Tokio support never mandatory for synchronous consumers. Service supervision composes domain owners without duplicating their journals, job grammar, authority or retention. Distillery keeps its own jobs, leases and scheduling (a job scheduler and a task executor differ). Observability owns the shared envelope, bounded retention, causal references, independent readers and explicit loss, while producers own redaction and outcome truth (family promotion 194-213).
- Existing homes are starting points: Armillary for execution, Djinn for supervision, Distillery for compute, Castellan for custody, Apparatus for observations, Genet, Cambium and Mere adapters for application semantics. Repository membership is not an authority boundary (family promotion 216-220).
- Secret custody: general workers and diagnostics receive only the operations or material their admitted contract requires; the mixed derived-key release stays unchanged pending the custody design (family promotion 212).
- Accessibility, automation and inspectors consume the owner's projection and admission path; observing an operation does not authorize controlling it (family promotion 214).
- Share only contracts that represent the same fact. Keep the registry's nonblocking ingress separate from Apparatus's independent-reader retained history, and each layer's count, byte and age limits, rejection policy and loss accounting. No universal event enum, global semantic revision or new diagnostics daemon (family promotion 349-357).
- Hosts and Chirograph keep authorization, confirmation, current-target and session, document and revision lifetime validation, dispatch and execution. Hest's effect and destructive declarations describe commands and grant no permission. An accepted invocation or handled input establishes admission or dispatch only; terminal completion needs the executing owner's actual reply, so Chirograph `IntentResult::Accepted` never becomes terminal `Done`. Apparatus records a redacted copy with producer-supplied operation and source scope (family promotion 374-382).
- UxTree's Inker-specific projections and ux-events' browser vocabulary remain adapters that generic users must not inherit. Diagnostics and local automation keep working without Djinn running (family promotion 385-390).
- Promotion preserves public APIs, crate names, licenses, stored formats, authority rules and default feature semantics. Storage-only consumers never acquire replication, an updater or participant execution; Stickleback keeps policy-before-insert; Luggage keeps a portable release identity core and a separately selected native updater; Servitor evaluates host-supplied authority while hosts execute work (family promotion 13-25).
- The versioned Stickleback 0.2 / Murm T7 cut is an explicit API and feature exception: callers import live sessions from `murm-sync` instead of `stickleback::net`. Stored bytes, admission policy and lane identifiers retain their existing semantics; receiving roots qualify the breaking import change together.
- Excluded: a universal transport rewrite, a replacement async runtime, new product panes, a universal automation protocol, model or backend work, a new capability algebra, a vault-format migration and new restart or retry defaults (family promotion 419-422).
- Pool drop is nonblocking: it neither cancels jobs nor joins OS threads, an actor's completion handle owns the completion observation, and `workers()` counts threads ever spawned and is no shutdown signal (family promotion 742-746).

## Phases

### F1. Observations and semantic action evidence, the broader migrations
The contained pieces are qualified (fresh host admission and routing results, the opt-in Registry-to-Apparatus bridge, shared DOM semantics, shared leaf and producer composition, flattened producer discovery, producer content-origin repair, producer identity validation, read-only observation and selector). Open: product startup wiring; target lifetime tokens and generations; shared semantic construction; neutral contributor contracts; mutable read and act boundaries; process-final diagnostic evidence; full producer coordinate mapping (forward transform and clip, flattened-ancestor mapping, which stay with the supplier and are not duplicated in Mere); read-only and immutable semantic observations; report and collector adoption; Registry publication and other product adoption. Detailed implementation and acceptance stay in the diagnostics plan.
- Done when: the two paths (a worker and a UI evidence path) produce bounded, redacted evidence with independent inspection and test readers; overload and restart gaps stay visible; presentation stamps keep their qualified meaning of queue submission, with separate run and host scope; exact image and state correlation and human screen-reader acceptance remain the diagnostics plan's separate gates.
- Needs: `2026-10-28_diagnostics_and_accessibility_plan.md` phases; the Hest H3 and H4 deliveries
- Rulings: the boundary scope of 2026-10-19
- From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:4,274-365,392-398`

### F2. Truthful action results through the owner's admitted path
The remaining action migration carries the admission-versus-completion distinction through the Hest owner's two-host and Chirograph round-trip gates, then through a selected current target's automation and platform adapters. Taproot and Mesquite consume the host's retained projection and geometry.
- Done when: replaced or expired targets refuse; unsupported, stale, ambiguous, cancelled and failed operations retain their actual results; retry and cancel controls traverse the original owner's admitted command path.
- Needs: F1 for target lifetimes; the Hest plan's published delivery cohorts
- Rulings: the boundary scope of 2026-10-19
- From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:366-398`

### F3. Source homes and repository grouping
After the functional seams qualify, promote the resident and its custody implementation, and choose a workspace grouping from the actual dependency graph; separate public crates can share a repository without acquiring each other's runtime dependencies. History, notices and immutable consumer pins are preserved as in the completed promotion. The eventual repository grouping stays open until the dependency cuts are qualified. No umbrella name, Armillary relocation or mandatory Apparatus promotion is a prerequisite.
- Done when: the resident and its custody implementation have a source home, a workspace grouping is chosen from the real dependency graph, and one external foundation consumer and one real resident consumer are qualified.
- Needs: F1 and F2; the custody plan, since separate crates, cargo-deny and source-home promotion do not establish containment
- Rulings: the boundary scope of 2026-10-19
- From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:10-11,400-417`

## Open questions

- **Which Q112 and Q113 lines stand?** Two stale lines say pool admission and worker retirement remain open, while the 2026-10-21 section records both repaired in Armillary `92d5b15` (the product root stays on its earlier pin pending the ESP, Knot and Mere intake, completed by the 2026-10-22 rollout). From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:551-554,653-654,733-775`.
- **Which hairy, high-impact issues are the named follow-ups?** The original instruction was to document issues that would consume the bounded audit pass and retain them as named follow-ups with evidence; none is enumerated in the plan. From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:422-425`.

## Findings

- 2026-10-19: the Eidetic repository owns Muniment, Chartulary, Eidetic, Hagiograph, Stickleback, Luggage, `mere-proofs` and Servitor as separate crates; Armillary has its own repository (and the sibling crate `apparatus-core`); ESP its own for model execution; Gemot its own for the governance and community family (Mien, Mooting, Moothold, Commons); Errand continues in the Smolweb repository as `merely-made/errand` with its thirteen protocol crates. Mere remains the graph application platform: Graphshell, Djinn, Cambium, Conatus, the graph kernel, canvas and application integration stay (family promotion 11-37, 179-192).
- 2026-10-21: `mere-apparatus` stays Mere's integration facade over `apparatus-core`: it re-exports the public types and forwards `serde`, while the Registry collector, the structural accessibility projection and the default projection behavior remain in Mere (family promotion 679-703).
- 2026-10-28: F3's dependency evidence is in [the source homes map](../technical_architecture/2026-10-28_source_homes_map.md). With no consumer in Mere, `station-custody` (used only by Murm's `signalman-desktop`), the four Conatus engine crates (used only by Isocosm) and Knot's three crates sit in Mere. `mesh` and `mere-site` have no Mere dependency. `document-host` and `ux-events` have no consumer anywhere read.
- Not established: any unfiltered Murm transport-suite or physical-radio pass (family promotion 541-543); durable audit-log qualification by `apparatus-core`, which is a bounded observation copy while products keep authoritative results and persistence (family promotion 707-708); the Knot checkpoint feature lane and an unfiltered standalone Knot suite; native UI, physical devices and heavy models, which keep separate owners (family promotion 591-593, 626, 835-836).

## Progress

- 2026-11-04: shared native site model and host code moved to Gemot `4492351`;
  Knot and Mere now consume those packages directly. The shared generation and
  receiving plan lives in Gemot. Mora/CMUdict, Pipit and Turquet remain next,
  in that order; the community namespace/agreed-work workflow remains in scope.
- 2026-11-04: Mark accepted the generator and consolidation order. The generator
  spans native formats and later Markdown/HTML targets; scripting remains a
  follow-on design. Turnstone's sky embedding is a target, not current behavior.
  Mark then clarified that its richer daily-sky implementation should be the
  donor for the shared Cleromancy capability, preserving both consumers' behavior.
- 2026-11-04: refined the source-home scope after Mark's site and small-repository
  review. The source homes map owns the recommendations and receiving gates;
  no Site, Mora, Pipit or Turquet source move has landed in this refinement.
- 2026-11-04: Station Custody moved to Murm's standalone
  `crates/station-custody` workspace. Its twelve Rust files retain their donor
  blobs; Signalman uses that local path. Mere's member/source removal leaves
  87 workspace members and removes the station/radio family from the default
  native normal/build graph. Resident NomadNet still has an optional Retinue
  dependency. The source homes map and Murm's family integration plan record
  receiving tests and the separate physical/headed boundary.
- 2026-10-28: cut from the family promotion plan; nothing new landed.
- 2026-10-28: source homes map added for F3; nothing moved.
