// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable projection captures: immutable, content-addressed scene snapshots
//! with their presentation sidecar, for persistence or a resource carrier.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{ContentHash, PresentationManifest, ProjectionSnapshot, SceneSnapshot};

/// First portable projection-capture wire shape. A capture is delivery truth,
/// not a source-time cursor: it reproduces this exact Scenotime table snapshot
/// and its separately addressed presentation resources.
pub const PROJECTION_CAPTURE_VERSION: u16 = 1;

/// A portable, immutable scene capture. The session identity and cache policy
/// are intentionally absent: neither contributes to the realized scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionCaptureV1 {
    pub version: u16,
    pub scene: SceneSnapshot,
    #[serde(default)]
    pub presentation: PresentationManifest,
}

/// A rejected projection capture or load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionCaptureError {
    UnsupportedVersion { found: u16 },
    InvalidScene(String),
    MissingPresentationResource(ContentHash),
    Decode(String),
    Encode(String),
}

impl fmt::Display for ProjectionCaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion { found } => {
                write!(formatter, "unsupported projection-capture version {found}")
            },
            Self::InvalidScene(error) => write!(formatter, "invalid projection capture: {error}"),
            Self::MissingPresentationResource(resource) => write!(
                formatter,
                "projection capture resource {resource} is unavailable"
            ),
            Self::Decode(error) => {
                write!(formatter, "could not decode projection capture: {error}")
            },
            Self::Encode(error) => {
                write!(formatter, "could not encode projection capture: {error}")
            },
        }
    }
}

impl std::error::Error for ProjectionCaptureError {}

impl ProjectionCaptureV1 {
    /// Capture one already-authorized projection snapshot. This preserves the
    /// Scenotime epoch and revision, but does not claim they are source history.
    pub fn from_projection(snapshot: &ProjectionSnapshot) -> Result<Self, ProjectionCaptureError> {
        let capture = Self {
            version: PROJECTION_CAPTURE_VERSION,
            scene: snapshot.scene.clone(),
            presentation: snapshot.presentation.clone(),
        };
        capture.validate()?;
        Ok(capture)
    }

    /// Every separately-addressed resource the captured presentation requires.
    pub fn presentation_resources(&self) -> BTreeSet<ContentHash> {
        self.presentation
            .offers
            .values()
            .flatten()
            .map(|offer| offer.resource)
            .collect()
    }

    /// Reject malformed tables before disclosure or persistence.
    pub fn validate(&self) -> Result<(), ProjectionCaptureError> {
        if self.version != PROJECTION_CAPTURE_VERSION {
            return Err(ProjectionCaptureError::UnsupportedVersion {
                found: self.version,
            });
        }
        self.scene
            .validate()
            .map_err(|error| ProjectionCaptureError::InvalidScene(format!("{error:?}")))
    }

    /// Ensure each named presentation resource can be read before a recipient
    /// is told the projection capture is available.
    pub fn verify_resources(
        &self,
        mut available: impl FnMut(ContentHash) -> bool,
    ) -> Result<(), ProjectionCaptureError> {
        self.validate()?;
        for resource in self.presentation_resources() {
            if !available(resource) {
                return Err(ProjectionCaptureError::MissingPresentationResource(
                    resource,
                ));
            }
        }
        Ok(())
    }

    /// Stable serialized bytes used for persistence or a carrier resource.
    pub fn encode(&self) -> Result<Vec<u8>, ProjectionCaptureError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|error| ProjectionCaptureError::Encode(error.to_string()))
    }

    /// Decode and validate a projection capture before it reaches a renderer.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProjectionCaptureError> {
        let capture = serde_json::from_slice::<Self>(bytes)
            .map_err(|error| ProjectionCaptureError::Decode(error.to_string()))?;
        capture.validate()?;
        Ok(capture)
    }

    /// The projection capture's content address. A resource carrier may use
    /// this as its resource key without knowing any source graph or live session.
    pub fn content_address(&self) -> Result<ContentHash, ProjectionCaptureError> {
        Ok(ContentHash::of(&self.encode()?))
    }
}
