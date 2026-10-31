# UI, editor, shell and execution ownership audit

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

The biggest ownership question is ordinary UI machinery, not a new actor runtime. Cambium and Rootstock implement a substantial widget, input, focus, editing, scrolling, accessibility-composition and host stack. That is a strategic cost worth comparing against an established toolkit. A unified wgpu device does not automatically require owning every widget: Rootstock already includes egui and Iced embedding adapters using the caller's device and queue. Native-only configuration, admitted scene operations, focus/IME composition and accessibility are real remaining constraints.

The clearest small cuts are color-science arithmetic and redundant snapshot undo. The next bounded evaluations are MCP protocol plumbing, the large-document editor buffer, and Knot's generic line-merge fallback. Domain-specific command declarations, source-preserving Djot behavior, scene projection identities and admission policy should remain local.

One correction to the preliminary audit is material: Armillary's custom Pool is not shown to be a live product dependency. The inspected products call the separate actor harness, usually thread-per-actor. Defending the pool from its historical thread-local-cache rationale overstates current evidence.

## Ranked findings

| Rank | Owned machinery | Verdict | Confidence | Candidate / action | Decisive qualification |
|---|---|---|---|---|---|
| 1 | Tinct sRGB transfer, HSL/OKLCH transforms and WCAG arithmetic | Replace internal arithmetic | High | Reuse workspace `color` where precision permits, or `palette` for f64 + contrast; retain Tinct seed/token/gating API | Exact built-in token outputs or explicit reviewed drift; contrast thresholds; hue wrap; neutral/NaN/out-of-gamut behavior; native/Wasm comparison |
| 2 | Woodshed view-local fold undo stacks | Replace duplicate with existing Backstep | High | `backstep::History<Vec<FoldRecord>>` with explicit cap 64 | Create/expand/undo/redo/reopen, new edits after undo, host selection reset, existing cap behavior |
| 3 | MCP JSON-RPC stdio framing/version negotiation/resource/tool envelope | Evaluate official `rmcp` adapter | High that duplicated standards surface exists; medium drop-in fit | Keep route/carrier admission and tools; delegate MCP lifecycle/serialization/transport where SDK supports required eras | Legacy and 2026-07-28 discovery/version contract; notification/batch policy; unknown versions; EOF and grant expiry; tools/resources refusal equivalence |
| 4 | Whole-string editor buffer plus repeated grapheme scans and whole-buffer snapshots | Evaluate rope-backed document path | High structural finding; medium replacement scope | `crop` byte-index rope or Ropey; preserve short text fields; retain grapheme/IME policy and Backstep grouping | Multi-MiB mid-document edit/caret/undo memory and latency, UTF-8 byte proposal boundary, ZWJ/combining/bidi, incremental highlight and exact save |
| 5 | Ordinary widgets/chrome/focus/scroll/host assembly | Strategic expensive ownership | High maintenance-cost finding; medium benefit until prototype | Compare egui/Iced shell around retained graph/document producers; existing native adapters provide starting point | One real editor+graph+menus+split panes on shared GPU, focus/IME/a11y/reopen, native and browser target gates; compare removed code and UX regressions |
| 6 | Knot generic line-based three-way merge fallback | Evaluate `diffy` | High duplicated primitive; medium semantic fit | Replace generic line reconciliation only, retain Djot block alignment and causal history | Same-position insertions, edits on range boundaries, identical edits, CRLF/no-final-newline, conflicts remain explicit, byte-preserving output and deterministic head hash |
| 7 | Unused Armillary public adaptive Pool | Trim if complete consumer census confirms | Medium-high lexical census; no dynamic or external-user proof | Remove/restrict unsupported pool API, retain active actor harness; do not replace unused API with another executor | Consumer/API inventory at every supported pin, fixture migration, external publication review; if retained, rerun worker lifetime/churn/TLS reason |
| 8 | Meristem fork of Xilem core | Evaluate upstream convergence, keep required extensions today | High source mismatch | Propose upstream splice moves, then consume upstream or keep small reviewed patch; avoid treating removed optional APIs as fork necessity | Keyed reorder/cross-parent portable move retains DOM/node/focus state; action routing; unchanged supported application behaviors |
| 9 | Parley-related text navigation/editing | Evaluate narrower upstream editor reuse | Medium | Compare `parley::editing::PlainEditor` for simple fields or lower-level selection APIs | Single-style editor limitation, highlighting, preedit ghost state, undo boundaries, bidi/soft wrap and source-byte compatibility |
| 10 | Woodshed WSOLA time stretching | Evaluate DSP supplier for quality | Medium | SoundTouch or Signalsmith Stretch if native C++ acceptable; preserve source-clock adapter | Blinded voice/percussion/music quality, rate transitions, very short EOF, sample clocks/source anchors, CPU/latency, licensing and Wasm build |
| 11 | Woodshed WebVTT transcript subset | Evaluate parser library | Medium | `vtt` parser, keep transcript extraction policy, original payload and bounded fetching | Malformed-block recovery, BOM/CR/NUL, cue IDs/settings preserved, cue markup/entity behavior, overlaps, size limits and Wasm |
| 12 | Workbench split/tab/float reducer | Keep under current toolkit; reassess with shell migration | High mismatch | egui_dock/egui_tiles only if shell uses egui; they are not host-neutral reducer drop-ins | Serialization and IDs, pane close/tearout effects, float docking, declarative authority separation, native/browser input |
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

### 5. UI toolkit ownership is the largest strategic cost

At the frozen Mere revision, source file inventories (unit tests included) are: Cambium 87 files/1,511,944 bytes, Rootstock 62/858,494, native host 12/182,984, browser host 19/186,497, Meristem 21/114,688, Sprigging 8/101,890. Rootstock `src/lib.rs:7-25` names retained layout, hit testing, input routing, focus/spatial navigation, pacing and accessibility. `src/owned_layout.rs:61-85` owns style/layout/text/scroll/image state. These are ordinary toolkit capabilities with recurring standards/platform costs, even when the graph model is distinctive.

Rootstock already has `embedding-egui` and `embedding-iced` features (`Cargo.toml:21-27`); `src/embedding.rs:7-10` states caller-owned device/queue and admitted sparse scene scope, and `src/embedding/egui.rs:6-7` lets egui own encoding/submission and viewport/scissor. Both adapters are gated off Wasm (`embedding.rs:23-30`). This is concrete evidence that a shared GPU can coexist with another UI toolkit; it does not prove every graph paint operation or browser deployment works through it.

A decisive prototype should host one real graph plus editor, command palette, docked panes and theme controls in an established shell, use the existing domain projections, and measure how much bespoke input/widget/host code can retire. Include exact a11y subtree composition, IME, focus, scaling and browser behavior. Replacing the entire toolkit in one migration is not justified by a file count. Equally, an existing design decision to own the toolkit does not settle whether the ongoing cost is worth it.

Mere Workbench `crates/cambium/workbench/lib.rs:5-21` is explicitly a host-neutral presentation reducer, with domain authority projected onto it. [egui_dock](https://docs.rs/egui_dock/latest/egui_dock/) and [egui_tiles](https://docs.rs/egui_tiles/latest/egui_tiles/) provide mature split/tab/docking interaction within egui. They become attractive with the shell prototype, not as an isolated dependency swap inside Cambium.

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

Woodshed `crates/audio-primitives/src/stretch.rs` is a genuine own WSOLA time-stretch implementation, about 19.7KB including tests. It controls windows, correlation, overlap-add and streaming state with a bit-exact unity path (`:1-40`). [SoundTouch](https://docs.rs/soundtouch/latest/soundtouch/struct.SoundTouch.html) provides time/tempo/pitch processing; Signalsmith has a [dedicated C++ pitch/time library](https://github.com/Signalsmith-Audio/signalsmith-stretch). Evaluate sound quality and complexity, with native C++/license/Wasm constraints and a required source-time mapping adapter. Ordinary resampling libraries are not substitutes for pitch-preserving time stretch.

`audio-primitives/src/onset.rs:1-42` owns energy-envelope onset/BPM heuristics rather than a full MIR suite. Its own rationale is single-line attacks, allocation control and no FFT; that is a testable requirement, not a veto on spectral-flux or established DSP alternatives. Evaluate only if actual attack-detection quality or scope warrants the dependency. Click synthesis, peak/meter envelopes and calibration timing are small kernels; no clear replacement would erase more integration than it adds in this spotcheck.

`crates/timed-text/src/lib.rs:27-142` hand parses a WebVTT transcript subset and strips cue tags/entities with simple character/string passes. [vtt](https://docs.rs/vtt/latest/vtt/) supplies cue/settings/timestamp types and parsing. Candidate must preserve raw payload and subset projection, plus malformed cue recovery; a parser that fails the whole file on one malformed block may not fit. The small implementation is not a high-severity urgent rewrite, but further standards expansion should start with a parser supplier and the [WebVTT parsing rules](https://www.w3.org/TR/webvtt1/).

MIDI is already bought: `crates/audio-pattern/src/midi.rs:6` uses Midly; its README records strict parsing with Rayon excluded. Audio decoding/driving similarly uses CPAL, Symphonia/Firewheel, rtrb, pitch-detector and pitch-detection. Do not recommend adopting libraries already integrated.

## Bounded next decisions

1. Replace numeric color conversion behind unchanged public Tinct policy and consolidate Woodshed fold undo into Backstep.
2. Prototype SDK-backed MCP plus rope-backed document storage separately, with measurable deleted code and exact compatibility receipts.
3. Establish the real Armillary Pool consumer list; remove unused runtime surface if supported rather than choosing a replacement executor.
4. Run one concrete shell/toolkit comparison using the existing native embedding seam before expanding bespoke widget/host features.
5. Keep source authority, causal admission, domain scenes and command policy local. Require each proposed supplier to meet the current behavior it replaces; do not turn documented rationale into permanent immunity from comparison.

Memory assisted source discovery only: MEMORY.md:24-27 and 164-170 supplied the earlier retained-ownership judgement and Xilem provenance warning; all substantive UI findings above were checked against current frozen source.
