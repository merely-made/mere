// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A predicate field nested in a stored record (declared-lens D28, D38): the
//! UUIDv5 of the IRI, and the IRI text only when the core pack lacks the
//! term, as `{"uuid": .., "iri": ..}`. In memory the field stays the IRI. A
//! field stored before D28 holds the IRI text alone and still reads; a UUID
//! that is not the IRI's, or one the core pack cannot resolve with no IRI
//! beside it, is refused. The engine's own records are flat
//! (`predicate_uuid`, `predicate`); hosts that nest the pair use this.
//!
//! `#[serde(with = "mere::model::relation::stored::nested")]` on a `String`,
//! `nested::option` on an `Option<String>`.

use std::fmt;

use super::{predicate_uuid, read_predicate, stored_iri};
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use uuid::Uuid;

const FIELDS: &[&str] = &["uuid", "iri"];

pub fn serialize<S: Serializer>(predicate: &str, serializer: S) -> Result<S::Ok, S::Error> {
    let human = serializer.is_human_readable();
    let iri = stored_iri(predicate);
    let len = 1 + usize::from(!human || iri.is_some());
    let mut state = serializer.serialize_struct("StoredPredicate", len)?;
    state.serialize_field("uuid", &predicate_uuid(predicate))?;
    if !human || iri.is_some() {
        state.serialize_field("iri", &iri)?;
    }
    state.end()
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    read(deserializer, |iri| iri)
}

/// Read a stored predicate; `old` turns text stored before D28 (an IRI, or
/// for some fields an old name) into the IRI.
pub fn read<'de, D: Deserializer<'de>>(
    deserializer: D,
    old: fn(String) -> String,
) -> Result<String, D::Error> {
    if deserializer.is_human_readable() {
        deserializer.deserialize_any(StoredVisitor { old })
    } else {
        deserializer.deserialize_struct("StoredPredicate", FIELDS, StoredVisitor { old })
    }
}

struct StoredVisitor {
    old: fn(String) -> String,
}

impl<'de> Visitor<'de> for StoredVisitor {
    type Value = String;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a predicate's UUID and IRI, or an IRI")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<String, E> {
        Ok((self.old)(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<String, E> {
        Ok((self.old)(value))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<String, A::Error> {
        let uuid: Uuid = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let iri: Option<String> = seq.next_element()?.unwrap_or(None);
        read_predicate(Some(uuid), iri).map_err(de::Error::custom)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<String, A::Error> {
        let (mut uuid, mut iri) = (None, None);
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "uuid" => uuid = Some(map.next_value::<Uuid>()?),
                "iri" => iri = map.next_value::<Option<String>>()?,
                other => return Err(de::Error::unknown_field(other, FIELDS)),
            }
        }
        read_predicate(uuid, iri).map_err(de::Error::custom)
    }
}

/// An optional stored predicate.
pub mod option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    struct Stored<'a>(&'a str);

    impl Serialize for Stored<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            super::serialize(self.0, serializer)
        }
    }

    struct Read(String);

    impl<'de> Deserialize<'de> for Read {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            super::deserialize(deserializer).map(Read)
        }
    }

    pub fn serialize<S: Serializer>(
        predicate: &Option<String>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        predicate.as_deref().map(Stored).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<String>, D::Error> {
        Ok(Option::<Read>::deserialize(deserializer)?.map(|read| read.0))
    }
}

#[cfg(test)]
#[path = "stored_nested_tests.rs"]
mod tests;
