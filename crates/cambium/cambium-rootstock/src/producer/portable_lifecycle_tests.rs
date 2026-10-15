// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation/export clocks, ownership and accepted source metadata.
use super::*;

#[test]
fn portable_presentation_advances_once_and_export_only_observes() {
    use std::{cell::Cell, rc::Rc};
    struct CountingSource(Rc<Cell<u64>>);
    impl TextureProducer for CountingSource {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn portable_scene(
            &mut self,
            frame: &ProducerFrameInfo,
        ) -> Result<ProducedScene, PortableProducerError> {
            if frame.purpose == ProducerFramePurpose::Presentation {
                self.0.set(self.0.get() + 1);
            }
            Ok(ProducedScene {
                scene: netrender::Scene::new(8, 6),
                source_identity: 7,
                source_revision: self.0.get(),
                alpha: SourceAlpha::Straight,
                encoding: SourceEncoding::Srgb,
            })
        }
    }
    let (dom, layout) = fixture();
    let steps = Rc::new(Cell::new(0));
    let mut registry = ProducerRegistry::new();
    registry
        .register(7, CountingSource(steps.clone()), &[])
        .unwrap();
    let plan = registry
        .plan_portable_for(
            &layout,
            &dom,
            1.0,
            &|_, _| false,
            ProducerFramePurpose::Presentation,
        )
        .unwrap();
    let env = envelope(&plan);
    let presented = registry.admit_portable(plan, env).unwrap();
    assert_eq!(steps.get(), 1);
    let plan = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let env = envelope(&plan);
    let exported = registry.admit_portable(plan, env).unwrap();
    assert_eq!(steps.get(), 1, "capture must not advance dynamics");
    assert_eq!(
        presented.producers[0].source_revision,
        exported.producers[0].source_revision
    );
    assert_eq!(
        presented.producers[0].content_generation,
        exported.producers[0].content_generation
    );
}

#[test]
fn portable_presentation_suspends_once_per_visibility_transition() {
    use std::{cell::Cell, rc::Rc};
    struct Source(Rc<Cell<usize>>);
    impl TextureProducer for Source {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn portable_scene(
            &mut self,
            _: &ProducerFrameInfo,
        ) -> Result<ProducedScene, PortableProducerError> {
            Ok(ProducedScene {
                scene: netrender::Scene::new(8, 6),
                source_identity: 7,
                source_revision: 1,
                alpha: SourceAlpha::Straight,
                encoding: SourceEncoding::Srgb,
            })
        }
        fn suspend(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let (dom, layout) = fixture();
    let suspends = Rc::new(Cell::new(0));
    let mut registry = ProducerRegistry::new();
    registry.register(7, Source(suspends.clone()), &[]).unwrap();
    let plan = registry
        .plan_portable_for(
            &layout,
            &dom,
            1.0,
            &|_, _| false,
            ProducerFramePurpose::Presentation,
        )
        .unwrap();
    let env = envelope(&plan);
    registry.admit_portable(plan, env).unwrap();
    registry.suspend_except(&|_, _| false, None);
    registry.suspend_except(&|_, _| false, None);
    assert_eq!(suspends.get(), 1);
    // Reading the suspended source must not reactivate its dynamics clock.
    let plan = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let env = envelope(&plan);
    registry.admit_portable(plan, env).unwrap();
    registry.suspend_except(&|_, _| false, None);
    assert_eq!(suspends.get(), 1);
    let plan = registry
        .plan_portable_for(
            &layout,
            &dom,
            1.0,
            &|_, _| false,
            ProducerFramePurpose::Presentation,
        )
        .unwrap();
    let env = envelope(&plan);
    registry.admit_portable(plan, env).unwrap();
    registry.suspend_except(&|key, owner| key == 7 && owner.is_some(), None);
    assert_eq!(suspends.get(), 1, "another host must keep its clock");
    registry.suspend_except(&|_, _| false, None);
    assert_eq!(suspends.get(), 2);
}

#[test]
fn portable_rebind_preserves_metadata_and_removal_commits_after_admission() {
    use std::{cell::Cell, rc::Rc};
    struct Source(Rc<Cell<usize>>);
    impl TextureProducer for Source {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn portable_scene(
            &mut self,
            _: &ProducerFrameInfo,
        ) -> Result<ProducedScene, PortableProducerError> {
            Ok(ProducedScene {
                scene: netrender::Scene::new(8, 6),
                source_identity: 7,
                source_revision: 41,
                alpha: SourceAlpha::Premultiplied,
                encoding: SourceEncoding::Srgb,
            })
        }
        fn retire(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let (dom, layout) = fixture();
    let retires = Rc::new(Cell::new(0));
    let mut registry = ProducerRegistry::new();
    registry.register(7, Source(retires.clone()), &[]).unwrap();
    let plan = registry
        .plan_portable(&layout, &dom, 1.0, &|_, _| false)
        .unwrap();
    let env = envelope(&plan);
    let original = registry.admit_portable(plan, env).unwrap();
    assert_eq!(original.producers[0].alpha, ResourceAlpha::Premultiplied);
    assert_eq!(original.producers[0].encoding, ResourceEncoding::Srgb);
    let old_owner = registry.entries[&7].portable.as_ref().unwrap().owner;
    let mut rebound = ScriptedDom::new();
    let name = |local| QualName::new(None, Namespace::from(""), LocalName::from(local));
    rebound.create_element(name("unused"));
    let leaf = rebound.create_element(name("custom-leaf"));
    rebound.set_attribute(leaf, name("key"), "7");
    rebound.append_child(rebound.document(), leaf);
    assert_ne!(old_owner, leaf);
    let layout = OwnedLayout::new(
        &rebound,
        &["custom-leaf { display:block;width:8px;height:6px; }"],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    let plan = registry
        .plan_portable(&layout, &rebound, 1.0, &|_, _| false)
        .unwrap();
    assert_eq!(
        retires.get(),
        0,
        "source planning cannot retire the live owner"
    );
    let env = envelope(&plan);
    let fresh = registry.admit_portable(plan, env).unwrap();
    assert_eq!(retires.get(), 1);
    assert_ne!(
        original.producers[0].content_generation,
        fresh.producers[0].content_generation
    );
    assert_eq!(fresh.producers[0].source_revision, 41);
    let empty = ScriptedDom::new();
    let empty_layout = OwnedLayout::new(&empty, &[], 160.0, 120.0, &[], &Default::default());
    let held = registry
        .plan_portable(&empty_layout, &empty, 1.0, &|key, owner| {
            key == 7 && owner.is_some()
        })
        .unwrap();
    let mut env = original.envelope.clone();
    env.commands.clear();
    registry.admit_portable(held, env.clone()).unwrap();
    assert!(registry.contains(7), "another host still owns this source");
    let absent = registry
        .plan_portable(&empty_layout, &empty, 1.0, &|_, _| false)
        .unwrap();
    assert!(registry.contains(7));
    assert_eq!(retires.get(), 1);
    registry.admit_portable(absent, env).unwrap();
    assert!(!registry.contains(7));
    assert_eq!(retires.get(), 2);
}
