// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real Rootstock hosts and application-owned reader producers across one
//! shared recovery owner. Distinct leaf keys respect the renderer-global image
//! namespace. The ignored gate proves actual Device::destroy notification and
//! producer-image rehydration; its headless targets do not prove presentation.

#![cfg(all(feature = "native-host", not(target_arch = "wasm32")))]

use cambium::{AnyView, GenetCtx, GenetElement, custom_leaf};
use cambium_genet_winit_host::SceneProducer;
use cambium_rootstock::{
    AppCtx, CaptureRefusal, Host, HostHooks, HostOptions, HostState, HostWake, HostWindow,
    ProducerRole, ProducerSemantics, Runner, ScriptedDom, Surface,
};
use genet_render_host::{
    DeviceLossNotice, PendingRgbaReadback, ReadbackFailure, RecoveryStatus, RenderCore, RgbaFrame,
    SharedRenderCore,
};
use netrender::{ColorLoad, NO_CLIP, NetrenderOptions, Scene};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tabard::theme::registry::Mode;
use tabard_workshop::{ReaderSpecimen, WorkshopState};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 120;
const FIRST_KEY: u64 = 701;
const SECOND_KEY: u64 = 702;

struct WindowState {
    workshop: WorkshopState,
    leaf_key: u64,
}
type Child = Box<dyn AnyView<WindowState, (), GenetCtx, GenetElement>>;
type Logic = Box<dyn FnMut(&WindowState) -> Child>;
type WindowHost = Host<WindowState, Logic, Child>;

struct HeadlessWindow;
impl HostWindow for HeadlessWindow {
    fn request_redraw(&self) {}
    fn inner_size(&self) -> (u32, u32) {
        (WIDTH, HEIGHT)
    }
    fn scale_factor(&self) -> f64 {
        1.0
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}

struct HeadlessSurface {
    core: Arc<RenderCore>,
    drops: Arc<AtomicUsize>,
}
impl Surface for HeadlessSurface {
    fn core(&self) -> &RenderCore {
        &self.core
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        None
    }
}
impl Drop for HeadlessSurface {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

fn host(key: u64, primary_hex: &str, wakes: Arc<AtomicUsize>) -> WindowHost {
    let mut workshop = WorkshopState::in_memory();
    let mut authored = workshop.draft_theme().clone();
    authored.name = format!("Shared recovery authored source {key}");
    authored.seeds.primary.a = 73;
    // The first 120px of the real article contains neutral paper/text, before
    // any accent-bearing links. Distinct primary seeds alone do not make this
    // visible band distinct. Select real application modes so its background
    // and foreground are observably different without altering reader content.
    let mode = if key == FIRST_KEY {
        Mode::Light
    } else {
        Mode::Dark
    };
    workshop.edit_definition(&authored, Some(mode)).unwrap();
    *workshop.text_field_mut("seed-hex").unwrap() = cambium::TextInput::new(primary_hex);
    assert!(workshop.apply_hex());
    let mut state = HostState::new();
    state.window = Some(Box::new(HeadlessWindow));
    state.shared.sheet =
        format!("custom-leaf {{ display:block;width:{WIDTH}px;height:{HEIGHT}px; }}");
    let logic: Logic = Box::new(|state| {
        Box::new(custom_leaf::<WindowState, ()>(
            state.leaf_key,
            WIDTH,
            HEIGHT,
        )) as Child
    });
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        logic,
        WindowState {
            workshop,
            leaf_key: key,
        },
    ));
    let mut retained: Option<Rc<RefCell<SceneProducer<ReaderSpecimen>>>> = None;
    let hooks = HostHooks {
        frame: Box::new(move |ctx: &mut AppCtx<'_, WindowState, Logic, Child>| {
            let source = ctx.runner.state().workshop.reader_preview();
            let producer = retained.get_or_insert_with(|| {
                Rc::new(RefCell::new(SceneProducer::new(
                    source.clone(),
                    0x7265_636f_7665_0000 + key,
                    ReaderSpecimen::frame,
                    ReaderSpecimen::revision,
                    |reader| {
                        Some(ProducerSemantics {
                            role: Some(ProducerRole::Image),
                            name: Some(reader.accessible_name().into()),
                            children: vec![],
                        })
                    },
                )))
            });
            producer.borrow_mut().set_source(source);
            if !ctx.producers.contains(key) {
                ctx.producers.register(key, producer.clone(), &[]).unwrap();
            }
            false
        }),
        ..HostHooks::inert()
    };
    let wake = HostWake::new(
        state.wake_pending.clone(),
        Arc::new(move || {
            wakes.fetch_add(1, Ordering::SeqCst);
        }),
    );
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    host.set_surface_size(WIDTH as f32, HEIGHT as f32);
    host.prepare_frame();
    host.relayout(WIDTH as f32, HEIGHT as f32);
    host
}

fn authority(host: &WindowHost) -> (String, String, bool, Rc<RefCell<ReaderSpecimen>>) {
    let state = &host.s.runner.as_ref().unwrap().state().workshop;
    (
        tabard::portable::theme_json(state.draft_theme()).unwrap(),
        state.mode_key(),
        state.is_dirty(),
        state.reader_preview(),
    )
}

fn assert_authority(
    host: &WindowHost,
    before: &(String, String, bool, Rc<RefCell<ReaderSpecimen>>),
) {
    let after = authority(host);
    assert_eq!(
        (&after.0, &after.1, after.2),
        (&before.0, &before.1, before.2)
    );
    assert!(
        Rc::ptr_eq(&after.3, &before.3),
        "device recovery must not replace the application source"
    );
    assert!(
        host.s
            .shared
            .producers
            .contains(host.s.runner.as_ref().unwrap().state().leaf_key)
    );
}

fn options() -> NetrenderOptions {
    NetrenderOptions {
        enable_vello: true,
        tile_cache_size: Some(32),
        ..Default::default()
    }
}

fn install_surface(host: &mut WindowHost, core: Arc<RenderCore>, drops: Arc<AtomicUsize>) {
    host.s.shared.render_core = Some(core.clone());
    host.s.surface = Some(Box::new(HeadlessSurface { core, drops }));
}

fn producer_readback(host: &mut WindowHost, key: u64) -> PendingRgbaReadback {
    // Actual Rootstock redraw stages the bound production SceneProducer output.
    // It has no swapchain to acquire. Read that registered external image into
    // an owned target, rather than constructing another preview/document.
    host.redraw();
    let core = host.s.shared.render_core.as_ref().unwrap();
    let mut scene = Scene::new(WIDTH, HEIGHT);
    scene.push_image_full(
        0.0,
        0.0,
        WIDTH as f32,
        HEIGHT as f32,
        [0.0, 0.0, 1.0, 1.0],
        [1.0; 4],
        host.s
            .shared
            .producers
            .image_identity(key)
            .unwrap()
            .renderer_key,
        0,
        NO_CLIP,
    );
    let (texture, _) = core.rasterize_scaled_for(
        0x7265_636f_7078_0000 + key,
        &scene,
        WIDTH,
        HEIGHT,
        ColorLoad::Clear(wgpu::Color::TRANSPARENT),
        1.0,
    );
    // The queued copy owns its cancellable staging state. No local texture or
    // view survives this helper; the loss monitor cancels the staging handles.
    core.start_rgba8_readback(&texture, WIDTH, HEIGHT).unwrap()
}

fn producer_pixels(host: &mut WindowHost, key: u64) -> RgbaFrame {
    let mut readback = producer_readback(host, key);
    let start = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "replacement producer pixel readback exceeded deadline"
        );
        if let Some(result) = readback.poll() {
            let frame = result.unwrap();
            assert!(!frame.is_blank());
            let colors: std::collections::BTreeSet<_> = frame.rgba.chunks_exact(4).collect();
            assert!(
                colors.len() > 12,
                "actual production reader foreground must be present"
            );
            return frame;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn arm_refusal_checks(host: &mut WindowHost, notices: Rc<RefCell<Vec<(&'static str, u64)>>>) {
    let image = notices.clone();
    host.s.pending_capture = Some(Box::new(move |result| match result {
        Err(CaptureRefusal::DeviceLost(loss)) => {
            image.borrow_mut().push(("image", loss.generation))
        },
        _ => panic!("pending image capture must receive typed device-loss refusal"),
    }));
    let stamped = notices.clone();
    host.s.pending_stamped_capture = Some(Box::new(move |result| match result {
        Err(CaptureRefusal::DeviceLost(loss)) => {
            stamped.borrow_mut().push(("stamped", loss.generation))
        },
        _ => panic!("pending stamped capture must receive typed device-loss refusal"),
    }));
    host.s.pending_paint_capture = Some(Box::new(move |result| match result {
        Err(CaptureRefusal::DeviceLost(loss)) => {
            notices.borrow_mut().push(("paint", loss.generation))
        },
        _ => panic!("pending paint capture must receive typed device-loss refusal"),
    }));
}

fn write_frame_diagnostics(label: &str, frames: &[(&str, &RgbaFrame)]) {
    let rows: Vec<_> = frames.iter().map(|(name, frame)| serde_json::json!({
        "name":name,"width":frame.width,"height":frame.height,
        "byte_count":frame.rgba.len(),"fnv1a_digest":format!("{:016x}",frame.digest()),
        "distinct_colors":frame.rgba.chunks_exact(4).collect::<std::collections::BTreeSet<_>>().len(),
    })).collect();
    eprintln!(
        "recovery pixel diagnostic {label}: {}",
        serde_json::to_string(&rows).unwrap()
    );
    if let Some(directory) = std::env::var_os("SHARED_DEVICE_RECOVERY_OUT") {
        let directory = std::path::PathBuf::from(directory).join(label);
        std::fs::create_dir_all(&directory).unwrap();
        for (name, frame) in frames {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(format!("{name}.rgba")))
                .unwrap();
            std::io::Write::write_all(&mut file, &frame.rgba).unwrap();
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("diagnostics.json"))
            .unwrap();
        std::io::Write::write_all(&mut file, &serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}

#[test]
fn real_two_host_detachment_bounds_failed_factory_and_preserves_workshop_sources() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let count = attempts.clone();
    let loss = DeviceLossNotice {
        generation: 41,
        reason: wgpu::DeviceLostReason::Unknown,
        message: "CPU-only detached-loss qualification; no device destruction claimed".into(),
    };
    let owner = SharedRenderCore::from_detached_loss(
        loss.clone(),
        Arc::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
            Err("qualification replacement intentionally unavailable".into())
        }),
    );
    let wakes = Arc::new(AtomicUsize::new(0));
    let mut first = host(FIRST_KEY, "#2255AA", wakes.clone());
    let mut second = host(SECOND_KEY, "#33AA66", wakes);
    let first_state = authority(&first);
    let second_state = authority(&second);
    let first_capture = first.capture_portable().unwrap();
    let second_capture = second.capture_portable().unwrap();
    first.attach_shared_recovery(owner.clone());
    second.attach_shared_recovery(owner.clone());
    assert!(matches!(
        owner.status(),
        RecoveryStatus::Waiting {
            remaining_hosts: 2,
            ..
        }
    ));
    let refusals = Rc::new(RefCell::new(Vec::new()));
    arm_refusal_checks(&mut first, refusals.clone());
    assert!(matches!(
        first.poll_device_recovery(),
        RecoveryStatus::Waiting {
            remaining_hosts: 1,
            ..
        }
    ));
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        0,
        "one host must not replace a still-attached shared core"
    );
    assert!(matches!(
        second.poll_device_recovery(),
        RecoveryStatus::Failed { .. }
    ));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    for _ in 0..8 {
        assert!(matches!(
            first.poll_device_recovery(),
            RecoveryStatus::Failed { .. }
        ));
        assert!(matches!(
            second.poll_device_recovery(),
            RecoveryStatus::Failed { .. }
        ));
    }
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "failed factory is sticky rather than a redraw reboot loop"
    );
    assert_eq!(
        *refusals.borrow(),
        [("image", 41), ("stamped", 41), ("paint", 41)]
    );
    assert!(first.s.shared.render_core.is_none() && second.s.shared.render_core.is_none());
    assert_authority(&first, &first_state);
    assert_authority(&second, &second_state);
    let after_first = first.capture_portable().unwrap();
    let after_second = second.capture_portable().unwrap();
    for (before, after) in [
        (&first_capture, &after_first),
        (&second_capture, &after_second),
    ] {
        assert_eq!(before.producers.len(), 1);
        assert_eq!(after.producers.len(), 1);
        assert_eq!(
            before.producers[0].source_identity,
            after.producers[0].source_identity
        );
        assert_eq!(
            before.producers[0].source_revision,
            after.producers[0].source_revision
        );
        assert_eq!(
            before.producers[0].logical_size,
            after.producers[0].logical_size
        );
        assert_eq!(
            before.producers[0].physical_size,
            after.producers[0].physical_size
        );
    }
}

#[test]
#[ignore = "actual Device::destroy and replacement pixels; root serializes GPU and imposes finite process timeout"]
fn actual_shared_device_destroy_replaces_once_rehydrates_producers_then_bounds_later_failure() {
    let initial = Arc::new(RenderCore::boot(options()).expect("explicit GPU acceptance adapter"));
    let initial_generation = initial.generation();
    let adapter = initial.renderer().wgpu_device.core.adapter.get_info();
    let old_core = Arc::downgrade(&initial);
    let old_core_at_factory = old_core.clone();
    let attempts = Arc::new(AtomicUsize::new(0));
    let count = attempts.clone();
    // Install the recipe before either host's default-owner association.
    let owner = SharedRenderCore::with_factory(
        initial.clone(),
        Arc::new(move || {
            let attempt = count.fetch_add(1, Ordering::SeqCst) + 1;
            if attempt > 1 {
                return Err("qualification second replacement intentionally unavailable".into());
            }
            assert!(
                old_core_at_factory.upgrade().is_none(),
                "all owned old-core references must be released before replacement boot"
            );
            RenderCore::boot(options()).map(Arc::new)
        }),
    );
    assert!(
        Arc::ptr_eq(&initial.shared_recovery(), &owner),
        "core has one replacement owner"
    );
    let wakes = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut first = host(FIRST_KEY, "#2255AA", wakes.clone());
    let mut second = host(SECOND_KEY, "#33AA66", wakes.clone());
    install_surface(&mut first, initial.clone(), drops.clone());
    install_surface(&mut second, initial.clone(), drops.clone());
    first.attach_shared_recovery(owner.clone());
    second.attach_shared_recovery(owner.clone());
    let first_state = authority(&first);
    let second_state = authority(&second);
    let first_reader_bg = first_state.3.borrow().palette().bg.clone();
    let second_reader_bg = second_state.3.borrow().palette().bg.clone();
    assert_ne!(
        first_reader_bg, second_reader_bg,
        "application-owned mode must distinguish actual reader paper before GPU staging"
    );
    let before_first = producer_pixels(&mut first, FIRST_KEY);
    let before_second = producer_pixels(&mut second, SECOND_KEY);
    write_frame_diagnostics(
        "before-distinctness",
        &[("first", &before_first), ("second", &before_second)],
    );
    assert!(
        before_first.rgba != before_second.rgba,
        "distinct application sources must not alias one global image key; first={:016x} second={:016x}, reader paper first={} second={}",
        before_first.digest(),
        before_second.digest(),
        first_reader_bg,
        second_reader_bg
    );
    let refusals = Rc::new(RefCell::new(Vec::new()));
    arm_refusal_checks(&mut first, refusals.clone());
    let mut inflight = producer_readback(&mut first, FIRST_KEY);
    assert_eq!(inflight.generation(), initial_generation);
    let wakes_before_loss = wakes.load(Ordering::SeqCst);
    let actual_loss = actual_destroy_notice(&initial);
    match inflight.poll() {
        Some(Err(ReadbackFailure::DeviceLost(notice))) => assert_eq!(notice, actual_loss),
        _ => panic!(
            "in-flight copy must receive exactly the old device's typed loss, never late pixels"
        ),
    }
    assert!(matches!(
        inflight.poll(),
        Some(Err(ReadbackFailure::AlreadyCollected))
    ));
    assert!(
        wakes.load(Ordering::SeqCst) > wakes_before_loss,
        "real device callback wakes subscribed hosts"
    );
    assert!(matches!(
        first.poll_device_recovery(),
        RecoveryStatus::Waiting {
            remaining_hosts: 1,
            ..
        }
    ));
    assert!(first.s.surface.is_none());
    assert!(second.s.surface.is_some());
    assert_eq!(attempts.load(Ordering::SeqCst), 0);
    drop(initial);
    let next_generation = match second.poll_device_recovery() {
        RecoveryStatus::Ready { generation } => generation,
        other => panic!("one replacement after both real hosts detached: {other:?}"),
    };
    assert_ne!(next_generation, initial_generation);
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(old_core.upgrade().is_none());
    assert_eq!(
        drops.load(Ordering::SeqCst),
        2,
        "both old host surfaces dropped before replacement"
    );
    assert_eq!(
        first.poll_device_recovery(),
        RecoveryStatus::Ready {
            generation: next_generation
        }
    );
    let new_first = first.s.shared.render_core.as_ref().unwrap().clone();
    let new_second = second.s.shared.render_core.as_ref().unwrap().clone();
    assert!(
        Arc::ptr_eq(&new_first, &new_second),
        "both hosts adopt one replacement core"
    );
    assert!(
        !owner.notify_loss(actual_loss.clone()),
        "old generation callback cannot replace the new shared core"
    );
    assert_eq!(
        *refusals.borrow(),
        [
            ("image", initial_generation),
            ("stamped", initial_generation),
            ("paint", initial_generation)
        ]
    );
    install_surface(&mut first, new_first.clone(), drops.clone());
    install_surface(&mut second, new_second, drops.clone());
    let after_first = producer_pixels(&mut first, FIRST_KEY);
    let after_second = producer_pixels(&mut second, SECOND_KEY);
    write_frame_diagnostics(
        "after-rehydration",
        &[("first", &after_first), ("second", &after_second)],
    );
    assert!(
        matches!(
            inflight.poll(),
            Some(Err(ReadbackFailure::AlreadyCollected))
        ),
        "replacement cannot deliver old capture pixels"
    );
    drop(inflight);
    assert!(
        before_first.rgba == after_first.rgba,
        "first actual registered reader producer rehydrates identically; before={:016x} after={:016x}",
        before_first.digest(),
        after_first.digest()
    );
    assert!(
        before_second.rgba == after_second.rgba,
        "second actual registered reader producer rehydrates identically; before={:016x} after={:016x}",
        before_second.digest(),
        after_second.digest()
    );
    assert_authority(&first, &first_state);
    assert_authority(&second, &second_state);
    // A later actual loss takes the same two-host barrier but the factory
    // intentionally fails. Repeated redraw polls cannot trigger more boots.
    let later_loss = actual_destroy_notice(&new_first);
    assert_eq!(later_loss.generation, next_generation);
    assert!(matches!(
        first.poll_device_recovery(),
        RecoveryStatus::Waiting {
            remaining_hosts: 1,
            ..
        }
    ));
    let replacement_core = Arc::downgrade(&new_first);
    drop(new_first);
    assert!(matches!(
        second.poll_device_recovery(),
        RecoveryStatus::Failed { .. }
    ));
    for _ in 0..8 {
        first.poll_device_recovery();
        second.poll_device_recovery();
    }
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert!(replacement_core.upgrade().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 4);
    assert!(first.s.surface.is_none() && second.s.surface.is_none());
    assert_authority(&first, &first_state);
    assert_authority(&second, &second_state);
    if let Some(directory) = std::env::var_os("SHARED_DEVICE_RECOVERY_OUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let mut frames = Vec::new();
        for (name, frame) in [
            ("first-before", &before_first),
            ("second-before", &before_second),
            ("first-after", &after_first),
            ("second-after", &after_second),
        ] {
            let filename = format!("{name}.rgba");
            let path = directory.join(&filename);
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .expect("artifact output must be fresh; refusing to overwrite a previous receipt");
            std::io::Write::write_all(&mut file, &frame.rgba).unwrap();
            frames.push(serde_json::json!({"name":name,"file":filename,"width":frame.width,"height":frame.height,"byte_count":frame.rgba.len(),"fnv1a_digest":format!("{:016x}",frame.digest())}));
        }
        let receipt = serde_json::json!({
            "schema_version":1,
            "scope":"actual headless shared-device destroy/replacement and registered production-reader image rehydration; no swapchain presentation or physical driver reset",
            "namespace_scope":"distinct host-local leaf keys 701/702; does not qualify identical full-Workshop reserved-key namespaces",
            "adapter":{"name":adapter.name,"vendor":adapter.vendor,"device":adapter.device,"device_type":format!("{:?}",adapter.device_type),"backend":format!("{:?}",adapter.backend),"driver":adapter.driver,"driver_info":adapter.driver_info},
            "initial_generation":initial_generation,"replacement_generation":next_generation,
            "actual_losses":[{"generation":actual_loss.generation,"reason":format!("{:?}",actual_loss.reason),"message":actual_loss.message},{"generation":later_loss.generation,"reason":format!("{:?}",later_loss.reason),"message":later_loss.message}],
            "factory_attempts":attempts.load(Ordering::SeqCst),"surface_drops":drops.load(Ordering::SeqCst),
            "host_wakes":wakes.load(Ordering::SeqCst),"source_authority_preserved":true,
            "source_authority":{"first":{"mode":first_state.1,"dirty":first_state.2,"reader_rc_preserved":true},"second":{"mode":second_state.1,"dirty":second_state.2,"reader_rc_preserved":true}},
            "source_reader_paper":{"first":first_reader_bg,"second":second_reader_bg},
            "stale_notice_deduplicated":true,"inflight_old_generation_refused_once":true,"no_late_old_pixels":true,
            "all_owned_old_core_references_released_before_boot":true,
            "capture_refusals":&*refusals.borrow(),"pixel_equality":{"first":true,"second":true},"frames":frames,
            "second_replacement_failure_sticky":true
        });
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("receipt.json"))
            .unwrap();
        std::io::Write::write_all(&mut file, &serde_json::to_vec_pretty(&receipt).unwrap())
            .unwrap();
    }
}

fn actual_destroy_notice(core: &RenderCore) -> DeviceLossNotice {
    core.device().destroy();
    let start = Instant::now();
    loop {
        let _ = core.device().poll(wgpu::PollType::Poll);
        if let Some(notice) = core.loss_notice() {
            assert_eq!(notice.generation, core.generation());
            assert_eq!(notice.reason, wgpu::DeviceLostReason::Destroyed);
            return notice;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "actual device-destroy callback not observed"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
