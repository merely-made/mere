# Fleece gathering and Eidetic intake

**Date:** 2026-10-11
**Status:** in progress. Mark approved the five dependency shuffles and integration.

## Accepted direction

Fleece clips, crawls and captures content. Crawling is bounded, goal-directed
traversal over supplied content, graph relationships and engine-generated
results. Web pages are one source adapter. Ordinary browsing does not initiate
crawling. Network acquisition has explicit request, per-response and total-byte
budgets, bounded depth/fan-out, cancellation and site politeness. Local traversal
requires no network adapter. Inference proposes useful next steps through an
injected selection seam; Fleece enforces configured limits. Model selection and
autonomous background scraping are outside this shuffle.

Eidetic is the durable typed-content family. Import becomes its intake and
interchange capability; bookmarks/history are source adapters. Chartulary/kernel
own graph identities and mutations. Pandect retains session scope, journals,
write authority and live-view state. Muniment owns storage mechanisms. This
inherits family-composition brief section 8's primary graph model, clips and live
entities. It does not reopen GC or destruction-order semantics.

## Dependency moves

1. Keep Genet's published `fleece` extractor and dependency witness intact. Add
   Mere's `mere-fleece` library (`fleece`) over dependency alias `fleece-extract`.
   Move crawl into its modules, separating generic gathering from graph binding.
2. Move portable import models/parsers/history lowering into `eidetic::import`;
   clipping into Fleece; graph insertion into Mere's adapter; Knot formatting
   into its consumer. Remove the former import/crawl packages after migration.
3. Move shared Fleece-to-Eidetic evidence/annotation assembly and JSON-LD source
   projection from document-lanes into optional Fleece modules. Preserve every
   existing serialized schema identity and capture-evidence field.
4. Move store-neutral content caching and facet-schema validation from Pandect
   to Eidetic. Generic image bytes use the durable-content floor; graph/image
   references, retention policy, graph thaw and session authority stay local.
5. Move Fjall's Backend implementation to Muniment and concrete HTTPS/Iroh
   BlobFetcher implementations to fetch/transport modules. Eidetic retains the
   manifest/fetch contract and content-hash verification.

Components are modules in their named capability; no new leaf-component crates.
Optional integrations preserve the lightweight extraction and intake cones.

## Lane ownership

- Traversal lane: `fleece/src/traversal*`, `crawl*`, former `crates/crawl/**`.
- Intake lane: `eidetic-core/src/import/**`, former `crates/import/**`,
  `fleece/src/clip*`, Mere clip adapter and Turnstone Knot formatting.
- Backend lane: Fjall/HTTPS/Iroh source modules and direct consumer paths.
- Parent: manifests/module declarations, capture bridge, Pandect stores, docs,
  source pins, dependency review and serial integration gates.

Use shared main with disjoint ownership. Preserve concurrent Graphshell Forme,
Genet Forms and Turnstone CI work. Build only in stable
`C:/t/cargo-targets/mere` and `C:/t/cargo-targets/turnstone`. Genet needs no source
changes. The parent serializes Cargo resolution and builds.

## Done conditions

- Engine/local traversal honors goal selection and finite bounds with no fetcher.
- Network traversal enforces request/byte exhaustion, cancellation and existing
  robots/politeness behavior; ordinary browsing starts no crawl.
- Eidetic intake parses bookmarks/history without Fleece, render, kernel or
  application-session dependencies.
- Fleece clips and annotations save/reopen with unchanged identities/evidence.
- Eidetic's ordinary cone excludes concrete database/network implementations;
  Muniment Fjall and fetch adapters pass their existing behavior tests.
- Pandect keeps journal/write authority; relocated stores retain their exact
  keys and schema identities, with store/reopen and graph-adapter tests passing.
- In-workspace consumers and the forcing Turnstone consumer compile against
  integrated sources. Record native tests, wasm checks and external gaps
  separately; automated checks do not establish headed acceptance.

## Findings

2026-10-11: Mere `3d21c352` has unrelated Graphshell edits; Genet `cca45fc7a5c`
has concurrent Forms edits; Turnstone has untracked `.github/`. Import's renderer
and graph dependencies come from web_clip. Crawl injects fetch/process callbacks
but binds gathering to GraphContribution/GraphId. Document-lanes owns the shared
capture bridge; Pandect owns content/image stores and facet-schema validation;
Fjall implements Muniment's Backend while housed in Eidetic. The newer
resource/surface split remains authoritative.

## Progress

- 2026-10-11: accepted scope and disjoint ownership recorded before implementation.

- 2026-10-11: all five source moves implemented. Existing annotation schema bytes,
  `content/` cache keys, `content/image/` keys and Fjall's `eidetic` partition
  retained. Genet's extractor and its exact source revision are unchanged.
- Independent source review found and fixed terminal cancellation after a failed
  observation/request, shorter paths discovered after goal-ranked deep visits,
  and incomplete byte evidence when an in-flight request is dropped. Shorter
  paths may reobserve a supplied node; those attempts consume the same node budget.

## Verification

Native automated receipts on 2026-10-11, using `C:/t/cargo-targets/mere`:

- `cargo test -p mere-fleece --all-features --lib --offline -j 2`: 55 passed,
  including engine/local goals, bounded network acquisition, clip lowering,
  graph binding, offline JSON-LD and annotation save/reopen.
- `cargo test -p eidetic -p muniment --no-default-features --features muniment/fjall
  --offline -j 2`: Eidetic 124 passed, Muniment 54 passed, Fjall manifest reopen
  integration 1 passed. Two preexisting JSON Schema tests assumed a disabled
  feature; they now run with that feature, while a lightweight-build test proves
  the unavailable-validator error.
- Normal dependency trees with defaults disabled: Eidetic 149 lines, Mere Fleece
  34 lines. Neither contains Pandect, graph-kernel, rendering, Fetch, Fjall or
  Iroh; Eidetic also excludes Fleece and Inker. Dev dependencies are separate.

- `cargo test -p mere-fleece --all-features --test eidetic_peer_transfer
  --offline -j 2`: 1 passed, including authenticated transfer and receiver-store
  reopen with retained source evidence.
- Turnstone `python scripts/cargo_mode.py local check --lib --offline -j 2`:
  passed against the current Mere checkout and unchanged pinned Genet/Knot
  sources, using `C:/t/cargo-targets/turnstone`. The portable lock was unchanged.
  Knot still consumes Pandect's published schema helper names, so forwarding
  reexports preserve that API while Eidetic owns their implementation.
- The initial concrete Fetch gate passed 25 tests. Source review then found
  Hyper's hidden canceled pooled-request retry; the default crawler adapter now
  constructs a separate client with retries explicitly disabled and explicit
  TLS trust. Its rerun is pending; pooled-cancellation race coverage is not claimed.

Consumer aggregate, retry-free Fetch and wasm gates are running. Exact portable
consumer pins still require integration. These receipts do not establish headed
product acceptance or an implemented inference model.
