// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use zbus::Connection;
use zbus::message::Header;
use zbus::names::BusName;
use zbus::zvariant::OwnedObjectPath;

use super::vault::SecretServiceVault;
use super::{
    DbusResult, SERVICE_PATH, SecretDbusError, SecretServiceLimits, SecretServiceStore,
    item_path, parse_collection_path, parse_item_path,
};
use crate::secret_service::sessions::{SessionRefusal, SessionTable};
use crate::secret_service::{
    MetadataSnapshot, SecretCollection, SecretCollectionId, SecretItem, SecretItemId,
};

/// Bus-authenticated caller facts supplied to the resident access policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretServiceCaller {
    /// Unique D-Bus connection name from the method header.
    pub bus_name: String,
    /// Unix uid reported by the session bus, when available.
    pub unix_user_id: Option<u32>,
    /// Process id reported by the session bus, when available.
    pub process_id: Option<u32>,
    /// Current `/proc/<pid>/exe` target, when resolvable.
    pub executable: Option<PathBuf>,
    /// Linux security label bytes reported by the bus, when available.
    pub linux_security_label: Option<Vec<u8>>,
}

/// Security-relevant operation presented to the host policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecretServiceOperation {
    /// Establish a transfer session.
    OpenSession,
    /// Read the service collection catalog.
    ListCollections,
    /// Create a collection.
    CreateCollection,
    /// Search item attributes.
    Search,
    /// Lock named objects.
    Lock,
    /// Unlock named objects.
    Unlock,
    /// Resolve a collection alias.
    ReadAlias,
    /// Change a collection alias.
    SetAlias,
    /// Read collection metadata.
    ReadCollection(SecretCollectionId),
    /// Change collection metadata.
    ChangeCollection(SecretCollectionId),
    /// Delete a collection and its items.
    DeleteCollection(SecretCollectionId),
    /// Create an item in a collection.
    CreateItem(SecretCollectionId),
    /// Read item metadata.
    ReadItem(SecretItemId),
    /// Release an item's secret bytes.
    ReadSecret(SecretItemId),
    /// Change item metadata or secret bytes.
    ChangeItem(SecretItemId),
    /// Delete an item.
    DeleteItem(SecretItemId),
}

/// Host-owned policy deciding which bus-authenticated callers may use the adapter.
pub trait SecretServiceAccessPolicy: Send + Sync + 'static {
    /// Return true only when this exact caller may perform the operation.
    fn allows(&self, caller: &SecretServiceCaller, operation: &SecretServiceOperation) -> bool;
}

impl<F> SecretServiceAccessPolicy for F
where
    F: Fn(&SecretServiceCaller, &SecretServiceOperation) -> bool + Send + Sync + 'static,
{
    fn allows(&self, caller: &SecretServiceCaller, operation: &SecretServiceOperation) -> bool {
        self(caller, operation)
    }
}

pub(super) struct ServiceState {
    pub(super) store: SecretServiceStore,
    pub(super) vault: Arc<dyn SecretServiceVault>,
    policy: Arc<dyn SecretServiceAccessPolicy>,
    sessions: SessionTable,
    /// What a locked service answers from (ruling 11): refreshed at serve,
    /// at each unlock and after each write made here.
    snapshot: Mutex<MetadataSnapshot>,
}

impl ServiceState {
    pub(super) fn new(
        store: SecretServiceStore,
        limits: SecretServiceLimits,
        policy: Arc<dyn SecretServiceAccessPolicy>,
        vault: Arc<dyn SecretServiceVault>,
    ) -> Self {
        let state = Self {
            store,
            vault,
            policy,
            sessions: SessionTable::new(limits.max_sessions),
            snapshot: Mutex::new(MetadataSnapshot::default()),
        };
        state.refresh_snapshot();
        state
    }

    /// Whether the vault is locked: every collection and item follows it
    /// (ruling 10).
    pub(super) fn locked(&self) -> bool {
        self.vault.is_locked()
    }

    /// Retake the snapshot; kept as it was while locked.
    pub(super) fn refresh_snapshot(&self) {
        if let Ok(snapshot) = self.store.snapshot() {
            *self.snapshot.lock().unwrap() = snapshot;
        }
    }

    pub(super) fn snapshot(&self) -> MetadataSnapshot {
        self.snapshot.lock().unwrap().clone()
    }

    pub(super) fn collections(&self) -> DbusResult<Vec<SecretCollection>> {
        match self.locked() {
            true => Ok(self.snapshot.lock().unwrap().collections()),
            false => Ok(self.store.collections()?),
        }
    }

    pub(super) fn collection(&self, id: SecretCollectionId) -> DbusResult<SecretCollection> {
        match self.locked() {
            true => Ok(self.snapshot.lock().unwrap().collection(id)?),
            false => Ok(self.store.collection(id)?),
        }
    }

    pub(super) fn items(&self, collection: SecretCollectionId) -> DbusResult<Vec<SecretItem>> {
        match self.locked() {
            true => Ok(self.snapshot.lock().unwrap().items(collection)?),
            false => Ok(self.store.items(collection)?),
        }
    }

    pub(super) fn item(&self, id: SecretItemId) -> DbusResult<SecretItem> {
        match self.locked() {
            true => Ok(self.snapshot.lock().unwrap().item(id)?),
            false => Ok(self.store.item(id)?),
        }
    }

    pub(super) fn search(
        &self,
        attributes: &BTreeMap<String, String>,
    ) -> DbusResult<Vec<SecretItem>> {
        match self.locked() {
            true => Ok(self.snapshot.lock().unwrap().search(attributes)),
            false => Ok(self.store.search(attributes)?),
        }
    }

    pub(super) fn read_alias(&self, alias: &str) -> DbusResult<Option<SecretCollectionId>> {
        match self.locked() {
            true => Ok(self.snapshot.lock().unwrap().read_alias(alias)),
            false => Ok(self.store.read_alias(alias)?),
        }
    }

    pub(super) async fn authorize(
        &self,
        connection: &Connection,
        header: &Header<'_>,
        operation: SecretServiceOperation,
    ) -> DbusResult<SecretServiceCaller> {
        let caller = caller(connection, header).await?;
        if self.policy.allows(&caller, &operation) {
            Ok(caller)
        } else {
            Err(SecretDbusError::AccessDenied(format!(
                "caller {} is not admitted for {operation:?}",
                caller.bus_name
            )))
        }
    }

    pub(super) fn open_session(&self, owner: &str) -> DbusResult<OwnedObjectPath> {
        let path = self.sessions.open(owner).map_err(session_error)?;
        Ok(OwnedObjectPath::try_from(path).expect("UUID session path is valid"))
    }

    pub(super) fn require_session(&self, path: &OwnedObjectPath, owner: &str) -> DbusResult<()> {
        self.sessions
            .require(path.as_str(), owner)
            .map_err(session_error)
    }

    pub(super) fn close_session(&self, path: &OwnedObjectPath, owner: &str) -> DbusResult<()> {
        self.sessions
            .close(path.as_str(), owner)
            .map_err(session_error)
    }

    pub(super) fn require_collection_unlocked(&self, id: SecretCollectionId) -> DbusResult<()> {
        if self.locked() {
            Err(SecretDbusError::IsLocked(format!(
                "collection {id} is locked"
            )))
        } else {
            Ok(())
        }
    }

    pub(super) fn require_item_unlocked(&self, id: SecretItemId) -> DbusResult<()> {
        if self.locked() {
            Err(SecretDbusError::IsLocked(format!("item {id} is locked")))
        } else {
            Ok(())
        }
    }

    /// Whether `path` names a collection, alias or item this service holds.
    pub(super) fn holds(&self, path: &OwnedObjectPath) -> DbusResult<bool> {
        if let Some(alias) = path.as_str().strip_prefix(&format!("{SERVICE_PATH}/aliases/")) {
            return Ok(self.read_alias(alias)?.is_some());
        }
        if let Some(id) = parse_collection_path(path) {
            self.collection(id)?;
            return Ok(true);
        }
        if let Some(id) = parse_item_path(path) {
            self.item(id)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// A client's `Lock` engages the whole vault lock (ruling 68).
    pub(super) fn lock_vault(&self) -> DbusResult<()> {
        self.vault.lock().map_err(SecretDbusError::Failed)
    }

    /// Search results split as the specification asks: every item follows
    /// the vault, so all are locked or none.
    pub(super) fn split_locked(
        &self,
        items: impl IntoIterator<Item = SecretItemId>,
    ) -> (Vec<OwnedObjectPath>, Vec<OwnedObjectPath>) {
        let paths = items.into_iter().map(item_path).collect();
        match self.locked() {
            true => (Vec::new(), paths),
            false => (paths, Vec::new()),
        }
    }
}

/// The D-Bus errors the session table's refusals have always produced.
fn session_error(refusal: SessionRefusal) -> SecretDbusError {
    match refusal {
        SessionRefusal::Full(max) => SecretDbusError::LimitsExceeded(format!(
            "the resident retains at most {max} Secret Service sessions"
        )),
        SessionRefusal::Absent => SecretDbusError::NoSession(
            "session is absent or belongs to another D-Bus connection".into(),
        ),
    }
}

async fn caller(connection: &Connection, header: &Header<'_>) -> DbusResult<SecretServiceCaller> {
    let sender = header
        .sender()
        .ok_or_else(|| {
            SecretDbusError::AccessDenied("method has no bus-authenticated sender".into())
        })?
        .to_owned();
    let proxy = zbus::fdo::DBusProxy::new(connection).await?;
    let credentials = proxy
        .get_connection_credentials(BusName::from(sender.clone()))
        .await?;
    let process_id = credentials.process_id();
    let executable = process_id.and_then(|pid| std::fs::read_link(format!("/proc/{pid}/exe")).ok());
    Ok(SecretServiceCaller {
        bus_name: sender.to_string(),
        unix_user_id: credentials.unix_user_id(),
        process_id,
        executable,
        linux_security_label: credentials.linux_security_label().cloned(),
    })
}
