// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph codicils over any session graph (engine split D265, D266). A host's
//! [`CodicilHook`] names the schema ids its snapshot codicils carry and
//! redacts its own snapshot; the save, open and list paths run through
//! eidetic's runtime-schema payload layer. Compose belongs to the host whose
//! graph it merges.

use eidetic::manifest::load_manifest;
use eidetic::{
    BlobManifest, BlobSource, Error, NoFetcher, PayloadSealer, PrivacyClass, TrustEnvelope,
    list_payloads, load_payload_sealed, save_payload_sealed,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::*;

/// How a host's graph binds to graph codicils.
pub struct CodicilHook<G: SessionGraph> {
    /// The unprofiled schema a codicil is saved and read under.
    pub graph_schema: &'static [u8],
    /// The schema a codicil with an explicit placement is saved and read under.
    pub profiled_schema: &'static [u8],
    /// Input adapters: the v3 profiled, v2 and bare-snapshot v1 schemas.
    pub v3_profiled_schema: &'static [u8],
    pub v2_schema: &'static [u8],
    pub v1_schema: &'static [u8],
    /// Strip what `policy` excludes from a snapshot, in place.
    pub redact: fn(&RedactionPolicy, &mut G::Snapshot),
}

fn schema(id: &[u8]) -> SchemaRef {
    SchemaRef::from_id(ManifestId::of_blob(id))
}

impl<G: SessionGraph> ProfiledGraphCodicil<G> {
    /// The graph this codicil loads as under its placement, through the
    /// checked load every open and save runs.
    pub fn into_graph_with(self) -> Result<G> {
        checked_graph_with(&self.codicil.snapshot, self.codicil.facets, self.placement)
    }
}

/// The graph a codicil's parts load as under `placement`: the checked load
/// every open and every save runs, so a save refuses what an open would.
pub(super) fn checked_graph_with<G: SessionGraph>(
    snapshot: &G::Snapshot,
    facets: NodeFacetStore,
    placement: Option<PlacementProfile>,
) -> Result<G> {
    let mut graph = crate::graph_placement::materialize_snapshot_with::<G>(snapshot, placement)
        .map_err(|error| Error::new(format!("invalid graph codicil snapshot: {error}")))?;
    graph
        .install_facets(facets)
        .map_err(|error| Error::new(format!("invalid graph codicil facets: {error}")))?;
    Ok(graph)
}

/// Save `value` under schema `id`, as eidetic's typed save does.
async fn save_payload<T: Serialize>(
    store: &mut dyn Store,
    sealer: Option<&dyn PayloadSealer>,
    id: &[u8],
    value: &T,
    provenance: ProvenanceRecord,
    created_at: Timestamp,
) -> Result<ManifestId> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| Error::new(format!("typed serialize: {e}")))?;
    save_payload_sealed(
        store,
        sealer,
        schema(id),
        &bytes,
        Vec::<BlobSource>::new(),
        PrivacyClass::LocalOnly,
        provenance,
        TrustEnvelope::self_asserted(),
        created_at,
    )
    .await
}

/// Load the payload under `manifest` as schema `id`, as eidetic's typed load does.
async fn load_payload<T: DeserializeOwned>(
    store: &mut dyn Store,
    sealer: Option<&dyn PayloadSealer>,
    id: &[u8],
    manifest: ManifestId,
) -> Result<Option<T>> {
    let mut fetcher = NoFetcher;
    load_payload_sealed(store, &mut fetcher, sealer, schema(id), manifest)
        .await?
        .map(|bytes| {
            serde_json::from_slice(&bytes)
                .map_err(|e| Error::new(format!("typed deserialize: {e}")))
        })
        .transpose()
}

/// [`save_graph_snapshot_codicil_sealed`] over any graph, through `hook`.
pub async fn save_graph_snapshot_codicil_sealed_with<G: SessionGraph>(
    store: &mut dyn Store,
    sealer: Option<&dyn PayloadSealer>,
    hook: &CodicilHook<G>,
    mut snapshot: G::Snapshot,
    mut facets: NodeFacetStore,
    redaction: RedactionPolicy,
    created_at: Timestamp,
) -> Result<ManifestId> {
    checked_graph_with::<G>(&snapshot, facets.clone(), None)?;
    (hook.redact)(&redaction, &mut snapshot);
    redaction.apply_facets(&mut facets);
    save_payload(
        store,
        sealer,
        hook.graph_schema,
        &GraphCodicil::<G> {
            snapshot,
            facets,
            sessions: Vec::new(),
        },
        graph_codicil_provenance(created_at),
        created_at,
    )
    .await
}

/// [`save_profiled_graph_codicil_sealed`] over any graph, through `hook`.
pub async fn save_profiled_graph_codicil_sealed_with<G: SessionGraph>(
    store: &mut dyn Store,
    sealer: Option<&dyn PayloadSealer>,
    hook: &CodicilHook<G>,
    mut input: ProfiledGraphCodicil<G>,
    redaction: RedactionPolicy,
    created_at: Timestamp,
) -> Result<ManifestId> {
    checked_graph_with::<G>(
        &input.codicil.snapshot,
        input.codicil.facets.clone(),
        input.placement,
    )?;
    (hook.redact)(&redaction, &mut input.codicil.snapshot);
    redaction.apply_facets(&mut input.codicil.facets);
    if redaction != RedactionPolicy::include_all() {
        input.codicil.sessions.clear();
    }
    save_payload(
        store,
        sealer,
        hook.profiled_schema,
        &input,
        graph_codicil_provenance(created_at),
        created_at,
    )
    .await
}

/// [`open_codicil_as_session_sealed`] over any graph, through `hook`.
pub async fn open_codicil_as_session_sealed_with<G: SessionGraph>(
    store: &mut dyn Store,
    sealer: Option<&dyn PayloadSealer>,
    hook: &CodicilHook<G>,
    id: ManifestId,
) -> Result<Option<G>> {
    load_profiled_graph_codicil_sealed_with(store, sealer, hook, id)
        .await?
        .map(ProfiledGraphCodicil::into_graph_with)
        .transpose()
}

/// [`list_graph_codicils`] over any graph's schemas.
pub async fn list_graph_codicils_with<G: SessionGraph>(
    store: &mut dyn Store,
    hook: &CodicilHook<G>,
) -> Result<Vec<BlobManifest>> {
    let mut manifests = Vec::new();
    for id in [
        hook.graph_schema,
        hook.v1_schema,
        hook.v2_schema,
        hook.v3_profiled_schema,
        hook.profiled_schema,
    ] {
        manifests.extend(list_payloads(store, schema(id)).await?);
    }
    Ok(manifests)
}

/// [`save_profiled_session_codicil_checked`] over any graph, through `hook`.
pub async fn save_profiled_session_codicil_checked_with<G: SessionGraph>(
    store: &mut dyn Store,
    hook: &CodicilHook<G>,
    payload: ProfiledGraphCodicil<G>,
    created_at: Timestamp,
    validate: impl FnOnce(&G) -> std::result::Result<(), String>,
) -> Result<ManifestId> {
    validate(&payload.clone().into_graph_with()?).map_err(Error::new)?;
    let mut provenance = graph_codicil_provenance(created_at);
    for source in &payload.codicil.sessions {
        if let Some(source) = &source.manifest.source_codicil {
            let id = parse_codicil_id(&source.0).map_err(Error::new)?;
            if !provenance.upstream.contains(&id) {
                provenance.upstream.push(id);
            }
        }
    }
    if !provenance.upstream.is_empty() {
        provenance.origin = ProvenanceOrigin::Derived;
    }
    save_archive_codicil_payload_with(store, hook, payload, provenance, created_at).await
}

pub(super) async fn save_archive_codicil_payload_with<G: SessionGraph>(
    store: &mut dyn Store,
    hook: &CodicilHook<G>,
    payload: ProfiledGraphCodicil<G>,
    provenance: ProvenanceRecord,
    created_at: Timestamp,
) -> Result<ManifestId> {
    if payload.placement.is_some() {
        save_payload(
            store,
            None,
            hook.profiled_schema,
            &payload,
            provenance,
            created_at,
        )
        .await
    } else {
        save_payload(
            store,
            None,
            hook.graph_schema,
            &payload.codicil,
            provenance,
            created_at,
        )
        .await
    }
}

/// Refuse a snapshot that does not materialize unqualified.
fn opens<G: SessionGraph>(snapshot: &G::Snapshot, what: &str) -> Result<()> {
    G::from_snapshot(snapshot, false)
        .map(|_| ())
        .map_err(|error| Error::new(format!("invalid {what}snapshot: {error}")))
}

/// [`load_profiled_graph_codicil_sealed`] over any graph's schemas.
pub async fn load_profiled_graph_codicil_sealed_with<G: SessionGraph>(
    store: &mut dyn Store,
    sealer: Option<&dyn PayloadSealer>,
    hook: &CodicilHook<G>,
    id: ManifestId,
) -> Result<Option<ProfiledGraphCodicil<G>>> {
    let Some(manifest) = load_manifest(store, id).await? else {
        return Ok(None);
    };
    if manifest.schema == schema(hook.profiled_schema) {
        let payload: Option<ProfiledGraphCodicil<G>> =
            load_payload(store, sealer, hook.profiled_schema, id).await?;
        if let Some(input) = &payload {
            opens::<G>(&input.codicil.snapshot, "graph codicil ")?;
        }
        return Ok(payload);
    }
    if manifest.schema == schema(hook.v3_profiled_schema) {
        let payload: Option<ProfiledGraphCodicil<G>> =
            load_payload(store, sealer, hook.v3_profiled_schema, id).await?;
        if let Some(input) = &payload {
            opens::<G>(&input.codicil.snapshot, "v3 graph codicil ")?;
        }
        return Ok(payload);
    }
    for (schema_id, what) in [
        (hook.graph_schema, "graph codicil "),
        (hook.v2_schema, "v2 graph codicil "),
    ] {
        if manifest.schema == schema(schema_id) {
            let payload: Option<GraphCodicil<G>> =
                load_payload(store, sealer, schema_id, id).await?;
            if let Some(codicil) = &payload {
                opens::<G>(&codicil.snapshot, what)?;
            }
            return Ok(payload.map(|codicil| ProfiledGraphCodicil {
                codicil,
                placement: None,
            }));
        }
    }
    if manifest.schema == schema(hook.v1_schema) {
        let legacy: Option<G::Snapshot> = load_payload(store, sealer, hook.v1_schema, id).await?;
        return legacy
            .map(|snapshot| {
                let graph = G::from_snapshot(&snapshot, false).map_err(|error| {
                    Error::new(format!("invalid legacy graph codicil snapshot: {error}"))
                })?;
                Ok(ProfiledGraphCodicil {
                    codicil: GraphCodicil {
                        snapshot: graph.to_snapshot(),
                        facets: graph.facets().clone(),
                        sessions: Vec::new(),
                    },
                    placement: None,
                })
            })
            .transpose();
    }
    Err(Error::new(format!(
        "manifest {} is not a graph codicil",
        id
    )))
}
