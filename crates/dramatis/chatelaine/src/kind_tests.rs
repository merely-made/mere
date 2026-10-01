// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A test per kind: its metadata is exactly §1's, holds no sealed member, and
//! round-trips through JSON and postcard.

use std::collections::BTreeSet;

use serde_json::Value;

use super::*;
use crate::samples;

/// The kind's name and field names as JSON writes them.
fn shape(kind: &CredentialKind) -> (String, BTreeSet<String>) {
    match serde_json::to_value(kind).unwrap() {
        Value::String(name) => (name, BTreeSet::new()),
        Value::Object(map) => {
            let (name, body) = map.into_iter().next().unwrap();
            let fields = body.as_object().unwrap().keys().cloned().collect();
            (name, fields)
        },
        other => panic!("unexpected JSON shape {other}"),
    }
}

/// Assert `kind` is `cxf_type` with exactly `fields` and none of the CXF
/// members §1 seals for it (named as chatelaine would name them), and that it
/// round-trips in both formats.
fn check(kind: CredentialKind, cxf_type: Option<&str>, fields: &[&str], sealed: &[&str]) {
    assert_eq!(kind.cxf_type(), cxf_type);
    let (name, actual) = shape(&kind);
    // A CXF kind's tag is its CXF type, except Otp's (ruling 37).
    let tag = match &kind {
        CredentialKind::Otp { .. } => "otp",
        CredentialKind::Secret { .. } => "secret",
        CredentialKind::Unknown { .. } => "unknown",
        _ => cxf_type.unwrap(),
    };
    assert_eq!(name, tag, "the variant's tag");
    let expected: BTreeSet<String> = fields.iter().map(|f| f.to_string()).collect();
    assert_eq!(actual, expected, "{name} carries exactly its metadata");
    for field in sealed {
        assert!(
            !actual.contains(*field),
            "{name} has a sealed field {field}"
        );
    }

    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(
        serde_json::from_str::<CredentialKind>(&json).unwrap(),
        kind,
        "JSON"
    );
    let bytes = postcard::to_allocvec(&kind).unwrap();
    assert_eq!(
        postcard::from_bytes::<CredentialKind>(&bytes).unwrap(),
        kind,
        "postcard"
    );
}

#[test]
fn address_carries_nothing() {
    let sealed = [
        "street_address",
        "postal_code",
        "city",
        "territory",
        "country",
        "tel",
    ];
    check(samples::address(), Some("address"), &[], &sealed);
}

#[test]
fn api_key_carries_username_key_type_and_url() {
    let fields = ["username", "key_type", "url"];
    let sealed = ["key", "valid_from", "expiry_date", "expiry"];
    check(samples::api_key(), Some("api-key"), &fields, &sealed);
}

#[test]
fn basic_auth_carries_the_username() {
    check(
        samples::basic_auth(),
        Some("basic-auth"),
        &["username"],
        &["password"],
    );
}

#[test]
fn credit_card_carries_type_expiry_and_last_four() {
    let fields = ["card_type", "expiry", "last_four"];
    let sealed = [
        "number",
        "verification_number",
        "pin",
        "full_name",
        "valid_from",
    ];
    check(
        samples::credit_card(),
        Some("credit-card"),
        &fields,
        &sealed,
    );
}

#[test]
fn custom_fields_carry_their_labels() {
    let fields = ["label", "field_labels"];
    check(
        samples::custom_fields(),
        Some("custom-fields"),
        &fields,
        &["fields", "values"],
    );
}

#[test]
fn drivers_license_carries_country_territory_and_expiry() {
    let fields = ["issuing_country", "territory", "expiry"];
    let sealed = [
        "license_number",
        "full_name",
        "birth_date",
        "issue_date",
        "issuing_authority",
        "license_class",
    ];
    check(
        samples::drivers_license(),
        Some("drivers-license"),
        &fields,
        &sealed,
    );
}

#[test]
fn file_carries_name_size_and_integrity_hash() {
    let fields = ["name", "size", "integrity_hash"];
    check(samples::file(), Some("file"), &fields, &["bytes", "data"]);
}

#[test]
fn generated_password_carries_nothing() {
    check(
        samples::generated_password(),
        Some("generated-password"),
        &[],
        &["password"],
    );
}

#[test]
fn identity_document_carries_country_and_expiry() {
    let fields = ["issuing_country", "expiry"];
    let sealed = [
        "document_number",
        "identification_number",
        "nationality",
        "full_name",
        "birth_date",
        "birth_place",
        "sex",
        "issue_date",
        "issuing_authority",
    ];
    check(
        samples::identity_document(),
        Some("identity-document"),
        &fields,
        &sealed,
    );
}

#[test]
fn item_reference_carries_the_link() {
    check(
        samples::item_reference(),
        Some("item-reference"),
        &["item"],
        &[],
    );
}

#[test]
fn note_carries_nothing() {
    check(samples::note(), Some("note"), &[], &["content"]);
}

#[test]
fn passkey_carries_rp_id_username_and_display_name() {
    let fields = ["rp_id", "username", "user_display_name"];
    let sealed = ["key", "credential_id", "user_handle", "fido2_extensions"];
    check(samples::passkey(), Some("passkey"), &fields, &sealed);
}

#[test]
fn passport_carries_kind_country_and_expiry() {
    let fields = ["passport_type", "issuing_country", "expiry"];
    let sealed = [
        "passport_number",
        "national_identification_number",
        "nationality",
        "full_name",
        "birth_date",
        "birth_place",
        "sex",
        "issue_date",
        "issuing_authority",
    ];
    check(samples::passport(), Some("passport"), &fields, &sealed);
}

#[test]
fn person_name_carries_nothing() {
    let sealed = [
        "title",
        "given",
        "given_informal",
        "given2",
        "surname_prefix",
        "surname",
        "surname2",
        "credentials",
        "generation",
    ];
    check(samples::person_name(), Some("person-name"), &[], &sealed);
}

#[test]
fn ssh_key_carries_type_fingerprint_and_comment() {
    let fields = ["key_type", "fingerprint", "comment"];
    let sealed = [
        "private_key",
        "creation_date",
        "expiry_date",
        "key_generation_source",
    ];
    check(samples::ssh_key(), Some("ssh-key"), &fields, &sealed);
}

#[test]
fn otp_carries_what_castellans_otp_items_describe() {
    let fields = ["account", "issuer", "algorithm", "code_style", "mode"];
    for kind in [samples::totp(), samples::hotp(), samples::steam_guard()] {
        check(kind, Some("totp"), &fields, &["secret", "counter"]);
    }
}

#[test]
fn an_hotp_credential_holds_no_counter() {
    let json = serde_json::to_value(samples::hotp()).unwrap();
    assert_eq!(json["otp"]["mode"], "hotp");
}

#[test]
fn wifi_carries_ssid_and_security_type() {
    let fields = ["ssid", "security"];
    let sealed = ["passphrase", "hidden"];
    check(samples::wifi(), Some("wifi"), &fields, &sealed);
    let other = CredentialKind::Wifi {
        ssid: None,
        security: Some(WifiSecurity::Other("wpa2-enterprise".to_string())),
    };
    check(other, Some("wifi"), &fields, &sealed);
}

#[test]
fn a_secret_service_secret_carries_content_type_and_attributes() {
    let fields = ["content_type", "attributes"];
    check(samples::secret(), None, &fields, &["secret", "value"]);
}

#[test]
fn an_unknown_kind_preserves_its_type_string() {
    check(
        samples::unknown(),
        Some("future-credential"),
        &["cxf_type"],
        &[],
    );
}

#[test]
fn the_samples_cover_every_cxf_v1_type_once() {
    let kinds = samples::every_kind();
    let types: Vec<&str> = kinds
        .iter()
        .map(|kind| kind.cxf_type().unwrap_or("(secret service)"))
        .collect();
    assert_eq!(&types[..17], &CXF_V1_TYPES[..]);
    assert_eq!(types.len(), 19);
    let distinct: BTreeSet<&str> = CXF_V1_TYPES.iter().copied().collect();
    assert_eq!(distinct.len(), 17, "no type listed twice");
}
