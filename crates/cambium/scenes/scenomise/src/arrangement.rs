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

/// What a row shows when its option is left out: the value, "auto", "none",
/// or a measured default as its number and its words when the items are known.
pub fn option_placeholder(spec: &OptionSpec, resolved: Option<&str>) -> String {
    match &spec.default {
        OptionDefault::Value(value) => value.clone(),
        OptionDefault::Auto => "auto".to_string(),
        OptionDefault::Empty => "none".to_string(),
        OptionDefault::Measured(words) => match resolved {
            Some(number) => format!("{number} ({words})"),
            None => words.clone(),
        },
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
    fn placeholders_show_the_default_and_measured_numbers() {
        let card = sceno::Size2::new(164.0, 68.0);
        let defaults = arrangement_defaults("grid", card, 9, 16.0);
        let width = Family::Grid
            .options()
            .into_iter()
            .find(|spec| spec.key == "cell_width")
            .unwrap();
        assert_eq!(
            option_placeholder(&width, defaults.get("cell_width").map(String::as_str)),
            "164 (the largest item's width)"
        );
        assert_eq!(option_placeholder(&width, None), "the largest item's width");
        let curve = Family::Spiral
            .options()
            .into_iter()
            .find(|s| s.key == "curve")
            .unwrap();
        assert_eq!(option_placeholder(&curve, None), "square_root");
        let depth = Family::LSystem
            .options()
            .into_iter()
            .find(|s| s.key == "depth")
            .unwrap();
        assert_eq!(option_placeholder(&depth, None), "auto");
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
