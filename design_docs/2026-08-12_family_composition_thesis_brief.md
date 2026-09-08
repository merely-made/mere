# The Family Composition Thesis: every application its own datalake, one identity across, composable views

**Date:** 2026-08-12
**Kind:** research brief digesting a chat chain (Mark's framing prompt + assistant
response, 2026-08-12); analysis, terminology alignment, and system-shape prior
art added here. Extended 2026-09-08 with stack-pillar research lanes (§7).
Research includes initial model and Rust arena experiments; full consumer experiments and
implementation promotion remain open (see §7's experiment results).
**Anchors:** [application prospects brief](2026-07-24_application_prospects_brief.md)
(the three-seam composition thesis this elevates),
[Graphshell remote projection host plan](mere_docs/implementation_strategy/2026-07-22_graphshell_remote_projection_host_plan.md)
and [reference host plan](mere_docs/implementation_strategy/2026-07-27_graphshell_reference_host_plan.md)
(the rulings this extends),
[credential port + gazette brief](mere_docs/research/2026-08-10_credential_port_gazette_brief.md)
(the castellan split this generalizes),
[persona model brief](mere_docs/research/2026-05-14_persona_model_brief.md),
[participant gate + packs plan](mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md).

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
  petitions from denizens holding grants. The chain's insistence that a
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
  [2026-08-06](https://github.com/merely-made/retinue/blob/main/design_docs/2026-08-06_signalman_founding.md)),
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
| **Solid** (pods) | One user-owned pod; apps are views requesting access | The inverse split. A universal personal store pushes cross-app schema agreement onto every app pair, and the app ecosystem never came. Mere's per-app authority avoids the shared-schema tax; the projection contract, not a shared store, is the interop surface. |
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

**Status:** bounded source survey complete for R1/R2; initial model and Rust arena experiments
run, full consumer probes open. The survey observations below remain code
inspection; measured results are separated at the end of §7. This is the shared research
home; implementation belongs in the repository that owns the selected seam.

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
| Configuration | Typed owner settings, providers, Cambium presentation, host application | Already governed by the [configuration ownership plan](mere_docs/implementation_strategy/2026-08-06_configuration_ownership_settings_projection_plan.md); use its gaps rather than opening a duplicate lane. |

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
  `ports/ortet/src/shell.rs` currently redraws while unsettled.
- Genet `components/script-runtime-api/lib.rs` already exposes timer delay,
  worker work/pumping and worker shutdown. In the inspected
  `components/genet-scripted/document.rs`, the Livery pump advances timers,
  microtasks, capture and GC; it does not call `pump_workers`, and pending
  work is timer-based. This identifies an integration question for O5, not
  evidence that worker support is absent from the runtime.
- Genet `components/genet-render-host/src/lib.rs` and Netrender
  `netrender/src/renderer/init.rs` already place device creation with the
  embedder. Isometry `crates/isometry-runtime/src/render.rs` owns its tenant
  and stale-view refusal. Mesocosm
  `crates/mesocosm-core/src/voxel_profile.rs` gates disposable derived state
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
`design_docs/2026-09-05_reader_appearance_isolation_plan.md`.

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

Turnstone's `design_docs/2026-08-28_page_capture_plan.md` already scopes P2/P3:
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

## What this brief deliberately does not do

This brief records the composition thesis and its research lanes. It does
not found crates or promote proposed APIs. The historical sidequests and §7
implementation candidates enter owner-specific plans when their research
and consumer gates are met; existing implementation plans retain authority
over work already in progress.
