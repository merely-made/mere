// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Chirograph** — the projection contract, executed in duplicate.
//!
//! Carrier-neutral Graphshell wire vocabulary.
//!
//! A message carries Scenograph's product-free score and scene types. Transport,
//! authorization, application models, and rendered content stay outside `sceno`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use sceno::{InstanceId, Score};
use serde::{Deserialize, Serialize};

// The card vocabulary: neutral, a label for a projected resource rather than a
// wire message. It was its own crate from 2026-08-14 to 2026-09-23 and is a
// module again; nothing but chirograph used it.
mod command_effect;
mod command_invocation;
pub mod line;
mod presentation;
pub mod titulus;
pub use titulus::{
    ActionFormChoiceV1, ActionFormError, ActionFormFieldV1, ActionFormV1, CardValueV1, ContentHash,
    ContentReferenceError, PortableCardV1, PortableContentRefV1, Sha256NamedInformation,
};

pub use presentation::{PresentationValidationError, validate_presentation};
pub use scenotime::{Revision, SceneDiff, SceneEpoch, SceneSnapshot};

// Portable projection captures, apart from the live wire they snapshot.
mod capture;
pub use capture::{
    CaptureAuthorityV1, PROJECTION_CAPTURE_V2, PROJECTION_CAPTURE_VERSION, ProjectionCaptureError,
    ProjectionCaptureV1, ProjectionCaptureV2,
};

/// The first compatible Graphshell wire version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    /// Protocol 1.1 adds the payload-free carrier revision notice.
    pub const V1_1: Self = Self { major: 1, minor: 1 };
    /// Protocol 1.2 adds editable-text resources.
    pub const V1_2: Self = Self { major: 1, minor: 2 };
    /// Latest protocol 1.x. Protocol 1.3 adds attributable derived-cache
    /// metadata to editable-text resources.
    pub const V1: Self = Self { major: 1, minor: 3 };
}

/// An endpoint-scoped projection session. It is opaque to Graphshell clients.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectionSession(pub String);

/// A requested score plus the client's observed protocol version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionRequest {
    pub version: ProtocolVersion,
    pub session: ProjectionSession,
    pub score: Score,
}

/// One endpoint-advertised projection that a generic Graphshell host may open.
///
/// The request carries only the product-free score vocabulary needed to select
/// the projection. Source data remains endpoint-side until `snapshot`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionOffer {
    pub label: String,
    pub request: ProjectionRequest,
}

/// The projections available from one endpoint process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EndpointDescriptor {
    pub label: String,
    pub projections: Vec<ProjectionOffer>,
}

/// Client presentation features negotiated independently of the renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PresentationCapability {
    NativeGlyph,
    PortableCard,
    Image,
    EditableText,
}

/// One named capability set used during offer selection.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProfile {
    pub capabilities: BTreeSet<PresentationCapability>,
}

impl CapabilityProfile {
    pub fn new(capabilities: impl IntoIterator<Item = PresentationCapability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    pub fn supports(&self, capability: PresentationCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// A snapshot-local handle to one set of ordered presentation offers.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PresentationKey(pub String);

/// A stable session-scoped action reference advertised by an endpoint.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IntentReference(pub String);

/// Whether invoking an advertised action changes local curation, domain truth,
/// or asks the endpoint to perform an external effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentEffect {
    Curation,
    DomainTruth,
    ExternalEffect,
}

/// An action carried into accessibility and permission surfaces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvertisedAction {
    pub intent: IntentReference,
    pub label: String,
    pub explanation: String,
    pub payload_schema: String,
    /// A small endpoint-authored input contract for this action, when the
    /// payload can be composed from exact string values. Its absence preserves
    /// the existing opaque-payload path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_form: Option<ActionFormV1>,
    pub effect: IntentEffect,
}

impl AdvertisedAction {
    /// Build this action's JSON payload from its advertised input form.
    ///
    /// Actions without a form keep their established payload path: an empty
    /// value map produces no payload, while nonempty values are refused rather
    /// than silently inventing an encoding.
    pub fn compose_payload(
        &self,
        values: &BTreeMap<String, String>,
    ) -> Result<Vec<u8>, ActionFormError> {
        match &self.input_form {
            Some(form) => form.compose_payload(&self.payload_schema, values),
            None if values.is_empty() => Ok(Vec::new()),
            None => Err(ActionFormError::NoInputForm),
        }
    }
}

/// The semantic role available before any resource bytes arrive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRole {
    Graphic,
    Article,
    Image,
}

/// How the realized content relates to the footprint placed by Scenograph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundsRelationship {
    FillFootprint,
    FitWithinFootprint,
    IntrinsicWithinFootprint,
}

/// Semantics that remain usable when the richest resource cannot be decoded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationSemantics {
    pub label: String,
    pub role: SemanticRole,
    pub bounds: BoundsRelationship,
    pub actions: Vec<AdvertisedAction>,
}

/// Versioned payload encodings understood by the first Graphshell host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresentationCodec {
    NativeGlyphV1,
    PortableCardV1,
    ImageV1 { mime_type: String },
    EditableTextV1,
}

/// One independently fetchable representation, ordered richest-first within
/// a manifest entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationOffer {
    pub codec: PresentationCodec,
    pub resource: ContentHash,
    pub byte_size: u64,
    pub requires: PresentationCapability,
    pub semantics: PresentationSemantics,
}

/// Connects one scene instance to one presentation key without adding a
/// Graphshell-owned reference to `sceno::ProjectedItem`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationBinding {
    pub instance: InstanceId,
    pub key: PresentationKey,
}

/// Presentation metadata beside a scene. Resource bytes travel separately.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationManifest {
    pub bindings: Vec<PresentationBinding>,
    pub offers: BTreeMap<PresentationKey, Vec<PresentationOffer>>,
}

impl PresentationManifest {
    pub fn offers_for(&self, instance: InstanceId) -> Option<&[PresentationOffer]> {
        let key = self
            .bindings
            .iter()
            .find(|binding| binding.instance == instance)?
            .key
            .clone();
        self.offers.get(&key).map(Vec::as_slice)
    }
}

/// A complete epoch-preserving scene snapshot plus its presentation sidecar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionSnapshot {
    pub version: ProtocolVersion,
    pub session: ProjectionSession,
    pub scene: SceneSnapshot,
    #[serde(default)]
    pub presentation: PresentationManifest,
    #[serde(default)]
    pub cache_policy: CachePolicy,
}

/// Schema for an opaque reference to a producer-owned live-view record. The
/// carrier transports this reference without parsing source truth or gaining
/// authority over it; the receiving endpoint's participant gate resolves it.
pub const LIVE_VIEW_REFERENCE_SCHEMA: &str = "mere.live-view-reference/v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveViewReferenceV1 {
    pub schema: String,
    pub record: String,
}

impl LiveViewReferenceV1 {
    pub fn new(record: impl Into<String>) -> Self {
        Self {
            schema: LIVE_VIEW_REFERENCE_SCHEMA.to_string(),
            record: record.into(),
        }
    }

    pub fn validate(&self) -> Result<(), LiveViewReferenceError> {
        if self.schema != LIVE_VIEW_REFERENCE_SCHEMA {
            return Err(LiveViewReferenceError::UnsupportedSchema {
                found: self.schema.clone(),
            });
        }
        if self.record.trim().is_empty() {
            return Err(LiveViewReferenceError::MissingRecord);
        }
        if !is_opaque_live_view_reference(&self.record) {
            return Err(LiveViewReferenceError::InvalidOpaqueReference);
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, LiveViewReferenceError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|error| LiveViewReferenceError::Encode(error.to_string()))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, LiveViewReferenceError> {
        let reference = serde_json::from_slice::<Self>(bytes)
            .map_err(|error| LiveViewReferenceError::Decode(error.to_string()))?;
        reference.validate()?;
        Ok(reference)
    }
}

fn is_opaque_live_view_reference(reference: &str) -> bool {
    let Some((scheme, identifier)) = reference.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && !identifier.is_empty()
        && reference.len() <= 256
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveViewReferenceError {
    UnsupportedSchema { found: String },
    MissingRecord,
    InvalidOpaqueReference,
    Decode(String),
    Encode(String),
}

impl fmt::Display for LiveViewReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { found } => {
                write!(
                    formatter,
                    "unsupported live-view reference schema {found:?}"
                )
            },
            Self::MissingRecord => write!(formatter, "live-view record reference is missing"),
            Self::InvalidOpaqueReference => write!(
                formatter,
                "live-view record reference must be an opaque scheme:identifier value"
            ),
            Self::Decode(error) => {
                write!(formatter, "could not decode live-view reference: {error}")
            },
            Self::Encode(error) => {
                write!(formatter, "could not encode live-view reference: {error}")
            },
        }
    }
}

impl std::error::Error for LiveViewReferenceError {}

/// Presentation and cache changes that accompany a Scenotime scene diff.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresentationChange {
    Bind(PresentationBinding),
    Unbind {
        instance: InstanceId,
    },
    ReplaceOffers {
        key: PresentationKey,
        offers: Vec<PresentationOffer>,
    },
    RemoveOffers {
        key: PresentationKey,
    },
    InvalidateResource {
        resource: ContentHash,
    },
}

/// One revision transition in a projection session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionDiff {
    pub version: ProtocolVersion,
    pub session: ProjectionSession,
    pub scene: SceneDiff,
    pub presentation: Vec<PresentationChange>,
    pub status: Option<SessionStatus>,
}

/// The last revision a client has applied in memory: the session, epoch and
/// revision of its mounted scene. It says nothing about what the client has
/// stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionAck {
    pub session: ProjectionSession,
    pub epoch: SceneEpoch,
    pub revision: Revision,
}

/// Reconnect from a client's last acknowledged epoch and revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeRequest {
    pub session: ProjectionSession,
    pub epoch: SceneEpoch,
    pub revision: Revision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResumeReply {
    Current(ProjectionAck),
    Diffs(Vec<ProjectionDiff>),
    Snapshot(Box<ProjectionSnapshot>),
}

/// Most raw bytes one resource chunk carries before encoding.
///
/// Sized against the native-messaging frame cap rather than chosen round.
/// `bytes` is base64 in the chunk reply, which costs 4/3, so 512 KiB raw
/// arrives as about 683 KiB and leaves room for the rest of the frame under a
/// 1 MiB cap. Serializing raw `Vec<u8>` through JSON instead would cost
/// roughly 4x and put a single chunk over the cap on its own, which is why
/// this reply carries text where [`ResourceResponse`] carries bytes.
pub const MAX_RESOURCE_CHUNK_BYTES: u32 = 512 * 1024;

/// Reassembles a resource from sequential chunk replies.
///
/// Sequential on purpose: chunks are requested one window at a time, so an
/// out-of-order or duplicated frame is a protocol fault to report rather than
/// a case to reorder around. The whole-resource hash is verified once at the
/// end, so a corrupted frame that somehow passed its own check still cannot
/// be delivered as the resource.
#[derive(Clone, Debug, Default)]
pub struct ResourceAssembly {
    bytes: Vec<u8>,
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(text).ok()
}

pub const EDITABLE_TEXT_SAVE_INTENT: &str = "graphshell.editable-text.save";
pub const EDITABLE_TEXT_SAVE_SCHEMA: &str = "graphshell.editable-text.save/v1";

pub const KNOT_CLIP_INSERT_INTENT: &str = "knot.clip.insert";
pub const KNOT_CLIP_INSERT_SCHEMA: &str = "knot.clip.insert/v1";

pub const KNOT_CLIP_INSERT_SCHEMA_V2: &str = "knot.clip.insert/v2";

pub const KNOT_TRANSCLUSION_RESOLVE_INTENT: &str = "knot.transclusion.resolve";
pub const KNOT_TRANSCLUSION_RESOLVE_SCHEMA: &str = "knot.transclusion.resolve/v1";
pub const KNOT_BLOCK_RUN_INTENT: &str = "knot.block.run";
pub const KNOT_BLOCK_RUN_SCHEMA: &str = "knot.block.run/v1";

/// A semantic intent invocation. `payload` is deliberately opaque at G1; its
/// advertised schema is versioned and validation remains endpoint-side.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentInvocation {
    pub session: ProjectionSession,
    pub target: InstanceId,
    pub observed_epoch: SceneEpoch,
    pub observed_revision: Revision,
    pub intent: String,
    pub payload: Vec<u8>,
}

/// The result of endpoint-side intent validation and dispatch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentResult {
    Accepted,
    Rejected {
        reason: String,
    },
    Stale {
        current_epoch: SceneEpoch,
        current_revision: Revision,
    },
}

/// How a host reaches one endpoint, whatever sits between them.
///
/// The protocol has always described itself as running over an *unspecified*
/// carrier; this is that seam made real. A carrier moves request and response
/// bodies and surfaces notices, and knows nothing about what they mean.
///
/// Deliberately not `Send + Sync`: a browser carrier is single-threaded, and
/// requiring thread-safety here would exclude the target the protocol most
/// needs to reach. A host that wants a carrier on another thread owns that
/// choice; the trait does not impose it.
///
/// The surface is blocking, which is what the first carrier is. A network
/// carrier will want either an async sibling or a worker thread behind this
/// same shape, and that is a decision to make when one is written rather than
/// guessed at now.
pub trait Carrier {
    /// Send one request and wait for its response.
    fn request(&mut self, body: CarrierRequestBody) -> Result<CarrierResponseBody, CarrierError>;

    /// Take one already-received notice, if any is queued. Never blocks.
    fn take_notice(&mut self) -> Option<CarrierNotice>;

    /// Block until a notice arrives.
    fn wait_for_notice(&mut self) -> Result<CarrierNotice, CarrierError>;

    /// Release whatever the carrier holds open.
    ///
    /// `&mut self` rather than `self` so this is callable on a boxed carrier,
    /// which is the whole point of the seam. Implementations should tolerate a
    /// second call rather than assuming exactly one.
    fn shutdown(&mut self) -> Result<(), CarrierError>;
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[path = "resources.rs"]
mod resources;

pub use resources::{
    AssemblyError, CachePolicy, CacheRetention, ResourceChunkRequest, ResourceChunkResponse,
    ResourceRequest, ResourceResponse,
};

#[path = "text_payloads.rs"]
mod text_payloads;

pub use text_payloads::{
    DerivedCacheInfoV1, DerivedTextV1, EditableTextV1, InsertKnotClipV1, InsertKnotClipV2,
    KnotClipArtifactRoleV1, KnotClipArtifactV1, KnotClipFidelityV1, KnotClipObservedEdgeV1,
    KnotClipSelectorV1, KnotEffectV1, NativeGlyphV1, SaveTextV1, TextEncoding,
};

#[path = "retained.rs"]
mod retained;

pub use retained::{
    RETAINED_PAGES_CEILING, RETAINED_PAGES_DEFAULT, RetainedKeeping, RetainedLinkWire,
    RetainedPageWire, RetainedPagesLimits, RetainedPagesRequest, RetainedPagesResponse,
    RetainedTruncation, RetainedUnretainedWire, RetainedVersionWire,
};

#[path = "carrier.rs"]
mod carrier;

pub use carrier::{
    CarrierError, CarrierFailure, CarrierNotice, CarrierOutput, CarrierRequest, CarrierRequestBody,
    CarrierResponse, CarrierResponseBody, SessionOpen, SessionOpened, SessionStatus,
};

#[path = "selections.rs"]
mod selections;

pub use selections::{
    CoordinatedSelection, Selection, SelectionClause, SelectionResolution, SelectionRole,
    SelectionTarget,
};
