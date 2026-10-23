// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The frame pipeline: retained layout, paint emission, presentation, and
//! accessibility synchronization. Extracted from the woodshed-genet donor.
//!
//! A frame is deliberately in two halves. [`Host::relayout`] brings the
//! retained layout up to date and needs no GPU and no window; everything after
//! it rasterizes and presents. The split is what lets [`Harness`](crate::Harness)
//! run an application's real layout, hit testing, and input routing in an
//! ordinary `cargo test`.

use std::collections::HashMap;

use crate::A11yAction;
use cambium::PointerClick;
use genet_render::VisualCaret;
use genet_scripted_dom::NodeId;
use layout_dom_api::{DomMutation, LayoutDom as _};
use paint_list_api::{DeviceIntSize, PaintEnvelope};

use crate::input::to_visual_caret;
use crate::meristem_bounds::RootView;
use crate::{AppCtx, Host};

fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}

#[derive(Debug)]
pub(crate) enum LeafFragmentAdmissionError {
    RendererAccess(genet_render_host::RasterizeFailure),
    Translation {
        key: u64,
        epoch: u64,
        error: paint_list_render::TranslationError,
    },
    Retained {
        key: u64,
        epoch: u64,
        error: netrender::RetainedFragmentError,
    },
}

impl std::fmt::Display for LeafFragmentAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RendererAccess(error) => {
                write!(f, "retained fragment renderer access refused: {error}")
            },
            Self::Translation { key, epoch, error } => {
                write!(
                    f,
                    "leaf {key} epoch {epoch} paint translation refused: {error}"
                )
            },
            Self::Retained { key, epoch, error } => {
                write!(
                    f,
                    "leaf {key} epoch {epoch} retained fragment refused: {error}"
                )
            },
        }
    }
}

#[cfg(test)]
mod gpu_binding_controls {
    use super::*;
    use crate::{CaptureRefusal, HostHooks, HostOptions, HostState, HostWake, Runner, ScriptedDom};
    use cambium::{AnyView, GenetCtx, GenetElement, custom_leaf};
    use paint_list_api::{
        CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PaintList as _,
        items::ExternalTextureItem,
    };
    use std::{cell::RefCell, rc::Rc, sync::Arc};

    struct TestSurface(genet_render_host::RenderCore);
    impl crate::Surface for TestSurface {
        fn core(&self) -> &genet_render_host::RenderCore {
            &self.0
        }
        fn format(&self) -> wgpu::TextureFormat {
            wgpu::TextureFormat::Rgba8Unorm
        }
        fn resize(&mut self, _: u32, _: u32) {}
        fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
            None
        }
    }

    struct TestWindow;
    impl crate::HostWindow for TestWindow {
        fn request_redraw(&self) {}
        fn inner_size(&self) -> (u32, u32) {
            (8, 6)
        }
        fn scale_factor(&self) -> f64 {
            1.0
        }
        fn set_ime_allowed(&self, _: bool) {}
        fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
    }

    struct BorderLeaf(Rc<std::cell::Cell<bool>>);
    impl sprigging::Leaf for BorderLeaf {
        fn measure(&mut self, _: sprigging::SizeHint, _: sprigging::SizeHint) -> sprigging::Size {
            sprigging::Size {
                width: 8.0,
                height: 6.0,
            }
        }
        fn paint(&mut self, cx: &mut sprigging::PaintCx<'_>) {
            let bounds = LayoutRect::new(LayoutPoint::new(0.0, 0.0), LayoutPoint::new(8.0, 6.0));
            if self.0.get() {
                cx.emit(PaintCmd::DrawBorder(paint_list_api::items::BorderItem {
                    placement: CommonPlacement::new(bounds),
                    widths: paint_list_api::LayoutSideOffsets::new(1.0, 1.0, 1.0, 1.0),
                    details: paint_list_api::items::BorderDetails::Normal(
                        paint_list_api::NormalBorder {
                            do_aa: false,
                            ..Default::default()
                        },
                    ),
                }));
            } else {
                cx.emit(PaintCmd::DrawRect(paint_list_api::items::RectItem {
                    placement: CommonPlacement::new(bounds),
                    color: paint_list_api::ColorF::new(1.0, 0.0, 0.0, 1.0),
                }));
            }
        }
        fn paint_dirty(&self) -> bool {
            true
        }
    }

    #[test]
    fn rejected_leaf_epoch_refuses_presentation_and_recovers_without_replacing_old_fragment() {
        let bad = Rc::new(std::cell::Cell::new(false));
        let mut state: HostState<App, Logic, Child> = HostState::new();
        state.shared.sheet = "custom-leaf { display:block;width:8px;height:6px; }".into();
        state
            .shared
            .leaves
            .insert(7, Box::new(BorderLeaf(bad.clone())));
        state.surface = Some(Box::new(TestSurface(
            genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
                enable_vello: true,
                tile_cache_size: Some(32),
                ..Default::default()
            })
            .expect("retained leaf test needs a headless adapter"),
        )));
        state.window = Some(Box::new(TestWindow));
        state.runner = Some(Runner::new(
            Rc::new(RefCell::new(ScriptedDom::new())),
            root as Logic,
            App {
                draft: String::new(),
            },
        ));
        let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
        let mut host = Host::new(
            HostOptions::default(),
            None,
            HostHooks::inert(),
            state,
            wake,
        );

        assert!(host.prepare_presentation_frame().unwrap().is_some());
        let original = host.s.shared.leaf_fragments[&7];
        bad.set(true);
        let refusal = Rc::new(RefCell::new(None));
        let out = refusal.clone();
        host.s.pending_capture = Some(Box::new(move |result| {
            *out.borrow_mut() = Some(result.err().expect("capture refused"));
        }));
        host.redraw();
        assert_eq!(host.s.shared.leaf_fragments[&7], original);
        assert!(matches!(
            refusal.borrow().as_ref(),
            Some(CaptureRefusal::PresentationError { error, .. })
                if error.contains("non-antialiased border geometry")
        ));
        assert!(host.s.last_redraw_presentation.is_none());

        bad.set(false);
        assert!(host.prepare_presentation_frame().unwrap().is_some());
        let corrected = host.s.shared.leaf_fragments[&7];
        assert_eq!(
            corrected.0, original.0,
            "the old id is updated after recovery"
        );
        assert!(corrected.1 > original.1, "only an admitted epoch is cached");
    }

    struct App {
        draft: String,
    }
    type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
    type Logic = fn(&App) -> Child;
    fn root(_: &App) -> Child {
        Box::new(custom_leaf::<App, ()>(7, 8, 6))
    }
    struct UnresolvedLeaf;
    impl sprigging::Leaf for UnresolvedLeaf {
        fn measure(&mut self, _: sprigging::SizeHint, _: sprigging::SizeHint) -> sprigging::Size {
            sprigging::Size {
                width: 8.0,
                height: 6.0,
            }
        }
        fn paint(&mut self, cx: &mut sprigging::PaintCx<'_>) {
            cx.emit(PaintCmd::DrawExternalTexture(ExternalTextureItem {
                placement: CommonPlacement::new(LayoutRect::new(
                    LayoutPoint::new(0.0, 0.0),
                    LayoutPoint::new(8.0, 6.0),
                )),
                texture_key: 7,
                content_generation: Some(1),
                opacity: 0.73,
            }));
        }
        fn paint_dirty(&self) -> bool {
            false
        }
    }

    #[test]
    fn actual_host_missing_gpu_binding_refuses_all_capture_forms_once_and_keeps_authority() {
        let mut state: HostState<App, Logic, Child> = HostState::new();
        state.shared.sheet = "custom-leaf { display:block;width:8px;height:6px; }".into();
        state.shared.leaves.insert(7, Box::new(UnresolvedLeaf));
        state.runner = Some(Runner::new(
            Rc::new(RefCell::new(ScriptedDom::new())),
            root as Logic,
            App {
                draft: "application-owned draft".into(),
            },
        ));
        let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
        let mut host: Host<App, Logic, Child> = Host::new(
            HostOptions::default(),
            None,
            HostHooks::inert(),
            state,
            wake,
        );
        host.relayout(8.0, 6.0);
        let list = host.emit_paint_list(8.0, 6.0, None).unwrap();
        assert!(list.commands().iter().any(|command| matches!(command,
            PaintCmd::DrawExternalTexture(item) if item.texture_key == 7 && item.content_generation == Some(1))));
        let refusals = Rc::new(RefCell::new(Vec::new()));
        fn record(
            result: CaptureRefusal,
            name: &'static str,
            out: &Rc<RefCell<Vec<&'static str>>>,
        ) {
            assert!(matches!(
                result,
                CaptureRefusal::ProducerResource {
                    generation: 0,
                    error: crate::ProducerGpuRemapError::MissingProducer { local_key: 7 }
                }
            ));
            out.borrow_mut().push(name);
        }
        let out = refusals.clone();
        host.s.pending_capture = Some(Box::new(move |result| {
            record(result.err().unwrap(), "image", &out)
        }));
        let out = refusals.clone();
        host.s.pending_stamped_capture = Some(Box::new(move |result| {
            record(result.err().unwrap(), "stamped", &out)
        }));
        let out = refusals.clone();
        host.s.pending_paint_capture = Some(Box::new(move |result| {
            record(result.err().unwrap(), "paint", &out)
        }));
        host.s.pending_presentation_observer = Some(Box::new(|_, _| {
            panic!("refused frame must never be presented")
        }));
        assert!(host.emit_native_scene(8.0, 6.0).unwrap().is_none());
        assert!(host.emit_native_scene(8.0, 6.0).unwrap().is_none());
        assert_eq!(*refusals.borrow(), ["image", "stamped", "paint"]);
        assert!(host.s.pending_presentation_observer.is_none());
        assert!(host.s.last_redraw_presentation.is_none());
        assert_eq!(
            host.s.runner.as_ref().unwrap().state().draft,
            "application-owned draft"
        );
        assert_eq!(
            host.s.shared.producers.gpu_remap_failure(),
            Some(crate::ProducerGpuRemapError::MissingProducer { local_key: 7 })
        );
        let after = host.emit_paint_list(8.0, 6.0, None).unwrap();
        assert_eq!(
            format!("{:?}", after.commands()),
            format!("{:?}", list.commands())
        );
    }
}

/// Opt-in startup attribution, including attempts that never present. Stores
/// only CPU timing and host identity; it does not retain render resources.
pub(crate) struct FrameTrace {
    started: crate::Instant,
    host: u64,
    last_presented: u64,
}

impl FrameTrace {
    fn enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var_os("CAMBIUM_HOST_FRAME_TRACE").is_some_and(|value| value != "0")
        })
    }

    fn new(host: u64, last_presented: u64) -> Option<Self> {
        Self::enabled().then(|| Self {
            started: crate::Instant::now(),
            host,
            last_presented,
        })
    }

    pub(crate) fn mark(&self, phase: &str, detail: std::fmt::Arguments<'_>) {
        eprintln!(
            "[cambium-frame] host={} last-presented={} elapsed={}us phase={phase} {detail}",
            self.host,
            self.last_presented,
            elapsed_us(self.started.elapsed()),
        );
    }
}

impl Drop for FrameTrace {
    fn drop(&mut self) {
        self.mark(
            "return",
            format_args!("unwinding={}", std::thread::panicking()),
        );
    }
}

struct SpriggingSource<'a> {
    rendered: &'a sprigging::RenderedLeaves,
    producers: &'a crate::ProducerRegistry,
    portable: Option<&'a crate::producer::registry::PortablePlan>,
}

impl SpriggingSource<'_> {
    fn leaf_commands(&self, key: u64) -> Option<Vec<paint_list_api::PaintCmd>> {
        if let Some(plan) = self.portable {
            return plan
                .commands(key)
                .or_else(|| self.producers.contains(key).then(Vec::new))
                .or_else(|| self.rendered.get(key).map(<[_]>::to_vec));
        }
        self.producers
            .commands(key)
            .or_else(|| self.rendered.get(key).map(<[_]>::to_vec))
    }
}

/// The focused text field's paint inputs for this frame: which node, where the
/// caret is, and the selection byte range when one should be drawn.
type FocusedOverlay = (NodeId, VisualCaret, Option<(usize, usize)>);

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Run the application's per-frame hook. Returns `true` when it wants more
    /// frames (an animation is live).
    pub fn prepare_frame(&mut self) -> bool {
        let animating = {
            let logical_size = self.logical_size();
            let (ui_zoom, zoom_changed) = self.take_zoom_edge();
            let geometry = self.s.geometry;
            let frame_profile = self.s.last_frame_profile;
            let commands = self.s.commands.clone();
            let window = self.s.window.as_deref();
            let layout = self.s.layout.as_ref();
            let Some(runner) = self.s.runner.as_mut() else {
                return false;
            };
            let mut ctx = AppCtx {
                runner,
                layout,
                window,
                logical_size,
                ui_zoom,
                zoom_changed,
                leaves: &mut self.s.shared.leaves,
                files: &mut self.s.files,
                producers: &mut self.s.shared.producers,
                set_sheet: &mut self.s.pending_sheet,
                set_ui_zoom: &mut self.s.pending_ui_zoom,
                close: &mut self.s.close_requested,
                wake: &self.wake,
                capture: &mut self.s.pending_capture,
                presentation_observer: &mut self.s.pending_presentation_observer,
                capture_stamped: &mut self.s.pending_stamped_capture,
                capture_paint: &mut self.s.pending_paint_capture,
                pointer: &mut self.s.pending_pointer,
                a11y: &mut self.s.pending_a11y,
                a11y_owner: &self.s.a11y_owner,
                a11y_publication: crate::A11yPublicationSnapshot {
                    host: self.s.presentation_host,
                    sequence: self.s.a11y_publication_sequence,
                    publication: self.s.a11y_publication,
                },
                scroll: &mut self.s.pending_scroll,
                window_commands: &commands,
                render_core: self.s.shared.render_core.as_ref(),
                geometry,
                frame_profile,
                presentation: self.s.last_redraw_presentation,
            };
            (self.hooks.frame)(&mut ctx)
        };
        if let Some(sheet) = self.s.pending_sheet.take() {
            self.swap_sheet(sheet);
        }
        if let Some(zoom) = self.s.pending_ui_zoom.take() {
            self.set_ui_zoom(zoom);
        }
        animating
    }

    /// Bring the retained layout up to date at `(lw, lh)` logical px: drain the
    /// runner's DOM mutations, rebuild the host-owned Livery frame when its
    /// inputs changed, and re-render custom-paint leaves at their new boxes. No
    /// GPU, no window. Returns whether an animation is still live.
    /// Republish the Window-Controls-Overlay titlebar area when it changes.
    ///
    /// As a stylesheet rule rather than an inline style on the root: the root's
    /// `style` attribute belongs to the application, and a host that
    /// overwrites it each frame would silently drop whatever the app put
    /// there. A `:root` rule declares the same inheriting custom properties
    /// without touching the DOM at all.
    ///
    /// Returns whether the values moved, because a changed sheet has to force
    /// the full relayout path — the incremental one applies DOM mutations, and
    /// this is not one.
    fn publish_titlebar_area(&mut self, lw: f32) -> bool {
        // The platform reports what it reserved in *device*-logical pixels —
        // macOS's traffic lights are a fixed size on screen, not a fixed
        // number of CSS pixels — and the sheet declares CSS ones. Under zoom
        // those are different units, so the strip is divided into layout space
        // before it is published; otherwise the page's content would clear a
        // gap of the wrong height at every zoom but 1.
        let zoom = self.ui_zoom();
        let insets = self
            .s
            .window
            .as_ref()
            .map_or(crate::TitlebarInsets::NONE, |w| {
                let insets = w.titlebar_insets();
                crate::TitlebarInsets {
                    left: insets.left / zoom,
                    right: insets.right / zoom,
                    height: insets.height / zoom,
                }
            });
        if self.s.titlebar_published == Some((insets, lw)) {
            return false;
        }
        self.s.titlebar_published = Some((insets, lw));
        self.s.titlebar_sheet = format!(":root {{ {} }}", insets.declarations(lw));
        true
    }

    pub fn relayout(&mut self, lw: f32, lh: f32) -> bool {
        if self.s.runner.is_none() {
            return false;
        }
        let layout_update_started = crate::Instant::now();
        let now_s = self.s.anim_base.elapsed().as_secs_f64();
        let titlebar_moved = self.publish_titlebar_area(lw);
        // Another window swapped the shared sheet: lay out afresh, as the
        // window that swapped it does.
        if self.s.layout_sheet_generation != self.s.shared.sheet_generation {
            self.s.layout = None;
            self.s.layout_size = (0.0, 0.0);
            self.s.layout_sheet_generation = self.s.shared.sheet_generation;
        }
        let runner = self.s.runner.as_mut().expect("checked above");
        let mut muts: Vec<DomMutation<NodeId>> = Vec::new();
        runner.drain_mutations(&mut muts);
        let dom = runner.dom();
        let mount = runner.mount();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, mount);
        let sheets: Vec<&str> = vec![self.s.shared.sheet.as_str(), self.s.titlebar_sheet.as_str()];
        let mutation_count = muts.len() as u64;
        let size_changed = self.s.layout_size != (lw, lh);
        let mut tick_us = 0;
        let apply_us = 0;
        let mut rebuild_us = 0;
        let mut style_resolve_us = 0;
        let mut layout_with_text_us = 0;
        let mut content_extent_us = 0;
        let mut rebuilt = false;
        match self.s.layout.as_mut() {
            Some(layout) if muts.is_empty() && !size_changed && !titlebar_moved => {
                let phase = crate::Instant::now();
                let _ = layout.tick_animations(&view, now_s);
                tick_us = elapsed_us(phase.elapsed());
            },
            Some(layout) if !size_changed && !titlebar_moved => {
                rebuilt = true;
                let phase = crate::Instant::now();
                layout.rebuild(&view, lw, lh, &muts);
                rebuild_us = elapsed_us(phase.elapsed());
            },
            _ => {
                rebuilt = true;
                let phase = crate::Instant::now();
                let mut layout = crate::OwnedLayout::new(
                    &view,
                    &sheets,
                    lw,
                    lh,
                    &self.s.shared.fonts,
                    &self.s.shared.images,
                );
                // Carry BOTH scroll planes across rebuilds: element scroll and
                // the document scroll. Dropping the latter snaps a scrolled
                // page back to the top on structural re-render. Both setters
                // clamp against the layout this session has already done, so a
                // container that shrank or stopped scrolling cannot carry a
                // stale offset in.
                if let Some(prev) = self.s.layout.as_ref() {
                    layout.set_element_scroll(&view, prev.element_scroll().clone());
                    layout.set_viewport_scroll(prev.viewport_scroll());
                }
                self.s.layout = Some(layout);
                self.s.layout_size = (lw, lh);
                rebuild_us = elapsed_us(phase.elapsed());
            },
        }
        // A caret that moved since the last relayout (an edit, a caret key, a
        // click) is kept in view within its field, as a single-line input
        // keeps it. One that has not moved stays where the user scrolled it.
        let caret = self.focused_overlay().map(|(node, caret, _)| (node, caret));
        match caret {
            Some((node, caret)) if self.s.caret_followed != Some((node, caret.byte)) => {
                let layout = self.s.layout.as_mut().expect("layout just ensured");
                let _ = layout.caret_into_view(&view, node, caret);
                self.s.caret_followed = Some((node, caret.byte));
            },
            Some(_) => {},
            None => self.s.caret_followed = None,
        }
        // Scroll requests resolve against the layout just brought current, so
        // a node the requesting dispatch created already has a box. Each
        // resolves on its own: one that finds nothing drops only itself.
        if !self.s.pending_scroll.is_empty() {
            let layout = self.s.layout.as_mut().expect("layout just ensured");
            let now = crate::Instant::now();
            for request in std::mem::take(&mut self.s.pending_scroll) {
                for target in layout.scroll_into_view(&view, request.node, request.align) {
                    self.s.scrollbar_fade.note(target, now);
                }
            }
        }
        let layout = self.s.layout.as_ref().expect("layout just ensured");
        if rebuilt {
            (style_resolve_us, layout_with_text_us, content_extent_us) = layout.stage_timings();
        }
        let anim_active = layout.has_active_animations();
        let layout_update_us = elapsed_us(layout_update_started.elapsed());
        let leaf_boxes_started = crate::Instant::now();
        let sizes: HashMap<u64, (f32, f32)> = layout.custom_leaf_boxes(&view).into_iter().collect();
        self.s.leaf_keys.clear();
        self.s.leaf_keys.extend(sizes.keys().copied());
        let leaf_boxes_us = elapsed_us(leaf_boxes_started.elapsed());
        let leaf_render_started = crate::Instant::now();
        let leaf_repaints = self.s.shared.leaves.render_into(
            |key| {
                sizes
                    .get(&key)
                    .map(|&(width, height)| sprigging::Size { width, height })
            },
            &mut self.s.shared.rendered,
        );
        self.s.last_layout_update_us = layout_update_us;
        self.s.last_layout_tick_us = tick_us;
        self.s.last_layout_apply_us = apply_us;
        self.s.last_layout_rebuild_us = rebuild_us;
        self.s.last_style_resolve_us = style_resolve_us;
        self.s.last_layout_with_text_us = layout_with_text_us;
        self.s.last_content_extent_us = content_extent_us;
        self.s.last_layout_mutations = mutation_count;
        self.s.last_layout_rebuilt = rebuilt;
        self.s.last_leaf_boxes_us = leaf_boxes_us;
        self.s.last_leaf_render_us = elapsed_us(leaf_render_started.elapsed());
        self.s.last_leaf_repaints = leaf_repaints as u64;
        anim_active
    }

    /// Netrender roadmap E4 — reconcile the renderer's retained-fragment
    /// registry with this frame's rendered leaves. Runs each redraw after
    /// `relayout` (which refreshed `rendered`) and before `emit_scene`, so the
    /// map the emitter consults is current by construction.
    ///
    /// Only Path-A splices free of per-frame state become fragments: a splice
    /// carrying `DrawExternalTexture` or `DrawShadow` keeps the inline path,
    /// because composite textures and blurred shadow masks are rebuilt per
    /// frame by the host painter and cannot be retained in a lowering.
    pub(crate) fn sync_leaf_fragments(&mut self) -> Result<(), LeafFragmentAdmissionError> {
        use paint_list_api::PaintCmd;

        let Some(surface) = self.s.surface.as_deref().or_else(|| {
            self.s
                .presentation_driver
                .as_ref()
                .and_then(|driver| driver.wgpu_surface())
        }) else {
            // No surface this turn (suspended, or not yet booted). The
            // renderer lives in the render core, which outlives the surface,
            // so fragments registered before a suspend are still valid after
            // the resume and are kept, not forgotten.
            return Ok(());
        };
        // One operation over this window's leaves; nothing below rasterizes
        // or calls a producer while the browser frame gate is held.
        let renderer = crate::try_renderer(surface.core())
            .map_err(LeafFragmentAdmissionError::RendererAccess)?;

        let mut seen: Vec<u64> = Vec::new();
        for (key, epoch, splice) in self.s.shared.rendered.path_a_entries() {
            let fragmentable = !splice.iter().any(|cmd| {
                matches!(
                    cmd,
                    PaintCmd::DrawExternalTexture(_) | PaintCmd::DrawShadow(_)
                )
            });
            if !fragmentable {
                if let Some((id, _)) = self.s.shared.leaf_fragments.remove(&key) {
                    let _ = renderer.remove_fragment(id);
                }
                continue;
            }
            seen.push(key);
            match self.s.shared.leaf_fragments.get(&key) {
                Some((_, e)) if *e == epoch => {},
                Some(&(id, _)) => {
                    let fragment =
                        paint_list_render::translate_paint_cmds_to_fragment(splice, &[], &[])
                            .map_err(|error| LeafFragmentAdmissionError::Translation {
                                key,
                                epoch,
                                error,
                            })?;
                    renderer
                        .try_update_fragment(id, fragment)
                        .map_err(|error| LeafFragmentAdmissionError::Retained {
                            key,
                            epoch,
                            error,
                        })?;
                    self.s.shared.leaf_fragments.insert(key, (id, epoch));
                },
                None => {
                    let fragment =
                        paint_list_render::translate_paint_cmds_to_fragment(splice, &[], &[])
                            .map_err(|error| LeafFragmentAdmissionError::Translation {
                                key,
                                epoch,
                                error,
                            })?;
                    let id = renderer.try_register_fragment(fragment).map_err(|error| {
                        LeafFragmentAdmissionError::Retained { key, epoch, error }
                    })?;
                    self.s.shared.leaf_fragments.insert(key, (id, epoch));
                },
            }
        }
        // Sweep leaves that no longer render (removed from the registry or
        // not laid out): their retained lowerings go with them.
        let stale: Vec<u64> = self
            .s
            .shared
            .leaf_fragments
            .keys()
            .copied()
            .filter(|k| !seen.contains(k))
            .collect();
        for key in stale {
            if let Some((id, _)) = self.s.shared.leaf_fragments.remove(&key) {
                let _ = renderer.remove_fragment(id);
            }
        }
        Ok(())
    }

    /// Where the focused field's caret paints, `(x, y, width, height)`, in the
    /// coordinates the cursor uses. `None` with no focused field, before a
    /// layout, or while the caret is scrolled out of its field.
    pub fn focused_caret_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (node, caret, _) = self.focused_overlay()?;
        let layout = self.s.layout.as_ref()?;
        let runner = self.s.runner.as_ref()?;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let rect = layout.caret_rect_for_position(&view, node, caret, 2.0)?;
        Some((rect.x, rect.y, rect.width, rect.height))
    }

    /// Where the focused field's selection paints, one rect per line run, in
    /// the same coordinates, clipped to the field. Empty with no selection.
    pub fn focused_selection_rects(&self) -> Vec<(f32, f32, f32, f32)> {
        let Some((node, _, Some((start, end)))) = self.focused_overlay() else {
            return Vec::new();
        };
        let (Some(layout), Some(runner)) = (self.s.layout.as_ref(), self.s.runner.as_ref()) else {
            return Vec::new();
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        layout
            .selection_rects(&view, node, start, end)
            .into_iter()
            .map(|rect| (rect.x, rect.y, rect.width, rect.height))
            .collect()
    }

    /// The focused text field's paint inputs, as the application maps them.
    fn focused_overlay(&self) -> Option<FocusedOverlay> {
        let runner = self.s.runner.as_ref()?;
        let slot = (self.hooks.focused_text)(runner)?;
        let input = (slot.get)(runner.state());
        let mut caret = to_visual_caret(input.caret_position());
        caret.byte = input.caret_byte_in_render();
        let selection = if input.composition().is_none() && input.has_selection() {
            Some(input.selection_bytes())
        } else {
            None
        };
        Some((slot.node, caret, selection))
    }

    /// Tell the platform IME where the caret is, so a candidate window opens
    /// beside the text rather than at the window origin.
    pub(crate) fn sync_ime_area(&self) {
        let (Some(window), Some(layout), Some(runner)) = (
            self.s.window.as_ref(),
            self.s.layout.as_ref(),
            self.s.runner.as_ref(),
        ) else {
            return;
        };
        let Some((node, caret, _)) = self.focused_overlay() else {
            return;
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let Some(rect) = layout.caret_rect_for_position(&view, node, caret, 2.0) else {
            return;
        };
        // The seam takes the *platform's* logical coordinates (winit multiplies
        // them by the device scale, a browser by nothing), and the caret rect
        // is in layout coordinates. Those differ by exactly the zoom, so the
        // rect is carried back across it here — a candidate window that opened
        // at four fifths of the caret's position would be a zoom bug the user
        // sees before any other.
        let zoom = f64::from(self.ui_zoom());
        window.set_ime_cursor_area(
            rect.x as f64 * zoom,
            rect.y as f64 * zoom,
            (rect.width.max(2.0) as f64) * zoom,
            (rect.height.max(1.0) as f64) * zoom,
        );
    }

    /// Emit this frame's paint list with selection and caret inside the text
    /// field's CSS paint context, then any active overlay scrollbars, and
    /// lower it to a netrender scene plus the GPU resources that scene names.
    pub(crate) fn emit_paint_list(
        &mut self,
        lw: f32,
        lh: f32,
        portable: Option<&crate::producer::registry::PortablePlan>,
    ) -> Option<genet_livery::LiveryPaintList> {
        let focused_overlay = self.focused_overlay();
        let runner = self.s.runner.as_ref()?;
        let layout = self.s.layout.as_mut()?;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let source = SpriggingSource {
            rendered: &self.s.shared.rendered,
            producers: &self.s.shared.producers,
            portable,
        };
        let mut list = layout.emit_paint_list_with_leaves(
            &view,
            DeviceIntSize::new(lw as i32, lh as i32),
            focused_overlay,
            |key| source.leaf_commands(key),
            // A retained fragment has no CSS clip or layer identity in the
            // renderer yet. Keep custom leaves in the recorded Livery slot as
            // ordinary commands until that composition boundary is real.
            // This preserves their stacking relation to DOM overlays.
            |_| None,
        );
        // Overlay scrollbar thumbs mid-hold/mid-fade: the engine draws the
        // geometry, the shared fade clock supplies alpha.
        let now = crate::Instant::now();
        let fade = &self.s.scrollbar_fade;
        layout.append_scrollbars(&view, &mut list, &|t| fade.alpha(t, now));
        Some(list)
    }

    /// Capture the current retained layout and actual producer callbacks without
    /// acquiring a device. Call frame preparation and relayout before this, as
    /// the host does for a presented frame. No cache/lifecycle commit precedes
    /// complete envelope admission.
    pub fn capture_portable(
        &mut self,
    ) -> Result<crate::PortablePaintCapture, crate::PortableCaptureError> {
        self.capture_portable_for(crate::ProducerFramePurpose::Export)
    }

    pub(crate) fn capture_portable_for(
        &mut self,
        purpose: crate::ProducerFramePurpose,
    ) -> Result<crate::PortablePaintCapture, crate::PortableCaptureError> {
        use crate::PortableCaptureError;
        let scale = self.layout_scale() as f32;
        let plan = {
            let runner = self
                .s
                .runner
                .as_ref()
                .ok_or(PortableCaptureError::MissingLayout)?;
            let layout = self
                .s
                .layout
                .as_ref()
                .ok_or(PortableCaptureError::MissingLayout)?;
            let dom = runner.dom();
            let dom = dom.borrow();
            let view = crate::WindowDom::new(&dom, runner.mount());
            let shared = &mut self.s.shared;
            let held = &shared.held_elsewhere;
            shared.producers.plan_portable_for(
                layout,
                &view,
                scale,
                &|key, owner| held.contains(&key) || owner.is_some_and(|node| view.elsewhere(node)),
                purpose,
            )?
        };
        let (width, height) = self.s.layout_size;
        let list = self
            .emit_paint_list(width, height, Some(&plan))
            .ok_or(PortableCaptureError::MissingLayout)?;
        self.s
            .shared
            .producers
            .admit_portable(plan, PaintEnvelope::from_list(&list))
    }

    /// Translate a native frame once and bind its producer draws to this
    /// registry's physical renderer identities. A refused binding refuses the
    /// frame and its captures; it never presents unbound producer content.
    pub(crate) fn emit_native_scene(
        &mut self,
        lw: f32,
        lh: f32,
    ) -> Result<
        Option<(PaintEnvelope, paint_list_render::TranslatedDisplayList)>,
        paint_list_render::TranslationError,
    > {
        let Some(list) = self.emit_paint_list(lw, lh, None) else {
            return Ok(None);
        };
        let envelope = PaintEnvelope::from_list(&list);
        let mut translated =
            paint_list_render::translate_envelope_with_external_textures(&envelope)?;
        if let Err(error) = self
            .s
            .shared
            .producers
            .remap_gpu_display_list(&mut translated)
        {
            self.refuse_producer_resource(error);
            return Ok(None);
        }
        Ok(Some((envelope, translated)))
    }

    pub(crate) fn refuse_producer_resource(&mut self, error: crate::ProducerGpuRemapError) {
        if FrameTrace::enabled() {
            eprintln!(
                "[cambium-frame] host={} last-presented={} phase=producer-resource-refused {error}",
                self.s.presentation_host, self.s.presentation_sequence,
            );
        }
        self.s.shared.producers.record_gpu_remap_failure(error);
        let refusal = crate::CaptureRefusal::ProducerResource {
            generation: self
                .s
                .shared
                .render_core
                .as_ref()
                .map_or(0, |core| core.generation()),
            error,
        };
        self.s.last_redraw_presentation = None;
        if let Some(capture) = self.s.pending_capture.take() {
            capture(Err(refusal.clone()));
        }
        if let Some(capture) = self.s.pending_stamped_capture.take() {
            capture(Err(refusal.clone()));
        }
        if let Some(capture) = self.s.pending_paint_capture.take() {
            capture(Err(refusal));
        }
        self.s.pending_presentation_observer = None;
    }

    pub fn redraw(&mut self) {
        let trace = FrameTrace::new(self.s.presentation_host, self.s.presentation_sequence);
        macro_rules! trace {
            ($phase:expr, $($args:tt)*) => {
                if let Some(trace) = &trace {
                    trace.mark($phase, format_args!($($args)*));
                }
            };
        }
        trace!("begin", "");
        let prepared = match self.prepare_presentation_frame() {
            Ok(Some(frame)) => frame,
            Ok(None) => return,
            Err(failure) => {
                self.refuse_presentation(failure);
                return;
            },
        };
        if self.s.presentation_driver.is_some() {
            self.present_with_driver(prepared);
            return;
        }
        let frame_started = prepared.started;
        let mut profile = prepared.profile;
        let Some(surface) = self.s.surface.as_deref() else {
            return;
        };
        let output = crate::presentation::wgpu::WgpuPresentation::new(surface).present(
            &prepared,
            &mut profile,
            trace.as_ref(),
        );
        let output = match output {
            Ok(Some(output)) => output,
            Ok(None) => {
                profile.total_us = elapsed_us(frame_started.elapsed());
                self.s.last_frame_profile = Some(profile);
                return;
            },
            Err(failure) => {
                self.refuse_presentation(failure);
                return;
            },
        };
        let crate::presentation::wgpu::WgpuOutput { _texture, view } = output;
        let paint_capture = match &prepared.paint {
            crate::PreparedPaint::Native(envelope) if self.s.pending_paint_capture.is_some() => {
                Some(envelope.clone())
            },
            _ => None,
        };
        let presented = prepared.presentation_stamp();
        self.s.presentation_sequence = presented.sequence;
        let sequence = presented.sequence;
        self.s.last_redraw_presentation = Some(presented);
        trace!("presentation-recorded", "sequence={sequence}");
        // Freeze product state before any capture callback, queued pointer
        // dispatch or platform accessibility action can change the product.
        let _ = self.s.capture_owner.resume(
            crate::PresentationBackend::Wgpu,
            presented.context_generation,
        );
        self.observe_presentation(presented);
        let surface = self.s.surface.as_ref().expect("presented surface exists");
        // A capture armed by the application: run it while the rasterized
        // view is still alive.
        let phase = crate::Instant::now();
        if let Some(envelope) = paint_capture
            && let Some(capture) = self.s.pending_paint_capture.take()
        {
            capture(Ok(envelope));
        }
        if let Some(capture) = self.s.pending_capture.take() {
            capture(self.s.capture_owner.begin(
                crate::PresentationBackend::Wgpu,
                presented,
                || crate::capture::start_presentation_readback(&**surface, &view, presented),
            ));
        }
        if let Some(capture) = self.s.pending_stamped_capture.take() {
            capture(self.s.capture_owner.begin(
                crate::PresentationBackend::Wgpu,
                presented,
                || crate::capture::start_presentation_readback(&**surface, &view, presented),
            ));
        }
        profile.capture_us = elapsed_us(phase.elapsed());
        if prepared.animating
            && let Some(window) = self.s.window.as_ref()
        {
            window.request_redraw();
        }
        // A `frame` hook runs before this frame's layout, so anything it queued
        // is delivered here instead — hit-testing against the layout that was
        // just built rather than the previous one.
        let phase = crate::Instant::now();
        self.drain_pointer();
        profile.pointer_us = elapsed_us(phase.elapsed());
        profile.total_us = elapsed_us(frame_started.elapsed());
        self.s.last_frame_profile = Some(profile);
    }

    /// Latest owner-stamped publication observation; reading does not sync.
    pub fn a11y_publication(&self) -> crate::A11yPublicationSnapshot {
        crate::A11yPublicationSnapshot {
            host: self.s.presentation_host,
            sequence: self.s.a11y_publication_sequence,
            publication: self.s.a11y_publication,
        }
    }

    pub(crate) fn stamp_a11y_publication(
        &mut self,
        publication: crate::A11yPublication,
    ) -> crate::A11yPublicationSnapshot {
        self.s.a11y_publication = publication;
        self.s.a11y_publication_sequence = self
            .s
            .a11y_publication_sequence
            .checked_add(1)
            .expect("accessibility observation sequence exhausted");
        self.a11y_publication()
    }

    pub(crate) fn record_a11y_publication(&mut self, publication: crate::A11yPublication) {
        let snapshot = self.stamp_a11y_publication(publication);
        if let Some(observer) = &mut self.s.a11y_publication_observer {
            observer(snapshot);
        }
    }

    /// Hand this frame to the accessibility host: build/install/update the
    /// tree, and route the screen reader's requests back into the retained DOM.
    /// Called after `redraw`, once this frame's layout exists. The first sync
    /// reveals the hidden window (install-before-show).
    ///
    /// The two request kinds stay apart, because they mean different things to
    /// the person using the reader: `Click` is "do this control's thing" and
    /// goes through the same dispatch a mouse press does; `Focus` is "put the
    /// cursor here" and only moves focus. Collapsing them would fire every
    /// control a reader navigates across.
    pub fn sync_a11y(&mut self) -> Vec<crate::A11yDispatch> {
        use crate::{
            A11yAdapterState as Adapter, A11yPublication as Publication,
            A11yPublicationSkip as Skip,
        };
        let skip = if !self.s.a11y_owner.get() {
            Some(Skip::OwnerEnded)
        } else if self.s.a11y.is_none() {
            Some(Skip::NoAdapter)
        } else if self.s.runner.is_none() {
            Some(Skip::NoRunner)
        } else if self.s.layout.is_none() {
            Some(Skip::NoLayout)
        } else {
            None
        };
        if let Some(reason) = skip {
            let adapter = self
                .s
                .a11y
                .as_ref()
                .map_or(Adapter::NotConfigured, |a11y| a11y.adapter_state());
            self.record_a11y_publication(Publication::skipped(adapter, reason));
            return Vec::new();
        }
        // A screen reader is told physical client pixels, and the tree it is
        // told them about is laid out in layout pixels: the transform between
        // them is the layout scale, zoom included. A reader that read the
        // device scale alone would point at a control's unzoomed position.
        let layout_scale = self.layout_scale();
        let requests = {
            let (dom, mount, focus) = match self.s.runner.as_ref() {
                Some(runner) => (runner.dom(), runner.mount(), runner.focus()),
                None => return Vec::new(),
            };
            let dom_ref = dom.borrow();
            let view = crate::WindowDom::new(&dom_ref, mount);
            // Focus can change without hover; last_focus belongs to restyling,
            // not to the accessibility adapter's current focus report.
            let focus = focus.map(|node| view.opaque_id(node));
            let (Some(a11y), Some(layout)) = (self.s.a11y.as_mut(), self.s.layout.as_ref()) else {
                return Vec::new();
            };
            // The window is the adapter's own now, so the seam does not carry it.
            a11y.sync(
                &crate::A11yOwner::new(&self.s.a11y_owner),
                &view,
                layout,
                &mut self.s.shared.leaves,
                &mut self.s.shared.producers,
                focus,
                layout_scale,
            )
        };
        self.record_a11y_publication(requests.publication);
        self.apply_a11y_requests(&requests.requests)
    }

    /// Route drained screen-reader requests into the retained DOM. Split out of
    /// [`sync_a11y`](Self::sync_a11y) so the routing is exercisable without an
    /// OS adapter.
    /// Results correspond to input order; dispatch is not terminal completion.
    pub fn apply_a11y_requests(
        &mut self,
        requests: &[crate::A11yRequest],
    ) -> Vec<crate::A11yDispatch> {
        use crate::A11yDispatch;
        let mut results = Vec::with_capacity(requests.len());
        let mut dispatched = false;
        for request in requests {
            // Raw calls name this operation's entry context. Freeze once before
            // contributor callbacks; bound publications retain their original life.
            let mut request = request.clone();
            if let Some(runner) = self.s.runner.as_ref() {
                let dom = runner.dom();
                let dom = dom.borrow();
                request = request.bind_publication(
                    &crate::A11yOwner::new(&self.s.a11y_owner),
                    &crate::WindowDom::new(&dom, runner.mount()),
                    &self.s.shared.leaves,
                    &self.s.shared.producers,
                );
            }
            // Recompute against the current DOM for each request: a previous
            // callback in this batch may have removed or disabled its target.
            let node = match self.admit_a11y_request(&request) {
                Ok(node) => node,
                Err(reason) => {
                    results.push(A11yDispatch::Rejected(reason));
                    continue;
                },
            };
            let runner = self.s.runner.as_mut().expect("admission requires a runner");
            let result = match &request.target {
                crate::A11yTarget::Produced(produced) => {
                    if request.action == A11yAction::Click {
                        match self.s.shared.producers.act_checked(
                            produced,
                            request.provenance.as_ref().and_then(|p| p.child()),
                        ) {
                            crate::ProducerActionDispatch::Dispatched => {
                                runner.set_focus(Some(node));
                                A11yDispatch::Dispatched
                            },
                            crate::ProducerActionDispatch::Declined => {
                                A11yDispatch::ProducerDeclined
                            },
                            crate::ProducerActionDispatch::StaleTarget => {
                                A11yDispatch::Rejected(crate::A11yRejection::StaleTarget)
                            },
                        }
                    } else {
                        runner.set_focus(Some(node));
                        A11yDispatch::Dispatched
                    }
                },
                crate::A11yTarget::Node(_) => {
                    match request.action {
                        A11yAction::Click => {
                            runner.dispatch_click(node, PointerClick::at((0.0, 0.0)));
                        },
                        A11yAction::Focus => runner.set_focus(Some(node)),
                        A11yAction::SetValue(value) => {
                            runner.dispatch_value(node, cambium::ValueEvent { value });
                        },
                    }
                    A11yDispatch::Dispatched
                },
            };
            dispatched |= result == A11yDispatch::Dispatched;
            results.push(result);
        }
        if dispatched {
            self.hover();
            self.after_dispatch();
        }
        results
    }

    /// Admission belongs to the host, not to a platform's last-synced ID map.
    /// Layout visibility is the retained host view. Bound publications also
    /// check host owner, document/mount and contributor lives. Raw requests name
    /// the current context. Neither form establishes an atomic presentation transaction.
    fn admit_a11y_request(
        &mut self,
        request: &crate::A11yRequest,
    ) -> Result<crate::NodeId, crate::A11yRejection> {
        use crate::A11yRejection as Refusal;
        use layout_dom_api::LayoutDom as _;
        if !self.s.a11y_owner.get() {
            return Err(Refusal::Cancelled);
        }
        if matches!(request.action, A11yAction::SetValue(value) if !value.is_finite()) {
            return Err(Refusal::InvalidValue);
        }
        let (Some(runner), Some(layout)) = (self.s.runner.as_ref(), self.s.layout.as_ref()) else {
            return Err(Refusal::Unavailable);
        };
        let dom = runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, runner.mount());
        if !request.publication_is_current(
            &crate::A11yOwner::new(&self.s.a11y_owner),
            &view,
            &self.s.shared.leaves,
            &self.s.shared.producers,
        ) {
            return Err(Refusal::StaleTarget);
        }
        if let crate::A11yTarget::Node(node) = request.target {
            if !view.contains(node) {
                return Err(Refusal::TargetAbsent);
            }
        }
        let projection = crate::project_semantics(
            &view,
            layout,
            &mut self.s.shared.leaves,
            &mut self.s.shared.producers,
            runner.focus().map(|node| view.opaque_id(node)),
        );
        let node = match &request.target {
            crate::A11yTarget::Node(node) => *node,
            crate::A11yTarget::Produced(produced) => *projection
                .producer_slots
                .get(&produced.slot)
                .ok_or(Refusal::TargetAbsent)?,
        };
        let id = accesskit::NodeId(view.opaque_id(node));
        if !projection.dom_targets.contains_key(&id) {
            return Err(Refusal::TargetAbsent);
        }
        let semantic = projection
            .tree
            .nodes
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, node)| node)
            .filter(|node| !node.is_hidden())
            .ok_or(Refusal::TargetAbsent)?;
        if semantic.is_disabled() {
            return Err(Refusal::Disabled);
        }
        let action = match request.action {
            A11yAction::Click => accesskit::Action::Click,
            A11yAction::Focus => accesskit::Action::Focus,
            A11yAction::SetValue(_) => accesskit::Action::SetValue,
        };
        let semantic = if let crate::A11yTarget::Produced(produced) = &request.target {
            let id = projection
                .produced_targets
                .iter()
                .find_map(|(id, target)| (target == produced).then_some(*id))
                .ok_or(Refusal::Unsupported)?;
            projection
                .tree
                .nodes
                .iter()
                .find(|(candidate, _)| *candidate == id)
                .map(|(_, node)| node)
                .ok_or(Refusal::TargetAbsent)?
        } else {
            semantic
        };
        if !semantic.supports_action(action) {
            return Err(Refusal::Unsupported);
        }
        // A contributor callback is required to be observational. Recheck its
        // authored life even if interior mutation violated that contract.
        if !request.publication_is_current(
            &crate::A11yOwner::new(&self.s.a11y_owner),
            &view,
            &self.s.shared.leaves,
            &self.s.shared.producers,
        ) {
            return Err(Refusal::StaleTarget);
        }
        Ok(node)
    }

    /// Drive `:hover` / `:focus` restyles on target change. The retained layout
    /// skips the cascade when no rule can depend on that state, and skips
    /// text layout when the cascade leaves the computed styles unchanged.
    pub fn hover(&mut self) {
        let (Some(runner), Some(layout)) = (self.s.runner.as_ref(), self.s.layout.as_mut()) else {
            return;
        };
        let (x, y) = self.s.cursor;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let hovered_node = layout.hit_test(&view, x, y);
        let focused_node = runner.focus();
        let hovered = hovered_node.map(|n| layout_dom_api::LayoutDom::opaque_id(&view, n));
        let focused = focused_node.map(|n| layout_dom_api::LayoutDom::opaque_id(&view, n));
        if (hovered, focused) == (self.s.last_hover, self.s.last_focus) {
            return;
        }
        self.s.last_hover = hovered;
        self.s.last_focus = focused;
        // Bring the transition clock to now before the flip, so a
        // hover/focus transition runs from now rather than a stale
        // idle-frozen clock.
        let now_s = self.s.anim_base.elapsed().as_secs_f64();
        let _ = layout.tick_animations(&view, now_s);
        if layout.set_interaction(&view, hovered_node, focused_node) {
            drop(dom_ref);
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    /// Route Cambium `on_hover` Enter/Leave as the hit node changes. The host
    /// owns transition detection. Registered hover handlers also receive local
    /// coordinates on Move; motion over unregistered content does no dispatch.
    pub fn hover_dispatch(&mut self) {
        use cambium::{HoverEvent, HoverPhase};
        let target = self
            .hit_at_cursor()
            .and_then(|hit| self.s.runner.as_ref()?.hover_target(hit));
        if target == self.s.last_hover_hit {
            if let Some(target) = target {
                let (local, size) = self.local_in(target);
                if let Some(runner) = self.s.runner.as_mut() {
                    runner.dispatch_hover(target, HoverEvent::new(HoverPhase::Move, local, size));
                }
                self.after_dispatch();
            }
            return;
        }
        let old = self.s.last_hover_hit.take();
        self.s.last_hover_hit = target;
        for (node, phase) in [(old, HoverPhase::Leave), (target, HoverPhase::Enter)] {
            if let Some(node) = node {
                // A view replacement can retire the cached Leave target; its
                // handler can also retire the prospective Enter target. Neither
                // has geometry or an event route in the current document.
                let live = self
                    .s
                    .runner
                    .as_ref()
                    .is_some_and(|runner| runner.dom().borrow().is_live(node));
                if !live {
                    if self.s.last_hover_hit == Some(node) {
                        self.s.last_hover_hit = None;
                    }
                    continue;
                }
                let (local, size) = self.local_in(node);
                if let Some(runner) = self.s.runner.as_mut() {
                    runner.dispatch_hover(node, HoverEvent::new(phase, local, size));
                }
            }
        }
        self.after_dispatch();
    }
}
