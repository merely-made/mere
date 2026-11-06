// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's retained page text over its route (Eidetic S6).
//!
//! The reservoir owns the store (`pandect::RetainedText`, opened beside the
//! mere's archive); this module only lends it to the one explicit local route
//! an application attached a session through. Two things pass here:
//!
//! - the read-only, bounded `RetainedPages` request, answered for the attached
//!   graph session and no other, with the graph's links from the host's
//!   projected relations; and
//! - two owner-checked intents on that graph session: keep a page's text, and
//!   change the mere's "Keep page text" setting. Keeping writes the same
//!   records the browser extension writes and only while the setting is on and
//!   custody allows; otherwise the answer says why and nothing is written.

use super::*;
use chirograph::{
    RetainedKeeping, RetainedPagesRequest, RetainedPagesResponse,
};
use graphshell::goal_surface::{GraphFacts, RetainedPages, door as page_door};
use graphshell::kept_text::{
    KEEP_PAGES_INTENT, KEEP_SETTING_INTENT, KeepPagesV1, KeepSettingV1, KeepTextPolicy, KeptBatch,
    judge, record_kept_pages,
};
use pandect::{KeepPageText, Keeping, RetainedText};

/// Whether `intent` is one of the retained-text intents.
pub(super) fn is_retained_intent(intent: &IntentInvocation) -> bool {
    matches!(
        intent.intent.as_str(),
        KEEP_PAGES_INTENT | KEEP_SETTING_INTENT
    )
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}

fn wire(keeping: Keeping) -> RetainedKeeping {
    match keeping {
        Keeping::Off => RetainedKeeping::Off,
        Keeping::On => RetainedKeeping::On,
        Keeping::Paused(reason) => RetainedKeeping::Paused { reason },
    }
}

impl MereShared {
    /// The mere's retained store, opened by the reservoir's owner on first use.
    async fn retained(&self) -> Result<&RetainedText<RedbBackend>, String> {
        self.retained
            .get_or_init(|| async {
                pandect::open_mere_retained_backend(&self.shared_root, self.persona, self.mere)
                    .map(|backend| RetainedText::new(backend, Arc::clone(&self.retained_custody)))
                    .map_err(|error| error.to_string())
            })
            .await
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl MereEndpoint {
    /// The retained pages for the attached graph session, within limits.
    pub(super) fn read_retained(
        &mut self,
        request: RetainedPagesRequest,
    ) -> Result<RetainedPagesResponse, String> {
        if request.session.0 != MERE_GRAPH {
            return Err("retained pages are read for the attached graph session only".into());
        }
        let limits = request.limits.effective();
        let (shared, host) = (Arc::clone(&self.shared), Arc::clone(&self.host));
        run(async move {
            let store = shared.retained().await?;
            let keeping = wire(store.keeping().await?);
            // Copy the graph's facts out, then read the store without the lock.
            let facts = GraphFacts::read(host.lock().await.graph());
            let pages = RetainedPages::load_bounded(
                "resident",
                store.backend(),
                &facts,
                page_door::bounds(limits),
                false,
            )
            .await?;
            Ok(page_door::to_wire(&pages, request.session, limits, keeping))
        })
    }

    /// Keep a page's text, or change the setting. Never writes unless the
    /// mere's own setting is on and custody allows.
    pub(super) fn invoke_retained(
        &mut self,
        intent: IntentInvocation,
    ) -> Result<IntentResult, String> {
        let shared = Arc::clone(&self.shared);
        run(async move {
            let store = shared.retained().await?;
            let rejected = |reason: String| Ok(IntentResult::Rejected { reason });
            if intent.intent == KEEP_SETTING_INTENT {
                let asked: KeepSettingV1 =
                    serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
                return match store
                    .set_setting(KeepPageText {
                        enabled: asked.enabled,
                        max_bytes: asked.max_bytes,
                        excluded_origins: asked.excluded_origins,
                    })
                    .await
                {
                    Ok(()) => Ok(IntentResult::Accepted),
                    Err(reason) => rejected(reason),
                };
            }
            let asked: KeepPagesV1 =
                serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
            let setting = match store.permit_write().await {
                Ok(setting) => setting,
                Err(Keeping::Off) => return rejected("Keep page text is off for this mere.".into()),
                Err(Keeping::On) => unreachable!("permit_write never refuses with On"),
                Err(Keeping::Paused(reason)) => {
                    return rejected(format!("Keeping page text is paused. {reason}"));
                },
            };
            let policy = KeepTextPolicy {
                enabled: true,
                max_bytes: setting.max_bytes,
                excluded_origins: setting.excluded_origins,
            };
            // The resident stamps its own clock, and says why a page is refused
            // rather than dropping it silently.
            let at_ms = now_ms();
            let mut pages = asked.pages;
            for page in &mut pages {
                page.at_ms = at_ms;
                if let Err(refusal) = judge(&policy, page) {
                    return rejected(refusal.reason().into());
                }
            }
            let summary = record_kept_pages(store.backend(), &KeptBatch { policy, pages }).await;
            match summary.error {
                Some(error) => rejected(format!("Page text could not be kept: {error}")),
                None => Ok(IntentResult::Accepted),
            }
        })
    }
}
