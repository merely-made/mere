// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use zbus::Connection;
use zbus::message::Header;
use zbus::names::BusName;
use zbus::zvariant::OwnedObjectPath;

use super::{
    DbusResult, SecretDbusError, SecretServiceLimits, SecretServiceStore, item_path,
    parse_collection_path, parse_item_path,
};
use crate::secret_service::sessions::{SessionRefusal, SessionTable};
use crate::secret_service::{SecretCollectionId, SecretItemId};

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
    policy: Arc<dyn SecretServiceAccessPolicy>,
    sessions: SessionTable,
    locked_collections: Mutex<BTreeSet<SecretCollectionId>>,
    locked_items: Mutex<BTreeSet<SecretItemId>>,
}

impl ServiceState {
    pub(super) fn new(
        store: SecretServiceStore,
        limits: SecretServiceLimits,
        policy: Arc<dyn SecretServiceAccessPolicy>,
    ) -> Self {
        Self {
            store,
            policy,
            sessions: SessionTable::new(limits.max_sessions),
            locked_collections: Mutex::new(BTreeSet::new()),
            locked_items: Mutex::new(BTreeSet::new()),
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

    pub(super) fn collection_locked(&self, id: SecretCollectionId) -> bool {
        self.locked_collections.lock().unwrap().contains(&id)
    }

    pub(super) fn item_locked(&self, id: SecretItemId) -> DbusResult<bool> {
        if self.locked_items.lock().unwrap().contains(&id) {
            return Ok(true);
        }
        let item = self.store.item(id)?;
        Ok(self.collection_locked(item.collection))
    }

    pub(super) fn require_collection_unlocked(&self, id: SecretCollectionId) -> DbusResult<()> {
        if self.collection_locked(id) {
            Err(SecretDbusError::IsLocked(format!(
                "collection {id} is locked"
            )))
        } else {
            Ok(())
        }
    }

    pub(super) fn require_item_unlocked(&self, id: SecretItemId) -> DbusResult<()> {
        if self.item_locked(id)? {
            Err(SecretDbusError::IsLocked(format!("item {id} is locked")))
        } else {
            Ok(())
        }
    }

    pub(super) fn lock_object(&self, path: &OwnedObjectPath) -> DbusResult<bool> {
        if let Some(id) = parse_collection_path(path) {
            self.store.collection(id)?;
            self.locked_collections.lock().unwrap().insert(id);
            return Ok(true);
        }
        if let Some(id) = parse_item_path(path) {
            self.store.item(id)?;
            self.locked_items.lock().unwrap().insert(id);
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn unlock_object(&self, path: &OwnedObjectPath) -> DbusResult<bool> {
        if let Some(id) = parse_collection_path(path) {
            self.store.collection(id)?;
            self.locked_collections.lock().unwrap().remove(&id);
            return Ok(true);
        }
        if let Some(id) = parse_item_path(path) {
            self.store.item(id)?;
            self.locked_items.lock().unwrap().remove(&id);
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn split_locked(
        &self,
        items: impl IntoIterator<Item = SecretItemId>,
    ) -> DbusResult<(Vec<OwnedObjectPath>, Vec<OwnedObjectPath>)> {
        let mut unlocked = Vec::new();
        let mut locked = Vec::new();
        for id in items {
            if self.item_locked(id)? {
                locked.push(item_path(id));
            } else {
                unlocked.push(item_path(id));
            }
        }
        Ok((unlocked, locked))
    }

    pub(super) fn forget_item(&self, id: SecretItemId) {
        self.locked_items.lock().unwrap().remove(&id);
    }

    pub(super) fn forget_collection(
        &self,
        id: SecretCollectionId,
        items: impl IntoIterator<Item = SecretItemId>,
    ) {
        self.locked_collections.lock().unwrap().remove(&id);
        let mut locked_items = self.locked_items.lock().unwrap();
        for item in items {
            locked_items.remove(&item);
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
