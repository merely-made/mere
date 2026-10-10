// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zbus::message::{Header, Message};
use zbus::names::ErrorName;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Signature, Type, Value};
use zbus::{Connection, DBusError};
use zeroize::Zeroizing;

use super::{
    SecretCollectionId, SecretItemId, SecretServiceError, SecretServiceLimits, SecretServiceStore,
};

mod objects;
mod prompt;
mod service;
mod state;
mod vault;

pub use state::{SecretServiceAccessPolicy, SecretServiceCaller, SecretServiceOperation};
pub use vault::SecretServiceVault;

use objects::{CollectionInterface, ItemInterface};
use service::ServiceInterface;
use state::ServiceState;

pub(super) const SERVICE_NAME: &str = "org.freedesktop.secrets";
pub(super) const SERVICE_PATH: &str = "/org/freedesktop/secrets";
pub(super) const ROOT_PATH: &str = "/";
pub(super) const COLLECTION_LABEL_PROPERTY: &str = "org.freedesktop.Secret.Collection.Label";
pub(super) const ITEM_LABEL_PROPERTY: &str = "org.freedesktop.Secret.Item.Label";
pub(super) const ITEM_ATTRIBUTES_PROPERTY: &str = "org.freedesktop.Secret.Item.Attributes";

/// The spec's `Secret` struct: session, parameters, value, content type.
pub(super) type DbusSecret = (OwnedObjectPath, Vec<u8>, SecretBytes, String);
pub(super) type DbusResult<T> = Result<T, SecretDbusError>;

/// A secret's bytes on their way to or from the bus, zeroized when dropped.
/// Encoded as `ay`, exactly as the `Vec<u8>` it replaces. zbus's own message
/// buffers, which hold the encoded bytes, are outside its reach.
pub(super) struct SecretBytes(Zeroizing<Vec<u8>>);

impl SecretBytes {
    pub(super) fn new(bytes: Zeroizing<Vec<u8>>) -> Self {
        Self(bytes)
    }

    /// Move the bytes out without copying; the caller zeroizes them from here.
    pub(super) fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut *self.0)
    }
}

impl Type for SecretBytes {
    const SIGNATURE: &'static Signature = <Vec<u8> as Type>::SIGNATURE;
}

impl Serialize for SecretBytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.as_slice().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SecretBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<u8>::deserialize(deserializer).map(|bytes| Self(Zeroizing::new(bytes)))
    }
}

/// Running Secret Service name and object tree.
pub struct SecretServiceServer {
    connection: Connection,
    lock_watch: tokio::task::JoinHandle<()>,
}

impl Drop for SecretServiceServer {
    fn drop(&mut self) {
        self.lock_watch.abort();
    }
}

impl SecretServiceServer {
    /// Session-bus connection that owns `org.freedesktop.secrets`.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

/// Failure while starting the D-Bus adapter.
#[derive(Debug, thiserror::Error)]
pub enum SecretServiceStartError {
    /// The collection store could not initialize.
    #[error(transparent)]
    Storage(#[from] SecretServiceError),
    /// The session bus, name, or object server could not start.
    #[error("start Secret Service D-Bus adapter: {0}")]
    Dbus(#[from] zbus::Error),
}

/// Serve one persona's collections as Freedesktop Secret Service 0.2.
///
/// The standard bus name is requested without replacement. Startup therefore
/// fails when another keyring already owns the desktop surface. Every
/// collection and item follows `vault`'s lock (vault lock rulings 10, 67,
/// 68); serve while it is unlocked, so the snapshot a locked service answers
/// from has something in it.
pub async fn serve(
    store: SecretServiceStore,
    policy: Arc<dyn SecretServiceAccessPolicy>,
    default_collection_label: &str,
    vault: Arc<dyn SecretServiceVault>,
) -> Result<SecretServiceServer, SecretServiceStartError> {
    store.ensure_default_collection(default_collection_label, now_unix_secs())?;
    let limits = store.limits();
    let state = Arc::new(ServiceState::new(store, limits, policy, vault));
    let connection = zbus::connection::Builder::session()?
        .name(SERVICE_NAME)?
        .serve_at(SERVICE_PATH, ServiceInterface::new(Arc::clone(&state)))?
        .build()
        .await?;

    for collection in state.store.collections()? {
        register_collection(&connection, Arc::clone(&state), collection.id).await?;
        for item in state.store.items(collection.id)? {
            register_item(&connection, Arc::clone(&state), item.id).await?;
        }
    }
    for (alias, collection) in state.store.aliases()? {
        register_alias(&connection, Arc::clone(&state), &alias, collection).await?;
    }

    let lock_watch = watch_lock(connection.clone(), state);
    Ok(SecretServiceServer {
        connection,
        lock_watch,
    })
}

/// Follow the vault: retake the snapshot on each unlock and announce
/// `Locked` on every object at each change.
fn watch_lock(connection: Connection, state: Arc<ServiceState>) -> tokio::task::JoinHandle<()> {
    let mut lock = state.vault.watch();
    tokio::spawn(async move {
        while lock.changed().await.is_ok() {
            let locked = *lock.borrow_and_update();
            if !locked {
                state.refresh_snapshot();
            }
            if let Err(error) = announce_lock(&connection, &state, locked).await {
                tracing::warn!(%error, "Secret Service lock change not announced");
            }
        }
    })
}

/// `PropertiesChanged` for `Locked` on each collection and item, and the
/// service's `CollectionChanged` for each collection.
async fn announce_lock(
    connection: &Connection,
    state: &ServiceState,
    locked: bool,
) -> zbus::Result<()> {
    let snapshot = state.snapshot();
    let changed = HashMap::from([("Locked", Value::from(locked))]);
    let announce = |path: OwnedObjectPath, interface: &'static str| {
        let changed = &changed;
        async move {
            connection
                .emit_signal(
                    None::<()>,
                    path,
                    "org.freedesktop.DBus.Properties",
                    "PropertiesChanged",
                    &(interface, changed, Vec::<String>::new()),
                )
                .await
        }
    };
    let service = zbus::object_server::SignalEmitter::new(connection, SERVICE_PATH)?;
    for collection in snapshot.collections() {
        let path = collection_path(collection.id);
        announce(path.clone(), "org.freedesktop.Secret.Collection").await?;
        service::ServiceInterface::collection_changed(&service, path).await?;
        for item in snapshot.items(collection.id).unwrap_or_default() {
            announce(item_path(item.id), "org.freedesktop.Secret.Item").await?;
        }
    }
    Ok(())
}

async fn register_collection(
    connection: &Connection,
    state: Arc<ServiceState>,
    id: SecretCollectionId,
) -> Result<(), zbus::Error> {
    connection
        .object_server()
        .at(collection_path(id), CollectionInterface::new(state, id))
        .await?;
    Ok(())
}

async fn register_item(
    connection: &Connection,
    state: Arc<ServiceState>,
    id: SecretItemId,
) -> Result<(), zbus::Error> {
    connection
        .object_server()
        .at(item_path(id), ItemInterface::new(state, id))
        .await?;
    Ok(())
}

async fn register_alias(
    connection: &Connection,
    state: Arc<ServiceState>,
    alias: &str,
    id: SecretCollectionId,
) -> Result<(), zbus::Error> {
    connection
        .object_server()
        .at(alias_path(alias)?, CollectionInterface::new(state, id))
        .await?;
    Ok(())
}

pub(super) fn collection_path(id: SecretCollectionId) -> OwnedObjectPath {
    OwnedObjectPath::try_from(format!(
        "{SERVICE_PATH}/collection/{}",
        id.as_uuid().simple()
    ))
    .expect("UUID collection path is valid")
}

pub(super) fn item_path(id: SecretItemId) -> OwnedObjectPath {
    OwnedObjectPath::try_from(format!("{SERVICE_PATH}/item/{}", id.as_uuid().simple()))
        .expect("UUID item path is valid")
}

pub(super) fn alias_path(alias: &str) -> DbusResult<OwnedObjectPath> {
    OwnedObjectPath::try_from(format!("{SERVICE_PATH}/aliases/{alias}")).map_err(|_| {
        SecretDbusError::InvalidArgs(format!(
            "collection alias {alias:?} is not a valid D-Bus object-path element"
        ))
    })
}

pub(super) fn root_path() -> OwnedObjectPath {
    OwnedObjectPath::try_from(ROOT_PATH).expect("root object path is valid")
}

pub(super) fn parse_collection_path(path: &OwnedObjectPath) -> Option<SecretCollectionId> {
    parse_uuid_path(path, &format!("{SERVICE_PATH}/collection/")).map(SecretCollectionId::from_uuid)
}

pub(super) fn parse_item_path(path: &OwnedObjectPath) -> Option<SecretItemId> {
    parse_uuid_path(path, &format!("{SERVICE_PATH}/item/")).map(SecretItemId::from_uuid)
}

fn parse_uuid_path(path: &OwnedObjectPath, prefix: &str) -> Option<uuid::Uuid> {
    path.as_str().strip_prefix(prefix)?.parse().ok()
}

pub(super) fn property_string(
    properties: &HashMap<String, OwnedValue>,
    key: &'static str,
) -> DbusResult<String> {
    properties
        .get(key)
        .ok_or_else(|| SecretDbusError::InvalidArgs(format!("missing property {key}")))
        .and_then(|value| {
            String::try_from(&**value).map_err(|_| {
                SecretDbusError::InvalidArgs(format!("property {key} must be a string"))
            })
        })
}

pub(super) fn property_attributes(
    properties: &HashMap<String, OwnedValue>,
) -> DbusResult<HashMap<String, String>> {
    let value = properties
        .get(ITEM_ATTRIBUTES_PROPERTY)
        .ok_or_else(|| {
            SecretDbusError::InvalidArgs(format!("missing property {ITEM_ATTRIBUTES_PROPERTY}"))
        })?
        .try_clone()
        .map_err(SecretDbusError::from)?;
    value.try_into().map_err(|_| {
        SecretDbusError::InvalidArgs(format!(
            "property {ITEM_ATTRIBUTES_PROPERTY} must be a string dictionary"
        ))
    })
}

pub(super) fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[derive(Debug)]
pub(super) enum SecretDbusError {
    IsLocked(String),
    NoSession(String),
    NoSuchObject(String),
    AccessDenied(String),
    NotSupported(String),
    InvalidArgs(String),
    LimitsExceeded(String),
    Failed(String),
}

impl fmt::Display for SecretDbusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.description().unwrap_or("Secret Service error"))
    }
}

impl std::error::Error for SecretDbusError {}

impl DBusError for SecretDbusError {
    fn create_reply(&self, call: &Header<'_>) -> zbus::Result<Message> {
        Message::error(call, self.name())?.build(&(self.description().unwrap_or_default(),))
    }

    fn name(&self) -> ErrorName<'_> {
        let name = match self {
            Self::IsLocked(_) => "org.freedesktop.Secret.Error.IsLocked",
            Self::NoSession(_) => "org.freedesktop.Secret.Error.NoSession",
            Self::NoSuchObject(_) => "org.freedesktop.Secret.Error.NoSuchObject",
            Self::AccessDenied(_) => "org.freedesktop.DBus.Error.AccessDenied",
            Self::NotSupported(_) => "org.freedesktop.DBus.Error.NotSupported",
            Self::InvalidArgs(_) => "org.freedesktop.DBus.Error.InvalidArgs",
            Self::LimitsExceeded(_) => "org.freedesktop.DBus.Error.LimitsExceeded",
            Self::Failed(_) => "org.freedesktop.DBus.Error.Failed",
        };
        ErrorName::from_static_str_unchecked(name)
    }

    fn description(&self) -> Option<&str> {
        let description = match self {
            Self::IsLocked(description)
            | Self::NoSession(description)
            | Self::NoSuchObject(description)
            | Self::AccessDenied(description)
            | Self::NotSupported(description)
            | Self::InvalidArgs(description)
            | Self::LimitsExceeded(description)
            | Self::Failed(description) => description,
        };
        Some(description)
    }
}

/// The snapshot's lookups (chatelaine, ruling D30), as the store's own.
impl From<chatelaine::MetadataLookupError> for SecretDbusError {
    fn from(error: chatelaine::MetadataLookupError) -> Self {
        SecretServiceError::from(error).into()
    }
}

impl From<SecretServiceError> for SecretDbusError {
    fn from(error: SecretServiceError) -> Self {
        match error {
            SecretServiceError::CollectionNotFound(_) | SecretServiceError::ItemNotFound(_) => {
                Self::NoSuchObject(error.to_string())
            },
            SecretServiceError::Limit(_) => Self::LimitsExceeded(error.to_string()),
            SecretServiceError::InvalidText(_) => Self::InvalidArgs(error.to_string()),
            _ => Self::Failed(error.to_string()),
        }
    }
}

impl From<zbus::Error> for SecretDbusError {
    fn from(error: zbus::Error) -> Self {
        Self::Failed(error.to_string())
    }
}

impl From<zbus::fdo::Error> for SecretDbusError {
    fn from(error: zbus::fdo::Error) -> Self {
        Self::Failed(error.to_string())
    }
}

impl From<zbus::zvariant::Error> for SecretDbusError {
    fn from(error: zbus::zvariant::Error) -> Self {
        Self::InvalidArgs(error.to_string())
    }
}

impl From<SecretDbusError> for zbus::fdo::Error {
    fn from(error: SecretDbusError) -> Self {
        let description = error.to_string();
        match error {
            SecretDbusError::AccessDenied(_) => Self::AccessDenied(description),
            SecretDbusError::NotSupported(_) => Self::NotSupported(description),
            SecretDbusError::InvalidArgs(_) => Self::InvalidArgs(description),
            SecretDbusError::LimitsExceeded(_) => Self::LimitsExceeded(description),
            SecretDbusError::IsLocked(_)
            | SecretDbusError::NoSession(_)
            | SecretDbusError::NoSuchObject(_)
            | SecretDbusError::Failed(_) => Self::Failed(description),
        }
    }
}

impl From<SecretServiceError> for zbus::fdo::Error {
    fn from(error: SecretServiceError) -> Self {
        SecretDbusError::from(error).into()
    }
}

impl From<SecretDbusError> for zbus::Error {
    fn from(error: SecretDbusError) -> Self {
        Self::FDO(Box::new(error.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compiled and run on Linux only, where zbus is.
    #[test]
    fn secret_bytes_keep_the_secret_struct_signature() {
        assert_eq!(<SecretBytes as Type>::SIGNATURE.to_string(), "ay");
        assert_eq!(<DbusSecret as Type>::SIGNATURE.to_string(), "(oayays)");
    }
}
