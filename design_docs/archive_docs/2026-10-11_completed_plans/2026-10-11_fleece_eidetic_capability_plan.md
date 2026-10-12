# Fleece gathering and Eidetic intake

**Date:** 2026-10-11
**Status (2026-10-11):** landed. All five dependency shuffles and the portable
Turnstone consumer integration are qualified below.

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
- `cargo test -p mere-fetch --no-default-features --features https-blobs,iroh-blobs
  --lib --offline -j 2`: 25 passed after the retry-free client change, including
  seven streaming crawler tests and HTTPS/Iroh adapters. Source review found
  Hyper's hidden canceled pooled-request retry; the default crawler adapter now
  constructs a separate client with retries explicitly disabled and explicit
  TLS trust. Pooled-cancellation race coverage is not claimed.

- `cargo check -p eidetic -p mere-fleece --no-default-features --features
  mere-fleece/clip,mere-fleece/eidetic-bridge,mere-fleece/structured-data
  --target wasm32-unknown-unknown --lib --offline -j 2`: passed. Native HTTP
  crawling is cfg-gated; the pure traversal runtime remains portable.

- `cargo test -p mere -p pandect -p mere-alembic -p mere-athanor -p mere-moot
  -p mere-document-lanes -p eidetic --no-default-features --features
  mere/clip,mere-moot/captured-web,eidetic/json-schema,mere-alembic/recall
  --lib --offline -j 2`: 493 passed. A stale surface-only clip relation assertion
  now checks the public resource projection. Athanor's tagged-retention failures
  exposed Alembic's old raw-tag read; `node_content_tags` preserves the existing
  tag-promotion rule and physics-pin independence. No new retention policy.
- Capture-only consumer gate, without clipping: `cargo test -p pandect
  -p mere-moot -p mere-document-lanes --no-default-features --features
  mere-moot/captured-web --lib --offline -j 2`: 309 passed after declaring capture's
  explicit URL dependency.

- Portable Turnstone metadata gate: `python scripts/cargo_mode.py verify
  --metadata-only` passed, 1,291 packages, zero external path overrides. The
  source-family audit resolved 75 Mere packages at `5758724e`, five Knot packages
  at `e2bc47cf`, six Woodshed packages at `3c8e4edb`, 30 Genet packages at
  `7422e906` and four Dramatis packages at `c1d62076`, one revision per family.
  All 1,444 registry package/version/source entries match the prior lock.
  Portable lock SHA256:
  `1505845c225549367fb93ef9c4c0cdd15a5b1e86a1fb84198ae09b06273b5cd2`.
- `cargo check --lib --tests --locked --offline -j 2` in the primary Turnstone
  checkout: passed using `C:/t/cargo-targets/turnstone`, after the final source
  migration and formatting corrections. This qualifies the library and test
  targets against the portable source family above; it does not execute
  Turnstone tests. Cargo emits a discovery diagnostic for Servo's intentionally
  invalid tidy fixture, but the locked update, metadata and check commands all
  complete with exit status zero.

The compatible Knot and Woodshed commits were based on Turnstone's prior pins
in clean worktrees because both primary checkouts had concurrent work. Their
branches preserve those changes and keep the qualified commits reachable. Mere's
primary main subsequently accepted a concurrent custody/RC dependency lane;
these source receipts and Turnstone pins qualify `5758724e`, before that lane.

These receipts do not establish headed product acceptance or an implemented
inference model. The deferred Eidetic plan retains the concrete consumer and
pooled-cancellation race test follow-through before this plan is archived.


## Completion (2026-10-11)

All approved moves are implemented and pushed. Source-qualified consumer pins
avoid pulling concurrent Knot RC and Woodshed transcript work into this pass.
The open query/model consumer, headed acceptance and deterministic HTTP pool
race test were extracted into the deferred Eidetic plan before archival.
The task reused stable Mere and Turnstone targets; no isolated Cargo home was
created. Turnstone's consumer migration is committed and pushed as
`1034fb64b133022926bb4c7135b2b3281d09cc3b`. Both clean owned Knot/Woodshed
worktrees were removed; their local and pushed `fleece-integration` branches
remain for exact consumer-pin reachability. Concurrent primary work is preserved.

The completion pass repaired active links to relocated clip/Fjall files and
marked Micron's old import-path log citation as historical. Missing Exhibit A
headers in seven new owned sources were filled; `relicense_headers.py --check`
passes for the complete owned source tree. These final source edits are comments.
The D2 audit remains at its existing 252/259 active-path baseline: seven
unrelated missing records and a duplicate README basename finding. The scoped
judgments and completed checkpoint are recorded in batch 66; this task leaves no
new missing active judgment.
