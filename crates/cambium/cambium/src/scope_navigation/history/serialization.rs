// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<P: ScopeValue, A: ScopeValue> Serialize for ScopeNavigation<P, A> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WireRef {
            version: WIRE_VERSION,
            lineage: &self.lineage,
            bookmarks: self
                .bookmarks
                .iter()
                .enumerate()
                .map(|(visit, value)| IndexedBookmarkRef { visit, value })
                .collect(),
            remembered: &self.remembered,
        }
        .serialize(serializer)
    }
}

impl<'de, P: ScopeValue, A: ScopeValue> Deserialize<'de> for ScopeNavigation<P, A> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| D::Error::custom("Expected scope navigation object"))?;
        // Current-format markers commit to current decoding. Corrupt new data
        // must never be mistaken for a legacy route.
        if ["version", "lineage", "bookmarks"]
            .iter()
            .any(|key| object.contains_key(*key))
        {
            let wire: Wire<P, A> = serde_json::from_value(value).map_err(D::Error::custom)?;
            if wire.version != WIRE_VERSION {
                return Err(D::Error::custom("Unsupported scope navigation version"));
            }
            if wire.bookmarks.len() != wire.lineage.visits.len() {
                return Err(D::Error::custom("Scope bookmark count mismatch"));
            }
            let mut bookmarks = vec![None; wire.lineage.visits.len()];
            for bookmark in wire.bookmarks {
                let Some(slot) = bookmarks.get_mut(bookmark.visit) else {
                    return Err(D::Error::custom("Scope bookmark visit out of bounds"));
                };
                if slot.replace(bookmark.value).is_some() {
                    return Err(D::Error::custom("Duplicate scope bookmark visit"));
                }
            }
            let bookmarks = bookmarks
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| D::Error::custom("Missing scope bookmark visit"))?;
            let nav = Self {
                lineage: canonical_snapshot(wire.lineage),
                bookmarks,
                remembered: wire.remembered,
            };
            validate_wire_state(&nav.lineage, &nav.bookmarks, &nav.remembered)
                .map_err(D::Error::custom)?;
            return Ok(nav);
        }

        let legacy: LegacyWire<P, A> = serde_json::from_value(value).map_err(D::Error::custom)?;
        if legacy.entries.len() > LEGACY_MAX_ENTRIES
            || legacy.remembered.len() > MAX_REMEMBERED
            || (legacy.entries.is_empty() && legacy.cursor != 0)
            || (!legacy.entries.is_empty() && legacy.cursor >= legacy.entries.len())
        {
            return Err(D::Error::custom("Invalid legacy scope navigation history"));
        }
        for visit in legacy.entries.iter().chain(&legacy.remembered) {
            validate_scope_visit(visit).map_err(D::Error::custom)?;
        }
        let mut nav = Self::default();
        for visit in legacy.entries {
            // Legacy history retained repeated places as separate positions.
            nav.append_visit(visit);
        }
        if nav.lineage.visits.is_empty() {
            if legacy.cursor != 0 {
                return Err(D::Error::custom("Invalid legacy scope cursor"));
            }
        } else {
            let current_index = legacy.cursor;
            let last_index = nav.bookmarks.len() - 1;
            for _ in current_index..last_index {
                if !nav.step(false) {
                    return Err(D::Error::custom("Invalid legacy scope cursor"));
                }
            }
        }
        // Reapply old bookmark ordering and exact-place replacement semantics.
        nav.remembered.clear();
        // Legacy route entries were remembered as they were replayed. The old
        // list only persists the final bounded bookmark set, so preserve it.
        for visit in legacy.remembered {
            nav.remember(visit);
        }
        Ok(nav)
    }
}
