// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Finite two-mount Workshop recovery and explicit mount cleanup. Current-frame
//! asynchronous WebGPU error scopes and shader-counter qualification remain open.
#![cfg(target_arch = "wasm32")]

use cambium_genet_web_host::{
    MountLifetime, MountPhase, MountStatus, Mounted, mount_with_core_future_and_lifetime,
    mount_with_render_core,
};
use cambium_rootstock::{
    AppCtx, CaptureRefusal, FocusedTextSlot, Frame, Host, HostFont, HostHooks, HostOptions,
    HostWindow, Init, KeyPress, NamedKey, PendingFrame, PresentedFrame, ProducerImageIdentity,
    start_frame_readback,
};
use genet_render_host::{ReadbackFailure, RecoveryStatus, RenderCore};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use netrender::{ColorLoad, NO_CLIP, Scene};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context as TaskContext, Poll, Waker},
};
use tabard::theme::registry::Mode;
use tabard_workshop::{
    READER_LEAF_KEY, STYLESHEET_LEAF_KEY, WorkshopState, WorkshopView,
    preview_bindings::PreviewBindings, workshop_stylesheet, workshop_view,
};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlCanvasElement;

#[cfg(feature = "current-frame-controls")]
mod quality_controls;

#[cfg(feature = "current-frame-controls")]
mod acquired_controls;

#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub async fn run_current_frame_quality(first_canvas:String, second_canvas:String, run:String) -> Result<JsValue,JsValue> {
    if USED.with(|u| u.replace(true)) { return Err(JsValue::from_str("fixture is one-shot")); }
    if run.is_empty() || run.len() > 128 { return Err(JsValue::from_str("bounded run identifier required")); }
    match quality_controls::execute(&first_canvas,&second_canvas,&run).await {
        Ok(receipt) => { STATUS.with(|s| *s.borrow_mut()=receipt.clone()); js_sys::JSON::parse(&receipt.to_string()) },
        Err(error) => {
            let cleanup = dispose_lifetimes().await;
            let receipt = json!({"status":"fail","run":run,"error":error,"cleanup":format!("{cleanup:?}"),
                "current_frame_async_quality_qualified":false,"current_frame_async_error_scopes_qualified":false});
            STATUS.with(|s| *s.borrow_mut()=receipt.clone()); Err(JsValue::from_str(&receipt.to_string()))
        },
    }
}

type Logic = Box<dyn FnMut(&WorkshopState) -> WorkshopView>;
type Application = Host<WorkshopState, Logic, WorkshopView>;
type BrowserMount = Mounted<WorkshopState, Logic, WorkshopView>;
type Context<'a> = AppCtx<'a, WorkshopState, Logic, WorkshopView>;
const BYTES_LIMIT: usize = 32 * 1024 * 1024;
const RETAINED_LIMIT: usize = 128 * 1024 * 1024;
const KEYS: [u64; 2] = [READER_LEAF_KEY, STYLESHEET_LEAF_KEY];

thread_local! {
    static USED: Cell<bool> = const { Cell::new(false) };
    static STATUS: RefCell<Value> = RefCell::new(json!({"status":"not_started"}));
    static PIXELS: RefCell<BTreeMap<String, Frame>> = RefCell::new(BTreeMap::new());
    static LIFETIMES: RefCell<Vec<MountLifetime>> = RefCell::new(Vec::new());
}

#[wasm_bindgen(start)]
pub fn initialize() {
    cambium_genet_web_host::run_static_constructors_once();
    console_error_panic_hook::set_once();
}
#[wasm_bindgen]
pub fn status() -> Result<JsValue, JsValue> {
    STATUS.with(|s| js_sys::JSON::parse(&s.borrow().to_string()))
}
#[wasm_bindgen]
pub fn capture_rgba(name: String) -> Result<js_sys::Uint8Array, JsValue> {
    PIXELS.with(|pixels| {
        pixels
            .borrow()
            .get(&name)
            .map(|frame| js_sys::Uint8Array::from(frame.rgba.as_slice()))
            .ok_or_else(|| JsValue::from_str("unknown capture artifact"))
    })
}
#[wasm_bindgen]
pub fn clear_capture_artifacts() {
    PIXELS.with(|p| p.borrow_mut().clear());
}
fn mount_status(s: MountStatus) -> Value {
    json!({"phase":format!("{:?}",s.phase),"startup_pending":s.startup_pending,
        "cleanup_pending":s.cleanup_pending,"listeners":s.listeners,"raf_pending":s.raf_pending,
        "listener_removal_failures":s.listener_removal_failures})
}
#[wasm_bindgen]
pub fn lifetime_status() -> Result<JsValue, JsValue> {
    let states: Vec<_> = LIFETIMES.with(|l| {
        l.borrow()
            .iter()
            .map(|l| mount_status(l.status()))
            .collect()
    });
    js_sys::JSON::parse(&json!({"mounts":states}).to_string())
}
async fn dispose_lifetimes() -> Result<Value, String> {
    let lives = LIFETIMES.with(|l| l.borrow().clone());
    for life in &lives {
        life.dispose();
    }
    wait(|| {
        Ok(lives
            .iter()
            .all(|l| l.status().phase == MountPhase::Disposed))
    })
    .await?;
    for life in &lives {
        clean(life.status())?;
    }
    Ok(
        json!({"status":"disposed","mounts":lives.iter().map(|l|mount_status(l.status())).collect::<Vec<_>>(),
        "scope":"owned mount startup, listeners, RAF and host bindings; excludes physical GPU destruction, JS WebGPU promise cancellation and shared recovery-task disposal"}),
    )
}
fn clean(s: MountStatus) -> Result<(), String> {
    if s.phase != MountPhase::Disposed
        || s.startup_pending
        || s.cleanup_pending
        || s.listeners != 0
        || s.raf_pending
        || s.listener_removal_failures != 0
    {
        return Err(format!(
            "incomplete owned mount cleanup: {}",
            mount_status(s)
        ));
    }
    Ok(())
}
#[wasm_bindgen]
pub async fn dispose() -> Result<JsValue, JsValue> {
    let v = dispose_lifetimes()
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    js_sys::JSON::parse(&v.to_string())
}
#[wasm_bindgen]
pub async fn run_shared_recovery(
    first_canvas: String,
    second_canvas: String,
    run: String,
) -> Result<JsValue, JsValue> {
    if USED.with(|u| u.replace(true)) {
        return Err(JsValue::from_str("fixture is one-shot"));
    }
    if run.is_empty() || run.len() > 128 {
        return Err(JsValue::from_str("bounded run identifier required"));
    }
    match execute(&first_canvas, &second_canvas, &run).await {
        Ok(receipt) => {
            STATUS.with(|s| *s.borrow_mut() = receipt.clone());
            js_sys::JSON::parse(&receipt.to_string())
        },
        Err(error) => {
            let cleanup = dispose_lifetimes().await;
            let receipt = json!({"status":"fail","run":run,"error":error,"cleanup":format!("{cleanup:?}"),
                "current_frame_async_quality_qualified":false,"current_frame_async_error_scopes_qualified":false});
            STATUS.with(|s| *s.borrow_mut() = receipt.clone());
            Err(JsValue::from_str(&receipt.to_string()))
        },
    }
}
fn phase(run: &str, name: &str) {
    STATUS.with(|s| *s.borrow_mut() = json!({"status":"running","run":run,"phase":name}));
}
fn now() -> Result<f64, String> {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .ok_or("monotonic browser clock unavailable".into())
}
async fn turn() -> Result<(), String> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        if let Err(e) = web_sys::window()
            .ok_or(JsValue::NULL)
            .and_then(|w| w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 20))
        {
            let _ = reject.call1(&JsValue::NULL, &e);
        }
    });
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|e| format!("timer: {e:?}"))
}
async fn wait(mut ready: impl FnMut() -> Result<bool, String>) -> Result<(), String> {
    let deadline = now()? + 5000.;
    loop {
        if now()? >= deadline {
            return Err("five-second fixture operation deadline exceeded".into());
        }
        let done = ready()?;
        if now()? >= deadline {
            return Err("fixture completion arrived after deadline".into());
        }
        if done {
            return Ok(());
        }
        turn().await?;
    }
}
fn canvas(id: &str) -> Result<HtmlCanvasElement, String> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
        .ok_or_else(|| format!("missing canvas {id}"))?
        .dyn_into()
        .map_err(|_| "element is not canvas".into())
}
fn size(canvas: &HtmlCanvasElement, w: u32, h: u32) -> Result<(), String> {
    canvas
        .style()
        .set_property("width", &format!("{w}px"))
        .map_err(|e| format!("canvas CSS: {e:?}"))?;
    canvas
        .style()
        .set_property("height", &format!("{h}px"))
        .map_err(|e| format!("canvas CSS: {e:?}"))?;
    Ok(())
}
fn frame_bytes(w: u32, h: u32) -> Result<usize, String> {
    (w as usize)
        .checked_mul(h as usize)
        .and_then(|p| p.checked_mul(4))
        .filter(|b| *b > 0 && *b <= BYTES_LIMIT)
        .ok_or("copied image exceeds nonempty 32 MiB bound".into())
}
fn valid(frame: &Frame) -> Result<(), String> {
    if frame.rgba.len() != frame_bytes(frame.width, frame.height)? || frame.is_blank() {
        return Err("invalid or blank actual RGBA image".into());
    }
    Ok(())
}
fn generic_fonts() -> Vec<(String, Vec<u8>)> {
    // Fixture-owned explicit bindings, never a global/system fallback. The
    // bundled face is intentionally selected for all four requested roles;
    // this gate does not claim serif or fixed-width typography fidelity.
    let bytes = include_bytes!("../../../examples/genet_web_smoke/assets/Roboto-Regular.ttf");
    ["sans-serif", "system-ui", "serif", "monospace"]
        .into_iter()
        .map(|name| (name.into(), bytes.to_vec()))
        .collect()
}
fn state(name: &str, mode: Mode) -> Result<WorkshopState, String> {
    let mut state = WorkshopState::in_memory();
    let mut theme = state.draft_theme().clone();
    theme.name = name.into();
    theme.seeds.primary.r = 94;
    state.edit_definition(&theme, Some(mode))?;
    state
        .reader_preview()
        .borrow_mut()
        .set_generic_fonts(generic_fonts())
        .map_err(|e| format!("reader font ledger: {e:?}"))?;
    state
        .stylesheet_preview()
        .borrow_mut()
        .set_generic_fonts(generic_fonts())
        .map_err(|e| format!("stylesheet font ledger: {e:?}"))?;
    Ok(state)
}
fn hooks() -> HostHooks<WorkshopState, Logic, WorkshopView> {
    let mut previews = PreviewBindings::default();
    HostHooks {
        frame: Box::new(move |ctx| {
            previews.register(ctx.runner.state(), ctx.leaves, ctx.producers);
            false
        }),
        after_dispatch: Box::new(|ctx| ctx.runner.update(WorkshopState::sync_controls)),
        focused_text: Box::new(|runner| {
            let node = runner.focus()?;
            let dom = runner.dom();
            let field = dom
                .borrow()
                .attribute(node, &Namespace::from(""), &LocalName::from("data-field"))?
                .to_string();
            runner.state().text_field(&field)?;
            let mutable = field.clone();
            Some(FocusedTextSlot {
                node,
                get: Box::new(move |s| s.text_field(&field).expect("actual focused field")),
                get_mut: Box::new(move |s| {
                    s.text_field_mut(&mutable).expect("actual focused field")
                }),
            })
        }),
        ..HostHooks::inert()
    }
}
fn focus_attribute(host: &mut Application, attribute: &str, value: &str) -> Result<(), String> {
    for _ in 0..150 {
        let runner = host.s.runner.as_ref().ok_or("runner unavailable")?;
        if let Some(node) = runner.focus() {
            if runner
                .dom()
                .borrow()
                .attribute(node, &Namespace::from(""), &LocalName::from(attribute))
                .is_some_and(|v| v == value)
            {
                return Ok(());
            }
        }
        let tab = KeyPress::named(NamedKey::Tab);
        host.key(&tab);
        host.key_released(&tab.key);
    }
    Err(format!(
        "production focus target {attribute}={value} not reached"
    ))
}
/// Synthetic DOM adapter control: actual owned canvas/mirror listeners receive
/// keyup, with no claim to physical OS keyboard or assistive-technology input.
fn tab_release_controls(
    mount: &Mounted<WorkshopState, Logic, WorkshopView>,
) -> Result<Value, String> {
    let canvas = mount.window().canvas();
    // The mirror belongs to this canvas: never use a page-global first match
    // when two independent hosts have otherwise identical mirror attributes.
    let mut sibling = canvas.next_sibling();
    let mut mirror = None;
    while let Some(node) = sibling {
        if node.dyn_ref::<HtmlCanvasElement>().is_some() {
            break;
        }
        if let Some(element) = node.dyn_ref::<web_sys::Element>() {
            if element.has_attribute("data-cambium-mirror") {
                mirror = Some(element.clone());
                break;
            }
        }
        sibling = node.next_sibling();
    }
    let mirror = mirror.ok_or("own canvas accessibility mirror unavailable")?;
    let targets: [(&str, &web_sys::EventTarget); 2] =
        [("canvas", canvas.as_ref()), ("mirror", mirror.as_ref())];
    let constructor = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("KeyboardEvent"))
        .map_err(|e| format!("KeyboardEvent constructor: {e:?}"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "KeyboardEvent constructor unavailable")?;
    let mut controls = Vec::new();
    for (name, target) in targets {
        let (prior_focus, authority) = {
            let mut host = mount.host().borrow_mut();
            let prior_focus = host.s.runner.as_ref().ok_or("runner")?.focus();
            let authority = host
                .s
                .runner
                .as_ref()
                .ok_or("runner")?
                .state()
                .draft_theme()
                .clone();
            host.key(&KeyPress::named(NamedKey::Tab));
            if !host.s.tab_held {
                return Err("real Tab press failed to engage held navigation".into());
            }
            (prior_focus, authority)
        };
        let pressed_focus = mount
            .host()
            .borrow()
            .s
            .runner
            .as_ref()
            .ok_or("runner")?
            .focus();
        let init = js_sys::Object::new();
        for (key, value) in [
            ("key", JsValue::from_str("Tab")),
            ("bubbles", JsValue::TRUE),
            ("cancelable", JsValue::TRUE),
        ] {
            js_sys::Reflect::set(&init, &JsValue::from_str(key), &value)
                .map_err(|e| format!("KeyboardEvent init: {e:?}"))?;
        }
        let args = js_sys::Array::new();
        args.push(&JsValue::from_str("keyup"));
        args.push(&init);
        let event = js_sys::Reflect::construct(&constructor, &args)
            .map_err(|e| format!("KeyboardEvent construction: {e:?}"))?
            .dyn_into::<web_sys::KeyboardEvent>()
            .map_err(|_| "constructed keyup event has wrong type")?;
        if event.key() != "Tab" {
            return Err("synthetic keyup lost its actual Tab key".into());
        }
        target
            .dispatch_event(&event)
            .map_err(|e| format!("owned {name} keyup dispatch: {e:?}"))?;
        {
            let mut host = mount.host().borrow_mut();
            let runner = host.s.runner.as_ref().ok_or("runner")?;
            if host.s.tab_held
                || runner.focus() != pressed_focus
                || runner.state().draft_theme() != &authority
            {
                return Err(format!(
                    "owned {name} keyup failed to release Tab or changed focus/application authority"
                ));
            }
            // Return through real reverse traversal, without setting focus.
            let reverse =
                KeyPress::named(NamedKey::Tab).with_modifiers(cambium_rootstock::Modifiers {
                    shift: true,
                    ..cambium_rootstock::Modifiers::NONE
                });
            host.key(&reverse);
            host.key_released(&reverse.key);
            if host.s.runner.as_ref().ok_or("runner")?.focus() != prior_focus {
                return Err("synthetic keyup control failed to preserve original focus".into());
            }
        }
        controls.push(json!({"target":name,"event":"synthetic DOM keyup Tab","tab_release_verified":true,
            "focus_unchanged_by_release":true,"original_focus_restored_by_reverse_tab":true,"application_authority_preserved":true}));
    }
    Ok(
        json!({"scope":"synthetic DOM adapter control; not OS keyboard or OS assistive technology","controls":controls}),
    )
}

fn presentation(p: PresentedFrame) -> Value {
    json!({"host":p.host,"sequence":p.sequence,"width":p.width,"height":p.height,"layout_scale":p.layout_scale})
}
fn image_json(key: u64, image: ProducerImageIdentity) -> Value {
    json!({"local_key":key.to_string(),"renderer_key":image.renderer_key.to_string(),"generation":image.generation,"stamp":image.stamp,"size":image.size})
}
fn font_scene_receipt(scene: &Scene) -> Result<Value, String> {
    let glyphs: usize = scene
        .ops
        .iter()
        .filter_map(|op| match op {
            netrender::SceneOp::GlyphRun(run) => Some(run.glyphs.len()),
            _ => None,
        })
        .sum();
    let fonts: Vec<_> = scene.fonts.iter().filter(|font| !font.data.as_ref().is_empty())
        .map(|font| json!({"bytes":font.data.as_ref().len(),"sha256":digest_bytes(font.data.as_ref())})).collect();
    if glyphs == 0 || fonts.is_empty() {
        return Err("actual retained producer contains no shaped glyphs/font resources".into());
    }
    Ok(json!({"glyphs":glyphs,"fonts":fonts}))
}
fn seal(ctx: &Context<'_>, p: PresentedFrame) -> Result<Value, String> {
    let state = ctx.runner.state();
    if ctx.retained_font_count() == 0 || ctx.font_registration_error_count() != 0 {
        return Err("actual outer Workshop font registration/shaping missing or refused".into());
    }
    let mut images = Vec::new();
    for key in KEYS {
        if ctx.producers.error(key).is_some() {
            return Err("production producer refused".into());
        }
        images.push(image_json(
            key,
            ctx.producers
                .image_identity(key)
                .ok_or("both full Workshop previews must be staged")?,
        ));
    }
    let reader_size = ctx
        .producers
        .image_identity(READER_LEAF_KEY)
        .ok_or("reader font extent")?
        .size;
    let stylesheet_size = ctx
        .producers
        .image_identity(STYLESHEET_LEAF_KEY)
        .ok_or("stylesheet font extent")?
        .size;
    let reader_fonts = font_scene_receipt(
        &state
            .reader_preview()
            .borrow_mut()
            .frame(reader_size[0], reader_size[1]),
    )?;
    let stylesheet_fonts = font_scene_receipt(
        &state
            .stylesheet_preview()
            .borrow_mut()
            .frame(stylesheet_size[0], stylesheet_size[1]),
    )?;
    Ok(
        json!({"font_resources":{"outer_materialized_fonts":ctx.retained_font_count(),"outer_registration_errors":ctx.font_registration_error_count(),"reader":reader_fonts,"stylesheet":stylesheet_fonts},
        "authority":{"theme":state.draft_theme(),"mode":state.mode_key(),"dirty":state.is_dirty(),"status":state.status(),
        "focus":format!("{:?}",ctx.runner.focus()),"zoom":ctx.ui_zoom,"graph_selected":state.selected_graph_node(),
        "name_text":state.text_field("name").map(|f|f.text()),"name_selection":format!("{:?}",state.text_field("name").map(|f|f.caret_selection()))},
        "runner":format!("{:p}",ctx.runner),"reader_source":format!("{:p}",Rc::as_ptr(&state.reader_preview())),
        "stylesheet_source":format!("{:p}",Rc::as_ptr(&state.stylesheet_preview())),
        "generation":ctx.producers.image_identity(READER_LEAF_KEY).unwrap().generation,"presentation":presentation(p),"images":images}),
    )
}
struct Capture {
    frame: Frame,
    seal: Value,
}
async fn collect(mut pending: PendingFrame, started: f64) -> Result<Frame, String> {
    let mut frame = None;
    wait(|| {
        if now()? - started >= 5000. {
            return Err("copied readback exceeded five seconds from admission".into());
        }
        if let Some(result) = pending.poll() {
            if now()? - started >= 5000. {
                return Err("readback ready after admission deadline".into());
            }
            frame = Some(result.map_err(|e| format!("actual readback: {e}"))?);
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .await?;
    let frame = frame.ok_or("readback missing")?;
    valid(&frame)?;
    Ok(frame)
}
async fn capture(mount: &BrowserMount) -> Result<Capture, String> {
    let observed = Rc::new(RefCell::new(None));
    let copied = Rc::new(RefCell::new(None));
    {
        let mut host = mount.host().borrow_mut();
        let out = observed.clone();
        host.s.pending_presentation_observer = Some(Box::new(move |ctx, p| {
            *out.borrow_mut() = Some(seal(ctx, p))
        }));
        let out = copied.clone();
        host.s.pending_stamped_capture = Some(Box::new(move |result| {
            *out.borrow_mut() = Some(result.map_err(|e| e.to_string()).and_then(|c| {
                let p = c.presentation;
                frame_bytes(p.width, p.height)?;
                Ok((
                    p,
                    now()?,
                    start_frame_readback(c.surface, c.view, p.width, p.height)?,
                ))
            }));
        }));
    }
    mount.window().request_redraw();
    let queued = wait(|| Ok(copied.borrow().is_some())).await;
    if let Err(e) = queued {
        let mut h = mount.host().borrow_mut();
        h.s.pending_stamped_capture = None;
        h.s.pending_presentation_observer = None;
        return Err(e);
    }
    let (p, started, pending) = copied.borrow_mut().take().ok_or("missing capture")??;
    let seal = observed
        .borrow_mut()
        .take()
        .ok_or("capture lacks presentation authority seal")??;
    if seal["presentation"] != presentation(p) {
        return Err("capture does not match presentation seal".into());
    }
    let frame = collect(pending, started).await?;
    if frame.width != p.width || frame.height != p.height {
        return Err("readback extent mismatch".into());
    }
    Ok(Capture { frame, seal })
}
async fn loss_readback(mount: &BrowserMount) -> Result<PendingFrame, String> {
    let copied = Rc::new(RefCell::new(None));
    let out = copied.clone();
    mount.host().borrow_mut().s.pending_capture = Some(Box::new(move |result| {
        *out.borrow_mut() = Some(result.map_err(|e| e.to_string()).and_then(|c| {
            frame_bytes(c.width, c.height)?;
            start_frame_readback(c.surface, c.view, c.width, c.height)
        }));
    }));
    mount.window().request_redraw();
    if let Err(e) = wait(|| Ok(copied.borrow().is_some())).await {
        mount.host().borrow_mut().s.pending_capture = None;
        return Err(e);
    }
    copied.borrow_mut().take().ok_or("missing loss readback")?
}
async fn sources(mount: &BrowserMount, index: usize) -> Result<Vec<Capture>, String> {
    let mut captures = Vec::new();
    for (slot, key) in KEYS.into_iter().enumerate() {
        let (image, core) = {
            let h = mount.host().borrow();
            if h.s.shared.producers.error(key).is_some() {
                return Err("actual producer refused before complete sample".into());
            }
            (
                h.s.shared
                    .producers
                    .image_identity(key)
                    .ok_or("missing complete producer")?,
                h.s.shared
                    .render_core
                    .as_ref()
                    .ok_or("source sample lacks current core")?
                    .clone(),
            )
        };
        let [w, h] = image.size;
        frame_bytes(w, h)?;
        if image.generation != core.generation() {
            return Err("stale producer generation".into());
        }
        let mut scene = Scene::new(w, h);
        scene.push_image_full(
            0.,
            0.,
            w as f32,
            h as f32,
            [0., 0., 1., 1.],
            [1.; 4],
            image.renderer_key,
            0,
            NO_CLIP,
        );
        let (texture, view) = owned_raster(&core,
            0x6272_6f77_7365_7200 + (index * 2 + slot) as u64,&scene,w,h,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),1.).await?;
        let started = now()?;
        let pending = core.start_rgba8_readback(&texture, w, h)?;
        drop(view);
        drop(texture);
        drop(core);
        let frame = collect(pending, started).await?;
        if frame.width != w || frame.height != h {
            return Err("complete producer extent mismatch".into());
        }
        captures.push(Capture {
            frame,
            seal: image_json(key, image),
        });
    }
    Ok(captures)
}
// Source sampling participates in the same actual FIFO/scope owner as hosts;
// legacy synchronous Wasm raster cannot establish this attribution boundary.
async fn owned_raster(core:&Arc<RenderCore>, key:u64, scene:&Scene, width:u32,height:u32,
    clear:ColorLoad,scale:f32) -> Result<(wgpu::Texture,wgpu::TextureView),String> {
    let mut submission = None;
    wait(|| match core.try_begin_browser_frame() {
        Ok(frame) => { submission=Some(frame); Ok(true) },
        Err(genet_render_host::BrowserFrameStartError::Busy) => Ok(false),
        Err(error) => Err(format!("owned source admission: {error:?}")),
    }).await?;
    let mut submission=submission.ok_or("owned source admission absent")?;
    let output=submission.submit(|commands| commands.rasterize_scaled_for(key,scene,width,height,clear,scale));
    let mut pending=submission.seal().map_err(|e|format!("owned source seal: {e:?}"))?;
    let mut accepted=None;
    wait(|| if let Some(result)=pending.poll() { accepted=Some(result); Ok(true) } else { Ok(false) }).await?;
    let accepted=accepted.ok_or("owned source completion absent")?.map_err(|e|format!("owned source quality: {e:?}"))?;
    accepted.check_current().map_err(|e|format!("owned source acceptance stale: {e:?}"))?;
    drop(accepted);
    output.map_err(|e|format!("owned source raster refused: {e:?}"))
}

fn adapter_identity(core: &RenderCore) -> Value {
    let info = core.renderer().wgpu_device.core.adapter.get_info();
    json!({"name":info.name,"backend":format!("{:?}",info.backend),
        "device_type":format!("{:?}",info.device_type),"vendor":info.vendor,"device":info.device,
        "driver":info.driver,"driver_info":info.driver_info,
        "source":"actual fixture RenderCore adapter; browser fields may be redacted"})
}
fn digest(frame: &Frame) -> String {
    digest_bytes(&frame.rgba)
}
fn digest_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hexadecimal = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut hexadecimal, "{byte:02x}").expect("writing to String cannot fail");
    }
    hexadecimal
}
fn compare_pixels(before: &Frame, after: &Frame) -> Result<Value, String> {
    if before.width != after.width
        || before.height != after.height
        || before.rgba.len() != after.rgba.len()
    {
        return Err("comparison extent changed".into());
    }
    let mut max_rgb = 0;
    let mut max_alpha = 0;
    let mut sum = 0u64;
    let mut changed = 0;
    for (a, b) in before.rgba.chunks_exact(4).zip(after.rgba.chunks_exact(4)) {
        let mut different = false;
        for i in 0..4 {
            let d = a[i].abs_diff(b[i]);
            sum += u64::from(d);
            different |= d != 0;
            if i == 3 {
                max_alpha = max_alpha.max(d);
            } else {
                max_rgb = max_rgb.max(d);
            }
        }
        changed += usize::from(different);
    }
    let mean = sum as f64 / before.rgba.len() as f64;
    if max_rgb > 3 || max_alpha > 2 || mean > 1. {
        return Err(format!(
            "unchanged pixel limits exceeded: RGB{max_rgb}/alpha{max_alpha}/mean{mean}"
        ));
    }
    Ok(
        json!({"width":before.width,"height":before.height,"max_rgb":max_rgb,"max_alpha":max_alpha,"mean_absolute":mean,
        "changed_pixels":changed,"exact_rgba":before.rgba==after.rgba,"before_sha256":digest(before),"after_sha256":digest(after),
        "limits":{"rgb":3,"alpha":2,"mean_absolute":1},"scope":"uniform unchanged strict interior limits on entire copied image"}),
    )
}
fn compare_source(before: &Capture, after: &Capture) -> Result<Value, String> {
    for field in ["local_key", "renderer_key", "size"] {
        if before.seal[field] != after.seal[field] {
            return Err(format!("producer identity changed: {field}"));
        }
    }
    if before.seal["generation"] == after.seal["generation"]
        || after.seal["stamp"].as_u64() <= before.seal["stamp"].as_u64()
    {
        return Err("producer did not rehydrate in fresh generation/stamp".into());
    }
    Ok(
        json!({"before":before.seal,"after":after.seal,"pixels":compare_pixels(&before.frame,&after.frame)?,
        "scope":"full actual registered physical image; no crop, source reconstruction or matte"}),
    )
}
fn compare(before: &Capture, after: &Capture) -> Result<Value, String> {
    for field in ["authority", "runner", "reader_source", "stylesheet_source"] {
        if before.seal[field] != after.seal[field] {
            return Err(format!("application authority changed: {field}"));
        }
    }
    if before.seal["generation"] == after.seal["generation"]
        || before.seal["presentation"]["host"] != after.seal["presentation"]["host"]
        || after.seal["presentation"]["sequence"].as_u64()
            <= before.seal["presentation"]["sequence"].as_u64()
    {
        return Err("fresh generation/same-host presentation not observed".into());
    }
    for slot in 0..2 {
        for field in ["local_key", "renderer_key", "size"] {
            if before.seal["images"][slot][field] != after.seal["images"][slot][field] {
                return Err(format!("image changed: {field}"));
            }
        }
        if after.seal["images"][slot]["generation"] != after.seal["generation"]
            || after.seal["images"][slot]["stamp"].as_u64()
                <= before.seal["images"][slot]["stamp"].as_u64()
        {
            return Err("image rehydration stamp not advanced".into());
        }
    }
    Ok(
        json!({"before":before.seal,"after":after.seal,"pixels":compare_pixels(&before.frame,&after.frame)?}),
    )
}
fn control_init(
    count: Rc<Cell<usize>>,
) -> impl FnOnce(
    &dyn HostWindow,
    &cambium_rootstock::WindowCommands,
    &cambium_rootstock::HostWake,
) -> Init<WorkshopState, Logic> {
    move |_, _, _| {
        count.set(count.get() + 1);
        Init {
            state: WorkshopState::in_memory(),
            logic: Box::new(workshop_view) as Logic,
            sheet: workshop_stylesheet(),
            fonts: Vec::new(),
            images: Vec::new(),
        }
    }
}
struct HeldCandidate {
    core: Option<Arc<RenderCore>>,
    release: Rc<Cell<bool>>,
    waker: Rc<RefCell<Option<Waker>>>,
    life: MountLifetime,
    delivered: Rc<Cell<bool>>,
}
impl Future for HeldCandidate {
    type Output = Result<Arc<RenderCore>, String>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        if !self.release.get() {
            *self.waker.borrow_mut() = Some(cx.waker().clone());
            return Poll::Pending;
        }
        self.life.dispose();
        self.delivered.set(true);
        Poll::Ready(Ok(self.core.take().expect("candidate delivered once")))
    }
}
async fn cancellation_controls(canvas: HtmlCanvasElement) -> Result<Value, String> {
    let before = MountLifetime::new();
    before.dispose();
    let polled = Rc::new(Cell::new(0));
    let init = Rc::new(Cell::new(0));
    let count = polled.clone();
    let acquire = async move {
        count.set(count.get() + 1);
        RenderCore::boot_async((HostOptions::default().netrender)())
            .await
            .map(Arc::new)
    };
    if mount_with_core_future_and_lifetime(
        canvas.clone(),
        HostOptions::default(),
        before.clone(),
        acquire,
        control_init(init.clone()),
        hooks(),
    )
    .await
    .is_ok()
    {
        return Err("disposed startup activated".into());
    }
    wait(|| Ok(before.status().phase == MountPhase::Disposed)).await?;
    clean(before.status())?;
    if polled.get() != 0 || init.get() != 0 {
        return Err("disposed-before-acquisition executed startup".into());
    }
    let candidate = Arc::new(RenderCore::boot_async((HostOptions::default().netrender)()).await?);
    let generation = candidate.generation();
    let weak = Arc::downgrade(&candidate);
    // Caller observation authority only; generic caller-provided acquisition
    // correctly refuses activation without destroying a potentially shared core.
    let observer = candidate.clone();
    let late = MountLifetime::new();
    let release = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None));
    let delivered = Rc::new(Cell::new(false));
    let init_late = Rc::new(Cell::new(0));
    let held = HeldCandidate {
        core: Some(candidate),
        release: release.clone(),
        waker: waker.clone(),
        life: late.clone(),
        delivered: delivered.clone(),
    };
    let mut mount = mount_with_core_future_and_lifetime(
        canvas,
        HostOptions::default(),
        late.clone(),
        held,
        control_init(init_late.clone()),
        hooks(),
    );
    let pending =
        std::future::poll_fn(|cx| Poll::Ready(matches!(mount.as_mut().poll(cx), Poll::Pending)))
            .await;
    if !pending {
        return Err("held real acquisition did not remain pending".into());
    }
    release.set(true);
    if let Some(w) = waker.borrow_mut().take() {
        w.wake();
    }
    if mount.await.is_ok() || !delivered.get() || init_late.get() != 0 {
        return Err("late real candidate escaped revoked startup authority".into());
    }
    wait(|| Ok(late.status().phase == MountPhase::Disposed)).await?;
    clean(late.status())?;
    // This candidate was privately booted by the fixture, so only the fixture
    // has authority to close its device after cancellation completes.
    observer.device().destroy();
    let mut control_notice = None;
    wait(|| {
        let Some(notice) = observer.loss_notice() else {
            return Ok(false);
        };
        if notice.generation != generation || notice.reason != wgpu::DeviceLostReason::Destroyed {
            return Err(format!(
                "cancellation control closure returned unexpected loss: {notice:?}"
            ));
        }
        control_notice = Some(notice);
        Ok(true)
    })
    .await?;
    let notice =
        control_notice.ok_or("actual cancellation-control device closure notice absent")?;
    drop(observer);
    if weak.upgrade().is_some() {
        return Err("cancelled real candidate retained Rust core".into());
    }
    Ok(
        json!({"disposed_before_acquisition":{"acquisition_polls":polled.get(),"init_calls":init.get(),"status":mount_status(before.status())},
        "late_actual_candidate":{"generation":generation,"delivered_after_revocation":delivered.get(),"init_calls":init_late.get(),"rust_core_released":true,"status":mount_status(late.status()),
            "control_device_closure":{"actual_device_destroyed":true,"matching_loss_notice_observed":true,"generation":notice.generation,"reason":format!("{:?}",notice.reason),"observer_dropped":true}},
        "scope":"actual mount startup/lifetime seam plus explicit closure of the fixture-private control device; excludes physical driver reset and cancellation of underlying JS WebGPU promises"}),
    )
}

async fn execute(first_id: &str, second_id: &str, run: &str) -> Result<Value, String> {
    let started = now()?;
    let first_canvas = canvas(first_id)?;
    let second_canvas = canvas(second_id)?;
    if first_canvas == second_canvas {
        return Err("two independent canvases required".into());
    }
    size(&first_canvas, 1280, 1100)?;
    size(&second_canvas, 1280, 1100)?;
    let options = || HostOptions {
        title: "Shared Workshop recovery fixture".into(),
        ..HostOptions::default()
    };
    let cancellation = cancellation_controls(second_canvas.clone()).await?;
    phase(run, "boot_shared_core");
    let core = Arc::new(RenderCore::boot_async((options().netrender)()).await?);
    let generation = core.generation();
    let initial_adapter = adapter_identity(&core);
    let old = Arc::downgrade(&core);
    let owner = core.shared_recovery();
    let mut mounts = Vec::new();
    for (canvas, name, mode) in [
        (first_canvas, "Browser-first-draft", Mode::Light),
        (second_canvas, "Browser-second-draft", Mode::Dark),
    ] {
        let state = state(name, mode)?;
        let mount = mount_with_render_core(
            canvas,
            options(),
            core.clone(),
            move |_, _, _| Init {
                state,
                logic: Box::new(workshop_view) as Logic,
                sheet: workshop_stylesheet(),
                fonts: generic_fonts()
                    .into_iter()
                    .map(|(family, bytes)| HostFont {
                        family: Some(family),
                        bytes,
                    })
                    .collect(),
                images: Vec::new(),
            },
            hooks(),
        )
        .await?;
        LIFETIMES.with(|lifetimes| lifetimes.borrow_mut().push(mount.lifetime().clone()));
        {
            let mut host = mount.host().borrow_mut();
            focus_attribute(
                &mut host,
                "data-mode",
                if name.contains("first") {
                    "light"
                } else {
                    "dark"
                },
            )?;
        }
        mounts.push(mount);
    }
    phase(run, "baseline");
    let first_before = capture(&mounts[0]).await?;
    let second_before = capture(&mounts[1]).await?;
    phase(run, "complete_producer_baselines");
    // Account for all twelve retained copies before collecting the first full
    // producer baseline. Rehydration must preserve every accepted extent.
    let mut retained_bytes = frame_bytes(first_before.frame.width, first_before.frame.height)?
        .checked_add(frame_bytes(
            second_before.frame.width,
            second_before.frame.height,
        )?)
        .and_then(|n| n.checked_mul(2))
        .ok_or("retained byte arithmetic overflow")?;
    for mount in &mounts {
        let host = mount.host().borrow();
        for key in KEYS {
            let image = host
                .s
                .shared
                .producers
                .image_identity(key)
                .ok_or("missing producer for byte admission")?;
            let bytes = frame_bytes(image.size[0], image.size[1])?
                .checked_mul(2)
                .ok_or("source byte arithmetic overflow")?;
            retained_bytes = retained_bytes
                .checked_add(bytes)
                .ok_or("retained byte arithmetic overflow")?;
        }
    }
    if retained_bytes > RETAINED_LIMIT {
        return Err("complete before/after artifact plan exceeds 128 MiB retention bound".into());
    }
    let first_sources_before = sources(&mounts[0], 0).await?;
    let second_sources_before = sources(&mounts[1], 1).await?;
    for slot in 0..2 {
        if digest(&first_sources_before[slot].frame) == digest(&second_sources_before[slot].frame) {
            return Err("independently authored full producer images are indistinguishable".into());
        }
    }
    if (0..2).any(|index| {
        first_before.seal["images"][index]["renderer_key"]
            == second_before.seal["images"][index]["renderer_key"]
    }) {
        return Err("independent registries alias physical producer keys".into());
    }
    if digest(&first_before.frame) == digest(&second_before.frame) {
        return Err("independent authored Workshop captures are indistinguishable".into());
    }
    // The second mount is genuinely skipped by the browser raster scheduler.
    // Its real shared-owner wake must still detach/acknowledge without RAF.
    mounts[1].host().borrow_mut().set_hidden(true);
    let failures = Rc::new(RefCell::new(Vec::<CaptureRefusal>::new()));
    let unexpected_capture = Rc::new(Cell::new(false));
    {
        let mut host = mounts[1].host().borrow_mut();
        let sink = failures.clone();
        let unexpected = unexpected_capture.clone();
        host.s.pending_capture = Some(Box::new(move |result| match result {
            Err(error) => sink.borrow_mut().push(error),
            Ok(_) => unexpected.set(true),
        }));
        let sink = failures.clone();
        let unexpected = unexpected_capture.clone();
        host.s.pending_stamped_capture = Some(Box::new(move |result| match result {
            Err(error) => sink.borrow_mut().push(error),
            Ok(_) => unexpected.set(true),
        }));
        let sink = failures.clone();
        let unexpected = unexpected_capture.clone();
        host.s.pending_paint_capture = Some(Box::new(move |result| match result {
            Err(error) => sink.borrow_mut().push(error),
            Ok(_) => unexpected.set(true),
        }));
    }
    phase(run, "queued_loss_readback");
    let mut cancelled_readback = loss_readback(&mounts[0]).await?;
    phase(run, "owned_device_destroy");
    core.device().destroy();
    drop(core);
    wait(|| {
        if let RecoveryStatus::Failed { error, .. } = owner.status() {
            return Err(format!("replacement failed: {error}"));
        }
        Ok(mounts.iter().all(|mount| {
            let host = mount.host().borrow();
            host.s
                .shared
                .render_core
                .as_ref()
                .is_some_and(|core| core.generation() != generation)
                && host.s.surface.is_some()
                && !host.s.needs_recovery_surface
        }))
    })
    .await?;
    if unexpected_capture.get() {
        return Err("hidden loss capture unexpectedly succeeded".into());
    }
    let loss = failures
        .borrow()
        .first()
        .and_then(|error| match error {
            CaptureRefusal::DeviceLost(loss) => Some(loss.clone()),
            _ => None,
        })
        .ok_or("hidden host did not deliver actual loss refusal")?;
    if failures.borrow().len() != 3
        || failures
            .borrow()
            .iter()
            .any(|error| !matches!(error,CaptureRefusal::DeviceLost(notice) if notice == &loss))
        || loss.generation != generation
        || loss.reason != wgpu::DeviceLostReason::Destroyed
    {
        return Err("loss captures not exactly-once and generation-branded".into());
    }
    if !matches!(cancelled_readback.poll(), Some(Err(ReadbackFailure::DeviceLost(notice))) if notice == loss)
    {
        return Err(
            "queued browser readback did not refuse with actual old-generation loss".into(),
        );
    }
    if !matches!(
        cancelled_readback.poll(),
        Some(Err(ReadbackFailure::AlreadyCollected))
    ) {
        return Err("queued browser readback replayed its loss after collection".into());
    }
    drop(cancelled_readback);
    if old.upgrade().is_some() {
        return Err("old core retained after replacement".into());
    }
    let replacement = owner
        .current()
        .map_err(|status| format!("replacement unavailable: {status:?}"))?;
    for mount in &mounts {
        let host = mount.host().borrow();
        if !Arc::ptr_eq(
            host.s
                .shared
                .render_core
                .as_ref()
                .ok_or("missing new core")?,
            &replacement,
        ) {
            return Err("mounts adopted different replacement cores".into());
        }
    }
    if !mounts[1].host().borrow().s.hidden {
        return Err("recovery changed hidden-host authority".into());
    }
    mounts[1].host().borrow_mut().set_hidden(false);
    phase(run, "rehydrated_captures");
    let first_after = capture(&mounts[0]).await?;
    let second_after = capture(&mounts[1]).await?;
    let first = compare(&first_before, &first_after)?;
    let second = compare(&second_before, &second_after)?;
    phase(run, "complete_producer_rehydration");
    let first_sources_after = sources(&mounts[0], 0).await?;
    let second_sources_after = sources(&mounts[1], 1).await?;
    let full_producers: Result<Vec<_>, String> = (0..2)
        .map(|slot| {
            Ok(json!({"slot":slot,
        "first":compare_source(&first_sources_before[slot],&first_sources_after[slot])?,
        "second":compare_source(&second_sources_before[slot],&second_sources_after[slot])?}))
        })
        .collect();
    let full_producers = full_producers?;
    phase(run, "continued_input_resize");
    let tab_release = tab_release_controls(&mounts[0])?;
    let before_input = mounts[0]
        .host()
        .borrow()
        .s
        .runner
        .as_ref()
        .ok_or("runner")?
        .state()
        .draft_theme()
        .seeds
        .primary;
    {
        let mut host = mounts[0].host().borrow_mut();
        focus_attribute(&mut host, "aria-label", "Primary hue")?;
        host.key(&KeyPress::named(NamedKey::ArrowRight));
    }
    let after_input = capture(&mounts[0]).await?;
    let (after_primary, tab_held, focused) = {
        let host = mounts[0].host().borrow();
        let runner = host.s.runner.as_ref().ok_or("runner")?;
        let focus = runner.focus();
        let dom = runner.dom();
        let dom = dom.borrow();
        let attr = |name: &str| {
            focus
                .and_then(|node| dom.attribute(node, &Namespace::from(""), &LocalName::from(name)))
                .map(|value| value.chars().take(128).collect::<String>())
        };
        (
            runner.state().draft_theme().seeds.primary,
            host.s.tab_held,
            json!({"node":format!("{focus:?}"),"role":attr("role"),"label":attr("aria-label")}),
        )
    };
    let before_pixel_hash = digest(&first_after.frame);
    let after_pixel_hash = digest(&after_input.frame);
    let input_diagnostic = json!({"tab_held":tab_held,"focus":focused,
        "before_primary":before_input,"after_primary":after_primary,
        "before_full_pixel_sha256":before_pixel_hash,"after_full_pixel_sha256":after_pixel_hash});
    if after_primary == before_input || after_pixel_hash == before_pixel_hash {
        return Err(format!(
            "post-recovery input did not change real Workshop state/pixels; diagnostics={input_diagnostic}"
        ));
    }
    size(mounts[0].window().canvas(), 960, 780)?;
    let resized = capture(&mounts[0]).await?;
    if resized.frame.width == after_input.frame.width
        || resized.frame.height == after_input.frame.height
    {
        return Err("post-recovery resize did not change presented extent".into());
    }
    if failures.borrow().len() != 3 {
        return Err("loss callback refusal was replayed after recovery".into());
    }
    let receipt = json!({"status":"pass","run":run,"elapsed_ms":now()?-started,
        "cancellation_controls":cancellation,"tab_release_controls":tab_release,
        "scope":"two actual full Workshop mounts, identical local keys, hidden second host, controlled owned Device.destroy; same-canvas queue presentation and complete presented/source RGBA under unchanged limits; compositor visibility requires separate browser review",
        "current_frame_async_quality_qualified":false,"current_frame_async_error_scopes_qualified":false,
        "physical_driver_reset":false,"os_accessibility_actions":false,"factory_count_inferred":false,
        "initial_adapter":initial_adapter,"replacement_adapter":adapter_identity(&replacement),
        "lost_generation":generation,"replacement_generation":replacement.generation(),"loss_reason":format!("{:?}",loss.reason),
        "exactly_once_capture_refusals":failures.borrow().len(),"queued_readback_loss_then_already_collected":true,"old_core_released":true,"shared_replacement_arc":true,
        "first":first,"second":second,"complete_registered_producers":full_producers,"input_route":"logical Host::key dispatch; real DOM gestures remain separate","post_input":{"seal":after_input.seal,"sha256":digest(&after_input.frame)},
        "resized":{"seal":resized.seal,"sha256":digest(&resized.frame)},
        "quality_pending_unqualified":format!("{:?}",replacement.renderer().vello_curve_quality_status()),
        "artifact_names":["first_before","first_after","second_before","second_after","first_reader_before","first_stylesheet_before","first_reader_after","first_stylesheet_after","second_reader_before","second_stylesheet_before","second_reader_after","second_stylesheet_after"]});
    let mut artifacts = vec![
        ("first_before".to_string(), first_before.frame),
        ("first_after".to_string(), first_after.frame),
        ("second_before".to_string(), second_before.frame),
        ("second_after".to_string(), second_after.frame),
    ];
    for (prefix, phase, copies) in [
        ("first", "before", first_sources_before),
        ("first", "after", first_sources_after),
        ("second", "before", second_sources_before),
        ("second", "after", second_sources_after),
    ] {
        for (slot, capture) in copies.into_iter().enumerate() {
            artifacts.push((
                format!(
                    "{prefix}_{}_{phase}",
                    if slot == 0 { "reader" } else { "stylesheet" }
                ),
                capture.frame,
            ));
        }
    }
    let bytes = artifacts
        .iter()
        .try_fold(0usize, |total, (_, frame)| {
            total.checked_add(frame.rgba.len())
        })
        .ok_or("artifact byte accounting overflow")?;
    if bytes > RETAINED_LIMIT {
        return Err("copied artifact bundle exceeds 128 MiB retention bound".into());
    }
    PIXELS.with(|pixels| {
        let mut pixels = pixels.borrow_mut();
        for (name, frame) in artifacts {
            pixels.insert(name, frame);
        }
    });
    let cleanup = dispose_lifetimes().await?;
    for mount in &mounts {
        let host = mount.host().borrow();
        if host.s.surface.is_some()
            || host.s.shared.render_core.is_some()
            || host.s.shared.render_recovery.is_some()
        {
            return Err("disposed mount retained host GPU bindings".into());
        }
    }
    // Both independent mounts have retired their surfaces/subscriptions, and
    // all caller-owned readbacks are collected or dropped. This fixture owns
    // the whole group: generic mount disposal must never destroy shared devices.
    let closed_generation = replacement.generation();
    let closed_core = Arc::downgrade(&replacement);
    replacement.device().destroy();
    let mut closure_notice = None;
    wait(|| {
        let Some(notice) = replacement.loss_notice() else {
            return Ok(false);
        };
        if notice.generation != closed_generation
            || notice.reason != wgpu::DeviceLostReason::Destroyed
        {
            return Err(format!(
                "replacement group closure returned unexpected loss: {notice:?}"
            ));
        }
        closure_notice = Some(notice);
        Ok(true)
    })
    .await?;
    let notice = closure_notice.ok_or("matching replacement device closure notice absent")?;
    drop(owner);
    drop(replacement);
    if closed_core.upgrade().is_some() {
        return Err("destroyed replacement group retained Rust core after explicit closure".into());
    }
    let mut receipt = receipt;
    receipt["activated_cleanup"] = cleanup;
    receipt["disposed_host_gpu_bindings_released"] = json!(true);
    receipt["actual_replacement_device_destroyed"] = json!(true);
    receipt["replacement_group_closure"] = json!({
        "generation":notice.generation,"reason":format!("{:?}",notice.reason),
        "matching_loss_notice_observed":true,"owner_and_core_dropped":true,"rust_core_released":true,
        "scope":"explicit Device.destroy of the fixture-exclusive replacement group after completed mount disposal; excludes physical driver reset"
    });
    Ok(receipt)
}

// Explicit diagnostic entrypoint, separate from root/reference-only gates.
#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub async fn run_acquired_surface_recovery(first_canvas:String,second_canvas:String,run:String)->Result<JsValue,JsValue>{
    if USED.with(|u|u.replace(true)){return Err(JsValue::from_str("fixture is one-shot"))}
    if run.is_empty() || run.len()>128{return Err(JsValue::from_str("bounded run identifier required"))}
    let result=acquired_controls::execute_recovery(&first_canvas,&second_canvas,&run).await;
    match result {
        Ok(receipt)=>{STATUS.with(|s|*s.borrow_mut()=receipt.clone());js_sys::JSON::parse(&receipt.to_string())},
        Err(error)=>{let cleanup=dispose_lifetimes().await;let remaining=acquired_controls::clear();
            let receipt=json!({"status":"fail","run":run,"error":error,"cleanup":format!("{cleanup:?}"),"remaining_capture_budget_bytes":remaining});
            STATUS.with(|s|*s.borrow_mut()=receipt.clone());Err(JsValue::from_str(&receipt.to_string()))}
    }
}
#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub fn capture_acquired_rgba(name:String)->Result<js_sys::Uint8Array,JsValue>{
    acquired_controls::export(&name).map_err(|e|JsValue::from_str(&e))
}
#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub fn admit_acquired_comparison(first:String,second:String)->Result<(),JsValue>{
    acquired_controls::compare_admission(&first,&second).map_err(|e|JsValue::from_str(&e))
}
#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub fn release_acquired_work(){acquired_controls::release_work()}
#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub fn clear_acquired_artifacts()->u64{acquired_controls::clear()}

#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub async fn run_acquired_surface_lifetime(first_canvas:String,second_canvas:String,run:String)->Result<JsValue,JsValue>{
    if USED.with(|u|u.replace(true)){return Err(JsValue::from_str("fixture is one-shot"))}
    if run.is_empty()||run.len()>128{return Err(JsValue::from_str("bounded run identifier required"))}
    finish_acquired_batch(acquired_controls::execute_lifetime(&first_canvas,&second_canvas,&run).await,&run).await
}
#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub async fn run_acquired_surface_faults(first_canvas:String,second_canvas:String,run:String)->Result<JsValue,JsValue>{
    if USED.with(|u|u.replace(true)){return Err(JsValue::from_str("fixture is one-shot"))}
    if run.is_empty()||run.len()>128{return Err(JsValue::from_str("bounded run identifier required"))}
    finish_acquired_batch(acquired_controls::execute_faults(&first_canvas,&second_canvas,&run).await,&run).await
}
#[cfg(feature = "current-frame-controls")]
async fn finish_acquired_batch(result:Result<Value,String>,run:&str)->Result<JsValue,JsValue>{
    match result {Ok(receipt)=>{STATUS.with(|s|*s.borrow_mut()=receipt.clone());js_sys::JSON::parse(&receipt.to_string())},
        Err(error)=>{let cleanup=dispose_lifetimes().await;let remaining=acquired_controls::clear();let receipt=json!({"status":"fail","run":run,"error":error,
            "cleanup":format!("{cleanup:?}"),"remaining_capture_budget_bytes":remaining});STATUS.with(|s|*s.borrow_mut()=receipt.clone());Err(JsValue::from_str(&receipt.to_string()))}}
}

#[cfg(feature = "current-frame-controls")]
#[wasm_bindgen]
pub async fn run_acquired_surface_lost(first_canvas:String,second_canvas:String,run:String)->Result<JsValue,JsValue>{
    if USED.with(|u|u.replace(true)){return Err(JsValue::from_str("fixture is one-shot"))}
    if run.is_empty()||run.len()>128{return Err(JsValue::from_str("bounded run identifier required"))}
    finish_acquired_batch(acquired_controls::execute_lost(&first_canvas,&second_canvas,&run).await,&run).await
}
