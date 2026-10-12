// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Native owner composition. The host serializes this with configuration and
//! authority ingestion; the UI supplies intentions, never grants or key material.
use super::*;
use eidetic::retention::{self, Limits};
use eidetic::seal::PayloadSealer;
use eidetic::{BlobManifest, BlobSource, Hash, ManifestId, Store};
use gemot::moot::hosting::{DeviceCeiling, HostingPolicy, HostingRequest};
use gemot::moot::{CollectionId, Moot, MootAuthorizationRequest};
use mien::GateDecision;
use muniment::Backend;
use personae::IdentityProvider;
use servitor::{Cap, cap_path};

#[derive(Clone, Debug)]
pub struct Config {
    /// Exact payloads explicitly approved by the supplying content owner for
    /// public export. A manifest's unsigned privacy tag is not this permission.
    pub public_export_manifests: BTreeSet<[u8; 32]>,
    /// Actual configured owner powers, not inferred from membership or a UI offer.
    pub read_cap: Cap,
    pub retain_cap: Cap,
    pub retention_revision: [u8; 32],
    pub retention_limits: Limits,
    pub hosting: HostingPolicy,
    pub device_ceiling: DeviceCeiling,
    pub device: [u8; 32],
    pub requested_host_bytes: u64,
    pub requested_duration_ms: u64,
}
/// Actual application owners and wallet keys are supplied explicitly. Local
/// copies and hosting facts live in their existing stores, never in Pane.
pub struct Owner<'a, B, P: ?Sized> {
    pub moot: &'a Moot<B>,
    pub identity: &'a P,
    pub encounter: &'a mut dyn Store,
    pub custody: &'a mut dyn retention::CustodyStore,
    pub sealer: Option<&'a dyn PayloadSealer>,
    pub config: Config,
    pub selected_collection: Option<CollectionId>,
}
impl<B: Backend + Clone, P: IdentityProvider + ?Sized> Owner<'_, B, P> {
    fn reader(&self) -> [u8; 32] {
        self.identity.master_public_key().to_bytes()
    }
    async fn allowed(&self, cap: &Cap, at_ms: u64) -> Result<(), String> {
        let group = self.moot.membership().await.map_err(|e| e.to_string())?;
        match self
            .moot
            .authorize_current_capability(
                &group,
                &MootAuthorizationRequest {
                    subject: self.reader(),
                    capability_path: cap_path(cap),
                    at_ms,
                },
            )
            .await
            .map_err(|e| e.to_string())?
        {
            GateDecision::Allow => Ok(()),
            GateDecision::Deny(reason) => {
                Err(format!("Current owner permission refused: {reason:?}"))
            },
        }
    }
    fn id(reference: ResourceRef) -> ManifestId {
        ManifestId::from_hash(
            Hash::parse(&format!("blake3:{}", hex(reference.manifest))).expect("digest"),
        )
    }
    fn terms(&self, action: ResourceAction) -> ActionTerms {
        let description = match action {
            ResourceAction::RetainCopy => format!(
                "Retain in this profile: up to {} payload bytes, {} stored bytes total. Private copies require wallet sealing.",
                self.config.retention_limits.max_payload_bytes,
                self.config.retention_limits.max_stored_bytes
            ),
            ResourceAction::OfferHosting => format!(
                "Offer to {} for up to {} ms and {} bytes under community/device ceilings. Retains a local copy first; a promise is not a retrieval observation.",
                self.config.hosting.audience,
                self.config
                    .requested_duration_ms
                    .min(self.config.hosting.max_duration_ms)
                    .min(self.config.device_ceiling.max_duration_ms),
                self.config
                    .requested_host_bytes
                    .min(self.config.hosting.max_bytes)
                    .min(self.config.device_ceiling.max_bytes)
            ),
            _ => "Read the exact permitted content locally".into(),
        };
        // The complete local configuration is bound, including powers, policy
        // revisions and ceilings. Changing settings requires another preparation.
        let revision = *Hash::of(
            format!(
                "resource-owner-terms/v1:{:?}:{action:?}:{}",
                self.config,
                hex(self.reader())
            )
            .as_bytes(),
        )
        .as_bytes();
        ActionTerms {
            revision,
            description,
        }
    }
    async fn exact(&self, reference: ResourceRef, at_ms: u64) -> Result<(), String> {
        if reference.moot != self.moot.moot_id().0 {
            return Err("Foreign Moot resource".into());
        }
        self.allowed(&self.config.read_cap, at_ms).await?;
        if !self
            .moot
            .authorized_fauna(at_ms)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .any(|f| f.op_hash == reference.contribution && f.manifest_id == reference.manifest)
        {
            return Err("Exact resource withdrawn or no longer authorized".into());
        }
        if let Some(id) = self.selected_collection {
            let collection = self
                .moot
                .authorized_collection(id, at_ms)
                .await
                .map_err(|e| e.to_string())?
                .ok_or("Selected collection unavailable")?;
            if !collection
                .effective_selected
                .iter()
                .any(|c| c.moot_id == reference.moot && c.share == reference.contribution)
            {
                return Err("Resource left the selected collection".into());
            }
        }
        Ok(())
    }
    async fn content(&mut self, reference: ResourceRef) -> Result<(BlobManifest, Vec<u8>), String> {
        let reader = self.reader();
        let id = Self::id(reference);
        if let Some((_, m, bytes)) = retention::read(self.custody, reader, id, self.sealer)
            .await
            .map_err(|e| e.to_string())?
        {
            // Owned copies keep their local sealing marker. They can be opened
            // or served directly; a duplicate retention uses the encounter form.
            return Ok((m, bytes));
        }
        let mut m = eidetic::manifest::load_manifest(self.encounter, id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Exact manifest unavailable locally")?;
        if m.id != id
            || m.content_hash != id.0
            || m.byte_size > self.config.retention_limits.max_payload_bytes
        {
            return Err("Exact content identity or read budget refused".into());
        }
        m.sources
            .retain(|s| matches!(s, BlobSource::Local { .. } | BlobSource::Embedded { .. }));
        if eidetic::seal::seal_marker(&m)
            .map_err(|e| e.to_string())?
            .is_some()
            && !eidetic::seal::is_private_lane(m.privacy)
        {
            return Err("Source privacy/seal metadata is inconsistent".into());
        }
        let bytes = eidetic::seal::resolve_sealed_blob(
            self.encounter,
            &mut eidetic::NoFetcher,
            self.sealer,
            &m,
        )
        .await
        .map_err(|e| e.to_string())?;
        if Hash::of(&bytes) != id.0 || bytes.len() as u64 != m.byte_size {
            return Err("Exact content verification refused".into());
        }
        Ok((m, bytes))
    }
    fn hosting_request(&self, reference: ResourceRef, at_ms: u64) -> HostingRequest {
        HostingRequest {
            contribution: reference.contribution,
            manifest: reference.manifest,
            device: self.config.device,
            audience: self.config.hosting.audience.clone(),
            max_bytes: self.config.requested_host_bytes,
            duration_ms: self.config.requested_duration_ms,
            at_ms,
        }
    }
    pub async fn read(&mut self, at_ms: u64) -> Result<Snapshot, String> {
        let reader = self.reader();
        self.allowed(&self.config.read_cap, at_ms).await?;
        let mut collection = if let Some(id) = self.selected_collection {
            let view = self
                .moot
                .authorized_collection(id, at_ms)
                .await
                .map_err(|e| e.to_string())?
                .ok_or("Selected collection unavailable")?;
            Some((
                CollectionRead {
                    id: id.0,
                    name: view.name,
                    heads: view.version.frontier,
                    membership_commitment: view.version.membership_commitment,
                    fork: Some(Offer::Unsupported {
                        reason: "Collection fork handler is not installed".into(),
                    }),
                },
                view.effective_selected,
            ))
        } else {
            None
        };
        let hosting = self
            .moot
            .hosting_records()
            .await
            .map_err(|e| e.to_string())?;
        let mut resources = vec![];
        for f in self
            .moot
            .authorized_fauna(at_ms)
            .await
            .map_err(|e| e.to_string())?
        {
            let reference = ResourceRef {
                moot: self.moot.moot_id().0,
                contribution: f.op_hash,
                manifest: f.manifest_id,
            };
            if collection.as_ref().is_some_and(|(_, members)| {
                !members
                    .iter()
                    .any(|c| c.moot_id == reference.moot && c.share == reference.contribution)
            }) {
                continue;
            }
            let held =
                retention::read(self.custody, reader, Self::id(reference), self.sealer).await;
            let retention = match &held {
                Ok(Some((copy, _, _))) => RetentionRead::Held {
                    owner_receipt: format!(
                        "profile {} / manifest {}",
                        hex(copy.owner),
                        hex(reference.manifest)
                    ),
                    retained_at_ms: copy.retained_at_ms,
                },
                Ok(None) => RetentionRead::Unknown,
                Err(reason) => RetentionRead::Unavailable {
                    reason: reason.to_string(),
                },
            };
            let (content, content_bytes, public_content) = match self.content(reference).await {
                Ok((m, bytes)) => (
                    ContentRead::Resolved,
                    bytes.len() as u64,
                    m.privacy == eidetic::PrivacyClass::PublicPortable,
                ),
                Err(reason) => (ContentRead::Unavailable { reason }, 0, false),
            };
            let mut offers = BTreeMap::new();
            offers.insert(
                ResourceAction::Open,
                if content == ContentRead::Resolved {
                    Offer::Offered
                } else {
                    Offer::Withheld {
                        reason: "Content is not verified locally".into(),
                    }
                },
            );
            let copy_allowed = self.allowed(&self.config.retain_cap, at_ms).await;
            offers.insert(
                ResourceAction::RetainCopy,
                match &copy_allowed {
                    Ok(()) if content == ContentRead::Resolved => Offer::Offered,
                    Ok(()) => Offer::Withheld {
                        reason: "Content is not verified locally".into(),
                    },
                    Err(reason) => Offer::Withheld {
                        reason: reason.clone(),
                    },
                },
            );
            let host_allowed = if copy_allowed.is_ok()
                && public_content
                && self
                    .config
                    .public_export_manifests
                    .contains(&reference.manifest)
            {
                self.moot
                    .prepare_hosting(
                        self.reader(),
                        &self.hosting_request(reference, at_ms),
                        content_bytes,
                        &self.config.hosting,
                        self.config.device_ceiling,
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            } else {
                Err("This hosting adapter requires public-portable content and explicit copy permission".into())
            };
            offers.insert(
                ResourceAction::OfferHosting,
                match host_allowed {
                    Ok(()) if content == ContentRead::Resolved => Offer::Offered,
                    Ok(()) => Offer::Withheld {
                        reason: "Content is not verified locally".into(),
                    },
                    Err(reason) => Offer::Withheld { reason },
                },
            );
            offers.insert(
                ResourceAction::ForkContent,
                Offer::Unsupported {
                    reason: "Content fork handler is not installed".into(),
                },
            );
            let hosting_read = hosting
                .iter()
                .rev()
                .find(|r| {
                    r.promise.host == self.reader()
                        && r.promise.device == self.config.device
                        && r.promise.contribution == reference.contribution
                        && r.promise.manifest == reference.manifest
                })
                .map(|r| {
                    if at_ms >= r.promise.expires_at_ms {
                        HostingRead::Expired {
                            owner_record: hex(r.operation),
                        }
                    } else {
                        HostingRead::Promise {
                            owner_record: format!(
                                "{} / {} / until {}",
                                hex(r.operation),
                                r.promise.audience,
                                r.promise.expires_at_ms
                            ),
                        }
                    }
                })
                .unwrap_or(HostingRead::Unknown);
            let terms = [
                ResourceAction::Open,
                ResourceAction::RetainCopy,
                ResourceAction::OfferHosting,
            ]
            .into_iter()
            .map(|a| (a, self.terms(a)))
            .collect();
            resources.push(ResourceRead {
                reference,
                title: f.title,
                schema: f.schema_id,
                content,
                retention,
                hosting: hosting_read,
                terms,
                offers,
            });
        }
        Ok(Snapshot {
            moot: self.moot.moot_id().0,
            reader: self.reader(),
            collection: collection.take().map(|(c, _)| c),
            resources,
            activities: vec![],
        })
    }
    /// The successful Open body is transient. Writes return receipts only and
    /// require a subsequent owner read to disclose the resulting custody/promise.
    pub async fn dispatch(&mut self, request: &Request, at_ms: u64) -> (Outcome, Option<Vec<u8>>) {
        let result = self.execute(request, at_ms).await;
        match result {
            Ok((receipt, body)) => (
                Outcome::Accepted {
                    owner_receipt: receipt,
                },
                body,
            ),
            Err(reason) => (Outcome::Refused { reason }, None),
        }
    }
    async fn execute(
        &mut self,
        request: &Request,
        at_ms: u64,
    ) -> Result<(String, Option<Vec<u8>>), String> {
        let reader = self.reader();
        if request.reader != self.reader() {
            return Err("Request reader is not this signing identity".into());
        }
        let current = self.read(at_ms).await?;
        validate_request(request, &current).map_err(|e| e.to_string())?;
        let Target::Resource {
            reference, action, ..
        } = &request.target
        else {
            return Err("This owner handles resource operations only".into());
        };
        self.exact(*reference, at_ms).await?;
        let (mut manifest, bytes) = self.content(*reference).await?;
        if *action == ResourceAction::Open {
            return Ok(("Verified exact local resource".into(), Some(bytes)));
        }
        if !matches!(
            action,
            ResourceAction::RetainCopy | ResourceAction::OfferHosting
        ) {
            return Err("Resource action unsupported".into());
        }
        if *action == ResourceAction::OfferHosting
            && (manifest.privacy != eidetic::PrivacyClass::PublicPortable
                || !self
                    .config
                    .public_export_manifests
                    .contains(&reference.manifest))
        {
            return Err("This hosting adapter serves public-portable content only".into());
        }
        self.allowed(&self.config.retain_cap, at_ms).await?;
        let host_request = self.hosting_request(*reference, at_ms);
        let prepared = if *action == ResourceAction::OfferHosting {
            Some(
                self.moot
                    .prepare_hosting(
                        self.reader(),
                        &host_request,
                        bytes.len() as u64,
                        &self.config.hosting,
                        self.config.device_ceiling,
                    )
                    .await
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        // The source's at-rest marker describes its source key. The destination
        // is sealed independently through the same actual wallet owner.
        if let serde_json::Value::Object(metadata) = &mut manifest.schema_metadata {
            metadata.remove(eidetic::seal::SEAL_MARKER_KEY);
        }
        let copy = retention::retain(
            self.custody,
            reader,
            &manifest,
            &bytes,
            self.config.retention_limits,
            self.config.retention_revision,
            at_ms,
            self.sealer,
        )
        .await
        .map_err(|e| e.to_string())?;
        if let Some(prepared) = prepared {
            let receipt = self
                .moot
                .commit_hosting(
                    self.identity,
                    &host_request,
                    bytes.len() as u64,
                    &self.config.hosting,
                    self.config.device_ceiling,
                    &prepared,
                )
                .await
                .map_err(|e| {
                    format!(
                        "Copy retained at {}; hosting command failed; refresh owner records before retry: {e}",
                        copy.retained_at_ms
                    )
                })?;
            Ok((
                format!("Signed hosting promise {}", hex(receipt.operation)),
                None,
            ))
        } else {
            Ok((
                format!("Verified copy retained at {}", copy.retained_at_ms),
                None,
            ))
        }
    }
    pub async fn serve(
        &mut self,
        reference: ResourceRef,
        audience: &str,
        at_ms: u64,
    ) -> Result<Vec<u8>, String> {
        let reader = self.reader();
        let record = self
            .moot
            .hosting_records()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .rev()
            .find(|r| {
                r.promise.host == self.reader()
                    && r.promise.device == self.config.device
                    && r.promise.contribution == reference.contribution
                    && r.promise.manifest == reference.manifest
            })
            .ok_or("No signed hosting promise for this device")?;
        let (_, manifest, bytes) =
            retention::read(self.custody, reader, Self::id(reference), self.sealer)
                .await
                .map_err(|e| e.to_string())?
                .ok_or("Hosted bytes are not held locally")?;
        if reference.moot != self.moot.moot_id().0
            || manifest.privacy != eidetic::PrivacyClass::PublicPortable
            || !self
                .config
                .public_export_manifests
                .contains(&reference.manifest)
        {
            return Err("This hosting adapter serves exact public-portable content only".into());
        }
        self.moot
            .check_hosting(
                &record,
                audience,
                &self.config.hosting,
                self.config.device_ceiling,
                bytes.len() as u64,
                at_ms,
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(bytes)
    }
}
fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
