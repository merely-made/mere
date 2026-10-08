// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable projection captures: immutable, content-addressed scene snapshots
//! with their presentation sidecar, for persistence or a resource carrier.
//!
//! Version 1 is the scene and its presentation. Version 2 adds an optional
//! pre-solved [`Score`] and the authority identity the scene was projected
//! from, so a static viewer that does not run the compiler can still render
//! and attribute a portable scene. A capture is one instant: a scene-edit
//! history travels beside it as its own artifact, never inside it.

use std::collections::BTreeSet;
use std::fmt;

use sceno::{SCORE_VERSION, Score, ScoreVersionError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{
    ContentHash, PresentationManifest, ProjectionSnapshot, SceneSnapshot, Sha256NamedInformation,
};

/// First portable projection-capture wire shape. A capture is delivery truth,
/// not a source-time cursor: it reproduces this exact Scenotime table snapshot
/// and its separately addressed presentation resources.
pub const PROJECTION_CAPTURE_VERSION: u16 = 1;

/// Second portable projection-capture wire shape: version 1 plus an optional
/// pre-solved score and an optional authority identity.
pub const PROJECTION_CAPTURE_V2: u16 = 2;

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

/// The authority a captured scene was projected from.
///
/// `adapter` names the resolver, `schema` the authority record's own schema,
/// `sha256` the authority bytes, and `generation` the adapter-stamped input
/// generation that a score solved from this authority carries. Chirograph
/// checks only that the generation agrees with the capture's score; how the
/// generation relates to the scene's epoch is the host's convention.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureAuthorityV1 {
    pub adapter: String,
    pub schema: String,
    pub sha256: Sha256NamedInformation,
    pub generation: u64,
}

/// A portable scene capture with the inputs that produced it.
///
/// `score` is the pre-solved request the scene realizes, for a viewer that
/// wants it without the compiler; `authority` names where that request came
/// from. Either may be absent. A capture lifted from version 1 has neither.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionCaptureV2 {
    pub version: u16,
    pub scene: SceneSnapshot,
    #[serde(default)]
    pub presentation: PresentationManifest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<Score>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<CaptureAuthorityV1>,
}

/// A rejected projection capture or load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionCaptureError {
    UnsupportedVersion {
        found: u16,
    },
    InvalidScene(String),
    /// The captured score is newer than this reader's sceno.
    NewerScore(ScoreVersionError),
    /// The captured score and authority disagree about the input generation.
    GenerationMismatch {
        score: u64,
        authority: u64,
    },
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
            Self::NewerScore(error) => write!(formatter, "projection capture score: {error}"),
            Self::GenerationMismatch { score, authority } => write!(
                formatter,
                "projection capture score generation {score} differs from its authority's \
                 {authority}"
            ),
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

/// What every capture version shares: a version number, a presentation
/// sidecar, and its own validation. The byte, address, and resource logic
/// below is written once against this.
trait CaptureWire: Serialize + DeserializeOwned {
    const VERSION: u16;

    fn presentation(&self) -> &PresentationManifest;

    fn validate(&self) -> Result<(), ProjectionCaptureError>;
}

impl CaptureWire for ProjectionCaptureV1 {
    const VERSION: u16 = PROJECTION_CAPTURE_VERSION;

    fn presentation(&self) -> &PresentationManifest {
        &self.presentation
    }

    fn validate(&self) -> Result<(), ProjectionCaptureError> {
        require_version(self.version, Self::VERSION)?;
        validate_scene(&self.scene)
    }
}

impl CaptureWire for ProjectionCaptureV2 {
    const VERSION: u16 = PROJECTION_CAPTURE_V2;

    fn presentation(&self) -> &PresentationManifest {
        &self.presentation
    }

    fn validate(&self) -> Result<(), ProjectionCaptureError> {
        require_version(self.version, Self::VERSION)?;
        validate_scene(&self.scene)?;
        if let Some(score) = &self.score
            && score.version > SCORE_VERSION
        {
            return Err(ProjectionCaptureError::NewerScore(ScoreVersionError {
                found: score.version,
                reader: SCORE_VERSION,
            }));
        }
        if let (Some(score), Some(authority)) = (&self.score, &self.authority)
            && score.generation != authority.generation
        {
            return Err(ProjectionCaptureError::GenerationMismatch {
                score: score.generation,
                authority: authority.generation,
            });
        }
        Ok(())
    }
}

fn require_version(found: u16, expected: u16) -> Result<(), ProjectionCaptureError> {
    (found == expected)
        .then_some(())
        .ok_or(ProjectionCaptureError::UnsupportedVersion { found })
}

fn validate_scene(scene: &SceneSnapshot) -> Result<(), ProjectionCaptureError> {
    scene
        .validate()
        .map_err(|error| ProjectionCaptureError::InvalidScene(format!("{error:?}")))
}

fn presentation_resources(presentation: &PresentationManifest) -> BTreeSet<ContentHash> {
    presentation
        .offers
        .values()
        .flatten()
        .map(|offer| offer.resource)
        .collect()
}

fn verify_resources<C: CaptureWire>(
    capture: &C,
    mut available: impl FnMut(ContentHash) -> bool,
) -> Result<(), ProjectionCaptureError> {
    capture.validate()?;
    for resource in presentation_resources(capture.presentation()) {
        if !available(resource) {
            return Err(ProjectionCaptureError::MissingPresentationResource(
                resource,
            ));
        }
    }
    Ok(())
}

fn encode<C: CaptureWire>(capture: &C) -> Result<Vec<u8>, ProjectionCaptureError> {
    capture.validate()?;
    serde_json::to_vec(capture).map_err(|error| ProjectionCaptureError::Encode(error.to_string()))
}

/// The version a capture's bytes declare, read before the strict shape so a
/// capture of another version is refused as that, not as an unknown field.
fn declared_version(bytes: &[u8]) -> Result<u16, ProjectionCaptureError> {
    #[derive(Deserialize)]
    struct Declared {
        version: u16,
    }
    serde_json::from_slice::<Declared>(bytes)
        .map(|declared| declared.version)
        .map_err(|error| ProjectionCaptureError::Decode(error.to_string()))
}

fn decode_exact<C: CaptureWire>(bytes: &[u8]) -> Result<C, ProjectionCaptureError> {
    require_version(declared_version(bytes)?, C::VERSION)?;
    let capture = serde_json::from_slice::<C>(bytes)
        .map_err(|error| ProjectionCaptureError::Decode(error.to_string()))?;
    capture.validate()?;
    Ok(capture)
}

fn content_address<C: CaptureWire>(capture: &C) -> Result<ContentHash, ProjectionCaptureError> {
    Ok(ContentHash::of(&encode(capture)?))
}

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
        presentation_resources(&self.presentation)
    }

    /// Reject malformed tables before disclosure or persistence.
    pub fn validate(&self) -> Result<(), ProjectionCaptureError> {
        CaptureWire::validate(self)
    }

    /// Ensure each named presentation resource can be read before a recipient
    /// is told the projection capture is available.
    pub fn verify_resources(
        &self,
        available: impl FnMut(ContentHash) -> bool,
    ) -> Result<(), ProjectionCaptureError> {
        verify_resources(self, available)
    }

    /// Stable serialized bytes used for persistence or a carrier resource.
    pub fn encode(&self) -> Result<Vec<u8>, ProjectionCaptureError> {
        encode(self)
    }

    /// Decode and validate a projection capture before it reaches a renderer.
    /// Only version 1 is read here; [`ProjectionCaptureV2::decode`] reads both.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProjectionCaptureError> {
        decode_exact(bytes)
    }

    /// The projection capture's content address. A resource carrier may use
    /// this as its resource key without knowing any source graph or live session.
    pub fn content_address(&self) -> Result<ContentHash, ProjectionCaptureError> {
        content_address(self)
    }
}

/// Lifting a version 1 capture changes only its version: it gains no score
/// and no authority. Its content address changes with its bytes.
impl From<ProjectionCaptureV1> for ProjectionCaptureV2 {
    fn from(capture: ProjectionCaptureV1) -> Self {
        Self {
            version: PROJECTION_CAPTURE_V2,
            scene: capture.scene,
            presentation: capture.presentation,
            score: None,
            authority: None,
        }
    }
}

impl ProjectionCaptureV2 {
    /// Every separately-addressed resource the captured presentation requires.
    pub fn presentation_resources(&self) -> BTreeSet<ContentHash> {
        presentation_resources(&self.presentation)
    }

    /// Reject malformed tables, a score newer than this reader, and a score
    /// whose generation disagrees with its authority's.
    pub fn validate(&self) -> Result<(), ProjectionCaptureError> {
        CaptureWire::validate(self)
    }

    /// Ensure each named presentation resource can be read before a recipient
    /// is told the projection capture is available.
    pub fn verify_resources(
        &self,
        available: impl FnMut(ContentHash) -> bool,
    ) -> Result<(), ProjectionCaptureError> {
        verify_resources(self, available)
    }

    /// Stable serialized bytes used for persistence or a carrier resource.
    pub fn encode(&self) -> Result<Vec<u8>, ProjectionCaptureError> {
        encode(self)
    }

    /// Decode and validate a capture of either version. A version 1 capture is
    /// lifted; any other version is refused.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProjectionCaptureError> {
        match declared_version(bytes)? {
            PROJECTION_CAPTURE_VERSION => {
                decode_exact::<ProjectionCaptureV1>(bytes).map(Self::from)
            },
            _ => decode_exact(bytes),
        }
    }

    /// The projection capture's content address, over its version 2 bytes.
    pub fn content_address(&self) -> Result<ContentHash, ProjectionCaptureError> {
        content_address(self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sceno::{Arrangement, Scene, Spiral};

    use super::*;
    use crate::{
        BoundsRelationship, PresentationCapability, PresentationCodec, PresentationKey,
        PresentationOffer, PresentationSemantics, Revision, SceneEpoch, SemanticRole,
    };

    fn presentation() -> PresentationManifest {
        PresentationManifest {
            bindings: Vec::new(),
            offers: BTreeMap::from([(
                PresentationKey("fixture:scene".to_string()),
                vec![PresentationOffer {
                    codec: PresentationCodec::PortableCardV1,
                    resource: ContentHash::of(b"captured presentation"),
                    byte_size: 21,
                    requires: PresentationCapability::PortableCard,
                    semantics: PresentationSemantics {
                        label: "Captured fixture".to_string(),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FitWithinFootprint,
                        actions: Vec::new(),
                    },
                }],
            )]),
        }
    }

    fn v1() -> ProjectionCaptureV1 {
        ProjectionCaptureV1 {
            version: PROJECTION_CAPTURE_VERSION,
            scene: SceneSnapshot::from_dense(SceneEpoch(9), Revision(4), Scene::new())
                .expect("fixture scene is valid"),
            presentation: presentation(),
        }
    }

    fn score(generation: u64) -> Score {
        let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
        score.generation = generation;
        score
    }

    fn authority(generation: u64) -> CaptureAuthorityV1 {
        CaptureAuthorityV1 {
            adapter: "fixture.adapter".to_string(),
            schema: "fixture.authority/v1".to_string(),
            sha256: Sha256NamedInformation::of(b"authority bytes"),
            generation,
        }
    }

    fn v2() -> ProjectionCaptureV2 {
        ProjectionCaptureV2 {
            score: Some(score(7)),
            authority: Some(authority(7)),
            ..ProjectionCaptureV2::from(v1())
        }
    }

    #[test]
    fn v2_round_trips_with_score_and_authority() {
        let capture = v2();
        let bytes = capture.encode().expect("encodes");
        let restored = ProjectionCaptureV2::decode(&bytes).expect("decodes");
        assert_eq!(restored, capture);
        assert_eq!(
            restored.content_address().expect("address"),
            ContentHash::of(&bytes)
        );
        assert_eq!(
            restored.presentation_resources(),
            v1().presentation_resources()
        );
        assert!(restored.verify_resources(|_| true).is_ok());
        assert!(matches!(
            restored.verify_resources(|_| false),
            Err(ProjectionCaptureError::MissingPresentationResource(_))
        ));
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(
            json["authority"]["sha256"],
            Sha256NamedInformation::of(b"authority bytes").to_string(),
            "the authority digest travels as an RFC 6920 name"
        );
    }

    #[test]
    fn v1_bytes_still_read_as_v1_and_v1_refuses_v2() {
        let bytes = v1().encode().expect("encodes");
        assert_eq!(ProjectionCaptureV1::decode(&bytes).expect("decodes"), v1());
        let v2_bytes = v2().encode().expect("encodes");
        assert_eq!(
            ProjectionCaptureV1::decode(&v2_bytes),
            Err(ProjectionCaptureError::UnsupportedVersion { found: 2 })
        );
    }

    #[test]
    fn v1_lifts_to_v2_without_score_or_authority() {
        let lifted = ProjectionCaptureV2::from(v1());
        assert_eq!(lifted.version, PROJECTION_CAPTURE_V2);
        assert_eq!(lifted.scene, v1().scene);
        assert_eq!(lifted.presentation, v1().presentation);
        assert!(lifted.score.is_none() && lifted.authority.is_none());
        let decoded = ProjectionCaptureV2::decode(&v1().encode().expect("encodes")).expect("lifts");
        assert_eq!(decoded, lifted);
        let bytes = lifted.encode().expect("encodes");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert!(json.get("score").is_none() && json.get("authority").is_none());
        assert_ne!(
            lifted.content_address().expect("address"),
            v1().content_address().expect("address"),
            "a lifted capture is a new artifact"
        );
    }

    #[test]
    fn generation_mismatch_is_refused() {
        let mismatched = ProjectionCaptureV2 {
            authority: Some(authority(8)),
            ..v2()
        };
        let expected = Err(ProjectionCaptureError::GenerationMismatch {
            score: 7,
            authority: 8,
        });
        assert_eq!(mismatched.validate(), expected);
        assert!(mismatched.encode().is_err());
        let bytes = serde_json::to_vec(&mismatched).expect("raw bytes");
        assert_eq!(ProjectionCaptureV2::decode(&bytes).map(|_| ()), expected);

        // Either half alone has nothing to disagree with.
        for half in [
            ProjectionCaptureV2 {
                score: None,
                ..mismatched.clone()
            },
            ProjectionCaptureV2 {
                authority: None,
                ..mismatched
            },
        ] {
            assert_eq!(half.validate(), Ok(()));
        }
    }

    #[test]
    fn a_score_newer_than_the_reader_is_refused() {
        let mut newer = v2();
        newer.score.as_mut().expect("score").version = SCORE_VERSION + 1;
        assert!(matches!(
            newer.validate(),
            Err(ProjectionCaptureError::NewerScore(ScoreVersionError { found, reader }))
                if found == SCORE_VERSION + 1 && reader == SCORE_VERSION
        ));
        // On the wire, sceno refuses it while reading the score.
        let bytes = serde_json::to_vec(&newer).expect("raw bytes");
        assert!(matches!(
            ProjectionCaptureV2::decode(&bytes),
            Err(ProjectionCaptureError::Decode(_))
        ));
    }

    #[test]
    fn unknown_keys_and_versions_are_refused() {
        let mut json = serde_json::to_value(v2()).expect("json");
        json["trace"] = serde_json::json!([]);
        let bytes = serde_json::to_vec(&json).expect("bytes");
        assert!(matches!(
            ProjectionCaptureV2::decode(&bytes),
            Err(ProjectionCaptureError::Decode(error)) if error.contains("trace")
        ));

        let mut json = serde_json::to_value(v2()).expect("json");
        json["authority"]["record"] = serde_json::json!("x");
        let bytes = serde_json::to_vec(&json).expect("bytes");
        assert!(matches!(
            ProjectionCaptureV2::decode(&bytes),
            Err(ProjectionCaptureError::Decode(error)) if error.contains("record")
        ));

        let mut json = serde_json::to_value(v2()).expect("json");
        json["version"] = serde_json::json!(3);
        let bytes = serde_json::to_vec(&json).expect("bytes");
        assert_eq!(
            ProjectionCaptureV2::decode(&bytes),
            Err(ProjectionCaptureError::UnsupportedVersion { found: 3 })
        );
    }
}
