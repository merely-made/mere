// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The browser's half of the scenario lane.
//!
//! The lane is rootstock's ([`cambium_rootstock::scenario`]). A browser reads a
//! frame back a few frames after it was presented ([`PendingFrame`]), keeps no
//! files, and closes nothing when a run ends. The page decides what a capture
//! and a receipt become, through the two callbacks it gives [`WebLane::new`].

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use cambium::OpenedFile;
use cambium_rootstock::scenario::LaneHost;
use cambium_rootstock::{CaptureFn, Frame};

use crate::capture::{PendingFrame, capture_into};

/// What the page does with a captured frame.
type Keep = Box<dyn FnMut(&str, &Frame)>;
/// What the page does with the finished receipt, and whether it passed.
type Publish = Box<dyn FnMut(bool, &str)>;

/// The browser's half of a lane: frames read back asynchronously, and
/// evidence handed to the page.
pub struct WebLane {
    slot: Rc<RefCell<Option<PendingFrame>>>,
    keep: Keep,
    publish: Publish,
}

impl WebLane {
    /// `keep` receives each captured frame, and `publish` the finished
    /// receipt with whether it passed.
    pub fn new(
        keep: impl FnMut(&str, &Frame) + 'static,
        publish: impl FnMut(bool, &str) + 'static,
    ) -> Self {
        Self {
            slot: Rc::default(),
            keep: Box::new(keep),
            publish: Box::new(publish),
        }
    }
}

impl LaneHost for WebLane {
    fn arm_capture(
        &mut self,
        capture: &mut Option<CaptureFn>,
    ) -> Box<dyn FnMut() -> Option<Frame>> {
        *capture = Some(capture_into(self.slot.clone()));
        let slot = self.slot.clone();
        Box::new(move || {
            if !slot.borrow().as_ref().is_some_and(PendingFrame::ready) {
                return None;
            }
            // A frame that fails to map never lands, and the lane reports it
            // as lost once its patience runs out.
            slot.borrow_mut().take()?.take().ok()
        })
    }

    fn keep(&mut self, name: &str, frame: &Frame) -> Result<Option<PathBuf>, String> {
        (self.keep)(name, frame);
        Ok(None)
    }

    fn open_file(&mut self, argument: &str) -> Result<OpenedFile, String> {
        Err(format!("file {argument}: a browser lane reads no files"))
    }

    fn publish(&mut self, ok: bool, text: &str) {
        (self.publish)(ok, text);
    }

    fn closes_on_finish(&self) -> bool {
        false
    }
}
