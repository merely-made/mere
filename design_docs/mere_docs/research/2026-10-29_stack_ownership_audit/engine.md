# Engine and rendering ownership audit, 2026-10-29

Read-only source assessment. No builds, runtime probes, migrations, commits or repository edits were performed. Recommendations are ownership judgments; replacement compatibility remains untested. Source line counts below include inline tests and comments and exclude separate `tests/` directories; they indicate size, not complexity or defect counts.

## Reviewed revisions

- Genet: `565458ea3ab816940484079330eb165a878a22e0` (G below).
- Netrender: `11e208b84a981993df8077bb6025f8b3adbe2291` (N).
- Genet actually consumes Vano `b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88`, Boa/Bole `ddf4e31b0d6cf86c11ea8907b0d07e4799d62368`, Vello `276b46648c08f3d2062e70559afa999842e1873c`. These immutable pins, rather than their later main branches, define its engine/render contract. Inspected fork main metadata too: Vano `298c22b7`, Boa `8d9b3141`, Vello `9cbe466b`.
- Graft `d991685b5324820bc47f598ef3d8e801e5cb61f5`; Scry `5254c8f011f3aaca3d23583ee42ee3a678d517e8`; Weld `7f04a38bb77696a8ca584565529f7c81fd8fddd4`.
- Arboard local custom-formats source `bec2dcd179deff24720e64107a0993bb36453df4`; Genet's manifest selects the remote custom-formats branch, so this is local implementation evidence, not proof of a freshly resolved remote package.

Applicable supplied `/home/markik` instructions were observed; no additional AGENTS.md appeared in these reviewed repository trees or immediate parent locations. All reads used Git objects where possible and left working files intact.

## Best additional opportunities

### 1. Replace the WebIDL lexer/parser with a maintained WebIDL parser

**Confidence: high technical overlap, medium migration value.** G `support/idl-interface-table/src/idl.rs:8-67` explicitly implements a subset reader and handwritten tokenizer; the file has 527 lines. It skips all definition-body material except attributes. `Cargo.toml:1-5` explicitly says the tool has no dependencies because it parses by hand. `src/lib.rs` then owns useful project-specific reflection overrides and HTML tag mapping.

Use [Weedle](https://docs.rs/weedle/latest/weedle/), already used by [wasm-bindgen's WebIDL generator](https://docs.rs/wasm-bindgen-webidl/latest/src/wasm_bindgen_webidl/lib.rs.html), for syntax. Keep Genet's interface-table model, override reasons, exposure policy, tag map and deterministic generator. A full binding generator is a larger and separate project.

**Mismatch:** parser coverage, accepted WPT dialect, extended attributes and currently ignored constructs may differ. This is a build/developer tool, so adding a parser does not require adding it to the shipped browser dependency cone.

**Deciding test:** run both parsers over the exact vendored `interfaces/*.idl`; compare generated `html_interfaces_generated.rs` byte-for-byte and reject unexplained dropped interfaces, parents, attributes or exposure changes. Add malformed/partial/mixin fixtures to make parser drift visible. This is a stronger low-risk candidate than replacing the live DOM.

### 2. Evaluate Linebender `color` for Livery's numerical color machinery

**Confidence: high overlap, medium-high adoption potential.** G `components/livery/src/values/color/space/mod.rs:9-25` records owned CSS Color 4 conversion algorithms harvested from the old Stylo fork. `space/mod.rs:295`, `space/perceptual.rs`, and `space/rgb.rs` implement conversions, matrices, white-point adaptation and hue handling. The whole color directory is 3,564 lines across 12 Rust files, including additional parser/computation machinery.

[Linebender color](https://docs.rs/color/latest/color/) explicitly targets CSS Color 4 conversions. [DynamicColor](https://docs.rs/color/latest/color/struct.DynamicColor.html) supports missing components, conversion and CSS interpolation. Replace numerical conversions and perhaps interpolation, rather than force all specified/computed CSS values into a general color type.

**Mismatch:** preserve currentColor, relative-color expressions, deferred calc/variables, CSS serialization, missing/powerless hue, alpha premultiplication, clipping and color-mix rules. Existing provenance is a deliberate ownership choice, but it does not make numerical maintenance inherently worthwhile. Match the exact compatible dependency version already used by the visual stack before selecting another.

**Deciding test:** conversion vectors in every supported space including negative/out-of-gamut values, D50/D65 round trips, missing channels, powerless hues, translucent interpolation; then Livery color tests and CSS color WPT/reftest before/after. Require bounded numerical tolerances and attribute rendering differences.

### 3. Evaluate ICU's blob provider instead of the custom dictionary file format

**Confidence: high overlap, medium adoption potential.** G `support/patches/parley/src/analysis/dictionaries.rs:8-13,30,151,251-298` owns `GSEGDIC1`, a parser, validation, shared-byte backing and typed ICU providers in a 456-line file. `GENET_PATCHES.md:99-123` says explicitly that there is no `icu_provider_blob`. The sensible product behavior is runtime dictionary loading by group to keep megabytes out of the initial Wasm download.

[ICU4X BlobDataProvider](https://docs.rs/icu_provider_blob/latest/icu_provider_blob/struct.BlobDataProvider.html) loads serialized ICU data at runtime and provides `Send + Sync`. Preserve demand detection, two-group loading, fallback behavior and diagnostics; delegate bytes/data-provider mechanics if the size trade works.

**Mismatch:** the custom provider projects only the dictionary markers while compiled-in non-dictionary rules stay separate. A stock complete segmenter blob could accidentally put the megabytes back into the initial artifact or require a different provider composition. Blob format size, deserialization features, ICU marker/version compatibility and zero-copy ownership all matter.

**Deciding test:** export only those exact markers and attributes into two ICU blobs; compare compressed initial Wasm plus downloaded bytes, load RSS, rejection of truncated/incorrect blobs, `Send` compilation and all six complex-script segmentation outputs against the baked-data oracle. Avoid replacing a compact format with a heavier generic solution without measuring.

### 4. Replace the retained media analyzer's quadratic DFT if this media lane is revived

**Confidence: very high algorithmic replacement; low present product priority.** G `components/media/audio/analyser_node.rs:212-235` names `compute_fft` but directly sums sine/cosine for every sample for every output bin. It is O(N²), with allowed sizes up to 32,768. Use [RealFFT](https://docs.rs/crate/realfft/latest) or RustFFT and retain Web Audio's Blackman window, smoothing and decibel conversion.

**Critical scope qualification:** manifest consumers of `servo-media-*` are confined to the retained media family and `components/media/examples`; the current scripted runtime has no AudioContext binding using it. This is inherited retained code, not evidence of a live page bottleneck. Do not spend product time optimizing an unused analyzer merely because it is an obvious algorithmic reinvention.

**Mismatch/test:** library normalization and Nyquist bin layout differ. Compare impulse, DC, bin-centered/off-bin sine, silence and random inputs against the existing slow oracle for every allowed size; preserve 1/N scaling, smoothing history and byte clamping; measure large transforms with reusable plans/scratch buffers and no audio-thread allocation.

### 5. Evaluate sharing Unicode Bidi_Class data rather than maintaining another generated table

**Confidence: high overlap, conditional value.** G `components/genet-scripted-dom/forms_directionality_data.rs:1-9,631` is a 641-line generated Unicode 17 classification table plus local generator/corpus. `forms_directionality.rs:24` uses it only for HTML's first-strong directionality. Genet's layout already uses ICU properties and bidi through its text libraries.

[ICU CodePointMapData<BidiClass>](https://docs.rs/icu_properties/latest/icu_properties/props/struct.BidiClass.html) supplies L/R/AL values. Use that lookup if a Unicode-matched provider can be shared without imposing an expensive new data cone on small DOM users. Keep HTML ancestry, slot, shadow-host, dir=auto and input-type rules in Genet.

**Mismatch/test:** Unicode version must remain 17, including defaults for unassigned characters. Compare every Unicode scalar against the retained source corpus and measure native/Wasm artifact growth. A tiny generated table may remain cheaper for consumers that otherwise never need ICU. Do not replace first-strong HTML policy with the full bidi layout algorithm.

## High-cost ownership decisions

### ESSL compiler and WebGL state: investigate ANGLE before expanding the owned compiler

G `components/webgl-essl/` has 10,005 non-test-file lines across lexer, recursive-descent/Pratt parser, AST, typing, function registry, validation, reflection and lowering. `lib.rs:5-37` describes its partial ESSL 3 delta and ANGLE-inspired WebGL restrictions; `Cargo.toml` already buys rspirv and Naga for SPIR-V-to-WGSL. `components/webgl-wgpu/lib.rs:5-11` adds browser graphics state on the shared wgpu device.

[ANGLE's standalone GLSL ES translator](https://github.com/google/angle/blob/main/doc/DevSetup.md) is independent of its full GLES/EGL implementation and offers SPIR-V output. It is the serious mature comparator. **Recommendation: strategic evaluate, medium confidence**, not a claimed drop-in. It adds C++ tooling/FFI and complicates pure-Rust and browser-Wasm goals, reflection, stage linking and uniform packing. Replacing the entire graphics backend may introduce a second GPU device rather than preserve the unified-device goal.

**Deciding experiment:** two adapters feeding the same wgpu output and reflection contract, same accepted/rejected ESSL corpus and Khronos WebGL shader validation/linking cases, limits/resource abuse tests, native/wasm build feasibility, and pixels/uniform layouts for both engines. Use ANGLE as a differential oracle even if it cannot be shipped. Naga's ability to output GLES is not evidence that it accepts ESSL 1.00 input or implements browser shader restrictions; a text preprocessor around Naga is not a qualified WebGL compiler replacement.

### Cadency selectors: a deliberate reinvention with a real upstream alternative

G `components/cadency/` has 1,550 non-test-file Rust lines. `README.md:7-13` lists supported selectors and explicit omissions (:has, named namespace prefixes, quirks). `design_docs/2026-09-18_cadency_selector_engine_plan.md:10-23` records the earlier decision to remove even `selectors` and `servo_arc`, and decline harvesting upstream generic machinery. That explains the code; the current audit can reconsider it.

The [selectors crate](https://docs.rs/selectors/latest/selectors/) exposes parsing/matching over an Element adapter, relative-selector machinery, bloom filters and nth-index caches. **Recommendation: strategic evaluate, high overlap, medium net-benefit confidence.** If complete web compatibility is the goal, this is more economically questionable ownership than a DOM arena. If the earlier source-family ban remains a present preference, Cadency is its continuing standards-maintenance cost. Restore a library boundary, not all of Stylo, if that trade is now acceptable.

**Test:** same CSS/DOM adapter, specificity/forgiving-list parsing, shadow scope/part/exportparts, state vocabulary, CSS rejection sets, DOM query and cascade WPT, and invalidation/large-tree performance. Existing reach/dependency/key inspection needs equivalent facts or conservative invalidation. Generic matching alone does not supply browser cascade behavior.

### WHATWG Streams: evaluate vendored upstream algorithms, retain the private host integration

G `components/script-runtime-api/streams/{core,readable,writable,piping}.js` has 3,712 lines. `streams.rs:5-7` and `core.js:60` own agent-shared weak brands and per-realm captured intrinsics. Fetch uses private operation tables, promise jobs and buffer transfer. This is a substantial standards subsystem, not just a queue wrapper.

[web-streams-polyfill](https://github.com/MattiasBuelens/web-streams-polyfill) offers vendorable ponyfill algorithms and tests against WPT, with documented exceptions and real buffer-transfer prerequisites. **Recommendation: evaluate, medium confidence.** Its own compatibility statements are not browser-embedder security qualification. A runtime author cannot assume a package designed for application environments has the same hostile-global, cross-realm or native Fetch integration contract.

**Test:** graft its algorithm core under Genet's captured host operations; run exact current Streams/Fetch WPT plus existing native regression suites on Boa and Vano, mutated globals/prototypes, BYOB real detachment, realm identity/branding, tee/backpressure/error scheduling and live-response clone rejection. `design_docs/2026-10-06_whatwg_streams_plan.md` records recent improvements and earlier losses; retain its before/after accounting. Transferable streams remain separate. The goal is fewer owned algorithms while preserving a browser-quality boundary.

### Vano and Boa: keep the VM reuse, audit fork deltas rather than proposing another interpreter

G `components/script-engine-{nova,boa}/Cargo.toml` make the split explicit: Vano for 64-bit, Boa for wasm32/32-bit and a second native conformance path. Vano's own `nova_vm/Cargo.toml` already uses Oxc parser/AST/semantics, regress, temporal_rs, numeric and Unicode crates. There is no evidence that Genet is hand-writing its JavaScript parser, regex engine or Temporal implementation. Piccolo is likewise a reused Lua VM.

The continuing cost is maintaining two adapters and forks for host realm/caller provenance, GC/reflector lifetimes, interruption, buffer transfer, job queues, snapshots and clocks. See G `docs/2026-07-04_vano_strategy_note.md` and `design_docs/2026-10-08_boa_stable_upstream_refresh.md`. **Recommendation: strategic keep/evaluate consolidation, not replace wholesale.** Vano's data-oriented heap cloning is a real architecture feature; Boa covers the target Vano cannot. Choosing one VM or a C/C++ VM is an explicit product/target trade, not a zero-cost dependency upgrade.

**Fork retirement test:** exact adapter and realm/GC regression suites against pristine upstream; identify every failed contract; upstream generic host seams and remove only absorbed patches. Native checks and wasm compilation do not prove browser-hosted scripting acceptance.

## Rendering: mostly appropriate library use, plus strategic retained state

N already delegates rasterization to Vello classic, CPU and Sparse Strips, and text to Parley/Skrifa. `netrender_text/src/lib.rs:7-21` states the adapter boundary. Replacing it with cosmic-text would replace one external shaper with another; it would not remove a homegrown shaper.

Owned substantial pieces include scene/resource identity and validation; paint-list lowering; tile dependency hashing/indexing (`netrender/src/tile_cache/`, 1,797 lines); retained/flattened fragment handling; hit testing; filter chains/blur masks; image execution graph (`netrender_device/src/render_graph.rs`, 1,637 lines); device custody, readback and external-image contracts. These serve the shared-device, retained-scene and multi-producer architecture. A generic LRU or DAG library does not supply order-sensitive rendering dependencies, halo/clip/transform damage or texture retirement.

**Recommendation: keep the contracts and retained caching unless profiling proves their cost; evaluate small mechanics only.** Replacing render-graph scheduling with a generic graph crate deletes little of its access validation, output reachability, transient-image or callback ownership. Replacing all retained rendering with Vello immediate mode is a different performance model, not a free library swap. Deciding evidence is cold/full-rebuild pixel equivalence after mutations, image/font replacement and device loss, plus measured unchanged/partial frame CPU/GPU/memory costs.

Vello pin `276b4664` carries native shader progress/coarse traversal, resident atlas, allocation recovery, bounded append and device-pixel quality changes (its commit log immediately preceding the pin). **Evaluate upstreaming and retirement per patch.** Later Vello main is not proof those repairs are absorbed, and upstream's current demos are not proof of Genet's exact hardware paths. Require current replay/pixel/timeout receipts on native Vulkan/DX12/Metal and direct browser WebGL2/WebGPU before changing the pin.

Graft/Scry/Weld already buy the three full-browser choices: Servo, platform WebKit/WebView2/WPE, and CEF. Their reviewed READMEs and source inventories show Scry/Weld native resource imports delegated to Graft, while producer callback lifetimes, copy/retain policy, synchronization, navigation and events remain local. **Keep that integration boundary.** A stock webview widget does not promise offscreen pixels into the same wgpu device. Using these existing full engines more often can reduce Genet's standards surface strategically, but will not preserve every pure-Rust, DOM access, portability or graph integration property automatically. Hardware status is per backend and receipt; older all-platform statements do not qualify new engine pins.

## Other reviewed boundaries and rejected easy substitutions

- **HTML/XML parsing/serialization, URL, text encodings, segmentation, SVG/image decoding and font shaping are already bought:** html5ever/xml5ever/markup5ever, url, encoding_rs, unicode-segmentation, ICU, resvg/image, Parley/Fontique/Skrifa. G `components/genet-static-dom`, `genet-scripted-dom/parser.rs`, `genet-text/src/lib.rs`, and manifests show wrappers/adapters. Do not recommend replacing them as if they were local algorithms.
- **CSS/layout is mixed:** Livery owns properties/cascade/values; Buckram owns browser box/fragment/containing-block/table/text integration, but delegates block/float/flex/grid kernels to genet-taffy. G `support/patches/taffy/GENET_PATCHES.md:3-26` records the 2026-10-28 refresh to upstream 0.15.1 and retirement of private intrinsic-size and out-of-flow algorithms. Recommending those already-completed adoptions would be stale. Retire remaining fork hunks through focused static-position, float, order, containment and automatic-minimum tests. Generic GUI layout does not implement all CSS layout.
- **Parley/Fontique fork:** current patch logs explain join-control fallback/spacing, trailing whitespace for float bands, Windows cluster fallback and diagnostics. Refresh/retire those against upstream before swapping shapers. Runtime dictionary encoding is the distinct candidate above. Comparing Netrender's upstream Parley 0.12 and Genet's 0.10 fork is worthwhile; different versions are not proof of redundant functionality.
- **Live DOM/Web APIs:** G scripted DOM, events, bootstrap, tree/mutation/shadow/selection/custom-element/frame/navigation/form/fetch/security modules own actual browser semantics and GC identity. Existing XML trees, web-sys host bindings or a JS server runtime are not drop-in replacements. Web Crypto primitives already use RustCrypto. Keep policy/realm/identity; buy isolated standards mechanics where available.
- **Forms date parsing:** the hand-written calendar conversions are real code, but G `forms_validation.rs:2430` explicitly preserves oversized HTML-valid dates beyond f64 conversion. A blind `chrono`/`time` replacement would narrow valid inputs. Keep HTML syntax/range semantics unless a measured adapter preserves arbitrary-length years and step behavior. Regress already supplies HTML UnicodeSets patterns, with a two-line vendored parser fix and explicit retirement condition in its patch log.
- **XPath:** G `components/xpath` owns 3,076 lines of inherited tokenizer/parser/evaluator over a live generic DOM adapter. [sxd-xpath](https://docs.rs/sxd-xpath/latest/sxd_xpath/) targets XPath 1.0 but uses sxd-document nodes. A copied XML tree would lose live DOM identity and mutation semantics. Low-priority evaluate parser/evaluator reuse only if a zero-copy Node adapter is feasible; use DOM XPath WPT and namespace/tree-order mutation cases to decide.
- **Structured clone:** G `structured_clone.rs:5-22` deliberately serializes sender values and reconstructs recipient intrinsics, includes platform transfer registrations and worker transport. JSON/bincode/postcard do not implement ECMAScript object identity, exceptions or transfer semantics. Keep browser algorithm until a compatible backend-independent implementation is established.
- **Worker IPC:** G `worker_process.rs:25,128-139` uses a small bounded length-prefixed JSON protocol with version/backend handshakes and explicit process limits. [Tokio LengthDelimitedCodec](https://docs.rs/tokio-util/latest/tokio_util/codec/length_delimited/struct.Builder.html) overlaps framing, but the implementation uses blocking pipes/threads. Do not introduce an async runtime/codec migration merely to delete a few framing lines. Keep kill/reap, bounded admission, nested-worker and trust policy; rerun malformed/oversized/startup/shutdown tests for any replacement.
- **Clipboard:** G `genet-clipboard/system.rs:25,82-111` actually uses arboard fork `ClipboardData`, `CustomItem`, `set_data` and `get_custom`. Its top-level documentation describing custom MIME/multiple formats as future work is stale. Do not report them absent. The remaining file-list combination and cross-platform custom-format custody are real fork costs; prefer upstreaming/retiring that extension rather than inventing another OS backend. Preserve the long-lived clipboard owner; validate text+HTML+image+custom and external-app round trips per OS. Source implementation is not hardware qualification.
- **Media:** codecs/player/WebRTC already use GStreamer, and resampling uses Speex. The retained Web Audio graph, oscillator/automation/biquad/IIR/panner/waveshaper are additional inherited ownership, but generic audio playback or a biquad crate does not supply Web Audio scheduling and channel rules. Prioritize a library-backed numerical kernel when activating this lane, with offline audio conformance, rather than assume media support is already page-integrated.
- **Cache/cookies:** G netfetcher already uses external URL, PSL, HTTP-date, cookie parsing, compression and transport. Its remaining HTTP-cache policy and browser cookie semantics are real candidates covered by the broader audit; retain storage/privacy/request policy. The endpoint is not replacing WHATWG Fetch with reqwest.

## Coverage limit

This is an extensive architectural/source inventory over the named repositories and fork ledgers, not a line-by-line semantic proof of all algorithms. No candidate has a compatibility patch, no browser acceptance was rerun, and no performance gain is claimed from source alone. Source, historical receipts, fresh compile evidence and actual hardware execution remain distinct.

## Fork budget matrix

Budget means the kind of maintenance being bought, not an estimated schedule. Delta descriptions below come from the reviewed source/manifests/patch ledgers, not an independently reconstructed byte-exact upstream diff.

| Fork / source | Continuing owned obligation | Judgment | What permits shrinking or retiring it |
|---|---|---|---|
| Vano, pinned `b1f3d25d` | Broad VM integration, data-oriented heap/snapshot behavior, host hooks, realm/GC/interrupt/clock/buffer support; 64-bit target policy | **Strategic, high budget.** Most important fork to have a deliberate scope and regular upstream/test262 reconciliation | Exact Genet adapter/native/realm tests against upstream plus preserved heap-clone/host contracts. No generic parser swap removes VM semantic work |
| Boa/Bole, pinned `ddf4e31b` | Caller-realm provenance, finalization/GC semantics, narrow buffer-transfer feature and 32-bit/Wasm host behavior | **Strategic, medium-high budget.** Keep while it covers required targets and provides meaningful differential coverage; avoid implementing the same missing web APIs separately in each VM | Upstream host APIs satisfy the same tests, or an explicit target/support decision removes this VM. Refresh from upstream before new local semantic repairs |
| genet-taffy 0.16.0, upstream 0.15.1 | Static-position/containing-block seams, float bands, CSS order, containment, automatic-minimum cache semantics | **Keep small delta, medium budget.** Actual layout kernel already reused; latest refresh has retired real custom work | Authoritative complete-delta reproduction plus Buckram/Livery regression gates; remove one upstream-absorbed hunk at a time |
| genet-parley 0.10.0 | Join controls/fallback/spacing, float-line trailing whitespace, diagnostics, runtime dictionaries and paired fallback API | **Evaluate refresh, medium budget.** Challenge custom serialization first; avoid reimplementing shaping | Current fork tests against upstream Parley 0.12 or later compatible release, Windows/font/float/text corpus and initial-Wasm-size comparison |
| genet-fontique 0.10.0 | Windows real-cluster DirectWrite fallback instead of script sample/cache | **Upstream/retire, low-medium budget.** Platform correctness reason is concrete | Full cluster, UTF-16 range, scale and fallback-selection receipts on Windows; paired Parley API migration |
| Vello `276b4664` | Bounded append, shader traversal/progress/allocation/atlas recovery and curve quality across classic/sparse backends | **Upstream/retire per repair, medium-high budget.** GPU correctness must remain tested on actual producer/backend combinations | Matched image replay, no-hang/timeout, atlas persistence, allocation recovery and device-scale tests on native and browser routes; current later main alone is insufficient |
| Arboard custom-formats, local `bec2dcd` | Multi-format/custom MIME implementations on Win/macOS/X11/Wayland; file-list combination and persistence policy still need treatment | **Upstream/retire, medium platform budget.** Reuse already correct in direction; avoid another clipboard abstraction backend | External application format round trips and ownership/persistence on each OS, then same tests against upstream API. Consumer docs are stale relative to implementation |
| Regress 0.11.1 native form patch | Two-line nested negated UnicodeSets complement fix; native UTF-16 forms matcher only | **Retire promptly when absorbed, low budget.** Do not fork an entire regex engine further | Exact nested-set/form pattern tests on an approved registry release. Boa/Vano registry regex instances are separate and cannot be presumed fixed |
| ipc-channel 0.22.0 manifest fork | Default force-inprocess plus optional OS transport dependencies, to avoid platform dependency contamination | **Evaluate removing from retained media family, low technical budget.** Only reviewed production source consumers are media player/dummy/GStreamer; new worker-process transport does not use it | For single-process media, migrate channels to std/crossbeam if serialization/transfer behavior is unnecessary; otherwise upstream equivalent feature split. Check all public media APIs before replacement |
| sonic-rs 0.5.8 | Wasm ahash feature gating avoids getrandom 0.3; native retains runtime RNG | **Retire on dependency absorption, low budget.** This is platform dependency surgery, not local JSON ownership | Current sonic/ahash resolution achieves required Wasm entropy/dependency closure with native behavior unchanged; compile exact browser targets |
| gpu-allocator 0.28.0 vendored patch | Platform ABI/dependency compatibility around windows crate identities, as recorded in root manifest; exact current source delta not reconstructed in this audit | **Inventory exact delta then retire if redundant, unknown-low budget.** Do not claim a custom allocator algorithm | Byte diff against pristine version, compatible wgpu/Windows package identity and native DX12 allocation tests; a manifest patch can be cheaper than a renderer rewrite |

Graft/Scry/Weld are integration projects over reused engines, not three additional browser-engine forks owned by this stack. Their version-update, platform FFI, GPU import and callback synchronization budgets remain real and should be tracked separately from Genet standards work.
