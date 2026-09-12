# Terminology

Canonical terminology for the Mere workspace. This file is the long-term authoritative reference for project vocabulary; until it's fully populated, [`2026-05-04_lexicon_brief.md`](2026-05-04_lexicon_brief.md) is the working source of truth.

For terms not addressed here, see the donor harvest indexes ([full harvest](mere_docs/research/2026-05-27_graphshell_docs_full_harvest.md), [concept brief](mere_docs/research/2026-05-17_graphshell_harvest_brief.md)). The donor `graphshell` repo is GitHub-archived (read-only) and its local clone was deleted 2026-05-27, so the old `../../graphshell/design_docs/` path no longer resolves.

## Top-level

- **Mere** — the platform: an application framework for showing data in useful projections, reaching it peer-to-peer wherever it lives, sharing it on the user's terms, and deriving and maintaining identities in relation to people and communities. Not a browser. Triple-meaning positioning: *merely* (humble) + *mere* (a small lake — still-water surface) + slant-rhyme with *mirror*. Lowercase **a mere** is one dataspace (see In-product vocabulary). Ruled 2026-09-05: the earlier "the product, the browser itself" sense belongs to **Turnstone**.
- **Turnstone** — the browser: the first Mere application, where browsing history is the graph rather than a tab strip. Its canvas is a mere. Absorbed the *Meerkat* / *Merecat* working names.
- **Genet** — the render engine: a pure-Rust, wgpu-rendered Servo fork that hosts web content, the Cambium UI layer, and embedded 3D and plotting surfaces on one GPU device. Progressively aligning with web platform tests. Formerly *Serval*.
- **Cambium** — the UI framework layer between Genet and the applications (formerly *xilem-serval*). Shared by Turnstone, Woodshed, Isometry, Strophe, and the other siblings. The scene lane (`sceno`, `scenomise`, `scenotime`) lives under the same umbrella as the widget lane, with separate state models.
- **Merely** — parent brand / company-name layer (adopted 2026-07-09, replacing *Strophos*). Takes the umbrella from the product rather than the reverse: Mere's own positioning already leads with *merely* ("merely a browser!"), so the parent is the adverb the product was named for. Humility as a house style. The prior *Strophos* (Greek στρόφος, "twist/turn," chosen to sit beside Verso's Latin "turned") is retired at every level; the twist/turn etymology no longer carries brand weight.

## Architectural roles (printing-press metaphor)

- **Engines** — there are exactly two top-level engines, and the word is reserved for them (ruled 2026-09-05). **Genet** is the render engine (what pixels get drawn). **Mere** is the projection engine (what data gets shown, where, to whom, as whom). Wry and Nematic are not top-level engines: Wry is a third-party system webview reached through Scrying, and Nematic is a family of *readers* (below).
- **Reader** — a concrete content parser implementing `inker::Engine`. Prose says *reader*; the code identifier stays `inker::Engine`, migrating opportunistically like `edge` (see **link**). **Nematic** is the portable smolweb reader family (Gemini/Gopher/Markdown/RSS-Atom and the rest listed under Reader layer).
- **Inker** — engine controller. Selects which engine renders which content; manages engine lifecycle; routes URIs to engines. Also owns the **flip**: re-presenting the same address through a different engine with the user's place and session carried across. Ruled and executed 2026-09-05: the `verso-tile` crate (1k lines; one external consumer, `mere-fetch`, for its `Cookie` type) folded into `inker` as `inker::flip` (`api`, `orchestrator`, `scry`, `genet` behind `genet-donor`) rather than taking a name of its own. The *Tympan* proposal is withdrawn with it.
- **Platen** — graph-aware composition surface. Knows graph semantics; presses node-data into renderable form for the tile layer to receive. Also the projection half of the workbench: `forme` owns what a workbench *is*, platen projects it as a tree.
- **Graphshell** — Mere's reference port: the application that showcases what Mere can do, the way Castellan showcases the identity and contacts capabilities and the way pelt is the reference browser over the render lane. Because a mere is reachable wherever it lives, the reference port is also the **remote projection host**: a wasm-first web/mobile client that connects to applications running on the user's other devices, receives sceno scenes and diffs, and returns granted intents. Viewing your laptop's graph from your phone is the canonical case; truth stays on the laptop, the phone holds a projection. It owns saved remote views and cross-application curation, never source truth. One sense (ruled 2026-09-05); the archived donor browser of the same name is in the retired table, and Mere's internal `shell` crates (`mere-chrome`, `mere-comms`) are unrelated.
- **Eidetic** — private local memory crate (formerly *Mnem*). Persistence layer for graph snapshots, traversal logs, settings, browsing memory. Distinct from a Moot's shared lanes. Name evokes eidetic memory ("remembered with high fidelity"). The substrate codicils are distilled from.

## Ports and hosts

The port law ([composition thesis](2026-08-12_family_composition_thesis_brief.md)): the stack owns a capability, a **port** is its first-party embeddable embodiment, and applications compose the relevant subset. Ports live in `mere/ports/`.

- **Castellan** — the identity port: the embodiment of the `personae` capability. Its keeper surface (`view`, `projection`) renders *about* secrets and never contains them; `PersonaeHost` (`authority`) holds the vault, serves the SSH agent, brokers approvals, and applies intents. Founded 2026-08-14 when graphshell's identity surface moved out to it. Castellan owns no repository of its own.
- **pelt** — the tiled reference browser over the render lane, with the static and scripted adapters: `ports/pelt` (`core`, `desktop`). Moved from genet to mere with the other application components; it is a port of the stack, not of Genet. A genet is a clonal colony, an animal's pelt is what it shows the world.
- **ortet** — Genet's raw host: one window, one document, no chrome (`genet/ports/ortet`). The original individual a clonal colony descends from; the one headed port that proves Genet runs with no Mere crate in its dependency cone (`assert_ortet_cone`). Distinct from pelt, which composes Mere.
- **Scrying** — external web content through the system webview (WebView2 first; engine ID `scrying.web`): the webview's GPU frames imported into the host's wgpu device and composited at the tile rect via the external-texture pass. The mechanism by which Wry reaches the screen; a host concern, never a reader.
- **fleece** — Genet's render-free extractor: walks the profile-neutral LayoutDom in document order and mints the readable article plus Web Annotation selectors (`TextQuoteSelector`, `TextPositionSelector`) and text-fragment anchors. An engine capability, not an application port; its `Article` shape is deliberately proprietary because no standard for "the readable article" exists.
- **tabard** — the design-token port: the derived palette as a DTCG (Design Tokens Format Module 2025.10) token tree, from which the CSS custom-property emitter and the host theme-struct emitter are both derived.

## Substrate (graph truth, persistence, motion)

- **kernel** — the portable identity, authority, and mutation kernel (`crates/graph/graph-kernel`): where graph truth lives. Compiles to `wasm32-unknown-unknown` with no UI, GPU, Servo, or platform I/O dependencies. Everything on a canvas is a projection of kernel state.
- **chartulary** (aka *chart*) — the generic content-addressed container graph: a `Graph<N, E>` where nodes are content-addressed containers and edges are typed relations, over one shared app-agnostic model. A chartulary is the register a house kept its charters and muniments in. Realizes **nested graph** containment; `chartulary::rdf` holds the RDF projection (folded in from *Scholia*, 2026-08-31).
- **muniment** — the portable persistence store: the muniment room where a household keeps its deeds as evidence. Durable state and nothing more; the storage under Eidetic, Stickleback, and every journal.
- **Journal** — `muniment::Journal<T>`, an append-only log of immutable entries with `append` and `replay`; edits are never destroyed, a change is a new entry. The plain storage primitive, distinct from a **Codicil** (a typed, content-addressed record) and from **Hagiograph** (a view over journals). Formerly `codicil::Codicil<T>` (renamed 2026-08-31).
- **seiche** — Rapier-backed force integration: the crate that realizes the graph's bodies as motion (`crates/conatus/seiche`). A seiche is the standing-wave oscillation of a lake, the whole body of water rocking; it moves the *mere* the way this crate moves the graph's bodies. Owns colliders and therefore press/select/drag (see **gnode**); holds tensor force laws (from *quint*, 2026-08-31).
- **sceno / scenomise / scenotime** — the scenograph family under Cambium (the crates still describe themselves as "the scenograph projection engine"; *scenograph* survives as the family name, not as a crate): `sceno` is the contract (scene snapshots, footprints, spaces, measurements), `scenomise` holds the solver registry, `scenotime` the temporal half. The generic `scenograph` facade crate is gone; the crates.io name `scenograph` is held for a scene editor. Say *sceno scene*, not *Scenograph scene*, for what a projection host receives.

## Surfaces (frame tree)

Terms from the [graph roster and frame taxonomy](mere_docs/design/2026-06-07_graph_roster_and_frame_taxonomy.md).

- **frame** — a leaf of the window's frame tree. Panes and the workbench are frames.
- **pane** — a frame-tree leaf answering one question about the graph or system: roster ("what is in the graph"), gloss ("how do I see and navigate it"), inspector ("what is this page made of"), apparatus ("how is the system doing"), comms. Summoned, toggled, and arranged from the shellbar.
- **tile** — a slot of node content inside the **workbench** (the tile tree: tabs and slots of pages). A tile shows what a node references; a pane shows something *about* the graph. Tiles can be torn out to other windows over the same backing graph.
- **shellbar** — the docked, edge-configurable chrome strip outside the frame tree that summons, toggles, and arranges panes: the frame tree's own UI. Its screen edge (Left/Right/Top/Bottom) is the *edge* sense unrelated to graph edges.
- **Navigator** — the historical name for the single graph-navigation surface per window, configurable in scope and form factor and never split into instances. Ruled 2026-06-07: *the Navigator is gloss, expanded*. Use **gloss** for the pane; *Navigator* survives only in older docs and in the phrase "expanded radially in the Navigator" under **volvelle**.

## Reader layer (inker / nematic / document model)

- **Reader** (code: `inker::Engine`) — `engine_id() -> &str`, `render(&EngineInput) -> Result<EngineDocument, EngineError>`. Sixteen nematic readers ship today: `markdown`, `gemtext`, `gopher`, `feed`, `text`, `file`, `finger`, `knot`, `knot-djot`, `scroll`, `misfin`, `nex`, `guppy`, `spartan`, `titan`, `html-fragment`. Counted by shipping `ENGINE_ID`, per the rule that implemented readers are listed and unwired ones are marked planned. Two need a note rather than an exclusion: `knot-djot` is knot's default body handling outside the blocks rather than a peer format, and `html-fragment` sits behind a feature that is on by default (`default = ["html-fragment"]`). Plus `genet.web` (external) and `host.external-protocol` / `graphshell.internal` (host-side).
- **Protocol-faithfulness rule** — protocol readers (gemini, gopher, RSS/Atom, finger, scroll, misfin, nex, guppy) populate document blocks only with what the source spec actually says. They do not invent semantic structure the spec doesn't define. RSS `<item>` becomes `FeedEntry`; finger plain text stays plain text; gopher menu items use the `gopher://` URL synthesis from RFC 4266. The only Mere-defined format that's allowed to be richer is **knot**.
- **Semantic-block intent** — the four `Block` variants beyond structural shape that name *what content means*, not just *how it's laid out*: `FeedHeader`, `FeedEntry`, `MetadataRow`, `Badge`. Intelligence layers (search, summarise, recommend, recall) match on these intents. Adopting them in protocol readers is *more* spec-faithful (RSS / Atom literally have entry-typed items), not an invention.
- **Trust ladder** — the `DocumentTrustState` enum: **Trusted** (verified through a chain of trust — TLS root, signed envelope), **Tofu** (first-contact-accepted, "trust on first use"), **Insecure** (unauthenticated transport — plain HTTP, file://), **Broken** (verification attempted and failed — cert mismatch, sig invalid), **Unknown** (default; not yet evaluated).
- **Provenance** — `DocumentProvenance` carries `source_kind` (engine ID), `canonical_uri`, `fetched_at` (RFC 3339), `source_label`. Readers populate `source_kind` + `canonical_uri`; the host fills in `fetched_at` after transport.
- **Knot** — Mere's native note / clip format. Frontmatter (YAML subset) + polyglot CommonMark body where fenced code blocks with protocol language tags (`gemtext`, `gopher`, `nex`, `feed-entry`, `feed-header`, `metadata-row`, `badge`) expand into real semantic blocks. Wikilinks `[[name]]` rewrite to `mere://node/<slug>`; hashtags `#tag` extract to `Badge` siblings. The only Mere-defined content format. Engine ID `nematic.knot`; default content-type `text/x-knot`.
- **Three-head Hekate** — Genet's planned evolution into a smolweb-extract / middlenet / fullweb negotiator for the same HTML input. Not yet built; locks in that nematic does not own an HTML reader-mode reader — HTML in any rendering depth is Genet's job (fleece is the extract head's seed). Hekate = three-headed Greek goddess of crossroads.

## Memory naming retired

- **Mnem** — replaced by **Eidetic**. The prototype name `mnem` was unavailable on crates.io.

## Comms layers

- **Stickleback** — the shared replicated-space layer beneath every signed peer
  domain: joined spaces and their drain, policy-before-insert processing,
  muniment-backed replicated storage, checkpoints, retention mechanics, and
  native drop carriage. The package is `stickleback`. It was `murm-replication`
  until 2026-07-26, when the multi-consumer reality (Murm, Mesh, Moot, and
  transport) earned it a name for the boundary rather than one consumer. A
  domain supplies its own operation grammar, addressing, authorization, and
  materialization — Stickleback never infers authority from transport access or
  visible membership.
- **Murm** — the peer-exchange family, a domain over Stickleback. Its public
  conversation service owns invitation-scoped murmurs, mail, and co-op exchange.
  - **Murmuring** — retired inner crate. Its signed-operation grammar and
    conversation engine were folded into `murm` on 2026-07-14. Internal
    mechanics use `ConversationEngine` and `ConversationStore`.
- **Moot** — the governed-space domain over Stickleback. It owns community
  identity, membership, constitution, governed settings, moderation,
  recognition, Standing, Tulpa adoption, FLORA receipts, and community
  projections.
  - **Mooting** — current home of recognition policy and, temporarily, the
    generic `MunimentStore`. The store moves to `stickleback`; the name is
    not a generic-plumbing law.
- **Moothold** — reserved for actual multi-moot holding or federation behavior.
  The current package also contains single-moot code for historical reasons;
  that code becomes the public Moot service during the peer-runtime reframe.
- **Gerund law (retired 2026-07-12)** — `murmuring`:`murm` ::
  `mooting`:`moothold` described the old workspace partition, not a durable
  semantic rule. The `murmuring` package was folded into `murm` on 2026-07-14.
  Use role-descriptive internal names and keep the product words for product
  concepts.
- **Gazetteer** — handle-resolution index: turns a name / handle / key into reachable, trust-stated endpoints (WebFinger today; NIP-05 / atproto-did / moot web-of-trust to come). An index / *directory*, not a broadcast *gazette*, so it sits on the persona / identity tier (`ports/gazette`; the crate is `gaz`), promoted out of the murm supercrate 2026-07-08. Incubating — no consumer wired yet, and its blocking HTTP needs an async port first.

## In-product vocabulary

- **murmur** — the user-facing word for an invitation-scoped conversation
  between identified participants. A murmur is the container, and individual
  posts are utterances within it. Participant count does not select Murm versus
  Moot; a Moot is distinguished by durable governance. Product surfaces say
  murmur.
- **Cable** — the semantic ancestry for conversations, channels, and signed
  posts. `mere/cable/v1` names Mere's Cable-shaped p2panda dialect. It does not
  claim wire interoperability with the cabal-club Cable protocol. Use
  `Conversation*` (`ConversationId`, `ConversationKey`, `ConversationHandle`)
  for storage/runtime mechanics and keep Cable terminology at this explicit
  protocol boundary. The borrowed *cabal* type prefix is retired (2026-09-05):
  it named cabal-club's thing while disclaiming cabal-club's wire format.
- **moot** *(count noun)* — a persistent themed federatable graph-view community: what a mere becomes when shared. Ruled with Mark 2026-07-30, two faces: the genesis face (a moot *begins* when your mere is shared; the tiers are escalating socialization of the mere, the datastructure moot makes social) and the grown face (a mature moot may govern one or more meres, which is the region-grafting model; a shared world never stops being a mere). Substrate rule: solo meres are born share-ready (one-writer signed spaces), so becoming a moot is a membership change, never a format migration
- **fili** — reserved name for Moot lineage: community ancestry, forks, and
  genealogy across related moots. Do not use it for ordinary event history,
  retention, or storage mechanics.
- **Hagiograph** — the legend and memorial layer: what memory makes of history
  through legends, memorials, epithets, and retelling. It is a view over event
  journals, distinct from the journal itself, an immutable Eidetic Codicil,
  and Moot descent in **fili**. This is the meaning formerly assigned to the
  standalone `tulpa` reservation; renamed 2026-08-31.
- **Tulpa** — a Moot's community-recognized, revisioned shared artifact and
  persistent collective identity. Signed proposals freeze their recognition
  electorate; endorsements adopt an exact artifact version; revocation and
  rollback change the effective projection while retaining every source fact.
  Tulpa takes the collective role formerly discussed as *egregore* and lives
  inside `gemot`, not as a standalone crate. A FLORA candidate may become a
  Tulpa through this adoption lane, but Tulpa is not limited to model weights.
- **gemot** *(count noun)* — a sovereign assembly of mootholds (t4; renamed from *coalition* 2026-07-30, which had renamed *demesne* 2026-06-04). OE *gemōt*, the collective form of *mōt* itself: the assembly of assemblies, rejoining the moot/moothold word-family where *coalition* was the Latinate outlier. crates.io `gemot` already held (0.1.0, claimed 2026-07-14 as the assembly-layer crate: Moot lifecycle, governance, replication, Standing, Tulpa, and FLORA), so the t4 count noun and the governance crate share the name deliberately
- **suzerainty** *(relation)* — the outer-tier ↔ inner-member relationship (moothold ↔ moot, gemot ↔ moothold); overlordship without absorbing internal sovereignty
- **volvelle** — UI form factor: a moot expanded radially in the Navigator (medieval rotating-disc knowledge instrument)
- **astroid** — internal UX vocab for hub-collapse: collapsing a subgraph to its central node forms an astroid-shaped boundary curve. The result is a *supernode* (derived, rebuilt per view, never synced), not a nested graph
- **servitor** — the resident helper unit: an installed extension or local agent, living as a node bearing a nested graph, holding a personae identity and a capability grant, proposing changes through the participant gate (so it cannot exceed its grant, and every act is attributed and revertible). Chosen by Mark 2026-07-17 for the chaos-magic sense: created, named, task-scoped, dissolvable. Crate name reserved on crates.io the same day (0.0.1). A human peer is never a servitor; both are *denizens*. *Animula* (Hadrian's "guest and companion of the body") is banked as the companion-flavored runner-up. See the [participant gate and packs plan](mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md)
- **denizen** — the umbrella word for anything admitted to act through the gate: a personae identity holding a grant and the right to submit petitions. Human moot peers, servitors, and scenario runners are all denizens; the trusted UI is not (it writes the journal directly). From English legal history: denization admitted an outsider by letters patent with a defined subset of rights, which is exactly the signed manifest plus grant. Ruled 2026-07-17 (replaces the working word *participant*)
- **petition** — a denizen's proposed change: a typed batch (graph edits lowering to captured deltas, app effects as Actions) validated against the grant and the journal revision before atomic, attributed apply. The journal records granted petitions. Ruled 2026-07-17 (replaces the working word *proposal*)
- **watch** — a denizen's subscription: the scope of the graph whose committed changes wake its body, with the containment law watch ⊆ read ⊆ grant (you cannot be woken by what you cannot read). Watches are declared in the pack manifest and reviewed at install beside the rings; a chain of wakes is a *cascade*, bounded by a budget that is a setting. Ruled by Mark 2026-08-13. Code note: `Watch` will collide with `tokio::sync::watch` and filesystem watchers; prefer `GraphWatch` or similar at the type level. See the [graph behaviors plan](mere_docs/implementation_strategy/2026-08-13_graph_behaviors_plan.md)
- **pack / mod** — the installable-bundle words, split by trust depth (ruled 2026-07-17): a **pack** is the plain user-facing word for a shallow-rung bundle (scenario/macro data, scripts; "campaign pack", "command pack"), a **mod** is a deeper-rung bundle (wasm components and beyond) whose grant reaches further. One Eidetic Codicil envelope sits underneath. Coheres with the existing `register-mod-loader` / `WasmModRuntime` naming
- **swatch** — a compact graph-canvas projection embedded in a pane: a scoped rendering of a graph or nested graph, either mirroring the main view (a minimap) or projecting through its own lens (independent layout, scope, or overlays; the gloss is a pane containing a swatch). A representation, never an identity: gnodes render in an orrery or swatch, while the graph itself lives in the kernel. A swatch over a servitor's nested graph is that servitor's inspection UI. Wording ruled 2026-07-17
- **gloss** — the graph-navigation pane: one surface per window, configurable in scope and form factor, that displays a swatch so the user can see their data differently without altering the main graph. It is what the *Navigator* became (ruled 2026-06-07); an outline is form-factor-agnostic and scope is the dial. This is the only sense of *gloss* in project docs; for the explanatory sense say *shorthand* or *informal name* (ruled 2026-09-05)
- **nested graph** — a graph (a set of relations) contained *within* a node: authored, and it syncs. Contrast: a **subgraph** is a selection of kernel nodes and the links among them (a scope, never a container, never synced as a thing); a **supernode** is a subgraph collapsed for display (derived, see *astroid*); a *swatch* is a canvas representation that may render a nested graph but never is one. *Graphlet* is retired (2026-09-05) and left to network science, where it means a small induced-subgraph motif. Ruled 2026-07-17; realized by the chartulary containment capability per the [participant gate and packs plan](mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md)
- **mere** *(lowercase, count noun)* — a configurable spatial dataspace: the unit an application integrates. Isometry's overmap, Woodshed's stage, Strophe's arrangement, and Turnstone's canvas are each a mere; a user has many. Capital **Mere** is the platform, lowercase **a mere** is one dataspace, and the platform is deliberately named for its unit. The word is the lake sense already carried in Mere's own positioning above (a small lake, still-water surface). **Amended 2026-08-13 (with Mark):** two borrowed terms of art apply on different axes, and a mere is genuinely both. *Dataspace* is the integration axis ([Franklin, Halevy & Maier, SIGMOD Record 34:4, 2005](https://dl.acm.org/doi/10.1145/1107499.1107502)): interrelated heterogeneous sources queried and navigated without full upfront integration, with relationships added *pay-as-you-go* — which is precisely cross-application linking between applications that never agree on a schema. *Datalake* is the storage axis (Dixon, 2010): raw native retention, schema-on-read, one accretive pool — of which this stack's *schema at the codicil boundary* is the sharper statement. The lake term's usual "derivative copies beside someone else's system of record" reading is deployment practice accreted onto Dixon's actual contrast (natural versus cleansed-and-bottled), not part of the definition, so it does not contradict a mere holding source truth. Where that connotation would mislead, say **reservoir**: the engineered impoundment with a catalog and a drain valve, which is [IBM's own coinage](https://www.redbooks.ibm.com/Redbooks.nsf/RedpieceAbstracts/sg248274.html) for the governed lake, and which retention epochs, provenance, and native drop are what actually supply. Reservoir is shorthand for explaining the distinction, not a minted term. Ruled with Mark 2026-07-26: the concept had no name and was informally covered by *orrery* while the reference host was the only application, which is why the word stopped stretching once four products each integrated one. Tier 1 is *your root mere*, replacing *orrery* at that tier. Amended 2026-07-30: the tiers are escalating socialization of the mere itself, so solo is just a mere (not a moot-of-one), a moot is when your mere is shared, and sharing never stops a mere being a mere
- **orrery** *(form factor)* — the cosmos-style spatial form factor: a whole dataspace seen at once, force-directed and in-scene. A way a **mere** is *rendered*, exactly as **volvelle** names the radial-moot form factor. Narrowed 2026-07-26 from its former lexicon sense ("a user's root graph view", tier 1), which **mere** now carries. Not a tier and not a container; `Scope::Orrery` and "orrery root" in code already mean this form factor, but the enum name says *scope*; rename to a form-factor enum when that file is next touched
- **Standing** — community-scoped reputation derived from signed receipts of
  commitment follow-through. It accrues against a persona-chain root and is a
  deterministic projection, not a transferable token. The previous name was
  **Tessera**. Existing `tessera.redb` stores and serialized
  `tessera_operations` remain readable during migration.
- **Codicil** — Eidetic's canonical schema-typed, immutable,
  content-addressed exchange record. This replaces **Engram**. The old source
  alias and legacy schema tags remain readable; new schemas and APIs use
  Codicil. An append-only sequence of events is instead a
  `muniment::Journal<T>`.
- **FLORA** — federated LoRA. In the Wang et al. FLoRA protocol, participant A
  factors stack vertically and B factors horizontally, with each participant's
  scale applied once to B. Heterogeneous ranks sum to the exact global rank
  under an explicit budget. Lower-case *flora* meaning a Moot's accumulated
  payload collection is retired; that was a capitalization-driven
  misunderstanding.
- **kith / kin** — contact tier distinction: *kith* = those known to you; *kin* = close. Orthogonal to moot membership.
- **gnode** — a node's rendered body on a graph canvas: the visible, spatially-placed object standing at the node's position in an orrery or swatch. A projection, never truth: rebuilt per frame from kernel truth plus seiche's live state, stores nothing. At most one per (node, pane instance); zero when off-scope/off-pane (the node demotes to an underlay dot, which is not a gnode). Anatomy: **body** (silhouette or custom hull, spatially coincident with the seiche collider: the face IS the collider), **face** (the body's texture: state color, favicon, or sprite), **caption** (label beside, LOD-driven); emphasis channels: selection = ring + lift, hover = wash, focus = focus ring, with color reserved for activation state. One primitive, two render tiers: a chrome-DOM `.gnode` element (focused pane) or an in-scene Scene layer (secondary panes; `render_gnodes_as_dom` picks per pane). Pointer-inert: seiche owns press/select/drag through the collider; a11y bounds are read off the gnode's painted rect. Distinct from the **node** (the graph object that references addressed things), from a **card** (summoned *about* a node or selection), and from non-spatial representations (roster row, tile tab, session chip). Etymology: g(raph)-node, coined 2026-06-02 as the orrery pool's CSS class; kept 2026-07-02 for the gnostic reading (the knowable body of the node). Full model: [node_card_summoning_design](mere_docs/design/2026-07-01_node_card_summoning_design.md).
- **link** — the user-facing word for a connection between nodes (adopted 2026-07-04; Mark: "if I started over today, I'd just call 'em links"). Plainer than *edge*, carries the right hyperlink lineage, and matches the statement-bucket data model (a link IS a statement: subject node, predicate, object node, with provenance and its own `StatementId`; the drawn connection between two nodes is the pair-local bucket that enumerates them — see the [petgraph-RDF plan](mere_docs/implementation_strategy/2026-06-18_petgraph_rdf_plan.md)). Scope: **product surfaces say link** — menu labels, omnibar verbs (`hide_link`, `show_all_links`, `relate`/`unrelate`), counters, roster tab (already "Links"), docs written from here on. **Code identifiers stay `edge`** (kernel types, petgraph vocabulary, CSS classes like `.roster-edge`) and migrate opportunistically when a file is touched for other reasons, never as a churn pass. *Edge* is not retired as an internal term — it is graph-theory vocabulary and correct at the petgraph layer; it is retired from user-visible copy. The shellbar's screen *edge* (Left/Right/Top/Bottom) is an unrelated sense and keeps its name. Turnstone has both graph links and page hyperlinks on screen; where both are visible, say **relation** for the graph link and **hyperlink** for the one in the content, and note that a followed hyperlink becomes a *navigation link* in the graph.

## Code-identifier survivors

Retired words that remain as code identifiers, migrating opportunistically when the file is touched for other reasons and never as a churn pass (the `edge` rule). Recorded 2026-09-05 so nobody re-audits them.

| Identifier | Retired word | Live sense |
|---|---|---|
| `edge` (kernel types, `.roster-edge`) | edge (user copy) | graph-theory vocabulary at the petgraph layer |
| `inker::Engine`, `ENGINE_ID`, `engine_id()` | engine (for readers) | the reader trait |
| `Scope::Orrery` | orrery (as tier) | the form factor; rename to a form-factor enum when touched |
| `crates/forme` (`forme`, `mere-forme`) | forme (as graphlet qualifier) | a *different* sense that stays live: the workbench arrangement authority, named for the locked-up printing forme. The retirement covers only the graphlet-qualifier use |
| `tessera.redb`, `tessera_operations` | Tessera | on-disk compatibility only |

## Retired terms (do not revive)

| Retired | Replacement | Reason |
|---------|-------------|--------|
| Graphshell *(the archived donor browser)* | **Turnstone** (the browser) / **Graphshell** (Mere's reference port and remote projection host) | The donor product was absorbed. The name was reclaimed 2026-07-22 for the reference port; that is now its only live sense. |
| Meerkat / Merecat | **Turnstone** | Working names for the browser |
| Serval | **Genet** | Render engine rename |
| xilem-serval | **Cambium** | UI layer rename |
| Strophos | **Merely** | Parent brand replaced 2026-07-09 |
| strophalos | none | Never used; too close to Strophos and Strophe |
| Engram | **Codicil** | Exchange record rename; legacy schema tags stay readable |
| Tessera | **Standing** | Reputation rename; `tessera.redb` stays readable during migration |
| demesne → coalition | **gemot** | t4 count noun, renamed 2026-06-04 then 2026-07-30 |
| participant | **denizen** | Ruled 2026-07-17 |
| proposal | **petition** | Ruled 2026-07-17 |
| graphlet / forme *(as its qualifier)* | **subgraph** (selection) / **supernode** (collapsed) / **nested graph** (containment) | *Graphlet* is a network-science term of art (small induced-subgraph motif); *forme* only ever qualified it. The `forme` crate is an unrelated live sense |
| cabal (`Cabal*` types) | `Conversation*` | Borrowed from cabal-club while disclaiming its wire format |
| Verso / `verso-tile` | `inker::flip` | Donor project is archived and owns the name; the crate is one type-consumer wide and folds into inker (ruled 2026-09-05) |
| Tympan | none | Proposed 2026-09-05 as verso-tile's new name; withdrawn the same day with the fold |
| Scenograph *(the facade crate and "Scenograph scenes")* | **sceno** / **scenomise** / **scenotime** | Facade crate removed; the crates.io name is held for a scene editor. *Scenograph* as the family's name stays |
| Navigator | **gloss** | One surface per window; ruled 2026-06-07 |
| Scholia | `chartulary::rdf` | Folded 2026-08-31 |
| quint / quint-shaders | `numen` (expressions), `seiche` (force laws), `conatus::resident` (GPU state) | Folded 2026-08-31 |
| Verse *(network layer)* | folded into Mere-at-network-scope | The Navigator handles networked-community as a form-factor of the same surface |
| Murmuration *(community layer, as a product or crate name)* | **Moothold** + count noun *moot* | TESS wall (Murmuration, Inc., civic-tech). The plain word stays usable in prose, e.g. Retinue's fleet vocabulary; only the crate is `murm` |
| Gist *(contribution unit)* | **Codicil** (via Engram) | Already canonical and richer |
| Flock *(contact grouping)* | **Kith / Kin** | More nuanced relational tiering |
| Mootcore | **Moothold** | Rename within this conversation |
| Verso *(as engine-controller)* | split: **tile** layer (rendering surface) + **inker** (reader controller and flip) | Two distinct concerns |
| Middlenet | **Nematic** | Better metaphor (aligned-but-flowing threads) |
| Mnem | **Eidetic** | Prototype name unavailable on crates.io; eidetic evokes "remembered with high fidelity" |
| Engine *(for inker-level parsers, in prose)* | **reader** | *Engine* is reserved for Genet and Mere; code keeps `inker::Engine` until touched |
| `nematic.smolweb` *(umbrella reader ID)* | Per-protocol IDs (`nematic.gemtext`, `nematic.gopher`, `nematic.finger`) | Concrete readers now exist for each smolweb protocol |
| HTML reader-mode in nematic | Future Genet head (three-head Hekate negotiator) | HTML in any rendering depth is Genet's job, not nematic's |
| "pelt is a port of Genet" | pelt is a port of the stack; **ortet** is Genet's raw host | pelt moved from genet to `mere/ports/pelt` |

## Status

Canonical for the words it defines. The lexicon brief remains the working source for anything not yet promoted here.

Done-condition: every capitalized term used in the body has its own entry or a retired-table row, and no word appears in two senses on this page. Met 2026-09-05 for the thirteen previously undefined terms (seiche, chartulary, Navigator, sceno family, muniment, Journal, kernel, shellbar, pane, tile, Scrying, Castellan, pelt); ortet, fleece and tabard were added in the same pass because the tree uses them.

Resolved 2026-09-12: the `graphlets` crate rename landed as `crates/graph/subgraph` (package `mere-subgraph`, lib `subgraph`), with forme's `graphlet.rs` becoming `crates/forme/forme/src/subgraph.rs` (`SubgraphId`, `SubgraphRef`, `SubgraphSpec`, `SubgraphKind`, `SubgraphBinding`, `SubgraphMemberDelta`), `SessionSubgraphs`, the `subgraphs.json` sidecar, the `graph:fit_subgraph` / `node:remove_from_subgraph` actions, `ArrangementKind::Supernode` (was `CollapsedGraphlet`), and the roster tab "Subgraphs". Kind variants (Ego, Component, Corridor, ...) and binding variants (UnlinkedSession, Linked, Branched) are unchanged. The survivors row for the crate is struck; the retired-table row above stays as the record. Docs written before the retirement keep the word under a leading note or a historical-citation marker rather than a rewrite.

Open: `design_docs/verso_docs/` keeps its name until someone has a reason to move it.
