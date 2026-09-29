// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Platform readback, leaving capture scheduling and completion in the lane.
use cambium_rootstock::{CaptureFn, Frame, read_frame};
use std::{cell::RefCell, rc::Rc};

/// Polled once per presented frame until pixels or an error arrive.
pub type Readback = Box<dyn FnMut() -> Option<Result<Frame, String>>>;

/// A host's readback mechanism. Browser implementations must not block.
pub trait CaptureBackend {
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback;
    /// Whether named scenario captures receive filesystem paths. Browser
    /// products publish pixels through `Product::inspect` instead.
    fn writes_files(&self) -> bool {
        true
    }
    /// Maximum presented frames to wait for an asynchronous readback.
    fn patience(&self) -> u64 {
        8
    }
}

pub(crate) struct NativeCapture;
impl CaptureBackend for NativeCapture {
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback {
        let slot = Rc::new(RefCell::new(None));
        let sink = slot.clone();
        *capture = Some(Box::new(move |surface, view, width, height| {
            *sink.borrow_mut() = Some(
                read_frame(surface, view, width, height)
                    .ok_or_else(|| "native frame readback failed".to_owned()),
            );
        }));
        Box::new(move || slot.borrow_mut().take())
    }
}
