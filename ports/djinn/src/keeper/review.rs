// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Quarantine routes, with native resident consent and stale-item refusal.
use super::*;
use graphshell::native::custody::{CustodyAnswer, CustodyRefusal};
#[cfg(feature = "cxf")]
use graphshell::native::quarantine::{QuarantineCard, ReviewAction, ReviewRequest};
#[cfg(feature = "cxf")]
use personae::ProfileId;

fn refusal(error: impl std::fmt::Display) -> CustodyRefusal {
    CustodyRefusal::Refused {
        reason: error.to_string(),
    }
}
impl<S: IdentityStorage + 'static> Keeper<S> {
    pub(crate) fn unlock_native(&self) -> Result<CustodyAnswer, CustodyRefusal> {
        match apply_native_identity_action(
            &self.host,
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
    ) -> Result<Vec<graphshell::native::quarantine::UnassignedImport>, CustodyRefusal> {
        let profile = self.host.current_profile_id();
        self.host
            .with_profile_authority(&profile, || {
                let resident = self.credentials.get().ok_or(CustodyRefusal::NotServed)?;
                resident.unassigned_items().map_err(refusal).map(|rows| {
                    rows.into_iter()
                        .map(
                            |(persona, items)| graphshell::native::quarantine::UnassignedImport {
                                persona: *persona.as_uuid(),
                                titles: items.iter().map(|i| i.title.clone()).collect(),
                                item_count: items.len(),
                                revision: castellan::review::namespace_revision(&items),
                            },
                        )
                        .collect()
                })
            })
            .map_err(|error| {
                if matches!(error, IdentityError::Locked) {
                    CustodyRefusal::Locked
                } else {
                    refusal(error)
                }
            })?
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn assign_import(
        &self,
        request: graphshell::native::quarantine::AssignImportRequest,
    ) -> Result<CustodyAnswer, CustodyRefusal> {
        // Confirm against the owner's selected destination, not an app assertion.
        self.host
            .with_profile_authority(&request.profile, || ())
            .map_err(refusal)?;
        let imports = self
            .unassigned_imports()?
            .into_iter()
            .find(|i| i.persona == request.persona && i.revision == request.revision)
            .ok_or_else(|| refusal("Older import changed or was already assigned."))?;
        let title = format!(
            "Older import {} ({} items)",
            imports.persona, imports.item_count
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
        self.host
            .with_profile_authority(&request.profile, || {
                self.credentials
                    .get()
                    .ok_or(CustodyRefusal::NotServed)?
                    .assign_import(
                        personae::PersonaId::from_uuid(request.persona),
                        &request.profile,
                        &request.revision,
                    )
                    .map_err(refusal)?;
                Ok(CustodyAnswer::Done)
            })
            .map_err(|error| {
                if matches!(error, IdentityError::Locked) {
                    CustodyRefusal::Locked
                } else {
                    refusal(error)
                }
            })?
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn quarantine(
        &self,
        profile: &ProfileId,
    ) -> Result<Vec<QuarantineCard>, CustodyRefusal> {
        self.host
            .with_profile_authority(profile, || {
                let resident = self.credentials.get().ok_or(CustodyRefusal::NotServed)?;
                let mut cards = Vec::new();
                for persona in resident.profile_personas(profile).map_err(refusal)? {
                    for item in resident.items(persona).list().map_err(refusal)? {
                        if item.state == chatelaine::ItemState::Quarantined {
                            cards.push(QuarantineCard {
                                persona: *persona.as_uuid(),
                                revision: castellan::review::revision(&item),
                                accept_blocked: castellan::review::accept_blocked(&item),
                                item,
                            });
                        }
                    }
                }
                Ok(cards)
            })
            .map_err(|error| {
                if matches!(error, IdentityError::Locked) {
                    CustodyRefusal::Locked
                } else {
                    refusal(error)
                }
            })?
    }
    #[cfg(feature = "cxf")]
    pub(crate) fn review_quarantine(
        &self,
        request: ReviewRequest,
    ) -> Result<CustodyAnswer, CustodyRefusal> {
        let card = self
            .quarantine(&request.profile)?
            .into_iter()
            .find(|card| card.persona == request.persona && card.item.id == request.item)
            .ok_or_else(|| refusal("Item is not quarantined under this persona."))?;
        if card.revision != request.revision {
            return Err(refusal("Item changed; review it again."));
        }
        if request.action == ReviewAction::Accept {
            if let Some(reason) = card.accept_blocked {
                return Err(refusal(reason));
            }
        }
        // Never hold authority while awaiting a user. Lock and metadata are
        // rechecked afterwards, against the exact card the resident displayed.
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
        if !self
            .ui
            .confirm_quarantine(&safe(&request.profile.0), &safe(&card.item.title), action)
            .map_err(|_| refusal("Djinn's consent surface is unavailable."))?
        {
            return Err(refusal("Review cancelled; item remains quarantined."));
        }
        self.host
            .with_profile_authority(&request.profile, || {
                let resident = self.credentials.get().ok_or(CustodyRefusal::NotServed)?;
                let persona = personae::PersonaId::from_uuid(request.persona);
                if !resident
                    .profile_personas(&request.profile)
                    .map_err(refusal)?
                    .contains(&persona)
                {
                    return Err(refusal("Item namespace is not owned by this persona."));
                }
                resident
                    .items(persona)
                    .review(
                        card.item.id,
                        &request.revision,
                        request.action == ReviewAction::Accept,
                    )
                    .map_err(refusal)?;
                Ok(CustodyAnswer::Done)
            })
            .map_err(|error| {
                if matches!(error, IdentityError::Locked) {
                    CustodyRefusal::Locked
                } else {
                    refusal(error)
                }
            })?
    }
}
