// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The file seam: a host's platform chooser answers a view's file request.
//!
//! A view asks through [`cambium::open_file`]. After the dispatch that made
//! the request, the host hands it to its [`FileChooser`], and the chooser
//! answers through a [`FileAnswer`], at once as a desktop dialog does or later
//! as a browser does. Answers wait for the host's next frame, which dispatches
//! each to the element that asked. A host with no chooser answers with nothing
//! chosen, so the view is never left waiting.

use std::sync::{Arc, Mutex};

use cambium::{FileEvent, FileRequest};

use crate::{HostWake, NodeId};

/// Answers waiting for the host's next frame, each with the element it goes to.
pub(crate) type FileAnswers = Arc<Mutex<Vec<(NodeId, FileEvent)>>>;

/// A platform's file chooser: a browser's file input, a desktop dialog, or a
/// test's prepared files.
pub trait FileChooser {
    /// Show the chooser for `request`, and answer through `answer` once the
    /// person has chosen or cancelled.
    fn open(&mut self, request: &FileRequest, answer: FileAnswer);
}

/// How a chooser answers one request.
pub struct FileAnswer {
    node: NodeId,
    answers: FileAnswers,
    wake: HostWake,
}

impl FileAnswer {
    pub(crate) fn new(node: NodeId, answers: FileAnswers, wake: HostWake) -> Self {
        Self {
            node,
            answers,
            wake,
        }
    }

    /// Answer with the files chosen, or none when the person cancelled, and
    /// wake the host to deliver it.
    pub fn send(self, event: FileEvent) {
        if let Ok(mut answers) = self.answers.lock() {
            answers.push((self.node, event));
        }
        self.wake.wake();
    }
}
