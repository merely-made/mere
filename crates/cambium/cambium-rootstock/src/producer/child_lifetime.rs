// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Weak};

/// One authored child's life. Create at model insertion/replacement, retain
/// through ordinary updates, and drop on removal. Semantic reads only observe
/// its token; they must not allocate or reconcile lives.
#[derive(Clone, Debug, Default)]
pub struct ProducerChildLifetime(Arc<()>);

impl ProducerChildLifetime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(&self) -> ProducerChildToken {
        ProducerChildToken(Arc::downgrade(&self.0))
    }
}

impl PartialEq for ProducerChildLifetime {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ProducerChildLifetime {}

/// An observation of a child life, retaining neither the child nor its owner.
/// Compare with the current model's token, including when an old owner is live.
#[derive(Clone, Debug)]
pub struct ProducerChildToken(Weak<()>);

impl ProducerChildToken {
    pub fn is_live(&self) -> bool {
        self.0.upgrade().is_some()
    }

    /// Check within the mutable model borrow that will dispatch the action.
    /// Two absent tokens preserve the explicit nonreused-key contract.
    pub fn matches_current(expected: Option<&Self>, current: Option<&Self>) -> bool {
        match (expected, current) {
            (None, None) => true,
            (Some(a), Some(b)) => a.is_live() && b.is_live() && a == b,
            _ => false,
        }
    }
}

impl PartialEq for ProducerChildToken {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}
impl Eq for ProducerChildToken {}
