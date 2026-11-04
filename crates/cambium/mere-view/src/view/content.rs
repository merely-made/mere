/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// The overmap band: each session a container node with the archives saved
/// from it on it, the archive shelf, and minting last.
pub(super) fn overmap<State, Action, Change, Request>(
    view: &MereView<'_>,
    band: u32,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let width = container_width(view.width);
    let chips = |session: Option<&SessionEntry>| -> MereViewElement<State, Action> {
        let items: Vec<MereViewElement<State, Action>> = archives_on(model, session)
            .into_iter()
            .map(|entry| {
                let steps =
                    archive_actions(view, entry, "Select", on_change.clone(), on_request.clone());
                Box::new(
                    el::<_, State, Action>(
                        "li",
                        (
                            el::<_, State, Action>("span", entry.name.clone())
                                .attr("class", "mere-view-archive-name")
                                .attr("title", entry.detail.clone()),
                            el::<_, State, Action>("div", steps)
                                .attr("class", "mere-view-archive-steps"),
                        ),
                    )
                    .attr("class", "mere-view-archive")
                    .attr("data-archive", entry.key.clone())
                    .attr("style", format!("height:{}px", chip_height(model, entry))),
                ) as MereViewElement<State, Action>
            })
            .collect();
        Box::new(
            el::<_, State, Action>("ul", items)
                .attr("class", "mere-view-container-archives")
                .attr("aria-label", "Archives"),
        )
    };
    let items: Vec<MereViewElement<State, Action>> = containers(model)
        .iter()
        .map(|container| {
            let size = format!(
                "width:{width}px;height:{}px",
                container_height(model, container)
            );
            match container {
                Container::Session(session) => Box::new(
                    el::<_, State, Action>(
                        "section",
                        (
                            el::<_, State, Action>("span", session.name.clone())
                                .attr("class", "mere-view-session-name"),
                            el::<_, State, Action>("span", session_words(session))
                                .attr("class", "mere-view-session-state"),
                            el::<_, State, Action>(
                                "div",
                                session_steps(
                                    session,
                                    can_save(model, session),
                                    on_request.clone(),
                                ),
                            )
                            .attr("class", "mere-view-session-steps"),
                            chips(Some(*session)),
                        ),
                    )
                    .attr("class", "mere-view-container mere-view-session")
                    .attr("data-container", "session")
                    .attr("role", "group")
                    .attr("aria-label", format!("Session {}", session.name))
                    .attr("data-session", session.id.as_uuid().to_string())
                    .attr("data-attached", session.attached.to_string())
                    .attr("data-trashed", session.trashed.to_string())
                    .attr("style", size),
                ) as MereViewElement<State, Action>,
                Container::Shelf => {
                    let compose: Vec<MereViewElement<State, Action>> = model
                        .archive
                        .as_ref()
                        .is_some_and(|archive| archive.can_compose)
                        .then(|| compose_button(view, on_request.clone()))
                        .into_iter()
                        .collect();
                    Box::new(
                        el::<_, State, Action>(
                            "section",
                            (
                                el::<_, State, Action>("span", "Archive")
                                    .attr("class", "mere-view-container-title"),
                                el::<_, State, Action>("div", compose)
                                    .attr("class", "mere-view-session-steps"),
                                chips(None),
                            ),
                        )
                        .attr("class", "mere-view-container mere-view-shelf")
                        .attr("data-container", "shelf")
                        .attr("role", "group")
                        .attr("aria-label", "Archive")
                        .attr("style", size),
                    )
                },
                Container::Mint => Box::new(
                    el::<_, State, Action>(
                        "div",
                        request_button(
                            "New session",
                            MereViewRequest::Mint,
                            on_request.clone(),
                            vec![("data-request", "mint".into())],
                        ),
                    )
                    .attr("class", "mere-view-container mere-view-mint")
                    .attr("data-container", "mint")
                    .attr("style", size),
                ),
            }
        })
        .collect();
    Box::new(
        el::<_, State, Action>("div", items)
            .attr("class", "mere-view-overmap")
            .attr("role", "group")
            .attr("aria-label", "Overmap: sessions and archives")
            .attr("style", format!("width:{}px;height:{band}px", view.width)),
    )
}

pub(super) fn graph<State, Action, Change, Request>(
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
    let swatch = view.swatch();
    let hover = {
        let on_change = on_change.clone();
        move |state: &mut State, key: Option<String>| on_change(state, MereViewEvent::Hover(key))
    };
    let focus = {
        let on_change = on_change.clone();
        move |state: &mut State, key: Option<String>| on_change(state, MereViewEvent::Focus(key))
    };
    // A tier node opens its own widget; every other node is the host's.
    let tier = tier_shown(view.model);
    let model = view.model;
    let cards: Vec<TierCard> = model
        .sessions
        .iter()
        .map(|session| TierCard::Session(session.id))
        .chain([TierCard::Shelf])
        .filter(|card| card_listed(model, *card))
        .collect();
    let archives: Vec<String> = view
        .model
        .archive
        .iter()
        .flat_map(|archive| archive.entries.iter().map(|entry| entry.key.clone()))
        .collect();
    let expanded = view.state.expanded;
    let activate = {
        let on_change = on_change.clone();
        let on_request = on_request.clone();
        move |state: &mut State, key: String| {
            if tier {
                if let Some((card, toggles)) =
                    card_of_node(&key).filter(|(card, _)| cards.contains(card))
                {
                    let open = (!toggles || expanded != Some(card)).then_some(card);
                    return on_change(state, MereViewEvent::Expand(open));
                }
                if let Some(archive) =
                    archive_of_node(&key).filter(|key| archives.iter().any(|a| a == key))
                {
                    return on_change(
                        state,
                        MereViewEvent::ArchivePopover(
                            archive.to_string(),
                            DetailPopoverEvent::TogglePinned,
                        ),
                    );
                }
            }
            on_request(state, MereViewRequest::Activate(key))
        }
    };
    let canvas = graph_canvas_swatch_with_focus_and_drag_and_relations(
        &swatch,
        activate,
        hover,
        focus,
        |_: &mut State, _: GraphCanvasNodeDrag<String>| {},
        |_: &mut State, _: String| {},
        |_: &mut State, _: Option<String>| {},
        |_: &mut State| {},
        |_: &mut State, _: PointerEvent| {},
        |_: &mut State, _: WheelEvent| {},
    );
    let (card, widgets) = if tier {
        (
            tier_card(view, &swatch, on_change.clone(), on_request.clone()),
            tier_widgets(view, &swatch, on_change, on_request),
        )
    } else {
        (Keyed::new(Vec::new()), Vec::new())
    };
    Box::new(
        el::<_, State, Action>("div", (canvas, card, widgets))
            .attr("class", "mere-view-graph")
            .attr("data-tier", if tier { "shown" } else { "none" }),
    )
}

/// The pending merge's card, over the view below its bar: each conflict with
/// its three values, its choices and a hand value to write, then Apply and
/// Cancel.
pub(super) fn merge_card<State, Action, Change, Request>(
    view: &MereView<'_>,
    merge: &MergeModel,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let top = if wide(view.width) {
        BAR_HEIGHT
    } else {
        NARROW_BAR_HEIGHT
    } + 8;
    let title = el::<_, State, Action>(
        "h2",
        format!("Merge {} into {}", merge.source_name, merge.target_name),
    )
    .attr("class", "mere-view-merge-title");
    let dropped: MereViewElement<State, Action> = if merge.dropped.is_empty() {
        Box::new(el::<_, State, Action>("div", ()).attr("style", "display:none"))
    } else {
        let items: Vec<MereViewElement<State, Action>> = merge
            .dropped
            .iter()
            .map(|words| {
                Box::new(el::<_, State, Action>("li", words.clone()))
                    as MereViewElement<State, Action>
            })
            .collect();
        Box::new(
            el::<_, State, Action>("ul", items)
                .attr("class", "mere-view-merge-dropped")
                .attr("role", "status")
                .attr("aria-label", "Dropped choices"),
        )
    };
    let conflicts: Vec<MereViewElement<State, Action>> = merge
        .conflicts
        .iter()
        .map(|conflict| merge_conflict(view, conflict, on_change.clone(), on_request.clone()))
        .collect();
    let list = el::<_, State, Action>("ul", conflicts)
        .attr("class", "mere-view-merge-conflicts")
        .attr("aria-label", "Conflicts");
    let resolved = merge.resolved();
    let apply: MereViewElement<State, Action> = if resolved {
        request_button(
            "Apply",
            MereViewRequest::Merge(MergeRequest::Apply),
            on_request.clone(),
            vec![("aria-label", "Apply merge".to_string())],
        )
    } else {
        Box::new(
            button("Apply", |_: &mut State, _| {})
                .attr("aria-label", "Apply merge")
                .attr("aria-disabled", "true"),
        )
    };
    let cancel = request_button(
        "Cancel",
        MereViewRequest::Merge(MergeRequest::Cancel),
        on_request,
        vec![("aria-label", "Cancel merge".to_string())],
    );
    let actions =
        el::<_, State, Action>("div", (apply, cancel)).attr("class", "mere-view-merge-actions");
    Box::new(
        el::<_, State, Action>("section", (title, dropped, list, actions))
            .attr("class", "mere-view-merge")
            .attr("style", format!("top:{top}px"))
            .attr("role", "group")
            .attr(
                "aria-label",
                format!(
                    "Pending merge: {} into {}",
                    merge.source_name, merge.target_name
                ),
            )
            .attr("data-resolved", resolved.to_string()),
    )
}

/// One conflict: its part, its base, source and target values, its choices
/// and the hand value field.
pub(super) fn merge_conflict<State, Action, Change, Request>(
    view: &MereView<'_>,
    conflict: &MergeConflictEntry,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let words = |value: &str| {
        if value.is_empty() {
            "absent".to_string()
        } else {
            value.to_string()
        }
    };
    let seed = match &conflict.chosen {
        Some(MergeOption::Hand(text)) => text.clone(),
        _ => conflict.target.clone(),
    };
    let draft = view
        .state
        .merge_drafts
        .get(&conflict.key)
        .cloned()
        .unwrap_or_else(|| TextInput::new(seed.clone()));
    let label = &conflict.label;
    let mut options = vec![
        (
            MergeOption::Source,
            "Source",
            format!("Take the source's {label}"),
        ),
        (
            MergeOption::Target,
            "Target",
            format!("Keep the target's {label}"),
        ),
        (
            MergeOption::Hand(draft.text().to_string()),
            "Edit by hand",
            format!("Use the hand value for {label}"),
        ),
    ];
    if conflict.add_as_new {
        options.push((
            MergeOption::AddAsNew,
            "Add as new",
            format!("Add {label} as new"),
        ));
    }
    let chosen = conflict.chosen.as_ref().map(MergeOption::token);
    let choices: Vec<MereViewElement<State, Action>> = options
        .into_iter()
        .map(|(option, text, aria)| {
            let pressed = chosen == Some(option.token());
            request_button(
                text,
                MereViewRequest::Merge(MergeRequest::Choose(conflict.key.clone(), option.clone())),
                on_request.clone(),
                vec![
                    ("aria-label", aria),
                    ("aria-pressed", pressed.to_string()),
                    ("data-choice", option.token().to_string()),
                ],
            )
        })
        .collect();
    let key = conflict.key.clone();
    let field = on_key(
        el::<_, State, Action>("div", caret_field_children::<State, Action>(&draft, &[]))
            .attr("class", "mere-view-merge-hand")
            .attr("role", "textbox")
            .attr("aria-label", format!("Hand value for {label}"))
            .attr("data-cambium-text-value", draft.text().to_string()),
        move |state: &mut State, event: KeyEvent| {
            if let Some(command) = text_command_from_key(&event, TextFieldMode::SingleLine) {
                on_change(
                    state,
                    MereViewEvent::MergeDraft {
                        key: key.clone(),
                        seed: seed.clone(),
                        command,
                    },
                );
            }
        },
    );
    let line =
        |class: &'static str, text: String| el::<_, State, Action>("p", text).attr("class", class);
    Box::new(
        el::<_, State, Action>(
            "li",
            (
                line("mere-view-merge-part", label.clone()),
                line(
                    "mere-view-merge-base",
                    format!("Base: {}", words(&conflict.base)),
                ),
                line(
                    "mere-view-merge-source",
                    format!("Source: {}", words(&conflict.source)),
                ),
                line(
                    "mere-view-merge-target",
                    format!("Target: {}", words(&conflict.target)),
                ),
                el::<_, State, Action>("div", choices).attr("class", "mere-view-merge-choices"),
                field,
            ),
        )
        .attr("class", "mere-view-merge-conflict")
        .attr("data-key", conflict.key.clone())
        .attr("data-chosen", chosen.unwrap_or("none")),
    )
}

pub(super) fn status_panel<State, Action, Request>(
    token: &'static str,
    note: &StatusNote,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let message =
        el::<_, State, Action>("p", note.message.clone()).attr("class", "mere-view-state-message");
    let action: Vec<MereViewElement<State, Action>> = note
        .action
        .iter()
        .map(|action| {
            request_button(
                action.label.clone(),
                MereViewRequest::Host(action.key.clone()),
                on_request.clone(),
                vec![("data-action", action.key.clone())],
            )
        })
        .collect();
    Box::new(
        el::<_, State, Action>("div", (message, action))
            .attr("class", "mere-view-state")
            .attr("role", "status")
            .attr("data-status", token),
    )
}
