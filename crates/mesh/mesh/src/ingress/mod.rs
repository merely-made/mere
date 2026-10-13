// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One opt-in admitted service: exact inventory and original signed-record
//! submission share two active slots and one four-session carrier queue.
//! Notochord settles session authority; MeshStore remains operation admission.

mod budget;
mod client;
mod lifecycle;
mod records;
mod session;

use crate::MeshStore;
use muniment::Backend;
use notochord::{LocalNetworkPolicy, RequestedAction, RevocationLedger};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use stickleback::inventory::{ObjectDigest, Snapshot};
use tokio::sync::Semaphore;

pub use client::{read_exact_inventory, submit_original};
pub use lifecycle::IngressHandle;

/// Dedicated admitted protocol, distinct from the older object-first example.
pub const INGRESS_ALPN: &str = "mere/mesh-retained-admitted/v1";
/// Domain owning the two service actions.
pub const INGRESS_DOMAIN: &str = "mere.mesh";
/// Namespace is additionally bound by Notochord's network identity.
pub const INGRESS_PATH: &str = "/services/retained-records";
/// Action that permits record submission, without granting inventory visibility.
pub const SUBMIT_ACTION: &str = "submit";
/// Separate action permitting the retained availability window to be read.
pub const INVENTORY_ACTION: &str = "inventory";
/// Global active-call ceiling, including handshake and all reply I/O.
pub const ACTIVE_CALLS: usize = 2;
/// Pending plus queued transport sessions, before service dequeue.
pub const QUEUED_CALLS: usize = 4;
/// Whole opaque record ceiling.
pub const OBJECT_BYTES: u32 = 16 * 1024;
/// Reply payload ceiling.
pub const RESULT_BYTES: u32 = 2 * 1024;
/// Exact inventory request/response payload ceiling.
pub const CONTROL_BYTES: u32 = 1024;

/// Owner-supplied policy, revocations, clock and finite call/retention lifetimes.
/// Policy changes require stopping and remounting this service.
pub struct IngressConfig {
    pub policy: LocalNetworkPolicy,
    pub revocations: RevocationLedger,
    pub call_timeout: Duration,
    pub retained_for: Duration,
    pub now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
}

#[derive(Debug, thiserror::Error)]
pub enum IngressError {
    #[error("invalid ingress configuration: {0}")]
    Config(&'static str),
    #[error("ingress protocol: {0}")]
    Protocol(&'static str),
    #[error(transparent)]
    Transport(#[from] transport::TransportError),
    #[error(transparent)]
    Frame(#[from] notochord::FrameError),
    #[error(transparent)]
    Object(#[from] stickleback::ObjectStreamError),
    #[error(transparent)]
    Inventory(#[from] stickleback::inventory::InventoryError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("ingress task: {0}")]
    Task(String),
    #[error("session not admitted: {0}")]
    Denied(String),
}

#[derive(Default)]
struct Counters {
    active: AtomicUsize,
    peak: AtomicUsize,
    completed: AtomicUsize,
    timeouts: AtomicUsize,
    denied: AtomicUsize,
    owner_calls: AtomicUsize,
    inserted: AtomicUsize,
    duplicates: AtomicUsize,
    refused: AtomicUsize,
    request_bytes: AtomicUsize,
    reply_bytes: AtomicUsize,
}

/// Service observations; active includes pre-admission stalled handshakes.
#[derive(Clone, Debug, Serialize)]
pub struct IngressStats {
    pub active: usize,
    pub peak_active: usize,
    pub completed: usize,
    pub timeouts: usize,
    pub denied_sessions: usize,
    pub owner_calls: usize,
    pub inserted: usize,
    pub duplicates: usize,
    pub refused_records: usize,
    pub request_frame_bytes: usize,
    pub reply_frame_bytes: usize,
}

/// An advisory owner result. Missing receipt cannot establish whether a record
/// committed; retrying identical original bytes uses existing duplicate behavior.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitResult {
    pub schema: String,
    pub record_digest: String,
    pub operation_id: Option<String>,
    pub outcome: String,
    pub reason: Option<String>,
}

/// Mesh-specific admitted service, retaining at most four accepted originals.
pub struct RecordIngress<B: Backend + Clone + Send + Sync + 'static> {
    store: MeshStore<B>,
    mesh: [u8; 32],
    config: IngressConfig,
    slots: Arc<Semaphore>,
    admitted: Arc<Mutex<u32>>,
    records: Mutex<records::Records>,
    counters: Counters,
}

impl<B: Backend + Clone + Send + Sync + 'static> RecordIngress<B> {
    pub fn new(
        store: MeshStore<B>,
        mesh: [u8; 32],
        mut config: IngressConfig,
    ) -> Result<Arc<Self>, IngressError> {
        if config.policy.network.0 != mesh {
            return Err(IngressError::Config("policy namespace"));
        }
        if config.call_timeout.is_zero()
            || config.call_timeout > Duration::from_secs(60)
            || config.retained_for.is_zero()
            || config.retained_for.as_millis() == 0
            || config.retained_for > Duration::from_secs(86_400)
        {
            return Err(IngressError::Config("lifetime"));
        }
        // Narrow the shared handshake codec's bounds for this small-object service.
        config.policy.limits.max_hello_bytes = config.policy.limits.max_hello_bytes.min(4096);
        config.policy.limits.max_reply_bytes =
            config.policy.limits.max_reply_bytes.min(RESULT_BYTES);
        Ok(Arc::new(Self {
            store,
            mesh,
            config,
            slots: Arc::new(Semaphore::new(ACTIVE_CALLS)),
            admitted: Arc::new(Mutex::new(0)),
            records: Mutex::new(records::Records::new(mesh)),
            counters: Counters::default(),
        }))
    }

    pub fn stats(&self) -> IngressStats {
        let get = |a: &AtomicUsize| a.load(Ordering::Relaxed);
        IngressStats {
            active: get(&self.counters.active),
            peak_active: get(&self.counters.peak),
            completed: get(&self.counters.completed),
            timeouts: get(&self.counters.timeouts),
            denied_sessions: get(&self.counters.denied),
            owner_calls: get(&self.counters.owner_calls),
            inserted: get(&self.counters.inserted),
            duplicates: get(&self.counters.duplicates),
            refused_records: get(&self.counters.refused),
            request_frame_bytes: get(&self.counters.request_bytes),
            reply_frame_bytes: get(&self.counters.reply_bytes),
        }
    }

    /// Local owner observation; not a network authorization bypass.
    pub fn retained_snapshot(&self) -> Result<Snapshot, IngressError> {
        self.records
            .lock()
            .unwrap()
            .snapshot((self.config.now_ms)())
    }

    /// Local owner retrieval for byte-preservation checks or an owning sender.
    pub fn retained_original(
        &self,
        digest: &ObjectDigest,
    ) -> Result<Option<Vec<u8>>, IngressError> {
        self.records
            .lock()
            .unwrap()
            .original(digest, (self.config.now_ms)())
    }
}

/// Build one explicit action triple for clients and owner policy.
pub fn requested_action(action: &str) -> RequestedAction {
    RequestedAction {
        domain: INGRESS_DOMAIN.into(),
        path: INGRESS_PATH.into(),
        action: action.into(),
    }
}

/// Version-one exact inventory request, under the separate inventory action.
pub fn inventory_request(mesh: [u8; 32]) -> Vec<u8> {
    let mut bytes = b"MIR1".to_vec();
    bytes.extend_from_slice(&mesh);
    bytes
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
