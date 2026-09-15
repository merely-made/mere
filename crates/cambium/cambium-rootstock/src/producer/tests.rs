// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{NodeId, OwnedLayout, ScriptedDom, Surface};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, QualName};
use paint_list_api::{DeviceIntSize, PaintList};

fn name(local: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(local))
}

fn fixture(sheet: &str) -> (ScriptedDom, OwnedLayout, NodeId) {
    let mut dom = ScriptedDom::new();
    let node = dom.create_element(name("custom-leaf"));
    dom.set_attribute(node, name("key"), "7");
    dom.append_child(dom.document(), node);
    let layout = OwnedLayout::new(&dom, &[sheet], 160.0, 120.0, &[], &Default::default());
    (dom, layout, node)
}

const SHEET: &str = "custom-leaf { display:block;width:8px;height:6px;color:rgb(64,128,192); }";

#[derive(Default)]
struct Producer {
    view: Option<wgpu::TextureView>,
    generation: u64,
    last_color: Option<[f32; 4]>,
    replace_view: bool,
    pixel_override: Option<[u8; 4]>,
    linear: bool,
    frames: Vec<ProducerFrameInfo>,
    suspended: usize,
    retired: usize,
}

impl TextureProducer for Producer {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        self.frames.push(cx.frame.clone());
        let color = cx.frame.appearance.color().unwrap_or([1.0; 4]);
        let changed_color = self.last_color != Some(color);
        if self.view.is_none() || cx.frame.needs_frame || self.replace_view {
            let [width, height] = cx.frame.physical_size;
            let texture = cx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("rootstock producer test"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.view = Some(texture.create_view(&Default::default()));
            self.replace_view = false;
        }
        if changed_color {
            self.generation += 1;
        }
        self.last_color = Some(color);
        let view = self.view.as_ref().unwrap();
        let [width, height] = cx.frame.physical_size;
        let pixel = self
            .pixel_override
            .unwrap_or_else(|| color.map(|value| (value * 255.0).round() as u8));
        let bytes = pixel.repeat((width * height) as usize);
        cx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: view.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            view.texture().size(),
        );
        Some(ProducedTexture {
            view: view.clone(),
            generation: self.generation,
            alpha: SourceAlpha::Straight,
            encoding: if self.linear {
                SourceEncoding::LinearSrgb
            } else {
                SourceEncoding::Srgb
            },
        })
    }
    fn suspend(&mut self) {
        self.suspended += 1;
        self.view = None;
    }
    fn retire(&mut self) {
        self.retired += 1;
        self.view = None;
    }
}

#[test]
fn cpu_registration_rejects_aliases_and_does_not_replace_a_live_producer() {
    let producer = Rc::new(RefCell::new(Producer::default()));
    let mut registry = ProducerRegistry::new();
    registry.register(7, producer.clone(), &["color"]).unwrap();
    assert_eq!(
        registry.register(7, producer.clone(), &[]),
        Err(ProducerRegistrationError::AlreadyRegistered)
    );
    assert_eq!(
        registry.register(1_u64 << 62, producer.clone(), &[]),
        Err(ProducerRegistrationError::KeyOutOfRange)
    );
    assert_eq!(producer.borrow().retired, 0);
    assert!(registry.remove(7));
    assert!(!registry.remove(7));
    assert_eq!(producer.borrow().retired, 1);
    registry.register(7, producer.clone(), &[]).unwrap();
    drop(registry);
    assert_eq!(producer.borrow().retired, 2);
}

#[test]
fn cpu_physical_extent_accounts_for_zoom_fractional_size_and_device_limits() {
    assert_eq!(
        registry::physical_size((10.5, 20.0), 1.5, 4096),
        Some([16, 30])
    );
    for (size, scale) in [
        ((0.0, 1.0), 1.0),
        ((1.0, 1.0), 0.0),
        ((f32::NAN, 2.0), 1.0),
        ((1.0, 1.0), f32::INFINITY),
        ((4097.0, 1.0), 1.0),
    ] {
        assert_eq!(registry::physical_size(size, scale, 4096), None);
    }
}

#[test]
fn cpu_appearance_reads_only_declared_values_with_typed_used_color() {
    let (dom, layout, node) = fixture(
        "custom-leaf { display:block;width:8px;height:6px;color:rgba(51,102,153,.5);--scene-mark: moss;opacity:.25; }",
    );
    let appearance = layout.resolved_appearance(node, &["color".into(), "--scene-mark".into()]);
    let color = appearance.color().unwrap();
    for (actual, expected) in color.into_iter().zip([0.2, 0.4, 0.6, 0.5]) {
        assert!((actual - expected).abs() < 1e-5);
    }
    assert_eq!(appearance.css("--scene-mark").unwrap().trim(), "moss");
    assert_eq!(appearance.css("opacity"), None);
    assert_eq!(
        layout.resolved_appearance(node, &[]),
        ResolvedAppearance::default()
    );
    assert_eq!(layout.custom_leaf_boxes(&dom), vec![(7, (8.0, 6.0))]);
}

#[test]
fn cpu_content_extent_and_pointer_mapping_exclude_border_and_padding() {
    let (dom, layout, node) = fixture(
        "custom-leaf { position:absolute;display:block;left:20px;top:30px;width:80px;height:40px;padding:7px;border:5px solid black;transform:scale(2);transform-origin:0 0; }",
    );
    assert_eq!(layout.custom_leaf_boxes(&dom), vec![(7, (80.0, 40.0))]);
    let geometry = layout.element_geometry(&dom, node).unwrap();
    // Content starts at (20+2*12,30+2*12). One content pixel is two document pixels.
    let local = geometry.map_to_content(54.0, 68.0).unwrap();
    assert!((local.0 - 5.0).abs() < 1e-5 && (local.1 - 7.0).abs() < 1e-5);
    assert_eq!(geometry.map_to_content(42.0, 54.0), None);
    assert_eq!(
        layout.local_coordinates(&dom, node, (54.0, 68.0)),
        Some(((5.0, 7.0), (80.0, 40.0)))
    );
}

struct TestSurface(genet_render_host::RenderCore);
impl Surface for TestSurface {
    fn core(&self) -> &genet_render_host::RenderCore {
        &self.0
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        None
    }
}

fn surface() -> TestSurface {
    TestSurface(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            ..Default::default()
        })
        .expect("GPU producer fixture needs a headless adapter"),
    )
}

fn pixel(
    surface: &TestSurface,
    registry: &ProducerRegistry,
    dom: &ScriptedDom,
    layout: &mut OwnedLayout,
) -> [u8; 4] {
    let list = layout.emit_paint_list_with_leaves(
        dom,
        DeviceIntSize::new(160, 120),
        |key| registry.commands(key),
        |_| None,
    );
    let translated = paint_list_render::translate_paint_cmd_stream(
        list.viewport(),
        list.commands(),
        list.fonts(),
        list.images(),
    );
    let (texture, _) = surface.core().rasterize(
        &translated.scene,
        160,
        120,
        netrender::ColorLoad::Clear(wgpu::Color::BLACK),
    );
    let frame = surface
        .core()
        .read_rgba8_texture(&texture, 160, 120)
        .unwrap();
    frame.rgba[(2 * 160 + 2) * 4..(2 * 160 + 2) * 4 + 4]
        .try_into()
        .unwrap()
}

#[test]
fn gpu_producer_refresh_resize_suspend_remove_and_recreate_use_fresh_images() {
    let surface = surface();
    let (mut dom, mut layout, node) = fixture(SHEET);
    let producer = Rc::new(RefCell::new(Producer::default()));
    let mut registry = ProducerRegistry::new();
    registry.register(7, producer.clone(), &["color"]).unwrap();
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [64, 128, 192, 255]
    );
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 0);
    assert!(!producer.borrow().frames.last().unwrap().needs_frame);
    let generation = producer.borrow().generation;
    producer.borrow_mut().replace_view = true;
    producer.borrow_mut().pixel_override = Some([200, 32, 80, 255]);
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        producer.borrow().generation,
        generation,
        "source identity protects a reused generation"
    );
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [200, 32, 80, 255],
        "a reused generation still invalidates the document's old pixels"
    );

    let decoration = SHEET.replace(
        "color:rgb",
        "box-shadow:1px 1px 2px black;border-color:red;color:rgb",
    );
    layout = OwnedLayout::new(&dom, &[&decoration], 160.0, 120.0, &[], &Default::default());
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 0);
    producer.borrow_mut().pixel_override = None;
    layout = OwnedLayout::new(
        &dom,
        &[&SHEET.replace("64,128,192", "0,255,0")],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [0, 255, 0, 255]
    );

    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.5).stages, 1);
    assert_eq!(
        producer.borrow().frames.last().unwrap().physical_size,
        [12, 9]
    );
    assert!(producer.borrow().frames.last().unwrap().needs_frame);
    layout = OwnedLayout::new(
        &dom,
        &["custom-leaf { display:none; }"],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).suspensions,
        1
    );
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).render_calls,
        0
    );
    assert_eq!(producer.borrow().suspended, 1);
    assert!(registry.contains(7));
    assert!(registry.commands(7).unwrap().is_empty());
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    registry.suspend_all(Some(surface.renderer()));
    assert!(registry.commands(7).unwrap().is_empty());
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);

    dom.remove(node);
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).retirements,
        1
    );
    assert!(!registry.contains(7));
    assert_eq!(producer.borrow().retired, 1);
    assert!(
        !surface
            .renderer()
            .unregister_external_image(netrender::external_image_key(7))
    );
    let new_node = dom.create_element(name("custom-leaf"));
    dom.set_attribute(new_node, name("key"), "7");
    dom.append_child(dom.document(), new_node);
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    registry.register(7, producer.clone(), &["color"]).unwrap();
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [64, 128, 192, 255]
    );
    registry.remove(7);
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).retirements,
        1
    );
    assert_eq!(producer.borrow().retired, 2);
}

#[test]
fn gpu_unsupported_encoding_and_duplicate_dom_key_refuse_stale_pixels() {
    let surface = surface();
    let (mut dom, mut layout, _) = fixture(SHEET);
    let producer = Rc::new(RefCell::new(Producer::default()));
    let mut registry = ProducerRegistry::new();
    registry.register(7, producer.clone(), &["color"]).unwrap();
    registry.prepare(&surface, &layout, &dom, 1.0);
    producer.borrow_mut().linear = true;
    assert_eq!(
        registry
            .prepare(&surface, &layout, &dom, 1.0)
            .invalid_frames,
        1
    );
    assert_eq!(registry.error(7), Some(ProducerError::UnsupportedEncoding));
    assert!(registry.commands(7).unwrap().is_empty());
    producer.borrow_mut().linear = false;
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    let duplicate = dom.create_element(name("custom-leaf"));
    dom.set_attribute(duplicate, name("key"), "7");
    dom.append_child(dom.document(), duplicate);
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    let stats = registry.prepare(&surface, &layout, &dom, 1.0);
    assert_eq!((stats.render_calls, stats.invalid_frames), (0, 1));
    assert_eq!(registry.error(7), Some(ProducerError::DuplicateDomKey));
    assert!(registry.commands(7).unwrap().is_empty());
}
