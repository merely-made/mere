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

use castellan::authority::PersonaeHost;
use castellan::custody::IdentityStorage;
use graphshell::browser_carrier::{NativeIdentityAction, NativeIdentityResult};
use graphshell::identity::IdentitySurfaceSnapshot;
use graphshell::native::app_admission::AppId;
use graphshell::native::custody::CustodyCall;
use graphshell::native::local_session::{DoorIdentity, door_salts};
use graphshell::native::resident_identity::{CustodyFuture, ResidentIdentity};
use personae::{IdentityError, IdentityProvider, RetainedKeys};

use crate::custody::ReleasePolicy;
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

impl<S: IdentityStorage + 'static> ResidentIdentity for Keeper<S> {
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        self.host.snapshot()
    }

    fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
        self.host
            .apply_intent(intent, payload)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn native_action(&self, action: NativeIdentityAction) -> NativeIdentityResult {
        apply_native_identity_action(&self.host, self.ui.as_ref(), action)
    }

    fn custody(self: Arc<Self>, app: AppId, call: CustodyCall) -> CustodyFuture {
        Box::pin(crate::custody::answer(self, app, call))
    }
}
