// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **chatelaine**, the secret half of the Mere platform's credential model,
//! as a plain item taxonomy.
//!
//! Named for the waist-worn chain that held the household's keys, and by
//! extension the keeper of them. The chatelaine holds what must never be
//! shown: passwords, 2FA seeds, tokens, foreign key material. Everything it
//! stands for is *damaged by disclosure*: a password shown is burned, a TOTP
//! seed shown is cloned, a bearer token shown is stolen. Chatelaine items are
//! exercised (filled, generated, released through the gate), never
//! presented.
//!
//! So this crate holds none of it. It is the taxonomy (ruling 7): the items,
//! their credentials' kinds and the secret-free metadata of each, plain
//! serializable data that any host view may show without harm. The secrets
//! are castellan's, sealed one payload per credential.
//!
//! The boundaries are the point:
//!
//! - **Not the proofs.** That is `insigne`: public-key artifacts made to be
//!   shown. The insigne/chatelaine boundary is cryptographic, not filing.
//! - **Not the keeper.** That is `castellan`, which seals the secrets,
//!   exercises the items behind gate petitions, and runs every import.
//! - **Not the substrate.** Storage and sealing are personae's vault.
//!
//! chatelaine holds no secret bytes, does no storage and no cryptography, and
//! reads neither a clock nor a random source: ids are built from bytes the
//! caller supplies, and timestamps are Unix seconds the caller supplies.
//!
//! ## The shape
//!
//! An [`Item`] is CXF-shaped (ruling 16): a titled container of typed
//! [`Credential`]s, with a subtitle, a [`Scope`] of sites and Android apps,
//! tags, a favorite flag and timestamps, and an [`ItemState`]: in the vault,
//! or quarantined for the user's review. A [`Collection`] groups items and
//! nests. A [`Link`] refers to an item, as an `item-reference` credential and
//! a collection's members do. An item or collection imported from CXF keeps
//! its id in that file as a [`SourceId`], so an export can write it back
//! (ruling 36); chatelaine's own ids are UUIDs.
//!
//! [`CredentialKind`] covers CXF v1.0's 17 types ([`CXF_V1_TYPES`]), the
//! Freedesktop Secret Service's generic secret, and `Unknown`, which keeps the
//! type string of a kind newer than v1.0. [`disposition`] gives each kind's
//! ruled import treatment (rulings 10 to 15 and 25).
//!
//! ## The metadata line
//!
//! Ruled 2026-10-01 (rulings 23 and 24): **identifying fields only**. Every
//! item shows its title, subtitle, kind, scope, tags, favorite flag and
//! timestamps. Per kind, chatelaine holds only what tells two credentials
//! apart; castellan seals everything else. This is stricter than CXF's own
//! `concealed-string` line, which leaves document numbers, addresses, names
//! and note contents displayable.
//!
//! | Kind | chatelaine holds | castellan seals |
//! |---|---|---|
//! | basic-auth | username | password |
//! | generated-password | nothing | password |
//! | totp (the `Otp` kind) | account, issuer, algorithm, code style (digits), period and t0, or HOTP | secret, HOTP counter |
//! | api-key | username, key type, URL | key, validity dates |
//! | wifi | SSID, security type | passphrase, hidden flag |
//! | passkey | rpId, username, user display name | key, credential id, user handle, extensions |
//! | ssh-key | key type, fingerprint (derived), comment | private key, dates, generation source |
//! | credit-card | card type, expiry, last four (derived at import) | number, verification number, PIN, full name, valid-from |
//! | passport | document code, issuing country, expiry | numbers, names, birth data, sex, nationality, issue date, authority |
//! | drivers-license | issuing country, territory, expiry | number, name, birth date, issue date, authority, licence class |
//! | identity-document | issuing country, expiry | numbers, names, birth data, sex, nationality, issue date, authority |
//! | address, person-name | nothing | every field |
//! | note | nothing | content |
//! | custom-fields | the section label and each field's label | field values |
//! | file | name, size, integrity hash | bytes |
//! | item-reference | the link | nothing |
//! | Secret Service secret | content type, lookup attributes; its label is the item title | bytes |
//! | unknown | the CXF type string | every field, verbatim |
//!
//! Derived values (a card's last four, an SSH key's fingerprint) are derived
//! by castellan, which holds the secret they come from; chatelaine only
//! checks their form. Values with a closed form are checked when made and
//! when loaded: [`LastFour`], [`CountryCode`], [`SubdivisionCode`], [`Date`],
//! [`YearMonth`], [`SshFingerprint`] and [`SourceId`].
//!
//! ## One-time passwords
//!
//! CXF's `totp` is the [`CredentialKind::Otp`] kind, which also describes
//! HOTP and Steam Guard items (ruling 37). [`OtpAlgorithm`] and
//! [`OtpCodeStyle`] moved here from castellan, which re-exports them, so each
//! has one definition. [`OtpMode`] is the shape only: time-based with its
//! period and t0, or counter-based. The HOTP counter is mutable,
//! freshness-critical state, so castellan's own `OtpKind` keeps it.
//!
//! ## Wire formats
//!
//! Every type serializes with serde, and is tested through JSON and postcard.
//! Enums are externally tagged and optional fields are always written, so the
//! non-self-describing binary formats work too. Variant order is the binary
//! format: new kinds go at the end.

#![warn(missing_docs)]
#![doc(html_no_source)]

mod disposition;
mod id;
mod item;
mod kind;
mod otp;
#[cfg(test)]
mod samples;
mod value;

pub use disposition::{Disposition, disposition};
pub use id::{CollectionId, CredentialId, IdParseError, ItemId};
pub use item::{AndroidApp, AppCertificate, Collection, Credential, Item, ItemState, Link, Scope};
pub use kind::{CXF_V1_TYPES, CredentialKind, WifiSecurity};
pub use otp::{OtpAlgorithm, OtpCodeStyle, OtpMode};
pub use value::{
    CountryCode, Date, LastFour, SourceId, SshFingerprint, SubdivisionCode, ValueError, YearMonth,
};
