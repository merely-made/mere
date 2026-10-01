// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Items, their credentials, and the collections that group them, shaped as
//! CXF shapes them (ruling 16).
//!
//! Timestamps are Unix seconds the caller supplies, as CXF and the Secret
//! Service both count them; chatelaine reads no clock.

use serde::{Deserialize, Serialize};

use crate::id::{CollectionId, CredentialId, ItemId};
use crate::kind::CredentialKind;
use crate::value::SourceId;

/// A titled container of typed credentials.
///
/// Every field here may be shown. The secrets the credentials stand for are
/// sealed by castellan, one payload per credential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// The item's id.
    pub id: ItemId,
    /// Its id in the CXF file it was imported from, if it came from one.
    pub source_id: Option<SourceId>,
    /// The name the user knows it by.
    pub title: String,
    /// A further description.
    pub subtitle: Option<String>,
    /// Where its credentials are meant to be used.
    pub scope: Option<Scope>,
    /// User-defined tags.
    pub tags: Vec<String>,
    /// Whether the user marked it a favorite.
    pub favorite: bool,
    /// When it was created, in Unix seconds.
    pub created_at: Option<u64>,
    /// When it was last changed, in Unix seconds.
    pub modified_at: Option<u64>,
    /// Its credentials, each a kind with its metadata.
    pub credentials: Vec<Credential>,
    /// In the vault, or waiting for the user's review.
    pub state: ItemState,
}

/// One credential of an item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    /// The credential's id, which names its sealed payload.
    pub id: CredentialId,
    /// What it is, with its secret-free metadata.
    pub kind: CredentialKind,
}

/// Whether an item may be exercised.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemState {
    /// Accepted: castellan may fill, release or show it.
    Vault,
    /// Imported but not yet accepted: sealed, and never filled, released or
    /// shown until the user accepts it (invariant 12).
    Quarantined,
}

/// Where an item's credentials are meant to be used: CXF's
/// `CredentialScope`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    /// Sites, as URIs.
    pub urls: Vec<String>,
    /// Android apps.
    pub android_apps: Vec<AndroidApp>,
}

/// An Android app an item's credentials may fill.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AndroidApp {
    /// The application id, such as `com.example.app`.
    pub bundle_id: String,
    /// The fingerprint of the app's signing certificate.
    pub certificate: Option<AppCertificate>,
    /// The app's display name.
    pub name: Option<String>,
}

/// An Android signing certificate's fingerprint: a hash of a public
/// certificate, so not a secret.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppCertificate {
    /// The hash bytes.
    pub fingerprint: Vec<u8>,
    /// The hash, as CXF names it (`sha256` or `sha512`).
    pub hash_algorithm: String,
}

/// A reference to an item, as an `item-reference` credential and a
/// collection's members carry one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Link {
    /// The item referred to.
    pub item: ItemId,
}

/// A named group of items, which may nest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    /// The collection's id.
    pub id: CollectionId,
    /// Its id in the CXF file it was imported from, if it came from one.
    pub source_id: Option<SourceId>,
    /// Its name.
    pub title: String,
    /// A further description.
    pub subtitle: Option<String>,
    /// When it was created, in Unix seconds.
    pub created_at: Option<u64>,
    /// When it was last changed, in Unix seconds.
    pub modified_at: Option<u64>,
    /// The items it holds.
    pub items: Vec<Link>,
    /// Collections nested inside it.
    pub sub_collections: Vec<Collection>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::samples;

    fn round_trip<T>(value: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + core::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value, "JSON");
        let bytes = postcard::to_allocvec(value).unwrap();
        assert_eq!(
            &postcard::from_bytes::<T>(&bytes).unwrap(),
            value,
            "postcard"
        );
    }

    #[test]
    fn an_item_holding_every_kind_round_trips() {
        let item = samples::item(samples::every_kind());
        assert_eq!(item.credentials.len(), 19, "17 CXF kinds, Secret, Unknown");
        round_trip(&item);
    }

    #[test]
    fn a_quarantined_item_round_trips_with_its_state() {
        let mut item = samples::item(vec![samples::credit_card()]);
        item.state = ItemState::Quarantined;
        round_trip(&item);
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["state"], "quarantined");
    }

    #[test]
    fn an_item_keeps_its_cxf_source_id() {
        let item = samples::item(vec![samples::note()]);
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["source_id"], "ZmllbGQx");
        round_trip(&item);
    }

    #[test]
    fn an_item_without_source_scope_or_timestamps_round_trips() {
        let mut item = samples::item(vec![samples::note()]);
        item.source_id = None;
        item.scope = None;
        item.subtitle = None;
        item.created_at = None;
        item.modified_at = None;
        round_trip(&item);
    }

    #[test]
    fn collections_nest_and_round_trip() {
        let leaf = Collection {
            id: CollectionId::from_random([3; 16]),
            source_id: None,
            title: "Banking".to_string(),
            subtitle: None,
            created_at: None,
            modified_at: None,
            items: vec![Link {
                item: samples::item_id(1),
            }],
            sub_collections: Vec::new(),
        };
        let root = Collection {
            id: CollectionId::from_random([4; 16]),
            source_id: Some(SourceId::parse("Y29sbGVjdGlvbi0x").unwrap()),
            title: "Household".to_string(),
            subtitle: Some("Bills and accounts".to_string()),
            created_at: Some(1_790_000_000),
            modified_at: Some(1_790_000_100),
            items: vec![Link {
                item: samples::item_id(2),
            }],
            sub_collections: vec![leaf.clone()],
        };
        round_trip(&root);
        assert_eq!(root.sub_collections[0], leaf);
        let json = serde_json::to_value(&root).unwrap();
        assert_eq!(json["source_id"], "Y29sbGVjdGlvbi0x");
        assert!(json["sub_collections"][0]["source_id"].is_null());
    }
}
