// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A domain's authority as a reviewed wasm guest (cycle breaks P3, D15 (h)).
//!
//! The resident runs a graph domain's validator as a component loaded at runtime, so no
//! Mere manifest names the application that owns the domain. This module is the resident's
//! side of it: the candidate encoding, the running guest, and the owner's review and pin.
//!
//! **Stronger review than a view guest.** A D12 guest is a view: the host re-applies what
//! it returns, so a lying guest changes only its own block. An authority guest is the gate,
//! and a lying validator admits anything. So the owner's review binds the pack author's
//! key to the domain (a later pack for the same domain must come from that key, or the
//! owner must rebind it on purpose), pins the component hash, and every load re-checks
//! both. Revoking removes the component and the domain refuses as pending. The pack is the
//! D12 signed pack (`eidetic` pack signing), which asks for [`POWER`] and nothing else.
//!
//! **Pending while absent.** A domain whose guest is not installed, was revoked, no longer
//! matches its pin or fails to load is still registered, as an authority that refuses every
//! candidate with [`crate::domain_registry::PENDING`]. An operation waits; it is never
//! applied unchecked.
//!
//! **The candidate.** The complete candidate crosses as one JSON object,
//! `{"snapshot": ..., "facets": ...}`: the kernel's snapshot of the candidate graph and its
//! facet store (a snapshot alone does not carry the facets, and the validator over it fails
//! for want of the domain record). It is declared as
//! [`authority_host::GRAPH_SNAPSHOT_AND_FACETS`]; a guest claiming another format is refused
//! at review.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use authority_host::{AuthorityComponent, Budget, Verdict};
use eidetic::pack::{PackManifest, PackPartRole, PackVerdict, verify_pack};
use eidetic::{ManifestId, TrustEnvelope};
use kernel::graph::Graph;
use ostiary::app_admission::AppId;
use personae::PersonaId;
use serde::{Deserialize, Serialize};

use crate::domain_registry::{AdmissionBuilder, Authority, DomainRegistration, GuestAuthority};

/// The one power an authority pack may ask for.
pub const POWER: &str = "power:domain-authority";
/// Where the install store lives under the shared root.
pub const STORE_DIR: &str = "authority-guests";
/// Most bytes of a signed pack (the capsule disclosure cap).
pub const MAX_PACK_BYTES: usize = 1024 * 1024;

/// The hash a pin and a pack part name a component by: BLAKE3 of its bytes.
pub fn component_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Encode a complete candidate graph in the declared format. The cost of this is the
/// resident's, counted in [`GuestStats::encode`].
pub fn encode_candidate(graph: &Graph) -> Result<Vec<u8>, String> {
    #[derive(Serialize)]
    struct Candidate<'a, S: Serialize> {
        snapshot: &'a S,
        facets: &'a kernel::graph::NodeFacetStore,
    }
    let snapshot = graph.to_snapshot_at(0);
    serde_json::to_vec(&Candidate {
        snapshot: &snapshot,
        facets: graph.facets(),
    })
    .map_err(|error| format!("candidate encoding: {error}"))
}

/// What the running guest has cost, for status and for the measurements the plan records.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GuestStats {
    pub calls: u64,
    pub refused: u64,
    pub trapped: u64,
    /// Time the resident spent encoding candidates.
    pub encode: Duration,
    /// Time inside the guest, instantiate included.
    pub guest: Duration,
    pub max_encode: Duration,
    pub max_guest: Duration,
    pub max_fuel: u64,
    pub max_memory: usize,
    pub max_candidate: usize,
}

/// A loaded, verified guest that decides one domain.
pub struct GuestRuntime {
    component: AuthorityComponent,
    domain: String,
    hash: [u8; 32],
    stats: Mutex<GuestStats>,
}

impl GuestRuntime {
    /// Load the exact `bytes`, which must hash to `expected` and claim `domain`, in the
    /// format the resident encodes.
    pub fn load(domain: &str, bytes: &[u8], expected: &[u8; 32]) -> Result<Self, ReviewError> {
        let hash = component_hash(bytes);
        if &hash != expected {
            return Err(ReviewError::Component("component does not match the pinned hash".into()));
        }
        let engine = shared_engine().map_err(|error| ReviewError::Component(error.to_string()))?;
        let component = AuthorityComponent::load(&engine, bytes, Budget::default())
            .map_err(|error| ReviewError::Component(error.to_string()))?;
        let claim = component.claim();
        if claim.domain != domain {
            return Err(ReviewError::Claim(format!(
                "the guest claims domain {:?}, not {domain:?}",
                claim.domain
            )));
        }
        if claim.graph_format != authority_host::GRAPH_SNAPSHOT_AND_FACETS {
            return Err(ReviewError::Claim(format!(
                "the guest reads format {:?}; this resident encodes {}",
                claim.graph_format,
                authority_host::GRAPH_SNAPSHOT_AND_FACETS
            )));
        }
        Ok(Self {
            component,
            domain: domain.to_owned(),
            hash,
            stats: Mutex::new(GuestStats::default()),
        })
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub fn claim(&self) -> &authority_host::Claim {
        self.component.claim()
    }

    pub fn component_bytes(&self) -> usize {
        self.component.component_bytes()
    }

    pub fn stats(&self) -> GuestStats {
        *self.stats.lock().expect("guest stats")
    }

    fn record(&self, encode: Duration, candidate: usize, verdict: &Verdict, metrics: authority_host::Metrics) {
        let mut stats = self.stats.lock().expect("guest stats");
        stats.calls += 1;
        match verdict {
            Verdict::Admitted => {},
            Verdict::Refused(_) => stats.refused += 1,
            Verdict::Trapped(_) => stats.trapped += 1,
        }
        stats.encode += encode;
        stats.guest += metrics.elapsed;
        stats.max_encode = stats.max_encode.max(encode);
        stats.max_guest = stats.max_guest.max(metrics.elapsed);
        stats.max_fuel = stats.max_fuel.max(metrics.fuel);
        stats.max_memory = stats.max_memory.max(metrics.peak_memory);
        stats.max_candidate = stats.max_candidate.max(candidate);
    }

    /// Whether the complete candidate is admissible: encode it, ask the guest. A trap or an
    /// encoding failure refuses it.
    pub fn validate(&self, graph: &Graph) -> Result<(), String> {
        let started = Instant::now();
        let bytes = encode_candidate(graph)?;
        let encode = started.elapsed();
        let (verdict, metrics) = self.component.validate_measured(&bytes);
        self.record(encode, bytes.len(), &verdict, metrics);
        verdict.into_result()
    }

    /// The before/after check; admits when the guest says it has none.
    pub fn admit(&self, before: &Graph, after: &Graph) -> Result<(), String> {
        if !self.component.claim().admission {
            return Ok(());
        }
        let started = Instant::now();
        let (before, after) = (encode_candidate(before)?, encode_candidate(after)?);
        let encode = started.elapsed();
        let (verdict, metrics) = self.component.admit_measured(&before, &after);
        self.record(encode, before.len() + after.len(), &verdict, metrics);
        verdict.into_result()
    }

    pub(crate) fn validator(self: &Arc<Self>) -> pandect::GraphValidator {
        let runtime = self.clone();
        Arc::new(move |graph| runtime.validate(graph))
    }

    pub(crate) fn admission(self: &Arc<Self>) -> Option<AdmissionBuilder> {
        if !self.component.claim().admission {
            return None;
        }
        let runtime = self.clone();
        Some(Arc::new(move |_profiles| {
            let runtime = runtime.clone();
            Arc::new(move |before, after| runtime.admit(before, after))
        }))
    }
}

fn shared_engine() -> authority_host::wasmtime::Result<authority_host::wasmtime::Engine> {
    static ENGINE: OnceLock<authority_host::wasmtime::Engine> = OnceLock::new();
    if let Some(engine) = ENGINE.get() {
        return Ok(engine.clone());
    }
    let engine = AuthorityComponent::engine()?;
    Ok(ENGINE.get_or_init(|| engine).clone())
}

/// Why a review, an approval or a load was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewError {
    /// The pack bytes, address, signature or inventory do not verify.
    Pack(String),
    /// The pack asks for something other than [`POWER`] alone.
    Scope(String),
    /// The component does not match the pack, does not load, or fails its pin.
    Component(String),
    /// The component's claim is unusable here: another domain or another format.
    Claim(String),
    /// The domain is bound to another author's key; the owner must rebind it on purpose.
    AuthorMismatch { domain: String, bound: String, offered: String },
    /// The store could not be read or written.
    Store(String),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pack(message) => write!(f, "authority pack: {message}"),
            Self::Scope(message) => write!(f, "authority pack asks for more than {POWER}: {message}"),
            Self::Component(message) => write!(f, "authority component: {message}"),
            Self::Claim(message) => write!(f, "authority claim: {message}"),
            Self::AuthorMismatch { domain, bound, offered } => write!(
                f,
                "domain {domain} is bound to author {bound}; the pack is signed by {offered}"
            ),
            Self::Store(message) => write!(f, "authority store: {message}"),
        }
    }
}

impl std::error::Error for ReviewError {}

/// The signed pack as it travels: the manifest and the author's signature over it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPack {
    pub manifest: PackManifest,
    pub trust: TrustEnvelope,
}

/// What the owner decided about one domain: the author key bound to it and the component
/// hash pinned. This is what must agree across the persona's paired devices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    pub domain: String,
    /// The pack author's key, 32 bytes as hex.
    pub author: String,
    /// The pinned component, BLAKE3 as hex.
    pub component: String,
    /// The grammar version the guest claimed when it was approved.
    pub version: String,
    /// The owning application label the registration carries.
    pub owner: String,
    /// The owner revoked this guest: the component is gone and the domain is pending.
    #[serde(default)]
    pub revoked: bool,
}

/// How two devices' pins for one domain compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinAgreement {
    Agree,
    /// Same author, another component: the other device holds a different version.
    ComponentDiffers,
    /// Another author key: the domain is bound to different keys. Refuse until resolved.
    AuthorDiffers,
}

impl Pin {
    /// Compare with the pin a paired device reports for the same domain.
    pub fn compare(&self, other: &Pin) -> PinAgreement {
        if self.author != other.author {
            PinAgreement::AuthorDiffers
        } else if self.component != other.component {
            PinAgreement::ComponentDiffers
        } else {
            PinAgreement::Agree
        }
    }
}

/// A verified pack and component awaiting the owner's decision. It names what the review
/// surface shows (the domain, the author's key, the component address, the grant asked) and
/// grants nothing until [`Self::approve`].
pub struct AuthorityReview {
    pub domain: String,
    pub version: String,
    pub graph_format: String,
    pub admission: bool,
    /// The pack author's key as hex.
    pub author: String,
    pub pack_name: String,
    pub pack_hash: String,
    pub component: [u8; 32],
    pub component_bytes: usize,
    pack_bytes: Vec<u8>,
    component_data: Vec<u8>,
}

fn verify_pack_bytes(pack_bytes: &[u8], pack_hash: &str) -> Result<SignedPack, ReviewError> {
    if pack_bytes.len() > MAX_PACK_BYTES || ManifestId::of_blob(pack_bytes).0.to_hex() != pack_hash {
        return Err(ReviewError::Pack("address or size mismatch".into()));
    }
    let pack: SignedPack = serde_json::from_slice(pack_bytes).map_err(|error| ReviewError::Pack(error.to_string()))?;
    if verify_pack(&pack.manifest, &pack.trust) != PackVerdict::Trusted {
        return Err(ReviewError::Pack("signature failed".into()));
    }
    Ok(pack)
}

fn verify_inventory(pack: &SignedPack, component: &[u8]) -> Result<(), ReviewError> {
    let scopes = &pack.manifest.requested_scopes;
    if scopes.len() != 1 || scopes[0] != POWER {
        return Err(ReviewError::Scope(format!("asked for {scopes:?}")));
    }
    let parts = &pack.manifest.parts;
    if component.len() > authority_host::MAX_COMPONENT_BYTES
        || parts.len() != 1
        || parts[0].role != PackPartRole::WasmComponent
        || parts[0].bytes != component.len() as u64
        || parts[0].blob != ManifestId::of_blob(component)
    {
        return Err(ReviewError::Pack("component inventory, address or size mismatch".into()));
    }
    Ok(())
}

impl AuthorityReview {
    /// Verify a signed pack and its one component, load the component, and read its claim.
    /// Nothing is installed.
    pub fn prepare(pack_bytes: &[u8], pack_hash: &str, component: &[u8]) -> Result<Self, ReviewError> {
        let pack = verify_pack_bytes(pack_bytes, pack_hash)?;
        verify_inventory(&pack, component)?;
        let hash = component_hash(component);
        let claim = {
            // Load under the claim's own domain: read it first, then hold the guest to it.
            let engine = shared_engine().map_err(|error| ReviewError::Component(error.to_string()))?;
            AuthorityComponent::load(&engine, component, Budget::default())
                .map_err(|error| ReviewError::Component(error.to_string()))?
                .claim()
                .clone()
        };
        if pandect::DomainId::new(claim.domain.as_str()).is_err() {
            return Err(ReviewError::Claim(format!("{:?} is not a domain id", claim.domain)));
        }
        GuestRuntime::load(&claim.domain, component, &hash)?;
        Ok(Self {
            domain: claim.domain,
            version: claim.version,
            graph_format: claim.graph_format,
            admission: claim.admission,
            author: pack.manifest.author.clone(),
            pack_name: pack.manifest.name.clone(),
            pack_hash: pack_hash.to_owned(),
            component: hash,
            component_bytes: component.len(),
            pack_bytes: pack_bytes.to_vec(),
            component_data: component.to_vec(),
        })
    }

    /// The owner's approval: bind this author's key to the domain, pin the component, and
    /// install it. `owner` is the application label the registration carries. A domain bound
    /// to another author's key refuses unless `rebind` says the owner means to change it.
    pub fn approve(self, store: &GuestStore, owner: AppId, rebind: bool) -> Result<Pin, ReviewError> {
        if let Some(bound) = store.pin(&self.domain) {
            if bound.author != self.author && !rebind {
                return Err(ReviewError::AuthorMismatch {
                    domain: self.domain,
                    bound: bound.author,
                    offered: self.author,
                });
            }
        }
        let pin = Pin {
            domain: self.domain.clone(),
            author: self.author.clone(),
            component: hex(&self.component),
            version: self.version.clone(),
            owner: owner.as_str().to_owned(),
            revoked: false,
        };
        let dir = store.domain_dir(&self.domain);
        let io = |error: std::io::Error| ReviewError::Store(error.to_string());
        std::fs::create_dir_all(&dir).map_err(io)?;
        write_atomic(&dir.join("component.wasm"), &self.component_data).map_err(io)?;
        write_atomic(&dir.join("pack.json"), &self.pack_bytes).map_err(io)?;
        // The pin is written last: until it is, the domain has no approval.
        write_atomic(&dir.join("pin.json"), &serde_json::to_vec_pretty(&pin).expect("pin encodes")).map_err(io)?;
        Ok(pin)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let staged = path.with_extension(format!("tmp{}", std::process::id()));
    std::fs::write(&staged, bytes)?;
    std::fs::rename(&staged, path)
}

/// The persona's installed authority guests: `<shared root>/authority-guests/<persona>/<domain>/`
/// holding `pin.json` (the owner's decision), `pack.json` and `component.wasm`.
#[derive(Clone, Debug)]
pub struct GuestStore {
    dir: PathBuf,
}

/// An interned `&'static str`, for the registration's version label. The set is bounded by
/// the grammar versions guests have claimed.
fn intern(text: &str) -> &'static str {
    static SET: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
    let mut set = SET.get_or_init(Default::default).lock().expect("interner");
    if let Some(existing) = set.get(text) {
        return *existing;
    }
    let leaked: &'static str = Box::leak(text.to_owned().into_boxed_str());
    set.insert(leaked);
    leaked
}

impl GuestStore {
    /// The store for `persona` under `shared_root`. Creates nothing until a guest is approved.
    pub fn at(shared_root: &Path, persona: PersonaId) -> Self {
        Self {
            dir: shared_root.join(STORE_DIR).join(persona.as_uuid().to_string()),
        }
    }

    fn domain_dir(&self, domain: &str) -> PathBuf {
        self.dir.join(domain)
    }

    /// The owner's pin for `domain`, if one was ever approved (a revoked pin still reads).
    pub fn pin(&self, domain: &str) -> Option<Pin> {
        let bytes = std::fs::read(self.domain_dir(domain).join("pin.json")).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Every approved or revoked pin, by domain.
    pub fn pins(&self) -> Vec<Pin> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut pins: Vec<Pin> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| self.pin(&entry.file_name().to_string_lossy()))
            .collect();
        pins.sort_by(|a, b| a.domain.cmp(&b.domain));
        pins
    }

    /// Revoke the owner's approval: the component and pack are removed, the pin stays marked
    /// revoked, and the domain refuses as pending until a new review approves a guest.
    pub fn revoke(&self, domain: &str) -> Result<(), ReviewError> {
        let mut pin = self
            .pin(domain)
            .ok_or_else(|| ReviewError::Store(format!("no guest is approved for {domain}")))?;
        pin.revoked = true;
        let dir = self.domain_dir(domain);
        let io = |error: std::io::Error| ReviewError::Store(error.to_string());
        write_atomic(&dir.join("pin.json"), &serde_json::to_vec_pretty(&pin).expect("pin encodes")).map_err(io)?;
        for file in ["component.wasm", "pack.json"] {
            match std::fs::remove_file(dir.join(file)) {
                Ok(()) => {},
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                Err(error) => return Err(io(error)),
            }
        }
        Ok(())
    }

    /// Load the guest installed for `pin`, re-verifying everything review checked: the pack
    /// address is not stored, so the signature, the author against the pin, the inventory,
    /// the component hash against the pin, the claim.
    fn load(&self, pin: &Pin) -> Result<Arc<GuestRuntime>, ReviewError> {
        if pin.revoked {
            return Err(ReviewError::Component("revoked by the owner".into()));
        }
        let dir = self.domain_dir(&pin.domain);
        let read = |name: &str| {
            std::fs::read(dir.join(name)).map_err(|error| ReviewError::Component(format!("{name}: {error}")))
        };
        let (pack_bytes, component) = (read("pack.json")?, read("component.wasm")?);
        let pack = verify_pack_bytes(&pack_bytes, &ManifestId::of_blob(&pack_bytes).0.to_hex())?;
        if pack.manifest.author != pin.author {
            return Err(ReviewError::AuthorMismatch {
                domain: pin.domain.clone(),
                bound: pin.author.clone(),
                offered: pack.manifest.author,
            });
        }
        verify_inventory(&pack, &component)?;
        let mut expected = [0u8; 32];
        for (index, byte) in expected.iter_mut().enumerate() {
            let digits = pin.component.get(2 * index..2 * index + 2).unwrap_or("zz");
            *byte = u8::from_str_radix(digits, 16)
                .map_err(|_| ReviewError::Component("the pinned hash is not 32 bytes of hex".into()))?;
        }
        Ok(Arc::new(GuestRuntime::load(&pin.domain, &component, &expected)?))
    }

    /// One registration per pinned domain, as `Authority::Guest`. A domain whose guest loads
    /// is served by it; one whose guest is revoked, missing, altered or broken is registered
    /// as pending and refuses every candidate (the reason is in the refusal text).
    pub fn registrations(&self) -> Vec<DomainRegistration> {
        self.pins()
            .into_iter()
            .map(|pin| {
                let mut pinned = [0u8; 32];
                for (index, byte) in pinned.iter_mut().enumerate() {
                    *byte = pin
                        .component
                        .get(2 * index..2 * index + 2)
                        .and_then(|digits| u8::from_str_radix(digits, 16).ok())
                        .unwrap_or_default();
                }
                let authority = match self.load(&pin) {
                    Ok(runtime) => GuestAuthority::installed(pinned, runtime),
                    Err(error) => GuestAuthority::absent(pinned, error.to_string()),
                };
                DomainRegistration::new(
                    pin.domain.clone(),
                    AppId::new(pin.owner.as_str()),
                    intern(&pin.version),
                    Authority::Guest(authority),
                )
            })
            .collect()
    }
}
