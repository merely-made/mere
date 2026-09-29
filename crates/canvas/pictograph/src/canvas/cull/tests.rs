// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use paint_list_api::{
    ColorF, CommonPlacement, LayerSpec, LayoutPoint, PathCommand, PathData, RectItem, StrokeCap,
    StrokeItem, TransformKind, TransformSpec,
};

fn bounds(x: f32, y: f32, w: f32, h: f32) -> LayoutRect {
    LayoutRect::new(LayoutPoint::new(x, y), LayoutPoint::new(x + w, y + h))
}

fn rect(x: f32, y: f32) -> PaintCmd {
    PaintCmd::DrawRect(RectItem {
        placement: CommonPlacement::new(bounds(x, y, 10.0, 10.0)),
        color: ColorF::new(1.0, 0.0, 0.0, 1.0),
    })
}

fn transform(x: f32, y: f32, scale: f32) -> PaintCmd {
    PaintCmd::PushTransform(TransformSpec {
        origin: LayoutPoint::new(x, y),
        transform: LayoutTransform::scale(scale, scale, 1.0),
        kind: TransformKind::Standard,
    })
}

#[test]
fn nested_transforms_cull_in_screen_space_and_keep_stack_order() {
    let commands = vec![
        transform(50.0, 20.0, 2.0),
        transform(-20.0, 0.0, 0.5),
        rect(0.0, 0.0), // screen (10,20), visible
        rect(150.0, 0.0),
        PaintCmd::PopTransform,
        PaintCmd::PopTransform,
        rect(20.0, 30.0),
    ];
    let kept = visible_commands(&commands, bounds(0.0, 0.0, 100.0, 100.0));
    assert_eq!(kept.len(), 6);
    assert!(matches!(kept[2], PaintCmd::DrawRect(_)));
    assert!(matches!(kept[3], PaintCmd::PopTransform));
    assert!(matches!(kept[4], PaintCmd::PopTransform));
    assert!(matches!(kept[5], PaintCmd::DrawRect(_)));
}

fn edge(y: f32) -> PaintCmd {
    PaintCmd::DrawStroke(StrokeItem {
        placement: CommonPlacement::new(bounds(-100.0, y, 300.0, 0.0)),
        path: PathData {
            commands: vec![
                PathCommand::MoveTo(LayoutPoint::new(-100.0, y)),
                PathCommand::LineTo(LayoutPoint::new(200.0, y)),
            ],
        },
        width: 4.0,
        color: ColorF::new(1.0, 0.0, 0.0, 1.0),
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        dash: None,
    })
}

#[test]
fn crossing_and_grazing_edges_survive_but_distant_edges_do_not() {
    let commands = vec![edge(50.0), edge(-1.0), edge(-20.0)];
    let kept = visible_commands(&commands, bounds(0.0, 0.0, 100.0, 100.0));
    assert_eq!(kept.len(), 2);
}

#[test]
fn a_visible_caption_is_independent_of_its_offscreen_body() {
    // Two primitives in one node transform: the body is offscreen while its
    // right-hand caption box crosses the viewport. Never cull the whole node
    // using only its center or face bounds.
    let commands = vec![
        transform(-50.0, 20.0, 1.0),
        rect(0.0, 0.0),
        rect(48.0, 0.0),
        PaintCmd::PopTransform,
    ];
    let kept = visible_commands(&commands, bounds(0.0, 0.0, 100.0, 100.0));
    assert_eq!(kept.len(), 3);
    let PaintCmd::DrawRect(caption) = &kept[1] else {
        panic!("caption dropped")
    };
    assert_eq!(caption.placement.bounds.min.x, 48.0);
}

#[test]
fn filters_can_reach_the_viewport_from_outside() {
    let commands = vec![
        PaintCmd::PushLayer(LayerSpec {
            filters: vec![paint_list_api::FilterOp::Blur(40.0)],
            ..Default::default()
        }),
        rect(-30.0, 20.0),
        PaintCmd::PopLayer,
        rect(-30.0, 20.0),
    ];
    assert_eq!(
        visible_commands(&commands, bounds(0.0, 0.0, 100.0, 100.0)).len(),
        3
    );
}

#[test]
fn large_offscreen_population_does_not_reach_scene_lowering() {
    let mut commands = vec![rect(20.0, 20.0)];
    commands.extend((0..2000).map(|index| rect(200.0 + index as f32 * 20.0, 20.0)));
    assert_eq!(
        visible_commands(&commands, bounds(0.0, 0.0, 100.0, 100.0)).len(),
        1
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn culling_preserves_gpu_pixels_and_detects_a_lost_crossing_edge() {
    use netrender::{ColorLoad, NetrenderOptions, create_netrender_instance};
    use netrender_vello::wgpu;
    use paint_list_api::DeviceIntSize;
    use paint_list_render::{CompositeLayer, composite_paint_layers};

    let handles = netrender::boot().expect("GPU device");
    let renderer = create_netrender_instance(
        handles,
        NetrenderOptions {
            tile_cache_size: Some(64),
            enable_vello: true,
            ..Default::default()
        },
    )
    .expect("renderer");
    let target = renderer
        .wgpu_device
        .core
        .device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("pictograph culling comparison"),
            size: wgpu::Extent3d {
                width: 100,
                height: 100,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
    let view = target.create_view(&Default::default());
    let render = |commands: &[PaintCmd]| {
        let scene = composite_paint_layers(
            DeviceIntSize::new(100, 100),
            &[CompositeLayer::commands_only(commands)],
        )
        .scene;
        renderer.render_vello(&scene, &view, ColorLoad::Clear(wgpu::Color::BLACK));
        renderer.wgpu_device.read_rgba8_texture(&target, 100, 100)
    };
    let mut commands = vec![edge(50.0), edge(-1.0), edge(-20.0), rect(20.0, 20.0)];
    commands.extend((0..64).map(|index| rect(200.0 + index as f32 * 20.0, 20.0)));
    commands.extend([
        PaintCmd::PushTransform(TransformSpec {
            origin: LayoutPoint::new(80.0, 20.0),
            transform: LayoutTransform::rotation(0.0, 0.0, 1.0, euclid::Angle::radians(0.6)),
            kind: TransformKind::Standard,
        }),
        rect(0.0, 0.0),
        rect(200.0, 0.0),
        PaintCmd::PopTransform,
    ]);
    let expected = render(&commands);
    let kept = visible_commands(&commands, bounds(0.0, 0.0, 100.0, 100.0));
    assert!(kept.len() < commands.len() / 2);
    assert_eq!(render(&kept), expected, "culling changed visible pixels");
    // Positive control: endpoint-only culling would lose this crossing edge.
    assert_ne!(
        render(&kept[1..]),
        expected,
        "pixel comparison missed a lost edge"
    );
}
