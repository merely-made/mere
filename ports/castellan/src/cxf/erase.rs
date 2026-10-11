// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Consume parsed CXF allocations without formatting credential values.
use credential_exchange_format::*;
use zeroize::Zeroize;
pub(super) trait Erase {
    fn erase(self);
}
impl Erase for String {
    fn erase(mut self) {
        self.zeroize();
    }
}
impl<T: Erase> Erase for Option<T> {
    fn erase(self) {
        if let Some(v) = self {
            v.erase();
        }
    }
}
impl<T: Erase> Erase for Vec<T> {
    fn erase(self) {
        for v in self {
            v.erase();
        }
    }
}
impl<T: Erase> Erase for Box<T> {
    fn erase(self) {
        (*self).erase();
    }
}
impl Erase for B64Url {
    fn erase(self) {
        let mut v: Vec<u8> = self.into();
        v.zeroize();
    }
}
impl Erase for B32 {
    fn erase(self) {
        let mut v: Vec<u8> = self.into();
        v.zeroize();
    }
}
impl Erase for serde_json::Value {
    fn erase(self) {
        match self {
            Self::String(v) => v.erase(),
            Self::Array(v) => v.erase(),
            Self::Object(v) => {
                for (k, v) in v {
                    k.erase();
                    v.erase();
                }
            },
            _ => {},
        }
    }
}
impl Erase for serde_json::Map<String, serde_json::Value> {
    fn erase(self) {
        for (k, v) in self {
            k.erase();
            v.erase();
        }
    }
}
impl Erase for () {
    fn erase(self) {}
}
impl Erase for u8 {
    fn erase(self) {}
}
impl Erase for u16 {
    fn erase(self) {}
}
impl Erase for u64 {
    fn erase(self) {}
}
impl Erase for bool {
    fn erase(self) {}
}
impl<T: Erase + EditableFieldType> Erase for Expected<T> {
    fn erase(self) {
        match self.into_expected() {
            Ok(v) => v.erase(),
            Err(v) => v.erase(),
        }
    }
}
impl<T: Erase + EditableFieldType> Erase for EditableField<T> {
    fn erase(self) {
        self.id.erase();
        self.value.erase();
        self.label.erase();
        self.extensions.erase();
    }
}
impl Erase for Extension {
    fn erase(self) {
        match self {
            Self::Unknown(v) => v.erase(),
            Self::External(v) => v.erase(),
            Self::Shared(v) => v.erase(),
            _ => {},
        }
    }
}
macro_rules! fields {($t:ty,$($f:ident),* $(,)?)=>{impl Erase for $t {fn erase(self){$(self.$f.erase();)*}}};}
macro_rules! plain {($($t:ty),* $(,)?)=>{$(impl Erase for $t {fn erase(self){}})*};}
plain!(
    EditableFieldBoolean,
    EditableFieldDate,
    EditableFieldYearMonth,
    Version
);
impl Erase for EditableFieldString {
    fn erase(self) {
        self.0.erase();
    }
}
impl Erase for EditableFieldConcealedString {
    fn erase(self) {
        self.0.erase();
    }
}
impl Erase for EditableFieldEmail {
    fn erase(self) {
        self.0.erase();
    }
}
impl Erase for EditableFieldNumber {
    fn erase(self) {
        self.0.erase();
    }
}
impl Erase for EditableFieldSubdivisionCode {
    fn erase(self) {
        self.0.erase();
    }
}
impl Erase for EditableFieldCountryCode {
    fn erase(self) {
        self.0.erase();
    }
}
fields!(
    Header,
    version,
    exporter_rp_id,
    exporter_display_name,
    timestamp,
    accounts
);
fields!(
    Account,
    id,
    username,
    email,
    full_name,
    collections,
    items,
    extensions
);
fields!(
    Collection,
    id,
    creation_at,
    modified_at,
    title,
    subtitle,
    items,
    sub_collections,
    extensions
);
fields!(LinkedItem, item, account);
fields!(
    Item,
    id,
    creation_at,
    modified_at,
    title,
    subtitle,
    favorite,
    scope,
    credentials,
    tags,
    extensions
);
fields!(ItemReferenceCredential, reference);
fields!(
    ApiKeyCredential,
    key,
    username,
    key_type,
    url,
    valid_from,
    expiry_date
);
fields!(BasicAuthCredential, username, password);
fields!(GeneratedPasswordCredential, password);
fields!(
    SshKeyCredential,
    key_type,
    private_key,
    key_comment,
    creation_date,
    expiry_date,
    key_generation_source
);
fields!(
    TotpCredential,
    secret,
    period,
    digits,
    username,
    algorithm,
    issuer
);
fields!(
    WifiCredential,
    ssid,
    network_security_type,
    passphrase,
    hidden
);
fields!(
    AddressCredential,
    street_address,
    postal_code,
    city,
    territory,
    country,
    tel
);
fields!(
    CreditCardCredential,
    number,
    full_name,
    card_type,
    verification_number,
    pin,
    expiry_date,
    valid_from
);
fields!(
    DriversLicenseCredential,
    full_name,
    birth_date,
    issue_date,
    expiry_date,
    issuing_authority,
    territory,
    country,
    license_number,
    license_class
);
fields!(
    IdentityDocumentCredential,
    issuing_country,
    document_number,
    identification_number,
    nationality,
    full_name,
    birth_date,
    birth_place,
    sex,
    issue_date,
    expiry_date,
    issuing_authority
);
fields!(
    PassportCredential,
    issuing_country,
    passport_type,
    passport_number,
    national_identification_number,
    nationality,
    full_name,
    birth_date,
    birth_place,
    sex,
    issue_date,
    expiry_date,
    issuing_authority
);
fields!(
    PersonNameCredential,
    title,
    given,
    given_informal,
    given2,
    surname_prefix,
    surname,
    surname2,
    credentials,
    generation
);
fields!(CustomFieldsCredential, id, label, fields, extensions);
fields!(FileCredential, id, name, decrypted_size, integrity_hash);
fields!(NoteCredential, content);
fields!(
    PasskeyCredential,
    credential_id,
    rp_id,
    username,
    user_display_name,
    user_handle,
    key,
    fido2_extensions
);
fields!(
    Fido2Extensions,
    hmac_credentials,
    cred_blob,
    large_blob,
    payments
);
fields!(
    Fido2HmacCredentials,
    algorithm,
    cred_with_uv,
    cred_without_uv
);
fields!(Fido2LargeBlob, uncompressed_size, data);
fields!(CredentialScope, urls, android_apps);
fields!(AndroidAppIdCredential, bundle_id, certificate, name);
fields!(AndroidAppCertificateFingerprint, fingerprint, hash_alg);
fields!(SharedExtension, accessors);
fields!(SharingAccessor, ty, account_id, name, permissions);
impl Erase for SharingAccessorType {
    fn erase(self) {
        if let Self::Unknown(v) = self {
            v.erase();
        }
    }
}
impl Erase for SharingAccessorPermission {
    fn erase(self) {
        if let Self::Unknown(v) = self {
            v.erase();
        }
    }
}
impl Erase for OTPHashAlgorithm {
    fn erase(self) {
        if let Self::Unknown(v) = self {
            v.erase();
        }
    }
}
impl Erase for AndroidAppHashAlgorithm {
    fn erase(self) {
        if let Self::Other(v) = self {
            v.erase();
        }
    }
}
impl Erase for Fido2HmacCredentialAlgorithm {
    fn erase(self) {
        if let Self::Other(v) = self {
            v.erase();
        }
    }
}
impl Erase for EditableFieldWifiNetworkSecurityType {
    fn erase(self) {
        if let Self::Other(v) = self {
            v.erase();
        }
    }
}
impl Erase for EditableFieldValue {
    fn erase(self) {
        match self {
            Self::String(x) => x.erase(),
            Self::ConcealedString(x) => x.erase(),
            Self::Email(x) => x.erase(),
            Self::Number(x) => x.erase(),
            Self::Boolean(x) => x.erase(),
            Self::Date(x) => x.erase(),
            Self::YearMonth(x) => x.erase(),
            Self::SubdivisionCode(x) => x.erase(),
            Self::CountryCode(x) => x.erase(),
            Self::WifiNetworkSecurityType(x) => x.erase(),
            _ => {},
        }
    }
}
impl Erase for UnexpectedField {
    fn erase(self) {
        match self {
            Self::String(x) => x.erase(),
            Self::ConcealedString(x) => x.erase(),
            Self::Email(x) => x.erase(),
            Self::Number(x) => x.erase(),
            Self::Boolean(x) => x.erase(),
            Self::Date(x) => x.erase(),
            Self::YearMonth(x) => x.erase(),
            Self::SubdivisionCode(x) => x.erase(),
            Self::CountryCode(x) => x.erase(),
            Self::WifiNetworkSecurityType(x) => x.erase(),
            Self::Unknown { value, .. } => value.erase(),
            _ => {},
        }
    }
}
impl Erase for Credential {
    fn erase(self) {
        match self {
            Self::Address(x) => x.erase(),
            Self::ApiKey(x) => x.erase(),
            Self::BasicAuth(x) => x.erase(),
            Self::CreditCard(x) => x.erase(),
            Self::CustomFields(x) => x.erase(),
            Self::DriversLicense(x) => x.erase(),
            Self::File(x) => x.erase(),
            Self::GeneratedPassword(x) => x.erase(),
            Self::IdentityDocument(x) => x.erase(),
            Self::ItemReference(x) => x.erase(),
            Self::Note(x) => x.erase(),
            Self::Passkey(x) => x.erase(),
            Self::Passport(x) => x.erase(),
            Self::PersonName(x) => x.erase(),
            Self::SshKey(x) => x.erase(),
            Self::Totp(x) => x.erase(),
            Self::Wifi(x) => x.erase(),
            Self::Unknown { ty, content } => {
                ty.erase();
                content.erase();
            },
            _ => {},
        }
    }
}
pub(super) struct Json(pub serde_json::Value);
impl Drop for Json {
    fn drop(&mut self) {
        std::mem::take(&mut self.0).erase();
    }
}
