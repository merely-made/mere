// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Projection into mere's AccessKit lane.
//!
//! The frozen realization already answers what a reader needs; this hands the
//! same answer to the platform assistive stack instead of to markup. Roles come
//! from AccessKit's own vocabulary rather than Graphics ARIA's, because that is
//! what the OS layer speaks: the document is a `Document`, instances and
//! relations are `ListItem`s under `List` groups, and every node carries a
//! label, since an unlabelled node is exactly the failure this target exists to
//! catch.
//!
//! This was `FrozenScene::to_ux_tree` behind `graphshell-client`'s `accesskit`
//! feature. It lives here because it builds [`UxTree`], which is Mere's, and
//! `graphshell-client` moved to the Graphshell repository, which depends on
//! nothing in Mere (cycle breaks plan, D9). Import [`FrozenUxTree`] to call it.

use accesskit::{Node, NodeId, Role};
use graphshell_client::frozen::{FrozenRole, FrozenScene};

use crate::{UxTree, node_id_for_path};

/// The AccessKit tree of a frozen scene, as an extension of
/// [`FrozenScene`], which lives in `graphshell-client`.
pub trait FrozenUxTree {
    /// Build the AccessKit tree a screen reader traverses.
    ///
    /// `path` namespaces the ids so several projections can be stitched
    /// under one application root without colliding. Node order is
    /// descendants-then-root, matching [`UxTree`]'s stated contract so a
    /// consumer can zip straight into a `TreeUpdate`.
    fn to_ux_tree(&self, path: &str) -> UxTree;
}

impl FrozenUxTree for FrozenScene {
    fn to_ux_tree(&self, path: &str) -> UxTree {
        let mut nodes: Vec<(NodeId, Node)> = Vec::new();

        let mut instance_ids = Vec::new();
        for instance in &self.instances {
            let id = node_id_for_path(&format!(
                "{path}/instance/{}/{}/{}",
                instance.instance.0, instance.source.adapter, instance.source.id
            ));
            let mut node = Node::new(match instance.role {
                FrozenRole::Symbol => Role::Image,
                FrozenRole::Object => Role::ListItem,
                FrozenRole::LiveContent => Role::ListItem,
            });
            node.set_label(instance.name.clone());
            nodes.push((id, node));
            instance_ids.push(id);
        }
        let instances_id = node_id_for_path(&format!("{path}/instances"));
        let mut instances_group = Node::new(Role::List);
        instances_group.set_label(format!("{} items", self.instances.len()));
        instances_group.set_children(instance_ids);
        nodes.push((instances_id, instances_group));

        let mut fold_ids = Vec::new();
        for fold in &self.folds {
            let mut member_ids = Vec::new();
            for member in &fold.members {
                let id = node_id_for_path(&format!(
                    "{path}/fold/{}/instance/{}/{}/{}",
                    fold.fold, member.instance.0, member.source.adapter, member.source.id
                ));
                let mut node = Node::new(match member.role {
                    FrozenRole::Symbol => Role::Image,
                    FrozenRole::Object | FrozenRole::LiveContent => Role::ListItem,
                });
                node.set_label(member.name.clone());
                nodes.push((id, node));
                member_ids.push(id);
            }
            let id = node_id_for_path(&format!("{path}/fold/{}", fold.fold));
            let mut group = Node::new(Role::Group);
            group.set_label(fold.heading());
            group.set_children(member_ids);
            nodes.push((id, group));
            fold_ids.push(id);
        }

        let mut relation_ids = Vec::new();
        for (index, relation) in self.relations.iter().enumerate() {
            let id = node_id_for_path(&format!("{path}/relation/{index}"));
            let mut node = Node::new(Role::ListItem);
            let kind = relation
                .kind
                .clone()
                .unwrap_or_else(|| "related to".to_owned());
            node.set_label(format!("{} {} {}", relation.from, kind, relation.to));
            nodes.push((id, node));
            relation_ids.push(id);
        }
        let relations_id = node_id_for_path(&format!("{path}/relations"));
        let mut relations_group = Node::new(Role::List);
        relations_group.set_label(format!("{} relationships", self.relations.len()));
        relations_group.set_children(relation_ids);
        nodes.push((relations_id, relations_group));

        let mut unmet_ids = Vec::new();
        for held in &self.unmet_holds {
            let id = node_id_for_path(&format!(
                "{path}/unmet/{}/{}",
                held.source.adapter, held.source.id
            ));
            let mut node = Node::new(Role::ListItem);
            node.set_label(format!(
                "{} could not be placed at ({}, {})",
                held.source.id, held.at.x, held.at.y
            ));
            nodes.push((id, node));
            unmet_ids.push(id);
        }
        let mut children = vec![instances_id];
        if !fold_ids.is_empty() {
            let folds_id = node_id_for_path(&format!("{path}/folds"));
            let mut folds_group = Node::new(Role::List);
            folds_group.set_label(format!("{} folded groups", fold_ids.len()));
            folds_group.set_children(fold_ids);
            nodes.push((folds_id, folds_group));
            children.push(folds_id);
        }
        children.push(relations_id);
        if !unmet_ids.is_empty() {
            let unmet_id = node_id_for_path(&format!("{path}/unmet"));
            let mut unmet_group = Node::new(Role::List);
            unmet_group.set_label("Placements that could not be honored".to_owned());
            unmet_group.set_children(unmet_ids);
            nodes.push((unmet_id, unmet_group));
            children.push(unmet_id);
        }

        let root_id = node_id_for_path(path);
        let mut root = Node::new(Role::Document);
        root.set_label(self.name.clone());
        root.set_description(self.summary.clone());
        root.set_children(children);
        nodes.push((root_id, root));

        UxTree {
            root: root_id,
            nodes,
        }
    }
}
