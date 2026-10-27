// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The one rule for assertion handles over persisted edge rows: a handle
//! names one assertion on one row, across both strata. A repeat is refused
//! whether or not the copy is identical. Checked load and both freezes ask
//! here; none of them decides legality on its own.

use super::GraphStratum;
use crate::persistence::PersistedEdge;
use std::collections::BTreeMap;
use std::fmt;

/// The row that holds a handle, as a refusal names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleRow {
    pub stratum: GraphStratum,
    pub from: String,
    pub to: String,
}

impl HandleRow {
    fn of(stratum: GraphStratum, row: &PersistedEdge) -> Self {
        Self {
            stratum,
            from: row.from_node_id.clone(),
            to: row.to_node_id.clone(),
        }
    }
}

/// A handle that appears a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionHandleRepeat {
    pub statement_id: String,
    pub first: HandleRow,
    pub again: HandleRow,
}

impl fmt::Display for AssertionHandleRepeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let site = |row: &HandleRow| format!("{:?} pair {} -> {}", row.stratum, row.from, row.to);
        write!(
            f,
            "assertion handle {:?} repeats: held on {} and again on {}; one handle means one row",
            self.statement_id,
            site(&self.first),
            site(&self.again)
        )
    }
}

impl std::error::Error for AssertionHandleRepeat {}

/// Handles held so far, each with the row that holds it.
#[derive(Default)]
pub struct AssertionHandles<'a> {
    held: BTreeMap<&'a str, (GraphStratum, &'a PersistedEdge)>,
    legacy: BTreeMap<&'a str, &'a PersistedEdge>,
}

impl<'a> AssertionHandles<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Hold every handle of `row`; the first one already held is refused.
    pub fn admit(
        &mut self,
        stratum: GraphStratum,
        row: &'a PersistedEdge,
    ) -> Result<(), AssertionHandleRepeat> {
        for statement in row.retained_assertions() {
            let id = statement.statement_id.as_str();
            let first = self
                .held
                .get(id)
                .copied()
                .or_else(|| self.legacy.get(id).map(|row| (GraphStratum::Surface, *row)));
            if let Some((first, first_row)) = first {
                return Err(AssertionHandleRepeat {
                    statement_id: id.to_owned(),
                    first: HandleRow::of(first, first_row),
                    again: HandleRow::of(stratum, row),
                });
            }
            self.held.insert(id, (stratum, row));
        }
        Ok(())
    }

    /// Schema 0 Surface rows predate canonical handles, so repeats among them
    /// stay loadable. Rows admitted later still may not reuse their handles.
    pub fn hold_legacy_surface(&mut self, row: &'a PersistedEdge) {
        for statement in row.retained_assertions() {
            self.legacy
                .entry(statement.statement_id.as_str())
                .or_insert(row);
        }
    }

    pub fn holds(&self, handle: &str) -> bool {
        self.held.contains_key(handle) || self.legacy.contains_key(handle)
    }

    /// Admitted handles with their rows, in handle order.
    pub fn rows(&self) -> impl Iterator<Item = (&'a str, &'a PersistedEdge)> + '_ {
        self.held.iter().map(|(handle, (_, row))| (*handle, *row))
    }
}

/// Refuse the first handle that repeats across `rows`.
pub fn validate_assertion_handles<'a>(
    rows: impl IntoIterator<Item = (GraphStratum, &'a PersistedEdge)>,
) -> Result<(), AssertionHandleRepeat> {
    let mut handles = AssertionHandles::new();
    rows.into_iter()
        .try_for_each(|(stratum, row)| handles.admit(stratum, row))
}
