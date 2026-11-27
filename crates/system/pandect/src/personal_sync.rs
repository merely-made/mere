// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! H7 personal-device synchronization for Graphshell's local Mere graph.
//!
//! Graphshell keeps its own event grammar and deterministic fold. Stickleback
//! supplies the shared causal authoring helpers, policy-before-insert storage,
//! and LogSync join/drain used by Commons and Knot.

use identity::AttestationKeys;
use insigne::CheckFault;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use eidetic::PrivacyClass;
use identity::{IdentityError, IdentityProvider};
use insigne::DerivedKeyAttestation;
use mere::model::author::Author;
use mere::model::{AssertionBasis, RelationSelector};
use mere::profile::core::{self as core_pack, ids};
use mere::session::SessionGraph;
use muniment::Backend;
use p2panda_core::cbor::{decode_cbor_strict, encode_cbor};
use p2panda_core::{Body, Hash, Header, Operation, SigningKey, Topic, VerifyingKey};
use p2panda_store::topics::TopicStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stickleback::{
    Admission, CausalEntry, CausalError, CausalIndex, CausalLimits, DataKeyring, GroupCiphertext,
    GroupPrekeyBundle, GroupSessionDispatch, MunimentStore, OperationPolicy, OperationProcessor,
    PendingCausalOperation, ProcessError, Reject, StoreTarget, author_head, causal_projection,
    observed_frontier, stable_writer_subject, validate_causal_metadata,
};
use uuid::Uuid;

#[path = "personal_sync/assertions.rs"]
mod assertions;
#[path = "personal_sync/relation.rs"]
mod relation;
pub use relation::RelationClaim;

use crate::access::{ACCESS_HISTORY_FACET, AccessHistory, AccessRecord};
use crate::{SAVED_SCENE_FACET_V2, SAVED_SCENE_FACET_V3, SavedSceneV2};

pub const PERSONAL_GRAPH_LOG: u64 = 0;
pub const PERSONAL_GRAPH_LIMITS: CausalLimits = CausalLimits {
    max_parents: 64,
    max_payload_bytes: 1024 * 1024,
};
pub const MAX_EVENTS_PER_OPERATION: usize = 256;

/// How one operation's body is protected.
///
/// Carried in the signed header rather than guessed from the body, so a device
/// knows what it is holding before it tries to read it. Knot's `KnotSyncExt`
/// names its profile the same way and for the same reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PersonalEncryption {
    /// The body is the CBOR record itself. Every operation written before
    /// encryption existed.
    #[default]
    Plaintext,
    /// The body is a `GroupCiphertext` sealed with this graph's data keyring.
    GroupV1,
}

impl PersonalEncryption {
    /// Whether this operation's body can be read without a key.
    ///
    /// Also the `skip_serializing_if` predicate below, which is load bearing:
    /// see [`PersonalGraphExt::encryption`].
    pub fn is_plaintext(&self) -> bool {
        matches!(self, Self::Plaintext)
    }
}

/// Signed addressing extension for one personal graph pool.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalGraphExt {
    pub graph: [u8; 32],
    /// **`skip_serializing_if` is not cosmetic here.** `Header::to_bytes`
    /// re-encodes the whole header, extensions included, and both `hash()` and
    /// signature verification digest that encoding. A field that serialized on
    /// every operation would change the bytes of headers already signed and
    /// stored, so every existing operation would fail to verify. Omitting the
    /// plaintext case keeps those headers byte-identical to what they already
    /// are, which is what makes reading old operations possible at all.
    #[serde(default, skip_serializing_if = "PersonalEncryption::is_plaintext")]
    pub encryption: PersonalEncryption,
    /// The frontier this operation observed, repeated where a device without
    /// the key can still read it.
    ///
    /// Knot carries its parents in the header for the same reason: causal
    /// admission cannot depend on a body it may not be able to open. The
    /// record keeps its own copy for operations written before this existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<[u8; 32]>,
    /// Who wrote this, checkable without the payload key.
    ///
    /// This is what makes a sealed operation admissible at all. The roster
    /// admits stable persona roots, and deriving one needs the attestation;
    /// while it lived only in the sealed body, a device without the key could
    /// not tell an admitted writer from a stranger, so it refused everything
    /// sealed and never saw those operations again.
    ///
    /// Not a secret: it attests a public derived key, and the operation is
    /// signed by that key in the clear regardless.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer_attestation: Option<DerivedKeyAttestation>,
}

/// The bounded, secret-free events Graphshell can place on its generic lane.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PersonalGraphEvent {
    AddNode {
        id: Uuid,
        address: String,
        title: String,
    },
    RemoveNode {
        id: Uuid,
    },
    SetTitle {
        node: Uuid,
        title: String,
    },
    AddTag {
        node: Uuid,
        tag: String,
    },
    RemoveTag {
        node: Uuid,
        tag: String,
    },
    AssertRelation {
        from: Uuid,
        to: Uuid,
        /// The relation by its predicate IRI (declared-lens D17).
        #[serde(flatten)]
        relation: RelationClaim,
        /// `None` is the legacy assertion event shape. Version 1 adds exact
        /// IDs and authored time for every relation family.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        assertion_version: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        statement_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asserted_at_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        basis: Option<AssertionBasis>,
    },
    RetractRelation {
        from: Uuid,
        to: Uuid,
        selector: RelationSelector,
    },
    RetractAssertion {
        from: Uuid,
        to: Uuid,
        statement_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        withdrawn_at_ms: Option<u64>,
    },
    SetFacet {
        node: Uuid,
        facet: String,
        value: Value,
    },
    RemoveFacet {
        node: Uuid,
        facet: String,
    },
    AppendAccess {
        record: AccessRecord,
    },
    SaveScene {
        node: Uuid,
        scene: SavedSceneV2,
    },
    /// scope=persona; movement=persona-synced opt-in; mutability=live;
    /// security=ordinary. The handler id is a preference, not a credential or
    /// a secret-bearing payload.
    SetHandlerPreference {
        key: String,
        handler: String,
    },
    ObserveBlobAvailability {
        observation: BlobAvailabilityObservation,
    },
    /// This device's group pre-key bundle, so a keyed member can add it.
    ///
    /// Public by construction: it is what another device needs in order to
    /// key this one, and it authenticates back to a Personae root.
    PublishPrekey {
        /// `stickleback::GroupPrekeyBundle::to_bytes`.
        bundle: Vec<u8>,
    },
    /// Name a persona root that should be able to read this graph.
    ///
    /// Intent, not the key itself. The key group reconciles toward this, which
    /// is what lets a revocation stand: every device folds the same set from
    /// the same operations, so none of them re-seats a device another one
    /// retired.
    ///
    /// Authority is checked when this is folded rather than when it is
    /// admitted. Only a device already in the reader set may name another, and
    /// the first is named by whoever creates the group. Checking it at
    /// admission would be circular, since the set being decided is the one the
    /// check would need.
    AdmitReader {
        root: [u8; 32],
        /// What a person calls this device. Never load bearing.
        label: String,
    },
    /// Withdraw a persona root's ability to read this graph.
    ///
    /// The departed device keeps what it could already read; no scheme takes
    /// that back. What this ends is everything written afterwards.
    RetireReader {
        root: [u8; 32],
    },
    /// One membership or rotation step in the graph's key group.
    ///
    /// The broadcast control frame plus the per-recipient direct frames, which
    /// the group scheme has already sealed to their recipients. Carrying them
    /// in the clear here discloses nothing and is what lets a device that has
    /// no key yet still receive one.
    GroupDispatch {
        /// `stickleback::GroupSessionDispatch::to_bytes`.
        dispatch: Vec<u8>,
    },
}

impl PersonalGraphEvent {
    /// Whether this event carries key agreement rather than graph content.
    ///
    /// Key traffic must stay readable by a device that has no key, since
    /// reading it is how that device gets one. Sealing it would be a lock
    /// whose only key is inside the box.
    pub fn is_key_agreement(&self) -> bool {
        matches!(
            self,
            Self::PublishPrekey { .. }
                | Self::GroupDispatch { .. }
                | Self::AdmitReader { .. }
                | Self::RetireReader { .. }
        )
    }
}

/// One immutable statement about which device currently holds addressed bytes.
///
/// Blob bytes never enter this record. Later observations supersede the
/// current device state without deleting its earlier chronology.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlobAvailabilityObservation {
    pub record_id: Uuid,
    pub container_id: Uuid,
    pub blob: [u8; 32],
    pub device: String,
    pub available: bool,
    pub at_ms: u64,
}

/// One signed authoring turn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalGraphRecord {
    pub events: Vec<PersonalGraphEvent>,
    #[serde(default)]
    pub parents: Vec<[u8; 32]>,
    #[serde(default)]
    pub writer_attestation: Option<DerivedKeyAttestation>,
}

/// Stable persona roots admitted to one personal graph pool.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncRoster {
    subjects: BTreeSet<[u8; 32]>,
}

impl SyncRoster {
    pub fn new(subjects: impl IntoIterator<Item = [u8; 32]>) -> Self {
        Self {
            subjects: subjects.into_iter().collect(),
        }
    }

    pub fn admits(&self, subject: &[u8; 32]) -> bool {
        self.subjects.contains(subject)
    }
}

/// One class of synthetic node: a node that exists only to carry a facet.
///
/// `AddNode` is otherwise ungated, so a carrier materializes on every admitted
/// device even where its facet is filtered out, leaving a titled node with
/// nothing on it. A rule ties the carrier to the facet it exists for.
///
/// This is presentation, not confidentiality. The personal lane is plaintext,
/// so a device that declines to project a carrier still receives and stores
/// the operation. The roster is what bounds who can read a personal graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticAddressRule {
    /// Address prefix that marks the carrier, e.g. `mere://transfer/`.
    pub prefix: String,
    /// The facet whose selection the carrier follows.
    pub facet: String,
    /// When set, the carrier projects only where the address names the local
    /// device as one of its path segments. A device that has not been told its
    /// own key projects every carrier, so a missed setting over-shows rather
    /// than silently hiding the feature.
    pub device_scoped: bool,
}

/// Local, user-owned projection settings. Operations remain retained so a
/// later settings change can reproject without asking peers to resend them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncSelection {
    facets: BTreeSet<String>,
    synthetic: Vec<SyntheticAddressRule>,
    local_device: Option<String>,
    pub access_records: bool,
    pub saved_scenes: bool,
    /// Opt-in persona preference lane. Disabled by default, so a device can
    /// retain the operation without projecting the preference locally.
    pub handler_preferences: bool,
    pub blob_availability: bool,
}

impl SyncSelection {
    pub fn with_facets(mut self, facets: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.facets = facets.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_access_records(mut self, enabled: bool) -> Self {
        self.access_records = enabled;
        self
    }

    pub fn with_saved_scenes(mut self, enabled: bool) -> Self {
        self.saved_scenes = enabled;
        self
    }

    pub fn with_handler_preferences(mut self, enabled: bool) -> Self {
        self.handler_preferences = enabled;
        self
    }

    pub fn with_blob_availability(mut self, enabled: bool) -> Self {
        self.blob_availability = enabled;
        self
    }

    pub fn with_synthetic_addresses(
        mut self,
        rules: impl IntoIterator<Item = SyntheticAddressRule>,
    ) -> Self {
        self.synthetic = rules.into_iter().collect();
        self
    }

    /// Tell this replica its own device key, so device-scoped carriers
    /// addressed elsewhere stay out of the projection.
    pub fn with_local_device(mut self, device: impl Into<String>) -> Self {
        self.local_device = Some(device.into());
        self
    }

    fn projects(&self, event: &PersonalGraphEvent) -> bool {
        match event {
            PersonalGraphEvent::AddNode { address, .. } => self.projects_node(address),
            PersonalGraphEvent::SetFacet { facet, .. }
            | PersonalGraphEvent::RemoveFacet { facet, .. } => self.facets.contains(facet),
            PersonalGraphEvent::AppendAccess { .. } => self.access_records,
            PersonalGraphEvent::SaveScene { .. } => self.saved_scenes,
            PersonalGraphEvent::SetHandlerPreference { .. } => self.handler_preferences,
            PersonalGraphEvent::ObserveBlobAvailability { .. } => self.blob_availability,
            _ => true,
        }
    }

    /// Whether a node address materializes here. Plain addresses always do;
    /// a carrier follows its [`SyntheticAddressRule`].
    ///
    /// A filtered carrier takes its facet with it: the facet event replays as
    /// `ReplaySetNodeFacetById`, which the kernel drops when the node is
    /// absent, so one gate covers both events.
    fn projects_node(&self, address: &str) -> bool {
        let Some(rule) = self
            .synthetic
            .iter()
            .find(|rule| address.starts_with(&rule.prefix))
        else {
            return true;
        };
        if !self.facets.contains(&rule.facet) {
            return false;
        }
        let Some(local) = self.local_device.as_deref().filter(|_| rule.device_scoped) else {
            return true;
        };
        address[rule.prefix.len()..]
            .split('/')
            .any(|segment| segment.eq_ignore_ascii_case(local))
    }
}

/// A scalar or mutually exclusive target edited concurrently.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncConflict {
    pub target: String,
    pub operations: Vec<[u8; 32]>,
}

/// Stable identity receipt retained beside the projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriterReceipt {
    pub operation: [u8; 32],
    pub signer: [u8; 32],
    pub stable_subject: [u8; 32],
}

/// One key-agreement step read off the lane, in causal order.
///
/// `author_root` is the stable writer subject the lane already authenticates,
/// which is exactly what `GroupSession::process` requires. Carrying it here
/// means the caller never has to re-derive who authored a control frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyAgreementStep {
    pub operation: [u8; 32],
    pub author_root: [u8; 32],
    pub step: KeyAgreementEvent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyAgreementEvent {
    /// A device published a pre-key bundle so it can be added.
    Prekey(Vec<u8>),
    /// A membership or rotation step.
    Dispatch(Vec<u8>),
}

/// Deterministic projection of one retained operation set; its graph type is
/// the kernel's until the surface readers move (engine split D241, D244).
pub struct SyncProjection<G: mere::session::SessionGraph> {
    pub graph: G,
    pub access_records: Vec<AccessRecord>,
    pub scenes: BTreeMap<Uuid, SavedSceneV2>,
    pub handler_preferences: BTreeMap<String, String>,
    pub blob_availability: Vec<BlobAvailabilityObservation>,
    pub available_blobs: BTreeMap<[u8; 32], BTreeSet<String>>,
    /// Persona roots the graph says may read it, with their labels.
    ///
    /// The condition the key group reconciles toward. Every device folds this
    /// identically from the same operations, so a device retired on one is
    /// retired everywhere without anyone being told twice.
    pub readers: BTreeMap<[u8; 32], String>,
    /// Operations this device holds and cannot read yet, because they are
    /// sealed and it has no key.
    ///
    /// Reported rather than hidden: a graph that is quietly missing part of
    /// itself looks identical to one that is complete, and the difference is
    /// exactly what an owner needs to see while a device waits to be keyed.
    pub unreadable: usize,
    /// Key-agreement steps in causal order, for a caller to hand to its group
    /// session. Present whether or not this device can read the sealed
    /// operations around them, since reading these is how it gets the key.
    pub key_agreement: Vec<KeyAgreementStep>,
    pub pending: Vec<PendingCausalOperation>,
    pub conflicts: Vec<SyncConflict>,
    pub writers: Vec<WriterReceipt>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PersonalGraphWireError {
    #[error("personal graph operation has no body")]
    MissingBody,
    #[error("personal graph operation body is malformed")]
    Malformed,
    /// The operation is sealed and this device holds no key for this graph.
    ///
    /// Loud rather than silent. A device that cannot read an operation must
    /// not store it as though it had, because the graph would then differ
    /// between devices with nothing saying why.
    #[error("personal graph operation is sealed and this device has no key for the graph")]
    NoKey,
    #[error("personal graph operation could not be unsealed: {0}")]
    Unsealable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PersonalGraphError {
    #[error(transparent)]
    Store(#[from] muniment::StoreError),
    #[error(transparent)]
    Wire(#[from] PersonalGraphWireError),
    #[error(transparent)]
    Causal(#[from] CausalError),
    #[error(transparent)]
    Process(#[from] ProcessError),
    #[error("local authoring is blocked by {0} operations with missing causal history")]
    PendingHistory(usize),
    #[error("event is outside this device's selected sync profile: {0}")]
    NotSelected(String),
    #[error("event is categorically excluded from generic graph sync: {0}")]
    Excluded(String),
    #[error("writer is not in this personal graph roster")]
    WriterNotAdmitted,
    #[error("personal graph operation could not be sealed: {0}")]
    Seal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PersonalGraphIdentityError {
    #[error(transparent)]
    Identity(#[from] IdentityError),
    #[error("identity provider returned an invalid personal-graph writer attestation: {0}")]
    InvalidAttestation(CheckFault),
    #[error("identity provider attested a different personal-graph writer")]
    WriterMismatch,
}

#[derive(Clone)]
struct PersonalGraphPolicy {
    graph: [u8; 32],
    roster: SyncRoster,
    /// Admission reads the record to check its events, so a sealed operation
    /// cannot be admitted without the key. Absent, a sealed operation is
    /// refused with a reason rather than stored unvalidated.
    keyring: Option<Arc<DataKeyring>>,
}

impl OperationPolicy<PersonalGraphExt> for PersonalGraphPolicy {
    type LogId = u64;

    fn admit(
        &self,
        operation: &Operation<PersonalGraphExt>,
    ) -> Result<Admission<Self::LogId>, Reject> {
        if operation.header.extensions.graph != self.graph {
            return Err(Reject::new(
                "wrong-personal-graph",
                "operation addresses another personal graph",
            ));
        }
        // Everything checkable without the body, first. A sealed operation
        // this device cannot open is still attributable and still bounded,
        // and refusing it would lose it: LogSync does not offer twice, so a
        // device keyed later would carry a permanent hole.
        let subject = header_subject(operation)?;
        let record = match from_operation(operation, self.keyring.as_deref()) {
            Ok(record) => record,
            Err(PersonalGraphWireError::NoKey) => {
                let Some(subject) = subject else {
                    return Err(Reject::new(
                        "unattributable-sealed-operation",
                        "a sealed operation carries no writer attestation in its header, so this                          device cannot tell whether its writer is admitted",
                    ));
                };
                if !self.roster.admits(&subject) {
                    return Err(Reject::new(
                        "personal-graph-writer-not-admitted",
                        "stable writer subject is outside this graph roster",
                    ));
                }
                validate_causal_metadata(
                    operation,
                    &operation.header.extensions.parents,
                    PERSONAL_GRAPH_LIMITS,
                )
                .map_err(|error| {
                    Reject::new("invalid-personal-graph-causality", error.to_string())
                })?;
                // Its events go unchecked, which is the price of holding it.
                // They were authored by an admitted writer and the payload is
                // size-bounded above; the alternative is losing the operation
                // entirely. Projection re-admits it once the key arrives, so
                // nothing unread is ever folded into the graph.
                return Ok(Admission::keep(StoreTarget::new(
                    Topic::from(self.graph),
                    PERSONAL_GRAPH_LOG,
                )));
            },
            Err(error) => {
                return Err(Reject::new(
                    "invalid-personal-graph-record",
                    error.to_string(),
                ));
            },
        };
        validate_causal_metadata(operation, &record.parents, PERSONAL_GRAPH_LIMITS)
            .map_err(|error| Reject::new("invalid-personal-graph-causality", error.to_string()))?;
        if record.events.is_empty() || record.events.len() > MAX_EVENTS_PER_OPERATION {
            return Err(Reject::new(
                "personal-graph-event-count",
                format!(
                    "operation carries {} events; expected 1..={MAX_EVENTS_PER_OPERATION}",
                    record.events.len()
                ),
            ));
        }
        for event in &record.events {
            validate_event(event)?;
        }
        one_kind_per_operation(&record.events)?;
        // Key agreement must be readable without a key, so it may not be
        // sealed. Checked at intake rather than trusted, because a sealed
        // dispatch would be undiagnosable from the outside: it would look
        // exactly like an operation this device simply has no key for.
        if operation.header.extensions.encryption != PersonalEncryption::Plaintext
            && record
                .events
                .iter()
                .any(PersonalGraphEvent::is_key_agreement)
        {
            return Err(Reject::new(
                "sealed-key-agreement",
                "key agreement must travel in the clear; sealing it would need the key it carries",
            ));
        }
        let subject = match subject {
            Some(subject) => subject,
            None => stable_subject(operation, &record)?,
        };
        if !self.roster.admits(&subject) {
            return Err(Reject::new(
                "personal-graph-writer-not-admitted",
                "stable writer subject is outside this graph roster",
            ));
        }
        Ok(Admission::keep(StoreTarget::new(
            Topic::from(self.graph),
            PERSONAL_GRAPH_LOG,
        )))
    }
}

/// The stable writer subject, from the header alone.
///
/// `None` for operations written before the attestation moved into the header,
/// which are readable anyway and fall back to the record.
fn header_subject(operation: &Operation<PersonalGraphExt>) -> Result<Option<[u8; 32]>, Reject> {
    let Some(attestation) = operation.header.extensions.writer_attestation.as_ref() else {
        return Ok(None);
    };
    stable_writer_subject(
        *operation.header.verifying_key.as_bytes(),
        Some(attestation),
        &personal_graph_identity_salt(operation.header.extensions.graph),
    )
    .map(Some)
    .map_err(|error| Reject::new(error.code(), error.to_string()))
}

fn validate_event(event: &PersonalGraphEvent) -> Result<(), Reject> {
    assertions::validate_event(event)?;
    match event {
        PersonalGraphEvent::AddNode { address, .. } if address.trim().is_empty() => Err(
            Reject::new("empty-personal-graph-address", "node address is empty"),
        ),
        PersonalGraphEvent::AddTag { tag, .. } | PersonalGraphEvent::RemoveTag { tag, .. }
            if tag.trim().is_empty() =>
        {
            Err(Reject::new("empty-personal-graph-tag", "tag is empty"))
        },
        PersonalGraphEvent::SetFacet { facet, .. }
        | PersonalGraphEvent::RemoveFacet { facet, .. } => validate_facet_name(facet),
        PersonalGraphEvent::AppendAccess { record }
            if matches!(
                record.privacy,
                PrivacyClass::LocalOnly | PrivacyClass::MootScoped
            ) =>
        {
            Err(Reject::new(
                "access-record-not-portable",
                "access record was not marked for trusted-peer or public portability",
            ))
        },
        PersonalGraphEvent::SetHandlerPreference { key, handler }
            if key.trim().is_empty() || handler.trim().is_empty() =>
        {
            Err(Reject::new(
                "empty-handler-preference",
                "handler preference key and value must be non-empty",
            ))
        },
        PersonalGraphEvent::ObserveBlobAvailability { observation }
            if observation.device.trim().is_empty() =>
        {
            Err(Reject::new(
                "empty-blob-availability-device",
                "blob availability observation must name its source device",
            ))
        },
        // Decoding a bundle verifies its identity signature back to a Personae
        // root, so a forged pre-key is refused at intake rather than at the
        // moment somebody tries to key it.
        PersonalGraphEvent::PublishPrekey { bundle } => GroupPrekeyBundle::from_bytes(bundle)
            .map(|_| ())
            .map_err(|error| Reject::new("invalid-group-prekey", error.to_string())),
        PersonalGraphEvent::GroupDispatch { dispatch } => {
            decode_cbor_strict::<GroupSessionDispatch, _>(dispatch.as_slice())
                .map(|_| ())
                .map_err(|error| Reject::new("invalid-group-dispatch", error.to_string()))
        },
        _ => Ok(()),
    }
}

/// Refuse an operation that mixes key agreement with graph content.
///
/// Not tidiness. Key agreement is authored in the clear, so a content event
/// sharing the operation would be published in the clear with it. Separating
/// them at the grammar means that cannot happen by accident.
fn one_kind_per_operation(events: &[PersonalGraphEvent]) -> Result<(), Reject> {
    let key = events
        .iter()
        .filter(|event| event.is_key_agreement())
        .count();
    if key == 0 || key == events.len() {
        return Ok(());
    }
    Err(Reject::new(
        "mixed-key-and-content-operation",
        "an operation carries either key agreement or graph content, never both",
    ))
}

fn validate_facet_name(facet: &str) -> Result<(), Reject> {
    let normalized = facet.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized.len() > 256 {
        return Err(Reject::new(
            "invalid-personal-graph-facet",
            "facet id is empty or too long",
        ));
    }
    // A version-4 scene travels as an ordinary facet, opaque, so an older
    // device stores and passes it on (dynamics grammar plan, F158); the
    // facets the typed scene event writes stay reserved to it.
    if facet == ACCESS_HISTORY_FACET
        || facet == SAVED_SCENE_FACET_V3
        || facet == SAVED_SCENE_FACET_V2
    {
        return Err(Reject::new(
            "reserved-personal-graph-facet",
            "facet has a dedicated append or scene event",
        ));
    }
    const SECRET_MARKERS: &[&str] = &[
        "credential",
        "decrypted",
        "private-epoch",
        "private_key",
        "private-key",
        "seed",
        "vault-root",
        "vault_root",
    ];
    if SECRET_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Err(Reject::new(
            "secret-bearing-facet",
            "credential and secret-bearing facets cannot use generic graph sync",
        ));
    }
    Ok(())
}

pub fn personal_graph_identity_salt(graph: [u8; 32]) -> Vec<u8> {
    let mut salt = Vec::with_capacity(64);
    salt.extend_from_slice(b"graphshell.personal-graph.writer.v1/");
    salt.extend_from_slice(&graph);
    salt
}

fn stable_subject(
    operation: &Operation<PersonalGraphExt>,
    record: &PersonalGraphRecord,
) -> Result<[u8; 32], Reject> {
    stable_writer_subject(
        *operation.header.verifying_key.as_bytes(),
        record.writer_attestation.as_ref(),
        &personal_graph_identity_salt(operation.header.extensions.graph),
    )
    .map_err(|error| Reject::new(error.code(), error.to_string()))
}

/// Read one operation's record, unsealing it when the header says it is sealed.
///
/// The header names the protection, so this never guesses from the bytes. A
/// device without the key gets [`PersonalGraphWireError::NoKey`] rather than a
/// parse failure, because "I cannot read this" and "this is corrupt" call for
/// different responses from whoever sees the error.
pub fn from_operation(
    operation: &Operation<PersonalGraphExt>,
    keyring: Option<&DataKeyring>,
) -> Result<PersonalGraphRecord, PersonalGraphWireError> {
    let body = operation
        .body
        .as_ref()
        .ok_or(PersonalGraphWireError::MissingBody)?;
    let bytes = body.to_bytes();
    match operation.header.extensions.encryption {
        PersonalEncryption::Plaintext => {
            decode_cbor_strict(bytes.as_slice()).map_err(|_| PersonalGraphWireError::Malformed)
        },
        PersonalEncryption::GroupV1 => {
            let keyring = keyring.ok_or(PersonalGraphWireError::NoKey)?;
            let envelope: GroupCiphertext = decode_cbor_strict(bytes.as_slice())
                .map_err(|_| PersonalGraphWireError::Malformed)?;
            let plaintext = keyring
                .open(&envelope)
                .map_err(|error| PersonalGraphWireError::Unsealable(error.to_string()))?;
            decode_cbor_strict(plaintext.as_slice()).map_err(|_| PersonalGraphWireError::Malformed)
        },
    }
}

/// Write one operation, sealing it when a keyring is present.
///
/// Sealing is opt-in by the caller holding a key, not a default: a device that
/// sealed while a sibling had no key would still replicate, and the sibling
/// would simply stop being able to read the graph. Until keys are distributed,
/// no keyring means no sealing means nothing breaks.
fn to_operation(
    signing_seed: [u8; 32],
    graph: [u8; 32],
    record: &PersonalGraphRecord,
    seq_num: u32,
    backlink: Option<[u8; 32]>,
    keyring: Option<&DataKeyring>,
) -> Result<Operation<PersonalGraphExt>, PersonalGraphError> {
    let signing_key = SigningKey::from_bytes(&signing_seed);
    let record_bytes = encode_cbor(record).expect("a personal graph record always CBOR-encodes");
    // Key agreement stays in the clear even here. A device with no key reads
    // this operation to get one, so sealing it would be a lock whose only key
    // is inside the box. Admission refuses a sealed one for the same reason.
    let carries_keys = record
        .events
        .iter()
        .any(PersonalGraphEvent::is_key_agreement);
    let (body_bytes, encryption) = match keyring.filter(|_| !carries_keys) {
        Some(keyring) => {
            let envelope = keyring
                .seal_random(&record_bytes)
                .map_err(|error| PersonalGraphError::Seal(error.to_string()))?;
            let sealed = encode_cbor(&envelope)
                .map_err(|error| PersonalGraphError::Seal(error.to_string()))?;
            (sealed, PersonalEncryption::GroupV1)
        },
        None => (record_bytes, PersonalEncryption::Plaintext),
    };
    let body = Body::from_bytes(&body_bytes);
    // p2panda 0.7.1 made the header's CBOR cache, size and digest private
    // and folded signing into the builder: `build` encodes, signs and
    // caches the digest in one step, so the struct-literal + `sign` pair
    // has no equivalent. `body` sets payload_size and payload_hash.
    let header = Header::builder()
        .body(&body_bytes)
        .seq_num(seq_num)
        .backlink(backlink.map(Hash::from))
        .build(
            &signing_key,
            PersonalGraphExt {
                graph,
                encryption,
                parents: record.parents.clone(),
                writer_attestation: record.writer_attestation.clone(),
            },
        );
    let hash = header.hash();
    Ok(Operation {
        hash,
        header,
        body: Some(body),
    })
}

#[derive(Clone)]
struct StoredRecord {
    operation: Operation<PersonalGraphExt>,
    record: PersonalGraphRecord,
    log_id: u64,
}

async fn load_records<B: Backend + Clone + Send + Sync + 'static>(
    store: &MunimentStore<B, PersonalGraphExt>,
    graph: [u8; 32],
    keyring: Option<&Arc<DataKeyring>>,
) -> Result<(Vec<StoredRecord>, usize), PersonalGraphError> {
    let by_author: BTreeMap<VerifyingKey, Vec<u64>> =
        TopicStore::<Topic, VerifyingKey, u64>::resolve(store, &Topic::from(graph)).await?;
    let mut records = Vec::new();
    let mut unreadable = 0usize;
    for (author, mut logs) in by_author {
        logs.sort_unstable();
        logs.dedup();
        for log_id in logs {
            let entries = store
                .get_log_entries(&author, &log_id, None, None)
                .await?
                .unwrap_or_default();
            for (operation, _) in entries {
                match from_operation(&operation, keyring.map(Arc::as_ref)) {
                    Ok(record) => records.push(StoredRecord {
                        operation,
                        record,
                        log_id,
                    }),
                    // Held but not readable yet. Skipped rather than fatal:
                    // one operation this device has no key for must not stop
                    // it projecting the rest, and it becomes readable the
                    // moment the key arrives, with nothing to re-fetch.
                    Err(PersonalGraphWireError::NoKey) => unreadable += 1,
                    Err(error) => return Err(error.into()),
                }
            }
        }
    }
    Ok((records, unreadable))
}

fn causal_entries(records: &[StoredRecord]) -> Vec<CausalEntry<u64>> {
    records
        .iter()
        .map(|record| {
            CausalEntry::from_operation(
                &record.operation,
                record.log_id,
                record.record.parents.clone(),
            )
        })
        .collect()
}

pub async fn accept_into<B: Backend + Clone + Send + Sync + 'static>(
    store: &MunimentStore<B, PersonalGraphExt>,
    graph: [u8; 32],
    roster: &SyncRoster,
    keyring: Option<&Arc<DataKeyring>>,
    operation: &Operation<PersonalGraphExt>,
) -> Result<bool, ProcessError> {
    let processor = OperationProcessor::new(
        store.clone(),
        PersonalGraphPolicy {
            graph,
            roster: roster.clone(),
            keyring: keyring.map(Arc::clone),
        },
    );
    processor.preflight(operation)?;
    if let Ok(record) = from_operation(operation, keyring.map(Arc::as_ref)) {
        let (records, _) = load_records(store, graph, keyring)
            .await
            .map_err(|error| Reject::new("assertion-history-unavailable", error.to_string()))?;
        let mut ids = assertions::IdentityIndex::default();
        let entries = causal_entries(&records);
        let entries_by_hash = causal_entry_index(&entries);
        let causal = causal_projection(&entries)
            .map_err(|error| Reject::new("assertion-history-unavailable", error.to_string()))?;
        for &index in &causal.order {
            let stored = &records[index];
            observe_assertion_ids(
                &mut ids,
                &stored.operation,
                &stored.record,
                &entries_by_hash,
            )
            .map_err(|error| Reject::new("conflicting-assertion-id", error.to_string()))?;
        }
        observe_assertion_ids(&mut ids, operation, &record, &entries_by_hash)
            .map_err(|error| Reject::new("conflicting-assertion-id", error.to_string()))?;
    }
    Ok(processor.process(operation).await?.inserted())
}

fn observe_assertion_ids(
    ids: &mut assertions::IdentityIndex,
    operation: &Operation<PersonalGraphExt>,
    record: &PersonalGraphRecord,
    entries_by_hash: &BTreeMap<[u8; 32], &CausalEntry<u64>>,
) -> Result<(), PersonalGraphError> {
    let subject = stable_subject(operation, record)
        .map_err(|error| PersonalGraphError::Excluded(error.to_string()))?;
    let asserter = Author::person(hex32(&subject)).asserter_iri();
    let operation_hash = *operation.hash.as_bytes();
    let has_retraction = record
        .events
        .iter()
        .any(|event| matches!(event, PersonalGraphEvent::RetractAssertion { .. }));
    let causally_known = if has_retraction {
        causal_known_operations(
            entries_by_hash,
            operation_hash,
            record.parents.iter().copied().chain(
                operation
                    .header
                    .backlink
                    .as_ref()
                    .map(|hash| *hash.as_bytes()),
            ),
        )
    } else {
        std::collections::HashSet::new()
    };
    for (index, event) in record.events.iter().enumerate() {
        let fallback = format!(
            "urn:mere:statement:personal:{}:{index}",
            hex32(operation.hash.as_bytes())
        );
        ids.observe(event, &asserter, &fallback, operation_hash, &causally_known)?;
    }
    Ok(())
}

fn causal_known_operations(
    by_hash: &BTreeMap<[u8; 32], &CausalEntry<u64>>,
    operation: [u8; 32],
    direct: impl IntoIterator<Item = [u8; 32]>,
) -> std::collections::HashSet<[u8; 32]> {
    let mut known = std::collections::HashSet::from([operation]);
    let mut pending = direct.into_iter().collect::<Vec<_>>();
    while let Some(hash) = pending.pop() {
        if !known.insert(hash) {
            continue;
        }
        if let Some(entry) = by_hash.get(&hash) {
            pending.extend(entry.parents.iter().copied());
            pending.extend(entry.backlink);
        }
    }
    known
}

fn causal_entry_index(entries: &[CausalEntry<u64>]) -> BTreeMap<[u8; 32], &CausalEntry<u64>> {
    entries
        .iter()
        .map(|entry| (entry.operation, entry))
        .collect()
}

/// One local device replica.
pub struct PersonalGraphReplica<B: Backend + Clone + Send + Sync + 'static> {
    store: MunimentStore<B, PersonalGraphExt>,
    graph: [u8; 32],
    signing_seed: [u8; 32],
    writer_attestation: Option<DerivedKeyAttestation>,
    roster: SyncRoster,
    selection: SyncSelection,
    /// Present only once this device has been given the graph's key. Absent,
    /// this replica writes plaintext exactly as before, which is what keeps a
    /// sibling that has no key still able to read what this one writes.
    keyring: Option<Arc<DataKeyring>>,
}

impl<B: Backend + Clone + Send + Sync + 'static> PersonalGraphReplica<B> {
    pub fn new(
        backend: B,
        graph: [u8; 32],
        signing_seed: [u8; 32],
        roster: SyncRoster,
        selection: SyncSelection,
    ) -> Self {
        Self {
            store: MunimentStore::new(backend),
            graph,
            signing_seed,
            writer_attestation: None,
            roster,
            selection,
            keyring: None,
        }
    }

    /// Hand this replica the graph's data keyring.
    ///
    /// Sealing starts from the next operation it authors. Reading is unrelated
    /// to this call: an operation already stored plaintext stays readable, and
    /// the header on each operation says which it is.
    pub fn set_keyring(&mut self, keyring: Arc<DataKeyring>) {
        self.keyring = Some(keyring);
    }

    /// Whether this replica seals what it writes.
    pub fn seals(&self) -> bool {
        self.keyring.is_some()
    }

    /// Replace the roster this replica projects through.
    ///
    /// The replica keeps its own copy because projection re-checks every
    /// stored operation against it. Admission and projection must therefore
    /// move together: a device admitted on intake but absent here has its
    /// operations accepted into the store and then refused on the way out,
    /// which fails the whole projection rather than the one operation.
    pub fn set_roster(&mut self, roster: SyncRoster) {
        self.roster = roster;
    }

    pub fn for_identity<P: IdentityProvider + ?Sized>(
        backend: B,
        graph: [u8; 32],
        identity: &P,
        roster: SyncRoster,
        selection: SyncSelection,
    ) -> Result<Self, PersonalGraphIdentityError> {
        let salt = personal_graph_identity_salt(graph);
        let keypair = identity.derive_keypair(&salt)?;
        let attestation = identity.attest_derived_key(&salt)?;
        let checked = attestation
            .check(&salt)
            .map_err(PersonalGraphIdentityError::InvalidAttestation)?;
        if *checked.derived() != keypair.public_key().to_bytes() {
            return Err(PersonalGraphIdentityError::WriterMismatch);
        }
        Ok(Self {
            store: MunimentStore::new(backend),
            graph,
            signing_seed: keypair.to_seed(),
            writer_attestation: Some(attestation),
            roster,
            selection,
            keyring: None,
        })
    }

    pub fn sync_store(&self) -> MunimentStore<B, PersonalGraphExt> {
        self.store.clone()
    }

    pub fn roster(&self) -> &SyncRoster {
        &self.roster
    }

    /// Whether this device authors and projects blob-availability records.
    ///
    /// Anything that stages bytes for a sibling depends on this: without it,
    /// the bytes are held but never advertised, so no peer can learn to ask
    /// for them. Callers check it before staging rather than discovering it as
    /// a refused event afterwards.
    pub fn serves_blob_availability(&self) -> bool {
        self.selection.blob_availability
    }

    pub async fn author(
        &mut self,
        mut events: Vec<PersonalGraphEvent>,
    ) -> Result<Operation<PersonalGraphExt>, PersonalGraphError> {
        if events.is_empty() || events.len() > MAX_EVENTS_PER_OPERATION {
            return Err(PersonalGraphError::Excluded(format!(
                "expected 1..={MAX_EVENTS_PER_OPERATION} events"
            )));
        }
        for event in &mut events {
            if matches!(
                event,
                PersonalGraphEvent::RetractRelation { selector, .. } if names_semantic(selector)
            ) {
                return Err(PersonalGraphError::Excluded(
                    "new semantic retractions must name an assertion id".into(),
                ));
            }
            assertions::prepare_event(event);
            validate_event(event)
                .map_err(|error| PersonalGraphError::Excluded(error.to_string()))?;
            if !self.selection.projects(event) {
                return Err(PersonalGraphError::NotSelected(event_target(event)));
            }
        }

        let (records, _) = load_records(&self.store, self.graph, self.keyring.as_ref()).await?;
        let entries = causal_entries(&records);
        let causal = causal_projection(&entries)?;
        if !causal.pending.is_empty() {
            return Err(PersonalGraphError::PendingHistory(causal.pending.len()));
        }
        let parents = observed_frontier(&entries)?;
        let signing_key = SigningKey::from_bytes(&self.signing_seed);
        let subject = if let Some(attestation) = self.writer_attestation.as_ref() {
            attestation
                .master_public_key()
                .map(|key| key.to_bytes())
                .map_err(|_| PersonalGraphError::WriterNotAdmitted)?
        } else {
            *signing_key.verifying_key().as_bytes()
        };
        if !self.roster.admits(&subject) {
            return Err(PersonalGraphError::WriterNotAdmitted);
        }
        let (seq_num, backlink) = author_head(
            &entries,
            *signing_key.verifying_key().as_bytes(),
            &PERSONAL_GRAPH_LOG,
        )?;
        let operation = to_operation(
            self.signing_seed,
            self.graph,
            &PersonalGraphRecord {
                events,
                parents,
                writer_attestation: self.writer_attestation.clone(),
            },
            seq_num,
            backlink,
            self.keyring.as_deref(),
        )?;
        self.accept(&operation).await?;
        Ok(operation)
    }

    pub async fn accept(
        &self,
        operation: &Operation<PersonalGraphExt>,
    ) -> Result<bool, ProcessError> {
        accept_into(
            &self.store,
            self.graph,
            &self.roster,
            self.keyring.as_ref(),
            operation,
        )
        .await
    }

    /// The projection of what this replica holds into any graph, through the
    /// host's `fold` (engine split D259).
    pub async fn projection_with<G: SessionGraph, F: PersonalFold<G> + ?Sized>(
        &self,
        fold: &F,
    ) -> Result<SyncProjection<G>, PersonalGraphError> {
        materialize_with(
            &self.store,
            self.graph,
            &self.roster,
            self.keyring.as_ref(),
            &self.selection,
            fold,
        )
        .await
    }
}

/// Read the key agreement without holding a key.
///
/// The pass a device runs before it can read anything else: it has been
/// paired, it has published a pre-key, and it needs the dispatch that keys it.
/// Sealed operations are skipped rather than refused, because being unable to
/// read them is the entire reason this runs.
///
/// Skipping is safe here and nowhere else. Key agreement is plaintext by
/// construction and admission refuses a sealed one, so a sealed operation
/// cannot be a step this pass needs.
pub async fn key_agreement<B: Backend + Clone + Send + Sync + 'static>(
    store: &MunimentStore<B, PersonalGraphExt>,
    graph: [u8; 32],
) -> Result<Vec<KeyAgreementStep>, PersonalGraphError> {
    let by_author: BTreeMap<VerifyingKey, Vec<u64>> =
        TopicStore::<Topic, VerifyingKey, u64>::resolve(store, &Topic::from(graph)).await?;
    let mut steps = Vec::new();
    for (author, mut logs) in by_author {
        logs.sort_unstable();
        logs.dedup();
        for log_id in logs {
            let entries = store
                .get_log_entries(&author, &log_id, None, None)
                .await?
                .unwrap_or_default();
            for (operation, _) in entries {
                if operation.header.extensions.encryption != PersonalEncryption::Plaintext {
                    continue;
                }
                let Ok(record) = from_operation(&operation, None) else {
                    continue;
                };
                let Ok(subject) = stable_subject(&operation, &record) else {
                    continue;
                };
                for event in &record.events {
                    let step = match event {
                        PersonalGraphEvent::PublishPrekey { bundle } => {
                            KeyAgreementEvent::Prekey(bundle.clone())
                        },
                        PersonalGraphEvent::GroupDispatch { dispatch } => {
                            KeyAgreementEvent::Dispatch(dispatch.clone())
                        },
                        _ => continue,
                    };
                    steps.push((
                        operation.header.seq_num,
                        KeyAgreementStep {
                            operation: *operation.hash.as_bytes(),
                            author_root: subject,
                            step,
                        },
                    ));
                }
            }
        }
    }
    // Causal order needs every operation, which this pass deliberately does not
    // have. Sequence per author with the operation hash as a tiebreak is a
    // deterministic order every device agrees on, which is what the group
    // scheme needs; it orders its own steps internally.
    steps.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.operation.cmp(&right.1.operation))
    });
    Ok(steps.into_iter().map(|(_, step)| step).collect())
}

/// Materialize the personal graph into any graph, through the host's `fold`
/// (engine split D259): the protocol, its admission and its bookkeeping stay
/// here, and the fold writes the graph events into the host's graph.
pub async fn materialize_with<B, G, F>(
    store: &MunimentStore<B, PersonalGraphExt>,
    graph_id: [u8; 32],
    roster: &SyncRoster,
    keyring: Option<&Arc<DataKeyring>>,
    selection: &SyncSelection,
    fold: &F,
) -> Result<SyncProjection<G>, PersonalGraphError>
where
    B: Backend + Clone + Send + Sync + 'static,
    G: SessionGraph,
    F: PersonalFold<G> + ?Sized,
{
    let (records, unreadable) = load_records(store, graph_id, keyring).await?;
    let entries = causal_entries(&records);
    let entries_by_hash = causal_entry_index(&entries);
    let causal = causal_projection(&entries)?;
    let policy = PersonalGraphPolicy {
        graph: graph_id,
        roster: roster.clone(),
        keyring: keyring.map(Arc::clone),
    };
    let processor = OperationProcessor::new(store.clone(), policy);
    let conflicts = collect_conflicts(&records, &entries, &causal.order, selection);
    let mut graph = G::default();
    let mut access = BTreeMap::<Uuid, AccessRecord>::new();
    let mut scenes = BTreeMap::new();
    let mut handlers = BTreeMap::new();
    let mut blob_availability = BTreeMap::<Uuid, BlobAvailabilityObservation>::new();
    let mut writers = Vec::new();
    let mut key_agreement = Vec::new();
    let mut assertion_ids = assertions::IdentityIndex::default();

    for &index in &causal.order {
        let stored = &records[index];
        processor.preflight(&stored.operation)?;
        let subject = stable_subject(&stored.operation, &stored.record)
            .map_err(|error| PersonalGraphError::Excluded(error.to_string()))?;
        let asserter_iri = Author::person(hex32(&subject)).asserter_iri();
        writers.push(WriterReceipt {
            operation: *stored.operation.hash.as_bytes(),
            signer: *stored.operation.header.verifying_key.as_bytes(),
            stable_subject: subject,
        });
        let has_retraction = stored
            .record
            .events
            .iter()
            .any(|event| matches!(event, PersonalGraphEvent::RetractAssertion { .. }));
        let causally_known = if has_retraction {
            causal_known_operations(
                &entries_by_hash,
                *stored.operation.hash.as_bytes(),
                stored.record.parents.iter().copied().chain(
                    stored
                        .operation
                        .header
                        .backlink
                        .as_ref()
                        .map(|hash| *hash.as_bytes()),
                ),
            )
        } else {
            std::collections::HashSet::new()
        };
        for (event_index, event) in stored.record.events.iter().enumerate() {
            let fallback_id = format!(
                "urn:mere:statement:personal:{}:{event_index}",
                hex32(stored.operation.hash.as_bytes()),
            );
            assertion_ids.observe(
                event,
                &asserter_iri,
                &fallback_id,
                *stored.operation.hash.as_bytes(),
                &causally_known,
            )?;
            // Never gated by selection. A device cannot opt out of the traffic
            // that keys it and still expect to read the graph.
            match event {
                PersonalGraphEvent::PublishPrekey { bundle } => {
                    key_agreement.push(KeyAgreementStep {
                        operation: *stored.operation.hash.as_bytes(),
                        author_root: subject,
                        step: KeyAgreementEvent::Prekey(bundle.clone()),
                    });
                    continue;
                },
                PersonalGraphEvent::GroupDispatch { dispatch } => {
                    key_agreement.push(KeyAgreementStep {
                        operation: *stored.operation.hash.as_bytes(),
                        author_root: subject,
                        step: KeyAgreementEvent::Dispatch(dispatch.clone()),
                    });
                    continue;
                },
                _ => {},
            }
            if selection.projects(event) {
                apply_event(
                    fold,
                    &mut graph,
                    &mut access,
                    &mut scenes,
                    &mut handlers,
                    &mut blob_availability,
                    &asserter_iri,
                    &fallback_id,
                    event,
                )?;
            }
        }
    }

    let mut access_records: Vec<_> = access.into_values().collect();
    access_records.sort_by_key(|record| (record.at_ms, record.device.clone(), record.record_id));
    if selection.access_records {
        let mut by_node = BTreeMap::<Uuid, Vec<AccessRecord>>::new();
        for record in &access_records {
            by_node
                .entry(record.container_id)
                .or_default()
                .push(record.clone());
        }
        for (node, records) in by_node {
            fold.set_access_history(&mut graph, node, AccessHistory { records });
        }
    }

    let mut blob_availability = blob_availability.into_values().collect::<Vec<_>>();
    blob_availability.sort_by_key(|observation| {
        (
            observation.at_ms,
            observation.device.clone(),
            observation.record_id,
        )
    });
    let mut latest = BTreeMap::<([u8; 32], String), &BlobAvailabilityObservation>::new();
    for observation in &blob_availability {
        let key = (observation.blob, observation.device.clone());
        let replace = latest.get(&key).is_none_or(|current| {
            (observation.at_ms, observation.record_id) > (current.at_ms, current.record_id)
        });
        if replace {
            latest.insert(key, observation);
        }
    }
    let mut available_blobs = BTreeMap::<[u8; 32], BTreeSet<String>>::new();
    for ((blob, device), observation) in latest {
        if observation.available {
            available_blobs.entry(blob).or_default().insert(device);
        }
    }

    Ok(SyncProjection {
        graph,
        access_records,
        scenes,
        handler_preferences: handlers,
        blob_availability,
        available_blobs,
        readers: fold_readers(&records, &causal.order),
        unreadable,
        key_agreement,
        pending: causal.pending,
        conflicts,
        writers,
    })
}

/// How a host folds personal-graph events into its graph (engine split D259).
/// pandect keeps the protocol and its bookkeeping: access records, saved
/// scenes, handler preferences and blob availability are collected here, and
/// key agreement and readers fold into nothing.
pub trait PersonalFold<G: SessionGraph> {
    /// Fold one graph event written by `asserter_iri`; `fallback_id` names a
    /// statement the event leaves unnamed. A saved scene arrives here too,
    /// for the graph to hold.
    fn apply(
        &self,
        graph: &mut G,
        asserter_iri: &str,
        fallback_id: &str,
        event: &PersonalGraphEvent,
    ) -> Result<(), PersonalGraphError>;

    /// Hold `history` as node `node`'s access history, once every event has
    /// folded.
    fn set_access_history(&self, graph: &mut G, node: Uuid, history: AccessHistory);
}

#[allow(clippy::too_many_arguments)]
fn apply_event<G: SessionGraph, F: PersonalFold<G> + ?Sized>(
    fold: &F,
    graph: &mut G,
    access: &mut BTreeMap<Uuid, AccessRecord>,
    scenes: &mut BTreeMap<Uuid, SavedSceneV2>,
    handlers: &mut BTreeMap<String, String>,
    blob_availability: &mut BTreeMap<Uuid, BlobAvailabilityObservation>,
    asserter_iri: &str,
    fallback_id: &str,
    event: &PersonalGraphEvent,
) -> Result<(), PersonalGraphError> {
    match event {
        PersonalGraphEvent::AppendAccess { record } => {
            access
                .entry(record.record_id)
                .or_insert_with(|| record.clone());
        },
        // A scene saved before version 4 (F158): it folds under its own
        // facet and converts when it opens (F142).
        PersonalGraphEvent::SaveScene { node, scene } => {
            scenes.insert(*node, scene.clone());
            fold.apply(graph, asserter_iri, fallback_id, event)?;
        },
        PersonalGraphEvent::SetHandlerPreference { key, handler } => {
            handlers.insert(key.clone(), handler.clone());
        },
        PersonalGraphEvent::ObserveBlobAvailability { observation } => {
            blob_availability
                .entry(observation.record_id)
                .or_insert_with(|| observation.clone());
        },
        // Key agreement is not graph content and folds into nothing. It is
        // collected separately, because acting on it changes this device's
        // keys rather than the graph everyone shares.
        PersonalGraphEvent::PublishPrekey { .. }
        | PersonalGraphEvent::GroupDispatch { .. }
        | PersonalGraphEvent::AdmitReader { .. }
        | PersonalGraphEvent::RetireReader { .. } => {},
        _ => fold.apply(graph, asserter_iri, fallback_id, event)?,
    }
    Ok(())
}

/// Fold the reader set from the operations that named it.
///
/// Deterministic and total: every device that holds the same operations
/// computes the same set, which is the whole point. A revocation is not an
/// action one device performs on another; it is a fact all of them read.
///
/// The first admit bootstraps. Before anyone is a reader there is nobody who
/// could authorize one, so the earliest admit in causal order stands on its
/// author's own authority and seats that author too. After that, only a
/// current reader may admit or retire, and an event from anyone else is
/// ignored rather than refused: it was validly authored on the lane and is
/// simply not authoritative about this.
fn fold_readers(records: &[StoredRecord], order: &[usize]) -> BTreeMap<[u8; 32], String> {
    let mut readers: BTreeMap<[u8; 32], String> = BTreeMap::new();
    for &index in order {
        let stored = &records[index];
        let Ok(author) = stable_subject(&stored.operation, &stored.record) else {
            continue;
        };
        for event in &stored.record.events {
            match event {
                PersonalGraphEvent::AdmitReader { root, label } => {
                    if readers.is_empty() {
                        readers.insert(author, String::new());
                    }
                    if readers.contains_key(&author) {
                        readers.insert(*root, label.clone());
                    }
                },
                PersonalGraphEvent::RetireReader { root } => {
                    // A reader may retire itself; that is leaving, not a
                    // coup. What it may not do is retire the set out of
                    // existence, because nobody could then admit anyone.
                    if readers.contains_key(&author) && readers.len() > 1 {
                        readers.remove(root);
                    }
                },
                _ => {},
            }
        }
    }
    readers
}

fn collect_conflicts(
    records: &[StoredRecord],
    entries: &[CausalEntry<u64>],
    order: &[usize],
    selection: &SyncSelection,
) -> Vec<SyncConflict> {
    let effective: BTreeSet<_> = order.iter().copied().collect();
    let mut conflicts = BTreeMap::<String, BTreeSet<[u8; 32]>>::new();
    // Indexed once rather than per pair: this scan asks two reachability
    // questions for every pair of records, and rebuilding the hash index inside
    // each made the pair test alone O(R^3 log R) in stored operations.
    let causal = CausalIndex::new(entries);
    // `event_target` formats a fresh String, so it is computed once per event
    // here instead of three times per event pair inside the loops below.
    let targets: Vec<Vec<Option<String>>> = records
        .iter()
        .map(|record| {
            record
                .record
                .events
                .iter()
                .map(|event| selection.projects(event).then(|| event_target(event)))
                .collect()
        })
        .collect();
    for left in 0..records.len() {
        if !effective.contains(&left) {
            continue;
        }
        for right in (left + 1)..records.len() {
            if !effective.contains(&right)
                || causal.happens_before(entries[left].operation, entries[right].operation)
                || causal.happens_before(entries[right].operation, entries[left].operation)
            {
                continue;
            }
            for (left_index, left_event) in records[left].record.events.iter().enumerate() {
                let Some(left_target) = targets[left][left_index].as_ref() else {
                    continue;
                };
                for (right_index, right_event) in records[right].record.events.iter().enumerate() {
                    let Some(right_target) = targets[right][right_index].as_ref() else {
                        continue;
                    };
                    if left_event == right_event || left_target != right_target {
                        continue;
                    }
                    conflicts.entry(left_target.clone()).or_default().extend([
                        *records[left].operation.hash.as_bytes(),
                        *records[right].operation.hash.as_bytes(),
                    ]);
                }
            }
        }
    }
    conflicts
        .into_iter()
        .map(|(target, operations)| SyncConflict {
            target,
            operations: operations.into_iter().collect(),
        })
        .collect()
}

fn event_target(event: &PersonalGraphEvent) -> String {
    match event {
        PersonalGraphEvent::AddNode { id, .. } | PersonalGraphEvent::RemoveNode { id } => {
            format!("node/{id}")
        },
        PersonalGraphEvent::SetTitle { node, .. } => format!("node/{node}/title"),
        PersonalGraphEvent::AddTag { node, tag } | PersonalGraphEvent::RemoveTag { node, tag } => {
            format!("node/{node}/tag/{tag}")
        },
        PersonalGraphEvent::AssertRelation {
            from, to, relation, ..
        } => format!("relation/{from}/{to}/{}", relation.predicate),
        PersonalGraphEvent::RetractRelation { from, to, selector } => {
            format!("relation/{from}/{to}/{selector:?}")
        },
        PersonalGraphEvent::RetractAssertion {
            from,
            to,
            statement_id,
            ..
        } => {
            format!("assertion/{from}/{to}/{statement_id}")
        },
        PersonalGraphEvent::SetFacet { node, facet, .. }
        | PersonalGraphEvent::RemoveFacet { node, facet } => {
            format!("node/{node}/facet/{facet}")
        },
        PersonalGraphEvent::AppendAccess { record } => {
            format!("access/{}", record.record_id)
        },
        PersonalGraphEvent::SaveScene { node, .. } => format!("scene/{node}"),
        PersonalGraphEvent::SetHandlerPreference { key, .. } => format!("handler/{key}"),
        PersonalGraphEvent::ObserveBlobAvailability { observation } => {
            format!("blob-availability/{}", observation.record_id)
        },
        // Concurrent key steps are not a conflict to report: the group scheme
        // orders them itself, and two devices publishing at once is ordinary.
        PersonalGraphEvent::PublishPrekey { bundle } => {
            format!("group-prekey/{}", Hash::digest(bundle))
        },
        PersonalGraphEvent::GroupDispatch { dispatch } => {
            format!("group-dispatch/{}", Hash::digest(dispatch))
        },
        // Concurrent edits to one root's readership are a genuine conflict
        // worth surfacing: two devices disagreeing about who may read is not
        // something to resolve quietly.
        PersonalGraphEvent::AdmitReader { root, .. }
        | PersonalGraphEvent::RetireReader { root } => {
            format!("reader/{}", hex32(root))
        },
    }
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Whether `selector` names the semantic family or a core semantic predicate:
/// such retractions must name an assertion id instead.
fn names_semantic(selector: &RelationSelector) -> bool {
    match selector {
        RelationSelector::Family(family) => family == core_pack::iri(ids::FAMILY_SEMANTIC),
        RelationSelector::Predicate(predicate) => {
            core_pack::terms().family(predicate) == Some(ids::FAMILY_SEMANTIC)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_surface::{StatementFold, Surface};
    use identity::{IdentityProvider, InMemoryProvider};
    use mere::model::relation::legacy;
    use mere::model::{GraphScope, SemanticStatement};
    use mere::profile::intern::TermId;
    use muniment::MemoryBackend;
    use std::sync::Arc;

    const GRAPH: [u8; 32] = [0x77; 32];
    const A: Uuid = Uuid::from_u128(0xa1);
    const B: Uuid = Uuid::from_u128(0xb2);

    fn relation() -> RelationClaim {
        RelationClaim::new(core_pack::iri(ids::REL_CITES), None)
    }

    fn tile_group() -> String {
        legacy::predicate_iri("Arrangement", "TileGroup")
    }

    async fn assertion_fixture() -> (PersonalGraphReplica<MemoryBackend>, String, String) {
        let alice = InMemoryProvider::from_seed([0xa1; 32]);
        let bob = InMemoryProvider::from_seed([0xb2; 32]);
        let roots = [
            alice.master_public_key().to_bytes(),
            bob.master_public_key().to_bytes(),
        ];
        let roster = SyncRoster::new(roots);
        let mut replica = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            GRAPH,
            &alice,
            roster.clone(),
            selection(),
        )
        .unwrap();
        let mut peer = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            GRAPH,
            &bob,
            roster,
            selection(),
        )
        .unwrap();
        let seed = replica
            .author(vec![
                PersonalGraphEvent::AddNode {
                    id: A,
                    address: "https://a.test/".into(),
                    title: "A".into(),
                },
                PersonalGraphEvent::AddNode {
                    id: B,
                    address: "https://b.test/".into(),
                    title: "B".into(),
                },
            ])
            .await
            .unwrap();
        peer.accept(&seed).await.unwrap();
        replica
            .author(vec![PersonalGraphEvent::AssertRelation {
                from: A,
                to: B,
                relation: relation(),
                assertion_version: None,
                statement_id: None,
                asserted_at_ms: None,
                basis: None,
            }])
            .await
            .unwrap();
        let assertion = peer
            .author(vec![PersonalGraphEvent::AssertRelation {
                from: A,
                to: B,
                relation: relation(),
                assertion_version: None,
                statement_id: None,
                asserted_at_ms: None,
                basis: None,
            }])
            .await
            .unwrap();
        replica.accept(&assertion).await.unwrap();
        (
            replica,
            Author::person(hex32(&roots[0])).asserter_iri(),
            Author::person(hex32(&roots[1])).asserter_iri(),
        )
    }

    /// The held statements between A and B, in the surface's fold.
    fn assertion_statements(projection: &SyncProjection<Surface>) -> Vec<SemanticStatement> {
        projection.graph.held_statements(A, B)
    }

    #[tokio::test]
    async fn p6_sync_authors_and_replays_exact_ids_for_every_relation_family() {
        let (mut replica, alice, _) = assertion_fixture().await;
        let baseline =
            assertion_statements(&replica.projection_with(&StatementFold).await.unwrap())
                .into_iter()
                .map(|statement| (statement.statement_id.clone(), statement))
                .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            baseline.len(),
            2,
            "the fixture begins with two signed citations"
        );
        assert!(
            baseline
                .values()
                .all(|statement| { statement.predicate == core_pack::iri(ids::REL_CITES) })
        );
        let assertions = [
            RelationClaim::new(
                core_pack::iri(ids::REL_SUPPORTS),
                Some("supports the source passage".into()),
            ),
            RelationClaim::new(core_pack::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER), None),
            RelationClaim::new(core_pack::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER), None),
            RelationClaim::new(tile_group(), None),
            RelationClaim::new(core_pack::iri(ids::REL_IMPORTED_BOOKMARK_FOLDER), None),
            RelationClaim::new(core_pack::iri(ids::REL_PROVENANCE_CLIPPED_FROM), None),
        ];
        let operation = replica
            .author(
                assertions
                    .iter()
                    .cloned()
                    .map(|relation| PersonalGraphEvent::AssertRelation {
                        from: A,
                        to: B,
                        relation,
                        assertion_version: None,
                        statement_id: None,
                        asserted_at_ms: None,
                        basis: Some(AssertionBasis {
                            note: Some("signed sync fixture".into()),
                            evidence: vec!["urn:evidence:fixture".into()],
                            ..Default::default()
                        }),
                    })
                    .collect(),
            )
            .await
            .unwrap();
        let authored = from_operation(&operation, None).unwrap();
        assert!(authored.events.iter().all(|event| matches!(
            event,
            PersonalGraphEvent::AssertRelation {
                assertion_version: Some(1),
                statement_id: Some(_),
                asserted_at_ms: Some(_),
                ..
            }
        )));
        let authored_ids = authored
            .events
            .iter()
            .filter_map(|event| match event {
                PersonalGraphEvent::AssertRelation {
                    assertion_version: Some(1),
                    statement_id: Some(id),
                    ..
                } => Some(id.clone()),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(authored_ids.len(), assertions.len());
        let authored_records = |projection: &SyncProjection<Surface>| {
            assertion_statements(projection)
                .into_iter()
                .filter(|statement| authored_ids.contains(&statement.statement_id))
                .collect::<Vec<_>>()
        };
        let baseline_records = |projection: &SyncProjection<Surface>| {
            assertion_statements(projection)
                .into_iter()
                .filter(|statement| !authored_ids.contains(&statement.statement_id))
                .map(|statement| (statement.statement_id.clone(), statement))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let initial_projection = replica.projection_with(&StatementFold).await.unwrap();
        let held = assertion_statements(&initial_projection);
        let authored_held = authored_records(&initial_projection);
        assert_eq!(held.len(), assertions.len() + 2);
        assert_eq!(authored_held.len(), assertions.len());
        assert_eq!(
            baseline_records(&initial_projection),
            baseline,
            "adding all-family assertions preserves both existing signed citations exactly"
        );
        let ids: Vec<_> = authored_held
            .iter()
            .map(|statement| statement.statement_id.clone())
            .collect();
        assert_eq!(
            ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
            assertions.len()
        );
        for statement in &authored_held {
            assert_eq!(statement.provenance_iri.as_deref(), Some(alice.as_str()));
            assert_eq!(statement.asserted_at_ms, statement.first_asserted_ms);
            assert!(statement.asserted_at_ms.is_some());
            assert_eq!(statement.graph_scope, GraphScope::Default);
            assert_eq!(statement.basis.note.as_deref(), Some("signed sync fixture"));
            assert_eq!(
                statement.basis.evidence,
                vec!["urn:evidence:fixture".to_owned()]
            );
            assert!(statement.withdrawn.is_none());
        }

        let collection_ids: Vec<_> = authored_held
            .iter()
            .filter(|statement| {
                statement.predicate == core_pack::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER)
            })
            .map(|statement| statement.statement_id.clone())
            .collect();
        assert_eq!(collection_ids.len(), 2);
        let collection_id = collection_ids[0].clone();
        let sibling_id = collection_ids[1].clone();
        replica
            .author(vec![PersonalGraphEvent::RetractAssertion {
                from: A,
                to: B,
                statement_id: collection_id.clone(),
                withdrawn_at_ms: Some(777),
            }])
            .await
            .unwrap();
        let history = replica.projection_with(&StatementFold).await.unwrap();
        let retained = history.graph.find_statement(&collection_id).unwrap();
        assert_eq!(retained.withdrawn.as_ref().unwrap().at_ms, Some(777));
        assert_eq!(
            retained.withdrawn.as_ref().unwrap().by.as_deref(),
            Some(alice.as_str())
        );
        assert_eq!(assertion_statements(&history).len(), assertions.len() + 1);
        assert_eq!(authored_records(&history).len(), assertions.len() - 1);
        assert_eq!(
            baseline_records(&history),
            baseline,
            "withdrawing one new assertion preserves both prior signed citations exactly"
        );
        assert!(history.graph.find_statement(&sibling_id).unwrap().is_held());
        for before in authored_held
            .iter()
            .filter(|claim| claim.statement_id != collection_id)
        {
            assert_eq!(
                history.graph.find_statement(&before.statement_id).as_ref(),
                Some(before),
                "withdrawing one exact reason preserves every other authored record"
            );
        }

        let receiver = PersonalGraphReplica::new(
            MemoryBackend::new(),
            GRAPH,
            replica.signing_seed,
            replica.roster.clone(),
            selection(),
        );
        let (records, _) = load_records(&replica.store, GRAPH, None).await.unwrap();
        let order = causal_projection(&causal_entries(&records)).unwrap().order;
        for index in order {
            receiver.accept(&records[index].operation).await.unwrap();
        }
        let replayed = receiver.projection_with(&StatementFold).await.unwrap();
        assert_eq!(assertion_statements(&replayed).len(), assertions.len() + 1);
        assert_eq!(authored_records(&replayed).len(), assertions.len() - 1);
        assert_eq!(baseline_records(&replayed), baseline);
        let replayed_tombstone = replayed.graph.find_statement(&collection_id).unwrap();
        assert_eq!(
            replayed_tombstone.withdrawn.as_ref().unwrap().at_ms,
            Some(777)
        );
        assert_eq!(
            replayed_tombstone.withdrawn.as_ref().unwrap().by.as_deref(),
            Some(alice.as_str())
        );
        assert_eq!(
            replayed.graph.find_statement(&sibling_id).unwrap(),
            history.graph.find_statement(&sibling_id).unwrap()
        );
    }

    #[tokio::test]
    async fn p6_legacy_selector_retracts_only_source_authored_nonsemantic_records() {
        let (mut replica, alice, bob) = assertion_fixture().await;
        let alice_identity = InMemoryProvider::from_seed([0xa1; 32]);
        let bob_identity = InMemoryProvider::from_seed([0xb2; 32]);
        let roster = SyncRoster::new([
            alice_identity.master_public_key().to_bytes(),
            bob_identity.master_public_key().to_bytes(),
        ]);
        let create = |id: &str| PersonalGraphEvent::AssertRelation {
            from: A,
            to: B,
            relation: RelationClaim::new(tile_group(), None),
            assertion_version: Some(1),
            statement_id: Some(id.into()),
            asserted_at_ms: Some(50),
            basis: Some(AssertionBasis {
                note: Some("tile grouping".into()),
                ..Default::default()
            }),
        };
        let alice_event = create("urn:assertion:alice-arrangement");
        replica.author(vec![alice_event]).await.unwrap();
        let mut bob_replica = PersonalGraphReplica::for_identity(
            MemoryBackend::new(),
            GRAPH,
            &bob_identity,
            roster,
            selection(),
        )
        .unwrap();
        let (records, _) = load_records(&replica.store, GRAPH, None).await.unwrap();
        for index in causal_projection(&causal_entries(&records)).unwrap().order {
            bob_replica.accept(&records[index].operation).await.unwrap();
        }
        let bob_event = PersonalGraphEvent::AssertRelation {
            from: A,
            to: B,
            relation: RelationClaim::new(tile_group(), None),
            assertion_version: Some(1),
            statement_id: Some("urn:assertion:bob-arrangement".into()),
            asserted_at_ms: Some(51),
            basis: Some(AssertionBasis {
                note: Some("tile grouping".into()),
                ..Default::default()
            }),
        };
        let bob_operation = bob_replica.author(vec![bob_event]).await.unwrap();
        replica.accept(&bob_operation).await.unwrap();
        assert!(
            replica
                .author(vec![PersonalGraphEvent::RetractRelation {
                    from: A,
                    to: B,
                    selector: RelationSelector::predicate(tile_group()),
                }])
                .await
                .is_ok()
        );
        let projection = replica.projection_with(&StatementFold).await.unwrap();
        let graph = &projection.graph;
        let alice_statement = graph
            .find_statement("urn:assertion:alice-arrangement")
            .unwrap();
        let bob_statement = graph
            .find_statement("urn:assertion:bob-arrangement")
            .unwrap();
        assert_eq!(
            alice_statement.provenance_iri.as_deref(),
            Some(alice.as_str())
        );
        assert!(alice_statement.withdrawn.is_some());
        assert_eq!(bob_statement.provenance_iri.as_deref(), Some(bob.as_str()));
        assert!(bob_statement.withdrawn.is_none());
    }

    #[tokio::test]
    async fn p1_sync_retraction_preserves_the_other_asserter() {
        let (replica, alice, bob) = assertion_fixture().await;
        let before = assertion_statements(&replica.projection_with(&StatementFold).await.unwrap());
        let alice_id = before
            .iter()
            .find(|s| s.provenance_iri.as_deref() == Some(&alice))
            .unwrap()
            .statement_id
            .clone();
        assert_eq!(
            before.len(),
            2,
            "positive control: both sources asserted the relation"
        );
        assert!(
            before
                .iter()
                .any(|s| s.provenance_iri.as_deref() == Some(&alice))
        );
        assert!(
            before
                .iter()
                .any(|s| s.provenance_iri.as_deref() == Some(&bob))
        );
        legacy_signed(
            &replica,
            vec![PersonalGraphEvent::RetractRelation {
                from: A,
                to: B,
                selector: RelationSelector::predicate(core_pack::iri(ids::REL_CITES)),
            }],
        )
        .await;
        let after = assertion_statements(&replica.projection_with(&StatementFold).await.unwrap());
        assert!(
            !after
                .iter()
                .any(|s| s.provenance_iri.as_deref() == Some(&alice)),
            "positive control: the writer's assertion was retracted"
        );
        assert_eq!(after.len(), 1, "another asserter's assertion must remain");
        assert_eq!(after[0].provenance_iri.as_deref(), Some(bob.as_str()));
        let withdrawn = replica
            .projection_with(&StatementFold)
            .await
            .unwrap()
            .graph
            .find_statement(&alice_id)
            .expect("legacy sync retraction retains exact history");
        assert_eq!(
            withdrawn.withdrawn.as_ref().unwrap().at_ms,
            None,
            "legacy retraction events carry no recorded time"
        );
    }

    async fn legacy_signed(
        replica: &PersonalGraphReplica<MemoryBackend>,
        events: Vec<PersonalGraphEvent>,
    ) -> Operation<PersonalGraphExt> {
        let (records, _) = load_records(&replica.store, GRAPH, replica.keyring.as_ref())
            .await
            .unwrap();
        let entries = causal_entries(&records);
        let signer = SigningKey::from_bytes(&replica.signing_seed);
        let (seq_num, backlink) = author_head(
            &entries,
            *signer.verifying_key().as_bytes(),
            &PERSONAL_GRAPH_LOG,
        )
        .unwrap();
        let operation = to_operation(
            replica.signing_seed,
            GRAPH,
            &PersonalGraphRecord {
                events,
                parents: observed_frontier(&entries).unwrap(),
                writer_attestation: replica.writer_attestation.clone(),
            },
            seq_num,
            backlink,
            replica.keyring.as_deref(),
        )
        .unwrap();
        replica.accept(&operation).await.unwrap();
        operation
    }

    fn statement_event(kind: TermId, id: &str, time: u64) -> PersonalGraphEvent {
        PersonalGraphEvent::AssertRelation {
            from: A,
            to: B,
            relation: RelationClaim::new(core_pack::iri(kind), None),
            assertion_version: None,
            statement_id: Some(id.into()),
            asserted_at_ms: Some(time),
            basis: None,
        }
    }

    #[tokio::test]
    async fn p1_sync_caller_ids_times_and_exact_retractions_survive_replication() {
        let (mut replica, _, _) = assertion_fixture().await;
        let id = "urn:co-op:assertion:one";
        let operation = replica
            .author(vec![statement_event(ids::REL_SUPPORTS, id, 100)])
            .await
            .unwrap();
        let record = from_operation(&operation, None).unwrap();
        let mut newly_authored = statement_event(ids::REL_SUPPORTS, id, 100);
        if let PersonalGraphEvent::AssertRelation {
            assertion_version, ..
        } = &mut newly_authored
        {
            *assertion_version = Some(1);
        }
        assert_eq!(record.events, vec![newly_authored]);
        replica
            .author(vec![statement_event(
                ids::REL_SUPPORTS,
                "urn:co-op:unused",
                200,
            )])
            .await
            .unwrap();
        let before = assertion_statements(&replica.projection_with(&StatementFold).await.unwrap());
        assert_eq!(before.len(), 4);
        assert_eq!(
            before
                .iter()
                .find(|s| s.statement_id == id)
                .unwrap()
                .asserted_at_ms,
            Some(100),
            "an explicit signed ID is not merged with another ID for the same reason"
        );
        assert_eq!(
            before
                .iter()
                .find(|s| s.statement_id == "urn:co-op:unused")
                .unwrap()
                .asserted_at_ms,
            Some(200)
        );

        let receiver = PersonalGraphReplica::new(
            MemoryBackend::new(),
            GRAPH,
            replica.signing_seed,
            replica.roster.clone(),
            selection(),
        );
        let (records, _) = load_records(&replica.store, GRAPH, None).await.unwrap();
        let order = causal_projection(&causal_entries(&records)).unwrap().order;
        for index in order {
            receiver.accept(&records[index].operation).await.unwrap();
        }
        assert_eq!(
            assertion_statements(&receiver.projection_with(&StatementFold).await.unwrap()),
            before
        );
        assert!(
            replica
                .author(vec![PersonalGraphEvent::RetractAssertion {
                    from: A,
                    to: B,
                    statement_id: "urn:co-op:missing".into(),
                    withdrawn_at_ms: None,
                }])
                .await
                .is_err(),
            "unknown-ID withdrawal is rejected before signing or causal admission"
        );
        assert_eq!(
            assertion_statements(&replica.projection_with(&StatementFold).await.unwrap()),
            before
        );
        let retract = replica
            .author(vec![PersonalGraphEvent::RetractAssertion {
                from: A,
                to: B,
                statement_id: id.into(),
                withdrawn_at_ms: None,
            }])
            .await
            .unwrap();
        receiver.accept(&retract).await.unwrap();
        let after = assertion_statements(&receiver.projection_with(&StatementFold).await.unwrap());
        assert_eq!(after.len(), 3, "the other exact IDs remain held");
        assert!(!after.iter().any(|s| s.statement_id == id));
        assert!(after.iter().any(|s| s.statement_id == "urn:co-op:unused"));
        let withdrawn = receiver
            .projection_with(&StatementFold)
            .await
            .unwrap()
            .graph
            .find_statement(id)
            .expect("exact withdrawal remains in retained history");
        assert!(withdrawn.withdrawn.as_ref().unwrap().at_ms.is_some());
        assert_eq!(
            after,
            before
                .into_iter()
                .filter(|s| s.statement_id != id)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn p1_sync_legacy_bytes_and_fallback_handles_are_stable() {
        // An event signed before L7 named its relation by family and
        // sub-kind; it reads as the IRI it meant, and new events write the IRI.
        #[derive(Serialize)]
        enum OldAssertion {
            Semantic {
                sub_kind: &'static str,
                label: Option<String>,
            },
        }
        #[derive(Serialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        enum OldEvent {
            AssertRelation {
                from: Uuid,
                to: Uuid,
                assertion: OldAssertion,
            },
        }
        let old = OldEvent::AssertRelation {
            from: A,
            to: B,
            assertion: OldAssertion::Semantic {
                sub_kind: "Cites",
                label: None,
            },
        };
        let legacy = PersonalGraphEvent::AssertRelation {
            from: A,
            to: B,
            relation: relation(),
            assertion_version: None,
            statement_id: None,
            asserted_at_ms: None,
            basis: None,
        };
        let read: PersonalGraphEvent = decode_cbor_strict(&encode_cbor(&old).unwrap()[..]).unwrap();
        assert_eq!(read, legacy);
        let written = encode_cbor(&legacy).unwrap();
        assert_ne!(
            written,
            encode_cbor(&old).unwrap(),
            "new writes name the IRI"
        );
        assert_eq!(
            decode_cbor_strict::<PersonalGraphEvent, _>(&written[..]).unwrap(),
            legacy
        );
        let (replica, _, _) = assertion_fixture().await;
        let operation = legacy_signed(
            &replica,
            vec![PersonalGraphEvent::AssertRelation {
                relation: RelationClaim::new(core_pack::iri(ids::REL_SUPPORTS), None),
                from: A,
                to: B,
                assertion_version: None,
                statement_id: None,
                asserted_at_ms: None,
                basis: None,
            }],
        )
        .await;
        let first = assertion_statements(&replica.projection_with(&StatementFold).await.unwrap());
        let expected = format!(
            "urn:mere:statement:personal:{}:0",
            hex32(operation.hash.as_bytes())
        );
        let legacy = first.iter().find(|s| s.statement_id == expected).unwrap();
        assert_eq!(legacy.asserted_at_ms, None, "old logs recorded no time");
        assert_eq!(
            first.iter().filter(|s| s.asserted_at_ms.is_some()).count(),
            2,
            "positive control: both new signed assertions carry times"
        );
        assert_eq!(
            assertion_statements(&replica.projection_with(&StatementFold).await.unwrap()),
            first
        );
        assert_eq!(operation.header.hash(), operation.hash);
        assert_eq!(
            encode_cbor(&from_operation(&operation, None).unwrap()).unwrap(),
            operation.body.as_ref().unwrap().to_bytes()
        );
    }

    #[tokio::test]
    async fn p1_sync_legacy_family_retraction_withdraws_only_its_root() {
        let (mut replica, alice, bob) = assertion_fixture().await;
        replica
            .author(vec![statement_event(
                ids::REL_SUPPORTS,
                "urn:co-op:extra",
                100,
            )])
            .await
            .unwrap();
        assert_eq!(
            assertion_statements(&replica.projection_with(&StatementFold).await.unwrap()).len(),
            3
        );
        let event = PersonalGraphEvent::RetractRelation {
            from: A,
            to: B,
            selector: RelationSelector::family(ids::FAMILY_SEMANTIC),
        };
        assert!(
            replica.author(vec![event.clone()]).await.is_err(),
            "new withdrawals must name an id"
        );
        legacy_signed(&replica, vec![event]).await;
        let remaining =
            assertion_statements(&replica.projection_with(&StatementFold).await.unwrap());
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].provenance_iri.as_deref(), Some(bob.as_str()));
        assert!(
            remaining
                .iter()
                .all(|s| s.provenance_iri.as_deref() != Some(alice.as_str()))
        );
    }

    fn selection() -> SyncSelection {
        SyncSelection::default()
            .with_facets(["graphshell.test/v1"])
            .with_access_records(true)
            .with_saved_scenes(true)
            .with_handler_preferences(true)
            .with_blob_availability(true)
    }

    fn keyring() -> Arc<DataKeyring> {
        let mut keyring = DataKeyring::new();
        keyring.rotate_random().expect("a fresh secret");
        Arc::new(keyring)
    }

    /// The property the whole migration rests on.
    ///
    /// `Header::to_bytes` re-encodes extensions, and both `hash()` and
    /// signature verification digest that encoding. If adding `encryption`
    /// changed how a plaintext extension encodes, every operation already
    /// signed and stored would stop verifying. `skip_serializing_if` is what
    /// prevents that, and this is the assertion that keeps it.
    #[test]
    fn the_header_fields_added_since_do_not_change_an_old_header() {
        #[derive(Serialize)]
        struct ExtensionAsItWasBeforeEncryption {
            graph: [u8; 32],
        }

        let before = encode_cbor(&ExtensionAsItWasBeforeEncryption { graph: GRAPH }).unwrap();
        let after = encode_cbor(&PersonalGraphExt {
            graph: GRAPH,
            encryption: PersonalEncryption::Plaintext,
            parents: Vec::new(),
            writer_attestation: None,
        })
        .unwrap();
        assert_eq!(
            before, after,
            "a plaintext extension must encode exactly as it did before the field existed"
        );

        // And the field is really carried when it is not plaintext, so the
        // omission above is not simply the field never being written.
        let sealed = encode_cbor(&PersonalGraphExt {
            graph: GRAPH,
            encryption: PersonalEncryption::GroupV1,
            parents: Vec::new(),
            writer_attestation: None,
        })
        .unwrap();
        assert_ne!(before, sealed);

        // Same for the two fields that moved out of the sealed body. Each is
        // absent on an operation written before it existed, and each changes
        // the bytes when present, so neither can quietly rewrite a signed
        // header nor quietly fail to travel.
        let with_parents = encode_cbor(&PersonalGraphExt {
            graph: GRAPH,
            encryption: PersonalEncryption::Plaintext,
            parents: vec![[0x01; 32]],
            writer_attestation: None,
        })
        .unwrap();
        assert_ne!(before, with_parents);
    }

    /// A different keyring is not a wrong password prompt: it is an operation
    /// this device cannot read, and it must be reported as such.
    #[tokio::test]
    async fn a_foreign_keyring_cannot_open_another_graphs_operations() {
        let seed = [0x73; 32];
        let subject = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let roster = SyncRoster::new([subject]);
        let mut sealing =
            PersonalGraphReplica::new(MemoryBackend::new(), GRAPH, seed, roster, selection());
        sealing.set_keyring(keyring());
        let sealed = sealing
            .author(vec![PersonalGraphEvent::AddNode {
                id: A,
                address: "https://sealed.test/".into(),
                title: "Sealed".into(),
            }])
            .await
            .unwrap();

        let stranger = keyring();
        let read = from_operation(&sealed, Some(stranger.as_ref()));
        assert!(
            matches!(read, Err(PersonalGraphWireError::Unsealable(_))),
            "expected an unsealable error, got {read:?}"
        );
    }

    /// The property the whole distribution design rests on: a device with no
    /// key must still be able to read the traffic that keys it.
    #[tokio::test]
    async fn key_agreement_stays_readable_when_everything_else_is_sealed() {
        let seed = [0x74; 32];
        let subject = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let roster = SyncRoster::new([subject]);
        let backend = MemoryBackend::new();
        let mut replica =
            PersonalGraphReplica::new(backend.clone(), GRAPH, seed, roster, selection());
        replica.set_keyring(keyring());

        // Content authored under a key, and a pre-key published beside it.
        replica
            .author(vec![PersonalGraphEvent::AddNode {
                id: A,
                address: "https://sealed.test/".into(),
                title: "Sealed".into(),
            }])
            .await
            .unwrap();
        let (_, bundle) = stickleback::GroupSession::new(
            stickleback::GroupSessionId(GRAPH),
            &identity::InMemoryProvider::from_seed([0x75; 32]),
        )
        .unwrap();
        let published = replica
            .author(vec![PersonalGraphEvent::PublishPrekey {
                bundle: bundle.to_bytes().unwrap(),
            }])
            .await
            .unwrap();

        assert_eq!(
            published.header.extensions.encryption,
            PersonalEncryption::Plaintext,
            "key agreement must not be sealed even by a device that holds a key"
        );

        // The reader a new device is: no key at all.
        let steps = key_agreement(&replica.sync_store(), GRAPH).await.unwrap();
        assert_eq!(
            steps.len(),
            1,
            "the sealed content is skipped, the pre-key is not"
        );
        assert!(matches!(steps[0].step, KeyAgreementEvent::Prekey(_)));
        assert_eq!(
            steps[0].author_root, subject,
            "the lane authenticates the author"
        );
    }

    /// An operation may carry key agreement or graph content, never both.
    /// Mixing would publish content in the clear alongside the key traffic.
    #[tokio::test]
    async fn an_operation_carrying_keys_may_not_also_carry_content() {
        let seed = [0x76; 32];
        let subject = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let mut replica = PersonalGraphReplica::new(
            MemoryBackend::new(),
            GRAPH,
            seed,
            SyncRoster::new([subject]),
            selection(),
        );
        let (_, bundle) = stickleback::GroupSession::new(
            stickleback::GroupSessionId(GRAPH),
            &identity::InMemoryProvider::from_seed([0x77; 32]),
        )
        .unwrap();
        let mixed = replica
            .author(vec![
                PersonalGraphEvent::PublishPrekey {
                    bundle: bundle.to_bytes().unwrap(),
                },
                PersonalGraphEvent::AddNode {
                    id: A,
                    address: "https://would-be-public.test/".into(),
                    title: "Content beside a key".into(),
                },
            ])
            .await;
        assert!(
            mixed.is_err(),
            "mixing must be refused, not merely discouraged"
        );
    }

    /// A forged pre-key is refused at intake, not when somebody tries to use it.
    #[tokio::test]
    async fn a_prekey_that_does_not_authenticate_is_refused() {
        let seed = [0x78; 32];
        let subject = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let mut replica = PersonalGraphReplica::new(
            MemoryBackend::new(),
            GRAPH,
            seed,
            SyncRoster::new([subject]),
            selection(),
        );
        let refused = replica
            .author(vec![PersonalGraphEvent::PublishPrekey {
                bundle: b"not a bundle".to_vec(),
            }])
            .await;
        assert!(refused.is_err());
    }

    /// F158: a version-4 scene travels as an ordinary facet, so a record
    /// carrying one decodes on any device and folds into the facet store
    /// opaque; the facets the typed scene event writes stay reserved to it,
    /// and an older scene event folds under its own facet, to convert when
    /// it opens.
    #[test]
    fn a_version_4_scene_travels_as_an_opaque_facet() {
        use crate::SAVED_SCENE_FACET;
        assert!(validate_facet_name(SAVED_SCENE_FACET).is_ok());
        for reserved in [SAVED_SCENE_FACET_V3, SAVED_SCENE_FACET_V2] {
            assert_eq!(
                validate_facet_name(reserved).unwrap_err().code,
                "reserved-personal-graph-facet"
            );
        }
        let scene = serde_json::json!({ "name": "v4", "dynamics": { "version": 9 } });
        let record = PersonalGraphRecord {
            events: vec![PersonalGraphEvent::SetFacet {
                node: A,
                facet: SAVED_SCENE_FACET.into(),
                value: scene.clone(),
            }],
            parents: Vec::new(),
            writer_attestation: None,
        };
        let bytes = encode_cbor(&record).expect("encodes");
        let back: PersonalGraphRecord =
            decode_cbor_strict(bytes.as_slice()).expect("a spec this reader refuses still decodes");
        let PersonalGraphEvent::SetFacet { value, .. } = &back.events[0] else {
            panic!("the event comes back as it went");
        };
        assert_eq!(value, &scene);
        assert_eq!(encode_cbor(&back).unwrap(), bytes, "byte for byte");
    }
}
