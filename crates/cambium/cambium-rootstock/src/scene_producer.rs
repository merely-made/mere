// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained application scenes on the host's existing renderer and device.

use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    PortableProducerError, ProducedScene, ProducedTexture, ProducerContext, ProducerFrameInfo,
    ProducerSemantics, SourceAlpha, SourceEncoding, TextureProducer,
};
use netrender::{ColorLoad, Scene};

static NEXT_SOURCE_IDENTITY: AtomicU64 = AtomicU64::new(1);

fn source_identity() -> u64 {
    NEXT_SOURCE_IDENTITY
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .expect("scene producer source identity exhausted")
}

/// Rasterize an application's scene through its host's render core. The
/// application supplies its existing scene, revision and accessibility APIs;
/// this adapter owns only the retained GPU output and producer lifecycle.
///
/// Give each independently rendered source a distinct raster key. A source
/// revision must change whenever its visible content changes; resizing and
/// the host's `needs_frame` flag also force rasterization. Callback functions
/// operate on the currently bound source, including after a library/session
/// reopen replaces an otherwise identically revisioned source.
pub struct SceneProducer<T> {
    source: Rc<RefCell<T>>,
    source_identity: u64,
    raster_key: u64,
    frame_callback: fn(&mut T, u32, u32) -> Scene,
    revision_callback: fn(&T) -> u64,
    semantics_callback: fn(&T) -> Option<ProducerSemantics>,
    texture: Option<wgpu::Texture>,
    physical_size: [u32; 2],
    logical_size: (u32, u32),
    layout_scale: f32,
    revision: u64,
    generation: u64,
    prepared: Option<PreparedSceneTexture>,
}

struct PreparedSceneTexture {
    texture: wgpu::Texture,
    physical_size: [u32; 2],
    logical_size: (u32, u32),
    layout_scale: f32,
    source_identity: u64,
    revision: u64,
    generation: u64,
}

impl<T> SceneProducer<T> {
    pub fn new(
        source: Rc<RefCell<T>>,
        raster_key: u64,
        frame: fn(&mut T, u32, u32) -> Scene,
        revision: fn(&T) -> u64,
        semantics: fn(&T) -> Option<ProducerSemantics>,
    ) -> Self {
        Self {
            source,
            source_identity: source_identity(),
            raster_key,
            frame_callback: frame,
            revision_callback: revision,
            semantics_callback: semantics,
            texture: None,
            physical_size: [0, 0],
            logical_size: (0, 0),
            layout_scale: 0.0,
            revision: 0,
            generation: 0,
            prepared: None,
        }
    }

    /// Retain the existing output when its source handle is unchanged. A new
    /// source identity invalidates it even if the revision numbers coincide.
    pub fn set_source(&mut self, source: Rc<RefCell<T>>) -> bool {
        if Rc::ptr_eq(&self.source, &source) {
            return false;
        }
        self.source = source;
        self.prepared = None;
        self.source_identity = source_identity();
        self.revision = 0;
        self.texture = None;
        true
    }

    fn source_revision(&self) -> u64 {
        (self.revision_callback)(&self.source.borrow())
    }

    fn scene(&mut self, width: u32, height: u32) -> Scene {
        (self.frame_callback)(&mut self.source.borrow_mut(), width, height)
    }
}

impl<T> TextureProducer for SceneProducer<T> {
    fn prepare_checked(
        &mut self,
        cx: &ProducerContext<'_>,
        #[cfg(target_arch = "wasm32")] commands: &genet_render_host::BrowserFrameCommands<'_>,
    ) -> Result<Option<ProducedTexture>, genet_render_host::RasterizeFailure> {
        self.prepared = None;
        let logical_size = (
            cx.frame.logical_size.0.round().max(1.0) as u32,
            cx.frame.logical_size.1.round().max(1.0) as u32,
        );
        if !cx.frame.needs_frame && self.texture.is_some()
            && self.revision == self.source_revision()
            && self.physical_size == cx.frame.physical_size
            && self.logical_size == logical_size
            && self.layout_scale == cx.frame.layout_scale
        { return Ok(None); }
        let generation = self.generation.checked_add(1)
            .ok_or_else(|| crate::producer::staged_producer_refusal("scene producer generation exhausted"))?;
        let scene = self.scene(logical_size.0, logical_size.1);
        let revision = self.source_revision();
        let [width, height] = cx.frame.physical_size;
        #[cfg(not(target_arch = "wasm32"))]
        let raster = cx.core.try_rasterize_scaled_for(
            self.raster_key, &scene, width, height,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT), cx.frame.layout_scale,
        );
        #[cfg(target_arch = "wasm32")]
        let raster = commands.rasterize_scaled_for(
            self.raster_key, &scene, width, height,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT), cx.frame.layout_scale,
        );
        let (texture, view) = raster?;
        self.prepared = Some(PreparedSceneTexture {
            texture, physical_size:cx.frame.physical_size, logical_size,
            layout_scale:cx.frame.layout_scale, source_identity:self.source_identity,
            revision, generation,
        });
        Ok(Some(ProducedTexture { view, generation, alpha:SourceAlpha::Straight, encoding:SourceEncoding::Srgb }))
    }

    fn supports_deferred_acceptance(&self) -> bool { true }

    fn prepared_is_current(&self) -> bool {
        match &self.prepared {
            Some(prepared) => prepared.source_identity == self.source_identity && prepared.revision == self.source_revision(),
            None => self.texture.is_some() && self.revision == self.source_revision(),
        }
    }

    fn commit_prepared(&mut self) {
        if let Some(prepared) = self.prepared.take() {
            self.texture = Some(prepared.texture);
            self.physical_size = prepared.physical_size;
            self.logical_size = prepared.logical_size;
            self.layout_scale = prepared.layout_scale;
            self.revision = prepared.revision;
            self.generation = prepared.generation;
        }
    }

    fn commit_prepared_retiring(&mut self) -> Option<Box<dyn std::any::Any>> {
        if self.prepared.is_none() { return None; }
        let retired = self.texture.take();
        self.commit_prepared();
        retired.map(|texture| Box::new(texture) as Box<dyn std::any::Any>)
    }

    fn abort_prepared(&mut self) { self.prepared = None; }

    fn take_prepared(&mut self) -> Option<Box<dyn std::any::Any>> {
        self.prepared.take().map(|prepared| Box::new(prepared) as Box<dyn std::any::Any>)
    }

    fn take_suspended_resources(&mut self) -> Option<Box<dyn std::any::Any>> {
        let resources = (self.texture.take(),self.prepared.take());
        Some(Box::new(resources))
    }

    fn portable_scene(
        &mut self,
        frame: &ProducerFrameInfo,
    ) -> Result<ProducedScene, PortableProducerError> {
        let width = frame.logical_size.0.round().max(1.0) as u32;
        let height = frame.logical_size.1.round().max(1.0) as u32;
        let scene = self.scene(width, height);
        Ok(ProducedScene {
            scene,
            source_identity: self.source_identity,
            source_revision: self.source_revision(),
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        })
    }

    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        // Legacy direct callers keep their unit/Option contract. Rootstock
        // uses render_checked so a refusal cannot retain a stale image.
        self.render_checked(cx).ok().flatten()
    }

    fn render_checked(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, genet_render_host::RasterizeFailure> {
        let logical_size = (
            cx.frame.logical_size.0.round().max(1.0) as u32,
            cx.frame.logical_size.1.round().max(1.0) as u32,
        );
        if !cx.frame.needs_frame
            && self.texture.is_some()
            && self.revision == self.source_revision()
            && self.physical_size == cx.frame.physical_size
            && self.logical_size == logical_size
            && self.layout_scale == cx.frame.layout_scale
        {
            return Ok(None);
        }
        let scene = self.scene(logical_size.0, logical_size.1);
        let [width, height] = cx.frame.physical_size;
        let (texture, view) = cx.core.try_rasterize_scaled_for(
            self.raster_key,
            &scene,
            width,
            height,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            cx.frame.layout_scale,
        )?;
        self.texture = Some(texture);
        self.physical_size = cx.frame.physical_size;
        self.logical_size = logical_size;
        self.layout_scale = cx.frame.layout_scale;
        self.revision = self.source_revision();
        self.generation = self
            .generation
            .checked_add(1)
            .expect("scene producer generation exhausted");
        Ok(Some(ProducedTexture {
            view,
            generation: self.generation,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        }))
    }

    fn suspend(&mut self) {
        self.prepared = None;
        self.texture = None;
    }

    fn semantics(&mut self) -> Option<ProducerSemantics> {
        (self.semantics_callback)(&self.source.borrow())
    }
}

#[cfg(test)]
mod tests {
    use crate::ProducerRole;

    use super::*;

    struct Source {
        name: String,
        revision: u64,
        frames: Vec<(u32, u32)>,
    }

    fn source(name: &str) -> Rc<RefCell<Source>> {
        Rc::new(RefCell::new(Source {
            name: name.to_owned(),
            revision: 7,
            frames: Vec::new(),
        }))
    }

    fn frame(source: &mut Source, width: u32, height: u32) -> Scene {
        source.frames.push((width, height));
        Scene::new(width, height)
    }

    fn revision(source: &Source) -> u64 {
        source.revision
    }

    fn semantics(source: &Source) -> Option<ProducerSemantics> {
        Some(ProducerSemantics {
            role: Some(ProducerRole::Image),
            name: Some(source.name.clone()),
            children: Vec::new(),
        })
    }

    #[test]
    fn rebinding_redirects_all_callbacks_even_when_revisions_coincide() {
        let original = source("Original document");
        let replacement = source("Reopened document");
        let mut producer = SceneProducer::new(original.clone(), 81, frame, revision, semantics);
        producer.scene(320, 240);
        producer.revision = producer.source_revision();
        producer.generation = 11;

        assert!(!producer.set_source(original.clone()));
        assert_eq!(producer.revision, 7);
        assert_eq!(producer.generation, 11);
        assert!(producer.set_source(replacement.clone()));
        assert_eq!(
            producer.revision, 0,
            "matching revisions must not preserve old source output"
        );
        assert_eq!(
            producer.generation, 11,
            "output generations remain monotonic across rebinds"
        );
        assert_eq!(producer.source_revision(), 7);
        assert_eq!(
            producer.semantics().unwrap().name.as_deref(),
            Some("Reopened document")
        );
        producer.scene(640, 480);
        assert_eq!(original.borrow().frames, [(320, 240)]);
        assert_eq!(replacement.borrow().frames, [(640, 480)]);

        replacement.borrow_mut().revision = 8;
        replacement.borrow_mut().name = "Updated document".into();
        assert_eq!(producer.source_revision(), 8);
        assert_eq!(
            producer.semantics().unwrap().name.as_deref(),
            Some("Updated document")
        );
    }

    #[test]
    fn an_absent_semantic_projection_keeps_the_host_dom_semantics() {
        let mut producer =
            SceneProducer::new(source("DOM owns the name"), 82, frame, revision, |_| None);
        assert_eq!(producer.semantics(), None);
    }
}
