// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
