// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::Frame;
use netrender::vello_backends::{HybridSession, SparseResourceLimits, SparseSessionError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Caller render-pass attachment contract. The private intermediate is
/// single-sample RGBA8 with premultiplied sRGB values; painting uses source-over.
/// For an sRGB attachment the shader decodes RGB before hardware re-encoding.
pub struct TargetConfig {
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
}
#[derive(Debug)]
pub enum GpuError {
    Scene(SparseSessionError),
    InvalidConfiguration(&'static str),
    Disposed,
    DeviceMismatch,
}
impl From<SparseSessionError> for GpuError {
    fn from(v: SparseSessionError) -> Self {
        Self::Scene(v)
    }
}

/// Resources for one independent appearance on the supplied device generation.
/// No window/surface/adapter is created. No method calls Queue::submit. The
/// caller must submit the encoded frame before preparing this appearance again,
/// because the renderer updates GPU buffers using writes to the supplied queue.
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    generation: u64,
    config: TargetConfig,
    limits: SparseResourceLimits,
    session: Option<HybridSession>,
    pipeline: Option<wgpu::RenderPipeline>,
    layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    texture: Option<wgpu::Texture>,
    bind: Option<wgpu::BindGroup>,
    disposed: bool,
}
impl Gpu {
    /// Capabilities declared before allocation. Sparse GPU uses ordinary wgpu
    /// baseline features/limits; optional provider/backend capabilities remain
    /// with Netrender's existing admission owner.
    pub fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    pub fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        generation: u64,
        config: TargetConfig,
        limits: SparseResourceLimits,
    ) -> Result<Self, GpuError> {
        if !config
            .format
            .guaranteed_format_features(device.features())
            .flags
            .sample_count_supported(config.sample_count)
            || !matches!(
                config.format,
                wgpu::TextureFormat::Rgba8Unorm
                    | wgpu::TextureFormat::Bgra8Unorm
                    | wgpu::TextureFormat::Rgba8UnormSrgb
                    | wgpu::TextureFormat::Bgra8UnormSrgb
            )
        {
            return Err(GpuError::InvalidConfiguration(
                "RGBA/BGRA 8-bit target and supported sample count required",
            ));
        }
        if !device.features().contains(Self::required_features())
            || !Self::required_limits().check_limits(&device.limits())
        {
            return Err(GpuError::InvalidConfiguration(
                "caller device does not meet embedding baseline features/limits",
            ));
        }
        let session = HybridSession::new(device, queue, limits)?;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mere embedded image"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mere embedded image"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let source = format!(
            r#"
struct Vertex {{ @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }}
@vertex fn vs(@builtin(vertex_index) n: u32)->Vertex {{
    let uv = vec2<f32>(f32((n<<1u)&2u), f32(n&2u));
    return Vertex(vec4<f32>(uv.x*2.-1.,1.-uv.y*2.,0.,1.),uv);
}}
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;
fn decode(v:vec3<f32>)->vec3<f32> {{
    return select(pow((v+vec3<f32>(0.055))/1.055,vec3<f32>(2.4)),v/12.92,v<=vec3<f32>(0.04045));
}}
@fragment fn fs(v:Vertex)->@location(0) vec4<f32> {{
    let c=textureSample(image,image_sampler,v.uv);
    if ({}) {{ return vec4<f32>(decode(c.rgb/max(c.a,0.000001))*c.a,c.a); }}
    return c;
}}
"#,
            config.format.is_srgb()
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mere embedded image"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mere embedded image"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: config.sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mere embedded image"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            generation,
            config,
            limits,
            session: Some(session),
            pipeline: Some(pipeline),
            layout: Some(layout),
            sampler: Some(sampler),
            texture: None,
            bind: None,
            disposed: false,
        })
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn config(&self) -> TargetConfig {
        self.config
    }
    /// The caller's shared renderer recovery owner supplies the new generation
    /// and GPU handles after its detachment barrier. This adapter owns only its
    /// device-bound scene resources; retained owner/session facts stay outside.
    pub fn recover(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        generation: u64,
    ) -> Result<(), GpuError> {
        if self.disposed {
            return Err(GpuError::Disposed);
        }
        if generation == self.generation {
            return if device == &self.device && queue == &self.queue {
                Ok(())
            } else {
                Err(GpuError::DeviceMismatch)
            };
        }
        *self = Self::new(device, queue, generation, self.config, self.limits)?;
        Ok(())
    }
    pub fn prepare(
        &mut self,
        frame: &Frame,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), GpuError> {
        if self.disposed {
            return Err(GpuError::Disposed);
        }
        if device != &self.device || queue != &self.queue {
            return Err(GpuError::DeviceMismatch);
        }
        frame
            .placement
            .validate()
            .map_err(GpuError::InvalidConfiguration)?;
        if frame.produced.encoding != crate::SourceEncoding::Srgb {
            return Err(GpuError::InvalidConfiguration(
                "embedding requires declared sRGB scene encoding",
            ));
        }
        let scene = frame.physical_scene();
        let size = [scene.viewport_width, scene.viewport_height];
        if size.iter().any(|n| {
            *n == 0 || *n > device.limits().max_texture_dimension_2d || *n > u16::MAX as u32
        }) {
            return Err(GpuError::InvalidConfiguration(
                "appearance dimensions exceed caller device/sparse limits",
            ));
        }
        // Keep the previous usable target until complete scene admission and
        // encoding succeed. Failure never exposes a partially prepared image.
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Mere embedded scene"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.session
            .as_mut()
            .ok_or(GpuError::Disposed)?
            .render(&scene, device, queue, encoder, &texture)?;
        let view = texture.create_view(&Default::default());
        self.bind = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mere embedded image"),
            layout: self.layout.as_ref().ok_or(GpuError::Disposed)?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        self.sampler.as_ref().ok_or(GpuError::Disposed)?,
                    ),
                },
            ],
        }));
        self.texture = Some(texture);
        Ok(())
    }
    /// Paint into a caller render pass whose viewport/scissor identify this
    /// appearance. egui/Iced set those before their custom rendering callback.
    pub fn paint(&self, pass: &mut wgpu::RenderPass<'_>) -> bool {
        if self.disposed {
            return false;
        }
        let Some(bind) = &self.bind else {
            return false;
        };
        let Some(pipeline) = &self.pipeline else {
            return false;
        };
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
        true
    }
    pub fn suspend(&mut self) {
        self.texture = None;
        self.bind = None;
    }
    pub fn dispose(&mut self) {
        self.suspend();
        self.session = None;
        self.pipeline = None;
        self.layout = None;
        self.sampler = None;
        self.disposed = true;
    }
}
