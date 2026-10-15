// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
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
        *capture = Some(Box::new(move |capture| {
            *sink.borrow_mut() = Some(
                capture
                    .map_err(|error| error.to_string())
                    .map(PendingFrame::new),
            );
        }));
        Ok(Box::new(move || {
            if !slot
                .borrow()
                .as_ref()
                .is_some_and(|result: &Result<PendingFrame, String>| {
                    result.is_err() || result.as_ref().is_ok_and(|frame| frame.ready())
                })
            {
                return None;
            }
            let frame = match slot.borrow_mut().take()? {
                Ok(frame) => frame,
                Err(error) => return Some(Err(error)),
            };
            Some(
                frame
                    .take_stamped()
                    .map(|(presentation, pixels)| StampedFrame {
                        identity: identity.clone(),
                        presentation,
                        pixels,
                    }),
            )
        }))
    }
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback {
        let slot = Rc::new(RefCell::new(None));
        *capture = Some(capture_into(slot.clone()));
        Box::new(move || {
            if !slot
                .borrow()
                .as_ref()
                .is_some_and(|result: &Result<PendingFrame, String>| {
                    result.is_err() || result.as_ref().is_ok_and(|frame| frame.ready())
                })
            {
                return None;
            }
            Some(slot.borrow_mut().take()?.and_then(PendingFrame::take))
        })
    }
    fn writes_files(&self) -> bool {
        false
    }
    fn patience(&self) -> u64 {
        120
    }
}
