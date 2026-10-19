// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The captured-delta wire shape before `EdgeAssertion::decay_progress` was
//! retired. Runtime assertions no longer expose or use that value, but journal
//! entries written by older Mere versions must still replay. This tolerance is
//! scoped to captured replay data: its historical reader ignored unknown keys,
//! so the adapter deliberately keeps that behavior. Current live-command input
//! deserializes `EdgeAssertion` directly and retains its strict field check.

use rkyv::{
    Archive, Deserialize, Place, Serialize,
    rancor::Fallible,
    with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

use crate::graph::{
    ArrangementSubKind, ContainmentSubKind, EdgeAssertion, ImportedSubKind, ProvenanceSubKind,
    SemanticSubKind,
};

/// Frozen legacy shape. Keep variant order and the semantic field order in sync
/// with the old `EdgeAssertion`; the decay slot is always `None` on new writes.
#[derive(Archive, Serialize, Deserialize, SerdeSerialize, SerdeDeserialize)]
pub(crate) enum LegacyEdgeAssertion {
    Semantic {
        sub_kind: SemanticSubKind,
        label: Option<String>,
        #[serde(default)]
        decay_progress: Option<f32>,
    },
    Containment {
        sub_kind: ContainmentSubKind,
    },
    Arrangement {
        sub_kind: ArrangementSubKind,
    },
    Imported {
        sub_kind: ImportedSubKind,
    },
    Provenance {
        sub_kind: ProvenanceSubKind,
    },
}

impl From<&EdgeAssertion> for LegacyEdgeAssertion {
    fn from(value: &EdgeAssertion) -> Self {
        match value {
            EdgeAssertion::Semantic { sub_kind, label } => Self::Semantic {
                sub_kind: *sub_kind,
                label: label.clone(),
                decay_progress: None,
            },
            EdgeAssertion::Containment { sub_kind } => Self::Containment {
                sub_kind: *sub_kind,
            },
            EdgeAssertion::Arrangement { sub_kind } => Self::Arrangement {
                sub_kind: *sub_kind,
            },
            EdgeAssertion::Imported { sub_kind } => Self::Imported {
                sub_kind: *sub_kind,
            },
            EdgeAssertion::Provenance { sub_kind } => Self::Provenance {
                sub_kind: *sub_kind,
            },
        }
    }
}

impl From<LegacyEdgeAssertion> for EdgeAssertion {
    fn from(value: LegacyEdgeAssertion) -> Self {
        match value {
            LegacyEdgeAssertion::Semantic {
                sub_kind, label, ..
            } => Self::Semantic { sub_kind, label },
            LegacyEdgeAssertion::Containment { sub_kind } => Self::Containment { sub_kind },
            LegacyEdgeAssertion::Arrangement { sub_kind } => Self::Arrangement { sub_kind },
            LegacyEdgeAssertion::Imported { sub_kind } => Self::Imported { sub_kind },
            LegacyEdgeAssertion::Provenance { sub_kind } => Self::Provenance { sub_kind },
        }
    }
}

/// Serde adapter scoped to captured journal assertions. Human-readable output
/// uses the current field-free shape; legacy input accepts and drops the old
/// value. Positional codecs retain the historical `Option<f32>` slot.
pub(crate) fn serialize<S>(value: &EdgeAssertion, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if serializer.is_human_readable() {
        SerdeSerialize::serialize(value, serializer)
    } else {
        SerdeSerialize::serialize(&LegacyEdgeAssertion::from(value), serializer)
    }
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<EdgeAssertion, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let legacy: LegacyEdgeAssertion = SerdeDeserialize::deserialize(deserializer)?;
    Ok(legacy.into())
}

/// rkyv adapter preserving the previous archived representation of the
/// assertion field inside `CapturedDelta` only.
pub(crate) struct Rkyv;

impl ArchiveWith<EdgeAssertion> for Rkyv {
    type Archived = <LegacyEdgeAssertion as Archive>::Archived;
    type Resolver = <LegacyEdgeAssertion as Archive>::Resolver;

    fn resolve_with(field: &EdgeAssertion, resolver: Self::Resolver, out: Place<Self::Archived>) {
        LegacyEdgeAssertion::from(field).resolve(resolver, out);
    }
}

impl<S> SerializeWith<EdgeAssertion, S> for Rkyv
where
    S: Fallible + ?Sized,
    LegacyEdgeAssertion: Serialize<S>,
{
    fn serialize_with(
        field: &EdgeAssertion,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        rkyv::Serialize::serialize(&LegacyEdgeAssertion::from(field), serializer)
    }
}

impl<D> DeserializeWith<<LegacyEdgeAssertion as Archive>::Archived, EdgeAssertion, D> for Rkyv
where
    D: Fallible + ?Sized,
    <LegacyEdgeAssertion as Archive>::Archived: Deserialize<LegacyEdgeAssertion, D>,
{
    fn deserialize_with(
        field: &<LegacyEdgeAssertion as Archive>::Archived,
        deserializer: &mut D,
    ) -> Result<EdgeAssertion, D::Error> {
        field.deserialize(deserializer).map(Into::into)
    }
}
