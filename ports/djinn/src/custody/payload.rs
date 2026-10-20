// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Retained-content seal operations under stable custody/profile authority.
use crate::keeper::Keeper;
use castellan::custody::IdentityStorage;
use eidetic::{Hash, PayloadSealer};
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyRefusal};
use graphshell_endpoint::door::custody_payload::{
    PayloadBytes, PayloadOperation, PayloadRequest, PayloadResult,
};
use personae::{PersonaId, ProfileId};
use std::path::PathBuf;

pub(crate) struct PayloadCustody {
    pub profile: ProfileId,
    pub persona: PersonaId,
    pub root: PathBuf,
}

pub(super) fn answer<S: IdentityStorage + 'static>(
    keeper: &Keeper<S>,
    app: &AppId,
    request: PayloadRequest,
) -> Result<CustodyAnswer, CustodyRefusal> {
    // Route admission alone does not authorize use of retained-content keys.
    if app.as_str() != "turnstone" {
        return Err(CustodyRefusal::NotServed);
    }
    request.validate().map_err(super::refused)?;
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
    keeper
        .host()
        .with_current_profile_authority(&request.profile, &request.master, || {
            let result = match request.operation {
                PayloadOperation::Seal { bytes } => {
                    let sealer = castellan::custody::wallet::epoch_sealer_for_persona(
                        &binding.root,
                        binding.persona,
                    )
                    .map_err(super::failed)?
                    .ok_or(CustodyRefusal::Locked)?
                    .with_scope(scope.clone());
                    let (bytes, marker) = sealer
                        .seal(&request.content_hash, &bytes.0)
                        .map_err(super::failed)?;
                    PayloadResult::Sealed {
                        bytes: PayloadBytes(bytes),
                        marker,
                    }
                },
                PayloadOperation::Unseal { bytes, marker } => {
                    let bytes = castellan::custody::wallet::unseal_persona_payload(
                        &binding.root,
                        binding.persona,
                        &request.content_hash,
                        &scope,
                        &marker,
                        &bytes.0,
                    )
                    .map_err(super::failed)?
                    .ok_or(CustodyRefusal::Locked)?;
                    if Hash::of(&bytes) != request.content_hash {
                        return Err(super::refused("unsealed retained payload hash differs"));
                    }
                    PayloadResult::Clear {
                        bytes: PayloadBytes(bytes),
                    }
                },
            };
            Ok(CustodyAnswer::Payload { result })
        })
        .map_err(super::identity)?
}
