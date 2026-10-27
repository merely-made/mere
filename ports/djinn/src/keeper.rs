// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The keeper djinn hands Graphshell's doors: custody behind graphshell's
//! `ResidentIdentity` (dramatis repo plan, D15).
//!
//! Graphshell names no castellan; its identity endpoint, browser door,
//! application door and custody route are generic over `ResidentIdentity`,
//! and this is the one implementation. It also answers the custody route
//! ([`crate::custody`]): the calls every application makes instead of opening
//! a vault, storage or wallet (D5). Custody itself is
//! [`ResidentCustody`]: the independently launched custodian in a
//! production build, so root-level acts happen there and only namespaced
//! derived keys reach this process (custody isolation plan, H2).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::custody_host::{CredentialCall, ResidentCustody};
use crate::door::browser_carrier::{NativeIdentityAction, NativeIdentityResult};
use crate::door::identity_endpoint::IdentityAuthority;
use crate::door::local_session::{DoorIdentity, door_salts};
use crate::door::resident_identity::{CustodyFuture, ResidentIdentity};
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::CustodyCall;
use graphshell_endpoint::door::identity::IdentitySurfaceSnapshot;
use personae::{IdentityError, RetainedKeys};

use crate::custody::ReleasePolicy;
mod review;
#[cfg(feature = "in-process-custody")]
pub(crate) mod signing;
use crate::identity_ui::{
    NativeIdentityUi, UnavailableNativeIdentityUi, apply_native_identity_action,
};

/// Custody, as Graphshell's doors and the custody route see it.
pub struct Keeper {
    host: Arc<dyn ResidentCustody>,
    ui: Arc<dyn NativeIdentityUi>,
    wallet_root: Option<PathBuf>,
    policy: ReleasePolicy,
    payload_custody: std::sync::OnceLock<crate::custody::payload::PayloadCustody>,
}

impl Keeper {
    /// Keep `host`, with no native surface, no wallet and the default
    /// release policy.
    pub fn new(host: Arc<dyn ResidentCustody>) -> Self {
        Self {
            host,
            ui: Arc::new(UnavailableNativeIdentityUi),
            wallet_root: None,
            policy: ReleasePolicy::default(),
            payload_custody: std::sync::OnceLock::new(),
        }
    }

    /// The native surface unlock prompts and the SSH key picker appear on.
    pub fn with_ui(mut self, ui: Arc<dyn NativeIdentityUi>) -> Self {
        self.ui = ui;
        self
    }

    /// The wallet root grants and stations are issued from (D8, D17): the
    /// same carry root the host was composed with. A custodian uses its own.
    pub fn with_wallet_root(mut self, root: Option<PathBuf>) -> Self {
        self.wallet_root = root;
        self
    }

    /// Bind retained-content custody to the owner's explicit wallet namespace.
    /// This does not create a wallet, epoch or secret. Configure before serving.
    pub fn configure_payload_custody(
        &self,
        profile: personae::ProfileId,
        persona: personae::PersonaId,
        root: PathBuf,
    ) -> Result<(), String> {
        self.payload_custody
            .set(crate::custody::payload::PayloadCustody {
                profile,
                persona,
                root,
            })
            .map_err(|_| "retained-content custody is already configured".into())
    }

    pub(crate) fn payload_custody(&self) -> Option<&crate::custody::payload::PayloadCustody> {
        self.payload_custody.get()
    }

    /// Replace which salts may be released (D11).
    pub fn with_release_policy(mut self, policy: ReleasePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Custody itself, for the resident's own composition.
    pub fn host(&self) -> &Arc<dyn ResidentCustody> {
        &self.host
    }

    /// Owner-only CXF commands: the source is read here and parsed and
    /// committed by the custodian, which holds the credential catalog.
    #[cfg(feature = "cxf")]
    pub(crate) fn cxf_command(&self, command: &str, args: &[String]) -> Result<String, String> {
        if self.host.is_locked() {
            return Err("vault is locked".into());
        }
        let source = args.first().ok_or("CXF command requires a source file")?;
        let bytes = std::fs::read(source).map_err(|_| "cannot read CXF source file")?;
        let refused = |r: graphshell_endpoint::door::custody::CustodyRefusal| match r {
            graphshell_endpoint::door::custody::CustodyRefusal::Refused { reason } => reason,
            other => format!("{other:?}"),
        };
        if command == "cxf-preview" {
            if args.len() != 1 {
                return Err("usage: cxf-preview <source.cxf>".into());
            }
            let preview = self.host.credentials(CredentialCall::CxfPreview(bytes)).map_err(refused)?;
            return serde_json::to_string_pretty(&preview)
                .map(|s| format!("{s}\n"))
                .map_err(|_| "cannot encode CXF preview".into());
        }
        if args.len() != 3 {
            return Err(
                "usage: cxf-import <source.cxf> <destinations.json> <preview-sha256>".into(),
            );
        }
        let choices = std::fs::read(&args[1]).map_err(|_| "cannot read destination choices")?;
        let destinations: serde_json::Value =
            serde_json::from_slice(&choices).map_err(|_| "invalid destination choices")?;
        let receipt = self
            .host
            .credentials(CredentialCall::CxfImport {
                source: bytes,
                destinations,
                preview_sha256: args[2].clone(),
            })
            .map_err(refused)?;
        serde_json::to_string_pretty(&receipt)
            .map(|s| format!("{s}\n"))
            .map_err(|_| "cannot encode CXF receipt".into())
    }

    pub(crate) fn wallet_root(&self) -> Option<&Path> {
        self.wallet_root.as_deref()
    }

    pub(crate) fn policy(&self) -> &ReleasePolicy {
        &self.policy
    }
}

impl std::ops::Deref for Keeper {
    type Target = dyn ResidentCustody;

    fn deref(&self) -> &dyn ResidentCustody {
        self.host.as_ref()
    }
}

impl DoorIdentity for Keeper {
    fn door_keys(&self) -> Result<Arc<RetainedKeys>, IdentityError> {
        self.host
            .retained_keys(&door_salts(self.host.master_public_key().to_bytes()))
    }
}

impl IdentityAuthority for Keeper {
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        self.host.snapshot()
    }

    fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
        use dramatis::intents::{SIGNING_APPROVE_IDLE_INTENT, SIGNING_APPROVE_ONCE_INTENT, SIGNING_DENY_INTENT};
        if matches!(intent, SIGNING_APPROVE_IDLE_INTENT | SIGNING_APPROVE_ONCE_INTENT | SIGNING_DENY_INTENT) {
            return self.host.confirm_signing(self.ui.as_ref(), intent, payload);
        }
        self.host.apply_intent(intent, payload)
    }
}

impl ResidentIdentity for Keeper {
    fn native_action(&self, action: NativeIdentityAction) -> NativeIdentityResult {
        apply_native_identity_action(self.host.as_ref(), self.ui.as_ref(), action)
    }

    fn custody(self: Arc<Self>, app: AppId, call: CustodyCall) -> CustodyFuture {
        Box::pin(crate::custody::answer(self, app, call))
    }
}
