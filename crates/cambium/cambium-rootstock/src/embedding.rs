// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Caller-scheduled native scene embedding. The caller keeps its device, queue,
//! encoder, target, event loop and submission. This opt-in realization uses
//! Netrender's admitted sparse GPU scene contract, including outline text and
//! held images. Unsupported scene operations return errors rather than pixels.
//! CPU session/storage authority never enters the toolkit rendering callback.

use crate::{
    ProducedScene, ProducerFrameInfo, ProducerSemantics, ResolvedAppearance, TextureProducer,
};
use document_session_api::SessionInput;
use netrender::{Scene, SceneOp, Transform};

mod gpu;
mod input;
pub use gpu::{Gpu, GpuError, TargetConfig};
pub use input::{OwnerInput, ScrollUnits};
#[cfg(all(feature = "embedding-egui", not(target_arch = "wasm32")))]
pub mod egui;
#[cfg(all(feature = "embedding-egui", not(target_arch = "wasm32")))]
mod egui_input;
#[cfg(all(feature = "embedding-iced", not(target_arch = "wasm32")))]
pub mod iced;
#[cfg(all(feature = "embedding-iced", not(target_arch = "wasm32")))]
mod iced_input;

/// Coordinates supplied by the toolkit, in its logical pixels. Bounds name the
/// full appearance; clip names its visible part. Scale is physical/logical.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub bounds: [f32; 4],
    pub clip: [f32; 4],
    pub scale: f32,
}
impl Placement {
    pub fn validate(self) -> Result<Self, &'static str> {
        if !self
            .bounds
            .iter()
            .chain(self.clip.iter())
            .all(|n| n.is_finite())
            || !self.scale.is_finite()
            || self.scale <= 0.
            || self.bounds[2] <= 0.
            || self.bounds[3] <= 0.
            || self.clip[2] < 0.
            || self.clip[3] < 0.
        {
            return Err("finite positive bounds/scale and nonnegative clipping required");
        }
        Ok(self)
    }
    pub fn physical_size(self) -> [u32; 2] {
        [
            (self.bounds[2] * self.scale).ceil() as u32,
            (self.bounds[3] * self.scale).ceil() as u32,
        ]
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        [self.bounds, self.clip]
            .iter()
            .all(|r| x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3])
    }
}

/// An immutable scene packet for a render callback. It contains retained source
/// facts and CPU paint resources, never a storage interface or a live owner.
#[derive(Clone, Debug)]
pub struct Frame {
    pub appearance: u64,
    pub placement: Placement,
    pub produced: ProducedScene,
}
impl Frame {
    pub fn physical_scene(&self) -> Scene {
        let mut scene = self.produced.scene.clone();
        let scale = self.placement.scale;
        let size = self.placement.physical_size();
        scene.viewport_width = size[0];
        scene.viewport_height = size[1];
        let transform = Transform::scale_2d(scale, scale);
        for t in &mut scene.transforms {
            *t = t.then(&transform);
        }
        // Preserve Scene's identity sentinel. Every operation now names the
        // transformed copy of its old palette entry, including former id zero.
        scene.transforms.insert(0, Transform::IDENTITY);
        // Primitive clips/radii are explicitly device-space, unlike geometry
        // and layer clip shapes which travel through the transform palette.
        for op in &mut scene.ops {
            let transform_id = match op {
                SceneOp::Rect(v) => &mut v.transform_id,
                SceneOp::Stroke(v) => &mut v.transform_id,
                SceneOp::Gradient(v) => &mut v.transform_id,
                SceneOp::Image(v) => &mut v.transform_id,
                SceneOp::Pattern(v) => &mut v.transform_id,
                SceneOp::Shape(v) => &mut v.transform_id,
                SceneOp::GlyphRun(v) => &mut v.transform_id,
                SceneOp::PushLayer(v) => &mut v.transform_id,
                SceneOp::Fragment(v) => &mut v.transform_id,
                SceneOp::PopLayer => continue,
            };
            *transform_id = transform_id.saturating_add(1);
            let (clip, radii) = match op {
                SceneOp::Rect(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                SceneOp::Stroke(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                SceneOp::Gradient(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                SceneOp::Image(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                SceneOp::Pattern(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                SceneOp::Shape(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                SceneOp::GlyphRun(v) => (&mut v.clip_rect, &mut v.clip_corner_radii),
                _ => continue,
            };
            for value in clip.iter_mut().chain(radii.iter_mut()) {
                *value *= scale;
            }
        }
        scene
    }
}

/// One independent appearance of a content owner. Suspension/disposal releases
/// producer resources without deleting retained material. GPU generation is a
/// separate host concern handled by [`Gpu::recover`].
pub struct View<P: TextureProducer> {
    appearance: u64,
    producer: P,
    placement: Placement,
    visible: bool,
    focused: bool,
    captured: bool,
    disposed: bool,
    invalidated: bool,
    wake_at: Option<std::time::Duration>,
}
impl<P: TextureProducer> View<P> {
    pub fn new(appearance: u64, producer: P, placement: Placement) -> Result<Self, &'static str> {
        Ok(Self {
            appearance,
            producer,
            placement: placement.validate()?,
            visible: true,
            focused: false,
            captured: false,
            disposed: false,
            invalidated: true,
            wake_at: None,
        })
    }
    pub fn placement(&self) -> Placement {
        self.placement
    }
    pub fn resize(&mut self, placement: Placement) -> Result<(), &'static str> {
        if self.disposed {
            return Err("appearance disposed");
        }
        self.placement = placement.validate()?;
        self.invalidate();
        Ok(())
    }
    pub fn invalidate(&mut self) {
        if !self.disposed {
            self.invalidated = true;
        }
    }
    pub fn needs_frame(&self) -> bool {
        !self.disposed && self.visible && self.invalidated
    }
    /// The content owner reports its next timer deadline using the caller's
    /// monotonic timebase. The caller schedules its own event-loop wake.
    pub fn request_wake(&mut self, at: std::time::Duration) {
        if !self.disposed {
            self.wake_at = Some(self.wake_at.map_or(at, |old| old.min(at)));
        }
    }
    pub fn next_wake(&self) -> Option<std::time::Duration> {
        if self.disposed || !self.visible {
            None
        } else {
            self.wake_at
        }
    }
    pub fn wake(&mut self, now: std::time::Duration) -> bool {
        if self.next_wake().is_some_and(|at| at <= now) {
            self.wake_at = None;
            self.invalidate();
            true
        } else {
            false
        }
    }
    pub fn set_visible(&mut self, visible: bool) {
        if self.disposed || self.visible == visible {
            return;
        }
        self.visible = visible;
        self.invalidate();
        if !visible {
            self.producer.suspend();
            self.focused = false;
            self.captured = false;
        }
    }
    pub fn set_pointer_capture(&mut self, captured: bool) {
        self.captured = captured && self.visible && !self.disposed;
    }
    /// Routes only this appearance's input. The callback belongs to the actual
    /// session owner and returns its cursor/capture/navigation/IME effects.
    /// Host keyboard focus must be explicitly granted with Focus(true).
    pub fn input<R>(
        &mut self,
        mut input: SessionInput,
        owner: impl FnOnce(SessionInput) -> R,
    ) -> Option<R> {
        if self.disposed || !self.visible {
            return None;
        }
        match &mut input {
            SessionInput::Focus(focus) => {
                self.focused = *focus;
                if !*focus {
                    self.captured = false;
                }
            },
            SessionInput::PointerMoved { x, y, .. } | SessionInput::PointerButton { x, y, .. } => {
                if !self.captured && !self.placement.contains(*x, *y) {
                    return None;
                }
                *x -= self.placement.bounds[0];
                *y -= self.placement.bounds[1];
            },
            SessionInput::Cancel => self.captured = false,
            _ if !self.focused => return None,
            _ => (),
        }
        self.invalidate();
        Some(owner(input))
    }
    pub fn dispatch<R>(
        &mut self,
        input: OwnerInput,
        owner: impl FnOnce(OwnerInput) -> R,
    ) -> Option<R> {
        match input {
            OwnerInput::Session(input) => {
                self.input(input, |input| owner(OwnerInput::Session(input)))
            },
            OwnerInput::Wheel {
                mut position,
                delta,
            } => {
                if self.disposed
                    || !self.visible
                    || (!self.captured && !self.placement.contains(position[0], position[1]))
                {
                    return None;
                }
                position[0] -= self.placement.bounds[0];
                position[1] -= self.placement.bounds[1];
                self.invalidate();
                Some(owner(OwnerInput::Wheel { position, delta }))
            },
        }
    }
    pub fn frame(
        &mut self,
        timestamp: Option<std::time::Duration>,
    ) -> Result<Frame, crate::PortableProducerError> {
        if self.disposed || !self.visible {
            return Err(crate::PortableProducerError::InvalidSource);
        }
        let info = ProducerFrameInfo {
            logical_size: (self.placement.bounds[2], self.placement.bounds[3]),
            physical_size: self.placement.physical_size(),
            layout_scale: self.placement.scale,
            needs_frame: self.invalidated,
            timestamp,
            appearance: ResolvedAppearance::default(),
        };
        let produced = self.producer.portable_scene(&info)?;
        self.invalidated = false;
        Ok(Frame {
            appearance: self.appearance,
            placement: self.placement,
            produced,
        })
    }
    pub fn semantics(&mut self) -> Option<ProducerSemantics> {
        if self.disposed || !self.visible {
            None
        } else {
            self.producer.semantics()
        }
    }
    pub fn accessibility_action(&mut self, key: u64, action: &str) -> bool {
        if self.disposed || !self.visible {
            return false;
        }
        let handled = self.producer.act(key, action);
        if handled {
            self.invalidate();
        }
        handled
    }
    pub fn dispose(&mut self) {
        if !self.disposed {
            self.producer.retire();
            self.disposed = true;
            self.focused = false;
            self.captured = false;
            self.invalidated = false;
            self.wake_at = None;
        }
    }
}
impl<P: TextureProducer> Drop for View<P> {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[cfg(test)]
mod tests;
