// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The journal's envelope (engine split D242): one entry is a resource write
//! the engine owns, or a surface write whose type only the host names.
//!
//! Both halves are tagged by variant name and nothing else. Serializing an
//! envelope forwards to the half it holds, so the JSON journal, muniment's
//! canonical digest and the CBOR wire see the same calls, and the same bytes,
//! as the host's own enum would give them. Deserializing reads the tag first
//! and hands the entry to [`ResourceDelta`] when the tag is one of its
//! variants, and to the surface type otherwise. A format that tags enums by
//! position, such as postcard, is refused: positions differ between the halves.

use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, EnumAccess, IntoDeserializer, VariantAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::author::Author;
use super::persistence_edge::PersistedEdge;
use super::resource_record::PersistedResourceRecord;

/// A write to the resource store, in its stable-id replay form. The variant
/// names and fields are the kernel's `CapturedDelta` ones, so stored entries
/// read as either.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceDelta {
    ReplaySetResourceRecordById {
        resource_id: String,
        record: Option<PersistedResourceRecord>,
    },
    ReplaySetResourceEdgesByIds {
        from_resource_id: String,
        to_resource_id: String,
        edges: Vec<PersistedEdge>,
    },
}

impl ResourceDelta {
    /// Every variant's tag: the names that route an entry to this half.
    pub const VARIANTS: &'static [&'static str] =
        &["ReplaySetResourceRecordById", "ReplaySetResourceEdgesByIds"];

    /// Whether `tag` names a resource write.
    pub fn owns(tag: &str) -> bool {
        Self::VARIANTS.contains(&tag)
    }
}

/// One journal delta: a resource write, or a host's surface write `S`.
#[derive(Debug, Clone, PartialEq)]
pub enum JournalDelta<S> {
    Resource(ResourceDelta),
    Surface(S),
}

/// A resource write as a host's journal delta: the engine's own resource
/// writes reach any host's journal through this (engine split D257).
impl<S> From<ResourceDelta> for JournalDelta<S> {
    fn from(delta: ResourceDelta) -> Self {
        Self::Resource(delta)
    }
}

/// A journal entry: who wrote it and what. Field names and order match the
/// kernel's `AttributedDelta`, and so does its rkyv archive.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct Attributed<D> {
    pub author: Author,
    pub delta: D,
}

impl<S: Serialize> Serialize for JournalDelta<S> {
    fn serialize<Z: Serializer>(&self, serializer: Z) -> Result<Z::Ok, Z::Error> {
        match self {
            Self::Resource(delta) => delta.serialize(serializer),
            Self::Surface(delta) => delta.serialize(serializer),
        }
    }
}

impl<'de, S: Deserialize<'de>> Deserialize<'de> for JournalDelta<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_enum("JournalDelta", ResourceDelta::VARIANTS, Route(PhantomData))
    }
}

/// Reads the tag, then routes the rest of the entry by it.
struct Route<S>(PhantomData<S>);

impl<'de, S: Deserialize<'de>> Visitor<'de> for Route<S> {
    type Value = JournalDelta<S>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a journal delta tagged by variant name")
    }

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        let (Tag(tag), variant) = data.variant::<Tag>()?;
        let replay = Replay {
            tag,
            variant,
            de: PhantomData,
        };
        if ResourceDelta::owns(&replay.tag) {
            ResourceDelta::deserialize(replay).map(JournalDelta::Resource)
        } else {
            S::deserialize(replay).map(JournalDelta::Surface)
        }
    }
}

/// A variant name; a positional tag is refused.
struct Tag(String);

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_identifier(TagVisitor)
    }
}

struct TagVisitor;

impl Visitor<'_> for TagVisitor {
    type Value = Tag;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a variant name (journal deltas are tagged by name, not position)")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Tag, E> {
        Ok(Tag(v.to_owned()))
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Tag, E> {
        std::str::from_utf8(v)
            .map(|v| Tag(v.to_owned()))
            .map_err(|_| E::invalid_value(de::Unexpected::Bytes(v), &self))
    }
}

/// The entry again, as an enum whose tag has already been read, for the half
/// it routes to.
struct Replay<'de, A> {
    tag: String,
    variant: A,
    de: PhantomData<&'de ()>,
}

impl<'de, A: VariantAccess<'de>> Deserializer<'de> for Replay<'de, A> {
    type Error = A::Error;

    fn deserialize_any<W: Visitor<'de>>(self, visitor: W) -> Result<W::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

impl<'de, A: VariantAccess<'de>> EnumAccess<'de> for Replay<'de, A> {
    type Error = A::Error;
    type Variant = A;

    fn variant_seed<T: de::DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<(T::Value, A), Self::Error> {
        let tag: de::value::StringDeserializer<A::Error> = self.tag.into_deserializer();
        Ok((seed.deserialize(tag)?, self.variant))
    }
}

#[cfg(test)]
#[path = "journal_delta_tests.rs"]
mod tests;
