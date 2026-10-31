/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

struct Registry;
impl CompositionValidator for Registry {
    fn validate_component(
        &self,
        item: &ComponentDescriptor,
        previous: Option<&ComponentDescriptor>,
    ) -> Result<BindingAdmission, String> {
        if item.kind.0 != "text" && item.kind.0 != "chart" {
            return Err("unregistered kind".into());
        }
        if item.config.get("refuse") == Some(&Value::Bool(true)) {
            return Err("configuration refused".into());
        }
        if item.binding.get("malformed") == Some(&Value::Bool(true)) {
            return if previous
                .is_some_and(|old| old.kind == item.kind && old.binding == item.binding)
            {
                Ok(BindingAdmission::PreviouslyAuthoredUnavailable)
            } else {
                Err("unsupported binding".into())
            };
        }
        Ok(BindingAdmission::Available)
    }
}

fn descriptor(key: &str) -> ComponentDescriptor {
    ComponentDescriptor {
        key: key.into(),
        kind: "text".into(),
        config: Value::Null,
        binding: Value::Null,
    }
}

fn active(key: &str) -> PlacementNode {
    PlacementNode::Component { key: key.into() }
}

fn base() -> Composition {
    Composition {
        components: vec![
            descriptor("a"),
            descriptor("b"),
            descriptor("hidden"),
            descriptor("removed"),
        ],
        placement: vec![
            active("a"),
            PlacementNode::Group {
                key: "g".into(),
                children: vec![active("b")],
            },
        ],
        unplaced: vec!["hidden".into(), "removed".into()],
    }
}

#[test]
fn transaction_diff_classifies_component_and_structure_edits() {
    let mut tx =
        CompositionTransaction::new(base(), CompositionLimits::default(), &Registry).unwrap();
    let mut a = descriptor("a");
    a.kind = "chart".into();
    a.config = serde_json::json!({"color":"blue"});
    a.binding = serde_json::json!({"source":"new"});
    let mut b = descriptor("b");
    b.config = serde_json::json!({"changed":true});
    let candidate = Composition {
        components: vec![a, b, descriptor("hidden"), descriptor("created")],
        placement: vec![
            active("b"),
            active("a"),
            PlacementNode::Drawer {
                key: "d".into(),
                children: vec![active("hidden"), active("created")],
            },
        ],
        unplaced: vec![],
    };
    let edits = tx.apply(candidate, &Registry).unwrap();
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::Create(item) if item.key.0 == "created"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::Remove(key) if key.0 == "removed"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::ReplaceKind { key, .. } if key.0 == "a"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::ChangeBinding { key, .. } if key.0 == "a"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::ChangeConfig { key, .. } if key.0 == "a"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::Regroup { key, .. } if key.0 == "b"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::Move { key, .. } if key.0 == "a"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::Show { key, .. } if key.0 == "hidden"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::CreateGroup(container) if container.key == "d"))
    );
    assert!(
        edits
            .iter()
            .any(|e| matches!(e, CompositionEdit::RemoveGroup(container) if container.key == "g"))
    );
    assert!(edits.iter().any(|e| matches!(e, CompositionEdit::ReorderChildren(container) if container.kind == ContainerKind::Root)));

    let mut hide = tx.accepted().clone();
    hide.placement
        .retain(|node| !matches!(node, PlacementNode::Component { key } if key.0 == "a"));
    hide.unplaced.push("a".into());
    assert!(
        tx.apply(hide, &Registry)
            .unwrap()
            .iter()
            .any(|e| matches!(e, CompositionEdit::Hide(key) if key.0 == "a"))
    );
}

#[test]
fn equal_configuration_is_a_noop_and_edit_order_is_deterministic() {
    let model = base();
    let mut first =
        CompositionTransaction::new(model.clone(), CompositionLimits::default(), &Registry)
            .unwrap();
    let mut second =
        CompositionTransaction::new(model.clone(), CompositionLimits::default(), &Registry)
            .unwrap();
    assert!(first.apply(model.clone(), &Registry).unwrap().is_empty());
    let mut changed = model.clone();
    changed.components[0].config = serde_json::json!({"x":1});
    assert_eq!(
        first.apply(changed.clone(), &Registry).unwrap(),
        second.apply(changed, &Registry).unwrap()
    );
}

#[test]
fn invalid_placements_limits_and_app_refusal_leave_accepted_snapshot_unchanged() {
    let limits = CompositionLimits::default();
    let mut tx = CompositionTransaction::new(base(), limits, &Registry).unwrap();
    let accepted = tx.accepted().clone();
    let mut duplicate = accepted.clone();
    duplicate.placement.push(active("a"));
    assert!(matches!(
        tx.apply(duplicate, &Registry),
        Err(CompositionError::DuplicatePlacement(_))
    ));
    let mut unknown = accepted.clone();
    unknown.placement.push(active("ghost"));
    assert!(matches!(
        tx.apply(unknown, &Registry),
        Err(CompositionError::UnknownComponent(_))
    ));
    let mut duplicate_descriptor = accepted.clone();
    duplicate_descriptor
        .components
        .push(duplicate_descriptor.components[0].clone());
    assert!(matches!(
        tx.apply(duplicate_descriptor, &Registry),
        Err(CompositionError::DuplicateComponent(_))
    ));
    let mut duplicate_group = accepted.clone();
    duplicate_group.placement = vec![
        PlacementNode::Group {
            key: "same".into(),
            children: vec![active("a")],
        },
        PlacementNode::Group {
            key: "same".into(),
            children: vec![active("b")],
        },
    ];
    assert!(matches!(
        tx.apply(duplicate_group, &Registry),
        Err(CompositionError::DuplicateGroup(_))
    ));
    let mut duplicate_unplaced = accepted.clone();
    duplicate_unplaced.unplaced.push("hidden".into());
    assert!(matches!(
        tx.apply(duplicate_unplaced, &Registry),
        Err(CompositionError::DuplicateUnplaced(_))
    ));
    let mut too_many = accepted.clone();
    too_many.components.push(descriptor("extra"));
    too_many.placement.push(active("extra"));
    let count_limit = CompositionLimits {
        max_components: accepted.components.len(),
        ..limits
    };
    let mut count_bounded =
        CompositionTransaction::new(accepted.clone(), count_limit, &Registry).unwrap();
    assert!(matches!(
        count_bounded.apply(too_many, &Registry),
        Err(CompositionError::TooManyComponents { .. })
    ));
    let mut too_deep = accepted.clone();
    let mut nested = active("a");
    for n in 0..=MAX_PLACEMENT_DEPTH {
        nested = PlacementNode::Group {
            key: format!("g{n}").into(),
            children: vec![nested],
        };
    }
    too_deep.placement = vec![nested];
    let unbounded_depth = CompositionLimits {
        max_depth: usize::MAX,
        ..limits
    };
    let mut hard_bounded =
        CompositionTransaction::new(accepted.clone(), unbounded_depth, &Registry).unwrap();
    assert!(matches!(
        hard_bounded.apply(too_deep.clone(), &Registry),
        Err(CompositionError::PlacementTooDeep { .. })
    ));
    assert!(matches!(
        tx.apply(too_deep, &Registry),
        Err(CompositionError::PlacementTooDeep { .. })
    ));
    let mut too_large = accepted.clone();
    too_large.components[0].config = Value::String("x".repeat(MAX_ENCODED_BYTES));
    let small = CompositionLimits {
        max_encoded_bytes: 512,
        ..limits
    };
    let mut bounded = CompositionTransaction::new(accepted.clone(), small, &Registry).unwrap();
    assert!(matches!(
        bounded.apply(too_large, &Registry),
        Err(CompositionError::EncodedTooLarge { .. })
    ));
    let mut too_many_groups = accepted.clone();
    too_many_groups.placement = vec![PlacementNode::Group {
        key: "one".into(),
        children: vec![active("a"), active("b")],
    }];
    too_many_groups.unplaced.push("hidden".into());
    too_many_groups.unplaced.push("removed".into());
    let group_limit = CompositionLimits {
        max_groups: 0,
        ..limits
    };
    let mut group_bounded =
        CompositionTransaction::new(Composition::default(), group_limit, &Registry).unwrap();
    assert!(matches!(
        group_bounded.apply(too_many_groups, &Registry),
        Err(CompositionError::TooManyGroups { .. })
    ));
    let mut too_deep_payload = accepted.clone();
    let mut nested_value = Value::Null;
    for _ in 0..=MAX_PAYLOAD_DEPTH {
        nested_value = Value::Array(vec![nested_value]);
    }
    too_deep_payload.components[0].config = nested_value;
    assert!(matches!(
        tx.apply(too_deep_payload, &Registry),
        Err(CompositionError::PayloadTooDeep { .. })
    ));
    let mut refused = accepted.clone();
    refused.components[0].config = serde_json::json!({"refuse":true});
    assert!(matches!(
        tx.apply(refused, &Registry),
        Err(CompositionError::InvalidComponent { .. })
    ));
    let mut unsupported_kind = accepted.clone();
    unsupported_kind.components[0].kind = "unknown-kind".into();
    assert!(matches!(
        tx.apply(unsupported_kind, &Registry),
        Err(CompositionError::InvalidComponent { .. })
    ));
    let mut unsupported_new_binding = accepted.clone();
    unsupported_new_binding.components[0].binding = serde_json::json!({"malformed":true});
    assert!(matches!(
        tx.apply(unsupported_new_binding, &Registry),
        Err(CompositionError::InvalidComponent { .. })
    ));
    assert_eq!(tx.accepted(), &accepted);
    assert_eq!(bounded.accepted(), &accepted);
}

#[test]
fn unavailable_binding_requires_exact_previous_authored_binding() {
    let mut prior = descriptor("a");
    prior.binding = serde_json::json!({"malformed":true});
    let model = Composition {
        components: vec![prior.clone()],
        placement: vec![active("a")],
        unplaced: vec![],
    };
    assert!(
        CompositionTransaction::new(model.clone(), CompositionLimits::default(), &Registry)
            .is_err()
    );
    let mut tx = CompositionTransaction::restore_authored(
        model.clone(),
        CompositionLimits::default(),
        &Registry,
    )
    .unwrap();
    assert_eq!(
        tx.apply(model.clone(), &Registry).unwrap(),
        Vec::<CompositionEdit>::new()
    );
    let mut newly_unsupported = model.clone();
    let mut new_item = descriptor("new");
    new_item.binding = serde_json::json!({"malformed":true});
    newly_unsupported.components.push(new_item);
    newly_unsupported.placement.push(active("new"));
    assert!(tx.apply(newly_unsupported, &Registry).is_err());
    assert_eq!(tx.accepted(), &model);
    let mut changed = model;
    changed.components[0].binding = serde_json::json!({"malformed":true,"new":true});
    assert!(tx.apply(changed, &Registry).is_err());
}
