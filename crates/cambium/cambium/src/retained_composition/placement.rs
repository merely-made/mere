// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, BTreeSet};

use super::{ROOT_CONTAINER, RetainedState};
use crate::composition::{
    ComponentKey, Composition, ContainerKey, ContainerKind, PlacementLocation, PlacementNode,
};
#[derive(Clone, Debug)]
pub(super) struct Placement {
    pub(super) container: ContainerKey,
    pub(super) location: PlacementLocation,
    pub(super) signature: Vec<(ContainerKey, usize)>,
    pub(super) hidden: bool,
}

pub(super) fn placements(
    model: &Composition,
    collapsed: &BTreeSet<ContainerKey>,
) -> BTreeMap<ComponentKey, Placement> {
    fn visit(
        nodes: &[PlacementNode],
        container: &ContainerKey,
        path: &[ContainerKey],
        signature: &[(ContainerKey, usize)],
        inherited_hidden: bool,
        collapsed: &BTreeSet<ContainerKey>,
        result: &mut BTreeMap<ComponentKey, Placement>,
    ) {
        for (index, node) in nodes.iter().enumerate() {
            match node {
                PlacementNode::Component { key } => {
                    result.insert(
                        key.clone(),
                        Placement {
                            container: container.clone(),
                            location: PlacementLocation {
                                containers: path.to_vec(),
                                index,
                            },
                            signature: signature
                                .iter()
                                .cloned()
                                .chain(std::iter::once((container.clone(), index)))
                                .collect(),
                            hidden: inherited_hidden,
                        },
                    );
                },
                PlacementNode::Group { key, children } => {
                    let next = ContainerKey {
                        kind: ContainerKind::Group,
                        key: key.0.clone(),
                    };
                    let mut child_path = path.to_vec();
                    child_path.push(next.clone());
                    let mut child_signature = signature.to_vec();
                    child_signature.push((container.clone(), index));
                    visit(
                        children,
                        &next,
                        &child_path,
                        &child_signature,
                        inherited_hidden || collapsed.contains(&next),
                        collapsed,
                        result,
                    );
                },
                PlacementNode::Drawer { key, children } => {
                    let next = ContainerKey {
                        kind: ContainerKind::Drawer,
                        key: key.0.clone(),
                    };
                    let mut child_path = path.to_vec();
                    child_path.push(next.clone());
                    let mut child_signature = signature.to_vec();
                    child_signature.push((container.clone(), index));
                    visit(
                        children,
                        &next,
                        &child_path,
                        &child_signature,
                        inherited_hidden || collapsed.contains(&next),
                        collapsed,
                        result,
                    );
                },
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(
        &model.placement,
        &ROOT_CONTAINER,
        &[],
        &[],
        false,
        collapsed,
        &mut result,
    );
    for key in &model.unplaced {
        result.insert(
            key.clone(),
            Placement {
                container: ROOT_CONTAINER.clone(),
                location: PlacementLocation {
                    containers: Vec::new(),
                    index: usize::MAX,
                },
                signature: Vec::new(),
                hidden: true,
            },
        );
    }
    result
}

pub(super) fn take_slot<State, Action>(state: &mut RetainedState<State, Action>) -> Option<u64> {
    let current = state.next_slot;
    state.next_slot = current.checked_add(1)?;
    Some(current)
}

pub(super) fn empty_placement() -> Placement {
    Placement {
        container: ROOT_CONTAINER.clone(),
        location: crate::composition::PlacementLocation {
            containers: Vec::new(),
            index: usize::MAX,
        },
        signature: Vec::new(),
        hidden: true,
    }
}
