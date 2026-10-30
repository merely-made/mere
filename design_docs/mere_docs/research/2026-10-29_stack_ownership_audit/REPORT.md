# Stack ownership audit

2026-10-29

The useful reductions are duplicate or incomplete mechanics inside the chosen architecture. The original audit overreached by treating several architectural commitments as replacement candidates. That recommendation is withdrawn. R29, R30 and R33 are struck; the other unproven architectural alternatives below are outside the reuse queue.

Pure Rust, transport and protocol modularity, retained data with projections, and one wgpu device are requirements of this review. Component identity, hit testing, input and accessibility must survive projection across Graphshell. An alternative that solves only rendering or native execution has not preserved this design. A different architecture can be argued for, but the argument must name the capabilities, migration and costs together before it becomes proposed work.

An established dependency is useful when it removes an actual obligation at lower total cost. Smaller source files alone do not establish that. The stack already buys substantial parsing, rendering, storage, tensor, crypto and transport machinery.

## Scope and evidence

This is a broad subsystem-level ownership review with source and dependency inspection. The original instance-based searches missed related implementations, so this is not an exhaustive family-wide pattern census. It is not a line-by-line correctness audit, security certification or completed migration trial.

The [manifest census](../../../../support/receipts/stack-ownership-audit-20261029/inventory.json) inventories 23 local repositories/source mirrors and 721 package manifest records. That count includes examples, validation harnesses and upstream source mirrors, particularly Firefox; it does **not** mean 721 independent owned implementations were deeply reviewed. The [Turnstone lock census](../../../../support/receipts/stack-ownership-audit-20261029/turnstone-locked-sources.json) records the actual 22 Merely/fork Git source identities in the inspected consumer lockfile. Branch-main dependencies and latest supplier main are not assumed identical. The relocated audit-validation.json records the original snapshot’s sizes and link check before these document revisions.

Deep source review covered Genet/Netrender, Mere/Cambium/Graphshell, Knot, Eidetic, Dramatis/custody, Murm/Retinue, pinned Errand/Gemot/ESP/Armillary, and the relevant maintained forks. Sibling Woodshed/Isometry/Cleromancy code received dependency inventory and focused source checks. Turquet was checked at Turnstone's actual 0.13 pin. Full sibling domain-algorithm audits and physical radio/audio/browser/GPU/filesystem testing remain outside this pass.

Other work advanced refs during the review. Each detailed report records explicit reviewed revisions; the inventory is a recorded snapshot, not a promise that every source is still current when this is read. The original source-review snapshot changed no production code. Subsequent Dramatis branch work covers only the Personae MAC and Insigne codec subset; it is described below. Existing receipts describe the revisions actually tested.

Detailed evidence, exact paths, primary library documentation and deciding tests:

- [Engine, rendering and forks](engine.md)
- [GUI, editing, application infrastructure and sibling spotchecks](ui.md)
- [Transport, wire formats, radio and community](network.md)
- [Storage, identity, inference and Turnstone](storage-identity-inference.md)

Security reuse follow-up (2026-10-30): the [Dramatis/Castellan comparison](storage-identity-inference.md#security-reuse-follow-up-2026-10-30) maps custody, delegation and storage obligations to Bitwarden, COSE, age, UCAN, Biscuit and specialist services. Bitwarden’s local keystore is the strongest substantive partial candidate; COSE is the narrow standards-envelope candidate. Current artifacts, audit scope, native build costs and remaining authorization/freshness duties are explicit. No replacement is adopted or qualified.

## Reuse criteria

Candidates below need both a concrete benefit and a compatible cost. Prefer an already-used primitive or internal consolidation when it removes the same obligation.

| Check | What decides it |
|---|---|
| Obligation removed | Name the duplicate algorithm, missing standards behavior or recurring defect the adapter actually eliminates. Count the policy and glue that remain. |
| Complete dependency cost | Inspect new packages and features in the affected consumer, including default builds, cold build/fetch time, disk/artifact size, toolchains and ongoing updates. A sibling dependency is not necessarily already in this consumer's feature graph. |
| Target fit | Preserve MSRV 1.97.1, browser/Wasm and no_std/embedded cones where applicable. C/C++ and ANGLE are excluded from these proposals. Rust alone does not establish fit. |
| Behavioral fit | Preserve accepted/refused inputs, canonical signed bytes, bounds, secret handling, source provenance, durable identities and projection contracts. |
| Proportionate verification | Use existing independent vectors and relevant checks. Small primitive substitutions need a focused diff and results, not a mandatory plan, receipt directory and separate reviewer for each helper. |

“Evaluate” names a candidate, not an approved dependency. The original “replace” labels were too strong where dependency and adapter costs had not been established. No new product implementation is in progress under this audit.

### Small and reasonably direct reductions

| ID | Owned implementation | Recommendation | Why and what stays local |
|---|---|---|---|
| R01 | Authentication comparisons across Personae, Castellan and Murm | **Use dedicated constant-time primitives; family work remains open** | Dramatis branch work covers one Personae fixed-size MAC. Four additional handwritten comparisons are verified below. Keep length/refusal behavior and target features; no timing exploit was demonstrated. |
| R02 | Base32/Base64/Base58/hex/percent codecs across identity, transport and apps | **Evaluate/consolidate by codec family** | Insigne Base58/hex is implemented on a comparison branch only. Remaining Base32/Base64/percent copies are not complete. Compare adapters, canonical acceptance, buffers/zeroization and the actual added feature closure before choosing data-encoding, bs58, hex or percent-encoding. |
| R03 | Eidetic bookmark HTML tokenizer, attributes and entity table | **Evaluate html5ever tokenizer against a bounded parser repair** | First-`>` scanning and partial entity support are concrete gaps. Preserve bookmark structure, dates, folders and intake notices. html5ever is not already in Eidetic’s inspected closure; its package/build/target cost is unmeasured. Plan is on hold, with no parser or dependency change. |
| R04 | Netfetcher HTTP freshness/revalidation policy | **Evaluate http-cache-semantics** | Missing Vary and heuristic freshness are concrete coverage gaps. Compare supported behavior and adapter/dependency cost while retaining browser cache modes, partitioning, limits and storage. |
| R05 | Turnstone Content-Disposition and filename* parsing | **Correct the syntax; evaluate a focused parser** | Semicolon splitting and discarded charset/language are concrete gaps. Keep filename safety and custody. Select reuse only after quoted/extended-parameter tests and dependency comparison; a parser has not been chosen. |
| R06 | Chartulary handwritten N-Quads formatting | **Evaluate oxrdf/oxttl syntax handling** | Existing Mere RDF use offers possible reuse, not proof of a cheap Chartulary feature graph. Preserve mapping, disclosure, ordering and valid IRI/literal behavior; compare the exact closure before adoption. |
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

### Durable formats and existing supplier maintenance

| ID | Owned implementation | Recommendation | Constraint |
|---|---|---|---|
| R23 | Chartulary manual URL canonicalization used for resource UUIDs | **Evaluate syntax reuse without changing identity policy** | A standalone parser or validated parsed input could remove grammar work. Different normalization bytes change durable UUIDs. Establish the intended profile and migration requirement before changing behavior; adding url is not automatically a compatible fix. |
| R24 | Luggage staging trust | **Review signed artifact binding across staging** | A mutable saved digest is not renewed signature verification. Establish callers and threat model, then preserve trusted release identity through installation. A TUF/platform-updater migration is not required to investigate this concrete concern. No live exploit was established. |
| R36 | Maintained upstream forks | **Review local deltas when refreshing suppliers** | Retire patches only when upstream preserves the same shared-device, layout, access, target and lifecycle behavior. Use existing provenance records rather than add a new reporting system. |

### Outside the reuse queue

These IDs remain for traceability. They are not implementation instructions or scheduled prototypes. A stack-level argument could reopen one, but this audit has not established it. Scope is upstream of cost comparisons: reducing supported capabilities changes the problem being solved, so it cannot be counted as a like-for-like ownership saving.

| ID | Architectural choice | Corrected assessment |
|---|---|---|
| R25 | Gemot group/auth/key integration | No demonstrated p2panda-spaces cut preserves retained authority, offline revocation, key profiles and Muniment storage. Narrow primitive reuse remains possible; orchestration replacement is unproven. |
| R26 | Causal processing/replay/pruning | Existing Stickleback seams already consolidate work. High-level p2panda's storage and admission assumptions have not been reconciled with the stack. Withdraw the orchestration replacement recommendation. |
| R27 | Cadency selector parser/matcher | A Rust standards library is a possible comparator, but this review did not establish its feature/build cone, invalidation seam or retained semantics. Withdraw the recommendation to reconsider ownership on size alone. |
| R28 | WHATWG Streams host/runtime algorithms | No compatible polyfill/host integration was demonstrated across realms, brands, callbacks, microtasks and bounds. Withdraw the replacement recommendation. |
| R29 | Genet ESSL/WebGL compiler path | ~~Investigate ANGLE as a replacement~~. **Struck.** C++ conflicts with the pure-Rust constraint; emitting SPIR-V does not establish stack compatibility. |
| R30 | Cambium controls and shell | ~~Prototype replacing common controls through egui/Iced~~. **Struck.** GPU embedding does not show that stable retained identity, component semantics, hit testing and accessibility can cross Graphshell. No replacement case was made. |
| R31 | ESP model architectures and execution | An established native provider has not preserved the unified device, Wasm, bounded library-owned streaming, privacy or continuation contract. Withdraw the additional-provider recommendation from this queue. |
| R32 | Muniment DirectoryBackend | Backend selection is an explicit product/storage choice. Redb/fjall already exist. Source size and recovery complexity do not establish that readable incremental files can be replaced; withdraw the generic preference. |
| R33 | Woodshed WSOLA | ~~Evaluate SoundTouch/Signalsmith as replacements~~. **Struck.** Proposed C++ suppliers do not meet the language constraint; no compatible source-clock/quality benefit was demonstrated. |
| R34 | Direct radio protocols | Native Python/stock sidecars change deployment, custody and direct/no_std capability. Withdraw the sidecar recommendation; generic bounded wire primitives are separate candidates. |
| R35 | Turquet model/data providers | Kernel-based alternatives change offline, data-size and embedded contracts. No provider substitution is proposed by this audit. |
| R40 | Vano and Boa adapters | Different retained/snapshot and target capabilities were identified, but no equivalent consolidation was shown. Withdraw the consolidation recommendation. |
| R41 | Genet and general-web coverage | Existing full-engine adapters do not demonstrate equivalent DOM access, projections, portability or shared-resource behavior. Withdraw the full-engine substitution argument. Define any changed capability scope explicitly before arguing its architecture or cost. |

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

## Corrections and current implementation scope

The original review counted several individual instances as if they covered a pattern. R01 and R02 are family findings and remain open. A targeted recheck on 2026-10-29 verified these additional handwritten authentication comparisons:

| Source revision | Location | Observation |
|---|---|---|
| Mere `c6290142866eefead385e728c5c5da24ac6f624a` | `ports/djinn/castellan/src/otp/mod.rs:381` | Slice/XOR helper. Castellan already directly depends on constant_time_eq 0.4.2 and uses it in two custody storage files. |
| Murm/Retinue `7db6fe66efa9d454c77ccad2763862632f300121` | `native/crates/webrtc-carrier/src/codec.rs:27` | XOR accumulator followed by black_box. |
| Same Murm revision | `crates/radio-hand/src/control/position_disclosure.rs:407` | Fixed-array tag comparison using XOR accumulation. |
| Same Murm revision | `crates/retinue/src/ifac.rs:242` | Handwritten XOR/OR fold comparing a signature-derived IFAC suffix. |

This is a verified minimum, not a claim of complete pattern coverage. The remaining pass should search implementations and call sites across supported source owners and pins, rather than close R01 after one supplier patch. R02 likewise needs codec-family coverage and actual consumer graphs.

Dramatis branch `stack-reduction-identity-20261029` contains the Personae MAC and Insigne Base58/hex substitutions. Pre-merge Linux supplier tests passed on the product tree at `27b731f`; review/docs followed through `fd2c3a5`. The branch then merged upstream `051e95a` at `e79db34`; the queued merged-head gate was stopped. That merged revision is not requalified. No consumer pin adoption or MSRV/Wasm/runtime assurance is implied.

Other useful corrections remain: Armillary's actor harness is active while the reviewed reusable Pool callers were fixtures/probes; iroh encrypts endpoint traffic end to end through relays, so relay confidentiality does not justify additional Noise by itself. Persona/session identity and carrier-neutral contracts may still justify that layer.

## Follow-up order

Keep the existing Dramatis diff available for comparison. Next source review should finish the comparison/codec pattern census and inspect existing in-family reuse such as Backstep, already-used RDF/color primitives and unused Pool callers. HTTP cache/header/bookmark gaps deserve a benefit-versus-cost comparison. These are candidate reviews, not an automatic implementation sequence. Luggage's concrete trust concern should be assessed on its own merits rather than made contingent on choosing a larger updater framework.

The work is paused for architectural correction. New dependencies or implementations should follow an explicit choice of candidate. Tiny accepted substitutions can use ordinary commits and relevant tests; broad adapters need only the extra evidence their risks require.

## Remaining limits

The original review was source-only. It did not measure competing dependency/build graphs, qualify replacement adapters, certify security, or establish runtime, browser, hardware or conformance parity. Later supplier tests cover only the exact Dramatis subset and revisions above. Raw inventories are historical evidence under support/receipts, not an active design specification. Third-party licensing/advisories and unreviewed sibling algorithms were not exhaustively audited.
