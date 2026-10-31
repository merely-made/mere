# Stack ownership audit

2026-10-29

The stack has more avoidable ownership than the initial five-candidate review found. Most of the clear reductions are standards mechanics embedded inside otherwise useful project libraries. The largest maintenance commitments are the CSS selector engine, shader compiler, ordinary GUI controls, model implementations, custom transactional directory storage and independent radio protocols. Those deserve explicit product decisions rather than being treated as incidental infrastructure.

The useful default is: own authority, retained identity, projections and product behavior; purchase standardized parsing, encoding, numerical conversions and execution machinery where a compatible implementation exists. A dependency count is not a maintenance-cost measure. A small standards implementation copied into six crates is still six things that can drift.

## Scope and evidence

This is a comprehensive subsystem-level ownership audit, with source and dependency inspection. It is not a line-by-line correctness audit, security certification or completed migration trial.

The [manifest census](inventory.json) inventories 23 local repositories/source mirrors and 721 package manifest records. That count includes examples, validation harnesses and upstream source mirrors, particularly Firefox; it does **not** mean 721 independent owned implementations were deeply reviewed. The [Turnstone lock census](turnstone-locked-sources.json) records the actual 22 Merely/fork Git source identities in the inspected consumer lockfile. Branch-main dependencies and latest supplier main are not assumed identical.

Deep source review covered Genet/Netrender, Mere/Cambium/Graphshell, Knot, Eidetic, Dramatis/custody, Murm/Retinue, pinned Errand/Gemot/ESP/Armillary, and the relevant maintained forks. Sibling Woodshed/Isometry/Cleromancy code received dependency inventory and focused source checks. Turquet was checked at Turnstone's actual 0.13 pin. Full sibling domain-algorithm audits and physical radio/audio/browser/GPU/filesystem testing remain outside this pass.

Other work advanced refs during the review. Each detailed report records explicit reviewed revisions; the inventory is a recorded snapshot, not a promise that every source is still current when this is read. No production source, checkout or dependency pins were changed. Existing receipts describe historical qualification only.

Detailed evidence, exact paths, primary library documentation and deciding tests:

- [Engine, rendering and forks](engine.md)
- [GUI, editing, application infrastructure and sibling spotchecks](ui.md)
- [Transport, wire formats, radio and community](network.md)
- [Storage, identity, inference and Turnstone](storage-identity-inference.md)

## Ranked decisions

“Replace” means the ownership reduction has strong justification; compatibility and target tests still have to pass. “Evaluate” means material constraints could defeat the proposed library. “Reconsider” means the decision changes a product or architectural commitment. None of these is a claim that a replacement was implemented or qualified.

### Small and reasonably direct reductions

| ID | Owned implementation | Recommendation | Why and what stays local |
|---|---|---|---|
| R01 | Personae freshness-ledger MAC equality | **Replace** with dedicated constant-time comparison | A handwritten XOR loop is an unnecessary cryptographic implementation claim. Keep fixed-length MAC types and failure handling. No timing exploit was demonstrated. |
| R02 | Base32/Base64/Base58/hex/percent codecs across identity, transport and apps | **Replace/consolidate** on data-encoding, bs58 and percent-encoding | Custom alphabets, bounded buffers and strict acceptance can live in adapters; they do not require independent codec algorithms. Preserve malformed-input and zeroization policy. |
| R03 | Eidetic bookmark HTML tokenizer, attributes and entity table | **Replace** with html5ever tokenizer | Current first-`>` scan ignores quoted delimiters; named-entity support is a small subset. Keep bookmark structure, dates, folder identity and intake notices. |
| R04 | Netfetcher HTTP freshness/revalidation policy | **Replace** with http-cache-semantics after an adapter gate | Existing implementation deliberately defers Vary, heuristic freshness and more standards behavior. Keep browser cache modes, profile partitioning, size limits and storage. |
| R05 | Turnstone Content-Disposition and filename* parsing | **Replace** with a proven HTTP-aware parser plus percent-encoding | Semicolon splitting and discarded charset/language are compatibility debt. Keep filename safety, destination policy and source custody. Exact parser selection needs corpus testing. |
| R06 | Chartulary handwritten N-Quads formatting | **Replace** with oxrdf/oxttl syntax handling | Mere's RDF tier already buys this ecosystem. Keep graph-to-triple and disclosure semantics. Validate arbitrary IRI/literal inputs and deterministic ordering. |
| R07 | Tinct and Livery color-space arithmetic | **Evaluate toward replacement** using already-present color, or palette | Theme seed/token policy is distinctive; transfer functions, matrices and standard color spaces are not. Tinct f64 versus color f32, gamut rules and palette snapshots must be checked. |
| R08 | Woodshed OverviewFolds snapshot undo stacks | **Consolidate internally** on Backstep | Shared bounded/coalesced undo already exists. Keep the document model and saved-state policy. |
| R09 | Armillary's reusable growable worker Pool | **Remove if the remaining caller census confirms unused** | Reviewed live Mere/Turnstone/ESP/Gemot paths use the actor harness rather than this pool; visible consumers are fixtures/probes. Keep active actor/message and task-scope contracts. Public downstream users may require a deprecation. |
| R10 | ESP unseeded RNG seed derived through RandomState hashing | **Replace the entropy shortcut** with getrandom/rand_core | Preserve the pinned deterministic sampler and draw-position continuation. A hash-map implementation detail should not be the entropy API. |

### Substantial standards work with plausible library alternatives

| ID | Owned implementation | Recommendation | Main compatibility gate |
|---|---|---|---|
| R11 | Errand RSS/Atom semantic walker, HTML stripping, separate consumer JSON Feed parsing | **Evaluate feed-rs**, with an existing HTML tokenizer for plain-text extraction | Preserve chapter/transcript resources, raw provenance, original dates, absent versus synthesized IDs, namespace semantics, diagnostics and limits. quick-xml is already used, so merely adding it again solves nothing. |
| R12 | Netfetcher cookie storage/matching/expiry/order | **Evaluate cookie_store** beneath browser policy | SameSite, HttpOnly/script access, prefixes, public-suffix enforcement, persistence and profile isolation remain explicit. A generic HTTP jar is not full browser-cookie policy. |
| R13 | Graphshell MCP stdio, JSON-RPC and protocol-era machinery | **Evaluate official Rust MCP SDK** | Preserve admitted-route authority, stable invoke envelope, bounds and compatibility with the exact protocol revisions currently served. SDK support may lag the newest era. |
| R14 | Retinue/Outrider bounded MessagePack primitives and skippers | **Evaluate rmp low-level primitives** | Preserve no_std, canonical signed bytes, nesting/length ceilings and preallocation refusals. Avoid replacing a bounded subset with an unbounded Value tree. |
| R15 | Sennet schema-free protobuf scanner/writer | **Evaluate a supported generic wire reader** | Does not require importing Meshtastic schemas. Preserve field observations, overflow/truncation rules, borrowed buffers and firmware size. Prost's internal encoding API is not a stable seam. |
| R16 | Remaining independent Retinue KISS encoder/deframer | **Consolidate internally** on Selvage | Tulle/firmware already share Selvage; only residual duplication counts. Keep command-byte and flow/beacon policy. |
| R17 | String-based editing and whole-document text history | **Evaluate text-buffer/editor components when scale requires them** | Ropey/crop or an existing Parley editing seam could reduce indexing/storage work. Preserve grapheme/UTF-16/byte offsets, IME, selection, local undo and exact saved state. A rope alone does not supply an editor. |
| R18 | Knot three-way diff/merge mechanics | **Evaluate established diff primitives**, retain merge policy | This is causal whole-document collaboration, not an independently invented per-character CRDT. Preserve deterministic conflict hunks and authored lineage; a CRDT swap is a different design. |
| R19 | Timed-text's handwritten WebVTT subset | **Evaluate a WebVTT parser before broadening** | Preserve source offsets, timestamp-map/cue semantics, malformed input behavior and selected subset diagnostics. Small fixed subsets can remain appropriate. |
| R20 | ESP output grammar machine/tokenizer byte mapping | **Evaluate llguidance** | Check byte/token continuation, forced tokens, sampling draws, private grammar data, Wasm feature graph and costs. Durable acceptance/replay rules stay with ESP. |
| R21 | Parley custom GSEGDIC1 dictionary data/provider format | **Evaluate ICU4X provider/blob tooling** | Custom format is not automatically needed for custom segmentation policy. Verify dictionary algorithms, script coverage, data revision, binary size, load cost, host/browser targets and offline use. |
| R22 | HTTP/3 ad hoc resolver/connection orchestration | **Reuse host async resolution/connector policy** | Current path blocks on ToSocketAddrs, chooses the first address, binds IPv4, creates an endpoint per request and collects the body. This is integration debt, not a handwritten DNS engine. Verify IPv6, address fallback, deadlines, reuse and bounded streaming. |
| R37 | Genet WebIDL subset lexer/parser in a development tool | **Evaluate toward replacement** with Weedle | Keep reflection overrides, exposure and tag mapping. Compare generated interface tables over the complete vendored IDL corpus. A build-tool parser does not add a shipped browser runtime dependency. |
| R38 | Separate generated Unicode Bidi_Class directionality table | **Evaluate shared ICU properties** | Preserve Unicode 17 defaults and HTML first-strong rules. Keep the small table if sharing ICU would enlarge otherwise-light DOM consumers materially. |
| R39 | Retained Web Audio analyzer's quadratic DFT | **Replace the numerical kernel if the lane is activated** with RealFFT/RustFFT | The current scripted runtime has no bound AudioContext using it. Preserve Web Audio window/smoothing/scaling and no callback allocation; this is not an established live bottleneck. |

### Decisions with durable formats, authority or major product consequences

| ID | Owned implementation | Recommendation | Why a mechanical swap would be wrong |
|---|---|---|---|
| R23 | Chartulary manual URL canonicalization used for resource UUIDs | **Reconsider syntax ownership**, use standalone url or validated parsed input | Tracking/identity policy is ours; URL grammar need not be. Changing normalized bytes changes durable UUIDs, so versioning and old-key aliases are required. |
| R24 | Luggage updater fork and added staging trust | **Review before expanding; compare current upstream/TUF components** | Saved mutable digest checking is not renewed signature verification. Preserve trusted release identity through staging. TUF concerns metadata/key/rollback safety; platform installation is a separate purchase. No live consumer exploit was established. |
| R25 | Gemot generic group/auth/key orchestration | **Evaluate p2panda-spaces already present in the supplier** | Keep constitution, quorum, capability and durable-history semantics. Preserve offline revocation, encrypted history profiles and Muniment storage. Upstream APIs are unstable. |
| R26 | Generic causal processing/replay/pruning orchestration | **Evaluate high-level p2panda pieces narrowly** | Existing shared Stickleback seams already remove duplication. Upstream's SQLite/native assumptions cannot silently replace browser storage, policy-before-insert or carrier-neutral authority. |
| R27 | Cadency CSS selector parser/matcher | **Reconsider owning a standards engine** | Servo selectors is a real alternative. A previous ruling to remove it is historical context, not proof that ownership is economical. Compare omitted syntax, quirks, specificity, invalidation, performance and smaller feature cones. |
| R28 | WHATWG Streams algorithms in engine bootstrap | **Evaluate a pinned reference implementation/polyfill** | Preserve realms, native brands, host callbacks, byte streams, microtasks and refusal boundaries. A standards polyfill may be reusable, but this is not copy-and-paste equivalence. |
| R29 | Genet ESSL parser/checker/WebGL validator/lowering | **Reconsider this as a standalone compiler commitment** | ANGLE's independent translator can emit SPIR-V without taking GPU-device ownership. It adds C++ and target/build complexity. Pure Rust/Wasm is a real reason to retain, with a correspondingly real CTS/differential burden. |
| R30 | Ordinary application widgets and shell in Cambium | **Prototype buying common controls through egui/Iced** | Rootstock already has caller-owned-device native adapters. The shared GPU device alone is not sufficient justification for every widget. Browser parity, IME, accessibility, text/selection, projection and retained semantics are decisive. |
| R31 | ESP model architectures, import formats, quantization, BERT and decoder optimization | **Budget explicitly; evaluate an established additional native provider** | Burn already supplies tensors, so this is model-compatibility ownership rather than a numerical engine from scratch. Shared wgpu, browser, library-owned bounded streaming and continuation may justify some retained implementations. |
| R32 | Muniment DirectoryBackend redo log/transaction/recovery engine | **Prefer redb/fjall for ordinary authoritative state** | Retain custom directory recovery only where readable incremental files are an actual product requirement. Turnstone's inspected direct features are redb/fjall, not directory. File export and a transactional store are different needs. |
| R33 | Woodshed WSOLA time-stretch DSP | **Evaluate maintained audio-quality implementations** | SoundTouch/Signalsmith introduce language, license, Wasm and source-clock tradeoffs. Keep product timing semantics; decide with blinded/audio-quality, real-time and clock tests. |
| R34 | Independent Reticulum/LXMF/Meshtastic/MeshCore implementations | **Keep embedded scope deliberate; consider stock desktop sidecar modes** | These are large compatibility projects. Python/stock firmware reduces desktop burden but cannot replace direct PHY/no_std or preserve every custody boundary automatically. Buy generic primitives inside them. |
| R35 | Turquet analytical ephemeris and event algorithms | **Keep the provider contract; compare model/data choices per consumer** | It already delegates hifitime and SOFA-derived orientation. A desktop SPK provider might reduce analytical maintenance but changes kernel/data/embedded requirements. Use the actual 0.13 consumer pin in comparisons. |
| R36 | Maintained upstream forks as a portfolio | **Track each local delta and its retirement condition** | Several forks have real shared-device, layout or access boundaries; several are narrow patches. Removing a source pin does not prove upstream provides the same behavior. See engine/UI/network reports for actual deltas. |
| R40 | Maintaining Vano and Boa adapters/forks together | **Reconsider consolidation only with explicit target tradeoffs** | Vano's data-oriented heap/snapshots and Boa's Wasm/32-bit coverage are material differences. Both already reuse parsers, regex and standards libraries. Retire absorbed fork patches rather than propose an additional interpreter. |
| R41 | Supplying general website compatibility through the owned Genet web engine | **Reconsider the required scope against the full engines already integrated** | Graft/Servo, Scry/system engines and Weld/CEF already provide alternative producer boundaries. Using them more could avoid browser standards work, at the cost of pure Rust, inspectable DOM, target/runtime and exact shared-resource guarantees. Prove current backend behavior before promising a default or fallback. This is the broadest product-cost decision, not a recommendation to discard Genet. |

## What should remain ours

| Area | Existing purchase | Justified owned boundary |
|---|---|---|
| Durable bytes and graph structure | redb, fjall, IndexedDB, zip, petgraph, slotmap | Muniment/Chartulary APIs, resource identity, domain models and explicit backend selection |
| Replicated operations and encryption | p2panda primitives, groups and sync | Admission before insertion, retention and epoch policy, deterministic materialization |
| Identity and custody | dalek, XChaCha20-Poly1305, Argon2, ssh-key, ssh-agent-lib, OS APIs | Personas, delegation, explicit release/approval, secret-free read models |
| Internet transport | iroh/quinn, snow, str0m, Hyper/rustls/h3/tungstenite | Carrier selection, persona/carrier identity separation, session admission, bounded queues |
| Browser networking | HTTP/TLS engines plus cookie and parser libraries | Fetch/CORS/referrer/security context, per-profile state and policy |
| Search | Unicode segmentation and existing tensor backends | Small exact BM25/flat search, admitted corpus enumeration, source versions, citation selectors and ranking policy |
| Rendering | wgpu, Vello, Naga, Taffy, Parley, skrifa, fontique | One device/resource lifecycle, producer admission, browser layout semantics and presentation contracts |
| Graph views | petgraph, Rapier/Burn and toolkit backends | Scenograph truth/projection split, arrangement policy, field laws, interactions and product commands |
| Editing and collaboration | existing text/layout/diff components where compatible | Authored document model, exact source persistence, causal lineage and conflict decisions |
| Community | shared auth/crypto/replication engines | Constitution, electorate, quorum, capabilities, privacy and federation semantics |
| Sibling products | CPAL/Firewheel, MIDI/decoder/DSP crates, Rapier, astronomy standards foundations | Musical/game/research semantics and reproducible domain facts |

The presence of a wrapper crate is not itself a reinvention. Most of the stack already purchases large and difficult primitives. Removing small adapters just to reduce the crate list would obscure ownership without removing the underlying obligations.

## Corrections to the initial pass

1. Armillary's worker-pool rationale was previously accepted from its documentation. A current live-callsite search did not establish that the reusable pool is used in the reviewed applications. The active actor harness should be distinguished from that optional pool.
2. The initial pass did not surface the custom CSS selector engine, Streams algorithms, dictionary data format, text-buffer/merge layers, ordinary widget surface, protobuf/MessagePack engines or directory transaction log. Those are more substantial than a few small encoding helpers.
3. Iroh already encrypts endpoint traffic end to end through relays. The contrary implication in `transport/src/noise.rs` is inaccurate. Additional Noise can still have persona/session-identity or carrier-neutrality reasons; basic iroh relay confidentiality is not one.
4. A single wgpu device does not inherently require a custom ordinary-widget toolkit or shader translator. Existing embedding/translation seams show potential alternatives; target/build/accessibility parity remains unqualified.

## Suggested order and done conditions

1. Replace the MAC equality primitive, codec copies, bookmark tokenizer, header parser and RDF serializer. Done when old independent vectors and rejection behavior pass through library adapters and redundant algorithms are deleted.
2. Establish one narrowly specified atomic-file adapter, shared color calculations and shared snapshot undo; finish the worker-pool caller census. Done when actual consumers use the shared path and platform/precision/history behavior is preserved.
3. Trial HTTP caching, feeds, cookies and MCP against existing acceptance corpora. Done when provenance, bounds, browser policy and protocol-era compatibility are demonstrated, including previously unsupported standards cases.
4. Compare MessagePack/protobuf primitives, ICU data loading, llguidance, text/diff primitives and HTTP/3 orchestration. Done when the relevant no_std/browser/memory/security semantics hold and a meaningful amount of machinery is removed.
5. Make separate cost decisions for selectors, Streams, ESSL, ordinary GUI controls, model implementations, directory transactions and radio scope. Each needs a bounded competing prototype and a measured retained benefit. Previous specifications should inform that comparison rather than decide it in advance.

## Remaining limits

No replacement dependency was built, no current test suite was run, and no new runtime/hardware/conformance qualification is implied. This pass establishes where ownership appears avoidable and exactly what would have to survive a change. Third-party vulnerability advisories, licensing compatibility and the full algorithms of every sibling application need their own scoped reviews if they become deciding factors. The saved manifest/lock inventories make the remaining coverage visible rather than treating an unchecked area as clean.
