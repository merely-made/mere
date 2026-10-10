// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Swatches and facets (Scenograph editor plan, SE53 to SE70).
//!
//! A [`Swatch`] is a mere seen at a [`Scope`], small enough to embed. A
//! projection swatch names a recipe and variant, which compiles to a scene; a
//! reflection swatch follows another view, or a sibling cell, live. A
//! [`Facet`] lays swatches out as cells along one or two [`Axis`] values, each
//! axis declaring whether its cells share a scale. A facet compiles to one
//! scene with a space per cell. Only the view is portable here; editing
//! inside a swatch waits for an interaction intent (SE65).

use curation::SubgraphSpec;
use serde::{Deserialize, Serialize};

use crate::ValidationIssue;

/// What part of a mere a swatch shows (SE63). Ids are the host's spellings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum Scope {
    Node(String),
    Subgraph(SubgraphSpec),
    /// The graph nested within this node.
    NestedGraph(String),
    Mere(String),
    /// An explicit set of ids, as a person picked them (SE83).
    Selection(Vec<String>),
}

/// How a swatch gets its recipe (SE58).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SwatchMode {
    /// Owns a recipe: an authored definition and, optionally, one of its
    /// variants.
    Projection {
        definition_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        variant_id: Option<String>,
    },
    /// Follows another view's recipe and camera live (SE66).
    Reflection { of: Followed },
}

/// What a reflection follows (SE66).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Followed {
    /// A stored view, as pandect names one: `views/<app>/<view>`.
    View { app: String, view: String },
    /// Another swatch in the same facet.
    Cell { swatch_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Swatch {
    pub id: String,
    pub scope: Scope,
    pub mode: SwatchMode,
}

/// What an axis varies (SE54, SE59). Dynamics joins when the recipe has a
/// dynamics slot (SE69).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisKind {
    Scope,
    Arrangement,
    Dynamics,
}

/// Whether an axis's cells are drawn at one scale (SE56).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisScale {
    /// Sizes and distances compare across the axis.
    #[default]
    Shared,
    /// Each cell fits its own content.
    Independent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    pub kind: AxisKind,
    /// One label per position along the axis.
    pub labels: Vec<String>,
    #[serde(default)]
    pub scale: AxisScale,
}

/// One cell: a swatch at a column and, when the facet has rows, a row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FacetCell {
    pub column: usize,
    #[serde(default)]
    pub row: usize,
    pub swatch: Swatch,
}

/// Swatches laid out along one or two axes (SE64).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Facet {
    pub id: String,
    pub columns: Axis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Axis>,
    pub cells: Vec<FacetCell>,
}

impl Facet {
    /// Every position is on an axis, held by one cell at most, swatch ids are
    /// unique, and a reflection follows a cell that exists and is not itself.
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let rows = self.rows.as_ref().map_or(1, |axis| axis.labels.len());
        if self.columns.labels.is_empty() {
            issues.push(ValidationIssue::error(
                "columns",
                "a facet needs at least one column",
            ));
        }
        if self
            .rows
            .as_ref()
            .is_some_and(|axis| axis.labels.is_empty())
        {
            issues.push(ValidationIssue::error(
                "rows",
                "an axis needs at least one label",
            ));
        }
        let mut seen_positions = Vec::new();
        let mut seen_ids: Vec<&str> = Vec::new();
        for (index, cell) in self.cells.iter().enumerate() {
            let field = format!("cells[{index}]");
            if cell.column >= self.columns.labels.len() || cell.row >= rows {
                issues.push(ValidationIssue::error(
                    &field,
                    "sits outside the facet's axes",
                ));
            }
            if seen_positions.contains(&(cell.row, cell.column)) {
                issues.push(ValidationIssue::error(
                    &field,
                    "shares its position with another cell",
                ));
            }
            seen_positions.push((cell.row, cell.column));
            if seen_ids.contains(&cell.swatch.id.as_str()) {
                issues.push(ValidationIssue::error(
                    &field,
                    "repeats another cell's swatch id",
                ));
            }
            seen_ids.push(&cell.swatch.id);
        }
        for (index, cell) in self.cells.iter().enumerate() {
            if let SwatchMode::Reflection {
                of: Followed::Cell { swatch_id },
            } = &cell.swatch.mode
            {
                let field = format!("cells[{index}].swatch.mode");
                if swatch_id == &cell.swatch.id {
                    issues.push(ValidationIssue::error(
                        &field,
                        "a cell cannot reflect itself",
                    ));
                } else if !self.cells.iter().any(|other| &other.swatch.id == swatch_id) {
                    issues.push(ValidationIssue::error(
                        &field,
                        "reflects a cell this facet does not have",
                    ));
                }
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

#[cfg(test)]
mod tests {
    use curation::SubgraphKind;

    use super::*;

    fn projection(id: &str, variant: &str) -> Swatch {
        Swatch {
            id: id.into(),
            scope: Scope::Subgraph(SubgraphSpec {
                kind: SubgraphKind::Ego { radius: 1 },
                anchors: vec!["n1".into()],
                primary_anchor: Some("n1".into()),
                selectors: Vec::new(),
            }),
            mode: SwatchMode::Projection {
                definition_id: "recipe".into(),
                variant_id: Some(variant.into()),
            },
        }
    }

    fn facet() -> Facet {
        Facet {
            id: "compare".into(),
            columns: Axis {
                kind: AxisKind::Arrangement,
                labels: vec!["Working".into(), "Spiral".into()],
                scale: AxisScale::Shared,
            },
            rows: None,
            cells: vec![
                FacetCell {
                    column: 0,
                    row: 0,
                    swatch: Swatch {
                        id: "working".into(),
                        scope: Scope::Mere("m".into()),
                        mode: SwatchMode::Reflection {
                            of: Followed::View {
                                app: "graphshell".into(),
                                view: "projection-editor".into(),
                            },
                        },
                    },
                },
                FacetCell {
                    column: 1,
                    row: 0,
                    swatch: projection("spiral", "spiral"),
                },
            ],
        }
    }

    #[test]
    fn a_facet_mixing_a_reflection_and_a_projection_validates_and_round_trips() {
        let facet = facet();
        assert_eq!(facet.validate(), Ok(()));
        let json = serde_json::to_string(&facet).expect("writes");
        assert!(json.contains(r#""mode":{"kind":"reflection","of":{"kind":"view","app":"graphshell","view":"projection-editor"}}"#));
        assert!(
            json.contains(r#""scope":{"kind":"subgraph","value":{"kind":{"Ego":{"radius":1}}"#)
        );
        assert_eq!(serde_json::from_str::<Facet>(&json).expect("reads"), facet);
    }

    #[test]
    fn a_scale_left_out_is_shared() {
        let axis: Axis = serde_json::from_str(r#"{"kind":"scope","labels":["a"]}"#).expect("reads");
        assert_eq!(axis.scale, AxisScale::Shared);
    }

    #[test]
    fn misplaced_repeated_and_dangling_cells_are_refused() {
        let mut facet = facet();
        facet.cells.push(FacetCell {
            column: 1,
            row: 0,
            swatch: projection("spiral", "grid"),
        });
        facet.cells.push(FacetCell {
            column: 2,
            row: 0,
            swatch: projection("far", "grid"),
        });
        facet.cells.push(FacetCell {
            column: 0,
            row: 1,
            swatch: Swatch {
                id: "echo".into(),
                scope: Scope::Node("n1".into()),
                mode: SwatchMode::Reflection {
                    of: Followed::Cell {
                        swatch_id: "missing".into(),
                    },
                },
            },
        });
        let messages: Vec<String> = facet
            .validate()
            .expect_err("refused")
            .into_iter()
            .map(|issue| format!("{}: {}", issue.field, issue.message))
            .collect();
        assert!(messages.contains(&"cells[2]: shares its position with another cell".to_string()));
        assert!(messages.contains(&"cells[2]: repeats another cell's swatch id".to_string()));
        assert!(messages.contains(&"cells[3]: sits outside the facet's axes".to_string()));
        assert!(messages.contains(&"cells[4]: sits outside the facet's axes".to_string()));
        assert!(messages.contains(
            &"cells[4].swatch.mode: reflects a cell this facet does not have".to_string()
        ));
    }
}
