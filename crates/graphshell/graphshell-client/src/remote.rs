// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A remote session's operation sequencing, with the transport left out.
//!
//! [`SessionDriver`] answers "what goes on the wire next" for one operation.
//! A host presenting a remote board needs the layer above it: which operation
//! is in flight, what each answer implies next, and what the person is told.
//! That layer lived in the browser page, welded to its host. It lives here so
//! every host — both browser pages and a native one — runs the same rules, and
//! so those rules run under an ordinary `cargo test`.
//!
//! ## The rules
//!
//! One operation at a time, remembered as the [`RemoteOp`] in flight and
//! finished when the endpoint answers:
//!
//! - discovery is followed by a mount, or, on a link that already mounted
//!   (a reconnect), by a poll that keeps the mount;
//! - an accepted intent is followed by a poll, because the bell it rang is
//!   written on the next round trip;
//! - a rejected intent is followed by a resnapshot, so "unchanged" is
//!   measured rather than assumed — a refusal rings no bell;
//! - every queued bell drives a resume, which the endpoint answers by diff.
//!
//! ## What the host does
//!
//! The session never sends. Lines it wants written queue in an outbox the
//! host drains ([`RemoteSession::take_outgoing`]) onto whatever carries them;
//! lines that arrive go to [`RemoteSession::on_line`]. Observable steps queue
//! as event lines ([`RemoteSession::take_events`]) for the host's receipts.
//! Joining, rejoining and the transport itself stay the host's: it reports
//! them through the lifecycle methods ([`RemoteSession::joined`],
//! [`RemoteSession::rejoined`], [`RemoteSession::channel_closed`], ...).

use std::collections::{BTreeSet, VecDeque};

use chirograph::{AdvertisedAction, CapabilityProfile, IntentResult, ProjectionSession};
use sceno::InstanceId;

use crate::action_draft::{ActionDraft, ActionDraftTarget};
use crate::core::{Outcome, Progress};
use crate::driver::{Advance, SessionDriver};
use crate::{ClientState, MountedScene};

/// The operation whose answer is awaited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteOp {
    Discover,
    Mount,
    Resnapshot,
    Invoke,
    Poll,
    Resume,
}

/// The action surface: its status line, how many invocations were sent, and
/// the draft open for the person to fill, with the position it targets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionForm {
    pub status: String,
    pub count: u32,
    pub draft: Option<ActionDraft>,
    pub target: Option<ActionDraftTarget>,
}

impl ActionForm {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            ..Self::default()
        }
    }

    /// Drop the open draft and its target.
    pub fn close(&mut self) {
        self.draft = None;
        self.target = None;
    }

    /// Open `action` as a draft against `target` at the observed position.
    /// Returns whether it waits for values (a bounded form); a plain action
    /// is ready to submit as it stands.
    pub fn open_action(
        &mut self,
        session: ProjectionSession,
        target: InstanceId,
        action: AdvertisedAction,
        observed: (scenotime::SceneEpoch, scenotime::Revision),
    ) -> bool {
        let bounded = action.input_form.is_some();
        if bounded {
            self.status = format!("Choose values · {}", action.label);
        }
        self.draft = Some(ActionDraft::new(action));
        self.target = Some(ActionDraftTarget {
            session,
            target,
            observed_epoch: observed.0,
            observed_revision: observed.1,
        });
        bounded
    }

    /// Open the first bounded form `session` advertises, at its mounted
    /// position. Plain actions are not drafts; when none is bounded the status
    /// says how many actions are on offer.
    pub fn open_first_bounded(
        &mut self,
        client: &ClientState,
        session: &ProjectionSession,
        profile: &CapabilityProfile,
    ) {
        let Some((observed_epoch, observed_revision)) = client
            .mounted(session)
            .map(|mounted| (mounted.scene.epoch, mounted.scene.revision))
        else {
            self.status = "Failed · remote projection is not mounted".to_string();
            return;
        };
        let tree = match client.accessibility_tree(session, profile) {
            Ok(tree) => tree,
            Err(error) => {
                self.status = format!("Failed · remote accessibility tree: {error:?}");
                return;
            },
        };
        let Some((target, action)) = tree.children.iter().find_map(|item| {
            item.actions
                .iter()
                .find(|action| action.input_form.is_some())
                .cloned()
                .map(|action| (item.instance, action))
        }) else {
            let count: usize = tree.children.iter().map(|item| item.actions.len()).sum();
            self.status = format!("{count} remote action(s) advertised");
            return;
        };
        self.status = format!("Choose values · {}", action.label);
        self.draft = Some(ActionDraft::new(action));
        self.target = Some(ActionDraftTarget {
            session: session.clone(),
            target,
            observed_epoch,
            observed_revision,
        });
    }

    /// Select one advertised value in the open draft.
    pub fn choose(&mut self, field: &str, value: &str) {
        let Some(draft) = self.draft.as_mut() else {
            self.status = "Failed · no remote action draft is open".to_string();
            return;
        };
        self.status = match draft.choose(field, value) {
            Ok(()) => format!("Selected {field}"),
            Err(error) => format!("Choose values · {error}"),
        };
    }
}

/// The actions a mounted scene advertises, one per intent in tree order, each
/// with the first instance that offers it as its target. The surface offers
/// what the endpoint offers, not one button per card.
pub fn advertised_actions(
    client: &ClientState,
    session: &ProjectionSession,
    profile: &CapabilityProfile,
) -> Vec<(InstanceId, AdvertisedAction)> {
    let Ok(tree) = client.accessibility_tree(session, profile) else {
        return Vec::new();
    };
    let mut seen = BTreeSet::new();
    tree.children
        .iter()
        .flat_map(|item| {
            item.actions
                .iter()
                .cloned()
                .map(move |action| (item.instance, action))
        })
        .filter(|(_, action)| seen.insert(action.intent.0.clone()))
        .collect()
}

/// The names a mounted scene's presentation semantics give its cards.
pub fn card_labels(
    client: &ClientState,
    session: &ProjectionSession,
    profile: &CapabilityProfile,
) -> Vec<String> {
    client
        .accessibility_tree(session, profile)
        .map(|tree| tree.children.into_iter().map(|item| item.label).collect())
        .unwrap_or_default()
}

/// One remote endpoint over a line transport the host owns. See the module
/// docs.
pub struct RemoteSession {
    driver: SessionDriver,
    profile: CapabilityProfile,
    pending: Option<RemoteOp>,
    /// The acknowledged revision before the resume in flight, for the
    /// `diff · a → b` line.
    resume_before: Option<u64>,
    session: Option<ProjectionSession>,
    status: String,
    /// A transport step in flight that is not a protocol operation: a
    /// disconnect landing, a rejoin, a nudge.
    link_busy: bool,
    /// Set by [`RemoteSession::disconnect`], so a closing channel reads
    /// "disconnected" rather than a host that went away.
    closing: bool,
    rejoins: u32,
    last_resume: String,
    /// The action surface. Public so a host can carry its own form across
    /// when a session replaces another link.
    pub form: ActionForm,
    outbox: VecDeque<String>,
    events: Vec<String>,
}

impl RemoteSession {
    /// A session for an endpoint not yet discovered.
    pub fn new(profile: CapabilityProfile) -> Self {
        Self {
            driver: SessionDriver::new(profile.clone()),
            profile,
            pending: None,
            resume_before: None,
            session: None,
            status: "discovering".to_string(),
            link_busy: false,
            closing: false,
            rejoins: 0,
            last_resume: String::new(),
            form: ActionForm::new("Ready"),
            outbox: VecDeque::new(),
            events: Vec::new(),
        }
    }

    // ── Reading ─────────────────────────────────────────────────────────────

    pub fn driver(&self) -> &SessionDriver {
        &self.driver
    }

    pub fn profile(&self) -> &CapabilityProfile {
        &self.profile
    }

    /// The client holding the scene, once discovery has answered.
    pub fn client(&self) -> Option<&ClientState> {
        self.driver.core().map(|core| core.client())
    }

    /// The mounted projection's session, once mounted.
    pub fn session(&self) -> Option<&ProjectionSession> {
        self.session.as_ref()
    }

    pub fn mounted(&self) -> Option<&MountedScene> {
        self.client()?.mounted(self.session.as_ref()?)
    }

    /// The acknowledged revision, if mounted.
    pub fn revision(&self) -> Option<u64> {
        self.client()?
            .acknowledgement(self.session.as_ref()?)
            .map(|ack| ack.revision.0)
    }

    /// The authority-emitted generation the mounted scene carries.
    pub fn generation(&self) -> Option<u64> {
        self.mounted().map(|mounted| mounted.scene.tables.generation)
    }

    pub fn pending(&self) -> Option<RemoteOp> {
        self.pending
    }

    /// Whether an answer is still to come: an operation, a request on the
    /// wire, a queued bell, or a transport step. What a scenario `wait` holds
    /// on.
    pub fn in_flight(&self) -> bool {
        self.link_busy
            || self.pending.is_some()
            || self.driver.is_awaiting()
            || self.driver.queued_notices() > 0
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    /// How the last resume ended: `diff · a → b` or `already current at r`.
    pub fn last_resume(&self) -> &str {
        &self.last_resume
    }

    pub fn rejoins(&self) -> u32 {
        self.rejoins
    }

    pub fn is_closing(&self) -> bool {
        self.closing
    }

    /// The active-session line.
    pub fn label(&self) -> String {
        match self.mounted() {
            Some(mounted) => format!(
                "Remote projection · {} objects · revision {}",
                mounted.scene.tables.items.len(),
                mounted.scene.revision.0
            ),
            None => format!("Remote projection · {}", self.status),
        }
    }

    /// The selection line and the address: the endpoint's label and the
    /// mounted session.
    pub fn selection(&self) -> (String, String) {
        let label = self
            .driver
            .core()
            .map(|core| core.descriptor().label.clone())
            .unwrap_or_else(|| "joining".to_string());
        let address = self
            .session
            .as_ref()
            .map(|session| session.0.clone())
            .unwrap_or_default();
        (label, address)
    }

    /// The advertised actions; see [`advertised_actions`].
    pub fn actions(&self) -> Vec<(InstanceId, AdvertisedAction)> {
        match (self.client(), self.session.as_ref()) {
            (Some(client), Some(session)) => advertised_actions(client, session, &self.profile),
            _ => Vec::new(),
        }
    }

    pub fn card_labels(&self) -> Vec<String> {
        match (self.client(), self.session.as_ref()) {
            (Some(client), Some(session)) => card_labels(client, session, &self.profile),
            _ => Vec::new(),
        }
    }

    // ── The host's queues ───────────────────────────────────────────────────

    /// Lines to write to the endpoint, in order.
    pub fn take_outgoing(&mut self) -> Vec<String> {
        self.outbox.drain(..).collect()
    }

    /// Steps worth a receipt line, in order.
    pub fn take_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.events)
    }

    // ── The link's lifecycle, reported by the host ──────────────────────────

    /// The first link is up: discover.
    pub fn joined(&mut self) {
        self.status = "discovering".to_string();
        self.events.push("remote-joined".to_string());
        self.begin_discovery();
    }

    /// A transport status line while the link is being built.
    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    /// Close from this end. The driver, its core and the mount are kept for
    /// a reconnect; the host closes its channel.
    pub fn disconnect(&mut self) {
        self.closing = true;
        self.pending = None;
        self.driver.disconnect();
        self.link_busy = true;
        self.status = "disconnecting".to_string();
        self.events.push("remote-disconnect".to_string());
    }

    /// The channel closed: on our word after a disconnect, or the host's.
    pub fn channel_closed(&mut self) {
        self.status = if self.closing {
            "disconnected".to_string()
        } else {
            "closed: the host closed the channel".to_string()
        };
        self.link_busy = false;
        self.events.push("remote-channel-closed".to_string());
    }

    /// A transport step failed: a read, a rejoin.
    pub fn link_failed(&mut self, error: impl std::fmt::Display) {
        self.link_busy = false;
        self.fail(error);
    }

    /// A new link to the same endpoint is being built.
    pub fn reconnecting(&mut self) {
        self.link_busy = true;
        self.status = "reconnecting".to_string();
        self.events.push("remote-reconnect".to_string());
    }

    /// The new link is up: rediscover, which keeps the mount, and resume by
    /// diff from whatever bells the poll after it collects.
    pub fn rejoined(&mut self) {
        self.closing = false;
        self.pending = None;
        self.rejoins += 1;
        self.link_busy = false;
        self.status = "rediscovering".to_string();
        self.events.push("remote-rejoined".to_string());
        self.begin_discovery();
    }

    /// The host was asked to move the board natively (a receipt hook).
    pub fn nudging(&mut self) {
        self.link_busy = true;
        self.events.push("remote-nudge".to_string());
    }

    /// The nudge answered with the host's new revision, or failed.
    pub fn nudged(&mut self, result: Result<String, String>) {
        self.link_busy = false;
        match result {
            Ok(revision) => {
                let revision = revision.trim();
                self.form.status = format!("Nudged · host at revision {revision}");
                self.events
                    .push(format!("remote-nudged revision {revision}"));
            },
            Err(error) => self.fail(format!("nudge: {error}")),
        }
    }

    /// Stop the operation in flight and say why.
    pub fn fail(&mut self, error: impl std::fmt::Display) {
        self.pending = None;
        self.status = format!("error: {error}");
        self.form.status = format!("Failed · remote: {error}");
        self.events.push(format!("remote-error {error}"));
    }

    // ── The wire ────────────────────────────────────────────────────────────

    /// One line from the endpoint.
    pub fn on_line(&mut self, line: &str) {
        let advance = self.driver.on_line(line);
        self.carry(advance);
    }

    // ── Actions ─────────────────────────────────────────────────────────────

    /// Invoke the `index`th advertised action. A bounded form opens as a
    /// draft for the person to fill; a plain action submits at once.
    pub fn invoke_action(&mut self, index: usize) {
        let Some((target, action)) = self.actions().into_iter().nth(index) else {
            self.form.status = format!("Failed · no remote action #{index}");
            return;
        };
        let Some((session, ack)) = self.session.clone().and_then(|session| {
            let ack = self.client()?.acknowledgement(&session)?;
            Some((session, ack))
        }) else {
            self.form.status = "Failed · remote projection is not acknowledged".to_string();
            return;
        };
        if !self
            .form
            .open_action(session, target, action, (ack.epoch, ack.revision))
        {
            self.submit_draft();
        }
    }

    /// Open the first bounded form the scene advertises.
    pub fn open_first_bounded(&mut self) {
        let (Some(client), Some(session)) = (self.driver.core().map(|c| c.client()), &self.session)
        else {
            self.form.status = "Failed · remote projection is not mounted".to_string();
            return;
        };
        self.form.open_first_bounded(client, session, &self.profile);
    }

    /// Submit the open draft. The answer lands through [`RemoteSession::on_line`].
    pub fn submit_draft(&mut self) {
        let Some(target) = self.form.target.clone() else {
            self.form.status = "Failed · no remote action draft target is open".to_string();
            return;
        };
        let Some(draft) = self.form.draft.as_mut() else {
            self.form.status = "Failed · no remote action draft is open".to_string();
            return;
        };
        // A value the person has not chosen yet is theirs to fix, not the
        // link's failure: the draft stays open and says what it needs.
        if let Err(error) = draft.invocation(&target) {
            self.form.status = format!("Choose required values · {error}");
            return;
        }
        let Some(core) = self.driver.core_mut() else {
            self.form.status = "Failed · remote link is not discovered".to_string();
            return;
        };
        let progress = core.submit_action_draft(&target, draft);
        let label = draft.action().label.clone();
        self.form.count = self.form.count.saturating_add(1);
        self.form.status = format!("Invoking · {label}");
        self.begin(RemoteOp::Invoke, progress);
    }

    // ── Sequencing ──────────────────────────────────────────────────────────

    fn begin_discovery(&mut self) {
        self.pending = Some(RemoteOp::Discover);
        let advance = self.driver.discover();
        self.carry(advance);
    }

    /// Start an operation: hand the driver the core's progress and carry the
    /// first step — a line out, or an answer already at hand.
    fn begin(&mut self, op: RemoteOp, progress: Result<Progress<Outcome>, String>) {
        let advance = progress.and_then(|progress| self.driver.begin(progress));
        self.pending = Some(op);
        self.carry(advance);
    }

    fn carry(&mut self, advance: Result<Advance, String>) {
        match advance {
            Ok(Advance::Send(line)) => self.outbox.push_back(line),
            Ok(Advance::Done(outcome)) => self.finish(outcome),
            Ok(Advance::Noted) => self.drain_bells(),
            Err(error) => self.fail(error),
        }
    }

    fn poll_progress(&mut self) -> Result<Progress<Outcome>, String> {
        self.driver
            .core_mut()
            .map(|core| core.poll())
            .ok_or_else(|| "not discovered".to_string())
    }

    /// The endpoint answered: finish the operation in flight, and begin
    /// whatever it implies next.
    fn finish(&mut self, outcome: Outcome) {
        let op = self.pending.take();
        self.events.push(format!("remote-done {op:?}"));
        match (op, outcome) {
            (Some(RemoteOp::Discover), Outcome::Descriptor(descriptor)) => {
                self.status = format!("discovered · {}", descriptor.label);
                if self.session.is_some() {
                    // A rediscovery on a link that already mounted: the
                    // reconnect. The mount is kept; whatever moved while the
                    // link was down comes back as bells, which a poll rings.
                    self.status = "open".to_string();
                    let progress = self.poll_progress();
                    self.begin(RemoteOp::Poll, progress);
                } else {
                    let progress = self
                        .driver
                        .core_mut()
                        .ok_or_else(|| "not discovered".to_string())
                        .and_then(|core| core.mount(0));
                    self.begin(RemoteOp::Mount, progress);
                }
            },
            (Some(RemoteOp::Mount), Outcome::Mounted(session)) => {
                self.session = Some(session);
                self.status = "open".to_string();
            },
            (Some(RemoteOp::Resnapshot), Outcome::Resnapshotted) => {
                let revision = self.revision().unwrap_or_default();
                self.form.status = format!("{} · revision after {revision}", self.form.status);
            },
            (Some(RemoteOp::Invoke), Outcome::Intent(result)) => {
                self.form.close();
                match *result {
                    IntentResult::Accepted => {
                        self.form.status = format!("Accepted · {} invocation(s)", self.form.count);
                        // The acceptance rang the bell; a round trip lets the
                        // endpoint write it, and the bell drives the resume.
                        let progress = self.poll_progress();
                        self.begin(RemoteOp::Poll, progress);
                    },
                    IntentResult::Rejected { reason } => {
                        self.form.status = format!("Rejected · {reason}");
                        // No bell on a refusal: read the position back by
                        // snapshot, so "unchanged" is measured.
                        let progress = match (self.driver.core_mut(), self.session.clone()) {
                            (Some(core), Some(session)) => core.resnapshot(&session),
                            _ => Err("not mounted".to_string()),
                        };
                        self.begin(RemoteOp::Resnapshot, progress);
                    },
                    IntentResult::Stale {
                        current_revision, ..
                    } => {
                        self.form.status =
                            format!("Stale · host at {} · reopen the action", current_revision.0);
                    },
                }
            },
            // A poll answers with whether the core itself folded anything;
            // the bells the driver queued are drained either way.
            (Some(RemoteOp::Poll), Outcome::Changed(_) | Outcome::Descriptor(_)) => {
                self.drain_bells()
            },
            (Some(RemoteOp::Resume), Outcome::Changed(changed)) => {
                let after = self.revision().unwrap_or_default();
                let before = self
                    .resume_before
                    .take()
                    .map(|revision| revision.to_string())
                    .unwrap_or_else(|| "?".to_string());
                self.last_resume = if changed {
                    format!("diff · {before} → {after}")
                } else {
                    format!("already current at {after}")
                };
                self.drain_bells();
            },
            (op, outcome) => self.fail(format!("unexpected answer to {op:?}: {outcome:?}")),
        }
    }

    /// Resume from the next queued bell, if nothing else is in flight.
    fn drain_bells(&mut self) {
        if self.pending.is_some() {
            return;
        }
        let Some(notice) = self.driver.take_notice() else {
            return;
        };
        self.resume_before = self
            .client()
            .and_then(|client| client.acknowledgement(&notice.session))
            .map(|ack| ack.revision.0);
        let revision = notice.revision.0;
        let progress = self
            .driver
            .core_mut()
            .ok_or_else(|| "not discovered".to_string())
            .and_then(|core| core.resume_from_notice(notice));
        self.events.push(format!("remote-bell revision {revision}"));
        self.begin(RemoteOp::Resume, progress);
    }
}

#[cfg(test)]
mod tests;
