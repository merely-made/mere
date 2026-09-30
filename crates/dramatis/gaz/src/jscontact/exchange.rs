// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Public projections and explicit private restoration.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::card::{entries, optional_text, text, uri};
use super::{
    Card, ExchangeError, IdentityClaim, ImportedCard, JsContactFormat, PublicCard, invalid,
};
use crate::{
    Anchor, Contact, ContactTier, Endpoint, EndpointKind, Handle, HandleKind, LocalId, TypedKey,
};

impl IdentityClaim {
    fn contact(
        &self,
        name: &str,
        handles: Vec<Handle>,
        endpoints: Vec<Endpoint>,
    ) -> Result<Contact, ExchangeError> {
        // Contact's existing deserializer owns all anchor/proof/key invariants.
        // Exchange introduces neither another validator nor a trusted bypass.
        serde_json::from_value(json!({
            "petname":name,"anchor":self.anchor,"root_line":self.root_line,"attested":self.attested,
            "handles":handles,"endpoints":endpoints,"tier":ContactTier::Kith,
            "last_contact_ms":null,"note":null,
        }))
        .map_err(|error| invalid(format!("invalid identity claim: {error}")))
    }

    fn of(contact: &Contact) -> Self {
        Self {
            anchor: contact.anchor().clone(),
            root_line: contact.root_line().to_vec(),
            attested: contact.attested().to_vec(),
        }
    }
}

impl JsContactFormat {
    /// Export a LOSSLESS PRIVATE record, including note, trust, tier and history.
    ///
    /// Intended for personal backup/transfer, never persona publication. The
    /// local Contact JSON is stored as text inside a versioned vendor property:
    /// its u64 timestamps remain exact without imposing them as JSContact Ints.
    pub fn export_contact(&self, contact: &Contact) -> Result<Card, ExchangeError> {
        let mut card = self.project(contact)?;
        card[self.property("gazLocal")] = json!({
            "version":"1", "contactJson":serde_json::to_string(contact).map_err(|e| invalid(e.to_string()))?,
        });
        Card::from_value(card)
    }

    /// Build the card a persona publishes from explicitly selected public fields.
    /// No book, persona scope, private note, trust, tier or recency is included.
    pub fn publish(&self, public: &PublicCard) -> Result<Card, ExchangeError> {
        let handles = public
            .handles
            .iter()
            .map(|h| Handle::new(h.kind.clone(), &h.value))
            .collect();
        let endpoints = public
            .endpoints
            .iter()
            .map(|e| Endpoint::new(e.kind.clone(), &e.address))
            .collect();
        let contact = public.identity.contact(&public.name, handles, endpoints)?;
        Card::from_value(self.project(&contact)?)
    }

    /// Restore a private export, including local trust and private relationships.
    ///
    /// The caller must restrict this path to its own trusted backup. A peer
    /// cannot choose this mode through a Card property. Public projections must
    /// still agree with the validated local record; disagreements refuse whole.
    pub fn restore_contact(&self, card: &Card) -> Result<Contact, ExchangeError> {
        let local = card
            .as_value()
            .get(self.property("gazLocal"))
            .ok_or_else(|| invalid("private local record is absent"))?;
        let local = super::card::object(local, "gazLocal")?;
        if text(local, "version")? != "1" {
            return Err(invalid("unsupported local record version"));
        }
        let contact: Contact = serde_json::from_str(text(local, "contactJson")?)
            .map_err(|error| invalid(format!("invalid local record: {error}")))?;
        let projected = self.project(&contact)?;
        for field in [
            "uid",
            "name",
            "onlineServices",
            "cryptoKeys",
            self.property("gazIdentity").as_str(),
        ] {
            if card.as_value().get(field) != projected.get(field) {
                return Err(invalid(format!("private record disagrees with {field}")));
            }
        }
        Ok(contact)
    }

    /// Turn a peer's card into unverified claims and retain the original Card.
    ///
    /// Local vendor state is ignored even when it advertises Verified or Kin.
    /// A non-anchor uid requires an optional caller-chosen local id; reuse that
    /// id for repeat imports. PLC uids require a structural identity claim with
    /// a root key. No fetching, signature checks, or updates to a book occur.
    pub fn import(
        &self,
        card: &Card,
        local: Option<LocalId>,
    ) -> Result<ImportedCard, ExchangeError> {
        let source = card.as_value();
        let name = display_name(source, card.uid());
        let (handles, endpoints) = self.addresses(source)?;
        let contact = if let Some(value) = source.get(self.property("gazIdentity")) {
            let value = super::card::object(value, "gazIdentity")?;
            if text(value, "version")? != "1" {
                return Err(invalid("unsupported identity claim version"));
            }
            let identity: IdentityClaim = serde_json::from_value(
                value
                    .get("identity")
                    .cloned()
                    .ok_or_else(|| invalid("identity is absent"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
            if card.uid().parse::<Anchor>().ok().as_ref() != Some(&identity.anchor) {
                return Err(invalid("identity claim disagrees with uid"));
            }
            let contact = identity.contact(&name, handles, endpoints)?;
            let required = key_uris(&contact);
            let actual = entries(source, "cryptoKeys")?
                .into_iter()
                .map(|(_, k)| text(k, "uri").map(str::to_owned))
                .collect::<Result<BTreeSet<_>, _>>()?;
            if !required.is_subset(&actual) {
                return Err(invalid("identity claim keys missing from cryptoKeys"));
            }
            contact
        } else {
            let anchor = match card.uid().parse::<Anchor>() {
                Ok(anchor) => anchor,
                Err(_) => {
                    // RFC 9553's basic example uses a bare UUID; other free text
                    // is preserved in source and receives the host's local id.
                    match LocalId::parse(&format!("urn:uuid:{}", card.uid())) {
                        Ok(id) => Anchor::Local(id),
                        Err(_) => {
                            if card.uid().starts_with("did:key:")
                                || card.uid().starts_with("did:plc:")
                                || card.uid().to_ascii_lowercase().starts_with("urn:uuid:")
                            {
                                return Err(invalid("malformed anchor uid"));
                            }
                            Anchor::Local(local.ok_or(ExchangeError::LocalAnchorRequired)?)
                        },
                    }
                },
            };
            let roots = anchor
                .as_key()
                .map(|key| crate::RootKey {
                    key: *key,
                    proof: None,
                })
                .into_iter()
                .collect();
            IdentityClaim {
                anchor,
                root_line: roots,
                attested: Vec::new(),
            }
            .contact(&name, handles, endpoints)?
        };
        let mut unbound_keys = Vec::new();
        for (_, resource) in entries(source, "cryptoKeys")? {
            if let Ok(key) = text(resource, "uri")?.parse::<TypedKey>()
                && !contact.knows_key(&key)
                && !unbound_keys.contains(&key)
            {
                unbound_keys.push(key);
            }
        }
        Ok(ImportedCard {
            contact,
            source: card.clone(),
            unbound_keys,
        })
    }

    fn project(&self, contact: &Contact) -> Result<Value, ExchangeError> {
        // Revalidate even a public struct's mutable lists before emitting it.
        let identity = IdentityClaim::of(contact);
        identity.contact(
            &contact.petname,
            contact.handles.clone(),
            contact.endpoints.clone(),
        )?;
        let mut services = Map::new();
        for handle in &contact.handles {
            let mut service = json!({"@type":"OnlineService","user":handle.value,"service":handle_service(&handle.kind)});
            service[self.property("gazHandleKind")] =
                serde_json::to_value(&handle.kind).map_err(|e| invalid(e.to_string()))?;
            insert_stable(&mut services, "h", service)?;
        }
        for endpoint in &contact.endpoints {
            let mut service =
                json!({"@type":"OnlineService","service":endpoint_service(&endpoint.kind)});
            service[if uri(&endpoint.address) {
                "uri"
            } else {
                "user"
            }] = json!(endpoint.address);
            service[self.property("gazEndpointKind")] =
                serde_json::to_value(&endpoint.kind).map_err(|e| invalid(e.to_string()))?;
            insert_stable(&mut services, "e", service)?;
        }
        let mut keys = Map::new();
        for uri in key_uris(contact) {
            insert_stable(&mut keys, "k", json!({"@type":"CryptoKey","uri":uri}))?;
        }
        let mut value = json!({"@type":"Card","version":"1.0","uid":contact.anchor().to_string(),
            "kind":"individual","name":{"@type":"Name","full":contact.petname}});
        if !services.is_empty() {
            value["onlineServices"] = Value::Object(services);
        }
        if !keys.is_empty() {
            value["cryptoKeys"] = Value::Object(keys);
        }
        value[self.property("gazIdentity")] = json!({"version":"1","identity":identity});
        Ok(value)
    }

    fn addresses(&self, source: &Value) -> Result<(Vec<Handle>, Vec<Endpoint>), ExchangeError> {
        let mut handles = Vec::new();
        let mut endpoints = Vec::new();
        for (_, service) in entries(source, "onlineServices")? {
            let h = service.get(&self.property("gazHandleKind"));
            let e = service.get(&self.property("gazEndpointKind"));
            if h.is_some() && e.is_some() {
                return Err(invalid(
                    "online service claims both handle and endpoint roles",
                ));
            }
            if let Some(kind) = h {
                let kind = serde_json::from_value(kind.clone())
                    .map_err(|e| invalid(format!("invalid handle kind: {e}")))?;
                handles.push(Handle::new(kind, text(service, "user")?));
            } else if let Some(kind) = e {
                let kind = serde_json::from_value(kind.clone())
                    .map_err(|e| invalid(format!("invalid endpoint kind: {e}")))?;
                let address = optional_text(service, "uri")?
                    .or(optional_text(service, "user")?)
                    .ok_or_else(|| invalid("endpoint address absent"))?;
                endpoints.push(Endpoint::new(kind, address));
            } else {
                let protocol = optional_text(service, "service")?.unwrap_or("online-service");
                if let Some(user) = optional_text(service, "user")? {
                    handles.push(Handle::new(handle_kind(protocol), user));
                }
                if let Some(address) = optional_text(service, "uri")? {
                    endpoints.push(Endpoint::new(endpoint_kind(protocol, address), address));
                }
            }
        }
        for (_, email) in entries(source, "emails")? {
            endpoints.push(Endpoint::new(
                EndpointKind::Other("email".into()),
                text(email, "address")?,
            ));
        }
        for (_, phone) in entries(source, "phones")? {
            endpoints.push(Endpoint::new(
                EndpointKind::Other("phone".into()),
                text(phone, "number")?,
            ));
        }
        Ok((handles, endpoints))
    }
}

fn key_uris(contact: &Contact) -> BTreeSet<String> {
    contact
        .root_line()
        .iter()
        .map(|root| &root.key)
        .chain(contact.attested().iter().map(|key| &key.key))
        .map(|key| {
            let text = key.to_string();
            if uri(&text) {
                text
            } else {
                // Reticulum's rnid text is plain hex, not a URI. Its resource
                // carries the whole 64-byte public identity in a data URI.
                let bytes = (0..text.len())
                    .step_by(2)
                    .map(|i| format!("%{}", &text[i..i + 2]))
                    .collect::<String>();
                format!("data:application/octet-stream,{bytes}")
            }
        })
        .collect()
}

fn insert_stable(
    map: &mut Map<String, Value>,
    prefix: &str,
    value: Value,
) -> Result<(), ExchangeError> {
    // FNV-1a 128 is an entry-name fingerprint, not an identity/security hash.
    // Stable content ids survive array reorder or changes to unrelated entries.
    // A collision between distinct entries refuses instead of dropping one.
    // Projected entry values are flat objects (the enum payload, if any, is
    // a single-property object). Sort fields explicitly so serde_json's
    // optional preserve_order feature cannot change their fingerprints.
    let canonical: BTreeMap<_, _> = super::card::object(&value, "projected entry")?
        .iter()
        .collect();
    let encoded = serde_json::to_vec(&canonical).map_err(|e| invalid(e.to_string()))?;
    let mut hash = 0x6c62272e07bb014262b821756295c58du128;
    for byte in encoded {
        hash = (hash ^ u128::from(byte)).wrapping_mul(0x1000000000000000000013b);
    }
    let id = format!("{prefix}{hash:032x}");
    if map.get(&id).is_some_and(|old| old != &value) {
        return Err(invalid("exchange entry id collision"));
    }
    map.insert(id, value);
    Ok(())
}

fn handle_service(kind: &HandleKind) -> &str {
    match kind {
        HandleKind::Acct => "acct",
        HandleKind::Did => "did",
        HandleKind::Nostr => "nostr",
        HandleKind::Other(s) => s,
    }
}

fn endpoint_service(kind: &EndpointKind) -> &str {
    match kind {
        EndpointKind::Misfin => "misfin",
        EndpointKind::Murm => "murm",
        EndpointKind::Gemini => "gemini",
        EndpointKind::Gopher => "gopher",
        EndpointKind::ActivityPub => "activitypub",
        EndpointKind::Http => "http",
        EndpointKind::Other(s) => s,
    }
}

fn handle_kind(service: &str) -> HandleKind {
    match service.to_ascii_lowercase().as_str() {
        "acct" => HandleKind::Acct,
        "did" => HandleKind::Did,
        "nostr" => HandleKind::Nostr,
        _ => HandleKind::Other(service.to_owned()),
    }
}

fn endpoint_kind(service: &str, address: &str) -> EndpointKind {
    match service.to_ascii_lowercase().as_str() {
        "misfin" => EndpointKind::Misfin,
        "murm" => EndpointKind::Murm,
        "gemini" => EndpointKind::Gemini,
        "gopher" => EndpointKind::Gopher,
        "activitypub" => EndpointKind::ActivityPub,
        "http" | "https" => EndpointKind::Http,
        _ if address.starts_with("http:") || address.starts_with("https:") => EndpointKind::Http,
        _ => EndpointKind::Other(service.to_owned()),
    }
}

fn display_name(card: &Value, uid: &str) -> String {
    if let Some(full) = card
        .get("name")
        .and_then(|n| n.get("full"))
        .and_then(Value::as_str)
    {
        return full.to_owned();
    }
    if let Some(components) = card
        .get("name")
        .and_then(|n| n.get("components"))
        .and_then(Value::as_array)
    {
        let name = &card["name"];
        let ordered = name
            .get("isOrdered")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let separator = name
            .get("defaultSeparator")
            .and_then(Value::as_str)
            .unwrap_or(" ");
        let mut full = String::new();
        let mut previous_separator = false;
        for component in components {
            let is_separator = ordered && component["kind"] == "separator";
            if !full.is_empty() && !is_separator && !previous_separator {
                full.push_str(separator);
            }
            full.push_str(
                component["value"]
                    .as_str()
                    .expect("validated name component"),
            );
            previous_separator = is_separator;
        }
        return full;
    }
    uid.to_owned()
}
