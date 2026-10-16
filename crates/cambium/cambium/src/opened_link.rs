// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One opened-link surface for source-owned assertions. The host advertises
//! actions and handles them against its current source; these records confer
//! no authority. Layout is compiled by Scenomise, and the accessible list
//! reads the same source members as the Timeline.

use scenograph::link::{
    LinkAssertion, LinkBucket, LinkKey, LinkListGroup, LinkPairReading, LinkSource, LinkSpanEnd,
    LinkTimelineLane, OpenedLinkLayout, OpenedLinkMember, OpenedLinkOptions,
};

use crate::{
    AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, OptionalAction, View, button,
    detail_panel, el,
};

/// One operation advertised by the source adapter for this exact assertion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkMemberAction {
    pub id: String,
    pub label: String,
}

/// The source context and exact assertion observed when an action was offered.
/// Consumers must re-resolve it before writing or performing an effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkMemberTarget {
    pub source: LinkSource,
    pub link: LinkKey,
    pub from: String,
    pub to: String,
    pub assertion_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpenedLinkEvent {
    Layout(OpenedLinkLayout),
    Member {
        target: LinkMemberTarget,
        action: String,
    },
}

fn time(value: Option<u64>, absent: &str) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| absent.into())
}

fn assertion_rows(assertion: &LinkAssertion, bucket: &LinkBucket) -> Vec<DetailRow> {
    let end = match &assertion.withdrawn {
        Some(withdrawn) => withdrawn
            .at_ms
            .map(|t| format!("withdrawn at {t}"))
            .unwrap_or_else(|| "withdrawn; time unavailable".into()),
        None => "open (currently held)".into(),
    };
    let mut rows = vec![
        DetailRow::new("Assertion handle", &assertion.id),
        DetailRow::new("Predicate", &assertion.predicate),
        DetailRow::new(
            "Graph scope",
            match &assertion.scope {
                scenograph::link::LinkGraphScope::Default => "Default".into(),
                scenograph::link::LinkGraphScope::Source => "Source".into(),
                scenograph::link::LinkGraphScope::User => "User".into(),
                scenograph::link::LinkGraphScope::Agent => "Agent".into(),
                scenograph::link::LinkGraphScope::Moot => "Moot".into(),
                scenograph::link::LinkGraphScope::Custom(scope) => format!("Custom: {scope}"),
            },
        ),
        DetailRow::new(
            "Displayed direction",
            format!("{} to {}", bucket.displayed_from, bucket.displayed_to),
        ),
        DetailRow::new(
            "Asserter",
            assertion
                .asserter
                .as_deref()
                .unwrap_or("Attribution not held here"),
        ),
        DetailRow::new(
            "First to lifecycle end",
            format!("{} to {end}", time(assertion.first_asserted_ms, "Undated")),
        ),
        DetailRow::new(
            "Last confirmation (ms)",
            time(assertion.asserted_at_ms, "No confirmation time held"),
        ),
        DetailRow::new(
            "Withdrawal by",
            assertion
                .withdrawn
                .as_ref()
                .map(|w| w.by.as_deref().unwrap_or("Attribution not held here"))
                .unwrap_or("Currently held"),
        ),
        DetailRow::new(
            "Basis note",
            assertion
                .basis
                .note
                .as_deref()
                .or(assertion.label.as_deref())
                .unwrap_or("No basis note held"),
        ),
        DetailRow::new(
            "Basis method",
            assertion
                .basis
                .method
                .as_ref()
                .map(|m| {
                    m.version
                        .as_ref()
                        .map(|v| format!("{} ({v})", m.iri))
                        .unwrap_or_else(|| m.iri.clone())
                })
                .unwrap_or_else(|| "No method held".into()),
        ),
        DetailRow::new(
            "Basis evidence",
            if assertion.basis.evidence.is_empty() {
                "No evidence held".into()
            } else {
                assertion.basis.evidence.join(", ")
            },
        ),
    ];
    if assertion
        .withdrawn
        .as_ref()
        .and_then(|w| w.at_ms)
        .zip(assertion.first_asserted_ms)
        .is_some_and(|(end, first)| end < first)
    {
        rows.push(DetailRow::new(
            "Source clock order",
            "Recorded withdrawal precedes first assertion; times are preserved.",
        ));
    }
    rows
}

fn layout_choices(layout: &OpenedLinkLayout) -> Vec<(&'static str, OpenedLinkLayout)> {
    let stack = match layout {
        OpenedLinkLayout::List { stack, .. } => *stack,
        _ => sceno::Stack::default(),
    };
    let timeline = match layout {
        OpenedLinkLayout::Timeline { timeline, .. } => *timeline,
        _ => sceno::Timeline::default(),
    };
    vec![
        (
            "List by family",
            OpenedLinkLayout::List {
                group_by: LinkListGroup::Family,
                stack,
            },
        ),
        (
            "List by source",
            OpenedLinkLayout::List {
                group_by: LinkListGroup::Source,
                stack,
            },
        ),
        (
            "Timeline by family",
            OpenedLinkLayout::Timeline {
                lane_by: LinkTimelineLane::Family,
                timeline,
            },
        ),
        (
            "Timeline by asserter",
            OpenedLinkLayout::Timeline {
                lane_by: LinkTimelineLane::Asserter,
                timeline,
            },
        ),
        (
            "Timeline by direction",
            OpenedLinkLayout::Timeline {
                lane_by: LinkTimelineLane::Direction,
                timeline,
            },
        ),
    ]
}

/// Render one disclosed snapshot. `retained_history` changes row wording only;
/// the source adapter chooses held/history before supplying the reading.
pub fn opened_link_swatch<State, Action, OA, OnEvent, Actions>(
    swatch: &scenograph::swatch::Swatch,
    reading: &LinkPairReading,
    layout: &OpenedLinkLayout,
    options: OpenedLinkOptions,
    retained_history: bool,
    actions: Actions,
    on_event: OnEvent,
) -> impl View<State, Action, GenetCtx, Element = GenetElement> + use<State, Action, OA, OnEvent, Actions>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
    OnEvent: Fn(&mut State, OpenedLinkEvent) -> OA + Clone + 'static,
    Actions: Fn(&LinkSource, &LinkKey, &LinkBucket, &LinkAssertion) -> Vec<LinkMemberAction>,
{
    type Child<S, A> = Box<dyn AnyView<S, A, GenetCtx, GenetElement>>;
    let mut children: Vec<Child<State, Action>> = Vec::new();
    if let scenograph::swatch::Scope::Link(key) = &swatch.scope {
        children.push(Box::new(el("h3", format!("{:?} link", key.stratum))));
    }
    let mut controls: Vec<Child<State, Action>> = Vec::new();
    for (label, choice) in layout_choices(layout) {
        let selected = &choice == layout;
        let emit = on_event.clone();
        controls.push(Box::new(
            button(label, move |state: &mut State, _| {
                emit(state, OpenedLinkEvent::Layout(choice.clone()))
            })
            .attr("aria-pressed", selected.to_string()),
        ));
    }
    children.push(Box::new(
        el("div", controls).attr("class", "opened-link-controls"),
    ));
    let source_rows = match &reading.source {
        LinkSource::Available { binding, revision } => vec![
            DetailRow::new("Source", &binding.authority),
            DetailRow::new("Domain", &binding.domain),
            DetailRow::new("Source resource", &binding.resource),
            DetailRow::new(
                "Source revision",
                match revision {
                    scenograph::link::LinkSourceRevision::Available { revision } => {
                        revision.to_string()
                    },
                    scenograph::link::LinkSourceRevision::Unavailable { reason } => {
                        format!("Unavailable: {reason}")
                    },
                },
            ),
        ],
        LinkSource::Unavailable { reason } => vec![DetailRow::new("Source unavailable", reason)],
    };
    children.push(Box::new(detail_panel(&[DetailSection::new(
        "Link source",
        source_rows,
    )])));
    let opened = match scenomise::compile_opened_link_scope(&swatch.scope, reading, layout, options)
    {
        Ok(value) => value,
        Err(error) => {
            children.push(Box::new(el("p", error.to_string()).attr("role", "status")));
            return el("section", children).attr("class", "opened-link-swatch");
        },
    };
    if let OpenedLinkLayout::Timeline { timeline, .. } = layout {
        children.push(Box::new(el("h3", "Assertion timeline")));
        if let Some(now) = options.now_ms {
            children.push(Box::new(el(
                "p",
                format!("Held assertions observed at {now} ms"),
            )));
        }
        if let Some((first, last)) = opened.time_range_ms {
            children.push(Box::new(el(
                "p",
                format!("Record time (ms): {first} to {last}"),
            )));
        }
        children.push(Box::new(el(
            "p",
            format!(
                "Lanes: {}",
                opened
                    .lanes
                    .iter()
                    .map(|lane| lane.label.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        )));
        let minimum_y = opened
            .scene
            .items
            .iter()
            .map(|item| item.transform.translate.y)
            .reduce(f32::min)
            .unwrap_or(timeline.origin.y);
        let maximum_y = opened
            .scene
            .items
            .iter()
            .map(|item| item.transform.translate.y)
            .reduce(f32::max)
            .unwrap_or(timeline.origin.y);
        let mut plot: Vec<Child<State, Action>> = Vec::new();
        for entry in &opened.entries {
            let y = opened.scene.items[entry.instance.0 as usize]
                .transform
                .translate
                .y;
            let label = match opened.resolve(entry) {
                Some(OpenedLinkMember::Assertion(assertion)) => assertion
                    .basis
                    .note
                    .as_deref()
                    .or(assertion.label.as_deref())
                    .or(assertion.recognized_kind.as_deref())
                    .unwrap_or(&entry.label),
                _ => &entry.label,
            };
            let interval = entry
                .span
                .as_ref()
                .map(|span| match span.end {
                    LinkSpanEnd::WithdrawalAt(end) => format!("{} to {end}", span.start_ms),
                    LinkSpanEnd::WithdrawalTimeUnknown => {
                        format!("{} to ? (withdrawn)", span.start_ms)
                    },
                    LinkSpanEnd::OpenAt(end) => format!("{} to {end} (held)", span.start_ms),
                    LinkSpanEnd::OpenTimeUnknown => format!("{} to ? (held)", span.start_ms),
                })
                .unwrap_or_else(|| "Undated".into());
            let mut row: Vec<Child<State, Action>> = vec![Box::new(
                el("div", format!("{interval} · {label}"))
                    .attr(
                        "aria-label",
                        format!("{}: {interval}; {label}; {}", entry.lane_label, entry.label),
                    )
                    .attr(
                        "style",
                        "font-size:11px;white-space:nowrap;overflow:hidden;",
                    ),
            )];
            if let Some(span) = &entry.span {
                let percent = |value: f32| {
                    if timeline.axis_length > 0.0 {
                        ((value - timeline.origin.x) / timeline.axis_length * 100.0)
                            .clamp(0.0, 100.0)
                    } else {
                        0.0
                    }
                };
                let x = percent(span.x_start);
                let end = span.x_end.map(percent);
                let left = end.map(|end| end.min(x)).unwrap_or(x);
                let width = end.map(|end| (end - x).abs()).unwrap_or(0.0);
                let state = match span.end {
                    LinkSpanEnd::WithdrawalAt(_) => "withdrawn",
                    LinkSpanEnd::WithdrawalTimeUnknown => "withdrawn; time unavailable",
                    LinkSpanEnd::OpenAt(_) => "held; open at observation time",
                    LinkSpanEnd::OpenTimeUnknown => "held; observation time unavailable",
                };
                // Unknown ends have no x_end: render a point and keep the visible
                // question mark above it. A solid border is admitted by portable
                // presentation, including when only the top side is decorated.
                row.push(Box::new(el("div", el::<_, State, Action>("div", ())
                    .attr("style", format!("position:absolute;left:{left}%;width:{width}%;min-width:4px;border-top:5px solid currentColor;top:8px;")))
                    .attr("style", "position:relative;height:22px;margin-right:6px;border-bottom:1px solid currentColor;")
                    .attr("role", "img")
                    .attr("aria-label", format!("{}; starts {}; {state}", entry.label, span.start_ms))
                    .attr("data-span-state", state)));
            } else {
                row.push(Box::new(el("span", "Undated; no time position inferred")));
            }
            plot.push(Box::new(
                el("div", row)
                    .attr("class", "opened-link-timeline-row")
                    .attr(
                        "style",
                        format!(
                            "position:absolute;left:0;right:0;top:{}px;height:{}px;",
                            y - minimum_y,
                            timeline.row_gap
                        ),
                    )
                    .attr("data-scene-instance", entry.instance.0.to_string())
                    .attr("data-scene-row", entry.row.to_string())
                    .attr("data-scene-y", y.to_string()),
            ));
        }
        children.push(Box::new(
            el("div", plot)
                .attr(
                    "style",
                    format!(
                        "position:relative;width:100%;height:{}px;",
                        maximum_y - minimum_y + timeline.row_gap
                    ),
                )
                .attr("aria-label", "Assertion spans"),
        ));
        children.push(Box::new(el("h3", "Assertions in time order")));
    }
    let mut entries: Vec<_> = opened.entries.iter().collect();
    if matches!(layout, OpenedLinkLayout::Timeline { .. }) {
        // The alternate remains chronological even when visual lanes group
        // by asserter or direction. Unknown first times sort last.
        entries.sort_by_key(|entry| {
            let first = match opened.resolve(entry) {
                Some(OpenedLinkMember::Assertion(assertion)) => assertion.first_asserted_ms,
                _ => None,
            };
            (
                first.is_none(),
                first.unwrap_or_default(),
                entry.source.link_index,
                entry.source.bucket_index,
                entry.row,
            )
        });
    }
    let mut previous_group = None;
    let mut assertion_number = 0;
    for entry in entries {
        if matches!(layout, OpenedLinkLayout::List { .. })
            && previous_group != Some(entry.lane_index)
        {
            children.push(Box::new(el("h3", entry.lane_label.clone())));
            previous_group = Some(entry.lane_index);
        }
        let link = &opened.reading.links[entry.source.link_index];
        let bucket = &link.buckets[entry.source.bucket_index];
        match opened.resolve(entry) {
            Some(OpenedLinkMember::Assertion(assertion)) => {
                assertion_number += 1;
                let title = if retained_history {
                    format!("Retained assertion {assertion_number}")
                } else {
                    format!("Assertion {assertion_number}")
                };
                let mut rows = assertion_rows(assertion, bucket);
                rows.push(DetailRow::new(
                    "Source stratum",
                    format!("{:?}", link.key.stratum),
                ));
                children.push(Box::new(detail_panel(&[DetailSection::new(title, rows)])));
                for action in actions(&opened.reading.source, &link.key, bucket, assertion) {
                    let target = LinkMemberTarget {
                        source: opened.reading.source.clone(),
                        link: link.key.clone(),
                        from: bucket.source_from.id.clone(),
                        to: bucket.source_to.id.clone(),
                        assertion_id: assertion.id.clone(),
                    };
                    let emit = on_event.clone();
                    let id = action.id.clone();
                    let label = action
                        .label
                        .replace("{number}", &assertion_number.to_string());
                    children.push(Box::new(
                        button(label, move |state: &mut State, _| {
                            emit(
                                state,
                                OpenedLinkEvent::Member {
                                    target: target.clone(),
                                    action: id.clone(),
                                },
                            )
                        })
                        .attr("data-assertion-action", &action.id)
                        .attr("data-assertion-id", &assertion.id),
                    ));
                }
            },
            Some(OpenedLinkMember::LegacyMember(member)) => {
                children.push(Box::new(detail_panel(&[DetailSection::new(
                    "Legacy member (read only)",
                    vec![
                        DetailRow::new("Family", &member.family),
                        DetailRow::new(
                            "Kind",
                            member.sub_kind.as_deref().unwrap_or("Not specified"),
                        ),
                        DetailRow::new(
                            "Assertion identity",
                            "Unavailable; no source action offered",
                        ),
                    ],
                )])))
            },
            Some(OpenedLinkMember::LegacySummary(summary)) => {
                children.push(Box::new(detail_panel(&[DetailSection::new(
                    "Legacy summary (read only)",
                    vec![
                        DetailRow::new("Family", &summary.family),
                        DetailRow::new(
                            "Assertion identity",
                            "Unavailable; no source action offered",
                        ),
                    ],
                )])))
            },
            None => {},
        }
    }
    if opened.entries.is_empty() {
        children.push(Box::new(el("p", "No assertions disclosed for this pair")));
    }
    if !reading.coverage.limits.is_empty() {
        children.push(Box::new(el("h3", "Known limits")));
        for limit in &reading.coverage.limits {
            children.push(Box::new(
                el("p", format!("{:?}: {}", limit.layer, limit.reason)).attr("role", "status"),
            ));
        }
    }
    el("section", children).attr("class", "opened-link-swatch")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomHandle, GenetAppRunner, PointerClick};
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::{LayoutDom, LocalName, Namespace};
    use scenograph::link::{
        LinkAssertionBasis, LinkCoverage, LinkEndpoint, LinkGraphScope, LinkLegacyMember,
        LinkReading, LinkSourceRevision, LinkStratum, LinkWithdrawal,
    };
    use scenograph::{PublicSourceRevision, SourceBinding};
    use std::{cell::RefCell, rc::Rc};

    struct State {
        reading: LinkPairReading,
        events: Vec<OpenedLinkEvent>,
    }
    type Child = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;

    fn view(state: &State) -> Child {
        let swatch = scenograph::swatch::Swatch {
            id: "opened-test".into(),
            scope: scenograph::swatch::Scope::Link(state.reading.links[0].key.clone()),
            mode: scenograph::swatch::SwatchMode::Projection {
                definition_id: "opened-link.timeline".into(),
                variant_id: None,
            },
        };
        Box::new(opened_link_swatch(
            &swatch,
            &state.reading,
            &OpenedLinkLayout::Timeline {
                lane_by: LinkTimelineLane::Asserter,
                timeline: sceno::Timeline::default(),
            },
            OpenedLinkOptions { now_ms: Some(40) },
            true,
            |_source, _key, _bucket, assertion| {
                if assertion.withdrawn.is_some() {
                    return Vec::new();
                }
                vec![LinkMemberAction {
                    id: "inspect".into(),
                    label: "Inspect {number}".into(),
                }]
            },
            |state: &mut State, event| state.events.push(event),
        ))
    }

    fn reading() -> LinkPairReading {
        let assertion = |withdrawn| LinkAssertion {
            id: "same-source-local-id".into(),
            predicate: "urn:test:related".into(),
            recognized_kind: None,
            label: None,
            scope: LinkGraphScope::Default,
            asserter: Some("urn:test:asserter".into()),
            first_asserted_ms: Some(10),
            asserted_at_ms: Some(10),
            withdrawn,
            basis: LinkAssertionBasis::default(),
        };
        let bucket = |from: &str, to: &str, assertions| LinkBucket {
            displayed_from: from.into(),
            displayed_to: to.into(),
            source_from: LinkEndpoint {
                id: from.into(),
                address: None,
            },
            source_to: LinkEndpoint {
                id: to.into(),
                address: None,
            },
            assertions,
            legacy_members: vec![LinkLegacyMember {
                family: "containment".into(),
                sub_kind: None,
            }],
            legacy_summaries: Vec::new(),
        };
        LinkPairReading {
            source: LinkSource::Available {
                binding: SourceBinding {
                    authority: "test-owner".into(),
                    domain: "graph".into(),
                    resource: "g1".into(),
                },
                revision: LinkSourceRevision::Available {
                    revision: PublicSourceRevision::from("r17"),
                },
            },
            links: vec![LinkReading {
                key: LinkKey::new(LinkStratum::Surface, "a", "b"),
                buckets: vec![
                    bucket(
                        "a",
                        "b",
                        vec![
                            assertion(None),
                            assertion(Some(LinkWithdrawal {
                                at_ms: None,
                                by: None,
                            })),
                        ],
                    ),
                    bucket("b", "a", vec![assertion(None)]),
                ],
            }],
            coverage: LinkCoverage::default(),
        }
    }

    #[test]
    fn action_carries_exact_source_direction_and_id_without_legacy_or_withdrawn_actions() {
        let reading = reading();
        let original = reading.clone();
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            view as fn(&State) -> Child,
            State {
                reading,
                events: Vec::new(),
            },
        );
        let buttons = {
            let d = dom.borrow();
            let ns = Namespace::from("");
            d.all_with_class(d.document(), "opened-link-swatch")
                .into_iter()
                .flat_map(|root| descendants(&d, root))
                .filter(|node| {
                    d.attribute(*node, &ns, &LocalName::from("data-assertion-action"))
                        == Some("inspect")
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            buttons.len(),
            2,
            "withdrawn and id-less legacy members have no advertised action"
        );
        for button in buttons {
            runner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
        }
        let targets = runner
            .state()
            .events
            .iter()
            .filter_map(|event| match event {
                OpenedLinkEvent::Member { target, action } => {
                    assert_eq!(action, "inspect");
                    Some(target)
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].assertion_id, targets[1].assertion_id);
        assert_eq!(
            (&targets[0].from, &targets[0].to),
            (&"a".to_string(), &"b".to_string())
        );
        assert_eq!(
            (&targets[1].from, &targets[1].to),
            (&"b".to_string(), &"a".to_string())
        );
        assert!(
            targets
                .iter()
                .all(|target| target.source == original.source
                    && target.link == original.links[0].key)
        );
        assert_eq!(
            runner.state().reading,
            original,
            "presentation never mutates source facts"
        );
    }

    fn descendants(
        dom: &ScriptedDom,
        root: genet_scripted_dom::NodeId,
    ) -> Vec<genet_scripted_dom::NodeId> {
        let mut out = vec![root];
        for child in dom.dom_children(root) {
            out.extend(descendants(dom, child));
        }
        out
    }
}
