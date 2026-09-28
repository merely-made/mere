// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Browser readback for Mesquite. The shared lane owns capture completion.
use crate::capture::{PendingFrame, capture_into};
use cambium_rootstock::CaptureFn;
use mesquite::{CaptureBackend, Readback};
use std::{cell::RefCell, rc::Rc};

/// Asynchronous captures delivered to the product without filesystem writes.
#[derive(Default)]
pub struct WebCapture;
impl CaptureBackend for WebCapture {
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback {
        let slot = Rc::new(RefCell::new(None));
        *capture = Some(capture_into(slot.clone()));
        Box::new(move || {
            if !slot.borrow().as_ref().is_some_and(PendingFrame::ready) {
                return None;
            }
            Some(slot.borrow_mut().take()?.take())
        })
    }
    fn writes_files(&self) -> bool {
        false
    }
    fn patience(&self) -> u64 {
        120
    }
}
