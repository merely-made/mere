// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Browser readback for Mesquite. The shared lane owns capture completion.
use crate::capture::{PendingFrame, capture_into};
use cambium_rootstock::{CaptureFn, StampedCaptureFn};
use mesquite::{CaptureBackend, Readback, StampedFrame, StampedReadback};
use std::{cell::RefCell, rc::Rc};

/// Asynchronous captures delivered to the product without filesystem writes.
#[derive(Default)]
pub struct WebCapture;
impl CaptureBackend for WebCapture {
    fn arm_stamped(
        &mut self,
        capture: &mut Option<StampedCaptureFn>,
        identity: mesquite::CaptureRequest,
    ) -> Result<StampedReadback, String> {
        let slot = Rc::new(RefCell::new(None));
        let sink = slot.clone();
        *capture = Some(Box::new(move |surface, view, presentation| {
            *sink.borrow_mut() = Some((
                presentation,
                crate::capture::start(surface, view, presentation.width, presentation.height),
            ));
        }));
        Ok(Box::new(move || {
            if !slot
                .borrow()
                .as_ref()
                .is_some_and(|(_, frame): &(_, PendingFrame)| frame.ready())
            {
                return None;
            }
            let (presentation, frame) = slot.borrow_mut().take()?;
            Some(frame.take().map(|pixels| StampedFrame {
                identity: identity.clone(),
                presentation,
                pixels,
            }))
        }))
    }
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
