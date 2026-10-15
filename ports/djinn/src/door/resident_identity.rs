// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident identity authority the doors serve, as a trait.
//!
//! Graphshell holds no custody (dramatis repo plan, rulings D4 and D15). The
//! browser and application doors and the custody route are generic over
//! [`ResidentIdentity`], and djinn implements it over castellan's
//! `PersonaeHost` ([`crate::keeper`]). The identity endpoint, which stays in
//! Graphshell, reads only the snapshot and intents half,
//! [`IdentityAuthority`], which this trait extends. It was Graphshell's
//! `native::resident_identity` until the stack lift plan's P5.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::door::local_session::DoorIdentity;
use graphshell::browser_carrier::{
    NativeIdentityAction, NativeIdentityFailure, NativeIdentityResult,
};
use graphshell::identity_endpoint::IdentityAuthority;
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal};

/// One custody call in flight.
pub type CustodyFuture =
    Pin<Box<dyn Future<Output = Result<CustodyAnswer, CustodyRefusal>> + Send + 'static>>;

/// What a resident identity authority answers for Graphshell's doors.
pub trait ResidentIdentity: DoorIdentity + IdentityAuthority + Send + Sync + 'static {
    /// A native-only identity action, on the resident's own surface (an
    /// unlock prompt, a file picker). An authority without a surface
    /// refuses.
    fn native_action(&self, action: NativeIdentityAction) -> NativeIdentityResult {
        let _ = action;
        NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::UiUnavailable,
        }
    }

    /// Answer one call on the custody route ([`graphshell_endpoint::door::custody`]),
    /// from the application the door admitted.
    fn custody(self: Arc<Self>, app: AppId, call: CustodyCall) -> CustodyFuture {
        let _ = (app, call);
        Box::pin(async { Err(CustodyRefusal::NotServed) })
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! A fixed authority for Graphshell's own tests: a snapshot it never
    //! changes and in-memory door keys. Tests that need the real keeper live
    //! with djinn, which links castellan.

    use std::sync::{Arc, Mutex};

    use personae::{IdentityError, InMemoryProvider, RetainedKeys};

    use super::*;
    use graphshell_endpoint::door::identity::{
        AgentListenerView, CarryView, IdentitySurfaceSnapshot, VaultLockView, VaultProtectionView,
        VaultView,
    };

    pub(crate) struct FixedIdentity {
        provider: InMemoryProvider,
        snapshot: IdentitySurfaceSnapshot,
        pub(crate) intents: Mutex<Vec<String>>,
    }

    impl FixedIdentity {
        pub(crate) fn new(seed: u8) -> Arc<Self> {
            Arc::new(Self {
                provider: InMemoryProvider::from_seed([seed; 32]),
                snapshot: fixed_snapshot(),
                intents: Mutex::new(Vec::new()),
            })
        }
    }

    pub(crate) fn fixed_snapshot() -> IdentitySurfaceSnapshot {
        IdentitySurfaceSnapshot {
            vault: VaultView {
                protection: VaultProtectionView::Ephemeral,
                lock: VaultLockView::Unlocked,
                agent: AgentListenerView::StandaloneRetained,
            },
            profiles: Vec::new(),
            ssh_keys: Vec::new(),
            carry: CarryView::default(),
            pending_signing: Vec::new(),
            signing_history: Vec::new(),
        }
    }

    impl DoorIdentity for FixedIdentity {
        fn door_keys(&self) -> Result<Arc<RetainedKeys>, IdentityError> {
            self.provider.door_keys()
        }
    }

    impl ResidentIdentity for FixedIdentity {}

    impl IdentityAuthority for FixedIdentity {
        fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
            Ok(self.snapshot.clone())
        }

        /// Records every intent. The one rule it keeps is the keeper's
        /// confirmation gate: a payload saying `"confirmed": false` is
        /// refused, as an unconfirmed device revocation is.
        fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
            let unconfirmed = serde_json::from_slice::<serde_json::Value>(payload)
                .ok()
                .and_then(|value| value.get("confirmed").and_then(|c| c.as_bool()))
                == Some(false);
            if unconfirmed {
                return Err("the action must be confirmed".to_string());
            }
            self.intents.lock().unwrap().push(intent.to_string());
            Ok(())
        }
    }
}
