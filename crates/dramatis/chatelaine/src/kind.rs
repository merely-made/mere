// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Credential kinds and the identifying metadata each may carry.
//!
//! Each variant holds only what tells two credentials apart (rulings 23 and
//! 24); its doc names what castellan seals instead. No variant has a field
//! for a sealed value.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::item::Link;
use crate::otp::{OtpAlgorithm, OtpCodeStyle, OtpMode};
use crate::value::{CountryCode, Date, LastFour, SshFingerprint, SubdivisionCode, YearMonth};

/// The 17 credential types of CXF v1.0 (Proposed Standard with errata,
/// 2026-03-09), as the specification spells them.
///
/// A type outside this list is newer than v1.0 and becomes
/// [`CredentialKind::Unknown`].
pub const CXF_V1_TYPES: [&str; 17] = [
    "address",
    "api-key",
    "basic-auth",
    "credit-card",
    "custom-fields",
    "drivers-license",
    "file",
    "generated-password",
    "identity-document",
    "item-reference",
    "note",
    "passkey",
    "passport",
    "person-name",
    "ssh-key",
    "totp",
    "wifi",
];

/// What kind of credential this is, with its secret-free metadata.
///
/// CXF's 17 kinds come first, in the specification's order, then the two
/// kinds CXF does not define. Variant order is the binary wire format: add
/// new kinds at the end.
///
/// Each CXF kind is named for its CXF type except [`CredentialKind::Otp`],
/// CXF's `totp`, which holds HOTP too; [`CredentialKind::cxf_type`] gives the
/// CXF name of every kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CredentialKind {
    /// Autofill data for address forms. Sealed: every field.
    Address,
    /// A key for an API. Sealed: the key and its validity dates.
    ApiKey {
        /// The username the key belongs to.
        username: Option<String>,
        /// The key's type, such as a bearer token or a JWT.
        key_type: Option<String>,
        /// The URL the key is used with.
        url: Option<String>,
    },
    /// A username and password. Sealed: the password.
    BasicAuth {
        /// The username.
        username: Option<String>,
    },
    /// A payment card, quarantined on import. Sealed: the number, the
    /// verification number, the PIN, the cardholder's name and valid-from.
    CreditCard {
        /// The card's vendor (CXF's `cardType`), which is its issuer here:
        /// CXF has no separate issuer field.
        card_type: Option<String>,
        /// The expiry month.
        expiry: Option<YearMonth>,
        /// The number's last four digits, derived by castellan at import.
        last_four: Option<LastFour>,
    },
    /// User-defined fields. Sealed: the field values.
    CustomFields {
        /// The section's own label.
        label: Option<String>,
        /// Each field's label, in field order, so a view can line them up
        /// with the sealed values.
        field_labels: Vec<Option<String>>,
    },
    /// A driver's licence, quarantined on import. Sealed: the number, the
    /// holder's name and birth date, the issue date, the authority and the
    /// licence class.
    DriversLicense {
        /// The licence's country of origin.
        issuing_country: Option<CountryCode>,
        /// The state or province that issued it.
        territory: Option<SubdivisionCode>,
        /// The expiry date.
        expiry: Option<Date>,
    },
    /// A file, quarantined on import until castellan holds blobs. Sealed:
    /// the bytes.
    File {
        /// The file name, extension included.
        name: String,
        /// The decrypted size in bytes.
        size: u64,
        /// SHA-256 of the decrypted bytes, as CXF carries it.
        integrity_hash: [u8; 32],
    },
    /// A machine-generated password. Sealed: the password.
    GeneratedPassword,
    /// Any other identity document (a national id, a tax number),
    /// quarantined on import. Sealed: the numbers, the holder's name, birth
    /// data, sex and nationality, the issue date and the authority.
    IdentityDocument {
        /// The issuing country.
        issuing_country: Option<CountryCode>,
        /// The expiry date.
        expiry: Option<Date>,
    },
    /// A reference to another item, kept as a link rather than stored as a
    /// credential of its own. Nothing is sealed.
    ItemReference(Link),
    /// A note. Sealed: the content.
    Note,
    /// A passkey, quarantined on import until a passkey provider exists.
    /// Sealed: the key, the credential id, the user handle and the
    /// extensions.
    Passkey {
        /// The WebAuthn relying party id.
        rp_id: String,
        /// The account's username.
        username: String,
        /// The account's display name.
        user_display_name: String,
    },
    /// A passport, quarantined on import. Sealed: the numbers, the holder's
    /// name, birth data, sex and nationality, the issue date and the
    /// authority.
    Passport {
        /// The ICAO Doc 9303 document code, such as `P`: the document kind.
        passport_type: Option<String>,
        /// The issuing country.
        issuing_country: Option<CountryCode>,
        /// The expiry date.
        expiry: Option<Date>,
    },
    /// Autofill data for a person's name. Sealed: every field.
    PersonName,
    /// An SSH key, which import routes to personae's SSH slots. Sealed: the
    /// private key, its dates and its generation source.
    SshKey {
        /// The SSH public key algorithm, such as `ssh-ed25519`.
        key_type: String,
        /// The public key's fingerprint, derived by castellan.
        fingerprint: SshFingerprint,
        /// The key's comment.
        comment: Option<String>,
    },
    /// A one-time password: CXF's `totp`, and everything castellan's OTP
    /// items describe, HOTP and Steam Guard included (ruling 37). Sealed:
    /// the secret, and an HOTP item's counter.
    Otp {
        /// The account the codes are for.
        account: String,
        /// The issuing service.
        issuer: Option<String>,
        /// The HMAC hash.
        algorithm: OtpAlgorithm,
        /// How a code is written; decimal styles carry the digit count.
        code_style: OtpCodeStyle,
        /// Time-based with its period, or counter-based.
        mode: OtpMode,
    },
    /// A Wi-Fi network. Sealed: the passphrase and the hidden flag.
    Wifi {
        /// The network name.
        ssid: Option<String>,
        /// The network's security type.
        security: Option<WifiSecurity>,
    },
    /// A Freedesktop Secret Service secret. The item's label is the item's
    /// title. Sealed: the bytes.
    Secret {
        /// The secret's content type, such as `text/plain`.
        content_type: String,
        /// The lookup attributes clients search by.
        attributes: BTreeMap<String, String>,
    },
    /// A CXF type newer than v1.0, quarantined on import. Sealed: every
    /// field, verbatim.
    Unknown {
        /// The type string as the file wrote it.
        cxf_type: String,
    },
}

impl CredentialKind {
    /// The CXF type this kind imports from and exports to. `None` for a
    /// Secret Service secret, which CXF does not define.
    pub fn cxf_type(&self) -> Option<&str> {
        Some(match self {
            Self::Address => "address",
            Self::ApiKey { .. } => "api-key",
            Self::BasicAuth { .. } => "basic-auth",
            Self::CreditCard { .. } => "credit-card",
            Self::CustomFields { .. } => "custom-fields",
            Self::DriversLicense { .. } => "drivers-license",
            Self::File { .. } => "file",
            Self::GeneratedPassword => "generated-password",
            Self::IdentityDocument { .. } => "identity-document",
            Self::ItemReference(_) => "item-reference",
            Self::Note => "note",
            Self::Passkey { .. } => "passkey",
            Self::Passport { .. } => "passport",
            Self::PersonName => "person-name",
            Self::SshKey { .. } => "ssh-key",
            Self::Otp { .. } => "totp",
            Self::Wifi { .. } => "wifi",
            Self::Secret { .. } => return None,
            Self::Unknown { cxf_type } => cxf_type,
        })
    }
}

/// A Wi-Fi network's security type, CXF's `wifi-network-security-type`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WifiSecurity {
    /// An open network.
    Unsecured,
    /// WPA-Personal.
    WpaPersonal,
    /// WPA2-Personal.
    Wpa2Personal,
    /// WPA3-Personal.
    Wpa3Personal,
    /// WEP.
    Wep,
    /// Free text, which CXF also allows here.
    Other(String),
}

#[cfg(test)]
#[path = "kind_tests.rs"]
mod tests;
