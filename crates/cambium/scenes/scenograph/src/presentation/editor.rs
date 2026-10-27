// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-neutral authoring state for presentation rule sets.
//!
//! This module owns private drafts and saved undo/redo transitions. Hosts own
//! serialization, storage, graph identity, and acknowledgement delivery.
use super::{
    Condition, Context, Effect, Issue, PRESENTATION_RULE_VERSION, PrimitiveKind, Resolution, Rule,
    RuleSet, Target,
};
use backstep::History;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::sync::atomic::{AtomicU32, Ordering};

const RECORD_LIMIT: usize = 1024 * 1024;
const HISTORY_LIMIT: usize = 32;

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("validated presentation data serializes")
}

/// Refuse unknown JSON properties instead of silently discarding authored input.
fn unknown(input: &Value, known: &Value, path: &str, issues: &mut Vec<Issue>) {
    match (input, known) {
        (Value::Object(input), Value::Object(known)) => {
            for (key, value) in input {
                let path = format!("{path}.{key}");
                if let Some(known) = known.get(key) {
                    unknown(value, known, &path, issues);
                } else {
                    issues.push(Issue {
                        field: path,
                        message: "unknown property".into(),
                    });
                }
            }
        },
        (Value::Array(input), Value::Array(known)) => {
            for (index, (input, known)) in input.iter().zip(known).enumerate() {
                unknown(input, known, &format!("{path}[{index}]"), issues);
            }
        },
        _ => {},
    }
}

pub fn parse<T: DeserializeOwned + Serialize>(text: &str, path: &str) -> Result<T, Vec<Issue>> {
    let invalid = |message: String| {
        vec![Issue {
            field: path.into(),
            message,
        }]
    };
    if text.len() > RECORD_LIMIT {
        return Err(invalid("input exceeds 1 MiB".into()));
    }
    let input: Value = serde_json::from_str(text).map_err(|e| invalid(e.to_string()))?;
    let value: T = serde_json::from_value(input.clone()).map_err(|e| invalid(e.to_string()))?;
    let mut issues = Vec::new();
    unknown(
        &input,
        &serde_json::to_value(&value).map_err(|e| invalid(e.to_string()))?,
        path,
        &mut issues,
    );
    if issues.is_empty() {
        Ok(value)
    } else {
        Err(issues)
    }
}

/// Raw fields stay editable and undoable while malformed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDraft {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub priority: String,
    pub target: String,
    pub condition: String,
    pub effects: String,
}
impl From<&Rule> for RuleDraft {
    fn from(rule: &Rule) -> Self {
        Self {
            id: rule.id.clone(),
            label: rule.label.clone(),
            enabled: rule.enabled,
            priority: rule.priority.to_string(),
            target: json(&rule.target),
            condition: json(&rule.condition),
            effects: json(&rule.effects),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RuleSetDraft {
    pub rules: Vec<RuleDraft>,
}
impl From<&RuleSet> for RuleSetDraft {
    fn from(rules: &RuleSet) -> Self {
        Self {
            rules: rules.rules.iter().map(RuleDraft::from).collect(),
        }
    }
}
impl RuleSetDraft {
    pub fn validate(&self) -> Result<RuleSet, Vec<Issue>> {
        if self.rules.len() > 256 {
            return Err(vec![Issue {
                field: "rules".into(),
                message: "at most 256 rules are supported".into(),
            }]);
        }
        let mut issues = Vec::new();
        let mut rules = Vec::new();
        for (index, draft) in self.rules.iter().enumerate() {
            let path = format!("rules[{index}]");
            let priority = draft.priority.trim().parse::<i32>().map_err(|_| {
                vec![Issue {
                    field: format!("{path}.priority"),
                    message: "priority must be a signed 32-bit integer".into(),
                }]
            });
            let target = parse::<Target>(&draft.target, &format!("{path}.target"));
            let condition = parse::<Condition>(&draft.condition, &format!("{path}.condition"));
            let effects = parse::<Vec<Effect>>(&draft.effects, &format!("{path}.effects"));
            match (priority, target, condition, effects) {
                (Ok(priority), Ok(target), Ok(condition), Ok(effects)) => rules.push(Rule {
                    id: draft.id.clone(),
                    label: draft.label.clone(),
                    enabled: draft.enabled,
                    priority,
                    target,
                    condition,
                    effects,
                }),
                (priority, target, condition, effects) => {
                    for errors in [priority.err(), target.err(), condition.err(), effects.err()]
                        .into_iter()
                        .flatten()
                    {
                        issues.extend(errors);
                    }
                },
            }
        }
        if !issues.is_empty() {
            return Err(issues);
        }
        let rules = RuleSet {
            version: PRESENTATION_RULE_VERSION,
            rules,
        };
        rules.validate()?;
        Ok(rules)
    }
    pub fn add(&mut self, kind: PrimitiveKind) -> usize {
        let prefix = match kind {
            PrimitiveKind::Node => "node",
            PrimitiveKind::Relation => "link",
            PrimitiveKind::Field => "field",
        };
        let mut number = 1;
        let id = loop {
            let candidate = format!("{prefix}-{number}");
            if !self.rules.iter().any(|r| r.id == candidate) {
                break candidate;
            }
            number += 1;
        };
        let effect = match kind {
            PrimitiveKind::Node => Effect::Details { visible: true },
            PrimitiveKind::Relation => Effect::Provenance { visible: true },
            PrimitiveKind::Field => Effect::RegionGuides { visible: true },
        };
        self.rules.push(RuleDraft::from(&Rule {
            id,
            label: format!("New {prefix} rule"),
            enabled: true,
            priority: 0,
            target: Target {
                kind,
                ids: Default::default(),
            },
            condition: Condition::Always,
            effects: vec![effect],
        }));
        self.rules.len() - 1
    }
}

/// Durable state needed by an adapter to serialize an acknowledged save.
#[derive(Clone, Debug, PartialEq)]
pub struct SavedState {
    pub generation: u64,
    pub rules: RuleSet,
    pub undo: Vec<RuleSet>,
    pub redo: Vec<RuleSet>,
}
impl SavedState {
    pub fn empty() -> Self {
        Self {
            generation: 0,
            rules: RuleSet {
                version: PRESENTATION_RULE_VERSION,
                rules: Vec::new(),
            },
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.undo.len() > HISTORY_LIMIT || self.redo.len() > HISTORY_LIMIT {
            return Err("saved history exceeds 32 steps".into());
        }
        for rules in std::iter::once(&self.rules)
            .chain(&self.undo)
            .chain(&self.redo)
        {
            rules
                .validate()
                .map_err(|e| format!("Invalid saved presentation rules: {e:?}"))?;
        }
        Ok(())
    }
}

static NEXT_WORKSPACE: AtomicU32 = AtomicU32::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationId {
    workspace: u32,
    sequence: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveKind {
    Apply,
    Undo,
    Redo,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedOperation {
    pub id: OperationId,
    pub state: SavedState,
}

struct Draft {
    value: RuleSetDraft,
    history: History<RuleSetDraft, String>,
}
pub struct RuleWorkspace {
    identity: u32,
    saved: SavedState,
    draft: Option<Draft>,
    pending: Option<PreparedOperation>,
    next_operation: u64,
}
impl RuleWorkspace {
    pub fn new(saved: SavedState) -> Result<Self, String> {
        saved.validate()?;
        let identity = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        assert_ne!(identity, 0, "presentation workspace identity exhausted");
        Ok(Self {
            identity,
            saved,
            draft: None,
            pending: None,
            next_operation: 0,
        })
    }
    pub fn committed(&self) -> &RuleSet {
        &self.saved.rules
    }
    pub fn saved_state(&self) -> &SavedState {
        &self.saved
    }
    pub fn draft(&self) -> Option<&RuleSetDraft> {
        self.draft.as_ref().map(|d| &d.value)
    }
    pub fn editing(&self) -> bool {
        self.draft.is_some()
    }
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub fn dirty(&self) -> bool {
        self.draft()
            .is_some_and(|d| d != &RuleSetDraft::from(&self.saved.rules))
    }
    pub fn generation(&self) -> u64 {
        self.saved.generation
    }
    pub fn can_undo(&self) -> bool {
        !self.busy()
            && self
                .draft
                .as_ref()
                .map_or(!self.saved.undo.is_empty(), |d| d.history.can_undo())
    }
    pub fn can_redo(&self) -> bool {
        !self.busy()
            && self
                .draft
                .as_ref()
                .map_or(!self.saved.redo.is_empty(), |d| d.history.can_redo())
    }
    fn idle(&self) -> Result<(), String> {
        if self.busy() {
            Err("Wait for the presentation save to finish".into())
        } else {
            Ok(())
        }
    }
    pub fn begin(&mut self) -> Result<(), String> {
        self.idle()?;
        if self.draft.is_none() {
            self.draft = Some(Draft {
                value: RuleSetDraft::from(&self.saved.rules),
                history: History::new().with_cap(HISTORY_LIMIT).with_window(400),
            });
        }
        Ok(())
    }
    pub fn edit(
        &mut self,
        key: Option<String>,
        now_ms: u64,
        change: impl FnOnce(&mut RuleSetDraft),
    ) -> Result<(), String> {
        self.idle()?;
        let draft = self.draft.as_mut().ok_or("Begin a rule draft first")?;
        let mut next = draft.value.clone();
        change(&mut next);
        let size: usize = next
            .rules
            .iter()
            .map(|r| {
                r.id.len()
                    + r.label.len()
                    + r.priority.len()
                    + r.target.len()
                    + r.condition.len()
                    + r.effects.len()
            })
            .sum();
        if next.rules.len() > 256 || size > RECORD_LIMIT {
            return Err("rule draft exceeds its 256-rule / 1 MiB limit".into());
        }
        if next != draft.value {
            draft.history.record(draft.value.clone(), key, now_ms);
            draft.value = next;
        }
        Ok(())
    }
    pub fn undo_draft(&mut self, redo: bool) -> Result<(), String> {
        self.idle()?;
        let draft = self.draft.as_mut().ok_or("No rule draft is open")?;
        let next = if redo {
            draft.history.redo(draft.value.clone())
        } else {
            draft.history.undo(draft.value.clone())
        };
        if let Some(next) = next {
            draft.value = next;
        }
        Ok(())
    }
    pub fn discard(&mut self) -> Result<(), String> {
        self.idle()?;
        self.draft = None;
        Ok(())
    }
    pub fn rules(&self) -> Result<RuleSet, Vec<Issue>> {
        self.draft()
            .map_or_else(|| Ok(self.saved.rules.clone()), RuleSetDraft::validate)
    }
    pub fn preview(&self, context: &Context) -> Result<Resolution, Vec<Issue>> {
        self.rules()?.resolve(context)
    }
    pub fn prepare(&mut self, kind: SaveKind) -> Result<PreparedOperation, String> {
        self.idle()?;
        let mut next = self.saved.clone();
        match kind {
            SaveKind::Apply => {
                let rules = self
                    .draft()
                    .ok_or("No rule draft is open")?
                    .validate()
                    .map_err(|e| format!("Rule draft refused: {e:?}"))?;
                if rules == next.rules {
                    return Err("No rule changes to apply".into());
                }
                next.undo.push(next.rules.clone());
                if next.undo.len() > HISTORY_LIMIT {
                    next.undo.remove(0);
                }
                next.redo.clear();
                next.rules = rules;
            },
            SaveKind::Undo | SaveKind::Redo => {
                if self.editing() {
                    return Err("Discard or apply the draft before saved undo".into());
                }
                let (from, to) = if kind == SaveKind::Undo {
                    (&mut next.undo, &mut next.redo)
                } else {
                    (&mut next.redo, &mut next.undo)
                };
                let rules = from.pop().ok_or("No saved rule step is available")?;
                to.push(next.rules.clone());
                next.rules = rules;
            },
        }
        next.generation = next
            .generation
            .checked_add(1)
            .ok_or("presentation generation exhausted")?;
        next.validate()?;
        self.next_operation = self
            .next_operation
            .checked_add(1)
            .ok_or("presentation operation sequence exhausted")?;
        let operation = PreparedOperation {
            id: OperationId {
                workspace: self.identity,
                sequence: self.next_operation,
            },
            state: next,
        };
        self.pending = Some(operation.clone());
        Ok(operation)
    }
    /// A completion promotes its prepared snapshot only when its operation id
    /// still matches. Failure or a stale id leaves committed state and draft intact.
    pub fn finish(&mut self, id: OperationId, result: Result<(), String>) -> Result<bool, String> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(false);
        };
        if pending.id != id {
            return Ok(false);
        }
        let pending = self.pending.take().expect("checked pending operation");
        result?;
        self.saved = pending.state;
        self.draft = None;
        Ok(true)
    }
    /// Replace the committed baseline from a freshly loaded record while
    /// retaining the private draft for explicit review and re-application.
    pub fn reconcile(&mut self, saved: SavedState) -> Result<(), String> {
        self.idle()?;
        saved.validate()?;
        self.saved = saved;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{Property, ViewState};

    fn context(id: &str) -> Context {
        Context {
            id: id.into(),
            kind: PrimitiveKind::Node,
            facts: Default::default(),
            states: Default::default(),
            screen_size: Some(64.0),
            previous_matches: Default::default(),
        }
    }
    fn add(workspace: &mut RuleWorkspace, kind: PrimitiveKind) {
        workspace.begin().unwrap();
        workspace
            .edit(None, 1000, |draft| {
                draft.add(kind);
            })
            .unwrap();
    }

    #[test]
    fn strict_parse_rejects_unknown_nested_fields_and_malformed_draft_remains_undoable() {
        let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
        add(&mut workspace, PrimitiveKind::Node);
        let valid = workspace.rules().unwrap();
        workspace
            .edit(None, 1010, |draft| {
                draft.rules[0].condition =
                    r#"{"kind":"not","condition":{"kind":"always","surprise":true}}"#.into()
            })
            .unwrap();
        assert!(
            workspace
                .rules()
                .unwrap_err()
                .iter()
                .any(|issue| issue.field.ends_with("condition.surprise"))
        );
        workspace
            .edit(None, 1020, |draft| draft.rules[0].condition = "{".into())
            .unwrap();
        assert!(workspace.rules().is_err());
        assert!(workspace.prepare(SaveKind::Apply).is_err());
        workspace.undo_draft(false).unwrap();
        assert!(workspace.rules().is_err());
        workspace.undo_draft(false).unwrap();
        assert_eq!(workspace.rules().unwrap(), valid);
    }

    #[test]
    fn reordering_tied_rules_changes_winner_and_draft_undo_restores_it() {
        let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
        add(&mut workspace, PrimitiveKind::Node);
        workspace
            .edit(None, 1010, |draft| {
                draft.add(PrimitiveKind::Node);
            })
            .unwrap();
        workspace
            .edit(None, 1020, |draft| {
                draft.rules[1].effects = r#"[{"kind":"details","visible":false}]"#.into()
            })
            .unwrap();
        let before = workspace.preview(&context("one")).unwrap();
        assert_eq!(before.winners[&Property::Details].rule_id, "node-2");
        workspace
            .edit(None, 1030, |draft| draft.rules.swap(0, 1))
            .unwrap();
        assert_eq!(
            workspace.preview(&context("one")).unwrap().winners[&Property::Details].rule_id,
            "node-1"
        );
        workspace.undo_draft(false).unwrap();
        assert_eq!(workspace.preview(&context("one")).unwrap(), before);
    }

    #[test]
    fn draft_preview_is_pure_and_hysteresis_is_occurrence_local() {
        let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
        add(&mut workspace, PrimitiveKind::Node);
        workspace
            .edit(None, 1010, |draft| {
                draft.rules[0].target = r#"{"kind":"node","ids":["first","second"]}"#.into();
                draft.rules[0].condition =
                    r#"{"kind":"screen_size","enter_below":48,"exit_above":84}"#.into();
            })
            .unwrap();
        let mut first = context("first");
        first.previous_matches.insert("node-1".into());
        first.states.insert(ViewState::Selected);
        let before = first.clone();
        assert!(
            workspace
                .preview(&first)
                .unwrap()
                .matched_rules
                .contains("node-1")
        );
        assert_eq!(first, before);
        assert!(
            workspace
                .preview(&context("second"))
                .unwrap()
                .winners
                .is_empty()
        );
        assert!(
            workspace
                .preview(&context("other"))
                .unwrap()
                .winners
                .is_empty()
        );
        assert_eq!(workspace.committed(), &SavedState::empty().rules);
    }

    #[test]
    fn bounds_and_noop_apply_are_refused() {
        let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
        workspace.begin().unwrap();
        assert!(workspace.prepare(SaveKind::Apply).is_err());
        add(&mut workspace, PrimitiveKind::Node);
        assert!(
            workspace
                .edit(None, 1010, |draft| draft.rules[0].label =
                    "x".repeat(RECORD_LIMIT))
                .is_err()
        );
        assert!(workspace.rules().is_ok());
    }

    #[test]
    fn saved_history_is_capped_at_thirty_two_steps() {
        let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
        add(&mut workspace, PrimitiveKind::Node);
        let apply = workspace.prepare(SaveKind::Apply).unwrap();
        workspace.finish(apply.id, Ok(())).unwrap();
        for index in 0..35 {
            workspace.begin().unwrap();
            workspace
                .edit(None, 2000 + index, |draft| {
                    draft.rules[0].label = format!("label {index}")
                })
                .unwrap();
            let apply = workspace.prepare(SaveKind::Apply).unwrap();
            workspace.finish(apply.id, Ok(())).unwrap();
        }
        for _ in 0..32 {
            let undo = workspace.prepare(SaveKind::Undo).unwrap();
            workspace.finish(undo.id, Ok(())).unwrap();
        }
        assert!(!workspace.can_undo());
        assert!(workspace.can_redo());
    }
}
