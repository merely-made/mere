// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use murm_ocapn::{BoxFuture, CallContext, CallError, HostObject};

pub(super) struct CollectionSession<B, P> {
    pub(super) owner: CollectionOwner<B, P>,
    pub(super) authority: MootSessionAuthority,
}

impl<B, P> HostObject for CollectionSession<B, P>
where
    B: Backend + Clone + Send + Sync + 'static,
    P: IdentityProvider + Send + Sync + 'static,
{
    fn call<'a>(
        &'a self,
        context: CallContext,
        args: Vec<Value>,
    ) -> BoxFuture<'a, Result<Value, CallError>> {
        Box::pin(async move {
            let supplied = context
                .authority::<MootSessionAuthority>()
                .ok_or_else(|| CallError::message("missing admitted coop authority"))?;
            if !self.authority.same_admission(supplied) || context.session().is_closed() {
                return Err(CallError::message("foreign or closed coop session"));
            }
            let result = match args.as_slice() {
                [Value::Symbol(method)] if method == "view" => self
                    .owner
                    .read_bound(&self.authority, Some(context.session()))
                    .await
                    .and_then(|view| encode("gemot:collection-view", &view)),
                [Value::Symbol(method), Value::Cap(callback)] if method == "subscribe" => self
                    .owner
                    .subscribe(&self.authority, context.session().clone(), callback.clone())
                    .await
                    .map(|id| Value::Integer(id as i64)),
                [Value::Symbol(method), Value::Integer(id)]
                    if method == "unsubscribe" && *id > 0 =>
                {
                    self.owner
                        .unsubscribe(&self.authority, context.session().session_id(), *id as u64)
                        .await
                        .map(Value::Bool)
                },
                [Value::Symbol(method), Value::Bytes(bytes)]
                    if method == "set-membership" && bytes.len() <= 4096 =>
                {
                    let proposal: MembershipProposal = serde_json::from_slice(bytes)
                        .map_err(|_| CallError::message("malformed bounded membership proposal"))?;
                    let accepted = self
                        .owner
                        .propose_bound(&self.authority, proposal, Some(context.session()))
                        .await
                        .map_err(wire_error)?;
                    // The ordinary domain command has already committed. Failed
                    // callbacks do not turn it into a second mutation attempt.
                    self.owner.emit_current().await;
                    encode("gemot:proposal-receipt", &accepted.receipt)
                },
                _ => Err(CoopError::Malformed),
            };
            result.map_err(wire_error)
        })
    }
}

fn wire_error(error: CoopError) -> CallError {
    // Host-facing errors retain their diagnostic detail; an exported object
    // must not disclose storage paths or backend internals to a remote peer.
    match error {
        CoopError::Domain(_) => CallError::message("domain operation failed"),
        other => CallError::message(other.to_string()),
    }
}

fn encode(label: &str, value: &impl Serialize) -> Result<Value, CoopError> {
    let bytes = serde_json::to_vec(value).map_err(|_| CoopError::Malformed)?;
    if bytes.len() > MAX_VIEW_BYTES {
        return Err(CoopError::Limit);
    }
    Ok(Value::Record {
        label: label.into(),
        fields: vec![Value::Bytes(bytes)],
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmissionReport {
    pub enqueued: usize,
    pub removed: usize,
}

impl<B, P> CollectionOwner<B, P>
where
    B: Backend + Clone + Send + Sync + 'static,
    P: IdentityProvider + Send + Sync + 'static,
{
    async fn subscribe(
        &self,
        authority: &MootSessionAuthority,
        session: SessionHandle,
        callback: Capability,
    ) -> Result<u64, CoopError> {
        if session.is_closed() || callback.session_id() != session.session_id() {
            return Err(CoopError::SessionClosed);
        }
        let mut state = self.inner.state.lock().await;
        if session.is_closed() {
            return Err(CoopError::SessionClosed);
        }
        // Read authority is checked again under the same coordinator as the
        // subscription insertion; no remote reference grants access itself.
        self.snapshot_locked(&state, authority, (self.inner.clock)())
            .await?;
        if state.subscriptions.len() >= MAX_SUBSCRIPTIONS
            || state
                .subscriptions
                .values()
                .filter(|entry| entry.session.session_id() == session.session_id())
                .count()
                >= MAX_SUBSCRIPTIONS_PER_SESSION
            || (!state.watchers.contains_key(&session.session_id())
                && state.watchers.len() >= MAX_WATCHED_SESSIONS)
        {
            return Err(CoopError::Limit);
        }
        let id = state.next_subscription;
        state.next_subscription = state
            .next_subscription
            .checked_add(1)
            .filter(|next| *next <= i64::MAX as u64)
            .ok_or(CoopError::Limit)?;
        state.subscriptions.insert(
            id,
            Subscription {
                authority: authority.clone(),
                session: session.clone(),
                callback,
            },
        );
        state
            .watchers
            .entry(session.session_id())
            .or_insert_with(|| {
                let weak = Arc::downgrade(&self.inner);
                tokio::spawn(async move {
                    session.closed().await;
                    if let Some(inner) = weak.upgrade() {
                        let mut state = inner.state.lock().await;
                        remove_session(&mut state, session.session_id());
                    }
                })
            });
        Ok(id)
    }
    async fn unsubscribe(
        &self,
        authority: &MootSessionAuthority,
        session: SessionId,
        id: u64,
    ) -> Result<bool, CoopError> {
        let mut state = self.inner.state.lock().await;
        self.snapshot_locked(&state, authority, (self.inner.clock)())
            .await?;
        let owned = state.subscriptions.get(&id).is_some_and(|entry| {
            entry.session.session_id() == session && entry.authority.same_admission(authority)
        });
        if owned {
            state.subscriptions.remove(&id);
            cleanup_watcher(&mut state, session);
        }
        Ok(owned)
    }
    /// Host-triggered projection updates after coordinated domain changes.
    /// Each callback is checked immediately before a synchronous bounded queue
    /// insertion while authority changes are excluded. No peer reply is awaited.
    /// An emission already enqueued before revocation may finish in flight;
    /// later emissions and invocations recheck authority and fail closed.
    pub async fn emit_current(&self) -> EmissionReport {
        let mut state = self.inner.state.lock().await;
        let ids: Vec<_> = state.subscriptions.keys().copied().collect();
        let mut report = EmissionReport::default();
        for id in ids {
            let Some(entry) = state.subscriptions.get(&id) else {
                continue;
            };
            let session_id = entry.session.session_id();
            let result = if entry.session.is_closed() {
                Err(CoopError::SessionClosed)
            } else {
                match self
                    .snapshot_locked(&state, &entry.authority, (self.inner.clock)())
                    .await
                {
                    Ok(view) => match encode("gemot:collection-view", &view) {
                        // Snapshot preparation can cross a session deadline
                        // even though the coordinator excludes ledger updates.
                        // Recheck the retained lease at the enqueue boundary.
                        Ok(value) => match self
                            .allowed(
                                &state,
                                entry.authority.subject,
                                MootAccessLevel::Read,
                                (self.inner.clock)(),
                            )
                            .await
                        {
                            Ok(_) => entry
                                .authority
                                .check(&state.revocations, (self.inner.clock)())
                                .and_then(|()| {
                                    entry
                                        .session
                                        .try_send(
                                            &entry.callback,
                                            vec![Value::symbol("changed"), value],
                                        )
                                        .map_err(|_| CoopError::Backpressure)
                                }),
                            Err(error) => Err(error),
                        },
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(error),
                }
            };
            match result {
                Ok(()) => report.enqueued += 1,
                Err(_) => {
                    state.subscriptions.remove(&id);
                    cleanup_watcher(&mut state, session_id);
                    report.removed += 1;
                },
            }
        }
        report
    }
}

fn cleanup_watcher<B, P>(state: &mut OwnerState<B, P>, session: SessionId) {
    if !state
        .subscriptions
        .values()
        .any(|entry| entry.session.session_id() == session)
        && let Some(task) = state.watchers.remove(&session)
    {
        task.abort();
    }
}
fn remove_session<B, P>(state: &mut OwnerState<B, P>, session: SessionId) {
    state
        .subscriptions
        .retain(|_, entry| entry.session.session_id() != session);
    // This task may be removing its own already-completing handle. Dropping a
    // JoinHandle detaches; no extra task is spawned and no capability remains.
    state.watchers.remove(&session);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_domain_failures_do_not_disclose_local_storage_diagnostics() {
        let local = CoopError::Domain("cannot open /private/owner/store/objects.redb".into());
        assert!(local.to_string().contains("/private/owner/store"));
        let CallError::Broken(Value::String(remote)) = wire_error(local) else {
            panic!("wire refusal must be a bounded public message")
        };
        assert_eq!(remote, "domain operation failed");
        assert!(!remote.contains("objects.redb"));
    }
}
