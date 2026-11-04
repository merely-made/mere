// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> HostState<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    pub fn new() -> Self {
        Self {
            window: None,
            titlebar_published: None,
            titlebar_sheet: String::new(),
            surface: None,
            presentation_driver: None,
            recovery_status: None,
            last_presentation_failure: None,
            last_raster_failure: None,
            recovery_subscription: None,
            detached_generation: None,
            needs_recovery_surface: false,
            runner: None,
            layout: None,
            layout_size: (0.0, 0.0),
            user_zoom: 1.0,
            fit_factor: 1.0,
            zoom_seen: 1.0,
            surface_size: None,
            last_layout_update_us: 0,
            last_layout_tick_us: 0,
            last_layout_apply_us: 0,
            last_layout_rebuild_us: 0,
            last_style_resolve_us: 0,
            last_layout_with_text_us: 0,
            last_content_extent_us: 0,
            last_layout_mutations: 0,
            last_layout_rebuilt: false,
            last_leaf_boxes_us: 0,
            last_leaf_render_us: 0,
            last_leaf_repaints: 0,
            shared: AppShared::default(),
            layout_sheet_generation: 0,
            leaf_keys: Vec::new(),
            cursor: (0.0, 0.0),
            pointer_id: 0,
            modifiers: Modifiers::NONE,
            text_drag: None,
            last_hover: None,
            last_focus: None,
            last_hover_hit: None,
            anim_base: crate::Instant::now(),
            a11y: None,
            a11y_publication_observer: None,
            a11y_publication: crate::A11yPublication::default(),
            a11y_publication_sequence: 0,
            files: None,
            file_answers: Default::default(),
            a11y_wake: Arc::new(AtomicBool::new(false)),
            hidden: false,
            commands: WindowCommands::new(),
            geometry: None,
            wake_pending: Arc::new(AtomicBool::new(false)),
            scrollbar_fade: ScrollbarFade::new(),
            close_requested: false,
            pending_sheet: None,
            pending_ui_zoom: None,
            pending_capture: None,
            capture_owner: crate::CaptureOwner::default(),
            pending_presentation_observer: None,
            pending_stamped_capture: None,
            presentation_host: next_host_identity(),
            presentation_sequence: 0,
            last_redraw_presentation: None,
            pending_paint_capture: None,
            pending_pointer: Vec::new(),
            pending_a11y: Vec::new(),
            a11y_owner: Rc::new(std::cell::Cell::new(true)),
            draining_a11y: false,
            pending_scroll: Vec::new(),
            caret_followed: None,
            last_frame_profile: None,
            tab_held: false,
        }
    }

    /// Store the bundled resources an [`Init`] carried. One call so every
    /// event source lands them the same way; the image list becomes the
    /// URL-keyed ledger Livery resolves against.
    pub fn set_resources(&mut self, fonts: Vec<HostFont>, images: Vec<HostImage>) {
        self.shared.fonts = fonts;
        self.shared.images = images
            .into_iter()
            .map(|image| (image.url, image.bytes))
            .collect();
    }
}

impl Drop for HostDisposal {
    fn drop(&mut self) {
        // Registry cleanup uses its weak core while the detached core is live.
        // Release all owned GPU resources before opening the detach barrier.
        drop(self.producers.take());
        let fragments = std::mem::take(&mut self.fragments);
        if let Some(core) = &self.core {
            for &(id, _) in fragments.values() {
                crate::retire_fragment(core, id);
            }
        }
        drop(fragments);
        // An installed driver owns its context/surface and outstanding output.
        if let Some(mut driver) = self.driver.take() {
            driver.dispose();
        }
        drop(self.surface.take());
        drop(self.core.take());
        drop(self.subscription.take());
        drop(self.owner.take());
        if let Some(capture) = self.capture.take() {
            capture(Err(CaptureRefusal::HostDisposed));
        }
        if let Some(capture) = self.stamped.take() {
            capture(Err(CaptureRefusal::HostDisposed));
        }
        if let Some(capture) = self.paint.take() {
            capture(Err(CaptureRefusal::HostDisposed));
        }
        if let (Some(mut observer), Some(notice)) =
            (self.a11y_observer.take(), self.a11y_notice.take())
        {
            observer(notice);
        }
    }
}

impl Default for AppShared {
    fn default() -> Self {
        Self {
            render_core: None,
            render_recovery: None,
            sheet: String::new(),
            fonts: Vec::new(),
            images: std::collections::HashMap::new(),
            leaves: sprigging::LeafRegistry::new(),
            producers: crate::ProducerRegistry::new(),
            rendered: sprigging::RenderedLeaves::new(),
            leaf_fragments: std::collections::HashMap::new(),
            sheet_generation: 0,
            held_elsewhere: std::collections::HashSet::new(),
        }
    }
}
