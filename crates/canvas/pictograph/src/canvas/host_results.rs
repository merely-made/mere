// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Captured layout-result ownership, checked at installation.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct LayoutResultSource {
    owner: Arc<()>,
    view_epoch: u64,
    structure: kernel::graph::GraphInputKey,
}
impl PartialEq for LayoutResultSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner)
            && self.view_epoch == other.view_epoch
            && self.structure == other.structure
    }
}
impl LayoutResultSource {
    pub(super) fn capture(canvas: &Canvas) -> Self {
        Self {
            owner: canvas.result_owner.clone(),
            view_epoch: canvas.graph_view_epoch,
            structure: canvas.graph.structural_input_key(),
        }
    }
    pub(super) fn is_current(&self, canvas: &Canvas) -> bool {
        Arc::ptr_eq(&self.owner, &canvas.result_owner)
            && self.view_epoch == canvas.graph_view_epoch
            && self.structure == canvas.graph.structural_input_key()
    }
}
