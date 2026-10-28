// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Retained-content seal operations under stable custody/profile authority.
use crate::keeper::Keeper;
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyRefusal};
use graphshell_endpoint::door::custody_payload::PayloadRequest;
use personae::{PersonaId, ProfileId};
use std::path::PathBuf;

pub(crate) struct PayloadCustody {
    pub profile: ProfileId,
    pub persona: PersonaId,
    pub root: PathBuf,
}

pub(super) fn answer(
    keeper: &Keeper,
    app: &AppId,
    request: PayloadRequest,
) -> Result<CustodyAnswer, CustodyRefusal> {
    // Route admission alone does not authorize use of retained-content keys.
    if app.as_str() != "turnstone" {
        return Err(CustodyRefusal::NotServed);
    }
    request.validate().map_err(super::refused)?;
    if let Some(expected) = request.custody_epoch.as_ref() {
        if keeper.host().custody_epoch()?.as_ref() != Some(expected) {
            return Err(super::refused("retained-content custody generation changed"));
        }
    }
    let binding = keeper.payload_custody().ok_or_else(|| {
        super::refused("retained-content custody has no configured wallet namespace")
    })?;
    if request.profile != binding.profile {
        return Err(super::refused("retained-content profile is not admitted"));
    }
    // The admitted application cannot use this operation to decrypt another
    // lane's wallet-sealed artifacts. This AAD binding is independent from the
    // immutable payload's canonical content identity.
    let scope = serde_json::to_vec(&(
        "turnstone.retained-clips/custody-v1",
        &request.profile.0,
        &request.master,
    ))
    .map_err(super::failed)?;
    let result = keeper.host().payload(&binding.root, binding.persona, request, scope)?;
    Ok(CustodyAnswer::Payload { result })
}
