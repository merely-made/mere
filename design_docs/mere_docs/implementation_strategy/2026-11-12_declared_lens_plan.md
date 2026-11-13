# Declared-lens plan: packs, lenses and the engine's loader

**Date**: 2026-11-12
**Status**: plan, 2026-11-12; L0 done the same day: Mark ruled M1 to M9, each on its
recommendation (D1 to D9), and the first reader's R1 to R9 and the conflict readings stand as
argued (D10). L1 (the header and envelope) landed on `eidetic` main `c96bea1` the same day; D11 and D12 followed. L2 is next.
**For**: the plan the [linked-data platform lessons brief](../research/2026-11-06_linked_data_platform_lessons_brief.md)
owes. Its Open says of D2: "What remains is the stance doc rewrite ('closed lens' to 'declared
lenses') and a declared-lens plan to carry the work. Neither is written yet." The
[engine split plan](2026-11-07_engine_split_plan.md)'s P6 lists it under Needs ("the declared-lens
plan for the pack format"). Its D143 places it as "a doc lane beside the stages that need no pack,
and its own forks go to Mark (F-M1)".
**Related**: the [standards stance](../technical_architecture/2026-11-12_standards_stance.md)
("vocabularies are packs"), the [data formats brief](../../2026-10-06_data_formats_brief.md), the
[conformance ledger](../testing/2026-11-09_conformance_ledger.md), the
[link model](../technical_architecture/2026-10-15_link_model.md), the
[quad index plan](2026-11-09_quad_index_plan.md), doctrine Stack 2, Stack 3, Stack 8 and Surfaces 3
([doctrine](../../doctrine.md)). The P6 assessment is `Code/testing/engine-split/p6_assessment.md`;
F-M1 there is this plan, and its §4 and §5 give the stage layout and work order this plan fits into.

**Method** (Mark, 2026-11-12): "we need a spec and metrics to evaluate by. If we're implementing an
understood standard we can look at proper references for, that's also good." Every format and
mechanism below names the published spec it follows and the metric that judges it. Where no spec
exists, the plan says so and names the reference designs it compared.

## 1. The rulings this plan carries

Quoted from the lessons brief's Decisions unless marked otherwise.

| Ruling | What it fixes here |
|---|---|
| D2 | "authored TOML vocabulary packs declare predicates, families (workspace ones included), retractability and presentation"; "IRIs are interned to ids when a pack loads"; "today's families and sub-kinds ship as a first-party core pack" |
| D6, D21, D31 | Names are open rows of axes. "behavior dispatches on the axes, never on a name; a row may name a handler by id, which activation verifies". Axes are "observed, not closed": "a pack declaring an unobserved value is accepted and stored" |
| D7 | "a type is a class IRI declared in a pack, with its own axes"; "frame, tile and split become a first-party workbench pack" |
| D8, D11 | "a build step reads the core pack and emits constants for its interned ids"; stored identity is "the UUIDv5 of its IRI, with the pack digest in the version header" |
| D9 | "The IRI is the identity, pack authors write CURIEs against prefixes the pack declares, and a slug is only a display label" |
| D10 | One shared header "in eidetic", carrying "format, a namespaced id, version, SPDX licence and credit; requires and provides, and typed `contributes`; scopes and declared files; `{id, version, digest}` and an optional signature". "It lives beside `mere.pack/v1`, per domain contract C5, so knot reads the core pack without depending on mere's graph crates" |
| D12 (refined by D21) | Families "may name a behavior key"; "overlaps are refused unless settled" |
| D13 | "The core pack goes through the same loader, verification and interning as any pack" |
| D14 | "Scenes carry kinds as IRIs plus the pins of the packs they used" |
| D15, D16 | Knot's predicates stay signed operations, "exportable as packs". "Grantable capabilities are a closed stack list; pack requirements are open IRIs" |
| D20 (its bullets joined) | "signing is over deterministic CBOR of the parsed form, with eidetic's Trusted/Unsigned/Broken verdict; local packs may stay unsigned; a host's policy requires Trusted for anything shared, published or arriving from a peer; the core pack is signed with a release key; eidetic moves from serde_json to CBOR signing" |
| D22 | "Pack digests are multihash over the parsed canonical form, BLAKE3 by default" |
| D24 to D29 | RDFS and OWL 2 RL are admitted, and "every derived fact names its rule in `basis.method`". Derived facts live in a rebuildable local layer "keyed by rule digest and log frontier". "one runtime engine runs the OWL 2 RL ruleset and pack rules in SPARQL 1.2 RL, labeled as a draft". differential-dataflow is gated on a wasm probe |
| D32 | An unobserved value takes "the pack's fallback chain, then the core pack's axis default"; "which defaults the core pack sets is the declared-lens plan's to work out" |
| Engine split P6 (D143 reading) | The thin slice: "load a resource, select a pinned pack, derive one fact, report one shape violation, reopen the same revision with the same results". "Reopen" means "the journal cursor, passed in by the caller". Pack rules are SPARQL 1.2 RL, and "`sh:SPARQLRule` and the SHACL 1.2 inference rules lower into it" |
| Engine split P7 | "the engine carries the terms, their stratum and the axes it observes, and has no behavior for them; the host owns the behavior"; done when "the pack loads through the common loader" and "no engine code names a workspace term" |
| Engine split D14 and receipts F18 | D14 deleted the lens registry; the Open questions add "the pack sense stays with the declared-lens plan". The receipt row: "a lens is a decision bundle over five owners" |
| Data formats rule (F1, F3) | TOML when a person authors; "Identity is never the source text"; CBOR "where bytes cross a trust or ownership boundary", "deterministic, and decoding is strict wherever bytes are signed, content-addressed or validated on the wire" |
| Doctrine Stack 2 | "A product never manufactures keys, grants or trust roots to fill a missing stack configuration" |
| Doctrine Surfaces 3 | "activation verifies the declaration: a mod that registers what it did not declare is refused" |

## 2. What a pack is, and what a lens is

**A term** is a declared name: a predicate, class or family IRI (D9), with one value or one
fallback chain per axis (D6, D21, D31, D32). The axes are those D6 lists: stratum, durability,
retractability, projectability, force contribution, symmetry and presentation class. D26 adds the
derivation axes.

**A pack** is a versioned, content-addressed bundle of declarations, carried under the D10 header.
Its body is one or more of these:
- vocabulary: prefixes and term rows;
- rules: SPARQL 1.2 RL rule sets, or SHACL rules that lower into them;
- shapes: SHACL shapes graphs;
- datasheets;
- referenced parts: wasm components, scripts and assets, which is what `mere.pack/v1` carries today.

**A lens** (a declared lens) is the declared bundle that selects packs and presentation. It holds:
- an ordered list of pack requirements;
- the settlements D12 requires when two packs declare the same name;
- which rule sets and shape sets of the selected packs are active, and over which view;
- the presentation choices per term or class: the presentation-class value, and what a class
  opens into (Surfaces 3, D7).

The engine's stages table calls this the semantic profile: "a versioned user-agent pack plus
admitted vocabulary, shape and rule packs; record what was selected". This plan reads the
user-agent pack as the core pack (D13), much as a browser's user-agent stylesheet sits under page
styles. The loader's output is the profile plus a **selection record**: the lens pin, the resolved
pins and the verdicts.

**Reconciling with F18.** The deleted registry's `LensDefinition` (removed in `4b88b720c`) held:
- `physics: PhysicsProfile`;
- `layout: LayoutMode` and `layout_algorithm_id`;
- `theme: Option<ThemeData>`;
- `filters: Vec<String>`;
- `overlay_descriptor`.

The receipts called that "a decision bundle over five owners" and sent physics to Seiche and theme
to tabard. A declared lens keeps only the decisions the engine owns:
- recognition: which vocabulary is admitted;
- interpretation: which rules and shapes run;
- the presentation class of each term. This is an axis value; how it is drawn is the host's call.

Physics, layout, overlays and theme stay with their owners. A host profile names a lens by its pin
rather than living inside it (fork M2). The old `filters` become the host's selection chain
(lessons §4.5: "saved query or frozen selection, forme arrangement, platen projection, surface").
Whether a lens is itself a pack is fork M1; the rest of this plan assumes the recommendation, a pack
of kind `lens`.

## 3. The pack on the wire

### 3.1 Two forms, one identity

- **Authored form.** A pack is authored as a directory: `pack.toml` holds the header, prefixes and
  term rows, written with CURIEs. `rules/*.rl` holds SPARQL 1.2 RL text, `shapes/*.ttl` holds SHACL
  in Turtle, and `data/*.toml` holds datasheets in Livery's shape. This follows the data formats
  rule: TOML because a person authors it, and "no hand-edited JSON in a pack or config directory".
- **Canonical form.** A pack compiler produces one CBOR document from the parsed authored form. It
  is encoded under RFC 8949 §4.2.1's core deterministic encoding requirements, against a schema
  written in CDDL (RFC 8610). In this form CURIEs are expanded to IRIs, rules are normalized (fork
  M7) and shapes are canonicalized (fork M9). The canonical bytes are what is digested (D22), what
  is signed (D20) and what crosses a boundary. Inside the stack they are stored as the same bytes,
  a muniment blob, so a pack has exactly one encoding of its identity (Conflicts, C6).
- **Strict decoding.** Decode, re-encode deterministically and compare bytes. Unequal bytes are
  refused. This is the data formats rule's "strict wherever bytes are signed, content-addressed or
  validated on the wire". Mere already has the precedent of `p2panda_core::cbor::decode_cbor_strict`
  (`crates/system/pandect/src/personal_sync.rs:26`).
- **Spec and metric.** The specs are RFC 8949 §4.2.1 and RFC 8610. The metrics:
  - byte-identity fixtures: authored, then canonical, then decoded, then re-encoded, all equal;
  - one refusal fixture for each §4.2.1 requirement: a non-shortest integer, an indefinite length,
    unsorted map keys and a duplicate key;
  - a proptest that compiling any permutation of the authored rows yields the same bytes.

  No conformance suite exists for deterministic CBOR. The reference designs compared are COSE's own
  deterministic Sig_structure (RFC 9052 §4.4), CWT (RFC 8392, integer claim keys) and the dCBOR
  Internet-Draft (draft-mcnally-deterministic-cbor; its status was not checked).

### 3.2 The header (D10)

Illustrative CDDL, not final. Fork M3 settles the key style; this sketch uses text keys.

```cddl
pack = {
  "format":      "mere.pack/2",
  "id":          iri,                ; absolute IRI (R4)
  "version":     semver,             ; SemVer 2.0.0
  "kind":        [+ pack-kind],      ; "vocabulary" / "rules" / "shapes" / "lens" / "participant"
  "licence":     tstr,               ; SPDX license expression
  "credit":      [* credit],
  "requires":    [* requirement],    ; C3: packs and environment requirements, typed
  "provides":    [* iri],            ; namespaces whose terms this pack declares
  "contributes": [* contribution],   ; Surfaces 3's typed list
  "scopes":      [* tstr],           ; grants asked for, from the host's closed list (D16)
  "parts":       [* part],           ; declared files
  "body":        body,
}
requirement = { "pack": iri, "version": semver-req, ? "digest": multihash }
            / { "capability": iri }                      ; D16: open requirement IRIs
part = { "name": tstr, "role": tstr, "media": tstr, "digest": multihash, "bytes": uint }
multihash = bstr                                         ; uvarint(code) ++ uvarint(len) ++ digest
```

`{id, version, digest}` is the pin. The digest is taken of the canonical document, so it rides
beside the document: in the envelope, in a requirement, in a selection record. It is never inside
the document.

| Field | Spec | Metric |
|---|---|---|
| `id` | an RFC 3987 IRI. OWL 2's ontology IRI and `owl:versionIRI` (OWL 2 Structural Specification §3) are the RDF reference | an id that is not an absolute IRI is refused (fixture) |
| `version`, `requires.version` | SemVer 2.0.0 (semver.org; a published spec, not a standards body's) | a requirement-resolution table test |
| `licence` | the SPDX license expression syntax (SPDX specification) | parsed and refused if malformed (fixture) |
| `digest` | multihash (an IETF Internet-Draft; labeled by status per D4). BLAKE3 is code 0x1e, SHA-256 is 0x12 (D22) | known-answer digests of fixture packs, under both functions |
| `parts` | no standard. OCI's content descriptor (media type, digest, size) is the reference design copied | a tampered part fails at load (fixture) |

Eidetic already has the multihash type: `HashFn` with BLAKE3 at multicodec `0x1e`
(`eidetic/crates/eidetic-core/src/schema.rs:28-38`). `Hash` holds a function tag and a `[u8; 32]`
digest (`schema.rs:102-104`), and writes `uvarint(code) ++ uvarint(len) ++ digest` (`schema.rs:179-185`).
SHA-256, which D22 also accepts, is one more `HashFn` variant of the same length.

### 3.3 The body, by kind

- **Vocabulary.** The body holds `prefixes`, a CURIE map (W3C CURIE Syntax 1.0, used only in the
  authored form), and `terms`, the rows. Each row holds:
  - `iri`;
  - `kind` (predicate, class or family);
  - `family` (an IRI);
  - `axes`, each an axis IRI mapped to one value IRI or a chain of them, newest first (D32);
  - an optional `handler` id (D21);
  - labels, as `rdfs:label`-style language-tagged strings (D9's slug is a label).

  Axis and value names are IRIs under Mere's existing namespace `https://mere.computer/ns/`
  (`crates/graph/engine/src/model/edge_data.rs:356`), so an unobserved value is still a well-formed
  name that is kept (D31). The persisted relation identity is the UUIDv5 of the IRI (RFC 9562 §5.5,
  the URL namespace). It is derived at load and never stored in the pack. The engine already mints
  UUIDv5 over that namespace (`crates/graph/engine/src/model/resource_content.rs:30`).
- **Rules.** The body holds rule sets. Each carries:
  - its language IRI and dated revision: SPARQL 1.2 RL, a W3C Working Draft (lessons §9: published
    2026-11-05), labeled as a draft (D4, D26);
  - the normalized rule text (fork M7);
  - a per-set rule digest, the key D25 and the P6 reading F-R3 use for the derived layer.

  SHACL rules (`sh:SPARQLRule`, from the 2017 SHACL Advanced Features Working Group Note, and the
  SHACL 1.2 inference rules) can be carried in a shapes graph and lowered at load, in
  `mere::interpret::derive` (D143 F-R9). The RDFS and OWL 2 RL rulesets ship as first-party rules
  packs that a lens switches on (R9). OWL 2 RL is specified as rule tables in OWL 2 Profiles §4.3.
  Whether SPARQL 1.2 RL can express its list rules (for example `prp-spo2` and `cls-int1`) is
  unverified, and the first rules lane checks it.
- **Shapes.** The body holds SHACL shapes graphs as RDF. Their canonical form is fork M9; the
  recommendation is RDFC-1.0 canonical N-Quads (W3C Recommendation), computed by the RDFC stage P6
  builds through `oxrdf`'s `rdfc-10` (D143 F-R7).
- **Lens.** The body holds:
  - `select`: requirements, in order;
  - `settle`: pairs of a term IRI and the pack id that wins it;
  - `rules` and `shapes`: which sets are active, and the declared view the shapes run over (stages
    table: "SHACL against a declared view");
  - `entail`: whether RDFS and OWL 2 RL are on;
  - `present`: a term or class IRI with its presentation class and what it opens into.
- **Participant.** The body holds today's `mere.pack/v1` content, the requested scopes and the part
  inventory (§3.7).

Inline or referenced: terms, rules and shapes are inline in the canonical body, because the engine
interprets them. Wasm, scripts and assets are referenced parts, held as muniment blobs as `mere.pack/v1`
already does (`eidetic/crates/eidetic-core/src/pack.rs:70-82`). The signature covers each part
through its digest (R6).

### 3.4 Signing (D20)

The rulings fix three things:
- signing is optional for local packs and required to cross a trust boundary;
- the signature is over deterministic CBOR;
- the verdict ladder stays as written in `pack.rs`: `Trusted` when the signature verifies, `Unsigned`
  when none is claimed, and `Broken`, refused, when one is claimed and fails (`pack.rs:107-119`).

They do not fix the container, the algorithm identifier or who counts as an acceptable signer.

Recommended (fork M4): **COSE_Sign1** (RFC 9052 §4.2), detached, over the canonical bytes:
- the protected header carries `alg` = -19, Ed25519 under RFC 9864's fully specified identifiers
  (RFC 9864 is Standards Track, October 2025, and deprecates the polymorphic EdDSA, -8);
- `content type` names `mere.pack/2`;
- `kid` is the signer's `did:key`, which Dramatis' `insigne` parses (D143 F-R8);
- several signatures (an author and a release key, say) are several COSE_Sign1 values;
- in an Eidetic codicil the COSE bytes ride in `TrustEnvelope::signatures` as
  `SignatureRef("cose-sign1:<base64url>")`. Today's binding is "versioned by its prefix"
  (`pack.rs:33-36`), so this needs no schema change.

The libraries are `coset` (Google, Apache-2.0; 0.4.2 per the P6 assessment §3.5) for the
structures, and the family's `ed25519-dalek` 3 through personae's `identity` crate, which Eidetic
already takes as `pack-signing` (`eidetic/crates/eidetic-core/Cargo.toml`). No cryptographic code is
ours, per the security rule as D143 refined it.

Metrics:
- RFC 8032 §7.1's Ed25519 vectors, which are upstream's tests;
- the cose-wg Examples repository's EdDSA Sign1 cases, if they are vendored (fork M4; not checked
  by me);
- negative tests of our own: each header field tampered, a part digest swapped, a wrong `kid`, an
  `alg` other than -19, a payload swapped. Each must be `Broken`. A foreign binding stays `Unsigned`,
  as `pack.rs:322-324` already tests;
- measured verify cost for the core pack, recorded.

**Trusted is not accepted.** `pack.rs:29-31` says "`Trusted` means *the signature verifies* —
nothing more". D20's "a host's policy requires Trusted" names no signer. Who counts as acceptable,
including the core pack's release key and where it lives, is fork M5 (Conflicts, C5).

### 3.5 Pinning and dependencies

- **A pin** is `{id, version, digest}`. The lessons brief's §8 survey found isocosm's
  `PackPin {id, version, hash}` and woodshed's `PackIdentity {id, version, digest}` already in this
  shape. I did not re-read those two repositories.
- **A requirement** names a pack id and a version requirement (fork M6; the recommendation is a
  lower bound, as the family's Cargo rows have under D139). The digests the loader resolved go into
  the selection record. Scenes carry those pins (D14), and the derived layer and validation reports
  key on them with the cursor (F-R3).
- **Resolution rules**, each refused with a typed diagnostic:
  - a dependency cycle;
  - two versions of one pack id in a single lens. This is the stack's own posture:
    `multiple-versions = "deny"`;
  - a requirement no available pack satisfies;
  - a name declared by two packs with different rows that the lens does not settle (D12);
  - `Broken` at any origin;
  - `Unsigned` or an unaccepted signer where the origin crosses trust (D20, M5).
- **Spec and metric.** Pack resolution has no standard. The reference designs compared:
  - Cargo's SemVer resolver, and the family's own lower-bound rows plus resolved notes (D139);
  - OCI image manifests, where digests are the authority and tags are advisory;
  - The Update Framework (TUF) specification, for signed metadata and rollback refusal;
  - OWL 2's `owl:imports`, which imports by IRI and has no version requirement.

  The metrics are golden selection records, a proptest that resolution ignores input order, and one
  refusal fixture per rule above. A pack-format row joins the ledger, which already allows "pack
  version" rows (ledger line 5), scored by these fixtures. No W3C suite exists for it.

### 3.6 How rules and shapes are checked when a pack loads

- **Rules.** Each rule set is parsed by the rule-level parser over `spargebra` that lessons D26
  costs, then checked for well-formedness and stratification. It is handed on keyed by its rule
  digest. The metrics are the `data-shapes` checkout's `sparql12-rl` suite: 243 tests, of which 173
  are syntax, 8 well-formedness, 14 stratification and 48 evaluation (P6 assessment §2, C2). The
  first three groups score at load, and evaluation scores with `interpret::derive`. The lowered SHACL
  rules score against `shacl12-inference-rules` (42 `sht:Infer`, 6 Validate).
- **Shapes.** A pack's shapes graph is first validated against SHACL's own shapes for shapes graphs
  (SHACL, the appendix "SHACL Shapes to Validate Shapes Graphs"). This runs through the SHACL crate
  beside `mere` (D143), so a malformed shapes pack is refused with a report rather than
  half-applied. Validation of data is the SHACL rows' job (SHACL 1.2 core 162 and sparql 98; SHACL
  1.0 as a second row).
- **Bound work** (Stack 8). The loader carries limits on pack bytes, term count and rule count, and
  RDFC-1.0 runs on `mere-rdfc` with its budgets on Hash N-Degree Quads calls and permutations (D11;
  a blank-node count cannot refuse the poison clique). `test074c` is the negative control. Each limit is a typed refusal, never a hang.

### 3.7 Migrating from `mere.pack/v1`

The check the P6 assessment asked for, at Eidetic `origin/main` `1371574`, confirms that
`eidetic/crates/eidetic-core/src/pack.rs` (326 lines) is not D10's header:
- `PackManifest` holds `name`, `version`, `author` (a hex Ed25519 key), `requested_scopes` and
  `parts` (`pack.rs:85-99`), under the schema id `"mere.pack/v1"` (`pack.rs:48`);
- it has no namespaced id, no licence or credit, no requires or provides, no `contributes` and no
  digest pin;
- its canonical bytes are `serde_json::to_vec` (`pack.rs:121-125`);
- its binding is `personae:ed25519:<pub>:<sig>` (`pack.rs:50-52, 130-138`).

`v1` is live in three repositories:
- Graphshell's authority guest verifies it (`graphshell/host/ports/mere-resident/src/authority_guest.rs:39, 326-332`);
- Turnstone's blocks port builds it in its tests and an example (`turnstone/ports/graphshell/blocks/src/main.rs:426-447`, inside `mod tests`; `examples/render_view.rs:14`);
- Woodshed's notation crate builds and verifies it (`woodshed/crates/woodshed-notation/tests/block_conformance.rs:10, 59`),
  read at Woodshed `origin/main`.

The authority guest's wire is `SignedPack` decoded with `serde_json`, and its pack address is
BLAKE3 over those JSON bytes (`authority_guest.rs:327, 330`). That is a source-text identity, which
the data formats rule forbids.

The mapping:

| v1 | v2 |
|---|---|
| `name` | a label; `id` is new and becomes an IRI |
| `version` | `version` (SemVer) |
| `author` | the COSE `kid`, as a `did:key` |
| `requested_scopes` | `scopes` |
| `parts` | `parts` (`role`; `ManifestId` becomes a multihash; `bytes`) |
| `canonical_bytes` | the canonical CBOR |

The kind is `participant`. Stored approvals survive: the guest's `Pin` keys on the author key and
the component's BLAKE3 (`authority_guest.rs:267-283`), not on the pack's bytes. The recommendation
(R3) converts the three consumers in lockstep and deletes v1, per DOC_POLICY §3, unless the
conversion lane finds stored v1 packs in real user data.

## 4. The loader's contract in the engine

The module is `mere::profile`, the stage "Load a semantic profile". It sits beside `profile::vocab`,
which is all `crates/graph/engine/src/profile.rs` holds today. That file's doc comment is stale (P6
assessment §1): the predicate registry moved to `crates/graph/engine/src/model/predicate_registry.rs`.

| Module | Job |
|---|---|
| `profile::pack` | the Mere bodies (vocabulary, rules, shapes, lens), parsed from the authored TOML or the canonical CBOR through `eidetic::pack`'s header and codec |
| `profile::resolve` | requirements to pins, with the refusals of §3.5 |
| `profile::intern` | IRIs to dense ids and UUIDv5 identities; axis rows as the observed enums plus a carrier (D31); the fallback chain, then the core default (D32) |
| `profile::lens` | settlements, active sets, presentation choices; the selection record |
| `profile::load` | the entry point below |
| `profile/core/` | the core pack's authored TOML, embedded (D13); `build.rs` emits the id constants (D8) |

The contract is illustrative, not compile-ready:

```rust
pub fn load(lens: PackRef, store: &dyn PackSource, policy: &TrustPolicy, limits: &Limits)
    -> Result<Profile, LoadRefusal>;

pub struct Profile {
    pub selection: SelectionRecord,   // lens pin, resolved pins in order, verdicts
    pub terms: TermTable,             // dense id -> axis row; IRI <-> id; UUIDv5
    pub rules: Vec<RuleSet>,          // keyed by rule digest, for interpret::derive
    pub shapes: Vec<ShapeSet>,        // canonical graphs, for the SHACL crate beside mere
    pub diagnostics: Vec<Diagnostic>, // accepted-with-notes: unobserved values, inactive sets
}
```

The steps, in order:
1. Strict decode.
2. Digest, compared with the pin.
3. COSE verification to a verdict, then policy.
4. Resolution.
5. Declare-and-verify. The header's `contributes` and `provides` must match the body, the Surfaces 3
   rule. Every row needs every axis or a chain. Handler ids are verified (D21). Rules must parse and
   stratify, and shapes must pass SHACL-SHACL.
6. Interning.
7. Hand-off.

The loader does no I/O. `PackSource` is the caller's: the embedded core, a local store or bytes the
host fetched. P5 L6b's gate, no HTTP client in `cargo tree -p mere`, holds. Diagnostics are typed
data returned to the caller (engine-split D26: "engine stages return typed results as data"); domain
policy decides what to do with them.

**Where each piece lives** (fork R1 settles the vocabulary types):

| Piece | Home |
|---|---|
| Header, pin, requirement, part; canonical CBOR codec and strict decode; multihash digest; COSE sign and verify; verdict | Eidetic, the `eidetic` crate (`eidetic/crates/eidetic-core`), module `eidetic::pack`, beside v1 as D10 rules. The Eidetic squish audit already says "D10's pack header splits out of `PackManifest` in `eidetic::pack`" (engine split Open questions). Nothing here goes into chartulary, so none of its pending successors (`portolan`, `hodograph`, `quadstore`, `fieldbook`; Mark, 2026-11-12) is touched. The split's mapping is not recorded at either repository's `origin/main`, so this is unverified |
| The vocabulary and lens body data types, axis values as IRIs, with no engine enums | `eidetic::pack` too, under R1, so Knot reads the core pack without the engine (D10, C5) |
| Resolution, interning, axis enums, lowering, the selection record, the embedded core pack | `mere::profile` |
| Rule evaluation; derived-layer placement | `mere::interpret::derive` (D143). If F-M4's one-index design wins, the quad index changes, and by name that is the pending `quadstore`, inferred from the name only |
| Shapes evaluation | the SHACL crate beside `mere` (D143) |
| Fetching packs, storing them as blobs, install review and grants for `scopes`, the signer-acceptance UI, lens choice, host profiles (physics presets, theme, layout), Graphshell's experience pack (P7) and the workbench pack (D7) | the host, Graphshell |
| The pack tool (`build`, `sign`, `verify`) | an example binary in `eidetic` first (R7); signing keys come from custody (Dramatis), never minted by the tool (Stack 2) |

**How P6 and P7 consume it.**
- P6's thin slice: "select a pinned pack" is `load` over a fixture lens whose resolved pins are
  checked against a golden selection record. "derive one fact" takes `Profile.rules`, and "report
  one shape violation" takes `Profile.shapes`. "Reopen ... with the same results" compares the
  selection record and the results at the same cursor.
- P7: Graphshell's experience pack (traversal, arrangement, containment-as-layout) loads through
  this loader. Its terms are rows whose axes the engine observes, and its behavior keys are host
  handlers (D21). P7's last done-condition, "no engine code names a workspace term", is met when the
  three workspace arms leave `crates/graph/engine/src/model/predicate_registry.rs:76-81` and
  `RelationKind`'s `Traversal` and `Arrangement` variants (`crates/graph/engine/src/model/edge_taxonomy.rs:387-395`).
  Today 22 non-test lines across two engine files name them.

## 5. Phases

No time estimates. Each phase lists what it needs.

### L0. Rulings
Forks M1 to M9 go to Mark as cards, and R1 to R9 to the first reader.
- Done when: each answer is recorded under Decisions as `declared-lens D<n>`, in answer order, in
  the words given, with its outcome. The plan and DOC_README are amended the same turn. The lessons
  brief's Open and engine split P6's Needs line link here.

### L1. Header and envelope in Eidetic
`eidetic::pack` gains `PackHeader`, `Pin`, `Requirement`, `Part`, the canonical codec, strict decode,
the digest (with SHA-256 added to `HashFn`), COSE_Sign1 sign and verify, and the verdict. It lands
in Eidetic first.
- Needs: M3, M4, M5 (for the policy type); R4.
- Done when:
  - the CDDL schema is in the repository, and the §3.1 byte-identity and refusal fixtures pass;
  - the §3.4 negative signature tests pass, along with the Sign1 vectors if M4 vendors them;
  - `cargo check -p eidetic --no-default-features --features pack-signing --target wasm32-unknown-unknown`
    passes;
  - encode, decode and verify costs for a 1,000-term fixture are recorded in the receipt file.
- Done, 2026-11-12 (`eidetic` `c96bea1`): each condition met; the evidence is in the receipt file,
  "L1, the header and envelope".

### L2. Bodies, loader and core pack (no rules or shapes yet)
Covers the vocabulary and lens bodies, `mere::profile::{pack, resolve, intern, lens, load}`, the core
pack in TOML with build-generated constants, and the selection record.
- Needs: L1; M1, M6; R1, R2, R5.
- Done when:
  - the core pack loads through `load` on native and wasm;
  - the pack-derived strata reproduce `built_in_relation_stratum` for every `RelationKind`. That is,
    the existing test `all_recognized_kinds_match_the_settled_placement_table` passes against the
    pack (`crates/graph/engine/src/model/predicate_registry.rs:108`);
  - every id the engine references exists in the core pack (D8's activation check);
  - the §3.5 golden and refusal fixtures pass, and the pack-format ledger row exists with results;
  - `cargo tree -p mere` gains only `eidetic`'s substrate and its dependencies, and they are listed,
    with no HTTP client;
  - boot parse cost is recorded.

### L3. The hot-path probe (lessons Open, D6)
"Dispatch on interned ids over the surface graph at frame rate is unmeasured. The declared-lens plan
carries a probe with both controls before the enums go."
- Needs: L2; M8.
- Done when: axis-row lookup by dense id is measured against today's enum match at 10k and 100k
  edges, headless, the way P5 measured the engine's frame share. The positive control is the enum
  match; the negative control is an empty loop. The result is within M8's threshold, or it comes
  back as a fork. No enum is removed before this lands.

### L4. Rules and shapes in packs
Rule sets parse, stratify and hand off by digest; shapes are canonicalized and checked by SHACL-SHACL;
the first-party RDFS and OWL 2 RL rules packs are written.
- Needs: L2; M7, M9; P6's RDFC, SHACL and derive lanes (P6 assessment §5, L2, L4 and L5).
- Done when:
  - the `sparql12-rl` syntax, well-formedness and stratification groups (173, 8 and 14) score at
    load;
  - evaluation (48) and `shacl12-inference-rules` score through derive;
  - a shapes pack round-trips byte-identical;
  - the poison cases are refused by the Hash N-Degree Quads call budget (D11; a blank-node count cannot refuse `test074c`).

### L5. The thin slice's pack step
- Needs: L2, and L4 for the derive and shape steps.
- Done when: P6's thin-slice test selects its pack through `load` and compares selection records at
  reopen. P6 owns the test; this plan owns the step.

### L6. Migration off v1
Graphshell's authority guest, Turnstone's blocks and Woodshed's notation move to the `participant`
kind, and v1 is deleted.
- Needs: L1; R3.
- Done when: the three consumers' tests pass on v2; the authority guest addresses packs by canonical
  digest; a stored `Pin` still compares `Agree` after conversion; `PACK_SCHEMA` (`mere.pack/v1`) no
  longer appears at Eidetic `origin/main`.

### L7. The core pack replaces the predicate enums
This is the debt D6 and D8 name. The steps:
- `SemanticSubKind`, `EdgeFamily` and the other family enums give way to the generated constants
  and axis rows;
- stored data migrates by journal replay (D8; graph semantics ruling 11);
- `predicate_declarations.rs` stays in the kernel until P8 moves it. Its per-mere declarations
  become a local pack, read in the same record shape (lessons §5).

- Needs: L3 passing, L2.
- Done when:
  - no engine code matches on a predicate enum outside the generated constants;
  - a replay test reopens a pre-migration journal with identical held assertions;
  - the conformance rows hold.

### L8. P7's pack
- Needs: L2; P7 in the engine split plan.
- Done when: P7's own three done-conditions hold. This plan supplies the loader and the workbench
  pack shape (D7).

## 6. Mechanisms, specs and metrics

| Mechanism | Spec | Metric |
|---|---|---|
| Authored form | TOML 1.0 (toml.io); data formats rule | parse fixtures; the "no hand-edited JSON in a pack directory" check |
| Canonical form | RFC 8949 §4.2.1; RFC 8610 CDDL | byte-identity, refusal and permutation fixtures (§3.1) |
| Pack id, term identity | RFC 3987 IRIs; W3C CURIE Syntax 1.0; RFC 9562 UUIDv5 | IRI refusal fixtures; UUIDv5 known answers |
| Version, requirements | SemVer 2.0.0 | resolution table tests |
| Digest | multihash Internet-Draft; BLAKE3 0x1e, SHA-256 0x12 | known-answer digests under both |
| Signature | RFC 9052 COSE_Sign1; RFC 9864 alg -19; RFC 8032 Ed25519 | upstream vectors; Sign1 vectors (M4); our negative tests |
| Licence | SPDX license expressions | malformed-expression fixtures |
| Rules | SPARQL 1.2 RL (W3C Working Draft, labeled); SHACL-AF Note and SHACL 1.2 inference rules (lowered) | `sparql12-rl` (243), `shacl12-inference-rules` (48) |
| Entailment | RDF 1.1 Semantics (RDFS); OWL 2 Profiles §4.3 (RL) | rdf-mt RDFS rows; the OWL 2 RL conformance cases (vendored under F-M5) |
| Shapes | SHACL 1.2 Core and SPARQL; SHACL-SHACL | `shacl12` core 162, sparql 98; a shapes-graph refusal fixture |
| Shapes canonical form | RDFC-1.0 | `rdf-canon` (P6's row); round-trip fixture |
| Resolution, overlaps, lens | none; Cargo, OCI, TUF and `owl:imports` compared | golden selection records; refusal fixtures; an order-independence proptest |
| Hot-path dispatch | none | L3's probe with both controls, against M8 |

## 7. Forks

### Mark's

**M1. Is a lens a pack?**
Evidence:
- D10's header already carries requires, digest and signature, and D14's scenes carry pack pins;
- a lens of kind `lens` gets identity, signing, pinning and loading at no extra cost;
- the deleted `LensDefinition` was a separate record type, and it bundled five owners' decisions.

Options:
- (a) **A pack of kind `lens`, with the same header and loader** (recommended);
- (b) a separate lens record with its own digest and signature;
- (c) no lens object: the host passes a pack list and the engine records it.

**M2. Do host sections ride in the lens?**
Evidence: F18 asked "Engine pack (declared-lens plan) vs host presentation profile". D14 deleted the
registry and kept "the pack sense" here. D10 lets "each product's manifest embed [the header] and
keep its own sections".

Options:
- (a) **A separate host profile, itself a pack, that pins a lens; physics, theme and layout live
  there, and the engine never decodes them** (recommended);
- (b) host sections inside the lens as opaque typed sections the engine passes through;
- (c) no shared host profile, each host keeping its own settings.

**M3. The canonical encoding.**
Evidence: D20 says "deterministic CBOR" but names no profile. The data formats rule wants CBOR
"readable without our schema", which favors text keys. CWT and COSE headers use integer keys.

Options:
- (a) **RFC 8949 §4.2.1 core deterministic encoding, text keys, no tags, no floats in
  identity-bearing fields, CDDL schema, strict decode by re-encode** (recommended);
- (b) the same with integer keys, CWT-style: smaller, but unreadable without the schema;
- (c) the dCBOR Internet-Draft's stricter profile, whose status is unchecked.

**M4. The signature container.**
Evidence: §3.4. Today's binding is a string of our own over JSON (`pack.rs:50-52`). COSE is the IETF
standard for CBOR signatures, and `coset` is maintained.

Options:
- (a) **COSE_Sign1, detached, `alg` -19, `kid` a `did:key`, carried as a `cose-sign1:` binding; the
  cose-wg Sign1 examples vendored under `Code/crates` as the vector suite** (recommended);
- (b) keep the string binding with a new prefix over the CBOR bytes: no standard, the smallest change;
- (c) a VC Data Integrity proof (eddsa-rdfc-2022) over an RDF rendering of the pack: a standard,
  but it makes every pack RDF-canonicalized and depends on P6's verifier.

**M5. Signer acceptance and the release key.**
Evidence: `Trusted` means only that the signature verifies (`pack.rs:29-31`). D20 requires Trusted
to cross trust but names no signer. D20 has the core pack "signed with a release key". Stack 2 bars
a product from manufacturing trust roots. The authority guest already binds an author key per
domain and compares across devices (`authority_guest.rs:267-283, 295-306`).

Options:
- (a) **The release key is held in custody (Dramatis), its public key built into the engine as the
  one built-in accepted signer; any other signer is accepted per pack id by the host's install
  review, which binds the author key the way the authority guest's `Pin` does** (recommended);
- (b) a host-configured trust list only, with no built-in key;
- (c) any Trusted verdict suffices, reading D20 literally.

**M6. How packs require each other.**
Evidence: Code/CLAUDE.md, under D139: "version requirements are lower bounds, never `=x.y.z`", with
the resolved versions kept as notes. D14 and P6's thin slice need exact pins for reproducibility.

Options:
- (a) **lower-bound SemVer requirements in packs; the resolved digests recorded in the selection
  record, which scenes and the derived layer carry** (recommended; it mirrors D139);
- (b) exact digest pins in `requires`: reproducible, but every update to a dependency forces dependents to re-sign;
- (c) both allowed, with a digest optional on a requirement.

**M7. The canonical form of a rule (the rule digest D25 keys on).**
Evidence: "Identity is never the source text". No standard canonical SPARQL serialization exists.
`spargebra` prints its algebra back as SPARQL.

Options:
- (a) **the rule re-serialized from its parsed form by our printer, as UTF-8 text inside the CBOR**
  (recommended);
- (b) our own CBOR encoding of the rule's syntax tree;
- (c) the normalized source text, which breaks the rule.

**M8. The hot-path threshold (L3).** This is a quantity. Evidence: the P5 fold kept every row
"within 1.5%"; the frame budget itself is P8's (D4, D141).

Options:
- (a) **within 5% of the engine's headless frame share at 100k** (recommended);
- (b) within 1.5%, P5's noise band;
- (c) within 10%.

**M9. The canonical form of a shapes graph.**
Evidence: SHACL graphs are full of blank nodes (property shapes, RDF lists). RDFC-1.0 is a
Recommendation, already in the tree, but exponential in the worst case and without RDF 1.2 triple
terms (P6 assessment §3.3). Skolemizing (RDF 1.1 Concepts §3.5's `/.well-known/genid/`) still
needs a canonical labeling to be deterministic.

Options:
- (a) **RDFC-1.0 canonical N-Quads under the loader's blank-node limit; triple terms refused in
  shapes packs until RDFC supports them** (recommended);
- (b) skolemize at compile time with RDFC-assigned labels, so loads need no RDFC (the cost moves to
  authoring);
- (c) carry Turtle text and use RDFC only for the digest.

The pack id's form is not a Mark fork: D9 already rules that identity is an IRI, so it is R4.

### The first reader's

| Fork | Decision argued | Evidence |
|---|---|---|
| R1. Where the vocabulary and lens body types live | In `eidetic::pack` beside the header, as plain data with axis values as IRIs. Interpretation stays in `mere::profile` | D10's reason: "so knot reads the core pack without depending on mere's graph crates". `mere` is now the engine crate, so a body type there would make Knot depend on the engine |
| R2. The core pack's axis defaults (D32) | stratum `Resource`; durability retained until compaction; retractability by the asserter only; projectable; no force contribution; directed; presentation class generic; not derived | Today's fallback is `Resource` (`predicate_registry.rs:97-99`; `predicate_declarations.rs:136-137`). The link model: withdrawal "is allowed to the asserter, per the predicate's retractability", and the record "remains until compaction". Lessons §5: "generic fallback rendering" |
| R3. v1's retirement | Convert the three consumers in lockstep and delete v1 (§3.7) | DOC_POLICY §3: no shims "unless they are needed for real user data". Approvals key on the component and author, not on pack bytes |
| R4. The pack id | An absolute IRI | D9; OWL 2's ontology IRI; scenes already carry IRIs (D14) |
| R5. Module names | `mere::profile::{pack, resolve, intern, lens, load}`, with the core pack under `profile/core/` | The P6 assessment §4 places the loader in `profile`; D141 F3 creates modules only when they have code |
| R6. Inline or referenced | Terms, rules and shapes inline; wasm, scripts and assets referenced by digest | `pack.rs:70-82` already references parts; the engine must interpret the inline kinds |
| R7. The pack tool's home | An example binary in `eidetic`; keys from custody | Stack 2; the tool needs the header crate and nothing of the engine |
| R8. The diagnostics shape | A typed enum with location (pack id, file, term IRI), returned as data | engine-split D26; F-R10 keeps SHACL's own report as RDF |
| R9. RDFS and OWL 2 RL | Ship as first-party rules packs in SPARQL 1.2 RL, switched by the lens's `entail` | D26: one engine for "the OWL 2 RL ruleset and pack rules"; D29's native evaluators are per axis, not per ruleset |

## 8. Conflicts

**C1. Where physics presets went.** The engine split's Open questions: "F18 is closed by D14: the
lens registry is deleted, the physics presets go with the canvas, and the pack sense stays with the
declared-lens plan." The receipt row: "physics tuning to Seiche", and F18 itself: "its physics
tuning belongs to Seiche either way". The canvas is the host's, not Seiche's.
- Reading: D14 deleted the code, so the host canvas holds presets and Seiche supplies the
  integration. Either way physics is outside the lens (M2).

**C2. The core pack's native keys against P7.** D12: "the core pack names native keys for
Traversal and Arrangement" (refined by D21 to "may name"). P7: "Traversal, arrangement and
containment-as-layout become Graphshell's pack over the engine ... and has no behavior for them".
D7 also places "frame, tile and split" in "a first-party workbench pack".
- Reading: P7 is newer and more specific. The core pack does not carry the workspace terms;
  Graphshell's pack does, with host handlers.

**C3. What D10's `requires` means.** D10: "requires and provides, and typed `contributes`". D16:
"What a pack needs from its environment is an open set named by IRI". Pack-to-pack dependencies
appear in no ruling, yet the thin slice and D14 need them.
- Reading (§3.2): `requires` is typed, holding either a pack requirement or a capability IRI.
  `scopes` holds grant requests from D16's closed list.

**C4. Pins against the lock posture.** Code/CLAUDE.md: "Stop pinning. Stop hand pinning. Stop
editing locks." (2026-11-09), with requirements as "lower bounds, never `=x.y.z`". On the other
side: P6's "select a pinned pack", D14's "IRIs + pack pins" and D10's `{id, version, digest}`.
- Reading: build resolution and data identity are different things. M6 (a) gives packs lower-bound
  requirements, with resolved digests recorded beside them. That is the git-note pattern, applied to
  a selection record.

**C5. Trusted against acceptance.** `pack.rs:29-31`: "`Trusted` means *the signature verifies* —
nothing more". D20: "a host's policy requires Trusted for anything shared, published or arriving
from a peer". Taken together, a pack signed by any key crosses trust. This is a gap rather than a
contradiction, and M5 fills it.

**C6. Postcard inside the stack against CBOR identity.** The data formats rule: "postcard for bytes
that stay inside the stack: storage, snapshots, hashes". D20 and D22: signing and digest over
deterministic CBOR.
- Reading: a pack's identity is its CBOR, because a pack crosses trust. Storing those bytes is not
  a second encoding. Derived caches the engine keeps, such as an interned table, are postcard behind
  `mere-framing`'s header.

**C7. A loader in the engine, a body Knot can read.** D10 puts the header in Eidetic so Knot avoids
Mere's graph crates. The stages table puts the "pack loader new" in the engine, and D143's order
has the engine build it. These agree under R1's reading: the data types are in Eidetic, and
interpretation is in `mere::profile`. If R1 is overturned, this becomes a real conflict.

**Code against a ruling (not a ruling conflict).** The authority guest addresses packs by BLAKE3
over their JSON bytes (`authority_guest.rs:327`), against "Identity is never the source text". L6
removes it.

## Decisions

- **D1. A lens is a pack of kind `lens` (Mark, 2026-11-12):** M1. Ruling: "A pack of kind lens (Recommended)". A lens carries D10's header and goes through the same loader as every pack, so it gets identity, signing, pinning and loading with no record type of its own; the core pack is the user-agent lens. Outcome: new.

  Links: area The stack · draws on linked-data-platform-lessons D2, linked-data-platform-lessons D10 · plans engine-split P6 · summary A declared lens is a pack of kind lens, with the shared header and loader.

- **D2. Physics, theme and layout live in a host profile that pins a lens (Mark, 2026-11-12):** M2. Ruling: "Host profile pins a lens (Recommended)". The host profile is itself a pack; the engine never decodes it. This settles F18's "engine pack vs host presentation profile" and the plan's C1: physics is outside the lens whichever owner holds its presets. Outcome: new.

  Links: area The stack · draws on linked-data-platform-lessons D2, linked-data-platform-lessons D10, engine-split D14 · plans engine-split P6 · summary A host-profile pack carries physics, theme and layout and pins a lens; the engine never reads it.

- **D3. The release key is the one built-in signer; others are accepted per pack by install review (Mark, 2026-11-12):** M5. Ruling: "Release key + install review (Recommended)". The release key is held in custody (Dramatis) and its public key is built into the engine; any other signer is accepted per pack id through the host's install review, which pins the author key the way the authority guest's `Pin` does. This fills the gap between `pack.rs`'s "`Trusted` means *the signature verifies* — nothing more" and D20's "requires Trusted" (C5). Outcome: refines linked-data-platform-lessons D20.

  Links: area The stack · draws on linked-data-platform-lessons D20 · plans engine-split P6 · summary Only the custody-held release key is trusted by default; any other pack signer is accepted per pack at install review.

- **D4. Packs require each other by lower bound, and the resolved digests are recorded (Mark, 2026-11-12):** M6. Ruling: "Lower bounds + recorded digests (Recommended)". A pack's requirements are lower-bound SemVer; the exact digests a load resolved go into a selection record that scenes and the derived layer carry, the git-note pattern of D139 applied to data. This reconciles "stop pinning" with "select a pinned pack" (C4) and makes D10's `requires` typed, a pack requirement or a capability IRI (C3). Outcome: applies engine-split D139.

  Links: area The stack · draws on engine-split D139, linked-data-platform-lessons D10, linked-data-platform-lessons D16 · plans engine-split P6 · summary Pack requirements are lower bounds; the digests a load resolved are recorded in a selection record.

- **D5. A pack's canonical bytes are RFC 8949 §4.2.1 deterministic CBOR with text keys (Mark, 2026-11-12):** M3. Ruling: "§4.2.1, text keys (Recommended)". Core deterministic encoding, text keys, no tags, no floats in identity-bearing fields, a CDDL schema (RFC 8610), and strict decoding by re-encoding and comparing bytes. Metric: byte-identity round trips and strict-decode refusals. Outcome: refines linked-data-platform-lessons D20.

  Links: area The stack · draws on linked-data-platform-lessons D20 · plans engine-split P6 · summary Packs are canonical RFC 8949 deterministic CBOR with text keys and a CDDL schema, decoded strictly.

- **D6. Packs are signed with a detached COSE_Sign1 (Mark, 2026-11-12):** M4. Ruling: "COSE_Sign1 (Recommended)". RFC 9052 COSE_Sign1, detached, algorithm -19 (Ed25519, RFC 9864), `kid` a `did:key`, built on `coset` and the family's `ed25519-dalek` 3, carried in eidetic's trust envelope as a `cose-sign1:` binding; the cose-wg Sign1 examples are vendored under `Code/crates` as its vector suite. Outcome: refines linked-data-platform-lessons D20.

  Links: area The stack · draws on linked-data-platform-lessons D20 · plans engine-split P6 · summary Pack signatures are detached COSE_Sign1 with Ed25519, judged against the cose-wg examples.

- **D7. A rule's canonical form is its parsed form printed back (Mark, 2026-11-12):** M7. Ruling: "Re-serialized parsed form (Recommended)". The rule is parsed and printed with `spargebra`'s printer, and that UTF-8 text inside the CBOR is what the rule digest (lessons D25) covers. A printer change across `spargebra` versions moves digests and costs a derived-layer rebuild, which is accepted because that layer is always rebuildable. Outcome: applies linked-data-platform-lessons D25.

  Links: area The stack · draws on linked-data-platform-lessons D25, engine-split D143 · plans engine-split P6 · summary Rule digests cover the rule as spargebra prints its parsed form.

- **D8. A shapes graph's canonical form is RDFC-1.0 under a blank-node limit (Mark, 2026-11-12):** M9. Ruling: "RDFC-1.0 with a limit (Recommended)". Canonical N-Quads by RDFC-1.0 (`oxrdf`'s `rdfc-10`, engine-split D143 F-R7), under the loader's blank-node limit; RDF 1.2 triple terms are refused in shapes packs until RDFC supports them. Metric: the `rdf-canon` suite. Outcome: applies engine-split D143.

  Links: area The stack · draws on engine-split D143 · plans engine-split P6 · summary Shapes-graph digests are RDFC-1.0 canonical N-Quads, with a blank-node limit and no triple terms yet.

- **D9. The pack path may cost up to 5% of the headless frame share (Mark, 2026-11-12):** M8, a quantity. Ruling: "Within 5% (Recommended)", against 1.5% (P5's noise band) and 10%. L3's probe compares the enum path and the pack path in one run at 100k nodes; the core pack replaces the predicate enums only within that bound. Outcome: new.

  Links: area The stack · draws on engine-split D141 · plans engine-split P6 · summary Pack-driven predicate lookup may cost at most 5% of the engine's headless frame share at 100k nodes.

- **D10. The first reader's forks and the conflict readings (first reader, 2026-11-12, listed to Mark):** Decided as argued in Forks, "The first reader's": R1, the body types live in `eidetic::pack` beside the header, so Knot reads the core pack without the engine (which also settles C7); R2, the core pack's axis defaults; R3, the three v1 consumers convert in lockstep and v1 is deleted; R4, a pack id is an absolute IRI; R5, the loader's modules are `mere::profile::{pack, resolve, intern, lens, load}`; R6, terms, rules and shapes inline, wasm, scripts and assets by digest; R7, the pack tool starts as an example binary in eidetic with keys from custody; R8, diagnostics are a typed enum with a location; R9, RDFS and OWL 2 RL ship as first-party rules packs. The conflict readings stand as written in Conflicts: C2 (the workspace terms go to Graphshell's pack, not the core pack, since P7 is newer and more specific) and C6 (a pack's identity is its CBOR; engine-side caches are postcard behind `mere-framing`). Eidetic's chartulary split landed after the plan was read (Eidetic `33e0c9a`: `portolan`, `hodograph`, `quadstore`, `fieldbook`); the plan puts nothing in those crates, so the split does not move it. Outcome: refines linked-data-platform-lessons D10, D12, D20; applies engine-split D143.

  Links: area The stack · draws on linked-data-platform-lessons D10, linked-data-platform-lessons D12, linked-data-platform-lessons D20, engine-split D143 · plans engine-split P6 · summary The pack body types live in eidetic::pack, the loader in mere::profile, RDFS and OWL RL ship as rules packs, and v1 is converted in lockstep and deleted.

- **D11. RDFC-1.0's poison limit is a carried patch on the more compliant implementation, with nothing sent upstream (Mark, 2026-11-12):** engine split P6 L2 found that `oxrdf` 0.3.5's `rdfc-10` passes the `rdf-canon` suite's 64 eval and 21 map tests but has no work limit: the poison test `test074c`, a 10-node blank-node clique, ran past 2 CPU-minutes, and calling `oxrdf` directly does the same. A blank-node count cannot refuse it, since positive inputs reach 19 blank nodes, so D8's "under the loader's blank-node limit" does not hold. The spec's own mitigation is a budget on calls to Hash N-Degree Quads. Asked whether to patch and offer the patch upstream: "Dude. My priority is not bothering upstreams with fixes generated by ai. Patch, but presume I'm gonna say no; it's just a pita for overworked people. Why would i add to their suffering." (Oxigraph's CONTRIBUTING.md, read afterwards, asks contributors for "significant added value beyond the tool's output" and an issue before significant work.) The patch turned out larger than a counter: `oxrdf`'s RDFC code is about 760 lines over its private interner (about 570 more), while zkp-ld's `rdf-canon` 0.15.3 already has the call limit but sits on `oxrdf` 0.2 and `sha2` 0.10. Asked which to carry: "Take and improve the more spec compliant of the two on the whole (beyond this one feature)". So the two are compared against the whole RDFC-1.0 spec (`Code/testing/engine-split/rdfc_comparison.md`), the winner is carried as a crate of our own with provenance and its licence, improved where the spec says (the call budget, the dependency versions, any gap found), and judged by the `rdf-canon` suite including its negative test. D8's mechanism becomes the call budget; its canonical form stands. Outcome: refines declared-lens D8.

  Links: area The stack · draws on declared-lens D8, engine-split D143 · plans engine-split P6 · summary RDFC-1.0 runs on whichever of oxrdf's and rdf-canon's implementations is more spec compliant, carried and improved by us with the spec's call budget; nothing goes upstream.

- **D12. The header's format details left open by the plan (first reader, 2026-11-12, listed to Mark):** the L1 lane fixed six details where the plan's CDDL was illustrative, each free to change while no `mere.pack/2` pack exists: `credit` is a list of attribution lines in the author's order; a contribution is `{kind, id}` with `kind` one of the four in Surfaces 3 (presentation, pane-adapter, pane, servitor); `body` has one entry per declared kind, holding plain data with no floats, tags or null and integers within int64, with typed bodies left to L2; `host-profile` is a sixth kind, since D2 makes the host profile a pack; a requirement carries no digest and its `version` is a bare SemVer lower bound (D4); set-valued arrays are sorted by their encoded bytes and refuse duplicates, while `credit` keeps its order. Also decided there: `kid`, `alg` and the content type `mere.pack/2` sit in the protected header; the envelope is tagged (18) and untagged ones are refused; any `crit` is refused; every claimed signature must verify; a local pack signed by an unaccepted signer is admitted as Local; the release key is a field on `TrustPolicy`, and eidetic holds no key. Outcome: refines declared-lens D5, D6.

  Links: area The stack · draws on declared-lens D2, declared-lens D4, declared-lens D5, declared-lens D6 · plans engine-split P6 · summary The mere.pack/2 header fixes credit order, contribution kinds, plain-data bodies, a host-profile kind and sorted sets, with kid and alg protected and untagged or crit envelopes refused.

## Findings

- 2026-11-12: read at Mere `origin/main` `48fbe3d8d`, Eidetic `1371574`, Graphshell `a377c352`,
  Turnstone `0a4636ae`. Woodshed was read only for its v1 call sites. No Seiche checkout exists under
  `Code/repos` or `Code/crates`, so Seiche was not read.
- `eidetic/crates/eidetic-core/src/pack.rs` is 326 lines and is the participant `mere.pack/v1`,
  signed over `serde_json`: not D10's header (§3.7). The assessment's path `eidetic-core/src/pack.rs`
  is under `crates/` at `origin/main`.
- The engine's `profile` stage holds only `vocab` (`crates/graph/engine/src/profile.rs`, 13 lines).
  Its doc comment still says the registry is in `mere-kernel`.
- `PredicateDeclaration` holds `declaration_id`, `stratum`, `author` and `declared_at_ms`, under the
  facet `semantic.predicate-declarations/v1`
  (`crates/graph/graph-kernel/src/graph/predicate_declarations.rs:20-31`). It is in the kernel, not
  the engine.
- The deleted lens registry's `LensDefinition` fields are listed in §2 (`4b88b720c^:crates/system/registry/src/lens/registry.rs:17-25`).
- Mere's CBOR today: `ciborium` in pandect (`crates/system/pandect/Cargo.toml:42`), strict decoding
  through p2panda (`personal_sync.rs:26`). Mere has no COSE crate.
- The chartulary split names (`portolan`, `hodograph`, `quadstore`, `fieldbook`) appear nowhere at
  Eidetic or Mere `origin/main`, so the mapping of pieces to them is unverified.

## Progress

- 2026-11-12: plan written on branch `lens-plan`; forks open; nothing built.
- 2026-11-12: L0 done. Mark ruled M1 to M9 in two card rounds and a third for the quantity, every one on its recommendation (D1 to D9); the first reader's forks and conflict readings recorded as D10. The lessons brief's Open and engine split P6's Needs line link here. Landed on Mere main.
- 2026-11-12: L1 landed on `eidetic` main `c96bea1`, "Pack header mere.pack/2 and its COSE_Sign1 envelope". `PackHeader` with its CDDL (`crates/eidetic-core/src/pack/mere-pack-2.cddl`), the strict canonical codec, multihash digests (BLAKE3, SHA-256), detached COSE_Sign1 sign and verify behind `pack-signing`, the verdict, and `TrustPolicy::admit` for D3; `mere.pack/1` unchanged. The cose-wg Examples are vendored at `Code/crates/cose-examples` (`53c9d63`); four Ed25519 Sign1 examples verify and `eddsa-sig-01` re-signs byte for byte. Costs and test counts are in the receipt file. Four muniment `redb_backend::upgrade_tests` fail on Windows with "Access is denied" identically at untouched `origin/main`, outside this work. D11 (RDFC's poison limit) and D12 (the header's open format details) recorded.
