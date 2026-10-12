// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Owner bridge for an admitted reader's collection and local public content.
//!
//! Gemot supplies current effective contributions, versions and command gates.
//! Eidetic verifies locally held public bytes. Reads perform no acquisition or
//! storage writes. Private access, durable copying, hosting and application
//! participation require their respective owners; this bridge refuses them.

use eidetic::manifest::{load_manifest, resolve_blob};
use eidetic::{BlobManifest, BlobSource, ManifestId, NoFetcher, PrivacyClass, Store};
use gemot::moot::{
    CollectionFork, CollectionId, CollectionRef, CollectionView, FaunaEntry, Moot,
    MootAuthorizationRequest, MootError, collection_cap,
};
use mien::GateDecision;
use muniment::Backend;
use personae::IdentityProvider;
use servitor::cap_path;

use crate::resource_coop::{
    ActivityRead, CollectionRead, ContentRead, HostingRead, Offer, Outcome, Request,
    ResourceAction, ResourceRead, ResourceRef, Snapshot, Target,
};

pub use crate::resource_coop::ForkDestination;

#[derive(Debug)]
pub enum Error {
    Gemot(Box<MootError>),
    Refused(String),
    Unsupported(String),
    ChangedDuringRead,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gemot(error) => error.fmt(f),
            Self::Refused(reason) | Self::Unsupported(reason) => f.write_str(reason),
            Self::ChangedDuringRead => f.write_str("Collection authority changed during the read"),
        }
    }
}
impl std::error::Error for Error {}
impl Error {
    pub fn outcome(&self) -> Outcome {
        match self {
            Self::Unsupported(reason) => Outcome::Unsupported {
                reason: reason.clone(),
            },
            _ => Outcome::Refused {
                reason: self.to_string(),
            },
        }
    }
}
impl From<MootError> for Error {
    fn from(error: MootError) -> Self {
        Self::Gemot(Box::new(error))
    }
}

/// Actual owner output. Opened bytes go to the host reader, never into a card.
#[derive(Debug)]
pub enum Applied {
    Opened {
        reference: ResourceRef,
        bytes: Vec<u8>,
    },
    CollectionForked {
        collection: [u8; 32],
        operation: [u8; 32],
    },
}

impl Applied {
    pub fn outcome(&self) -> Outcome {
        let receipt = match self {
            Self::Opened { reference, .. } => format!("eidetic:local/{}", hex(&reference.manifest)),
            Self::CollectionForked { operation, .. } => {
                format!("gemot:operation/{}", hex(operation))
            },
        };
        Outcome::Accepted {
            owner_receipt: receipt,
        }
    }
}

async fn admitted<B: Backend + Clone>(moot: &Moot<B>, reader: [u8; 32]) -> Result<(), Error> {
    if moot
        .membership()
        .await?
        .member_snapshots()
        .iter()
        .any(|member| member.member == reader)
    {
        Ok(())
    } else {
        Err(Error::Refused(
            "This reader is not a current Moot member".into(),
        ))
    }
}

/// Build from current owner reads. Application records are already disclosed by
/// their owner; they retain that owner's explicit offers and lifecycle report.
/// No offer is derived from membership, related content or this adapter.
pub async fn snapshot<B: Backend + Clone>(
    moot: &Moot<B>,
    reader: [u8; 32],
    collection_id: [u8; 32],
    destination: Option<&ForkDestination>,
    activities: Vec<ActivityRead>,
    store: &mut dyn Store,
    at_ms: u64,
) -> Result<Snapshot, Error> {
    admitted(moot, reader).await?;
    let view = moot
        .authorized_collection(CollectionId(collection_id), at_ms)
        .await?
        .ok_or_else(|| Error::Refused("The collection has no current authorized view".into()))?;
    let fauna = moot.authorized_fauna(at_ms).await?;
    let mut resources = Vec::new();
    for selected in &view.effective_selected {
        let share = fauna
            .iter()
            .find(|share| share.op_hash == selected.share)
            .ok_or(Error::ChangedDuringRead)?;
        let reference = resource_ref(moot, share);
        let (content, open) = match public_local_bytes(store, reference).await {
            Ok(_) => (ContentRead::Resolved, Offer::Offered),
            Err(reason) => (
                ContentRead::Unavailable {
                    reason: reason.to_string(),
                },
                match reason {
                    LocalError::Unavailable(reason) => Offer::Withheld { reason },
                    LocalError::Unsupported(reason) => Offer::Unsupported { reason },
                },
            ),
        };
        resources.push(ResourceRead {
            reference,
            shared_by: Some(share.shared_by),
            title: share.title.clone(),
            schema: share.schema_id.clone(),
            content,
            hosting: HostingRead::Unknown,
            offers: [
                (ResourceAction::Open, open),
                (
                    ResourceAction::RetainCopy,
                    unsupported("No content-copy grant owner is installed"),
                ),
                (
                    ResourceAction::ForkContent,
                    unsupported("No content-fork owner is installed"),
                ),
                (
                    ResourceAction::OfferHosting,
                    unsupported("No production hosting command owner is installed"),
                ),
            ]
            .into(),
        });
    }
    let fork = match destination {
        Some(destination) => Some(fork_offer(moot, reader, &view, destination, at_ms).await?),
        None => Some(unsupported(
            "Choose and review a destination collection first",
        )),
    };
    // These owner APIs do not supply a cross-store transaction. Refuse a changed
    // view rather than publishing a mixed read. Dispatch repeats the checks.
    admitted(moot, reader).await?;
    let fresh = moot
        .authorized_collection(CollectionId(collection_id), at_ms)
        .await?;
    if fresh.as_ref() != Some(&view) || moot.authorized_fauna(at_ms).await? != fauna {
        return Err(Error::ChangedDuringRead);
    }
    let read = Snapshot {
        moot: moot.moot_id().0,
        reader,
        collection: Some(CollectionRead {
            id: collection_id,
            name: view.name,
            heads: view.version.frontier,
            membership_commitment: view.version.membership_commitment,
            fork,
            fork_destination: destination.cloned(),
        }),
        resources,
        activities,
    };
    read.check()
        .map_err(|error| Error::Refused(error.to_string()))?;
    Ok(read)
}

fn unsupported(reason: &str) -> Offer {
    Offer::Unsupported {
        reason: reason.into(),
    }
}

async fn fork_offer<B: Backend + Clone>(
    moot: &Moot<B>,
    reader: [u8; 32],
    source: &CollectionView,
    destination: &ForkDestination,
    at_ms: u64,
) -> Result<Offer, Error> {
    if destination.name.trim().is_empty() || destination.id == source.collection.collection_id.0 {
        return Ok(Offer::Withheld {
            reason: "Choose a named new collection".into(),
        });
    }
    if source.selected != source.effective_selected {
        return Ok(Offer::Withheld {
            reason: "The exact parent includes currently withheld contributions".into(),
        });
    }
    let roster = moot
        .object_store()
        .roster(moot.moot_id().0)
        .await
        .map_err(MootError::from)?;
    if roster.collections.iter().any(|fact| match &fact.event {
        gemot::moot::CollectionEvent::Declared { collection_id, .. } => {
            collection_id.0 == destination.id
        },
        gemot::moot::CollectionEvent::Changed { collection, .. } => {
            collection.collection_id.0 == destination.id
        },
    }) {
        return Ok(Offer::Withheld {
            reason: "That collection identity already has history".into(),
        });
    }
    let group = moot.membership().await?;
    let request = MootAuthorizationRequest {
        subject: reader,
        capability_path: cap_path(&collection_cap(CollectionRef {
            moot_id: moot.moot_id().0,
            collection_id: CollectionId(destination.id),
        })),
        at_ms,
    };
    Ok(
        match moot.authorize_current_capability(&group, &request).await? {
            GateDecision::Allow => Offer::Offered,
            GateDecision::Deny(reason) => Offer::Withheld {
                reason: format!("Current destination authority refused: {reason:?}"),
            },
        },
    )
}

/// Handle supported intentions using current authority and identity. The host
/// must separately route activity requests to their exact application owner.
pub async fn dispatch<B: Backend + Clone, P: IdentityProvider + ?Sized>(
    moot: &Moot<B>,
    identity: &P,
    store: &mut dyn Store,
    request: &Request,
    destination: Option<&ForkDestination>,
    at_ms: u64,
) -> Result<Applied, Error> {
    let reader = identity.master_public_key().to_bytes();
    if request.reader != reader {
        return Err(Error::Refused(
            "The request belongs to another reader".into(),
        ));
    }
    admitted(moot, reader).await?;
    match &request.target {
        Target::Resource {
            reference,
            action: ResourceAction::Open,
        } => {
            let fauna = moot.authorized_fauna(at_ms).await?;
            if reference.moot != moot.moot_id().0
                || !fauna
                    .iter()
                    .any(|share| resource_ref(moot, share) == *reference)
            {
                return Err(Error::Refused(
                    "The exact contribution is no longer authorized".into(),
                ));
            }
            let bytes =
                public_local_bytes(store, *reference)
                    .await
                    .map_err(|error| match error {
                        LocalError::Unavailable(reason) => Error::Refused(reason),
                        LocalError::Unsupported(reason) => Error::Unsupported(reason),
                    })?;
            admitted(moot, reader).await?;
            if !moot
                .authorized_fauna(at_ms)
                .await?
                .iter()
                .any(|share| resource_ref(moot, share) == *reference)
            {
                return Err(Error::ChangedDuringRead);
            }
            Ok(Applied::Opened {
                reference: *reference,
                bytes,
            })
        },
        Target::CollectionFork {
            moot: requested,
            observed,
        } => {
            if *requested != moot.moot_id().0 {
                return Err(Error::Refused("The request belongs to another Moot".into()));
            }
            let source = moot
                .authorized_collection(CollectionId(observed.id), at_ms)
                .await?
                .ok_or_else(|| {
                    Error::Refused("The source collection is no longer authorized".into())
                })?;
            if source.version.frontier != observed.heads
                || source.version.membership_commitment != observed.membership_commitment
            {
                return Err(Error::Refused(
                    "The exact source collection version changed".into(),
                ));
            }
            let destination = destination
                .ok_or_else(|| Error::Refused("A reviewed destination is required".into()))?;
            if observed.fork_destination.as_ref() != Some(destination) {
                return Err(Error::Refused("The reviewed destination changed".into()));
            }
            match fork_offer(moot, reader, &source, destination, at_ms).await? {
                Offer::Offered => {},
                other => {
                    return Err(Error::Refused(format!(
                        "Collection fork refused: {other:?}"
                    )));
                },
            }
            let fork = CollectionFork::new(source.version, source.effective_selected);
            let receipt = moot
                .declare_collection_for_identity(
                    identity,
                    CollectionId(destination.id),
                    destination.name.clone(),
                    Some(fork),
                    at_ms,
                )
                .await?;
            Ok(Applied::CollectionForked {
                collection: destination.id,
                operation: receipt.operation,
            })
        },
        Target::Resource { .. } => Err(Error::Unsupported(
            "This action needs a content grant or hosting owner".into(),
        )),
        Target::Activity { .. } => Err(Error::Unsupported(
            "Route this action to its exact application owner".into(),
        )),
    }
}

fn resource_ref<B: Backend + Clone>(moot: &Moot<B>, share: &FaunaEntry) -> ResourceRef {
    ResourceRef {
        moot: moot.moot_id().0,
        contribution: share.op_hash,
        manifest: share.manifest_id,
    }
}

enum LocalError {
    Unavailable(String),
    Unsupported(String),
}
impl std::fmt::Display for LocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(reason) | Self::Unsupported(reason) => f.write_str(reason),
        }
    }
}

async fn public_local_bytes(
    store: &mut dyn Store,
    reference: ResourceRef,
) -> Result<Vec<u8>, LocalError> {
    let id = ManifestId(
        eidetic::Hash::parse(&hex(&reference.manifest))
            .map_err(|error| LocalError::Unavailable(error.to_string()))?,
    );
    let manifest: BlobManifest = load_manifest(store, id)
        .await
        .map_err(|error| LocalError::Unavailable(error.to_string()))?
        .ok_or_else(|| LocalError::Unavailable("The exact manifest is not held locally".into()))?;
    if manifest.id != id || manifest.content_hash != id.0 {
        return Err(LocalError::Unavailable(
            "Manifest identity or content hash is invalid".into(),
        ));
    }
    if manifest.manifest_version != BlobManifest::CURRENT_VERSION {
        return Err(LocalError::Unsupported(
            "The manifest version is unsupported".into(),
        ));
    }
    if manifest.byte_size > 4 * 1024 * 1024 {
        return Err(LocalError::Unsupported(
            "This local reader accepts at most 4 MiB; larger content needs a streaming reader"
                .into(),
        ));
    }
    if manifest.privacy != PrivacyClass::PublicPortable {
        return Err(LocalError::Unsupported(
            "Private content needs its own access and key adapter".into(),
        ));
    }
    if eidetic::seal_marker(&manifest)
        .map_err(|error| LocalError::Unavailable(error.to_string()))?
        .is_some()
    {
        return Err(LocalError::Unsupported(
            "Sealed content needs its own key adapter".into(),
        ));
    }
    let local = BlobManifest {
        sources: manifest
            .sources
            .iter()
            .filter(|source| {
                matches!(
                    source,
                    BlobSource::Local { .. } | BlobSource::Embedded { .. }
                )
            })
            .cloned()
            .collect(),
        ..manifest
    };
    let bytes = resolve_blob(store, &mut NoFetcher, &local)
        .await
        .map_err(|error| LocalError::Unavailable(error.to_string()))?;
    if bytes.len() as u64 != local.byte_size {
        return Err(LocalError::Unavailable(
            "Content size differs from the exact manifest".into(),
        ));
    }
    Ok(bytes)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
