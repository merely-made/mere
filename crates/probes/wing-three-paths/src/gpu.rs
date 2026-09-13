// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Target textures, readback, process memory, and small stats helpers.

use std::time::Duration;

use netrender::{NetrenderOptions, Renderer, create_netrender_instance};

pub const TILE: u32 = 256;

pub fn make_target(device: &wgpu::Device, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("l0c target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

pub fn new_renderer(handles: &netrender::WgpuHandles) -> Renderer {
    create_netrender_instance(
        handles.clone(),
        NetrenderOptions {
            tile_cache_size: Some(TILE),
            enable_vello: true,
            ..Default::default()
        },
    )
    .expect("create_netrender_instance")
}

/// Read an RGBA8 texture back as tightly packed rows.
pub fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("l0c readback"),
        size: (padded * h) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("l0c readback"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        tx.send(r).ok();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    rx.recv().expect("map").expect("map ok");
    let data = slice.get_mapped_range().expect("mapped range");
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    for row in 0..h {
        let s = (row * padded) as usize;
        out.extend_from_slice(&data[s..s + unpadded as usize]);
    }
    drop(data);
    staging.unmap();
    out
}

pub fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

pub fn median(v: &mut Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

pub fn max(v: &[f64]) -> f64 {
    v.iter().cloned().filter(|x| x.is_finite()).fold(f64::NAN, f64::max)
}

pub fn jf(x: f64) -> serde_json::Value {
    if x.is_finite() { serde_json::json!(x) } else { serde_json::Value::Null }
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool: usize,
    quota_paged_pool: usize,
    quota_peak_non_paged_pool: usize,
    quota_non_paged_pool: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[cfg(windows)]
unsafe extern "system" {
    fn K32GetProcessMemoryInfo(process: isize, counters: *mut ProcessMemoryCounters, cb: u32) -> i32;
    fn GetCurrentProcess() -> isize;
}

#[cfg(windows)]
pub fn rss_mb() -> (f64, f64) {
    let mut c = ProcessMemoryCounters {
        cb: size_of::<ProcessMemoryCounters>() as u32,
        ..Default::default()
    };
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    if ok == 0 {
        return (f64::NAN, f64::NAN);
    }
    let mb = 1024.0 * 1024.0;
    (c.working_set_size as f64 / mb, c.peak_working_set_size as f64 / mb)
}

#[cfg(not(windows))]
pub fn rss_mb() -> (f64, f64) {
    (f64::NAN, f64::NAN)
}
