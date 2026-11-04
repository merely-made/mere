/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

pub(super) fn sessions<State, Action, Change, Request>(
    view: &MereView<'_>,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    // Minting heads the list, beside the other lifecycle steps.
    let mint: Option<MereViewElement<State, Action>> = view.model.can_mint.then(|| {
        Box::new(
            el::<_, State, Action>(
                "li",
                request_button(
                    "New session",
                    MereViewRequest::Mint,
                    on_request.clone(),
                    vec![("data-request", "mint".into())],
                ),
            )
            .attr("class", "mere-view-mint"),
        ) as MereViewElement<State, Action>
    });
    let items: Vec<MereViewElement<State, Action>> = mint
        .into_iter()
        .chain(view.model.sessions.iter().map(|session| {
            session_item(
                session,
                view.model
                    .archive
                    .as_ref()
                    .is_some_and(|archive| archive.save_sessions.contains(&session.id)),
                on_request.clone(),
            )
        }))
        .chain(archive_items(view, on_change, on_request.clone()))
        .collect();
    // A band or tier holds the sessions; the hidden column then holds no
    // second copy of their controls.
    let items = if overmap_shown(view.model) || tier_shown(view.model) {
        Vec::new()
    } else {
        items
    };
    let size = if !has_session_content(view.model)
        || overmap_shown(view.model)
        || tier_shown(view.model)
    {
        "display:none".to_string()
    } else if view.model.hide_graph {
        "width:100%;flex:1;min-height:0;max-height:none".to_string()
    } else if wide(view.width) {
        format!("width:{SESSIONS_WIDTH}px")
    } else {
        let body = body_height(view.width, view.height);
        let graph = graph_size(view.width, view.height, true).1;
        format!("max-height:{}px", body.saturating_sub(graph))
    };
    Box::new(
        el::<_, State, Action>("ul", items)
            .attr("class", "mere-view-sessions")
            .attr("aria-label", "Sessions")
            .attr("style", size),
    )
}

/// A session's state in words: whether it is shown here or trashed, and the
/// host's detail.
pub(crate) fn session_words(session: &SessionEntry) -> String {
    let mut words: Vec<&str> = Vec::new();
    if session.attached {
        words.push("Shown here");
    }
    if session.trashed {
        words.push("In the trash");
    }
    if let Some(detail) = &session.detail {
        words.push(detail);
    }
    words.join(", ")
}

pub(super) fn session_item<State, Action, Request>(
    session: &SessionEntry,
    can_save: bool,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let name = el::<_, State, Action>("span", session.name.clone())
        .attr("class", "mere-view-session-name");
    let state = el::<_, State, Action>("span", session_words(session))
        .attr("class", "mere-view-session-state");
    let steps = el::<_, State, Action>("div", session_steps(session, can_save, on_request))
        .attr("class", "mere-view-session-steps");
    Box::new(
        el::<_, State, Action>("li", (name, state, steps))
            .attr("class", "mere-view-session")
            .attr("data-session", session.id.as_uuid().to_string())
            .attr("data-attached", session.attached.to_string())
            .attr("data-trashed", session.trashed.to_string()),
    )
}

/// The steps the host offers on `session`, and Save when the archive takes it.
pub(crate) fn session_steps<State, Action, Request>(
    session: &SessionEntry,
    can_save: bool,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let mut steps: Vec<MereViewElement<State, Action>> = session
        .steps
        .iter()
        .map(|&step| {
            request_button(
                step.label(),
                MereViewRequest::Session(session.id, step),
                on_request.clone(),
                vec![
                    ("data-step", step.token().into()),
                    (
                        "data-session-step",
                        format!("{}:{}", step.token(), session.id.as_uuid()),
                    ),
                    ("aria-label", format!("{} {}", step.label(), session.name)),
                ],
            )
        })
        .collect();
    if can_save {
        steps.push(request_button(
            "Save to archive",
            MereViewRequest::Archive(ArchiveRequest::Save(session.id)),
            on_request,
            vec![
                ("data-step", "save-archive".into()),
                ("data-attached", session.attached.to_string()),
                ("aria-label", format!("Save {} to archive", session.name)),
            ],
        ));
    }
    steps
}

pub(super) fn archive_items<State, Action, Change, Request>(
    view: &MereView<'_>,
    on_change: Change,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let Some(archive) = &view.model.archive else {
        return Vec::new();
    };
    let mut result = vec![Box::new(
        el::<_, State, Action>("li", "Archive").attr("class", "mere-view-archive-heading"),
    ) as MereViewElement<State, Action>];
    if archive.can_compose {
        result.push(Box::new(el::<_, State, Action>(
            "li",
            compose_button(view, on_request.clone()),
        )));
    }
    for entry in &archive.entries {
        let actions = archive_actions(
            view,
            entry,
            "Select for composition",
            on_change.clone(),
            on_request.clone(),
        );
        result.push(Box::new(
            el::<_, State, Action>(
                "li",
                (
                    el::<_, State, Action>("span", entry.name.clone()),
                    el::<_, State, Action>("span", entry.detail.clone()),
                    el::<_, State, Action>("div", actions).attr("class", "mere-view-archive-steps"),
                ),
            )
            .attr("class", "mere-view-archive")
            .attr("data-archive", entry.key.clone()),
        ));
    }
    result
}

/// Compose, enabled once two archives are selected.
pub(crate) fn compose_button<State, Action, Request>(
    view: &MereView<'_>,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let selected = view.state.selected_archives.clone();
    let mut compose = button("Compose selected archives", move |state: &mut State, _| {
        if selected.len() >= 2 {
            on_request(
                state,
                MereViewRequest::Archive(ArchiveRequest::Compose(selected.clone())),
            )
        }
    })
    .attr(
        "aria-disabled",
        (view.state.selected_archives.len() < 2).to_string(),
    )
    .attr("data-request", "compose-archives");
    if view.state.selected_archives.len() < 2 {
        compose = compose.attr("disabled", "");
    }
    Box::new(compose)
}

/// An archive's controls: open it as a session, and select it for
/// composition, labelled `select_label` on screen.
pub(super) fn archive_actions<State, Action, Change, Request>(
    view: &MereView<'_>,
    entry: &ArchiveEntry,
    select_label: &str,
    on_change: Change,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let mut actions = Vec::new();
    if entry.can_open {
        actions.push(request_button(
            "Open as session",
            MereViewRequest::Archive(ArchiveRequest::Open(entry.key.clone())),
            on_request,
            vec![
                ("data-step", "open-archive".into()),
                ("data-archive", entry.key.clone()),
                ("aria-label", format!("Open {} as session", entry.name)),
            ],
        ));
    }
    if view
        .model
        .archive
        .as_ref()
        .is_some_and(|archive| archive.can_compose)
    {
        let key = entry.key.clone();
        actions.push(Box::new(
            button(select_label, move |state: &mut State, _| {
                on_change(state, MereViewEvent::SelectArchive(key.clone()))
            })
            .attr("data-step", "select-archive")
            .attr(
                "aria-label",
                format!("Select {} for composition", entry.name),
            )
            .attr(
                "aria-pressed",
                view.state
                    .selected_archives
                    .contains(&entry.key)
                    .to_string(),
            ),
        ));
    }
    actions
}
