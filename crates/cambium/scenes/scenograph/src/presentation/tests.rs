// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

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
fn unavailable_inputs_suppress_whole_rules_and_preserve_known_false() {
    let available = InputAvailability {
        states: BTreeSet::from([ViewState::Selected]),
        screen_size: false,
    };
    let set = rules(vec![
        rule(
            "unknown-not",
            10,
            Condition::Not {
                condition: Box::new(Condition::State {
                    state: ViewState::ForegroundPinned,
                }),
            },
            Representation::Glyph,
        ),
        rule(
            "unknown-any",
            9,
            Condition::Any {
                conditions: vec![
                    Condition::Always,
                    Condition::State {
                        state: ViewState::Overview,
                    },
                ],
            },
            Representation::Glyph,
        ),
        rule(
            "unknown-measurement",
            8,
            Condition::ScreenSize {
                enter_below: 24.0,
                exit_above: 48.0,
            },
            Representation::Glyph,
        ),
        rule(
            "known-false",
            0,
            Condition::Not {
                condition: Box::new(Condition::State {
                    state: ViewState::Selected,
                }),
            },
            Representation::Card,
        ),
    ]);
    let resolved = set
        .resolve_with_availability(&context("a"), &available)
        .unwrap();
    assert_eq!(
        resolved.winners[&Property::Representation].rule_id,
        "known-false"
    );
    assert!(
        resolved.traces[..3]
            .iter()
            .all(|trace| { !trace.matched && trace.reason.contains("unavailable") })
    );
    assert!(resolved.traces[3].matched);
}

#[test]
fn unavailable_input_does_not_hide_invalid_rules() {
    let mut malformed = rule(
        "invalid",
        0,
        Condition::State {
            state: ViewState::Overview,
        },
        Representation::Card,
    );
    malformed.effects.clear();
    let error = rules(vec![malformed])
        .resolve_with_availability(&context("a"), &InputAvailability::default())
        .unwrap_err();
    assert_eq!(error[0].field, "rules[0].effects");
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
