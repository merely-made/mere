// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Event-driven shared mere panel over one admitted endpoint. SessionDriver
//! owns wire correlation and resource validation; this adapter sequences the
//! sessions, graph and optional archive readings and never owns a store.
use crate::{
    route as mere_route,
    wire::{MERE_ARCHIVE, MERE_GRAPH, MERE_SESSIONS},
};
use chirograph::{
    CapabilityProfile, ContentHash, EndpointDescriptor, ProjectionSession, ProjectionSnapshot,
    ProtocolVersion,
};
use graphshell_client::core::Outcome;
use graphshell_client::driver::Advance;
use graphshell_client::{MountedScene, ResolvedContent, SessionDriver};
use mere_view::{
    HostAction, MereViewModel, MereViewRequest, MergeOption, MergeRequest, StatusNote, ViewStatus,
};
use sceno::InstanceId;
use std::collections::{BTreeMap, HashMap, VecDeque};
#[derive(Clone)]
enum Task {
    Mount(usize),
    Resolve(ProjectionSession, InstanceId, ContentHash),
    Invoke(MereViewRequest),
    RecordAction {
        key: String,
        target: graphshell_client::ActionDraftTarget,
        action: chirograph::AdvertisedAction,
        command: hest::Invocation,
    },
    Media(ProjectionSession, InstanceId),
    /// A pending merge card's material in the sessions projection.
    MergeMedia(ProjectionSession, InstanceId),
}
pub struct MereRouteSession {
    /// The admitted application this panel reads as. The owner discloses
    /// only that application's own parallel view state.
    application: String,
    driver: SessionDriver,
    indices: Vec<usize>,
    sessions: ProjectionSession,
    graph: ProjectionSession,
    archive: Option<ProjectionSession>,
    tasks: VecDeque<Task>,
    current: Option<Task>,
    cards: HashMap<ContentHash, Vec<u8>>,
    outbox: Vec<String>,
    model: MereViewModel,
    notice: Option<String>,
    closed: bool,
    primary: Option<MountedScene>,
    saved_view: pandect::ViewIntent,
    read_only: bool,
    material: Option<crate::wire::SessionGraphV1>,
    records: HashMap<String, chirograph::PortableCardV1>,
    inspected: Option<String>,
    transient_layout: Option<String>,
    /// The pending merge the owner discloses, and the choices made on it here
    /// so far, by conflict key. Choices stay with this panel until Apply.
    merge: Option<crate::wire::PendingMergeV1>,
    merge_choices: BTreeMap<String, MergeOption>,
}

/// The panel over a native admitted stream: `graphshell_client::pump`
/// carries its lines (transport agnosticism note, T3), as a browser's data
/// channel does. A failure is the panel's own (`fail`), so the pump goes on
/// only while the panel is open.
impl graphshell_client::LineDriven for MereRouteSession {
    fn take_outgoing(&mut self) -> Vec<String> {
        MereRouteSession::take_outgoing(self)
    }

    fn on_line(&mut self, line: &str) -> Result<(), String> {
        MereRouteSession::on_line(self, line);
        if !self.closed {
            return Ok(());
        }
        Err(match &self.model.status {
            ViewStatus::Unavailable(note) => note.message.clone(),
            _ => "The resident mere is unavailable.".into(),
        })
    }
}

#[cfg(test)]
#[path = "route_session/busy_tests.rs"]
mod busy_tests;

#[path = "route_session/progress.rs"]
mod progress;

#[path = "route_session/operations.rs"]
mod operations;
