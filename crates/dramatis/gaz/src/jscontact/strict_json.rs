// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! JSON intake with duplicate-name refusal, including nested opaque extensions.

use core::fmt;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};

use super::{ExchangeError, invalid};

pub(super) fn parse(bytes: &[u8]) -> Result<Value, ExchangeError> {
    serde_json::from_slice::<StrictValue>(bytes)
        .map(|value| value.0)
        .map_err(|error| invalid(error.to_string()))
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StrictVisitor)
    }
}

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = StrictValue;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON without duplicate object names")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(v)))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(v.into())))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(v.into())))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Number::from_f64(v)
            .map(|v| StrictValue(Value::Number(v)))
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(v.to_owned())))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(v)))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON object name"));
            }
            values.insert(key, map.next_value::<StrictValue>()?.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}
