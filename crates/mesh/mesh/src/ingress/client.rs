// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{
    CONTROL_BYTES, INVENTORY_ACTION, IngressError, OBJECT_BYTES, RESULT_BYTES, SUBMIT_ACTION,
    SubmitResult, inventory_request, requested_action,
};
use notochord::{HandshakeLimits, SessionHello, SessionReply};
use std::time::Duration;
use stickleback::inventory::Summary;
use stickleback::{ObjectStreamLimits, read_object, write_object};
use tokio::io::{AsyncRead, AsyncWrite};

async fn admit<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    hello: &SessionHello,
) -> Result<(), IngressError> {
    let limits = HandshakeLimits {
        max_hello_bytes: 4096,
        max_reply_bytes: RESULT_BYTES,
        ..HandshakeLimits::default()
    };
    let reply = notochord::initiate_session(stream, hello, &limits)
        .await
        .map_err(|e| IngressError::Task(e.to_string()))?;
    match reply {
        SessionReply::Accept { .. } => Ok(()),
        SessionReply::Reject { reason } => Err(IngressError::Denied(reason.to_string())),
    }
}

/// Read one complete exact snapshot after its distinct inventory permission is
/// admitted. The caller binds the hello to the actual dialed carrier connection.
pub async fn read_exact_inventory<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    hello: &SessionHello,
    mesh: [u8; 32],
    lifetime: Duration,
) -> Result<Summary, IngressError> {
    if hello.action != requested_action(INVENTORY_ACTION) || hello.network.0 != mesh {
        return Err(IngressError::Protocol("inventory hello action/namespace"));
    }
    let limits = ObjectStreamLimits::new(CONTROL_BYTES, lifetime)?;
    tokio::time::timeout(lifetime, async {
        admit(stream, hello).await?;
        write_object(stream, &inventory_request(mesh), limits).await?;
        let summary = Summary::decode(&read_object(stream, limits).await?)?;
        if summary.namespace() != mesh
            || !matches!(&summary, Summary::Exact(page) if page.start() == 0 && page.next_cursor().is_none()) {
            return Err(IngressError::Protocol("incomplete/non-exact inventory response"));
        }
        Ok(summary)
    }).await.map_err(|_| IngressError::Protocol("inventory call deadline"))?
}

/// Submit unchanged original bytes and obtain an advisory remote owner result.
/// This result does not become another local admission marker. A lost result is
/// ambiguous; retry the same original signed bytes and operation identity.
pub async fn submit_original<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    hello: &SessionHello,
    original: &[u8],
    lifetime: Duration,
) -> Result<SubmitResult, IngressError> {
    if hello.action != requested_action(SUBMIT_ACTION) {
        return Err(IngressError::Protocol("submit hello action"));
    }
    let object_limits = ObjectStreamLimits::new(OBJECT_BYTES, lifetime)?;
    let result_limits = ObjectStreamLimits::new(RESULT_BYTES, lifetime)?;
    if original.len() > OBJECT_BYTES as usize {
        return Err(IngressError::Protocol("original size"));
    }
    tokio::time::timeout(lifetime, async {
        admit(stream, hello).await?;
        let digest = blake3::hash(original);
        notochord::write_frame(stream, digest.as_bytes(), 32).await?;
        write_object(stream, original, object_limits).await?;
        let result: SubmitResult =
            serde_json::from_slice(&read_object(stream, result_limits).await?)?;
        if result.schema != "mere.admitted-record-result/v1"
            || result.record_digest != digest.to_hex().as_str()
            || !matches!(
                result.outcome.as_str(),
                "inserted" | "duplicate" | "hydrated-payload" | "refused"
            )
        {
            return Err(IngressError::Protocol("result identity/vocabulary"));
        }
        Ok(result)
    })
    .await
    .map_err(|_| IngressError::Protocol("submit call deadline"))?
}
