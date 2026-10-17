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

    pub(super) fn run(_: &str, _: &mut Engine<'_>) -> (bool, String) {
        (false, "script: no script engine".into())
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
    /// budget and sees only the globals listed on `RuleSet::resolve_with_scripts`.
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
        self.resolve_inner(context, None)
    }

    fn resolve_inner(
        &self,
        context: &Context,
        mut engine: scripts::Engine<'_>,
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
            } else {
                let previously_matched = context.previous_matches.contains(&rule.id);
                let bound = if contains_script(&rule.condition) {
                    scripts::bind(&mut engine, context, previously_matched)
                } else {
                    Ok(())
                };
                match bound {
                    Ok(()) => evaluate(&rule.condition, context, previously_matched, &mut engine),
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

fn issue(issues: &mut Vec<Issue>, field: &str, message: &str) {
    issues.push(Issue {
        field: field.into(),
        message: message.into(),
    });
}

fn validate_condition(
    condition: &Condition,
    path: &str,
    depth: usize,
    count: &mut usize,
    issues: &mut Vec<Issue>,
) {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_CONDITIONS {
        issue(
            issues,
            path,
            "condition complexity exceeds the 32-depth / 256-condition limit",
        );
        return;
    }
    match condition {
        Condition::HasFact { fact }
        | Condition::Equals { fact, .. }
        | Condition::Number { fact, .. }
            if fact.trim().is_empty() =>
        {
            issue(issues, path, "a fact name is required")
        },
        Condition::Equals { value, .. } if !value.finite() => {
            issue(issues, path, "condition values must be finite")
        },
        Condition::Number { value, .. } if !value.is_finite() => {
            issue(issues, path, "comparison values must be finite")
        },
        Condition::ScreenSize {
            enter_below,
            exit_above,
        } if !enter_below.is_finite()
            || !exit_above.is_finite()
            || *enter_below < 0.0
            || *enter_below >= *exit_above =>
        {
            issue(
                issues,
                path,
                "screen thresholds must be finite, nonnegative and enter < exit",
            )
        },
        Condition::All { conditions } | Condition::Any { conditions } => {
            if conditions.is_empty() {
                issue(issues, path, "compound conditions must not be empty");
            }
            for (index, child) in conditions.iter().enumerate() {
                if *count >= MAX_CONDITIONS {
                    issue(
                        issues,
                        path,
                        "at most 256 conditions per rule are supported",
                    );
                    break;
                }
                validate_condition(
                    child,
                    &format!("{path}.conditions[{index}]"),
                    depth + 1,
                    count,
                    issues,
                );
            }
        },
        Condition::Not { condition } => validate_condition(
            condition,
            &format!("{path}.condition"),
            depth + 1,
            count,
            issues,
        ),
        Condition::Script { source } if source.trim().is_empty() => {
            issue(issues, path, "a script condition needs source")
        },
        Condition::Script { source } if source.len() > MAX_SCRIPT_BYTES => {
            issue(issues, path, "script conditions are at most 4096 bytes")
        },
        _ => {},
    }
}

fn evaluate(
    condition: &Condition,
    context: &Context,
    previously_matched: bool,
    engine: &mut scripts::Engine<'_>,
) -> (bool, String) {
    match condition {
        Condition::Always => (true, "always".into()),
        Condition::State { state } => {
            let matched = context.states.contains(state);
            let label = serde_json::to_string(state).expect("state serialization");
            (
                matched,
                format!("state {}: {matched}", label.trim_matches('"')),
            )
        },
        Condition::HasFact { fact } => {
            let matched = context.facts.contains_key(fact);
            (matched, format!("has fact {fact}: {matched}"))
        },
        Condition::Equals { fact, value } => match context.facts.get(fact) {
            None => (false, format!("fact {fact}: missing")),
            Some(observed) => {
                let matched = observed == value;
                (
                    matched,
                    format!(
                        "fact {fact}: {} == {} -> {matched}",
                        observed.label(),
                        value.label()
                    ),
                )
            },
        },
        Condition::Number { fact, op, value } => match context.facts.get(fact) {
            None => (false, format!("fact {fact}: missing")),
            Some(Scalar::Number(observed)) => {
                let matched = op.compare(*observed, *value);
                (
                    matched,
                    format!(
                        "fact {fact}: {observed} {} {value} -> {matched}",
                        op.label()
                    ),
                )
            },
            Some(_) => (false, format!("fact {fact}: not numeric")),
        },
        Condition::ScreenSize {
            enter_below,
            exit_above,
        } => match context.screen_size {
            None => (false, "screen size: missing".into()),
            Some(size) => {
                let threshold = if previously_matched {
                    *exit_above
                } else {
                    *enter_below
                };
                let matched = if previously_matched {
                    size < threshold
                } else {
                    size <= threshold
                };
                let phase = if previously_matched { "exit" } else { "enter" };
                (
                    matched,
                    format!("screen size {size}, {phase} threshold {threshold}: {matched}"),
                )
            },
        },
        Condition::All { conditions } | Condition::Any { conditions } => {
            let results: Vec<_> = conditions
                .iter()
                .map(|child| evaluate(child, context, previously_matched, engine))
                .collect();
            let all = matches!(condition, Condition::All { .. });
            let matched = if all {
                results.iter().all(|result| result.0)
            } else {
                results.iter().any(|result| result.0)
            };
            let label = if all { "all" } else { "any" };
            (
                matched,
                format!(
                    "{label}: {}",
                    results
                        .iter()
                        .map(|result| result.1.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            )
        },
        Condition::Not { condition } => {
            let (matched, reason) = evaluate(condition, context, previously_matched, engine);
            (!matched, format!("not: {reason}"))
        },
        Condition::Script { source } => scripts::run(source, engine),
    }
}

fn contains_script(condition: &Condition) -> bool {
    match condition {
        Condition::Script { .. } => true,
        Condition::All { conditions } | Condition::Any { conditions } => {
            conditions.iter().any(contains_script)
        },
        Condition::Not { condition } => contains_script(condition),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "scripts")]
    use script_engine_api::{Budget, HostFunction, ScriptValue, ValueEngine, ValueError};

    fn context(id: &str) -> Context {
        Context {
            id: id.into(),
            kind: PrimitiveKind::Node,
            facts: BTreeMap::new(),
            states: BTreeSet::new(),
            screen_size: Some(100.0),
            previous_matches: BTreeSet::new(),
        }
    }
    fn rule(id: &str, priority: i32, condition: Condition, representation: Representation) -> Rule {
        Rule {
            id: id.into(),
            label: id.into(),
            enabled: true,
            priority,
            target: Target {
                kind: PrimitiveKind::Node,
                ids: BTreeSet::new(),
            },
            condition,
            effects: vec![Effect::Representation {
                value: representation,
            }],
        }
    }
    fn rules(rules: Vec<Rule>) -> RuleSet {
        RuleSet {
            version: PRESENTATION_RULE_VERSION,
            rules,
        }
    }

    #[test]
    fn priorities_ties_and_overridden_matches_are_explained() {
        let set = rules(vec![
            rule("base", 0, Condition::Always, Representation::Card),
            rule("high", 10, Condition::Always, Representation::Snapshot),
            rule("low-later", 2, Condition::Always, Representation::Glyph),
            rule("tie-later", 10, Condition::Always, Representation::LivePane),
        ]);
        let result = set.resolve(&context("a")).unwrap();
        assert_eq!(
            result.winners[&Property::Representation].rule_id,
            "tie-later"
        );
        assert_eq!(result.matched_rules.len(), 4);
        assert!(result.traces[1].matched && result.traces[1].winning_properties.is_empty());
        assert_eq!(
            result.traces[3].winning_properties,
            vec![Property::Representation]
        );
    }

    #[test]
    fn a_shared_resource_does_not_share_selection_or_occurrence_targets() {
        let mut a = context("tab-a");
        a.facts.insert(
            "resource".into(),
            Scalar::Text("https://example.org".into()),
        );
        let mut b = a.clone();
        b.id = "tab-b".into();
        a.states.insert(ViewState::Selected);
        let selected = rule(
            "selected",
            1,
            Condition::State {
                state: ViewState::Selected,
            },
            Representation::Card,
        );
        let mut instance = rule("one-access", 2, Condition::Always, Representation::LivePane);
        instance.target.ids.insert("tab-a".into());
        let set = rules(vec![selected, instance]);
        assert_eq!(set.resolve(&a).unwrap().matched_rules.len(), 2);
        assert!(set.resolve(&b).unwrap().winners.is_empty());
        assert!(!b.states.contains(&ViewState::Selected));
    }

    #[test]
    fn missing_facts_and_wrong_types_do_not_coerce_to_zero() {
        let set = rules(vec![rule(
            "count",
            0,
            Condition::Number {
                fact: "count".into(),
                op: Comparison::Eq,
                value: 0.0,
            },
            Representation::Card,
        )]);
        let mut item = context("a");
        assert_eq!(
            set.resolve(&item).unwrap().traces[0].reason,
            "fact count: missing"
        );
        item.facts.insert("count".into(), Scalar::Text("0".into()));
        assert_eq!(
            set.resolve(&item).unwrap().traces[0].reason,
            "fact count: not numeric"
        );
        item.facts.insert("count".into(), Scalar::Number(0.0));
        assert!(set.resolve(&item).unwrap().traces[0].matched);
    }

    #[test]
    fn compact_rungs_enter_hold_and_exit_without_moving_the_item() {
        let set = rules(vec![rule(
            "compact",
            5,
            Condition::ScreenSize {
                enter_below: 48.0,
                exit_above: 84.0,
            },
            Representation::Glyph,
        )]);
        let mut item = context("a");
        item.screen_size = Some(48.0);
        let first = set.resolve(&item).unwrap();
        assert!(first.traces[0].matched);
        item.previous_matches = first.matched_rules;
        item.screen_size = Some(70.0);
        assert!(set.resolve(&item).unwrap().traces[0].matched);
        item.screen_size = Some(84.0);
        assert!(!set.resolve(&item).unwrap().traces[0].matched);
        item.screen_size = None;
        assert!(!set.resolve(&item).unwrap().traces[0].matched);
    }

    #[test]
    fn malformed_sets_are_refused_before_any_effect_is_applied() {
        let mut bad = rule(
            "bad",
            0,
            Condition::ScreenSize {
                enter_below: 84.0,
                exit_above: 48.0,
            },
            Representation::Card,
        );
        bad.target.kind = PrimitiveKind::Field;
        bad.effects.push(Effect::Representation {
            value: Representation::Glyph,
        });
        let set = rules(vec![
            rule("valid", 0, Condition::Always, Representation::Card),
            bad,
        ]);
        let errors = set.resolve(&context("a")).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|issue| issue.message.contains("threshold"))
        );
        assert!(
            errors
                .iter()
                .any(|issue| issue.message.contains("incompatible"))
        );
        assert!(
            errors
                .iter()
                .any(|issue| issue.message.contains("only once"))
        );
    }

    #[test]
    fn storage_roundtrips_and_flags_are_independent() {
        let set = rules(vec![rule(
            "foreground",
            1,
            Condition::State {
                state: ViewState::ForegroundPinned,
            },
            Representation::Card,
        )]);
        let encoded = serde_json::to_string(&set).unwrap();
        let decoded: RuleSet = serde_json::from_str(&encoded).unwrap();
        assert_eq!(set, decoded);
        assert_eq!(encoded, serde_json::to_string(&decoded).unwrap());
        let mut item = context("a");
        item.states.insert(ViewState::PositionPinned);
        assert!(decoded.resolve(&item).unwrap().winners.is_empty());
        item.states.insert(ViewState::ForegroundPinned);
        assert!(!decoded.resolve(&item).unwrap().winners.is_empty());
    }

    #[test]
    fn duplicate_ids_disabled_rules_and_unbounded_conditions_are_checked() {
        let base = rule("base", 0, Condition::Always, Representation::Card);
        assert!(rules(vec![base.clone(), base.clone()]).validate().is_err());
        let mut disabled = base.clone();
        disabled.enabled = false;
        assert_eq!(
            rules(vec![disabled]).resolve(&context("a")).unwrap().traces[0].reason,
            "disabled"
        );
        let mut deep = Condition::Always;
        for _ in 0..40 {
            deep = Condition::Not {
                condition: Box::new(deep),
            };
        }
        assert!(
            rules(vec![rule("deep", 0, deep, Representation::Card)])
                .validate()
                .is_err()
        );
        let mut item = context("a");
        item.screen_size = Some(f64::NAN);
        assert!(rules(vec![base]).resolve(&item).is_err());
    }

    /// A scripted stand-in for an engine: the source names the outcome, and the
    /// globals and budgets it was given are kept for inspection.
    #[cfg(feature = "scripts")]
    #[derive(Default)]
    struct Scripted {
        globals: BTreeMap<String, ScriptValue>,
        budgets: Vec<Budget>,
        sources: Vec<String>,
    }

    #[cfg(feature = "scripts")]
    impl ValueEngine for Scripted {
        fn set_global(&mut self, name: &str, value: ScriptValue) -> Result<(), ValueError> {
            self.globals.insert(name.into(), value);
            Ok(())
        }
        fn register(&mut self, _: &str, _: usize, _: HostFunction) -> Result<(), ValueError> {
            Ok(())
        }
        fn eval(&mut self, source: &str, budget: Budget) -> Result<ScriptValue, ValueError> {
            self.budgets.push(budget);
            self.sources.push(source.into());
            match source {
                "yes" => Ok(ScriptValue::Bool(true)),
                "no" => Ok(ScriptValue::Bool(false)),
                "number" => Ok(ScriptValue::Int(1)),
                "over" => Err(ValueError::Budget),
                _ => Err(ValueError::Script(format!("cannot run {source}"))),
            }
        }
    }

    fn scripted(id: &str, priority: i32, source: &str, representation: Representation) -> Rule {
        rule(
            id,
            priority,
            Condition::Script {
                source: source.into(),
            },
            representation,
        )
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn a_script_condition_wins_by_the_same_precedence_as_any_other() {
        let set = rules(vec![
            rule("base", 0, Condition::Always, Representation::Card),
            scripted("by-script", 5, "yes", Representation::Sprite),
            scripted("quiet", 9, "no", Representation::Glyph),
        ]);
        let mut engine = Scripted::default();
        let resolved = set
            .resolve_with_scripts(&context("a"), &mut engine)
            .unwrap();
        assert_eq!(
            resolved.winners[&Property::Representation].rule_id,
            "by-script"
        );
        assert!(resolved.matched_rules.contains("by-script"));
        assert!(!resolved.matched_rules.contains("quiet"));
        assert_eq!(resolved.traces[1].reason, "script: true");
        // The budget is the fixed one, and only script rules reach the engine.
        assert_eq!(engine.sources, ["yes", "no"]);
        assert!(
            engine
                .budgets
                .iter()
                .all(|b| *b == Budget::Steps(SCRIPT_STEP_BUDGET))
        );
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn the_script_sees_the_occurrence() {
        let mut item = context("node-7");
        item.facts
            .insert("mime".into(), Scalar::Text("image/png".into()));
        item.facts.insert("weight".into(), Scalar::Number(2.5));
        item.facts.insert("pinned".into(), Scalar::Bool(true));
        item.states.insert(ViewState::Selected);
        item.states.insert(ViewState::Overview);
        item.previous_matches.insert("s".into());
        let mut engine = Scripted::default();
        rules(vec![scripted("s", 0, "yes", Representation::Card)])
            .resolve_with_scripts(&item, &mut engine)
            .unwrap();
        let text = |s: &str| ScriptValue::Str(s.into());
        assert_eq!(engine.globals["id"], text("node-7"));
        assert_eq!(engine.globals["kind"], text("node"));
        assert_eq!(
            engine.globals["facts"],
            ScriptValue::Map(BTreeMap::from([
                ("mime".to_string(), text("image/png")),
                ("pinned".to_string(), ScriptValue::Bool(true)),
                ("weight".to_string(), ScriptValue::Float(2.5)),
            ]))
        );
        assert_eq!(
            engine.globals["states"],
            ScriptValue::List(vec![text("selected"), text("overview")])
        );
        assert_eq!(engine.globals["screen_size"], ScriptValue::Float(100.0));
        assert_eq!(
            engine.globals["previously_matched"],
            ScriptValue::Bool(true)
        );
        item.screen_size = None;
        item.previous_matches.clear();
        rules(vec![scripted("s", 0, "yes", Representation::Card)])
            .resolve_with_scripts(&item, &mut engine)
            .unwrap();
        assert_eq!(engine.globals["screen_size"], ScriptValue::Unit);
        assert_eq!(
            engine.globals["previously_matched"],
            ScriptValue::Bool(false)
        );
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn a_failing_script_leaves_its_rule_unmatched_and_the_rest_resolve() {
        let set = rules(vec![
            scripted("broken", 9, "boom", Representation::Glyph),
            scripted("slow", 8, "over", Representation::Glyph),
            scripted("wrong", 7, "number", Representation::Glyph),
            rule("base", 0, Condition::Always, Representation::Card),
        ]);
        let resolved = set
            .resolve_with_scripts(&context("a"), &mut Scripted::default())
            .unwrap();
        assert_eq!(resolved.winners[&Property::Representation].rule_id, "base");
        let reasons: Vec<_> = resolved.traces.iter().map(|t| t.reason.as_str()).collect();
        assert_eq!(reasons[0], "script: script error: cannot run boom");
        assert_eq!(reasons[1], "script: the script ran past its budget");
        assert_eq!(reasons[2], "script: returned int, wanted a bool");
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn scripts_compose_inside_all_any_and_not() {
        let set = rules(vec![rule(
            "r",
            0,
            Condition::All {
                conditions: vec![
                    Condition::Always,
                    Condition::Not {
                        condition: Box::new(Condition::Script {
                            source: "no".into(),
                        }),
                    },
                ],
            },
            Representation::Card,
        )]);
        let resolved = set
            .resolve_with_scripts(&context("a"), &mut Scripted::default())
            .unwrap();
        assert!(resolved.matched_rules.contains("r"));
    }

    #[test]
    fn without_an_engine_a_set_with_a_script_is_refused() {
        let set = rules(vec![
            rule("base", 0, Condition::Always, Representation::Card),
            scripted("s", 1, "yes", Representation::Sprite),
        ]);
        let issues = set.resolve(&context("a")).unwrap_err();
        assert_eq!(issues[0].field, "rules[1].condition");
        assert!(issues[0].message.contains("script engine"));
        // A set without one resolves as it always did.
        assert!(
            rules(vec![rule("b", 0, Condition::Always, Representation::Card)])
                .resolve(&context("a"))
                .is_ok()
        );
    }

    #[test]
    fn script_conditions_validate_and_round_trip() {
        assert!(
            rules(vec![scripted("s", 0, "  ", Representation::Card)])
                .validate()
                .is_err()
        );
        let long = "1".repeat(MAX_SCRIPT_BYTES + 1);
        assert!(
            rules(vec![scripted("s", 0, &long, Representation::Card)])
                .validate()
                .is_err()
        );
        let set = rules(vec![scripted(
            "s",
            0,
            "facts.weight > 2.0",
            Representation::Card,
        )]);
        assert!(set.validate().is_ok());
        let json = serde_json::to_string(&set).unwrap();
        assert!(json.contains(r#""kind":"script""#));
        assert_eq!(serde_json::from_str::<RuleSet>(&json).unwrap(), set);
    }
}
