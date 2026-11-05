// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Target → condition → effect presentation rules (Scenograph editor R1).
//!
//! The host supplies disclosed facts and occurrence-local view state. Resolve
//! these decisions before measuring representations and arranging the scene.
//! Effects never mutate source authority, coordinates, pins or selection.
//! The host fills the existing `sceno::Representation` slots, implements relation
//! forms and overlays, and keeps hysteresis matches per view and occurrence.
//! Conditions are declarative, plus `Condition::Script` (SE92). The condition
//! is data and always present, so a rule set with a script validates and
//! round-trips in any host. Running one needs the `scripts` feature, which adds
//! the `ValueEngine` contract (Genet's `script-engine-api`) and
//! `RuleSet::resolve_with_scripts`; without it `resolve` refuses such a set.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "editor")]
pub mod editor;

use sceno::Representation;
use serde::{Deserialize, Serialize};

pub const PRESENTATION_RULE_VERSION: u16 = 1;
const MAX_RULES: usize = 256;
const MAX_CONDITIONS: usize = 256;
const MAX_DEPTH: usize = 32;
const MAX_SCRIPT_BYTES: usize = 4096;

#[cfg(feature = "scripts")]
mod scripts;
#[cfg(feature = "scripts")]
pub use scripts::SCRIPT_STEP_BUDGET;

/// Without the `scripts` feature there is no engine to run: the same entry
/// points as `scripts.rs`, doing nothing.
#[cfg(not(feature = "scripts"))]
mod scripts {
    use super::Context;

    pub(super) type Engine<'a> = Option<&'a mut ()>;

    pub(super) fn bind(_: &mut Engine<'_>, _: &Context, _: bool) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn run(_: &str, _: &mut Engine<'_>) -> Result<(bool, String), String> {
        Err("script: no script engine".into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    Node,
    Relation,
    Field,
}

/// IDs identify projected occurrences, not shared source resources.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub kind: PrimitiveKind,
    /// Empty means every occurrence of this kind.
    #[serde(default)]
    pub ids: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Scalar {
    Bool(bool),
    Number(f64),
    Text(String),
}

impl Scalar {
    fn label(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::Text(value) => serde_json::to_string(value).expect("string serialization"),
        }
    }
    fn finite(&self) -> bool {
        !matches!(self, Self::Number(value) if !value.is_finite())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewState {
    Selected,
    ForegroundPinned,
    PositionPinned,
    LayoutUnlocked,
    Inspected,
    Overview,
    /// Was forward, then deselected; stays until dismissed (SE91). Supplied
    /// by [`crate::attention::AttentionBook`].
    Retained,
    /// Offered by the scene rather than the person's content.
    Suggested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    Lt,
    Lte,
    Eq,
    Gte,
    Gt,
}

impl Comparison {
    fn compare(self, left: f64, right: f64) -> bool {
        match self {
            Self::Lt => left < right,
            Self::Lte => left <= right,
            Self::Eq => left == right,
            Self::Gte => left >= right,
            Self::Gt => left > right,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Eq => "eq",
            Self::Gte => "gte",
            Self::Gt => "gt",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Condition {
    Always,
    State {
        state: ViewState,
    },
    HasFact {
        fact: String,
    },
    Equals {
        fact: String,
        value: Scalar,
    },
    Number {
        fact: String,
        op: Comparison,
        value: f64,
    },
    /// Enter compact at or below `enter_below`; exit at or above `exit_above`.
    /// The gap prevents flicker. Missing measurements never match.
    ScreenSize {
        enter_below: f64,
        exit_above: f64,
    },
    All {
        conditions: Vec<Condition>,
    },
    Any {
        conditions: Vec<Condition>,
    },
    Not {
        condition: Box<Condition>,
    },
    /// A script that returns a bool (Scenograph editor plan, SE92). With the
    /// `scripts` feature it runs through a `ValueEngine` under a fixed step
    /// budget with the occurrence globals listed on `RuleSet::resolve_with_scripts`
    /// and the functions registered by the host.
    /// A script that fails, runs past its budget or returns anything else leaves
    /// its rule unmatched, with the reason in the trace.
    Script {
        source: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationForm {
    Stroke,
    Containment,
    Adjacency,
    MatrixCell,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    Representation { value: Representation },
    Details { visible: bool },
    RelationForm { value: RelationForm },
    Provenance { visible: bool },
    RegionGuides { visible: bool },
    DropPreviews { visible: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Property {
    Representation,
    Details,
    RelationForm,
    Provenance,
    RegionGuides,
    DropPreviews,
}

impl Effect {
    pub fn property(&self) -> Property {
        match self {
            Self::Representation { .. } => Property::Representation,
            Self::Details { .. } => Property::Details,
            Self::RelationForm { .. } => Property::RelationForm,
            Self::Provenance { .. } => Property::Provenance,
            Self::RegionGuides { .. } => Property::RegionGuides,
            Self::DropPreviews { .. } => Property::DropPreviews,
        }
    }
    /// These decisions must be resolved before representation measurement.
    pub fn affects_measurement(&self) -> bool {
        matches!(self, Self::Representation { .. } | Self::Details { .. })
    }
    fn target_kind(&self) -> PrimitiveKind {
        match self {
            Self::Representation { .. } | Self::Details { .. } => PrimitiveKind::Node,
            Self::RelationForm { .. } | Self::Provenance { .. } => PrimitiveKind::Relation,
            Self::RegionGuides { .. } | Self::DropPreviews { .. } => PrimitiveKind::Field,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    /// Higher priority wins; the later authored rule wins an equal-priority tie.
    pub priority: i32,
    pub target: Target,
    pub condition: Condition,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleSet {
    pub version: u16,
    pub rules: Vec<Rule>,
}

/// All view state is scoped to this occurrence in one host view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub kind: PrimitiveKind,
    #[serde(default)]
    pub facts: BTreeMap<String, Scalar>,
    #[serde(default)]
    pub states: BTreeSet<ViewState>,
    /// Measured detail size in screen pixels, supplied before choosing the rung.
    pub screen_size: Option<f64>,
    /// Prior matching rule IDs for this view and occurrence only.
    #[serde(default)]
    pub previous_matches: BTreeSet<String>,
}

/// Inputs that a host has actually disclosed for one rule context.
///
/// A missing state is not necessarily false: hosts may not have a view input
/// for every state. Screen measurements are likewise unavailable until the
/// host supplies one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputAvailability {
    pub states: BTreeSet<ViewState>,
    pub screen_size: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub rule_id: String,
    pub priority: i32,
    pub effect: Effect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub rule_id: String,
    pub matched: bool,
    pub reason: String,
    /// Empty on a matched rule means its properties were overridden.
    pub winning_properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    /// Absent properties fall back to the host/scene's base presentation.
    pub winners: BTreeMap<Property, Decision>,
    pub traces: Vec<Trace>,
    /// Feed back only to the same view and occurrence.
    pub matched_rules: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub field: String,
    pub message: String,
}

impl RuleSet {
    /// Validate atomically, including disabled rules so enabling one is safe.
    pub fn validate(&self) -> Result<(), Vec<Issue>> {
        let mut issues = Vec::new();
        if self.version != PRESENTATION_RULE_VERSION {
            issue(
                &mut issues,
                "version",
                "unsupported presentation rule version",
            );
        }
        if self.rules.len() > MAX_RULES {
            issue(&mut issues, "rules", "at most 256 rules are supported");
            return Err(issues);
        }
        let mut ids = BTreeSet::new();
        for (index, rule) in self.rules.iter().enumerate() {
            let path = format!("rules[{index}]");
            if rule.id.trim().is_empty() || !ids.insert(&rule.id) {
                issue(
                    &mut issues,
                    &format!("{path}.id"),
                    "rule IDs must be nonempty and unique",
                );
            }
            if rule.target.ids.iter().any(|id| id.trim().is_empty()) {
                issue(
                    &mut issues,
                    &format!("{path}.target.ids"),
                    "occurrence IDs must be nonempty",
                );
            }
            validate_condition(
                &rule.condition,
                &format!("{path}.condition"),
                0,
                &mut 0,
                &mut issues,
            );
            let mut properties = BTreeSet::new();
            if rule.effects.is_empty() {
                issue(
                    &mut issues,
                    &format!("{path}.effects"),
                    "a rule must declare an effect",
                );
            }
            for (effect_index, effect) in rule.effects.iter().enumerate() {
                let effect_path = format!("{path}.effects[{effect_index}]");
                if effect.target_kind() != rule.target.kind {
                    issue(
                        &mut issues,
                        &effect_path,
                        "effect is incompatible with target kind",
                    );
                }
                if !properties.insert(effect.property()) {
                    issue(
                        &mut issues,
                        &effect_path,
                        "a rule may set each property only once",
                    );
                }
                if matches!(effect, Effect::Representation { value: Representation::Open { kind } } if kind.trim().is_empty())
                {
                    issue(
                        &mut issues,
                        &effect_path,
                        "open representations need a host-recognized kind",
                    );
                }
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Resolve with no script engine. A set that carries a script condition is
    /// refused; the `scripts` feature adds `resolve_with_scripts`.
    pub fn resolve(&self, context: &Context) -> Result<Resolution, Vec<Issue>> {
        self.resolve_inner(context, None, None)
    }

    /// Resolve after suppressing any matching rule that depends on an
    /// unavailable host input. Unknown inputs suppress the whole rule,
    /// including under `Not` and `Any`; a disclosed false state remains a
    /// known false value in `context.states`.
    pub fn resolve_with_availability(
        &self,
        context: &Context,
        availability: &InputAvailability,
    ) -> Result<Resolution, Vec<Issue>> {
        self.resolve_inner(context, None, Some(availability))
    }

    fn resolve_inner(
        &self,
        context: &Context,
        mut engine: scripts::Engine<'_>,
        availability: Option<&InputAvailability>,
    ) -> Result<Resolution, Vec<Issue>> {
        self.validate()?;
        let mut issues = Vec::new();
        if engine.is_none() {
            for (index, rule) in self.rules.iter().enumerate() {
                if contains_script(&rule.condition) {
                    issue(
                        &mut issues,
                        &format!("rules[{index}].condition"),
                        "a script condition needs a script engine",
                    );
                }
            }
        }
        if context.id.trim().is_empty() {
            issue(&mut issues, "context.id", "an occurrence ID is required");
        }
        if context
            .screen_size
            .is_some_and(|size| !size.is_finite() || size < 0.0)
        {
            issue(
                &mut issues,
                "context.screen_size",
                "screen size must be finite and nonnegative",
            );
        }
        for (fact, value) in &context.facts {
            if !value.finite() {
                issue(
                    &mut issues,
                    &format!("context.facts.{fact}"),
                    "numeric facts must be finite",
                );
            }
        }
        if !issues.is_empty() {
            return Err(issues);
        }
        let mut result = Resolution {
            winners: BTreeMap::new(),
            traces: Vec::new(),
            matched_rules: BTreeSet::new(),
        };
        for rule in &self.rules {
            let (matched, reason) = if !rule.enabled {
                (false, "disabled".into())
            } else if rule.target.kind != context.kind
                || (!rule.target.ids.is_empty() && !rule.target.ids.contains(&context.id))
            {
                (false, "target does not include this occurrence".into())
            } else if let Some(reason) = availability
                .and_then(|availability| availability.unavailable(&rule.condition, context))
            {
                (
                    false,
                    format!("Host input unavailable: {reason}; rule skipped"),
                )
            } else {
                let previously_matched = context.previous_matches.contains(&rule.id);
                let bound = if contains_script(&rule.condition) {
                    scripts::bind(&mut engine, context, previously_matched)
                } else {
                    Ok(())
                };
                match bound {
                    Ok(()) => evaluate(&rule.condition, context, previously_matched, &mut engine)
                        .unwrap_or_else(|reason| (false, reason)),
                    Err(error) => (
                        false,
                        format!("script: could not bind the occurrence: {error}"),
                    ),
                }
            };
            if matched {
                result.matched_rules.insert(rule.id.clone());
                for effect in &rule.effects {
                    let key = effect.property();
                    if result
                        .winners
                        .get(&key)
                        .is_none_or(|winner| rule.priority >= winner.priority)
                    {
                        result.winners.insert(
                            key,
                            Decision {
                                rule_id: rule.id.clone(),
                                priority: rule.priority,
                                effect: effect.clone(),
                            },
                        );
                    }
                }
            }
            result.traces.push(Trace {
                rule_id: rule.id.clone(),
                matched,
                reason,
                winning_properties: Vec::new(),
            });
        }
        for trace in &mut result.traces {
            trace.winning_properties = result
                .winners
                .iter()
                .filter_map(|(key, winner)| (winner.rule_id == trace.rule_id).then_some(*key))
                .collect();
        }
        Ok(result)
    }
}

impl InputAvailability {
    fn unavailable(&self, condition: &Condition, context: &Context) -> Option<String> {
        match condition {
            Condition::State { state } if !self.states.contains(state) => {
                Some(format!("view state {state:?} is unavailable"))
            },
            Condition::ScreenSize { .. } if !self.screen_size || context.screen_size.is_none() => {
                Some("screen measurement is unavailable".into())
            },
            Condition::All { conditions } | Condition::Any { conditions } => conditions
                .iter()
                .find_map(|condition| self.unavailable(condition, context)),
            Condition::Not { condition } => self.unavailable(condition, context),
            _ => None,
        }
    }
}

fn issue(issues: &mut Vec<Issue>, field: &str, message: &str) {
    issues.push(Issue {
        field: field.into(),
        message: message.into(),
    });
}

#[cfg(test)]
#[path = "presentation/tests.rs"]
mod tests;

#[path = "presentation/conditions.rs"]
mod conditions;

use conditions::validate_condition;

use conditions::evaluate;

use conditions::contains_script;
