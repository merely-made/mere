// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl A11yRequest {
    /// A raw request naming the current context, without publication provenance.
    pub fn new(action: A11yAction, target: A11yTarget) -> Self {
        Self {
            action,
            target,
            provenance: None,
        }
    }

    /// Bind a publication's document/window subtree and contributors. This retains no
    /// document, contributor or host. The owner is the publishing host lifetime;
    /// a raw numeric OS ID still does not carry an event generation by itself.
    pub fn bind_publication(
        mut self,
        owner: &A11yOwner,
        dom: &WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> Self {
        if self.provenance.is_none() {
            self.provenance = Some(a11y_provenance::A11yProvenance::capture(
                owner,
                &self.target,
                dom,
                leaves,
                producers,
            ));
        }
        self
    }

    /// Whether two bound observations name the same target life. Ordinary
    /// value/layout updates keep it; an unbound request makes no such claim.
    pub fn same_publication_target(&self, other: &Self) -> bool {
        self.target == other.target
            && self
                .provenance
                .as_ref()
                .zip(other.provenance.as_ref())
                .is_some_and(|(a, b)| a == b)
    }

    pub(crate) fn publication_is_current(
        &self,
        owner: &A11yOwner,
        dom: &WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> bool {
        self.provenance
            .as_ref()
            .is_none_or(|binding| binding.is_current(owner, &self.target, dom, leaves, producers))
    }

    /// A request on a DOM node.
    pub fn node(action: A11yAction, node: NodeId) -> Self {
        Self::new(action, A11yTarget::Node(node))
    }

    /// A request on a drawn node's action button.
    pub fn produced(action: A11yAction, produced: ProducedAction) -> Self {
        Self::new(action, A11yTarget::Produced(produced))
    }
}
