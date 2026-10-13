// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use muniment::Backend;
use p2panda_core::cbor::{decode_cbor_strict, encode_cbor};
use std::sync::{Arc, Mutex, atomic::Ordering};
use stickleback::inventory::Summary;
use stickleback::{
    DropRecord, ObjectStreamLimits, ProcessOutcome, decode_operation_record, read_object,
    write_object,
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::time::{Instant, timeout_at};
use transport::{AcceptedSession, Alpn};

use super::{
    ACTIVE_CALLS, CONTROL_BYTES, INGRESS_ALPN, INVENTORY_ACTION, IngressError, OBJECT_BYTES,
    RESULT_BYTES, RecordIngress, SUBMIT_ACTION, SubmitResult, hex, inventory_request,
    requested_action,
};

struct AdmissionLease(Arc<Mutex<u32>>);
impl Drop for AdmissionLease {
    fn drop(&mut self) {
        *self.0.lock().unwrap() -= 1;
    }
}

struct Active<'a>(&'a super::Counters);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<B: Backend + Clone + Send + Sync + 'static> RecordIngress<B> {
    pub(super) async fn serve<S>(
        self: Arc<Self>,
        accepted: AcceptedSession<S>,
        arrived: Instant,
    ) -> Result<(), IngressError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let _permit = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| IngressError::Protocol("active call budget"))?;
        let active = self.counters.active.fetch_add(1, Ordering::Relaxed) + 1;
        self.counters.peak.fetch_max(active, Ordering::Relaxed);
        let _active = Active(&self.counters);
        let deadline = arrived + self.config.call_timeout;
        if Instant::now() >= deadline {
            self.counters.timeouts.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }
        let result = timeout_at(deadline, self.exchange(accepted, deadline)).await;
        self.counters.completed.fetch_add(1, Ordering::Relaxed);
        match result {
            Ok(result) => result,
            Err(_) => {
                self.counters.timeouts.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
        }
    }

    async fn exchange<S>(
        &self,
        accepted: AcceptedSession<S>,
        deadline: Instant,
    ) -> Result<(), IngressError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        if accepted.protocol != Alpn::new(INGRESS_ALPN) || !accepted.is_transport_authenticated() {
            return Err(IngressError::Protocol("carrier protocol/identity"));
        }
        let (stream, facts) = accepted.into_session();
        let mut stream = super::budget::BudgetStream::new(stream, &self.counters);
        let limits = self.config.policy.limits.clamped();
        let hello = notochord::read_frame(&mut stream, limits.max_hello_bytes).await?;
        // Sample owner time after the bounded hello read. Check and reserve the
        // policy's admitted-session count atomically, separately from raw slots.
        let (reply, principal, lease) = {
            let mut count = self.admitted.lock().unwrap();
            let (reply, outcome) = notochord::admit(
                &self.config.policy,
                &self.config.revocations,
                &hello,
                &facts,
                (self.config.now_ms)(),
                *count,
            );
            let lease = if outcome.is_ok() {
                *count += 1;
                Some(AdmissionLease(self.admitted.clone()))
            } else {
                None
            };
            (reply, outcome, lease)
        };
        let _lease = lease;
        notochord::write_frame(&mut stream, &reply, limits.max_reply_bytes).await?;
        let principal = match principal {
            Ok(principal) => principal,
            Err(_) => {
                self.counters.denied.fetch_add(1, Ordering::Relaxed);
                stream.shutdown().await?;
                return Ok(());
            },
        };
        if principal.action == requested_action(INVENTORY_ACTION) {
            let input = read_object(
                &mut stream,
                ObjectStreamLimits::new(CONTROL_BYTES, self.config.call_timeout)?,
            )
            .await?;
            if input != inventory_request(self.mesh) {
                return Err(IngressError::Protocol("inventory namespace/request"));
            }
            let snapshot = self.retained_snapshot()?;
            let output = Summary::Exact(snapshot.exact_page(0)?).encode();
            write_object(
                &mut stream,
                &output,
                ObjectStreamLimits::new(CONTROL_BYTES, self.config.call_timeout)?,
            )
            .await?;
            return Ok(());
        }
        if principal.action != requested_action(SUBMIT_ACTION) {
            return Err(IngressError::Protocol("action not served"));
        }
        let digest = notochord::read_frame(&mut stream, 32).await?;
        if digest.len() != 32 {
            return Err(IngressError::Protocol("expected record digest"));
        }
        let bytes = read_object(
            &mut stream,
            ObjectStreamLimits::new(OBJECT_BYTES, self.config.call_timeout)?,
        )
        .await?;
        let record_digest = *blake3::hash(&bytes).as_bytes();
        let mut result = SubmitResult {
            schema: "mere.admitted-record-result/v1".into(),
            record_digest: hex(&record_digest),
            operation_id: None,
            outcome: "refused".into(),
            reason: None,
        };
        if record_digest.as_slice() != digest {
            result.reason = Some("record digest mismatch".into());
        } else {
            let decoded: Result<p2panda_core::Operation<crate::MeshExt>, String> = (|| {
                let record: DropRecord =
                    decode_cbor_strict(bytes.as_slice()).map_err(|e| e.to_string())?;
                if encode_cbor(&record).map_err(|e| e.to_string())? != bytes {
                    return Err("noncanonical or trailing record bytes".into());
                }
                decode_operation_record::<crate::MeshExt>(&record)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "expected signed operation record".into())
            })();
            match decoded {
                Err(reason) => result.reason = Some(reason.chars().take(256).collect()),
                Ok(operation) => {
                    result.operation_id = Some(hex(operation.hash.as_bytes()));
                    if Instant::now() >= deadline {
                        return Err(IngressError::Protocol("deadline before owner"));
                    }
                    let now = (self.config.now_ms)();
                    let expires = now
                        .checked_add(self.config.retained_for.as_millis() as u64)
                        .ok_or(IngressError::Config("retention clock exhausted"))?;
                    self.counters.owner_calls.fetch_add(1, Ordering::Relaxed);
                    match self.store.accept_outcome(self.mesh, &operation).await {
                        Ok(outcome) => {
                            result.outcome = match outcome {
                                ProcessOutcome::Inserted { .. } => {
                                    self.counters.inserted.fetch_add(1, Ordering::Relaxed);
                                    "inserted"
                                },
                                ProcessOutcome::Duplicate => {
                                    self.counters.duplicates.fetch_add(1, Ordering::Relaxed);
                                    "duplicate"
                                },
                                ProcessOutcome::HydratedPayload => "hydrated-payload",
                            }
                            .into();
                            self.records.lock().unwrap().retain(bytes, now, expires)?;
                        },
                        Err(error) => {
                            result.reason = Some(error.to_string().chars().take(256).collect())
                        },
                    }
                },
            }
        }
        if result.outcome == "refused" {
            self.counters.refused.fetch_add(1, Ordering::Relaxed);
        }
        write_object(
            &mut stream,
            &serde_json::to_vec(&result)?,
            ObjectStreamLimits::new(RESULT_BYTES, self.config.call_timeout)?,
        )
        .await?;
        debug_assert!(self.counters.active.load(Ordering::Relaxed) <= ACTIVE_CALLS);
        Ok(())
    }
}
