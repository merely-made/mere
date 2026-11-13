# Linked-data standards: the engine's stance per standard

**Status:** ruled, 2026-11-12, engine split D142 (Mark: "Accept with those four changes (Recommended)"); the SHACL, ShEx, credential and pack-rule rows refined by D143 the same day.
The draft and its round are in `Code/testing/engine-split/standards_stance_draft.md`. Each row takes
one of four stances:

- **engine:** the `mere` crate implements it as a stage.
- **pack:** a vocabulary or rule set loaded through P6's pack loader.
- **host:** server or protocol work, outside the engine.
- **out:** not supported. The row records that this was decided, not left as a gap.

The engine's charter frames every row: dereference, parse, model, interpret, present through a host,
serve nothing. So consuming a protocol is acquisition, and serving one is a host's job. Security
work uses maintained libraries and published standards only, with no cryptographic code of our own.
Every "engine" row becomes a ledger row when its stage lands. Under D141 (F3), P5 creates only
stages that already have code, so these stances shape P6 onward and the format entry points, not
P5's seam.

| Area | Standard | Stance | Phase | Reason |
|---|---|---|---|---|
| Model | RDF 1.1 Concepts, XSD datatypes, IRIs | engine | P5 | The model itself. |
| Model | RDF 1.2 (triple terms) | engine | as Oxigraph ships it | Follows upstream; no fork (D141). |
| Syntax | Turtle, TriG, N-Triples, N-Quads | engine | P5 | Oxigraph's parsers; the ledger scores them. |
| Syntax | JSON-LD 1.1 (toRdf, fromRdf, expand, compact) | engine | P5 | Ledger rows exist. |
| Syntax | JSON-LD 1.1 Framing | engine | P6 | It is how a host asks for a shaped tree ("present through a host"). |
| Syntax | RDF/XML | engine, parse only | P5 | Much legacy data is RDF/XML; serializing it gains nothing. |
| Syntax | RDFa 1.1, HTML microdata, embedded JSON-LD | engine (acquire, extract) | P6 | Much linked data lives in pages. The host supplies the parse tree and the engine extracts from it. |
| Syntax | Notation3 | out | — | N3 is a rule language, and pack rules are the one rule mechanism. |
| Syntax | CBOR-LD, HDT | out | — | No consumer. The data-formats rule covers binary inside the stack. |
| Query | SPARQL 1.1 Query and result formats | engine | P5 | Ledger rows exist. |
| Query | SPARQL 1.2 | engine | as spareval ships it | Follows upstream. |
| Query | SPARQL 1.1 Update | engine | P6 | The edit path for a person's graph. Needs an asserter for INSERT DATA (D65). |
| Query | Federated Query (SERVICE) | engine (acquire), per-endpoint grant | P6 | A federated query sends bindings from a person's graph to the endpoint, so each remote endpoint needs its own grant, refused by default. A host fetch policy is not enough. |
| Query | SPARQL Protocol and Graph Store Protocol, client side | engine (acquire), per-endpoint grant | P6 | Reading a remote store is dereferencing. The same per-endpoint grant applies, refused by default. |
| Query | SPARQL Protocol, Graph Store Protocol and Service Description, server side | host | — | The engine serves nothing. |
| Query | Entailment regimes | out as such | — | The derived layer materializes RDFS and OWL RL, so queries see entailments without a regime machine. |
| Reasoning | RDFS | engine | P6 | — |
| Reasoning | OWL 2 RL | engine | P6 | Rule-based; it fits the derived layer. |
| Reasoning | OWL 2 EL, QL and DL | out | — | These need full reasoners, and no pack we foresee needs them. |
| Reasoning | RIF, SWRL | out | — | Pack rules are the one rule mechanism. |
| Validation | SHACL Core | engine | P6 | rudof's `shacl` in a crate beside `mere`, on a carried patch (D143). Rows: SHACL 1.2 core and sparql, SHACL 1.0 as a second row. |
| Validation | SHACL-SPARQL (constraints) | engine | P6 | Cheap, since SPARQL is present, and real shapes use it. |
| Validation | SHACL Advanced Features, SPARQL rules (`sh:SPARQLRule`), SHACL 1.2 inference rules | engine, as input formats that lower into pack rules | P6 | Pack rules are SPARQL 1.2 RL (lessons D26, a draft; D143). One rule mechanism remains. Rows: `sparql12-rl` and `shacl12-inference-rules`. |
| Validation | SHACL Advanced Features, functions and triple rules | out unless they lower into pack rules as cheaply | — | Overlaps pack rules. |
| Validation | ShEx | engine if small, on the SHACL library's code | P6 | rudof brings ShEx (D143), but it needs rudof's `QueryRDF` (SPARQL strings); it gets rows only if that is small, decided by the lane. |
| Proofs | RDF Dataset Canonicalization (RDFC-1.0) | engine | P6 | — |
| Proofs | VC Data Model 2.0, Data Integrity (eddsa-rdfc-2022, ecdsa-rdfc-2019), JCS | engine, verify only | P6 | A thin verifier of our own following the spec's algorithms over maintained crypto, with no crypto math of ours (D143). Issuing belongs to custody. |
| Proofs | VC secured with JOSE or COSE (VC-JOSE-COSE), SD-JWT | engine, verify only, later lanes | after P6's close | Under D143's thin-verifier ruling, over maintained JOSE and COSE crates, each behind its rows. |
| Identity | WebID, DID Core, DID resolution (did:key, did:web) | engine | P6 | Already in P6, against custody. |
| Identity | did:webvh | engine (resolve and verify), gated by lessons D19 | P6, once `didwebvh-rs` reaches 1.0 or a second maintainer | ADOPT as an anchor beside did:plc (lessons D5). Our resolver refuses rollbacks, and witnesses and watchers are optional and raise the trust grade (D17). Minting, portable on save, is custody's (D18). |
| Access | Content negotiation and Link headers | engine (acquire) | P5 | Dereferencing needs an Accept policy and Link-header discovery. |
| Access | LDP, Solid Protocol, Solid-OIDC, WAC, ACP | host (server side); engine acquire reads pods | — | Hosts serve pods, and the engine reads them like any dereference. The access-control vocabularies are packs. |
| Access | Linked Data Notifications, ActivityPub | host | — | Delivery protocols. ActivityStreams is a pack. |
| Access | Linked Data Fragments (TPF) | out | — | No consumer. |
| Vocabulary | RDF, RDFS, OWL, XSD terms | engine, built in | P5/P6 | The engine's own vocabulary. |
| Vocabulary | SKOS, PROV-O, DCAT, Dublin Core Terms, schema.org, FOAF, ActivityStreams 2, ODRL, ORG, OWL-Time, SOSA/SSN, VoID, Web Annotation, GeoSPARQL terms | pack | P6 onward | Each lands when a consumer loads it. |
| Vocabulary | GeoSPARQL query functions | out until a consumer | — | A query-function extension with no consumer. |
| Mapping | R2RML, RML, CSV on the Web | out | — | Import tooling, not engine stages. |
