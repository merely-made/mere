/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// Render the mere view.
///
/// `on_change` receives presentation events for [`MereViewState::apply`];
/// `on_request` receives what the person asked for.
pub fn mere_view<State, Action, Change, Request>(
    view: MereView<'_>,
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
    let band = view.overmap_height();
    let mut notice = el::<_, State, Action>("p", model.notice.clone().unwrap_or_default())
        .attr("class", "mere-view-notice")
        .attr("role", "status")
        .attr("aria-live", "polite");
    if band > 0 {
        // Below the band, over the top of the graph, where it covers no
        // container.
        notice = notice.attr("style", format!("top:{}px", band + 8));
    } else if tier_shown(model) {
        // Over the foot of the canvas, clear of the tier across its top.
        notice = notice.attr("style", "top:auto;bottom:8px");
    }
    let content = match model.status.note() {
        None if model.hide_graph => {
            Box::new(el::<_, State, Action>("div", ())) as MereViewElement<State, Action>
        },
        None => graph(&view, on_change.clone(), on_request.clone()),
        Some(note) => status_panel(model.status.token(), note, on_request.clone()),
    };
    // The notice rides over the top of the graph's area, where it covers no
    // control and needs no room of its own.
    let (width, height) = if model.hide_graph {
        (
            view.width,
            if model.status.note().is_some() {
                128
            } else {
                32
            },
        )
    } else {
        view.graph_size()
    };
    let layer: MereViewElement<State, Action> = if band > 0 {
        overmap(&view, band, on_change.clone(), on_request.clone())
    } else {
        Box::new(el::<_, State, Action>("div", ()).attr("style", "display:none"))
    };
    let main = el::<_, State, Action>("div", (notice, layer, content))
        .attr("class", "mere-view-main")
        .attr(
            "style",
            format!("width:{width}px;height:{}px", height + band),
        );
    let merge: MereViewElement<State, Action> = match &model.merge {
        Some(merge) => merge_card(&view, merge, on_change.clone(), on_request.clone()),
        None => Box::new(el::<_, State, Action>("div", ()).attr("style", "display:none")),
    };
    let body = el::<_, State, Action>(
        "div",
        (sessions(&view, on_change, on_request.clone()), main),
    )
    .attr("class", "mere-view-body");
    Box::new(
        el::<_, State, Action>("section", (bar(&view, on_request), body, merge))
            .attr("class", "mere-view")
            .attr(
                "style",
                format!("width:{}px;height:{}px", view.width, view.height),
            )
            .attr("role", "region")
            .attr("aria-label", format!("Mere: {}", model.title))
            .attr(
                "data-width",
                if wide(view.width) { "wide" } else { "narrow" },
            )
            .attr("data-status", model.status.token())
            .attr(
                "data-graph",
                if model.hide_graph { "hidden" } else { "shown" },
            )
            .attr("data-sessions", model.placement.token()),
    )
}
