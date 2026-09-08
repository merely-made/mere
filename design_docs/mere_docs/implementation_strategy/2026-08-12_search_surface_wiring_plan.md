# Search Surface Wiring Plan

**Date**: 2026-08-12
**Status**: open. W4's lexical n-gram input probe, W5's deterministic fusion
probe, and the V3 tokenized-URL repair completed 2026-08-31. Turnstone's live
trail-fusion caller and private captured-trail evaluation harness are
implemented on paired feature branches as of 2026-09-02. W4 canvas host wiring
remains open. W5 has an executable promotion gate, but the active profile does
not yet admit a real training/held-out selection. Spun out of the
[leverage census](../../2026-08-10_leverage_census_brief.md) (step 2), and
carries the census's audit answer for `mere-embed` inside it.

**Related**: eidetic-search's own crate docs (Phase 9, producer half), the
esp consolidation plan (the split that left embed's glue behind), the
2026-05-08 local-intelligence integration research (the architectural
anchor embed's lib.rs cites).

## 1. Audit results (verified 2026-08-12)

- **`mere-embed` is not a husk; the census's "retire" branch is closed.**
  It is the re-export shim over `esp::embed` plus three genuinely
  mere-coupled modules, all built and all unwired: `persistence`
  (save/load a `VectorIndex` through eidetic's typed-payload API),
  `field_bridge` and `canvas_search` (project query similarity into
  quint's field algebra over the graph canvas). Zero importers means
  capability awaiting wiring, not deadness. Keep, and wire below.
- **`eidetic-search` is the lexical half, ready.** `TrailIndex` minted
  *from* `BrowsingTrace` engrams (derived state, re-mintable, format
  version carried with the index): BM25 recall over tokenized titles/page
  text/URL components plus exact URL/domain terms,
  fast-field reports (`top_domains`, `visits_histogram`), and `fuse()`,
  the engine-agnostic reciprocal-rank seam that deliberately takes both
  rankings from the caller.
- **The missing precondition is capture.** `BrowsingTrace` exists only
  inside eidetic-core (model, tests, example). Turnstone authors none.
  Recall without a corpus returns nothing, so the first slice is capture,
  not search UI.

## 2. Slices

- **W1 — capture.** Turnstone authors `BrowsingTrace` engrams at
  navigation commit points (the observe/session-lifecycle seam),
  persona-scoped, into the existing eidetic store. **Done when** a real
  browsing session yields traces that re-mint into a `TrailIndex` (run
  eidetic-search's `eidetic-recall` example against the real store).
- **W2 — recall in the omnibar.** A non-privileged omnibar lane queries
  `TrailIndex::search`; hits render as actionable rows (open the page /
  summon the node). Index staleness is surfaced honestly and
  `FormatMismatch` re-mints rather than erroring, per the crate's own
  doctrine. **Done when** "where did I read about X" answers from the
  user's own trail. *(Landed 2026-08-12; see §5.)*
- **W3 — reports.** The trail/steward surface renders `top_domains` and
  `visits_histogram` from the fast-field columns (no re-index needed).
  Small; may ride W2's session.
  **Fleece boundary audit (2026-08-26, corrected 2026-09-07):** the 2026-08-26
  note claimed the live host capture path supplies extracted page text to the
  trace corpus. It does not: `TraceEvent` carries url and title only, and
  turnstone extracts only the title. The removal of `mere-eidetic-search`'s
  direct Fleece dependency (`genet/design_docs/2026-08-26_fleece_followthrough_plan.md`)
  stands on the right ground anyway: the index consumes traces, not a DOM, so
  text enters through the host, never through the search crate. See W6.
- **W6 — restore page text, rank by behaviour, key by content (ruled
  2026-09-07 by Mark on the
  [lighter recall brief](../../eidetic_docs/research/2026-09-07_lighter_recall_and_standards_ledger_brief.md)).**
  Four slices, in this order:
  - **W6a — measure.** Instrument turnstone's `RecallIndex::mint` to report
    trace and event counts, corpus bytes, and lexical and vector mint
    latency; run a real session; record the numbers here. The engine
    decision (in-tree BM25 with one shared tokenizer, `probly-search`, or
    tantivy kept) waits on this receipt. **Done when** a real session's
    numbers are in §5.
  - **W6b — frecency.** A fold over `TraceTransition` kinds and `at_ms`
    under a 30-day half-life (Firefox's bucket weights as the starting
    table), fused as a third ranking beside the lexical and vector ones.
    `dwell_ms` joins when a producer fills it. Lands before any engine
    change so the engine is judged against a lane that already ranks well.
    **Done when** a typed-URL prefix recalls the page the user picked last,
    and the fusion receipt shows the behavioural ranking contributing.
  - **W6c — page text, restored.** The capture plan's C5 was built in
    meerkat (`8b8b039`, 2026-06-28) and lost when turnstone obviated it. The
    host extracts `main_text` through fleece at fetch time and attaches it to
    a fingerprint-keyed page record (W6d), not to the trace event. Consent
    gating stays C4's. **Done when** a body-only term recalls the page from
    a real session, as the meerkat receipt once did.
  - **W6d — page identity by content.** Traces stay an event log; a page
    table is a projection keyed by a content fingerprint over `main_text`
    (blake3 exact, plus a near-duplicate hash). It is the dedup key and the
    index key, and collapses the review brief's five materializations to
    one page plus events. **Done when** two URLs for one page resolve to
    one page record and one index document.
  The tokenizer W6a's engine uses is genet's, once the UAX #29 segmentation
  component ruled the same day is founded there; until then the search lane
  keeps a local tokenizer and names the swap.
- **W4 — canvas semantic search.** Wire `canvas::canvas_search` +
  `canvas::field_bridge` into the canvas's live surface: a query becomes a
  similarity field over the canvas through quint, with
  `esp::embed::persistence` (feature `persistence`) saving the
  `VectorIndex` via eidetic. Start on the lexical embedding provider
  (deterministic, no Burn), `bert` behind its existing feature per esp's
  target matrix. *(Paths updated 2026-08-12 by the
  [eidetic reorg](2026-08-12_eidetic_reorg_plan.md): the modules now live in
  the crates that will use them, not in the deleted `mere-embed`.)*
- **W5 — fusion.** `fuse()` merges W2's lexical ranking with W4's vector
  ranking in the omnibar. Gated on both.

## 3. Non-goals

- The moot consume-half of `SearchIndexSpec` (deferred by its own doc).
- A crawl-driven corpus (`mere-crawl` stays parked pending the gazette
  feed pipeline, per the census).
- New embedding backends beyond what esp already ships.

## 4. Sequence

W1 first; W2 and W3 follow it; W4 is independent of W2/W3 and may
interleave; W5 last. Each slice lands with its own receipt against a real
store, not fixtures only.

## 5. Progress

- **2026-08-12 — W1 landed** (turnstone `539dacc`). The trail-memory port
  mirrors the recycle bin's actor shape exactly: session-scoped
  `FjallStore` at `sessions/<id>/memory`, `BrowsingTrace` segments through
  `BrowsingMemory`, `from` chained per owner inside the actor, flush on
  segment fill and every lifecycle edge (switch, close, release — the
  release rides the same Windows rename handshake as the bin, since the
  memory store lives in the session dir). Capture rides the observation
  drain as designed: the shell's `drain_app_events` is the first
  production consumer of `App::take_events`, mapping
  AddressOpened/NavigatedBack/NavigatedForward/Reloaded onto
  UrlTyped/Back/Forward/Reload with the root identity's public key hex as
  the owner tag. Three unit tests green (round trip with origin chaining,
  self-flush on a full segment, event mapping); lib check clean. Still
  open in W1's done-condition: the headed receipt — a real browsing
  session's store re-minted through eidetic-search's `eidetic-recall`
  example. W2 is unblocked.
- **2026-08-12 — W1 done condition closed** (turnstone `f22f61f`). Building
  the receipt exposed one real gap: nothing flushed on a normal quit, so a
  short session would have left an empty store. `ApplicationHandler::exiting`
  now releases the trail store under a bounded ack (the scenario driver's
  Done exits through the same hook). The receipt itself
  (`scenarios/trail_capture.scn`, fresh profile): three `mere://`
  navigations through the real shell landed as **1 trace, 3 traversals, 3
  distinct pages**; `eidetic-recall index` minted a 3-document trail index
  from the session's store; `search alpha` answered `mere://alpha` at 0.98
  with the capture-time timestamp. "Where did I read about X" answers from
  a real session's store. W1 is complete.

- **2026-08-31: W4 lexical n-gram input probe complete.** ESP's cheap
  `LexicalEmbeddingProvider` now accepts explicit token n-gram orders while
  `new(dimensions)` remains byte-for-byte compatible unigram hashing. Orders
  are positive, non-empty, sorted, and deduplicated. Higher orders hash token
  windows without allocating joined strings. The decoder stack, Eidetic
  artifacts, `TrailIndex`, and reciprocal-rank fusion seam are unchanged.
  DeepSeek's [Engram paper](https://arxiv.org/abs/2601.07372) puts deterministic
  hashed n-gram lookup, learned memory tables, and contextual gating inside a
  Transformer. This probe borrows only the phrase-addressing idea at the
  application retrieval layer. Mere's existing `Engram` envelope concept is
  untouched.

  The fixed receipt uses the existing `SemanticSearch` → dense `VectorIndex`
  path over 28 derived records and 15 held-out queries: two each for browsing,
  title, URL, entity, and command phrases, plus five unigram/order-insensitive
  controls. Phrase targets compete with shorter records containing the same
  unigrams in another order. This is a forcing corpus for phrase sensitivity,
  not a claim about a production browsing corpus.

  | token orders | Ranking@1 | phrase Ranking@1 | controls Ranking@1 |
  |---|---:|---:|---:|
  | `1` | 7/15 | 2/10 | 5/5 |
  | `1+2` | 15/15 | 10/10 | 5/5 |
  | `1+2+3` | 15/15 | 10/10 | 5/5 |

  Optimized Windows x86-64 cost receipt, 4,096 dimensions, single test thread.
  The ranges below cover three harness runs; each run reports the median of 11
  order-rotated samples:

  | token orders | build ns/document range | query ns range | dense vector bytes | JSON index bytes | occupied slots |
  |---|---:|---:|---:|---:|---:|
  | `1` | 3,914–4,374 | 232,099–244,752 | 458,752 | 459,936 | 125 |
  | `1+2` | 3,987–4,340 | 226,749–248,814 | 458,752 | 460,924 | 222 |
  | `1+2+3` | 3,936–4,495 | 227,815–248,751 | 458,752 | 461,569 | 291 |

  The build and query ranges overlap, so this harness found no latency
  regression distinguishable from run-to-run noise at this corpus size. Dense
  storage is unchanged because the index remains fixed-width; JSON grew by 988
  bytes for bigrams and 1,633 bytes for bigrams plus trigrams. The result
  supports keeping `1+2` as an application setting for phrase-heavy surfaces
  while retaining `1` as compatibility default. Trigrams earned no additional
  ranking win here. Because orders are a vector-space input, changing the
  setting must re-mint the derived index; the index never becomes authority.
  Reproduce with:

  ```text
  cargo test -p esp --test lexical_ngram_recall --offline -j 1 -- --nocapture --test-threads=1
  cargo test -p esp --release --test lexical_ngram_recall --offline -j 1 -- --ignored --nocapture --test-threads=1
  ```

  Final focused gates passed from the shared checkout with isolated
  `CARGO_TARGET_DIR`: 65 ESP library tests plus two active held-out tests in the
  integration target,
  package-local Clippy over all targets with warnings denied, the default ESP
  library check for `wasm32-unknown-unknown`, formatting, and `git diff --check`.
  A broader Clippy invocation that linted dependencies stopped in seven
  pre-existing Personae `redundant_slicing` warnings before ESP; the
  `--no-deps` ESP gate passed. The checkout's unrelated carrier/Luggage work
  and four incoming Distillery-plan commits were left untouched.

- **2026-08-31: W5 deterministic fusion probe complete; live fusion remains
  open.** The same 28 records and 15 held-out queries now pass through an actual
  `BrowsingTrace` -> Tantivy `TrailIndex` BM25 ranking and the existing
  reciprocal-rank `fuse()` seam. URL cases occupy the URL field and have no
  title, so the projection does not hide field behavior. ESP names
  `eidetic-search` only as a dev dependency for this consumer-side receipt;
  neither production crate gains a dependency on the other.

  | ranking | deterministic Ranking@1 | Ranking@3 | phrase Ranking@1 | control Ranking@1 | unique fused Ranking@1 | expected target tied at top |
  |---|---:|---:|---:|---:|---:|---:|
  | BM25 | 7/15 | 15/15 | 2/10 | 5/5 | - | - |
  | unigram feature vector | 7/15 | 15/15 | 2/10 | 5/5 | - | - |
  | `1+2` feature vector | 15/15 | 15/15 | 10/10 | 5/5 | - | - |
  | BM25 + unigram RRF, weights `(1, 1)` | 7/15 | 15/15 | 2/10 | 5/5 | 7/15 | 0 |
  | BM25 + `1+2` RRF, weights `(1, 1)` | 15/15 | 15/15 | 10/10 | 5/5 | 7/15 | 8 |
  | BM25 + `1+2` RRF, weights `(1, 2)` | 15/15 | 15/15 | 10/10 | 5/5 | 15/15 | 0 |

  The main stop sign matters more than the headline numbers. Equal-weight RRF
  gives eight corrected phrase targets exactly the same score as their reversed
  decoys; the documented URL tie-break happens to put the target first. Its
  apparent 15/15 is therefore not a fusion win. Giving the vector input twice
  the weight breaks those ties, but `(1, 2)` is only a probe of the existing
  setting seam, not a recommended default. A real captured-trail partition must
  select weights without reusing its held-out queries.

  The titleless-URL gap found by the first fusion run is closed in field set V3.
  The canonical `url` remains an exact stored `STRING`; a separate, non-stored
  `url_text` field indexes the same canonical bytes with Tantivy's default
  tokenizer. `TrailIndex::search` queries both representations. A single
  absolute URL bypasses the query-string grammar and uses an exact `TermQuery`,
  preventing `https:` from being misread as an unknown field. A V2 sidecar is
  rejected before Tantivy opens its segments, and rebuilding from the trace
  corpus writes V3 and restores component recall. Both titleless URL targets now
  appear at rank 2 under BM25, raising its Ranking@3 from 13/15 to 15/15 while
  preserving the useful contrast: bag-of-words BM25 still prefers the shorter
  reversed-order decoy, and `1+2` supplies the ordering signal.

  Direct gates passed in the isolated target: all 14 `eidetic-search` library
  tests, including V2 rejection/re-mint, titleless URL-component recall, and
  exact full-URL recall; the two active
  ESP integration tests; package-local Clippy with warnings denied; formatting;
  and `git diff --check`. The first direct `eidetic-search` test build took
  34m14s on the loaded shared host because its existing dev dependencies enable
  the BERT/WGPU example graph; the tests themselves ran in 0.35s. Pre-existing
  warnings came from patched Burn, Genet Nematic, and `mere-kernel`, not these
  crates.

  This remains a lexical forcing receipt. It compares BM25, unigram feature
  hashing, `1+2` feature hashing, and their actual RRF combinations. The fixed
  learned-vector comparison below closes the model-baseline gap for this
  fixture. W5 is done only when the live caller supplies both rankings over a
  real held-out trail corpus and selects weights without reusing its evaluation
  partition.

- **2026-09-01: fixed MiniLM baseline complete; live W5 remains open.** The
  ignored `learned_minilm_baseline` receipt runs the same 28 records and 15
  queries through ESP's real Burn/NdArray CPU loader and `SemanticSearch`, then
  brings that ranking to the same Eidetic BM25/RRF seam. It requires
  `SIBYLLA_MINILM_DIR`; repository-ignored model data never becomes a silent CI
  dependency. The test verifies every artifact before loading:

  | artifact | bytes | SHA-256 |
  |---|---:|---|
  | `config.json` | 612 | `953f9c0d463486b10a6871cc2fd59f223b2c70184f49815e7efbcab5d8908b41` |
  | `tokenizer.json` | 466,247 | `be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037` |
  | `model.safetensors` | 90,868,376 | `53aa51172d142c89d9012cce15ae4d6cc0ca6895895114379cacb4fab128d9db` |

  This is `sentence-transformers/all-MiniLM-L6-v2`: 384 dimensions, mean
  pooling, L2 normalization. Total artifact size is 91,335,235 bytes. Its
  28-record vector index is 43,008 bytes, compared with 458,752 bytes for the
  4,096-dimensional feature-hashed index; the dense model also carries its
  weights and runtime.

  | ranking | deterministic Ranking@1 | Ranking@3 | phrase Ranking@1 | control Ranking@1 | unique fused Ranking@1 | expected target tied at top |
  |---|---:|---:|---:|---:|---:|---:|
  | MiniLM | 12/15 | 15/15 | 7/10 | 5/5 | - | - |
  | BM25 + MiniLM RRF, weights `(1, 1)` | 12/15 | 15/15 | 7/10 | 5/5 | 7/15 | 5 |
  | BM25 + MiniLM RRF, weights `(1, 2)` | 12/15 | 15/15 | 7/10 | 5/5 | 12/15 | 0 |
  | BM25 + `1+2` RRF, weights `(1, 2)` | 15/15 | 15/15 | 10/10 | 5/5 | 15/15 | 0 |

  MiniLM leaves `graph query nodes`, `washington post company`, and
  `open downloads folder` at rank 2. Equal weighting turns five of its wins
  into exact BM25 ties. A 2x dense weight restores MiniLM's own 12 unique wins
  but adds none. On this deliberately phrase-sensitive fixture, `1+2` feature
  hashing therefore beats the learned model rather than merely beating a weak
  unigram stand-in.

  Optimized Windows x86-64 CPU measurements across nine runs on the loaded
  shared host were 78-86 ms to load, 243-261 ms to embed all 28 records, and
  323-349 ms for the 15-query sweep (21.5-23.3 ms/query). Three direct process
  runs peaked at 194,674,688-194,699,264 bytes working set. The earlier lexical
  cost harness measured 226,749-248,814 ns/query for `1+2`, but used repeated
  rotated sweeps; the raw figures establish the cost separation without
  pretending the protocols are identical. The first isolated release build
  took 26m06s under concurrent host load; that is compile cost, not inference
  latency.

  Reproduce the learned baseline with:

  ```text
  SIBYLLA_MINILM_DIR=/path/to/all-MiniLM-L6-v2 cargo test -p esp --release --features bert --test lexical_ngram_recall --offline -j 1 learned_minilm_baseline -- --ignored --nocapture --test-threads=1
  ```

  The conclusion stays narrow: expose `1+2` as the phrase-sensitive derived
  lookup setting and take it to a real captured-trail partition. Do not promote
  a live fusion weight from this forcing fixture. A broader corpus with true
  paraphrases is still required to judge the semantic value of MiniLM, while
  live W5 still owns caller wiring and weight selection.

- **2026-09-01: live trail-fusion caller implemented; real W5 selection remains
  open.** Turnstone `7d6e348` on `codex/0901-ngram-recall` consumes Mere
  `7b6ced78` on `codex/0901-ngram-search`. The trail actor now owns a disposable
  `TrailIndex` plus an optional ESP lexical-vector index over one latest record
  per canonical URL. Current graph and recycle-bin titles are projected into
  cloned traces while minting; browsing traces remain unchanged authority.
  Corpus, title, or enabled token-order changes re-mint the derived projection.
  Weight-only changes reuse it.

  Two live application settings expose cumulative token orders `1`, `1+2`, and
  `1+2+3`, plus phrase influence relative to BM25 from `0` through `4`. The
  defaults are order `2` and influence `0`. At zero influence Turnstone issues
  the original BM25 query with the original limit and never mints the vector
  projection. Positive influence widens both candidate heads, applies the
  existing deterministic RRF seam, and reduces to the omnibar limit.

  The remote-pinned Turnstone graph passed a library check and all-test
  type-check. Executed gates passed five trail-actor tests, three settings-owner
  tests, four retained-settings-pane tests, and warning-denied Clippy over the
  focused actor module. The actor receipt proves that enabling `1+2` with weight
  `2` corrects a reversed-word-order BM25 tie and that changing current titles
  re-mints without a new traversal. Broader Turnstone Clippy still reports its
  existing warning set; none point into the new recall logic.

  This closes the caller mechanics only. W4's canvas similarity-field and
  persistence wiring remain separate. W5's done condition still requires a
  real captured-trail training/evaluation split, weight selection on the
  training side, held-out ranking metrics, and a stated RAM and latency budget.

- **2026-09-02: private captured-trail gate implemented; current corpus does
  not admit selection.** Turnstone `57faef4` on `codex/0901-ngram-recall`
  adds the local-only harness and records its protocol in
  `turnstone/design_docs/2026-09-02_trail_recall_evaluation_plan.md`. The
  ignored receipt takes one explicit session, copies it before opening Fjall,
  overlays current graph and recycle-bin titles through the live caller's
  projection, and emits only digests, aggregate ranking metrics, and costs.
  The judgment manifest remains outside both repositories.

  Admission requires twenty distinct documents and disjoint training and
  held-out targets: five distinct phrase targets and five controls on each
  side. Queries must be unique after lexical normalization; phrase queries and
  projected titles must contain multiple tokens. The manifest explicitly sets
  maximum token orders, positive fusion weights, `Ranking@K`, an exact dense
  vector-payload budget, and a p95 query budget. Selection sees training cases
  only. The held-out verdict requires a unique-top-one phrase gain while
  preserving overall unique top one, Recall@K, and control unique top one.
  Resource overruns or metric ties keep BM25.

  The copied active-profile corpus at BLAKE3
  `db4d9b34d4acbc57ee6a1accf84a7e03e3b3760e87b2c1076c1d56af11db6da6`
  contains four traces, eleven traversals, seven distinct pages, and one page
  with a current projected title. The receipt therefore returned
  `insufficient_corpus` before loading judgments or selecting a weight. This
  is a negative admission receipt, not a phrase-feature loss. BM25 remains the
  evidence-backed default.

  The clean remote-pinned Turnstone test graph compiled. The synthetic
  training-only selection test, all five trail-memory actor tests, and the
  ignored active-profile admission receipt passed. Package Clippy exited zero;
  its existing broad warning set contained nothing in the two changed Rust
  files. Targeted formatting and `git diff --check` passed. The earlier
  disposable profile copy was verified and deleted after the receipt.

  W5's next done condition is concrete: capture at least twenty distinct pages
  with enough current multi-token titles, author the private cases from
  remembered intent before consulting stored titles, and run the admitted
  receipt. A setting becomes a promotion candidate only if that held-out run
  says so. W4 remains an independent canvas-field lane.

- **2026-09-07 — W6 ruled.** Mark answered the six questions of the
  [lighter recall brief](../../eidetic_docs/research/2026-09-07_lighter_recall_and_standards_ledger_brief.md)
  yes: measure before choosing the engine, frecency before the engine
  change, restore body text as a stated slice (C5 was lost with meerkat, not
  decided against), found one UAX #29 segmenter in genet, found the
  standards-to-features ledger in genet beside the WPT census, and read
  traces as an event log with a fingerprint-keyed page table projected over
  them. The false 2026-08-26 W3 note is corrected above. W6a's
  instrumentation is the first action.
- **2026-09-07 — W6a measured** (turnstone working tree, uncommitted;
  `MintReceipt` at `src/trail_memory.rs:322`, emitted as one `tracing::info!`
  line at mint; tests `mint_receipt_counts_the_corpus_exactly` and the
  `#[ignore]`d `mint_receipt_scale_ladder` and
  `captured_trail_mint_receipt`). Release build, Windows 11:

  | corpus | events | pages | bytes | lexical ms | vector ms | total ms |
  |---|---:|---:|---:|---:|---:|---:|
  | captured session, vector on | 35 | 11 | 712 | 37.6 | 0.2 | 37.8 |
  | synthetic 1k, vector on | 1,000 | 333 | 50,004 | 29.0 | 3.4 | 33.1 |
  | synthetic 10k, vector on | 10,000 | 3,333 | 530,001 | 89.8 | 34.6 | 130.8 |
  | synthetic 100k, vector on | 100,000 | 33,333 | 5,599,998 | 1,693.0 | 493.8 | 2,315.3 |

  Three findings. **tantivy's mint is about 40 ms of fixed cost** (directory
  wipe, index create, writer, commit, sidecar) regardless of corpus size, paid
  on every recall after a navigation; corpus size dominates only past about
  10k events. **The lexical lane indexes one document per event**, so at 100k
  events tantivy holds 100,000 documents against 33,333 pages; the 3x is
  free to reclaim. **The vector lane is the memory problem, not tantivy**:
  `esp`'s dense `VectorIndex` at 4,096 dims is 16 KB per page, 55 MB at 10k
  events and 546 MB at 100k, resident in the actor for a disposable index,
  and `fused_hits` scans it linearly per query. The real store measured is
  small (35 traversals, 11 pages), so the ladder carries the scaling claim.

  Build note: turnstone HEAD `c6ee31e` does not compile against its committed
  mere pin `d82afa17` (eight errors in `browse.rs`, `shell/mod.rs`,
  `shell/reader_observe.rs` from mere APIs that landed later), and patching
  mere alone splits `genet_scripted_dom`. The numbers were taken with mere
  and genet both patched to the local checkouts per invocation
  (`cargo --config`), nothing edited. The pin reconciliation is Mark's.
- **2026-09-07 — W6b built** (uncommitted at writing). The fold lives in the
  stack: `crates/eidetic/eidetic-core/src/browsing/frecency.rs` (pure,
  clockless; `TransitionWeights` exhaustive over `TraceTransition`,
  `FrecencyConfig` with a 30-day half-life and a +0.5 dwell bonus at 30 s,
  `frecency_by` keyed by a caller function so W6d's fingerprint slots in
  without a signature change). Defaults follow Firefox's visit bonuses over
  100: typed 20, link click and tab spawn 1, imported 0.75, back and forward
  0.25, reload, redirect and restore 0. `Unknown` is 1.0 rather than
  Firefox's 0 because turnstone maps the engine's `ContentNavigated` to
  `Unknown`, and at zero the whole live corpus scores nothing; ruled 1.0 by
  Mark, 2026-09-07. `eidetic-search::fusion` generalizes reciprocal-rank fusion to N
  weighted rankings (`Ranking`, `fuse_many`; `fuse` is the two-lane wrapper,
  `FusedHit` gains index-aligned `ranks`). Turnstone folds the table at mint
  (`MintReceipt.frecency`), filters candidates by whole-query substring over
  URL and title, and fuses a third lane at weight 2.0 (`RecallConfig::
  frecency_weight`; above 1.0 so an opposed two-lane tie resolves by
  behaviour rather than by URL order). Tests: four in eidetic-core, two new
  in fusion, `typed_prefix_recall_follows_frecency_over_title_overlap` in
  turnstone; 99 + 16 + 4 and 8 pass respectively.

  Receipt on the captured store (11 pages, thin corpus): the behavioural
  lane lifts the top-frecency page from fourth to first for a typed prefix.
  Cost: 33 µs beside a 61.7 ms lexical re-mint; 103 ms beside 12.15 s on the
  100k-event ladder. Follow-ons named, not built: adaptive input history
  (needs a picked-row signal from the omnibar and a store write path), a
  settings knob for the lane weight, a producer for `dwell_ms` (ledger row 4,
  Intersection Observer), and turnstone tuning of `FrecencyConfig`.
- **2026-09-07 — W6d built** (uncommitted at writing).
  `crates/eidetic/eidetic-core/src/browsing/page.rs`: `canonical_url`
  (lowercase scheme and host; drop fragment, default port, the trailing
  slash on an empty path, and `utm_*` plus an explicit tracking-param list;
  everything else verbatim; no URL crate, since the key must be computable
  in the storage layer), `PageFingerprint` with a visible `source` (`Text`
  or `CanonicalUrl`), blake3 exact over whitespace-collapsed text and a
  64-bit simhash over lowercased 3-word shingles, `PageRecord` (canonical
  URL set, last verbatim URL, best title, first and last seen, visits, the
  text slot W6c fills), `page_table` with the `text_for` shape of
  `rebuild_with_text`, near-duplicate collapse oldest-first under
  `PageTableConfig` (default Hamming 3, `EXACT` for 0), and
  `frecency_by_page`. The threshold was measured on page-length prose: one
  word changed costs 1 bit, a header swap 2, an unrelated page 37, a
  truncation to half 16; a 55-word paragraph made the same edit cost 8, so
  the near key means nothing until real bodies arrive with W6c. Six tests;
  `mere-eidetic` 105 pass.

  Turnstone: `recall_documents` and the lexical corpus are per page record
  (one tantivy document per page, the 3x W6a found), the frecency lane is
  keyed by fingerprint and re-keyed to the record's address so the fusion
  lanes and `RecallHit` still speak URLs; `MintReceipt` gains
  `collapsed_urls` and `page_table`. Test
  `two_urls_for_one_page_recall_once_with_one_frecency`; 9 pass. Ladder,
  release, vector on:

  | corpus | lexical before | lexical after | page table | total before | total after |
  |---|---:|---:|---:|---:|---:|
  | 10k | 89.8 ms | 65.3 ms | 27.8 ms | 130.8 ms | 136.8 ms |
  | 100k | 1,693.0 ms | 519.1 ms | 393.5 ms | 2,315.3 ms | 1,393.1 ms |

  Follow-ons named: `canonical_url` runs three times per event across the
  projection, corpus and frecency (memoized once); passing the fingerprint
  index through would cut about a third of the projection cost but widens a
  stack signature. Turnstone has never had its rustfmt sweep; an accidental
  `cargo fmt` rewrote 82 files and was reverted. `cargo clippy` cannot carry
  `--config`, so turnstone's clippy is unreachable under the scratch patch
  until the family re-pin.
- **2026-09-07 — W6c built** (uncommitted at writing). Page text is back,
  and it lives in the stack: `crates/eidetic/eidetic-core/src/browsing/text.rs`
  is `PageTextStore` over muniment's two stores, bytes at the
  content-addressed `blob/<blake3>` and one slot per address at
  `page-text/<blake3 of canonical_url>` holding `{url, blob, stored_at_ms}`,
  idempotent on an unchanged body; `PageTexts::lookup()` is the `text_for`
  closure `page_table` and `rebuild_with_text` take. muniment gains
  `impl Backend for &B` so one `FjallStore` carries both stores unmoved.
  The canonicalize-once follow-on landed with it: `page_table` returns a
  `PageTable { records, by_address }`, `frecency_by_page` takes the table,
  and `fingerprint_index` is gone. `mere-eidetic` 109 pass, `muniment` 38.

  Turnstone: `browse.rs::page_text` extracts `extract_main_text` and falls
  back to `extract_text` wherever main text is `None`, which is exactly the
  landing pages and app shells a body-term recall is otherwise blindest to;
  `Effect::RecordPageText` rides the trail actor's own handle as
  `TrailCommand::RecordText`, so a page's text and its visit stay ordered;
  the actor writes through `PageTextStore` into the session store and marks
  the index stale. `consented_to_keep` is the named C4 no-op, the one place
  a policy refuses. `RecallIndex::mint` calls `rebuild_with_text` with each
  record's own text; no `SearchIndexSpec` bump, since the text field has
  existed since `FIELDS_V2` and presence is per document. `MintReceipt`
  gains `pages_with_text` and `text_bytes`. Test
  `a_body_only_term_recalls_the_page` with a negative control in the same
  run; 10 pass.

  Headed receipt, `scenarios/trail_text_recall.scn` over a loopback fixture
  with two articles whose bodies alone carry HAGIOSCOPE and PARBUCKLE:
  `RESULT ok`, two captures, each term offers exactly its own page in the
  omnibar; artifacts at `Code/testing/turnstone/trail_text_recall/`. Done
  condition met: a body-only term recalls the page from a real session, as
  the meerkat receipt once did. Open, named for C4: no size cap on a stored
  body, and blobs are never collected (`forget` drops the slot only).
- **2026-09-07 — engine swap built** (uncommitted at writing). tantivy is
  retired from `crates/intel/eidetic-search`. `tokenize.rs` is one
  tokenizer for indexing and querying, UAX #29 word segmentation through
  `unicode-segmentation`, lowercased, with a no-op `Stemmer` hook;
  `Tokenizer::segment` is the named swap point for genet's segmentation
  component, and the type is exported for `esp`'s lexical embedder to share
  later. `bm25.rs` is a field-agnostic in-memory postings index with
  `Bm25Config` (k1 1.2, b 0.75), per-field IDF and length normalization,
  and `select_nth_unstable_by` before the head sort. `index.rs` keeps
  `TrailIndex::rebuild`, `rebuild_with_text`, `search`, `Hit`,
  `doc_count`, `top_domains`, `visits_histogram` and `open`; adds
  `rebuild_with_config` with `FieldWeights` (title 3.0, text 1.5, url
  1.0) and a `persist` flag, default on so disk behaviour is unchanged.
  Persistence writes the projected documents plus scoring settings as
  JSON and `open` replays the build, so a reopened index cannot rank
  differently from a fresh mint. `spec.rs` moves to `FIELDS_V4` with
  `engine_version` (serde alias for the old field), so a tantivy-era
  directory refuses as `FormatMismatch`. Two deliberate renames:
  `SearchError::Tantivy` to `Engine`, `tantivy_version` to
  `engine_version`; no consumer named either. The `domain` scoring field is
  dropped (the tokenized URL already carries the host parts); it stays a
  stored column for reports.

  Receipts: `cargo tree` 121 unique packages to 24 (the 98 the review brief
  counted); wasm32 check clean; `esp`'s `lexical_ngram_recall` fixture,
  which asserts tantivy's exact BM25 tallies and eight fusion ties, passes
  unchanged. Release, one document per page:

  | pages | mint, transient | mint, persisted | tantivy (W6d) | query |
  |---:|---:|---:|---:|---:|
  | 1,000 | 2.0 ms | 5.4 ms | about 40 ms fixed | 8 µs |
  | 10,000 | 25.4 ms | 40.0 ms | 65.3 ms | 59 µs |
  | 100,000 | 258.2 ms | 434.9 ms | 519.1 ms | 409 µs |

  Named, not built: CJK segments per ideograph without a dictionary (the
  genet component's problem, now a one-function swap); stemming stays off
  until body text warrants it; esp sharing the tokenizer; incremental
  updates as a tombstone plus vacuum, the invalidated-projection shape;
  turnstone still calls plain `rebuild` and so pays the persisted write it
  never reads, one argument at its call site takes the transient path.
- **2026-09-07 — W6e, a real corpus: Mark's Firefox history** (94,096
  visits over 20 months, 44,557 places; exported from a copy, never the
  live profile, and no address, title or term appears in any receipt).
  `scripts/firefox_history_export.py` and `import::history` (mapping table
  in its doc comment, asserted against `TransitionWeights::default()` by
  test) plus the `#[ignore]`d turnstone harness
  `firefox_history_corpus_receipt`. Receipts, release:

  | measure | value |
  |---|---:|
  | traces / events / pages | 2,941 / 94,096 / 41,822 |
  | addresses collapsed by canonicalization | 2,082 (top collapse sizes 422, 115, 82, 39, 35) |
  | mint, vector off | page table 529 ms, lexical 261 ms, frecency 85 ms, total 895 ms |
  | mint, vector on | plus 566 ms vector, total 1,440 ms; about 685 MB resident |
  | Spearman vs Firefox frecency | 0.859 (0.858 with the dwell bonus off) |
  | Firefox top-100 pages found in our top 100 | 36 |
  | events with dwell / over the 30 s bonus | 6,335 (6.7%) / 1,410 (1.5%) |
  | typed-prefix query, three hosts | 19 to 22 µs without the frecency lane; 20.8 to 22.7 ms with it; the top-frecency candidate came first in all three only with the lane |

  What the numbers say. The behavioural lane is right and slow: it
  substring-scans all 41,822 entries per keystroke, a thousand times the
  lexical query; it needs a candidate index (tokenized prefix lookup over
  the page table) before it can ship at this size. Global rank agreement
  with Firefox is strong and head agreement is weak, and the head is where
  the omnibar lives; the weight table and the half-life are the knobs. The
  dwell bonus is inert on this profile because Firefox's interaction data
  reaches back only about four months and only 1.5% of visits clear 30 s;
  the threshold or the producer, not the fold, is what to tune. Near-
  duplicate collapse did nothing because every fingerprint is URL-sourced
  without stored bodies, confirming W6d's caveat at scale. Firefox
  `visit_type` 4 and 8 never appear in this profile, so `AutoSubframe` is
  exercised only by the unit test. 1,299 of 20,675 referring visits are
  dangling (expired rows), capping reconstructed lineage. A weaker copy of
  the mapping lives in `eidetic-search`'s `eidetic-recall` example
  (`transition_of`), collapsing subframes and downloads to `Imported`;
  `import::history_to_traces` supersedes it and the example should call
  through. `visited_at_unix_secs` is second-granular; the exporter and the
  stable sort preserve intra-second order, a millisecond field would make
  that a property of the type.
- **2026-09-07 — turnstone takes the transient index** (`c863b8e`): the
  mint calls `rebuild_with_config` with `persist: false`, so the recall
  path pays the 258 ms mint at 100k pages rather than the 435 ms persisted
  write it never reads. Turnstone's W6c host half is `c863b8e`, the corpus
  harness `696bc2f`; mere's W6c stack half `5231ae3e`, engine `7a5b7e19`,
  corpus `fc0c9e8c`. All unpushed pending the family re-pin.
- **2026-09-07 — candidate index for the behavioural lane.** W6e found the
  lane substring-scanning all 41,822 pages per keystroke. `eidetic-search::
  candidates::CandidateIndex` is a token-prefix lookup over the shared
  tokenizer: four flat arrays (a token blob, token offsets, ascending
  deduplicated postings, posting offsets), a prefix range by two binary
  searches, AND across query tokens by sorted intersection; `candidate_ids`
  returns record ids in build order so a consumer scores a wide set by
  index. Semantics change, deliberate and tested: token prefix, not
  whole-query substring, so `zette` no longer reaches `gazette`; hosts need
  no special case because the tokenizer already splits `example.test`.
  Turnstone builds it at mint over each record's addresses and title
  (`MintReceipt.candidates`, `candidate_bytes`), keeps a score `Vec` aligned
  to the ids, and orders only the head with `select_nth_unstable_by`. Eight
  tests in eidetic-search; synthetic 42k pages build in 60 ms at 40 bytes
  per page, median query 74 µs. On the Firefox corpus, release:

  | query, two-char prefix | before | after | top-frecency first |
  |---|---:|---:|---|
  | busiest page 1 | 22.7 ms | 115 µs | yes |
  | busiest page 2 | 22.6 ms | 491 µs | yes |
  | busiest page 3 | 20.8 ms | 437 µs | yes |

  The index costs 488 ms and 4.6 MB (110 bytes per page) at mint on real
  addresses, which carry far more tokens than the synthetic ones; sharing
  the tokenization pass with the BM25 build would remove most of that and
  is the named follow-on. The Opus session limit interrupted the agent
  after the index was built; the id-aligned scoring and partial sort that
  closed the remaining cost were finished by hand.
- **2026-09-08 — the hashed vector lane retired from recall.** Mark's
  ruling on the 685 MB finding: retire first, measure, then revisit esp's
  representation (sparse storage, or 256 to 512 dimensions as esp's own
  docs advise) if a vector lane is wanted again. Turnstone's `RecallIndex`
  fuses two lanes, lexical and behavioural; `RecallConfig` is the one
  frecency weight; the `recall.phrase_order` and `recall.phrase_influence`
  settings are gone (an old persisted value deserializes and is never
  read); the esp dependency leaves turnstone; the evaluation module shrinks
  from 1,292 to 508 lines because its manifest apparatus was the vector A/B
  entire. Firefox corpus, release, before and after:

  | measure | before | after |
  |---|---:|---:|
  | mint total, lexical plus behavioural | 2,270 ms | 1,036 ms |
  | mint page table / lexical / candidates | 1,013 / 377 / 728 ms | 411 / 194 / 335 ms |
  | query with the behavioural lane, three prefixes | 102 / 551 / 578 µs | 46 / 255 / 279 µs |
  | peak resident, whole test process | 1,178 MB | 367 MB |
  | Spearman vs Firefox / top-100 overlap | 0.859 / 36 | 0.859 / 36 |

  Ranking is unchanged on every query line. The retained lanes halved
  because `recall_documents` built a concatenated title, URL and body string
  per page purely as vector-ingest input inside the page-table window; the
  rest of the speedup is reproducible but not isolated, allocator pressure
  being the plausible cause. Comparable: Firefox's own places.sqlite holds
  this whole profile in 36.7 MB, about 800 bytes per page; no production
  browser stores a vector per page. Next, when measured against a want: a
  sparse representation in esp (about 100 bytes per page) or 512 dims
  (85 MB), and a semantic embedder as the case a vector lane is for.
- **2026-09-08 — esp: sparse lexical vectors beside dense.** `SparseVector`
  (sorted `(u32, f32)` pairs, L2-normalized, no exact-zero entry as an
  invariant), `VectorIndex<K, V = Vec<f32>>` generic over an `IndexVector`
  trait so `SparseIndex<K>` is the same index with one `nearest` and one
  `SimilarityMetric`; the lexical provider emits sparse directly from its
  hashed pairs without a dense pass. Dense stays for `index_burn`, `bert`
  and W4's canvas search. Bit identity, not tolerance: the omitted buckets
  contribute exactly zero and signs are integers, asserted across a mixed
  corpus of scripts, dims and n-gram orders. `RECOMMENDED_DIMENSIONS_SHORT_
  TEXT` is 256..=512 and `hashing_stats` makes collision rate measurable.
  Bytes per vector `32 + 8·nnz` against `4·dims`. Receipt, release, 42,000
  synthetic titles, 100 queries:

  | dims | storage | bytes per vector | total | ingest | queries | collision rate |
  |---:|---|---:|---:|---:|---:|---:|
  | 4,096 | dense | 16,384 | 688 MB | 471 ms | 38.4 s | 0.90 |
  | 4,096 | sparse | 96 | 4 MB | 41 ms | 277 ms | |
  | 512 | dense | 2,048 | 86 MB | 86 ms | 5.2 s | 0.99 |
  | 512 | sparse | 96 | 4 MB | 44 ms | 310 ms | |

  Two findings. The top-10 sets differ on 37 of 100 queries only where a
  score tie straddles rank 10 and `nearest` breaks ties by hash-map
  iteration order, a pre-existing dense property; a key tiebreak would make
  ranking deterministic. And the collision rate says no storable dense
  dimension fixes corpus-scale collisions for a hashed embedding: 42,032
  distinct unigrams into 4,096 buckets collide at 0.90; the 256 to 512
  guidance is about one text's features, not a corpus vocabulary, so the
  lexical vector is a per-text similarity signal, not a corpus index.
  Consumers: nothing in turnstone or `ports/`; canvas search and the field
  bridge are generic over dimension; the mesh lexical codec takes the
  caller's dimension up to 4,096. `SemanticSearch` and the canvas surface
  stay dense-only pending a design call on how sparse reaches the facade.

