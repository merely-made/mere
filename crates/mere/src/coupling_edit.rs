// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One exact coupling's authored strength draft, independent of selection.
//!
//! The draft holds the coupling and its owning field as they were when it
//! began, a text the person edits with its own undo, and refuses to produce a
//! candidate if either changed underneath it. Writing the candidate is the
//! host's: it goes through the host's journal and author, never a second store.

use kernel::graph::{Coupling, CouplingId, Field, Graph};

#[derive(Clone, Debug)]
pub struct StrengthDraft {
    /// The graph the draft was begun against; a host refuses a draft that
    /// names another.
    pub graph_id: uuid::Uuid,
    pub baseline: Coupling,
    owner: Field,
    pub text: String,
    undo: Vec<String>,
    redo: Vec<String>,
}

impl StrengthDraft {
    pub fn begin(graph: &Graph, graph_id: uuid::Uuid, id: CouplingId) -> Result<Self, String> {
        let baseline = graph
            .couplings()
            .find(|c| c.id == id)
            .ok_or("The exact coupling is no longer held")?
            .clone();
        let owner = graph
            .field(baseline.field)
            .ok_or("The coupling owner is not held")?
            .clone();
        if !owner.is_active() || !baseline.strength.is_finite() {
            return Err(
                "Strength editing requires an active owner and finite held strength".into(),
            );
        }
        Ok(Self {
            graph_id,
            text: baseline.strength.to_string(),
            baseline,
            owner,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }

    pub fn set_text(&mut self, text: String) -> Result<(), String> {
        if text.len() > 128 {
            return Err("Strength text exceeds 128 bytes".into());
        }
        if text != self.text {
            self.undo.push(std::mem::replace(&mut self.text, text));
            if self.undo.len() > 64 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        Ok(())
    }

    pub fn undo(&mut self, redo: bool) {
        let (from, to) = if redo {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        if let Some(text) = from.pop() {
            to.push(std::mem::replace(&mut self.text, text));
        }
    }

    pub fn candidate(&self, graph: &Graph) -> Result<Coupling, String> {
        if self.text.len() > 128 {
            return Err("Strength text exceeds 128 bytes; draft retained".into());
        }
        let held = graph
            .couplings()
            .find(|c| c.id == self.baseline.id)
            .ok_or("The exact coupling was withdrawn; draft retained")?;
        if held != &self.baseline || graph.field(held.field) != Some(&self.owner) {
            return Err("The coupling or its owner changed; discard and inspect again, or keep the draft for comparison".into());
        }
        let strength: f32 = self
            .text
            .trim()
            .parse()
            .map_err(|_| "Enter a finite strength number")?;
        if !strength.is_finite() {
            return Err("Strength must be finite; draft retained".into());
        }
        let mut candidate = held.clone();
        candidate.strength = strength;
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    use kernel::graph::{
        CouplingResponse, FieldDefinition, FieldExtent, FieldId, NodeSelector, ScalarField,
    };
    use uuid::Uuid;

    const GRAPH: Uuid = Uuid::from_u128(1);

    fn fid() -> FieldId {
        FieldId::from_uuid(Uuid::from_u128(101))
    }

    fn cid(n: u128) -> CouplingId {
        CouplingId::from_uuid(Uuid::from_u128(n))
    }

    fn graph(strength: f32) -> Graph {
        let mut graph = Graph::new();
        let field = Field::new(fid(), FieldDefinition::Scalar(ScalarField::Const(0.0)))
            .with_extent(FieldExtent::Region {
                min_x: -1.0,
                min_y: -1.0,
                max_x: 1.0,
                max_y: 1.0,
            });
        apply_graph_delta(&mut graph, GraphDelta::AddField { field });
        let coupling = Coupling::new(
            cid(201),
            fid(),
            NodeSelector::All,
            CouplingResponse::AttractToMin,
            strength,
        );
        apply_graph_delta(&mut graph, GraphDelta::AddCoupling { coupling });
        graph
    }

    #[test]
    fn a_draft_proposes_the_parsed_strength_and_leaves_the_rest_alone() {
        let graph = graph(0.02);
        let mut draft = StrengthDraft::begin(&graph, GRAPH, cid(201)).unwrap();
        assert_eq!(draft.graph_id, GRAPH);
        assert_eq!(draft.text, "0.02");
        draft.set_text(" 0.125 ".into()).unwrap();
        let candidate = draft.candidate(&graph).unwrap();
        assert_eq!(candidate.strength, 0.125);
        assert_eq!(candidate.id, draft.baseline.id);
        assert_eq!(candidate.field, draft.baseline.field);
    }

    #[test]
    fn bad_text_is_refused_and_keeps_the_draft() {
        let graph = graph(0.02);
        let mut draft = StrengthDraft::begin(&graph, GRAPH, cid(201)).unwrap();
        for text in ["NaN", "inf", "1e99", "words"] {
            draft.set_text(text.into()).unwrap();
            assert!(draft.candidate(&graph).is_err(), "{text}");
        }
        assert!(draft.set_text("1".repeat(129)).is_err());
        assert_eq!(
            draft.text, "words",
            "a refused text leaves the draft as it was"
        );
    }

    #[test]
    fn the_draft_has_its_own_undo_and_redo() {
        let graph = graph(0.02);
        let mut draft = StrengthDraft::begin(&graph, GRAPH, cid(201)).unwrap();
        draft.set_text("1".into()).unwrap();
        draft.set_text("2".into()).unwrap();
        draft.undo(false);
        assert_eq!(draft.text, "1");
        draft.undo(false);
        assert_eq!(draft.text, "0.02");
        draft.undo(true);
        assert_eq!(draft.text, "1");
        draft.set_text("3".into()).unwrap();
        draft.undo(true);
        assert_eq!(draft.text, "3", "a new edit clears the redo stack");
    }

    #[test]
    fn a_withdrawn_or_changed_coupling_refuses_the_candidate() {
        let held = graph(0.02);
        let mut draft = StrengthDraft::begin(&held, GRAPH, cid(201)).unwrap();
        draft.set_text("0.5".into()).unwrap();

        let mut withdrawn = held.clone();
        apply_graph_delta(&mut withdrawn, GraphDelta::RetractCoupling { id: cid(201) });
        assert!(
            draft
                .candidate(&withdrawn)
                .unwrap_err()
                .contains("withdrawn")
        );

        let changed = graph(0.3);
        assert!(draft.candidate(&changed).unwrap_err().contains("changed"));
        assert!(StrengthDraft::begin(&withdrawn, GRAPH, cid(201)).is_err());
    }
}
