// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bind the Dramatis service to this host's graph-door identities and routes.

use std::sync::Arc;
use dramatis_resident::keeper::Keeper;
use dramatis_resident::native::{NativeIdentityAction, NativeIdentityResult};
use personae::{IdentityError, IdentityProvider, RetainedKeys};
use castellan_client::custody::CustodyCall;
use ostiary::{app_admission::AppId, declaration::AppAuthority};
use super::local_session::{DoorIdentity, door_salts};
use super::resident_identity::{CustodyFuture, ResidentIdentity};

impl DoorIdentity for Keeper {
    fn door_keys(&self) -> Result<Arc<RetainedKeys>, IdentityError> {
        self.retained_keys(&door_salts(self.master_public_key().to_bytes()))
    }
}

impl ResidentIdentity for Keeper {
    fn native_action(&self, action: NativeIdentityAction) -> NativeIdentityResult {
        Keeper::native_action(self, action)
    }

    fn custody(
        self: Arc<Self>,
        app: AppId,
        authority: Option<AppAuthority>,
        call: CustodyCall,
    ) -> CustodyFuture {
        self.answer_custody(app, authority, call)
    }
}
