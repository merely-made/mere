// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained source handoff and citation data, without an inference runtime.
use eidetic::models::recall::{
    InferencePassage, PassageSelector, RecallSourceId, RetainedSourceText,
};
use serde::{Deserialize, Serialize};
pub const RECALL_ASK_INTENT: &str = "distillery.inference/ask-recall-v1";
pub const ASK_PUBLISHED_INTENT: &str = "distillery.inference/ask-published-v1";
pub const PUBLISH_CORPUS_INTENT: &str = "distillery.inference/publish-corpus-v1";
pub const CHUNKER_VERSION: &str = "mere.unicode-chunks/v1";
/// Bound an inline publication below the application frame cap; reject rather than truncate.
pub const MAX_PUBLISHED_CORPUS_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishCorpusRequestV1 {
    /// Stable application/session namespace. The connected publisher owns its live heads.
    pub owner: String,
    pub sources: Vec<RetainedSourceText>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallConfig {
    pub chunk_chars: usize,
    pub max_passages: usize,
    /// Minimum non-padding characters for a host exact-quote annotation.
    /// Zero disables that path while retaining validated model markers.
    #[serde(default = "default_min_quote_chars")]
    pub min_quote_chars: usize,
    pub max_context_chars: usize,
    pub max_corpus_chars: usize,
}
fn default_min_quote_chars() -> usize {
    32
}
impl Default for RecallConfig {
    fn default() -> Self {
        Self {
            chunk_chars: 512,
            max_passages: 4,
            min_quote_chars: default_min_quote_chars(),
            max_context_chars: 12000,
            max_corpus_chars: 1000000,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallAskRequestV1 {
    pub ask: super::AskRequestV1,
    #[serde(default)]
    pub scope: Vec<RecallSourceId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppliedPassage {
    /// The numeric marker placed in the prompt, e.g. `[1]`.
    pub handle: u32,
    pub passage: InferencePassage,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StalePassage {
    pub source: RecallSourceId,
    pub indexed_version: String,
    pub current_version: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    pub handle: u32,
    pub passage: eidetic::ManifestId,
    pub basis: CitationBasis,
    /// Half-open Unicode scalar positions in the generated answer.
    pub answer_selector: PassageSelector,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationBasis {
    ModelMarker,
    VerbatimPassage,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecallEvidence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope: Vec<RecallSourceId>,
    pub config: RecallConfig,
    pub passages: Vec<SuppliedPassage>,
    pub stale: Vec<StalePassage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_citations: Vec<u32>,
}

/// Ask only against a namespace currently lent by this route session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedAskRequestV1 {
    pub owner: String,
    pub ask: super::AskRequestV1,
    #[serde(default)]
    pub scope: Vec<RecallSourceId>,
}
impl RecallConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.chunk_chars == 0
            || self.max_passages == 0
            || self.max_context_chars == 0
            || self.max_corpus_chars == 0
        {
            Err("recall limits must be greater than zero".into())
        } else {
            Ok(())
        }
    }
}
