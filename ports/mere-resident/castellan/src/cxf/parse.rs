// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::erase::{Erase, Json};
use super::*;
use base64::Engine;
use chatelaine::*;
use serde::Deserialize;
use serde_json::{Value, value::RawValue};

#[derive(Deserialize)]
struct RawHeader<'a> {
    version: RawVersion,
    #[serde(borrow)]
    accounts: Vec<RawAccount<'a>>,
}
#[derive(Deserialize)]
struct RawVersion {
    major: u64,
}
#[derive(Deserialize)]
struct RawAccount<'a> {
    #[serde(borrow)]
    items: Vec<RawItem<'a>>,
}
#[derive(Deserialize)]
struct RawItem<'a> {
    #[serde(borrow)]
    credentials: Vec<&'a RawValue>,
}

pub(super) fn read(bytes: Zeroizing<Vec<u8>>) -> Result<Import, ImportError> {
    let raw: RawHeader =
        serde_json::from_slice(&bytes).map_err(|_| ImportError("invalid CXF envelope"))?;
    if raw.version.major != 1 {
        return Err(ImportError("unsupported CXF major version"));
    }
    for account in &raw.accounts {
        for item in &account.items {
            for credential in &item.credentials {
                let v = Json(
                    serde_json::from_slice(credential.get().as_bytes())
                        .map_err(|_| ImportError("invalid credential JSON"))?,
                );
                if v.0.get("type").and_then(Value::as_str).is_none() {
                    return Err(ImportError("every credential must have a string type"));
                }
            }
        }
    }
    // Borrowing base64url decoding requires the original complete slice. Never
    // feed this crate a serde Value or a reader, and never format its values.
    let header: credential_exchange_format::Header =
        serde_json::from_slice(&bytes).map_err(|_| ImportError("invalid CXF envelope or field"))?;
    let canonical = serde_json::to_value(&header);
    header.erase();
    let canonical = Json(canonical.map_err(|_| ImportError("cannot normalize CXF document"))?);
    use sha2::Digest;
    let source_sha256 = sha2::Sha256::digest(&*bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let mut preview = Preview {
        source_sha256,
        accounts: Vec::new(),
        notices: Vec::new(),
    };
    let mut accounts = Vec::new();
    let mut account_ids = std::collections::BTreeSet::new();
    for (account, raw_account) in canonical.0["accounts"]
        .as_array()
        .ok_or(ImportError("missing accounts"))?
        .iter()
        .zip(&raw.accounts)
    {
        let id = text(account, "id").ok_or(ImportError("missing account id"))?;
        source(&id)?;
        if !account_ids.insert(id.clone()) {
            return Err(ImportError("duplicate CXF account id"));
        }
        let mut summary = AccountPreview {
            source_id: id.clone(),
            username: text(account, "username").unwrap_or_default(),
            email: text(account, "email").unwrap_or_default(),
            items: raw_account.items.len(),
            dispositions: BTreeMap::new(),
        };
        let mut items = Vec::new();
        let mut item_ids = std::collections::BTreeSet::new();
        for (item, raw_item) in account["items"]
            .as_array()
            .ok_or(ImportError("missing items"))?
            .iter()
            .zip(&raw_account.items)
        {
            let item_id = text(item, "id").ok_or(ImportError("missing item id"))?;
            if !item_ids.insert(item_id.clone()) {
                return Err(ImportError("duplicate CXF item id"));
            }
            let title = text(item, "title").ok_or(ImportError("missing item title"))?;
            sharing_notices(item, &id, &item_id, &mut preview.notices);
            let scope = scope(&item["scope"])?;
            let metadata = Item {
                id: ItemId::from_random(crate::items::random_id_bytes()),
                source_id: Some(source(&item_id)?),
                title: title.clone(),
                subtitle: text(item, "subtitle"),
                scope,
                tags: item["tags"]
                    .as_array()
                    .map(|v| {
                        v.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
                favorite: item["favorite"].as_bool().unwrap_or(false),
                created_at: item["creationAt"].as_u64(),
                modified_at: item["modifiedAt"].as_u64(),
                credentials: Vec::new(),
                state: ItemState::Vault,
            };
            let mut credentials = Vec::new();
            for (position, raw_credential) in raw_item.credentials.iter().enumerate() {
                let original = Json(
                    serde_json::from_slice(raw_credential.get().as_bytes())
                        .map_err(|_| ImportError("invalid credential"))?,
                );
                let ty = original.0["type"]
                    .as_str()
                    .ok_or(ImportError("missing credential type"))?;
                let normalized = &item["credentials"][position];
                let mut notice = |reason: &str| {
                    preview.notices.push(Notice {
                        account: id.clone(),
                        item: Some(item_id.clone()),
                        credential: Some(position),
                        reason: reason.into(),
                    })
                };
                let (prepared, disposition) = credential(
                    ty,
                    &original.0,
                    normalized,
                    raw_credential.get(),
                    &title,
                    &mut notice,
                )?;
                *summary.dispositions.entry(disposition.into()).or_default() += 1;
                if let Some(prepared) = prepared {
                    credentials.push(prepared);
                }
            }
            items.push(PreparedItem {
                metadata,
                credentials,
            });
        }
        preview.accounts.push(summary);
        for collection in account["collections"].as_array().into_iter().flatten() {
            collection_sharing_notices(collection, &id, &mut preview.notices);
        }
        accounts.push(PreparedAccount {
            id,
            items,
            collections: Json(account["collections"].clone()),
        });
    }
    Ok(Import { preview, accounts })
}

// CXF sharing describes the exporting provider's grants. This importer has
// no sharing provider: local ownership comes only from the explicit account
// destination, never an accessor in an untrusted file. Apply the enum ignore
// rules first and report supported grants that cannot be recreated.
fn sharing_notices(value: &Value, account: &str, item: &str, notices: &mut Vec<Notice>) {
    for extension in value["extensions"].as_array().into_iter().flatten() {
        if extension["name"] != "shared" {
            continue;
        }
        for accessor in extension["accessors"].as_array().into_iter().flatten() {
            let mut report = |reason: &str| {
                notices.push(Notice {
                    account: account.into(),
                    item: Some(item.into()),
                    credential: None,
                    reason: reason.into(),
                })
            };
            if !matches!(accessor["type"].as_str(), Some("user" | "group")) {
                report("unknown sharing accessor type ignored");
                continue;
            }
            let mut supported = 0;
            for permission in accessor["permissions"].as_array().into_iter().flatten() {
                if matches!(
                    permission.as_str(),
                    Some(
                        "read" | "readSecret" | "update" | "create" | "delete" | "share" | "manage"
                    )
                ) {
                    supported += 1;
                } else {
                    report("unknown sharing permission ignored");
                }
            }
            if supported == 0 {
                report("sharing accessor with no supported permissions ignored");
            } else {
                report("exporter sharing grant not recreated; local destination owns the import");
            }
        }
    }
}
fn collection_sharing_notices(value: &Value, account: &str, notices: &mut Vec<Notice>) {
    sharing_notices(value, account, value["id"].as_str().unwrap_or(""), notices);
    for child in value["subCollections"].as_array().into_iter().flatten() {
        collection_sharing_notices(child, account, notices);
    }
}

fn credential(
    ty: &str,
    original: &Value,
    normalized: &Value,
    raw: &str,
    title: &str,
    notice: &mut impl FnMut(&str),
) -> Result<(Option<PreparedCredential>, &'static str), ImportError> {
    let known = CXF_V1_TYPES.contains(&ty);
    let mut value = Json(normalized.clone());
    let mut quarantine = !known;
    let mut payload = None;
    let mut reference = None;
    let mut ssh = None;
    if ty == "passkey"
        && original
            .pointer("/fido2Extensions/hmacCredentials/algorithm")
            .and_then(Value::as_str)
            .is_some_and(|v| v != "hmac-sha256")
    {
        notice("unknown passkey HMAC algorithm ignored");
        return Ok((None, "ignored"));
    }
    if !known {
        notice("unknown credential type preserved verbatim in quarantine");
        return Ok((
            Some(PreparedCredential {
                kind: Some(CredentialKind::Unknown {
                    cxf_type: ty.into(),
                }),
                payload: Some(Payload::Secret {
                    bytes: raw.as_bytes().to_vec(),
                }),
                quarantine: true,
                reference: None,
                ssh: None,
            }),
            "quarantined",
        ));
    }
    // A known type can fall through the dependency's untagged Unknown arm.
    // Locally repair only the measured TOTP/custom-field gaps. Never call a
    // malformed known credential a newly introduced type.
    let typed: credential_exchange_format::Credential =
        serde_json::from_slice(raw.as_bytes()).map_err(|_| ImportError("invalid credential"))?;
    let malformed = matches!(
        &typed,
        credential_exchange_format::Credential::Unknown { .. }
    );
    typed.erase();
    if malformed && !matches!(ty, "totp" | "custom-fields") {
        notice("malformed known credential preserved in quarantine");
        return Ok((
            Some(PreparedCredential {
                kind: Some(empty_kind(ty)),
                payload: Some(Payload::Secret {
                    bytes: raw.as_bytes().to_vec(),
                }),
                quarantine: true,
                reference: None,
                ssh: None,
            }),
            "quarantined",
        ));
    }
    if ty == "custom-fields" {
        let Some(fields) = original["fields"].as_array() else {
            return malformed_known(ty, raw, notice);
        };
        let mut accepted = Vec::new();
        for field in fields {
            let Some(field_type) = field["fieldType"].as_str() else {
                return malformed_known(ty, raw, notice);
            };
            if ![
                "string",
                "concealed-string",
                "boolean",
                "date",
                "year-month",
                "subdivision-code",
                "country-code",
                "wifi-network-security-type",
                "email",
                "number",
            ]
            .contains(&field_type)
            {
                notice("unknown custom field type ignored");
                continue;
            }
            let encoded = Zeroizing::new(
                serde_json::to_vec(field).map_err(|_| ImportError("invalid custom field"))?,
            );
            match serde_json::from_slice::<credential_exchange_format::EditableFieldValue>(&encoded)
            {
                Ok(field) => {
                    let normalized = serde_json::to_value(&field);
                    field.erase();
                    accepted.push(normalized.map_err(|_| ImportError("invalid custom field"))?);
                },
                Err(_) => {
                    notice("invalid custom field value ignored");
                },
            }
        }
        std::mem::replace(
            &mut value.0,
            serde_json::json!({"type":ty,"label":text(original,"label"),"fields":accepted}),
        )
        .erase();
    }
    let kind = match ty {
        "address" => CredentialKind::Address,
        "person-name" => CredentialKind::PersonName,
        "note" => CredentialKind::Note,
        "generated-password" => CredentialKind::GeneratedPassword,
        "api-key" => CredentialKind::ApiKey {
            username: field(&value.0, "username"),
            key_type: field(&value.0, "keyType"),
            url: field(&value.0, "url"),
        },
        "basic-auth" => CredentialKind::BasicAuth {
            username: field(&value.0, "username"),
        },
        "credit-card" => {
            quarantine = true;
            let last = field(&value.0, "number").and_then(|mut number| {
                use zeroize::Zeroize;
                let digits: Zeroizing<String> =
                    Zeroizing::new(number.chars().filter(char::is_ascii_digit).collect());
                number.zeroize();
                digits
                    .get(digits.len().saturating_sub(4)..)
                    .and_then(|v| LastFour::parse(v).ok())
            });
            CredentialKind::CreditCard {
                card_type: field(&value.0, "cardType"),
                expiry: field(&value.0, "expiryDate").and_then(|v| YearMonth::parse(&v).ok()),
                last_four: last,
            }
        },
        "custom-fields" => CredentialKind::CustomFields {
            label: text(&value.0, "label"),
            field_labels: value.0["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| text(v, "label"))
                .collect(),
        },
        "drivers-license" => {
            quarantine = true;
            CredentialKind::DriversLicense {
                issuing_country: field(&value.0, "country")
                    .and_then(|v| CountryCode::parse(&v).ok()),
                territory: field(&value.0, "territory")
                    .and_then(|v| SubdivisionCode::parse(&v).ok()),
                expiry: field(&value.0, "expiryDate").and_then(|v| Date::parse(&v).ok()),
            }
        },
        "identity-document" => {
            quarantine = true;
            CredentialKind::IdentityDocument {
                issuing_country: field(&value.0, "issuingCountry")
                    .and_then(|v| CountryCode::parse(&v).ok()),
                expiry: field(&value.0, "expiryDate").and_then(|v| Date::parse(&v).ok()),
            }
        },
        "passport" => {
            quarantine = true;
            CredentialKind::Passport {
                passport_type: field(&value.0, "passportType"),
                issuing_country: field(&value.0, "issuingCountry")
                    .and_then(|v| CountryCode::parse(&v).ok()),
                expiry: field(&value.0, "expiryDate").and_then(|v| Date::parse(&v).ok()),
            }
        },
        "passkey" => {
            quarantine = true;
            CredentialKind::Passkey {
                rp_id: text(&value.0, "rpId").unwrap_or_default(),
                username: text(&value.0, "username").unwrap_or_default(),
                user_display_name: text(&value.0, "userDisplayName").unwrap_or_default(),
            }
        },
        "file" => {
            quarantine = true;
            let hash = decode64(text(&value.0, "integrityHash").as_deref().unwrap_or(""))?;
            let Ok(hash) = hash.as_slice().try_into() else {
                return malformed_known(ty, raw, notice);
            };
            CredentialKind::File {
                name: text(&value.0, "name").unwrap_or_default(),
                size: value.0["decryptedSize"].as_u64().unwrap_or_default(),
                integrity_hash: hash,
            }
        },
        "wifi" => CredentialKind::Wifi {
            ssid: field(&value.0, "ssid"),
            security: field(&value.0, "networkSecurityType")
                .and_then(|v| serde_json::from_value(Value::String(v)).ok()),
        },
        "item-reference" => {
            let target =
                text(&value.0["reference"], "item").ok_or(ImportError("invalid item reference"))?;
            source(&target)?;
            let account = text(&value.0["reference"], "account");
            if let Some(account) = &account {
                source(account)?;
            }
            reference = Some((account, target));
            return Ok((
                Some(PreparedCredential {
                    kind: None,
                    payload: None,
                    quarantine: false,
                    reference,
                    ssh: None,
                }),
                "link",
            ));
        },
        "totp" => {
            let Some(algorithm) = text(original, "algorithm") else {
                return malformed_known(ty, raw, notice);
            };
            let algorithm = match algorithm.as_str() {
                "sha1" => OtpAlgorithm::Sha1,
                "sha256" => OtpAlgorithm::Sha256,
                "sha512" => OtpAlgorithm::Sha512,
                _ => {
                    notice("unknown TOTP algorithm ignored");
                    return Ok((None, "ignored"));
                },
            };
            let Some(period) = original["period"]
                .as_u64()
                .filter(|v| (1..=65535).contains(v))
            else {
                return malformed_known(ty, raw, notice);
            };
            let Some(digits) = original["digits"].as_u64().filter(|v| *v <= 65535) else {
                return malformed_known(ty, raw, notice);
            };
            let Some(secret) = original["secret"].as_str() else {
                return malformed_known(ty, raw, notice);
            };
            let secret = match crate::otp::base32::decode(secret) {
                Ok(v) if !v.is_empty() => v,
                _ => {
                    notice("invalid base32 seed preserved in quarantine");
                    return malformed_known(ty, raw, notice);
                },
            };
            // Preserve unsupported counts in quarantine (ruling 68). They
            // must never reach the generator through credential exercise.
            if !(6..=10).contains(&digits) {
                quarantine = true;
                notice(&format!(
                    "unsupported OTP digit count {digits} preserved in quarantine"
                ));
            }
            let account = text(original, "username").unwrap_or_else(|| {
                notice("missing OTP username replaced by item title");
                title.into()
            });
            payload = Some(Payload::Otp {
                secret,
                counter: None,
            });
            CredentialKind::Otp {
                account,
                issuer: text(original, "issuer"),
                algorithm,
                code_style: OtpCodeStyle::Decimal {
                    digits: digits as u32,
                },
                mode: OtpMode::Totp { period, t0: 0 },
            }
        },
        "ssh-key" => {
            let key_type = text(&value.0, "keyType").ok_or(ImportError("missing SSH key type"))?;
            let der = decode64(text(&value.0, "privateKey").as_deref().unwrap_or(""))?;
            let comment = text(&value.0, "keyComment").unwrap_or_default();
            match super::ssh::decode(&key_type, &der, &comment) {
                Ok(private) => {
                    ssh = Some(private);
                },
                Err(_) => {
                    notice("SSH key refused: unsupported algorithm or invalid PKCS#8");
                    return Ok((None, "ignored"));
                },
            }
            return Ok((
                Some(PreparedCredential {
                    kind: None,
                    payload: None,
                    quarantine: false,
                    reference: None,
                    ssh,
                }),
                "ssh",
            ));
        },
        _ => return Err(ImportError("unhandled known CXF type")),
    };
    if payload.is_none() {
        payload = Some(Payload::Secret {
            bytes: serde_json::to_vec(&value.0)
                .map_err(|_| ImportError("cannot seal credential"))?,
        });
    }
    Ok((
        Some(PreparedCredential {
            kind: Some(kind),
            payload,
            quarantine,
            reference,
            ssh,
        }),
        if quarantine { "quarantined" } else { "stored" },
    ))
}

fn malformed_known(
    ty: &str,
    raw: &str,
    notice: &mut impl FnMut(&str),
) -> Result<(Option<PreparedCredential>, &'static str), ImportError> {
    notice("malformed known credential preserved in quarantine");
    Ok((
        Some(PreparedCredential {
            kind: Some(empty_kind(ty)),
            payload: Some(Payload::Secret {
                bytes: raw.as_bytes().to_vec(),
            }),
            quarantine: true,
            reference: None,
            ssh: None,
        }),
        "quarantined",
    ))
}
fn empty_kind(ty: &str) -> CredentialKind {
    // Refused known credentials retain their known taxonomy; no unknown
    // metadata or secret value is manufactured for the read model.
    match ty {
        "address" => CredentialKind::Address,
        "person-name" => CredentialKind::PersonName,
        "note" => CredentialKind::Note,
        "generated-password" => CredentialKind::GeneratedPassword,
        "basic-auth" => CredentialKind::BasicAuth { username: None },
        "api-key" => CredentialKind::ApiKey {
            username: None,
            key_type: None,
            url: None,
        },
        "credit-card" => CredentialKind::CreditCard {
            card_type: None,
            expiry: None,
            last_four: None,
        },
        "custom-fields" => CredentialKind::CustomFields {
            label: None,
            field_labels: Vec::new(),
        },
        "drivers-license" => CredentialKind::DriversLicense {
            issuing_country: None,
            territory: None,
            expiry: None,
        },
        "passport" => CredentialKind::Passport {
            passport_type: None,
            issuing_country: None,
            expiry: None,
        },
        "identity-document" => CredentialKind::IdentityDocument {
            issuing_country: None,
            expiry: None,
        },
        "wifi" => CredentialKind::Wifi {
            ssid: None,
            security: None,
        },
        "passkey" => CredentialKind::Passkey {
            rp_id: String::new(),
            username: String::new(),
            user_display_name: String::new(),
        },
        // Mandatory identifying fields cannot be fabricated. Refused File,
        // SSH, reference and OTP input is reported using its source type.
        _ => CredentialKind::RefusedCxf {
            cxf_type: ty.into(),
        },
    }
}
pub(super) fn text(v: &Value, key: &str) -> Option<String> {
    v[key].as_str().map(str::to_owned)
}
fn field(v: &Value, key: &str) -> Option<String> {
    let expected = match key {
        "number" => "concealed-string",
        "expiryDate" if v["type"] == "credit-card" => "year-month",
        "expiryDate" => "date",
        "country" | "issuingCountry" => "country-code",
        "territory" => "subdivision-code",
        "networkSecurityType" => "wifi-network-security-type",
        _ => "string",
    };
    if v[key]["fieldType"].as_str() != Some(expected) {
        return None;
    }
    v[key]
        .get("value")
        .and_then(Value::as_str)
        .map(str::to_owned)
}
pub(super) fn source(v: &str) -> Result<SourceId, ImportError> {
    SourceId::parse(v).map_err(|_| ImportError("invalid opaque source id"))
}
fn decode64(v: &str) -> Result<Zeroizing<Vec<u8>>, ImportError> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(v.trim_end_matches('='))
        .map(Zeroizing::new)
        .map_err(|_| ImportError("invalid base64url field"))
}
fn scope(v: &Value) -> Result<Option<Scope>, ImportError> {
    if v.is_null() {
        return Ok(None);
    }
    let urls = v["urls"]
        .as_array()
        .ok_or(ImportError("invalid credential scope"))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let mut android_apps = Vec::new();
    for app in v["androidApps"]
        .as_array()
        .ok_or(ImportError("invalid Android scope"))?
    {
        let certificate = if app["certificate"].is_null() {
            None
        } else {
            Some(AppCertificate {
                fingerprint: decode64(
                    text(&app["certificate"], "fingerprint")
                        .as_deref()
                        .unwrap_or(""),
                )?
                .to_vec(),
                hash_algorithm: text(&app["certificate"], "hashAlg")
                    .ok_or(ImportError("missing certificate hash algorithm"))?,
            })
        };
        android_apps.push(AndroidApp {
            bundle_id: text(app, "bundleId").ok_or(ImportError("missing Android bundle id"))?,
            certificate,
            name: text(app, "name"),
        });
    }
    Ok(Some(Scope { urls, android_apps }))
}
