// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-neutral Cambium resource summaries, inspector and coop controls.
//! The host takes prepared requests, dispatches them through actual owners,
//! delivers feedback to this reading and refreshes its owner snapshot.
//! Building, hiding or restyling the view never dispatches a domain command.

use cambium::{
    AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, button, detail_panel, el,
};

use crate::resource_coop::{
    ActivityAction, ActivityRef, ContentRead, HostingRead, Offer, Outcome, Pane, Refusal, Request,
    ResourceAction, ResourceRef, Snapshot,
};

pub type View = Box<dyn AnyView<Reading, (), GenetCtx, GenetElement>>;

/// One host-owned reading. The host chooses a unique DOM scope for each mount.
#[derive(Debug)]
pub struct Reading {
    id: String,
    pane: Pane,
    outbox: Option<Request>,
    status: String,
    exact_evidence: bool,
}

impl Reading {
    pub fn new(id: impl Into<String>, snapshot: Snapshot) -> Result<Self, String> {
        let id = id.into();
        if id.is_empty()
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        {
            return Err(
                "Choose a unique reading name using letters, digits, hyphens or underscores".into(),
            );
        }
        Ok(Self {
            id,
            pane: Pane::new(snapshot).map_err(|error| error.to_string())?,
            outbox: None,
            status: "Choose a shared resource or activity".into(),
            exact_evidence: false,
        })
    }
    pub fn pane(&self) -> &Pane {
        &self.pane
    }
    pub fn take_request(&mut self) -> Option<Request> {
        self.outbox.take()
    }
    /// Prepare a fresh resource intention from the current selection and owner
    /// offers. The embedding host must still dispatch and recheck authority.
    pub fn request_resource(&mut self, action: ResourceAction) -> Result<(), Refusal> {
        let request = self.pane.prepare_resource(action)?;
        self.prepared(Ok(request));
        Ok(())
    }
    pub fn refresh(&mut self, snapshot: Snapshot) -> Result<(), Refusal> {
        self.pane.refresh(snapshot)?;
        self.outbox = None;
        self.status = "Updated from the owners".into();
        Ok(())
    }
    pub fn complete(&mut self, request: &Request, outcome: Outcome) -> bool {
        let status = match &outcome {
            Outcome::Accepted { .. } => {
                "The owner accepted the request; refresh to see its resulting state".into()
            },
            Outcome::Refused { reason } => format!("The owner refused: {reason}"),
            Outcome::Unsupported { reason } => format!("Unavailable: {reason}"),
        };
        if !self.pane.complete(request, outcome) {
            return false;
        }
        self.status = status;
        self.outbox = None;
        true
    }
    fn id(&self, suffix: &str) -> String {
        format!("moot-{}-{suffix}", self.id)
    }
    /// Restore or change local attention against the current owner snapshot.
    /// This prepares no command and carries no permission to open the content.
    pub fn select_resource(&mut self, reference: ResourceRef) {
        let before = self.pane.selected_resource();
        match self.pane.select_resource(reference) {
            Ok(()) => {
                if before != Some(reference) {
                    self.outbox = None;
                }
                self.status = "Inspecting this exact resource".into();
            },
            Err(error) => self.status = error.to_string(),
        }
    }
    fn select_activity(&mut self, reference: &ActivityRef) {
        let before = self.pane.selected_activity().cloned();
        match self.pane.select_activity(reference) {
            Ok(()) => {
                if before.as_ref() != Some(reference) {
                    self.outbox = None;
                }
                self.status = "Inspecting this activity".into();
            },
            Err(error) => self.status = error.to_string(),
        }
    }
    fn prepared(&mut self, result: Result<Request, Refusal>) {
        match result {
            Ok(request) => {
                self.outbox = Some(request);
                self.status = "Request prepared for the owner".into();
            },
            Err(error) => self.status = error.to_string(),
        }
    }
}

/// Composes into a host through ordinary Cambium lens/state composition. The
/// small sheet supplies geometry only; the host supplies theme and typography.
pub const CSS: &str = ".moot-resource-coop{display:flex;flex-wrap:wrap;gap:12px}.moot-resource-main{flex:1;min-width:220px}.moot-resource-inspector{flex:1;min-width:220px}.moot-resource-coop p{overflow-wrap:anywhere}.moot-resource-coop button{margin:3px}.moot-resource-coop ul{padding-left:20px}";

pub fn view(reading: &Reading) -> View {
    let snapshot = reading.pane.snapshot();
    let mut main: Vec<View> = vec![Box::new(el(
        "h2",
        snapshot
            .collection
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Shared resources".into()),
    ))];
    let rows: Vec<View> = snapshot
        .resources
        .iter()
        .map(|resource| {
            let reference = resource.reference;
            Box::new(
                el(
                    "li",
                    (
                        el("h3", resource.title.clone()),
                        el("p", content_label(&resource.content)),
                        button("Inspect resource", move |reading: &mut Reading, _| {
                            reading.select_resource(reference)
                        })
                        .attr(
                            "id",
                            reading.id(&format!("inspect-{}", hex(&reference.contribution))),
                        )
                        .attr("aria-label", format!("Inspect {}", resource.title)),
                    ),
                )
                .attr("class", "moot-resource-summary"),
            ) as View
        })
        .collect();
    main.push(Box::new(
        el("ul", rows).attr("aria-label", "Shared resources"),
    ));
    if let Some(collection) = &snapshot.collection {
        if let Some(destination) = &collection.fork_destination {
            main.push(Box::new(el(
                "p",
                format!(
                    "Fork destination: {}. This copies collection references and ancestry.",
                    destination.name
                ),
            )));
        }
        let observed = collection.clone();
        let fork = button(
            "Fork collection references",
            move |reading: &mut Reading, _| {
                if reading.pane.snapshot().collection.as_ref() != Some(&observed) {
                    reading.status =
                        "The displayed collection changed; inspect its current version first"
                            .into();
                    return;
                }
                let result = reading.pane.prepare_collection_fork();
                reading.prepared(result);
            },
        )
        .attr("id", reading.id("fork-collection"));
        main.push(Box::new(if collection.fork == Some(Offer::Offered) {
            fork
        } else {
            fork.attr("disabled", "")
        }));
        main.push(Box::new(el("p", offer_label(collection.fork.as_ref()))));
    }
    main.push(Box::new(el("h2", "Coop activities")));
    for activity in &snapshot.activities {
        let reference = activity.reference.clone();
        main.push(Box::new(
            el(
                "section",
                (
                    el("h3", activity.title.clone()),
                    el(
                        "p",
                        format!(
                            "{} · {} related resources",
                            verdict_label(activity.lifecycle.verdict),
                            activity.resources.len()
                        ),
                    ),
                    button("Inspect activity", move |reading: &mut Reading, _| {
                        reading.select_activity(&reference)
                    })
                    .attr("aria-label", format!("Inspect {}", activity.title)),
                ),
            )
            .attr("class", "moot-activity-summary"),
        ));
    }

    let mut inspection: Vec<View> = vec![Box::new(el("h2", "Inspection"))];
    if let Some(resource) = snapshot
        .resources
        .iter()
        .find(|resource| Some(resource.reference) == reading.pane.selected_resource())
    {
        inspection.push(Box::new(el("h3", resource.title.clone())));
        inspection.push(Box::new(detail_panel(&[DetailSection::new(
            "Resource",
            vec![
                DetailRow::new("Content", content_label(&resource.content)),
                DetailRow::new("Hosting", hosting_label(&resource.hosting)),
            ],
        )])));
        for (action, label, suffix) in [
            (ResourceAction::Open, "Open local content", "open"),
            (ResourceAction::RetainCopy, "Retain a copy", "retain"),
            (ResourceAction::ForkContent, "Fork content", "fork-content"),
            (ResourceAction::OfferHosting, "Offer hosting", "host"),
        ] {
            let reference = resource.reference;
            let control = button(label, move |reading: &mut Reading, _| {
                if reading.pane.selected_resource() != Some(reference) {
                    return;
                }
                if let Err(error) = reading.request_resource(action) {
                    reading.status = error.to_string();
                }
            })
            .attr("id", reading.id(suffix));
            inspection.push(Box::new(
                if resource.offers.get(&action) == Some(&Offer::Offered) {
                    control
                } else {
                    control.attr("disabled", "")
                },
            ));
            inspection.push(Box::new(el("p", offer_label(resource.offers.get(&action)))));
        }
        inspection.push(Box::new(
            button(
                if reading.exact_evidence {
                    "Hide exact references"
                } else {
                    "Show exact references"
                },
                |reading: &mut Reading, _| reading.exact_evidence = !reading.exact_evidence,
            )
            .attr("id", reading.id("evidence")),
        ));
        if reading.exact_evidence {
            inspection.push(Box::new(detail_panel(&[DetailSection::new(
                "Exact references",
                vec![
                    DetailRow::new("Contribution", hex(&resource.reference.contribution)),
                    DetailRow::new("Manifest", hex(&resource.reference.manifest)),
                    DetailRow::new(
                        "Shared by",
                        resource
                            .shared_by
                            .map(|author| hex(&author))
                            .unwrap_or_else(|| "Not disclosed".into()),
                    ),
                    DetailRow::new("Schema", resource.schema.clone()),
                ],
            )])));
        }
    } else {
        inspection.push(Box::new(el(
            "p",
            "Choose a resource to inspect its content and hosting evidence",
        )));
    }
    if let Some(activity) = snapshot
        .activities
        .iter()
        .find(|activity| Some(&activity.reference) == reading.pane.selected_activity())
    {
        inspection.push(Box::new(el("h3", activity.title.clone())));
        inspection.push(Box::new(el(
            "p",
            activity
                .lifecycle
                .reason
                .clone()
                .unwrap_or_else(|| "The activity owner reports this lifecycle state".into()),
        )));
        for (action, label, suffix) in [
            (ActivityAction::Join, "Join activity", "join"),
            (ActivityAction::Leave, "Leave activity", "leave"),
        ] {
            let reference = activity.reference.clone();
            let control = button(label, move |reading: &mut Reading, _| {
                if reading.pane.selected_activity() != Some(&reference) {
                    return;
                }
                let result = reading.pane.prepare_activity(action);
                reading.prepared(result);
            })
            .attr("id", reading.id(suffix));
            inspection.push(Box::new(
                if activity.offers.get(&action) == Some(&Offer::Offered) {
                    control
                } else {
                    control.attr("disabled", "")
                },
            ));
            inspection.push(Box::new(el("p", offer_label(activity.offers.get(&action)))));
        }
    }
    Box::new(
        el(
            "section",
            vec![
                Box::new(el("div", main).attr("class", "moot-resource-main")) as View,
                Box::new(
                    el("aside", inspection)
                        .attr("class", "moot-resource-inspector")
                        .attr("aria-label", "Resource and activity inspection"),
                ),
                Box::new(
                    el("p", reading.status.clone())
                        .attr("role", "status")
                        .attr("id", reading.id("status")),
                ),
            ],
        )
        .attr("class", "moot-resource-coop")
        .attr("aria-label", format!("Moot reading {}", reading.id)),
    )
}

fn content_label(content: &ContentRead) -> String {
    match content {
        ContentRead::AddressOnly => "Address known; content availability unknown".into(),
        ContentRead::Resolved => "Content resolved by this reader".into(),
        ContentRead::Unavailable { reason } => format!("Content unavailable: {reason}"),
    }
}
fn verdict_label(verdict: crate::coop::Verdict) -> &'static str {
    use crate::coop::Verdict;
    match verdict {
        Verdict::Joined => "Participating",
        Verdict::Left => "Left",
        Verdict::Expired => "Permission expired",
        Verdict::Revoked => "Permission withdrawn",
        Verdict::NotJoined => "Not participating",
        Verdict::NotAdmitted => "Not admitted",
    }
}
fn hosting_label(hosting: &HostingRead) -> String {
    match hosting {
        HostingRead::Unknown => "Unknown".into(),
        HostingRead::Promise { .. } => "Hosting promised; retrieval not observed".into(),
        HostingRead::Observed { observed_at_ms, .. } => {
            format!("Retrieval observed at {observed_at_ms}; current reachability unknown")
        },
        HostingRead::Expired { .. } => "Hosting promise expired".into(),
        HostingRead::Withdrawn { .. } => "Hosting promise withdrawn".into(),
    }
}
fn offer_label(offer: Option<&Offer>) -> String {
    match offer {
        Some(Offer::Offered) => "Request available; the owner checks it when dispatched".into(),
        Some(Offer::Withheld { reason }) => format!("Withheld: {reason}"),
        Some(Offer::Unsupported { reason }) => format!("Unavailable: {reason}"),
        None => "The owner has not disclosed whether this action is available".into(),
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
