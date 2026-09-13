// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Path C: resident geometry. Every chunk and every part is a `BodyMesh`
//! drawn by Mesocosm's `LiveBodyRenderer` into a colour + Depth32Float
//! pair on the shared device, then composited over netrender's background
//! through `compose_external_texture`. A content change is a new content
//! address, so the adapter's cache builds a new vertex buffer.

use std::collections::BTreeMap;
use std::time::Instant;

use mesocosm_mesh::BodyMesh;
use mesocosm_render::live_body::{BodyDrawStats, LiveBody, LiveBodyRenderer};
use netrender::{ColorLoad, ExternalTexturePlacement, Renderer, Scene};

use crate::project::{BACKGROUND, View};
use crate::world::{Changes, World};

#[derive(Default, Debug, Clone, Copy)]
pub struct Invalidation {
    pub mesh_uploads: usize,
    pub mesh_upload_bytes: usize,
    pub instance_upload_bytes: usize,
    pub evictions: usize,
    pub draws: usize,
}

pub struct PathC {
    pub renderer: Renderer,
    lb: LiveBodyRenderer,
    #[allow(dead_code)]
    colour: wgpu::Texture,
    colour_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    chunk_meshes: BTreeMap<usize, BodyMesh>,
    part_meshes: BTreeMap<(usize, usize), (u64, BodyMesh)>,
    pub inv: Invalidation,
    pub last: BodyDrawStats,
}

impl PathC {
    pub fn new(renderer: Renderer, device: &wgpu::Device, w: u32, h: u32, cache_capacity: usize) -> Self {
        let lb = LiveBodyRenderer::new(device, wgpu::TextureFormat::Rgba8Unorm, cache_capacity);
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("l0c path c colour"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("l0c path c depth"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        Self {
            renderer,
            lb,
            colour_view: colour.create_view(&Default::default()),
            depth_view: depth.create_view(&Default::default()),
            colour,
            chunk_meshes: BTreeMap::new(),
            part_meshes: BTreeMap::new(),
            inv: Invalidation::default(),
            last: BodyDrawStats::default(),
        }
    }

    fn sync_meshes(&mut self, world: &World, changes: &Changes, first: bool) {
        for (ci, c) in world.chunks.iter().enumerate() {
            if first || changes.chunks.contains(&ci) {
                self.chunk_meshes.insert(ci, BodyMesh::single(c.reference, &c.volume));
            }
        }
        for (bi, b) in world.bodies.iter().enumerate() {
            for (pi, p) in b.parts.iter().enumerate() {
                let stale = self
                    .part_meshes
                    .get(&(bi, pi))
                    .is_none_or(|(g, _)| *g != p.generation);
                if stale {
                    self.part_meshes.insert((bi, pi), (p.generation, BodyMesh::single(p.reference, &p.volume)));
                }
                // Placement lives on the BodyMesh; the adapter builds the
                // model matrix from it every frame.
                let (_, mesh) = self.part_meshes.get_mut(&(bi, pi)).unwrap();
                let pl = &mut mesh.placements[0];
                pl.pivot = p.pivot;
                pl.pivot_at = p.pivot_at;
                pl.yaw = p.yaw;
            }
        }
    }

    pub fn frame(
        &mut self,
        view: &View,
        world: &World,
        changes: &Changes,
        first: bool,
        clip: Option<[f32; 4]>,
        target: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (f64, f64, f64) {
        let t0 = Instant::now();
        self.sync_meshes(world, changes, first);
        let mut bodies: Vec<LiveBody<'_>> = Vec::new();
        for (ci, c) in world.chunks.iter().enumerate() {
            let o = c.origin();
            bodies.push(LiveBody::new(&self.chunk_meshes[&ci], [o[0] as f32, o[1] as f32, o[2] as f32]));
        }
        for (bi, b) in world.bodies.iter().enumerate() {
            for pi in 0..b.parts.len() {
                let mut body = LiveBody::new(&self.part_meshes[&(bi, pi)].1, b.origin.to_array());
                body.yaw_radians = b.yaw_deg.to_radians();
                bodies.push(body);
            }
        }
        let build = crate::gpu::ms(t0.elapsed());

        let t1 = Instant::now();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("l0c path c") });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("l0c clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.colour_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        let stats = self
            .lb
            .draw(
                device,
                queue,
                &mut encoder,
                &self.colour_view,
                &self.depth_view,
                view.clip_from_world.to_cols_array_2d(),
                None,
                &bodies,
            )
            .expect("live body draw");
        queue.submit([encoder.finish()]);
        self.last = stats;
        self.inv.mesh_uploads += stats.mesh_uploads;
        self.inv.mesh_upload_bytes += stats.mesh_upload_bytes;
        self.inv.instance_upload_bytes += stats.instance_upload_bytes;
        self.inv.evictions += stats.evictions;
        self.inv.draws += stats.draws;

        let mut scene = Scene::new(view.w, view.h);
        scene.push_rect(0.0, 0.0, view.w as f32, view.h as f32, [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 1.0]);
        self.renderer.render_vello(&scene, target, ColorLoad::Clear(wgpu::Color::BLACK));
        let (dest, uv) = match clip {
            Some(c) => (c, [c[0] / view.w as f32, c[1] / view.h as f32, c[2] / view.w as f32, c[3] / view.h as f32]),
            None => ([0.0, 0.0, view.w as f32, view.h as f32], [0.0, 0.0, 1.0, 1.0]),
        };
        self.renderer.compose_external_texture(
            &self.colour_view,
            target,
            wgpu::TextureFormat::Rgba8Unorm,
            view.w,
            view.h,
            ExternalTexturePlacement::new(dest).with_uv(uv),
        );
        let encode = crate::gpu::ms(t1.elapsed());
        let t2 = Instant::now();
        device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
        (build, encode, crate::gpu::ms(t2.elapsed()))
    }
}
