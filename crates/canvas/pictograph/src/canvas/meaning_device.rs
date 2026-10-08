// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A sentence model on the host's own device for the Meaning channel
//! (feature `meaning-gpu`; dynamics grammar plan, G2: "Burn on the host
//! device, off-path").
//!
//! The model runs on the CubeCL device the host's [`PhysicsDevice`] already
//! registered from its renderer's handles, so physics, embeddings and the
//! renderer share one wgpu device and one CubeCL client. Native hosts only:
//! the wasm web build leaves the feature off and takes the lexical fallback.
//!
//! A host enabling this boots its device greedy (`netrender::TenantNeeds {
//! greedy: true }`, F31). CubeCL times kernels on the device whenever the
//! **adapter** offers `TIMESTAMP_QUERY` (cubecl-wgpu `runtime.rs`), so on a
//! device booted without it the model's first autotuned launch fails
//! validation on CubeCL's own thread and the run never lands. [`DeviceMeaning::load`]
//! refuses such a device with that reason rather than let it hang.

use std::path::Path;

use esp::embed::bert::Device;
use esp::embed::index_burn::{AFFINITY_GPU_MIN_ENTRIES, affinity_pairs_over_index};
use esp::embed::{EmbedError, EmbeddingProvider, VectorIndex};
use kernel::graph::NodeKey;

use super::PhysicsDevice;
use super::meaning::{Embedded, MeaningBackend, MeaningEngine, MeaningParams, embed_prefixed};
use super::meaning_model::MeaningModel;

/// Whether `device` can carry the model, and why not: CubeCL times kernels
/// on the device whenever the adapter offers `TIMESTAMP_QUERY`, so a device
/// that lacks a feature its adapter has fails the model's first launch.
pub fn check_meaning_device(device: &PhysicsDevice) -> Result<(), EmbedError> {
    let Some(features) = device.features() else {
        return Ok(());
    };
    if features.lacks_adapter_timestamps() {
        return Err(EmbedError::InvalidConfig(
            "the host's device lacks TIMESTAMP_QUERY, which its adapter offers; CubeCL times \
             kernels on the device whenever the adapter has it, so the model's first launch \
             would fail validation and never land. Boot the host device greedy \
             (netrender::TenantNeeds { greedy: true }; dynamics grammar plan, F31)"
                .to_string(),
        ));
    }
    Ok(())
}

/// The Burn device a host's physics device registered: the same CubeCL
/// client, never a new one.
pub fn host_meaning_device(device: &PhysicsDevice) -> Device {
    Device::new(device.client().device().clone())
}

/// A sentence model on a Burn wgpu device, with the pair search on the
/// device from [`AFFINITY_GPU_MIN_ENTRIES`] entries up (ESP's measured
/// crossover) and on the CPU below it.
pub struct DeviceMeaning {
    provider: Box<dyn EmbeddingProvider>,
    device: Device,
    params: MeaningParams,
    pair_threshold: usize,
    prefix: String,
}

impl DeviceMeaning {
    /// Load the model in `model_dir` (a HuggingFace layout) on the host's
    /// device.
    ///
    /// Refuses a device the model cannot run on ([`check_meaning_device`]).
    pub fn load(model_dir: impl AsRef<Path>, device: &PhysicsDevice) -> Result<Self, EmbedError> {
        check_meaning_device(device)?;
        Self::load_on(model_dir, host_meaning_device(device))
    }

    /// Load the model in `model_dir` on any Burn wgpu device, mean-pooled
    /// and unprefixed.
    pub fn load_on(model_dir: impl AsRef<Path>, device: Device) -> Result<Self, EmbedError> {
        let provider = esp::embed::bert::load_wgpu(model_dir, device.clone())?;
        Ok(Self {
            provider,
            device,
            params: MeaningParams::MODEL,
            pair_threshold: AFFINITY_GPU_MIN_ENTRIES,
            prefix: String::new(),
        })
    }

    /// The pinned model ([`MeaningModel::pinned`], F56) from the host's
    /// `models` directory, on the host's device, with its pooling and prefix.
    pub fn load_pinned(
        models: impl AsRef<Path>,
        device: &PhysicsDevice,
    ) -> Result<Self, EmbedError> {
        check_meaning_device(device)?;
        Self::load_model_on(MeaningModel::pinned(), models, host_meaning_device(device))
    }

    /// `model` from `models` on any Burn wgpu device, with its pooling and
    /// prefix.
    pub fn load_model_on(
        model: &MeaningModel,
        models: impl AsRef<Path>,
        device: Device,
    ) -> Result<Self, EmbedError> {
        let dir = model.check(models)?;
        let provider = esp::embed::bert::BertEmbeddingProvider::load(dir, device.clone())?
            .with_pooling(model.pooling()?);
        Ok(Self {
            provider: Box::new(provider),
            device,
            params: MeaningParams::MODEL,
            pair_threshold: AFFINITY_GPU_MIN_ENTRIES,
            prefix: model.prefix.clone(),
        })
    }

    /// Entry count at or above which pairs are found on the device (`0`:
    /// always).
    pub fn with_pair_threshold(mut self, threshold: usize) -> Self {
        self.pair_threshold = threshold;
        self
    }

    pub fn with_params(mut self, params: MeaningParams) -> Self {
        self.params = params;
        self
    }

    /// The device the model's tensors live on.
    pub fn device(&self) -> &Device {
        &self.device
    }
}

impl MeaningEngine for DeviceMeaning {
    fn backend(&self) -> MeaningBackend {
        MeaningBackend::ModelGpu
    }

    fn params(&self) -> MeaningParams {
        self.params
    }

    fn embed(&self, texts: &[&str]) -> Result<Embedded, EmbedError> {
        embed_prefixed(self.provider.as_ref(), &self.prefix, texts).map(Embedded::Dense)
    }

    /// The device's batched search from the threshold up; below it, the
    /// resumable row scan on the CPU.
    fn index_pairs(
        &self,
        index: &VectorIndex<NodeKey>,
        params: MeaningParams,
    ) -> Option<Vec<(NodeKey, NodeKey, f32)>> {
        (index.len() >= self.pair_threshold).then(|| {
            affinity_pairs_over_index(index, params.top_k, params.min_similarity, &self.device)
        })
    }
}
