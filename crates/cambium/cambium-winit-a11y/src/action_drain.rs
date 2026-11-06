// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Action service reads the last published bindings, never a fresh projection.

use super::*;
use cambium_rootstock::A11yOwner;
use genet_winit_host::A11yActionRequest;

impl A11yHost {
    pub(super) fn drain_native_actions(&mut self, owner: &A11yOwner) -> Vec<A11yRequest> {
        let actions = self.bridge.drain_actions();
        if self.window.is_none() && owner.is_live() {
            return Vec::new();
        }
        self.resolve_native_actions(owner, actions)
    }

    fn resolve_native_actions(
        &mut self,
        owner: &A11yOwner,
        actions: impl IntoIterator<Item = A11yActionRequest>,
    ) -> Vec<A11yRequest> {
        if !owner.is_live() {
            self.requests.clear();
            self.namespace.clear();
            return Vec::new();
        }
        actions
            .into_iter()
            .filter_map(|request| {
                self.map_parts(
                    request.action,
                    request.data,
                    request.target_tree,
                    request.target_node,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn action_drain_retains_the_old_binding_and_owner_revocation_clears_routes() {
        let marker = Rc::new(Cell::new(true));
        let owner = A11yOwner::new(&marker);
        let dom = genet_scripted_dom::ScriptedDom::new();
        let request = A11yRequest::node(A11yAction::Click, dom.document());
        let mut host = A11yHost::new(|| {});
        host.requests.insert(A11yNodeId(71), request.clone());
        let raw = || A11yActionRequest {
            action: Action::Click,
            data: None,
            target_tree: accesskit::TreeId::ROOT,
            target_node: A11yNodeId(71),
        };
        assert_eq!(
            host.resolve_native_actions(&owner, [raw()]),
            [request.clone()]
        );
        assert_eq!(host.requests.get(&A11yNodeId(71)), Some(&request));
        marker.set(false);
        assert!(host.resolve_native_actions(&owner, [raw()]).is_empty());
        assert!(host.requests.is_empty());
        let replacement = Rc::new(Cell::new(true));
        assert!(
            host.resolve_native_actions(&A11yOwner::new(&replacement), [raw()])
                .is_empty(),
            "an already consumed gesture cannot acquire replacement authority"
        );
    }
}
