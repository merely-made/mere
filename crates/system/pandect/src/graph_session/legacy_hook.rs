// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The legacy translation, as a session over any graph reaches it (engine
//! split D252, D260). A [`LegacyTranslationReceipt`] is read over the
//! session's graph type through the host's [`LegacyHook`]; a session without
//! one refuses a stored receipt.

use super::*;
pub use crate::graph_placement::LegacyHook;

/// The kernel graph's hook.
pub(super) use crate::graph_placement::KERNEL_LEGACY as KERNEL;

impl<B: Backend + Clone, G: SessionGraph> MereSessions<B, G> {
    /// Read a stored legacy translation of every session these open through
    /// `hook` (engine split D260).
    pub fn with_legacy(mut self, hook: &'static LegacyHook<G>) -> Self {
        self.hooks.legacy = Some(hook);
        self
    }
}
