// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The secret-free face of one OTP credential: the chatelaine item holding
//! it, and which of the item's credentials the gate exercises.

use chatelaine::{
    Credential, CredentialId, CredentialKind, Item, ItemId, OtpAlgorithm, OtpCodeStyle, OtpMode,
};

/// An item and the `Otp` credential in it that a petition or tile is about.
///
/// Built only by castellan, which checks that the credential is in the item
/// and is an `Otp` credential. Every field may be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtpCredential {
    item: Item,
    credential: CredentialId,
}

/// The display fields of an `Otp` credential.
#[derive(Clone, Copy)]
pub(super) struct OtpFields<'a> {
    pub(super) account: &'a str,
    pub(super) issuer: Option<&'a str>,
    pub(super) algorithm: OtpAlgorithm,
    pub(super) code_style: OtpCodeStyle,
    pub(super) mode: OtpMode,
}

impl<'a> OtpFields<'a> {
    pub(super) fn of(credential: &'a Credential) -> Option<Self> {
        match &credential.kind {
            CredentialKind::Otp {
                account,
                issuer,
                algorithm,
                code_style,
                mode,
            } => Some(Self {
                account,
                issuer: issuer.as_deref(),
                algorithm: *algorithm,
                code_style: *code_style,
                mode: *mode,
            }),
            _ => None,
        }
    }
}

impl OtpCredential {
    /// Pair an item with one of its credentials, if that credential is OTP.
    pub(crate) fn from_item(item: Item, credential: CredentialId) -> Option<Self> {
        let held = item
            .credentials
            .iter()
            .find(|candidate| candidate.id == credential)?;
        OtpFields::of(held)?;
        Some(Self { item, credential })
    }

    /// The whole item, every credential's metadata included.
    pub fn item(&self) -> &Item {
        &self.item
    }

    /// The item's id.
    pub fn item_id(&self) -> ItemId {
        self.item.id
    }

    /// The exercised credential's id.
    pub fn credential_id(&self) -> CredentialId {
        self.credential
    }

    /// The account the codes are for.
    pub fn account(&self) -> &str {
        self.fields().account
    }

    /// The issuing service, when the import named one.
    pub fn issuer(&self) -> Option<&str> {
        self.fields().issuer
    }

    /// The HMAC hash behind each code.
    pub fn algorithm(&self) -> OtpAlgorithm {
        self.fields().algorithm
    }

    /// How each code is written.
    pub fn code_style(&self) -> OtpCodeStyle {
        self.fields().code_style
    }

    /// Time-based with its period, or counter-based. The counter itself is
    /// sealed, never shown.
    pub fn mode(&self) -> OtpMode {
        self.fields().mode
    }

    fn fields(&self) -> OtpFields<'_> {
        self.item
            .credentials
            .iter()
            .find(|candidate| candidate.id == self.credential)
            .and_then(OtpFields::of)
            .expect("an OtpCredential is built only around an Otp credential it holds")
    }
}
