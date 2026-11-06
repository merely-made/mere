// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use layout_dom_api::{LayoutDomMut, LocalName, Namespace, QualName};
use std::time::{Duration, Instant};

struct Source {
    label: String,
    next_label: String,
    pending: Option<String>,
    live: bool,
    child: ProducerChildLifetime,
    commits: usize,
    discards: usize,
    required: bool,
}
impl Source {
    fn new() -> Self {
        Self {
            label: "accepted".into(),
            next_label: "candidate".into(),
            pending: None,
            live: true,
            child: ProducerChildLifetime::new(),
            commits: 0,
            discards: 0,
            required: false,
        }
    }
}
impl TextureProducer for Source {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        panic!("legacy callback")
    }
    fn supports_native_staging(&self) -> bool {
        true
    }
    fn render_staged(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, genet_render_host::RasterizeFailure> {
        self.required = cx.frame.needs_frame;
        self.pending = Some(self.next_label.clone());
        let [w, h] = cx.frame.physical_size;
        let mut scene = netrender::Scene::new(w, h);
        scene.push_rect(0., 0., w as f32, h as f32, [0.2, 0.4, 0.6, 1.]);
        let (_, view) = cx.try_rasterize_scaled_for(
            719,
            &scene,
            w,
            h,
            netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.,
        )?;
        Ok(Some(ProducedTexture {
            view,
            generation: 1,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        }))
    }
    fn publish_staged(&mut self) {
        if let Some(label) = self.pending.take() {
            self.label = label;
        }
        self.commits += 1;
    }
    fn discard_staged(&mut self) {
        self.pending = None;
        self.discards += 1;
    }
    fn staged_is_live(&self) -> bool {
        self.live
    }
    fn semantics(&self) -> Option<ProducerSemantics> {
        Some(ProducerSemantics {
            name: Some(self.pending.as_ref().unwrap_or(&self.label).clone()),
            children: vec![ProducerNode {
                key: 1,
                role: ProducerRole::GraphicsObject,
                name: "child".into(),
                rect: [0., 0., 8., 6.],
                actions: vec![],
            }],
            ..Default::default()
        })
    }
    fn child_lifetime(&self, key: u64) -> Option<ProducerChildToken> {
        (key == 1).then(|| self.child.token())
    }
}

#[test]
fn legacy_producers_do_not_implicitly_opt_into_native_staging() {
    struct Legacy;
    impl TextureProducer for Legacy {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
    }
    let mut registry = ProducerRegistry::new();
    assert!(registry.can_stage_native());
    registry.register(7, Legacy, &[]).unwrap();
    assert!(!registry.can_stage_native());
}

#[test]
fn rc_adapter_forwards_staging_capability_liveness_and_publication_hooks() {
    let source = Rc::new(RefCell::new(Source::new()));
    let mut adapter = source.clone();
    assert!(adapter.supports_native_staging());
    source.borrow_mut().pending = Some("candidate".into());
    adapter.publish_staged();
    assert_eq!(source.borrow().label, "candidate");
    assert_eq!(source.borrow().commits, 1);
    source.borrow_mut().pending = Some("discard".into());
    adapter.discard_staged();
    assert_eq!(source.borrow().label, "candidate");
    assert_eq!(source.borrow().discards, 1);
    source.borrow_mut().live = false;
    assert!(!adapter.staged_is_live());
}

#[test]
fn candidate_capture_does_not_keep_a_removed_registration_or_child_alive() {
    let mut registry = ProducerRegistry::new();
    registry.register(7, Source::new(), &[]).unwrap();
    let entry = registry.entries.get_mut(&7).unwrap();
    let mut candidate = Candidate::capture(entry);
    candidate.semantics = SemanticSnapshot::capture(&*entry.producer);
    assert!(candidate.belongs_to(entry));
    assert!(candidate.semantics.is_live());
    registry.remove(7);
    registry.register(7, Source::new(), &[]).unwrap();
    assert!(!candidate.belongs_to(&registry.entries[&7]));
    assert!(!candidate.semantics.is_live());
}

#[test]
fn accepted_semantics_and_child_tokens_do_not_delegate_to_pending_source_state() {
    let source = Rc::new(RefCell::new(Source::new()));
    let mut registry = ProducerRegistry::new();
    registry.register(7, source.clone(), &[]).unwrap();
    let entry = registry.entries.get_mut(&7).unwrap();
    entry.accepted_semantics = Some(SemanticSnapshot::capture(&*entry.producer));
    let accepted_child = registry.child_lifetime(7, 1).unwrap();
    source.borrow_mut().pending = Some("unpublished".into());
    source.borrow_mut().child = ProducerChildLifetime::new();
    assert_eq!(
        registry.semantics(7).unwrap().name.as_deref(),
        Some("accepted")
    );
    assert_eq!(registry.child_lifetime(7, 1), Some(accepted_child.clone()));
    assert!(!accepted_child.is_live());
    let action = ProducedAction {
        slot: 7,
        key: 1,
        id: "activate".into(),
    };
    assert_eq!(
        registry.act_checked(&action, Some(&accepted_child)),
        ProducerActionDispatch::StaleTarget
    );
}

struct Surface(Arc<genet_render_host::RenderCore>);
impl crate::Surface for Surface {
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
fn fixture() -> (crate::ScriptedDom, OwnedLayout) {
    let mut dom = crate::ScriptedDom::new();
    let name = |s| QualName::new(None, Namespace::from(""), LocalName::from(s));
    let node = dom.create_element(name("custom-leaf"));
    dom.set_attribute(node, name("key"), "7");
    dom.append_child(dom.document(), node);
    let layout = OwnedLayout::new(
        &dom,
        &["custom-leaf{display:block;width:8px;height:6px}"],
        160.,
        120.,
        &[],
        &Default::default(),
    );
    (dom, layout)
}
fn collect<T>(
    pending: &mut genet_render_host::PendingNativeFrame<T>,
) -> genet_render_host::AcceptedNativeFrame<T> {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(result) = pending.poll() {
            return result.unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "native producer work did not complete"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[ignore = "actual GPU; execute the serialized native producer gate"]
fn native_candidate_publishes_only_after_acceptance_and_discard_keeps_accepted_facts() {
    let core = Arc::new(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(8),
            ..Default::default()
        })
        .expect("actual native adapter required"),
    );
    let surface = Surface(core.clone());
    let (dom, layout) = fixture();
    let source = Rc::new(RefCell::new(Source::new()));
    let mut registry = ProducerRegistry::new();
    registry.bind_core(&core);
    registry.register(7, source.clone(), &[]).unwrap();
    let batch = core.try_begin_native_frame().unwrap();
    let stats = registry.prepare_window_staged(&surface, &layout, &dom, 1., &|_, _| false, &batch);
    assert_eq!(stats.stages, 1);
    assert!(registry.render_failure().is_none());
    assert_eq!(registry.image_identity(7), None);
    assert_eq!(registry.semantics(7), None);
    assert_eq!(registry.staged_image_identities().len(), 1);
    assert_eq!(source.borrow().commits, 0);
    let mut pending = batch.seal(());
    collect(&mut pending)
        .publish(|()| assert!(registry.publish_staged()))
        .unwrap();
    let accepted = registry.image_identity(7).unwrap();
    assert_eq!(
        registry.semantics(7).unwrap().name.as_deref(),
        Some("candidate")
    );
    assert_eq!(source.borrow().commits, 1);
    let accepted_child = registry.child_lifetime(7, 1).unwrap();

    let batch = core.try_begin_native_frame().unwrap();
    source.borrow_mut().next_label = "unpublished".into();
    registry.prepare_window_staged(&surface, &layout, &dom, 1., &|_, _| false, &batch);
    assert_eq!(registry.image_identity(7), Some(accepted));
    assert_eq!(
        registry.semantics(7).unwrap().name.as_deref(),
        Some("candidate")
    );
    let reserved = registry.staged_image_identities()[0].stamp;
    assert!(reserved > accepted.stamp);
    source.borrow_mut().live = false;
    assert!(!registry.staged_is_live());
    assert!(!registry.publish_staged());
    registry.discard_staged(Some(&core));
    assert_eq!(registry.image_identity(7), Some(accepted));
    assert_eq!(registry.child_lifetime(7, 1), Some(accepted_child));
    assert_eq!(source.borrow().discards, 1);
    assert_eq!(registry.next_stamp, reserved);
    let mut pending = batch.seal(());
    collect(&mut pending).publish(|()| ()).unwrap();
    source.borrow_mut().live = true;
    let batch = core.try_begin_native_frame().unwrap();
    registry.prepare_window_staged(&surface, &layout, &dom, 1., &|_, _| false, &batch);
    assert!(
        source.borrow().required,
        "discard must require a fresh producer draw"
    );
    assert!(registry.staged_image_identities()[0].stamp > reserved);
    registry.discard_staged(Some(&core));
    let mut pending = batch.seal(());
    collect(&mut pending).publish(|()| ()).unwrap();
}
