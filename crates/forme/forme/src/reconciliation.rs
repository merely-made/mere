// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Subgraph binding reconciliation.
//!
//! When a linked subgraph's graph-truth membership changes, the tree must
//! detect the delta, propose reconciliation to the host, and apply the
//! chosen outcome. This module implements the delta computation and
//! outcome application described in
//! `subgraph_projection_binding_spec.md §7`.

use std::collections::HashSet;

use crate::MemberId;
use crate::subgraph::{
    SubgraphBinding, SubgraphId, SubgraphMemberDelta, SubgraphSpec, ReconciliationChoice,
    ReconciliationProposal,
};
use crate::member::Lifecycle;
use crate::nav::TreeIntent;
use crate::tree::GraphTree;

/// Compute the roster delta between a linked subgraph's expected members
/// (from graph truth) and the tree's current members for that subgraph.
///
/// `graph_truth_members` is the authoritative member set as derived from
/// the graph using the subgraph's edge projection spec.
pub fn compute_roster_delta<N: MemberId>(
    tree: &GraphTree<N>,
    subgraph_id: SubgraphId,
    graph_truth_members: &[N],
) -> SubgraphMemberDelta<N> {
    let truth_set: HashSet<&N> = graph_truth_members.iter().collect();
    let tree_members: Vec<N> = tree
        .subgraph_members(subgraph_id)
        .into_iter()
        .cloned()
        .collect();
    let tree_set: HashSet<&N> = tree_members.iter().collect();

    let added: Vec<N> = graph_truth_members
        .iter()
        .filter(|m| !tree_set.contains(m))
        .cloned()
        .collect();

    let removed: Vec<N> = tree_members
        .iter()
        .filter(|m| !truth_set.contains(m))
        .cloned()
        .collect();

    SubgraphMemberDelta {
        added,
        removed,
        rebased_seeds: Vec::new(),
    }
}

/// Build a reconciliation proposal for a linked subgraph that has drifted
/// from graph truth.
///
/// Returns `None` if the delta is empty (no reconciliation needed).
pub fn propose_reconciliation<N: MemberId>(
    tree: &GraphTree<N>,
    subgraph_id: SubgraphId,
    graph_truth_members: &[N],
    reason: impl Into<String>,
) -> Option<ReconciliationProposal<N>> {
    let delta = compute_roster_delta(tree, subgraph_id, graph_truth_members);
    if delta.is_empty() {
        return None;
    }
    Some(ReconciliationProposal {
        subgraph_id,
        delta,
        reason: reason.into(),
    })
}

/// Apply a reconciliation choice to the tree.
///
/// Returns intents for the host to act on (activation requests for new
/// members, dismissal requests for removed members, etc.).
pub fn apply_reconciliation<N: MemberId>(
    tree: &mut GraphTree<N>,
    proposal: &ReconciliationProposal<N>,
    choice: ReconciliationChoice,
) -> Vec<TreeIntent<N>> {
    let gid = proposal.subgraph_id;
    let mut intents = Vec::new();

    match choice {
        ReconciliationChoice::ApplyKeepLinked => {
            // Add new members to the tree with subgraph membership.
            for member in &proposal.delta.added {
                if tree.get(member).is_none() {
                    tree.apply(crate::nav::NavAction::Attach {
                        member: member.clone(),
                        provenance: crate::member::Provenance::Derived {
                            connection: None,
                            derivation: format!("subgraph-reconciliation:{}", gid),
                        },
                    });
                }
                // Tag with subgraph membership.
                if let Some(entry) = tree.get_mut(member) {
                    if !entry.subgraph_membership.contains(&gid) {
                        entry.subgraph_membership.push(gid);
                    }
                }
                intents.push(TreeIntent::MemberAttached(member.clone()));
            }

            // Remove members no longer in graph truth.
            for member in &proposal.delta.removed {
                if let Some(entry) = tree.get_mut(member) {
                    entry.subgraph_membership.retain(|id| *id != gid);
                }
                // If the member has no remaining subgraph memberships and is Cold,
                // detach it entirely.
                let should_detach = tree
                    .get(member)
                    .map(|e| e.subgraph_membership.is_empty() && e.lifecycle == Lifecycle::Cold)
                    .unwrap_or(false);
                if should_detach {
                    tree.apply(crate::nav::NavAction::Detach {
                        member: member.clone(),
                        recursive: false,
                    });
                    intents.push(TreeIntent::MemberDetached(member.clone()));
                }
            }

            intents.push(TreeIntent::ReconciliationNeeded {
                subgraph: gid,
                reason: format!("applied: {}", proposal.reason),
            });
        },

        ReconciliationChoice::KeepAsUnlinkedSession => {
            // Convert binding to UnlinkedSession.
            if let Some(subgraph) = tree.subgraphs_mut().iter_mut().find(|g| g.id == gid) {
                subgraph.binding = SubgraphBinding::UnlinkedSession;
            }
        },

        ReconciliationChoice::SaveAsNewFork { ref reason } => {
            // Branch: preserve current roster, change binding to Branched.
            if let Some(subgraph) = tree.subgraphs_mut().iter_mut().find(|g| g.id == gid) {
                let parent_spec = match &subgraph.binding {
                    SubgraphBinding::Linked { spec } => spec.clone(),
                    _ => SubgraphSpec {
                        kind: subgraph
                            .kind
                            .clone()
                            .unwrap_or(crate::subgraph::SubgraphKind::Session),
                        anchors: Vec::new(),
                        primary_anchor: None,
                        selectors: Vec::new(),
                    },
                };
                subgraph.binding = SubgraphBinding::Branched {
                    parent_spec,
                    reason: reason.clone(),
                };
            }
        },

        ReconciliationChoice::Cancel => {
            // No-op: discard the proposal, tree unchanged.
        },
    }

    intents
}

/// Detect whether a manual operation (user-initiated attach/detach) on a
/// member that belongs to a linked subgraph should trigger a fork.
///
/// Returns the subgraph ID and a fork reason if the operation diverges
/// from the linked spec.
pub fn detect_fork_on_manual_override<N: MemberId>(
    tree: &GraphTree<N>,
    member: &N,
    operation: &str,
) -> Option<(SubgraphId, String)> {
    let entry = tree.get(member)?;
    for &gid in &entry.subgraph_membership {
        let subgraph = tree.subgraphs().iter().find(|g| g.id == gid)?;
        if matches!(subgraph.binding, SubgraphBinding::Linked { .. }) {
            return Some((
                gid,
                format!(
                    "manual {} on member {:?} diverges from linked subgraph {}",
                    operation, member, gid
                ),
            ));
        }
    }
    None
}

/// Transition a linked subgraph to Branched state.
///
/// Called when `detect_fork_on_manual_override` fires and the host
/// decides (or auto-policy decides) that the override should fork.
pub fn apply_fork<N: MemberId>(tree: &mut GraphTree<N>, subgraph_id: SubgraphId, reason: String) {
    if let Some(subgraph) = tree
        .subgraphs_mut()
        .iter_mut()
        .find(|g| g.id == subgraph_id)
    {
        let parent_spec = match &subgraph.binding {
            SubgraphBinding::Linked { spec } => spec.clone(),
            _ => return, // Not linked — nothing to fork.
        };
        subgraph.binding = SubgraphBinding::Branched {
            parent_spec,
            reason,
        };
    }
}

// ---------------------------------------------------------------------------
// Containment lens derivation
// ---------------------------------------------------------------------------

/// Input for containment lens derivation: origin/domain groupings
/// supplied by the host from ContainmentRelation edges.
///
/// The forme crate doesn't know about edge types, so the host
/// extracts containment relationships and passes them as group lists.
#[derive(Clone, Debug)]
pub struct ContainmentGroup<N: MemberId> {
    /// Label for this containment group (e.g. domain name, folder path).
    pub label: String,
    /// Members that belong to this group.
    pub members: Vec<N>,
}

/// Derive a containment-oriented topology for tree members.
///
/// Given containment groups, returns a list of (parent, children) pairs
/// that the host can apply as topology via `NavAction::Reparent`.
///
/// Members not in any group are collected under a synthetic "ungrouped"
/// parent (the first member of the ungrouped set).
pub fn derive_containment_topology<N: MemberId>(
    tree: &GraphTree<N>,
    groups: &[ContainmentGroup<N>],
) -> Vec<(N, Vec<N>)> {
    let all_members: HashSet<N> = tree.members().map(|(k, _)| k.clone()).collect();
    let mut grouped: HashSet<N> = HashSet::new();
    let mut result: Vec<(N, Vec<N>)> = Vec::new();

    for group in groups {
        // The first member in the group becomes the parent.
        let valid_members: Vec<N> = group
            .members
            .iter()
            .filter(|m| all_members.contains(m))
            .cloned()
            .collect();
        if valid_members.len() < 2 {
            // Single-member or empty groups don't need hierarchy.
            for m in &valid_members {
                grouped.insert(m.clone());
            }
            continue;
        }
        let parent = valid_members[0].clone();
        let children: Vec<N> = valid_members[1..].to_vec();
        for m in &valid_members {
            grouped.insert(m.clone());
        }
        result.push((parent, children));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subgraph::{SubgraphKind, SubgraphRef};
    use crate::member::Provenance;
    use crate::nav::NavAction;

    fn make_tree_with_linked_subgraph(members: &[u64], subgraph_members: &[u64]) -> GraphTree<u64> {
        let mut tree = GraphTree::new(
            crate::layout::LayoutMode::TreeStyleTabs,
            crate::lens::ProjectionLens::Traversal,
        );
        for &m in members {
            tree.apply(NavAction::Attach {
                member: m,
                provenance: Provenance::Manual {
                    source: None,
                    context: None,
                },
            });
        }

        let spec = SubgraphSpec {
            kind: SubgraphKind::Session,
            anchors: Vec::new(),
            primary_anchor: None,
            selectors: Vec::new(),
        };
        let subgraph = SubgraphRef {
            id: 0,
            anchors: Vec::new(),
            primary_anchor: None,
            binding: SubgraphBinding::Linked { spec },
            kind: Some(SubgraphKind::Session),
        };
        tree.add_subgraph(subgraph);

        for &m in subgraph_members {
            if let Some(entry) = tree.get_mut(&m) {
                entry.subgraph_membership.push(0);
            }
        }
        tree
    }

    #[test]
    fn empty_delta_when_synchronized() {
        let tree = make_tree_with_linked_subgraph(&[1, 2, 3], &[1, 2, 3]);
        let delta = compute_roster_delta(&tree, 0, &[1, 2, 3]);
        assert!(delta.is_empty());
    }

    #[test]
    fn delta_detects_additions() {
        let tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let delta = compute_roster_delta(&tree, 0, &[1, 2, 3]);
        assert_eq!(delta.added, vec![3]);
        assert!(delta.removed.is_empty());
    }

    #[test]
    fn delta_detects_removals() {
        let tree = make_tree_with_linked_subgraph(&[1, 2, 3], &[1, 2, 3]);
        let delta = compute_roster_delta(&tree, 0, &[1, 2]);
        assert!(delta.added.is_empty());
        assert_eq!(delta.removed, vec![3]);
    }

    #[test]
    fn delta_detects_both() {
        let tree = make_tree_with_linked_subgraph(&[1, 2, 3], &[1, 2, 3]);
        let delta = compute_roster_delta(&tree, 0, &[1, 2, 4]);
        assert_eq!(delta.added, vec![4]);
        assert_eq!(delta.removed, vec![3]);
    }

    #[test]
    fn propose_returns_none_when_synchronized() {
        let tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        assert!(propose_reconciliation(&tree, 0, &[1, 2], "test").is_none());
    }

    #[test]
    fn propose_returns_some_when_diverged() {
        let tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let proposal = propose_reconciliation(&tree, 0, &[1, 2, 3], "graph change");
        assert!(proposal.is_some());
        assert_eq!(proposal.unwrap().delta.added, vec![3]);
    }

    #[test]
    fn apply_keep_linked_adds_members() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let proposal = propose_reconciliation(&tree, 0, &[1, 2, 3], "test").unwrap();
        let intents =
            apply_reconciliation(&mut tree, &proposal, ReconciliationChoice::ApplyKeepLinked);

        // Member 3 should now exist in the tree.
        assert!(tree.get(&3).is_some());
        // And should be tagged with subgraph 0.
        assert!(tree.get(&3).unwrap().subgraph_membership.contains(&0));
        assert!(!intents.is_empty());
    }

    #[test]
    fn apply_keep_linked_removes_cold_unaffiliated_members() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2, 3], &[1, 2, 3]);
        // Make member 3 Cold so it can be auto-detached.
        tree.apply(NavAction::SetLifecycle(3, Lifecycle::Cold));
        let proposal = propose_reconciliation(&tree, 0, &[1, 2], "test").unwrap();
        apply_reconciliation(&mut tree, &proposal, ReconciliationChoice::ApplyKeepLinked);

        // Member 3 should be detached (no remaining subgraph membership + Cold).
        assert!(tree.get(&3).is_none());
    }

    #[test]
    fn apply_keep_unlinked_converts_binding() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let proposal = propose_reconciliation(&tree, 0, &[1, 2, 3], "test").unwrap();
        apply_reconciliation(
            &mut tree,
            &proposal,
            ReconciliationChoice::KeepAsUnlinkedSession,
        );

        let binding = &tree.subgraphs()[0].binding;
        assert!(matches!(binding, SubgraphBinding::UnlinkedSession));
    }

    #[test]
    fn apply_fork_preserves_parent_spec() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let proposal = propose_reconciliation(&tree, 0, &[1, 2, 3], "test").unwrap();
        apply_reconciliation(
            &mut tree,
            &proposal,
            ReconciliationChoice::SaveAsNewFork {
                reason: "user override".to_string(),
            },
        );

        let binding = &tree.subgraphs()[0].binding;
        match binding {
            SubgraphBinding::Branched {
                parent_spec,
                reason,
            } => {
                assert_eq!(parent_spec.kind, SubgraphKind::Session);
                assert_eq!(reason, "user override");
            },
            _ => panic!("expected Branched binding"),
        }
    }

    #[test]
    fn cancel_leaves_tree_unchanged() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let proposal = propose_reconciliation(&tree, 0, &[1, 2, 3], "test").unwrap();
        apply_reconciliation(&mut tree, &proposal, ReconciliationChoice::Cancel);

        // No member 3 should exist.
        assert!(tree.get(&3).is_none());
        // Binding should still be Linked.
        assert!(matches!(
            tree.subgraphs()[0].binding,
            SubgraphBinding::Linked { .. }
        ));
    }

    #[test]
    fn detect_fork_finds_linked_subgraph() {
        let tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        let result = detect_fork_on_manual_override(&tree, &1, "dismiss");
        assert!(result.is_some());
        let (gid, _reason) = result.unwrap();
        assert_eq!(gid, 0);
    }

    #[test]
    fn detect_fork_ignores_unlinked() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        // Convert to unlinked.
        tree.subgraphs_mut()[0].binding = SubgraphBinding::UnlinkedSession;
        let result = detect_fork_on_manual_override(&tree, &1, "dismiss");
        assert!(result.is_none());
    }

    #[test]
    fn apply_fork_transitions_to_forked() {
        let mut tree = make_tree_with_linked_subgraph(&[1, 2], &[1, 2]);
        apply_fork(&mut tree, 0, "manual override".to_string());
        assert!(matches!(
            tree.subgraphs()[0].binding,
            SubgraphBinding::Branched { .. }
        ));
    }

    #[test]
    fn containment_topology_groups_members() {
        let mut tree = GraphTree::new(
            crate::layout::LayoutMode::TreeStyleTabs,
            crate::lens::ProjectionLens::Traversal,
        );
        for i in 1..=6u64 {
            tree.apply(NavAction::Attach {
                member: i,
                provenance: Provenance::Manual {
                    source: None,
                    context: None,
                },
            });
        }

        let groups = vec![
            ContainmentGroup {
                label: "example.com".to_string(),
                members: vec![1, 2, 3],
            },
            ContainmentGroup {
                label: "other.org".to_string(),
                members: vec![4, 5],
            },
        ];

        let topology = derive_containment_topology(&tree, &groups);
        assert_eq!(topology.len(), 2);
        assert_eq!(topology[0].0, 1); // parent of first group
        assert_eq!(topology[0].1, vec![2, 3]); // children
        assert_eq!(topology[1].0, 4); // parent of second group
        assert_eq!(topology[1].1, vec![5]); // children
    }
}
