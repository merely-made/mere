// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A hidden receiving window needs its adapter before its first real picture.

use super::*;
use accesskit::{Node, Role, TreeId, TreeInfo};

fn window_root() -> TreeUpdate {
    // NativeIds allocates from one, process-wide. Zero belongs only to the
    // bootstrap window and never names a DOM, leaf or producer action target.
    let root = A11yNodeId(0);
    TreeUpdate {
        nodes: vec![(root, Node::new(Role::Window))],
        tree: Some(TreeInfo::new(root)),
        tree_id: TreeId::ROOT,
        focus: root,
    }
}

impl A11yHost {
    pub(super) fn bootstrap_native_window(&mut self) {
        // Later pending frames retain the already accepted AT tree and routes.
        if self.installed {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        self.requests.clear();
        self.namespace.clear();
        match self.bridge.install(&window, window_root()) {
            Ok(()) => self.installed = true,
            Err(error) => {
                eprintln!("[cambium-winit] accessibility window bootstrap failed: {error}");
                // Match normal publication's existing failure fallback: an
                // unavailable adapter must not keep the application hidden.
                // Successful installation still precedes revealing the HWND.
            },
        }
        window.set_visible(true);
        // The host records PendingPresentation, never Submitted, for bootstrap.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::LayoutDom;

    struct Ignore;
    impl accesskit_consumer::TreeChangeHandler for Ignore {
        fn node_added(&mut self, _: &accesskit_consumer::NodeRef<'_>) {}
        fn node_updated(
            &mut self,
            _: &accesskit_consumer::NodeRef<'_>,
            _: &accesskit_consumer::NodeRef<'_>,
        ) {
        }
        fn focus_moved(
            &mut self,
            _: Option<&accesskit_consumer::NodeRef<'_>>,
            _: Option<&accesskit_consumer::NodeRef<'_>>,
        ) {
        }
        fn node_removed(&mut self, _: &accesskit_consumer::NodeRef<'_>) {}
    }

    #[test]
    fn bootstrap_is_a_valid_root_only_tree_without_action_routes() {
        let tree = window_root();
        assert_eq!(tree.tree_id, TreeId::ROOT);
        assert_eq!(tree.nodes.len(), 1);
        let (id, node) = &tree.nodes[0];
        assert_eq!(*id, A11yNodeId(0));
        assert_eq!(tree.focus, *id);
        assert_eq!(tree.tree.as_ref().unwrap().root, *id);
        assert_eq!(node.role(), Role::Window);
        assert!(node.children().is_empty());
        let mut consumer = accesskit_consumer::Tree::new(tree, true);
        let accepted = A11yNodeId(1);
        let mut node = Node::new(Role::Window);
        node.set_label("accepted picture");
        consumer.update_and_process_changes(
            TreeUpdate {
                nodes: vec![(accepted, node)],
                tree: Some(TreeInfo::new(accepted)),
                tree_id: TreeId::ROOT,
                focus: accepted,
            },
            &mut Ignore,
        );
        assert!(
            consumer
                .state()
                .node_by_tree_local_id(A11yNodeId(0), TreeId::ROOT)
                .is_none()
        );
        let root = consumer
            .state()
            .node_by_tree_local_id(accepted, TreeId::ROOT)
            .unwrap();
        assert_eq!(root.role(), Role::Window);
        assert_eq!(root.label().as_deref(), Some("accepted picture"));
        let host = A11yHost::new(|| {});
        for action in [Action::Click, Action::Focus] {
            assert_eq!(
                host.map_parts(action, None, TreeId::ROOT, A11yNodeId(0)),
                None
            );
        }
    }

    #[test]
    fn bootstrap_without_a_native_window_installs_and_publishes_nothing() {
        let mut host = A11yHost::new(|| {});
        Accessibility::prepare_native_window(&mut host);
        assert!(!host.installed);
        assert!(host.requests.is_empty());
        assert_eq!(host.adapter_state(), A11yAdapterState::Detached);
    }

    #[test]
    fn later_pending_bootstrap_preserves_installed_tree_routes() {
        // CPU control of the installed guard, not a physical adapter receipt.
        let mut host = A11yHost::new(|| {});
        host.installed = true;
        let dom = genet_scripted_dom::ScriptedDom::new();
        let request = A11yRequest::node(A11yAction::Focus, dom.document());
        host.requests.insert(A11yNodeId(19), request.clone());
        Accessibility::prepare_native_window(&mut host);
        assert_eq!(host.requests.get(&A11yNodeId(19)), Some(&request));
        assert!(host.installed);
    }
}
