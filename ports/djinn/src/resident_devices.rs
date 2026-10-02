// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The paired-device directory: where each paired device is now, read from the
//! running resident over the owner-only application door (pairing plan D1,
//! ruling 11).
//!
//! Read-only. The profile's settings file names the paired devices; the live
//! transport says whether each is connected and on which path; the saved dial
//! hint arrives decoded, so no reader parses an iroh ticket. Unpaired peers that
//! discovery happens to see are not listed.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use chirograph::{
    BoundsRelationship, CachePolicy, CardValueV1, ContentHash, EndpointDescriptor,
    IntentInvocation, IntentResult, PortableCardV1, PresentationBinding, PresentationCapability,
    PresentationCodec, PresentationKey, PresentationManifest, PresentationOffer,
    PresentationSemantics, ProjectionOffer, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceRequest, ResourceResponse, SemanticRole,
};
use graphshell::native::app_admission::{AppId, AppRouteGrants};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::endpoint_catalog::{
    ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointRoute,
};
use graphshell::native::personal_sync_host::PersonalSyncHost;
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use serde::{Deserialize, Serialize};
use transport::p2panda_transport::KnownPeer;
use transport::{PeerAddr, PeerPath};

use crate::settings::{self as owner_settings, OwnerSettings, PairedDevice};

/// The resident route the directory is served on.
pub const DEVICE_DIRECTORY_ROUTE: &str = "device-directory-v1";
/// The first-party label the directory's caller connects as. A label, not a
/// credential: the door's owner-only endpoint is the boundary, and this route
/// is granted to this label alone.
pub const DEVICE_DIRECTORY_APP: &str = "djinn";
const SESSION: &str = "djinn.device-directory/v1";
const RESOURCE_LABEL: &str = "djinn.device-directory/v1";

/// Every paired device of this profile, and where each is now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceDirectoryV1 {
    pub version: u8,
    /// This device's own personal-graph node id.
    pub local_node: String,
    pub devices: Vec<PairedDeviceV1>,
}

/// One paired device, identified by its personal-graph node id (ruling 2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedDeviceV1 {
    pub node_id: String,
    pub label: String,
    /// The device's Personae root; `None` is a receive-only pairing.
    pub root: Option<String>,
    pub pairing_id: Option<String>,
    pub added_ms: u64,
    /// The endpoint holds an active path to it now.
    pub connected: bool,
    /// The transport holds an address for it. Not a live link.
    pub reachable: bool,
    /// Every address the transport holds for it, the live ones marked active.
    pub path: Vec<PathAddrV1>,
    /// The last saved dial hint, decoded. `None` when none was saved.
    pub hint: Option<SavedHintV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AddrKindV1 {
    Direct,
    Relay,
    Other,
}

/// One address on a device's current path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathAddrV1 {
    pub kind: AddrKindV1,
    pub addr: String,
    pub active: bool,
}

/// One address a saved hint carries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HintAddrV1 {
    pub kind: AddrKindV1,
    pub addr: String,
}

/// A saved dial hint, decoded into its addresses and relay.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedHintV1 {
    pub addrs: Vec<HintAddrV1>,
    /// Why the stored hint could not be read; `addrs` is empty then.
    pub unreadable: Option<String>,
}

type DirectoryFuture = Pin<Box<dyn Future<Output = Result<DeviceDirectoryV1, String>> + Send>>;

/// Where the directory's facts come from, read once per snapshot.
#[derive(Clone)]
pub struct DeviceDirectorySource {
    read: Arc<dyn Fn() -> DirectoryFuture + Send + Sync>,
}

impl DeviceDirectorySource {
    /// The live resident: the settings file names the paired devices, the host
    /// says where they are.
    pub fn live(host: Arc<PersonalSyncHost>, settings_file: PathBuf) -> Self {
        Self::from_fn(move || {
            let host = Arc::clone(&host);
            let settings_file = settings_file.clone();
            Box::pin(async move { read_live(&host, &settings_file).await })
        })
    }

    /// A profile with personal sync off has no directory, and says so.
    pub fn sync_off() -> Self {
        Self::from_fn(|| {
            Box::pin(async {
                Err(
                    "personal sync is not configured for this profile, so no device is paired"
                        .to_string(),
                )
            })
        })
    }

    /// Any reader, for receipts and other composers.
    pub fn from_fn(read: impl Fn() -> DirectoryFuture + Send + Sync + 'static) -> Self {
        Self {
            read: Arc::new(read),
        }
    }

    pub async fn read(&self) -> Result<DeviceDirectoryV1, String> {
        (self.read)().await
    }
}

async fn read_live(
    host: &PersonalSyncHost,
    settings_file: &Path,
) -> Result<DeviceDirectoryV1, String> {
    let settings = OwnerSettings::load(settings_file).map_err(|error| error.to_string())?;
    let paired = settings
        .sync
        .map(|sync| sync.paired_devices)
        .unwrap_or_default();
    let peers = host
        .known_peers()
        .await
        .map_err(|error| error.to_string())?;
    let mut devices = Vec::with_capacity(paired.len());
    for device in &paired {
        let node =
            owner_settings::parse_hex32(&device.node_id).map_err(|error| error.to_string())?;
        let live = peers.iter().find(|peer| peer.peer.to_bytes() == node);
        let paths = host
            .peer_paths(node)
            .await
            .map_err(|error| error.to_string())?;
        devices.push(directory_entry(device, live, &paths));
    }
    Ok(DeviceDirectoryV1 {
        version: 1,
        local_node: owner_settings::hex32(&host.node_id()),
        devices,
    })
}

/// One device's entry from its pairing record, the transport's view of it
/// (absent when the overlay does not list it), and its current paths.
pub fn directory_entry(
    device: &PairedDevice,
    live: Option<&KnownPeer>,
    paths: &[PeerPath],
) -> PairedDeviceV1 {
    PairedDeviceV1 {
        node_id: device.node_id.to_ascii_lowercase(),
        label: device.label.clone(),
        root: device.root.clone(),
        pairing_id: device.pairing_id.clone(),
        added_ms: device.added_ms,
        connected: live.is_some_and(|peer| peer.connected),
        reachable: live.is_some_and(|peer| peer.reachable),
        path: paths
            .iter()
            .map(|path| {
                let (kind, addr) = addr_parts(&path.addr);
                PathAddrV1 {
                    kind,
                    addr,
                    active: path.active,
                }
            })
            .collect(),
        hint: device
            .last_endpoint
            .as_deref()
            .map(|ticket| saved_hint(&device.node_id, ticket)),
    }
}

fn saved_hint(node_id: &str, ticket: &str) -> SavedHintV1 {
    let unreadable = |reason: String| SavedHintV1 {
        addrs: Vec::new(),
        unreadable: Some(reason),
    };
    match transport::decode_peer_ticket(ticket) {
        Ok((peer, addrs))
            if owner_settings::hex32(&peer.to_bytes()).eq_ignore_ascii_case(node_id) =>
        {
            SavedHintV1 {
                addrs: addrs
                    .iter()
                    .map(|addr| {
                        let (kind, addr) = addr_parts(addr);
                        HintAddrV1 { kind, addr }
                    })
                    .collect(),
                unreadable: None,
            }
        },
        Ok((peer, _)) => unreadable(format!(
            "the hint names another node, {}",
            owner_settings::hex32(&peer.to_bytes())
        )),
        Err(error) => unreadable(error.to_string()),
    }
}

fn addr_parts(addr: &PeerAddr) -> (AddrKindV1, String) {
    match addr {
        PeerAddr::Direct(addr) => (AddrKindV1::Direct, addr.to_string()),
        PeerAddr::Relay(url) => (AddrKindV1::Relay, url.clone()),
        PeerAddr::Other(addr) => (AddrKindV1::Other, addr.clone()),
    }
}

/// Grant the directory route to its one caller. The resident and its refusal
/// receipt both grant through here, so they cannot disagree.
pub fn grant(grants: &AppRouteGrants, route: ResidentEndpointRoute) {
    grants.grant(AppId::new(DEVICE_DIRECTORY_APP), route);
}

/// Read the directory through an open route: snapshot, card, typed resource.
pub async fn read_directory(
    client: &mut AppBrokerClient,
) -> Result<DeviceDirectoryV1, AppClientError> {
    let opened = client.open_session().await?;
    let request = opened
        .descriptor
        .projections
        .first()
        .ok_or_else(|| AppClientError::Refused("the directory offered no projection".into()))?
        .request
        .clone();
    let snapshot = client.snapshot(request).await?;
    let offer = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .next()
        .ok_or_else(|| AppClientError::Refused("the directory snapshot has no card".into()))?;
    let card: PortableCardV1 = serde_json::from_slice(
        &client
            .resource(snapshot.session.clone(), offer.resource)
            .await?,
    )?;
    let resource = *card.media.first().ok_or_else(|| {
        AppClientError::Refused("the directory card has no typed resource".into())
    })?;
    Ok(serde_json::from_slice(
        &client.resource(snapshot.session, resource).await?,
    )?)
}

/// The route's endpoint. A snapshot reads the directory once and keeps its
/// bytes, so the resources it names stay servable while paths move.
pub struct DeviceDirectoryEndpoint {
    source: DeviceDirectorySource,
    current: Option<(Vec<u8>, Vec<u8>)>,
}

impl DeviceDirectoryEndpoint {
    pub fn new(source: DeviceDirectorySource) -> Self {
        Self {
            source,
            current: None,
        }
    }

    pub fn register(
        source: DeviceDirectorySource,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        catalog.register(DEVICE_DIRECTORY_ROUTE, "Paired devices", move |_| {
            Ok(Self::new(source.clone()))
        })?;
        Ok(
            ResidentEndpointRoute::new(DEVICE_DIRECTORY_ROUTE, Duration::from_millis(50))
                .expect("device-directory route is valid"),
        )
    }

    fn session() -> ProjectionSession {
        ProjectionSession(SESSION.into())
    }

    fn request() -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: Self::session(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }

    /// Read now, and keep the directory and card bytes for resource requests.
    fn refresh(&mut self) -> Result<Vec<u8>, String> {
        let source = self.source.clone();
        let directory = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(source.read())
        })?;
        let directory_bytes = serde_json::to_vec(&directory).map_err(|error| error.to_string())?;
        let card = PortableCardV1 {
            title: "Paired devices".into(),
            values: directory.devices.iter().map(card_value).collect(),
            badges: vec![format!(
                "{}/{} connected",
                directory
                    .devices
                    .iter()
                    .filter(|device| device.connected)
                    .count(),
                directory.devices.len()
            )],
            media: vec![ContentHash::of(&directory_bytes)],
        };
        let card_bytes = serde_json::to_vec(&card).map_err(|error| error.to_string())?;
        self.current = Some((directory_bytes, card_bytes.clone()));
        Ok(card_bytes)
    }
}

fn card_value(device: &PairedDeviceV1) -> CardValueV1 {
    let live: Vec<&str> = device
        .path
        .iter()
        .filter(|addr| addr.active)
        .map(|addr| addr.addr.as_str())
        .collect();
    CardValueV1 {
        label: if device.label.is_empty() {
            device.node_id.clone()
        } else {
            device.label.clone()
        },
        value: if device.connected {
            format!("connected via {}", live.join(", "))
        } else {
            "not connected".into()
        },
    }
}

impl ProjectionCatalog for DeviceDirectoryEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Djinn paired devices".into(),
            projections: vec![ProjectionOffer {
                label: "Paired-device directory".into(),
                request: Self::request(),
            }],
        }
    }
}

impl ProjectionSource for DeviceDirectoryEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        if request.session != Self::session() {
            return Err("device-directory snapshot names another session".into());
        }
        let card_bytes = self.refresh()?;
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("djinn.device-directory", "devices"));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::IDENTITY,
            footprint: Footprint::Rect {
                size: Size2::new(1.0, 1.0),
            },
            representation: Representation::Card,
            layer: 0,
            visible: false,
            hit: None,
            channels: Vec::new(),
        });
        let key = PresentationKey(RESOURCE_LABEL.into());
        let mut presentation = PresentationManifest::default();
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: ContentHash::of(&card_bytes),
                byte_size: card_bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "Paired devices".into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FitWithinFootprint,
                    actions: Vec::new(),
                },
            }],
        );
        Ok(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: Self::session(),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene)
                .map_err(|error| format!("{error:?}"))?,
            presentation,
            cache_policy: CachePolicy::default(),
        })
    }
}

impl PresentationSource for DeviceDirectoryEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        if request.session != Self::session() {
            return Err("device-directory resource names another session".into());
        }
        let Some((directory, card)) = self.current.as_ref() else {
            return Err("request a device-directory snapshot first".into());
        };
        let bytes = if request.resource == ContentHash::of(card) {
            card.clone()
        } else if request.resource == ContentHash::of(directory) {
            directory.clone()
        } else {
            return Err("that resource is not in the current device-directory snapshot".into());
        };
        Ok(ResourceResponse {
            session: Self::session(),
            resource: request.resource,
            bytes,
        })
    }
}

impl IntentSink for DeviceDirectoryEndpoint {
    type Error = String;

    fn invoke(&mut self, _: IntentInvocation) -> Result<IntentResult, Self::Error> {
        Ok(IntentResult::Rejected {
            reason: "the device directory is read-only".into(),
        })
    }
}

#[cfg(test)]
mod tests;
