// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Deferred GPU loading. Host buffers survive between asks; device tensors
//! live only inside an exclusive foreground guard and are dropped before it.

use super::InferenceLane;
use crate::gpu::GpuAccess;
use esp::infer::decoder::{
    DecoderDevice, DecoderGpuKind, GpuAdapterFacts, GpuDeviceType, probe_gpu_adapter,
};
use esp::infer::library::ResolvedInferenceRow;
use esp::infer::provider::SessionIdentity;
use esp::infer::{GenerationRequest, InferError, InferenceProvider, ModelCapability};
use std::ops::ControlFlow;
use std::sync::Arc;

/// Probe the first discrete adapter before constructing the lazy device.
pub fn discrete_gpu_inference_device() -> Result<(DecoderDevice, GpuAdapterFacts), String> {
    let kind = DecoderGpuKind::DiscreteGpu(0);
    let facts = probe_gpu_adapter(kind.clone()).map_err(|error| error.to_string())?;
    if facts.device_type != GpuDeviceType::Discrete || !facts.matched_requested_class {
        return Err(format!(
            "inference requested a discrete GPU, but the runtime would bind {facts}"
        ));
    }
    Ok((DecoderDevice::wgpu(kind), facts))
}

/// Probe and compose on the actor thread. Resolution creates host buffers;
/// every ask reclaims the trainer before constructing any GPU model tensors.
pub fn open_gpu_library_lane<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    row: eidetic::ManifestId,
    max_tokens: usize,
    access: GpuAccess,
) -> InferenceLane {
    let loading = access.request();
    InferenceLane::try_open_with_max_tokens(max_tokens, move || {
        let _loading = loading;
        let (device, facts) = discrete_gpu_inference_device().map_err(InferError::Backend)?;
        let resolved = {
            let mut store = library.blocking_lock();
            pollster::block_on(esp::infer::library::resolve_library_session(
                &mut *store,
                row,
            ))?
        };
        let capability = resolved.capability("burn-wgpu")?;
        let session = resolved.session("burn-wgpu");
        let identity = SessionIdentity {
            session_id: session.id()?.to_string(),
            prompt_template_hash: session.prompt_template_hash.to_string(),
        };
        // Facts come from the same explicit adapter selection as the device.
        eprintln!("resident inference GPU: {facts}");
        Ok(Box::new(GpuLibraryProvider {
            resolved,
            device,
            access,
            max_tokens,
            capability,
            identity,
        }))
    })
}

struct GpuLibraryProvider {
    resolved: ResolvedInferenceRow,
    device: DecoderDevice,
    access: GpuAccess,
    max_tokens: usize,
    capability: ModelCapability,
    identity: SessionIdentity,
}

impl InferenceProvider for GpuLibraryProvider {
    fn capability(&self) -> &ModelCapability {
        &self.capability
    }
    fn session_identity(&self) -> Option<SessionIdentity> {
        Some(self.identity.clone())
    }

    fn generate_streaming(
        &self,
        request: &GenerationRequest,
        on_token: &mut dyn FnMut(&str) -> ControlFlow<()>,
    ) -> Result<String, InferError> {
        self.generate_streaming_controlled(request, on_token, &mut || false)
    }

    fn generate_streaming_controlled(
        &self,
        request: &GenerationRequest,
        on_token: &mut dyn FnMut(&str) -> ControlFlow<()>,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<String, InferError> {
        let reservation = self.access.request();
        let Some(_guard) = reservation.acquire(cancelled) else {
            return Ok(String::new());
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let provider = self
                .resolved
                .load(&self.device, "burn-wgpu", self.max_tokens)?;
            provider.generate_streaming_controlled(request, on_token, cancelled)
        }))
        .unwrap_or_else(|_| {
            Err(InferError::Backend(
                "GPU model load or generation panicked".into(),
            ))
        });
        self.device
            .sync()
            .map_err(|error| InferError::Backend(format!("GPU synchronize: {error}")))?;
        result
        // `provider` drops before `_guard`, including on failure or unwind.
    }
}
