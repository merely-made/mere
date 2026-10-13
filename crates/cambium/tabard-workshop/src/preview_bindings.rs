// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The same production preview bindings for native and browser hosts.
use crate::{
    GRAPH_LEAF_KEY, PreviewScene, READER_LEAF_KEY, ReaderSpecimen, STYLESHEET_LEAF_KEY,
    StylesheetSpecimen, WorkshopState,
};
use cambium_rootstock::{ProducerRole, ProducerSemantics, SceneProducer};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

fn scene_producer<T: PreviewScene>(source: Rc<RefCell<T>>, key: u64) -> SceneProducer<T> {
    SceneProducer::new(source, key, T::frame, T::revision, |source| {
        Some(ProducerSemantics {
            role: Some(ProducerRole::Image),
            name: Some(source.accessible_name().into()),
            children: Vec::new(),
        })
    })
}

/// Retained preview producers shared by standalone and embedded workshops.
/// Call `register` on every frame with the current workshop value. The binding
/// owns producer identity while the workshop owns preview source replacement.
/// Leaf keys are reserved by Tabard within the caller's host scope.
pub struct PreviewBindings {
    pub(super) raster_base: u64,
    pub(super) reader: Option<Rc<RefCell<SceneProducer<ReaderSpecimen>>>>,
    pub(super) stylesheet: Option<Rc<RefCell<SceneProducer<StylesheetSpecimen>>>>,
}

// A render core can serve several independent workshop windows. Their
// retained raster outputs need distinct identities even when leaf keys and
// initial revision numbers match. Allocation happens once per binding.
static NEXT_RASTER_BASE: AtomicU64 = AtomicU64::new(0x7461_6200_0000_0000);

impl Default for PreviewBindings {
    fn default() -> Self {
        let raster_base = NEXT_RASTER_BASE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |key| {
                key.checked_add(2)
            })
            .expect("Tabard preview raster identity exhausted");
        Self {
            raster_base,
            reader: None,
            stylesheet: None,
        }
    }
}

impl PreviewBindings {
    pub fn register(
        &mut self,
        state: &WorkshopState,
        leaves: &mut sprigging::LeafRegistry<u64>,
        producers: &mut cambium_rootstock::ProducerRegistry,
    ) {
        leaves.insert(GRAPH_LEAF_KEY, Box::new(state.graph_leaf()));
        let reader_key = self.raster_base;
        let stylesheet_key = self.raster_base + 1;
        let source = state.reader_preview();
        let producer = self.reader.get_or_insert_with(|| {
            Rc::new(RefCell::new(scene_producer(source.clone(), reader_key)))
        });
        producer.borrow_mut().set_source(source);
        if !producers.contains(READER_LEAF_KEY) {
            producers
                .register(READER_LEAF_KEY, producer.clone(), &[])
                .expect("reserved Tabard reader key");
        }
        let source = state.stylesheet_preview();
        let producer = self.stylesheet.get_or_insert_with(|| {
            Rc::new(RefCell::new(scene_producer(source.clone(), stylesheet_key)))
        });
        producer.borrow_mut().set_source(source);
        if !producers.contains(STYLESHEET_LEAF_KEY) {
            producers
                .register(STYLESHEET_LEAF_KEY, producer.clone(), &[])
                .expect("reserved Tabard stylesheet key");
        }
    }
}
