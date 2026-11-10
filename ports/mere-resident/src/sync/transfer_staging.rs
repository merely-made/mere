// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Stage-and-forward: putting every byte a transfer names onto the destination
//! before anything is applied.
//!
//! `apply_transfer` takes a source blob store and a destination blob store. On
//! one machine that is two arguments; across two devices it would be a store
//! that lives somewhere else, and every read inside apply would become a
//! network call that can fail halfway through a graph mutation. Fetching first
//! dissolves that: once the destination holds the manifest and every blob it
//! names, apply runs entirely locally and both arguments are the same store.
//!
//! Three BLAKE3 hash types meet here. `eidetic::Hash` addresses manifest
//! descriptors, `muniment::Hash` keys the product blob store, and
//! `transport::BlobHash` keys the iroh store the sync host serves. They agree
//! bytewise for the same content, and this module checks that rather than
//! assuming it: a silent disagreement would offer blobs under hashes no
//! destination can ask for.

use chirograph::ContentHash;
use eidetic::Hash;
use muniment::{Backend, BlobStore};
use transport::BlobHash;

use crate::sync::personal_sync_host::{PersonalSyncHost, PersonalSyncHostError};
use pandect::transfer::{TransferManifestV1, TransferOperation};
use pandect::transfer_offer::{OfferError, TransferOfferV1, offer_events, offer_for};

#[derive(Debug, thiserror::Error)]
pub enum StagingError {
    #[error(transparent)]
    Host(#[from] PersonalSyncHostError),
    #[error(transparent)]
    Store(#[from] muniment::StoreError),
    #[error(transparent)]
    Offer(#[from] OfferError),
    #[error("transfer manifest could not be encoded: {0}")]
    Encode(String),
    #[error("blob {hash} named by the manifest is not in the source store")]
    MissingSourceBlob { hash: String },
    /// The three blob addressings disagreed about the same bytes.
    #[error("{context}: staged {staged} but the manifest names {named}")]
    HashDisagreement {
        context: &'static str,
        staged: String,
        named: String,
    },
    #[error("offer names transfer {offered} but the fetched manifest is {fetched}")]
    ManifestMismatch { offered: String, fetched: String },
    #[error(
        "this device does not advertise blob availability, so a sibling could \
         not learn where the transfer's bytes are; enable the blob-availability \
         lane before offering a transfer"
    )]
    BlobLaneDisabled,
}

/// Stage a prepared manifest and its blobs for delivery, then announce it.
///
/// Order matters. Every blob is staged and advertised before the offer is
/// authored, so a destination that acts on the offer the instant it arrives
/// finds bytes rather than a promise. `stage_blob` already flushes before it
/// advertises each blob for the same reason.
pub async fn offer_transfer<B: Backend + Clone + Send + Sync + 'static>(
    host: &PersonalSyncHost,
    source_blobs: &BlobStore<B>,
    manifest: &TransferManifestV1,
    pairing_id: &str,
    at_ms: u64,
) -> Result<TransferOfferV1, StagingError> {
    // Checked before the first byte moves. Staging authors an availability
    // record per blob, so a host with that lane off fails partway through with
    // bytes already written and nothing advertised.
    if !host.serves_blobs().await {
        return Err(StagingError::BlobLaneDisabled);
    }
    for descriptor in &manifest.blobs {
        let key = muniment_hash(descriptor.content_hash)?;
        let bytes =
            source_blobs
                .get(&key)
                .await?
                .ok_or_else(|| StagingError::MissingSourceBlob {
                    hash: descriptor.content_hash.to_hex(),
                })?;
        let staged = host.stage_blob(descriptor.node_id, bytes).await?;
        if &staged != descriptor.content_hash.as_bytes() {
            return Err(StagingError::HashDisagreement {
                context: "staging a transfer blob",
                staged: hex(&staged),
                named: descriptor.content_hash.to_hex(),
            });
        }
    }

    let manifest_bytes =
        serde_json::to_vec(manifest).map_err(|error| StagingError::Encode(error.to_string()))?;
    let manifest_blob = Hash::of(&manifest_bytes);
    let byte_len = manifest_bytes.len() as u64;
    let staged = host
        .stage_blob(manifest.transfer_id, manifest_bytes)
        .await?;
    if &staged != manifest_blob.as_bytes() {
        return Err(StagingError::HashDisagreement {
            context: "staging the transfer manifest",
            staged: hex(&staged),
            named: manifest_blob.to_hex(),
        });
    }

    let offer = offer_for(manifest, manifest_blob, byte_len, pairing_id, at_ms)?;
    host.author(offer_events(&offer)?).await?;
    tracing::info!(
        transfer = %manifest.transfer_id,
        blobs = manifest.blobs.len(),
        bytes = offer.blob_bytes,
        "offered a transfer to a paired device"
    );
    Ok(offer)
}

/// Fetch everything an offer names into the local stores, and return the
/// manifest that governs the apply.
///
/// The returned manifest comes off the wire, not from the offer: the offer's
/// counts are a display summary and this is the authoritative record. The two
/// are checked against each other for the one thing that would make the pair
/// incoherent, which is naming different transfers.
///
/// Idempotent. `fetch_blob_by_availability` short-circuits on bytes already
/// held, so a re-run after a partial fetch costs a store lookup per blob.
pub async fn receive_transfer<B: Backend + Clone + Send + Sync + 'static>(
    host: &PersonalSyncHost,
    destination_blobs: &BlobStore<B>,
    offer: &TransferOfferV1,
) -> Result<TransferManifestV1, StagingError> {
    let manifest_bytes = fetch_into(host, *offer.manifest_blob.as_bytes()).await?;
    let manifest: TransferManifestV1 = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| StagingError::Encode(error.to_string()))?;
    if manifest.transfer_id != offer.transfer_id {
        return Err(StagingError::ManifestMismatch {
            offered: offer.transfer_id.to_string(),
            fetched: manifest.transfer_id.to_string(),
        });
    }

    for descriptor in &manifest.blobs {
        let bytes = fetch_into(host, *descriptor.content_hash.as_bytes()).await?;
        // Writing through the product store returns its own addressing of the
        // same bytes, which is the cheapest place to notice the three hash
        // constructions drifting apart.
        let stored = destination_blobs.put(&bytes).await?;
        if stored.as_bytes() != descriptor.content_hash.as_bytes() {
            return Err(StagingError::HashDisagreement {
                context: "storing a fetched transfer blob",
                staged: stored.to_hex(),
                named: descriptor.content_hash.to_hex(),
            });
        }
    }
    tracing::info!(
        transfer = %manifest.transfer_id,
        blobs = manifest.blobs.len(),
        operation = ?manifest.operation,
        "staged a transfer locally; apply no longer needs the source"
    );
    Ok(manifest)
}

/// The blobs a staged transfer needs handed to the browser, ready to release.
///
/// Read from the local store, so this is only callable after
/// [`receive_transfer`] has put every byte here. Returned rather than released
/// directly because releasing is a grant: what makes bytes reachable from a
/// browser is a person accepting the transfer, not this device holding them.
pub async fn released_blobs_for(
    host: &PersonalSyncHost,
    manifest: &TransferManifestV1,
) -> Result<Vec<(ContentHash, Vec<u8>)>, StagingError> {
    let mut released = Vec::with_capacity(manifest.blobs.len());
    for descriptor in &manifest.blobs {
        let blob = *descriptor.content_hash.as_bytes();
        let bytes = host
            .blobs()
            .get_bytes(BlobHash::from_bytes(blob))
            .await
            .map_err(|error| PersonalSyncHostError::Transport(error.to_string()))?
            .to_vec();
        // The browser addresses resources by its own hash type. Deriving it
        // from the bytes rather than converting the manifest's hash means a
        // disagreement surfaces here instead of as a resource the browser
        // asks for and never receives.
        let resource = ContentHash::of(&bytes);
        if resource.0 != blob {
            return Err(StagingError::HashDisagreement {
                context: "releasing a transfer blob to a browser",
                staged: format!("{resource}"),
                named: descriptor.content_hash.to_hex(),
            });
        }
        released.push((resource, bytes));
    }
    Ok(released)
}

/// Whether this offer may be applied here at all, before any bytes move.
///
/// Grant is the pairing, not the device: `pairing_id` is minted at pair time
/// and retired by unpair, so a transfer queued under a retired pairing is
/// refused even though the two node ids still match.
pub fn offer_is_grantable(offer: &TransferOfferV1, current_pairing: Option<&str>) -> bool {
    match current_pairing {
        Some(pairing) => offer.pairing_id == pairing,
        None => false,
    }
}

/// Whether a replicate offer's endpoints agree with its operation. Copy across
/// personas is legitimate; replicate across them is not, and `apply_transfer`
/// refuses it, so catching it here saves a fetch.
pub fn offer_is_coherent(offer: &TransferOfferV1) -> bool {
    offer.operation != TransferOperation::Replicate
        || offer.source.persona == offer.destination.persona
}

async fn fetch_into(host: &PersonalSyncHost, blob: [u8; 32]) -> Result<Vec<u8>, StagingError> {
    host.fetch_blob_by_availability(blob).await?;
    let bytes = host
        .blobs()
        .get_bytes(BlobHash::from_bytes(blob))
        .await
        .map_err(|error| PersonalSyncHostError::Transport(error.to_string()))?;
    Ok(bytes.to_vec())
}

/// Both are BLAKE3 of the same bytes; hex is the only conversion Muniment
/// exposes. A failure here means the two are no longer the same digest, which
/// is worth an error rather than an unwrap.
fn muniment_hash(hash: Hash) -> Result<muniment::Hash, StagingError> {
    muniment::Hash::from_hex(&hash.to_hex()).ok_or_else(|| StagingError::HashDisagreement {
        context: "reading a manifest blob from the product store",
        staged: "not a Muniment BLAKE3 hash".to_string(),
        named: hash.to_hex(),
    })
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::personal_sync_host::PersonalSyncHostConfig;
    use eidetic::{
        Codicil, ModerationState, PrivacyClass, ProvenanceOrigin, ProvenanceRecord, TimeBounds,
        Timestamp, TrustEnvelope, TrustLevel, TypedPayload, validate_payload,
    };
    use kernel::geometry::PortablePoint;
    use kernel::graph::Graph;
    use kernel::graph::apply::add_node;
    use muniment::MemoryBackend;
    use pandect::personal_sync::{SyncRoster, SyncSelection};
    use pandect::product_codicil::{
        PRODUCT_CODICIL_SCHEMA, ProductCodicilV3, ProfiledProductCodicil, TransferScope,
        product_codicil_schema,
    };
    use pandect::transfer::{
        AccessTransferPolicy, TRANSFER_MANIFEST_SCHEMA, TransferBlobV1, TransferEndpointV1,
        TransferRouteV1,
    };
    use pandect::transfer_offer::TRANSFER_OFFER_FACET;
    use personae::{IdentityProvider, InMemoryProvider};
    use uuid::Uuid;

    const PERSONA: &str = "personae://persona/staging";
    const FILE_BYTES: &[u8] = b"bytes that must survive the source going away\n";
    const EXPORTED_AT_MS: u64 = 1_700_000_000_500;

    fn endpoint(device: [u8; 32]) -> TransferEndpointV1 {
        TransferEndpointV1 {
            graph: "graphshell://graph/staging".to_string(),
            persona: PERSONA.to_string(),
            device: format!("personae://device/{}", hex(&device)),
        }
    }

    /// A manifest as the app's `prepare_transfer` writes one, built from the
    /// record types alone: a selection of one URL node and one file node, and
    /// the file's bytes put into the source store under the hash it names.
    async fn source_manifest(
        source: [u8; 32],
        destination: [u8; 32],
        source_blobs: &BlobStore<MemoryBackend>,
    ) -> TransferManifestV1 {
        let (url, file) = (Uuid::new_v4(), Uuid::new_v4());
        let mut graph = Graph::new();
        for (id, address) in [
            (url, "https://example.test/s3"),
            (file, "file:///s3-reference.txt"),
        ] {
            add_node(&mut graph, Some(id), address.to_string(), PortablePoint::new(0.0, 0.0));
        }
        let payload = serde_json::to_vec(&ProfiledProductCodicil {
            product: ProductCodicilV3 {
                schema: PRODUCT_CODICIL_SCHEMA.to_string(),
                scope: TransferScope::SelectedSubgraph,
                exported_at_ms: EXPORTED_AT_MS,
                graph: graph.to_snapshot(),
                facets: pandect::NodeFacetStore::new(),
                scene: None,
            },
            placement: None,
        })
        .unwrap();
        let selection_schema = product_codicil_schema();
        validate_payload(&selection_schema, &payload).unwrap();
        let selection = Codicil::new(
            ProductCodicilV3::schema_ref(),
            payload,
            PrivacyClass::TrustedPeersOnly,
            ProvenanceRecord {
                origin: ProvenanceOrigin::Generated,
                upstream: Vec::new(),
                tooling: None,
                generated_at: Timestamp(EXPORTED_AT_MS),
            },
            TrustEnvelope {
                level: TrustLevel::SelfAsserted,
                signatures: Vec::new(),
                moderation_state: ModerationState::Unreviewed,
            },
            TimeBounds::at(Timestamp(EXPORTED_AT_MS)),
        );
        let stored = source_blobs.put(FILE_BYTES).await.unwrap();
        let content_hash = Hash::of(FILE_BYTES);
        assert_eq!(stored.to_hex(), content_hash.to_hex());
        TransferManifestV1 {
            schema: TRANSFER_MANIFEST_SCHEMA.to_string(),
            transfer_id: Uuid::new_v4(),
            operation: TransferOperation::Replicate,
            source: endpoint(source),
            destination: endpoint(destination),
            route: TransferRouteV1 {
                carrier: "personal-sync".to_string(),
                peer: hex(&destination),
            },
            selection_schema,
            selection,
            blobs: vec![TransferBlobV1 {
                node_id: file,
                role: "primary".to_string(),
                media_type: "text/plain".to_string(),
                content_hash,
                byte_len: FILE_BYTES.len() as u64,
            }],
            access_policy: AccessTransferPolicy::ExcludeSourceHistory,
            access_records: Vec::new(),
        }
    }

    /// The whole point of stage-and-forward: by the time apply runs, the source
    /// is gone. Its sync host is closed and its blob store dropped, and the
    /// destination's one store must then hold the manifest and every byte it
    /// names, so the app's `apply_transfer` can take that store as both of its
    /// arguments. If any byte were still across the wire, this could not pass.
    #[tokio::test]
    async fn a_transfer_is_whole_on_the_destination_after_its_source_is_gone() {
        let directory = tempfile::tempdir().unwrap();
        let graph = [0xa3; 32];
        let source_identity = InMemoryProvider::from_seed([0xa4; 32]);
        let destination_identity = InMemoryProvider::from_seed([0xa5; 32]);
        let roster = SyncRoster::new([
            source_identity.master_public_key().to_bytes(),
            destination_identity.master_public_key().to_bytes(),
        ]);
        // Offers announce a transfer; blob availability is how its bytes are
        // found. A transfer needs both lanes, which `offer_transfer` checks.
        let selection = || {
            SyncSelection::default()
                .with_facets([TRANSFER_OFFER_FACET])
                .with_blob_availability(true)
        };

        let source_sync = PersonalSyncHost::open(
            &source_identity,
            PersonalSyncHostConfig {
                graph,
                store_path: directory.path().join("source.redb"),
                roster: roster.clone(),
                selection: selection(),
                peer_tickets: Vec::new(),
                peer_hints: Vec::new(),
                paired_nodes: Vec::new(),
                relay_urls: Vec::new(),
            },
        )
        .await
        .unwrap();
        let destination_sync = PersonalSyncHost::open(
            &destination_identity,
            PersonalSyncHostConfig {
                graph,
                store_path: directory.path().join("destination.redb"),
                roster,
                selection: selection(),
                peer_tickets: vec![source_sync.ticket().await.unwrap()],
                peer_hints: Vec::new(),
                paired_nodes: Vec::new(),
                relay_urls: Vec::new(),
            },
        )
        .await
        .unwrap();
        source_sync
            .pair_node(destination_sync.node_id())
            .await
            .unwrap();

        let source_blobs = BlobStore::new(MemoryBackend::new());
        let manifest = source_manifest(
            source_sync.node_id(),
            destination_sync.node_id(),
            &source_blobs,
        )
        .await;

        let offered = offer_transfer(
            &source_sync,
            &source_blobs,
            &manifest,
            "pairing-s3",
            1_700_000_002_000,
        )
        .await
        .unwrap();
        assert!(offer_is_coherent(&offered));
        assert!(offer_is_grantable(&offered, Some("pairing-s3")));
        assert!(
            !offer_is_grantable(&offered, Some("pairing-s3-after-repair")),
            "a re-paired device must not inherit a queued transfer"
        );
        assert_eq!(offered.nodes, 2, "the offer summarizes the selection");
        assert_eq!(offered.relations, 0);
        assert_eq!(offered.blobs, 1);
        assert_eq!(offered.blob_bytes, FILE_BYTES.len() as u64);

        // The destination learns of the transfer from the graph, not the test.
        let mut incoming = Vec::new();
        for _ in 0..80 {
            incoming = destination_sync.offers().await.unwrap();
            if !incoming.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        assert_eq!(incoming, vec![offered.clone()]);

        let destination_blobs = BlobStore::new(MemoryBackend::new());
        let fetched = receive_transfer(&destination_sync, &destination_blobs, &incoming[0])
            .await
            .unwrap();
        assert_eq!(fetched.transfer_id, manifest.transfer_id);

        // Everything the source contributes ends here.
        source_sync.close().await.unwrap();
        drop(source_blobs);

        assert_eq!(
            fetched, manifest,
            "the manifest off the wire is the one the source staged"
        );
        for descriptor in &fetched.blobs {
            let key = muniment_hash(descriptor.content_hash).unwrap();
            assert_eq!(
                destination_blobs.get(&key).await.unwrap().as_deref(),
                Some(FILE_BYTES),
                "every byte the manifest names is in the destination's own store"
            );
        }
        // Re-summarizing the fetched manifest decodes its selection again,
        // here, with the source gone.
        let resummarized = offer_for(
            &fetched,
            offered.manifest_blob,
            offered.manifest_byte_len,
            offered.pairing_id.clone(),
            offered.offered_at_ms,
        )
        .unwrap();
        assert_eq!(resummarized, offered, "the offer's summary held");

        // The bytes a browser would pull are the bytes that were staged.
        // Derived from the local store, so this also proves the release list
        // survives the source being gone.
        let released = released_blobs_for(&destination_sync, &fetched)
            .await
            .unwrap();
        assert_eq!(released.len(), 1);
        assert_eq!(released[0].1, FILE_BYTES);
        assert_eq!(
            released[0].0,
            chirograph::ContentHash::of(FILE_BYTES),
            "the browser addresses this blob by its own hash of the same bytes"
        );

        destination_sync.close().await.unwrap();
    }

    /// A missing lane must cost the offer, not leave bytes half-staged. The
    /// refusal has to land before the first `stage_blob`, because that call
    /// writes and advertises in one step.
    #[tokio::test]
    async fn offering_without_the_blob_lane_is_refused_before_any_byte_is_staged() {
        let directory = tempfile::tempdir().unwrap();
        let identity = InMemoryProvider::from_seed([0xa6; 32]);
        let sync = PersonalSyncHost::open(
            &identity,
            PersonalSyncHostConfig {
                graph: [0xa7; 32],
                store_path: directory.path().join("no-blob-lane.redb"),
                roster: SyncRoster::new([identity.master_public_key().to_bytes()]),
                // Offers on, blob availability off: the device can announce a
                // transfer it has no way to let anyone collect.
                selection: SyncSelection::default().with_facets([TRANSFER_OFFER_FACET]),
                peer_tickets: Vec::new(),
                peer_hints: Vec::new(),
                paired_nodes: Vec::new(),
                relay_urls: Vec::new(),
            },
        )
        .await
        .unwrap();

        let blobs = BlobStore::new(MemoryBackend::new());
        let manifest = source_manifest(sync.node_id(), [0xa8; 32], &blobs).await;

        let refused = offer_transfer(&sync, &blobs, &manifest, "pairing-s3", 1_700_000_002_000)
            .await
            .unwrap_err();
        assert!(matches!(refused, StagingError::BlobLaneDisabled));
        for descriptor in &manifest.blobs {
            assert!(
                !sync
                    .blobs()
                    .has(BlobHash::from_bytes(*descriptor.content_hash.as_bytes()))
                    .await
                    .unwrap(),
                "a refused offer must not leave its bytes in the serving store"
            );
        }
        assert!(
            sync.offers().await.unwrap().is_empty(),
            "and it must not announce a transfer nobody can collect"
        );

        sync.close().await.unwrap();
    }
}
