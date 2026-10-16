// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retry a failed stage placement from a durably committed token boundary.

use esp::infer::provider::{ExecutionPlacement, GenerationCheckpoint, SessionIdentity};
use esp::infer::{GenerationRequest, InferError, InferenceProvider, ModelCapability};
use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};

/// Lazy same-session loader called only when stage placement is unavailable.
pub type StreamingLoader = dyn Fn() -> Result<Box<dyn InferenceProvider>, InferError> + Send + Sync;

/// The fallback loader is lazy: remote stages need not fit in local memory.
/// It must stream the same row, format and backend class. A changed model,
/// template or loader still refuses the checkpoint through the ordinary guard.
pub struct StageFallbackProvider {
    staged: Mutex<Option<Box<dyn InferenceProvider>>>,
    capability: ModelCapability,
    identity: Option<SessionIdentity>,
    streaming: Arc<StreamingLoader>,
    device: String,
}

impl StageFallbackProvider {
    /// Retain the staged provider until it fails, then release its tensors
    /// before constructing a streamed replacement. Calls are serialized.
    pub fn new(
        staged: Box<dyn InferenceProvider>,
        streaming: Arc<StreamingLoader>,
        device: String,
    ) -> Result<Self, InferError> {
        if staged.session_identity().is_none() || device.trim().is_empty() {
            return Err(InferError::InvalidConfig(
                "stage fallback requires an exact session and device provenance".into(),
            ));
        }
        let capability = staged.capability().clone();
        let identity = staged.session_identity();
        Ok(Self {
            staged: Mutex::new(Some(staged)),
            capability,
            identity,
            streaming,
            device,
        })
    }
}

impl InferenceProvider for StageFallbackProvider {
    fn capability(&self) -> &ModelCapability {
        &self.capability
    }
    fn session_identity(&self) -> Option<SessionIdentity> {
        self.identity.clone()
    }
    fn generate_checkpointed(
        &self,
        request: &GenerationRequest,
        resume: Option<&GenerationCheckpoint>,
        commit: &mut dyn FnMut(&GenerationCheckpoint) -> Result<(), InferError>,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<GenerationCheckpoint, InferError> {
        let mut latest = resume.cloned();
        let mut commit_failed = false;
        let mut staged = self
            .staged
            .lock()
            .map_err(|_| InferError::Backend("staged provider poisoned".into()))?;
        let Some(provider) = staged.as_ref() else {
            drop(staged);
            let provider = (self.streaming)()?;
            if provider.session_identity() != self.session_identity()
                || provider.capability() != self.capability()
            {
                return Err(InferError::InvalidRequest(
                    "streaming fallback belongs to a different exact session".into(),
                ));
            }
            let mut result = provider.generate_checkpointed(
                request,
                resume,
                &mut |cp| {
                    let mut cp = cp.clone();
                    if cp.placements.is_empty() {
                        cp.placements.push(ExecutionPlacement {
                            at_token: 0,
                            device: self.device.clone(),
                            reason: "stages unavailable; local layer streaming".into(),
                        });
                    }
                    commit(&cp)
                },
                stop,
            )?;
            if result.placements.is_empty() {
                result.placements.push(ExecutionPlacement {
                    at_token: 0,
                    device: self.device.clone(),
                    reason: "stages unavailable; local layer streaming".into(),
                });
            }
            return Ok(result);
        };
        let result = provider.generate_checkpointed(
            request,
            resume,
            &mut |cp| {
                if let Err(error) = commit(cp) {
                    commit_failed = true;
                    return Err(error);
                }
                latest = Some(cp.clone());
                Ok(())
            },
            stop,
        );
        let Err(InferError::StageUnavailable(_)) = &result else {
            return result;
        };
        if commit_failed {
            return result;
        }
        let Some(mut cp) = latest else {
            return result;
        };
        // Release the staged embedding/head before the streaming loader allocates
        // its replacement. This placement is retired until the host replans it.
        staged.take();
        drop(staged);
        // Recommit the exact token/draw boundary with provenance before loading
        // any replacement tensors or spending another random draw.
        cp.placements.push(ExecutionPlacement {
            at_token: cp.token_ids.len(),
            device: self.device.clone(),
            reason: "leased stage unavailable; resumed by local layer streaming".into(),
        });
        commit(&cp)?;
        if stop() {
            return Ok(cp);
        }
        let provider = (self.streaming)()?;
        if provider.session_identity() != self.session_identity()
            || provider.capability() != self.capability()
        {
            return Err(InferError::InvalidRequest(
                "streaming fallback belongs to a different exact session".into(),
            ));
        }
        provider.generate_checkpointed(request, Some(&cp), commit, stop)
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
                    return Err(InferError::Backend("checkpoint text prefix changed".into()));
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
