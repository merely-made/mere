// SPDX-License-Identifier: MPL-2.0
//! Shared Mesquite lane over the ordinary window; it carries public state only.
use crate::{
    State,
    views::{Child, Logic, SHEET},
};
use mesquite::{Ctx, Product};
use taproot::ProbeSnapshot;
#[derive(Default)]
pub struct Acceptance {
    awaiting: std::collections::BTreeMap<String, String>,
}
impl Product for Acceptance {
    type State = State;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "dramatis";
    const SURFACE: &'static str = "app";
    const LOG_PREFIX: &'static str = "dramatis";
    fn sheet(&self) -> &str {
        SHEET
    }
    fn snapshot(&self, ctx: &Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
        let state = ctx.runner.state();
        ProbeSnapshot::default()
            .with_field("pid", std::process::id().to_string())
            .with_field("notice", state.notice.clone())
            .with_field("ready", state.snapshot.is_some().to_string())
            .with_field("busy", state.busy.to_string())
            .with_field(
                "profiles",
                state
                    .snapshot
                    .as_ref()
                    .map(|s| s.roster.entries.len())
                    .unwrap_or(0)
                    .to_string(),
            )
            .with_field(
                "locked",
                state
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.lock == dramatis::view::VaultLockView::Locked)
                    .to_string(),
            )
    }
    fn busy_mut(&mut self, ctx: &mut Ctx<'_, Self>, _: bool) -> Option<bool> {
        let snapshot = self.snapshot(ctx, 0, 1.);
        Some(
            ctx.runner.state().busy
                || self.awaiting.iter().any(|(k, v)| {
                    snapshot.fields.get(k).is_none_or(|actual| {
                        if let Some(part) = v.strip_prefix('~') {
                            !actual.contains(part)
                        } else {
                            actual != v
                        }
                    })
                }),
        )
    }
    fn app_step(
        &mut self,
        _: &mut Ctx<'_, Self>,
        _: mesquite::Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        if let Some(json) = line.strip_prefix("await ") {
            self.awaiting = serde_json::from_str(json).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err(format!("unknown step: {line}"))
        }
    }
}
