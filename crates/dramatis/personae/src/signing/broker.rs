// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The approval broker: pending requests, the short-TTL cache and visible
//! decisions. Custody, behind `agent` (dramatis repo plan, ruling D25).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::oneshot;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ApprovalCacheKey {
    profile: String,
    fingerprint: String,
    operation: String,
    adapter: String,
    requester: Option<String>,
    process: Option<String>,
    target: Option<String>,
    session: Option<String>,
}

impl From<&SigningRequest> for ApprovalCacheKey {
    fn from(request: &SigningRequest) -> Self {
        Self {
            profile: request.profile.clone(),
            fingerprint: request.public_key_fingerprint.clone(),
            operation: request.operation.clone(),
            adapter: request.adapter.clone(),
            requester: request.authenticated_requester.clone(),
            process: request.authenticated_process.clone(),
            target: request.authenticated_target.clone(),
            session: request.session_binding.clone(),
        }
    }
}

struct ResolvedDecision {
    decision: SigningDecision,
}

struct PendingEntry {
    request: SigningRequest,
    policy: SigningPolicy,
    expires_at_ms: u64,
    sender: oneshot::Sender<ResolvedDecision>,
}

#[derive(Default)]
struct BrokerState {
    pending: HashMap<Uuid, PendingEntry>,
    history: Vec<SigningRecord>,
    short_ttl: HashMap<ApprovalCacheKey, u64>,
}

/// Shared native approval broker used by Graphshell and signing adapters.
#[derive(Clone)]
pub struct ApprovalBroker {
    state: Arc<Mutex<BrokerState>>,
    decision_timeout: Duration,
}

impl ApprovalBroker {
    /// Build a broker with a bounded decision timeout.
    pub fn new(decision_timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(BrokerState::default())),
            decision_timeout,
        }
    }

    /// Authorize one request under its configured policy.
    pub async fn authorize(
        &self,
        request: SigningRequest,
        policy: SigningPolicy,
    ) -> Result<SigningAuthorization, AuthorizationError> {
        let now = now_ms();
        match policy {
            SigningPolicy::Session => {
                return Ok(SigningAuthorization {
                    request,
                    policy,
                    source: ApprovalSource::SessionPolicy,
                });
            },
            SigningPolicy::ShortTtl { idle_seconds } => {
                let key = ApprovalCacheKey::from(&request);
                let mut state = self.state.lock().unwrap();
                state.short_ttl.retain(|_, expires| *expires > now);
                if state
                    .short_ttl
                    .get(&key)
                    .is_some_and(|expires| *expires > now)
                {
                    state.short_ttl.insert(
                        key,
                        now.saturating_add(u64::from(idle_seconds).saturating_mul(1_000)),
                    );
                    return Ok(SigningAuthorization {
                        request,
                        policy,
                        source: ApprovalSource::CachedShortTtl,
                    });
                }
            },
            SigningPolicy::PerUse => {},
        }

        self.await_visible_decision(request, policy).await
    }

    async fn await_visible_decision(
        &self,
        request: SigningRequest,
        policy: SigningPolicy,
    ) -> Result<SigningAuthorization, AuthorizationError> {
        let (sender, receiver) = oneshot::channel();
        let expires_at_ms = now_ms().saturating_add(self.decision_timeout.as_millis() as u64);
        self.state.lock().unwrap().pending.insert(
            request.request_id,
            PendingEntry {
                request: request.clone(),
                policy,
                expires_at_ms,
                sender,
            },
        );

        match tokio::time::timeout(self.decision_timeout, receiver).await {
            Ok(Ok(resolved)) => match resolved.decision {
                SigningDecision::Approve { remember } => Ok(SigningAuthorization {
                    request,
                    policy,
                    source: match remember {
                        RememberApproval::Once => ApprovalSource::UserOnce,
                        RememberApproval::UntilIdle => ApprovalSource::UserUntilIdle,
                    },
                }),
                SigningDecision::Deny => {
                    self.push_terminal_record(request, policy, None, SigningRecordResult::Denied);
                    Err(AuthorizationError::Denied)
                },
            },
            Ok(Err(_)) => {
                self.state
                    .lock()
                    .unwrap()
                    .pending
                    .remove(&request.request_id);
                self.push_terminal_record(
                    request,
                    policy,
                    None,
                    SigningRecordResult::Failed {
                        code: SigningFailureCode::ApprovalChannelClosed,
                    },
                );
                Err(AuthorizationError::BrokerClosed)
            },
            Err(_) => {
                self.state
                    .lock()
                    .unwrap()
                    .pending
                    .remove(&request.request_id);
                self.push_terminal_record(request, policy, None, SigningRecordResult::TimedOut);
                Err(AuthorizationError::TimedOut)
            },
        }
    }

    /// Resolve one pending request from Graphshell's visible approval surface.
    pub fn decide(&self, request_id: Uuid, decision: SigningDecision) -> Result<(), DecisionError> {
        let mut state = self.state.lock().unwrap();
        let Some(entry) = state.pending.remove(&request_id) else {
            return Err(DecisionError::NotPending);
        };

        if matches!(
            (entry.policy, decision),
            (
                SigningPolicy::PerUse | SigningPolicy::Session,
                SigningDecision::Approve {
                    remember: RememberApproval::UntilIdle
                }
            )
        ) {
            state.pending.insert(request_id, entry);
            return Err(DecisionError::RememberNotAllowed);
        }

        let short_ttl_cache = if let (
            SigningPolicy::ShortTtl { idle_seconds },
            SigningDecision::Approve {
                remember: RememberApproval::UntilIdle,
            },
        ) = (entry.policy, decision)
        {
            Some((
                ApprovalCacheKey::from(&entry.request),
                now_ms().saturating_add(u64::from(idle_seconds).saturating_mul(1_000)),
            ))
        } else {
            None
        };

        entry
            .sender
            .send(ResolvedDecision { decision })
            .map_err(|_| DecisionError::RequestClosed)?;
        if let Some((key, expires_at_ms)) = short_ttl_cache {
            state.short_ttl.insert(key, expires_at_ms);
        }
        Ok(())
    }

    /// Append the final result for an authorized request.
    pub fn complete(&self, authorization: SigningAuthorization, result: SigningRecordResult) {
        self.push_terminal_record(
            authorization.request,
            authorization.policy,
            Some(authorization.source),
            result,
        );
    }

    /// Current visible requests, ordered by request time then id.
    pub fn pending(&self) -> Vec<PendingSigningRequest> {
        let mut pending: Vec<_> = self
            .state
            .lock()
            .unwrap()
            .pending
            .values()
            .map(|entry| PendingSigningRequest {
                request: entry.request.clone(),
                policy: entry.policy,
                expires_at_ms: entry.expires_at_ms,
            })
            .collect();
        pending.sort_by_key(|entry| (entry.request.requested_at_ms, entry.request.request_id));
        pending
    }

    /// Append-only signing decision/result history.
    pub fn history(&self) -> Vec<SigningRecord> {
        self.state.lock().unwrap().history.clone()
    }

    fn push_terminal_record(
        &self,
        request: SigningRequest,
        policy: SigningPolicy,
        approval_source: Option<ApprovalSource>,
        result: SigningRecordResult,
    ) {
        self.state.lock().unwrap().history.push(SigningRecord {
            request,
            policy,
            approval_source,
            result,
            completed_at_ms: now_ms(),
        });
    }
}

impl Default for ApprovalBroker {
    fn default() -> Self {
        Self::new(Duration::from_secs(120))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(payload: &[u8]) -> SigningRequest {
        SigningRequest::new("research", "SHA256:test", "ssh.sign", payload, "ssh-agent")
    }

    async fn wait_for_pending(broker: &ApprovalBroker) -> PendingSigningRequest {
        for _ in 0..100 {
            if let Some(pending) = broker.pending().into_iter().next() {
                return pending;
            }
            tokio::task::yield_now().await;
        }
        panic!("request never became pending");
    }

    #[tokio::test]
    async fn per_use_waits_for_a_visible_decision_and_records_the_result() {
        let broker = ApprovalBroker::new(Duration::from_secs(1));
        let waiting = broker.clone();
        let task = tokio::spawn(async move {
            waiting
                .authorize(request(b"payload-secret"), SigningPolicy::PerUse)
                .await
        });
        let pending = wait_for_pending(&broker).await;
        assert_eq!(pending.request.payload_digest.len(), "blake3:".len() + 64);
        broker
            .decide(
                pending.request.request_id,
                SigningDecision::Approve {
                    remember: RememberApproval::Once,
                },
            )
            .unwrap();
        let authorized = task.await.unwrap().unwrap();
        broker.complete(
            authorized,
            SigningRecordResult::Signed {
                signature_ref: "blake3:signature".to_string(),
            },
        );

        let history = broker.history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].approval_source, Some(ApprovalSource::UserOnce));
        let json = serde_json::to_string(&history).unwrap();
        assert!(!json.contains("payload-secret"));
    }

    #[tokio::test]
    async fn denial_returns_to_the_adapter_and_appends_one_record() {
        let broker = ApprovalBroker::new(Duration::from_secs(1));
        let waiting = broker.clone();
        let task = tokio::spawn(async move {
            waiting
                .authorize(request(b"deny-me"), SigningPolicy::PerUse)
                .await
        });
        let pending = wait_for_pending(&broker).await;
        broker
            .decide(pending.request.request_id, SigningDecision::Deny)
            .unwrap();
        assert_eq!(task.await.unwrap(), Err(AuthorizationError::Denied));
        assert_eq!(broker.history().len(), 1);
        assert_eq!(broker.history()[0].result, SigningRecordResult::Denied);
    }

    #[tokio::test]
    async fn short_ttl_reuses_then_expires_the_visible_approval() {
        let broker = ApprovalBroker::new(Duration::from_secs(1));
        let policy = SigningPolicy::ShortTtl { idle_seconds: 1 };
        let waiting = broker.clone();
        let first = tokio::spawn(async move { waiting.authorize(request(b"a"), policy).await });
        let pending = wait_for_pending(&broker).await;
        broker
            .decide(
                pending.request.request_id,
                SigningDecision::Approve {
                    remember: RememberApproval::UntilIdle,
                },
            )
            .unwrap();
        assert_eq!(
            first.await.unwrap().unwrap().source,
            ApprovalSource::UserUntilIdle
        );

        assert_eq!(
            broker
                .authorize(request(b"b"), policy)
                .await
                .unwrap()
                .source,
            ApprovalSource::CachedShortTtl
        );
        tokio::time::sleep(Duration::from_millis(1_100)).await;

        let waiting = broker.clone();
        let after_expiry =
            tokio::spawn(async move { waiting.authorize(request(b"c"), policy).await });
        let pending = wait_for_pending(&broker).await;
        broker
            .decide(pending.request.request_id, SigningDecision::Deny)
            .unwrap();
        assert_eq!(after_expiry.await.unwrap(), Err(AuthorizationError::Denied));
    }

    #[tokio::test]
    async fn per_use_cannot_be_widened_into_a_cached_approval() {
        let broker = ApprovalBroker::new(Duration::from_secs(1));
        let waiting = broker.clone();
        let task = tokio::spawn(async move {
            waiting
                .authorize(request(b"x"), SigningPolicy::PerUse)
                .await
        });
        let pending = wait_for_pending(&broker).await;
        assert_eq!(
            broker.decide(
                pending.request.request_id,
                SigningDecision::Approve {
                    remember: RememberApproval::UntilIdle,
                },
            ),
            Err(DecisionError::RememberNotAllowed)
        );
        broker
            .decide(pending.request.request_id, SigningDecision::Deny)
            .unwrap();
        assert_eq!(task.await.unwrap(), Err(AuthorizationError::Denied));
    }
}
