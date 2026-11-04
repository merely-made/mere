// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{DomHandle, GenetAppRunner, PointerClick};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use scenograph::link::{
    LinkAssertionBasis, LinkCoverage, LinkEndpoint, LinkGraphScope, LinkLegacyMember, LinkReading,
    LinkSourceRevision, LinkStratum, LinkWithdrawal,
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
            .all(|target| target.source == original.source && target.link == original.links[0].key)
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
