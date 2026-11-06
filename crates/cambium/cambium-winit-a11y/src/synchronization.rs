// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tree publication; queued requests retain the preceding namespace.

use super::*;

impl A11yHost {
    /// Project the current layout into an AccessKit tree (with each leaf's own
    /// semantics), install it on the first call — revealing `window`, which must
    /// have been created hidden so the adapter attaches first — and update it
    /// after. Returns the screen reader's Click and Focus requests, in request
    /// order and still typed, for the caller to route each through the matching
    /// path (activation for `Click`, focus for `Focus`).
    ///
    /// `focus` is the app's currently-focused DOM node's opaque id (from
    /// `LayoutDom::opaque_id`), used as the tree's focus when it is really in the
    /// tree, so a stale id never points the reader at nothing.
    pub(super) fn sync_inner(
        &mut self,
        owner: &cambium_rootstock::A11yOwner,
        dom: &cambium_rootstock::WindowDom<'_>,
        layout: &cambium_rootstock::OwnedLayout,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> A11ySync {
        if !owner.is_live() {
            self.bridge.drain_actions();
            self.requests.clear();
            self.namespace.clear();
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::skipped(
                    self.adapter_state(),
                    A11yPublicationSkip::OwnerEnded,
                ),
            };
        }
        let Some(window) = self.window.clone() else {
            // No window yet: nothing to install against, and no reader to ask.
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::skipped(
                    A11yAdapterState::Detached,
                    A11yPublicationSkip::NoWindow,
                ),
            };
        };
        // Resolve queued events through the publication they could have seen,
        // before reading contributors or replacing the published maps.
        let requests = self.drain_native_actions(owner);
        let projection =
            cambium_rootstock::project_semantics(dom, layout, leaves, producers, focus);
        let contribution_issues = projection.producer_issues.len();
        let bindings = projection.publication_bindings(owner, dom, leaves, producers);
        let mut tree = projection.tree;
        scale_tree_to_window(&mut tree, dom, layout_scale);
        let publication = bindings
            .map_err(|issue| {
                (
                    A11yPublicationRefusal::InvalidBindings,
                    format!("{issue:?}"),
                )
            })
            .and_then(|bindings| {
                self.namespace
                    .prepare(tree, bindings)
                    .map_err(|issue| (A11yPublicationRefusal::InvalidTree, format!("{issue:?}")))
            });
        let publication::Prepared {
            tree,
            namespace,
            requests: next,
        } = match publication {
            Ok(candidate) => candidate,
            Err((reason, issue)) => {
                self.requests.clear();
                self.namespace.clear();
                eprintln!("[cambium-winit] accessibility publication refused: {issue}");
                // Match install failure: a rejected accessibility tree must not
                // leave the application hidden. A later sync may retry.
                window.set_visible(true);
                return A11ySync {
                    requests,
                    publication: A11yPublication::refused(self.adapter_state(), reason),
                };
            },
        };
        let node_count = tree.nodes.len();
        let mut outcome = A11yPublicationOutcome::Submitted {
            nodes: node_count,
            contribution_issues,
        };
        if !self.installed {
            match self.bridge.install(&window, tree) {
                Ok(()) => {
                    self.installed = true;
                    self.requests = next;
                    self.namespace = namespace;
                    eprintln!(
                        "[cambium-winit] accessibility {:?}, {node_count} nodes projected",
                        self.bridge.status()
                    );
                },
                Err(e) => {
                    self.requests.clear();
                    self.namespace.clear();
                    outcome = A11yPublicationOutcome::Refused {
                        reason: A11yPublicationRefusal::AdapterInstall,
                    };
                    eprintln!("[cambium-winit] accessibility install failed: {e}");
                },
            }
            // Keep the app usable on adapter failure; retry installation on the
            // next sync rather than claiming that a failed install succeeded.
            window.set_visible(true);
        } else {
            self.bridge.update(tree);
            self.requests = next;
            self.namespace = namespace;
        }
        A11ySync {
            requests,
            publication: A11yPublication {
                adapter: self.adapter_state(),
                outcome,
            },
        }
    }
}
