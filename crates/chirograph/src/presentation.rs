// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Structural admission for a scene's product-free presentation sidecar.

use std::collections::BTreeSet;
use std::fmt;

use sceno::InstanceId;

use crate::{PresentationCapability, PresentationCodec, PresentationManifest, SceneSnapshot};

/// A presentation table does not describe bindings or offers the scene can use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationValidationError {
    AbsentOrTombstonedItem(InstanceId),
    DuplicateBinding(InstanceId),
    CodecCapabilityMismatch,
}

impl fmt::Display for PresentationValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsentOrTombstonedItem(instance) => write!(
                formatter,
                "binding for absent or tombstoned item {}",
                instance.0
            ),
            Self::DuplicateBinding(instance) => {
                write!(formatter, "duplicate binding for item {}", instance.0)
            },
            Self::CodecCapabilityMismatch => {
                formatter.write_str("presentation codec names the wrong required capability")
            },
        }
    }
}

impl std::error::Error for PresentationValidationError {}

/// Validate the structural relationship between a scene and its presentation
/// metadata. Session-local retention and resource availability stay with the
/// caller that owns those policies. Callers validate the scene tables separately.
pub fn validate_presentation(
    scene: &SceneSnapshot,
    presentation: &PresentationManifest,
) -> Result<(), PresentationValidationError> {
    let mut instances = BTreeSet::new();
    for binding in &presentation.bindings {
        if scene.active_item(binding.instance).is_none() {
            return Err(PresentationValidationError::AbsentOrTombstonedItem(
                binding.instance,
            ));
        }
        if !instances.insert(binding.instance.0) {
            return Err(PresentationValidationError::DuplicateBinding(
                binding.instance,
            ));
        }
    }
    for offer in presentation.offers.values().flatten() {
        let expected = match offer.codec {
            PresentationCodec::NativeGlyphV1 => PresentationCapability::NativeGlyph,
            PresentationCodec::PortableCardV1 => PresentationCapability::PortableCard,
            PresentationCodec::ImageV1 { .. } => PresentationCapability::Image,
            PresentationCodec::EditableTextV1 => PresentationCapability::EditableText,
        };
        if offer.requires != expected {
            return Err(PresentationValidationError::CodecCapabilityMismatch);
        }
    }
    Ok(())
}
