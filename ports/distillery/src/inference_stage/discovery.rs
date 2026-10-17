// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Prompt-free discovery and short-lived admission for automatic stage posts.

use super::*;

/// Library-owned callback: verify a row/session/backend and return the owner's
/// layer/KV budget. Shared/head/temporary memory has already been reserved.
pub type StageCatalog = dyn Fn(&StageRequest) -> Result<u64, String> + Send + Sync;

/// An authenticated live response, never durable resource or key authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageOffer {
    /// Exact public row/session/backend/range the peer queried.
    pub request: StageRequest,
    /// Runtime layer weights and declared context-KV payload budget.
    pub layer_bytes: u64,
    /// This worker's complete tensor-frame limit.
    pub max_hidden_bytes: usize,
    /// Actual worker capability floor, still enforced by its supervisor.
    pub requires: mesh::ResourceRequirements,
}

pub(super) struct StageReservation {
    poster: [u8; 32],
    input: mesh::BlobRef,
    until: std::time::Instant,
}

/// Discovery consults the existing verified library without loading tensors or
/// granting access to private prompts. Runtime loading verifies it again.
pub fn library_stage_catalog<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    loader: String,
    gpu: Option<GpuAccess>,
    layer_bytes: u64,
) -> Arc<StageCatalog> {
    Arc::new(move |request| {
        if request.loader != loader || gpu.as_ref().is_some_and(|g| g.foreground_requested()) {
            return Err("stage backend unavailable".into());
        }
        let resolved = pollster::block_on(esp::infer::library::resolve_library_session(
            &mut *library.blocking_lock(),
            request.row,
        ))
        .map_err(|e| e.to_string())?;
        resolved.capability(&loader).map_err(|e| e.to_string())?;
        if resolved
            .session(&loader)
            .id()
            .map_err(|e| e.to_string())?
            .to_string()
            != request.session
            || resolved.layer_artifacts().is_none()
        {
            return Err("stage row or exact session unavailable".into());
        }
        request
            .range
            .validate(
                resolved
                    .decoder_config()
                    .map_err(|e| e.to_string())?
                    .num_hidden_layers,
            )
            .map_err(|e| e.to_string())?;
        Ok(layer_bytes)
    })
}

impl StageService {
    fn discovery_permitted(&self, peer: [u8; 32]) -> Result<(), String> {
        if self.stopped.load(Ordering::SeqCst)
            || self.private_locked.load(Ordering::SeqCst)
            || self.busy.load(Ordering::SeqCst)
        {
            return Err("stage worker unavailable".into());
        }
        let state = self.state.read().unwrap();
        let board = state.board.as_ref().ok_or("stage board unavailable")?;
        for author in [self.author, peer] {
            if !board
                .devices()
                .master_of(&author)
                .is_some_and(|root| (self.readers)(root))
            {
                return Err("stage discovery requires current keyed graph readers".into());
            }
        }
        Ok(())
    }

    pub(super) fn reservation_permits(&self, job: &mesh::Job) -> bool {
        let Some(spec) = job.spec.as_ref() else {
            return true;
        };
        let Some(marker) = spec.input("reservation") else {
            return true;
        };
        spec.input("stage") == Some(marker)
            && self.reservation.lock().unwrap().as_ref().is_some_and(|r| {
                r.poster == job.posted_by
                    && &r.input == marker
                    && r.until > std::time::Instant::now()
            })
    }

    pub(super) async fn serve_discovery(
        &self,
        peer: [u8; 32],
        call: StageCall,
        stream: &mut transport::P2pandaStream,
    ) -> Result<(), String> {
        if call.operation == StageOperation::ReleaseReservation {
            let input = mesh::BlobRef::blake3(
                &serde_json::to_vec(&call.request).map_err(|e| e.to_string())?,
            );
            {
                let mut pending = self.reservation.lock().unwrap();
                if pending
                    .as_ref()
                    .is_some_and(|r| r.poster == peer && r.input == input)
                {
                    pending.take();
                }
            }
            notochord::write_frame(stream, b"released", CONTROL_BYTES as u32)
                .await
                .map_err(|e| e.to_string())?;
            return stream.shutdown().await.map_err(|e| e.to_string());
        }
        self.discovery_permitted(peer)?;
        let catalog = self
            .catalog
            .read()
            .unwrap()
            .clone()
            .ok_or("stage catalog not composed")?;
        let request = call.request.clone();
        if self
            .catalog_busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err("stage catalog busy".into());
        }
        let permit = BusyPermit(self.catalog_busy.clone());
        let layer_bytes = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            catalog(&request)
        })
        .await
        .map_err(|e| e.to_string())??;
        self.discovery_permitted(peer)?;
        let response = if call.operation == StageOperation::Reserve {
            let input = mesh::BlobRef::blake3(
                &serde_json::to_vec(&call.request).map_err(|e| e.to_string())?,
            );
            let mut pending = self.reservation.lock().unwrap();
            if pending.as_ref().is_some_and(|r| {
                r.until > std::time::Instant::now() && (r.poster != peer || r.input != input)
            }) {
                return Err("stage preparation already reserved".into());
            }
            *pending = Some(StageReservation {
                poster: peer,
                input,
                until: std::time::Instant::now() + self.settings.call_timeout,
            });
            b"reserved".to_vec()
        } else {
            serde_json::to_vec(&StageOffer {
                request: call.request,
                layer_bytes,
                max_hidden_bytes: self.settings.max_hidden_bytes,
                requires: self.settings.requires,
            })
            .map_err(|e| e.to_string())?
        };
        notochord::write_frame(stream, &response, CONTROL_BYTES as u32)
            .await
            .map_err(|e| e.to_string())?;
        stream.shutdown().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub(super) async fn query(
    transport: &P2pandaTransport,
    peer: PeerID,
    request: StageRequest,
    operation: StageOperation,
    deadline: Duration,
) -> Result<Vec<u8>, String> {
    tokio::time::timeout(deadline, async {
        let mut stream = transport
            .connect(peer, Alpn::new(STAGE_ALPN))
            .await
            .map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec(&StageCall {
            credential: Vec::new(),
            request,
            start: 0,
            operation,
        })
        .map_err(|e| e.to_string())?;
        notochord::write_frame(&mut stream, &bytes, CONTROL_BYTES as u32)
            .await
            .map_err(|e| e.to_string())?;
        notochord::read_frame(&mut stream, CONTROL_BYTES as u32)
            .await
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|_| "stage discovery timed out".to_string())?
}
