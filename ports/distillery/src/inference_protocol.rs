// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Application data for resident inference. This module never links ESP.
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[cfg(feature = "inference-wire")]
pub mod recall;

/// Exact model and template identity exposed to an application.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionIdentity {
    pub session_id: String,
    pub prompt_template_hash: String,
}

/// The first-party route an application asks through.
pub const INFERENCE_ASK_ROUTE: &str = "inference.ask";

/// How often a session is told the run table moved. Streamed fragments
/// reach a client at most this often (plan R5).
pub const INFERENCE_ASK_NOTICE_POLL: Duration = Duration::from_millis(100);

/// Schema of the run table served as the card's media.
pub const INFERENCE_RUNS_SCHEMA: &str = "distillery.inference-runs/v1";

/// Start a run. Payload: [`AskRequestV1`] as JSON.
pub const ASK_INTENT: &str = "distillery.inference/ask-v1";

/// Cancel a run. Payload: [`CancelRequestV1`] as JSON.
pub const CANCEL_INTENT: &str = "distillery.inference/cancel-v1";
/// Pause a durable run, retaining its token checkpoint.
pub const PAUSE_INTENT: &str = "distillery.inference/pause-v1";
/// Resume a retained run with its recorded generation settings.
pub const RESUME_INTENT: &str = "distillery.inference/resume-v1";
/// Ask with explicit sampling settings, still bounded by the owner cap.
pub const ASK_SAMPLED_INTENT: &str = "distillery.inference/ask-sampled-v1";

/// Sampling controls accepted by [`ASK_SAMPLED_INTENT`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SampledAskRequestV1 {
    pub ask: AskRequestV1,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: Option<f32>,
    pub seed: Option<u64>,
}

/// The `Ask` intent's payload. The caller names the run, so it can find the
/// run in the table without a reply channel: an intent result carries no data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskRequestV1 {
    /// Caller-chosen run name, unique within this lane.
    pub run: String,
    /// The raw question; a library provider renders its declared template.
    pub question: String,
}

/// The `Cancel` intent's payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelRequestV1 {
    /// The run to cancel.
    pub run: String,
}

/// Where a run stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStateV1 {
    /// Accepted, waiting behind earlier runs.
    Queued,
    /// Generating; `text` grows.
    Running,
    /// Stopped at a durable token boundary.
    Paused,
    /// Complete; `text` is the whole answer.
    Finished,
    /// Stopped by a cancel; `text` is what was generated before it.
    Cancelled,
    /// The provider refused or failed; see `error`.
    Failed,
}

impl RunStateV1 {
    /// Whether the run will change no further.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Finished | Self::Cancelled | Self::Failed)
    }
}

/// One run, as the route serves it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunV1<Session = SessionIdentity> {
    /// The caller's run name.
    pub run: String,
    /// The question asked.
    pub question: String,
    /// Where the run stands.
    pub state: RunStateV1,
    /// The answer so far, or in full once finished.
    pub text: String,
    /// Why a failed run failed.
    pub error: Option<String>,
    /// Exact session used for this run, absent for a stub or failed load.
    pub session: Option<Session>,
    #[serde(default)]
    pub tokens: usize,
    #[serde(default)]
    pub continuations: Vec<Continuation>,
    #[cfg(feature = "inference-wire")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recall: Option<recall::RecallEvidence>,
}

/// The run table, as the route serves it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRunsV1<Session = SessionIdentity> {
    /// [`INFERENCE_RUNS_SCHEMA`].
    pub schema: String,
    /// The provider's model id, once the actor has built it.
    pub model: Option<String>,
    /// Exact loaded session and template, absent for an unbound stub.
    pub session: Option<Session>,
    /// Provider construction error; asks fail with this instead of falling back.
    pub load_error: Option<String>,
    /// Runs in the order they were asked.
    pub runs: Vec<RunV1<Session>>,
}

/// One actual visit to a device, recorded before continuation starts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuation {
    pub device: String,
    pub at_token: usize,
    pub reason: String,
    pub first_differing_token: Option<usize>,
    #[serde(default)]
    pub compared_token_ids_hash: Option<String>,
}
