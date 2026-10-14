# The Family Composition Thesis: every application its own datalake, one identity across, composable views

**Date:** 2026-08-12
**Kind:** research brief digesting a chat chain (Mark's framing prompt + assistant
response, 2026-08-12); analysis, terminology alignment, and system-shape prior
art added here. Extended 2026-09-08 with stack-pillar research lanes (§7), and
2026-10-11 with the primary data model, clips and live entities (§8).
Research includes initial model and Rust arena experiments; full consumer experiments and
implementation promotion remain open (see §7's experiment results).
**Anchors:** [application prospects brief](2026-07-24_application_prospects_brief.md)
(the three-seam composition thesis this elevates),
[Graphshell remote projection host plan](archive_docs/2026-10-06_completed_plans/2026-07-22_graphshell_remote_projection_host_plan.md)
and [reference host plan](mere_docs/implementation_strategy/2026-07-27_graphshell_reference_host_plan.md)
(the rulings this extends),
[credential port + gazette brief](mere_docs/research/2026-08-10_credential_port_gazette_brief.md)
(the castellan split this generalizes),
[persona model brief](mere_docs/research/2026-05-14_persona_model_brief.md),
[participant gate + packs plan](archive_docs/2026-10-06_completed_plans/2026-07-17_participant_gate_packs_plan.md).

> **Note (2026-09-23): the unit is the data domain, not the application.** Mark
> ruled that "each data domain deserves its own mere. that isn't the same as
> every app getting its own mere, like two browsers should share a mere", and
> that any of an identity's applications can open any of its meres by default.
> All of an identity's meres together are its **reservoir**
> ([TERMINOLOGY](TERMINOLOGY.md)). This brief's "every application its own
> datalake" framing is kept below as its history. For the current rulings, see
> the [ambiance design](mere_docs/design/2026-09-23_ambiance_design.md) §2.

## 1. The origin framing (Mark, 2026-08-12, verbatim)

> i just look at it as, you can access your google drive in a bunch of apps.
> but what if every app is its own repository, its own datalake which can be
> associated with a cross application identity you create and manage, through
> a p2p substrate, network, and system. and then graphshell lets you look at
> and manage 'em, which is a capability that can be composed into other apps
> (like turnstone or signalman). because the ports are embeddable reference
> models of capabilities native to the mere stack.

The Drive analogy is the inversion that positions the family: Drive-like reach
(your things visible from many applications, from many devices) without the
universal repository, the single account namespace, or the landlord cloud. The
projection brief's Graphshell line already said half of this — "like a cloud
service, except the app is the backend, wherever it runs" — for *reachability*.
The chain extends it to the *data model*: there is no central store to reach.
Each application remains authoritative for its datalake. Composition exposes identity,
association, and views. The original shorthand was "never custody"; §7's
2026-09-08 clarification distinguishes delegated retention from surrendering
domain authority.

The response's concise formulation, kept as the family's four-noun shorthand:

> **Personal datalakes, cross-application identity, peer-to-peer association,
> and composable views.**

## 2. The thesis in canonical vocabulary

The chain's nouns land on existing terms; restated so the corpus stays one
vocabulary:

- **"Every app is its own repository/datalake"** → every application owns its
  **source truth** (the 2026-07-22 ruling: "Turnstone, Woodshed, Isometry,
  Hocket, and a radio-management application remain the authorities over
  their native data"). The spatial unit of that truth is **a mere**, which
  sits at the intersection of two borrowed terms of art: a *dataspace* on the
  integration axis (heterogeneous sources related pay-as-you-go, without
  upfront schema unification) and a *datalake* on the storage axis (raw
  native retention, schema-on-read). Ruled 2026-08-13 and recorded in
  [`TERMINOLOGY.md`](TERMINOLOGY.md), with the *reservoir* gloss for the case
  where the lake term's derivative-copy connotation would mislead. The
  prospects brief's "thin hosts wiring the three seams" and the chain's "rich
  repositories" are the same claim from opposite ends: thin over the
  substrate, rich in domain authority.
- **"Cross-application identity you create and manage"** → **personae** and
  the dramatis tier. The chain's SSO contrast (a Google account gives apps a
  common principal under Google's authority; this gives independently held
  stores recognition under the participant's authority) is the persona model's
  standing claim. Its "prove authorized relationships without learning
  unrelated identity material" is the **insigne** grammar: graded proofs made
  to be shown, where your insigne is what someone else's gaz keeps.
- **"P2p substrate, network, and system"** → stickleback replicated spaces
  under murm/moot domains, retinue's mesh, iroh/p2panda beneath, with the
  standing rule that the substrate never infers authority from transport
  access. The chain's "hosted infrastructure as a convenience without turning
  it into the owner" is the voluntary-hosting posture.
- **"Graphshell lets you look at and manage 'em"** → the ruled port: discover
  granted projection sessions, realize scenes, return typed intents through
  the participant gate.
- **"Projections"** → granted scores and scenes (scenograph); **"intents"** →
  petitions from participants holding grants. The chain's insistence that a
  projection is purposeful (per context, audience, capability) rather than a
  generic serialization is the score/admission machinery described at product
  altitude.

## 3. What the chain adds to the corpus

Four genuinely new items, one narrative device:

**The port law (the castellan split, generalized).** "The ports are embeddable
reference models of capabilities native to the mere stack." Castellan already
ruled this shape for credentials: an embeddable half any host composes, an
authority half that stays with the resident, custody without ownership. The
chain promotes that from a castellan decision to the *definition of a port*:
the stack owns the capability; the port is its first-party, coherent,
embeddable embodiment; and another application composes the relevant subset
without cloning the reference application or inheriting its authority. The
port answers "what does a complete first-party expression look like"; the
capability answers "what can a host embed without violating authority
boundaries." This prevents the standard platform failure where a capability is
technically reusable but practically trapped inside its first application.

**Graphshell as capability, not only port.** The 07-22/07-27 rulings made
Graphshell a client and reference host — the composition surface *over* the
family. The chain adds the reverse embedding: Turnstone or signalman composing
a Graphshell-powered pane (a Woodshed practice artifact beside a research
page; the field manual beside a radio) while remaining themselves. The
in-process precedent exists — a **swatch** is exactly a scoped embedded
projection — so the new claim is that the same composition works at the
session boundary: a host embeds scene realization + intent return the way it
would embed a swatch, holding a lens, never truth. No new machinery is implied
beyond the existing planes; what is new is naming it a first-class arrangement
rather than a degenerate case.

**The anti-shell test.** If Graphshell's capability only works with Graphshell
as the outermost application, the family shipped a shell with plugins. If the
same capability composes into a second host that keeps its own identity,
workflow, and authority, it is a platform capability. This is the product-tier
form of the discipline the engine work already follows ("the framing pays rent
only when a pipeline end has more than one occupant" — proof 4). Worth
adopting as a named gate when Graphshell embedding is ever scheduled: the
receipt is a *second host*, not a second view.

**The capability, named (Mark, 2026-09-02).** The thing that passes the
anti-shell test is the *projection manipulator*, and its name is
**Scenograph** — the boundary plan frees the name by dissolving the generic
facade into Cambium and reserves it for exactly this: the scene/projection
editor product, built with Cambium. The manipulator is authenticated access to all
the data of all the applications, used to customize projections to your own
taste — a home-page graph of app graphs. Graphshell is its preeminent host and
otherwise a viewer and redirector — reading simple structured content through
Genet and Workbench is in scope, the browser proper is Turnstone's — but the
manipulator is meant to appear in every application that
has its own app graph. That fixes what the second host must embed: not a
Graphshell pane, but the manipulator over the host's own graph, holding a
lens, never truth. The suite census §3 carries the charter sharpening and the
two placement consequences it leaves open.

**The narrowing gradient.** Possession ⊇ disclosure ⊇ synchronization ⊇
projection: what an application holds, what a grant admits, what a replicated
space carries, what a score selects — each strictly narrower, each owned by an
existing mechanism (vault/journal; grant/admission boundary; stickleback space
scope; score). Stated this compactly it is testable: any place a projection
can exceed its sync scope, or a sync exceed its disclosure, is a bug class
with a name.

> **Amended 2026-10-04 (Mark, [graph semantics plan](mere_docs/implementation_strategy/2026-10-04_graph_semantics_plan.md)
> ruling 4):** every query and projection result names the layer that limited
> it, and partial residency adds "not loaded". *Reading, not ruled*: residency
> sits inside possession (held on disk or another device, not in memory), so
> the named layers are possession, residency, disclosure, synchronization and
> projection.

**The release narrative (device, not schedule).** Woodshed proves an
application is worth using alone before any composition ships; Personae +
Castellan prove identity crosses applications through both headless and
graphical ports; Graphshell proves discovery and composition; a second host
embedding the capability passes the anti-shell test; Knot proves the
application model isn't secretly one product's shape. This is a *story
ordering*, not the build order — the corpus already interleaves (Knot's port
is furthest along; castellan's OTP core landed first) — but it is the right
order for what each release must *demonstrate*, and it explains why Woodshed
standing alone is a feature of the plan rather than a delay.

## 4. Corrections and cautions (terminology alignment)

- The chain's diagram labels a "Castellan repository … identity state,
  identities, grants, proofs." Castellan owns no repository: identity truth
  is personae's (dramatis tier), and castellan is the keeper — custody
  without ownership, browse-and-consent surfaces over material it does not
  hold. The port law survives the correction; the box label does not.
- **"Repository" should not be minted as vocabulary.** The corpus already
  uses *repository* for git repositories, with a standing rule that
  repository boundaries are packaging, never authority. The chain's sense is
  covered by **source truth** (the authority) and **a mere / datalake** (the
  dataspace). Three senses on one noun would be a lexicon regression.
- Signalman is retinue's application (`apps/signalman-desktop` *(historical citation)* <!-- doc-audit: historical-path -->, founded
  [2026-08-06](https://github.com/merely-made/murm/blob/main/design_docs/2026-08-06_signalman_founding.md)),
  whose founding already states this brief's authority split from its side:
  graphshell surfaces meres rather than owning radios. Its postilion base
  (station logic with no UI) is what would make an embedded cross-application
  pane a face among faces rather than a bolt-on.
- The chain describes application repositories as "heterogeneous… no giant
  universal schema." True and already doctrine (schema at the engram
  boundary; no shared core, the doctrine is the unit of reuse) — recorded
  here so the thesis is not read as a universal-schema proposal by a fresh
  reader.

## 5. System-shape prior art

The adjacent altitudes are already surveyed: protocol adoption in the
[p2p landscape brief](mere_docs/research/2026-05-31_murm_p2p_landscape_brief.md),
the projection engine in the
[prior-art brief](mere_docs/research/2026-07-21_projection_engine_prior_art_brief.md),
feature borrows in the
[borrowed ideas brief](mere_docs/research/2026-06-25_borrowed_ideas_brief.md).
Not yet surveyed is the *decomposition* altitude the chain speaks at: who owns
the data, and how applications compose over it. One lesson each.

| System | Decomposition | Lesson for this thesis |
|---|---|---|
| **Solid** (storages/pods) | Controlled resource storages; an app can read and write across more than one storage; RDF descriptions accompany media-typed resources | **Corrected 2026-10-11:** the [Solid Protocol](https://solidproject.org/TR/protocol) does not mandate one pod or a universal application schema. Shared meaning still requires compatible vocabularies or adapters, as it does in Mere. Compare domain authority, local operation, carrier contracts and composition rather than infer a schema requirement from linked data. The earlier adoption claim was unsupported by this survey. |
| **AT Protocol** (PDS repos) | One signed repo per *user*; apps are lexicon-namespaced collections inside it; portable DID identity | The nearest modern analog, mirrored: atproto = one datalake per person, apps as namespaces; mere = one datalake per app, personae across. Lexicons are its projection-contract analog. Validates identity-crosses-apps; hosting stays server-shaped. |
| **Sandstorm** (grains + Powerbox) | Per-instance app data ownership; user-mediated capability picker brokers all cross-app access | Per-app ownership plus consent-brokered composition worked; packaging a whole SPA per grain made composition heavy. The Powerbox is the ancestor of "only the resident asks for consent" (castellan's anti-spoofing rule). Keep the embeddable unit a *scene*, not an app. |
| **Plan 9** | Every program a file server; one tiny protocol (9P); per-process namespace composition | The cleanest ancestor of "every app exposes projections over a common contract." Composition was free because the contract was minimal and uniform. Sceno scenes + typed intents are the family's 9P; contract growth is the thing to resist. |
| **Holochain** (hApps) | Agent-centric: each app its own p2p network, per-agent source chains, app-defined validation | Literally "every app is its own repository through a p2p substrate" — and per-app sovereignty siloed, because no family-level composition surface emerged. Graphshell-as-capability is the piece whose absence kept that ecosystem fragmented. |
| **OpenDoc / OLE** | Embeddable component parts inside host documents | The embeddable-capability dream, dead of contract breadth, resource weight, and business model. Survival trait the family already holds: one narrow scene contract instead of arbitrary embedded runtimes. |
| **remoteStorage / unhosted** | Per-user storage server; per-app scoped directories; app-agnostic protocol | Bring-your-own-storage failed on adoption friction: the storage had to exist before any app was useful. The release narrative's Woodshed-first rule is the counter: each application must be worth using before the substrate is asked for. |

On novelty, the same verdict as the projection survey: every element has
precedent; no surveyed system combines per-application authority,
participant-held cross-application identity, p2p association without a
landlord, and an embeddable composition capability. The combination, not any
row, is the claim — and Holochain's row is the warning about shipping the
first three without the fourth.

## 6. Sidequests surfaced (each needs its own pass before code)

1. **Name the embedded-Graphshell seam.** A swatch at the session boundary:
   does it need a term and a plane-level contract of its own, or is it
   strictly a client-library packaging question? (Cheap to answer; do it when
   a second host first wants a pane.)
2. **Adopt the anti-shell test as a G-series gate.** A "G-embed" receipt —
   the capability composed into Turnstone or signalman-desktop — belongs in
   the Graphshell plan queue the day embedding is scheduled, with the second
   host, not Graphshell, naming the done condition.
3. **The gradient as invariants.** Possession ⊇ disclosure ⊇ sync ⊇
   projection could become admission-boundary assertions (a score cannot
   select what the space does not carry; a space cannot carry what the grant
   does not admit). Worth a `mere_docs/testing/` pass when the planes next
   move.
4. **Per-persona continuity vs correlation.** The chain's "continuity without
   homogenization" (same persona across apps by intent, different personae
   where separation matters, no app learning unrelated identity material) is
   personae doctrine; what lacks a written home is the *cross-application
   correlation* threat model — which compositions let a host join two
   personae it was shown separately. Dramatis-tier research brief candidate.

## 7. Stack pillars: research before implementation (2026-09-08)

**Status:** bounded source survey and initial model, arena, Boa runtime/document,
Muniment/redb and Turnstone correlation experiments recorded for R1/R2.
Broader consumer and headed gates remain open in the ten-slice ledger below.
The survey observations remain code inspection; measured results are separated
at the end of §7. This is the shared research home; implementation belongs in
the repository that owns the selected seam.

A pillar is a durable guarantee with an accountable owner, consumers, and
failure tests. It need not become a new crate. The arena, shared wgpu device,
Mere, and Cambium describe different architectural levels: storage/lifetime,
device ownership, application composition, and UI composition. Elevating a
contract means making its responsibilities and evidence discoverable, rather
than flattening those levels into one kind of component.

### Candidate map

| Concern | Existing owner or seam | Research disposition |
|---|---|---|
| Document storage and reachability | Genet `ScriptedDom`, runtime reflectors and pins | R2 tests reference validity and retention; an arena is not a durable identity system. |
| Shared GPU device and resource lifetime | Embedder device/queue; Genet RenderCore; Netrender tenants; product-owned resources | Device ownership already has a contract. R1 investigates wake, cancellation, and disposal around it. |
| Source authority and projections | Mere source bindings, Forme, sceno/scenomise/scenotime; product authorities | Existing [projection research](2026-08-23_projection_scenes_and_graph_native_platform.md); R2 challenges identity across source and derived instances. |
| UI composition | Cambium, Meristem, Sprigging and Genet adapters | Existing [Cambium architecture](cambium_docs/technical_architecture/2026-09-03_cambium_architecture.md); extend its consumer proofs when a specific seam fails. |
| Geometry, text and presentation | Livery/Buckram/Parley; Netrender PaintList/Scene | Candidate contracts for geometry/selection agreement and renderer-neutral presentation, distinct from sharing a device. Scope a lane when a concrete cross-consumer disagreement is identified. |
| Execution and lifetime | Retained document sessions, product runtime profiles, renderer tenants | R1 below; first investigate host drive demand and cancellation. |
| Identity, provenance and custody | Product identities and revisions; document handles; Muniment; replication | R2 below; first investigate invalidation and retained provenance. |
| Capabilities and authority | Mere capability algebra, Servitor and Gemot admission | Existing owners; prospective end-to-end grant/revocation research, keeping observation separate from mutation permission. |
| Resource resolution | Genet resource fetch interfaces; Mere transport/protocol adapters; host policy | Prospective lane for consistent resolution/cancellation across script, document and host paths. Fetching bytes does not decide admission or durable custody. |
| Semantic observation and control | Genet engine observables; Cambium input/accessibility; host automation | Prospective lane for semantic readback and action routing through real hosts. Ortet O5 is the first platform-host boundary proof. |
| Configuration | Typed owner settings, providers, Cambium presentation, host application | Already governed by the [configuration ownership plan](archive_docs/2026-10-06_completed_plans/2026-08-06_configuration_ownership_settings_projection_plan.md); use its gaps rather than opening a duplicate lane. |

Spatial, inference, audio and networking components may be domain pillars
without becoming obligations of every application. R1 and R2 are the first
research lanes; the candidate map does not schedule all remaining rows.

### Research and implementation gates

Each lane records existing guarantees, the unresolved question, competing
designs including retaining local behavior, representative consumers, and
experiments that can discriminate between those designs. A small disposable
probe may be part of research. It does not promote its API into stack law.

**Research done:** the named probes have results, negative cases and resource
costs; the decision states the chosen owner, rejected alternatives, unresolved
limits and whether any common contract is warranted. A result that keeps
contracts local is a successful research outcome.

**Implementation ready:** a selected contract names state and authority,
identifier scope, lifetime/invalidation, inputs/outputs, cancellation and
failure behavior, a forcing consumer, and bounded owner-specific slices.
Reuse the [runtime composition ledger's](mere_docs/implementation_strategy/2026-08-23_runtime_composition_acceptance_plan.md)
distinction: settled reusable mechanics can move for one forcing consumer;
cross-product orchestration, identity, authority and lease contracts require
a heterogeneous second consumer before promotion.

**Implementation done:** the selected slice passes its named regression
manifest, including refusal and teardown cases, through the actual consumer
path. Receipts record repository revisions, dirty overlay or patch digests,
lockfile and runner digests, features, engine/renderer/backend, server/host
mode, commands and required assertions. Aggregate counts measure progress.
Do not transfer a runtime-only result into a headed-host claim.

For Genet, `genet-wpt` remains the automated conformance runner; Ortet O5
(`genet/design_docs/2026-09-03_ortet_founding_plan.md`) is the scoped headed
scripted platform target. Mere's Pelt proves downstream composition. Research
here neither duplicates O5 nor treats its planned receipts as already met.

### R1. Execution and lifetime

**Question:** can a retained session describe when it needs driving while
leaving time sources, scheduling policy and wake mechanisms with the host?
Can session replacement cancel work without admitting a stale completion?

Source survey on 2026-09-08: Mere `c33dfe67`, Genet `ee0b314b3e9`, Isometry
`243c0dd`, Mesocosm `6fa5ebb`, Netrender `c77b0be84`. Concurrent working-tree
changes, particularly Genet worker/runtime work, make this an inspection
baseline rather than a reproducible test receipt. Repin before experiments.

- Genet `components/shared/document-session-api/src/session_engine.rs` keeps
  sessions single-owner and host-driven. `pump(now_ms)` and `settled()` expose
  driving and quiescence, without a deadline or an external-wake contract.
  `genet/ports/ortet/src/shell.rs` currently redraws while unsettled.
- Genet `components/script-runtime-api/lib.rs` already exposes timer delay,
  worker work/pumping and worker shutdown. In the inspected
  `components/genet-scripted/document.rs`, the Livery pump advances timers,
  microtasks, capture and GC; it does not call `pump_workers`, and pending
  work is timer-based. This identifies an integration question for O5, not
  evidence that worker support is absent from the runtime.
- Genet `components/genet-render-host/src/lib.rs` and Netrender
  `netrender/src/renderer/init.rs` already place device creation with the
  embedder. Isometry `crates/isometry-runtime/src/render.rs` *(historical citation)* <!-- doc-audit: historical-path --> owns its tenant
  and stale-view refusal. Mesocosm
  `repos/isometry/mesocosm/crates/mesocosm-runtime/src/voxel_profile.rs` gates disposable derived state
  against durable source revisions. These challenge lifetime assumptions
  without requiring the same clock or a universal resource lease.

Compare retaining `settled()` with host-local polling against a narrow drive
report exposing immediate work, a host-time deadline, and outstanding external
work. These facts can coexist; a mutually exclusive enum may be insufficient.
Specify wake registration and completion races before choosing the API, and
distinguish work outstanding from work runnable now. Product simulation ticks
remain distinct from UI wall time and GPU completion.

| Probe | Required observations and negative cases | Decision it enables |
|---|---|---|
| R1-A: scripted Ortet wake and replacement | Boa and Nova timer, microtask and worker reply through a real retained session; idle wake and completion racing wake registration; navigate away before reply; exactly-once delivery to the live session, stale reply refusal and bounded teardown; record pump/redraw counts while idle | Whether drive facts and cancellation need a session API, and whether O5 reaches the runtime worker path |
| R1-B: visibility and suspension | Inventory controls actually exposed by the selected session; visible/hidden behavior, timer clamping, suspension/resume where supported, and worker completion during suspension; unsupported controls remain explicit gaps | Which behavior belongs to document semantics and which policy belongs to the host; whether drive demand preserves those boundaries |
| R1-C: derived resource disposal | Isometry tenant update/removal and Mesocosm stale-revision refusal; record allocation/release, owner and last usable revision, including rejected stale views | Whether a shared lifetime contract is needed; retaining separate resource types is an admissible result |

Prospective implementation slices, gated by those results:

1. **Genet session contract:** the selected minimal drive/cancellation seam in
   `document-session-api` and scripted adapters, only if R1-A/B require it.
   Reconcile and repin worker work first; keep host policy out of the trait.
2. **Genet Ortet integration:** adopt the selected wake policy and regression
   fixture in existing O5, proving timer/worker wake and session replacement
   in Boa and Nova. This depends on the selected contract or a documented
   decision to retain the existing trait.
3. **Product resource correction:** an Isometry or Mesocosm disposal fix and
   receipt only if R1-C finds a failure. Shared lease extraction needs the
   demonstrated common requirement; a comparison alone does not authorize it.

### R2. Identity and custody

**Question:** which references must survive detachment, navigation, capture,
deletion, restoration and delegation, and which must become invalid? Can
those transitions be expressed at their current ownership boundaries?

The survey uses the Mere/Genet baseline above and Turnstone `fa4cca57363f`.
The taxonomy below describes distinct meanings, not a proposal to introduce
new types or a global identity registry.

| Identity or reference | Scope and owner | Must not be mistaken for |
|---|---|---|
| Arena node handle | Genet `NodeId`, monotonic within one `ScriptedDom`; retention depends on roots and pins | A durable page id or a portable cross-arena reference |
| Document ownership | Genet DOM semantics; distinct from parent/tree connectivity | Connectivity: detachment does not itself change the owning document |
| Browsing-context generation | Genet context slot/generation | The durable page entity occupying that context |
| Capture target generation | Turnstone host; required by the capture plan, not yet implemented | The current page or a context slot alone |
| Durable page entity | Turnstone graph node UUID; restore preserves the entity id | Its URL, title, content hash or current appearance |
| Artifact, revision and lineage | Muniment hashes immutable bytes; Eidetic manifests identify typed records; journals retain their own log/fork lineage | One identity shared by all observations of equal bytes |
| Appearance | Host/document plus pane role; owns view-specific layout, scroll and focus | Source content authority or a durable capture attachment key |
| Capture request and observation | Host request id scoped to a surface; proposed durable envelope identifies the successful observation | A request succeeding merely because an id was allocated |
| Principal and custodian | Existing identity/admission owners; proposed domain retention/delegation record | Ownership acquired merely by storing or serving bytes |

Evidence homes are Genet `components/genet-scripted-dom/lib.rs`,
`components/genet-documents/src/browsing_context.rs` and
`components/shared/document-session-api/src/page_capture.rs`; Mere
`crates/eidetic/muniment/src/blob.rs`,
`crates/eidetic/eidetic-core/src/deleted.rs` and `schema.rs`; Turnstone
`src/action.rs` and
`repos/turnstone/design_docs/2026-09-05_reader_appearance_isolation_plan.md`.

Genet's current arena fence is enabled only on 64-bit debug builds. Release
and wasm need an explicit refusal/translation contract for foreign handles.
Ownership facts are split across Rust and JS `ownerDocuments`; wrappers,
ranges, queued observer records, secondary documents and detached subtrees
need separate root accounting. Inspection also identifies a possible pin
bypass through `set_text_content -> release_subtree -> drop_subtree` when
observers are disabled. This began as a source-review hypothesis; the pinned
Rust experiment below now reproduces it without involving a JS backend.
The existing G5 section in
`genet/docs/2026-06-11_gc_arena_dom_plan.md` owns this investigation and any
arena fix; this lane adds the cross-stack comparison.

Compare an opaque document-scoped boundary handle (arena instance plus local
node, possibly encoded through a side table) with a session-owned,
generation-checked handle table. Measure lookup/storage cost and refusal
behavior on native release and wasm before selecting either. Internal IDs
may remain compact. Adoption needs an explicit ownership transition that
preserves required DOM identity and references; it cannot simply reinterpret
a raw id in another arena. A globally shared arena is not required by either
design.

Turnstone's `repos/turnstone/design_docs/2026-08-28_page_capture_plan.md` already scopes P2/P3:
freeze the session, durable node, document/navigation generation and
surface/request correlation at capture start; attach to that exact target or
refuse a stale completion. Engine results supply observed facts. The domain
envelope supplies attachment, acquisition context and retention references.
Muniment's byte deduplication does not merge two observations. The existing
generic `ProvenanceRecord` does not yet express all capture acquisition facts,
and tombstone capture references/reaping remain work in
`turnstone/design_docs/2026-09-06_page_lifecycle_plan.md`.

**Custody clarification:** composing a view does not surrender or merge the
originating application's authoritative store. Storage and retention can be
explicitly delegated: another custodian may hold, replicate or serve bytes
under stated terms while semantic authority remains with the domain owner.
An explicit delegation contract should record retention and disclosure obligations, provenance and
revocation behavior; revocation does not by itself prove remote erasure.
This is the sense in which §1's former "never custody" wording is narrowed.

| Probe | Required observations and negative cases | Decision it enables |
|---|---|---|
| R2-A: arena identity and roots | Boa/Nova retain, detach, text/fragment replacement, collect, adopt, mutate and release; observers off/on, independent wrapper/range/observer roots; release/wasm foreign-handle refusal; retained readback survives, ownership changes only on adoption, stale ids do not alias, and released nodes return to a bounded baseline | Select a G5 boundary representation and complete root inventory; use Ortet O5 for headed proof separately |
| R2-B: capture correlation | Start on page A/session S/generation G/surface P, then navigate, close or switch before completion; duplicate completion and equal-byte captures; attach once to the exact target or fail typed, and never to the replacement; distinct successful observations may share one blob | Select the domain envelope and adapter refusal contract without extending engine authority |
| R2-C: custody and recovery | Model one artifact referenced by a live page, a tombstone and a download facet; retire references in varied orders using MemoryBackend or temporary redb; preserve bytes until the last reference, model restoration of the same UUID and proposed envelope, recheck references before deletion, and keep local custody during redacted export | Determine the narrow retention/reference contract and versioning needed by existing lifecycle work |

R2-B compares a hosted Weld adapter with a retained Livery/unsupported route.
A model or fake can establish correlation/refusal during research; it cannot
prove pixels were captured or that the production adapter is integrated.
R2-C compares Turnstone recovery with export/place-share policy consumers.
These are distinct uses within one product, not the heterogeneous second
product required to promote a universal cross-product custody contract.

Prospective implementation slices, after the probes choose the contracts:

1. **Genet G5:** implement the chosen handle/ownership boundary and retention
   fixes in scripted DOM/runtime, reconciling concurrent work first. Keep
   native/wasm refusal and Boa/Nova root manifests explicit; Ortet owns the
   headed consumer receipt.
2. **Turnstone capture P2/P3:** add the deposit-only capture path, versioned
   observation envelope and stale-completion refusal through existing
   session/action/effect ownership. Genet reports capture facts and Muniment
   holds bytes. Depends on R2-B and the capture plan's existing compile gates.
3. **Existing lifecycle owners:** extend Eidetic typed tombstones and the
   product-supplied reference set for restoration, redaction and safe reaping.
   Select schema migration/refusal behavior before storing new records; route
   policy through existing Pandect/Athanor seams where applicable. Depends on
   the capture envelope and R2-C. Broader custody extraction remains gated by
   another product's actual requirement.

### R3. Resource resolution (opened 2026-09-20)

**Question:** when a first-party application needs remote bytes by range (a
podcast episode while it plays) or a held object by offset (the same episode
after download), can it get them through the host's one fetch policy and the
stack's content store, and what does a small standalone host pay for that?

The forcing consumer is Redshank in Turnstone. `redshank-playback` reads HTTP
ranges through its own ureq client, outside Turnstone's cookies, HSTS, cache,
redirect limit and transport seam, which
`turnstone/design_docs/2026-09-14_redshank_episode_surface_plan.md` records as
its open gate 1. A side client would also bypass the I2P and Arti lanes the
[reachability plan](mere_docs/implementation_strategy/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md)
puts in Turnstone's fetch stack. Existing guarantees: netfetcher sends `Range`
with identity encoding and streams bodies; the netfetcher plan (archived
2026-06-09) puts a fetcher pool in each host's session layer over persona-scoped
stores, and no such worker exists yet; the
[session store plan](mere_docs/implementation_strategy/2026-06-23_native_session_store_plan.md)
rules one substrate per persona; the
[resident plan](mere_docs/implementation_strategy/2026-08-20_device_resident_consolidation_plan.md)
makes iroh's BLAKE3 blob store the shared content store.

Rulings (Mark, 2026-09-20):

- Redshank's persona-held state (notes and sharing, synced library, voice
  bodies, offline episodes) goes through the device resident as a client, the
  way Knot's persona-vault mode does. Local-only listening stays embedded
  (resident plan invariant 9).
- Each host process runs its own HTTP fetcher pool over the one per-persona
  session store. The resident's network runtimes stay peer networking.
- Media ownership is split: Genet owns the player contract; Mere's net-media
  owns decoders and backends, with Redshank's Symphonia and Firewheel backend as
  its first increment.
- Held episodes use iroh's content store. A chunked Muniment blob format,
  proposed earlier the same week, is withdrawn.
- Where a ranged fetch contract lives, and whether standalone Redshank keeps
  its simple HTTP client, are decided after the probes. Keeping the standalone
  client local is an admissible result.

| Probe | Required observations and negative cases | Decision it enables |
|---|---|---|
| R3-A: ranged read on a persona-scoped context | Through real netfetcher from a blocking decoder-style thread: 206 with exact `Content-Range`, identity encoding, `If-Range` change detection, `Range` kept across a redirect with the final URL reported, a cookie from an earlier page fetch carried and another persona's cookie absent; negative: a ranged request in default cache mode after the whole object was cached, against the same request as no-store; a body observed streaming before the server finishes | What a ranged caller needs from the host, and which guards the host must apply |
| R3-B: held and partial episodes in iroh's store | Real iroh-blobs store: seek-and-read over a complete blob from a blocking thread; whether a streamed import is readable before it completes; a partial blob serves present ranges and refuses absent ones | Whether a held episode and a downloading one can share one reader, and where progressive HTTP download must live instead |
| R3-C: standalone cost | The same ranged read built on ureq and on netfetcher: packages Redshank's lock would gain and lose, build time, binary size, loopback latency; the policy the simple client skips, by construction | Whether the standalone host keeps its simple client |

Research done and implementation ready follow the gates above. No production
code changes in this lane.

#### R3 results (2026-09-20)

[Receipt](mere_docs/testing/receipts/2026-09-20_resource_resolution_probes/2026-09-20_resource_resolution_receipt.md),
with sources, locks and logs. Real netfetcher (genet `5ae30cad`), real
iroh-blobs 0.103, real ureq 3.4; loopback HTTP servers only, so no TLS, no WAN,
and no Redshank, Turnstone or resident code in the loop.

| Probe | Observed | Consequence |
|---|---|---|
| R3-A, 6 of 6 | A blocking thread over a shared runtime reads exact ranges with identity encoding at Redshank's own 6.6% figure; `If-Range` change, redirect with final URL, per-persona cookies and body streaming all hold. **Pinned negative:** once the whole object is cached, a default-mode ranged request never reaches the network and returns 200 with all 3,000,000 bytes. | netfetcher already carries everything a ranged caller needs. The host must send ranged requests as no-store until netfetcher's cache can answer a range from a stored body. HTTP permits the current behaviour, so this is a cost and a trap, not a conformance defect. |
| R3-B, 3 of 3 | A streamed import has no hash and nothing addressable until its last byte. A complete blob survives reopen and serves blocking seek-and-read at 2.8 ms per 64 KiB in release. **Pinned negative:** the reader refuses `SeekFrom::End`. With the hash known up front, a partial blob reads present ranges and errors on absent ones. | A held episode plays from iroh's store through a small bridge. A progressive HTTP download cannot live there while it downloads, because a feed gives no BLAKE3. Streaming stays HTTP ranges, held playback is the store, and the episode enters the store when the download completes. One reader for both means one decoder-facing `Read + Seek` with two sources behind it, not one store. |
| R3-C | Redshank's reader calls ureq's free function, which uses a use-once agent: 301 connections for 301 requests and 1.5 ms per read on loopback, a TLS handshake per 64 KiB chunk over `https`. One shared ureq agent: 0.10 ms, 1 connection. netfetcher: 0.13 ms, 1 connection. netfetcher with only its hyper transport adds 24 crates to Redshank's lock and drops 6, 47 s cold build against 15 s, 5.3 MB against 2.2 MB; default features add 47. | The simple client costs the standalone host nothing in speed once it shares an agent. The stack fetch is affordable for it, and buys cookies, HSTS, cache, redirect limit and transport seam that a standalone podcast player mostly does not exercise. |

Rulings on the results (Mark, 2026-09-20). The lane's recommendation is kept
beside each where he ruled otherwise, so the record shows both:

1. **Fix Redshank's per-chunk connection now.** One shared agent in
   `redshank-playback`'s range reader. Independent of everything below.
2. **Standalone Redshank moves onto netfetcher.** One HTTP stack across both
   hosts, and the standalone host gains the privacy lanes when they land. The
   measured cost is accepted: with only the hyper transport, 24 crates gained
   and 6 dropped, a 47 s cold build against 15 s, 5.3 MB against 2.2 MB. (The
   lane recommended keeping the simple client, as Redshank's own plan had it.)
3. **The ranged contract is a trait in Mere now,** with Redshank as its first
   consumer, ahead of a second one. (The lane recommended a supplied function
   until a second consumer appeared.) What R3-A fixes about its shape: one
   blocking call, a range in, and bytes, total length, validator, final URL and
   content type out; ranged requests go no-store.
4. **Turnstone's one persona-scoped context remains the larger piece** and is
   Mere and Turnstone work, not Redshank's: the fetch actor and document-lanes'
   `RemoteFetcher` sharing one context over the persona's jar and cache, and
   cookie persistence getting a caller again.
5. **Held episodes through the resident** wait for Redshank's resident client
   work; R3-B says the store side is ready and names the bridge's two
   obligations (translate seek-from-end, import on completion).

Rulings 2 to 4 are implementation and leave this research home for an
owner-specific plan in Mere.

### First experiment results (2026-09-08)

The [arena receipt](mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/arena/R2_A_RECEIPT.md),
[execution receipt](mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/execution/R1_EXECUTION_RECEIPT.md)
and [capture/custody receipt](mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/identity/R2_B_C_RECEIPT.md)
preserve the disposable scripts, commands, source census and limitations.
Both models were independently rerun after review. The arena fixture calls
real `ScriptedDom`/`Pins` from Genet `ee0b314b3e9`; the other fixtures model
proposed contracts. None is a threaded-runtime, storage-backend or headed receipt.

| Experiment | Observed result | Contract consequence |
|---|---|---|
| R2-A Rust arena baseline | Native x64 debug: 5 assertions pass, 2 fail. Text and fragment replacement with observers off delete a pinned child before collection; ordinary detachment and observers-on cases pass. | The retention failure is reproduced in the engine-owned store. G5 needs semantic mutation to preserve retained nodes until pin-aware collection can decide reclamation. |
| R2-A fence-disabled Rust arena | Dev profile with debug assertions disabled only for `genet-scripted-dom`: 4 pass, 3 fail. The added failure resolves a foreign `NodeId(2)` to a local `NodeId(2)`. | Cross-arena refusal cannot depend on debug assertions. This exercises the fence-disabled configuration, not full optimized release or wasm. |
| R2-A disposable retention correction | Same seven assertions, baseline debug configuration and lock: 7 pass after the scratch patch removes immediate deletion from the two replacement paths. | Supports a bounded G5 correction that leaves reclamation to pin-aware collection. The patch is preserved as an experiment; production source, backend-root coverage and the foreign-handle gap are unchanged. |
| R1 drive model | Pass: immediate work, deadline and external work coexist; query/register/recheck detects modeled completion races; stale and duplicate completions refuse. Broken one-bit/callback-only controls lose information or a wake. | Investigate an additive drive report and a registration protocol that cannot lose a wake. The model selects necessary information, not an implemented API or a concurrency proof. |
| R2-B capture model | Pass within the 14-test combined suite: exact frozen-target correlation, stale/duplicate refusal, distinct observations sharing one blob, and refusal to reuse a request identity after success or failure. Broken controls attach to the current page or collapse stored observations. | Request identity must remain unique for the surface-instance lifetime, including terminal failures; define exhaustion/restart epochs. Observation identity remains separate from artifact hash. |
| R2-C custody model | Pass within the same suite: all six retirement orders, two live owners of one artifact, recovery identity, transfer interleavings, collection recheck and redacted export. Broken controls expose stale-proposal deletion and a false orphan during remove-first transfer. | Track `(reference class, owner identity, artifact hash)`, not one bit per class. Add the destination reference before retiring the source; give transfer and collection one serialized or transactional boundary. Apply must recheck current references. |

The review itself found two useful weaknesses in the initial models: completed
request IDs could be reused, and a class-level set of hashes collapsed two
live owners. The retained scripts include the strengthened rules and negative
controls. SHA-256 in the Python model stands in for equality only; Muniment's
production artifact addressing remains BLAKE3.

The real `script-runtime-api` worker-suite attempt stopped at Cargo's shared
package-cache lock before compilation/test output. R1-A/B's retained-session
and Ortet paths, R1-C's actual resource disposal comparison, release/wasm
handle representation costs, and real capture/storage integration remain open.
The full research-done gate has not passed, and none of these model results
closes O5, G5, capture P2/P3 or page-lifecycle implementation acceptance.

### Bounded continuation, approximately ten slices (2026-09-08)

The user authorized implementation and further experiments in this direction,
with room for roughly ten slices and evidence-driven sidequests. This is a
working sequence, not a quota. A proven local defect can be corrected while
broader research continues; completing every R1/R2 experiment is not a
prerequisite for S1. Shared-contract promotion still requires its own evidence.
Existing owner plans remain the implementation homes.

| Slice | Work and natural owner | Done-condition | Current state |
|---|---|---|---|
| S1 | Genet G5 replacement retention | Production text/fragment replacement preserves retained descendants, readback and reattachment; observer records remain correct; unpinning and repeated replacement reclaim nodes; focused and existing arena tests pass on an exact overlay | Committed `ec5421b7591`: 6 production, 37 existing source and 7 research tests pass; `genet/design_docs/receipts/2026-09-08_g5_s1/receipt.md` |
| S2 | Genet G5 root closure | Exercise retained wrappers, descendants, queued observer/range roots and shadow/template edges; distinguish store roots from backend roots, fix reproduced local defects, and record residual owner-document/adoption requirements | Store correction committed `c52ee06f53a`: directed template-owner roots and stale metadata retirement, 58 tests pass; narrow Boa retained-wrapper/readback/release probe also passes; other backend/root cases open |
| S3 | Genet G5 handle representation | Compare viable all-target boundary identities against real raw-reflector/NodeId consumers; record rejection, exhaustion, memory/lookup cost and native/wasm feasibility; choose a bounded representation | Repinned research: 3 native debug tests and release model pass; wasm32 compiles only. 24/40 u64 remains a candidate; production raw handles still alias |
| S4 | Genet G5 handle implementation | Implement the selected boundary, with foreign/stale refusal and exhaustion tests through the supported backends and compiled target configurations; retain a separate wasm residual if it is not executed | Open: discard unused parallel-handle sketch; migrate approximately 70 runtime conversions coherently with semantic NodeId, high-bit engine and wasm execution gates |
| S5 | Genet runtime worker delivery | Run actual runtime message/timer/completion and shutdown assertions, including duplicate/stale cases, with a named regression manifest and isolated build provenance | Real Boa Runtime probe passes with pinned overlay: timer-only negative, worker-pump delivery and acknowledged idle; Genet S2/S5 backend receipt checked in; shutdown, stale and duplicate runtime cases remain open |
| S6 | Genet session drive contract | Expose the selected minimal runnable/deadline/external facts and lossless wake/cancellation protocol through a real retained session; test completion during registration and replacement | S6a committed `790ba660396`: worker forwarding and retained loader implemented; 2 actual Boa document tests pass (reply/reaction, freeze/resume, missing-route error and idle), preforward negative fails; automatic session-loader wiring, Livery/Nova and additive wake/drive gates remain open |
| S7 | Genet Ortet O5 integration | Wire native host wake and replacement to the real scripted session and obtain explicit Boa/Nova host receipts; automated conformance remains with genet-wpt | Native O5 hosting for Boa and Nova accepted 2026-09-08 (`genet/design_docs/2026-09-03_ortet_founding_plan.md`). G5 headed arena sequence accepted on both engines 2026-09-09 on branch `slice/s7-ortet-g5-20260909` (`genet/design_docs/receipts/2026-09-09_g5_ortet/receipt.md` *(historical citation)* <!-- doc-audit: historical-path -->); found and fixed Boa `force_gc` never calling `clear_kept_objects`; residual: page-level `WeakRef` over a DOM reflector never reports collection under Boa, folded into the realms lane. Merge into genet main waits on the realms continuation committing. AccessKit bridge-action receipt still open |
| S8 | Host visibility and resource disposal | Exercise real hidden/suspended/resumed session behavior and product-owned resource removal/stale-revision refusal; record policy ownership and any required local correction | Cambium native invalidation committed `fe2e24e9`; independent source reviews and diff check pass, actual GPU suspend/resume remains open |
| S9 | Turnstone capture correlation | Integrate exact target and nonreused request identity into the actual capture path, with stale/duplicate/unsupported refusal; keep envelope and pixel-storage acceptance explicit | Committed `c5f991e`: action-to-host correlation, replacement-id alias and bulk teardown corrections; 5 production-module tests plus external Inker sanity pass, no-default library check passes. Weld adapter compile is blocked before Turnstone by pinned Welding/current local Grafting API mismatch; headed pixels and durable deposit remain open. 2026-09-09: the Weld blocker was Turnstone drift, not the pin; the adapter had been uncompiled since the P1 contract landed. Migrated to welding/grafting heads and merged to turnstone main; hosted capture returns `Unsupported` until Inker's `WeldSurface` grows a capture hook (P2), the S9 Weld-side wiring was deleted and will be reimplemented then. Mark ruled Chromium's process sandbox the default: turnstone `436e087` adds the Windows bootstrap client-DLL route (`sandbox_bootstrap_win` crate, `RunWinMain`), `Sandboxed` by default when launched through CEF's bootstrap and `UnsandboxedTrustedContent` on the direct executable route, with `TURNSTONE_WELD_SANDBOX` as an explicit override that refuses unsupported combinations; a headed sandboxed boot succeeded on a local CEF 151.3.24 bundle (`turnstone/design_docs/2026-08-03_turnstone_engine_adoption_plan.md`) |
| S10 | Turnstone/Mere custody lifecycle | Integrate owner-specific references and a serialized/transactional transfer/collection boundary with recovery and disclosure policy, including same-kind multiple owners and stale collection proposals | Actual Muniment/redb probe: 3 pass, including reopen and stale-delete negative control; production writer/envelope integration remains open. 2026-09-09 ruling: a transactional read-then-write method on Muniment's `Backend` trait (redb and IndexedDB honor it, zip refuses); pricing in the orchestration scratch report, implementation lane launched 2026-09-09: `Backend::transact` landed on `slice/s10-custody-20260909` (`9ba9f790`) — honest for redb (one write transaction), memory (single mutex), and IndexedDB (one Readwrite transaction, wasm32 offline check-compiles); zip and the trait default both refuse with a typed `StoreError::NotTransactional`. The probe's scenarios pass as positive controls against the real backends (59 tests total); the apply-alone negative control is retained as a documented regression. Verified, not just read from source: redb's Windows `LockFile` is cross-process, held for the `Database` handle's whole lifetime (`design_docs/mere_docs/testing/receipts/2026-09-09_s10_custody_transact_receipt.md`). Turnstone wiring and process-kill recovery remain open |

The [actual custody-backend probe](mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/2026-09-08_custody_backend_receipt.md)
uses Muniment/redb and BLAKE3: owner-specific transfer survives reopen, a
rechecked collection refuses a later claim, and an atomic but stale delete
batch demonstrably loses the newly claimed bytes. This selects a requirement
for S10: all claims, transfers and collection share one serialized writer, or
the store provides conditional transactional reads and writes. `Backend::apply`
alone does not cover the read-to-delete interval. Process-kill recovery and
actual capture envelope/deposit integration remain open.
### Lifetime and ownership guarantees, elevated 2026-09-09

These ten slices exercise seven lifetime/ownership guarantees, each with an
accountable owner by path and at least one production or real-backend failure
test. None is a new crate: elevation means each keeps its existing owner,
gains a named regression manifest where one does not yet exist, and records
its open negative controls next to its passing ones. Two receipts already show
a currently failing control in production or real-backend code, the
release-profile foreign-handle alias (arena receipt) and the stale delete after
an atomic apply (custody-backend receipt); those stay open defects, not closed
research, until a fix lands with its own test. Guarantee 6 sits outside the
R1/R2 taxonomy above: it is a UI-composition-level lifetime guarantee, not
identity or execution, and is listed here so it is not lost between them.
Paths are repo-qualified because owners span genet, turnstone and mere.

#### 1. Arena node retention across replacement

**Guarantee:** a pinned node survives text/fragment replacement (with
observers on or off) until pin-aware collection decides reclamation; ordinary
detachment and 1,000-cycle churn return to baseline without aliasing.

**Owner:** Genet `components/genet-scripted-dom` (production fix landed;
`genet/docs/2026-06-11_gc_arena_dom_plan.md` G5 owns the investigation).

**Consumers:** `components/genet-documents/src/browsing_context.rs`;
`components/shared/document-session-api/src/page_capture.rs`; Genet's
scripted-DOM JS wrappers generally.

**Failure tests (exist):**
`genet/components/genet-scripted-dom/tests/replacement_retention.rs` —
`text_replacement_retains_descendants_without_observers`,
`text_replacement_retains_descendants_with_observers`,
`fragment_replacement_retains_descendants_without_observers`,
`fragment_replacement_retains_descendants_with_observers`,
`replacement_churn_reclaims_without_observers`,
`replacement_churn_reclaims_after_observer_delivery` (6 tests, S1 receipt).
`genet/components/genet-scripted-dom/tests/root_closure.rs` — 7 tests
including `live_template_retains_its_contents_owner`,
`pinned_contents_descendant_retains_owner_without_retaining_template`,
`shared_owner_does_not_retain_a_retired_sibling_fragment` (S2 receipt).

**Missing negative controls (proposed names):**
`foreign_handle_refuses_same_local_slot_on_release_profile` (the R2-A
fence-disabled run found this *fails* today: a foreign `NodeId` resolves to a
live local slot once debug assertions are off — this is an open defect, not
yet a passing control); `wasm32_release_replacement_retention` (untested
target).

**Residual:** release/wasm foreign-handle aliasing is a known-open defect
(R2-A fence-disabled result), not merely unproven; JS wrapper root inventory
beyond templates/shadow hosts; owning-document/adoption; headed Ortet O5.

#### 2. Node handle identity across arenas and targets

**Guarantee:** a node handle must not silently resolve to a different arena's
node of the same raw id; the boundary representation must refuse foreign or
stale handles on every supported target, not only native debug.

**Owner:** Genet `components/genet-scripted-dom` (`NodeId`/`ReflectorData`),
G5 S3/S4 in `genet/docs/2026-06-11_gc_arena_dom_plan.md`.

**Consumers:** ~70 runtime conversion sites narrowing `ReflectorData` via
`as usize` (S3 receipt); Boa/Nova reflector caches
(`genet/design_docs/2026-09-08_realms_plan.md`, "Corrected findings": *"Boa's
old context-wide reflector cache could alias equal raw node IDs from
different arenas in release builds. Debug arena tags concealed that case."* —
now scoped per-realm in that plan's continuation).

**Failure tests (exist, model-level only):** the S3 handle-boundary runner
(`genet/design_docs/receipts/2026-09-08_g5_s3_handle_boundary/receipt.md`) —
3/3 native-debug tests against real `genet-scripted-dom` nodes; a native
release run *reproducing* two arenas aliasing raw root `0`, which the modeled
24/40 packed handle keeps distinct. This is receipt evidence of the defect,
not a merged production test.

**Missing negative controls (proposed names):**
`packed_handle_refuses_foreign_arena_on_release_build` (production, not the
runner-only model); `packed_handle_overflow_refuses_rather_than_truncates`
(today `remint_node_id` panics on `usize` overflow instead of refusing);
`wasm32_runtime_boundary_execution` (S3 only compiled wasm32, did not
execute it).

**Residual:** no production packed-handle implementation exists yet (S4 is
explicitly open); consumer migration of ~70 sites undone; cross-`ScriptedDom`
adoption is "explicitly unsupported" per the S3 receipt; realm-scoping fix for
the Boa cache is itself mid-continuation, not yet gated (per the realms plan's
"Current gates": compilation/acceptance tests in progress, Ortet receipts
pending).

#### 3. Runtime worker wake, idle acknowledgement, and teardown

**Guarantee:** a document session forwards worker service after timers and a
microtask checkpoint, delivers replies exactly once to the live session, and
reports non-settled/pending status until the worker's idle acknowledgement —
not merely until a timer fires.

**Owner:** Genet `components/shared/document-session-api` (session
API/contract) and `components/script-runtime-api` (worker mechanism);
`genet/design_docs/2026-09-07_worker_plan.md` (dedicated-worker
implementation, landed `6155c8ab4ff`).

**Consumers:** `genet/ports/ortet/src/shell.rs` (drives `pump`/`settled` before
redraw); `components/genet-scripted/document.rs` scripted-document routes.

**Failure tests (exist):**
`genet/components/genet-scripted/tests/worker_service.rs` —
`document_pump_forwards_worker_service_and_waits_for_acknowledged_idle`,
`missing_worker_route_reports_error_and_eventually_quiesces` (S6a receipt, 2
real `ScriptedDocument<BoaEngine>` tests). S5 backend receipt: one passing
Boa-backed runtime-worker-wake test (`s2_s5_backend` receipt) proving
`pump_workers()`/`has_worker_work()` idle semantics against a real
`Runtime<BoaEngine>`.

**Missing negative controls (proposed names):**
`worker_shutdown_during_pending_reply_does_not_deliver_stale_completion`;
`duplicate_worker_completion_is_refused`; `nova_worker_wake_and_idle` (S5/S6
only exercise Boa); `preforward_regression` exists as a named negative-control
manifest already (`s6-preforward.patch` in the S6 receipt) and should be kept
as a standing test, not just a receipt artifact.

**Residual:** stale/duplicate runtime completion cases are explicitly open
(bounded-continuation table, S5 row); Nova path untested; a ready/deadline/
external drive report (R1's proposed contract) is not implemented — the R1
execution receipt only validates a disposable PowerShell model, and the real
`script-runtime-api` worker-suite attempt stopped on a Cargo package-cache
lock before producing output; headed Ortet O5 wake/replacement remains S7,
gated on S6.

#### 4. Session generation and stale-completion rejection

**Guarantee:** navigation, node closure, and session switch must retain the
exact captured target (session, durable node, document generation, surface)
or refuse a late completion; a completion cannot attach to a replacement
target merely because a request id was allocated.

**Owner:** Turnstone `src/shell/page_capture.rs` (this specific correlation
module — see item 5, since it is the same code); the general session-
generation concept also belongs to Genet's browsing-context slot/generation
(`components/genet-documents/src/browsing_context.rs`, per the brief's R2
identity taxonomy).

**Consumers:** Turnstone's capture request path; anything reading Turnstone's
`(session, node, generation, surface)` frozen target.

**Failure tests (exist, production):**
`turnstone/src/shell/page_capture.rs::tests` —
`exact_completion_is_single_use`,
`navigation_and_surface_replacement_refuse_old_completion`,
`session_switch_refuses_completion`,
`generation_exhaustion_rotates_surface_and_retires_pending`,
`admission_failure_spends_id_and_allocator_exhaustion_refuses` (S9 receipt,
5/5 passing against the real production module, hash-verified).

**Missing negative controls (proposed names):**
`process_kill_recovery_does_not_replay_stale_generation` (not modeled
anywhere yet); `weld_adapter_stale_completion_through_hosted_surface` (the
Weld/CEF adapter does not currently compile — see item 5's residual).

**Residual:** this is production-tested logic only at the correlation-module
level; it has not been exercised through an actual hosted surface (Weld
adapter compile-blocked) or headed pixels.

#### 5. Capture request identity single-use

**Guarantee:** a `(surface instance, request id)` pair is single-use through
terminal success or refusal; re-registration and duplicate/delayed completion
are refused; equal bytes may share one artifact entry while distinct
observations keep separate identities.

**Owner:** Turnstone `src/shell/page_capture.rs`, per
`turnstone/design_docs/2026-08-28_page_capture_plan.md` P2/P3 done-conditions
("single-flight," "typed busy," exact-target refusal).

**Consumers:** Genet capture-facts producer
(`components/shared/document-session-api/src/page_capture.rs`); Mere
Muniment as the byte store (`crates/eidetic/muniment/src/blob.rs`).

**Failure tests (exist):** same S9 production tests as item 4 (single module).
Model-level evidence for the broader envelope/observation-identity contract:
the R2-B/C `identity_custody_probe.py` (14 tests, 0 failures) — model only,
not production, per the identity receipt.

**Missing negative controls (proposed names):**
`weld_single_flight_refuses_second_request_while_outstanding` (P2
done-condition, no adapter compiles yet — S9 receipt: Weld check fails on a
Welding/Grafting API field mismatch, "before Turnstone compiled"); `deposit_
off_event_loop_thread_matches_download_custody_actor` (P2 done-condition, not
yet built).

**Residual:** P2 (hosted capture through Weld into custody) and P3 (the
provenance envelope) are both open per the plan; the Weld adapter does not
currently compile against pinned Welding/Grafting revisions (S9 receipt);
headed pixels and durable deposit are explicitly out of scope for S9.

#### 6. Renderer resource retirement on suspension

**Guarantee:** renderer-owned fragment/resource identities are retired when
the host suspends (surface lost) rather than reused against a new renderer
instance on resume.

**Owner:** Mere `crates/cambium/cambium-genet-winit-host` (Cambium's genet/
winit host adapter); Cambium architecture doc
(`repos/mere/design_docs/cambium_docs/technical_architecture/2026-09-03_cambium_architecture.md`).

**Consumers:** Cambium's application leaf registry (survives suspension by
design — only fragment IDs are cleared, per the commit).

**Failure tests:** no named regression test found in the diff itself; commit
`fe2e24e9` (`git -C repos/mere show --stat fe2e24e9`) is a 3-file, 20-line
change touching `crates/cambium/cambium-genet-winit-host/src/lib.rs` (the
`.clear()` on `self.s.leaf_fragments` in `suspended()`), the family-composition
brief, and the Cambium architecture doc. The bounded-continuation table (S8)
says: *"Cambium native invalidation committed `fe2e24e9`; independent source
reviews and diff check pass, actual GPU suspend/resume remains open."* — i.e.
verification so far is source review, not a running test.

**Missing negative controls (proposed names):**
`suspend_resume_does_not_reuse_stale_fragment_id`;
`suspend_before_first_frame_does_not_panic`; both currently unwritten.

**Residual:** actual GPU suspend/resume exercise is explicitly open (S8 row);
this is the one guarantee in this set with no executable test at all today,
only a reviewed diff.

#### 7. Custody claim/transfer/collection serialization

**Guarantee:** artifact ownership (`reference class, owner identity, artifact
hash`) must transfer with the destination reference added before the source
is removed, and collection must recheck current references at apply time, not
at proposal time — no serialized writer, no safe delete.

**Owner:** Mere `crates/eidetic/muniment` (blob/reference store) plus
`crates/eidetic/eidetic-core/src/deleted.rs` and `schema.rs` (tombstones);
Turnstone's lifecycle work
(`turnstone/design_docs/2026-09-06_page_lifecycle_plan.md`) as the product
consumer.

**Consumers:** Turnstone's live-page/tombstone/download reference classes.

**Failure tests (exist, against real Muniment/redb):** the custody-backend
receipt (`mere/design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/2026-09-08_custody_backend_receipt.md`,
`src/lib.rs` regression manifest) — 3 assertions pass: equal-bytes-one-blob
with separate same-class owner references surviving reopen; a collection
proposal refused when the adapter rechecks references at apply; and a
**negative control that fails today**: applying a precomputed delete batch
after a new claim still deletes the blob and leaves a dangling reference
("Atomic write batches do not make an earlier read current").

**Missing negative controls (proposed names):**
`process_kill_during_transfer_recovers_consistent_state`;
`competing_writer_bypassing_the_serialized_custody_actor_still_races`
(receipt: *"another writer bypassing it can still race its read/delete"* —
i.e. this is a known-open gap, not yet a control).

**Residual:** the receipt's own conclusion is that `Backend::apply` alone is
insufficient — production needs one serialized custody writer or a
transactional conditional read/write API, neither of which exists; process-
kill recovery, disclosure policy, and envelope persistence are all untested.

---

Sidequests require a concrete finding that affects one of those done-conditions,
a named owner and a bounded receipt. A root-accounting defect or a missing host
wake is sufficient; a desire to found another platform crate is not. Reconcile
concurrent edits before promoting a slice, and record research-only, compiled,
runtime and headed outcomes separately. The scope permits progression through
the sequence without a new approval for each routine slice.

## 8. Primary data model, clips and live entities (2026-10-11)

### Accepted direction

Mark, after the document-graph discussion and its critical review:

> yeah, primary data model, not one universal representation. clips are the
> portable content units, but live entities belong too.

The graph is the primary model through which Mere applications identify,
relate, retain, query, share and act on their data. A content or document graph
is a crucial dimension of that model, alongside live entities, activities and
arrangements. Graph structure carries meaningful relationships; it does not
require every byte, document element, event or simulation value to become a
separate graph node.

The unit remains **one mere per data domain**, as ruled on 2026-09-23. Several
applications can work in one mere; one application can work across meres.
An identity's meres form its reservoir. An application adapter exposes the
domain's identities, relationships, disclosed state and named operations
without replacing its authoritative document, media, game or device model.
Content principally authored and manipulated in graph scope may have native
graph structure. Foreign content keeps the structure its own renderer or
domain requires. These are compatible uses of the primary model.

This section is the canonical home for that direction. The
[scene record](2026-08-23_projection_scenes_and_graph_native_platform.md#916-primary-model-portable-clips-and-live-entities-2026-10-11),
[statement stance](mere_docs/technical_architecture/2026-05-22_statements_over_schema_stance.md)
and [composition spine](mere_docs/technical_architecture/2026-05-21_mere_composition_spine.md)
apply it at their respective seams.

### Browsing encounters (2026-10-11)

Mark agreed to preserve visits while reviewing Eidetic's foundations. Resource
identity, an individual encounter and a captured representation are distinct.
Revisiting one resource retains another encounter; two branches through the same
resource do not collapse into one trail. Preserve source-scoped visit identity,
parent references and known context, including an unresolved parent outside an
import range. Followed navigation, links contained in captured content, clipping
provenance and computed relationships retain their different evidence meanings.

The [Eidetic development plan](eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md)
owns the proposed intake and admission work with Chartulary/Stemma, Pandect,
Inker, Fleece and ESP. It treats dependency capabilities as part of development.
This direction does not claim that current browser intake preserves those facts
end to end; concrete wire, migration and retention decisions remain in the plan.

### Clips, live entities and appearances

A **clip** is content made into a portable unit by meaningful selection,
capture or export. The boundary is what a person can usefully take and share:
a selected passage, whole document, screenshot, recording, or supported
structured-data export. It need not be the smallest grammatical or semantic
part. A gameplay clip can contain video and a meaningful description; a game
can also export structured state when its domain defines a useful portable
form. Capturing gameplay does not require atomizing the game's simulation.

A clip can be compound and internally structured. Text, images, time ranges,
tables, semantic descriptions and source anchors can describe one clip without
each becoming an independent graph entity. Creating a separately shareable
subclip is an explicit operation, with its own identity and lineage. A
selector into a clip is not automatically another node.

A **live entity** has domain identity and operations before, and independently
of, any capture: an authored object, field, device, active session or shared
activity, for example. A disclosed view of one is not automatically a portable
copy. A clip can refer to a live entity or capture some of its state; the
capture has its own identity and does not inherit the source's operational
authority. Captured content and a live source may coexist in the graph.

These are roles rather than disjoint node types. An editable clip can also
have live authoring state; its captured version and its current operations
must be distinguishable. Domain identities retain their own contracts: the
browser's canonical-URL Resource binding does not require every field or
device to adopt that same identity scheme.

An **appearance** is a projected occurrence of either kind. Multiple
appearances can reference the same source, with separate positions, selection
and local view state. Showing, selecting or moving an appearance does not
create a clip, clone its source or transfer ownership. The resource identity,
access identity and occurrence identity remain distinct.

### What the common model governs

**Portable collections (2026-10-11):** Mark's intended dataspace can selectively
replicate a collection of notes, clips, documents and recognized structured
exports gathered from the applications' various runtimes. The receiving mere
uses a declared content profile, carrying required bytes and meaningful
relationships so supported content remains accessible with the source apps
stopped. Format understanding and complete retained material each need a
qualified receiving path. The [Eidetic development plan's collection gate](eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md#p6-qualify-storage-custody-retention-and-exchange)
now makes this outcome explicit. It preserves the existing distinction between
portable content and live source authority; general replication remains planned.

| Concern | Shared meaning and owner |
|---|---|
| Identity and relationships | Domain identities and attributed assertions connect content and entities. Source, capture, revision and occurrence are distinguishable. Two equal payloads need not be the same clip or have the same provenance. |
| Content structure | Ordering, containment, references and selectors may be meaningful content. Their interpretation belongs to the named domain or format. They are distinct from an appearance's current coordinates or layout. |
| Arrangement and projection | An arrangement can itself be authored and shared graph data. Its authority governs edits; projection produces its present geometry and appearances. Per-view focus, camera and transient state keep their declared scope. |
| Dynamics | Declared forces, constraints, event responses and scripts operate on named targets through owned interfaces. Focus, synchronization and every state change do not become physics by being represented in the graph. |
| Governance | Owners, attributed writes, grants, petitions, disclosure and retention give relationships operational consequences. Graph connectivity alone grants neither read nor write nor execution rights. |
| Transport | Carriers move disclosed data and permitted requests. Transport independence preserves these contracts while exposing availability, latency, ordering and other carrier limits. Connectivity is not authorization. |

This is Mere's architectural answer for the family, rather than a claim that
all decentralized applications require graphs. It makes participation and
cross-application composition addressable without dictating one domain schema,
one renderer, one process or one user interface. Genet still owns document and
web-platform behavior, including the long-term standards-compliance work;
Mere's graph model does not replace those semantics. A unified GPU device is a
composition direction, not evidence that every source is already rendered in
one display list.

### Clip policy rulings (2026-10-11)

Mark selected Knot Editor and Redshank as the forcing pair, with a real shared
host receiving their exports. He settled these four questions in **Assess Mere
dynamics status** (`01a122a7-f13f-7d11-af4f-0361e1b9492e`). The Eidetic/Fleece
lane owns the profile, owner-adapter and tag design in the
[development plan](eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md#knotredshank-profile-and-owner-adapters).
The rulings settle policy, not a wire schema or implementation receipt.

1. **CP1, retained source backing.** Mark: **"Keep selected material plus private
   source backing when available (Recommended)"**. Keep the selected material
   and available source evidence under configurable storage limits. Sharing
   discloses the selection; including enclosing material or more source context
   requires an explicit choice. Private backing and the disclosed package have
   separate dependency sets. Availability, retention limits and later removal
   remain visible; the default does not invent unavailable source bytes.
2. **CP2, Redshank's first shared profile.** Mark: **"Note/body first; source
   excerpt is explicit (Recommended)"**. Retain the text or voice-note body plus
   its recording reference and frozen target. A voice body carries its required
   audio bytes. The original recording is distinct backing; when its bytes are
   absent the receiver can use the note and reports source playback unavailable.
   Exporting a recording excerpt is an explicit operation with declared
   representation, timebase mapping and lineage.
3. **CP3, generated tags.** Mark: **"Queryable suggestions; become personal tags
   when accepted (Recommended)"**. Engine suggestions remain queryable with
   producer, run, input revision and generation evidence. Acceptance creates a
   personal tagging assertion citing that evidence; it preserves the engine's
   attribution and does not rewrite the suggestion as a human observation.
   Existing concept identity and asserter rules remain governed by C7/ruling 20
   of the [graph semantics plan](mere_docs/implementation_strategy/2026-10-04_graph_semantics_plan.md).
4. **CP4, unavailable source owner.** Mark: **"Queue domain-supported edits for
   review when the owner returns"**. Retain supported inert edit requests with
   source-owner/entity reference, expected source revision, payload schema,
   author and request identity. On return, resolve a fresh admitted target and
   review the request; recheck grants and source revision before submitting it.
   Persisted scene handles or session tokens cannot authorize replay. Independent
   local annotations remain usable. Unsupported operations refuse; submission,
   owner acceptance and durable completion remain separate outcomes.

### Remaining policies and adapter decisions

The accepted direction and CP1 to CP4 above settle the model, units and named
policy choices. The remaining lifecycle and interoperability recommendations
below are **proposals pending ruling**, not additional editor rulings or claims
of implemented platform contracts.

1. **Captured evidence and subsequent editing.** Preserve the captured version
   and its available source/revision provenance. Make edits explicit revisions
   or derivatives, and keep annotations attributable to their own author.
   An editable note can remain editable; presenting it as captured evidence
   should identify which retained version supports that presentation. A live
   reference is labelled as live, with freshness and availability disclosed.
   CP1 settles selected material plus available private backing as the default.
   Configurable limits, rights, deletion/reference behavior and the consequence
   of losing backing still need owner-specific decisions. Knot can retain
   immutable source artifacts beside an editable clip note; the family-wide
   producer and custody integration remains to be qualified.
2. **Compound clips and addressable parts.** Give the portable clip an identity
   and describe its payload roles, media/schema versions and source anchors.
   Parts can be addressed within it; promote a part to a separate clip when a
   user takes it independently. Knot and Redshank now force the first profile;
   its exact shape and capability promises still need qualification. Retained bytes,
   external references and unresolved dependencies must be distinguishable.
   Content hashes identify bytes, not authorship, entity identity or rights.
   A portable reference is not a guarantee that its content is available offline.
3. **Sharing a live entity.** Distinguish sharing a captured snapshot, sharing
   a disclosed view and granting permission to act on the live source. Name the
   owner, observed revision or time, supported operations and offline behavior.
   When disconnected, show retained state with its freshness. CP4 queues only
   domain-supported source edits for reviewed admission when the owner returns.
   Cloning material creates a new owned entity with lineage rather than silently
   claiming the source's live identity. The remaining decision is which live
   domain supplies the first shared-operation and reconnect contract.
4. **Cross-domain edits and semantic portability.** Preserve unfamiliar content
   and schema identifiers without claiming to understand or execute them.
   Recognized adapters can expose inspect, annotate, reclip, open, play, edit or
   apply as separate operations, subject to their actual capabilities and
   grants. Route changes to the named owner; an annotation in one domain need
   not mutate its source in another. A compound edit is atomic only when an
   owner supplies that contract; otherwise show the separate applies and
   failures. The remaining decision is the first edit adapter, including its
   conflict, undo and failure semantics. Common graph structure supplies no
   universal merge or distributed undo guarantee.

The distinction is **portable material, interpretable meaning and authorized
behavior**. A receiver may retain and show a clip without supporting its
structured data or being able to operate its live source. Scripts carried as
content remain inert until an authorized runtime admits them. Missing or
unsupported capability must be visible; a generic appearance is not a promise
of equivalent behavior in every application.

### Current evidence and a forcing consumer

Source inspection on 2026-10-11 supports the bounded seams, rather than a
completed general clip system:

- [`Container`](../crates/eidetic/chartulary/src/container.rs) separates identity,
  addresses and content references, permits inline authored text and nested
  graph references. Payloads can remain outside the graph's individual nodes.
- [`Graph`](../crates/graph/graph-kernel/src/graph/mod.rs) holds resource and
  surface graphs with explicit shown-resource bindings.
  [`SourceRef` and scene instances](../crates/cambium/scenes/sceno/src/scene.rs)
  distinguish source identity from projected occurrences.
- [`write_clip_node`](../crates/mere/src/clip.rs) records a caller-formatted clip
  and asserts resource `ClippedFrom` provenance. Semantic clip lowering belongs
  to [Fleece](../crates/fleece/src/clip.rs); the receiving document owns formatting.
  Image storage remains caller-owned. This does not establish the proposed
  captured-version policy or a general compound media package.
- Fleece (`genet/components/fleece/README.md` and
  `genet/components/fleece/src/lib.rs`) extracts document
  content from a caller-supplied DOM and provides versioned canonical-text
  anchors. Capture, media recording, source identity, storage and disclosure
  remain with the caller and domain owners. Extraction alone is not portability.

A useful next research proof would take a document clip and a gameplay media
clip, with an optional domain-defined data export, through a second application.
It should retain, inspect, annotate, reclip and reopen the portable material;
preserve unfamiliar payloads and schema identifiers; distinguish source,
capture and occurrence identities; and disclose an unavailable live owner.
An unsupported action must leave the source untouched. A captured scene face
must remain distinct from the live entity it describes. This is a candidate
consumer proof, not a claim that a game exporter or portable package exists.

Promote only the shared contract that heterogeneous consumers actually require,
with a named owner, identity and lifetime, capability/refusal behavior and a
receipt. Wire formats, storage migrations and generalized entity APIs stay in
their owner-specific plans. This reconciliation opens no implementation lane:
the field/editor checkpoints, B1/S2 consumer gates, coordinator review and F199
pause retain their existing meaning.

### Research review: logical model and usable sharing (2026-10-11)

The closest internal predecessor is the
[2026-07-08 unifying-graph vision](mere_docs/technical_architecture/2026-07-08_mere_as_the_unifying_graph.md):
heterogeneous app data becomes a legible pool through graph relationships.
The [generic substrate plan](mere_docs/technical_architecture/2026-07-08_generic_graph_substrate_plan.md)
separates stable identity from optional content, address and graph capabilities.
The [2026-09-02 composition assessment](mere_docs/research/2026-09-02_port_gui_composition_and_comparable_stacks_brief.md#4-the-crux-deterministic-domain-aware-projections-out-of-app-pattern-data)
adds purposeful, domain-aware projections and typed intents. These are earlier
forms of this direction, with dated mechanisms and different levels of proof.

External research provides useful tests of the model:

- [iDM/iMeMex (Dittrich and Vaz Salles, VLDB 2006)](https://www.vldb.org/conf/2006/p367-dittrich.pdf)
  models heterogeneous personal data, including file contents and streams, as
  a logical graph of resource views. Components can be computed on demand;
  the logical model does not require prior materialization. **Reading for
  Mere:** expose useful structure without requiring wholesale conversion of
  each domain. Queryable structure and a retained portable clip have different
  lifetimes; the latter needs an explicit capture/export contract.
- [Dataspaces (Franklin, Halevy and Maier, 2005)](https://sigmodrecord.org/?download_id=4825&smd_process_download=1)
  combines autonomous, heterogeneous participants with differing query and
  update capabilities. Integration becomes more precise where its benefit
  warrants the work. **Reading for Mere:** common graph structure can connect
  sources before complete semantic integration; disclose gaps and capabilities
  instead of presenting a shared identifier as shared understanding.
- [Web Annotation](https://www.w3.org/TR/annotation-model/#specific-resources)
  distinguishes a source resource, a selector for its relevant part, and state
  identifying the appropriate representation. **Reading for Mere:** preserve
  the distinction between a live target and an observed version. This is a
  useful anchor vocabulary; it does not supply a self-contained clip package
  or permission to operate the source.
- [Solid Protocol](https://solidproject.org/TR/protocol) describes apps using
  one or more controlled storages and media-typed resources with RDF metadata.
  It does not impose a universal application schema. The older Solid rows here
  and in the composition assessment have been corrected. A common graph model
  and common domain schemas are separate architectural choices.

The important engineering distinction is between **identifying material,
interpreting it and acting on it**. Content hashes and predicate identifiers
help with the first; agreed profiles or adapters supply the second; named
owners and authorized operations supply the third. Graph-primary design must
make those agreements concrete where an application promises interoperability.
Optional feature traits are not permission grants, and exposing an appearance
does not establish either semantic understanding or mutation authority. This
review refines the rationale without adopting another system's node schema or
changing the unresolved lifecycle policies above.

### Shared semantic core: existing contracts and missing seams (2026-10-11)

**Assessment:** the shared core already has most of its structural mechanisms.
The missing work is agreement on a small content profile, source-owner adapters
and heterogeneous consumer qualification. General graph identity, blob storage
or another universal entity representation are not the first missing pieces.

This is source inspection at Mere `8601f759`, Turnstone `1034fb6`, Knot Editor
`4d8b81c`, Woodshed `9e4b372` and Cleromancy `ff3b1f5`. Product paths below are
cross-repository citations. Concurrent Forme and Redshank scenario work is
outside the review. Existing automated receipts are cited separately; this
pass runs documentation checks and establishes no fresh runtime acceptance.

#### What exists and what it establishes

| Existing contract | Established boundary |
|---|---|
| [Chartulary capabilities](../crates/eidetic/chartulary/src/caps.rs), [reservoir index](../crates/system/pandect/src/reservoir.rs), [scene sources](../crates/cambium/scenes/sceno/src/scene.rs) | Stable domain/node identity, persona/domain-derived mere identity, and separate projected occurrence identity. These do not automatically resolve every foreign source to its current owner. |
| [Eidetic manifests](../crates/eidetic/eidetic-core/src/manifest.rs), [bundles](../crates/eidetic/eidetic-core/src/bundle.rs), [portable content references](../crates/chirograph/src/titulus.rs) | Verified bytes, schema references, provenance/privacy metadata and composite members with required flags. Bundle member kinds remain domain vocabulary; required-member verification does not define a recursive clip export or retention policy. |
| [Schema definitions](../crates/eidetic/eidetic-core/src/schema_def.rs), [opaque facets](../crates/eidetic/chartulary/src/facet.rs), [Fleece acquisition and annotation records](../crates/fleece/src/eidetic_bridge.rs) | Versioned interpretation seams, preservation of unfamiliar facet data, and bound source/capture/canonical-text/selector evidence. Validation is distinct from another app understanding a payload. |
| [Chirograph](../crates/chirograph/src/lib.rs), [Pandect live-view records](../crates/system/pandect/src/live_view.rs) | Advertised actions, payload schemas, scene revision checks, explicit session states and source-owned live/historical cursors. A persisted reference or advertised action supplies no grant. |

The [completed Fleece/Eidetic plan](archive_docs/2026-10-11_completed_plans/2026-10-11_fleece_eidetic_capability_plan.md)
records authenticated annotation-plus-source transfer and receiver-only disk
reopen, alongside native and wasm checks. Its transfer test constructs a
[specific envelope](../crates/fleece/tests/eidetic_peer_transfer.rs) containing
both manifests and both byte payloads. This proves that bounded path; it does
not establish a family clip package or arbitrary dependency traversal. The
recorded consumer pins qualify `5758724e`, before later Mere dependency work.

#### Actual consumer paths

- **Turnstone to Knot:** `turnstone/src/knot_authoring.rs` lowers a document
  clip into the advertised `InsertKnotClipV2` action, with source/observed
  artifacts, selectors and fidelity facts. Knot retains artifact references
  through `knot-editor/crates/knot-editor/src/clip_evidence.rs` and authors the
  editable note through its endpoint. That endpoint checks both the delivered
  scene revision and the document's source `base_token`, with an injected write
  grant. This is an existing cross-app document operation, not a missing intent
  protocol or proof of general media support.
- **Retained source documents:** `turnstone/src/trail_memory.rs` deposits exact
  acquired bytes and a Fleece record as `LocalOnly`. Its captured-collection
  adapter and [Moot projection](../ports/moot/src/captured_web.rs) consume
  authority-filtered contributions, retain separate share/capture provenance,
  and expose missing or unsupported records explicitly. The Fleece record
  carries its canonical text, which may extend beyond the selected passage.
  A selection boundary therefore cannot by itself determine what is disclosed.
- **Timed media:** `woodshed/ports/redshank/model/src/lib.rs` exports shareable
  text/audio annotations with media-fragment selectors. Its internal
  representation receipts distinguish same, different and unproven copies;
  local seek policy uses that evidence. The annotation export names an audio
  body by its blob identifier and omits the internal representation receipt.
  A receiver needs the corresponding bytes and a declared identity/seek
  contract to reproduce those behaviors. Exported JSON alone supplies neither.
- **Live authority:** `cleromancy/src/admitted.rs` exposes several endpoint
  views over one durable app authority, forwarding typed intents under admitted
  subjects. Knot likewise supplies live document operations. However, searches
  of Mere and these four product source trees found only a fixture implementation
  of `LiveViewSourceResolver`; `LiveViewReferenceGate` implementations in Mere
  were fixtures/canaries. General persisted live-view reopening is an adapter
  gap, distinct from the existing live endpoint paths.

#### The bounded gaps

| Missing seam | Concrete requirement and owner |
|---|---|
| Durable source resolution | Product adapters must map a retained domain/source reference to the current admitted owner and its current offers, with missing/denied/stale outcomes. Reuse Pandect/resident routing. A saved scene `InstanceId`, endpoint action or document base token must not become the durable identity of a live entity. |
| A usable clip profile | Fleece gathering and domain exporters need one agreed profile over Eidetic's existing mechanisms: payload roles, stable part names/selectors, schema versions, lineage, and required versus optional material. Distinguish retained bytes, external references and unresolved dependencies. A selected note, a captured page and a media excerpt make different promises. |
| Selector and meaning interoperability | Adapters must preserve each selector's representation and units. Fleece positions count Unicode code points in normalized DOM text; Knot quotation anchors can count UTF-8 source bytes; Redshank targets count media time. Define explicit supported mappings and refusals. Shared profiles can agree on source, part, annotation and derivative roles while leaving domain schemas specialized. |
| Capture, disclosure and keeping policy | Session/custody owners must connect graph references to the package's required material, implement CP1's private-backing default, and check disclosure for each exported part. A byte reference, transferable source kind or selected passage is not permission to share its enclosing document. Configurable limits, release and revocation consequences remain explicit owner decisions. |
| Live operations and outcomes | Each domain must supply source revision checks, admitted operations, conflict/failure behavior and reconnect semantics. Chirograph's scene revision is delivery state; Knot's additional base-token check is the concrete precedent. Queued writes, durable completion and undo need owner contracts rather than inference from an `Accepted` reply. |
| Adoption and qualification | Hosts must advertise which profiles, selectors and operations they support, preserve unfamiliar material through the appropriate opaque path, and visibly refuse unavailable capabilities. Different current product pins require qualification; source availability on Mere main is not uniform consumer adoption. |

**Chosen forcing pair; implementation proof pending:** Mark selected the existing
Knot document lane and Redshank timed-media lane. Retain a text clip and a
text/voice note through a second
host, then reopen with their source app stopped. Verify required bytes, source
and capture identities, exact selector units, opaque unsupported payloads and
independently owned annotations. Exercise one already-supported owner edit,
then a stale edit and denied operation; neither refusal may mutate the source.
Restart the owner and resolve a fresh target instead of replaying saved scene
handles. CP2 keeps source excerpts explicit; CP1's private backing must stay
outside selection-only sharing. Test attributed personal tags, queryable engine
suggestions and CP4's reviewed reconnect edits. This is a proposed proof, not a
claim that these flows all exist.

The open questions above now concern concrete profiles and policies, not
whether the stack needs foundational graph, bundle or action machinery. This
review adds no crate and opens no paused lane. The four later policy rulings
above now govern the profile design; broader lifecycle checkpoints remain open.

### Embedding in caller owned wgpu contexts draft

**Status (2026-10-13):** Bounded native readback qualified: six contract/input
tests pass, and real egui/Iced callbacks render retained content and a Mere graph.
Window input, physical accessibility and wider renderer paths remain open. Mark wants Mere, Genet, the stack and application components
to embed into wgpu contexts with less integration work and more flexibility
than the external browser trio. This is a consumer target, not a claim of a
qualified universal GUI adapter.

The containing application supplies its GPU context and keeps its window,
event loop and composition. It can embed a capability without UI, one app
pane, a graph projection, a workbench, or a larger application surface. The
embedded component retains its domain/session authority and exposes supported
operations through the existing owner interfaces. Rendering and resource
intake are separate capabilities: an embedded graph may show remote references,
retained clips or host-supplied live entities.

#### Resource leaves and sharing

The working *resource as leaf* idea makes a resource a meaningful unit of
identity and sharing. Granularity follows the domain or an explicit selection:
a document can be a leaf in one view and a composition of addressed parts in
another. Promoting a part to an independently retained/shared clip creates its
own identity and lineage; a selector alone does not mint a graph entity.
Resource, access and appearance identities remain distinct.

Murm can exchange disclosed material and references; Gemot governs community
contributions and collections that refer to that material. Contributions keep
their own identity and context, as in the
[Moot ownership model](moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#shared-model-and-ownership).
Sharing material preserves independent encounters. Authored arrangements and
application-defined coop operations can also be shared under their own scope.

Foreign platforms can supply references, captured selections or declared live
operations through different adapters. A browser extension can acquire
accessible page content with its granted permissions; a PWA can accept imports
and shares; an embedded component can receive objects and capabilities from a
cooperating host. These adapters feed the portable content/profile work above.
Retained material carries source and capture evidence; operating on its live
origin still uses an actual supported owner interface.

#### Existing seams and the remaining host boundary

| Existing seam | What it already separates | Remaining embedding question |
|---|---|---|
| Sprigging paint commands and Vello scene leaves | Custom paint production from its retained host | How another toolkit presents that output and forwards its semantics/input |
| Conatus `HostDevice` and `DeviceNeeds` | A tenant's needs from host-owned GPU handles | How the whole native document/UI lane accepts a caller-owned context |
| Cambium Rootstock `Surface` and host lifecycle | Platform events from shared document/UI behavior | Its current surface still exposes a `RenderCore` and acquires a `SurfaceTexture`; arbitrary target views need an explicit boundary |
| Inker `SurfaceProducer` | Producer commands/events from raw native frames, format, synchronization and allocation epoch | Preserve native interop and backend/thread requirements while fitting another host's render schedule |
| Mere view model and requests | Presentation from session/storage authority | Adapt toolkit input, focus and accessibility while retaining the same owner requests |

Source homes are `crates/cambium/sprigging/src/lib.rs`,
`crates/conatus/tenant/src/device.rs`,
`crates/cambium/cambium-rootstock/src/lib.rs`,
`crates/inker/inker/src/surface_engine.rs` and
`crates/cambium/mere-view/src/lib.rs`.
The external trio is Scrying, Graft and Weld, with opt-in Inker adapters;
native Pelt/Genet remains the baseline under the
[suite composition ruling](2026-08-22_turnstone_suite_composition_and_capability_census.md).

#### Proposed host contract

| Boundary | Required contract |
|---|---|
| GPU context | Accept compatible host Device/Queue handles; declare features and limits before allocation. Track device generation and rebuild device-bound state after loss. |
| Frame composition | Accept target view or an owned intermediate texture, dimensions, scale, clipping, pixel/alpha format and sample count. Specify command ordering, submission ownership and when returned resources may be released. |
| Input and semantics | Map pointer/keyboard/focus, capture, IME and caret geometry, clipboard, cursor and accessibility updates/actions. Keep host and embedded coordinate systems explicit. |
| Scheduling and lifecycle | Report invalidation and wake requests; honor resize, visibility, suspension and disposal. The caller owns its event loop, with producer thread requirements disclosed. |
| Data and operations | Supply retained content or admitted source interfaces. Preserve identity, source revision, provenance, capability outcomes and storage authority independently of the drawing target. |

Native Genet presentation should render using the supplied context. External
browser producers retain their existing native-frame import path, including
format, synchronization and resource lifetime. Paint/scene outputs may offer
another composition path where their resources and semantics are representable.
These are distinct render capabilities under one host-facing lifecycle;
adapters advertise the paths they actually support.

A wgpu backend provides a promising integration point when its toolkit exposes
these hooks. Matching the wgpu API version, device, target format and required
features is part of each adapter's compatibility matrix. A toolkit's use of
wgpu internally does not establish public access to those hooks.
[egui-wgpu's callback](https://docs.rs/egui-wgpu/0.36.2/egui_wgpu/trait.CallbackTrait.html)
exposes prepare/paint stages and GPU handles;
[Iced's primitive interface](https://docs.rs/iced_wgpu/0.14.0/iced_wgpu/primitive/trait.Primitive.html)
exposes prepare, render-pass drawing or encoder/target rendering. These are
candidate adapter seams, not tested stack integrations.

#### First qualification target

Embed the same retained text/media view and Mere graph pane in two independent
toolkit hosts, initially egui and Iced, using compatible dependency versions.
Both callers should retain their event loop and GPU submission. Verify clipped
rendering, resize/scale changes, independent focus, keyboard/IME behavior,
accessibility action routing, wake/suspend/dispose and device recovery. The
same source shown twice must preserve separate access/appearance state, and
closing one appearance must release only its unnecessary resources.

The first source implementation extends existing Cambium Rootstock owners rather
than adding a second runtime. The opt-in adapters select egui 0.36.2 and exact
Iced source `ab336f6d6403d7206d82c7c2caa46143efde8284`, both using the stack's
wgpu 30 API. Released Iced 0.14 uses wgpu 27 and is not that qualified supplier.
The caller owns Device, Queue, command encoder, submission and device generation.
The adapter imports immutable native scenes from actual retained owners.

The bounded encoder path uses pinned NetRender's SparseHybridSession. It supports
native outline text and CPU-supplied images, using NetRender's atlas-cache-disabled
owned text path, and explicitly refuses unsupported fragments/filters. The
separate GPU glyph-atlas path can submit internally and needs supplier ownership
work before it is admitted. The
current default Classic Vello path submits internally; full encoder ownership
requires a separate supplier change that also protects deferred GPU resource
recycling and readback. Sparse admission does not prove Classic parity.

The Graphshell consumer bench must render actual held HTML, selected Fleece
text/WAV packets, image content and an actual Mere Canvas in both toolkit
callbacks, including clipping, repeated appearance state, source identity and
host generation replacement. Input conversion tests and GPU readbacks are
separate evidence from physical toolkit-window IME/AT acceptance. Receipts live
in `ports/graphshell/docs/receipts/caller_owned_toolkits_20261013/`; pending
checks are not passing receipts. The executed headless bench produced sixteen
byte-identical egui/Iced image pairs, with outside-clip, scale and independent
access checks. Rehydration changes caller generation on the same Device/Queue
after suspension; it does not demonstrate an actual device replacement or loss.
The final combined Genet b6/Netrender 6a bench retains all sixteen identical
pairs. Its `final/` receipt records the RTX 4060 Laptop Vulkan adapter, one
caller Device/Queue shared by both hosts, source hashes, PNG hashes and the
same-handle generation 1-to-2 transition. The later scheduling observer has its
own pool/consumer controls; those changes do not relabel the measured GPU basis.
The locked GPU shader supplier still forces
WESL/WGSL 0.4.0 with an older-compiler comment. That dependency currency question
remains with its supplier; this adapter does not rewrite the selected compiler
or existing Genet, Murm or Vello pins.

Qualify the native lane first. Optional Scrying/Graft/Weld providers retain
their separate backend and packaging gates. Extend existing render/session
owners before promoting a facade; source, producer and application semantics
remain with their respective owners.

### Browser WebGL2 presentation tenants design (2026-10-13)

**Status:** written design for owner review. The owner approved tenancy as the
ownership model and authorized this design. Written-design review precedes
implementation planning. This continues the embedding draft above; native
egui/Iced integration keeps its separate gates.

#### Purpose and first consumer

Graphshell's viewer must present the same disclosed graph and Cambium controls
with WebGL2 available and WebGPU unavailable. The browser host owns the graphics
context, frame schedule and presentation. Vello GPU is a renderer tenant;
graph and document producers are content tenants. Native presentation retains
its single host-owned wgpu device and queue.

First acceptance covers the viewer cone, its Inspector and ordinary controls.
Full product, remote and applet feature combinations require their own gates.
Genet's page-facing WebGL API and WebGL2 conformance are independent work.

#### Verified starting point

Source inspection used Mere `0b9b7a9b2`. Rootstock's `Surface` in
`crates/cambium/cambium-rootstock/src/lib.rs` exposes `RenderCore`,
`TextureFormat` and `SurfaceTexture`. `ProducerContext` and `ProducedTexture`
in `src/producer.rs` also expose wgpu handles. Cambium's web host `surface.rs`
and Graphshell's `src/web_gpu.rs` boot that core. The browser manifest enables
wgpu's `webgpu` feature. Direct WebGL2 needs a shared-host boundary change.

Conatus `tenant/src/device.rs` supplies `HostDevice` and `DeviceNeeds` for its
lit-body renderer. That crate retains its domain and wgpu contract. Presentation
belongs to Rootstock/platform hosts; scene lowering belongs to Netrender and
its providers. This design founds no generic tenant crate.

On October 13 the ThinkPad's Chrome 155 produced exact pixels through WebGL2
before resize, after resize and after forced context loss/restoration. The
hardware run identified AMD Radeon Graphics and peaked at about 313 MiB under
a 1 GiB kernel cap; SwiftShader passed separately. The fixture shadowed
`navigator.gpu` on its own page. This proves browser lifecycle feasibility,
not Vello or Graphshell acceptance. Raw source/result hashes are in
`Code/testing/genet/webgl-viewer-20261013/receipt.json`.

#### Ownership and frame boundary

Rootstock owns retained DOM, layout, input, focus and accessibility. Shared
frame preparation produces paint/scene descriptions and logical resource
identities. Presentation consumes that output, viewport, physical scale,
clipping and clear/composition policy; it reports presented, deferred,
context-lost or failed outcomes. These describe a proposed contract, not
compile-ready Rust signatures.

The host owns context creation, resize, wake delivery, loss listeners and
disposal. The wgpu adapter owns acquisition and submission. The WebGL2 adapter
owns its drawing-buffer/framebuffer target and one Vello GPU renderer on that
context. Device-specific handles stay within their adapter. Optional same-device
wgpu services remain available to native/WebGPU tenants; their absence on
direct WebGL2 is explicit.

Reuse neutral Genet paint commands and Netrender lowering. Retained geometry,
glyphs, images, transforms and placements preserve logical identity, while
each adapter makes its own resource/cache representation. Device-bound textures
and encoded renderer fragments cannot become interchangeable common handles.
Where the WebGL provider cannot transform an encoded fragment, re-emit retained
source commands and measure the cost. Keep one graph/layout implementation.

Vello GPU's direct WebGL2 route uses no compute shaders. It must be reachable
through Rootstock's shared redraw/semantics lifecycle, rather than a second
Graphshell render loop. Refactor the presentation portion of the host; retain
session authority and platform input with their current owners.

#### Content tenants and resources

Each image registration carries owner/appearance identity, content revision,
dimensions, pixel/alpha convention and context generation. Revision and
generation are separate. Scope registrations to the owning host/context;
refuse retired registrations, stale generations and incompatible resources
before composition.

The wgpu adapter supplies same-device texture views. A WebGL2 content adapter
supplies same-context textures or imports CPU pixels/browser image sources
through an advertised upload route. Native wgpu textures and external WebGL
contexts require an explicit transfer path; they cannot pass as host resources.
For the first viewer, use retained graph scene commands where representable,
preserving its model, picking and accessibility without an unavailable GPU
texture producer.

The importing adapter handles alpha conversion once. Qualify transparent
pixels, source-over order, clipping and scaled placement. Image caches update
on content revision and retire with their owning appearance. Import failures
name their owner and reason; content is never silently dropped.

Reserve ordered WebGL renderer/producer phases. Every phase establishes the
state it relies on: Vello can change GL bindings and active texture units.
Restoration of prior caller state is not assumed. Arbitrary third-party sharing
of the context is outside first acceptance.

#### Selection, capability and recovery

Expose a host-configurable presentation choice: `auto`, `webgpu`, `webgl2`.
An explicit choice boots that backend or reports a capability failure. `auto`
tries WebGPU and then direct WebGL2 after an attributed boot refusal. Use a
fresh presentation canvas if the first context choice prevents reuse, moving
listeners/semantics bindings while retaining application state. An ordinary
draw failure never silently changes backend.

Report supported scene/import operations and compute availability. Direct
WebGL2 uses the viewer's existing CPU dynamics route. Explicit requests for
unavailable GPU compute report that outcome. Required unsupported paint fails
with backend and operation identified; inventory the real viewer's operations
before advertising support.

On `webglcontextlost`, prevent default restoration cancellation, suspend GPU
work and report unavailable presentation. Preserve CPU model/retained source
state; invalidate GPU resources and outstanding captures. Input may continue
updating the model. Restoration advances context generation, rebuilds renderer
and producer resources, requests fresh producer frames and draws the latest
state. Reject old handles and completion callbacks. Hidden/zero-size surfaces
defer presentation; resumption requests a frame. Disposal releases owned
listeners/resources without releasing another appearance's state.

#### Acceptance and done-conditions

1. Freeze supplier revisions, features, locks, bundle hashes, browser versions
   and actual adapter identities. Qualify a Vello GPU WebGL2 scene first.
2. With WebGPU unavailable, mount the real Graphshell viewer through the shared
   Cambium host. Verify backend disclosure and graph/text/image/control pixels,
   clipping, opacity, scale and resize. Compare equivalent WebGPU/WebGL2 scenes
   with declared pixel tolerances.
3. Update graph and producer-image content in place, observing revisions and
   changed pixels. Remove/recreate an appearance and reject its stale resource
   registration. Include a stale-generation negative control.
4. Force loss/restoration on the viewer's actual context. Mutate its model
   while lost, then show the latest state. Verify resize, capture completion or
   refusal, and orderly disposal.
5. Exercise pointer picking, keyboard focus/actions and the existing DOM
   accessibility mirror with equivalent semantic targets on both backends.
   Automated mirror/action results remain distinct from human AT acceptance.
6. Preserve native/WebGPU host, same-device producer and viewer guards. Record
   unsupported operations and refusal controls. Measure retained update cost
   without inventing a performance threshold.

First acceptance requires a physical-GPU viewer receipt on an available host
and a functional receipt from a second browser implementation. Label software
rendering separately. Builds/browser tests use the common machine admission
lock and verified memory caps; choose ThinkPad or O-PC using fresh resource
readings. Existing owner permission covers those tests.

#### Alternatives and scope

A viewer-only loop could demonstrate rendering sooner but fork the shared
host lifecycle. WebGL hidden behind wgpu would not provide the selected direct
Vello GPU route or resolve the compute renderer's requirements. The selected
approach gives the shared host two presentation adapters over prepared content.

After written-design review, owner-specific dated plans sequence the interface,
Netrender lowering, browser adapter and viewer gates. This does not qualify
egui/Iced embedding, page WebGL2, media, or arbitrary foreign-context sharing.
Those retain separate contracts and acceptance.

## What this brief deliberately does not do

This brief records the composition thesis and its research lanes. It does
not found crates or promote proposed APIs. The historical sidequests and §7
implementation candidates enter owner-specific plans when their research
and consumer gates are met; existing implementation plans retain authority
over work already in progress.


## Shared Rust toolchain and minimum version (2026-10-13)

**Owner direction:** use the newest Rust across the stack, with build-directory
support, and unify the declared minimum versions as well. Existing manifest or
CI floors are observations of the tree, not evidence that the owner requested
older-compiler compatibility. The initial Murm move plan's inference of a
Rust 1.88 preservation requirement is withdrawn.

**Working toolchain:** current stable Rust 1.99.0, verified against the
[official release announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/).
Pin this exact release in active owned build roots for development, formatting,
linting and current-compiler qualification. Advance the working toolchain as
new stable releases are qualified.

**Selected shared MSRV: Rust 1.97.1**, independently of the working toolchain.
The owner accepted this floor provided no requirement pins a newer minimum.
The earlier interpretation requiring `rust-version = "1.99"` and simultaneous
pin/minimum upgrades is withdrawn. Align owned packages to
`rust-version = "1.97.1"` during the qualified rollout; advance the minimum only
when language use, supported dependencies or an explicit support decision
require it. Published older versions keep their historical metadata; do not
rewrite third-party/vendored suppliers' minimums to manufacture agreement.

The locked Mere metadata on 2026-10-13 contains 1,614 resolved packages and no
declared floor above 1.97.1. Iroh 1.3.0 declares 1.91, the refreshed p2panda
supplier declares 1.96, Knot packages declare 1.97, and WESL/WGSL packages
have the highest declaration at 1.97.1. Fetched owner manifests across eleven
active repositories likewise contain no declared floor above 1.97.1. Several
repositories pin 1.98.1 for development; that pin alone is not an MSRV.

A further scan of 71 tracked lockfiles inspected 2,491 cached registry package
manifests, again without a declaration above 1.97.1. There are 277 uncached
registry package versions, independent Git suppliers outside Mere are not
exhaustively resolved, and some packages declare no floor. These inventories
support selecting 1.97.1 for rollout; they do not prove every supported build
works on it. Qualify supported feature/target configurations on that compiler
and resolve any newly discovered higher requirement before claiming shared
support. Evidence: `Code/targets/murm-shared-msrv-selection-20261013.json`.

CI must exercise both policies: current-toolchain development checks and a
separate selected-MSRV build/test lane for supported configurations. Passing
on 1.99 alone does not establish compatibility with the MSRV. A newer working
pin does not change the support promise automatically.

This policy covers the active first-party repository family and its independent
application, library, probe, browser and firmware workspaces, including the
Murm/Retinue foundation, Mere, Genet, Dramatis, Turnstone, Woodshed and Knot.
The execution inventory must resolve current active homes and source pins;
archived standalone clones are not additional migration targets. A target that
actually needs a compiler fork or nightly has a documented target realization
whose compiler is checked against the separately selected support floor, rather
than an assumed permanent older MSRV exception. Workspace separation is justified by
dependency graphs, feature unification, target realizations and profiles; it
must not choose radio/native MSRVs merely from repository placement.

Use Cargo's supported `build.build-dir` for intermediate build artifacts and
`build.target-dir` for delivered outputs. Build-dir support stabilized in
Cargo 1.91; the 1.99 working toolchain satisfies that requirement without
requiring a 1.99 library MSRV. See the
[Cargo build-dir documentation](https://doc.rust-lang.org/cargo/reference/config.html#buildbuild-dir)
and [stabilization record](https://doc.rust-lang.org/cargo/reference/unstable.html#build-dir).
Use workspace/platform/configuration isolation, and audit environment/CLI
path overrides and cache keys. Packaging and test harnesses must discover
actual Cargo artifact/executable paths rather than assume the intermediate
layout is permanently `target/debug/deps`. A shared version does not require
all configurations to write into one unqualified cache directory.

The coordinated rollout inventories toolchain pins, owned minimums, CI jobs
and independent build roots, qualifies the new working toolchain, and
independently qualifies the selected 1.97.1 MSRV. It checks native, browser/Wasm
and embedded paths plus packaging/artifact discovery. It records compiler
versions in receipts and updates consumers' source pins where compiler changes require
supplier adoption. An older compiler's passing receipt is not a new compiler's
qualification. The Murm move plan carries this as MOVE1, before implementation
imports and consumer cutover.

**Current execution status:** defaults are published across the active Rust
repository family; the first Murm supplier import is published and its primary
Mere source cutover is qualified. Broader platform compiler qualification remains
separate from declaration-policy checks. Isolated checkouts
of all 21 active Rust repositories in the organization now carry the 1.99.0
working pins, 1.97.1 owned-package declarations and separate intermediate build
directories. Root metadata policy checks pass on both compiler versions (42
checks). Vendored/patch suppliers and frozen historical receipts are preserved.
The shared wgpu CI support-floor default is aligned too. Compiler qualification,
publication and the first Murm imports are recorded in the canonical Murm move
plan. Metadata checks alone do not establish all-platform MSRV compatibility.
