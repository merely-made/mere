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

/// Explicit opt-in; existing asks keep their prose contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerFormat {
    #[default]
    Prose,
    ClaimSupportV1,
}

/// The exact bounded task given to the model, independent of its claimed truth.
/// Absent on historical v1 answers and ordinary prose asks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnswerContract {
    pub method: String,
    pub question: eidetic::ManifestId,
    pub prompt: eidetic::ManifestId,
    pub context: eidetic::ManifestId,
    pub max_claims: usize,
    /// Hash of the exact enforced decoding grammar; absent on historical tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_grammar: Option<eidetic::ManifestId>,
    /// Ordered exact question excerpts; absent on historical contracts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_facts: Option<eidetic::models::requested::RequestedFactSet>,
}

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
    #[serde(default, skip_serializing_if = "is_prose")]
    pub answer_format: AnswerFormat,
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
    /// Exact quote in a structured claim; source selector lives in support.
    ClaimQuotation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecallEvidence {
    #[serde(default, skip_serializing_if = "is_prose")]
    pub answer_format: AnswerFormat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_contract: Option<AnswerContract>,
    /// Reproducible retained assessment; current availability is reported by
    /// `stale` and must never overwrite this historical evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support: Option<eidetic::models::answer::AnswerSupport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<EmbeddingRecallEvidence>,
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
    #[serde(default, skip_serializing_if = "is_prose")]
    pub answer_format: AnswerFormat,
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

/// Owner-selected optional vector lane. It is unavailable without the resident's
/// explicit inference-embedding feature; absence preserves lexical-only recall.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingRecallSettings {
    pub provider: EmbeddingRecallProvider,
    pub max_index_bytes: usize,
    pub lexical_weight: u32,
    pub vector_weight: u32,
    pub rrf_k: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EmbeddingRecallProvider {
    LexicalSparse {
        dimensions: usize,
        token_ngram_orders: Vec<usize>,
    },
    /// An already admitted model artifact in the resident's existing library.
    Library { model: String },
}
impl EmbeddingRecallSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_index_bytes == 0
            || self.lexical_weight == 0
            || self.vector_weight == 0
            || self.rrf_k == 0
        {
            return Err(
                "embedding recall requires positive byte, ranking and fusion bounds".into(),
            );
        }
        match &self.provider {
            EmbeddingRecallProvider::LexicalSparse {
                dimensions,
                token_ngram_orders,
            } => {
                if *dimensions == 0
                    || *dimensions > u32::MAX as usize
                    || token_ngram_orders.is_empty()
                    || token_ngram_orders.contains(&0)
                {
                    return Err("invalid sparse embedding recall settings".into());
                }
            },
            EmbeddingRecallProvider::Library { model } => {
                eidetic::Hash::parse(model).map_err(|e| e.to_string())?;
            },
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingRecallEvidence {
    pub envelope: eidetic::ManifestId,
    pub index: eidetic::ManifestId,
    pub cache: String,
    pub vector_bytes: usize,
    pub persisted_bytes: usize,
    pub lexical_weight: u32,
    pub vector_weight: u32,
    pub rrf_k: u32,
}

fn is_prose(format: &AnswerFormat) -> bool {
    *format == AnswerFormat::Prose
}

#[cfg(test)]
mod answer_wire_tests {
    use super::*;
    #[test]
    fn existing_prose_asks_keep_their_wire_shape() {
        let raw = r#"{"owner":"one","ask":{"run":"r","question":"q"},"scope":[]}"#;
        let request: PublishedAskRequestV1 = serde_json::from_str(raw).unwrap();
        assert_eq!(request.answer_format, AnswerFormat::Prose);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(raw).unwrap(),
            serde_json::to_value(request).unwrap()
        );
    }
}
