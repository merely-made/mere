// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tiny allocation-free trace vocabulary for Meristem's no_std core.
//!
//! Producer zero is reserved for Meristem. The schema is represented by a
//! fixed numeric kind and bounded numeric path facts; application messages and
//! view identities are deliberately never formatted here.

#[cfg(target_has_atomic = "64")]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_has_atomic = "64")]
static NEXT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
#[repr(u8)]
pub(crate) enum CoreTraceKind {
    AnyViewReused = 1,
    AnyViewReplaced = 2,
    AnyViewTornDown = 3,
    GenerationMatched = 4,
    GenerationRefused = 5,
    MessagePathAdvanced = 6,
    MessageConsumed = 7,
    MessageDeclined = 8,
    SequenceChildAdded = 9,
    SequenceChildRemoved = 10,
    SequenceMessageMatched = 11,
    SequenceMessageRefused = 12,
    CoverageUnavailable = 255,
}

impl CoreTraceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::AnyViewReused => "any_view_reused",
            Self::AnyViewReplaced => "any_view_replaced",
            Self::AnyViewTornDown => "any_view_torn_down",
            Self::GenerationMatched => "generation_matched",
            Self::GenerationRefused => "generation_refused",
            Self::MessagePathAdvanced => "message_path_advanced",
            Self::MessageConsumed => "message_consumed",
            Self::MessageDeclined => "message_declined",
            Self::SequenceChildAdded => "sequence_child_added",
            Self::SequenceChildRemoved => "sequence_child_removed",
            Self::SequenceMessageMatched => "sequence_message_matched",
            Self::SequenceMessageRefused => "sequence_message_refused",
            Self::CoverageUnavailable => "coverage_unavailable",
        }
    }
}

#[cfg(target_has_atomic = "64")]
pub(crate) fn emit(kind: CoreTraceKind, generation: Option<u64>, path_depth: Option<usize>) {
    let sequence = NEXT_SEQUENCE.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
        next.checked_add(1)
    });
    let Ok(sequence) = sequence else {
        tracing::warn!(
            target: "meristem::reactive",
            schema_version = 1_u16,
            producer_id = 0_u64,
            kind = CoreTraceKind::CoverageUnavailable.as_str(),
            reason = "sequence_exhausted",
            last_sequence = u64::MAX - 1,
            "reactive diagnostic coverage gap"
        );
        return;
    };
    tracing::trace!(
        target: "meristem::reactive",
        schema_version = 1_u16,
        producer_id = 0_u64,
        sequence,
        kind = kind.as_str(),
        generation = generation.unwrap_or(u64::MAX),
        generation_present = generation.is_some(),
        path_depth = path_depth.map(|depth| depth.min(u16::MAX as usize) as u16).unwrap_or(u16::MAX),
        path_depth_present = path_depth.is_some(),
        path_truncated = path_depth.is_some_and(|depth| depth > u16::MAX as usize),
        "meristem reactive decision"
    );
}

#[cfg(not(target_has_atomic = "64"))]
pub(crate) fn emit(_kind: CoreTraceKind, _generation: Option<u64>, _path_depth: Option<usize>) {
    // This target cannot provide a process-unique checked u64 sequence. Keep
    // the core portable and make the missing trace coverage explicit.
    tracing::warn!(
        target: "meristem::reactive",
        schema_version = 1_u16,
        producer_id = 0_u64,
        kind = CoreTraceKind::CoverageUnavailable.as_str(),
        reason = "target_lacks_atomic_u64",
        "reactive diagnostic coverage gap"
    );
}
