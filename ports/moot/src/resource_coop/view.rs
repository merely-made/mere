// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Ordinary Cambium controls over a disclosed reading; the host routes intentions.
use super::control::{Controller, Intent};
use super::*;
use cambium::{
    AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, View, button, detail_panel, el,
};

type Child<S, A> = Box<dyn AnyView<S, A, GenetCtx, GenetElement>>;
pub fn hex(id: [u8; 32]) -> String {
    id.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn offer_label(offer: Option<&Offer>) -> String {
    match offer {
        Some(Offer::Offered) => "Request available; owner checks at dispatch".into(),
        Some(Offer::Withheld { reason }) => format!("Withheld: {reason}"),
        Some(Offer::Unsupported { reason }) => format!("Unsupported: {reason}"),
        None => "Not disclosed".into(),
    }
}
fn action<S: 'static, A: 'static, F: Fn(&mut S, Intent) + Clone + 'static>(
    id: String,
    label: impl Into<String>,
    offer: Option<&Offer>,
    intent: Intent,
    send: F,
) -> Child<S, A> {
    let enabled = matches!(offer, Some(Offer::Offered));
    let button = button(label, move |state: &mut S, _| send(state, intent.clone())).attr("id", id);
    Box::new(if enabled {
        button
    } else {
        button.attr("disabled", "")
    })
}

/// Each call renders one independent Controller. Inspection updates attention;
/// operation buttons emit a typed intention through `send`, never store writes.
pub fn view<S: 'static, A: 'static, F: Fn(&mut S, Intent) + Clone + 'static>(
    scope: &str,
    controller: &Controller,
    send: F,
) -> impl View<S, A, GenetCtx, Element = GenetElement> + use<S, A, F> {
    let pane = &controller.pane;
    let snapshot = pane.snapshot();
    let mut children: Vec<Child<S, A>> = vec![Box::new(el("h2", "Shared resources and coop"))];
    let refresh = send.clone();
    children.push(Box::new(
        button("Refresh owner read", move |state: &mut S, _| {
            refresh(state, Intent::Refresh)
        })
        .attr("id", format!("{scope}-refresh")),
    ));
    if let Some(collection) = &snapshot.collection {
        children.push(Box::new(el("h3", collection.name.clone())));
        children.push(action(
            format!("{scope}-collection-fork"),
            "Fork collection references",
            collection.fork.as_ref(),
            Intent::ForkCollection,
            send.clone(),
        ));
        children.push(Box::new(el(
            "p",
            format!("Collection fork: {}", offer_label(collection.fork.as_ref())),
        )));
    }
    for (number, resource) in snapshot.resources.iter().enumerate() {
        let reference = resource.reference;
        let callback = send.clone();
        children.push(Box::new(
            button(
                format!("Inspect resource {}: {}", number + 1, resource.title),
                move |state: &mut S, _| callback(state, Intent::InspectResource(reference)),
            )
            .attr("id", format!("{scope}-resource-{}", number + 1)),
        ));
        children.push(Box::new(el(
            "p",
            format!(
                "{} · {}",
                resource.schema,
                match &resource.content {
                    ContentRead::AddressOnly => "Content address only".into(),
                    ContentRead::Resolved => "Content resolved; retention separate".into(),
                    ContentRead::Unavailable { reason } => format!("Content unavailable: {reason}"),
                }
            ),
        )));
    }
    if let Some(selected) = pane.selected_resource() {
        if let Some(resource) = snapshot
            .resources
            .iter()
            .find(|resource| resource.reference == selected)
        {
            children.push(Box::new(detail_panel(&[DetailSection::new(
                format!("Resource: {}", resource.title),
                vec![
                    DetailRow::new("Contribution", hex(selected.contribution)),
                    DetailRow::new("Exact manifest", hex(selected.manifest)),
                    DetailRow::new("Schema", &resource.schema),
                    DetailRow::new(
                        "Local copy",
                        match &resource.retention {
                            RetentionRead::Unknown => "Not disclosed".into(),
                            RetentionRead::Held {
                                owner_receipt,
                                retained_at_ms,
                            } => format!("Verified held copy at {retained_at_ms}: {owner_receipt}"),
                            RetentionRead::Unavailable { reason } => {
                                format!("Unavailable: {reason}")
                            },
                        },
                    ),
                    DetailRow::new(
                        "Hosting",
                        match &resource.hosting {
                            HostingRead::Unknown => "Not disclosed".into(),
                            HostingRead::Promise { owner_record } => {
                                format!("Promise: {owner_record}; availability unverified")
                            },
                            HostingRead::Observed {
                                owner_receipt,
                                observed_at_ms,
                            } => format!("Observed at {observed_at_ms}: {owner_receipt}"),
                            HostingRead::Expired { owner_record } => {
                                format!("Expired: {owner_record}")
                            },
                            HostingRead::Withdrawn { owner_record } => {
                                format!("Withdrawn: {owner_record}")
                            },
                        },
                    ),
                ],
            )])));
            for (action_kind, label) in [
                (ResourceAction::Open, "Open resource"),
                (ResourceAction::RetainCopy, "Retain a copy"),
                (ResourceAction::ForkContent, "Fork content"),
                (ResourceAction::OfferHosting, "Offer hosting"),
            ] {
                let offer = resource.offers.get(&action_kind);
                children.push(action(
                    format!("{scope}-resource-{action_kind:?}"),
                    label,
                    offer,
                    Intent::Resource(action_kind),
                    send.clone(),
                ));
                children.push(Box::new(el(
                    "p",
                    format!("{label}: {}", offer_label(offer)),
                )));
                if let Some(terms) = resource.terms.get(&action_kind) {
                    children.push(Box::new(el("p", terms.description.clone())));
                }
            }
        }
    }
    for (number, activity) in snapshot.activities.iter().enumerate() {
        let reference = activity.reference.clone();
        let callback = send.clone();
        children.push(Box::new(
            button(
                format!("Inspect activity {}: {}", number + 1, activity.title),
                move |state: &mut S, _| callback(state, Intent::InspectActivity(reference.clone())),
            )
            .attr("id", format!("{scope}-activity-{}", number + 1)),
        ));
    }
    if let Some(selected) = pane.selected_activity() {
        if let Some(activity) = snapshot
            .activities
            .iter()
            .find(|activity| &activity.reference == selected)
        {
            let lifecycle = &activity.lifecycle;
            children.push(Box::new(detail_panel(&[DetailSection::new(
                format!("Activity: {}", activity.title),
                vec![
                    DetailRow::new("Activity owner", &selected.owner),
                    DetailRow::new("Exact activity", &selected.activity),
                    DetailRow::new("Participation", lifecycle.verdict.name()),
                    DetailRow::new("Reason", lifecycle.reason.as_deref().unwrap_or("None")),
                    DetailRow::new(
                        "Membership at this read",
                        lifecycle.membership.map_or_else(
                            || "Not disclosed".into(),
                            |m| {
                                format!(
                                    "{} members; {:?} access; presence unverified",
                                    m.members, m.access
                                )
                            },
                        ),
                    ),
                    DetailRow::new("Observed at", lifecycle.now_ms.to_string()),
                    DetailRow::new(
                        "Reading after withdrawal",
                        format!("{:?}", lifecycle.reading),
                    ),
                ],
            )])));
            for (kind, label) in [
                (ActivityAction::Join, "Join activity"),
                (ActivityAction::Leave, "Leave activity"),
            ] {
                let offer = activity.offers.get(&kind);
                children.push(action(
                    format!("{scope}-activity-{kind:?}"),
                    label,
                    offer,
                    Intent::Activity(kind),
                    send.clone(),
                ));
                children.push(Box::new(el(
                    "p",
                    format!("{label}: {}", offer_label(offer)),
                )));
            }
        }
    }
    if pane.pending().is_some() {
        children.push(Box::new(
            el("p", "Waiting for the action owner").attr("role", "status"),
        ));
    }
    if let Some(outcome) = pane.outcome() {
        children.push(Box::new(el("p",match outcome {
            Outcome::Accepted{owner_receipt}=>format!("Owner accepted: {owner_receipt}. Refresh from the owner to see changed state."),
            Outcome::Refused{reason}=>format!("Owner refused: {reason}"),Outcome::Unsupported{reason}=>format!("Owner does not support this action: {reason}"),
        }).attr("role","status")));
    }
    if let Some(notice) = &controller.notice {
        children.push(Box::new(el("p", notice.clone()).attr("role", "status")));
    }
    el("section", children)
        .attr("class", "moot-resource-coop")
        .attr("aria-label", "Shared resources and coop")
}
