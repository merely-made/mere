// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Preserved Card JSON, with validation of the mapped RFC 9553 shapes.

use iri_string::types::UriStr;
use serde_json::{Map, Value};

use super::{ExchangeError, invalid, strict_json};

/// A JSContact Card retaining original map ids and unmapped JSON properties.
///
/// Validation covers metadata, name, nicknames, online services, crypto resources,
/// email and phone shapes. Other standard properties are retained opaquely;
/// this is not a full RFC 9553 schema validator. Parsing never dereferences URIs.
#[derive(Clone, Debug, PartialEq)]
pub struct Card(Value);

impl Card {
    /// Parse one Card, refusing duplicate object names at every depth and
    /// trailing input. JSON's normal recursion limit remains enabled.
    pub fn parse(bytes: &[u8]) -> Result<Self, ExchangeError> {
        Self::from_value(strict_json::parse(bytes)?)
    }

    /// Validate an already-parsed value. Duplicate names cannot be recovered
    /// from a Value; prefer `parse` at an untrusted byte boundary.
    pub fn from_value(value: Value) -> Result<Self, ExchangeError> {
        validate(&value)?;
        Ok(Self(value))
    }

    /// Serialize without losing unknown fields or changing original map ids.
    /// This preserves JSON values, not whitespace or property ordering.
    pub fn to_json(&self) -> Result<Vec<u8>, ExchangeError> {
        serde_json::to_vec(&self.0).map_err(|error| invalid(error.to_string()))
    }

    /// The source exchange object, including properties Gaz does not map.
    pub fn as_value(&self) -> &Value {
        &self.0
    }

    /// The original identifier. JSContact permits URIs and free text here.
    pub fn uid(&self) -> &str {
        self.0["uid"].as_str().expect("validated uid")
    }
}

pub(super) fn uri(value: &str) -> bool {
    UriStr::new(value).is_ok()
}

pub(super) fn object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a Map<String, Value>, ExchangeError> {
    value
        .as_object()
        .ok_or_else(|| invalid(format!("{field} must be an object")))
}

pub(super) fn text<'a>(map: &'a Map<String, Value>, field: &str) -> Result<&'a str, ExchangeError> {
    map.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("{field} must be a string")))
}

pub(super) fn optional_text<'a>(
    map: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, ExchangeError> {
    map.get(field).map(|_| text(map, field)).transpose()
}

type Entries<'a> = Vec<(&'a str, &'a Map<String, Value>)>;

pub(super) fn entries<'a>(card: &'a Value, field: &str) -> Result<Entries<'a>, ExchangeError> {
    let Some(value) = card.get(field) else {
        return Ok(Vec::new());
    };
    object(value, field)?
        .iter()
        .map(|(id, value)| {
            if id.is_empty()
                || id.len() > 255
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(invalid(format!("invalid Id in {field}")));
            }
            Ok((id.as_str(), object(value, field)?))
        })
        .collect()
}

fn shape(map: &Map<String, Value>, kind: &str, known: &[&str]) -> Result<(), ExchangeError> {
    for name in map.keys() {
        if name == "extra" {
            return Err(invalid("extra is reserved by JSContact"));
        }
        if !name.contains(':') && !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'@') {
            return Err(invalid("invalid JSContact property name"));
        }
        if known
            .iter()
            .any(|known| name.eq_ignore_ascii_case(known) && name != known)
        {
            return Err(invalid(format!("wrong case for {name}")));
        }
    }
    if optional_text(map, "@type")?.is_some_and(|value| value != kind) {
        return Err(invalid(format!("@type must be {kind}")));
    }
    Ok(())
}

fn common(map: &Map<String, Value>) -> Result<(), ExchangeError> {
    optional_text(map, "label")?;
    if let Some(pref) = map.get("pref")
        && !pref.as_u64().is_some_and(|n| (1..=100).contains(&n))
    {
        return Err(invalid("pref must be an integer from 1 through 100"));
    }
    if let Some(contexts) = map.get("contexts")
        && object(contexts, "contexts")?
            .values()
            .any(|v| !v.is_boolean())
    {
        return Err(invalid("contexts must contain boolean values"));
    }
    Ok(())
}

fn validate(value: &Value) -> Result<(), ExchangeError> {
    let card = object(value, "Card")?;
    shape(
        card,
        "Card",
        &[
            "@type",
            "version",
            "uid",
            "kind",
            "name",
            "nicknames",
            "onlineServices",
            "emails",
            "phones",
            "cryptoKeys",
            "language",
            "created",
            "updated",
            "prodId",
            "relatedTo",
            "members",
            "organizations",
            "speakToAs",
            "titles",
            "preferredLanguages",
            "calendars",
            "schedulingAddresses",
            "addresses",
            "directories",
            "links",
            "media",
            "localizations",
            "anniversaries",
            "keywords",
            "notes",
            "personalInfo",
        ],
    )?;
    if text(card, "@type")? != "Card" {
        return Err(invalid("@type must be Card"));
    }
    let version = text(card, "version")?;
    let Some((major, minor)) = version.split_once('.') else {
        return Err(invalid("invalid version"));
    };
    if major.is_empty()
        || !major.bytes().all(|b| b.is_ascii_digit())
        || major.parse::<u64>().ok() != Some(1)
        || minor.is_empty()
        || !minor.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid("unsupported JSContact major version"));
    }
    text(card, "uid")?;
    if let Some(kind) = optional_text(card, "kind")? {
        let known = [
            "individual",
            "group",
            "org",
            "location",
            "device",
            "application",
        ];
        if known
            .iter()
            .any(|k| kind.eq_ignore_ascii_case(k) && kind != *k)
        {
            return Err(invalid("wrong case for kind"));
        }
    }
    if let Some(name) = card.get("name") {
        validate_name(object(name, "name")?)?;
    }
    for (_, item) in entries(value, "nicknames")? {
        shape(item, "Nickname", &["@type", "name", "contexts", "pref"])?;
        text(item, "name")?;
        common(item)?;
    }
    for (_, item) in entries(value, "onlineServices")? {
        shape(
            item,
            "OnlineService",
            &[
                "@type", "service", "uri", "user", "contexts", "pref", "label",
            ],
        )?;
        let address = optional_text(item, "uri")?;
        let user = optional_text(item, "user")?;
        if address.is_none() && user.is_none() {
            return Err(invalid("OnlineService requires uri or user"));
        }
        if address.is_some_and(|s| !uri(s)) {
            return Err(invalid("invalid OnlineService URI"));
        }
        optional_text(item, "service")?;
        common(item)?;
    }
    for (_, item) in entries(value, "cryptoKeys")? {
        shape(
            item,
            "CryptoKey",
            &["@type", "uri", "mediaType", "contexts", "pref", "label"],
        )?;
        if !uri(text(item, "uri")?) {
            return Err(invalid("invalid CryptoKey URI"));
        }
        optional_text(item, "mediaType")?;
        common(item)?;
    }
    for (field, kind, required) in [
        ("emails", "EmailAddress", "address"),
        ("phones", "Phone", "number"),
    ] {
        for (_, item) in entries(value, field)? {
            shape(
                item,
                kind,
                &["@type", required, "contexts", "pref", "label", "features"],
            )?;
            text(item, required)?;
            common(item)?;
        }
    }
    Ok(())
}

fn validate_name(name: &Map<String, Value>) -> Result<(), ExchangeError> {
    shape(
        name,
        "Name",
        &[
            "@type",
            "full",
            "components",
            "isOrdered",
            "defaultSeparator",
            "sortAs",
            "phoneticScript",
            "phoneticSystem",
        ],
    )?;
    let full = optional_text(name, "full")?;
    optional_text(name, "phoneticScript")?;
    optional_text(name, "phoneticSystem")?;
    let ordered = match name.get("isOrdered") {
        None => false,
        Some(value) => value
            .as_bool()
            .ok_or_else(|| invalid("isOrdered must be boolean"))?,
    };
    let separator = optional_text(name, "defaultSeparator")?;
    let components = name
        .get("components")
        .map(|c| {
            c.as_array()
                .ok_or_else(|| invalid("name components must be an array"))
        })
        .transpose()?;
    if full.is_none() && components.is_none() {
        return Err(invalid("name requires full or components"));
    }
    if separator.is_some() && (!ordered || components.is_none()) {
        return Err(invalid("defaultSeparator requires ordered components"));
    }
    if let Some(components) = components {
        let mut substantive = false;
        let mut previous_separator = false;
        for component in components {
            let component = object(component, "NameComponent")?;
            shape(
                component,
                "NameComponent",
                &["@type", "kind", "value", "phonetic"],
            )?;
            let kind = text(component, "kind")?;
            text(component, "value")?;
            optional_text(component, "phonetic")?;
            let known = [
                "title",
                "given",
                "given2",
                "surname",
                "surname2",
                "credential",
                "generation",
                "separator",
            ];
            if known
                .iter()
                .any(|k| kind.eq_ignore_ascii_case(k) && kind != *k)
            {
                return Err(invalid("wrong case for name component kind"));
            }
            if kind == "separator" {
                if !ordered || previous_separator {
                    return Err(invalid("invalid name separator"));
                }
            } else {
                substantive = true;
            }
            previous_separator = kind == "separator";
        }
        if !substantive {
            return Err(invalid("name components require a non-separator entry"));
        }
    }
    Ok(())
}
