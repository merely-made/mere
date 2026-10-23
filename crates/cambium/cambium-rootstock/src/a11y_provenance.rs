// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Publication and queue admission share contributor identity. This is not an
//! OS event-generation token or an atomic model transaction.

use crate::{A11yOwner, A11yTarget, NodeId, ProducerChildToken, ProducerRegistry, WindowDom};
use layout_dom_api::LayoutDom;
use sprigging::{LeafLifetime, LeafRegistry};
use std::sync::Weak;

/// Authorship of every published node, including non-action producer children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NodeTarget {
    Node(NodeId),
    Produced {
        slot: u64,
        key: u64,
        action: Option<String>,
    },
}
impl From<&A11yTarget> for NodeTarget {
    fn from(target: &A11yTarget) -> Self {
        match target {
            A11yTarget::Node(node) => Self::Node(*node),
            A11yTarget::Produced(action) => Self::Produced {
                slot: action.slot,
                key: action.key,
                action: Some(action.id.clone()),
            },
        }
    }
}

/// A weak authored identity, retaining no host, document, leaf or producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticNodeIdentity(pub(crate) A11yProvenance);

#[derive(Clone, Debug)]
pub(crate) struct A11yProvenance {
    owner: A11yOwner,
    target: NodeTarget,
    document: NodeId,
    mount: NodeId,
    producer: Option<Weak<()>>,
    child: Option<ProducerChildToken>,
    leaf: Option<(u64, Option<LeafLifetime>)>,
}

impl PartialEq for A11yProvenance {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.target == other.target
            && self.document == other.document
            && self.mount == other.mount
            && match (&self.producer, &other.producer) {
                (Some(a), Some(b)) => a.ptr_eq(b),
                (None, None) => true,
                _ => false,
            }
            && self.child == other.child
            && self.leaf == other.leaf
    }
}

impl Eq for A11yProvenance {}

impl A11yProvenance {
    pub(crate) fn child(&self) -> Option<&ProducerChildToken> {
        self.child.as_ref()
    }

    pub(crate) fn contributors_live(&self) -> bool {
        self.child.as_ref().is_none_or(ProducerChildToken::is_live)
            && self
                .producer
                .as_ref()
                .is_none_or(|producer| producer.upgrade().is_some())
    }
    pub(crate) fn target_matches(&self, target: &A11yTarget) -> bool {
        self.target == NodeTarget::from(target)
    }

    pub fn capture(
        owner: &A11yOwner,
        target: &A11yTarget,
        dom: &WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> Self {
        Self::capture_node(owner, NodeTarget::from(target), dom, leaves, producers)
    }

    pub(crate) fn capture_node(
        owner: &A11yOwner,
        target: NodeTarget,
        dom: &WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> Self {
        let slot = match &target {
            NodeTarget::Produced { slot, .. } => Some(*slot),
            NodeTarget::Node(node) => dom
                .contains(*node)
                .then(|| crate::owned_layout::custom_leaf_key(dom, *node))
                .flatten(),
        };
        Self {
            owner: owner.clone(),
            document: dom.dom().document(),
            mount: dom.document(),
            producer: slot.and_then(|key| producers.action_lifetime(key)),
            child: match &target {
                NodeTarget::Produced { slot, key, .. } => producers.child_lifetime(*slot, *key),
                _ => None,
            },
            leaf: match &target {
                NodeTarget::Node(_) => slot.map(|key| (key, leaves.lifetime(&key))),
                _ => None,
            },
            target,
        }
    }

    pub fn is_current(
        &self,
        owner: &A11yOwner,
        target: &A11yTarget,
        dom: &WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> bool {
        // An absent registration cannot lend authority to a later producer.
        self.owner.is_live()
            && self.contributors_live()
            && (!matches!(target, A11yTarget::Produced(_)) || self.producer.is_some())
            && *self == Self::capture(owner, target, dom, leaves, producers)
    }
}
