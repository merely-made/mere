// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Quarantine routes over the custodian's credential catalog. Djinn shows
//! the exact item before forwarding; the custodian re-checks the item or
//! import revision under profile authority and applies its own consent
//! policy. A stale item is refused there, not here.
use super::*;
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyRefusal};
#[cfg(feature = "cxf")]
use graphshell_endpoint::door::quarantine::{QuarantineCard, ReviewAction, ReviewRequest};
#[cfg(feature = "cxf")]
use personae::ProfileId;

#[cfg(feature = "cxf")]
fn refusal(error: impl std::fmt::Display) -> CustodyRefusal {
    CustodyRefusal::Refused {
        reason: error.to_string(),
    }
}

impl Keeper {
    pub(crate) fn change_passphrase_native(
        &self,
    ) -> dramatis::authentication::PassphraseChangeReceiptV1 {
        crate::identity_ui::change_passphrase(self.host.as_ref(), self.ui.as_ref())
    }
    pub(crate) fn unlock_native(&self) -> Result<CustodyAnswer, CustodyRefusal> {
        match apply_native_identity_action(
            self.host.as_ref(),
            self.ui.as_ref(),
            NativeIdentityAction::UnlockVault,
        ) {
            NativeIdentityResult::UnlockedVault => Ok(CustodyAnswer::Done),
            _ => Err(CustodyRefusal::Refused {
                reason: "Unlock was cancelled or unavailable.".into(),
            }),
        }
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn unassigned_imports(
        &self,
    ) -> Result<Vec<graphshell_endpoint::door::quarantine::UnassignedImport>, CustodyRefusal> {
        let value = self.host.credentials(CredentialCall::UnassignedImports)?;
        serde_json::from_value(value).map_err(refusal)
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn assign_import(
        &self,
        request: graphshell_endpoint::door::quarantine::AssignImportRequest,
    ) -> Result<CustodyAnswer, CustodyRefusal> {
        let imports = self
            .unassigned_imports()?
            .into_iter()
            .find(|i| {
                i.store == request.store
                    && i.persona == request.persona
                    && i.revision == request.revision
            })
            .ok_or_else(|| refusal("Older import changed or was already assigned."))?;
        let title = format!(
            "Older import {} ({} items; store {})",
            imports.persona, imports.item_count, imports.store.0
        );
        let profile = request
            .profile
            .0
            .chars()
            .take(160)
            .filter(|c| !c.is_control())
            .collect::<String>();
        if !self
            .ui
            .confirm_quarantine(&profile, &title, "Assign all items to this persona")
            .map_err(|_| refusal("Djinn's consent surface is unavailable."))?
        {
            return Err(refusal(
                "Assignment cancelled; older imports remain unassigned.",
            ));
        }
        let request = serde_json::to_value(&request).map_err(refusal)?;
        self.host.credentials(CredentialCall::AssignImport(request))?;
        Ok(CustodyAnswer::Done)
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn quarantine(
        &self,
        profile: &ProfileId,
    ) -> Result<Vec<QuarantineCard>, CustodyRefusal> {
        let value = self.host.credentials(CredentialCall::Quarantine(profile.clone()))?;
        serde_json::from_value(value).map_err(refusal)
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn review_quarantine(
        &self,
        request: ReviewRequest,
    ) -> Result<CustodyAnswer, CustodyRefusal> {
        let card = self
            .quarantine(&request.profile)?
            .into_iter()
            .find(|card| {
                card.store == request.store
                    && card.persona == request.persona
                    && card.item.id == request.item
            })
            .ok_or_else(|| refusal("Item is not quarantined under this persona."))?;
        if card.revision != request.revision {
            return Err(refusal("Item changed; review it again."));
        }
        if request.action == ReviewAction::Accept
            && let Some(reason) = card.accept_blocked
        {
            return Err(refusal(reason));
        }
        let safe = |s: &str| {
            s.chars()
                .take(160)
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect::<String>()
        };
        let action = if request.action == ReviewAction::Accept {
            "Accept"
        } else {
            "Permanently delete"
        };
        let title = format!(
            "{} (import store {})",
            safe(&card.item.title),
            safe(&request.store.0)
        );
        if !self
            .ui
            .confirm_quarantine(&safe(&request.profile.0), &title, action)
            .map_err(|_| refusal("Djinn's consent surface is unavailable."))?
        {
            return Err(refusal("Review cancelled; item remains quarantined."));
        }
        let request = serde_json::to_value(&request).map_err(refusal)?;
        self.host.credentials(CredentialCall::Review(request))?;
        Ok(CustodyAnswer::Done)
    }
}
