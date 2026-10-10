// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reading a presented frame back.
//!
//! Every self-driving Cambium app needs this and every one had written it: the
//! standard wgpu readback (compose into an owned `COPY_SRC` target, copy to a
//! row-aligned buffer, map it, strip the per-row padding). It lives here so a
//! scenario receipt is an in-process readback of the frame that was actually
//! presented — no compositor, no foreground window, and no chance of
//! photographing the wrong window.
//!
//! Event-loop hosts start an owned readback and poll it on later turns, so
//! waiting for GPU completion cannot stall other windows sharing the device.
//!
//! What the bytes become is the application's business: woodshed writes a PNG,
//! the host's own smoke example digests them.

use crate::Surface;
use netrender::ExternalTexturePlacement;

/// A presented frame, read back through the shared render-host machinery.
pub type Frame = genet_render_host::RgbaFrame;

/// An owned readback that can be polled without waiting on the shared GPU.
pub type PendingFrame = genet_render_host::PendingRgbaReadback;

/// Compose the presented view and start copying it into host memory.
/// Poll the returned frame on later event-loop turns; it retains the original
/// pixels even when another window draws through the same render core.
pub fn start_frame_readback(
    surface: &dyn Surface,
    view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Result<PendingFrame, String> {
    let target = capture_target(surface, view, width, height);
    surface.core().start_rgba8_readback(&target, width, height)
}

/// Compose `view` — the rasterized frame the host just presented — into an
/// owned target and read it back. `None` if the readback failed.
/// This synchronous path waits for at most five seconds; event-loop capture
/// backends should use [`start_frame_readback`] instead.
pub fn read_frame(
    surface: &dyn Surface,
    view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Option<Frame> {
    let target = capture_target(surface, view, width, height);
    surface
        .core()
        .read_rgba8_texture(&target, width, height)
        .map_err(|error| eprintln!("[cambium-host] {error}"))
        .ok()
}

fn capture_target(
    surface: &dyn Surface,
    view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> wgpu::Texture {
    let target = surface.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("cambium host frame capture"),
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
    target
}
