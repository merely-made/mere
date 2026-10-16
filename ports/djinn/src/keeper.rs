// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The keeper djinn hands Graphshell's doors: castellan's `PersonaeHost`
//! behind graphshell's `ResidentIdentity` (dramatis repo plan, D15).
//!
//! Graphshell names no castellan; its identity endpoint, browser door,
//! application door and custody route are generic over `ResidentIdentity`,
//! and this is the one implementation. It also answers the custody route
//! ([`crate::custody`]): the calls every application makes instead of opening
//! a vault, storage or wallet (D5), with root-level acts kept here and only
//! namespaced derived keys released (D11).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::door::browser_carrier::{NativeIdentityAction, NativeIdentityResult};
use crate::door::identity_endpoint::IdentityAuthority;
use crate::door::local_session::{DoorIdentity, door_salts};
use crate::door::resident_identity::{CustodyFuture, ResidentIdentity};
use castellan::authority::PersonaeHost;
use castellan::custody::IdentityStorage;
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::CustodyCall;
use graphshell_endpoint::door::identity::IdentitySurfaceSnapshot;
use personae::{IdentityError, IdentityProvider, RetainedKeys};

use crate::custody::ReleasePolicy;
mod review;
use crate::identity_ui::{
    NativeIdentityUi, UnavailableNativeIdentityUi, apply_native_identity_action,
};

/// castellan's resident authority, as Graphshell's doors and the custody
/// route see it.
pub struct Keeper<S: IdentityStorage + 'static> {
    host: Arc<PersonaeHost<S>>,
    ui: Arc<dyn NativeIdentityUi>,
    wallet_root: Option<PathBuf>,
    policy: ReleasePolicy,
    #[cfg(feature = "cxf")]
    credentials: std::sync::OnceLock<castellan::resident::CastellanResident>,
    #[cfg(feature = "cxf")]
    credential_stores: std::sync::OnceLock<
        std::collections::BTreeMap<personae::ProfileId, castellan::resident::CastellanResident>,
    >,
}

impl<S: IdentityStorage + 'static> Keeper<S> {
    /// Keep `host`, with no native surface, no wallet and the default
    /// release policy.
    pub fn new(host: Arc<PersonaeHost<S>>) -> Self {
        Self {
            host,
            ui: Arc::new(UnavailableNativeIdentityUi),
            wallet_root: None,
            policy: ReleasePolicy::default(),
            #[cfg(feature = "cxf")]
            credentials: std::sync::OnceLock::new(),
            #[cfg(feature = "cxf")]
            credential_stores: std::sync::OnceLock::new(),
        }
    }

    /// The native surface unlock prompts and the SSH key picker appear on.
    pub fn with_ui(mut self, ui: Arc<dyn NativeIdentityUi>) -> Self {
        self.ui = ui;
        self
    }

    /// The wallet root grants and stations are issued from (D8, D17): the
    /// same carry root the host was composed with.
    pub fn with_wallet_root(mut self, root: Option<PathBuf>) -> Self {
        self.wallet_root = root;
        self
    }

    /// Replace which salts may be released (D11).
    pub fn with_release_policy(mut self, policy: ReleasePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The authority itself, for the resident's own composition.
    pub fn host(&self) -> &Arc<PersonaeHost<S>> {
        &self.host
    }

    /// Attach the resident's existing, lock-bound credential authority.
    #[cfg(feature = "cxf")]
    pub fn attach_credentials(
        &self,
        credentials: castellan::resident::CastellanResident,
    ) -> Result<(), String> {
        let profile = self.host.current_profile_id();
        self.attach_credential_stores(
            profile.clone(),
            std::collections::BTreeMap::from([(profile, credentials)]),
        )
    }

    /// Attach the startup catalog before serving the app door. Each store keeps
    /// its original key persona; CXF commands retain the primary resident store.
    #[cfg(feature = "cxf")]
    pub fn attach_credential_stores(
        &self,
        primary: personae::ProfileId,
        stores: std::collections::BTreeMap<
            personae::ProfileId,
            castellan::resident::CastellanResident,
        >,
    ) -> Result<(), String> {
        let initial = stores
            .get(&primary)
            .ok_or("primary credential store is unavailable")?
            .clone();
        self.credential_stores
            .set(stores)
            .map_err(|_| "credential authority already attached".to_string())?;
        self.credentials
            .set(initial)
            .map_err(|_| "credential authority already attached".into())
    }
    /// Execute owner-only CXF commands without opening another vault.
    #[cfg(feature = "cxf")]
    pub(crate) fn cxf_command(&self, command: &str, args: &[String]) -> Result<String, String> {
        if self.host.is_locked() {
            return Err("vault is locked".into());
        }
        let source = args.first().ok_or("CXF command requires a source file")?;
        let bytes = std::fs::read(source).map_err(|_| "cannot read CXF source file")?;
        let import = castellan::cxf::Import::read(bytes).map_err(|error| error.to_string())?;
        if command == "cxf-preview" {
            if args.len() != 1 {
                return Err("usage: cxf-preview <source.cxf>".into());
            }
            return serde_json::to_string_pretty(import.preview())
                .map(|s| format!("{s}\n"))
                .map_err(|_| "cannot encode CXF preview".into());
        }
        if args.len() != 3 {
            return Err(
                "usage: cxf-import <source.cxf> <destinations.json> <preview-sha256>".into(),
            );
        }
        if args[2] != import.preview().source_sha256 {
            return Err("CXF source changed; preview it again".into());
        }
        let choices = std::fs::read(&args[1]).map_err(|_| "cannot read destination choices")?;
        let destinations =
            serde_json::from_slice(&choices).map_err(|_| "invalid destination choices")?;
        let resident = self
            .credentials
            .get()
            .ok_or("credential authority is unavailable")?;
        match import.commit(destinations, resident, self.host.as_ref()) {
            Ok(receipt) => serde_json::to_string_pretty(&receipt)
                .map(|s| format!("{s}\n"))
                .map_err(|_| "cannot encode CXF receipt".into()),
            Err(failure) => Err(format!(
                "{}\n{}",
                failure.reason,
                serde_json::to_string_pretty(&failure.receipt)
                    .map_err(|_| "cannot encode partial CXF receipt")?
            )),
        }
    }

    pub(crate) fn wallet_root(&self) -> Option<&Path> {
        self.wallet_root.as_deref()
    }

    pub(crate) fn policy(&self) -> &ReleasePolicy {
        &self.policy
    }
}

impl<S: IdentityStorage + 'static> std::ops::Deref for Keeper<S> {
    type Target = PersonaeHost<S>;

    fn deref(&self) -> &PersonaeHost<S> {
        &self.host
    }
}

impl<S: IdentityStorage + 'static> DoorIdentity for Keeper<S> {
    fn door_keys(&self) -> Result<Arc<RetainedKeys>, IdentityError> {
        self.host
            .retained_keys(&door_salts(self.host.master_public_key().to_bytes()))
    }
}

impl<S: IdentityStorage + 'static> IdentityAuthority for Keeper<S> {
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        self.host.snapshot()
    }

    fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
        self.host
            .apply_intent(intent, payload)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

impl<S: IdentityStorage + 'static> ResidentIdentity for Keeper<S> {
    fn native_action(&self, action: NativeIdentityAction) -> NativeIdentityResult {
        apply_native_identity_action(&self.host, self.ui.as_ref(), action)
    }

    fn custody(self: Arc<Self>, app: AppId, call: CustodyCall) -> CustodyFuture {
        Box::pin(crate::custody::answer(self, app, call))
    }
}
