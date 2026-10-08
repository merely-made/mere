// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The reservoir: every mere a persona holds, one per data domain.
//!
//! A mere is its data domain's repository, and all of an identity's meres
//! together are its reservoir (TERMINOLOGY, *reservoir*). This module holds the
//! reservoir's index: which meres exist and which domain each holds. The device
//! resident owns the reservoir; applications reach meres through admitted
//! routes and never open these files (reservoir plan, V1).
//!
//! Layout: `<shared root>/personas/<persona>/reservoir/reservoir.redb`. The
//! redb file's exclusive lock is the one-owner rule: a second process that
//! opens the same reservoir is refused rather than left waiting.
//!
//! Ids are derived. A mere's id is a UUIDv5 of the persona and the domain, so
//! the same domain always names the same mere, on every device of that persona.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use identity::PersonaId;
use muniment::{Backend, JsonSlots, StoreError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine_profile_store::PERSONAS_DIR;

/// The reservoir's directory under a persona's shared-root directory.
pub const RESERVOIR_DIR: &str = "reservoir";
/// The reservoir index database inside [`RESERVOIR_DIR`].
pub const RESERVOIR_DB_FILENAME: &str = "reservoir.redb";
/// The directory under a reservoir that holds each mere's own store.
pub const MERES_DIR: &str = "meres";
/// The slot the index lives in.
pub const RESERVOIR_INDEX_SLOT: &str = "pandect/reservoir/index/v1";
/// Schema of the index document.
pub const RESERVOIR_INDEX_SCHEMA: &str = "pandect.reservoir.index/v1";
/// Schema of one mere record.
pub const MERE_RECORD_SCHEMA: &str = "pandect.reservoir.mere/v1";

/// The namespace every derived mere id is minted under. Fixed forever: changing
/// it would give every existing mere a new id.
const MERE_ID_NAMESPACE: Uuid = Uuid::from_u128(0x6d65_7265_2d72_6573_6572_766f_6972_0001);

const MAX_DOMAIN_LEN: usize = 64;

/// A data domain's stable identifier, such as `divination` or `browsing`.
///
/// Lowercase ASCII letters and digits, with `-` and `.` between them; at most
/// 64 bytes. Stable identifiers are what let two applications find the same
/// mere (reservoir plan, §7).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DomainId(String);

impl DomainId {
    pub fn new(id: impl Into<String>) -> Result<Self, ReservoirError> {
        let id = id.into();
        if valid_domain(&id) {
            Ok(Self(id))
        } else {
            Err(ReservoirError::InvalidDomain(id))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_domain(id: &str) -> bool {
    let bytes = id.as_bytes();
    let edge = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    !bytes.is_empty()
        && bytes.len() <= MAX_DOMAIN_LEN
        && edge(bytes[0])
        && edge(bytes[bytes.len() - 1])
        && bytes.iter().all(|&b| edge(b) || b == b'-' || b == b'.')
}

impl TryFrom<String> for DomainId {
    type Error = ReservoirError;
    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl From<DomainId> for String {
    fn from(id: DomainId) -> Self {
        id.0
    }
}

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A mere's id: derived from its persona and domain, never chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MereId(pub Uuid);

impl MereId {
    /// The id of `domain`'s mere in `persona`'s reservoir.
    pub fn derive(persona: PersonaId, domain: &DomainId) -> Self {
        let mut name = persona.as_uuid().as_bytes().to_vec();
        name.push(b'/');
        name.extend_from_slice(domain.as_str().as_bytes());
        Self(Uuid::new_v5(&MERE_ID_NAMESPACE, &name))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MereId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// One mere in the reservoir.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MereRecord {
    pub schema: String,
    pub id: MereId,
    pub persona: PersonaId,
    pub domain: DomainId,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReservoirIndex {
    schema: String,
    persona: PersonaId,
    meres: BTreeMap<DomainId, MereRecord>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    access: BTreeMap<MereId, BTreeMap<String, MereApplicationAccess>>,
}

/// A recorded owner decision for one application and one mere. Explicit
/// access defaults on; ambient crossing defaults off. A denial blocks both
/// routes without erasing the independent ambient decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MereApplicationAccess {
    pub denied: bool,
    pub ambient: bool,
    pub recorded_at_ms: u64,
    pub author: crate::Author,
}

/// Why the reservoir refused.
#[derive(Debug)]
pub enum ReservoirError {
    /// A domain identifier broke the naming rule.
    InvalidDomain(String),
    /// The stored index belongs to another persona.
    PersonaMismatch {
        expected: PersonaId,
        found: PersonaId,
    },
    /// A stored record fails its own identity rules.
    Corrupt(String),
    /// The reservoir could not be opened, including because another process
    /// already owns it.
    Open { path: PathBuf, reason: String },
    /// The store failed underneath.
    Store(StoreError),
}

impl fmt::Display for ReservoirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDomain(id) => write!(
                f,
                "invalid data domain {id:?}: use 1-64 lowercase letters or digits, with '-' or '.' between them"
            ),
            Self::PersonaMismatch { expected, found } => write!(
                f,
                "reservoir belongs to persona {}, not {}",
                found.as_uuid(),
                expected.as_uuid()
            ),
            Self::Corrupt(reason) => write!(f, "reservoir index is corrupt: {reason}"),
            Self::Open { path, reason } => {
                write!(
                    f,
                    "could not open the reservoir at {}: {reason}",
                    path.display()
                )
            },
            Self::Store(error) => write!(f, "reservoir store failed: {error}"),
        }
    }
}

impl std::error::Error for ReservoirError {}

impl From<StoreError> for ReservoirError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

/// Where a persona's reservoir lives under the shared root.
pub fn reservoir_dir(shared_root: &Path, persona: PersonaId) -> PathBuf {
    shared_root
        .join(PERSONAS_DIR)
        .join(persona.as_uuid().to_string())
        .join(RESERVOIR_DIR)
}

/// Open the durable backend for a persona's reservoir, creating it if absent.
///
/// This takes the reservoir's exclusive lock. Only the device resident, or an
/// application embedding it, should call this; a second caller is refused with
/// [`ReservoirError::Open`].
#[cfg(not(target_arch = "wasm32"))]
pub fn open_reservoir_backend(
    shared_root: &Path,
    persona: PersonaId,
) -> Result<muniment::RedbBackend, ReservoirError> {
    let dir = reservoir_dir(shared_root, persona);
    std::fs::create_dir_all(&dir).map_err(|error| ReservoirError::Open {
        path: dir.clone(),
        reason: error.to_string(),
    })?;
    let path = dir.join(RESERVOIR_DB_FILENAME);
    muniment::RedbBackend::open(&path).map_err(|error| ReservoirError::Open {
        path,
        reason: error.to_string(),
    })
}

/// Where mere `mere` keeps its sessions: `<reservoir>/meres/<mere id>/`.
pub fn mere_dir(shared_root: &Path, persona: PersonaId, mere: MereId) -> PathBuf {
    reservoir_dir(shared_root, persona)
        .join(MERES_DIR)
        .join(mere.0.to_string())
}

/// Open mere `mere`'s store: a muniment directory backend whose keys are the
/// session files ([`crate::graph_session`]). Only the reservoir's owner, who
/// already holds [`open_reservoir_backend`]'s lock, should call this; the
/// store takes its own lock too, so a second opener is refused at once.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_mere_backend(
    shared_root: &Path,
    persona: PersonaId,
    mere: MereId,
) -> Result<muniment::DirectoryBackend, ReservoirError> {
    let dir = mere_dir(shared_root, persona, mere);
    muniment::DirectoryBackend::open(&dir).map_err(|error| ReservoirError::Open {
        path: dir,
        reason: error.to_string(),
    })
}

/// A persona's reservoir index, persisted through a muniment backend.
pub struct ReservoirStore<B> {
    slots: JsonSlots<B>,
    index: ReservoirIndex,
}

impl<B: Backend> ReservoirStore<B> {
    /// Open `persona`'s reservoir. An empty backend yields an empty reservoir;
    /// a stored index is verified before it is trusted.
    pub async fn open(backend: B, persona: PersonaId) -> Result<Self, ReservoirError> {
        let slots = JsonSlots::new(backend);
        let index = match slots.load::<ReservoirIndex>(RESERVOIR_INDEX_SLOT).await? {
            None => ReservoirIndex {
                schema: RESERVOIR_INDEX_SCHEMA.to_string(),
                persona,
                meres: BTreeMap::new(),
                access: BTreeMap::new(),
            },
            Some(index) => {
                verify_index(&index, persona)?;
                index
            },
        };
        Ok(Self { slots, index })
    }

    pub fn persona(&self) -> PersonaId {
        self.index.persona
    }

    /// Every mere, ordered by domain.
    pub fn meres(&self) -> impl Iterator<Item = &MereRecord> {
        self.index.meres.values()
    }

    pub fn get(&self, domain: &DomainId) -> Option<&MereRecord> {
        self.index.meres.get(domain)
    }

    pub fn get_by_id(&self, id: MereId) -> Option<&MereRecord> {
        self.index.meres.values().find(|record| record.id == id)
    }

    /// Recorded application decisions, in mere/application order.
    pub fn access(&self) -> impl Iterator<Item = (MereId, &str, &MereApplicationAccess)> {
        self.index.access.iter().flat_map(|(mere, apps)| {
            apps.iter()
                .map(move |(app, access)| (*mere, app.as_str(), access))
        })
    }

    pub fn application_access(&self, mere: MereId, app: &str) -> Option<&MereApplicationAccess> {
        self.index.access.get(&mere)?.get(app)
    }

    /// Commit a decision in the reservoir's existing muniment index. Neither
    /// an unknown mere nor an ambiguous application label may acquire policy.
    /// Failed writes leave the in-memory decision unchanged.
    pub async fn set_application_access(
        &mut self,
        mere: MereId,
        app: String,
        access: MereApplicationAccess,
    ) -> Result<(), ReservoirError> {
        if self.get_by_id(mere).is_none() || !valid_application(&app) {
            return Err(ReservoirError::Corrupt(
                "access decision names an unknown mere or invalid application".into(),
            ));
        }
        let mut candidate = self.index.clone();
        candidate
            .access
            .entry(mere)
            .or_default()
            .insert(app, access);
        self.slots.save(RESERVOIR_INDEX_SLOT, &candidate).await?;
        self.index = candidate;
        Ok(())
    }

    /// The domain's mere, created if the reservoir has none yet. Returns the
    /// record and whether this call created it. A domain has exactly one mere.
    pub async fn ensure(
        &mut self,
        domain: DomainId,
        created_at_ms: u64,
    ) -> Result<(MereRecord, bool), ReservoirError> {
        if let Some(existing) = self.index.meres.get(&domain) {
            return Ok((existing.clone(), false));
        }
        let record = MereRecord {
            schema: MERE_RECORD_SCHEMA.to_string(),
            id: MereId::derive(self.index.persona, &domain),
            persona: self.index.persona,
            domain: domain.clone(),
            created_at_ms,
        };
        self.index.meres.insert(domain.clone(), record.clone());
        if let Err(error) = self.slots.save(RESERVOIR_INDEX_SLOT, &self.index).await {
            // Keep memory equal to what is stored.
            self.index.meres.remove(&domain);
            return Err(error.into());
        }
        Ok((record, true))
    }
}

fn verify_index(index: &ReservoirIndex, persona: PersonaId) -> Result<(), ReservoirError> {
    if index.schema != RESERVOIR_INDEX_SCHEMA {
        return Err(ReservoirError::Corrupt(format!(
            "unknown schema {:?}",
            index.schema
        )));
    }
    if index.persona != persona {
        return Err(ReservoirError::PersonaMismatch {
            expected: persona,
            found: index.persona,
        });
    }
    for (domain, record) in &index.meres {
        if record.schema != MERE_RECORD_SCHEMA
            || &record.domain != domain
            || record.persona != persona
            || record.id != MereId::derive(persona, domain)
        {
            return Err(ReservoirError::Corrupt(format!(
                "the record for domain {domain} does not match its identity"
            )));
        }
    }
    for (mere, apps) in &index.access {
        if !index.meres.values().any(|record| record.id == *mere)
            || apps.keys().any(|app| !valid_application(app))
        {
            return Err(ReservoirError::Corrupt(
                "access decisions name an unknown mere or invalid application".into(),
            ));
        }
    }
    Ok(())
}

fn valid_application(app: &str) -> bool {
    !app.is_empty()
        && app.len() <= 256
        && app == app.trim()
        && app == app.to_ascii_lowercase()
        && !app.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use muniment::MemoryBackend;
    use pollster::block_on;

    fn persona(n: u128) -> PersonaId {
        PersonaId(Uuid::from_u128(n))
    }

    fn domain(id: &str) -> DomainId {
        DomainId::new(id).unwrap()
    }

    #[test]
    fn domain_ids_follow_the_naming_rule() {
        for ok in ["divination", "browsing", "notes.seeds", "rng-2", "a", "9"] {
            assert!(DomainId::new(ok).is_ok(), "{ok} should be valid");
        }
        let too_long = "a".repeat(MAX_DOMAIN_LEN + 1);
        for bad in [
            "",
            "Divination",
            "-lead",
            "trail.",
            "has space",
            "under_score",
            too_long.as_str(),
        ] {
            assert!(DomainId::new(bad).is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn derived_ids_are_stable_and_distinct() {
        let a = MereId::derive(persona(1), &domain("divination"));
        assert_eq!(a, MereId::derive(persona(1), &domain("divination")));
        assert_ne!(a, MereId::derive(persona(2), &domain("divination")));
        assert_ne!(a, MereId::derive(persona(1), &domain("browsing")));
        assert_eq!(a.as_uuid().get_version_num(), 5);
    }

    #[test]
    fn a_domain_has_exactly_one_mere() {
        block_on(async {
            let mut store = ReservoirStore::open(MemoryBackend::new(), persona(1))
                .await
                .unwrap();
            let (first, created) = store.ensure(domain("divination"), 10).await.unwrap();
            assert!(created);
            let (again, created) = store.ensure(domain("divination"), 20).await.unwrap();
            assert!(!created);
            assert_eq!(first, again);
            assert_eq!(again.created_at_ms, 10);
            assert_eq!(store.meres().count(), 1);
            assert_eq!(store.get_by_id(first.id), Some(&first));
        });
    }

    #[test]
    fn the_index_survives_reopening_the_backend() {
        block_on(async {
            let backend = MemoryBackend::new();
            let mut store = ReservoirStore::open(backend.clone(), persona(1))
                .await
                .unwrap();
            store.ensure(domain("divination"), 1).await.unwrap();
            store.ensure(domain("browsing"), 2).await.unwrap();
            drop(store);

            let reopened = ReservoirStore::open(backend, persona(1)).await.unwrap();
            let domains: Vec<_> = reopened
                .meres()
                .map(|m| m.domain.as_str().to_string())
                .collect();
            assert_eq!(domains, ["browsing", "divination"]);
        });
    }

    #[test]
    fn another_persona_cannot_open_the_index() {
        block_on(async {
            let backend = MemoryBackend::new();
            let mut store = ReservoirStore::open(backend.clone(), persona(1))
                .await
                .unwrap();
            store.ensure(domain("divination"), 1).await.unwrap();
            let refused = ReservoirStore::open(backend, persona(2)).await;
            assert!(matches!(
                refused,
                Err(ReservoirError::PersonaMismatch { .. })
            ));
        });
    }

    #[test]
    fn a_tampered_record_is_refused() {
        block_on(async {
            let backend = MemoryBackend::new();
            let mut store = ReservoirStore::open(backend.clone(), persona(1))
                .await
                .unwrap();
            store.ensure(domain("divination"), 1).await.unwrap();
            let slots = JsonSlots::new(backend.clone());
            let mut index: ReservoirIndex =
                slots.load(RESERVOIR_INDEX_SLOT).await.unwrap().unwrap();
            index.meres.get_mut(&domain("divination")).unwrap().id = MereId(Uuid::from_u128(7));
            slots.save(RESERVOIR_INDEX_SLOT, &index).await.unwrap();
            let refused = ReservoirStore::open(backend, persona(1)).await;
            assert!(matches!(refused, Err(ReservoirError::Corrupt(_))));
        });
    }

    #[test]
    fn the_reservoir_sits_beside_the_persona() {
        let root = Path::new("root");
        let dir = reservoir_dir(root, persona(1));
        assert_eq!(
            dir,
            root.join("personas")
                .join(Uuid::from_u128(1).to_string())
                .join("reservoir")
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_durable_reservoir_reopens_and_refuses_a_second_owner() {
        let root = tempfile::tempdir().unwrap();
        let owner = open_reservoir_backend(root.path(), persona(1)).unwrap();
        let second = open_reservoir_backend(root.path(), persona(1));
        assert!(
            matches!(second, Err(ReservoirError::Open { .. })),
            "a second owner must be refused while the first holds the lock"
        );
        block_on(async {
            let mut store = ReservoirStore::open(owner, persona(1)).await.unwrap();
            store.ensure(domain("divination"), 5).await.unwrap();
        });
        // The first owner's store dropped above; the lock is free again.
        let reopened = open_reservoir_backend(root.path(), persona(1)).unwrap();
        block_on(async {
            let store = ReservoirStore::open(reopened, persona(1)).await.unwrap();
            assert_eq!(
                store.get(&domain("divination")).map(|m| m.created_at_ms),
                Some(5)
            );
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_mere_keeps_its_sessions_in_its_own_directory() {
        use crate::graph_session::MereSessions;
        use kernel::graph::Author;

        let root = tempfile::tempdir().unwrap();
        let mere = MereId::derive(persona(1), &domain("divination"));
        let store = open_mere_backend(root.path(), persona(1), mere).unwrap();
        assert!(
            matches!(
                open_mere_backend(root.path(), persona(1), mere),
                Err(ReservoirError::Open { .. })
            ),
            "a mere's store has one owner"
        );
        let session = block_on(MereSessions::new(store).mint(Author::user(), None)).unwrap();
        let manifest = mere_dir(root.path(), persona(1), mere)
            .join("sessions")
            .join(session.session_id.as_uuid().to_string())
            .join("manifest.json");
        assert!(manifest.is_file(), "{} exists", manifest.display());
        assert!(manifest.starts_with(reservoir_dir(root.path(), persona(1)).join(MERES_DIR)));
    }
    #[derive(Clone)]
    struct Fallible {
        memory: MemoryBackend,
        fail: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }
    #[async_trait::async_trait]
    impl Backend for Fallible {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            self.memory.get(key).await
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(StoreError::Backend("injected grant write failure".into()));
            }
            self.memory.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.memory.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.memory.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.memory.scan(start, end).await
        }
        async fn apply(&self, ops: &[muniment::WriteOp]) -> Result<(), StoreError> {
            self.memory.apply(ops).await
        }
    }

    #[test]
    fn access_defaults_migrate_and_failed_writes_change_no_decision() {
        block_on(async {
            let backend = Fallible {
                memory: MemoryBackend::new(),
                fail: Default::default(),
            };
            let mut store = ReservoirStore::open(backend.clone(), persona(1))
                .await
                .unwrap();
            let (mere, _) = store.ensure(domain("divination"), 1).await.unwrap();
            // The v1 index has no access field when no decisions exist.
            let raw = backend.get(RESERVOIR_INDEX_SLOT).await.unwrap().unwrap();
            assert!(!String::from_utf8(raw).unwrap().contains("access"));
            drop(store);
            let mut store = ReservoirStore::open(backend.clone(), persona(1))
                .await
                .unwrap();
            assert!(store.application_access(mere.id, "cleromancy").is_none());
            let recorded = MereApplicationAccess {
                denied: false,
                ambient: true,
                recorded_at_ms: 2,
                author: crate::Author::person(persona(1).as_uuid().to_string()).via("turnstone"),
            };
            store
                .set_application_access(mere.id, "cleromancy".into(), recorded.clone())
                .await
                .unwrap();
            backend
                .fail
                .store(true, std::sync::atomic::Ordering::SeqCst);
            let mut denied = recorded.clone();
            denied.denied = true;
            assert!(
                store
                    .set_application_access(mere.id, "cleromancy".into(), denied)
                    .await
                    .is_err()
            );
            assert_eq!(
                store.application_access(mere.id, "cleromancy"),
                Some(&recorded)
            );
            drop(store);
            let reopened = ReservoirStore::open(backend.clone(), persona(1))
                .await
                .unwrap();
            assert_eq!(
                reopened.application_access(mere.id, "cleromancy"),
                Some(&recorded)
            );
        });
    }
}
