// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Publication and queue admission share contributor identity. This is not an
//! OS event-generation token or an atomic model transaction.

use crate::{A11yOwner, A11yTarget, NodeId, ProducerChildToken, ProducerRegistry, WindowDom};
use layout_dom_api::LayoutDom;
use sprigging::{LeafLifetime, LeafRegistry};
use std::sync::Weak;

#[derive(Clone, Debug)]
pub(crate) struct A11yProvenance {
    owner: A11yOwner,
    target: A11yTarget,
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

impl A11yProvenance {
    pub fn capture(
        owner: &A11yOwner,
        target: &A11yTarget,
        dom: &WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> Self {
        let slot = match target {
            A11yTarget::Produced(action) => Some(action.slot),
            A11yTarget::Node(node) => dom
                .contains(*node)
                .then(|| crate::owned_layout::custom_leaf_key(dom, *node))
                .flatten(),
        };
        Self {
            owner: owner.clone(),
            target: target.clone(),
            document: dom.dom().document(),
            mount: dom.document(),
            producer: slot.and_then(|key| producers.action_lifetime(key)),
            child: match target {
                A11yTarget::Produced(action) => producers.child_lifetime(action.slot, action.key),
                _ => None,
            },
            leaf: match target {
                A11yTarget::Node(_) => slot.map(|key| (key, leaves.lifetime(&key))),
                _ => None,
            },
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
            && (!matches!(target, A11yTarget::Produced(_)) || self.producer.is_some())
            && *self == Self::capture(owner, target, dom, leaves, producers)
    }
}
