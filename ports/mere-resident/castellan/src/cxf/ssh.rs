// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::ImportError;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rsa::pkcs8::DecodePrivateKey;
use ssh_key::private::{EcdsaKeypair, Ed25519Keypair, RsaKeypair};

pub(super) fn decode(
    kind: &str,
    der: &[u8],
    comment: &str,
) -> Result<ssh_key::PrivateKey, ImportError> {
    let bad = || ImportError("invalid or unsupported PKCS#8 SSH key");
    let mut key: ssh_key::PrivateKey = match kind {
        "ssh-ed25519" => {
            Ed25519Keypair::from(ed25519_dalek::SigningKey::from_pkcs8_der(der).map_err(|_| bad())?)
                .into()
        },
        "ssh-rsa" => {
            RsaKeypair::try_from(rsa::RsaPrivateKey::from_pkcs8_der(der).map_err(|_| bad())?)
                .map_err(|_| bad())?
                .into()
        },
        "ecdsa-sha2-nistp256" => {
            let secret = p256::SecretKey::from_pkcs8_der(der).map_err(|_| bad())?;
            EcdsaKeypair::NistP256 {
                public: secret.public_key().to_encoded_point(false),
                private: secret.into(),
            }
            .into()
        },
        "ecdsa-sha2-nistp384" => {
            let secret = p384::SecretKey::from_pkcs8_der(der).map_err(|_| bad())?;
            EcdsaKeypair::NistP384 {
                public: secret.public_key().to_encoded_point(false),
                private: secret.into(),
            }
            .into()
        },
        _ => return Err(bad()),
    };
    key.set_comment(comment);
    crate::custody::ssh_sign::check_signable(key.key_data()).map_err(|_| bad())?;
    Ok(key)
}
