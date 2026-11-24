// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The rkyv adapter that archives a [`Uuid`] as its 16 bytes. Moved from the
//! kernel's identity module so model records can name a UUID too.

use rkyv::{
    Archive, Archived, Deserialize, Place, Resolver, Serialize,
    rancor::Fallible,
    with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use uuid::Uuid;

/// `#[rkyv(with = UuidAsBytes)]` on a [`Uuid`] field.
pub struct UuidAsBytes;

impl ArchiveWith<Uuid> for UuidAsBytes {
    type Archived = Archived<[u8; 16]>;
    type Resolver = Resolver<[u8; 16]>;

    fn resolve_with(field: &Uuid, resolver: Self::Resolver, out: Place<Self::Archived>) {
        let bytes = *field.as_bytes();
        bytes.resolve(resolver, out);
    }
}

impl<S> SerializeWith<Uuid, S> for UuidAsBytes
where
    S: Fallible + ?Sized,
    [u8; 16]: Serialize<S>,
{
    fn serialize_with(field: &Uuid, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let bytes = *field.as_bytes();
        bytes.serialize(serializer)
    }
}

impl<D> DeserializeWith<Archived<[u8; 16]>, Uuid, D> for UuidAsBytes
where
    D: Fallible + ?Sized,
    Archived<[u8; 16]>: Deserialize<[u8; 16], D>,
{
    fn deserialize_with(
        field: &Archived<[u8; 16]>,
        deserializer: &mut D,
    ) -> Result<Uuid, D::Error> {
        let bytes = field.deserialize(deserializer)?;
        Ok(Uuid::from_bytes(bytes))
    }
}
