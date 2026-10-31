// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The same retained evidence-brief controls for every embedding host.
//! Queued actions are requests; the owner supplies live gates and persistence.

use crate::workshop::{
    Control, Digest, InstallationStatus, Review, RunStatus, Source, WorkshopView,
};
use cambium::{AnyView, GenetCtx, GenetElement, PointerClick, button, el};

pub type Surface = Box<dyn AnyView<SurfaceState, (), GenetCtx, GenetElement>>;

pub const STYLESHEET: &str = "\
    .alembic-workshop {display:flex;flex-direction:column;padding:20px;gap:12px;background:var(--tabard-color-bg,#10191e);color:var(--tabard-color-text,#e2e8eb);font-family:sans-serif;font-size:14px;}\
    .alembic-workshop h1 {font-size:24px;margin:0;}\
    .alembic-workshop h2 {font-size:17px;margin:10px 0 4px;}\
    .alembic-workshop p {margin:4px 0;line-height:1.5;}\
    .alembic-actions {display:flex;flex-wrap:wrap;gap:8px;}\
    .alembic-workshop button {padding:8px 12px;background:var(--tabard-color-surface,#263942);color:inherit;border:1px solid var(--tabard-color-border,#5a7580);border-radius:5px;}\
    .alembic-workshop button[disabled] {opacity:.45;}\
    .alembic-workshop .primary {background:var(--tabard-color-primary,#386b78);}\
    .alembic-source {padding:8px 0;border-bottom:1px solid var(--tabard-color-border,#33464e);}\
    .alembic-detail {color:var(--tabard-color-text-dim,#aec1c9);font-size:12px;}\
    .alembic-preview {white-space:pre-wrap;overflow-wrap:anywhere;padding:12px;background:var(--tabard-color-surface,#1a2c34);font-size:13px;line-height:1.5;}\
    .alembic-error {padding:10px;border:1px solid #c57665;color:#ffcebf;}";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceChoice {
    pub key: String,
    pub title: String,
    pub version: String,
    pub selected: bool,
    pub available: bool,
    pub refusal: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedReading {
    pub title: String,
    pub version: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceAction {
    LoadExample,
    Run,
    Control(Control),
    OpenSource { proposal: Digest, source: Source },
    OpenResult(String),
}

#[derive(Clone, Debug, Default)]
pub struct SurfaceState {
    pub view: Option<WorkshopView>,
    pub choices: Vec<SourceChoice>,
    pub busy: bool,
    /// The owner has permanently retired this window's private binding.
    /// A fresh owner/window is required; pending local actions carry source data.
    pub owner_retired: bool,
    pub status: String,
    pub error: Option<String>,
    pub reading: Option<RetainedReading>,
    actions: Vec<SurfaceAction>,
}

impl SurfaceState {
    pub fn new() -> Self {
        Self {
            status: "Select two to four retained passages.".into(),
            ..Self::default()
        }
    }
    pub fn selected(&self) -> Vec<String> {
        self.choices
            .iter()
            .filter(|s| s.available && s.selected)
            .map(|s| s.key.clone())
            .collect()
    }
    pub fn take_actions(&mut self) -> Vec<SurfaceAction> {
        std::mem::take(&mut self.actions)
    }
    /// Merge an asynchronous owner observation without undoing a newer local
    /// selection. A replacement version requires a fresh selection.
    pub fn apply_owner_update(&mut self, mut update: Self) {
        // Retirement is terminal for this window. A queued older DTO cannot
        // bring private content or source-bearing actions back after lock.
        if self.owner_retired {
            return;
        }
        if update.owner_retired {
            self.actions.clear();
            update.actions.clear();
            update.view = None;
            update.reading = None;
            update.choices.clear();
            update.busy = true;
            *self = update;
            return;
        }
        for choice in &mut update.choices {
            if let Some(local) = self.choices.iter().find(|local| local.key == choice.key) {
                choice.selected =
                    local.version == choice.version && local.selected && choice.available;
            }
        }
        update.actions = self.take_actions();
        *self = update;
    }
    pub fn request(&mut self, action: SurfaceAction) {
        if self.owner_retired {
            return;
        }
        // Coalesce identical unhandled UI activations. Durable identity and
        // target deduplication remain the domain and owner's responsibility.
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
    }
    fn toggle(&mut self, key: &str) {
        if self.busy {
            return;
        }
        if let Some(source) = self
            .choices
            .iter_mut()
            .find(|s| s.key == key && s.available)
        {
            source.selected = !source.selected;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workshop_surface_delayed_owner_response_preserves_newer_selection() {
        let mut current = SurfaceState::new();
        current.choices = vec![SourceChoice {
            key: "A".into(),
            title: "Same title".into(),
            version: "v1".into(),
            selected: true,
            available: true,
            refusal: None,
        }];
        let old_response = current.clone();
        current.toggle("A");
        current.request(SurfaceAction::Run);
        current.apply_owner_update(old_response);
        assert!(!current.choices[0].selected);
        assert_eq!(current.take_actions(), [SurfaceAction::Run]);
        current.toggle("A");
        let mut replacement = current.clone();
        replacement.choices[0].version = "v2".into();
        current.apply_owner_update(replacement);
        assert!(
            !current.choices[0].selected,
            "a new version needs selection"
        );
    }
}

fn action(label: &'static str, id: &'static str, request: SurfaceAction, enabled: bool) -> Surface {
    let view = button(label, move |s: &mut SurfaceState, _: PointerClick| {
        if enabled {
            s.request(request.clone());
        }
    })
    .attr("data-action", id);
    if enabled {
        Box::new(view)
    } else {
        Box::new(view.attr("disabled", "").attr("aria-disabled", "true"))
    }
}

fn phase(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Installed => "Helper ready",
        RunStatus::Ready(_) => "Ready to continue",
        RunStatus::Awaiting(_) => "Preparing evidence brief",
        RunStatus::Paused(_) => "Paused",
        RunStatus::Reconciling(_) => "Checking an interrupted operation",
        RunStatus::Completed => "Proposal saved",
        RunStatus::Refused => "Run refused",
        RunStatus::Cancelled => "Run cancelled",
        RunStatus::Failed => "Run failed",
        RunStatus::BudgetExhausted => "Run limit reached",
    }
}

pub fn surface(state: &SurfaceState) -> Surface {
    let mut body: Vec<Surface> = vec![
        Box::new(el("h1", "Alembic")),
        Box::new(el("p", "Make an evidence brief from retained passages, then review it before adding a note.")),
        Box::new(el("p", "The helper reads only your selected retained versions and proposes a brief. Adding it to a graph is your separate decision.").attr("class", "alembic-detail")),
        action("Load public example", "alembic-example", SurfaceAction::LoadExample, !state.busy),
        Box::new(el("h2", "Retained sources")),
    ];
    for choice in &state.choices {
        let key = choice.key.clone();
        let label = format!(
            "{} {} — {}",
            if choice.selected { "[x]" } else { "[ ]" },
            choice.title,
            choice.version
        );
        let toggle = button(label, move |s: &mut SurfaceState, _: PointerClick| {
            s.toggle(&key)
        })
        .attr("data-action", "alembic-source-select")
        .attr("data-source", choice.key.clone())
        .attr("role", "checkbox")
        .attr(
            "aria-checked",
            if choice.selected { "true" } else { "false" },
        );
        let toggle: Surface = if choice.available && !state.busy {
            Box::new(toggle)
        } else {
            Box::new(toggle.attr("disabled", ""))
        };
        let mut row = vec![toggle];
        if let Some(reason) = &choice.refusal {
            row.push(Box::new(
                el("p", reason.clone()).attr("class", "alembic-detail"),
            ));
        }
        body.push(Box::new(el("div", row).attr("class", "alembic-source")));
    }
    let active = state
        .view
        .as_ref()
        .is_none_or(|v| v.lifecycle == InstallationStatus::Active);
    let allowed = state
        .view
        .as_ref()
        .is_none_or(|v| v.controls.contains(&Control::Run));
    body.push(action(
        "Make evidence brief",
        "alembic-run",
        SurfaceAction::Run,
        !state.busy && active && allowed && (2..=4).contains(&state.selected().len()),
    ));
    body.push(Box::new(
        el("p", state.status.clone())
            .attr("role", "status")
            .attr("data-fact", "alembic-status"),
    ));
    if let Some(error) = &state.error {
        body.push(Box::new(
            el("p", error.clone())
                .attr("role", "alert")
                .attr("class", "alembic-error"),
        ));
    }
    if let Some(view) = &state.view {
        body.push(Box::new(
            el(
                "p",
                match view.lifecycle {
                    InstallationStatus::Active => "Helper active",
                    InstallationStatus::Paused => "Helper paused",
                    InstallationStatus::Revoked => "Helper revoked",
                    InstallationStatus::Dissolved => "Helper dissolved",
                },
            )
            .attr("data-fact", "alembic-lifecycle"),
        ));
        body.push(Box::new(el("h2", phase(view.phase))));
        body.push(Box::new(
            el(
                "p",
                format!(
                    "{} decisions · {} tool calls · {} model tokens · {} ms",
                    view.usage.decisions,
                    view.usage.tool_calls,
                    view.usage.tokens,
                    view.usage.elapsed_ms
                ),
            )
            .attr("class", "alembic-detail"),
        ));
        let mut controls = vec![];
        for control in &view.controls {
            let (label, id) = match control {
                Control::Run => continue,
                Control::Pause => ("Pause", "alembic-pause"),
                Control::Resume => ("Resume", "alembic-resume"),
                Control::Revoke => ("Revoke helper", "alembic-revoke"),
                Control::Dissolve => ("Dissolve helper", "alembic-dissolve"),
                Control::Accept => ("Add brief to this graph", "alembic-accept"),
                Control::Reject => ("Reject brief", "alembic-reject"),
                Control::Reconcile => ("Check destination receipt", "alembic-reconcile"),
            };
            let enabled = !state.busy
                || matches!(
                    control,
                    Control::Pause | Control::Revoke | Control::Dissolve
                );
            controls.push(action(label, id, SurfaceAction::Control(*control), enabled));
        }
        body.push(Box::new(
            el("div", controls).attr("class", "alembic-actions"),
        ));
        if let Some(proposed) = &view.proposal {
            body.push(Box::new(
                el("p", proposed.djot.clone())
                    .attr("class", "alembic-preview")
                    .attr("data-fact", "alembic-proposal"),
            ));
            for source in &proposed.sources {
                let request = SurfaceAction::OpenSource {
                    proposal: proposed.digest,
                    source: source.source.clone(),
                };
                let label = format!(
                    "Open {} · version {}",
                    source.source.title, source.source.version
                );
                let enabled = !state.busy;
                body.push(Box::new(
                    button(label, move |s: &mut SurfaceState, _: PointerClick| {
                        if enabled {
                            s.request(request.clone());
                        }
                    })
                    .attr("data-action", "alembic-source-open")
                    .attr("data-resource", source.source.resource.clone()),
                ));
            }
        }
        match &view.review {
            Some(Review::Added { receipt, .. }) => {
                let resource = receipt.resource.clone();
                body.push(Box::new(el("p", "Added to graph; destination storage confirmed.").attr("data-fact", "alembic-added")));
                body.push(Box::new(button("Open retained brief", move |s: &mut SurfaceState, _: PointerClick| s.request(SurfaceAction::OpenResult(resource.clone()))).attr("data-action", "alembic-result-open")));
            }
            Some(Review::Applying(_)) | Some(Review::Reconciling(_)) => body.push(Box::new(el("p", "The destination outcome is being checked. A new note will not be created by guessing.").attr("data-fact", "alembic-reconciling"))),
            Some(Review::Rejected) => body.push(Box::new(el("p", "Brief rejected. Nothing was added to the graph.").attr("data-fact", "alembic-rejected"))),
            _ => {},
        }
    }
    if let Some(reading) = &state.reading {
        body.push(Box::new(el("h2", reading.title.clone())));
        body.push(Box::new(
            el("p", format!("Retained version {}", reading.version))
                .attr("class", "alembic-detail"),
        ));
        body.push(Box::new(
            el("p", reading.text.clone())
                .attr("class", "alembic-preview")
                .attr("data-fact", "alembic-reading"),
        ));
    }
    Box::new(
        el("main", body)
            .attr("class", "alembic-workshop")
            .attr("data-surface", "alembic.workshop.v1"),
    )
}
