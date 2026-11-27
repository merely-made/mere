// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit input placement; a missing profile remains unqualified.

use crate::NodeFacetStore;
use mere::model::Attributed;
use mere::model::legacy::{LegacyContentContext, LegacyResourceOriginResolution};
use mere::session::{SessionGraph, SessionJournal};
use muniment::{Journal, Seq};
use serde::{Deserialize, Serialize};

/// The writer's placement contract, independent of the claims currently held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementProfile {
    LegacySurfaceV1,
    RecordedStrataV1,
}

/// Materialize only the placement carried by the input boundary, over any
/// graph (engine split D257). Callers name the graph type: it is not
/// inferred from its snapshot.
pub fn materialize_snapshot_with<G: SessionGraph>(
    snapshot: &G::Snapshot,
    placement: Option<PlacementProfile>,
) -> Result<G, String> {
    match placement {
        Some(PlacementProfile::RecordedStrataV1) => G::from_snapshot(snapshot, true),
        None => G::from_snapshot(snapshot, false),
        Some(PlacementProfile::LegacySurfaceV1) => {
            Err("legacy graph placement requires qualified replay".into())
        },
    }
}

/// The retained baseline and the permanently qualified legacy journal prefix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPlacement {
    pub baseline: PlacementProfile,
    /// `None` is the empty prefix, never the session's future journal.
    pub legacy_journal_until: Option<Seq>,
    #[serde(default)]
    pub origins: Vec<LegacyResourceOriginResolution>,
}

impl SessionPlacement {
    pub fn recorded() -> Self {
        Self {
            baseline: PlacementProfile::RecordedStrataV1,
            legacy_journal_until: None,
            origins: Vec::new(),
        }
    }

    pub fn legacy(until: Seq, origins: Vec<LegacyResourceOriginResolution>) -> Self {
        Self {
            baseline: PlacementProfile::LegacySurfaceV1,
            legacy_journal_until: Some(until),
            origins,
        }
    }

    pub(crate) fn cutoff(&self) -> Seq {
        self.legacy_journal_until.unwrap_or(Seq(0))
    }
}

/// How a host's graph reads a stored legacy translation (engine split D252,
/// D260). The read path is pandect's, over [`SessionGraph`]; the host says
/// which assertion schema a stored snapshot was written with. A host installs
/// its hook on [`MereSessions`](crate::MereSessions).
pub struct LegacyHook<G: SessionGraph> {
    /// `0` for the legacy sidecar shape, which compares as it materializes
    /// now; `1` for the current shape, compared as stored; any other refuses.
    pub assertion_schema: fn(&G::Snapshot) -> u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound = "")]
pub(crate) struct FrozenGraph<G: SessionGraph> {
    pub graph: G::Snapshot,
    pub facets: NodeFacetStore,
}

impl<G: SessionGraph> std::fmt::Debug for FrozenGraph<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrozenGraph")
            .field("graph", &self.graph)
            .field("facets", &self.facets)
            .finish()
    }
}

fn comparison_graph_value<G: SessionGraph>(
    hook: &LegacyHook<G>,
    snapshot: &G::Snapshot,
) -> Option<serde_json::Value> {
    // Historical translation receipts keep their original legacy-sidecar JSON
    // and checksum. Only legacy-schema receipts are restored into the current
    // assertion shape for truth comparison; canonical snapshots retain their
    // exact persisted columns in the comparison.
    let mut value = match (hook.assertion_schema)(snapshot) {
        0 => serde_json::to_value(G::from_snapshot(snapshot, true).ok()?.to_snapshot()).ok()?,
        1 => serde_json::to_value(snapshot).ok()?,
        _ => return None,
    };
    // The envelope clock is not graph truth; its stored bytes remain covered
    // by the translation checksum.
    if let Some(fields) = value.as_object_mut() {
        fields.remove("timestamp_secs");
    }
    Some(value)
}

impl<G: SessionGraph> FrozenGraph<G> {
    pub fn of(graph: &G) -> Self {
        Self {
            graph: graph.to_snapshot(),
            facets: graph.facets().clone(),
        }
    }

    /// Whether the two hold the same graph truth, as `hook` reads their
    /// snapshots.
    pub fn same(&self, other: &Self, hook: &LegacyHook<G>) -> bool {
        self.facets == other.facets
            && matches!(
                (
                    comparison_graph_value(hook, &self.graph),
                    comparison_graph_value(hook, &other.graph)
                ),
                (Some(left), Some(right)) if left == right
            )
    }

    pub fn materialize(&self) -> Result<G, String> {
        let mut graph = G::from_snapshot(&self.graph, true)?;
        graph.install_facets(self.facets.clone())?;
        Ok(graph)
    }
}

/// An exact translation, separate from its retained original authority.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound = "")]
pub(crate) struct LegacyTranslationReceipt<G: SessionGraph> {
    version: u32,
    pub source_digest: String,
    pub translation_digest: String,
    pub placement: SessionPlacement,
    pub baseline: FrozenGraph<G>,
    pub entry_effects: Vec<Vec<G::Delta>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_context: Option<LegacyContentContext>,
    legacy_final: FrozenGraph<G>,
    /// The checksum of the payload as it was stored, when read from bytes.
    #[serde(skip)]
    stored_digest: Option<String>,
}

impl<G: SessionGraph> std::fmt::Debug for LegacyTranslationReceipt<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegacyTranslationReceipt")
            .field("version", &self.version)
            .field("source_digest", &self.source_digest)
            .field("translation_digest", &self.translation_digest)
            .field("placement", &self.placement)
            .field("baseline", &self.baseline)
            .field("entry_effects", &self.entry_effects)
            .field("content_context", &self.content_context)
            .field("legacy_final", &self.legacy_final)
            .finish()
    }
}

impl<G: SessionGraph> LegacyTranslationReceipt<G> {
    /// Read a stored receipt, keeping the checksum of its payload as stored
    /// (declared-lens D31 (1)): today's types read a receipt written before
    /// L7 but re-encode it in another shape, so the checksum is taken over
    /// the stored field text, insignificant whitespace removed.
    pub fn from_stored(bytes: &[u8]) -> Result<Self, String> {
        let mut receipt: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        receipt.stored_digest = Some(stored_digest(bytes)?);
        Ok(receipt)
    }

    fn digest(&self) -> Result<String, String> {
        if let Some(stored) = &self.stored_digest {
            return Ok(stored.clone());
        }
        let payload = (
            self.version,
            &self.source_digest,
            &self.placement,
            &self.baseline,
            &self.entry_effects,
            &self.legacy_final,
        );
        let bytes = if let Some(context) = &self.content_context {
            serde_json::to_vec(&(payload, context))
        } else {
            // Preserve the checksum of receipts written before content context.
            serde_json::to_vec(&payload)
        }
        .map_err(|error| error.to_string())?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    /// Refuse a receipt that is not exactly the translation of `entries`
    /// under `placement`, reading its snapshots through `hook`.
    pub fn validate_with(
        &self,
        hook: &LegacyHook<G>,
        placement: &SessionPlacement,
        source_digest: &str,
        entries: &[Attributed<G::Delta>],
    ) -> Result<(), String> {
        if self.version != 1
            || self.placement != *placement
            || placement.baseline != PlacementProfile::LegacySurfaceV1
            || self.entry_effects.len() != placement.cutoff().index()
            || placement.cutoff().index() > entries.len()
        {
            return Err("invalid legacy translation profile or cursor".into());
        }
        if self.source_digest != source_digest {
            return Err("retained legacy source changed".into());
        }
        if self.translation_digest != self.digest()? {
            return Err("legacy translation checksum mismatch".into());
        }
        for effect in self.entry_effects.iter().flatten() {
            validate_effect_ids(effect)?;
        }
        let graph = self
            .graph_at(entries, placement.cutoff())?
            .ok_or("invalid legacy translation cursor")?;
        if !FrozenGraph::of(&graph).same(&self.legacy_final, hook) {
            return Err("legacy translation effects do not reproduce their frozen result".into());
        }
        self.legacy_final.materialize()?;
        Ok(())
    }

    pub fn journal(
        &self,
        entries: &[Attributed<G::Delta>],
        cursor: Seq,
    ) -> Option<SessionJournal<G>> {
        if cursor.index() > entries.len() {
            return None;
        }
        let mut translated = Journal::new();
        for (index, entry) in entries[..cursor.index()].iter().enumerate() {
            if let Some(effects) = self.entry_effects.get(index) {
                for effect in effects {
                    translated.append(Attributed {
                        author: entry.author.clone(),
                        delta: effect.clone(),
                    });
                }
            } else {
                translated.append(entry.clone());
            }
        }
        Some(SessionJournal::from_log(translated))
    }

    pub fn graph_at(
        &self,
        entries: &[Attributed<G::Delta>],
        cursor: Seq,
    ) -> Result<Option<G>, String> {
        let Some(journal) = self.journal(entries, cursor) else {
            return Ok(None);
        };
        let graph = journal
            .snapshot_at_from(&self.baseline.materialize()?, journal.live_cursor())
            .map_err(|error| error.to_string())?;
        if let Some(graph) = &graph {
            graph.validate_handles()?;
        }
        Ok(graph)
    }

    pub fn effect_cursor(&self, cursor: Seq) -> Seq {
        let prefix = cursor.index().min(self.entry_effects.len());
        Seq(self.entry_effects[..prefix]
            .iter()
            .map(Vec::len)
            .sum::<usize>() as u64
            + cursor.0.saturating_sub(self.entry_effects.len() as u64))
    }
}

/// A receipt's payload fields as stored text, in the order its checksum
/// reads them.
#[derive(Deserialize)]
struct StoredPayload<'a> {
    #[serde(borrow)]
    version: &'a serde_json::value::RawValue,
    #[serde(borrow)]
    source_digest: &'a serde_json::value::RawValue,
    #[serde(borrow)]
    placement: &'a serde_json::value::RawValue,
    #[serde(borrow)]
    baseline: &'a serde_json::value::RawValue,
    #[serde(borrow)]
    entry_effects: &'a serde_json::value::RawValue,
    #[serde(borrow)]
    legacy_final: &'a serde_json::value::RawValue,
    #[serde(borrow, default)]
    content_context: Option<&'a serde_json::value::RawValue>,
}

/// The checksum a receipt's writer took, `serde_json::to_vec` of the payload
/// tuple, recomputed from the stored text: each field compacted, joined as
/// that tuple's JSON array.
fn stored_digest(bytes: &[u8]) -> Result<String, String> {
    let stored: StoredPayload<'_> = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let fields = [
        stored.version,
        stored.source_digest,
        stored.placement,
        stored.baseline,
        stored.entry_effects,
        stored.legacy_final,
    ];
    let mut payload = String::from("[");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            payload.push(',');
        }
        compact_into(&mut payload, field.get());
    }
    payload.push(']');
    if let Some(context) = stored.content_context {
        payload = format!("[{payload},");
        compact_into(&mut payload, context.get());
        payload.push(']');
    }
    Ok(blake3::hash(payload.as_bytes()).to_hex().to_string())
}

/// Append `json` without the whitespace outside its strings.
fn compact_into(out: &mut String, json: &str) {
    let (mut in_string, mut escaped) = (false, false);
    for c in json.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            out.push(c);
        } else if !matches!(c, ' ' | '\t' | '\n' | '\r') {
            in_string = c == '"';
            out.push(c);
        }
    }
}

fn validate_effect_ids<D: Serialize>(effect: &D) -> Result<(), String> {
    let encoded = serde_json::to_value(effect).map_err(|error| error.to_string())?;
    let (_, fields) = encoded
        .as_object()
        .and_then(|value| value.iter().next())
        .ok_or("invalid legacy translation capture")?;
    for (key, value) in fields
        .as_object()
        .ok_or("invalid legacy translation fields")?
    {
        if matches!(
            key.as_str(),
            "id" | "node_id"
                | "from_id"
                | "to_id"
                | "child_id"
                | "parent_id"
                | "resource_id"
                | "from_resource_id"
                | "to_resource_id"
                | "surface_id"
        ) && !value.is_null()
        {
            let id = value
                .as_str()
                .ok_or("invalid legacy translation stable id")?;
            uuid::Uuid::parse_str(id).map_err(|_| "invalid legacy translation stable id")?;
        }
    }
    Ok(())
}

pub(crate) fn source_digest(
    placement: &SessionPlacement,
    baseline: Option<&[u8]>,
    entries: &[Vec<u8>],
) -> Result<String, String> {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mere.legacy-translation-source/v1");
    let mut field = |bytes: Option<&[u8]>| {
        hash.update(&[u8::from(bytes.is_some())]);
        if let Some(bytes) = bytes {
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
    };
    let profile = serde_json::to_vec(placement).map_err(|error| error.to_string())?;
    field(Some(&profile));
    field(baseline);
    for entry in entries {
        field(Some(entry));
    }
    Ok(hash.finalize().to_hex().to_string())
}
