// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Actual portable/semantic receiving at shadow and slot boundaries.
use super::*;
use crate::{
    LeafRegistry, ProducerAction, ProducerNode, ProducerRole, ProducerSemantics, WindowDom,
};
use layout_dom_api::ShadowRootInit;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug)]
enum Boundary {
    Shadow,
    NestedShadow,
    Slotted,
}
struct Fixture {
    dom: ScriptedDom,
    window: NodeId,
    shadow: NodeId,
    leaf: NodeId,
    decoy: NodeId,
    slot: Option<NodeId>,
    fallback: Option<NodeId>,
}
fn name(local: &str) -> QualName {
    QualName::new(None, Namespace::default(), LocalName::from(local))
}
fn element(dom: &mut ScriptedDom, parent: NodeId, tag: &str) -> NodeId {
    let node = dom.create_element(name(tag));
    dom.append_child(parent, node);
    node
}
fn leaf(dom: &mut ScriptedDom, parent: NodeId) -> NodeId {
    let node = element(dom, parent, "custom-leaf");
    dom.set_attribute(node, name("key"), "7");
    dom.set_attribute(node, name("style"), "display:block;width:8px;height:6px");
    node
}
fn fixture(boundary: Boundary) -> Fixture {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let window = element(&mut dom, root, "main");
    let host = element(&mut dom, window, "section");
    let decoy = leaf(&mut dom, host);
    dom.set_attribute(decoy, name("slot"), "not-offered");
    let mut shadow = dom.attach_shadow_unchecked(host, ShadowRootInit::default(), false);
    if matches!(boundary, Boundary::NestedShadow) {
        let nested = element(&mut dom, shadow, "section");
        shadow = dom.attach_shadow_unchecked(nested, ShadowRootInit::default(), false);
    }
    let (node, slot, fallback) = if matches!(boundary, Boundary::Slotted) {
        let slot = element(&mut dom, shadow, "slot");
        dom.set_attribute(slot, name("name"), "scene");
        let fallback = leaf(&mut dom, slot);
        let assigned = leaf(&mut dom, host);
        dom.set_attribute(assigned, name("slot"), "scene");
        assert_eq!(dom.assigned_nodes(slot), [assigned]);
        (assigned, Some(slot), Some(fallback))
    } else {
        (leaf(&mut dom, shadow), None, None)
    };
    Fixture {
        dom,
        window,
        shadow,
        leaf: node,
        decoy,
        slot,
        fallback,
    }
}
fn layout(f: &Fixture) -> OwnedLayout {
    OwnedLayout::new(
        &WindowDom::new(&f.dom, f.window),
        &["main,section {display:block}"],
        160.0,
        120.0,
        &[],
        &Default::default(),
    )
}
#[derive(Default)]
struct Observed {
    frames: Vec<ProducerFrameInfo>,
    retires: usize,
    actions: usize,
}
struct SceneSource(Rc<RefCell<Observed>>);
impl TextureProducer for SceneSource {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
    fn portable_scene(
        &mut self,
        frame: &ProducerFrameInfo,
    ) -> Result<ProducedScene, PortableProducerError> {
        self.0.borrow_mut().frames.push(frame.clone());
        Ok(ProducedScene {
            scene: netrender::Scene::new(8, 6),
            source_identity: 7,
            source_revision: 1,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        })
    }
    fn semantics(&mut self) -> Option<ProducerSemantics> {
        Some(ProducerSemantics {
            role: Some(ProducerRole::Group),
            name: Some("Scene".into()),
            children: vec![ProducerNode {
                key: 40,
                role: ProducerRole::ListItem,
                name: "Card".into(),
                rect: [0.0, 0.0, 8.0, 6.0],
                actions: vec![ProducerAction {
                    id: "pin".into(),
                    label: "Pin".into(),
                    description: "Keep card".into(),
                }],
            }],
        })
    }
    fn act(&mut self, key: u64, id: &str) -> bool {
        self.0.borrow_mut().actions += 1;
        key == 40 && id == "pin"
    }
    fn retire(&mut self) {
        self.0.borrow_mut().retires += 1;
    }
}
fn registry() -> (ProducerRegistry, Rc<RefCell<Observed>>) {
    let observed = Rc::new(RefCell::new(Observed::default()));
    let mut registry = ProducerRegistry::new();
    registry
        .register(7, SceneSource(observed.clone()), &[])
        .unwrap();
    (registry, observed)
}
fn capture(
    registry: &mut ProducerRegistry,
    f: &Fixture,
    layout: &OwnedLayout,
) -> PortablePaintCapture {
    let view = WindowDom::new(&f.dom, f.window);
    let plan = registry
        .plan_portable(layout, &view, 1.0, &|_, _| false)
        .unwrap();
    let commands = plan
        .commands(7)
        .expect("the rendered slot has a portable source");
    let mut env = envelope(&plan);
    env.commands = commands;
    registry.admit_portable(plan, env).unwrap()
}
#[test]
fn captured_shadow_and_slotted_owner_matches_semantics_once() {
    for boundary in [Boundary::Shadow, Boundary::NestedShadow, Boundary::Slotted] {
        let f = fixture(boundary);
        let layout = layout(&f);
        let view = WindowDom::new(&f.dom, f.window);
        assert_eq!(
            layout
                .element_geometry(&view, f.leaf)
                .unwrap()
                .content_size(),
            (8.0, 6.0)
        );
        assert_eq!(
            layout.custom_leaf_nodes(&view),
            [(7, f.leaf)],
            "{boundary:?}: hidden light/fallback owners must not interfere"
        );
        let (mut registry, observed) = registry();
        let projection = crate::project_semantics(
            &view,
            &layout,
            &mut LeafRegistry::new(),
            &mut registry,
            None,
        );
        assert_eq!(projection.producer_slots.get(&7), Some(&f.leaf));
        assert!(!projection.dom_targets.values().any(|n| *n == f.decoy));
        assert_eq!(projection.produced_targets.len(), 1);
        assert!(registry.act(projection.produced_targets.values().next().unwrap()));
        let captured = capture(&mut registry, &f, &layout);
        assert_eq!(captured.producers.len(), 1);
        assert_eq!(captured.producers[0].logical_size, [8.0, 6.0]);
        assert_eq!(
            registry.entries[&7].portable.as_ref().unwrap().owner,
            f.leaf
        );
        assert_eq!(
            observed.borrow().frames.len(),
            1,
            "one callback, not one per light/shadow appearance"
        );
        assert_eq!(observed.borrow().actions, 1);
    }
}
#[test]
fn slot_reassignment_rebinds_fallback_and_restores_the_assigned_owner() {
    let mut f = fixture(Boundary::Slotted);
    let (mut registry, observed) = registry();
    let first = capture(&mut registry, &f, &layout(&f));
    f.dom.set_attribute(f.leaf, name("slot"), "not-offered");
    assert!(f.dom.assigned_nodes(f.slot.unwrap()).is_empty());
    let fallback = capture(&mut registry, &f, &layout(&f));
    assert_eq!(
        registry.entries[&7].portable.as_ref().unwrap().owner,
        f.fallback.unwrap()
    );
    assert_ne!(
        first.producers[0].content_generation,
        fallback.producers[0].content_generation
    );
    assert_eq!(observed.borrow().retires, 1);
    f.dom.set_attribute(f.leaf, name("slot"), "scene");
    let restored = capture(&mut registry, &f, &layout(&f));
    assert_eq!(
        registry.entries[&7].portable.as_ref().unwrap().owner,
        f.leaf
    );
    assert_ne!(
        restored.producers[0].content_generation,
        fallback.producers[0].content_generation
    );
    assert_eq!(observed.borrow().retires, 2);
    assert_eq!(observed.borrow().frames.len(), 3);
}
#[test]
fn removing_the_only_shadow_slot_retires_without_an_unassigned_replacement() {
    let mut f = fixture(Boundary::Slotted);
    let (mut registry, observed) = registry();
    capture(&mut registry, &f, &layout(&f));
    f.dom.remove(f.slot.unwrap());
    let layout = layout(&f);
    let view = WindowDom::new(&f.dom, f.window);
    assert!(layout.custom_leaf_nodes(&view).is_empty());
    let plan = registry
        .plan_portable(&layout, &view, 1.0, &|_, _| false)
        .unwrap();
    assert!(plan.commands(7).is_none());
    let env = PaintEnvelope {
        engine: paint_list_api::EngineId::GENET,
        viewport: DeviceIntSize::new(160, 120),
        generation: 1,
        commands: vec![],
        fonts: vec![],
        images: vec![],
    };
    registry.admit_portable(plan, env).unwrap();
    assert!(!registry.contains(7));
    assert_eq!(observed.borrow().frames.len(), 1);
    assert_eq!(observed.borrow().retires, 1);
}
#[test]
fn duplicate_visible_shadow_owners_refuse_before_source_callbacks() {
    let mut f = fixture(Boundary::Shadow);
    let shadow = f.shadow;
    leaf(&mut f.dom, shadow);
    let layout = layout(&f);
    let view = WindowDom::new(&f.dom, f.window);
    let (mut registry, observed) = registry();
    assert!(matches!(
        registry.plan_portable(&layout, &view, 1.0, &|_, _| false),
        Err(PortableCaptureError::DuplicateDomKey { texture_key: 7 })
    ));
    assert!(observed.borrow().frames.is_empty());
    assert_eq!(observed.borrow().retires, 0);
    assert!(registry.entries[&7].portable.is_none());
    let projection = crate::project_semantics(
        &view,
        &layout,
        &mut LeafRegistry::new(),
        &mut registry,
        None,
    );
    assert!(projection.produced_targets.is_empty());
    assert!(projection.producer_slots.is_empty());
}
#[test]
fn foreign_window_binding_does_not_replace_the_local_shadow_owner() {
    let mut f = fixture(Boundary::Shadow);
    let root = f.dom.document();
    let foreign = element(&mut f.dom, root, "main");
    let foreign_leaf = leaf(&mut f.dom, foreign);
    let layout = layout(&f);
    let view = WindowDom::new(&f.dom, f.window);
    assert!(!view.contains(foreign_leaf));
    assert_eq!(layout.custom_leaf_nodes(&view), [(7, f.leaf)]);
    let (mut registry, _) = registry();
    capture(&mut registry, &f, &layout);
    assert_eq!(
        registry.entries[&7].portable.as_ref().unwrap().owner,
        f.leaf
    );
}
