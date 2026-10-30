# UI, editor, shell and execution ownership audit

**Revision, 2026-10-29:** the [corrected audit](REPORT.md) controls the recommendations. This appendix retains source evidence at the revisions below. Established suppliers and source size do not establish net benefit. Compare the complete consumer feature/build/target cost, retained adapter work and actual behavior removed. Architectural substitutions below are withdrawn from the reuse queue; they would need a coherent stack-level argument. No new implementation is authorized by this appendix.

Read-only source audit, 2026-10-29. No builds, runtime tests, repository edits, commits or checkout changes were performed. Sibling origin/main refs were refreshed. Findings below are replacement candidates and architectural assessments, not qualified adapters.

## Frozen evidence

- Mere: `f631d1ec8909b978f1a31bce9e30f3e5bf65be78`. Initial inspection began at `f8469dd9a524c38b89b7c10aa8756151e27c5db7`; all UI implementation files discussed below are identical between these revisions, verified with `git diff --name-only`. Later source reads use the immutable f631 SHA. The moving ref advanced during the audit.
- Knot: `586bed467b328c589fbd54e5a0686b0be05aa242`.
- Woodshed: `a4356b74084e9016f6fdb9a7c6b8130def547021`.
- Isometry: `19e9a9880462b40ebe24f2dfdd4a3364e7f7270a`.
- Cleromancy: `e53e3d4f3621be986f5ecb2e3400d3285d7e4440`.
- Merecat/Turnstone: `a1a52f3bcbf6aa06b025df195843794f85aaf9ff`.
- Armillary consumer pin inspected from Cargo cache: `864f107d44bf759111ad2ea3c9036b2cfbf03f4c`.
- Firewheel consumer fork inspected from Cargo cache: `2b03846d2e0dcc6ded2a85367bb8c5071ae99cdc`, compared with upstream base `34b35cad0535bf0a9542ac977b34744805d6cc03`.

Paths below are repository-relative and must be read with the revision above. Local working trees are often substantially behind these sources. Static file inventories include embedded unit tests and are maintenance-surface indicators, not production LOC.

## Main conclusion

The review found possible local reuse in color arithmetic, duplicate fold undo and bounded text/protocol mechanics. Its proposed shell/toolkit comparison did not establish the stack's central projection contract and is withdrawn, along with the C++ DSP supplier recommendation.

Retained component semantics and stable identity must cross Graphshell together with hit testing, input and accessibility. Existing native GPU embedding demonstrates only one part of that contract. Neither widget counts nor device sharing establishes that an alternative toolkit can project those semantics without retaining or rebuilding the same machinery.

## Ranked findings

| Rank | Owned machinery | Verdict | Confidence | Candidate / action | Decisive qualification |
|---|---|---|---|---|---|
| 1 | Tinct sRGB transfer, HSL/OKLCH transforms and WCAG arithmetic | Replace internal arithmetic | High | Reuse workspace `color` where precision permits, or `palette` for f64 + contrast; retain Tinct seed/token/gating API | Exact built-in token outputs or explicit reviewed drift; contrast thresholds; hue wrap; neutral/NaN/out-of-gamut behavior; native/Wasm comparison |
| 2 | Woodshed view-local fold undo stacks | Replace duplicate with existing Backstep | High | `backstep::History<Vec<FoldRecord>>` with explicit cap 64 | Create/expand/undo/redo/reopen, new edits after undo, host selection reset, existing cap behavior |
| 3 | MCP JSON-RPC stdio framing/version negotiation/resource/tool envelope | Evaluate official `rmcp` adapter | High that duplicated standards surface exists; medium drop-in fit | Keep route/carrier admission and tools; delegate MCP lifecycle/serialization/transport where SDK supports required eras | Legacy and 2026-07-28 discovery/version contract; notification/batch policy; unknown versions; EOF and grant expiry; tools/resources refusal equivalence |
| 4 | Whole-string editor buffer plus repeated grapheme scans and whole-buffer snapshots | Evaluate rope-backed document path | High structural finding; medium replacement scope | `crop` byte-index rope or Ropey; preserve short text fields; retain grapheme/IME policy and Backstep grouping | Multi-MiB mid-document edit/caret/undo memory and latency, UTF-8 byte proposal boundary, ZWJ/combining/bidi, incremental highlight and exact save |
| 5 | Ordinary widgets/chrome/focus/scroll/host assembly | **Withdrawn, R30 struck** | No compatible semantic projection established | Retain as an architectural observation, not a toolkit migration | Any future case must preserve stable identity and component semantics across Graphshell, hit testing, a11y, input and native/browser targets |
| 6 | Knot generic line-based three-way merge fallback | Evaluate `diffy` | High duplicated primitive; medium semantic fit | Replace generic line reconciliation only, retain Djot block alignment and causal history | Same-position insertions, edits on range boundaries, identical edits, CRLF/no-final-newline, conflicts remain explicit, byte-preserving output and deterministic head hash |
| 7 | Unused Armillary public adaptive Pool | Trim if complete consumer census confirms | Medium-high lexical census; no dynamic or external-user proof | Remove/restrict unsupported pool API, retain active actor harness; do not replace unused API with another executor | Consumer/API inventory at every supported pin, fixture migration, external publication review; if retained, rerun worker lifetime/churn/TLS reason |
| 8 | Meristem fork of Xilem core | Evaluate upstream convergence, keep required extensions today | High source mismatch | Propose upstream splice moves, then consume upstream or keep small reviewed patch; avoid treating removed optional APIs as fork necessity | Keyed reorder/cross-parent portable move retains DOM/node/focus state; action routing; unchanged supported application behaviors |
| 9 | Parley-related text navigation/editing | Evaluate narrower upstream editor reuse | Medium | Compare `parley::editing::PlainEditor` for simple fields or lower-level selection APIs | Single-style editor limitation, highlighting, preedit ghost state, undo boundaries, bidi/soft wrap and source-byte compatibility |
| 10 | Woodshed WSOLA time stretching | **Withdrawn, R33 struck** | Proposed suppliers violate the language constraint | No SoundTouch/Signalsmith dependency or experiment proposed | Source-clock, quality and real-time behavior remain obligations of any separately argued alternative |
| 11 | Woodshed WebVTT transcript subset | Evaluate parser library | Medium | `vtt` parser, keep transcript extraction policy, original payload and bounded fetching | Malformed-block recovery, BOM/CR/NUL, cue IDs/settings preserved, cue markup/entity behavior, overlaps, size limits and Wasm |
| 12 | Workbench split/tab/float reducer | Keep the host-neutral projected reducer | No compatible substitution shown | Toolkit-owned dock trees have not demonstrated the same identity and authority boundary | Serialization, stable IDs, close/tearout effects, docking and native/browser input |
| 13 | Hest command declarations/menu choices/invocation outcomes | Keep; buy future fuzzy matching | High | `nucleo` if fuzzy palette is required; no reason to replace declarations | Preserve disabled reasons, destructive/effect metadata, MRU/personal ordering; search recall/latency if matcher changes |
| 14 | Scene contracts, arrangements, layout/physics composition | Keep domain contracts; evaluate individual numeric kernels as needed | High | Existing Petgraph/Rapier/Taffy already provide substrate; do not replace with whole scene/game engine by default | Determinism, pinned/anchored/seeded semantics, source/instance identity, measured footprints, Wasm, exact saved scene reconstruction |
| 15 | Firewheel bounded stream extension | Keep narrow fork while seeking upstream path | High current difference; retirement unqualified | Upstream stream nodes/streamed sampler plus fixed-resample fix when equivalent | Exact accepted/presented/queued source clocks, partial pushes, stale generations, EOF material, zero callback allocations/deallocations |
| 16 | Backstep shared history, source-edit proposals, scene JSON, syntax parser backend, a11y adapter glue | Keep small policy/glue over libraries | High | Already using Serde, Jotdown, Logos, AccessKit, Parley, winit, Wasmtime, Rhai | Maintain existing policy tests; do not create new parallel implementations |

## Exact evidence and scope of each cut

### 1. Color arithmetic is standard machinery inside a useful policy crate

Mere `crates/cambium/tinct/src/lib.rs:335-353` implements contrast, relative luminance and the sRGB transfer function. `:380-423` implements HSL conversion; `:429-520` owns OKLCH/sRGB matrices, transfer functions and gamut clamping. The crate's only dependency is Serde (`crates/cambium/tinct/Cargo.toml:15-16`). This duplicates established numeric color operations. The public `Srgb` DTO, `Seeds`, mode-specific role mapping, brand choices and contrast-gate policy remain useful local work.

Mere already declares `color = "0.3"` (`Cargo.toml:581`). That package provides CSS-oriented conversions and interpolation but uses f32 throughout, whereas Tinct's Oklch API uses f64. This precision mismatch means using the existing dependency is not automatically the best numerical choice. `palette` provides Oklch conversions and `Wcag21RelativeContrast` and is a stronger alternative for preserving f64 calculations. The gate should compare outputs, clamping and threshold decisions rather than assume mathematical equivalence implies identical theme pixels. See [color scope and precision](https://docs.rs/color/latest/color/), [Palette Oklch](https://docs.rs/palette/latest/palette/struct.Oklch.html), and [Palette contrast](https://docs.rs/palette/latest/palette/color_difference/trait.Wcag21RelativeContrast.html).

Tabard `crates/system/tabard/src/theme/seed.rs:7-24,48-65` adds role policy and an iterative contrast search over Tinct. That is an adapter/policy layer, not a second color-science engine to throw away.

### 2. Undo duplication already has an in-family supplier

Woodshed `crates/woodshed-views/src/stage/overview_folds.rs:13-53` owns `undo: Vec<Vec<FoldRecord>>`, `redo`, snapshot push, a cap of 64 and redo clearing. `:138-155` implements the symmetric pop/replace/push path. Mere Backstep `crates/cambium/backstep/src/lib.rs:14-39,60-101,110-126` already owns the same generic bounded snapshot behavior with configurable cap and coalescing. Replace the repeated mechanics and leave selection clearing, presentation admission and user notices in the product.

Do not generalize this finding to all `History` types. Isometry's `crates/isometry-system/src/pack_author/history.rs:1-24,37-76,106-140` owns a revision-keyed, byte-budgeted retained syntax-tree/text cache with pinned reader snapshots. That is not the same abstraction. Its use of Taplo/Rowan means TOML parsing is already bought. Isometry `crates/isometry-views/src/state.rs:137-138` has separate command `Step` undo/redo, requiring semantic review before migrating.

### 3. MCP plumbing has a maintained supplier

Mere `crates/graphshell/graphshell-mcp/src/lib.rs:108-160` performs line framing, JSON parsing, batch and notification handling; `:162-212` dispatches initialize/discover/tools/resources/ping and protocol responses. `src/rpc.rs:7-27,74-129,132-141` owns dual-era version negotiation, errors and JSON-RPC success/failure envelopes. The crate depends only on Chirograph/Sceno/Scenotime/Serde JSON plus optional endpoint (`Cargo.toml`). Official [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) supplies server and protocol types, [stdio and pluggable transports](https://docs.rs/rmcp/latest/rmcp/transport/index.html).

Current server is synchronous, single-threaded over a generic carrier; the modern 2026-07-28 stateless envelope coexists with legacy initialize. Do not assume SDK `latest` supports this precise contract merely because it is official. A small SDK adapter prototype is warranted. Preserve owner admission, grant expiry, write attributes and stable `mere_invoke` tool semantics. JSON-RPC standards handling is the replaceable part, not the carrier or graph vocabulary.

### 4. Document buffer scalability is worth measuring now

Mere `crates/cambium/cambium/src/controls/text_input/core.rs:52-60,77-94` stores both live document and undo snapshots as Strings. `:339-359` scans the whole text to translate grapheme/byte indices. `:380-385` edits the String in place; `word_motion.rs:24-53` scans from the beginning for word motion. `editor.rs:18-40,50-59` explicitly describes each history entry as a whole-buffer clone with cap 200. This is appropriate for short fields, but the same TextInput backs textarea/Knot editing.

Knot `crates/knot-document/Cargo.toml:30-39` consumes Cambium and Mere source-edit; `crates/knot-document/src/document_surface.rs` owns the surface. `mere-edit` is not a rope or editor engine: Mere `crates/system/source-edit/src/lib.rs:59-65,146-189` retains immutable exact source, validates simultaneous UTF-8 byte edits and constructs preview. Its wrong-document/stale-revision/source-change checks must survive a buffer swap.

[crop](https://docs.rs/crop/latest/crop/) offers a B-tree rope, logarithmic mutation, byte offsets and cheap shared clones. [Ropey](https://docs.rs/ropey/latest/ropey/) offers established UTF-8 text storage but indexes edits by Unicode scalar values rather than graphemes. Neither automatically replaces segmentation or the editor. A full-buffer `&str` consumer at each paint/highlight/save may erase rope gains; prototype the whole data path and compare peak retained memory. Backstep's generic history can retain cheap rope snapshots without replacing its grouping/saved-state policy.

[Parley PlainEditor](https://docs.rs/parley/latest/parley/editing/struct.PlainEditor.html) separately offers caret/selection/composition/layout support. It explicitly uses one style over all text. Highlight spans, source authority, transient ghost text and transaction boundaries are a possible mismatch. It is a candidate for simple fields or lower-level behavior reuse, not a proven replacement for Knot's document editor.

### 5. UI toolkit recommendation withdrawn: R30 struck

Cambium/Rootstock own retained layout, hit testing, input routing, focus, text/scroll/image state and accessibility. The original audit called these ordinary toolkit costs without accounting for why applications project retained component semantics into Graphshell.

Rootstock's native egui/Iced adapters establish caller-owned GPU embedding. They do not establish an external stable identity and semantic projection contract. A real substitute must let another host project and hit-test bought controls with stable identity, accessible roles/actions/state, focus and text input while preserving authority and the shared GPU resource lifecycle. An immediate-mode paint API or private toolkit widget tree does not establish this. No such adapter was demonstrated by this review.

The recommendation to prototype an egui/Iced shell is struck. Existing embedding features remain evidence of those features, not a basis for retiring Cambium or Workbench. A future architectural argument may challenge the design, but must account for the complete capability and migration cost first.

### 6. Knot's merge is not an in-house character CRDT

Knot `crates/knot-editor/src/djot_merge.rs:7-24,49-95` uses Jotdown offset spans to align Djot blocks and splice exact source bytes; identity/order changes fall back to line merge or explicit conflict. `:214-252` resolves common document ancestry through Stickleback's existing CausalIndex. `src/sync.rs:365-401,1142-1177` models multiple document versions, explicit conflicts and derived automatic merges. This is whole-document causal replication with constrained structural three-way merge, not a reinvention of Yjs/Automerge character operations.

The generic fallback at `djot_merge.rs:298-382` computes Myers edits using the existing `similar` crate, then owns overlap detection/order/assembly. [diffy::merge](https://docs.rs/diffy/latest/diffy/fn.merge.html) implements generic three-way text merge and returns errors with conflict markers. Candidate adapter should turn an error into an explicit conflict, never persist marker-laden text. Keep the Djot alignment and causal ancestor policy. Generic libraries can differ on overlapping identical edits, insertions at replacement boundaries and line endings, so this is an evaluation rather than an immediate semantic replacement. `automatic_text_merge_head` hashes the resulting body, making even benign output drift externally significant.

### 7. Armillary needs a usage cut before an executor swap

Armillary `crates/armillary/src/pool.rs:7-24` documents a historical need to reuse long-lived actor worker threads to bound leaked per-thread layout sharing caches. The implementation at `:119-157` owns the adaptive queue/idle/starting accounting and parked workers. `actor.rs:7-20,147-177,227-274` separates thread-per-actor and pooled spawn APIs, keeping !Send internal state constructed on its owning thread. The active harness's completion/panic/wake behavior is legitimate integration code.

The frozen workspace inventory was searched with `spawn_named_on|spawn_on\(|pool::Pool|armillary::Pool` across tracked Rust source. Relevant matches were only Mere `support/fixtures/armillary-consumer/src/main.rs` and a receipt probe, plus irrelevant vendored Firefox libraries. Mere live physics/community/meaning/fetch paths import `spawn` or `spawn_named`, e.g. `crates/conatus/seiche/src/runtime.rs:38`, `crates/canvas/pictograph/src/canvas/community_lane.rs:22`, `meaning_lane.rs:15`, `crates/system/fetch/src/actor.rs:13`. Root separately confirmed zero pool calls in current Turnstone src and pinned ESP/Gemot. A lexical scan is not proof against import aliasing, macro expansion or external registry consumers; complete the public/pinned consumer census before removing an API. It does refute a blanket claim that the current UI stack requires this pool. A focused search of current Genet `565458ea3ab816940484079330eb165a878a22e0` found no Armillary dependency or the documented sharing-cache leak in components; this was not a heap-profile proof that all TLS leaks are gone.

If the pool is retained, Tokio/Rayon are not trivial semantic replacements. [Tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) is intended for bounded blocking work and directs indefinite loops toward dedicated threads. Idle thread retirement, shared blocking-pool contention and shutdown behavior require review. [threadpool](https://docs.rs/threadpool/latest/threadpool/struct.ThreadPool.html) exposes fixed concurrency and manual resizing, not the same eager adaptive capacity policy. A Rayon fixed CPU pool can be occupied permanently by actor receive loops. Reusing a standard pool is possible with additional scheduling/admission policy, not free.

Armillary `tasks.rs:50-70,83-96,116-150` already buys Tokio JoinSet and adds admission fencing plus cumulative panic/cancel/rejection reports. [TaskTracker](https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html) tracks task lifetimes and frees results, but closing it does not itself forbid subsequent spawns, and it does not preserve JoinSet's result semantics. Replacing this small wrapper would reintroduce local policy around another type. `lifecycle.rs:61-75` merely closes resources in order and retains all failures. Keep these active thin wrappers.

### 8. Meristem is an upstream-derived fork with a specific mismatch

Mere's provenance ledger `design_docs/cambium_docs/technical_architecture/upstream-xilem.md:13-34,36-63,65-88` records the Xilem core ancestry, three move-preserving splice operations, deliberate removal of unused optional APIs, and Cambium's xilem_web-derived backend files. Current `crates/cambium/meristem/src/element_splice.rs:33-64` adds hoist/extract/adopt operations to preserve backing nodes through reorder/reparent. The current published [xilem_core 0.4 ElementSplice](https://docs.rs/xilem_core/latest/xilem_core/trait.ElementSplice.html) still lists only the ordinary splice operations. Therefore direct upstream replacement loses a material contract unless moved into an adapter or accepted upstream.

The best reassessment is upstream convergence or a small clearly replayable patch. The existence of deliberate API removals alone is a weak reason to own a fork; unused upstream APIs need not be used. Portable moves are the stronger constraint. This is distinct from the much larger question of owning Cambium/Rootstock itself.

### 9. Existing library use should not be mislabeled reinvention

- Graph structure already uses Petgraph (`crates/graph/graph-kernel/Cargo.toml`), constraint/layout code uses Taffy (`crates/forme/forme/Cargo.toml`), and live force layout uses Rapier enhanced determinism (`crates/conatus/seiche/Cargo.toml:24-31`). Seiche's force policies, holds, source clocks and field couplings remain project behavior.
- Scenomise `src/lib.rs:10-34` uses closed-form graph-independent placement and disclosed source signals. `src/relax.rs:7-26` deliberately uses bounded deterministic O(steps*n²) relaxation for tens of nodes, with pinned/anchored/seeded roles. Replacing this with a full simulation engine is not obviously cheaper. `seiche/src/barnes_hut.rs` is a genuine custom standard numerical kernel worth a focused alternative survey if scale or maintenance demands it, but no primary-source-qualified replacement was established here.
- Sceno `src/lib.rs:7-49` carries source/instance distinction, footprints and opaque domain references. SceneDocument `scenomise/src/document.rs:38-74` parses through Serde JSON, bounds size and checks linked readings. It owns a domain schema, not an independent JSON parser.
- Illume already uses Jotdown for Djot and Logos for a curated injection lexer pack (`crates/nematic/illume/Cargo.toml`; `src/lib.rs:7-18`). Tree-sitter would buy richer incremental parsing across languages, but adds grammars/build constraints and is not automatically appropriate for pure-Rust/Wasm Djot/prose highlighting. Keep the narrow injection/span adapter; evaluate Tree-sitter only against a demonstrated language-editor requirement.
- Hest declares command ID/label/category/effect/destructive/disabled state; hosts own enactment (`crates/cambium/hest/src/lib.rs:5-16`). Its search is a plain lowercase substring check (`menu/mod.rs:70-74`), not an elaborate homemade fuzzy matcher. If fuzzy matching is wanted, use [nucleo](https://docs.rs/nucleo-matcher/latest/nucleo_matcher/) rather than expanding a bespoke ranking engine; the docs recommend the high-level engine for larger interactive lists.
- AccessKit owns platform accessibility; Cambium's `cambium-winit-a11y` and `forme/uxtree` supply DOM/projection lowering and composition. That glue cannot be removed merely by depending on AccessKit, which is already present.
- Rhai and Wasmtime/WASI implement scripting execution (`crates/script/*/Cargo.toml`). The stack owns exposed capability policy, not another interpreter or Wasm engine.
- Knot's file writer owns file-identity/replacement behavior, including a Windows `ReplaceFile` path and tests retaining alternate data streams (`crates/knot-document/src/writer.rs`, tests at `:321,:357`). A generic atomic-write library must preserve those semantics. Woodshed's `storage.rs:538-587` separately supports mutable rename and immutable no-clobber hard-link publication. Shared durable write mechanics are worth evaluating, but these are not proven identical helpers.

## Sibling audio spotcheck

### Firewheel: narrow fork, not an owned audio engine

The cached fork is upstream Firewheel 0.14 base plus three commits. `git diff --stat 34b35ca..2b03846` reports 13 files, 2115 insertions/2 deletions, including the stream channel/reader/writer implementation, a 77-line packet-buffer adapter, a 606-line integration suite and docs/CI. The fork still delegates the graph engine, allocator/collector, CPAL driver, ring buffer, sample-rate conversion and FFT work to existing packages.

`design_docs/2026-10-11_standalone_stream_foundation.md:1-18` records upstream removal of reader/writer nodes and an unfinished streamed sampler. It specifies why the own PacketResamplerBuffer fixes a short-EOF copy-range defect. The actual `crates/firewheel-nodes/src/stream/resampler_buffer.rs:1-10,68-77` delegates Rubato processing and corrects the source range. `stream/channel.rs` owns bounded source/output accounting and admission. [Published Firewheel](https://docs.rs/firewheel/latest/firewheel/) remains 0.14 in the browsed docs, and published firewheel-nodes docs have no stream module. [fixed-resample](https://docs.rs/fixed-resample/latest/fixed_resample/) remains 0.13 and wraps Rubato's resampler; do not claim the defect is fixed upstream without source/reproducer verification.

Retirement should be actively sought, but only an upstream API matching accepted/presented/queued source clocks, pause/reset generations, bounded partial admission and short-EOF samples can retire this extension. Existing fork documentation records offline allocation guards, rate/channel edge cases and 288 short-EOF cases; those are historical receipts, not tests rerun by this audit. Hardware latency, drift and audible quality remain separate. The full fork should not grow into an alternative audio graph implementation.

### Custom DSP and standards candidates

Woodshed `crates/audio-primitives/src/stretch.rs` owns WSOLA windows, correlation, overlap-add and streaming state with a bit-exact unity path. **R33 is struck:** SoundTouch and Signalsmith were proposed despite their C++ requirement. They are excluded from this reuse queue. No compatible Rust supplier or quality/source-clock advantage was established. Ordinary resampling is not pitch-preserving time stretch.

`audio-primitives/src/onset.rs:1-42` owns energy-envelope onset/BPM heuristics rather than a full MIR suite. Its own rationale is single-line attacks, allocation control and no FFT; that is a testable requirement, not a veto on spectral-flux or established DSP alternatives. Evaluate only if actual attack-detection quality or scope warrants the dependency. Click synthesis, peak/meter envelopes and calibration timing are small kernels; no clear replacement would erase more integration than it adds in this spotcheck.

`crates/timed-text/src/lib.rs:27-142` hand parses a WebVTT transcript subset and strips cue tags/entities with simple character/string passes. [vtt](https://docs.rs/vtt/latest/vtt/) supplies cue/settings/timestamp types and parsing. Candidate must preserve raw payload and subset projection, plus malformed cue recovery; a parser that fails the whole file on one malformed block may not fit. The small implementation is not a high-severity urgent rewrite, but further standards expansion should start with a parser supplier and the [WebVTT parsing rules](https://www.w3.org/TR/webvtt1/).

MIDI is already bought: `crates/audio-pattern/src/midi.rs:6` uses Midly; its README records strict parsing with Rayon excluded. Audio decoding/driving similarly uses CPAL, Symphonia/Firewheel, rtrb, pitch-detector and pitch-detection. Do not recommend adopting libraries already integrated.

## Candidate follow-up

Prefer existing in-family reuse: inspect color conversion compatibility and Backstep fold undo, and finish the Pool caller census. MCP/text-buffer candidates need an actual benefit and complete consumer dependency cost before implementation. The toolkit and C++ DSP experiments are withdrawn. Small accepted substitutions need proportionate tests and ordinary commit notes; a new receipt hierarchy or mandatory independent review is not required for every helper.
