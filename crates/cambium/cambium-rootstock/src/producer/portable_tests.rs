// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::ScriptedDom;
use layout_dom_api::{LayoutDomMut, LocalName, Namespace, QualName};
use paint_list_api::{DeviceIntSize, PaintEnvelope};

struct Source {
    unsupported: bool,
    oversized: bool,
    additional_ops: usize,
}
impl TextureProducer for Source {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        let [width, height] = cx.frame.physical_size;
        let texture = cx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("portable refusal retained texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Some(ProducedTexture {
            view: texture.create_view(&Default::default()),
            generation: 1,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        })
    }
    fn portable_scene(
        &mut self,
        frame: &ProducerFrameInfo,
    ) -> Result<ProducedScene, PortableProducerError> {
        if self.unsupported {
            return Err(PortableProducerError::Unsupported);
        }
        let mut scene = netrender::Scene::new(
            frame.logical_size.0.round() as u32,
            frame.logical_size.1.round() as u32,
        );
        if self.oversized {
            scene.ops.resize(
                paint_list_render::bundle::BundleLimits::default().max_ops + 1,
                netrender::SceneOp::PopLayer,
            );
        }
        if self.additional_ops > 0 {
            scene
                .ops
                .resize(self.additional_ops, netrender::SceneOp::PopLayer);
        }
        Ok(ProducedScene {
            scene,
            source_identity: 1,
            source_revision: 1,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        })
    }
}

struct GpuOnly;
impl TextureProducer for GpuOnly {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
}

fn fixture() -> (ScriptedDom, OwnedLayout) {
    let mut dom = ScriptedDom::new();
    let name = |local| QualName::new(None, Namespace::from(""), LocalName::from(local));
    let node = dom.create_element(name("custom-leaf"));
    dom.set_attribute(node, name("key"), "7");
    dom.append_child(dom.document(), node);
    let layout = OwnedLayout::new(
        &dom,
        &["custom-leaf { display:block;width:8px;height:6px; }"],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    (dom, layout)
}

fn envelope(plan: &PortablePlan) -> PaintEnvelope {
    PaintEnvelope {
        engine: paint_list_api::EngineId::GENET,
        viewport: DeviceIntSize::new(160, 120),
        generation: 1,
        commands: plan.commands(7).unwrap(),
        fonts: vec![],
        images: vec![],
    }
}

#[path = "portable_lifecycle_tests.rs"]
mod lifecycle;

#[test]
fn default_gpu_only_source_refuses_without_registry_mutation() {
    let (dom, layout) = fixture();
    let mut registry = ProducerRegistry::new();
    registry.register(7, GpuOnly, &[]).unwrap();
    registry.next_stamp = 19;
    assert!(matches!(
        registry.plan_portable(&layout, &dom, 1.0, &|_, _| false),
        Err(PortableCaptureError::Producer {
            texture_key: 7,
            error: PortableProducerError::Unsupported
        })
    ));
    assert!(registry.contains(7));
    assert_eq!(registry.next_stamp, 19);
    assert!(registry.entries[&7].portable.is_none());
    assert!(registry.entries[&7].image.is_none());
    assert!(!registry.entries[&7].active);
    assert!(registry.pending_retire.is_empty());
}

#[test]
fn late_whole_envelope_refusal_preserves_prior_stamps_and_registration() {
    let (dom, layout) = fixture();
    let mut registry = ProducerRegistry::new();
    registry
        .register(
            7,
            Source {
                unsupported: false,
                oversized: false,
                additional_ops: 0,
            },
            &[],
        )
        .unwrap();
    let first = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let env = envelope(&first);
    registry.admit_portable(first, env).unwrap();
    let stamp = registry.next_stamp;
    let prior = registry.entries[&7]
        .portable
        .as_ref()
        .unwrap()
        .source
        .as_ref()
        .unwrap()
        .content_generation;
    let plan = registry
        .plan_portable(&layout, &dom, 2.0, &|_, _| false)
        .unwrap();
    let mut env = envelope(&plan);
    let PaintCmd::DrawExternalTexture(mut missing) = env.commands[0].clone() else {
        unreachable!()
    };
    missing.texture_key = 99;
    env.commands.push(PaintCmd::DrawExternalTexture(missing));
    assert!(matches!(
        registry.admit_portable(plan, env),
        Err(PortableCaptureError::Bundle(BundleError::MissingProducer(
            99
        )))
    ));
    assert_eq!(registry.next_stamp, stamp);
    assert_eq!(
        registry.entries[&7]
            .portable
            .as_ref()
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .content_generation,
        prior
    );
    assert!(registry.contains(7));
    assert!(registry.pending_retire.is_empty());
    let plan = registry
        .plan_portable(&layout, &dom, 2.0, &|_, _| false)
        .unwrap();
    let mut env = envelope(&plan);
    env.commands.push(PaintCmd::PopAllShadows);
    assert!(matches!(
        registry.admit_portable(plan, env),
        Err(PortableCaptureError::Bundle(_))
    ));
    assert_eq!(registry.next_stamp, stamp);
    assert_eq!(
        registry.entries[&7]
            .portable
            .as_ref()
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .content_generation,
        prior
    );
    assert!(registry.contains(7));
    assert!(registry.pending_retire.is_empty());
}

#[test]
fn oversized_callback_scene_refuses_before_digest_and_preserves_accepted_stamp() {
    let (dom, layout) = fixture();
    let source = std::rc::Rc::new(std::cell::RefCell::new(Source {
        unsupported: false,
        oversized: false,
        additional_ops: 0,
    }));
    let mut registry = ProducerRegistry::new();
    registry.register(7, source.clone(), &[]).unwrap();
    let plan = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let env = envelope(&plan);
    registry.admit_portable(plan, env).unwrap();
    let stamp = registry.next_stamp;
    let owner = registry.entries[&7].portable.as_ref().unwrap().owner;
    source.borrow_mut().oversized = true;
    // The shared preflight now charges aggregate structure before its
    // scene-op-specific check or layer validation. This fixture crosses
    // that exact first limit; assert its amount as well as its typed name.
    let limits = paint_list_render::bundle::BundleLimits::default();
    match registry.plan_portable(&layout, &dom, 1.0, &|_, _| false) {
        Err(PortableCaptureError::Bundle(BundleError::Budget {
            resource,
            requested,
            limit,
        })) => {
            assert_eq!(resource, "aggregate structure");
            assert_eq!(requested, limits.max_ops + 1);
            assert_eq!(limit, limits.max_ops);
        },
        Err(other) => panic!("expected aggregate source budget refusal, got {other:?}"),
        Ok(_) => panic!("oversized source must refuse before digest"),
    }
    assert_eq!(registry.next_stamp, stamp);
    let entry = &registry.entries[&7];
    assert_eq!(entry.portable.as_ref().unwrap().owner, owner);
    assert_eq!(
        entry
            .portable
            .as_ref()
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .content_generation,
        stamp
    );
    assert!(
        entry
            .portable
            .as_ref()
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .scene
            .ops
            .is_empty()
    );
    assert!(registry.contains(7));
    assert!(registry.pending_retire.is_empty());
}

#[test]
fn two_source_aggregate_refusal_preserves_all_accepted_metadata() {
    let (mut dom, _) = fixture();
    let name = |local| QualName::new(None, Namespace::from(""), LocalName::from(local));
    let node = dom.create_element(name("custom-leaf"));
    dom.set_attribute(node, name("key"), "8");
    dom.append_child(dom.document(), node);
    let layout = OwnedLayout::new(
        &dom,
        &["custom-leaf { display:block;width:8px;height:6px; }"],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    let mut registry = ProducerRegistry::new();
    let sources: Vec<_> = [7, 8]
        .into_iter()
        .map(|key| {
            let source = std::rc::Rc::new(std::cell::RefCell::new(Source {
                unsupported: false,
                oversized: false,
                additional_ops: 0,
            }));
            registry.register(key, source.clone(), &[]).unwrap();
            source
        })
        .collect();
    let plan = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let mut env = envelope(&plan);
    env.commands.extend(plan.commands(8).unwrap());
    registry.admit_portable(plan, env).unwrap();
    let stamp = registry.next_stamp;
    let accepted: Vec<_> = [7, 8]
        .into_iter()
        .map(|key| {
            let entry = &registry.entries[&key];
            let slot = entry.portable.as_ref().unwrap();
            (
                key,
                slot.owner,
                slot.source.as_ref().unwrap().content_generation,
                entry.owner,
                entry.active,
            )
        })
        .collect();
    let limits = paint_list_render::bundle::BundleLimits::default();
    for source in &sources {
        source.borrow_mut().additional_ops = limits.max_ops / 2 + 1;
    }
    // Each scene's structure is under the individual quota. Their combined
    // structure crosses the whole-plan quota before either gets hashed.
    match registry.plan_portable(&layout, &dom, 1.0, &|_, _| false) {
        Err(PortableCaptureError::Bundle(BundleError::Budget {
            resource,
            requested,
            limit,
        })) => {
            assert_eq!(resource, "aggregate structure");
            assert!(requested > limits.max_ops);
            assert_eq!(limit, limits.max_ops);
        },
        Err(other) => panic!("expected combined structure budget refusal, got {other:?}"),
        Ok(_) => panic!("combined source plan must refuse before digest"),
    }
    assert_eq!(registry.next_stamp, stamp);
    for (key, owner, content_generation, gpu_owner, active) in accepted {
        let entry = &registry.entries[&key];
        let slot = entry.portable.as_ref().unwrap();
        assert_eq!(slot.owner, owner);
        assert_eq!(
            slot.source.as_ref().unwrap().content_generation,
            content_generation
        );
        assert!(slot.source.as_ref().unwrap().scene.ops.is_empty());
        assert_eq!(entry.owner, gpu_owner);
        assert_eq!(entry.active, active);
        assert!(entry.image.is_none());
        assert!(registry.contains(key));
    }
    assert!(registry.pending_retire.is_empty());
}

#[test]
#[ignore = "real-device cache proof; root serializes GPU acceptance"]
fn portable_refusal_preserves_live_gpu_view_and_stamp() {
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
    let surface = TestSurface(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            ..Default::default()
        })
        .expect("explicit GPU acceptance adapter"),
    );
    let (dom, layout) = fixture();
    let source = std::rc::Rc::new(std::cell::RefCell::new(Source {
        unsupported: false,
        oversized: false,
        additional_ops: 0,
    }));
    let mut registry = ProducerRegistry::new();
    registry.register(7, source.clone(), &[]).unwrap();
    registry.prepare(&surface, &layout, &dom, 1.0);
    let plan = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let env = envelope(&plan);
    registry.admit_portable(plan, env).unwrap();
    let image = registry.entries[&7].image.as_ref().unwrap();
    let view = image.output.view.clone();
    let gpu_stamp = image.stamp;
    let stamp = registry.next_stamp;
    let owner = registry.entries[&7].owner;
    source.borrow_mut().unsupported = true;
    assert!(
        registry
            .plan_portable(&layout, &dom, 2.0, &|_, _| false)
            .is_err()
    );
    let entry = &registry.entries[&7];
    assert_eq!(entry.image.as_ref().unwrap().output.view, view);
    assert_eq!(entry.image.as_ref().unwrap().stamp, gpu_stamp);
    assert_eq!(entry.owner, owner);
    assert!(entry.active);
    assert!(entry.portable.is_some());
    assert_eq!(registry.next_stamp, stamp);
    assert!(registry.contains(7));
    assert!(registry.pending_retire.is_empty());
}
