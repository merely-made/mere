// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::Cell;
use std::rc::{Rc, Weak};

/// Weak publication identity for one host lifetime. It retains no host.
/// The host holds the marker strongly and sets it false when disposed. Use a
/// fresh marker for each host lifetime; never reactivate a disposed marker.
/// A custom host may supply its own marker; a different marker never authorizes an
/// existing Rootstock host, even with the same document and mount.
#[derive(Clone, Debug)]
pub struct A11yOwner(Weak<Cell<bool>>);

impl A11yOwner {
    pub fn new(marker: &Rc<Cell<bool>>) -> Self {
        Self(Rc::downgrade(marker))
    }

    pub fn is_live(&self) -> bool {
        self.0.upgrade().is_some_and(|owner| owner.get())
    }
}

impl PartialEq for A11yOwner {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}
impl Eq for A11yOwner {}
