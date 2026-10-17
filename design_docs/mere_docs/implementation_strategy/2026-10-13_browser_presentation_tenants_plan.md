# Browser presentation tenants implementation plan

> **For agentic workers:** Use superpowers:executing-plans for inline execution or superpowers:subagent-driven-development if the owner selects delegation. Complete each task's checks before moving to the next.

**Status:** coordinated adoption qualified, 2026-10-15, including merged upstream version/supplier updates. All four temporary Netrender bridge tables are retired. Earlier gate tuples remain explicit below.

**Continuation status (2026-10-16):** native shadow-mask/producer namespace correction and bounded Woodshed P4f receiving native/fresh-process qualification complete. Portable composition shadow-mask support retains its separate admission scope.

**Goal:** Run Graphshell's viewer through the shared Cambium host on direct WebGL2 when WebGPU is unavailable, including content updates, resize, recovery, capture, input and accessibility.

**Architecture:** Separate shared frame preparation from graphics presentation. Keep native/WebGPU device services in their adapter; add a direct Vello GPU WebGL2 adapter using Netrender's existing sparse lowering and portable producer content. Context generations govern graphics resources independently of content revisions.

**Tech stack:** Rust 1.99.0, declared MSRV 1.97.1, wasm-bindgen, web-sys, Genet paint contracts, Rootstock, Netrender, Vello GPU, browser WebGPU/WebGL2.

**Spec:** [Browser WebGL2 presentation tenants design](../../2026-08-12_family_composition_thesis_brief.md#browser-webgl2-presentation-tenants-design-2026-10-13).

## Global constraints

- Host owns graphics context, frame schedule and presentation. Renderer and content are tenants.
- First consumer is the viewer cone, Inspector and ordinary controls; other product feature combinations need separate qualification.
- Conatus's lit-body tenant remains domain-specific. Extend existing host and renderer owners.
- Presentation choice is configurable: `auto`, `webgpu`, `webgl2`. Forced choices report failure; ordinary draw errors never silently switch backend.
- Preserve the existing CPU dynamics route when compute is unavailable. Explicit unavailable compute requests report their outcome.
- Track resource owner/appearance, content revision, dimensions, alpha/encoding and context generation independently.
- Required unsupported paint and resource imports produce attributed refusals, never omitted content.
- Preserve CPU/application state through graphics loss. Reject old-generation handles and completions.
- Use the shared machine-heavy lock and verified kernel caps for heavy jobs, with swapping disabled. Read available resources before admission.
- Leave `/home/markik/Code/worktrees/mere-three-lanes` and its HEAD/index/config/locks/source untouched during Turnstone qualification.
- Work in `/home/markik/Code/worktrees/mere-webgl-presentation`, branch `webgl-presentation`, based on design commit `17f71d6fb`. Create a separate Netrender worktree when renderer implementation begins. Remove owned worktrees only after needed changes are integrated or saved for recovery.

## Review focus

1. A browser may bind a canvas context before backend boot fails: fallback must replace that canvas and keep input/focus bindings and model state.
2. Model state may advance while presentation is lost: restore the latest state and refuse stale capture completions.
3. A producer can retain the same image identity while changing pixels: content revision must invalidate the cache independently of context generation.
4. A hidden/zero-sized viewer must avoid invalid allocations and resume with a fresh frame at the actual scale.
5. A renderer can refuse an operation after earlier valid commands: preflight the complete frame before observable resource or framebuffer mutation.

## Source map and baseline

| Owner | Existing source | Responsibility |
|---|---|---|
| Rootstock | `src/frame.rs`, `src/host.rs`, `src/lib.rs` | Shared prepare/layout/redraw, presentation stamps, wgpu Surface |
| Rootstock | `src/producer.rs`, `src/producer/registry.rs` | Producer identity, texture route, portable scene export and admission |
| Rootstock | `src/capture.rs`, `src/recovery.rs` | Readback and graphics-loss lifecycle |
| Web host | `src/surface.rs`, `src/mount.rs`, `src/capture.rs`, `src/lane.rs`, `src/a11y.rs` | Browser target, event wiring, captures, ARIA mirror |
| Graphshell | `src/web_tree.rs`, `src/web_tree/presentation.rs`, `src/web_tree/physics.rs` | Viewer graph producer and capability selection |
| Graphshell | `web/scripts/build-viewer.py`, `web/tests/run-headed-receipt.py` | Shipping viewer build and browser receipt entry points |
| Netrender | `netrender/src/vello_backends.rs`, `vello_backends/sessions.rs`, `sessions/text.rs`, `sessions/validate.rs`, `fragments.rs` | Existing sparse lowering, images/text, complete admission, retained-source replay |

All Rootstock paths below have prefix `crates/cambium/cambium-rootstock/`; web-host paths have prefix `crates/cambium/cambium-genet-web-host/`; Graphshell paths have prefix `ports/graphshell/`. Netrender paths belong to its own isolated checkout.

### Task 1: Freeze dependencies and measure the viewer's actual paint inventory

**Files:** Modify this plan's Findings/Progress; create scoped receipts under `design_docs/mere_docs/testing/receipts/browser_presentation_20261013/`. No product code in this task.

**Interfaces:** Produces `baseline.json`, containing commit/lock/toolchain/browser identities, test commands/results and the viewer's actual paint operations and producer kinds. Later tasks use the frozen inputs, not the existing machine-local overlay graph.

- [ ] Check clean owned worktree status and available machine resources. Locate existing compiler/browser capped runners; acquire `/home/markik/Code/testing/machine-heavy.lock` for each heavy launch. Do not reuse a Turnstone source lease or generated target concurrently.
- [ ] Run portable metadata for the root and standalone viewer; archive lock hashes and resolved Genet/Netrender/Vello source IDs. Verify wasm-bindgen CLI matches `=0.2.129` before packaging. Preserve any resolution/build failure as a prerequisite failure, rather than rewriting the dependencies silently.

```sh
cargo +1.99.0 metadata --locked --format-version 1
cargo +1.99.0 metadata --manifest-path ports/graphshell/web/Cargo.toml --locked --format-version 1
```

- [ ] Through the admitted runner, run `cargo +1.99.0 test --locked -p cambium-rootstock -j1 -- --test-threads=1` and `cargo +1.99.0 check --locked -p cambium-genet-web-host --target wasm32-unknown-unknown -j1`. Run the existing viewer build script to a fresh output directory. Capture baseline graph, Inspector and transparent-image fixtures using existing scenario receipts.
- [ ] Use the actual portable paint capture to inventory operations, image/font resources, producer refusals and retained placements. List every operation required by the viewer against Netrender's current sparse admission. Keep full-product operations outside the viewer inventory.
- [ ] Commit the baseline receipt and Findings. Done when dependencies resolve from portable sources and the required operation inventory has an attributed baseline. Baseline failures are resolved before product edits.

### Task 2: Introduce presentation identity, policy and lifecycle contracts

**Files:** Create Rootstock `src/presentation.rs`, `src/presentation/tests.rs`; modify `src/lib.rs`, `src/host.rs`, `src/recovery.rs`.

**Interfaces:** The following new public types are the contract for Tasks 3–6. The snippet specifies signatures; implementations arrive in this task.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationChoice { Auto, WebGpu, WebGl2 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationBackend { Wgpu, WebGl2 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextGeneration(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationState { Ready, Lost, Restoring, Disposed }
pub struct PresentationLifetime {
    generation: ContextGeneration,
    state: PresentationState,
}
impl PresentationLifetime {
    pub fn new(generation: ContextGeneration) -> Self;
    pub fn generation(&self) -> ContextGeneration;
    pub fn state(&self) -> PresentationState;
    pub fn lose(&mut self);
    pub fn begin_restore(&mut self) -> Result<(), String>;
    pub fn finish_restore(&mut self) -> Result<ContextGeneration, String>;
    pub fn accepts(&self, generation: ContextGeneration) -> bool;
    pub fn dispose(&mut self);
}
pub fn backend_attempts(choice: PresentationChoice) -> &'static [PresentationBackend];
```

`Wgpu` identifies the shared graphics route; browser diagnostics also retain the actual BrowserWebGpu adapter identity. Successful boot, not presence of `navigator.gpu`, determines whether an attempt succeeds.

Add `context_generation: ContextGeneration` to `PresentedFrame` and migrate its
constructors. Add `CaptureRefusal::ContextLost { backend: PresentationBackend,
generation: ContextGeneration }` rather than inventing a wgpu device-loss notice
for WebGL. These changes brand shared captures and observations on either route.

- [x] Add failing state/policy tests. Require duplicate loss notifications to be idempotent, restore generation to advance once, overflow to refuse restoration, disposal to be terminal, and forced backend attempts to contain only the requested route.

```rust
#[test]
fn restored_context_refuses_old_completions() {
    let old = ContextGeneration(7);
    let mut lifetime = PresentationLifetime::new(old);
    lifetime.lose();
    assert!(!lifetime.accepts(old));
    lifetime.begin_restore().unwrap();
    let fresh = lifetime.finish_restore().unwrap();
    assert_ne!(fresh, old);
    assert!(!lifetime.accepts(old));
    assert!(lifetime.accepts(fresh));
}
```

- [x] Run the presentation tests to observe their missing-contract failure. Implement the state machine and export it. Put backend choice in host options with `Auto` as the documented configurable default; native hosts keep their current route.
- [x] Run presentation tests and existing Rootstock recovery tests through the admitted runner. Commit. Done when policy and lifecycle tests pass without creating a second device or mutating application state.

### Task 3: Extract shared frame preparation and portable graph production

**Files:** Create Rootstock `src/presentation/frame.rs`, `src/presentation/wgpu.rs`; modify `src/frame.rs`, `src/host.rs`, `src/producer.rs`, `src/producer/registry.rs`. Create Graphshell `src/web_tree/producer_scene.rs`; modify `src/web_tree.rs` and `src/web_tree/presentation.rs`.

**Interfaces:** Define `PreparedPaint::Native(PaintEnvelope)` and `PreparedPaint::Portable(PortablePaintCapture)` using the existing exported types. Define `PreparedFrame { paint: PreparedPaint, width: u32, height: u32, scale: f32, generation: ContextGeneration }`. Add `Host::prepare_presentation_frame(&mut self) -> Result<Option<PreparedFrame>, PresentationFailure>` under the existing Host generic bounds. `None` means a hidden/zero-size/deferred frame. `PresentationFailure` carries backend, generation, optional producer key, and an attributed message; it implements Display/Error. Task 4 translates portable packets; Task 5 installs the presentation adapter.

```rust
pub enum PreparedPaint {
    Native(paint_list_api::PaintEnvelope),
    Portable(crate::PortablePaintCapture),
}
pub struct PreparedFrame {
    pub paint: PreparedPaint,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub generation: ContextGeneration,
}
#[derive(Debug)]
pub struct PresentationFailure {
    pub backend: PresentationBackend,
    pub generation: ContextGeneration,
    pub producer_key: Option<u64>,
    pub message: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameContentMode { NativeTexture, PortableScene }
```

Preparation selects the mode reported by its installed presentation driver.
Native preparation stages current same-device producers and returns their paint
envelope; portable preparation uses complete portable admission. Both share
one frame hook, relayout, picking and semantic synchronization.

- [ ] Add CPU host fixtures proving frame hooks, dynamics and relayout run once, not once per rendering/capture route. Extend portable producer tests to cover removal, rebind, alpha metadata and partial-frame refusal. Keep an instrumented GPU-only producer that refuses portable export.
- [ ] Factor shared preparation out of `Host::redraw`. Preserve input/AT synchronization and frame observers. Wrap the existing wgpu raster/acquire/submit path in `WgpuPresentation`; its same-device services remain explicitly optional on `AppCtx`.
- [ ] Extract graph scene production from `TextureProducer::render` into `producer_scene.rs`, with one `produce(frame: &ProducerFrameInfo) -> Result<ProducedScene, PortableProducerError>` entry on the concrete viewer producer. Implement its existing `portable_scene` override through that entry. The wgpu render method consumes the same result, then rasterizes it. Keep dynamics timing, board selection, source identity and semantics unchanged.
- [ ] Use existing portable planning/admission to resolve content before committing frame lifecycle. Refuse unsupported GPU-only producers with their keys; keep old-pass native texture producers working.
- [ ] Add a hidden/zero-size/resume fixture: zero allocations while hidden, one current-state frame after resumption. Run Rootstock producer/accessibility/multi-host suites and Graphshell scene/dynamics tests; compare the native/WebGPU baseline captures. Commit. Done when both preparation modes share model/layout behavior and native same-device guards remain passing.

### Task 4: Add a direct WebGL2 realization to Netrender

**Files:** In an owned Netrender worktree, modify `netrender/Cargo.toml`, root `Cargo.toml`, `netrender/src/vello_backends.rs`, `netrender/src/vello_backends/sessions.rs`; create `netrender/src/vello_backends/webgl.rs` and `netrender/tests/webgl_scene.rs`. Extend existing sparse session validation/text helpers instead of copying lowerers.

**Interfaces:** Add `vello-webgl` with the same immutable Vello GPU/common/glifo source identities as sparse sessions. Target WebGL modules to wasm32. Export `WebGlSessionInit`, `WebGlSession`, `WebGlSessionError`; use `start(gl: WebGl2RenderingContext, limits: SparseResourceLimits) -> Result<WebGlSessionInit, WebGlSessionError>` and `try_finish(&mut self) -> Result<Option<WebGlSession>, WebGlSessionError>` for nonblocking initialization. `WebGlSession::render(&mut self, scene: &netrender::Scene, generation: u64) -> Result<(), WebGlSessionError>` validates the complete scene and generation before resource/target mutation. Reuse existing image-source setter/removal semantics.

- [ ] Create browser tests for ordered shapes, glyphs, transparent images, clips and opacity from Task 1's actual inventory. Feed a valid operation followed by an unsupported operation and assert unchanged framebuffer pixels/resources. Include a stale-generation refusal.
- [ ] Run these tests red before adding the session. Use the provider's initialization polling from successive animation frames. Extend existing sparse context lowering to the direct WebGL renderer; preserve admission, color/alpha conversions, glyph outline/text behavior and bounded resource limits.
- [ ] Upload CPU/browser image sources into this context. Keep token constructors private to the owning session so foreign-context textures cannot be registered by handle forgery. Define import refusal as part of `WebGlSessionError`; qualify replacement/removal of one image key without dropping unrelated images.
- [ ] Render on the physical AMD route already probed, then a second browser. Run existing CPU/Hybrid sparse resource, text, fragment and refusal suites. Commit the supplier and record immutable source before repinning Mere. Done when required viewer operations render or have an explicitly blocking refusal, with no partial-frame mutation.

### Task 5: Install shared browser presentation, fallback and recovery

**Files:** Create web-host `src/presentation.rs`, `src/presentation/webgl.rs`, `src/presentation/boot.rs`, `src/presentation/tests.rs`; modify `src/surface.rs`, `src/mount.rs`, `src/lib.rs`, `Cargo.toml`, and Rootstock `src/frame.rs`, `src/host.rs`, `src/presentation.rs`.

**Interfaces:** Add `PresentationDriver` with `backend`, `generation`, `resize`, `present`, `lose`, `restore`, `dispose` methods; `present(&mut self, frame: &PreparedFrame) -> Result<Option<PresentedFrame>, PresentationFailure>` returns a stamp only after successful presentation. `wgpu_surface(&self) -> Option<&dyn Surface>` provides existing device services only for the wgpu adapter. Own both concrete adapters behind this interface; do not fabricate a RenderCore for WebGL. Browser boot consumes `PresentationChoice` and reports the selected backend/context generation.

The signatures below are the task's required boundary. `restore` returns false
while rebuilding is in progress and true only when a fresh generation is ready.
Task 6 supplies the referenced pending-capture trait.

```rust
pub trait PresentationDriver {
    fn backend(&self) -> PresentationBackend;
    fn generation(&self) -> ContextGeneration;
    fn content_mode(&self) -> FrameContentMode;
    fn wgpu_surface(&self) -> Option<&dyn crate::Surface>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), PresentationFailure>;
    fn present(&mut self, frame: &PreparedFrame)
        -> Result<Option<crate::PresentedFrame>, PresentationFailure>;
    fn lose(&mut self);
    fn restore(&mut self) -> Result<bool, PresentationFailure>;
    fn begin_capture(&mut self, stamp: crate::PresentedFrame)
        -> Result<Box<dyn PendingPresentationCapture>, crate::CaptureRefusal>;
    fn dispose(&mut self);
}
```

Introduce the pending-capture declaration with this trait, then implement both
backends and migrate its consumers in Task 6. No intermediate commit may leave
an undefined type or uncompilable public interface.

- [ ] Add a fake-driver Rootstock test that successful, deferred and refused frames produce the correct observer/capture behavior. Ensure refused frames never repeat the prior successful stamp.
- [ ] Add browser boot tests for missing `navigator.gpu`, rejected device creation after canvas binding, forced backend refusal, and an ordinary render error that must not trigger fallback. Use isolated canvases, not global browser settings.
- [ ] Implement attempt ordering with fresh-canvas replacement when needed. Move listeners/accessibility binding and focus to the actual canvas without rerunning application initialization or creating a second model. Poll Vello initialization from rAF; keep the main thread available.
- [ ] Implement context loss/restoration through Task 2's lifetime. Invalidate renderer/image/font caches and capture requests, retain CPU content, rebuild from the current model, and refuse old-generation callbacks. Test two appearances to prove retiring one never destroys the other.
- [ ] Add pixel fixtures for the same-content-revision/new-context case and same-context/new-content-revision case. Run native/WebGPU multi-host and shared-device regressions. Commit. Done when shared redraw reaches either adapter and context replacement preserves input/semantics ownership.

### Task 6: Make capture and viewer configuration use the shared adapter

**Files:** Modify Rootstock `src/capture.rs`, `src/host.rs`; web-host `src/capture.rs`, `src/lane.rs`, `src/a11y.rs`; Graphshell `src/web_tree.rs`, `src/web_tree/physics.rs`, `src/web_tree/observe.rs`, `src/web_tree/host_boundary.rs`, `web/mount.js`, `web/Cargo.toml`, root `Cargo.toml` and both locks. Update all native capture consumers found by `rg 'CaptureFrame|StampedCaptureFrame|start_frame_readback' crates ports` in the same task.

**Interfaces:** Keep `PresentedFrame` as the successful presentation stamp. Introduce `PendingPresentationCapture::poll(&mut self) -> Option<Result<(PresentedFrame, Frame), CaptureRefusal>>`; backend-specific handles and row alignment remain private. Capture callbacks receive an owned pending capture instead of `Surface`/`TextureView` borrows. WebGL readback normalizes bottom-up framebuffer rows into the existing tightly packed top-down RGBA frame. Native/WebGPU use current asynchronous readback machinery.

```rust
pub trait PendingPresentationCapture {
    fn poll(&mut self)
        -> Option<Result<(crate::PresentedFrame, crate::Frame), crate::CaptureRefusal>>;
}
pub type PendingCaptureFn = Box<dyn FnOnce(
    Result<Box<dyn PendingPresentationCapture>, crate::CaptureRefusal>
) + 'static>;
```

Replace the old borrowed `CaptureFn`/`StampedCaptureFn` aliases and their
consumers with this owned callback contract in one compiling commit. The
presentation stamp contains the generation added in Task 2.

- [ ] Add red tests for odd-width RGBA rows, transparent edges, the last pixel after resize, loss between arm/completion, and refusing a completion from the previous context. Capture the rendered frame before later clears, and brand it with its actual presentation stamp.
- [ ] Route WebCapture/Mesquite through pending captures; migrate native consumers without changing their pixel or stamp contract. Keep captured-paint source export independent of screenshot readback.
- [ ] Expose `data-render-backend` and `?render_backend=` with exact values `auto`, `webgpu`, `webgl2`; explicit query choice overrides the element setting. Invalid values report a mount error. Report selected backend, context generation and compute availability in diagnostics.
- [ ] Default WebGL2 to existing CPU dynamics and test an explicitly requested unavailable GPU physics choice. Enable direct WebGL support additively in the browser viewer manifest; keep the same portable provider identities across root and standalone locks.
- [ ] Run native capture guards, Wasm host/viewer checks and the existing locked viewer packaging budget. Record bundle growth and any budget refusal instead of raising a cap silently. Commit. Done when configuration, capture and capability disclosure work through the actual viewer.

### Task 7: Qualify the real viewer and integrate the owned branches

**Files:** Create Graphshell `web/scenarios/presentation_webgl2.scn`, `web/tests/presentation-webgl-fixture.html`, `web/tests/run-presentation-webgl.py`; extend existing `web/tests/run-headed-receipt.py` and `capture-page.mjs` only for controls unavailable to the existing harness. Save receipts in the canonical browser presentation receipt directory. Update this plan and DOC_README.

**Interfaces:** Driver arguments are `--web`, `--out`, `--browser`, `--backend`, `--disable-webgpu`, `--force-context-loss`. The result JSON records source/bundle/browser/adapter identities, selected backend, presentation generation, every asserted pixel/semantic identity, resource admission/cap/cleanup data and an explicit pass/refusal outcome. Output must be new; reuse no unrelated browser profile.

- [ ] Write the real-viewer scenario: mount disclosed fixture, show Inspector, select/pin a node, update graph, qualify root-paint image replacement/removal in the actual consumer-host fixture, resize/scale, hide/resume, lose context, mutate while lost, restore, capture, remove/recreate an appearance and dispose. Declare analytic pixel expectations and baseline comparison tolerances before execution.
- [ ] Run software controls separately from physical AMD hardware. With WebGPU unavailable, verify the WebGL2 route and its update/loss/recovery behavior. Run a second browser implementation with equivalent functional assertions. Re-run WebGPU and native guards using the exact merged dependency source.
- [ ] Verify equivalent picking, keyboard focus/actions, clipping and ARIA mirror targets. Preserve human AT as an open manual gate rather than counting automated mirror checks as human validation. Run negative controls for missing context, unsupported paint and stale resource/capture identity.
- [ ] Freeze source for each heavy lease and revalidate watched files and HEAD before and after execution. Preserve every raw failure; do not rewrite baselines to erase unexplained movements.
- [ ] Review the complete branch, record supplier/consumer adoption, and integrate in a safe window after Turnstone releases its source lease. Preserve concurrent changes. Verify clean integrated source, portable dependency identities and required guards before publication.
- [ ] Remove only owned worktrees after their commits are integrated or preserved by a recoverable ref. Use `git worktree remove` for this manually created Mere worktree; preserve needed ignored receipts outside it first. Never force-remove a dirty checkout. Record cleanup and remaining feature/manual gates.

## Findings

- 2026-10-13: Mere worktree created at `17f71d6fb`. Native worktree creation was unavailable because this chat's cwd is not a Git repository, so the authorized Git fallback was used. Turnstone's checkout was not edited.
- 2026-10-13: `TextureProducer::portable_scene`, `ProducedScene`, `ProducerRegistry::plan_portable/admit_portable` and `Host::capture_portable` already exist. The concrete browser viewer producer still implements the GPU render path only, so it needs shared scene production rather than a duplicated graph renderer.
- 2026-10-13: Existing `CaptureFrame` and `StampedCaptureFrame` expose wgpu Surface/TextureView. Capture migration is part of the presentation change, not a renderer-only flag.
- 2026-10-13: Netrender's `vello_backends/sessions.rs` already owns CPU/Hybrid image/text admission. Direct WebGL should reuse that validation and lowering, with graphics upload/submission kept in the new backend module.

## Progress

- 2026-10-13: Design approved; owned Mere worktree created. Implementation plan drafted for owner review. No product edits or heavy builds launched. Worktree remains needed for the next execution stage; cleanup is required at integration/retirement.

- 2026-10-13: Root locked offline metadata resolves. Standalone viewer lock required two source-identity additions after the supplier move: luggage at Mere fabfd10ef and mere-webrtc-carrier at Retinue b49df894; versions and transitive edges unchanged. Original refusal and delta are preserved in baseline-preparation receipts. Baseline compiler/browser gates await the shared heavy slot.

- 2026-10-13: Native Rootstock baseline passed 87 tests with two existing ignores, no failures and zero memory pressure/OOM events on frozen source 230a41cd0. The policy test first failed on its missing exports, then all seven tests passed against the new implementation. Presentation stamps now carry context generation, and host options expose the browser policy. Broader host/capture regressions remain pending; Task 2 is not complete. Compact evidence is in `testing/receipts/browser_presentation_20261013/policy/red-green.json`.
- 2026-10-13: Execution rulings: keep the baseline in a private snapshot; finish measured portable producer inventory after the concrete viewer export exists; use Vello's public canvas initialization rather than an invented context-taking constructor; include owned listener/rAF teardown in browser disposal; prevent a portable export from stepping presentation dynamics twice.

- 2026-10-13: Task 2 complete: policy/lifetime red-to-green and combined Rootstock/Mesquite/native-host regression gate passed 294 tests, zero failures and 11 existing ignores, with unchanged source, zero memory-limit events and verified service cleanup. Task 1 Wasm/browser baseline remains separate; Task 3 shared frame preparation begins.

- 2026-10-13: Task 3 producer invocation contract passed its registry red-to-green fixture: presentation advances once; explicit export observes the same revision and content stamp. Kernel memory peak 876265472 bytes, zero pressure/OOM events, source unchanged, service inactive/MainPID 0. Shared frame preparation and concrete viewer extraction remain unqualified.

- 2026-10-13: Shared preparation and the concrete graph producer extraction pass 299 native host/recovery/capture tests (zero failures, 11 existing ignores) and the standalone viewer's locked Wasm check. Portable ownership, alpha/encoding, rebind, retirement, suspension and no-double-step fixtures use real registry callbacks. WgpuPresentation keeps the existing typed raster/acquire causes; the host commits prepared identity only after presentation. Evidence: `testing/receipts/browser_presentation_20261013/frame-preparation/red-green.json`.
- 2026-10-13: Frozen viewer Wasm build and matching bindgen packaging pass. The current Chrome/ANGLE Vulkan receipt cannot mount because WebGPU supplies no adapter; it times out without a successful frame. Its source-bound refusal is retained. The legacy Flatpak harness does not keep Chrome in the parent cgroup, so its RAM result is excluded; future tests will run the installed binary directly. Baseline gzip6 Wasm is 3,400,367 bytes against 3,407,872, leaving 7,505 bytes. Actual portable operation inventory and render comparisons move to the real adapter entry, with these qualification boundaries intact.

- 2026-10-13: Installed-driver redraw forwarding passes its two new whole-host fixtures and 301 combined native tests, with zero failures. Rootstock rejects a driver stamp from another context and retains sequence across deferred/refused frames. The legacy surface path remains covered; concrete browser driver, owned capture and disposal work are still pending. Supplier standalone GL fixtures now pass Chrome AMD and Firefox, plus 28 CPU/Hybrid sparse regressions.

- 2026-10-14: Portable composition passes four CPU controls: painter order/opacity/scale, source viewport clips, disjoint owner-local image keys, stable identity under producer-ledger reordering, complete source/stamp admission and keyed unsupported producer paint. It expands retained sources through Netrender’s existing fragment resolver. Shadow masks remain explicitly refused pending actual viewer inventory. The immutable supplier resolves once across Genet, Mere and standalone viewer via the HTTPS source alias. Renderer WESL exact0.4.0 conflicts with the root’s prior0.4.4 lock; compatible packages resolve at0.4.0, with Tenant validation still pending. Resolution intentionally changes locks and is not an unchanged-source test receipt.

- 2026-10-14: Inline implementation checkpoint: shared browser adapters, owned pending captures, generation/loss cleanup, exact shared wgpu surface factory, fallback canvas identity and viewer policy are implemented. Final native regressions pass 308 tests with 11 existing ignores; Tenant and native receipt consumers compile. Actual final viewer passes Chrome/AMD Auto fallback, Firefox WebGL2 and Chrome native Vulkan WebGPU controls. Boot rejection, partial listener construction and silent GPU-copy errors have red/green ownership controls. Canonical evidence is `testing/receipts/browser_presentation_20261013/presentation/qualification.json`.
- 2026-10-14: Acceptance remains bounded: Wasm gzip6 is 3,517,194 bytes against the unchanged 3,407,872 cap; pinned glue minification passes its cap. At scale 1, static-state backend comparisons pass RMS1/max16/alpha0; at scale 1.25 two comparisons refuse RMS1 (1.26/1.19), with identical alpha and a passing resized comparison. Preserve these gates, human AT, and the existing browser asynchronous device-loss replacement boundary. Whole-branch review, integration and owned checkout cleanup are pending.

- 2026-10-14: The single whole-branch review found five Important code issues, all reproduced and repaired in one inline fix pass. Final native309/0/11 and actual viewer Chrome/AMD32, Auto32, Firefox32, software32, WebGPU26 pass. Boot/constructor/pending-fallback, silent upload/JS exception and uncaptured-refusal retry controls pass. Consumer-host image replacement/removal passes five controls; the bounded viewer census is zero images, so viewer image-loading remains outside this first acceptance. See `testing/receipts/browser_presentation_20261013/presentation/review/README.md`.
- 2026-10-14: Delivery-final gzip6 is 3,674,899 bytes, still refused by the unchanged 3,407,872 cap; no cap was raised. The earlier 3.5 MiB proposal is insufficient; a 3.75 MiB owner decision is pending. Fresh final-artifact normal comparisons pass and fractional comparisons preserve RMS1 refusals (1.26449/1.19021), alpha0, passing odd resize. Preserve both old and new receipts.
- 2026-10-14: Shared-main integration is deferred while acceptance decisions remain open. Preserve supplier/consumer draft branches, archive raw inline evidence and remove only the two owned worktrees. The frozen Turnstone checkout remains untouched. Heavy gates are clear:35 review services inactive/MainPID0 and the shared lock free; Turnstone received the authorized resource-release handoff.

- 2026-10-14: Integrated on Mere main (`27a87059a`) with Netrender `68bcddc06`; lane branches archived. Main's `MountLifetime` is the browser disposal authority; conflict decisions are in `design_docs/2026-10-14_mere_lane_handoff.md`. A follow-up commit restored main's lockfile edges that re-resolution had changed (`spareval` had taken `sha2` 0.10 and broken the default-feature viewer).
- 2026-10-14: Fractional-scale limit cleared by aligning composition (owner option B2). Browser WebGPU now presents the same materialized portable scene as direct WebGL2 (`HostOptions::wgpu_portable_scene`, `WgpuDriver::with_portable_scene`), so the graph producer draws in the page pass with no intermediate texture or bilinear resample. Graphshell opts in and binds its GPU-physics device from the frame hook. Unchanged contracts: 125% RMS 0.258/0.332/0.328 (was 1.265/1.190/0.543), max 11, alpha 0; scale 1 unchanged at 0.252/0.300/0.299. Viewer: WebGL2, Auto, Firefox and SwiftShader 32 controls, WebGPU 26; native 855 passed. Receipt: `testing/receipts/browser_presentation_20261013/presentation/limits-followup-20261014/`.
- 2026-10-14: Delivery limit not cleared (owner option A2, structural reachability). Wasm gzip6 is 3,661,762 against 3,407,872; pre-merge main measured 3,394,183, so the cut must offset nearly all of the lane's +268,796 bytes. Named-symbol attribution: code grew 1.145 MB; the largest removable duplication is in suppliers — Genet layout's second Taffy instantiation (`InlineMeasure`, about −113 KB gzip) and one-pen-per-outline instantiations across Netrender and the Vello fork (about −56 KB). Together they leave about 85 KB unaddressed; the rest is the WebGL2 renderer itself. Supplier scope is an owner decision.
- 2026-10-15: Supplier de-generification (Mark's go-ahead). Genet `e8b8606f5` runs layout over a context-free `AlgorithmCore`, so Taffy is instantiated once per style type; the Vello fork (`mark-ik/all-vellos` `276b4664`) and Netrender `5fbc7b2ba` draw every glyph outline through `Vec<PathElement>`, one Skrifa interpreter copy. Measured gzip6: 3,661,762 before; 3,523,495 with only these changes (local Genet = old pin + `e8b8606`); 3,620,876 at the real pin, because Genet main also brings nine other commits (about +97 KB). Mere adopts Genet's browser `RendererAccess` frame-gate guard per operation: checked `try_renderer`, loss-safe `retire_owned_*` cleanup, no guard across producer callbacks. Native 855 passed; five browser routes and both scale comparisons unchanged. The single bundle stays over the cap; the split's estimated savings and required gating are in `limits-followup-20261014/limits-followup.json`.
- 2026-10-15: Per-route viewer artifacts (Mark: split, with Classic Vello optional in the suppliers). Netrender `81c8337a4` adds the subtractive `without-classic-vello`: Renderer cannot be constructed and every method is statically unreachable, so Classic's code and shaders are not linked. The web host gains `webgpu` and `portable` features; `build-viewer.py` builds `pkg/webgpu/` (`presentation-webgpu`) and `pkg/webgl2/` (`presentation-webgl2` + `without-classic-vello`) and holds each to the unchanged caps. `mount.js` loads one artifact: explicit policy loads its route; Auto probes for an actual WebGPU device and remounts on WebGL2 if the WebGPU artifact refuses at boot. gzip6: WebGL2 3,244,444 (passes); WebGPU 3,454,488 (46,616 over, reported, not cut). All routes pass on their own artifact (WebGL2/Auto/Firefox/SwiftShader 33 controls, WebGPU 27, Auto with WebGPU loads the WebGPU artifact); both scale comparisons pass. Genet render host's matching feature is the Genet lane's.
- 2026-10-15: Closed. Mark kept Classic Vello as an opt-out in the suppliers and amended R157's viewer Wasm cap to 3,460,300 B (3.30 MiB) per route artifact. Pins: Genet `ae3d9f5d972` (genet-render-host `without-classic-vello`), Netrender `f4f9c0d87` (the opt-out is a compile error off wasm32). gzip6: WebGPU 3,359,673, WebGL2 3,322,091; both pass the amended cap and, at these pins, also the earlier 3,407,872 B one. Fat-LTO layout varies by about 95 KB between builds with only cfg-level changes, so the extra headroom is real margin. All routes pass on their own artifacts; scale 1 and 125% comparisons pass; native 855 passed. Recorded options, not tasks: slim Genet main's recent browser-capture work in the viewer, align genet-parley onto one read-fonts.


## Coordinated supplier adoption, 2026-10-15

**Scope:** Genet adopts Netrender `f4f9c0d87e6a6030b61075a45118055dfe6a12df`
and its maintained Vello `276b46648c08f3d2062e70559afa999842e1873c` source.
Genet forwards `genet-render-host/without-classic-vello` to the renderer.
Mere then adopts that published Genet revision and removes the temporary
Netrender patch family from its root, viewer, web smoke and Distillery probe.
Direct renderer dependencies use the canonical supplier URL and one revision.
Viewer size levers remain options. Forms C and three-lanes are outside scope.

### Phases and done-conditions

1. Reconcile the primary Genet checkout with the newer published Phase B.
   Done when the older local snapshot is verified as superseded by the
   qualified upstream implementation, without replaying duplicate work.
   Mark clarified that a duplicate archive tag is unnecessary.
2. Update Genet's coordinated renderer/provider pins, feature forwarding and
   standalone smoke. Run render, render-host and Livery suites, the wasm opt-out
   check and its expected native rejection. Done when actual results and source
   identities are recorded, with any platform limits stated.
3. Publish the Genet supplier and adopt it in the four active Mere roots.
   Remove their bridge tables, refresh portable locks and inspect resolved
   identities. Run native consumer tests and browser host/viewer compilation,
   including the viewer's opt-out. Done when each root resolves one canonical
   Netrender family without the bridge and the owned changes are published.

### Findings

- 2026-10-15: Windows main held `b5813e22291`, an older Forms B snapshot.
  Remote main already contains qualified Phase B `a0d95ebd6e2`, additional
  validation fixes and subsequent Forms work. The older snapshot was not
  replayed over main. Its temporary duplicate archive tag was removed locally
  and remotely at Mark's clarification.
- 2026-10-15: The renderer bridge occurs in four active Cargo roots, not only
  the workspace and viewer. The web smoke and Distillery probe still used
  `6a24ae08f`; all four must move together before removing their bridges.
- 2026-10-15: Gates run from `Code` with explicit manifest paths, so the
  Genet checkout's ignored local supplier patches cannot replace published
  sources. Build output uses stable `C:/t/cargo-targets/genet` and
  `C:/t/cargo-targets/mere` paths. Existing unrelated builds are left alone.

### Progress

- 2026-10-15: Original Forms B snapshot tagged; primary Genet reconciled to
  remote main `8649c0c3200`. Mere fast-forwarded to published `99ecfdee`.
  Genet's renderer and provider pin updates and feature forwarding are prepared.

- 2026-10-15: Genet supplier published as `7ffe66208ca3059283ef8770de7d17aa8834ecc8`.
  Fresh Windows gates: Livery 678 passed/9 ignored, render 51 passed,
  render-host 17 passed/4 ignored, zero failures; wasm opt-out and standalone
  compile pass, and native opt-out is rejected by Netrender as intended.
  Receipt: `genet/design_docs/receipts/2026-10-15_netrender_adoption/README.md`.
  Prepublication rebase preserved upstream Forms documentation and one
  Documents regression test (`cb735d928a8`, `273431eff73`); gated production
  source and the qualified manifests are unchanged by that rebase.
  All four Mere roots now select this published Genet supplier; portable lock
  refresh and consumer qualification are in progress.

- 2026-10-15: All four portable metadata audits pass: exactly one canonical
  Netrender family at `f4f9c0d87`, and every resolved Genet Git package at
  `7ffe66208ca`. The main lock changes only Genet/Netrender source identities.
  The older Distillery probe lock also reconciles the existing workspace
  `edit-history` -> `backstep` rename, adds the Genet host's Gloo timer edge,
  and adopts the same Vello provider. Native host suites are compiling.

- 2026-10-15: Receiving native host suites pass 336 tests, zero failures,
  with 11 existing ignores (including four doctests). Viewer WebGPU/WebGL2
  and standalone web smoke wasm checks are running against the frozen locks.

- 2026-10-15: Mark clarified to drop the duplicate snapshot and follow published
  main. The temporary Forms B archive tag was removed locally and remotely;
  Genet main fast-forwarded to `eeb9774ada9`. The qualified Netrender adoption
  source stays pinned at `7ffe66208ca`; subsequent accessibility changes retain
  their separate upstream receipts. Both viewer wasm routes compile; the
  standalone web smoke gate is running.

- 2026-10-15: Coordinated adoption complete. All four portable roots resolve
  one canonical Netrender family at `f4f9c0d87` and published Genet
  `7ffe66208ca`. Native receiving tests: 336 passed, zero failed, 11 ignored.
  Both viewer wasm routes (WebGPU and WebGL2 with the forwarded Classic
  opt-out) and standalone web smoke compile against locked public suppliers.
  Source/config and lock hashes are unchanged after qualification. Receipt:
  [receiving audit and logs](../testing/receipts/2026-10-15_netrender_adoption/README.md).
  Viewer size levers remain options. Forms C and three-lanes were untouched;
  the unnecessary duplicate Forms B archive tag was removed at Mark's request.
  Build output remains in stable `C:/t/cargo-targets/genet` and
  `C:/t/cargo-targets/mere`; no isolated Cargo home or worktree was created.

- 2026-10-15: Upstream Mere advanced to `75a2e6ae` during qualification.
  Rebase preserves its Murm `fb6360c12b`, script-engine-api minimum version,
  package version bumps and other published work. Conflicts were restricted to
  the shared manifest and index; both owners' changes are retained. Fresh
  four-root metadata audits and all receiving gates are running on merged
  source. Original `99ecfdee` receipts remain unchanged as prior evidence.

- 2026-10-15: Merged-source qualification passes on `16104bbd` over upstream
  `75a2e6ae`: four dependency identity audits, 336 native tests with zero
  failures and 11 existing ignores, both viewer wasm routes and standalone
  web smoke. Six manifest/config and four lock hashes remain unchanged.
  The earlier interrupted lock wait ran zero tests. The final receiving
  receipt separates this integrated run from the original `99ecfdee` run.
  `C:/t/cargo-targets/mere/netrender-adoption` is retained for this gate:
  automatic approval review blocked cleanup after its Cargo marker and lack
  of live owners were verified. No isolated Cargo home or worktree was made.

## Native shadow-mask admission continuation (2026-10-16)

### Findings

- 2026-10-16: Woodshed's Rehearsal scene reaches Rootstock frame preparation
  but presents zero frames. In `crates/cambium/cambium-rootstock/src/producer/registry.rs`,
  GPU remapping rejects every translator-owned blurred-shadow mask because
  its generated key occupies the high external-image namespace. Removing
  only the mask-list refusal would still misclassify its image operation as
  a producer reference.
- 2026-10-16: Netrender generates shadow-mask keys from
  `0xFFFF_0000_0000_0000`; its renderer retains image overrides across frames.
  Preserve those generated keys and reserve their range from Rootstock's
  physical producer allocation. Moving masks into ordinary inline-image keys
  could let an old override capture a later inline image. Authored producer
  keys still admit all lower 62 bits, independently of physical allocation.
- 2026-10-16: External-texture metadata identifies each producer image's exact
  scene-operation boundary. That attribution distinguishes a producer whose
  local key numerically equals a generated mask key. Pattern references remain
  producer references. Validate the complete scene before rewriting anything;
  malformed mask ownership, stale producer stamps and opaque fragments still
  refuse the frame. Portable composition's shadow-mask admission stays separate.

### Phases and done-conditions

1. Reproduce refusal using the real paint translator's blurred-shadow command.
   Done when the focused regression fails at the producer remapping boundary.
2. Partition physical producer allocation below the mask range and use mask
   ownership plus producer draw boundaries during remapping. Done when shadow,
   mixed producer/clip/opacity, high-local-key collision, transactional refusal
   and malformed-mask controls pass, along with Rootstock/native-host suites.
3. Publish the supplier revision for Woodshed intake. Done when its exact
   locked graph builds, the explicit Related/Set/Card/Rehearsal flow presents,
   captures are reviewed and the same Set reopens in a fresh process. Receiving
   runtime results remain the Woodshed owner's responsibility and receipt.

### Progress

- 2026-10-16: Worktree reconciled to Mere `e43fcee761dc`; six focused regression
  controls added against the real translator. Mere owns the supplier correction
  and serialized Cargo checks; Woodshed holds its intake/native gates until
  the published revision arrives. No supplier selector or lock changes planned.
- 2026-10-16: The shadow-only control first reproduced the exact
  `ForeignInlineImage` refusal at generated key `0xFFFF_0000_0000_0000`.
  Physical producer allocation now ends below that band; mask ownership and
  exact producer draw boundaries preserve both generated images and the full
  authored local-key domain. All six focused controls pass. Rootstock and
  native-host suites pass 307 tests, zero failures, six existing ignores on
  the unchanged locked graph (Genet `e37762d752e`, Netrender `f4f9c0d87e`).
  Independent read-only review found no correctness blocker. The opt-in frame
  trace now attributes producer-resource refusals before presentation.
  [Qualification receipt](../../../support/receipts/producer-shadow-20261016/qualification.json)
  records commands, source/lock hashes and the separate receiving gate.
  The documentation audit adds no finding in this continuation; existing
  unrelated whole-tree findings remain. Woodshed native/reopen is pending.
- 2026-10-16: Woodshed receiving qualification passes on Mere `6575075057e`,
  consumer Rust source `d2adb09ac3b` and scenario-only successor `7500d1b`.
  Its preserved Netrender `8960024f7b` supplier independently exercises the fix
  beyond Mere's supplier-test graph. The guarded seed presents 342 frames,
  a fresh-process reopen 110 and the default/narrow layout phase 112.
  Fourteen captures were independently reviewed, including the final-binary
  save-refusal close panel. Both Stage and Rehearsal show the retained four-note
  realization; exact owner/Card IDs, instructions, realization float bits,
  cursor, projection, workspace and personal journal survive reopening.
  Directed and selected-Card auditions each reach the same output runtime
  without changing those durable facts. Receiving checks pass 789 tests plus
  five separate fresh-process child checks, with seven existing host ignores.
  [Receiving closure](../../../support/receipts/producer-shadow-20261016/woodshed-receiving.json)
  binds the source, binary, supplier graph and owning consumer receipt.
  Failed/mixed earlier trials are excluded. Physical audio, other platforms
  and portable shadow-mask composition retain their separate qualification.
