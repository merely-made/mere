// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resolved dataset a host hands the projection compiler.
//!
//! A product adapter reads its own authority and resolves it into this generic
//! table: typed fields, and occurrences carrying values and an exact source
//! reference. It is the in-host input to compilation and never part of the
//! remote projection protocol, where endpoints compile locally and send
//! scores. Its shape is product-free, so a new product adds rows, not types.

use std::collections::BTreeMap;

use sceno::SourceRef;
use serde::{Deserialize, Serialize};

use crate::{PublicSourceRevision, SourceBinding};

/// The type a product disclosed for a field in one resolved dataset.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionFieldType {
    Text,
    Number,
    Boolean,
}

/// One source value, kept small enough to make an adapter's disclosure plain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum ProjectionValue {
    Text(String),
    Number(f64),
    Boolean(bool),
}

impl ProjectionValue {
    pub fn field_type(&self) -> ProjectionFieldType {
        match self {
            Self::Text(_) => ProjectionFieldType::Text,
            Self::Number(_) => ProjectionFieldType::Number,
            Self::Boolean(_) => ProjectionFieldType::Boolean,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
}

/// One occurrence in a product's resolved reading.
///
/// `occurrence_id` is the host's selection and persistence identity. `source`
/// remains source truth identity, so two occurrences may intentionally name the
/// same source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionOccurrence {
    pub occurrence_id: String,
    pub source: SourceRef,
    pub values: BTreeMap<String, ProjectionValue>,
}

/// A product-resolved dataset, supplied to the compiler without a product
/// dependency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionDataset {
    pub source: SourceBinding,
    pub revision: PublicSourceRevision,
    pub fields: BTreeMap<String, ProjectionFieldType>,
    pub occurrences: Vec<ProjectionOccurrence>,
}
