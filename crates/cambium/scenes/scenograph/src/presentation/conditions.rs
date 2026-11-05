// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn validate_condition(
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

pub(super) fn evaluate(
    condition: &Condition,
    context: &Context,
    previously_matched: bool,
    engine: &mut scripts::Engine<'_>,
) -> Result<(bool, String), String> {
    let result = match condition {
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
            // Evaluate every child for the same engine/budget behavior as before,
            // but keep failures distinct from a genuine false condition.
            let results = results.into_iter().collect::<Result<Vec<_>, _>>()?;
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
            let (matched, reason) = evaluate(condition, context, previously_matched, engine)?;
            (!matched, format!("not: {reason}"))
        },
        Condition::Script { source } => return scripts::run(source, engine),
    };
    Ok(result)
}

pub(super) fn contains_script(condition: &Condition) -> bool {
    match condition {
        Condition::Script { .. } => true,
        Condition::All { conditions } | Condition::Any { conditions } => {
            conditions.iter().any(contains_script)
        },
        Condition::Not { condition } => contains_script(condition),
        _ => false,
    }
}
