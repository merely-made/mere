// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bringing a held dataset forward by re-projecting only what an edit touched
//! (quad index plan P4).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::mem::size_of;

use super::{Contribution, IndexedDataset, Row, meta, subject_term};
use crate::projection::ProjectionSource;
use crate::RdfSource;
use chartulary::quads::{TermId, View};
use oxrdf::Term;

/// What [`IndexedDataset::refresh`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refresh {
    /// The dataset was already at the graph's revision.
    Current,
    /// The touched sources were re-projected and applied in place.
    Incremental,
    /// The dataset was rebuilt with [`IndexedDataset::for_graph`]: the
    /// projection does not split, or a node was removed (the graph no longer
    /// holds the edges that say which other nodes' rows named it).
    Rebuilt,
}

impl IndexedDataset {
    /// Bring the dataset to `graph`'s current state by re-projecting only the
    /// `touched` sources and applying the difference to the index.
    ///
    /// The caller's contract: `touched` names every source an edit since the
    /// dataset's revision changed. Derive it only through the kernel's
    /// `touched_sources` over the captured edits, or rebuild with
    /// [`IndexedDataset::for_graph`]: that function matches every captured
    /// edit kind by name with no wildcard, so a new kind fails to compile until
    /// someone decides what it touches. The unchecked case is a caller building
    /// the list by hand, since the revision counters say that the graph moved,
    /// not which sources, and a missed source goes unnoticed. [`ProjectionSource::Resources`]
    /// re-projects every resource, and is assumed when the resource graph's
    /// version moved but no resource was named. A source whose IRI changed
    /// brings its dependents (a node's in-neighbours, a resource's in-edges).
    pub fn refresh<S: RdfSource + ?Sized>(
        &mut self,
        graph: &S,
        touched: &[ProjectionSource],
    ) -> Refresh {
        if self.is_current(graph) {
            return Refresh::Current;
        }
        if self.try_refresh(graph, touched).is_none() {
            *self = Self::for_graph(graph);
            return Refresh::Rebuilt;
        }
        Refresh::Incremental
    }

    /// `None` when only a rebuild can bring the dataset forward.
    fn try_refresh<S: RdfSource + ?Sized>(
        &mut self,
        graph: &S,
        touched: &[ProjectionSource],
    ) -> Option<()> {
        self.key?;
        let parts = self.parts.as_mut()?;
        let mut queue: BTreeSet<ProjectionSource> = touched.iter().copied().collect();
        let resources_version = graph.resource_graph().version();
        let names_a_resource = queue.iter().any(|source| {
            matches!(source, ProjectionSource::Resource(_) | ProjectionSource::Resources)
        });
        if resources_version != parts.resources_version && !names_a_resource {
            queue.insert(ProjectionSource::Resources);
        }
        if queue.remove(&ProjectionSource::Resources) {
            // Every resource, held before or present now.
            let is_resource = |source: &ProjectionSource| matches!(source, ProjectionSource::Resource(_));
            queue.extend(parts.sources.keys().copied().filter(is_resource));
            queue.extend(graph.sources().unwrap_or_default().into_iter().filter(is_resource));
        }

        // Net change in how many sources emit each row.
        let mut net: HashMap<Row, i64> = HashMap::new();
        let mut done: HashSet<ProjectionSource> = HashSet::new();
        while let Some(source) = queue.pop_first() {
            if !done.insert(source) {
                continue;
            }
            let old = parts.sources.remove(&source);
            let Some(quads) = graph.source_quads(source) else {
                // A removed node: its in-neighbours' rows named it.
                old.is_none().then_some(())?;
                continue;
            };
            if let Some(old_subject) = old.as_ref().and_then(|old| old.subject) {
                let same = quads
                    .subject
                    .as_deref()
                    .and_then(subject_term)
                    .and_then(|term| self.terms.lookup(&term))
                    == Some(old_subject);
                if !same {
                    // Every node whose rows name the old IRI is re-projected.
                    let dependents = graph.dependents(source);
                    queue.extend(dependents.into_iter().filter(|other| !done.contains(other)));
                }
            }
            let new = Contribution::new(&mut self.terms, quads);
            let old_rows: &[Row] = old.as_ref().map_or(&[], |old| &old.rows);
            diff(old_rows, &new.rows, &mut net);
            parts.sources.insert(source, new);
        }

        let (mut asserted, mut removed) = (Vec::new(), Vec::new());
        for (row, change) in net {
            if change == 0 {
                continue;
            }
            let before = match parts.shared.get(&row) {
                Some(&count) => i64::from(count),
                None => i64::from(self.index.contains(&row.0, &meta(row.1))),
            };
            let after = before + change;
            (after >= 0).then_some(())?;
            match (before, after) {
                (0, 1..) => asserted.push((row.0, meta(row.1))),
                (1.., 0) => removed.push((row.0, meta(row.1))),
                _ => {},
            }
            if after >= 2 {
                parts
                    .shared
                    .insert(row, u32::try_from(after).expect("sources fit u32"));
            } else {
                parts.shared.remove(&row);
            }
        }
        self.index.apply(asserted, [], removed);
        parts.resources_version = resources_version;
        self.key = Some(graph.projection_key());
        Some(())
    }

    /// Bytes the per-source record adds for [`refresh`](Self::refresh), by
    /// allocated capacity: each source's rows, the source map and the shared
    /// counts. Zero for a dataset whose projection does not split.
    pub fn refresh_bytes(&self) -> usize {
        let Some(parts) = &self.parts else {
            return 0;
        };
        let rows: usize = parts
            .sources
            .values()
            .map(|contribution| contribution.rows.len() * size_of::<Row>())
            .sum();
        let sources =
            parts.sources.capacity() * (size_of::<(ProjectionSource, Contribution)>() + 1);
        let shared = parts.shared.capacity() * (size_of::<(Row, u32)>() + 1);
        rows + sources + shared
    }

    /// Whether this dataset holds the same quads as a fresh
    /// [`IndexedDataset::for_graph`], in both views; the first difference
    /// otherwise. A test and debugging aid: it costs a rebuild.
    ///
    /// # Errors
    ///
    /// The first quad, with its view and held flag, that one side has and the
    /// other lacks.
    pub fn verify_against_fresh<S: RdfSource + ?Sized>(&self, graph: &S) -> Result<(), String> {
        let fresh = Self::for_graph(graph);
        for view in [View::Live, View::History] {
            let (mine, theirs) = (self.quad_set(view), fresh.quad_set(view));
            if let Some(extra) = mine.difference(&theirs).next() {
                return Err(format!("{view:?}: refreshed has {extra:?}, fresh does not"));
            }
            if let Some(missing) = theirs.difference(&mine).next() {
                return Err(format!("{view:?}: fresh has {missing:?}, refreshed does not"));
            }
        }
        Ok(())
    }

    fn quad_set(&self, view: View) -> HashSet<(Term, Term, Term, Option<Term>, bool)> {
        let term = |id: TermId| self.terms.resolve(id).clone();
        self.index
            .pattern(None, None, None, None, view)
            .map(|(quad, meta)| {
                let graph = (!quad.graph.is_default_graph()).then(|| term(quad.graph));
                (
                    term(quad.subject),
                    term(quad.predicate),
                    term(quad.object),
                    graph,
                    meta.held,
                )
            })
            .collect()
    }
}

/// Add `-1` for each row only in `old` and `+1` for each only in `new`; both
/// are sorted and distinct.
fn diff(old: &[Row], new: &[Row], net: &mut HashMap<Row, i64>) {
    let (mut i, mut j) = (0, 0);
    while i < old.len() || j < new.len() {
        match (old.get(i), new.get(j)) {
            (Some(a), Some(b)) if a == b => {
                i += 1;
                j += 1;
            },
            (Some(a), Some(b)) if a < b => {
                *net.entry(*a).or_default() -= 1;
                i += 1;
            },
            (Some(a), None) => {
                *net.entry(*a).or_default() -= 1;
                i += 1;
            },
            (_, Some(b)) => {
                *net.entry(*b).or_default() += 1;
                j += 1;
            },
            (None, None) => unreachable!(),
        }
    }
}
