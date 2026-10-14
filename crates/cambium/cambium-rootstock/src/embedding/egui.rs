// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Native egui-wgpu 0.36.2/wgpu 30 callback. The UI owner captures a Frame;
//! egui owns command encoding/submission and sets viewport/scissor at painting.
pub use super::egui_input::Input;
use super::{Frame, Gpu, TargetConfig};
pub use ::egui as toolkit;
pub use egui_wgpu as backend;
pub use epaint as paint;
use netrender::vello_backends::SparseResourceLimits;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct State {
    gpu: Option<Gpu>,
    epoch: u64,
    generation: u64,
    appearance: Option<u64>,
    disposed: bool,
    error: Option<String>,
}
/// One adapter per appearance. Clone shares only that appearance's GPU lifetime.
#[derive(Clone)]
pub struct Adapter {
    state: Arc<Mutex<State>>,
    config: TargetConfig,
    limits: SparseResourceLimits,
}
impl Adapter {
    pub fn new(config: TargetConfig, limits: SparseResourceLimits, generation: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                generation,
                ..State::default()
            })),
            config,
            limits,
        }
    }
    pub fn callback(&self, frame: Frame) -> epaint::PaintCallback {
        let b = frame.placement.bounds;
        let rect = epaint::emath::Rect::from_min_size(
            epaint::emath::pos2(b[0], b[1]),
            epaint::emath::vec2(b[2], b[3]),
        );
        let epoch = self.state.lock().expect("embedding adapter lock").epoch;
        egui_wgpu::Callback::new_paint_callback(
            rect,
            Callback {
                adapter: self.clone(),
                frame,
                epoch,
            },
        )
    }
    /// Called after the host's device recovery detachment barrier. Old queued
    /// callbacks become stale; fresh callbacks lazily use the new host handles.
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
struct Callback {
    adapter: Adapter,
    frame: Frame,
    epoch: u64,
}
impl egui_wgpu::CallbackTrait for Callback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut state = self.adapter.state.lock().expect("embedding adapter lock");
        if state.disposed || state.epoch != self.epoch {
            return vec![];
        }
        if state
            .appearance
            .is_some_and(|id| id != self.frame.appearance)
        {
            state.error = Some("egui adapter is bound to another appearance".into());
            state.gpu = None;
            return vec![];
        }
        state.appearance = Some(self.frame.appearance);
        if (screen.pixels_per_point - self.frame.placement.scale).abs() > 0.001 {
            state.error = Some("egui scale differs from captured appearance".into());
            state.gpu = None;
            return vec![];
        }
        let outcome = (|| {
            if state.gpu.is_none() {
                state.gpu = Some(Gpu::new(
                    device,
                    queue,
                    state.generation,
                    self.adapter.config,
                    self.adapter.limits,
                )?);
            }
            state
                .gpu
                .as_mut()
                .unwrap()
                .prepare(&self.frame, device, queue, encoder)
        })();
        if let Err(error) = outcome {
            state.error = Some(format!("{error:?}"));
            state.gpu = None;
        }
        vec![]
    }
    fn paint(
        &self,
        mut info: epaint::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        let state = self.adapter.state.lock().expect("embedding adapter lock");
        if !state.disposed && state.epoch == self.epoch {
            if let Some(gpu) = &state.gpu {
                let c = self.frame.placement.clip;
                info.clip_rect = info
                    .clip_rect
                    .intersect(epaint::emath::Rect::from_min_size(
                        epaint::emath::pos2(c[0], c[1]),
                        epaint::emath::vec2(c[2], c[3]),
                    ))
                    .intersect(info.viewport);
                let c = info.clip_rect_in_pixels();
                if c.width_px <= 0 || c.height_px <= 0 {
                    return;
                }
                pass.set_scissor_rect(
                    c.left_px as u32,
                    c.top_px as u32,
                    c.width_px as u32,
                    c.height_px as u32,
                );
                gpu.paint(pass);
            }
        }
    }
}
