// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Read-only observations of actual field-coupling execution in this canvas.
use std::num::NonZeroUsize;

use super::Canvas;

impl Canvas {
    /// Enable observations with an explicit row cap, or disable with `None`.
    /// This records no graph edits and does not request simulation ticks.
    pub fn set_coupling_trace_limit(&mut self, limit: Option<NonZeroUsize>) {
        self.physics.set_coupling_trace_limit(limit);
    }

    /// Latest executed tick, using the exact bound owner inputs. Offloaded
    /// evidence arrives through the normal physics refresh. The node keys are
    /// local body handles; use the captured member IDs for historical identity.
    /// Final integrated positions and other forces are outside this report.
    pub fn coupling_trace(&self) -> Option<&seiche::CouplingTrace> {
        self.physics.coupling_trace()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::geometry::PortablePoint;
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    use kernel::graph::{
        Coupling, CouplingId, CouplingResponse, Field, FieldDefinition, FieldId, Graph,
        NodeSelector, VectorField,
    };
    use uuid::Uuid;

    #[test]
    fn canvas_live_rebuild_keeps_exact_owners_and_clears_retired_binding_trace() {
        let mut graph = Graph::new();
        let node = graph.add_node_with_id(
            Uuid::from_u128(42),
            "mere://trace".into(),
            PortablePoint::zero(),
        );
        let field = FieldId::from_uuid(Uuid::from_u128(100));
        apply_graph_delta(
            &mut graph,
            GraphDelta::AddField {
                field: Field::new(
                    field,
                    FieldDefinition::Vector(VectorField::ConstVec { x: 8.0, y: 0.0 }),
                ),
            },
        );
        for id in [1, 2] {
            apply_graph_delta(
                &mut graph,
                GraphDelta::AddCoupling {
                    coupling: Coupling::new(
                        CouplingId::from_uuid(Uuid::from_u128(id)),
                        field,
                        NodeSelector::All,
                        CouplingResponse::FlowAdvect,
                        1.0,
                    ),
                },
            );
        }
        let mut canvas = Canvas::with_graph(graph);
        canvas.set_coupling_trace_limit(NonZeroUsize::new(8));
        canvas.physics.settle(1);
        canvas.physics.advance_frame(&mut canvas.view);
        let old = canvas.coupling_trace().unwrap().clone();
        assert_eq!(old.records.len(), 2);
        let mut ids: Vec<_> = old
            .records
            .iter()
            .map(|row| {
                let source = row.binding.source.unwrap();
                assert_eq!(source.field, field);
                assert_eq!(row.node, node);
                assert_eq!(
                    row.binding.target_identities[row.target_index],
                    Some(seiche::CouplingTargetIdentity {
                        member: 42,
                        occurrence: None
                    })
                );
                source.coupling.as_uuid().as_u128()
            })
            .collect();
        ids.sort();
        assert_eq!(ids, [1, 2]);
        let before = canvas.view.position_of(node);
        apply_graph_delta(&mut canvas.graph, GraphDelta::RetireField { id: field });
        canvas.rebuild_coupling_forces();
        assert!(canvas.coupling_trace().is_none());
        assert_eq!(canvas.view.position_of(node), before);
        canvas.physics.settle(1);
        canvas.physics.advance_frame(&mut canvas.view);
        let current = canvas.coupling_trace().unwrap();
        assert!(current.records.is_empty());
        assert_eq!(current.binding_generation, old.binding_generation + 1);
        assert_eq!(current.session, old.session);
        // A different canvas has a distinct runtime owner even with the same IDs.
        let mut other = Canvas::with_graph(canvas.graph.clone());
        other.set_coupling_trace_limit(NonZeroUsize::new(8));
        other.rebuild_coupling_forces();
        other.physics.settle(1);
        other.physics.advance_frame(&mut other.view);
        assert_ne!(other.coupling_trace().unwrap().session, old.session);
    }
}
