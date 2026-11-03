// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Retained-content seal operations under stable custody/profile authority.
use crate::keeper::Keeper;
use ostiary::app_admission::AppId;
use ostiary::declaration::AppAuthority;
use castellan_client::custody::{CustodyAnswer, CustodyRefusal};
use castellan_client::custody_payload::PayloadRequest;
use personae::{PersonaId, ProfileId};
use std::path::PathBuf;

pub(crate) struct PayloadCustody {
    pub profile: ProfileId,
    pub persona: PersonaId,
    pub root: PathBuf,
}

pub(super) fn answer(
    keeper: &Keeper,
    authority: &AppAuthority,
    request: PayloadRequest,
) -> Result<CustodyAnswer, CustodyRefusal> {
    // Route admission alone does not authorize use of retained-content keys.
    // The scope is the one declared and granted (capability door D30); until
    // every retained-content application declares one, the label's scope
    // stands in (djinn-slimming D22, removed by the M1 lane).
    let app_scope = match authority.payload_scope() {
        Ok(scope) => scope,
        Err(_) => label_scope(authority.app())?,
    };
    request.validate().map_err(super::refused)?;
    if let Some(expected) = request.custody_epoch.as_ref() {
        if keeper.host().custody_epoch()?.as_ref() != Some(expected) {
            return Err(super::refused(
                "retained-content custody generation changed",
            ));
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
    let scope = serde_json::to_vec(&(app_scope, &request.profile.0, &request.master))
        .map_err(super::failed)?;
    let result = keeper
        .host()
        .payload(&binding.root, binding.persona, request, scope)?;
    Ok(CustodyAnswer::Payload { result })
}

/// The scope each label sealed under before scopes were declared.
fn label_scope(app: &AppId) -> Result<&'static str, CustodyRefusal> {
    match app.as_str() {
        // Preserve the existing encoded AAD for retained Turnstone ciphertext.
        "turnstone" => Ok("turnstone.retained-clips/custody-v1"),
        "graphshell" => Ok("graphshell.retained-content/custody-v1"),
        _ => Err(CustodyRefusal::NotServed),
    }
}
