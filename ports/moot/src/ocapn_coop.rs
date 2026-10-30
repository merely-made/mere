// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Optional, fixed-collection live interaction over an already admitted session.
//!
//! OCapN identities and arguments confer no Gemot authority. This adapter keeps
//! the admitted subject/epoch, current session delegation and current Gemot
//! membership/typed capability checks separate. Its coordinator must also own
//! host ingress and policy changes; writes through cloned stores outside that
//! coordinator are outside this adapter's serialization contract.

#[path = "ocapn_coop/object.rs"]
mod object;
pub use object::EmissionReport;

use crate::coop::{Access, Grant, LifecycleReport, Membership, Reading, Verdict};
use gemot::moot::{
    CollectionChange, CollectionId, CollectionRef, CollectionView, ContributionRef, Moot,
    MootAccessLevel, MootAuthorizationInputs, MootAuthorizationProvider, MootAuthorizationRequest,
    MootCommandReceipt, MootGroup, MootOutboundOperation, collection_cap,
};
use mien::GateDecision;
use muniment::Backend;
use murm_ocapn::{Capability, SessionHandle, SessionId, Value};
use notochord::{
    AdmittedSecureSession, AuthorityLapse, NetworkId, RetainedAuthority, RevocationLedger,
    SECURE_WIRE_VERSION,
};
use personae::IdentityProvider;
use serde::{Deserialize, Serialize};
use servitor::cap_path;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;

pub const SERVICE_PROTOCOL: &[u8] = b"mere/gemot-coop/ocapn-suite-31f0b806/v1";
pub const SERVICE_DOMAIN: &str = "mere.gemot";
pub const SERVICE_PATH: &str = "/services/gemot-coop";
pub const SERVICE_ACTION: &str = "connect";
const MAX_HEADS: usize = 32;
const MAX_MEMBERS: usize = 128;
const MAX_NAME_BYTES: usize = 256;
const MAX_VIEW_BYTES: usize = 32 * 1024;
const MAX_SUBSCRIPTIONS: usize = 32;
const MAX_SUBSCRIPTIONS_PER_SESSION: usize = 4;
const MAX_WATCHED_SESSIONS: usize = 16;

/// Host-only immutable admission context. It is never serialized or parsed from
/// remote arguments. Construct a fresh context and object for every admission.
#[derive(Clone)]
pub struct MootSessionAuthority {
    subject: [u8; 32],
    epoch: [u8; 32],
    network: NetworkId,
    retained: RetainedAuthority,
}
impl MootSessionAuthority {
    /// Extract the conclusion from the actual secure admission result. A
    /// service grant permits using this adapter; Gemot still checks each method.
    pub fn from_admitted<S>(
        admitted: &AdmittedSecureSession<S>,
        expected_network: NetworkId,
    ) -> Result<Self, CoopError> {
        let session = &admitted.session;
        let action = &session.principal.action;
        if session.claims.wire_version != SECURE_WIRE_VERSION
            || session.claims.network != expected_network
            || session.claims.subject != session.principal.subject
            || session.claims.action != *action
            || action.domain != SERVICE_DOMAIN
            || action.path != SERVICE_PATH
            || action.action != SERVICE_ACTION
            || session.facts.protocol != SERVICE_PROTOCOL
            || admitted.secure_channel.protocol != SERVICE_PROTOCOL
        {
            return Err(CoopError::AdmissionContext);
        }
        Ok(Self {
            subject: session.principal.subject,
            epoch: session.principal.session_id,
            network: expected_network,
            retained: RetainedAuthority::from_admitted(session),
        })
    }
    pub fn subject(&self) -> [u8; 32] {
        self.subject
    }
    pub fn epoch(&self) -> [u8; 32] {
        self.epoch
    }
    pub fn network(&self) -> NetworkId {
        self.network
    }
    fn same_admission(&self, other: &Self) -> bool {
        self.subject == other.subject && self.epoch == other.epoch && self.network == other.network
    }
    fn check(&self, ledger: &RevocationLedger, at_ms: u64) -> Result<(), CoopError> {
        if let Some(lapse) = self.retained.lapse(ledger, at_ms) {
            return Err(match lapse {
                AuthorityLapse::Revoked => CoopError::Revoked,
                AuthorityLapse::Expired { .. } => CoopError::Expired,
            });
        }
        if !self.retained.covers(SERVICE_PATH, SERVICE_ACTION, at_ms) {
            return Err(CoopError::AdmissionContext);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoopError {
    AdmissionContext,
    SessionClosed,
    Revoked,
    Expired,
    NotMember,
    ReadOnly,
    CapabilityDenied,
    CollectionMissing,
    StaleHeads,
    Limit,
    Malformed,
    Backpressure,
    Domain(String),
}
impl fmt::Display for CoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CoopError {}
impl From<gemot::moot::MootError> for CoopError {
    fn from(error: gemot::moot::MootError) -> Self {
        match error {
            gemot::moot::MootError::StaleCollectionHeads => Self::StaleHeads,
            _ => Self::Domain(error.to_string()),
        }
    }
}

/// A bounded projection of the configured collection, not a second live state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionSnapshot {
    pub moot: [u8; 32],
    pub collection: [u8; 32],
    pub name: String,
    pub heads: Vec<[u8; 32]>,
    pub selected: Vec<[u8; 32]>,
    pub effective_selected: Vec<[u8; 32]>,
    pub lifecycle: LifecycleReport,
}

/// One fixed-collection membership intention. No subject, identity, clock,
/// arbitrary record, collection selector or method expression is accepted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipProposal {
    pub expected_heads: Vec<[u8; 32]>,
    pub contribution: [u8; 32],
    pub included: bool,
}

/// Attribution of the accepted domain command. The host is the actual author;
/// the remote admitted subject submitted an intention, not its signing key.
/// Submitter attribution is this returned/local receipt, not a new field in
/// Gemot's durable event. A host may retain it in its existing audit path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalReceipt {
    pub submitted_by: [u8; 32],
    pub authored_by: [u8; 32],
    pub session_epoch: [u8; 32],
    pub operation: [u8; 32],
    pub heads: Vec<[u8; 32]>,
    pub publication_queued: bool,
}

/// Ordinary Gemot output for the host's existing publication path.
pub struct AcceptedProposal {
    pub receipt: ProposalReceipt,
    pub domain_receipt: MootCommandReceipt,
    pub outbound: MootOutboundOperation,
}

/// Nonblocking host publication hook. A false result reports queue refusal;
/// the operation is already durable and can be recovered using Gemot::outbound.
/// The trusted implementation must perform bounded, nonblocking work and must
/// not reenter this owner's coordinator. The adapter cannot enforce an
/// arbitrary host implementation's synchronous execution time.
pub trait PublicationHook: Send + Sync + 'static {
    fn offer(&self, receipt: &MootCommandReceipt, operation: &MootOutboundOperation) -> bool;
}

struct Subscription {
    authority: MootSessionAuthority,
    session: SessionHandle,
    callback: Capability,
}

/// Host authority and data serialized with every adapter operation. The host
/// must apply ingested membership, constitution, delegation and revocation
/// updates through CollectionOwner::host_update rather than cloned store handles.
pub struct OwnerState<B, P> {
    pub moot: Moot<B>,
    pub identity: P,
    pub revocations: RevocationLedger,
    subscriptions: BTreeMap<u64, Subscription>,
    watchers: HashMap<SessionId, JoinHandle<()>>,
    next_subscription: u64,
}
impl<B, P> OwnerState<B, P> {
    pub fn new(moot: Moot<B>, identity: P, revocations: RevocationLedger) -> Self {
        Self {
            moot,
            identity,
            revocations,
            subscriptions: BTreeMap::new(),
            watchers: HashMap::new(),
            next_subscription: 1,
        }
    }
}
impl<B, P> Drop for OwnerState<B, P> {
    fn drop(&mut self) {
        for task in self.watchers.values() {
            task.abort();
        }
    }
}

struct Inner<B, P> {
    state: Mutex<OwnerState<B, P>>,
    collection: CollectionRef,
    network: NetworkId,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    publication: Arc<dyn PublicationHook>,
}

/// One configured collection and its existing owner, shared across live sessions.
pub struct CollectionOwner<B, P> {
    inner: Arc<Inner<B, P>>,
}
impl<B, P> Clone for CollectionOwner<B, P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct MethodMembership<'a> {
    group: &'a MootGroup,
    subject: [u8; 32],
    path: String,
    minimum: MootAccessLevel,
}
impl MootAuthorizationProvider for MethodMembership<'_> {
    fn inputs(&self, request: &MootAuthorizationRequest) -> MootAuthorizationInputs {
        let mut inputs = self.group.inputs(request);
        inputs.capability_covers = request.subject == self.subject
            && request.capability_path == self.path
            && self
                .group
                .member_snapshots()
                .iter()
                .any(|member| member.member == self.subject && member.access >= self.minimum);
        inputs
    }
}

impl<B, P> CollectionOwner<B, P>
where
    B: Backend + Clone + Send + Sync + 'static,
    P: IdentityProvider + Send + Sync + 'static,
{
    pub fn new(
        state: OwnerState<B, P>,
        collection: CollectionId,
        network: NetworkId,
        clock: Arc<dyn Fn() -> u64 + Send + Sync>,
        publication: Arc<dyn PublicationHook>,
    ) -> Self {
        let collection = CollectionRef {
            moot_id: state.moot.moot_id().0,
            collection_id: collection,
        };
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(state),
                collection,
                network,
                clock,
                publication,
            }),
        }
    }
    /// Coordinate trusted host ingress and policy updates with reads, emissions
    /// and commits. Do not retain a cloned store for unsynchronized mutation.
    pub async fn host_update(&self) -> MutexGuard<'_, OwnerState<B, P>> {
        self.inner.state.lock().await
    }
    pub async fn subscription_count(&self) -> usize {
        self.inner.state.lock().await.subscriptions.len()
    }
    pub fn session_object(
        &self,
        authority: MootSessionAuthority,
    ) -> Result<Arc<dyn murm_ocapn::HostObject>, CoopError> {
        if authority.network != self.inner.network {
            return Err(CoopError::AdmissionContext);
        }
        Ok(Arc::new(object::CollectionSession {
            owner: self.clone(),
            authority,
        }))
    }
    async fn allowed(
        &self,
        state: &OwnerState<B, P>,
        subject: [u8; 32],
        minimum: MootAccessLevel,
        at_ms: u64,
    ) -> Result<MootAccessLevel, CoopError> {
        let group = state.moot.membership().await?;
        let access = group
            .member_snapshots()
            .iter()
            .find(|member| member.member == subject)
            .map(|member| member.access)
            .ok_or(CoopError::NotMember)?;
        if access < minimum {
            return Err(CoopError::ReadOnly);
        }
        let path = cap_path(&collection_cap(self.inner.collection));
        let provider = MethodMembership {
            group: &group,
            subject,
            path: path.clone(),
            minimum,
        };
        let request = MootAuthorizationRequest {
            subject,
            capability_path: path,
            at_ms,
        };
        match state
            .moot
            .authorize_current_capability(&provider, &request)
            .await?
        {
            GateDecision::Allow => Ok(access),
            GateDecision::Deny(_) => Err(CoopError::CapabilityDenied),
        }
    }
    async fn snapshot_locked(
        &self,
        state: &OwnerState<B, P>,
        authority: &MootSessionAuthority,
        at_ms: u64,
    ) -> Result<CollectionSnapshot, CoopError> {
        if authority.network != self.inner.network {
            return Err(CoopError::AdmissionContext);
        }
        authority.check(&state.revocations, at_ms)?;
        let access = self
            .allowed(state, authority.subject, MootAccessLevel::Read, at_ms)
            .await?;
        let view = state
            .moot
            .authorized_collection(self.inner.collection.collection_id, at_ms)
            .await?
            .ok_or(CoopError::CollectionMissing)?;
        snapshot(&state.moot, view, authority, access, at_ms).await
    }
    /// Trusted host read using an actual admission conclusion. Remote calls
    /// use `session_object`, which also checks runtime session lifetime.
    /// Authority is evaluated at the owner clock under the coordinator; work
    /// already authorized at that reading may finish as the clock advances.
    pub async fn read(
        &self,
        authority: &MootSessionAuthority,
    ) -> Result<CollectionSnapshot, CoopError> {
        self.read_bound(authority, None).await
    }
    async fn read_bound(
        &self,
        authority: &MootSessionAuthority,
        session: Option<&SessionHandle>,
    ) -> Result<CollectionSnapshot, CoopError> {
        let state = self.inner.state.lock().await;
        if session.is_some_and(SessionHandle::is_closed) {
            return Err(CoopError::SessionClosed);
        }
        self.snapshot_locked(&state, authority, (self.inner.clock)())
            .await
    }
    /// Trusted host proposal using an actual admission conclusion. Remote
    /// calls use the bound object and cannot supply this authority themselves.
    /// A command authorized at the coordinated owner-clock reading may finish
    /// in its existing domain commit if the session closes or time advances.
    pub async fn propose(
        &self,
        authority: &MootSessionAuthority,
        proposal: MembershipProposal,
    ) -> Result<AcceptedProposal, CoopError> {
        self.propose_bound(authority, proposal, None).await
    }
    async fn propose_bound(
        &self,
        authority: &MootSessionAuthority,
        proposal: MembershipProposal,
        session: Option<&SessionHandle>,
    ) -> Result<AcceptedProposal, CoopError> {
        if proposal.expected_heads.is_empty() || proposal.expected_heads.len() > MAX_HEADS {
            return Err(CoopError::Limit);
        }
        let state = self.inner.state.lock().await;
        if session.is_some_and(SessionHandle::is_closed) {
            return Err(CoopError::SessionClosed);
        }
        let at_ms = (self.inner.clock)();
        if authority.network != self.inner.network {
            return Err(CoopError::AdmissionContext);
        }
        authority.check(&state.revocations, at_ms)?;
        self.allowed(&state, authority.subject, MootAccessLevel::Write, at_ms)
            .await?;
        let author = state.identity.master_public_key().to_bytes();
        self.allowed(&state, author, MootAccessLevel::Write, at_ms)
            .await?;
        let domain_receipt = state
            .moot
            .set_collection_membership_for_identity(
                &state.identity,
                self.inner.collection,
                proposal.expected_heads,
                CollectionChange::SetMembership {
                    contribution: ContributionRef {
                        moot_id: self.inner.collection.moot_id,
                        share: proposal.contribution,
                    },
                    included: proposal.included,
                },
                at_ms,
            )
            .await?;
        let outbound = state.moot.outbound(&domain_receipt).await?;
        let view = state
            .moot
            .authorized_collection(self.inner.collection.collection_id, at_ms)
            .await?
            .ok_or(CoopError::CollectionMissing)?;
        let publication_queued = self.inner.publication.offer(&domain_receipt, &outbound);
        let receipt = ProposalReceipt {
            submitted_by: authority.subject,
            authored_by: author,
            session_epoch: authority.epoch,
            operation: domain_receipt.operation,
            heads: view.heads,
            publication_queued,
        };
        Ok(AcceptedProposal {
            receipt,
            domain_receipt,
            outbound,
        })
    }
}

async fn snapshot<B: Backend + Clone>(
    moot: &Moot<B>,
    view: CollectionView,
    authority: &MootSessionAuthority,
    access: MootAccessLevel,
    at_ms: u64,
) -> Result<CollectionSnapshot, CoopError> {
    if view.name.len() > MAX_NAME_BYTES
        || view.heads.len() > MAX_HEADS
        || view.selected.len() > MAX_MEMBERS
        || view.effective_selected.len() > MAX_MEMBERS
    {
        return Err(CoopError::Limit);
    }
    let group = moot.membership().await?;
    let mut lifecycle = LifecycleReport::new(Verdict::Joined, None, at_ms);
    lifecycle.membership = Some(Membership {
        members: u32::try_from(group.member_snapshots().len()).map_err(|_| CoopError::Limit)?,
        access: match access {
            MootAccessLevel::Pull => Access::Pull,
            MootAccessLevel::Read => Access::Read,
            MootAccessLevel::Write => Access::Write,
            MootAccessLevel::Manage => Access::Manage,
        },
    });
    lifecycle.grant = Some(Grant {
        expires_at_ms: authority.retained.deadline_ms(),
        expired_at_ms: None,
    });
    lifecycle.reading = Reading::Unknown;
    let result = CollectionSnapshot {
        moot: view.collection.moot_id,
        collection: view.collection.collection_id.0,
        name: view.name,
        heads: view.heads,
        selected: view.selected.into_iter().map(|c| c.share).collect(),
        effective_selected: view
            .effective_selected
            .into_iter()
            .map(|c| c.share)
            .collect(),
        lifecycle,
    };
    if serde_json::to_vec(&result)
        .map_err(|_| CoopError::Malformed)?
        .len()
        > MAX_VIEW_BYTES
    {
        return Err(CoopError::Limit);
    }
    Ok(result)
}
