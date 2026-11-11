// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A physics Worker in process: [`seiche::PhysicsWorker`] behind a transport
//! that moves the same bytes a Web Worker's `postMessage` would, one interval
//! of the far side per [`Loopback::pump`], so a test drives the canvas's remote
//! backend deterministically.

use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use seiche::{PhysicsWorker, Transport};

fn clock() -> Duration {
    Duration::ZERO
}

struct Hub {
    worker: PhysicsWorker,
    inbox: VecDeque<Vec<u8>>,
}

#[derive(Clone)]
pub(crate) struct Loopback(Rc<RefCell<Hub>>);

impl Loopback {
    pub(crate) fn new() -> Self {
        Self(Rc::new(RefCell::new(Hub {
            worker: PhysicsWorker::new(clock),
            inbox: VecDeque::new(),
        })))
    }

    /// One interval of the far side, as the Worker's timer would run it.
    pub(crate) fn pump(&self) {
        let mut hub = self.0.borrow_mut();
        if let Some(update) = hub.worker.interval() {
            hub.inbox.push_back(update);
        }
    }

    pub(crate) fn transport(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }
}

impl Transport for Loopback {
    fn send(&mut self, message: Vec<u8>) {
        let mut hub = self.0.borrow_mut();
        let replies = hub.worker.receive(&message).expect("a whole message");
        hub.inbox.extend(replies);
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        self.0.borrow_mut().inbox.pop_front()
    }
}
