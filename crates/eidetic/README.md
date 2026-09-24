# eidetic

The durable-memory family covering raw bytes, append-only journals, the
container graph, semantic projection, the typed memory lane, and its backends.

**This directory is the portable core.** `muniment`, `chartulary`,
`hagiograph` and eidetic's `fjall`, `https-fetcher` and `iroh-fetcher`
features reach nothing that is
specific to the Mere product. Seven repositories outside this one already
consume `chartulary` and `muniment` on that basis.

**The one remaining Mere edge** is `eidetic`
([`eidetic-core`](eidetic-core)), which uses mere's `identity` crate for
Ed25519 pack signing: 18 lines in
[`eidetic-core/src/pack.rs`](eidetic-core/src/pack.rs) (the `use` at line 39,
`sign_pack` at 130-138, and the two `Ed25519PublicKey` / `Ed25519Signature`
uses inside `verify_pack` at 160-163), all behind the default-on
`pack-signing` feature. Turning that feature off compiles the lane without
the edge. That is the line the core stops at, and it is deliberately
recorded here rather than left to be rediscovered.

`eidetic-search` **is not here.** The tantivy trail index moved to
`crates/intel/eidetic-search` on 2026-09-04 under the platform boundary
plan's P5: it indexes browsing traces and reaches `esp` and `import`, Mere
product concerns, and it is the lexical half of a hybrid recall whose vector
half is `esp::embed`. The package was renamed from `mere-eidetic-search` to `eidetic-search` on
2026-09-18 (lib `eidetic_search` unchanged); turnstone pins it from `mere.git`.

## The substrate crates

Published under their own names. Formerly four sibling repos, merged into one
2026-07-21 and absorbed into this workspace 2026-07-23, histories preserved.

| Directory | Package | Contents |
|---|---|---|
| [`muniment`](muniment) | `muniment` | The storage floor: host-supplied backends, mutable typed slots, immutable content-addressed blobs, and `Journal<T>` append-only histories addressed by `Seq`. Ships memory, redb, zip, and IndexedDB backends. |
| [`chartulary`](chartulary) | `chartulary` | The content-addressed container graph. `Graph<N, E>` on petgraph with capability traits, default `Container`/`Relation` payloads, the `GraphLog` edit spine, runtime `facet` metadata, `content_class`, the `stemma` lineage layer, and the semantic-ring RDF projection at `chartulary::rdf`. |

## The eidetic lane

Mere's owner-scoped local memory, which carried the eidetic name first. Packages
carried a `mere-` prefix until 2026-09-18, when they took the bare `eidetic*`
names; `mere-eidetic` stays frozen at 0.0.2 on crates.io. Each sets a
`[lib] name`, so consumers write `use eidetic::…` either way. The lexical search crate that used to close this table is
`eidetic-search`, now at `crates/intel/eidetic-search`.

| Directory | Package | Lib name | Contents |
|---|---|---|---|
| [`eidetic-core`](eidetic-core) | `eidetic` | `eidetic` | Manifests, typed payloads, schemas, codicils, bundles, packs, sealing, browsing traces, model artifacts. `Store` is an alias for `muniment::Backend`. `FjallStore`, the production-default native backend, sits behind feature `fjall`; `HttpsFetcher` and `IrohFetcher`, the `BlobFetcher`s for `BlobSource::Https` and `BlobSource::Iroh`, sit behind features `https-fetcher` and `iroh-fetcher`. |

## Design docs

All of the family's docs live in the repo-level
[`design_docs/eidetic_docs/`](../../design_docs/eidetic_docs). The per-crate
`design_docs/` directories that used to sit beside `muniment` and
`chartulary` were collapsed there on 2026-08-24; core §4 of
[`DOC_POLICY.md`](../../design_docs/DOC_POLICY.md) forbids reintroducing
them.

## License

MPL-2.0 (see LICENSE).
