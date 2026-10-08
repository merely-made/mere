// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the Meaning receipts on the arXiv fixture share: a snapshot through
//! the canvas's own path, the purity row, the confusion print, and an engine
//! that replays vectors embedded once.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use kernel::graph::NodeKey;
use pictograph::canvas::{
    Canvas, Embedded, MeaningBackend, MeaningEngine, MeaningParams, MeaningSnapshot, PhysicsChoice,
    PhysicsKindSource, PhysicsLaw,
};

#[path = "../../src/canvas/tests/meaning_topics.rs"]
pub mod meaning_topics;

use meaning_topics::{
    ARXIV_CATEGORIES, arxiv_graph, confusion, f_measure, inverse_purity, partition, purity,
};

/// One snapshot on `engine`, through the canvas's ordinary path: Kinds by
/// meaning, inline.
pub fn snapshot_on(engine: Arc<dyn MeaningEngine>) -> (MeaningSnapshot, u64) {
    let (graph, _, _) = arxiv_graph();
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_meaning_engine(engine);
    // Kinds by meaning, set through the canvas's spec and its flat view
    // (dynamics grammar plan, F162).
    let mut spec = canvas.dynamics_spec().expect("the record reads");
    PhysicsChoice {
        law: PhysicsLaw::Kinds,
        kind: PhysicsKindSource::Meaning,
        ..PhysicsChoice::live(&canvas)
    }
    .write_into(&mut spec);
    canvas.set_dynamics_spec(&spec).expect("not refused");
    let snapshot = canvas.meaning().expect("a snapshot at build").clone();
    (snapshot, canvas.meaning_runs())
}

/// Purity, inverse purity, F, the group count and F against shuffled
/// topics, printed; `(F, F against shuffled)` returned.
pub fn row(
    name: &str,
    groups: &[(NodeKey, u32)],
    topics: &HashMap<NodeKey, usize>,
    shuffled: &HashMap<NodeKey, usize>,
) -> (f64, f64) {
    let f = f_measure(groups, topics);
    let control = f_measure(groups, shuffled);
    println!(
        "{name}: purity {:.3}, inverse purity {:.3}, F {f:.3}, groups {}, F against shuffled topics {control:.3}",
        purity(groups, topics),
        inverse_purity(groups, topics),
        partition(groups).len(),
    );
    (f, control)
}

/// Each large group's counts by category.
pub fn print_confusion(name: &str, groups: &[(NodeKey, u32)], topics: &HashMap<NodeKey, usize>) {
    println!(
        "{name}: groups holding at least 2% of the titles, counts by category {ARXIV_CATEGORIES:?}"
    );
    for (size, counts) in confusion(groups, topics, ARXIV_CATEGORIES.len(), 0.02) {
        println!("  group of {size:>3}: {counts:?}");
    }
}

/// Vectors embedded once, replayed for a sweep's tunings.
pub struct Fixed {
    pub vectors: Embedded,
    pub params: MeaningParams,
    pub backend: MeaningBackend,
}

impl MeaningEngine for Fixed {
    fn backend(&self) -> MeaningBackend {
        self.backend
    }

    fn params(&self) -> MeaningParams {
        self.params
    }

    fn embed(&self, texts: &[&str]) -> Result<Embedded, esp::embed::EmbedError> {
        assert_eq!(texts.len(), 900, "a sweep replays the whole corpus at once");
        Ok(self.vectors.clone())
    }
}

/// The fixture's titles, in key order.
pub fn arxiv_titles() -> Vec<String> {
    let (graph, keys, _) = arxiv_graph();
    keys.iter()
        .map(|k| graph.get_node(*k).unwrap().title.clone())
        .collect()
}
