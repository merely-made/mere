// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Running `Condition::Script` (Scenograph editor plan, SE92, SE98): the
//! `scripts` feature, which brings in Genet's `script-engine-api` and nothing
//! of any engine behind it.

use script_engine_api::{Budget, ScriptValue, ValueEngine, ValueError};

use super::*;

/// The budget one script condition runs under, in the engine's own steps.
pub const SCRIPT_STEP_BUDGET: u64 = 100_000;

/// What the resolver holds while it runs scripts: the host's engine, if any.
pub(super) type Engine<'a> = Option<&'a mut dyn ValueEngine>;

impl RuleSet {
    /// Resolve with script conditions run on `engine` (SE92). Before each
    /// script rule the engine's globals are set to the occurrence:
    /// `id` and `kind` (text), `facts` (a map; numbers are floats), `states`
    /// (a list of names), `screen_size` (a float, or unit when unmeasured) and
    /// `previously_matched` (a bool, for a script's own hysteresis). What a
    /// script may call is whatever the host registered on `engine`.
    pub fn resolve_with_scripts(
        &self,
        context: &Context,
        engine: &mut dyn ValueEngine,
    ) -> Result<Resolution, Vec<Issue>> {
        self.resolve_inner(context, Some(engine))
    }
}

/// Set the occurrence's globals on the engine, before a script rule runs.
pub(super) fn bind(
    engine: &mut Engine<'_>,
    context: &Context,
    previously_matched: bool,
) -> Result<(), String> {
    match engine.as_deref_mut() {
        Some(engine) => {
            bind_globals(engine, context, previously_matched).map_err(|error| error.to_string())
        },
        None => Ok(()),
    }
}

/// Evaluate one script condition, preserving failure until the rule boundary.
/// Boolean composition must not turn an engine failure into a match.
pub(super) fn run(source: &str, engine: &mut Engine<'_>) -> Result<(bool, String), String> {
    match engine {
        None => Err("script: no script engine".into()),
        Some(engine) => match engine.eval(source, Budget::Steps(SCRIPT_STEP_BUDGET)) {
            Ok(ScriptValue::Bool(matched)) => Ok((matched, format!("script: {matched}"))),
            Ok(other) => Err(format!("script: returned {}, wanted a bool", other.kind())),
            Err(error) => Err(format!("script: {error}")),
        },
    }
}

/// The occurrence as a script sees it. Facts keep their types; states are the
/// snake_case names `ViewState` serializes to.
fn bind_globals(
    engine: &mut dyn ValueEngine,
    context: &Context,
    previously_matched: bool,
) -> Result<(), ValueError> {
    let scalar = |value: &Scalar| match value {
        Scalar::Bool(value) => ScriptValue::Bool(*value),
        Scalar::Number(value) => ScriptValue::Float(*value),
        Scalar::Text(value) => ScriptValue::Str(value.clone()),
    };
    let text = |value: &str| ScriptValue::Str(value.to_string());
    let kind = serde_json::to_value(context.kind)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default();
    engine.set_global("id", text(&context.id))?;
    engine.set_global("kind", text(&kind))?;
    engine.set_global(
        "facts",
        ScriptValue::Map(
            context
                .facts
                .iter()
                .map(|(name, value)| (name.clone(), scalar(value)))
                .collect(),
        ),
    )?;
    engine.set_global(
        "states",
        ScriptValue::List(
            context
                .states
                .iter()
                .filter_map(|state| serde_json::to_value(state).ok())
                .filter_map(|value| value.as_str().map(|name| text(name)))
                .collect(),
        ),
    )?;
    engine.set_global(
        "screen_size",
        context
            .screen_size
            .map_or(ScriptValue::Unit, ScriptValue::Float),
    )?;
    engine.set_global("previously_matched", ScriptValue::Bool(previously_matched))
}
