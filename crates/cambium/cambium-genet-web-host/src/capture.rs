// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reading a presented frame back in a browser.
//!
//! Rootstock's `read_frame` waits for the copy to land, which a browser's main
//! thread cannot do: WebGPU maps a buffer only from its own event loop. So the
//! capture hook starts the same composition and copy, and the frame arrives as
//! a [`PendingFrame`] some frames later.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cambium_rootstock::{CaptureFn, Frame, Surface};
use netrender::ExternalTexturePlacement;

/// A frame being read back: `ready` some frames after it was presented, and
/// taken once.
pub struct PendingFrame {
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    done: Rc<Cell<bool>>,
}

impl PendingFrame {
    /// Whether the browser has mapped the frame's bytes.
    pub fn ready(&self) -> bool {
        self.done.get()
    }

    /// Tightly packed RGBA8 rows, top-down. Call once [`ready`](Self::ready).
    pub fn take(self) -> Result<Frame, String> {
        let slice = self.buffer.slice(..);
        let data = slice
            .get_mapped_range()
            .map_err(|error| format!("capture map failed: {error}"))?;
        let unpadded = (self.width * 4) as usize;
        let mut rgba = Vec::with_capacity(unpadded * self.height as usize);
        for row in 0..self.height as usize {
            let start = row * self.padded_bytes_per_row as usize;
            rgba.extend_from_slice(&data[start..start + unpadded]);
        }
        drop(data);
        self.buffer.unmap();
        Ok(Frame {
            width: self.width,
            height: self.height,
            rgba,
        })
    }
}

/// A capture hook that starts reading the next presented frame into `slot`.
/// Set it on `AppCtx::capture`; poll the slot's frame from later frames.
pub fn capture_into(slot: Rc<RefCell<Option<PendingFrame>>>) -> CaptureFn {
    Box::new(move |surface, view, width, height| {
        *slot.borrow_mut() = Some(start(surface, view, width, height));
    })
}

fn start(surface: &dyn Surface, view: &wgpu::TextureView, width: u32, height: u32) -> PendingFrame {
    let width = width.max(1);
    let height = height.max(1);
    let device = surface.device();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("cambium web frame capture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    surface.renderer().compose_external_texture(
        view,
        &target_view,
        wgpu::TextureFormat::Rgba8Unorm,
        width,
        height,
        ExternalTexturePlacement::new([0.0, 0.0, width as f32, height as f32]),
    );
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = (width * 4).div_ceil(align) * align;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("cambium web frame readback"),
        size: u64::from(padded_bytes_per_row) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("cambium web frame readback"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    surface.queue().submit([encoder.finish()]);
    let done = Rc::new(Cell::new(false));
    let flag = done.clone();
    // The browser calls this from its event loop once the copy has run; there
    // is nothing to poll on the main thread.
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |_| flag.set(true));
    PendingFrame {
        buffer,
        width,
        height,
        padded_bytes_per_row,
        done,
    }
}
