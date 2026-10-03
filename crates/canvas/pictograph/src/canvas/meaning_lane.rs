// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The off-thread Meaning lane: an [`armillary`] actor that runs one
//! embedding run per request and emits the snapshot with the content key it
//! was computed for. Used only on a native offloaded canvas, like the
//! community lane; the owner drops a result whose key no longer matches the
//! graph's content. (Dynamics grammar plan, G2.)

use std::sync::mpsc::Receiver;

use armillary::{ActorHandle, Emitter, Wake, spawn};
use esp::embed::EmbedError;

use super::meaning::{MeaningRequest, MeaningSnapshot, compute_meaning};

/// A finished run, with what it was computed for.
pub(crate) struct MeaningUpdate {
    pub content_key: u64,
    pub generation: u64,
    pub result: Result<MeaningSnapshot, EmbedError>,
}

/// The owner's side of the Meaning worker. Dropping it ends the thread.
pub(crate) struct MeaningActor {
    handle: ActorHandle<MeaningRequest>,
    updates: Receiver<MeaningUpdate>,
    /// The `(content key, generation)` in flight, so a stable graph is not
    /// re-dispatched while its run is pending.
    inflight: Option<(u64, u64)>,
}

impl MeaningActor {
    pub fn spawn(wake: Wake) -> Self {
        let (handle, updates) = spawn(wake, |commands, out: Emitter<MeaningUpdate>| {
            while let Ok(request) = commands.recv() {
                let result = compute_meaning(&request);
                out.emit(MeaningUpdate {
                    content_key: request.content_key,
                    generation: request.generation,
                    result,
                });
            }
        });
        Self {
            handle,
            updates,
            inflight: None,
        }
    }

    /// Dispatch a run, unless one for the same content and engine is pending.
    pub fn request(&mut self, request: MeaningRequest) {
        let tag = (request.content_key, request.generation);
        if self.inflight == Some(tag) {
            return;
        }
        if self.handle.command(request) {
            self.inflight = Some(tag);
        }
    }

    /// The `(content key, generation)` of the run in flight.
    pub fn inflight(&self) -> Option<(u64, u64)> {
        self.inflight
    }

    /// The freshest finished run, if any; older ones are superseded.
    pub fn drain(&mut self) -> Option<MeaningUpdate> {
        let mut latest = None;
        while let Ok(update) = self.updates.try_recv() {
            latest = Some(update);
        }
        if let Some(update) = &latest {
            if self.inflight == Some((update.content_key, update.generation)) {
                self.inflight = None;
            }
        }
        latest
    }
}
