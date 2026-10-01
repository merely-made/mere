// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One sample of every kind, for the tests.

use std::collections::BTreeMap;

use crate::{
    AndroidApp, AppCertificate, CountryCode, Credential, CredentialId, CredentialKind, Date, Item,
    ItemId, ItemState, LastFour, Link, OtpAlgorithm, OtpCodeStyle, OtpMode, Scope, SourceId,
    SshFingerprint, SubdivisionCode, WifiSecurity, YearMonth,
};

pub fn item_id(n: u8) -> ItemId {
    ItemId::from_random([n; 16])
}

pub fn address() -> CredentialKind {
    CredentialKind::Address
}

pub fn api_key() -> CredentialKind {
    CredentialKind::ApiKey {
        username: Some("deploy-bot".to_string()),
        key_type: Some("bearer".to_string()),
        url: Some("https://api.example.com".to_string()),
    }
}

pub fn basic_auth() -> CredentialKind {
    CredentialKind::BasicAuth {
        username: Some("mark".to_string()),
    }
}

pub fn credit_card() -> CredentialKind {
    CredentialKind::CreditCard {
        card_type: Some("Visa".to_string()),
        expiry: Some(YearMonth::parse("2029-07").unwrap()),
        last_four: Some(LastFour::parse("4242").unwrap()),
    }
}

pub fn custom_fields() -> CredentialKind {
    CredentialKind::CustomFields {
        label: Some("Router".to_string()),
        field_labels: vec![Some("Admin PIN".to_string()), None],
    }
}

pub fn drivers_license() -> CredentialKind {
    CredentialKind::DriversLicense {
        issuing_country: Some(CountryCode::parse("US").unwrap()),
        territory: Some(SubdivisionCode::parse("US-OR").unwrap()),
        expiry: Some(Date::parse("2030-05-17").unwrap()),
    }
}

pub fn file() -> CredentialKind {
    CredentialKind::File {
        name: "recovery-codes.pdf".to_string(),
        size: 48_213,
        integrity_hash: [0xab; 32],
    }
}

pub fn generated_password() -> CredentialKind {
    CredentialKind::GeneratedPassword
}

pub fn identity_document() -> CredentialKind {
    CredentialKind::IdentityDocument {
        issuing_country: Some(CountryCode::parse("NZ").unwrap()),
        expiry: None,
    }
}

pub fn item_reference() -> CredentialKind {
    CredentialKind::ItemReference(Link { item: item_id(9) })
}

pub fn note() -> CredentialKind {
    CredentialKind::Note
}

pub fn passkey() -> CredentialKind {
    CredentialKind::Passkey {
        rp_id: "example.com".to_string(),
        username: "mark@example.com".to_string(),
        user_display_name: "Mark".to_string(),
    }
}

pub fn passport() -> CredentialKind {
    CredentialKind::Passport {
        passport_type: Some("P".to_string()),
        issuing_country: Some(CountryCode::parse("CA").unwrap()),
        expiry: Some(Date::parse("2033-11-02").unwrap()),
    }
}

pub fn person_name() -> CredentialKind {
    CredentialKind::PersonName
}

pub fn ssh_key() -> CredentialKind {
    CredentialKind::SshKey {
        key_type: "ssh-ed25519".to_string(),
        fingerprint: SshFingerprint::parse("SHA256:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU")
            .unwrap(),
        comment: Some("mark@thinkpad".to_string()),
    }
}

pub fn totp() -> CredentialKind {
    CredentialKind::Otp {
        account: "mark".to_string(),
        issuer: Some("Merely".to_string()),
        algorithm: OtpAlgorithm::Sha256,
        code_style: OtpCodeStyle::Decimal { digits: 8 },
        mode: OtpMode::Totp { period: 30, t0: 0 },
    }
}

pub fn hotp() -> CredentialKind {
    CredentialKind::Otp {
        account: "mark".to_string(),
        issuer: None,
        algorithm: OtpAlgorithm::Sha1,
        code_style: OtpCodeStyle::Decimal { digits: 6 },
        mode: OtpMode::Hotp,
    }
}

pub fn steam_guard() -> CredentialKind {
    CredentialKind::Otp {
        account: "mark".to_string(),
        issuer: Some("Steam".to_string()),
        algorithm: OtpAlgorithm::Sha1,
        code_style: OtpCodeStyle::SteamGuard,
        mode: OtpMode::Totp { period: 30, t0: 0 },
    }
}

pub fn wifi() -> CredentialKind {
    CredentialKind::Wifi {
        ssid: Some("Home".to_string()),
        security: Some(WifiSecurity::Wpa3Personal),
    }
}

pub fn secret() -> CredentialKind {
    CredentialKind::Secret {
        content_type: "text/plain".to_string(),
        attributes: BTreeMap::from([
            ("service".to_string(), "mail".to_string()),
            ("user".to_string(), "mark".to_string()),
        ]),
    }
}

pub fn unknown() -> CredentialKind {
    CredentialKind::Unknown {
        cxf_type: "future-credential".to_string(),
    }
}

/// Every kind once: CXF's 17 in specification order, then Secret, then
/// Unknown.
pub fn every_kind() -> Vec<CredentialKind> {
    vec![
        address(),
        api_key(),
        basic_auth(),
        credit_card(),
        custom_fields(),
        drivers_license(),
        file(),
        generated_password(),
        identity_document(),
        item_reference(),
        note(),
        passkey(),
        passport(),
        person_name(),
        ssh_key(),
        totp(),
        wifi(),
        secret(),
        unknown(),
    ]
}

/// An item in the vault holding `kinds`, with every optional field set.
pub fn item(kinds: Vec<CredentialKind>) -> Item {
    Item {
        id: item_id(1),
        source_id: Some(SourceId::parse("ZmllbGQx").unwrap()),
        title: "Everything".to_string(),
        subtitle: Some("one of each".to_string()),
        scope: Some(Scope {
            urls: vec!["https://example.com/login".to_string()],
            android_apps: vec![AndroidApp {
                bundle_id: "com.example.app".to_string(),
                certificate: Some(AppCertificate {
                    fingerprint: vec![0x5a; 32],
                    hash_algorithm: "sha256".to_string(),
                }),
                name: Some("Example".to_string()),
            }],
        }),
        tags: vec!["work".to_string()],
        favorite: true,
        created_at: Some(1_790_000_000),
        modified_at: Some(1_790_086_400),
        credentials: kinds
            .into_iter()
            .zip(1u8..)
            .map(|(kind, n)| Credential {
                id: CredentialId::from_random([n; 16]),
                kind,
            })
            .collect(),
        state: ItemState::Vault,
    }
}
