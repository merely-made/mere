// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The payload for a native Graphshell glyph resource.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeGlyphV1 {
    pub label: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Text encodings accepted by the first editable-text resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextEncoding {
    Utf8,
}

/// A versioned editable source disclosed by an endpoint.
///
/// `base_token` is opaque to the client. It binds the save to the exact
/// document version the endpoint disclosed without exposing a native path,
/// vault identifier, or causal frontier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditableTextV1 {
    pub address: String,
    pub media_type: String,
    pub encoding: TextEncoding,
    pub source: String,
    pub base_token: Vec<u8>,
    /// An authority-issued revision for durable source provenance.
    ///
    /// This is distinct from the private `base_token`. Unsigned sources,
    /// including files-in-place, leave it absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_revision: Option<[u8; 32]>,
    /// A host-authorized rendering derived from `source`. A sealed source
    /// endpoint may restore this projection from its attributable cache.
    ///
    /// The base token still names the authored source. Consumers must not
    /// offer this text as the save buffer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived: Option<DerivedTextV1>,
}

/// A revisioned effect result attached to an editable source presentation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedTextV1 {
    pub source: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache: Option<DerivedCacheInfoV1>,
}

/// Attribution for a resolved result. Sealed source endpoints may persist it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedCacheInfoV1 {
    pub effect: String,
    pub sources: Vec<String>,
    pub provider_version: String,
    pub policy_fingerprint: String,
    pub fetched_at_unix_ms: u64,
    pub source_revision: u64,
}

/// Typed payload for [`EDITABLE_TEXT_SAVE_INTENT`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveTextV1 {
    pub base_token: Vec<u8>,
    pub source: String,
}

/// Typed payload for [`KNOT_CLIP_INSERT_INTENT`].
///
/// The target document comes from the invocation binding. `knot_body` is a
/// semantic Knot block stream without document frontmatter; the endpoint
/// records the source fields as structured provenance before appending it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertKnotClipV1 {
    pub base_token: Vec<u8>,
    pub source_url: String,
    pub title: Option<String>,
    pub selector: Option<String>,
    pub knot_body: String,
}

/// A source selector anchored in the artifact that clipping observed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum KnotClipSelectorV1 {
    /// W3C-style text quote selector. Prefix and suffix improve re-anchoring
    /// after a live resource changes.
    TextQuote {
        artifact_role: KnotClipArtifactRoleV1,
        exact: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        suffix: Option<String>,
    },
    /// W3C-style half-open character positions in the observed artifact.
    TextPosition {
        artifact_role: KnotClipArtifactRoleV1,
        start: u64,
        end: u64,
    },
    /// DOM coordinates retained for browser-engine captures. The quote is the
    /// portable fallback when another engine cannot address the same DOM.
    DomRange {
        artifact_role: KnotClipArtifactRoleV1,
        anchor_path: Vec<u32>,
        anchor_offset: u64,
        focus_path: Vec<u32>,
        focus_offset: u64,
        quote: String,
    },
}

/// What one retained artifact says about the capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KnotClipArtifactRoleV1 {
    /// Bytes returned by the source transport before document execution.
    SourceResponse,
    /// A serialized representation of the state actually observed by the
    /// lowering, such as a post-script DOM snapshot.
    ObservedRepresentation,
}

/// Artifact bytes offered to the authority that owns evidence retention.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnotClipArtifactV1 {
    pub role: KnotClipArtifactRoleV1,
    pub media_type: String,
    pub canonical_uri: String,
    pub bytes: Vec<u8>,
}

/// One known fidelity fact about lowering the retained artifact to Djot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnotClipFidelityV1 {
    /// Stable machine-readable class such as `omitted-node` or
    /// `arrangement-unchecked`.
    pub class: String,
    /// Human-readable detail suitable for an Inspector.
    pub detail: String,
    /// Selector into the retained artifact when the fact is localizable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<KnotClipSelectorV1>,
}

/// One link observed inside the clipped selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnotClipObservedEdgeV1 {
    pub target: String,
    /// Consumer vocabulary such as `link`; this remains explicitly observed,
    /// rather than asserted as graph truth.
    pub relation: String,
}

/// Evidence-bearing payload for [`KNOT_CLIP_INSERT_INTENT`].
///
/// The target document still comes from the invocation binding. Artifact bytes
/// cross this call only to an endpoint that has advertised v2 and injected a
/// retention authority; the endpoint replaces them with content references in
/// authored provenance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertKnotClipV2 {
    pub base_token: Vec<u8>,
    pub source_url: String,
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selectors: Vec<KnotClipSelectorV1>,
    pub knot_body: String,
    pub artifacts: Vec<KnotClipArtifactV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fidelity: Vec<KnotClipFidelityV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discovered_edges: Vec<KnotClipObservedEdgeV1>,
}

/// Typed consent receipt for Knot's derived-state document effects.
///
/// `confirmed` records an explicit product gesture when policy requires one.
/// The endpoint still decides whether the action is admitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnotEffectV1 {
    pub base_token: Vec<u8>,
    pub confirmed: bool,
}
