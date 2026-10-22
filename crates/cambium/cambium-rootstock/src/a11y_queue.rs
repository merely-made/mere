// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Private queue admission bindings. These retain no document or contributor.
//! Child-key lifetimes within one producer still belong to its authored model.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use crate::{
    A11yDispatch, A11yReceipt, A11yRejection, A11yRequest, NodeId, ProducerRegistry, ScriptedDom,
};

pub(crate) struct QueuedA11y {
    pub request: A11yRequest,
    reply: Rc<Cell<Option<A11yDispatch>>>,
    dom: Weak<RefCell<ScriptedDom>>,
    mount: NodeId,
}

impl QueuedA11y {
    pub fn new(
        request: A11yRequest,
        owner: &Rc<Cell<bool>>,
        dom: &Rc<RefCell<ScriptedDom>>,
        mount: NodeId,
        producers: &ProducerRegistry,
        leaves: &sprigging::LeafRegistry<u64>,
    ) -> (Self, A11yReceipt) {
        let request = {
            let document = dom.borrow();
            request.bind_publication(
                &crate::A11yOwner::new(owner),
                &crate::WindowDom::new(&document, mount),
                leaves,
                producers,
            )
        };
        let reply = Rc::new(Cell::new(None));
        (
            Self {
                request,
                reply: reply.clone(),
                dom: Rc::downgrade(dom),
                mount,
            },
            A11yReceipt(reply),
        )
    }

    pub fn is_current(
        &self,
        owner: &Rc<Cell<bool>>,
        dom: &Rc<RefCell<ScriptedDom>>,
        mount: NodeId,
        producers: &ProducerRegistry,
        leaves: &sprigging::LeafRegistry<u64>,
    ) -> bool {
        owner.get() && self.dom.ptr_eq(&Rc::downgrade(dom)) && self.mount == mount && {
            let document = dom.borrow();
            self.request.publication_is_current(
                &crate::A11yOwner::new(owner),
                &crate::WindowDom::new(&document, mount),
                leaves,
                producers,
            )
        }
    }

    pub fn finish(&self, result: A11yDispatch) {
        self.reply.set(Some(result));
    }
}

impl Drop for QueuedA11y {
    fn drop(&mut self) {
        if self.reply.get().is_none() {
            self.finish(A11yDispatch::Rejected(A11yRejection::Cancelled));
        }
    }
}
