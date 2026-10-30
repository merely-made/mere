# Storage, identity, inference and application ownership review

**Revision, 2026-10-29:** the [corrected audit](REPORT.md) controls the recommendations. This appendix retains source evidence at the revisions below. Established suppliers and source size do not establish net benefit. Compare the complete consumer feature/build/target cost, retained adapter work and actual behavior removed. Architectural substitutions below are withdrawn from the reuse queue; they would need a coherent stack-level argument. No new implementation is authorized by this appendix.

Reviewed 2026-10-29. This is source and dependency evidence, not migration or runtime qualification. Revisions are deliberately fixed because other work was advancing the repositories during the audit.

| Source | Revision reviewed |
|---|---|
| Eidetic | `01ee97b0b95d679430986dffd54c64ce9b3b5ea3` |
| Dramatis | `6711105e0284adb3e881fc9aebcc56f25000c1ca` |
| Turnstone | `a1a52f3bcb` (full revision in turnstone-locked-sources.json) |
| Mere custody | `f631d1ec8909b978f1a31bce9e30f3e5bf65be78` |
| ESP consumer pin | `bd0468c5c44c0b098494d30b3cfd0aa283cecf48` |
| Armillary consumer pin | `864f107d44bf759111ad2ea3c9036b2cfbf03f4c` |
| Turquet consumer pin | `bc3c454f755d0bfd70ab48bd9556a1cda2213d41` (0.13.0) |

## Findings

### 1. Use a dedicated MAC comparison; family scope remains open

**Confidence: high. Scope: small.** Dramatis `crates/personae/src/sealed_record_storage/freshness.rs:188-189,262` uses a local XOR/OR loop called `constant_time_eq` when authenticating the freshness-ledger MAC. Source-level absence of an early return on different bytes does not provide a compiler-resistant constant-time contract. Use a dedicated equality primitive, such as [`constant_time_eq`](https://docs.rs/constant_time_eq/latest/constant_time_eq/), already directly used by Castellan elsewhere in the stack, or an appropriate `subtle` comparison.

This is a reason to retire a local primitive, not evidence of a demonstrated timing exploit. Preserve the fixed 32-byte MAC type and failure result. Done when the local primitive is removed, valid/tampered ledger tests retain their results, and the dependency/target policy is checked.

### 2. Evaluate/consolidate commodity codecs across the family

**Confidence: high. Scope: small, cross-repository.** Additional to the earlier Castellan Base32 and Truchman Base64 findings, Dramatis `crates/insigne/src/encoding.rs:17,44,73,84` implements Base58 and hex. Eidetic `crates/eidetic-core/src/pack.rs:173,178` implements another hex pair, and `crates/servitor/src/lib.rs:116,129` has another representation. Turnstone has more hex decoders under `src/place.rs`, `src/resident_admission.rs`, and `src/resident_runs.rs`.

[`bs58`](https://docs.rs/bs58/latest/bs58/) supports custom alphabets and caller buffers. The tested 0.5.1 alloc configuration adds tinyvec; do not describe it as dependency-free. Inspect the exact feature closure in each consumer. [`data-encoding`](https://docs.rs/data-encoding/latest/data_encoding/) covers Base32/64/hex with explicit tolerance settings. [`percent-encoding`](https://docs.rs/percent-encoding/latest/percent_encoding/) covers the repeated byte decoding in Turnstone `src/download.rs:130`, Castellan OTP URI intake and Gaz account intake. Retain domain validation, exact-length checks, accepted historical casing, and secret-buffer zeroization in the adapters. A tiny codec is still an independent compatibility burden when copied across many crates.

Done when one selected implementation per encoding backs the affected adapters, wire/reference vectors match, malformed inputs keep the intended refusal behavior, and sensitive buffers are cleared on failure.

### 3. Compare bookmark syntax reuse with a bounded repair

**Confidence: high. Scope: bounded.** Eidetic `crates/eidetic-core/src/import/parser.rs:394,433,479,500` owns an angle-bracket tokenizer, attribute reader and a very small named-entity table. `tokenize_html` finds the first `>` without tracking quoted attribute values. A quoted `>` in an exported bookmark attribute therefore ends the token early. The entity table covers five names plus numeric forms, rather than HTML's named characters.

A candidate is [`html5ever::tokenizer`](https://docs.rs/html5ever/latest/html5ever/tokenizer/) with a token sink preserving the Netscape-bookmark folder/date/location mapping. Its complete added closure and build/target cost in Eidetic are unmeasured; compare them with repairing the bounded parser before choosing it. The tokenizer can be used without adopting the browser engine or constructing a full DOM. A narrow import format does justify narrow domain mapping; it does not require another HTML lexer.

Done when real Firefox/Chrome/Safari export fixtures, quoted delimiters, comments, omitted closing tags, Unicode named references, malformed exports and intake notices survive the adapter. This is static evidence of a parser limitation; no importer execution was performed here.

### 4. Correct download header parsing; choose reuse after comparison

**Confidence: high for replacement; chosen parser still needs trial.** Turnstone `src/download.rs:112` splits Content-Disposition on every semicolon and strips quotes. Quoted filenames containing semicolons and extended parameters with a declared charset are standards work, not product behavior. The current `filename*` branch discards charset/language and decodes as UTF-8 lossily.

Evaluate an RFC-aware parser such as [`content_disposition`](https://docs.rs/content_disposition/latest/content_disposition/), and use `percent-encoding` for the bytes. Validate the selected crate against HTTP Content-Disposition specifically; an email-oriented parser is not automatically a complete HTTP solution. Keep our filename sanitization, directory containment, collision policy, retained source identity and download custody.

Done when quoted separators/escapes, filename versus filename* precedence, encoded names, bad escapes, non-UTF-8 declared charsets and traversal attempts have independent fixtures.

### 5. Consolidate atomic file replacement on a maintained primitive

**Confidence: high that duplication exists; medium for an exact shared adapter.** The inspected stack has independent temporary-file/flush/rename implementations in Turnstone `src/session_graph_file.rs:84`, Personae `crates/personae/src/sealed_record_storage.rs:593`, Castellan `ports/djinn/castellan/src/custody/atomic_secret_file.rs`, and Muniment `crates/muniment/src/directory_backend.rs:227`. Their permissions, temporary naming, backup handling and directory-sync promises differ.

Evaluate [`atomic-write-file`](https://docs.rs/atomic-write-file/latest/atomic_write_file/) or the already-used `tempfile` building blocks through one small internal policy adapter. Do not collapse different durability guarantees into one falsely broad contract. Secret writes still require restrictive permissions and buffer treatment; graph saves still own backups. Single-file atomic replacement does not replace a multi-file transaction.

Done when the adapter preserves permissions, overwrite behavior, cleanup, backup ordering and the declared durability level on Windows/macOS/Linux, with injected write/rename failures and cold reopen. OS-specific guarantees remain explicit.

### 6. Reconsider hand-parsed URL canonicalization beneath durable identity

**Confidence: high for overlap, medium for migration feasibility.** Chartulary `crates/chartulary/src/canonical.rs:38,68,97` splits scheme, authority, host, port and query by hand. Its documentation says a URL crate is deliberately absent because WHATWG parsing belongs to the engine. The standalone [`url`](https://docs.rs/url/latest/url/) library can be used by a storage crate without depending on the engine.

The custom parts worth retaining are resource-identity policy: fragment treatment, tracking-parameter selection, meaningful path/query bytes and historical keys. The syntax mechanics should be supplied by a standard parser where possible, or the API should accept an already-validated parsed/canonical address. Merely lowercasing strings does not address IDNA, dot segments or the complete URL grammar.

**Migration constraint:** resource UUIDv5 values derive from these exact strings. A better normalizer can change durable identities. Version the normalization profile, preserve old identities/aliases, and test existing persisted fixtures before changing behavior. This is not an invitation to silently remint every graph node.

### 7. Delegate RDF syntax serialization

**Confidence: high. Scope: small.** Chartulary `crates/chartulary/src/rdf.rs:189-247` renders N-Quads through string formatting and a local literal escaper; `angle` simply wraps the IRI string in angle brackets. The larger Mere `mere-rdf` crate already depends on Oxigraph components for RDF/JSON-LD work.

Retain the graph-to-triple mapping and disclosure policy, but use [`oxttl`](https://docs.rs/oxttl/latest/oxttl/) / `oxrdf` for RDF term validation and N-Quads syntax. That removes a parallel standards serializer without importing a database. Test IRI delimiters/control characters, Unicode literals, language/datatype terms, deterministic output ordering and parser round trips. Do not claim a confirmed injection vulnerability without checking the input admission path.

### 8. DirectoryBackend: R32 recommendation withdrawn

Muniment `crates/muniment/src/directory_backend.rs` implements checksummed redo, replay, multi-step commit, truncation, locks, path restrictions and poisoned-state recovery. Redb and Fjall already exist as selectable backends. Turnstone's inspected direct feature set selects them, not DirectoryBackend.

The original recommendation to prefer a database for ordinary state is withdrawn from this audit. Human-readable incremental files and backend choice are architectural capabilities. No consumer/format migration preserving those requirements was shown. Recovery testing remains relevant to this backend, but its cost alone does not demonstrate an equivalent substitute.

### 9. Review Luggage's fork and staged-update trust before expanding it

**Confidence: high source concern, medium replacement fit.** Luggage is a documented derivative of cargo-packager-updater, not an updater invented from nothing. The custom staging path in `crates/luggage/src/staging.rs` saves an artifact and editable JSON containing its digest. `StagedUpdate::take_verified` compares against that saved digest; `Update::install_staged` then calls `install` without rechecking the original artifact signature against the current trusted `Update`.

Changing both the staged artifact and its saved digest is not rejected by that comparison alone. This matters under a threat model where staging can be modified; no executable install or exploit test was run. It is not a claim that Turnstone currently exposes this path (no direct caller appeared in inspected Turnstone source).

Before expanding this fork, compare the current upstream updater and a maintained signed-metadata client such as a TUF implementation. [TUF](https://theupdateframework.io/docs/overview/) addresses update metadata versions, expiration, trusted keys, rollback and compromise resilience; it does not replace our platform installer. At minimum keep/revalidate signed artifact identity across staging and bind the stage to the expected release and destination. Test tampering with artifact and metadata together, stale/replaced stages, mismatched versions/keys and interrupted publication.

### 10. Evaluate a grammar library for ESP constraints

**Confidence: high overlap, medium replacement fit.** ESP `crates/esp/src/infer/decoder/constraint.rs` owns a grammar machine, JSON-string acceptance, tokenizer alphabet inversion and allowed-token computation. [`llguidance`](https://github.com/guidance-ai/llguidance) offers a Rust grammar/token-mask implementation, Hugging Face tokenizer integration and forced-token advancement.

Keep the durable request grammar contract, acceptance semantics, model session identity and checkpoint policy. The exact byte/token history, sampling-draw accounting, private literal handling/zeroization, startup/masking costs and Wasm feature graph decide replacement feasibility. Extending support beyond the current verified ByteLevel tokenizer makes the case for reuse stronger; replacing the current narrow grammar is not yet qualified.

### 11. Replace the entropy shortcut; preserve the versioned sampler

**Confidence: high for the entropy API boundary; low urgency.** ESP `crates/esp/src/infer/decoder/sample.rs:32` obtains an unseeded generation seed by finishing a newly created `RandomState` hasher. The hash-map builder is an indirect and unspecified entropy contract. Use [`getrandom`](https://docs.rs/getrandom/latest/getrandom/) or `rand_core` for seed generation.

The deterministic SplitMix64 stream itself is tiny, versioned by behavior and tied to checkpoint `skip_draws`. Replacing it with a different PRNG merely to remove a few arithmetic lines can break replay for little benefit. Retain it or introduce an explicitly versioned RNG profile. Existing seeded generation and pause/resume results must stay exact; fresh entropy failures should be reported through the provider contract.

### 12. Model implementations: R31 recommendation withdrawn

The inspected ESP pin owns model architectures, imports, quantization, sampling, streaming and continuation while already buying Burn tensor/backend machinery. That is real owned scope, but not an entire numerical framework written from scratch.

The additional-native-provider recommendation is withdrawn. No supplier was demonstrated to preserve the shared wgpu device, Wasm target, bounded library-owned streaming, artifact/admission, private data, cancellation and continuation contract together. C/C++ suppliers are outside this review's constraints. An architectural argument could question that contract, but must name changed capabilities and migration/deployment costs rather than count model code as a free saving.

### 13. Retain small exact search; buy an approximate index only at demonstrated scale

**Decision: keep.** Eidetic's rebuildable BM25, field weighting, URL-exact lane, reports and rank fusion are small deterministic projection mechanics with a documented browser/Wasm and rebuild-cost rationale. ESP's `embed/index.rs` is explicitly a flat exact index, not a homebuilt ANN engine. Retrieval authorization, exact source-version identity and passage selectors are distinctive application guarantees.

Use existing Unicode segmentation (`genet-text` already delegates to `unicode-segmentation`). Benchmark real corpus size before adding an ANN backend; do not build an original HNSW implementation when one is needed. Do not trade complete admitted-set enumeration for approximate retrieval in structured/model-free answers.

### 14. Retain identity policy and portable storage boundaries

**Decision: keep with targeted primitive replacement above.** Personae delegates signing and sealing to ed25519-dalek and XChaCha20-Poly1305; Castellan uses Argon2, RustCrypto primitives, ssh-key, ssh-agent-lib and OS APIs. Notochord owns the project's admission/delegation semantics. Gaz's duplicate-name JSON visitor extends Serde admission rather than reimplementing a JSON lexer. Insigne's typed proofs, Chatelaine taxonomy and the separation of custody from secret-free views are product architecture.

Likewise Chartulary wraps petgraph, Muniment wraps redb/fjall/IndexedDB/zip, and Stickleback wraps p2panda. Replacing these facades wholesale would move policy into upstream-shaped code rather than remove algorithms. Their custom codecs and redundant standards formatting are the smaller useful cuts.

### 15. Turquet: meaningful model ownership; continue delegating standards foundations

**Corrected assessment: no provider substitution is proposed (R35).** Turnstone's actual pin is Turquet 0.13.0 `bc3c454`, not the newer cached 0.17.0. It is a history-preserving adoption of astro-rust; `Cargo.toml` uses hifitime for time scales and sofars for IAU orientation. ANISE is an optional verifier, not currently the runtime ephemeris provider at this pin.

The analytical VSOP87/ELP-style model and bounded event search have specific offline/embedded/data-size and provenance goals. An SPK/ANISE runtime provider would change the storage/data contract; no equivalent replacement benefit was demonstrated here. Avoid suggesting another astronomy library without comparing frames, timescales, supported epochs, errors, model/data licensing and Wasm/no_std requirements. The pinned AUDIT.md already distinguishes measured, corrected, example-only and unverified legacy calculations; an ordinary algebraic regression is not astronomical accuracy evidence.

## Coverage and limits

The above reviewed package boundaries, the named source implementations, their adjacent tests/documentation and consumer pins. It did not execute installs, timing attacks, model inference, browser conformance, filesystem crash tests or network/hardware tests. Candidate tests above are done conditions, not passing receipts. No repository source or checkout was changed.
