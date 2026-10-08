// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What an arrangement's options are, declared as data (Scenograph editor
//! plan, ruling B, SE7 and SE43).
//!
//! [`Arrangement::options`](crate::Arrangement::options) is an open map of
//! strings. Each arrangement that reads it declares its options here: the key,
//! one of the seven kinds its readers parse, and the default. One declaration
//! drives both the refusal of an undeclared key and an editor's rows, so the
//! two cannot drift. A default the arrangement measures from the items is
//! described in words here; the arrangement can resolve the number for a
//! given set of items on demand.

use serde::{Deserialize, Serialize};

/// One option an arrangement reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionSpec {
    /// The key in [`Arrangement::options`](crate::Arrangement::options).
    pub key: String,
    /// A plain label for an editor's row.
    pub label: String,
    pub kind: OptionKind,
    pub default: OptionDefault,
}

impl OptionSpec {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        kind: OptionKind,
        default: OptionDefault,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            kind,
            default,
        }
    }
}

/// What an option's value must be: the seven kinds the readers parse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OptionKind {
    /// A finite number.
    Finite,
    /// A finite number above zero.
    Positive,
    /// A whole number above zero.
    Count,
    /// A whole number from 0 to 255.
    Depth,
    /// `true` or `false`.
    Flag,
    /// One of the named choices, in the order an editor offers them.
    Choice { names: Vec<String> },
    /// Comma-separated names.
    List,
}

impl OptionKind {
    /// Why `value` is not this kind, in the readers' words, or `None` when it is.
    pub fn refusal(&self, value: &str) -> Option<String> {
        let ok = match self {
            Self::Finite => value.parse::<f32>().is_ok_and(f32::is_finite),
            Self::Positive => value.parse::<f32>().is_ok_and(|v| v.is_finite() && v > 0.0),
            Self::Count => value.parse::<u32>().is_ok_and(|v| v > 0),
            Self::Depth => value.parse::<u8>().is_ok(),
            Self::Flag => matches!(value, "true" | "false"),
            Self::Choice { names } => names.iter().any(|name| name == value),
            Self::List => true,
        };
        (!ok).then(|| match self {
            Self::Finite => "needs a finite number".to_string(),
            Self::Positive => "needs a positive finite number".to_string(),
            Self::Count => "needs a positive whole number".to_string(),
            Self::Depth => "needs a whole number from 0 to 255".to_string(),
            Self::Flag => "needs true or false".to_string(),
            Self::Choice { names } => format!("needs one of {}", names.join(", ")),
            Self::List => unreachable!("a list accepts any text"),
        })
    }
}

/// What an option is when the author leaves it out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum OptionDefault {
    /// A fixed value, in the option's own spelling.
    Value(String),
    /// Measured from the items, described in plain words; the arrangement
    /// resolves the number for a given set of items.
    Measured(String),
    /// The arrangement decides.
    Auto,
    /// Nothing: an empty list.
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specs() -> Vec<OptionSpec> {
        vec![
            OptionSpec::new(
                "cell_width",
                "Cell width",
                OptionKind::Positive,
                OptionDefault::Measured("the largest item's width".into()),
            ),
            OptionSpec::new(
                "curve",
                "Curve",
                OptionKind::Choice {
                    names: vec!["square_root".into(), "linear".into()],
                },
                OptionDefault::Value("square_root".into()),
            ),
            OptionSpec::new("depth", "Depth", OptionKind::Depth, OptionDefault::Auto),
            OptionSpec::new(
                "column_order",
                "Column order",
                OptionKind::List,
                OptionDefault::Empty,
            ),
        ]
    }

    #[test]
    fn a_declaration_round_trips_and_serializes_the_same_way_twice() {
        let first = serde_json::to_string(&specs()).expect("serializes");
        let back: Vec<OptionSpec> = serde_json::from_str(&first).expect("deserializes");
        assert_eq!(back, specs());
        assert_eq!(serde_json::to_string(&back).unwrap(), first);
        assert!(first.contains(r#""kind":{"kind":"choice","names":["square_root","linear"]}"#));
        assert!(
            first.contains(r#""default":{"kind":"measured","value":"the largest item's width"}"#)
        );
        assert!(first.contains(r#""default":{"kind":"auto"}"#));
    }

    #[test]
    fn each_kind_refuses_in_the_readers_words() {
        assert_eq!(OptionKind::Finite.refusal("1.5"), None);
        assert_eq!(
            OptionKind::Finite.refusal("inf").as_deref(),
            Some("needs a finite number")
        );
        assert_eq!(
            OptionKind::Positive.refusal("0").as_deref(),
            Some("needs a positive finite number")
        );
        assert_eq!(
            OptionKind::Count.refusal("0").as_deref(),
            Some("needs a positive whole number")
        );
        assert_eq!(OptionKind::Depth.refusal("255"), None);
        assert_eq!(
            OptionKind::Depth.refusal("256").as_deref(),
            Some("needs a whole number from 0 to 255")
        );
        assert_eq!(
            OptionKind::Flag.refusal("yes").as_deref(),
            Some("needs true or false")
        );
        let choice = OptionKind::Choice {
            names: vec!["a".into(), "b".into()],
        };
        assert_eq!(choice.refusal("b"), None);
        assert_eq!(choice.refusal("c").as_deref(), Some("needs one of a, b"));
        assert_eq!(OptionKind::List.refusal("x, y"), None);
    }
}
