// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit local Moot publication endpoint. Retention and authority owners are
//! supplied by the host; requests never open editor files or acquire peer data.

use crate::resident_site::{
    GeminiListenerPolicyV1, RunningGemini, identity_fingerprint, load_json,
    persistent_gemini_identity, save_json_atomic, start_gemini_handler, start_tls_handler,
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
    MAX_ENCODED_SNAPSHOT_BYTES, MAX_PAGE_BYTES, SiteFormat,
    publication::{OpenedSite, SiteBindingV1, open_binding},
};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[path = "micron_site.rs"]
mod micron_site;
pub use micron_site::NomadNetListenerPolicyV1;

const DESCRIPTOR: &str = "moot-site.json";
const MAX_BINDING_BYTES: usize = 16 * 1024;

/// The wire endpoint is chosen explicitly, independently of a retained profile.
/// Missing transport fields in existing V1 descriptors and commands mean Gemini.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MootSiteTransport {
    #[default]
    Gemini,
    Scroll,
    #[serde(rename = "nomadnet")]
    NomadNet,
}

impl MootSiteTransport {
    fn is_gemini(&self) -> bool {
        *self == Self::Gemini
    }

    fn format(self) -> SiteFormat {
        match self {
            Self::Gemini => SiteFormat::Gemini,
            Self::Scroll => SiteFormat::Scroll,
            Self::NomadNet => SiteFormat::Micron,
        }
    }

    fn require_format(self, format: SiteFormat) -> Result<(), String> {
        if format != self.format() {
            return Err(format!(
                "This endpoint requires a {} site",
                self.format().label()
            ));
        }
        Ok(())
    }
}

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
struct TlsDescriptor {
    version: u8,
    #[serde(default, skip_serializing_if = "MootSiteTransport::is_gemini")]
    transport: MootSiteTransport,
    binding: SiteBindingV1,
    policy: GeminiListenerPolicyV1,
    certificate_sha256: [u8; 32],
    running: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum NativeTransport {
    Nomadnet,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NomadNetDescriptor {
    version: u8,
    transport: NativeTransport,
    binding: SiteBindingV1,
    policy: NomadNetListenerPolicyV1,
    destination: [u8; 16],
    running: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Descriptor {
    NomadNet(NomadNetDescriptor),
    Tls(TlsDescriptor),
}

impl Descriptor {
    fn binding(&self) -> &SiteBindingV1 {
        match self {
            Self::Tls(d) => &d.binding,
            Self::NomadNet(d) => &d.binding,
        }
    }
    fn transport(&self) -> MootSiteTransport {
        match self {
            Self::Tls(d) => d.transport,
            Self::NomadNet(_) => MootSiteTransport::NomadNet,
        }
    }
    fn running(&self) -> bool {
        match self {
            Self::Tls(d) => d.running,
            Self::NomadNet(d) => d.running,
        }
    }
    fn set_running(&mut self, running: bool) {
        match self {
            Self::Tls(d) => d.running = running,
            Self::NomadNet(d) => d.running = running,
        }
    }
    fn resume(&self) -> bool {
        match self {
            Self::Tls(d) => d.policy.resume_on_restart,
            Self::NomadNet(d) => d.policy.resume_on_restart,
        }
    }
    fn certificate(&self) -> Option<[u8; 32]> {
        match self {
            Self::Tls(d) => Some(d.certificate_sha256),
            Self::NomadNet(_) => None,
        }
    }
    fn destination(&self) -> Option<[u8; 16]> {
        match self {
            Self::Tls(_) => None,
            Self::NomadNet(d) => Some(d.destination),
        }
    }
}

enum RunningNative {
    Tls(RunningGemini),
    #[cfg(feature = "moot-site-nomadnet")]
    NomadNet(micron_site::RunningNomadNet),
}
impl RunningNative {
    fn activate(&self) {
        match self {
            Self::Tls(r) => r.activate(),
            #[cfg(feature = "moot-site-nomadnet")]
            Self::NomadNet(r) => r.activate(),
        }
    }
    fn address(&self) -> SocketAddr {
        match self {
            Self::Tls(r) => r.address,
            #[cfg(feature = "moot-site-nomadnet")]
            Self::NomadNet(r) => r.address,
        }
    }
    fn close_and_join(&mut self) -> Result<armillary::tasks::TaskReport, String> {
        match self {
            Self::Tls(r) => r.close_and_join(),
            #[cfg(feature = "moot-site-nomadnet")]
            Self::NomadNet(r) => r.close_and_join(),
        }
    }
}

struct NativeSource<M, C> {
    moot: Arc<Moot<M>>,
    content: C,
    binding: SiteBindingV1,
    transport: MootSiteTransport,
    refusal: Mutex<Option<String>>,
}

impl<M, C> NativeSource<M, C>
where
    M: Backend + Clone,
    C: Backend + Clone,
{
    async fn read(&self, request: &SourceRequest) -> Result<Option<Item>, String> {
        if self.transport != MootSiteTransport::Gemini
            || request.scheme != Scheme::Gemini
            || request.query.is_some()
        {
            return Ok(None);
        }
        let site = self.open().await?;
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

    async fn open(&self) -> Result<OpenedSite, String> {
        let site = open_binding(
            &self.moot,
            &mut self.content.clone(),
            &self.binding,
            now_ms()?,
        )
        .await
        .map_err(|e| e.to_string())?;
        self.transport.require_format(site.snapshot().format)?;
        Ok(site)
    }

    async fn read_scroll(
        &self,
        request: &scroll_protocol::Request,
    ) -> Result<scroll_protocol::Reply, String> {
        if self.transport != MootSiteTransport::Scroll {
            return Err("This source is not a Scroll endpoint".into());
        }
        let site = self.open().await?;
        let uri = url::Url::parse(&request.uri).map_err(|error| error.to_string())?;
        // Snapshot V1 declares one language per path. As in Knot's native
        // publisher, preferences do not select a variant or rewrite its tag.
        Ok(match site.page(uri.path()) {
            Some(page) => {
                let mut mime = "text/scroll;charset=utf-8".to_string();
                if !page.metadata.language.is_empty() {
                    mime.push_str(";lang=");
                    mime.push_str(&page.metadata.language);
                }
                scroll_protocol::Reply {
                    header: scroll_protocol::Header::Success(scroll_protocol::SuccessHeader {
                        code: 20 + page.metadata.classification,
                        mimetype: mime,
                        author: Some(page.metadata.author.clone()),
                        published: Some(page.metadata.published.clone()),
                        modified: Some(page.metadata.modified.clone()),
                    }),
                    body: if request.metadata {
                        page.metadata.abstract_source.as_bytes().to_vec()
                    } else {
                        page.source.clone()
                    },
                }
            },
            None => scroll_status(51, "Not published"),
        })
    }

    #[cfg(feature = "moot-site-nomadnet")]
    fn nomadnet_reply(&self, request: &retinue::request::Request) -> Vec<u8> {
        let result = (|| {
            if !request.data.is_empty() {
                return Err("Static Micron pages reject submission data".to_string());
            }
            if self.transport != MootSiteTransport::NomadNet {
                return Err("This source is not a NomadNet endpoint".to_string());
            }
            let site = pollster::block_on(self.open())?;
            site.snapshot()
                .pages
                .iter()
                .find_map(|(path, page)| {
                    (retinue::nomadnet::page_path_hash(format!("/page{path}").as_bytes())
                        == request.path_hash)
                        .then(|| page.source.clone())
                })
                .ok_or_else(|| "Micron page is not published".to_string())
        })();
        let mut refusal = self.refusal.lock().expect("site refusal mutex poisoned");
        match result {
            Ok(body) => {
                *refusal = None;
                body
            },
            Err(reason) => {
                *refusal = Some(reason);
                Vec::new()
            },
        }
    }

    fn scroll_reply(&self, request: &scroll_protocol::Request) -> scroll_protocol::Reply {
        // Same inline local-owner read boundary as the Gemini Source below.
        let result = pollster::block_on(self.read_scroll(request));
        let mut refusal = self.refusal.lock().expect("site refusal mutex poisoned");
        match result {
            Ok(reply) => {
                *refusal = None;
                reply
            },
            Err(reason) => {
                *refusal = Some(reason);
                scroll_status(51, "Not published")
            },
        }
    }
}

fn scroll_status(code: u8, meta: &str) -> scroll_protocol::Reply {
    scroll_protocol::Reply {
        header: scroll_protocol::Header::Meta {
            code,
            status: scroll_protocol::Status::from_code(code).expect("static Scroll status"),
            meta: meta.into(),
        },
        body: Vec::new(),
    }
}

fn valid_scroll_target(uri: &str, port: u16) -> bool {
    url::Url::parse(uri).is_ok_and(|url| {
        url.scheme() == "scroll"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1"))
            && url.port().unwrap_or(5699) == port
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && !url.path().contains(';')
    })
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
    #[serde(default)]
    pub transport: Option<MootSiteTransport>,
    pub address: Option<SocketAddr>,
    pub certificate_sha256: Option<[u8; 32]>,
    pub destination: Option<[u8; 16]>,
    pub public_identity: Option<Vec<u8>>,
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
    running: Option<RunningNative>,
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
            descriptor.binding().validate().map_err(|e| e.to_string())?;
            match &descriptor {
                Descriptor::Tls(d) => {
                    if d.version != 1 || d.transport == MootSiteTransport::NomadNet {
                        return Err("Unsupported TLS endpoint descriptor".into());
                    }
                    d.policy.validate()?;
                    persistent_gemini_identity(root, Some(d.certificate_sha256))?;
                },
                Descriptor::NomadNet(d) => {
                    if d.version != 1 {
                        return Err("Unsupported NomadNet endpoint version".into());
                    }
                    d.policy.validate()?;
                    #[cfg(feature = "moot-site-nomadnet")]
                    micron_site::identity(root, Some(d.destination))?;
                    #[cfg(not(feature = "moot-site-nomadnet"))]
                    return Err("NomadNet hosting requires the moot-site-nomadnet feature".into());
                },
            }
            Some(descriptor)
        } else {
            None
        };
        let resume = descriptor
            .as_ref()
            .is_some_and(|d| d.running() && d.resume());
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
        self.configure_transport(binding, policy, MootSiteTransport::Gemini)
            .await
    }

    pub async fn configure_transport(
        &mut self,
        binding: SiteBindingV1,
        policy: GeminiListenerPolicyV1,
        transport: MootSiteTransport,
    ) -> Result<(), String> {
        if self.running.is_some() {
            return Err("Stop the endpoint before changing its binding".into());
        }
        if transport == MootSiteTransport::NomadNet {
            return Err("Use configure_nomadnet with native NomadNet policy".into());
        }
        policy.validate()?;
        let site = open_binding(&self.moot, &mut self.content.clone(), &binding, now_ms()?)
            .await
            .map_err(|e| e.to_string())?;
        transport.require_format(site.snapshot().format)?;
        let identity = persistent_gemini_identity(
            &self.root,
            self.descriptor.as_ref().and_then(Descriptor::certificate),
        )?;
        let next = Descriptor::Tls(TlsDescriptor {
            version: 1,
            transport,
            binding,
            policy,
            certificate_sha256: identity_fingerprint(&identity)?,
            running: false,
        });
        save_json_atomic(&self.root.join(DESCRIPTOR), &next)?;
        self.descriptor = Some(next);
        self.startup_refusal = None;
        Ok(())
    }

    pub async fn configure_nomadnet(
        &mut self,
        binding: SiteBindingV1,
        policy: NomadNetListenerPolicyV1,
    ) -> Result<(), String> {
        #[cfg(not(feature = "moot-site-nomadnet"))]
        {
            let _ = (binding, policy);
            Err("NomadNet hosting requires the moot-site-nomadnet feature".into())
        }
        #[cfg(feature = "moot-site-nomadnet")]
        {
            if self.running.is_some() {
                return Err("Stop the endpoint before changing its binding".into());
            }
            policy.validate()?;
            let site = open_binding(&self.moot, &mut self.content.clone(), &binding, now_ms()?)
                .await
                .map_err(|e| e.to_string())?;
            MootSiteTransport::NomadNet.require_format(site.snapshot().format)?;
            let identity = micron_site::identity(
                &self.root,
                self.descriptor.as_ref().and_then(Descriptor::destination),
            )?;
            let next = Descriptor::NomadNet(NomadNetDescriptor {
                version: 1,
                transport: NativeTransport::Nomadnet,
                binding,
                policy,
                destination: micron_site::destination(&identity),
                running: false,
            });
            save_json_atomic(&self.root.join(DESCRIPTOR), &next)?;
            self.descriptor = Some(next);
            self.startup_refusal = None;
            Ok(())
        }
    }

    pub async fn start(&mut self) -> Result<MootSiteStatus, String> {
        if self.running.is_some() {
            if let Some(source) = &self.source {
                source.open().await?;
            }
            return Ok(self.status());
        }
        let mut next = self
            .descriptor
            .clone()
            .ok_or("Configure a site binding first")?;
        let source = Arc::new(NativeSource {
            moot: self.moot.clone(),
            content: self.content.clone(),
            binding: next.binding().clone(),
            transport: next.transport(),
            refusal: Mutex::new(None),
        });
        source.open().await?;
        let handler_source = source.clone();
        let runtime = match &next {
            Descriptor::NomadNet(d) => {
                #[cfg(feature = "moot-site-nomadnet")]
                {
                    let identity = micron_site::identity(&self.root, Some(d.destination))?;
                    let refusal_source = source.clone();
                    RunningNative::NomadNet(micron_site::start(
                        move |request| handler_source.nomadnet_reply(request),
                        move |reason| {
                            *refusal_source
                                .refusal
                                .lock()
                                .expect("site refusal mutex poisoned") = Some(reason.to_string())
                        },
                        &d.policy,
                        identity,
                    )?)
                }
                #[cfg(not(feature = "moot-site-nomadnet"))]
                {
                    let _ = d;
                    return Err("NomadNet hosting requires the moot-site-nomadnet feature".into());
                }
            },
            Descriptor::Tls(d) => {
                let identity = persistent_gemini_identity(&self.root, Some(d.certificate_sha256))?;
                let runtime = match d.transport {
                    MootSiteTransport::Gemini => start_gemini_handler(
                        move |_| errand::serve::adapt::gemini(handler_source),
                        &d.policy,
                        &identity,
                    )?,
                    MootSiteTransport::Scroll => {
                        let timeout = Duration::from_millis(d.policy.request_timeout_ms);
                        start_tls_handler(
                            move |port| {
                                move |mut tls, _peer, ready: Arc<std::sync::atomic::AtomicBool>| {
                                    let source = handler_source.clone();
                                    async move {
                                        let handler = move |request: scroll_protocol::Request| {
                                            let reply = if !ready.load(Ordering::Acquire) {
                                                scroll_status(44, "Publication not committed")
                                            } else if valid_scroll_target(&request.uri, port) {
                                                source.scroll_reply(&request)
                                            } else {
                                                scroll_status(59, "Invalid static-site request")
                                            };
                                            std::future::ready(reply)
                                        };
                                        scroll_protocol::server::serve_connection(
                                            &mut tls,
                                            &handler,
                                            &scroll_protocol::server::ServerConfig {
                                                max_request_bytes: 4096,
                                                response_limits: scroll_protocol::ResponseLimits {
                                                    // Existing validated snapshot bounds include even
                                                    // long valid fractional timestamp metadata.
                                                    max_header_line_bytes:
                                                        MAX_ENCODED_SNAPSHOT_BYTES,
                                                    max_body_bytes: MAX_PAGE_BYTES,
                                                },
                                                timeout,
                                            },
                                        )
                                        .await?;
                                        tokio::io::AsyncWriteExt::shutdown(&mut tls).await
                                    }
                                }
                            },
                            &d.policy,
                            &identity,
                        )?
                    },
                    MootSiteTransport::NomadNet => return Err("Use native NomadNet policy".into()),
                };
                RunningNative::Tls(runtime)
            },
        };
        next.set_running(true);
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
            binding: self.descriptor.as_ref().map(|d| d.binding().clone()),
            transport: self.descriptor.as_ref().map(Descriptor::transport),
            address: self.running.as_ref().map(RunningNative::address),
            certificate_sha256: self.descriptor.as_ref().and_then(Descriptor::certificate),
            destination: self.descriptor.as_ref().and_then(Descriptor::destination),
            public_identity: {
                #[cfg(feature = "moot-site-nomadnet")]
                {
                    self.descriptor
                        .as_ref()
                        .and_then(Descriptor::destination)
                        .and_then(|destination| {
                            micron_site::identity(&self.root, Some(destination)).ok()
                        })
                        .map(|identity| identity.public().to_public_bytes().to_vec())
                }
                #[cfg(not(feature = "moot-site-nomadnet"))]
                {
                    None
                }
            },
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
            if !matches!(
                snapshot.format,
                SiteFormat::Gemini | SiteFormat::Scroll | SiteFormat::Micron
            ) {
                return Err("This endpoint imports Gemini, Scroll or Micron sites".into());
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
        next.set_running(false);
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

#[cfg(test)]
mod tests {
    use super::*;
    use gemot::moot::{CollectionId, CollectionRef};
    use site_model::publication::SiteRevisionPolicy;

    fn descriptor() -> TlsDescriptor {
        TlsDescriptor {
            version: 1,
            transport: MootSiteTransport::Gemini,
            binding: SiteBindingV1 {
                version: 1,
                collection: CollectionRef {
                    moot_id: [1; 32],
                    collection_id: CollectionId([2; 32]),
                },
                revision: SiteRevisionPolicy::FollowCollection,
            },
            policy: GeminiListenerPolicyV1::default(),
            certificate_sha256: [9; 32],
            running: true,
        }
    }

    #[test]
    fn legacy_descriptor_defaults_to_gemini_and_retains_serialized_bytes() {
        let descriptor = descriptor();
        let legacy = format!(
            "{{\"version\":1,\"binding\":{},\"policy\":{},\"certificate_sha256\":{},\"running\":true}}",
            serde_json::to_string(&descriptor.binding).unwrap(),
            serde_json::to_string(&descriptor.policy).unwrap(),
            serde_json::to_string(&descriptor.certificate_sha256).unwrap(),
        );
        let reopened: Descriptor = serde_json::from_str(&legacy).unwrap();
        assert_eq!(reopened.transport(), MootSiteTransport::Gemini);
        assert_eq!(serde_json::to_string(&reopened).unwrap(), legacy);
    }

    #[test]
    fn scroll_descriptor_persists_transport_and_rejects_unknown_transport() {
        let mut descriptor = descriptor();
        descriptor.transport = MootSiteTransport::Scroll;
        let bytes = serde_json::to_vec(&descriptor).unwrap();
        let reopened: Descriptor = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(reopened.transport(), MootSiteTransport::Scroll);
        let mut value = serde_json::to_value(descriptor).unwrap();
        value["transport"] = serde_json::json!("gopher");
        assert!(serde_json::from_value::<Descriptor>(value).is_err());
    }

    #[test]
    fn nomadnet_descriptor_uses_native_policy_and_identity() {
        let descriptor = Descriptor::NomadNet(NomadNetDescriptor {
            version: 1,
            transport: NativeTransport::Nomadnet,
            binding: descriptor().binding,
            policy: NomadNetListenerPolicyV1::default(),
            destination: [8; 16],
            running: false,
        });
        let value = serde_json::to_value(&descriptor).unwrap();
        assert_eq!(value["transport"], "nomadnet");
        assert!(value.get("certificate_sha256").is_none());
        let reopened: Descriptor = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(reopened.destination(), Some([8; 16]));
        assert_eq!(reopened.transport(), MootSiteTransport::NomadNet);
        assert_eq!(reopened.certificate(), None);
        let mut invalid = value;
        invalid["certificate_sha256"] = serde_json::json!(vec![0; 32]);
        assert!(serde_json::from_value::<Descriptor>(invalid).is_err());
        assert!(
            MootSiteTransport::NomadNet
                .require_format(SiteFormat::Micron)
                .is_ok()
        );
        assert!(
            MootSiteTransport::NomadNet
                .require_format(SiteFormat::Gemini)
                .is_err()
        );
    }

    #[cfg(not(feature = "moot-site-nomadnet"))]
    #[tokio::test]
    async fn disabled_nomadnet_backend_refuses_configure_and_saved_descriptor() {
        use gemot::moot::KeepBound;
        use muniment::MemoryBackend;

        let directory = tempfile::tempdir().unwrap();
        let moot = Arc::new(Moot::in_memory(
            MootId([1; 32]),
            [2; 32],
            MootRetentionSettings {
                revision: serde_json::from_value(
                    serde_json::json!({ "alg":"Blake3", "bytes":vec![3u8; 32] }),
                )
                .unwrap(),
                availability: AvailabilityPolicy {
                    promised_floor: KeepBound::Forever,
                },
                erasure: ErasurePolicy {
                    history_ceiling: KeepBound::UntilCheckpoint,
                },
            },
        ));
        let mut service =
            MootSiteService::open(directory.path(), moot.clone(), MemoryBackend::new())
                .await
                .unwrap();
        assert!(
            service
                .configure_nomadnet(
                    descriptor().binding.clone(),
                    NomadNetListenerPolicyV1::default()
                )
                .await
                .unwrap_err()
                .contains("moot-site-nomadnet")
        );
        assert!(service.status().transport.is_none());
        let saved = Descriptor::NomadNet(NomadNetDescriptor {
            version: 1,
            transport: NativeTransport::Nomadnet,
            binding: descriptor().binding,
            policy: NomadNetListenerPolicyV1::default(),
            destination: [8; 16],
            running: false,
        });
        save_json_atomic(&directory.path().join(DESCRIPTOR), &saved).unwrap();
        assert!(
            MootSiteService::open(directory.path(), moot, MemoryBackend::new())
                .await
                .err()
                .unwrap()
                .contains("moot-site-nomadnet")
        );
        assert!(
            !directory
                .path()
                .join("published-site-identity.json")
                .exists()
        );
        assert!(
            !directory
                .path()
                .join("moot-site-nomadnet-identity.json")
                .exists()
        );
    }

    #[test]
    fn transport_does_not_infer_or_accept_another_profile() {
        assert!(
            MootSiteTransport::Scroll
                .require_format(SiteFormat::Scroll)
                .is_ok()
        );
        assert!(
            MootSiteTransport::Gemini
                .require_format(SiteFormat::Gemini)
                .is_ok()
        );
        for format in [SiteFormat::Gemini, SiteFormat::Spartan, SiteFormat::Micron] {
            assert!(MootSiteTransport::Scroll.require_format(format).is_err());
        }
        assert!(
            MootSiteTransport::Gemini
                .require_format(SiteFormat::Scroll)
                .is_err()
        );
    }

    #[test]
    fn scroll_target_is_loopback_and_actual_port_with_no_static_site_effects() {
        for uri in [
            "scroll://localhost:1234/",
            "scroll://127.0.0.1:1234/index.scroll",
            "scroll://localhost:1234/a%20b.scroll",
        ] {
            assert!(valid_scroll_target(uri, 1234), "{uri}");
        }
        assert!(valid_scroll_target("scroll://localhost/", 5699));
        for uri in [
            "gemini://localhost:1234/",
            "scroll://elsewhere:1234/",
            "scroll://localhost:1235/",
            "scroll://localhost/",
            "scroll://user@localhost:1234/",
            "scroll://user:password@localhost:1234/",
            "scroll://localhost:1234/?",
            "scroll://localhost:1234/?input",
            "scroll://localhost:1234/#fragment",
            "scroll://localhost:1234/a;effect.scroll",
            "invalid",
        ] {
            assert!(!valid_scroll_target(uri, 1234), "{uri}");
        }
    }

    #[tokio::test]
    async fn selected_profile_change_refuses_configure_start_and_current_read() {
        use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
        use gemot::moot::{KeepBound, MootAccessLevel, MootMember, MootMembershipAction};
        use muniment::MemoryBackend;
        use personae::{IdentityProvider, InMemoryProvider};

        let directory = tempfile::tempdir().unwrap();
        let author = InMemoryProvider::from_seed([81; 32]);
        let root = author.master_public_key().to_bytes();
        let retention_digest = vec![82u8; 32];
        let retention = MootRetentionSettings {
            revision: serde_json::from_value(serde_json::json!({
                "alg": "Blake3", "bytes": retention_digest,
            }))
            .unwrap(),
            availability: AvailabilityPolicy {
                promised_floor: KeepBound::Forever,
            },
            erasure: ErasurePolicy {
                history_ceiling: KeepBound::UntilCheckpoint,
            },
        };
        let moot = Arc::new(Moot::in_memory(MootId([83; 32]), root, retention));
        let mut rules = ConstitutionRules::founder_only(root);
        // The closed, admitted-member fixture does not depend on Standing.
        rules.admission = serde_json::from_value(serde_json::json!({
            "MembersOnly": { "rate_limit": 100, "rate_window_ms": 1000 },
        }))
        .unwrap();
        rules.grant(CapabilityGrant {
            id: [84; 32],
            subject: root,
            // Native servitor encoding of Cap::scope("moot").
            path_prefix: "scope/moot".into(),
            not_before_ms: 0,
            expires_at_ms: None,
            delegation_depth: 0,
        });
        moot.found(author.master_keypair().to_seed(), None, None, rules, 1)
            .await
            .unwrap();
        moot.update_membership_for_identity(
            &author,
            MootMembershipAction::Create {
                initial_members: vec![MootMember {
                    member: root,
                    access: MootAccessLevel::Manage,
                }],
            },
        )
        .await
        .unwrap();
        let collection_id = CollectionId([85; 32]);
        moot.declare_collection_for_identity(
            &author,
            collection_id,
            "Native profile test".into(),
            None,
            now_ms().unwrap(),
        )
        .await
        .unwrap();
        let mut store = MemoryBackend::new();
        let scroll =
            knot_site::Site::create_for(&directory.path().join("scroll"), SiteFormat::Scroll)
                .unwrap()
                .publication()
                .unwrap()
                .to_snapshot_v1()
                .unwrap();
        let version = moot
            .authorized_collection(collection_id, now_ms().unwrap())
            .await
            .unwrap()
            .unwrap()
            .version;
        let first = site_model::publication::publish_public_snapshot(
            &moot,
            &author,
            &mut store,
            &scroll,
            &version,
            None,
            "Scroll".into(),
            now_ms().unwrap(),
        )
        .await
        .unwrap();
        let binding = SiteBindingV1 {
            version: 1,
            collection: version.collection,
            revision: SiteRevisionPolicy::FollowCollection,
        };
        let mut service = MootSiteService::open(
            &directory.path().join("resident"),
            moot.clone(),
            store.clone(),
        )
        .await
        .unwrap();
        assert!(
            service
                .configure(binding.clone(), GeminiListenerPolicyV1::default())
                .await
                .unwrap_err()
                .contains("Gemini")
        );
        assert!(service.status().transport.is_none());
        service
            .configure_transport(
                binding.clone(),
                GeminiListenerPolicyV1::default(),
                MootSiteTransport::Scroll,
            )
            .await
            .unwrap();
        service.start().await.unwrap();
        let source = service.source.as_ref().unwrap().clone();
        let request = scroll_protocol::Request {
            uri: "scroll://localhost/".into(),
            languages: vec!["de".into()],
            metadata: false,
        };
        assert!(matches!(
            source.scroll_reply(&request).header,
            scroll_protocol::Header::Success(_)
        ));

        let gemini =
            knot_site::Site::create_for(&directory.path().join("gemini"), SiteFormat::Gemini)
                .unwrap()
                .publication()
                .unwrap()
                .to_snapshot_v1()
                .unwrap();
        site_model::publication::publish_public_snapshot(
            &moot,
            &author,
            &mut store,
            &gemini,
            &first.selection.collection,
            Some(first.selection.contribution),
            "Gemini replacement".into(),
            now_ms().unwrap(),
        )
        .await
        .unwrap();
        assert!(service.start().await.unwrap_err().contains("Scroll"));
        assert!(matches!(
            source.scroll_reply(&request).header,
            scroll_protocol::Header::Meta { code: 51, .. }
        ));
        assert!(service.status().last_refusal.unwrap().contains("Scroll"));
        assert_eq!(service.status().transport, Some(MootSiteTransport::Scroll));
        service.stop().unwrap();
        assert!(service.start().await.unwrap_err().contains("Scroll"));
        assert!(service.status().address.is_none());
        service
            .configure_transport(
                binding,
                GeminiListenerPolicyV1::default(),
                MootSiteTransport::Gemini,
            )
            .await
            .unwrap();
        assert_eq!(service.status().transport, Some(MootSiteTransport::Gemini));
    }
}
