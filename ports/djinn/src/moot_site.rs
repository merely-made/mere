// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit local Moot publication endpoint. Retention and authority owners are
//! supplied by the host; requests never open editor files or acquire peer data.

use crate::resident_site::{
    GeminiListenerPolicyV1, RunningGemini, identity_fingerprint, load_json,
    persistent_gemini_identity, save_json_atomic, start_gemini_handler,
};
use errand::{
    Scheme,
    serve::{Item, Source, SourceRequest},
};
use gemot::moot::{
    AvailabilityPolicy, ErasurePolicy, Moot, MootFile, MootId, MootRetentionSettings,
    PolicyRevision,
};
use muniment::Backend;
use serde::{Deserialize, Serialize};
use site_model::{
    SiteFormat,
    publication::{SiteBindingV1, open_binding},
};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

const DESCRIPTOR: &str = "moot-site.json";
const MAX_BINDING_BYTES: usize = 16 * 1024;

/// Explicit local owner locations and the governed retention settings supplied
/// by the embedding host. The founder is recovered from signed Moot genesis.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MootSiteOwnersV1 {
    pub version: u8,
    pub moot_id: [u8; 32],
    pub moot_directory: PathBuf,
    pub content_database: PathBuf,
    pub retention_revision: PolicyRevision,
    pub availability: AvailabilityPolicy,
    pub erasure: ErasurePolicy,
}

impl MootSiteOwnersV1 {
    pub fn retention(&self) -> MootRetentionSettings {
        MootRetentionSettings {
            revision: self.retention_revision.clone(),
            availability: self.availability,
            erasure: self.erasure,
        }
    }
}

pub type DiskMootSiteService = MootSiteService<muniment::RedbBackend, muniment::RedbBackend>;

pub async fn open_owned(
    root: &Path,
    owners: &MootSiteOwnersV1,
) -> Result<DiskMootSiteService, String> {
    if owners.version != 1 {
        return Err("Unsupported Moot site owner version".into());
    }
    let moot = MootFile::open_existing(
        &owners.moot_directory,
        MootId(owners.moot_id),
        owners.retention(),
    )
    .await
    .map_err(|e| e.to_string())?;
    let content =
        muniment::RedbBackend::open(&owners.content_database).map_err(|e| e.to_string())?;
    MootSiteService::open(root, Arc::new(moot), content).await
}

fn now_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "Publication clock overflow".into())
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    version: u8,
    binding: SiteBindingV1,
    policy: GeminiListenerPolicyV1,
    certificate_sha256: [u8; 32],
    running: bool,
}

struct NativeSource<M, C> {
    moot: Arc<Moot<M>>,
    content: C,
    binding: SiteBindingV1,
    refusal: Mutex<Option<String>>,
}

impl<M, C> NativeSource<M, C>
where
    M: Backend + Clone,
    C: Backend + Clone,
{
    async fn read(&self, request: &SourceRequest) -> Result<Option<Item>, String> {
        if request.scheme != Scheme::Gemini || request.query.is_some() {
            return Ok(None);
        }
        let site = open_binding(
            &self.moot,
            &mut self.content.clone(),
            &self.binding,
            now_ms()?,
        )
        .await
        .map_err(|e| e.to_string())?;
        if site.snapshot().format != SiteFormat::Gemini {
            return Err("This resident endpoint requires a Gemini site".into());
        }
        Ok(site.page(&request.path).map(|page| {
            let mut mime = "text/gemini;charset=utf-8".to_string();
            if !page.metadata.language.is_empty() {
                mime.push_str(";lang=");
                mime.push_str(&page.metadata.language);
            }
            Item::Document {
                mime,
                body: page.source.clone(),
            }
        }))
    }
}

impl<M, C> Source for NativeSource<M, C>
where
    M: Backend + Clone + Send + Sync + 'static,
    C: Backend + Clone + Send + Sync + 'static,
{
    fn get(
        &self,
        request: &SourceRequest,
    ) -> impl std::future::Future<Output = Option<Item>> + Send {
        // Retained native backends perform synchronous local I/O. Drive this
        // non-Send owner read to completion before returning a Send ready future;
        // no detached blocking work can outlive the resident task scope. Network
        // deadlines cannot preempt local disk I/O or bounded JSON decoding.
        let result = pollster::block_on(self.read(request));
        let mut refusal = self.refusal.lock().expect("site refusal mutex poisoned");
        let item = match result {
            Ok(item) => {
                *refusal = None;
                item
            },
            Err(reason) => {
                *refusal = Some(reason);
                None
            },
        };
        std::future::ready(item)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MootSiteStatus {
    pub binding: Option<SiteBindingV1>,
    pub address: Option<SocketAddr>,
    pub certificate_sha256: Option<[u8; 32]>,
    pub last_refusal: Option<String>,
}

/// One endpoint per supplied root. The host owns exclusive configuration;
/// clones of the retained owners remain live for per-request revalidation.
pub struct MootSiteService<M, C> {
    root: PathBuf,
    moot: Arc<Moot<M>>,
    content: C,
    descriptor: Option<Descriptor>,
    source: Option<Arc<NativeSource<M, C>>>,
    running: Option<RunningGemini>,
    startup_refusal: Option<String>,
}

impl<M, C> MootSiteService<M, C>
where
    M: Backend + Clone + Send + Sync + 'static,
    C: Backend + Clone + Send + Sync + 'static,
{
    pub async fn open(root: &Path, moot: Arc<Moot<M>>, content: C) -> Result<Self, String> {
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let descriptor = if root.join(DESCRIPTOR).exists() {
            let descriptor = load_json::<Descriptor>(&root.join(DESCRIPTOR), MAX_BINDING_BYTES)?;
            if descriptor.version != 1 {
                return Err("Unsupported Moot endpoint version".into());
            }
            descriptor.binding.validate().map_err(|e| e.to_string())?;
            descriptor.policy.validate()?;
            persistent_gemini_identity(root, Some(descriptor.certificate_sha256))?;
            Some(descriptor)
        } else {
            None
        };
        let resume = descriptor
            .as_ref()
            .is_some_and(|d| d.running && d.policy.resume_on_restart);
        let mut service = Self {
            root: root.into(),
            moot,
            content,
            descriptor,
            source: None,
            running: None,
            startup_refusal: None,
        };
        if resume && let Err(error) = service.start().await {
            service.startup_refusal = Some(error);
        }
        Ok(service)
    }

    /// Install only while stopped. This explicit operation conveys the host's
    /// intent to expose the chosen public selection; it is not a hosting promise.
    pub async fn configure(
        &mut self,
        binding: SiteBindingV1,
        policy: GeminiListenerPolicyV1,
    ) -> Result<(), String> {
        if self.running.is_some() {
            return Err("Stop the endpoint before changing its binding".into());
        }
        policy.validate()?;
        let site = open_binding(&self.moot, &mut self.content.clone(), &binding, now_ms()?)
            .await
            .map_err(|e| e.to_string())?;
        if site.snapshot().format != SiteFormat::Gemini {
            return Err("This endpoint requires a Gemini site".into());
        }
        let identity = persistent_gemini_identity(
            &self.root,
            self.descriptor.as_ref().map(|d| d.certificate_sha256),
        )?;
        let next = Descriptor {
            version: 1,
            binding,
            policy,
            certificate_sha256: identity_fingerprint(&identity)?,
            running: false,
        };
        save_json_atomic(&self.root.join(DESCRIPTOR), &next)?;
        self.descriptor = Some(next);
        self.startup_refusal = None;
        Ok(())
    }

    pub async fn start(&mut self) -> Result<MootSiteStatus, String> {
        if self.running.is_some() {
            return Ok(self.status());
        }
        let mut next = self
            .descriptor
            .clone()
            .ok_or("Configure a site binding first")?;
        let source = Arc::new(NativeSource {
            moot: self.moot.clone(),
            content: self.content.clone(),
            binding: next.binding.clone(),
            refusal: Mutex::new(None),
        });
        source
            .read(&SourceRequest {
                scheme: Scheme::Gemini,
                path: "/".into(),
                query: None,
            })
            .await?;
        let identity = persistent_gemini_identity(&self.root, Some(next.certificate_sha256))?;
        let handler_source = source.clone();
        let runtime = start_gemini_handler(
            move |_| errand::serve::adapt::gemini(handler_source),
            &next.policy,
            &identity,
        )?;
        next.running = true;
        save_json_atomic(&self.root.join(DESCRIPTOR), &next)?;
        runtime.activate();
        self.descriptor = Some(next);
        self.source = Some(source);
        self.running = Some(runtime);
        self.startup_refusal = None;
        Ok(self.status())
    }

    pub fn status(&self) -> MootSiteStatus {
        MootSiteStatus {
            binding: self.descriptor.as_ref().map(|d| d.binding.clone()),
            address: self.running.as_ref().map(|r| r.address),
            certificate_sha256: self.descriptor.as_ref().map(|d| d.certificate_sha256),
            last_refusal: self
                .source
                .as_ref()
                .and_then(|s| s.refusal.lock().ok()?.clone())
                .or_else(|| self.startup_refusal.clone()),
        }
    }

    /// Explicit local handoff. Gemot admits signed native-drop records first;
    /// only an effective public site contribution may authorize retaining the
    /// supplied snapshot bytes. Reads never invoke this acquisition operation.
    pub async fn import_public<R: std::io::Read>(
        &mut self,
        records: R,
        snapshot: Option<site_model::PublishedSnapshotV1>,
    ) -> Result<(), String> {
        self.moot
            .import_plain_drop(records, stickleback::DropLimits::default())
            .await
            .map_err(|e| e.to_string())?;
        if let Some(snapshot) = snapshot {
            if snapshot.format != SiteFormat::Gemini {
                return Err("This endpoint requires a Gemini site".into());
            }
            let bytes = snapshot.encode()?;
            let manifest = eidetic::ManifestId::of_blob(&bytes);
            if !self
                .moot
                .authorized_fauna(now_ms()?)
                .await
                .map_err(|e| e.to_string())?
                .iter()
                .any(|share| {
                    share.manifest_id == *manifest.0.as_bytes()
                        && share.schema_id == site_model::publication::SNAPSHOT_SCHEMA_ID
                })
            {
                return Err("No effective site contribution names these bytes".into());
            }
            let store = &mut self.content.clone();
            eidetic::schema_def::bootstrap_meta_schema(store)
                .await
                .map_err(|e| e.to_string())?;
            eidetic::schema_def::save_schema(
                store,
                &site_model::publication::snapshot_schema(),
                eidetic::PrivacyClass::PublicPortable,
                eidetic::ProvenanceRecord::self_imported("mere-site schema"),
                eidetic::TrustEnvelope::self_asserted(),
                eidetic::Timestamp(now_ms()?),
            )
            .await
            .map_err(|e| e.to_string())?;
            eidetic::typed::save_typed(
                store,
                &snapshot,
                vec![],
                eidetic::PrivacyClass::PublicPortable,
                eidetic::ProvenanceRecord::self_imported("explicit signed site handoff"),
                eidetic::TrustEnvelope::self_asserted(),
                eidetic::Timestamp(now_ms()?),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        let joined = self.shutdown();
        let mut next = self.descriptor.clone().ok_or("No configured endpoint")?;
        next.running = false;
        save_json_atomic(&self.root.join(DESCRIPTOR), &next)?;
        self.descriptor = Some(next);
        joined?;
        Ok(())
    }

    /// Join listener/connection work while preserving explicit restart intent.
    pub fn shutdown(&mut self) -> Result<armillary::tasks::TaskReport, String> {
        let joined = match self.running.take() {
            Some(mut running) => running.close_and_join(),
            None => Ok(armillary::tasks::TaskReport::default()),
        };
        self.source = None;
        joined
    }
}
