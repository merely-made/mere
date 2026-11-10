// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl OwnedLayout {
    /// A retained session over the host's resource ledger: font faces to
    /// register into its text system, and image bytes to resolve `url()`
    /// against. The session is rebuilt whenever the ledger changes, so
    /// registration cannot outlive a face the host withdrew.
    ///
    /// Public so a host that is not rootstock can lay out a composed
    /// session's own document and project it for accessibility
    /// (`cambium_winit_a11y::project_tree`), as experiment E1a does.
    pub fn new<D: LayoutDom<NodeId = NodeId>>(
        dom: &D,
        sheets: &[&str],
        width: f32,
        height: f32,
        fonts: &[crate::HostFont],
        images: &HashMap<String, Vec<u8>>,
    ) -> Self {
        Self::with_vectors(dom, sheets, width, height, fonts, images, None)
    }

    /// [`new`](Self::new) with the application's vector images, resolved
    /// beside the image bytes.
    pub fn with_vectors<D: LayoutDom<NodeId = NodeId>>(
        dom: &D,
        sheets: &[&str],
        width: f32,
        height: f32,
        fonts: &[crate::HostFont],
        images: &HashMap<String, Vec<u8>>,
        vectors: Option<HostVectors>,
    ) -> Self {
        let style_set = StyleSet::cambium(sheets);
        let interaction_dependencies = interaction::Dependencies::new(&style_set);
        let device = Device::screen(width, height);
        let interactions = InteractionStates::default();
        let phase = crate::Instant::now();
        let mut style = IncrementalStyle::new();
        style.update(dom, &style_set, &device, &interactions, &[]);
        let style_resolve_us = elapsed_us(phase.elapsed());
        let mut text = TextSystem::new();
        let font_registration_errors = register_fonts(&mut text, fonts);
        let phase = crate::Instant::now();
        let (styles, fragments) = layout_with_text_system(
            dom,
            style.styles(),
            width,
            height,
            ViewportSizes::uniform(width, height),
            &mut text,
            &HostImageLookup {
                bytes: images,
                vectors: vectors.as_ref(),
            },
        )
        .expect("Cambium's authored Livery layout must resolve");
        let layout_with_text_us = elapsed_us(phase.elapsed());
        let phase = crate::Instant::now();
        let content_extent = content_extent(dom, &styles, &fragments);
        let content_extent_us = elapsed_us(phase.elapsed());
        Self {
            style_set,
            interaction_dependencies,
            device,
            interactions,
            hovered: None,
            focused: None,
            style,
            styles,
            fragments,
            text,
            font_registration_errors,
            viewport: (width, height),
            viewport_scroll: (0.0, 0.0),
            element_scroll: HashMap::new(),
            content_extent,
            images: images.clone(),
            vectors,
            style_resolve_us,
            layout_with_text_us,
            content_extent_us,
            generation: 1,
            input_lifetime: std::sync::Arc::new(()),
        }
    }

    /// Bring the session current with `dom`. `mutations` is every change made
    /// since the last `new` or `rebuild`: the cascade re-resolves only what
    /// they and the device could have touched. A stylesheet change is not a
    /// mutation; the host builds a fresh session for it. Layout is full.
    pub(crate) fn rebuild<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        width: f32,
        height: f32,
        mutations: &[DomMutation<NodeId>],
    ) {
        self.viewport = (width, height);
        self.device.set_viewport_size(width, height);
        self.update_style(dom, mutations);
        self.layout_resolved(dom);
    }

    pub(super) fn update_style<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        mutations: &[DomMutation<NodeId>],
    ) {
        let phase = crate::Instant::now();
        self.style.update(
            dom,
            &self.style_set,
            &self.device,
            &self.interactions,
            mutations,
        );
        self.style_resolve_us = elapsed_us(phase.elapsed());
    }

    pub(super) fn layout_resolved<D: LayoutDom<NodeId = NodeId>>(&mut self, dom: &D) {
        let (width, height) = self.viewport;
        let phase = crate::Instant::now();
        let (styles, fragments) = layout_with_text_system(
            dom,
            self.style.styles(),
            width,
            height,
            self.device.viewport_sizes,
            &mut self.text,
            &HostImageLookup {
                bytes: &self.images,
                vectors: self.vectors.as_ref(),
            },
        )
        .expect("Cambium's authored Livery layout must resolve");
        self.layout_with_text_us = elapsed_us(phase.elapsed());
        self.styles = styles;
        self.fragments = fragments;
        let phase = crate::Instant::now();
        self.content_extent = content_extent(dom, &self.styles, &self.fragments);
        self.content_extent_us = elapsed_us(phase.elapsed());
        self.clamp_viewport_scroll();
        self.clamp_element_scroll(dom);
        self.generation = self.generation.saturating_add(1);
    }

    pub(crate) fn stage_timings(&self) -> (u64, u64, u64) {
        (
            self.style_resolve_us,
            self.layout_with_text_us,
            self.content_extent_us,
        )
    }

    pub fn fragments(&self) -> &LiveryLayout<NodeId> {
        &self.fragments
    }

    /// Inline generated name text resolved by this retained layout's style owner.
    pub fn generated_text<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> (String, String) {
        genet_livery::rendered_generated_text(dom, &self.styles, node)
    }

    /// Whether `node` is rendered and visible under this layout's styles: no
    /// `display: none` above it and computed `visibility: visible`.
    pub fn rendered_visible<D: LayoutDom<NodeId = NodeId>>(&self, dom: &D, node: NodeId) -> bool {
        genet_livery::rendered_visible(dom, &self.styles, node)
    }

    /// Font instances this session's text system has materialised. The
    /// observable end of the host font seam.
    pub fn retained_font_count(&self) -> usize {
        self.text.retained_font_count()
    }

    /// Typed refusals from explicit generic resource registration.
    pub fn font_registration_errors(&self) -> &[genet_livery::GenericFontError] {
        &self.font_registration_errors
    }

    pub(crate) fn has_active_animations(&self) -> bool {
        false
    }

    pub(crate) fn tick_animations<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        _dom: &D,
        _now: f64,
    ) -> bool {
        false
    }

    /// Draw the application's vector images at a new layout scale. Layout is
    /// unchanged: an image's box comes from its CSS, not its detail.
    pub(crate) fn set_vector_scale(&mut self, scale: f32) {
        if let Some(vectors) = self.vectors.as_mut() {
            vectors.scale = scale;
        }
    }
}
