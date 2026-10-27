// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The one bounds check on a navigation snapshot's cross-references.
//!
//! Stemma hydration indexes its columns without checking, so any snapshot
//! load that installs a decoded [`SharedNavigationMemory`] runs this first.

use std::fmt;

use super::history::SharedNavigationMemory;

/// An index in the navigation snapshot that names no row of its column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationIndexError {
    /// The referring column, for example `visits.parent`.
    pub field: &'static str,
    /// The row of that column holding the index.
    pub at: usize,
    pub index: usize,
    /// How many rows the referenced column has.
    pub len: usize,
}

impl fmt::Display for NavigationIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            field,
            at,
            index,
            len,
        } = self;
        write!(
            f,
            "navigation {field}[{at}] names row {index} of {len} rows"
        )
    }
}

impl SharedNavigationMemory {
    /// Refuse a snapshot whose entry, visit or owner indices leave their column.
    pub fn check_indices(&self) -> Result<(), NavigationIndexError> {
        let snapshot = self.snapshot();
        let (entries, visits, owners) = (
            snapshot.entries.len(),
            snapshot.visits.len(),
            snapshot.owners.len(),
        );
        let within = |field, at, index: usize, len| {
            if index < len {
                return Ok(());
            }
            Err(NavigationIndexError {
                field,
                at,
                index,
                len,
            })
        };
        let optional = |field, at, index: Option<usize>, len| {
            index.map_or(Ok(()), |index| within(field, at, index, len))
        };
        for (at, visit) in snapshot.visits.iter().enumerate() {
            within("visits.entry", at, visit.entry, entries)?;
            optional("visits.parent", at, visit.parent, visits)?;
            for child in &visit.children {
                within("visits.children", at, *child, visits)?;
            }
            for binding in &visit.bindings {
                within("visits.bindings.owner", at, binding.owner, owners)?;
                optional(
                    "visits.bindings.forward_child",
                    at,
                    binding.forward_child,
                    visits,
                )?;
            }
        }
        for (at, owner) in snapshot.owners.iter().enumerate() {
            optional("owners.origin", at, owner.origin, visits)?;
            optional("owners.current", at, owner.current, visits)?;
            optional(
                "owners.pending_origin_parent",
                at,
                owner.pending_origin_parent,
                visits,
            )?;
            optional("owners.creator", at, owner.creator, owners)?;
            for visit in &owner.owned_visits {
                within("owners.owned_visits", at, *visit, visits)?;
            }
        }
        Ok(())
    }
}
