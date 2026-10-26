// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Projected signing actions request a resident-owned confirmation. A fresh
//! scene and a public request UUID are not evidence of the owner's decision.
use castellan::{authority::PersonaeHost, custody::IdentityStorage};
use dramatis::intents::{
    SIGNING_APPROVE_IDLE_INTENT, SIGNING_APPROVE_ONCE_INTENT, SIGNING_DENY_INTENT,
    SigningDecisionIntentV1,
};
use personae::signing::{RememberApproval, SigningDecision};

use crate::identity_ui::NativeIdentityUi;

pub(super) fn is_signing_decision(intent: &str) -> bool {
    matches!(
        intent,
        SIGNING_APPROVE_IDLE_INTENT | SIGNING_APPROVE_ONCE_INTENT | SIGNING_DENY_INTENT
    )
}

pub(super) fn confirm<S: IdentityStorage + 'static>(
    host: &PersonaeHost<S>,
    ui: &dyn NativeIdentityUi,
    intent: &str,
    payload: &[u8],
) -> Result<(), String> {
    if payload.len() > 4096 {
        return Err("signing decision payload is too large".into());
    }
    let decision = match intent {
        SIGNING_APPROVE_ONCE_INTENT => SigningDecision::Approve {
            remember: RememberApproval::Once,
        },
        SIGNING_APPROVE_IDLE_INTENT => SigningDecision::Approve {
            remember: RememberApproval::UntilIdle,
        },
        SIGNING_DENY_INTENT => SigningDecision::Deny,
        _ => return Err("unknown signing decision".into()),
    };
    let request: SigningDecisionIntentV1 =
        serde_json::from_slice(payload).map_err(|_| "invalid signing request id")?;
    let context = host
        .begin_native_signing(request.request_id)
        .map_err(|e| e.to_string())?;
    let approved = ui
        .confirm_signing(context.pending(), decision)
        .map_err(|_| "native signing confirmation unavailable")?;
    if !approved {
        return Err("native signing confirmation cancelled".into());
    }
    host.complete_native_signing(context, decision)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::door::browser_carrier::NativeIdentityFailure;
    use castellan::{
        custody::{IdentityVault, InMemoryStorage, Profile, ssh_slot},
        view::VaultProtectionView,
    };
    use ssh_agent_lib::{agent::Session, proto::SignRequest};
    use std::{
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    struct Ui {
        accept: bool,
        calls: AtomicUsize,
        during: Option<Box<dyn Fn() + Send + Sync>>,
    }
    impl NativeIdentityUi for Ui {
        fn confirm_signing(
            &self,
            _: &personae::signing::PendingSigningRequest,
            _: SigningDecision,
        ) -> Result<bool, NativeIdentityFailure> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(f) = &self.during {
                f();
            }
            Ok(self.accept)
        }
        fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
            Err(NativeIdentityFailure::UiUnavailable)
        }
        fn prompt_ssh_private_key_passphrase(
            &self,
        ) -> Result<Option<zeroize::Zeroizing<String>>, NativeIdentityFailure> {
            Err(NativeIdentityFailure::UiUnavailable)
        }
    }
    fn host() -> (Arc<PersonaeHost<InMemoryStorage>>, ssh_key::PublicKey) {
        let key = ssh_key::PrivateKey::random(&mut rand_core::OsRng, ssh_key::Algorithm::Ed25519)
            .unwrap();
        let public = ssh_key::PublicKey::from(&key);
        let mut profile = Profile::new(
            personae::ProfileId("synthetic".into()),
            "Synthetic",
            personae::Ed25519Keypair::from_seed([0x42; 32]),
        );
        profile.slots.insert(
            ssh_slot::protocol_key_for(&key),
            ssh_slot::slot_for(&key, personae::UnlockTier::PerUse).unwrap(),
        );
        let storage = InMemoryStorage::new();
        storage
            .save_profile(&Profile::new(
                personae::ProfileId("other".into()),
                "Other",
                personae::Ed25519Keypair::from_seed([0x43; 32]),
            ))
            .unwrap();
        (
            Arc::new(PersonaeHost::with_decision_timeout(
                IdentityVault::with_profile(storage, profile),
                None,
                VaultProtectionView::Ephemeral,
                Duration::from_secs(10),
            )),
            public,
        )
    }
    async fn pending(host: &PersonaeHost<InMemoryStorage>) -> Vec<u8> {
        loop {
            if let Some(p) = host.snapshot().unwrap().pending_signing.first() {
                return serde_json::to_vec(&SigningDecisionIntentV1 {
                    request_id: p.request.request_id,
                })
                .unwrap();
            }
            tokio::task::yield_now().await;
        }
    }
    #[tokio::test]
    async fn no_native_surface_and_cancelled_native_surface_cannot_release_a_real_signer() {
        let (host, public) = host();
        let mut agent = host.agent_session();
        let waiting = tokio::spawn(async move {
            agent
                .sign(SignRequest {
                    credential: public.key_data().clone().into(),
                    data: b"synthetic native confirmation".to_vec(),
                    flags: 0,
                })
                .await
        });
        let bytes = pending(&host).await;
        assert!(
            confirm(
                &host,
                &crate::identity_ui::UnavailableNativeIdentityUi,
                SIGNING_APPROVE_ONCE_INTENT,
                &bytes
            )
            .is_err()
        );
        let cancelled = Ui {
            accept: false,
            calls: AtomicUsize::new(0),
            during: None,
        };
        assert!(confirm(&host, &cancelled, SIGNING_APPROVE_ONCE_INTENT, &bytes).is_err());
        assert_eq!(cancelled.calls.load(Ordering::SeqCst), 1);
        assert!(!waiting.is_finished());
        let accepted = Ui {
            accept: true,
            calls: AtomicUsize::new(0),
            during: None,
        };
        confirm(&host, &accepted, SIGNING_APPROVE_ONCE_INTENT, &bytes).unwrap();
        assert!(waiting.await.unwrap().is_ok());
        assert_eq!(accepted.calls.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn a_profile_switch_during_the_native_prompt_invalidates_the_confirmed_request() {
        let (host, public) = host();
        let mut agent = host.agent_session();
        let waiting = tokio::spawn(async move {
            agent
                .sign(SignRequest {
                    credential: public.key_data().clone().into(),
                    data: b"stale native confirmation".to_vec(),
                    flags: 0,
                })
                .await
        });
        let bytes = pending(&host).await;
        let copy = host.clone();
        let ui = Ui {
            accept: true,
            calls: AtomicUsize::new(0),
            during: Some(Box::new(move || {
                copy.switch_profile(dramatis::intents::SwitchProfileIntentV1 {
                    profile: "other".into(),
                })
                .unwrap();
            })),
        };
        assert!(confirm(&host, &ui, SIGNING_APPROVE_ONCE_INTENT, &bytes).is_err());
        assert!(!waiting.is_finished());
        let request: SigningDecisionIntentV1 = serde_json::from_slice(&bytes).unwrap();
        host.deny(request.request_id).unwrap();
        assert!(waiting.await.unwrap().is_err());
    }
}
