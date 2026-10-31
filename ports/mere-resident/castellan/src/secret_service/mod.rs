// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [Freedesktop Secret Service 0.2] over Castellan's resident sealed storage.
//!
//! The portable types, the collection store and the transfer-session table
//! live on every target so their policy and persistence can be tested without
//! a bus. The store is a view of the persona's chatelaine items
//! ([`crate::items::ItemStore`], ruling 17). [`serve`] is the Linux
//! session-bus adapter. It implements the standard `org.freedesktop.secrets`
//! object tree and the strongly recommended `plain` transfer session.
//!
//! [Freedesktop Secret Service 0.2]: https://specifications.freedesktop.org/secret-service/latest/

#[cfg(any(test, target_os = "linux"))]
mod sessions;
mod store;

#[cfg(target_os = "linux")]
mod dbus;

pub use store::{
    MetadataSnapshot, NewSecretItem, SecretCollection, SecretCollectionId, SecretItem,
    SecretItemId, SecretServiceError, SecretServiceLimits, SecretServiceStore,
};

#[cfg(target_os = "linux")]
pub use dbus::{
    SecretServiceAccessPolicy, SecretServiceCaller, SecretServiceOperation, SecretServiceServer,
    SecretServiceStartError, SecretServiceVault, serve,
};
