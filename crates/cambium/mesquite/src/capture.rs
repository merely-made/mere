// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Platform readback, leaving capture scheduling and completion in the lane.
use cambium_rootstock::{CaptureFn, Frame, PresentedFrame, StampedCaptureFn, read_frame};
use std::{cell::RefCell, rc::Rc};

/// Polled once per presented frame until pixels or an error arrive.
pub type Readback = Box<dyn FnMut() -> Option<Result<Frame, String>>>;

pub struct StampedFrame {
    pub identity: crate::CaptureRequest,
    pub presentation: PresentedFrame,
    pub pixels: Frame,
}
pub type StampedReadback = Box<dyn FnMut() -> Option<Result<StampedFrame, String>>>;

/// A host's readback mechanism. Browser implementations must not block.
pub trait CaptureBackend {
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback;
    /// Pairing is opt-in. An older backend must refuse it rather than invent
    /// the identity of pixels obtained through its unstamped path.
    fn arm_stamped(
        &mut self,
        _capture: &mut Option<StampedCaptureFn>,
        _identity: crate::CaptureRequest,
    ) -> Result<StampedReadback, String> {
        Err("capture backend does not support presented-state pairing".into())
    }
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
    fn arm_stamped(
        &mut self,
        capture: &mut Option<StampedCaptureFn>,
        identity: crate::CaptureRequest,
    ) -> Result<StampedReadback, String> {
        let slot = Rc::new(RefCell::new(None));
        let sink = slot.clone();
        *capture = Some(Box::new(move |surface, view, presentation| {
            *sink.borrow_mut() = Some(
                read_frame(surface, view, presentation.width, presentation.height)
                    .ok_or_else(|| "native frame readback failed".to_owned())
                    .map(|pixels| StampedFrame {
                        identity,
                        presentation,
                        pixels,
                    }),
            );
        }));
        Ok(Box::new(move || slot.borrow_mut().take()))
    }
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
