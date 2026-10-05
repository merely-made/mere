// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Meaning channel's sentence model, pinned (dynamics grammar plan, G2,
//! F56 and F58): which model, at which published revision, under which
//! licence, with the pooling and prefix its card asks for, and every
//! artifact's size and hashes. The manifest is `meaning_model.json` beside
//! this file, in the shape of distillery's `decoder-model.json`.
//!
//! The weights stay outside the tree, in a models directory the host names;
//! loading checks each artifact is present at its pinned size. The hashes
//! were matched to the revision's published listing when the pin was made
//! (the manifest's `matched`), and a receipt rechecks them.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use esp::embed::EmbedError;
use esp::embed::bert::Pooling;
use serde::Deserialize;

use super::meaning::{MeaningBackend, ProviderMeaning};

const MANIFEST: &str = include_str!("meaning_model.json");

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: String,
    model: MeaningModel,
}

/// One pinned artifact.
#[derive(Clone, Debug, Deserialize)]
pub struct ModelArtifact {
    pub file: String,
    pub bytes: u64,
    pub sha256: String,
    /// The file's git blob id, the hash Hugging Face lists for files kept
    /// outside LFS.
    #[serde(default)]
    pub git_blob: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelArtifacts {
    pub config: ModelArtifact,
    pub tokenizer: ModelArtifact,
    pub weights: ModelArtifact,
}

/// The pinned model, as the manifest records it.
#[derive(Clone, Debug, Deserialize)]
pub struct MeaningModel {
    pub model_id: String,
    pub revision: String,
    pub card: String,
    pub license: String,
    /// The model's directory under the host's models directory.
    pub directory: String,
    pub architecture: String,
    /// `mean` or `cls`.
    pub pooling: String,
    /// Prepended to every text before it is embedded.
    pub prefix: String,
    pub dimensions: usize,
    pub artifacts: ModelArtifacts,
}

impl MeaningModel {
    /// The model the Meaning channel runs (F56).
    pub fn pinned() -> &'static MeaningModel {
        static PINNED: OnceLock<MeaningModel> = OnceLock::new();
        PINNED.get_or_init(|| {
            let manifest: Manifest =
                serde_json::from_str(MANIFEST).expect("meaning_model.json parses");
            assert_eq!(manifest.schema, "pictograph.meaning-model/v1");
            manifest.model
        })
    }

    pub fn artifacts(&self) -> [&ModelArtifact; 3] {
        [
            &self.artifacts.config,
            &self.artifacts.tokenizer,
            &self.artifacts.weights,
        ]
    }

    pub fn pooling(&self) -> Result<Pooling, EmbedError> {
        match self.pooling.as_str() {
            "mean" => Ok(Pooling::Mean),
            "cls" => Ok(Pooling::Cls),
            other => Err(EmbedError::InvalidConfig(format!(
                "{}: unknown pooling {other:?}",
                self.model_id
            ))),
        }
    }

    /// The model's directory under `models`, once every artifact is there at
    /// its pinned size.
    pub fn check(&self, models: impl AsRef<Path>) -> Result<PathBuf, EmbedError> {
        let dir = models.as_ref().join(&self.directory);
        for artifact in self.artifacts() {
            let path = dir.join(&artifact.file);
            let bytes = std::fs::metadata(&path)
                .map_err(|error| {
                    EmbedError::InvalidConfig(format!(
                        "{} at {}: {error}",
                        self.model_id,
                        path.display()
                    ))
                })?
                .len();
            if bytes != artifact.bytes {
                return Err(EmbedError::InvalidConfig(format!(
                    "{}: {} is {bytes} bytes, pinned at {} (revision {})",
                    self.model_id,
                    path.display(),
                    artifact.bytes,
                    self.revision
                )));
            }
        }
        Ok(dir)
    }

    /// The model on the CPU, with its pooling, prefix and the model tuning.
    pub fn load_cpu(&self, models: impl AsRef<Path>) -> Result<ProviderMeaning, EmbedError> {
        let dir = self.check(models)?;
        let provider = esp::embed::bert::BertEmbeddingProvider::load(
            dir,
            esp::embed::bert::Device::ndarray(),
        )?
        .with_pooling(self.pooling()?);
        Ok(
            ProviderMeaning::new(Box::new(provider), MeaningBackend::ModelCpu)
                .with_prefix(&self.prefix),
        )
    }
}
