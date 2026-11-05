// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Real host turns and worker wakes, without fabricating a swapchain frame.

use super::*;
use crate::{
    HostHooks, HostOptions, HostState, HostWake, ProducedTexture, ProducerContext, Runner,
    ScriptedDom, SourceAlpha, SourceEncoding, TextureProducer,
};
use ::wgpu;
use cambium::{AnyView, GenetCtx, GenetElement, custom_leaf};
use std::{
    cell::{Cell, RefCell},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;
type TestHost = Host<(), Logic, Child>;
fn root(_: &()) -> Child {
    Box::new(custom_leaf(7, 8, 6))
}

struct Window {
    size: Rc<Cell<(u32, u32)>>,
    requests: Rc<Cell<usize>>,
}
impl crate::HostWindow for Window {
    fn request_redraw(&self) {
        self.requests.set(self.requests.get() + 1);
    }
    fn inner_size(&self) -> (u32, u32) {
        self.size.get()
    }
    fn scale_factor(&self) -> f64 {
        1.
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}
struct Headless {
    core: Arc<genet_render_host::RenderCore>,
    acquisitions: Rc<Cell<usize>>,
    drops: Arc<AtomicUsize>,
    reconfigured: Rc<Cell<bool>>,
}
impl Surface for Headless {
    fn core(&self) -> &genet_render_host::RenderCore {
        &self.core
    }
    fn shared_core(&self) -> Option<Arc<genet_render_host::RenderCore>> {
        Some(self.core.clone())
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        self.acquisitions.set(self.acquisitions.get() + 1);
        None
    }
    fn acquire_status(&self) -> genet_render_host::SurfaceAcquire {
        if let Some(loss) = self.core.device_failure() {
            return genet_render_host::SurfaceAcquire::DeviceLost(loss);
        }
        if self.reconfigured.replace(false) {
            self.acquisitions.set(self.acquisitions.get() + 1);
            return genet_render_host::SurfaceAcquire::Reconfigured;
        }
        let _ = self.acquire();
        genet_render_host::SurfaceAcquire::Unavailable {
            reason: "headless control has no swapchain".into(),
        }
    }
}
impl Drop for Headless {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Default)]
struct Probe {
    attempts: usize,
    commits: usize,
    discards: usize,
    accepted: usize,
    candidate: Option<ProducedTexture>,
    invalid_shader: bool,
}
impl TextureProducer for Probe {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        panic!("legacy route")
    }
    fn supports_native_staging(&self) -> bool {
        true
    }
    fn render_staged(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, RasterizeFailure> {
        self.attempts += 1;
        if self.invalid_shader {
            // A real device validation failure captured by the active batch,
            // while this source still returns an otherwise valid image.
            let _invalid = cx
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("native-host-invalid-producer-shader"),
                    source: wgpu::ShaderSource::Wgsl("this is deliberately invalid WGSL".into()),
                });
        }
        let [w, h] = cx.frame.physical_size;
        let mut scene = netrender::Scene::new(w, h);
        scene.push_rect(0., 0., w as f32, h as f32, [0.3, 0.5, 0.7, 1.]);
        let (_, view) = cx.try_rasterize_scaled_for(
            917,
            &scene,
            w,
            h,
            netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.,
        )?;
        let output = ProducedTexture {
            view,
            generation: self.attempts as u64,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        };
        self.candidate = Some(output.clone());
        Ok(Some(output))
    }
    fn publish_staged(&mut self) {
        self.commits += 1;
        self.accepted = self.attempts;
        self.candidate = None;
    }
    fn discard_staged(&mut self) {
        self.discards += 1;
        self.candidate = None;
    }
    fn suspend(&mut self) {
        self.candidate = None;
    }
}

struct Fixture {
    host: TestHost,
    core: Arc<genet_render_host::RenderCore>,
    producer: Rc<RefCell<Probe>>,
    size: Rc<Cell<(u32, u32)>>,
    model: Rc<Cell<usize>>,
    accepted: Rc<Cell<usize>>,
    prepared: Rc<Cell<usize>>,
    application_wakes: Rc<Cell<usize>>,
    acquisitions: Rc<Cell<usize>>,
    drops: Arc<AtomicUsize>,
    wakes: Arc<AtomicUsize>,
    requests: Rc<Cell<usize>>,
    reconfigured: Rc<Cell<bool>>,
}
fn fixture() -> Fixture {
    let core = Arc::new(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(16),
            ..Default::default()
        })
        .expect("actual native adapter required"),
    );
    fixture_with_core(core)
}
fn fixture_with_core(core: Arc<genet_render_host::RenderCore>) -> Fixture {
    let mut state = HostState::<(), Logic, Child>::new();
    state.shared.sheet = "custom-leaf{display:block;width:8px;height:6px}".into();
    state.shared.render_core = Some(core.clone());
    let producer = Rc::new(RefCell::new(Probe::default()));
    state
        .shared
        .producers
        .register(7, producer.clone(), &[])
        .unwrap();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        (),
    ));
    let size = Rc::new(Cell::new((16, 16)));
    let requests = Rc::new(Cell::new(0));
    state.window = Some(Box::new(Window {
        size: size.clone(),
        requests: requests.clone(),
    }));
    let acquisitions = Rc::new(Cell::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let reconfigured = Rc::new(Cell::new(false));
    state.surface = Some(Box::new(Headless {
        core: core.clone(),
        acquisitions: acquisitions.clone(),
        drops: drops.clone(),
        reconfigured: reconfigured.clone(),
    }));
    let wakes = Arc::new(AtomicUsize::new(0));
    let signalled = wakes.clone();
    let wake = HostWake::new(
        state.wake_pending.clone(),
        Arc::new(move || {
            signalled.fetch_add(1, Ordering::SeqCst);
        }),
    );
    let model = Rc::new(Cell::new(7));
    let accepted = Rc::new(Cell::new(0));
    let prepared = Rc::new(Cell::new(0));
    let (read, write, count) = (model.clone(), accepted.clone(), prepared.clone());
    let mut hooks = HostHooks::inert();
    let application_wakes = Rc::new(Cell::new(0));
    let observed_wakes = application_wakes.clone();
    hooks.after_wake = Box::new(move |_| observed_wakes.set(observed_wakes.get() + 1));
    hooks.prepare_presentation = Box::new(move |_, _| {
        count.set(count.get() + 1);
        let fact = read.get();
        let write = write.clone();
        Some(PreparedPresentationObserver::new(move |_| write.set(fact)))
    });
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    // A loss control must never boot a replacement GPU on its UI test thread.
    host.attach_shared_recovery(genet_render_host::SharedRenderCore::with_factory(
        core.clone(),
        Arc::new(|| Err("replacement intentionally disabled in lifecycle control".into())),
    ));
    Fixture {
        requests,
        reconfigured,
        host,
        core,
        producer,
        size,
        model,
        accepted,
        prepared,
        application_wakes,
        acquisitions,
        drops,
        wakes,
    }
}

mod terminal;
mod wake;
mod work;
type Outcomes = Rc<RefCell<Vec<(&'static str, CaptureRefusal)>>>;
fn captures(host: &mut TestHost) -> Outcomes {
    let out = Rc::new(RefCell::new(Vec::new()));
    fn image(kind: &'static str, out: Outcomes) -> crate::CaptureFn {
        Box::new(move |result| {
            out.borrow_mut().push((
                kind,
                result.err().expect("unpresented image must be refused"),
            ))
        })
    }
    host.s.pending_capture = Some(image("image", out.clone()));
    host.s.pending_stamped_capture = Some(image("stamped", out.clone()));
    let paint = out.clone();
    host.s.pending_paint_capture = Some(Box::new(move |result| {
        paint.borrow_mut().push((
            "paint",
            result.err().expect("unpresented paint must be refused"),
        ))
    }));
    out
}
fn assert_unpublished(f: &Fixture) {
    assert_eq!(f.accepted.get(), 0);
    assert_eq!(f.producer.borrow().accepted, 0);
    assert_eq!(f.producer.borrow().commits, 0);
    assert_eq!(f.host.s.presentation_sequence, 0);
    assert!(f.host.s.last_redraw_presentation.is_none());
    assert!(f.host.s.displayed_input.is_none());
    assert!(f.host.s.shared.producers.image_identity(7).is_none());
}
fn collect_on_wakes(f: &mut Fixture) {
    let deadline = Instant::now() + Duration::from_secs(6);
    while f.host.s.pending_native.is_some() {
        assert!(
            Instant::now() < deadline,
            "native host did not collect its worker completion"
        );
        if f.host.process_wake() {
            f.host.redraw();
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
fn wait_lease_release(core: &genet_render_host::RenderCore) {
    let deadline = Instant::now() + Duration::from_secs(6);
    while core.native_frame_busy() {
        assert!(
            Instant::now() < deadline,
            "cancelled native lease was retained"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn assert_once(out: &Outcomes) {
    let mut names: Vec<_> = out.borrow().iter().map(|(name, _)| *name).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["image", "paint", "stamped"]);
}

#[test]
#[ignore = "actual GPU; execute the serialized native Host lifecycle gate"]
fn redraw_keeps_facts_pending_then_unavailable_surface_refuses_bound_captures_once() {
    let mut f = fixture();
    let out = captures(&mut f.host);
    f.host.redraw();
    assert!(f.host.s.pending_native.is_some());
    assert!(f.core.native_frame_busy());
    assert!(out.borrow().is_empty());
    assert_unpublished(&f);
    assert_eq!(f.prepared.get(), 1);
    assert_eq!(f.producer.borrow().attempts, 1);
    f.model.set(9);
    let later = Rc::new(Cell::new(0));
    let next = later.clone();
    f.host.s.pending_capture = Some(Box::new(move |_| next.set(next.get() + 1)));
    collect_on_wakes(&mut f);
    assert!(f.wakes.load(Ordering::SeqCst) > 0);
    assert_eq!(
        f.application_wakes.get(),
        0,
        "native completion is not an application wake"
    );
    assert_eq!(f.acquisitions.get(), 1);
    assert_unpublished(&f);
    assert_eq!(f.prepared.get(), 1);
    assert_eq!(f.producer.borrow().discards, 1);
    assert!(f.producer.borrow().candidate.is_none());
    assert_eq!(later.get(), 0);
    assert!(f.host.s.pending_capture.is_some());
    assert_once(&out);
    assert!(
        out.borrow()
            .iter()
            .all(|(_, r)| matches!(r, CaptureRefusal::SurfaceUnavailable { .. }))
    );
    wait_lease_release(&f.core);
    assert_eq!(f.core.try_reuse_native_raster(|| 17).unwrap(), 17);
    f.host.cancel_native(CaptureRefusal::HostDisposed);
    assert_once(&out);
    f.host.refuse_pending_captures(CaptureRefusal::HostDisposed);
    assert_eq!(later.get(), 1);
    f.host.wake.wake();
    assert!(f.host.process_wake());
    assert_eq!(
        f.application_wakes.get(),
        1,
        "an actual application wake still reaches its hook"
    );
}

#[test]
#[ignore = "actual GPU; execute the serialized native Host lifecycle gate"]
fn resize_and_hidden_cancel_before_surface_acquisition_or_publication() {
    for hidden in [false, true] {
        let mut f = fixture();
        let out = captures(&mut f.host);
        f.host.redraw();
        assert!(f.host.s.pending_native.is_some());
        if hidden {
            f.host.set_hidden(true);
        } else {
            f.size.set((20, 16));
        }
        // This turn is the UI resize/visibility event, independent of GPU readiness.
        f.host.redraw();
        assert!(f.host.s.pending_native.is_none());
        assert_unpublished(&f);
        assert_eq!(f.acquisitions.get(), 0);
        assert_once(&out);
        assert!(f.producer.borrow().candidate.is_none());
        wait_lease_release(&f.core);
        assert_eq!(f.core.try_reuse_native_raster(|| 18).unwrap(), 18);
        assert_eq!(f.drops.load(Ordering::SeqCst), 0);
    }
}

#[test]
#[ignore = "actual device destruction; execute the serialized native Host lifecycle gate"]
fn actual_loss_cancels_pending_pixels_and_releases_host_device_resources() {
    let mut f = fixture();
    let out = captures(&mut f.host);
    f.host.redraw();
    assert!(f.host.s.pending_native.is_some());
    let core = f.core.clone();
    let loss = std::thread::spawn(move || {
        core.device().destroy();
        let deadline = Instant::now() + Duration::from_secs(5);
        while core.device_failure().is_none() {
            assert!(
                Instant::now() < deadline,
                "actual destruction callback was not delivered"
            );
            let _ = core.device().poll(wgpu::PollType::Poll);
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    let deadline = Instant::now() + Duration::from_secs(6);
    // Hardware delivery is driven by the test worker, never by a UI GPU wait.
    while f.core.device_failure().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    f.host.process_wake();
    f.host.redraw();
    while f.host.s.surface.is_some() || f.host.s.shared.render_core.is_some() {
        assert!(
            Instant::now() < deadline,
            "host retained old-device resources after cancellation"
        );
        if f.host.process_wake() {
            f.host.redraw();
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    while !loss.is_finished() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    loss.join().unwrap();
    assert!(f.host.s.pending_native.is_none());
    assert_unpublished(&f);
    assert_once(&out);
    assert_eq!(f.drops.load(Ordering::SeqCst), 1);
    assert!(f.producer.borrow().candidate.is_none());
    wait_lease_release(&f.core);
    assert_eq!(
        f.core.device_failure().unwrap().reason,
        wgpu::DeviceLostReason::Destroyed
    );
    assert!(
        f.core.loss_notice().is_some(),
        "no cancellation admission may hold recovery closed"
    );
}

#[test]
#[ignore = "actual shader validation; execute the serialized native Host lifecycle gate"]
fn actual_producer_shader_refusal_never_publishes_and_next_valid_batch_reaches_acquisition() {
    let mut f = fixture();
    f.producer.borrow_mut().invalid_shader = true;
    let rejected = captures(&mut f.host);
    f.host.redraw();
    assert!(f.host.s.pending_native.is_some());
    assert!(rejected.borrow().is_empty());
    assert_unpublished(&f);
    collect_on_wakes(&mut f);
    assert_once(&rejected);
    assert!(
        rejected
            .borrow()
            .iter()
            .all(|(_, refusal)| matches!(refusal,
        CaptureRefusal::ProviderError {error,..} if matches!(&**error,
            netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Validation {..}))))
    );
    assert_unpublished(&f);
    assert_eq!(f.acquisitions.get(), 0);
    assert_eq!(f.producer.borrow().discards, 1);
    assert!(f.producer.borrow().candidate.is_none());
    assert!(f.core.device_failure().is_none());
    assert!(!f.host.s.needs_recovery_surface);
    let refused = f
        .host
        .frame_profile()
        .expect("refused work remains measured");
    assert!(!refused.native_pending);
    assert_eq!(refused.native_submit_us, 0);
    assert_eq!(refused.total_us, refused.native_collect_us);
    wait_lease_release(&f.core);
    assert_eq!(f.core.try_reuse_native_raster(|| 19).unwrap(), 19);

    f.producer.borrow_mut().invalid_shader = false;
    let valid = captures(&mut f.host);
    f.host.wake.wake();
    assert!(f.host.process_wake());
    let application_wakes = f.application_wakes.get();
    f.host.redraw();
    assert!(f.host.s.pending_native.is_some());
    collect_on_wakes(&mut f);
    assert_eq!(
        f.acquisitions.get(),
        1,
        "the healthy batch must reach the surface"
    );
    assert_once(&valid);
    assert!(
        valid
            .borrow()
            .iter()
            .all(|(_, refusal)| matches!(refusal, CaptureRefusal::SurfaceUnavailable { .. }))
    );
    assert_eq!(f.application_wakes.get(), application_wakes);
    assert_unpublished(&f);
    assert_eq!(f.producer.borrow().attempts, 2);
    assert_eq!(f.producer.borrow().discards, 2);
}
