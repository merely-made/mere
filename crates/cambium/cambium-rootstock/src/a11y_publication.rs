// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Publisher observations. These certify neither reader delivery nor actor work.
use crate::A11yRequest;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum A11yAdapterState {
    NotConfigured,
    Detached,
    Unavailable,
    Installed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum A11yPublicationSkip {
    NotSynced,
    NoAdapter,
    NoWindow,
    NoRunner,
    NoLayout,
    OwnerEnded,
}

/// Fixed codes only; contributor content and platform error strings stay out
/// of exported observations. The publisher may log its local error separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum A11yPublicationRefusal {
    InvalidBindings,
    InvalidTree,
    AdapterInstall,
    DetachedMirror,
    DomWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum A11yPublicationOutcome {
    NotAttempted {
        reason: A11yPublicationSkip,
    },
    /// Handed to the native bridge. Its update API has no acceptance receipt;
    /// this does not claim the adapter or assistive technology consumed it.
    Submitted {
        nodes: usize,
        contribution_issues: usize,
    },
    /// Browser DOM writes completed. Reader announcement/focus is not certified.
    Mirrored {
        nodes: usize,
        contribution_issues: usize,
    },
    Refused {
        reason: A11yPublicationRefusal,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct A11yPublication {
    pub adapter: A11yAdapterState,
    pub outcome: A11yPublicationOutcome,
}

impl Default for A11yPublication {
    fn default() -> Self {
        Self::skipped(
            A11yAdapterState::NotConfigured,
            A11yPublicationSkip::NotSynced,
        )
    }
}

impl A11yPublication {
    pub fn skipped(adapter: A11yAdapterState, reason: A11yPublicationSkip) -> Self {
        Self {
            adapter,
            outcome: A11yPublicationOutcome::NotAttempted { reason },
        }
    }

    pub fn refused(adapter: A11yAdapterState, reason: A11yPublicationRefusal) -> Self {
        Self {
            adapter,
            outcome: A11yPublicationOutcome::Refused { reason },
        }
    }
}

/// One sync's independent observations and pending actions. Publication refusal
/// does not turn previously drained requests into a claim of dispatch/completion.
pub struct A11ySync {
    pub requests: Vec<A11yRequest>,
    pub publication: A11yPublication,
}

/// Owner-stamped latest observation, not a presentation revision or target lease.
/// Sequence zero means unsampled. Polling this copied value advances nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct A11yPublicationSnapshot {
    pub host: u64,
    pub sequence: u64,
    pub publication: A11yPublication,
}
