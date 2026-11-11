# Linked-data platform lessons: LinkedDataHub and *First Principles of the Web*

**Date**: 2026-11-06
**Status**: Research brief with rulings. Thirty decisions taken with
Mark on 2026-11-06 (see Decisions):

- D1–D5 come from the reading;
- D6–D8 come from the enum-dispatch measurement in §5;
- D9–D23 come from the pack-shape survey (§8) and the did:webvh
  confirmation;
- D24–D30 come from the Datalog pass (§9).

The valid-time, content-ID and property-graph/tables/spatial passes (§9)
are open.

No code is proposed
here.
**Question**: AtomGraph's LinkedDataHub (LDH) and the book its author
published alongside it argue for a web-native application stack built on
open statements. What should mere take from them, and where do our plans
already go further?
**Sources**: [LinkedDataHub](https://github.com/AtomGraph/LinkedDataHub)
(Apache-2.0, read at `master` and `develop` on 2026-11-06), and
[First Principles of the Web](https://firstprinciplesoftheweb.org/), draft
0.2 by Martynas Jusevičius (CC BY-SA 4.0). The book is paraphrased here,
never copied. Three read-only lanes did the reading: book chapters 5–7, book
chapters 19–21 with every cited spec, and the LDH source with the book's
proof appendices. Standards status was checked by web search on the same
day.
**Related**: the
[statements-over-schema stance](../technical_architecture/2026-05-22_statements_over_schema_stance.md),
the [link model](../technical_architecture/2026-10-15_link_model.md), the
composition spine (`graphshell/design_docs/graphshell_docs/technical_architecture/2026-05-21_mere_composition_spine.md`),
the [standards survey](../../2026-08-24_standards_survey_brief.md), the
[portable application experience brief](../../2026-11-04_portable_application_experience_brief.md),
and genet's
[fleece preservation contract](../../../../genet/design_docs/2026-09-05_fleece_preservation_contract_plan.md).

## 1. What was and was not read

Read: book chapters 1–12, 14, 16, 18–22 and 24, plus the references
appendix. Also LDH's README, releases, `formal/`, `tests/http/`,
`AGENTS.md`, the `ldh` CLI and the versioning, RDF/POST and caching code
paths. Not read: book chapters 9, 13, 15, 17, 23 and 25, the prose proofs
in Appendix B, and the separate `First-Principles-of-the-Web/proofs` Lean
repository. Claims below that depend on those are marked.

Provenance note: 198 of the 225 non-merge commits on LDH `develop` between
2026-10-05 and 2026-11-06 carry Claude co-author trailers, the Lean models
included. The book shows the same uniform cadence. That does not make
either wrong, but it is why every claim below was checked against code or a
spec rather than taken from the text.

## 2. The book's state model against mere (chapters 5–7)

**Chapter 5** derives "state is a set of triples merged by set union" from
three requirements: universality, merge without coordination (total,
order-free, idempotent, over self-contained facts), and global names. The
bridge from web practice to the merge laws (its "Transposition Thesis") is
admitted to be argued, not proved. Arity three is chosen for minimality, and
chapter 9 then gives that up by adding a fourth position for the asserter.
That fourth position is where mere's assertions already sit: separate per
(sub-kind, predicate, scope, asserter), with scope mapping to a named graph
(graph semantics rulings 1, 5, 9).

Mere goes further on merge. In the book's own worked example, two parties
assert 15 facts and the union holds 14: a duplicate collapses and its second
attribution is lost. Mere keeps both assertions on purpose.

**Chapter 7** normalises every write to a delta, the facts removed and the
facts added, both computed from the old and new state. It concedes that
removal and addition do not commute. Its fix is that each party removes only
facts it asserted and disputes others' facts by asserting its own, so deltas
from different parties commute. That is the link model's rule nearly word
for word: withdraw belongs to the asserter, per the predicate's
retractability, and hide is curation.

The book has no answer for:

- a fact two parties both asserted, which union deletes for both when one
  withdraws;
- one party on two devices, since its If-Match/412 ordering needs a single
  serialization point;
- a withdrawn fact returning through a stale merge.

The link model's retained withdrawal records (L2) and the journal cover all
three. Mere's "a stale assert does not revive a withdrawal" is the missing
piece the book does not name.

Worth taking:

- **Delta normal form as a property.** The least (removed, added) pair that
  carries one state to another is unique and recomputable. That is a cheap
  property test over assert/withdraw sequences, and it feeds D3's
  reference model.
- **Forms as marked patterns.** A form is a fact pattern whose fields are
  variables, each pattern marked remove or add, so one pattern answers both
  "find" and "change". That fits the application experience brief's rule
  that a host forwards a typed intent to the owner. An edit intent can be a
  pattern plus bindings, and the owner computes and authorises the delta.
- **Order is data** (chapter 6). The forme already treats arrangement as
  authority, so this confirms rather than adds.
- **Canonical form before arrangement.** A deterministic, lossless sort of
  the selected facts before any layout decision makes "shuffle the input,
  same output" a test. For ground data it is a lexicographic sort. Blank
  nodes need RDFC-1.0, which the book defers to chapter 9 (unread).

## 3. Dataspaces, identity and the standards (chapters 19–20)

The book's dataspace is a tuple of an origin, an ontology, a SPARQL endpoint
and a stylesheet. Storage is deliberately excluded. Writes are the four HTTP
methods read as special cases of one delta (POST adds, PUT replaces, DELETE
empties, PATCH is general) over the
[SPARQL 1.1 Graph Store Protocol](https://www.w3.org/TR/sparql11-http-rdf-update/),
with the request URI naming the graph. Different origins never collide on
names, so the graph name still records the asserter after a merge.
Alignment between vocabularies is itself a fact (equivalence, subclass),
attributed and retractable. The book concedes that alignment cost is real
and argues only that it degrades more gracefully than schema mismatch.

Standards status as checked on 2026-11-06:

| Item | Status | For mere |
|---|---|---|
| WebID 1.0 | Community Group draft, not on the standards track; the hand-over to a W3C Working Group stalled in 2023 | ADOPT as the profile face (D4); anchoring in D5 |
| Web Access Control | Solid Community Group spec | WATCH. Custody grants (doctrine Stack 3: supplier, grant, lifetime owner) already say more |
| Linked Web Storage Protocol 1.0 | First Public Working Draft, 2026-03-31, about a year behind its 2024 charter | WATCH as WAC's and Solid's successor |
| SPARQL 1.1 Graph Store Protocol | Recommendation | The simplest standard fit if a mere serves over HTTP: document URL = named graph |
| LDP 1.0 | Recommendation (2015) | The book claims it adds nothing beyond containers, argued from definitions only; not adopted |
| XSLT | Chrome disables it on Stable at M158 (2026-11-17) and fully at M176 (2027-08); WHATWG removal at stage 3 | Confirms the W3C review's SKIP. The book's whole arrange step depends on it, and LDH survives only by shipping Saxon-JS |
| RDF/POST | AtomGraph community spec | PULL (§4.6) |
| Memento, RFC 7089 | Informational RFC, the protocol web archives speak | PULL (§4.2) |

There is no new web-platform-tests surface here. RDF/POST rides on ordinary
urlencoded form submission, which genet's forms plan already gates on WPT.
RDFa is gated by the W3C RDFa test suite, and the fleece contract already
assigns that processor to `mere-linked-data`.

## 4. LDH's six mechanisms, as built

### 4.1 Lean models with a differential oracle

`formal/http` (graph store, ETags, Memento) and `formal/acl` (WAC-style
authorization) on `master` come to 3,314 lines and 134 theorems. `develop`
adds an auth model and a client model, for about 4,700 lines and 170
theorems. There are no `sorry`s and no added axioms.

Depth is mixed:

- **Substantive:** the write invariants, and a Boolean ACL decision
  procedure proved equal to the propositional one.
- **True by construction:** no-lost-update (ETag collision-freedom is
  assumed as a structure field) and delegation non-interference.

What makes it useful in practice is not the proofs:

- The models compile to executables that predict the server's status and
  resulting graph from N-Quads dumps of a live instance, under seeded
  random scenarios.
- 500 generated TimeGate cases are diffed against the Java.
- A CI check fails a pull request that changes a mapped Java file without
  touching the model. That is a file-touch heuristic, not a semantic check.

D3 takes the pattern in Rust.

### 4.2 Memento and versioning

Memento is query parameters on the document URL: `?timegate` with
`Accept-Datetime` redirects to `?version=<sha>`, and `?timemap` answers in
PROV-O or link-format. The snapshots are sorted N-Triples mirrored to a
GitHub repository through the Contents API, committed asynchronously. Bursts
coalesce, so a version is a snapshot, not one per write. Deletes become
deletion commits.

Mere's journal is richer: per-assertion first and last times and withdrawal
records. That gives Memento two homes for us:

- a mere that serves over HTTP exposes its history through TimeGate and
  TimeMap rather than a custom API;
- fleece and Turnstone consume it from web archives, which the fleece
  contract already names for acquisition and replay.

### 4.3 Agents under the same access rules

Agents are ordinary WebIDs. Delegation is a client certificate plus
`On-Behalf-Of`, honoured because each user's profile names the delegate.
That delegation is unscoped: LDH's own `obo_complete` theorem proves the
delegate can act for anyone, and their roadmap lists scoping it as the
expected fix. Custody already scopes this.

The pattern worth taking is the assistant UI: show the plan, run read-only
plans at once, hold writes for an explicit Execute, and run them as the
user.

### 4.4 Self-federation

The book's test is two instances at two origins, each browsing, querying
and writing the other with only spec terms on the wire. LDH does not
actually run it: its "federation" scripts address two dataspaces of one
Tomcat. Two mere hosts exchanging only admitted-interface terms belongs as a
gate in the
[application experience plan](../implementation_strategy/2026-11-04_application_experience_plan.md),
with no in-process shortcut.

### 4.5 Select, arrange, present

LDH stores views as RDF documents: a stored query, a mode (list, table,
grid, map, graph), and blocks ordered by `rdf:_n`. The browser runs the
query and renders client-side. There are no per-stage validators. Caching
is Varnish with xkey purges, and the document ETag is SHA-256 over the
sorted graph folded with the variant.

Mere's chain (saved query or frozen selection, forme arrangement, platen
projection, surface) already separates the stages. Two things to take:

- per-stage validators;
- a sorted-graph hash as the resource validator.

### 4.6 RDF/POST

Even LDH routes around its own form encoding. Forms are parsed in the
browser and sent as RDF/XML (PUT, POST) or as a SPARQL Update that replaces
the subject's outgoing triples, which is not a minimal delta. Every blank
node is skolemised to `<doc>#id<UUID>`.

Mere's advantage is real: genet owns form submission, and a mere can be the
receiver. RDF/POST is still a vendor spec, so PULL: the edit-intent shape in
§2 reserves room for it.

## 5. Generic and opinionated: declared lenses

Chapter 21 is less generic than its title. Its "relative genericity" is a
vocabulary-blind base rendering, with overrides allowed only for names an
imported ontology declares. Domain computation is pushed out to agents that
submit deltas. That has the same shape as doctrine Surfaces 3, where
activation refuses a registration the manifest did not declare.

The real difference was where recognition lives. The book puts it in
installable data packages. Mere's 2026-05-22 stance put it in compiled,
exhaustive enums (`EdgeFamily`, 6 variants, and `SemanticSubKind`, 17; 766
references across 149 files).

That stance's reasons do not survive a check. "Behavior dispatch wants
exhaustive closed enums" was asserted, not measured:

- IRIs interned to integer ids at pack load make dispatch an integer
  compare;
- exhaustiveness is replaced by declare-and-verify at activation plus
  generic fallback rendering.

What remains are costs, not reasons:

- moving 766 references across 149 files;
- `EdgeFamily` is rkyv-archived, so stored data needs a migration, and
  graph semantics ruling 11 has migration replay the journal first;
- retractability, which the link model gives per predicate, must be
  declared per predicate in the pack.

D2 rules that everything moves to packs. The shared pack shape waits on the
consistency survey (Open, below).

**Measured on 2026-11-06, after D2.** Of 72 variant-level matches on
`SemanticSubKind` across the tree, 68 are the IRI table in `edge_data.rs`,
the snapshot encoder and decoder, and their tests. The behavior-side sites
are three lookup tables: stratum in `predicate_registry.rs`, CiTO and PROV
alignment in `linked-data`'s `vocab.rs`, and durability and provenance on
`ArrangementSubKind`. Pictograph already dispatches on the family as a
string. `RelationKind::OpenPredicate`, with its reserved sentinel ordinal in
the `u32` tag, is an open set inside the closed enum; the tag has 15
non-test consumers. `predicate_declarations.rs` already stores per-mere
declarations as a durable facet, with conflict handling and an
effective-stratum lookup that consults the built-in table first, so the
core pack is that table in the same record shape, loaded first. The
2026-06-19 statement kernel brief had already placed predicate semantics in
a registry rather than in family enums, along four axes; the stance's
guardrail and the code did not follow it. What the enums do give up is
compile-time exhaustiveness, which becomes an activation check (every row
has every axis; the core pack covers every id the code references), and
the stance's frame-rate worry, which nobody has measured. D6–D8 record the
model chosen on this evidence. D31 and D32, from Mark's later question,
replace "closed" with "observed": the finite part is the core's behavior,
not the vocabulary.

## 6. Identity: WebID, did:web, did:webvh

The 2026-09-23 ruling graded DID methods on what an anchor needs: stable
per account, able to rotate keys, bound by something other than a host. It
said did:web has "no history".

Rechecked on 2026-11-06:

- **did:web** can rotate keys, by republishing `did.json`. What it lacks is
  verifiable history: whoever controls the domain or hosting can rewrite it
  undetectably.
- **did:webvh**, [DIF v1.0](https://identity.foundation/didwebvh/v1.0), adds:
  - a self-certifying identifier derived from the genesis state;
  - a hash-chained log back to genesis;
  - optional pre-rotation and witnesses;
  - a parallel did:web view.

  Three implementations exist, among them the Rust crate
  [`didwebvh-rs`](https://docs.rs/crate/didwebvh-rs/0.4.2). It can be
  self-hosted, where did:plc depends on plc.directory.
- **WebID** profiles can carry keys (Multikey) and `solid:oidcIssuer` and
  rotate by editing. Like did:web they have no history.

D5 adds did:webvh beside did:plc. The confirmation lane's findings and the
choices still open on how we carry it are listed under Open.

## 7. Where each finding lands

| Finding | Lands in |
|---|---|
| Executable reference model and differential runs for assert/withdraw/compaction (D3) | [link model plan](../implementation_strategy/2026-10-28_link_model_plan.md) |
| Delta-normal-form and canon-under-shuffle properties | the same reference model |
| Memento exposure of assertion history | the application experience plan's serving phases |
| Memento consumption from web archives | genet's fleece contract and Turnstone capture |
| Self-federation gate | the application experience plan |
| Per-stage validators, sorted-graph hash | [graph semantics plan](../implementation_strategy/2026-10-28_graph_semantics_plan.md), resource strata |
| Edit intent as marked pattern, RDF/POST PULL | the application experience plan's intent seam |
| Plan-then-Execute assistant writes | dramatis custody |
| Declared lenses (D2) | the stance doc rewrite and a plan, after the survey |
| WebID ADOPT, maturity principle (D4); did:webvh (D5) | the standards survey; doctrine Stack 3 |
| Axes closed, names open; classes in packs; generated ids (D6–D8) | doctrine Stack 3; the declared-lens plan after the survey |

## 8. Pack-shape consistency survey

A read-only survey on 2026-11-06 found 21 places across the repos that
already declare vocabulary or behavior as data. They include:

- isocosm's content packs, edition pins, subsystems, appearance overlay,
  wing-glyphs and datasheets;
- genet's livery datasheet and WebIDL table;
- knot's signed user predicates;
- eidetic's impresa kinds and its signed `mere.pack/v1`;
- woodshed's audio packs;
- gemot's schema by digest;
- mere's mod manifests, WIT world, scenes and domain contract.

Dramatis has none.

The inconsistencies:

- **Term identity.** Seven schemes are in use: IRIs, `namespace:local`,
  bare slugs, dotted names, kebab names, `lens:` ids and persisted
  ordinals. Knot hand-mirrors mere's 17 slugs because it depends on no
  graph crate.
- **Pack digests.** Isocosm uses SHA-256 over source text. Woodshed uses
  BLAKE3 over source plus a format byte. Mere mods, wing-glyphs and impresa
  have none. Both source-text hashes break the data formats rule that
  identity is never the source text.
- **Signing.** Eidetic signs serde_json, where the rule asks for
  deterministic CBOR.
- **Format.** Woodshed's bundled pack and the wing-glyphs packs are
  hand-edited JSON, against "no hand-edited JSON in a pack directory".
- **Behavior hooks** attach five different ways: registry key, script
  path, native or wasm registration, a WIT name, and knot's `lowers_to`.

`PackPin {id, version, hash}` (isocosm) and `PackIdentity {id, version,
digest}` (woodshed) already agree, which seeded D10's shared header.

What should stay different:

- genet's livery and IDL tables, where recognition is fixed by a standard
  and the path is hot;
- closed domain event grammars (the domain contract's native rung);
- datasheets, which packs carry.

Two existing rules settle two of the survey's forks without a question. A
pack digest is taken over the parsed canonical form. Pack directories hold
TOML, so the JSON packs convert and dual loading goes.

Side notes: `crates/script/wit/world.wit` and
`crates/graphshell/graphshell-endpoint/wit/world.wit` are byte-identical
copies. A comment in isocosm's `content/pack.rs:13` still says "JSON format"
for a TOML-first manifest.

## 9. Other data models (2026-11-06)

Mark asked which other data models the stack might want. Datalog, valid
time, content IDs, and property graphs, tables and spatial values each got a
read-only pass the same day. Each pass came back with forks.

**Datalog and incremental views.** D24–D30 rule on this pass.

The stack already derives facts:

- containment is materialised from URL paths
  (`graph-kernel/src/graph/query.rs:369`);
- saved queries compute on read;
- knot's readings propose and never assert;
- isocosm derives stats in Lua through piccolo.

The link model's `basis.method` and `basis.evidence` (§6.2) are the
attribution slot for derived facts.

Engines:

- ascent and crepe compile rules into the binary through macros, so they
  cannot run rules that packs declare at runtime;
- differential-dataflow 0.26 (MIT, released 2026-11-05) handles runtime
  rules with true retraction, but its wasm support is unproven;
- SPARQL 1.2 RL, a Datalog-style rule language for RDF, was published as a
  W3C Working Draft on 2026-11-05.

Didgeridoo, the family's piccolo fork (genet pins it by git rev, and it is
not on crates.io under that name, despite the murm transport plan's line
204), meters work deterministically through fuel, but the VM is not
deterministic:

- every table seeds its hasher randomly (`src/table/raw.rs:83`);
- `math.random` is seeded from the OS (`src/stdlib/math.rs:296`), and
  isocosm-system still exposes it;
- floating-point maths depends on the platform.

Biscuit as shipped checks a 1 ms wall-clock limit, which would break domain
contract C6.

**Valid time (bitemporal facts).** Open.

- **Where real dates live.** Where valid time is real, it lives as a
  property inside a domain's payload: cleromancy's birth facts, sky spans
  and weather, and `schema:datePublished`.
- **Where it is faked.** Knot (`KnotAssertedTime::At`) and cleromancy's
  journal put real-world dates into record time. No migration can separate
  those later, and the cost grows with every write.
- **Already bitemporal:** eidetic's property records
  (`eidetic-core/src/records/mod.rs:111`). Chartulary keeps valid time
  separate by design.
- **What goes wrong if it lands in `basis` outside the reason key.**
  Reassertion overwrites `basis` in place (`edge_data/semantic.rs:42`), so
  valid time stored there would be silently overwritten. Making it part of
  the key turns a correction into a withdrawal plus an assertion.
- **Prior art:** SQL:2011 periods, XTDB v2, Datomic (transaction time
  only), OWL-Time (Recommendation, 2017), Wikibase qualifiers with
  precision, and EDTF / ISO 8601-2 for uncertain dates.
- **Timing.** The domain contract's fresh Stickleback logs (C1, C5) mean the
  field can be reserved in the new graph grammar rather than migrated into
  the old journal.

**Content IDs.** Open.

- `eidetic-core/src/schema.rs` already has a multihash `HashFn` (BLAKE3,
  multicodec `0x1e`), per the event-DAG brief §13, with hand-written
  parsing and a `blake3:<hex>` string form.
- CIDv1 adds a version, a content codec (`raw`, `dag-cbor`) and a
  multibase string, which is what atproto and IPFS use.
- The multiformats crates are maintained: `cid` 0.11.3, `multihash`
  0.19.5, `multibase` 0.9.3.
- Multihash and multibase are IETF drafts. CIDs are outside that IETF
  group's scope.

**Property graphs, tables, spatial values.** Open.

- **Property graphs.**
  - GQL (ISO/IEC 39075:2024) and SQL/PGQ are ISO standards.
  - No W3C mapping exists from RDF 1.2 to labelled property graphs, and no
    maintained, store-agnostic GQL evaluator exists in Rust.
  - A read-only export of nodes and edges from assertions is cheap and
    lossless.
  - Book chapter 15 argues the property graph is a query shape, not an
    identity layer.
- **Tables.**
  - CSVW (W3C Recommendation, 2015) has no Rust implementation and fits CSV
    imports, not TOML datasheets.
  - Nothing in the stack depends on Arrow or Parquet today. Murm's survey
    tracks (up to 100,000 points, JSON) and woodshed's frame-level analysis
    are the named consumers.
- **Spatial values.**
  - GeoJSON (RFC 7946) and GeoSPARQL 1.1 (OGC) are the standards.
  - `geo` and `geojson` are healthy crates.
  - Murm already has latitude and longitude, cleromancy has observer
    radians, and isocosm uses region coordinates rather than geodetic ones.

## Decisions 2026-11-06

**D1. Which LDH and book items to pursue.** Asked which of six items to
take further: (1) Lean proofs in CI, (2) Memento, (3) agents as identities
under the same access rules, (4) the self-federation test, (5) select,
arrange and present as separately cached stages, (6) RDF/POST. Mark: "1, 2
for sure, 3 (think we already do that or intend to), 4 for sure, 5 for sure,
and we have more of an advantage for 6 than they do even, so sure."
*Outcome:* applies to Stack 3 (open formats first). §4 records where each
landed; item 3 is already custody's.

Links: area The stack · draws on mere:stack.3 · summary Pursue Memento, self-federation, staged read caching and RDF/POST as PULL; formal checking of the assertion store; agent access is custody's.

**D2. Where recognition lives.** Asked whether a recognized predicate's
behavior lives in compiled enums, in data-declared vocabulary at the
ladder's data rung, or entirely in data. Mark chose "Move core to data".

This contradicted the 2026-05-22 stance, so it went back as a question
(resource graph only, both graphs, or data-declared but compiled in). His
answer, which reframed it:

> I'm willing to do 1 or 2 but it seems kind of ridiculous to imply generic
> is a detriment without qualifying why. Seems to just mean interoperable
> and definable for my purposes with my own vocabulary packs. Directly
> supporting modularity. And the toml, json, taplo… ideas percolating. And
> wit. And wasi. So it seems to me like "wants an exhaustive list" is not a
> reason, but a justification with no basis. Still, fully generic resources
> with, i don't know, app or pack or domain defined workspace families
> sounds reasonable too. Scenes could include that typing. Just seems like
> you want the enums because you like a long, closed list out of simplicity
> even though it directly contradicts the standards, the stack, and the
> goals

Put back as the end state for "everything in packs":

- authored TOML vocabulary packs declare predicates, families (workspace
  ones included), retractability and presentation;
- IRIs are interned to ids when a pack loads;
- scenes carry the typing;
- behavior beyond data climbs the ladder;
- today's families and sub-kinds ship as a first-party core pack.

Mark: "1, but check isocosm's efforts, genet's efforts… for consistency and
making it consistent where that helps."

*Outcome:* refines Stack 3, whose data rung gains declared vocabulary packs.
It replaces the stance's closed-lens guardrail. The stance rewrite waits on
the consistency survey.

Links: area The stack · amends mere:stack.3 · contradicts statements-over-schema stance (2026-05-22) two-layer guardrail · summary Recognition is declared in vocabulary packs, workspace families included; the current families become a first-party core pack; consistent with isocosm and genet where that helps.

**D3. The assertion store's checking instrument.** Asked to choose between
a Rust executable model with differential runs, a Lean 4 spec as an oracle,
or TLA+ with trace validation. Mark: "Rust model + differential". That
means a small executable reference model in Rust, proptest state-machine
runs diffed against the real store, and a drift check tying the model to
the sources it maps.
*Outcome:* new for the link model plan; no doctrine statement.

Links: area The stack · plans 2026-10-28_link_model_plan · summary The assertion store gets a Rust reference model checked differentially against the real store, with a drift check.

**D4. Standards maturity, and WebID.** Asked how to grade WebID, a
Community Group draft, when did:web had been graded PULL as a handle only.
Mark:

> Isn't this like saying we shouldn't use rdf 1.2 because it's not quite
> established? Even though it's clearly supportable and understandable, and
> it's an abundance of caution running decades that keeps these things from
> being standardized canonically long after they are technically viable and
> broadly adopted? Stuff like that, did:web, i actually think we should just
> support community drafts if we have a reason to use them. And we clearly
> do! If it's cheap.

The reply separated status from trust. It proposed the principle that a
draft or community report we have a use for is supported when it's cheap,
with every claim labeled by its status. It offered WebID as ADOPT for
profiles, with anchoring judged separately. Mark: "I am OK with one, but I
doubt that you seriously considered two. I don't believe that you checked
did:web's ability to do or not do that. Or webid." He was right: did:web's
rotation had not been checked. §6 records the check, and D5 the anchor
ruling.
*Outcome:* refines Stack 3 with the maturity principle. WebID becomes ADOPT
as the RDF profile face in the standards survey.

Links: area The stack · amends mere:stack.3 · summary Maturity is not a gate: a draft or community report with a use is supported when cheap, every claim labeled by status; WebID is ADOPT as the profile face.

**D5. Identity anchors.** Asked, with §6's evidence, what should anchor an
identity. Mark: "Add did:webvh beside did:plc", which was offered as:

- did:webvh becomes ADOPT as an anchor once a read-only lane confirms the
  v1.0 spec and the crate;
- did:web stays PULL as its fallback view;
- WebID anchors only when it links to a did:webvh or did:plc.

*Outcome:* applies to Stack 2 (identity comes from the stack). It amends the
standards survey's 2026-09-23 DID grading, pending confirmation.

Links: area The stack · draws on mere:stack.2 · amends standards survey DID grading (2026-09-23) · summary did:webvh joins did:plc as an anchor once confirmed; did:web is its fallback view; WebID anchors through either.

**D6. What stays closed.** Asked, with the §5 measurement, what stays
closed when recognition moves to packs: axes closed with names open, axes
and the six families closed, or everything open with behaviors named by
handler. Mark: "Axes closed, names open (Recommended)", offered as:

- predicates and families are open IRIs declared in packs;
- each declared name is a row of closed-valued axes the engine implements
  (stratum, durability, retractability, projectability, force contribution,
  symmetry, presentation class), compiled as enums;
- behavior dispatches on the axes, never on a name; a row may name a handler
  by id, which activation verifies, as Surfaces 3 already does for
  contributions;
- the doctrine's "owed, the core still compiles as enums" splits: the axes
  compile as enums for good, the predicate enums are the debt.

*Outcome:* refines Stack 3's data rung.

Links: area The stack · amends mere:stack.3 · draws on mere:surfaces.3 · summary Names are open and declared in packs; each is a row of closed axes the core compiles as enums; behavior dispatches on axes or a verified handler, never on a name; the predicate enums are owed to a core pack.

**D7. Typing resources and workspace entities.** Asked how nodes are typed
under the pack model, given that a `ResourceNode` is an IRI with no kind
enum and workspace entities exist only as `ArrangementSubKind` plus
workbench provenance strings: the same model for both, resources by pack
with workspace entities native, or defer to the survey. Mark: "Same model
for both (Recommended)", offered as:

- a type is a class IRI declared in a pack, with its own axes: stratum, what
  it opens into (the Surfaces 3 presentation declaration), lifecycle;
- frame, tile and split become a first-party workbench pack;
- scenes carry the typing, as D2 said.

*Outcome:* applies to Stack 3 as D6 words it (predicates and classes
alike). It feeds the stance rewrite.

Links: area The stack · draws on mere:stack.3, mere:surfaces.3 · summary Resources and workspace entities are typed by class IRIs declared in packs with their own axes; the workbench's entities become a first-party pack; scenes carry the typing.

**D8. What first-party Rust keeps at compile time.** Asked what names
first-party code keeps once the predicate enums go, given 15 non-test
consumers of the packed `u32` tag and rkyv-archived enums in stored data:
axis enums plus build-generated ids, axis enums with IRI string constants,
or a mirror enum regenerated from the pack with a mandatory fallback arm.
Mark: "Axis enums + generated ids (Recommended)", offered as:

- axis value sets are hand-written enums;
- a build step reads the core pack and emits constants for its interned ids,
  so first-party code never matches on a string;
- the stored form is the IRI digest, with the pack digest in the version
  header; stored data migrates by journal replay (graph semantics ruling
  11).

*Outcome:* no doctrine statement; a constraint for the declared-lens plan
that follows the survey.

Links: area The stack · draws on mere:stack.3 · summary Axis enums stay hand-written; the core pack's interned ids are build-generated constants; the stored form is IRI digest plus pack digest, migrated by replay.

D9–D19 are the forks of the §8 survey and the did:webvh lane, put as three
rounds.

**D9. Term identity in packs.** Asked to choose between IRIs, the
`namespace:local` ids that wing-glyphs and impresa use, or both persisted.
Mark: "IRI; CURIEs to author (Recommended)". The IRI is the identity, pack
authors write CURIEs against prefixes the pack declares, and a slug is only
a display label.
*Outcome:* applies to Stack 3.

Links: area The stack · draws on mere:stack.3 · summary A term's identity is its IRI; packs author CURIEs against declared prefixes; slugs are labels.

**D10. The pack envelope and its home.** Asked whether to have one shared
header, one stack-wide manifest, or a header in a new crate, given mere's
`ModManifest`, isocosm's `ContentPackManifest` and eidetic's
`mere.pack/v1`. Mark: "Shared header, in eidetic (Recommended)". The header
carries:

- format, a namespaced id, version, SPDX licence and credit;
- requires and provides, and typed `contributes`;
- scopes and declared files;
- `{id, version, digest}` and an optional signature.

Each product's manifest embeds it and keeps its own sections. It lives
beside `mere.pack/v1`, per domain contract C5, so knot reads the core pack
without depending on mere's graph crates.
*Outcome:* applies to Stack 3.

Links: area The stack · draws on mere:stack.3, domain-contract C5 · summary One shared pack header in eidetic, embedded by each product's manifest; products keep their own sections.

**D11. Persisted relation identity.** Asked to choose between UUIDv5 of the
IRI, IRI strings, or a pack ordinal plus pin. Mark: "UUIDv5 of the IRI
(Recommended)". That is the same scheme as resource keys (graph semantics
ruling 6), fixed at 16 bytes and stable across pack versions.
*Outcome:* applies. It names D8's "IRI digest", and D8's pack digest in the
version header stands.

Links: area The stack · draws on graph-semantics ruling 6 · amends linked-data-platform-lessons D8 · summary Stored relation identity is the UUIDv5 of its IRI, with the pack digest in the version header.

**D12. Families.** Asked whether families are declared and each names a
behavior key, are pure data, or map onto fixed behaviors. Mark: "Declared,
naming a behavior key (Recommended)". The key is satisfied on the ladder,
overlaps are refused unless settled, and the core pack names native keys
for Traversal and Arrangement.
*Outcome:* this sat against D6 and went back as a question. D21 resolves
it: a family *may* name a key.

Links: area The stack · draws on mere:stack.3 · summary Families are pack-declared and may name a behavior key bound on the ladder (refined by D21).

**D13. Core pack delivery.** Asked whether to embed and parse at boot,
generate Rust at build, or ship a file beside the binary. Mark: "Embedded,
parsed at boot (Recommended)". The core pack goes through the same loader,
verification and interning as any pack.
*Outcome:* applies. D8's build-generated id constants are computed from the
same embedded file.

Links: area The stack · draws on mere:stack.3, linked-data-platform-lessons D8 · summary The core pack is embedded and parsed at boot through the common loader; id constants are generated from the same file.

**D14. Scene typing on the wire.** Asked whether scenes carry IRIs plus
pack pins, interned ids plus a vocabulary table, or IRIs only. Mark: "IRIs
+ pack pins (Recommended)". A viewer missing a pack fetches it by pin or
renders generically.
*Outcome:* applies to D2's "scenes carry the typing".

Links: area The stack · draws on linked-data-platform-lessons D2 · summary Scenes carry kinds as IRIs plus the pins of the packs they used.

**D15. Knot's predicates.** Asked whether they stay signed operations,
become packs, or both. Mark: "Both: ops, exportable as packs (Recommended)".
They stay signed operations that the interner also reads, and they can be
exported as a vocabulary pack. The slug mirror goes, and `lowers_to` names a
core-pack IRI.
*Outcome:* applies.

Links: area The stack · plans knot-editor:2026-11-02_predicates_inference_scripting_plan · summary Knot predicates stay signed operations, read by the interner and exportable as packs; the slug mirror goes.

**D16. Capability vocabulary.** Asked whether grants and requirements are
both closed, both open, or split. Mark: "Grants closed, requirements open
(Recommended)". What a host can grant is a stack-owned closed list, so a
pack can request but never invent a privilege. What a pack needs from its
environment is an open set named by IRI.
*Outcome:* applies to Stack 2.

Links: area The stack · draws on mere:stack.2 · summary Grantable capabilities are a closed stack list; pack requirements are open IRIs.

**D17. did:webvh rollback protection.** Asked whether to rely on our
resolver watching with witnesses optional, require witnesses, or also use
third-party watchers. Mark: "1, and i'm ok with 3 if optional to raise the
trust grade as in one".

- Our resolver records the last versionId it saw per anchor and refuses
  rollbacks.
- Witnesses are optional and raise the trust grade.
- Third-party watchers are optional and also raise the grade.

*Outcome:* applies to Stack 2.

Links: area The stack · draws on mere:stack.2 · summary Our resolver refuses did:webvh rollbacks; witnesses and third-party watchers are optional and raise trust.

**D18. did:webvh portability.** Asked whether to mint portable and accept
either, require portable, or ask the user. Mark: "i would imagine that
default/anonymous nonidentities wouldn't be portable? or would they be like
generated identities portable by default and activated by user
config/saving? 1".

Reading: an anonymous or default non-identity is not a did:webvh at all. It
is a `did:key` or nothing, with no domain to move from, as the portable
experience brief keeps persistence and identity independent. A did:webvh is
minted only when a user saves or promotes an identity to a hosted anchor.
That genesis is the one moment `portable` can be set, and we set it. Peers'
non-portable anchors are accepted.
*Outcome:* applies.

Links: area The stack · draws on mere:stack.2 · summary did:webvh anchors are minted portable when a user saves an identity; anonymous identities are did:key or none; non-portable peer anchors are accepted.

**D19. Carrying didwebvh-rs.** Asked whether to use the crate verify-only
with an exact pin, use the full crate pinned, or wait. Mark: "Wait for 1.0".
did:webvh stays ADOPT on paper, and no code is carried until a 1.0 release
or a second maintainer. The upstream findings in Open enter the ledger only
when we carry it.
*Outcome:* applies.

Links: area The stack · summary No didwebvh-rs until 1.0 or a second maintainer; ADOPT stands on paper.

**D20. Signing vocabulary packs.** First asked whether signing is optional,
required or deferred. Mark: "what sort of friction is introduced by 2?" The
answer given:

- day to day it costs almost nothing, since a device key signs on save;
- the real cost is meaning: local signatures turn "Trusted" into "my
  machine touched it";
- the core pack needs a release key, existing packs need migrating, and
  test fixtures need keys.

Put back, Mark: "Optional; required to cross trust (Recommended)". That
means:

- signing is over deterministic CBOR of the parsed form, with eidetic's
  Trusted/Unsigned/Broken verdict;
- local packs may stay unsigned;
- a host's policy requires Trusted for anything shared, published or
  arriving from a peer;
- the core pack is signed with a release key;
- eidetic moves from serde_json to CBOR signing.

*Outcome:* applies to Stack 2. It replaces the stance's "no signing until
federation".

Links: area The stack · draws on mere:stack.2, data-formats rule (CBOR where bytes are signed) · summary Pack signatures are optional locally and required to cross a trust boundary; CBOR-signed with a verdict ladder; the core pack carries a release signature.

**D21. Families: D6 and D12.** Asked which holds for families: axes with an
optional handler (D6), axes plus a mandatory key, or a key only. Mark:
"Let's compare 'em. When we say axes, is there any cost?"

The comparison given: axes already exist for stratum (`GraphStratum`,
built-in in `predicate_registry.rs`, overridable by per-mere
`PredicateDeclaration`s, with 38 call sites outside those files). Their
costs:

- the axis list is itself closed, listing what the engine can do rather
  than what things are called;
- every row must fill every axis;
- combinations need validity rules;
- the frame-rate cost is unmeasured.

Mandatory keys cost a code handler for every family and hide behavior from
hosts and remote viewers.

Put back, Mark: "Axes, optional handler (Recommended)". That means:

- D6 governs families too;
- axes carry the behavior the engine composes;
- a handler is optional and verified when present;
- the core pack supplies axis defaults and the allowed combinations;
- a new axis is a core change.

*Outcome:* refines D12 from "must name a key" to "may name a key". D6
stands.

Links: area The stack · draws on linked-data-platform-lessons D6, mere:surfaces.3 · amends linked-data-platform-lessons D12 · summary Families are rows of closed axes with an optional verified handler; the core pack supplies axis defaults and allowed combinations.

**D22. Pack digest algorithm.** Asked to choose between BLAKE3, SHA-256, or
multihash with BLAKE3 as the default. Mark: "Multihash, BLAKE3 default".
Digests are self-describing, so SHA-256 ones (isocosm's PackPin today, and
did:webvh's own) verify too. The digest is taken over the parsed canonical
form.
*Outcome:* applies to Stack 3.

Links: area The stack · draws on mere:stack.3 · summary Pack digests are multihash over the parsed canonical form, BLAKE3 by default.

**D23. WebID-to-DID link.** Asked whether to use reciprocal `alsoKnownAs`,
accept either predicate if reciprocal, or use `owl:sameAs` in the profile.
Mark: "Reciprocal alsoKnownAs (Recommended)". The DID document names the
WebID and the profile names the DID. The two are linked only when both
directions verify, per Controlled Identifiers 1.0.
*Outcome:* applies to D4 and D5.

Links: area The stack · draws on linked-data-platform-lessons D4, linked-data-platform-lessons D5 · summary WebID and DID link by reciprocal alsoKnownAs, verified both ways.

D24–D30 rule on §9's Datalog pass.

**D24. Rules and entailment.** Asked whether the stance's exclusion of
"OWL, RDFS, entailment" covers declared rules. The options were:

- declared rules yes, entailment no;
- derivation axes only;
- also RDFS and OWL 2 RL, which the question said reverses the stance's
  exclusion.

Mark: "Also RDFS / OWL 2 RL".

*Outcome:* replaces the 2026-05-22 stance's exclusion of entailment. RDFS
and OWL 2 RL entailment are admitted, and every derived fact names its rule
in `basis.method`. The stance rewrite carries this.

Links: area The stack · contradicts statements-over-schema stance (2026-05-22) entailment exclusion · summary Declared rules and the RDFS and OWL 2 RL entailment profiles are admitted; derived facts name their rule.

**D25. Where derived facts live.** Asked whether derived facts are a local
layer plus promotion, computed on read, or written to the log. Mark: "Local
layer + promotion (Recommended)".

- Derived facts live in a local layer that can always be rebuilt, keyed by
  rule digest and log frontier, and never synced. That avoids every
  receiver re-deriving them under C6 and duplicating them under different
  asserters.
- A person or agent can promote one to an assertion carrying `basis`.

*Outcome:* new for the declared-lens plan.

Links: area The stack · draws on domain-contract C6, link-model §6.2 · summary Derived facts live in a rebuildable local layer that never syncs; promotion makes an assertion with basis.

**D26. How rules are declared.** First asked to choose between axes now
with SPARQL-RL reserved, SPARQL-RL now, or scripts only. Mark: "Idk, seems
like we have js, lua, rhai, potentially rune, python… scripting seems
simple to support. What's sparql-rl cost?"

The cost given:

- the language is a one-day-old draft;
- it needs our own rule-level parser on top of `spargebra`;
- runtime rules rule out the macro-based engines;
- OWL 2 RL (D24) needs a runtime rule engine anyway;
- scripts are cheap to host but give no stratification, no static read
  set, no incremental re-runs and nothing a remote host can inspect.

Put back. Mark: "One rule engine, scripts beside (Recommended)". That means:

- derivation axes on pack rows now;
- one runtime engine runs the OWL 2 RL ruleset and pack rules in SPARQL
  1.2 RL, labeled as a draft and implemented in order;
- any hosted script language can feed facts in (D27), but scripts are not
  the rule language.

*Outcome:* new for the declared-lens plan; applies to Stack 3's ladder.

Links: area The stack · draws on mere:stack.3, linked-data-platform-lessons D24 · summary One runtime rule engine for OWL 2 RL and SPARQL-RL pack rules; scripts feed facts but are not the rule language.

**D27. Didgeridoo and rules.** Asked how Lua and rules meet. The options
were:

- Lua feeds facts after determinism fixes;
- Lua called inside rules;
- Lua alone, with the host tracking reads;
- no bridge.

Mark: "Lua feeds facts, after determinism fixes (Recommended)".

- A script runs when its declared read queries change, and emits facts
  into an input relation.
- Didgeridoo first gets a fixed table hasher, `math.random` drawn from the
  host's entropy tape, and a fixed maths library.

*Outcome:* new. The fork changes belong to Didgeridoo, and isocosm-system's
exposed `math.random` is the first consumer to fix.

Links: area The stack · summary Lua feeds facts into rules on declared reads, after Didgeridoo is made deterministic (hasher, random, maths).

**D28. A withdrawn basis.** Asked what happens to a promoted assertion
whose premise is withdrawn: flag it, auto-withdraw it, or auto-withdraw it
only where the rule was the asserter. Mark: "Auto only for
rule-asserted".

Reading under D25: facts the rule itself derived live in the local layer
and vanish with their basis. An assertion a person promoted is flagged
"basis withdrawn" and stays held until its asserter withdraws it.

*Outcome:* applies to the link model's asserter-only withdrawal.

Links: area The stack · draws on link-model §6, linked-data-platform-lessons D25 · summary Rule-derived facts vanish with their basis; promoted assertions are flagged and stay held.

**D29. The rule engine.** Asked to choose between differential-dataflow
with a wasm probe first, a fixpoint loop over spareval, or native per-axis
code only. Mark: "differential-dataflow, wasm probe first (Recommended)".

- Native evaluators stay for each axis.
- differential-dataflow runs pack rules and OWL 2 RL.
- It is adopted only once a probe shows it compiling and running
  single-threaded on wasm.

*Outcome:* new for the declared-lens plan.

Links: area The stack · summary differential-dataflow runs pack rules and OWL 2 RL, gated on a wasm probe; native per-axis evaluators stay.

**D30. Grants and policy.** First asked to choose between native receiver
policy with Biscuit for grants, Biscuit for both, or one shared rule
engine. Mark: "Biscuit for both". Then Mark asked: "Last question: biscuit
or meadowcap?"

The answer given: the two do different jobs.

- Meadowcap carries grants, delegated by narrowing an area. Mark ruled it
  the delegation model on 2026-11-05, in the domain adoption plan.
- Biscuit adds Datalog caveats, but its rules have no negation or
  aggregation, and its 1 ms wall-clock limit breaks C6.
- Domain contract item 4 keeps the native validators as receiver policy.

Put back as a contradiction of both rulings. Mark: "Meadowcap + native +
Biscuit caveats (Recommended)". That means:

- Meadowcap carries grants;
- the native validators stay as receiver admission policy, with contract
  item 4 unchanged;
- Biscuit is used only for conditional caveats an area cannot express
  (moot gates), with `max_time` off.

That is the 2026-05-31 landscape brief's layering.

*Outcome:* applies to Stack 2; the D30 put-back supersedes "Biscuit for
both".

Links: area The stack · draws on mere:stack.2, domain-adoption authority grants (2026-11-05), domain-contract item 4 · summary Meadowcap for grants, native validators for receiver policy, Biscuit only for conditional caveats with no wall-clock limit.

**D31. Axis closure.** Mark asked why the axes had to be closed: "i don't
see why 'closed' is necessary, as opposed to 'observed' or 'utilized'". The
answer given: only behavior is finite, one code path per axis value the core
acts on. "Closed" added a policy on top, that a value outside that set is
refused at activation, which nothing forces and which the open-world stance
the brief imports argues against; the kernel already carries unknown names
in `RelationKind::OpenPredicate`. Put back as observed with a carrier, closed
with refusal, or mixed with stratum and family closed. Mark: "Observed, not
closed (Recommended)", offered as:

- each axis enum is the values the core observes plus a carrier for the
  rest, as `RelationKind` already carries `OpenPredicate`;
- a pack declaring an unobserved value is accepted and stored; Surfaces 3
  verifies that the declaration is well-formed, not that every value is
  known;
- a statement from a newer peer or pack survives on an older core.

*Outcome:* refines D6, and D21's cost "the axis list is itself closed";
Stack 3 is reworded. D8 stands: the axis enums stay hand-written, now with
a carrier variant.

Links: area The stack · amends linked-data-platform-lessons D6, mere:stack.3 · draws on linked-data-platform-lessons D21, mere:surfaces.3 · summary Axes are observed, not closed: each enum is the observed values plus a carrier; an unobserved value is accepted and kept.

**D32. An unobserved value's behavior.** Asked where the behavior for an
axis value the core does not observe comes from: a pack-declared fallback
chain, a per-axis core default, or both. Mark: "Both: chain first, core
default last". That means:

- a declaration may list values for an axis newest first, and the core
  takes the first it observes;
- if it observes none, the axis default the core pack supplies (D21)
  applies;
- which defaults the core pack sets is the declared-lens plan's to work out.

*Outcome:* new for the declared-lens plan; D21's axis defaults become the
chain's last step.

Links: area The stack · draws on linked-data-platform-lessons D21, linked-data-platform-lessons D31 · summary An unobserved axis value degrades by the pack's fallback chain, then the core pack's axis default.

## Open

- **Valid time (§9).** Six forks:
  1. reserve the field now in the new graph grammar, adopt fully now, or
     keep deferring;
  2. place it in `basis` inside the reason key, in `basis` outside it, or
     as a top-level field;
  3. what "current" means: held only, or held and valid now;
  4. the value: EDTF bounds with a UTC bracket and a time reference system
     IRI, a Wikibase-style precision, or eidetic's intervals;
  5. a valid-time axis on predicate rows, or not;
  6. RDF export: OWL-Time on the reifier, or schema.org dates.
- **Content IDs (§9).** Four forks:
  1. adopt CIDv1, or keep plain multihash;
  2. the string form: multibase or `blake3:<hex>`;
  3. replace eidetic's hand-written parsing with the multiformats crates;
  4. the codec for our canonical form: `dag-cbor` or `raw`.
- **Property graphs, tables, spatial values (§9).** Five forks:
  1. a property-graph export: build it, also add GQL, or skip;
  2. naming that export: our own draft shape, or wait for a standard;
  3. CSVW: a subset reader for CSV imports, or skip;
  4. Arrow and Parquet: a PULL row with murm survey tracks as the first
     pilot, or not yet;
  5. a `geometry` value kind on pack axes (CRS84 + GeoJSON), defer, or
     GeoSPARQL wholesale.

  Mark's 2026-09-23 redefinition of PULL (design for it now, implement in
  order) argues against the lane's "not yet" and "defer" recommendations
  for Arrow and spatial values, since named consumers exist.

- **Pack shape (D2).** Settled by D6–D16 and D20–D22 on the §8 survey. What
  remains is the stance doc rewrite ("closed lens" to "declared lenses") and
  a declared-lens plan to carry the work. Neither is written yet.
- **Hot-path cost (D6).** Dispatch on interned ids over the surface graph
  at frame rate is unmeasured. The declared-lens plan carries a probe with
  both controls before the enums go.
- **did:webvh carriage (D5).** Confirmed on 2026-11-06, so ADOPT is recorded
  in the standards survey.
  - **Spec.** v1.0 is a DIF Ratified Specification. Its Rust, Python and
    TypeScript implementations were released on 2025-08-07, and Java and Dart
    ones are now listed.
  - **Threat model.** A host can withhold, roll back or delete the log but
    cannot forge it. Rollback and fork detection rest on witnesses, watchers
    or a resolver's own cache.
  - **Crate.** `didwebvh-rs` 0.8.2 (2026-10-26) is Apache-2.0 under the DIF
    GitHub organisation, but in practice one maintainer writes it (144 of 149
    commits).
    - It makes breaking minor releases roughly monthly.
    - It runs 13 shared vectors and deliberately fails one, `witness-update`,
      where the vector and the spec text disagree.
    - Its crypto comes from `ed25519-dalek` 3 and `sha2`.
    - `tokio` is a required dependency. On wasm it can only resolve.
  - **Linking.** No spec defines a WebID-to-DID link. Controlled Identifiers
    1.0 counts two identifiers as equivalent only when each points back to
    the other.
  - **Settled by D17–D19:** our resolver watches, and witnesses and
    third-party watchers are optional; anchors are minted portable; no crate
    until 1.0. WebID links by reciprocal `alsoKnownAs` (D23).

  The spec-versus-vector disagreement and the domain-validation failures in
  0.5.3 are upstream matters. Nothing is filed; they enter the upstream
  ledger only if we carry the crate.
