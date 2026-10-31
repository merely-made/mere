/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Renderer-free, bounded authored composition data and atomic transactions.
//!
//! This module describes product-authored component instances and their
//! placement. It deliberately contains no renderer state, widget lifetime,
//! provider, or product-specific configuration interpretation.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_COMPONENTS: usize = 256;
pub const MAX_GROUPS: usize = 64;
pub const MAX_PLACEMENT_DEPTH: usize = 12;
pub const MAX_PAYLOAD_DEPTH: usize = 64;
pub const MAX_ENCODED_BYTES: usize = 1_048_576;

macro_rules! key_type {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }
    };
}

key_type!(ComponentKey);
key_type!(GroupKey);
key_type!(DrawerKey);
key_type!(ComponentKind);

/// An authored component instance. Product data stays opaque to Cambium.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentDescriptor {
    pub key: ComponentKey,
    pub kind: ComponentKind,
    #[serde(default)]
    pub config: Value,
    #[serde(default)]
    pub binding: Value,
}

/// Ordered placement tree. A group or drawer may contain components and other
/// groups/drawers. Components absent from this tree belong in `unplaced`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlacementNode {
    Component {
        key: ComponentKey,
    },
    Group {
        key: GroupKey,
        children: Vec<PlacementNode>,
    },
    Drawer {
        key: DrawerKey,
        children: Vec<PlacementNode>,
    },
}

/// An authored composition snapshot. Vector order is meaningful and stable.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Composition {
    #[serde(default)]
    pub components: Vec<ComponentDescriptor>,
    #[serde(default)]
    pub placement: Vec<PlacementNode>,
    #[serde(default)]
    pub unplaced: Vec<ComponentKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositionLimits {
    pub max_components: usize,
    pub max_groups: usize,
    pub max_depth: usize,
    pub max_encoded_bytes: usize,
}

impl Default for CompositionLimits {
    fn default() -> Self {
        Self {
            max_components: MAX_COMPONENTS,
            max_groups: MAX_GROUPS,
            max_depth: MAX_PLACEMENT_DEPTH,
            max_encoded_bytes: MAX_ENCODED_BYTES,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompositionError {
    TooManyComponents { limit: usize },
    TooManyGroups { limit: usize },
    PlacementTooDeep { limit: usize },
    PayloadTooDeep { limit: usize },
    EncodedTooLarge { limit: usize },
    EmptyIdentity { kind: &'static str },
    DuplicateComponent(ComponentKey),
    DuplicateGroup(String),
    UnknownComponent(ComponentKey),
    DuplicatePlacement(ComponentKey),
    DuplicateUnplaced(ComponentKey),
    ComponentNotAccountedFor(ComponentKey),
    InvalidComponent { key: ComponentKey, reason: String },
    Serialization(String),
}

/// Validator supplied by the embedding application. Validation must not commit
/// domain writes. Return `PreviouslyAuthoredUnavailable`
/// only when deliberately retaining a previously accepted binding that this
/// application can no longer resolve. New or malformed unsupported bindings
/// must return an error. `previous` is the descriptor from the accepted model.
pub trait CompositionValidator {
    fn validate_component(
        &self,
        candidate: &ComponentDescriptor,
        previous: Option<&ComponentDescriptor>,
    ) -> Result<BindingAdmission, String>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingAdmission {
    Available,
    PreviouslyAuthoredUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContainerKey {
    pub kind: ContainerKind,
    pub key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContainerKind {
    Root,
    Group,
    Drawer,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementLocation {
    /// Root-to-leaf group/drawer path. The final key is the containing list.
    pub containers: Vec<ContainerKey>,
    pub index: usize,
}

/// Deterministic semantic edits for the application, including its opaque data.
/// These are not diagnostic records: diagnostic projection must exclude raw
/// bindings and configuration. Structural moves are derived from component
/// locations; group/drawer creation and removal describe tree structure.
#[derive(Clone, Debug, PartialEq)]
pub enum CompositionEdit {
    Create(ComponentDescriptor),
    Remove(ComponentKey),
    ReplaceKind {
        key: ComponentKey,
        from: ComponentKind,
        to: ComponentKind,
    },
    ChangeBinding {
        key: ComponentKey,
        from: Value,
        to: Value,
    },
    ChangeConfig {
        key: ComponentKey,
        from: Value,
        to: Value,
    },
    Move {
        key: ComponentKey,
        from: PlacementLocation,
        to: PlacementLocation,
    },
    Regroup {
        key: ComponentKey,
        from: Vec<ContainerKey>,
        to: Vec<ContainerKey>,
    },
    Hide(ComponentKey),
    Show {
        key: ComponentKey,
        to: PlacementLocation,
    },
    CreateGroup(ContainerKey),
    RemoveGroup(ContainerKey),
    ReorderChildren(ContainerKey),
}

pub struct CompositionTransaction {
    accepted: Composition,
    limits: CompositionLimits,
}

impl CompositionTransaction {
    pub fn new(
        accepted: Composition,
        limits: CompositionLimits,
        validator: &dyn CompositionValidator,
    ) -> Result<Self, CompositionError> {
        validate(&accepted, limits, &BTreeMap::new(), validator)?;
        Ok(Self { accepted, limits })
    }

    /// Validate a model restored from durable authored state. The accepted
    /// descriptors are supplied as their own previous version so the app
    /// validator can explicitly retain bindings that have become unavailable.
    pub fn restore_authored(
        accepted: Composition,
        limits: CompositionLimits,
        validator: &dyn CompositionValidator,
    ) -> Result<Self, CompositionError> {
        let previous = component_map(&accepted);
        validate(&accepted, limits, &previous, validator)?;
        Ok(Self { accepted, limits })
    }

    pub fn accepted(&self) -> &Composition {
        &self.accepted
    }

    /// Validate the complete candidate before computing or publishing edits.
    /// On every error the accepted snapshot remains byte-for-byte unchanged.
    pub fn apply(
        &mut self,
        candidate: Composition,
        validator: &dyn CompositionValidator,
    ) -> Result<Vec<CompositionEdit>, CompositionError> {
        let previous = component_map(&self.accepted);
        validate(&candidate, self.limits, &previous, validator)?;
        let edits = diff(&self.accepted, &candidate);
        self.accepted = candidate;
        Ok(edits)
    }
}

fn component_map(model: &Composition) -> BTreeMap<ComponentKey, &ComponentDescriptor> {
    model
        .components
        .iter()
        .map(|item| (item.key.clone(), item))
        .collect()
}

fn validate(
    model: &Composition,
    limits: CompositionLimits,
    previous: &BTreeMap<ComponentKey, &ComponentDescriptor>,
    validator: &dyn CompositionValidator,
) -> Result<(), CompositionError> {
    if model.components.len() > limits.max_components {
        return Err(CompositionError::TooManyComponents {
            limit: limits.max_components,
        });
    }
    let mut components = BTreeMap::new();
    for component in &model.components {
        if component.key.0.is_empty() {
            return Err(CompositionError::EmptyIdentity { kind: "component" });
        }
        if component.kind.0.is_empty() {
            return Err(CompositionError::EmptyIdentity { kind: "kind" });
        }
        if components
            .insert(component.key.clone(), component)
            .is_some()
        {
            return Err(CompositionError::DuplicateComponent(component.key.clone()));
        }
    }
    let mut groups = BTreeSet::new();
    let mut placed = BTreeSet::new();
    let mut locations = BTreeMap::new();
    let mut stack: Vec<(&PlacementNode, Vec<ContainerKey>, usize, usize)> = model
        .placement
        .iter()
        .rev()
        .map(|node| (node, Vec::new(), 0, 0))
        .collect();
    while let Some((node, path, index, depth)) = stack.pop() {
        match node {
            PlacementNode::Component { key } => {
                if !components.contains_key(key) {
                    return Err(CompositionError::UnknownComponent(key.clone()));
                }
                if !placed.insert(key.clone()) {
                    return Err(CompositionError::DuplicatePlacement(key.clone()));
                }
                locations.insert(
                    key.clone(),
                    PlacementLocation {
                        containers: path,
                        index,
                    },
                );
            },
            PlacementNode::Group { key, children } => {
                let kind = ContainerKind::Group;
                let identity = key.0.clone();
                if identity.is_empty() {
                    return Err(CompositionError::EmptyIdentity { kind: "group" });
                }
                if !groups.insert(ContainerKey {
                    kind,
                    key: identity.clone(),
                }) {
                    return Err(CompositionError::DuplicateGroup(identity));
                }
                if groups.len() > limits.max_groups {
                    return Err(CompositionError::TooManyGroups {
                        limit: limits.max_groups,
                    });
                }
                let next_depth = depth + 1;
                if next_depth > limits.max_depth.min(MAX_PLACEMENT_DEPTH) {
                    return Err(CompositionError::PlacementTooDeep {
                        limit: limits.max_depth.min(MAX_PLACEMENT_DEPTH),
                    });
                }
                let mut child_path = path;
                child_path.push(ContainerKey {
                    kind,
                    key: identity,
                });
                for (child_index, child) in children.iter().enumerate().rev() {
                    stack.push((child, child_path.clone(), child_index, next_depth));
                }
            },
            PlacementNode::Drawer { key, children } => {
                let kind = ContainerKind::Drawer;
                let identity = key.0.clone();
                if identity.is_empty() {
                    return Err(CompositionError::EmptyIdentity { kind: "drawer" });
                }
                if !groups.insert(ContainerKey {
                    kind,
                    key: identity.clone(),
                }) {
                    return Err(CompositionError::DuplicateGroup(identity));
                }
                if groups.len() > limits.max_groups {
                    return Err(CompositionError::TooManyGroups {
                        limit: limits.max_groups,
                    });
                }
                let next_depth = depth + 1;
                if next_depth > limits.max_depth.min(MAX_PLACEMENT_DEPTH) {
                    return Err(CompositionError::PlacementTooDeep {
                        limit: limits.max_depth.min(MAX_PLACEMENT_DEPTH),
                    });
                }
                let mut child_path = path;
                child_path.push(ContainerKey {
                    kind,
                    key: identity,
                });
                for (child_index, child) in children.iter().enumerate().rev() {
                    stack.push((child, child_path.clone(), child_index, next_depth));
                }
            },
        }
    }
    let mut hidden = BTreeSet::new();
    for key in &model.unplaced {
        if !components.contains_key(key) {
            return Err(CompositionError::UnknownComponent(key.clone()));
        }
        if !hidden.insert(key.clone()) {
            return Err(CompositionError::DuplicateUnplaced(key.clone()));
        }
        if placed.contains(key) {
            return Err(CompositionError::DuplicatePlacement(key.clone()));
        }
    }
    for key in components.keys() {
        if !placed.contains(key) && !hidden.contains(key) {
            return Err(CompositionError::ComponentNotAccountedFor(key.clone()));
        }
    }
    for component in model.components.iter() {
        validate_payload_depth(&component.config, MAX_PAYLOAD_DEPTH)?;
        validate_payload_depth(&component.binding, MAX_PAYLOAD_DEPTH)?;
    }
    let mut writer = BoundedWriter {
        written: 0,
        limit: limits.max_encoded_bytes,
        exceeded: false,
    };
    serde_json::to_writer(&mut writer, model).map_err(|error| {
        if writer.exceeded {
            CompositionError::EncodedTooLarge {
                limit: writer.limit,
            }
        } else {
            CompositionError::Serialization(error.to_string())
        }
    })?;
    for component in model.components.iter() {
        let admission = validator
            .validate_component(component, previous.get(&component.key).copied())
            .map_err(|reason| CompositionError::InvalidComponent {
                key: component.key.clone(),
                reason,
            })?;
        if admission == BindingAdmission::PreviouslyAuthoredUnavailable {
            if let Some(old) = previous.get(&component.key).copied() {
                if old.kind != component.kind || old.binding != component.binding {
                    return Err(CompositionError::InvalidComponent {
                        key: component.key.clone(),
                        reason: "unavailable admission must retain the accepted kind and binding"
                            .into(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_payload_depth(value: &Value, limit: usize) -> Result<(), CompositionError> {
    let mut stack = vec![(value, 0usize)];
    while let Some((value, depth)) = stack.pop() {
        if depth > limit {
            return Err(CompositionError::PayloadTooDeep { limit });
        }
        match value {
            Value::Array(items) => stack.extend(items.iter().map(|item| (item, depth + 1))),
            Value::Object(items) => stack.extend(items.values().map(|item| (item, depth + 1))),
            _ => {},
        }
    }
    Ok(())
}

struct BoundedWriter {
    written: usize,
    limit: usize,
    exceeded: bool,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.written) {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "encoded composition exceeds limit",
            ));
        }
        self.written += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn collect_groups(model: &Composition) -> BTreeSet<ContainerKey> {
    let mut result = BTreeSet::new();
    let mut pending: Vec<&PlacementNode> = model.placement.iter().collect();
    while let Some(node) = pending.pop() {
        match node {
            PlacementNode::Component { .. } => {},
            PlacementNode::Group { key, children } => {
                result.insert(container(ContainerKind::Group, &key.0));
                pending.extend(children);
            },
            PlacementNode::Drawer { key, children } => {
                result.insert(container(ContainerKind::Drawer, &key.0));
                pending.extend(children);
            },
        }
    }
    result
}

#[rustfmt::skip]
fn diff(old: &Composition, new: &Composition) -> Vec<CompositionEdit> {
    let before = component_map(old); let after = component_map(new);
    let old_at = locations(old); let new_at = locations(new); let mut edits = Vec::new();
    for (key, prior) in &before {
        match after.get(key) {
            None => edits.push(CompositionEdit::Remove(key.clone())),
            Some(next) => {
                if prior.kind != next.kind { edits.push(CompositionEdit::ReplaceKind { key: key.clone(), from: prior.kind.clone(), to: next.kind.clone() }); }
                if prior.binding != next.binding { edits.push(CompositionEdit::ChangeBinding { key: key.clone(), from: prior.binding.clone(), to: next.binding.clone() }); }
                if prior.config != next.config { edits.push(CompositionEdit::ChangeConfig { key: key.clone(), from: prior.config.clone(), to: next.config.clone() }); }
                match (old_at.get(key), new_at.get(key)) {
                    (Some(from), Some(to)) if from.containers != to.containers => edits.push(CompositionEdit::Regroup { key: key.clone(), from: from.containers.clone(), to: to.containers.clone() }),
                    (Some(from), Some(to)) if from != to => edits.push(CompositionEdit::Move { key: key.clone(), from: from.clone(), to: to.clone() }),
                    (Some(_), None) => edits.push(CompositionEdit::Hide(key.clone())),
                    (None, Some(to)) => edits.push(CompositionEdit::Show { key: key.clone(), to: to.clone() }),
                    _ => {},
                }
            },
        }
    }
    for (key, descriptor) in &after { if !before.contains_key(key) { edits.push(CompositionEdit::Create((**descriptor).clone())); } }
    let old_groups = collect_groups(old); let new_groups = collect_groups(new);
    for group in new_groups.difference(&old_groups) { edits.push(CompositionEdit::CreateGroup(group.clone())); }
    for group in old_groups.difference(&new_groups) { edits.push(CompositionEdit::RemoveGroup(group.clone())); }
    let old_children = child_orders(old);
    for (parent, order) in child_orders(new) { if old_children.get(&parent).is_some_and(|prior| prior != &order) { edits.push(CompositionEdit::ReorderChildren(parent)); } }
    edits
}

fn container(kind: ContainerKind, key: &str) -> ContainerKey {
    ContainerKey {
        kind,
        key: key.into(),
    }
}

#[rustfmt::skip]
fn locations(model: &Composition) -> BTreeMap<ComponentKey, PlacementLocation> {
    let root = container(ContainerKind::Root, ""); let mut found = BTreeMap::new();
    let mut stack: Vec<(&PlacementNode, Vec<ContainerKey>, usize)> = model.placement.iter().enumerate().rev().map(|(i,n)| (n, vec![root.clone()], i)).collect();
    while let Some((node, path, index)) = stack.pop() {
        match node {
            PlacementNode::Component { key } => { found.insert(key.clone(), PlacementLocation { containers: path, index }); },
            PlacementNode::Group { key, children } => push_children(&mut stack, children, path, container(ContainerKind::Group, &key.0)),
            PlacementNode::Drawer { key, children } => push_children(&mut stack, children, path, container(ContainerKind::Drawer, &key.0)),
        }
    }
    found
}

#[rustfmt::skip]
fn push_children<'a>(stack: &mut Vec<(&'a PlacementNode, Vec<ContainerKey>, usize)>, children: &'a [PlacementNode], mut path: Vec<ContainerKey>, parent: ContainerKey) {
    path.push(parent); for (index, child) in children.iter().enumerate().rev() { stack.push((child, path.clone(), index)); }
}

#[rustfmt::skip]
fn child_orders(model: &Composition) -> BTreeMap<ContainerKey, Vec<String>> {
    let mut orders = BTreeMap::new(); let mut stack = vec![(&model.placement[..], container(ContainerKind::Root, ""))];
    while let Some((nodes, parent)) = stack.pop() {
        orders.insert(parent, nodes.iter().map(node_key).collect());
        for node in nodes { match node {
            PlacementNode::Group { key, children } => stack.push((children, container(ContainerKind::Group, &key.0))),
            PlacementNode::Drawer { key, children } => stack.push((children, container(ContainerKind::Drawer, &key.0))),
            PlacementNode::Component { .. } => {},
        }}
    }
    orders
}

fn node_key(node: &PlacementNode) -> String {
    match node {
        PlacementNode::Component { key } => format!("c:{}", key.0),
        PlacementNode::Group { key, .. } => format!("g:{}", key.0),
        PlacementNode::Drawer { key, .. } => format!("d:{}", key.0),
    }
}

#[cfg(test)]
mod tests;
