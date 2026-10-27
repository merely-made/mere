// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a captured record replays as. [`CapturedDelta::replay`](super::CapturedDelta::replay)
//! decides once; replay loops and writers only act on the answer.

use super::{
    GraphDelta, GraphDeltaResult, OnRefusal, ReplayRefused, apply_graph_delta, replay_quietly,
};
use crate::graph::Graph;
use crate::graph::resource_content::ContentError;

/// A captured record, turned into what a replay does with it.
#[derive(Debug, Clone)]
pub enum Replay {
    /// A delta to apply.
    Apply(GraphDelta),
    /// Obsolete by design (pre-phase-2 inline imagery): never a refusal.
    Skip,
    /// Corrupt: checked replay stops here, tolerant replay drops it.
    Malformed(Malformed),
}

/// A captured facet record whose value is not valid JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformed {
    pub node_id: String,
    pub facet: String,
    pub error: String,
}

impl Malformed {
    pub(super) fn new(node_id: &str, facet: &str, error: serde_json::Error) -> Self {
        Self {
            node_id: node_id.into(),
            facet: facet.into(),
            error: error.to_string(),
        }
    }

    /// The refusal a checked replay reports for this record at `position`.
    pub(crate) fn refused_at(self, position: usize) -> ReplayRefused {
        ReplayRefused {
            position,
            reason: ContentError::InvalidFacet(self.to_string()),
        }
    }
}

impl std::fmt::Display for Malformed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "facet {} of node {} is not valid JSON: {}",
            self.facet, self.node_id, self.error
        )
    }
}

/// Why a live write cannot carry out a captured record. Replay may skip or
/// drop one; a write refuses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unwritable {
    Skipped,
    Malformed(Malformed),
}

impl std::fmt::Display for Unwritable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skipped => f.write_str("obsolete record with no live form"),
            Self::Malformed(malformed) => malformed.fmt(f),
        }
    }
}

impl std::error::Error for Unwritable {}

impl Replay {
    /// The delta a live write carries out; a skipped or malformed record is refused.
    pub fn into_write(self) -> Result<GraphDelta, Unwritable> {
        match self {
            Self::Apply(delta) => Ok(delta),
            Self::Skip => Err(Unwritable::Skipped),
            Self::Malformed(malformed) => Err(Unwritable::Malformed(malformed)),
        }
    }
}

/// Replay a stream of outcomes quietly. `position` counts every outcome,
/// skipped ones included. Under [`OnRefusal::Stop`] a malformed record or a
/// refused delta ends the replay; under `Drop` either is passed over. A skip
/// is passed over under both.
pub(crate) fn replay_outcomes_onto<I>(
    graph: &mut Graph,
    outcomes: I,
    on_refusal: OnRefusal,
) -> Result<(), ReplayRefused>
where
    I: IntoIterator<Item = Replay>,
{
    replay_quietly(graph, |graph| {
        for (position, outcome) in outcomes.into_iter().enumerate() {
            let delta = match outcome {
                Replay::Apply(delta) => delta,
                Replay::Skip => continue,
                Replay::Malformed(bad) if on_refusal == OnRefusal::Stop => {
                    return Err(bad.refused_at(position));
                },
                Replay::Malformed(_) => continue,
            };
            if let GraphDeltaResult::Refused(reason) = apply_graph_delta(graph, delta)
                && on_refusal == OnRefusal::Stop
            {
                return Err(ReplayRefused { position, reason });
            }
        }
        Ok(())
    })
}
