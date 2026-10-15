// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Arrangement options as an editor shows them: the rows an arrangement id
//! declares, what a row shows when it is left out, and the edits that set or
//! clear one. Any host that edits an authored arrangement (a recipe editor, a
//! projection editor) reads the same declarations rather than keeping copies.

use std::collections::BTreeMap;

use scenograph::Arrangement;
use scenograph::options::{OptionDefault, OptionSpec};

use crate::catalog::{Family, Measure};
use crate::registry::SolverRegistry;

/// The options an arrangement id declares: a built-in family's, else a
/// registered solver's; `None` when the id names neither (track E5).
pub fn arrangement_options(kind: &str, registry: &SolverRegistry) -> Option<Vec<OptionSpec>> {
    Family::resolve(kind).map(Family::options).or_else(|| {
        registry
            .resolve(kind)
            .map(|solver| solver.capability().options)
    })
}

/// A built-in family's defaults resolved for these items, by option key;
/// empty for anything else (SE43).
pub fn arrangement_defaults(
    kind: &str,
    card: sceno::Size2,
    count: usize,
    spacing: f32,
) -> BTreeMap<String, String> {
    Family::resolve(kind)
        .map(|family| {
            family.resolved_defaults(&Measure {
                largest: card,
                count,
                spacing,
                coordinates: None,
            })
        })
        .unwrap_or_default()
}

/// What leaving an option out means, as values. A host words it for its own
/// rows: this carries the facts a label is made from, never the label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionFallback {
    /// A fixed default value.
    Value(String),
    /// Measured from the items. `resolved` is what it came to for them, when
    /// the host knows the items; `words` says what is measured.
    Measured {
        resolved: Option<String>,
        words: String,
    },
    /// Chosen by the arrangement from the items' shape. `resolved` is what it
    /// chose for them, when the host knows the items.
    Auto { resolved: Option<String> },
    /// No default: leaving the option out leaves it unset.
    Empty,
}

/// What `spec` falls back to when left out, given the number or name the items
/// resolved it to (from [`arrangement_defaults`]) when the host has them.
pub fn option_fallback(spec: &OptionSpec, resolved: Option<&str>) -> OptionFallback {
    let resolved = resolved.map(str::to_owned);
    match &spec.default {
        OptionDefault::Value(value) => OptionFallback::Value(value.clone()),
        OptionDefault::Measured(words) => OptionFallback::Measured {
            resolved,
            words: words.clone(),
        },
        OptionDefault::Auto => OptionFallback::Auto { resolved },
        OptionDefault::Empty => OptionFallback::Empty,
    }
}

/// `arrangement` with `key` set to `value`, or left out when `value` is
/// blank, so clearing a row returns the option to its default.
pub fn with_option(arrangement: &Arrangement, key: &str, value: &str) -> Arrangement {
    let mut next = arrangement.clone();
    if value.trim().is_empty() {
        next.options.remove(key);
    } else {
        next.options
            .insert(key.to_string(), value.trim().to_string());
    }
    next
}

/// `arrangement` switched to `kind`, keeping only the options `kind`
/// declares (SE44); the editor's history can undo the switch. While `kind`
/// names nothing known (an id being typed), every option is kept.
pub fn with_kind(arrangement: &Arrangement, kind: &str, registry: &SolverRegistry) -> Arrangement {
    let mut next = arrangement.clone();
    next.kind = kind.to_string();
    if let Some(declared) = arrangement_options(kind, registry) {
        next.options
            .retain(|key, _| declared.iter().any(|spec| &spec.key == key));
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenograph::options::OptionKind;

    #[test]
    fn every_family_and_a_registered_solver_have_rows() {
        let empty = SolverRegistry::new();
        for family in crate::catalog::FAMILIES {
            assert_eq!(
                arrangement_options(family.id(), &empty),
                Some(family.options())
            );
        }
        assert_eq!(arrangement_options("nothing.like.it", &empty), None);

        struct Line;
        impl crate::registry::Solver for Line {
            fn capability(&self) -> crate::registry::SolverCapability {
                let mut capability = crate::registry::SolverCapability::new("test.line", "Line");
                capability.options = vec![OptionSpec::new(
                    "step",
                    "Step",
                    OptionKind::Positive,
                    OptionDefault::Value("10".into()),
                )];
                capability
            }
            fn place(
                &self,
                _config: &serde_json::Value,
                items: &[&sceno::ScoreItem],
            ) -> Result<Vec<sceno::Vec2>, crate::registry::SolveError> {
                Ok(vec![sceno::Vec2::ZERO; items.len()])
            }
        }
        let mut registry = SolverRegistry::new();
        registry.register(std::sync::Arc::new(Line)).unwrap();
        let rows = arrangement_options("test.line", &registry).expect("the solver's rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].key, "step");
    }

    #[test]
    fn a_fallback_carries_the_default_and_what_the_items_resolved_it_to() {
        let card = sceno::Size2::new(164.0, 68.0);
        let defaults = arrangement_defaults("grid", card, 9, 16.0);
        let option = |family: Family, key: &str| {
            family
                .options()
                .into_iter()
                .find(|spec| spec.key == key)
                .unwrap()
        };
        let width = option(Family::Grid, "cell_width");
        assert_eq!(
            option_fallback(&width, defaults.get("cell_width").map(String::as_str)),
            OptionFallback::Measured {
                resolved: Some("164".into()),
                words: "the largest item's width".into(),
            }
        );
        assert_eq!(
            option_fallback(&width, None),
            OptionFallback::Measured {
                resolved: None,
                words: "the largest item's width".into(),
            }
        );
        let curve = option(Family::Spiral, "curve");
        assert_eq!(
            option_fallback(&curve, None),
            OptionFallback::Value("square_root".into())
        );
        let depth = option(Family::LSystem, "depth");
        assert_eq!(
            option_fallback(&depth, None),
            OptionFallback::Auto { resolved: None }
        );
        assert_eq!(
            option_fallback(&depth, Some("auto")),
            OptionFallback::Auto {
                resolved: Some("auto".into())
            }
        );
        let empty = OptionSpec::new("note", "Note", OptionKind::Positive, OptionDefault::Empty);
        assert_eq!(option_fallback(&empty, None), OptionFallback::Empty);
    }

    #[test]
    fn switching_arrangement_drops_options_the_new_one_does_not_declare() {
        let registry = SolverRegistry::new();
        let grid = with_option(
            &with_option(&Arrangement::default(), "columns", "3"),
            "cell_width",
            "200",
        );
        let spiral = with_kind(&grid, "spiral", &registry);
        assert_eq!(spiral.kind, "spiral");
        assert!(spiral.options.is_empty());
        let typing = with_kind(&grid, "gri", &registry);
        assert_eq!(typing.options, grid.options, "an id being typed keeps them");
        let back = with_kind(&typing, "grid", &registry);
        assert_eq!(back.options, grid.options);
    }

    #[test]
    fn a_blank_row_returns_the_option_to_its_default() {
        let set = with_option(&Arrangement::default(), "columns", " 4 ");
        assert_eq!(set.options.get("columns").map(String::as_str), Some("4"));
        let cleared = with_option(&set, "columns", "");
        assert!(cleared.options.is_empty());
    }
}
