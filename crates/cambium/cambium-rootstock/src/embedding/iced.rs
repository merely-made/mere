// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Iced wgpu30 Primitive adapter. Released Iced 0.14 uses wgpu27 and cannot
//! share this stack's GPU handles. This adapter targets the exact wgpu30 supplier
//! pinned by the feature manifest. Iced keeps its encoder and submission.
pub use super::iced_input::Input;
use super::{Frame, Gpu, TargetConfig};
pub use iced_wgpu as backend;
use iced_wgpu::{core::Rectangle, graphics::Viewport, primitive};
use netrender::vello_backends::SparseResourceLimits;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
struct State {
    gpu: Option<Gpu>,
    epoch: u64,
    generation: u64,
    appearance: Option<u64>,
    disposed: bool,
    error: Option<String>,
}
#[derive(Clone)]
pub struct Adapter {
    state: Arc<Mutex<State>>,
    limits: SparseResourceLimits,
}
impl Adapter {
    pub fn new(limits: SparseResourceLimits, generation: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                generation,
                ..State::default()
            })),
            limits,
        }
    }
    pub fn primitive(&self, frame: Frame) -> Primitive {
        let epoch = self.state.lock().expect("embedding adapter lock").epoch;
        Primitive {
            adapter: self.clone(),
            frame,
            epoch,
            prepared: AtomicBool::new(false),
        }
    }
    pub fn device_recovered(&self, generation: u64) {
        self.suspend();
        self.state
            .lock()
            .expect("embedding adapter lock")
            .generation = generation;
    }
    pub fn suspend(&self) {
        let mut state = self.state.lock().expect("embedding adapter lock");
        state.gpu = None;
        state.epoch = state
            .epoch
            .checked_add(1)
            .expect("embedding epoch exhausted");
    }
    pub fn dispose(&self) {
        self.suspend();
        self.state.lock().expect("embedding adapter lock").disposed = true;
    }
    pub fn take_error(&self) -> Option<String> {
        self.state
            .lock()
            .expect("embedding adapter lock")
            .error
            .take()
    }
}
pub struct Pipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
}
impl primitive::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self {
            device: device.clone(),
            queue: queue.clone(),
            format,
        }
    }
}
pub struct Primitive {
    adapter: Adapter,
    frame: Frame,
    epoch: u64,
    prepared: AtomicBool,
}
impl std::fmt::Debug for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MerePrimitive")
            .field("appearance", &self.frame.appearance)
            .field("epoch", &self.epoch)
            .finish()
    }
}
impl primitive::Primitive for Primitive {
    type Pipeline = Pipeline;
    fn prepare(
        &self,
        _pipeline: &mut Pipeline,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let b = self.frame.placement.bounds;
        self.prepared.store(false, Ordering::Release);
        if [bounds.x, bounds.y, bounds.width, bounds.height] != b
            || (viewport.scale_factor() as f32 - self.frame.placement.scale).abs() > 0.001
        {
            let mut state = self.adapter.state.lock().expect("embedding adapter lock");
            state.error = Some("Iced bounds/scale differ from captured appearance".into());
            state.gpu = None;
        } else {
            self.prepared.store(true, Ordering::Release);
        }
    }
    // Sparse scene work needs the caller encoder, absent from prepare/draw.
    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip: &Rectangle<u32>,
    ) {
        let mut state = self.adapter.state.lock().expect("embedding adapter lock");
        if state.disposed || state.epoch != self.epoch || !self.prepared.load(Ordering::Acquire) {
            return;
        }
        if state
            .appearance
            .is_some_and(|id| id != self.frame.appearance)
        {
            state.error = Some("Iced adapter is bound to another appearance".into());
            state.gpu = None;
            return;
        }
        state.appearance = Some(self.frame.appearance);
        let outcome = (|| {
            if state.gpu.is_none() {
                state.gpu = Some(Gpu::new(
                    &pipeline.device,
                    &pipeline.queue,
                    state.generation,
                    TargetConfig {
                        format: pipeline.format,
                        sample_count: 1,
                    },
                    self.adapter.limits,
                )?);
            }
            state.gpu.as_mut().unwrap().prepare(
                &self.frame,
                &pipeline.device,
                &pipeline.queue,
                encoder,
            )
        })();
        if let Err(error) = outcome {
            state.error = Some(format!("{error:?}"));
            state.gpu = None;
            return;
        }
        let b = self.frame.placement.bounds;
        let s = self.frame.placement.scale;
        let c = self.frame.placement.clip;
        let x = clip.x.max((c[0].max(b[0]) * s).max(0.).round() as u32);
        let y = clip.y.max((c[1].max(b[1]) * s).max(0.).round() as u32);
        let right =
            (clip.x + clip.width).min(((c[0] + c[2]).min(b[0] + b[2]) * s).max(0.).round() as u32);
        let bottom =
            (clip.y + clip.height).min(((c[1] + c[3]).min(b[1] + b[3]) * s).max(0.).round() as u32);
        if right <= x || bottom <= y {
            return;
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mere Iced embedding"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_viewport(b[0] * s, b[1] * s, b[2] * s, b[3] * s, 0., 1.);
        pass.set_scissor_rect(x, y, right - x, bottom - y);
        state.gpu.as_ref().unwrap().paint(&mut pass);
    }
}
