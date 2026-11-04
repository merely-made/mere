/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

pub(super) fn card<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, MereNodeKind>,
    on_change: Change,
    on_request: Request,
) -> Option<(String, MereViewElement<State, Action>)>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let open = view.open_card()?;
    let region = swatch.projected_node_footprint(&card_node_key(open))?;
    let session = match open {
        TierCard::Session(id) => model.sessions.iter().find(|session| session.id == id),
        TierCard::Shelf => None,
    };
    let (title, close_label) = match session {
        Some(session) => (
            session.name.clone(),
            format!("Close session {}", session.name),
        ),
        None => ("Archive".to_string(), "Close the archive".to_string()),
    };
    let close = {
        let on_change = on_change.clone();
        request_focus(
            button("Close", move |state: &mut State, _| {
                on_change(state, MereViewEvent::Expand(None))
            })
            .attr("class", "mere-view-card-close")
            .attr(
                "data-request",
                if session.is_some() {
                    "close-session"
                } else {
                    "close-shelf"
                },
            )
            .attr("aria-label", close_label),
            true,
        )
    };
    let head = el::<_, State, Action>(
        "div",
        (
            el::<_, State, Action>("span", title).attr("class", "mere-view-session-name"),
            close,
        ),
    )
    .attr("class", "mere-view-card-head");
    let popover = view.state.archive_popover.as_ref();
    let chips: Vec<MereViewElement<State, Action>> = card_archives(model, open)
        .into_iter()
        .map(|entry| {
            let key = entry.key.clone();
            let on_change = on_change.clone();
            let pinned = popover.is_some_and(|(shown, popover)| {
                *shown == entry.key && popover.mode == DetailPopoverMode::Pinned
            });
            // A chip takes focus back when its popover closes and it has no
            // node of its own (its archives collapsed into "+N").
            let returns = view.state.focus_return.as_deref()
                == Some(archive_node_key(&entry.key).as_str())
                && !swatch
                    .graph
                    .nodes
                    .iter()
                    .any(|node| node.id == archive_node_key(&entry.key));
            Box::new(
                el::<_, State, Action>(
                    "li",
                    request_focus(
                        button(entry.name.clone(), move |state: &mut State, _| {
                            on_change(
                                state,
                                MereViewEvent::ArchivePopover(
                                    key.clone(),
                                    DetailPopoverEvent::TogglePinned,
                                ),
                            )
                        })
                        .attr("class", "mere-view-archive-chip")
                        .attr("data-archive-chip", entry.key.clone())
                        .attr("aria-haspopup", "dialog")
                        .attr("aria-expanded", pinned.to_string()),
                        returns,
                    ),
                )
                .attr("class", "mere-view-archive"),
            ) as MereViewElement<State, Action>
        })
        .collect();
    let controls: Vec<MereViewElement<State, Action>> = match session {
        Some(session) => vec![
            Box::new(
                el::<_, State, Action>("span", session_words(session))
                    .attr("class", "mere-view-session-state"),
            ),
            Box::new(
                el::<_, State, Action>(
                    "div",
                    session_steps(session, can_save(model, session), on_request),
                )
                .attr("class", "mere-view-session-steps"),
            ),
        ],
        None => can_compose(model)
            .then(|| {
                Box::new(
                    el::<_, State, Action>("div", compose_button(view, on_request))
                        .attr("class", "mere-view-session-steps"),
                ) as MereViewElement<State, Action>
            })
            .into_iter()
            .collect(),
    };
    let body = (
        head,
        controls,
        el::<_, State, Action>("ul", chips)
            .attr("class", "mere-view-container-archives")
            .attr("aria-label", "Archives"),
    );
    let style = format!(
        "left:{}px;top:{}px;width:{}px;height:{}px",
        region.left, region.top, region.width, region.height
    );
    let card = match session {
        Some(session) => el::<_, State, Action>("section", body)
            .attr(
                "class",
                "mere-view-container mere-view-session mere-view-card",
            )
            .attr("data-container", "session")
            .attr("role", "group")
            .attr("aria-label", format!("Session {}", session.name))
            .attr("data-session", session.id.as_uuid().to_string())
            .attr("data-attached", session.attached.to_string())
            .attr("data-trashed", session.trashed.to_string())
            .attr("style", style),
        None => el::<_, State, Action>("section", body)
            .attr(
                "class",
                "mere-view-container mere-view-shelf mere-view-card",
            )
            .attr("data-container", "shelf")
            .attr("role", "group")
            .attr("aria-label", "Archive")
            .attr("style", style),
    };
    Some((
        card_token(open),
        Box::new(
            on_key(card, move |state: &mut State, event| {
                if matches!(event.key, Key::Named(NamedKey::Escape)) {
                    event.prevent_default();
                    on_change(state, MereViewEvent::Expand(None));
                }
            })
            .focusable(false),
        ),
    ))
}
