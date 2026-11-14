// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A graph entry behind an `Arc` (engine split D164). Cloning a graph copies
//! pointers; every write (`DerefMut`) goes through `Arc::make_mut`, which
//! copies a shared entry first, so a write can never reach another clone. Two
//! clones of one graph then differ only in entries written since, and a walk
//! comparing pointers ([`Shared::ptr_eq`]) finds them. Serde, `Debug` and
//! comparisons see the entry itself.
use portolan::{Address, Addressed, Classified, Identified, Labeled, Predicated, RelationClass};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// An entry shared between graph clones until one of them writes it.
#[derive(Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Shared<T>(Arc<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T: Clone> Shared<T> {
    pub fn into_inner(self) -> T {
        Arc::unwrap_or_clone(self.0)
    }
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
    }
}

impl<T: serde::Serialize> serde::Serialize for Shared<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Shared<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(Self::new)
    }
}

impl<T: Identified> Identified for Shared<T> {
    type Id = T::Id;
    fn id(&self) -> &T::Id {
        self.0.id()
    }
}

impl<T: Addressed> Addressed for Shared<T> {
    fn addresses(&self) -> Vec<Address> {
        self.0.addresses()
    }
}

impl<T: Labeled> Labeled for Shared<T> {
    fn title(&self) -> Option<&str> {
        self.0.title()
    }
    fn tags(&self) -> Vec<String> {
        self.0.tags()
    }
}

impl<T: Classified> Classified for Shared<T> {
    fn class(&self) -> RelationClass {
        self.0.class()
    }
}

impl<T: Predicated> Predicated for Shared<T> {
    fn predicate(&self) -> Option<&str> {
        self.0.predicate()
    }
}

impl<'a, T> IntoIterator for &'a Shared<Vec<T>> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
