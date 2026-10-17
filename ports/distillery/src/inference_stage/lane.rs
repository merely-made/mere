// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Actor composition for stage placement with local streamed recovery.

use super::{StageChain, StageFallbackProvider};
use crate::inference::{InferenceLane, RunJournal, borrow_layer_source, library_layer_source};
use esp::infer::decoder::DecoderDevice;
use esp::infer::library::ResolvedInferenceRow;
use esp::infer::provider::{GenerationCheckpoint, SessionIdentity};
use esp::infer::{GenerationRequest, InferError, InferenceProvider, ModelCapability};
use std::ops::ControlFlow;
use std::sync::Arc;

/// The host supplies newly leased, admitted stages for this exact session.
/// Called on the actor thread for each generation/resume. The host may use
/// its Tokio handle to post/wait for mesh leases; no private prompt is passed.
pub type StageChainFactory =
    dyn Fn(&ResolvedInferenceRow) -> Result<Arc<StageChain>, InferError> + Send + Sync;

/// Coordinator placement. GPU construction retains the resident's foreground
/// trainer-reclaim guard through the last stage close and local tensor drain.
pub enum StagedLaneDevice {
    /// NdArray for legacy f32 rows, Flex for explicit qualified format rows.
    Cpu,
    #[cfg(feature = "inference-gpu")]
    /// Probed discrete GPU under the resident's existing foreground owner.
    Gpu(crate::gpu::GpuAccess),
}

/// Compose the ordinary resident actor/journal with an explicit leased stage
/// plan. This is an embedding API; it does not discover peers or grant them
/// keys. Existing journals still enforce exact-session resume and lock pause.
pub fn open_staged_library_lane_with_journal<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    row: eidetic::ManifestId,
    max_tokens: usize,
    device: StagedLaneDevice,
    stages: Arc<StageChainFactory>,
    device_label: String,
    journal: Option<Arc<dyn RunJournal>>,
) -> Result<InferenceLane, String> {
    open_library_lane(
        library,
        row,
        max_tokens,
        device,
        stages,
        device_label,
        journal,
        None,
    )
}

/// Select whole-model residence within the owner's payload budget, otherwise
/// use discovered leased stages, then the existing same-session streaming rung.
pub fn open_automatic_library_lane_with_journal<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    row: eidetic::ManifestId,
    max_tokens: usize,
    device: StagedLaneDevice,
    stages: Arc<StageChainFactory>,
    local_resident_bytes: u64,
    device_label: String,
    journal: Option<Arc<dyn RunJournal>>,
) -> Result<InferenceLane, String> {
    open_library_lane(
        library,
        row,
        max_tokens,
        device,
        stages,
        device_label,
        journal,
        Some(local_resident_bytes),
    )
}

fn open_library_lane<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    row: eidetic::ManifestId,
    max_tokens: usize,
    device: StagedLaneDevice,
    stages: Arc<StageChainFactory>,
    device_label: String,
    journal: Option<Arc<dyn RunJournal>>,
    local_resident_bytes: Option<u64>,
) -> Result<InferenceLane, String> {
    InferenceLane::try_open_with_journal(
        max_tokens,
        move || {
            let resolved = {
                let mut store = library.blocking_lock();
                pollster::block_on(esp::infer::library::resolve_library_session(
                    &mut *store,
                    row,
                ))?
            };
            let (device, loader, gpu) = match device {
                StagedLaneDevice::Cpu if resolved.weight_format().is_some() => {
                    (DecoderDevice::flex(), "burn-flex", None)
                },
                StagedLaneDevice::Cpu => (DecoderDevice::ndarray(), "burn-ndarray", None),
                #[cfg(feature = "inference-gpu")]
                StagedLaneDevice::Gpu(access) => {
                    let (device, _) = crate::inference::discrete_gpu_inference_device()
                        .map_err(InferError::Backend)?;
                    (device, "burn-wgpu", Some(access))
                },
            };
            let capability = resolved.capability(loader)?;
            let session = resolved.session(loader);
            let identity = SessionIdentity {
                session_id: session.id()?.to_string(),
                prompt_template_hash: session.prompt_template_hash.to_string(),
            };
            let source = library_layer_source(library, &resolved);
            if source.is_none() || device_label.trim().is_empty() {
                return Err(InferError::InvalidConfig(
                    "stage lane requires indexed weights and a device label".into(),
                ));
            }
            Ok(Box::new(LibraryStages {
                resolved: Arc::new(resolved),
                source,
                device,
                loader,
                max_tokens,
                stages,
                gpu,
                capability,
                identity,
                device_label,
                local_resident_bytes,
            }))
        },
        journal,
    )
}

struct LibraryStages {
    resolved: Arc<ResolvedInferenceRow>,
    source: Option<Arc<dyn esp::infer::decoder::streaming::LayerSource>>,
    device: DecoderDevice,
    loader: &'static str,
    max_tokens: usize,
    stages: Arc<StageChainFactory>,
    gpu: Option<crate::gpu::GpuAccess>,
    capability: ModelCapability,
    identity: SessionIdentity,
    device_label: String,
    local_resident_bytes: Option<u64>,
}

#[derive(Debug)]
struct ActorStages(Arc<StageChain>);
impl esp::infer::decoder::stage::StageExecutor for ActorStages {
    fn returns_logits(&self) -> bool {
        true
    }
    fn session_id(&self) -> Option<&str> {
        esp::infer::decoder::stage::StageExecutor::session_id(self.0.as_ref())
    }
    fn forward(
        &self,
        hidden: esp::infer::decoder::stage::StageHidden,
        start: usize,
    ) -> Result<esp::infer::decoder::stage::StageHidden, InferError> {
        esp::infer::decoder::stage::StageExecutor::forward(self.0.as_ref(), hidden, start)
    }
}

struct FinishStages(Arc<StageChain>);
impl Drop for FinishStages {
    fn drop(&mut self) {
        self.0.finish();
    }
}

impl InferenceProvider for LibraryStages {
    fn capability(&self) -> &ModelCapability {
        &self.capability
    }
    fn session_identity(&self) -> Option<SessionIdentity> {
        Some(self.identity.clone())
    }
    fn generate_checkpointed(
        &self,
        request: &GenerationRequest,
        resume: Option<&GenerationCheckpoint>,
        commit: &mut dyn FnMut(&GenerationCheckpoint) -> Result<(), InferError>,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<GenerationCheckpoint, InferError> {
        let reservation = self.gpu.as_ref().map(|g| g.request());
        let _guard = match reservation.as_ref() {
            Some(request) => Some(request.acquire(stop).ok_or(InferError::Interrupted)?),
            None => None,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if self.local_resident_bytes.is_some_and(|budget| {
                budget > 0
                    && self
                        .resolved
                        .resident_payload_bytes()
                        .is_ok_and(|bytes| bytes <= budget)
            }) {
                let provider = self.resolved.load_resident_with_layer_source(
                    &self.device,
                    self.loader,
                    self.max_tokens,
                    borrow_layer_source(&self.source).ok_or_else(|| {
                        InferError::Backend("resident layers lack a library source".into())
                    })?,
                )?;
                let placement = esp::infer::provider::ExecutionPlacement {
                    at_token: resume.map_or(0, |cp| cp.token_ids.len()),
                    device: self.device_label.clone(),
                    reason: "whole model resident within owner payload budget".into(),
                };
                let stamp = |cp: &mut GenerationCheckpoint| {
                    if !cp.placements.contains(&placement) {
                        cp.placements.push(placement.clone());
                    }
                };
                let mut result = provider.generate_checkpointed(
                    request,
                    resume,
                    &mut |cp| {
                        let mut cp = cp.clone();
                        stamp(&mut cp);
                        commit(&cp)
                    },
                    stop,
                )?;
                stamp(&mut result);
                return Ok(result);
            }
            let chain = match (self.stages)(&self.resolved).and_then(|chain| {
                if let Err(error) = chain.wait_ready() {
                    chain.finish();
                    return Err(error);
                }
                Ok(chain)
            }) {
                Ok(chain) => chain,
                Err(InferError::StageUnavailable(_)) => {
                    let provider = self.resolved.load_with_layer_source(
                        &self.device,
                        self.loader,
                        self.max_tokens,
                        borrow_layer_source(&self.source),
                    )?;
                    let placement = esp::infer::provider::ExecutionPlacement {
                        at_token: resume.map_or(0, |cp| cp.token_ids.len()),
                        device: self.device_label.clone(),
                        reason: "stage plan unavailable; local layer streaming".into(),
                    };
                    let mut result = provider.generate_checkpointed(
                        request,
                        resume,
                        &mut |cp| {
                            let mut cp = cp.clone();
                            if !cp.placements.contains(&placement) {
                                cp.placements.push(placement.clone());
                            }
                            commit(&cp)
                        },
                        stop,
                    )?;
                    if !result.placements.contains(&placement) {
                        result.placements.push(placement);
                    }
                    return Ok(result);
                },
                Err(error) => return Err(error),
            };
            let placement = esp::infer::provider::ExecutionPlacement {
                at_token: resume.map_or(0, |cp| cp.token_ids.len()),
                device: self.device_label.clone(),
                reason: chain.placement_description(),
            };
            let _finish = FinishStages(chain.clone());
            let retired = chain.clone();
            let staged = self.resolved.load_with_stages(
                &self.device,
                self.loader,
                self.max_tokens,
                Box::new(ActorStages(chain)),
            )?;
            let resolved = self.resolved.clone();
            let source = self.source.clone();
            let device = self.device.clone();
            let loader = self.loader;
            let max_tokens = self.max_tokens;
            let fallback = StageFallbackProvider::new(
                Box::new(staged),
                Arc::new(move || {
                    retired.finish();
                    Ok(Box::new(resolved.load_with_layer_source(
                        &device,
                        loader,
                        max_tokens,
                        borrow_layer_source(&source),
                    )?))
                }),
                self.device_label.clone(),
            )?;
            let stamp = |cp: &mut GenerationCheckpoint| {
                if !cp.placements.contains(&placement) {
                    cp.placements.push(placement.clone());
                    cp.placements.sort_by_key(|visit| visit.at_token);
                }
            };
            let mut result = fallback.generate_checkpointed(
                request,
                resume,
                &mut |cp| {
                    let mut cp = cp.clone();
                    stamp(&mut cp);
                    commit(&cp)
                },
                stop,
            )?;
            stamp(&mut result);
            Ok(result)
        }))
        .unwrap_or_else(|_| Err(InferError::Backend("staged generation panicked".into())));
        // Includes panic/failure/pause. Foreground ownership ends only after
        // the scoped provider, its KV, and shared tensors have been dropped.
        self.device
            .sync()
            .map_err(|e| InferError::Backend(format!("staged coordinator drain: {e}")))?;
        result
    }
    fn generate_streaming(
        &self,
        request: &GenerationRequest,
        on_fragment: &mut dyn FnMut(&str) -> ControlFlow<()>,
    ) -> Result<String, InferError> {
        self.generate_streaming_controlled(request, on_fragment, &mut || false)
    }
    fn generate_streaming_controlled(
        &self,
        request: &GenerationRequest,
        on_fragment: &mut dyn FnMut(&str) -> ControlFlow<()>,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<String, InferError> {
        let mut emitted = String::new();
        let stopped = std::cell::Cell::new(false);
        self.generate_checkpointed(
            request,
            None,
            &mut |cp| {
                if !cp.text.starts_with(&emitted) {
                    return Err(InferError::Backend(
                        "staged checkpoint text prefix changed".into(),
                    ));
                }
                if cp.text.len() > emitted.len() {
                    stopped.set(on_fragment(&cp.text[emitted.len()..]).is_break());
                    emitted.clone_from(&cp.text);
                }
                Ok(())
            },
            &mut || stopped.get() || cancelled(),
        )
        .map(|cp| cp.text.clone())
    }
}
