// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//!
//! Bounded validation of graph-field dependencies used by responsive paths.

use std::collections::HashMap;

use kernel::graph::{FieldDefinition, FieldExtent, FieldId, Graph};
use numen::{MAX_FIELD_AST_DEPTH, MAX_FIELD_AST_NODES, ScalarField, VectorField};

use super::LinkPathFailure;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SampleKind {
    Scalar,
    Vector,
}

pub(super) fn validate_dependencies(
    graph: &Graph,
    roots: &[&VectorField],
) -> Result<Vec<FieldId>, LinkPathFailure> {
    let mut remaining = MAX_FIELD_AST_NODES;
    let mut pending = Vec::new();
    for root in roots {
        collect_vector(root, 0, &mut remaining, &mut pending)?;
    }

    let mut visited = HashMap::new();
    let mut dependencies = HashMap::new();
    while let Some((id, expected, depth)) = pending.pop() {
        if depth > MAX_FIELD_AST_DEPTH {
            return Err(LinkPathFailure::FieldDependencyTooLarge);
        }
        let Some(field) = graph.field(id) else {
            return Err(LinkPathFailure::MissingField(id));
        };
        if !field.is_active() {
            return Err(LinkPathFailure::RetiredField(id));
        }
        if !matches!(field.extent, FieldExtent::Global) {
            return Err(LinkPathFailure::UnsupportedExtent(id));
        }
        let kind_matches = matches!(
            (expected, &field.definition),
            (SampleKind::Vector, FieldDefinition::Vector(_))
                | (SampleKind::Scalar, FieldDefinition::Scalar(_))
        );
        if !kind_matches {
            return Err(LinkPathFailure::FieldSampleKindMismatch(id));
        }
        let visit_key = (id.as_uuid(), expected);
        if visited
            .get(&visit_key)
            .is_some_and(|previous| *previous >= depth)
        {
            continue;
        }
        visited.insert(visit_key, depth);
        dependencies.insert(id.as_uuid(), id);
        match &field.definition {
            FieldDefinition::Vector(vector) => {
                collect_vector(vector, depth, &mut remaining, &mut pending)?
            },
            FieldDefinition::Scalar(scalar) => {
                collect_scalar(scalar, depth, &mut remaining, &mut pending)?
            },
        }
    }
    Ok(dependencies.into_values().collect())
}

fn count_node(depth: usize, remaining: &mut usize) -> Result<(), LinkPathFailure> {
    if depth > MAX_FIELD_AST_DEPTH || *remaining == 0 {
        return Err(LinkPathFailure::FieldDependencyTooLarge);
    }
    *remaining -= 1;
    Ok(())
}

fn collect_vector(
    field: &VectorField,
    depth: usize,
    remaining: &mut usize,
    pending: &mut Vec<(FieldId, SampleKind, usize)>,
) -> Result<(), LinkPathFailure> {
    count_node(depth, remaining)?;
    match field {
        VectorField::ConstVec { .. } | VectorField::Coord => {},
        VectorField::Gradient(scalar) => collect_scalar(scalar, depth + 1, remaining, pending)?,
        VectorField::Perp(vector) | VectorField::ScaleConst(vector, _) => {
            collect_vector(vector, depth + 1, remaining, pending)?
        },
        VectorField::Add(left, right) => {
            collect_vector(left, depth + 1, remaining, pending)?;
            collect_vector(right, depth + 1, remaining, pending)?;
        },
        VectorField::Scale(vector, scalar) => {
            collect_vector(vector, depth + 1, remaining, pending)?;
            collect_scalar(scalar, depth + 1, remaining, pending)?;
        },
        VectorField::Sample(id) => pending.push((*id, SampleKind::Vector, depth + 1)),
    }
    Ok(())
}

fn collect_scalar(
    field: &ScalarField,
    depth: usize,
    remaining: &mut usize,
    pending: &mut Vec<(FieldId, SampleKind, usize)>,
) -> Result<(), LinkPathFailure> {
    count_node(depth, remaining)?;
    match field {
        ScalarField::Const(_) | ScalarField::CoordX | ScalarField::CoordY | ScalarField::Time => {},
        ScalarField::Gaussian { center, .. } | ScalarField::Disk { center, .. } => {
            collect_vector(center, depth + 1, remaining, pending)?
        },
        ScalarField::Linear { normal, .. } => {
            collect_vector(normal, depth + 1, remaining, pending)?
        },
        ScalarField::Add(left, right) | ScalarField::Mul(left, right) => {
            collect_scalar(left, depth + 1, remaining, pending)?;
            collect_scalar(right, depth + 1, remaining, pending)?;
        },
        ScalarField::Scale(scalar, _) | ScalarField::Negate(scalar) => {
            collect_scalar(scalar, depth + 1, remaining, pending)?
        },
        ScalarField::Dot(left, right) => {
            collect_vector(left, depth + 1, remaining, pending)?;
            collect_vector(right, depth + 1, remaining, pending)?;
        },
        ScalarField::Sample(id) => pending.push((*id, SampleKind::Scalar, depth + 1)),
    }
    Ok(())
}
