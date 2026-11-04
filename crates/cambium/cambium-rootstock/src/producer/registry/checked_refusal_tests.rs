// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
struct BoundSource {
    source: Rc<RefCell<String>>,
    suspended: Rc<Cell<usize>>,
}
impl TextureProducer for BoundSource {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
    fn suspend(&mut self) {
        self.suspended.set(self.suspended.get() + 1);
    }
    fn portable_scene(
        &mut self,
        _: &ProducerFrameInfo,
    ) -> Result<ProducedScene, PortableProducerError> {
        assert_eq!(&*self.source.borrow(), "retained scene authority");
        Ok(ProducedScene {
            scene: netrender::Scene::new(12, 9),
            source_identity: 73,
            source_revision: 19,
            alpha: SourceAlpha::Straight,
            encoding: SourceEncoding::Srgb,
        })
    }
}
#[test]
fn checked_refusal_invalidates_slot_without_retiring_bound_source() {
    let source = Rc::new(RefCell::new("retained scene authority".to_owned()));
    let suspended = Rc::new(Cell::new(0));
    let mut registry = ProducerRegistry::new();
    registry
        .register(
            7,
            BoundSource {
                source: source.clone(),
                suspended: suspended.clone(),
            },
            &[],
        )
        .unwrap();
    let entry = registry.entries.get_mut(&7).unwrap();
    entry.active = true;
    entry.logical_size = (12.0, 9.0);
    // This CPU control exercises real slot invalidation and source binding;
    // only the separate GPU gate can populate a hardware texture/view.
    refuse_render_entry(entry);
    assert!(!entry.active);
    assert!(entry.image.is_none());
    let portable = entry
        .producer
        .portable_scene(&ProducerFrameInfo {
            purpose: ProducerFramePurpose::Export,
            logical_size: (12.0, 9.0),
            physical_size: [12, 9],
            layout_scale: 1.0,
            needs_frame: true,
            timestamp: None,
            appearance: ResolvedAppearance::default(),
        })
        .unwrap();
    assert_eq!(
        (portable.source_identity, portable.source_revision),
        (73, 19)
    );
    assert_eq!(registry.error(7), Some(ProducerError::RenderRefused));
    assert!(registry.commands(7).unwrap().is_empty());
    assert!(registry.contains(7));
    assert_eq!(suspended.get(), 1);
    assert_eq!(&*source.borrow(), "retained scene authority");
}
