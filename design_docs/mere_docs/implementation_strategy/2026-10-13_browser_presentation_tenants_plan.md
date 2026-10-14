# Browser presentation tenants implementation plan

> **For agentic workers:** Use superpowers:executing-plans for inline execution or superpowers:subagent-driven-development if the owner selects delegation. Complete each task's checks before moving to the next.

**Status:** in progress, 2026-10-13. The owner approved the written design and selected inline execution. Task 1 baseline qualification is active; product implementation has not started.

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

- [ ] Add failing state/policy tests. Require duplicate loss notifications to be idempotent, restore generation to advance once, overflow to refuse restoration, disposal to be terminal, and forced backend attempts to contain only the requested route.

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

- [ ] Run the presentation tests to observe their missing-contract failure. Implement the state machine and export it. Put backend choice in host options with `Auto` as the documented configurable default; native hosts keep their current route.
- [ ] Run presentation tests and existing Rootstock recovery tests through the admitted runner. Commit. Done when policy and lifecycle tests pass without creating a second device or mutating application state.

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

- [ ] Write the real-viewer scenario: mount disclosed fixture, show Inspector, select/pin a node, update graph and image, resize/scale, hide/resume, lose context, mutate while lost, restore, capture, remove/recreate an appearance and dispose. Declare analytic pixel expectations and baseline comparison tolerances before execution.
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
