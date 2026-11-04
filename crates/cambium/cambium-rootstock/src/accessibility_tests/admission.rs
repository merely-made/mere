// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn current_target_admission_rechecks_each_request_and_reports_refusals() {
    use crate::{A11yAction as Action, A11yDispatch as Dispatch, A11yRejection as Refusal};
    let mut host = counter_host();
    let button = host.s.runner.as_ref().unwrap().focusables()[0];
    let requests = [
        A11yRequest::node(Action::Click, button),
        A11yRequest::node(Action::Click, button),
    ];
    assert_eq!(
        host.apply_a11y_requests(&requests),
        [Dispatch::Dispatched, Dispatch::Rejected(Refusal::Disabled)]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 1);
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(Action::SetValue(f64::NAN), button)]),
        [Dispatch::Rejected(Refusal::InvalidValue)]
    );
}

#[test]
fn current_target_admission_refuses_plain_and_detached_nodes() {
    use crate::{A11yAction as Action, A11yDispatch as Dispatch, A11yRejection as Refusal};
    use layout_dom_api::{LayoutDomMut as _, LocalName, Namespace, QualName};
    let mut host = counter_host();
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let mount = runner.mount();
    let mut dom = dom.borrow_mut();
    let mut pending = vec![mount];
    let plain = loop {
        let node = pending.pop().expect("plain fixture node");
        if dom.attribute(node, &Namespace::default(), &LocalName::from("id")) == Some("plain") {
            break node;
        }
        pending.extend(dom.dom_children(node));
    };
    let detached = dom.create_element(QualName::new(
        None,
        Namespace::default(),
        LocalName::from("button"),
    ));
    drop(dom);
    assert_eq!(
        host.apply_a11y_requests(&[
            A11yRequest::node(Action::Click, plain),
            A11yRequest::node(Action::Focus, detached),
        ]),
        [
            Dispatch::Rejected(Refusal::Unsupported),
            Dispatch::Rejected(Refusal::TargetAbsent)
        ]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
    assert_eq!(host.s.runner.as_ref().unwrap().focus(), None);
}

#[test]
fn current_producer_action_requires_a_present_slot_and_advertised_action() {
    use crate::*;
    struct Board {
        offered: Rc<RefCell<bool>>,
        calls: Rc<RefCell<usize>>,
    }
    impl TextureProducer for Board {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn semantics(&self) -> Option<ProducerSemantics> {
            Some(ProducerSemantics {
                children: vec![ProducerNode {
                    key: 40,
                    role: ProducerRole::ListItem,
                    name: "Card".into(),
                    rect: [0.0, 0.0, 10.0, 10.0],
                    actions: if *self.offered.borrow() {
                        vec![ProducerAction {
                            id: "pin".into(),
                            label: "Pin".into(),
                            description: "Pin".into(),
                        }]
                    } else {
                        vec![]
                    },
                }],
                ..Default::default()
            })
        }
        fn act(&mut self, _: u64, _: &str) -> bool {
            *self.calls.borrow_mut() += 1;
            false
        }
    }
    let mut host = counter_host();
    let offered = Rc::new(RefCell::new(true));
    let calls = Rc::new(RefCell::new(0));
    host.s
        .shared
        .producers
        .register(
            7,
            Board {
                offered: offered.clone(),
                calls: calls.clone(),
            },
            &[],
        )
        .unwrap();
    host.s.shared.leaves.insert(
        7,
        Box::new(sprigging::Knob::new(sprigging::Size {
            width: 80.0,
            height: 30.0,
        })),
    );
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let slot = host
        .s
        .layout
        .as_ref()
        .unwrap()
        .custom_leaf_nodes(&WindowDom::new(&dom.borrow(), runner.mount()))[0]
        .1;
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(A11yAction::SetValue(8.0), slot)]),
        [A11yDispatch::Rejected(A11yRejection::Unsupported)]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
    let request = A11yRequest::produced(
        A11yAction::Click,
        ProducedAction {
            slot: 7,
            key: 40,
            id: "pin".into(),
        },
    );
    assert_eq!(
        host.apply_a11y_requests(&[request.clone()]),
        [A11yDispatch::ProducerDeclined]
    );
    assert_eq!(*calls.borrow(), 1);
    assert_eq!(
        host.s.runner.as_ref().unwrap().focus(),
        None,
        "declined activation does not move focus"
    );
    *offered.borrow_mut() = false;
    assert_eq!(
        host.apply_a11y_requests(&[request.clone()]),
        [A11yDispatch::Rejected(A11yRejection::Unsupported)]
    );
    *offered.borrow_mut() = true;
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let slot = host
        .s
        .layout
        .as_ref()
        .unwrap()
        .custom_leaf_nodes(&WindowDom::new(&dom.borrow(), runner.mount()))[0]
        .1;
    use layout_dom_api::LayoutDomMut as _;
    dom.borrow_mut().remove(slot);
    assert_eq!(
        host.apply_a11y_requests(&[request]),
        [A11yDispatch::Rejected(A11yRejection::TargetAbsent)]
    );
    assert_eq!(*calls.borrow(), 1);
}

#[test]
fn identity_tests_current_admission_refuses_ambiguous_producer_action_without_dispatch() {
    use crate::*;
    struct Ambiguous(Rc<RefCell<usize>>, Rc<RefCell<bool>>);
    impl TextureProducer for Ambiguous {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn semantics(&self) -> Option<ProducerSemantics> {
            let action = ProducerAction {
                id: "pin".into(),
                label: "Pin".into(),
                description: "Keep card".into(),
            };
            Some(ProducerSemantics {
                children: vec![ProducerNode {
                    key: 40,
                    role: ProducerRole::ListItem,
                    name: "Card".into(),
                    rect: [0.0, 0.0, 10.0, 10.0],
                    actions: if *self.1.borrow() {
                        vec![action.clone(), action]
                    } else {
                        vec![action]
                    },
                }],
                ..Default::default()
            })
        }
        fn act(&mut self, _: u64, _: &str) -> bool {
            *self.0.borrow_mut() += 1;
            true
        }
    }
    let mut host = counter_host();
    let calls = Rc::new(RefCell::new(0));
    let duplicate = Rc::new(RefCell::new(true));
    host.s
        .shared
        .producers
        .register(7, Ambiguous(calls.clone(), duplicate.clone()), &[])
        .unwrap();
    let request = A11yRequest::produced(
        A11yAction::Click,
        ProducedAction {
            slot: 7,
            key: 40,
            id: "pin".into(),
        },
    );
    assert_eq!(
        host.apply_a11y_requests(&[request.clone()]),
        [A11yDispatch::Rejected(A11yRejection::TargetAbsent)]
    );
    assert_eq!(*calls.borrow(), 0);
    assert_eq!(host.s.runner.as_ref().unwrap().focus(), None);
    *duplicate.borrow_mut() = false;
    assert_eq!(
        host.apply_a11y_requests(&[request.clone()]),
        [A11yDispatch::Dispatched]
    );
    assert_eq!(*calls.borrow(), 1);
    *duplicate.borrow_mut() = true;
    assert_eq!(
        host.apply_a11y_requests(&[request]),
        [A11yDispatch::Rejected(A11yRejection::TargetAbsent)]
    );
    assert_eq!(
        *calls.borrow(),
        1,
        "readmission uses the current declaration"
    );
}

#[test]
fn current_leaf_contribution_can_admit_a_numeric_action() {
    use crate::{A11yAction, A11yDispatch};
    let mut host = counter_host();
    host.s.shared.leaves.insert(
        7,
        Box::new(sprigging::Knob::new(sprigging::Size {
            width: 80.0,
            height: 30.0,
        })),
    );
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let slot = host
        .s
        .layout
        .as_ref()
        .unwrap()
        .custom_leaf_nodes(&WindowDom::new(&dom.borrow(), runner.mount()))[0]
        .1;
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(A11yAction::SetValue(4.0), slot)]),
        [A11yDispatch::Dispatched]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 4);
    use layout_dom_api::LayoutDomMut as _;
    dom.borrow_mut()
        .set_attribute(slot, cambium::attr_qual("aria-readonly"), "true");
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(A11yAction::SetValue(8.0), slot)]),
        [A11yDispatch::Rejected(crate::A11yRejection::Unsupported)]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 4);
}

#[test]
fn action_admission_gives_context_sensitive_leaf_the_published_seed() {
    use crate::{A11yAction, A11yDispatch, A11yRejection};
    use layout_dom_api::LayoutDomMut as _;
    struct Seeded;
    impl sprigging::Leaf for Seeded {
        fn measure(&mut self, _: sprigging::SizeHint, _: sprigging::SizeHint) -> sprigging::Size {
            sprigging::Size {
                width: 80.0,
                height: 30.0,
            }
        }
        fn paint(&mut self, _: &mut sprigging::PaintCx<'_>) {}
        fn paint_dirty(&self) -> bool {
            false
        }
        fn accessibility(&self, node: &mut accesskit::Node) {
            assert_eq!(node.label(), Some("Gain"));
            assert!(node.bounds().is_some());
            node.set_role(accesskit::Role::Slider);
            node.add_action(accesskit::Action::SetValue);
        }
    }
    let mut host = counter_host();
    host.s.shared.leaves.insert(7, Box::new(Seeded));
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let slot = host
        .s
        .layout
        .as_ref()
        .unwrap()
        .custom_leaf_nodes(&WindowDom::new(&dom.borrow(), runner.mount()))[0]
        .1;
    dom.borrow_mut()
        .set_attribute(slot, cambium::attr_qual("aria-label"), "Gain");
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(A11yAction::SetValue(4.0), slot)]),
        [A11yDispatch::Dispatched]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 4);
    dom.borrow_mut()
        .set_attribute(slot, cambium::attr_qual("aria-disabled"), "true");
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(A11yAction::SetValue(8.0), slot)]),
        [A11yDispatch::Rejected(A11yRejection::Disabled)]
    );
    dom.borrow_mut()
        .set_attribute(slot, cambium::attr_qual("aria-hidden"), "true");
    assert_eq!(
        host.apply_a11y_requests(&[A11yRequest::node(A11yAction::SetValue(8.0), slot)]),
        [A11yDispatch::Rejected(A11yRejection::TargetAbsent)]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 4);
}
