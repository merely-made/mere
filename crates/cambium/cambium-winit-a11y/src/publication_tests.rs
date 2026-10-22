// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cambium_rootstock::{OwnedLayout, WindowDom};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::LayoutDomMut;

#[test]
fn publication_tests_mapper_preserves_the_published_binding_until_maps_are_replaced() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(true));
    let owner = cambium_rootstock::A11yOwner::new(&marker);
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(
        root,
        "<custom-leaf key='7' style='display:block;width:80px;height:30px'></custom-leaf>",
    );
    let node = dom.dom_children(root).next().unwrap();
    let view = WindowDom::document(&dom);
    let layout = OwnedLayout::new(&view, &[""], 320., 240., &[], &HashMap::new());
    let mut leaves = LeafRegistry::new();
    let producers = ProducerRegistry::new();
    let knob = || {
        Box::new(sprigging::Knob::new(sprigging::Size {
            width: 80.,
            height: 30.,
        }))
    };
    leaves.insert(7, knob());
    let publish = |owner: &cambium_rootstock::A11yOwner, leaves: &LeafRegistry<u64>| {
        cambium_rootstock::project_semantics(&view, &layout, leaves, &producers, None)
            .publication_requests(owner, &view, leaves, &producers)
            .into_iter()
            .map(|(id, request)| (A11yNodeId(id), request))
            .collect::<HashMap<_, _>>()
    };
    let mut host = A11yHost::new(|| {});
    host.requests = publish(&owner, &leaves);
    let target = A11yNodeId(view.opaque_id(node));
    let old = host
        .map_parts(
            Action::SetValue,
            Some(ActionData::NumericValue(4.)),
            accesskit::TreeId::ROOT,
            target,
        )
        .unwrap();
    leaves.insert(7, knob());
    let next = publish(&owner, &leaves);
    let pending = host
        .map_parts(Action::Click, None, accesskit::TreeId::ROOT, target)
        .unwrap();
    assert!(pending.same_publication_target(&old));
    assert!(!pending.same_publication_target(&next[&target]));
    host.requests = next;
    assert!(
        !old.same_publication_target(
            &host
                .map_parts(Action::Focus, None, accesskit::TreeId::ROOT, target)
                .unwrap()
        )
    );
}

#[test]
fn publication_owner_tokens_do_not_keep_the_host_marker_alive() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(true));
    let owner = cambium_rootstock::A11yOwner::new(&marker);
    assert!(owner.is_live());
    drop(marker);
    assert!(!owner.is_live());
}

#[test]
fn window_attachment_refuses_another_handle_without_replacing_the_original() {
    use std::sync::Arc;
    let original = Arc::new(());
    let other = Arc::new(());
    let mut window = None;
    attach_window(&mut window, original.clone()).unwrap();
    attach_window(&mut window, original.clone()).unwrap();
    assert!(attach_window(&mut window, other.clone()).is_err());
    assert!(Arc::ptr_eq(window.as_ref().unwrap(), &original));
    assert_eq!(Arc::strong_count(&other), 1);
}
