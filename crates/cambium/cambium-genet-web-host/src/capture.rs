// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Browser captures use the shared nonblocking, loss-cancellable readback.
use cambium_rootstock::{CaptureFn, Frame, Surface};
use std::{cell::RefCell, rc::Rc};

pub struct PendingFrame {
    inner: RefCell<Option<cambium_rootstock::PendingFrame>>,
    result: RefCell<Option<Result<Frame, String>>>,
}
impl PendingFrame {
    pub fn ready(&self) -> bool {
        if self.result.borrow().is_some() {
            return true;
        }
        let result = self
            .inner
            .borrow_mut()
            .as_mut()
            .and_then(|pending| pending.poll());
        if let Some(result) = result {
            self.inner.borrow_mut().take();
            *self.result.borrow_mut() = Some(result.map_err(|error| error.to_string()));
            true
        } else {
            false
        }
    }
    pub fn take(self) -> Result<Frame, String> {
        self.ready();
        self.result
            .into_inner()
            .ok_or_else(|| "browser readback not ready".to_owned())?
    }
}
pub fn capture_into(slot: Rc<RefCell<Option<Result<PendingFrame, String>>>>) -> CaptureFn {
    Box::new(move |capture| {
        *slot.borrow_mut() = Some(
            capture
                .map_err(|error| error.to_string())
                .and_then(|frame| start(frame.surface, frame.view, frame.width, frame.height)),
        );
    })
}
pub(crate) fn start(
    surface: &dyn Surface,
    view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Result<PendingFrame, String> {
    cambium_rootstock::start_frame_readback(surface, view, width, height).map(|inner| {
        PendingFrame {
            inner: RefCell::new(Some(inner)),
            result: RefCell::new(None),
        }
    })
}
