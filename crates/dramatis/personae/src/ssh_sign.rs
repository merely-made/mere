// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Agent signatures, by key type, and which keys the agent will hold.
//!
//! - Ed25519 and ECDSA P-256 and P-384 sign through `ssh-key`'s per-type
//!   signers, which are RustCrypto's.
//! - RSA signs through `ring`, `rsa-sha2-256` or `rsa-sha2-512` as the
//!   request's flags ask. The `rsa` crate builds the key once per signature,
//!   when the agent decodes it from the vault, and never signs or decrypts
//!   (rulings 52, 59, 63; RUSTSEC-2023-0071).
//!   `PrivateKey::try_sign` is never called: its RSA arm is the `rsa` crate's.
//! - P-521 is refused until `ssh-key` decodes every P-521 scalar (ruling 55).
//!
//! [`check_signable`] is the door: import and `ssh-add` refuse what it
//! refuses, with its reason (ruling 56).

use rsa::pkcs8::EncodePrivateKey;
use signature::Signer;
use ssh_key::private::{EcdsaKeypair, KeypairData, RsaKeypair};
use ssh_key::{Algorithm, HashAlg, Mpint, Signature};

/// `SSH_AGENT_RSA_SHA2_256` (draft-miller-ssh-agent §3.6.1).
pub const SSH_AGENT_RSA_SHA2_256: u32 = 0x02;
/// `SSH_AGENT_RSA_SHA2_512`.
pub const SSH_AGENT_RSA_SHA2_512: u32 = 0x04;

/// What ring signs, stated in refusals.
const RSA_LIMITS: &str = "the agent signs RSA keys of 2048 to 4096 bits whose primes are a multiple of 512 bits, with e of at least 65537";

/// Why a key is refused or a signature was not produced.
#[derive(Debug, thiserror::Error)]
pub enum SshSignError {
    /// An RSA request without a SHA-2 flag asks for `ssh-rsa` (SHA-1).
    #[error("RSA signing needs rsa-sha2-256 or rsa-sha2-512; ssh-rsa (SHA-1) is not signed")]
    RsaSha1Refused,
    /// The key could not be built, or ring would not load it.
    #[error("RSA key refused ({0}); {RSA_LIMITS}")]
    RsaKeyRejected(String),
    /// Ruling 55.
    #[error(
        "ECDSA P-521 keys are refused until ssh-key decodes every P-521 private scalar upstream"
    )]
    P521Refused,
    /// A key type the agent does not sign.
    #[error("the agent does not sign {0} keys")]
    Unsupported(String),
    /// The signer failed.
    #[error("signing failed")]
    Failed,
}

/// Whether the agent can sign with `key`; the import and `ssh-add` door.
pub fn check_signable(key: &KeypairData) -> Result<(), SshSignError> {
    match key {
        KeypairData::Ed25519(_) => Ok(()),
        KeypairData::Ecdsa(EcdsaKeypair::NistP521 { .. }) => Err(SshSignError::P521Refused),
        KeypairData::Ecdsa(_) => Ok(()),
        KeypairData::Rsa(keypair) => ring_keypair(keypair).map(drop),
        other => Err(unsupported(other)),
    }
}

/// Sign `data` with `key`, honouring the agent protocol's flags.
///
/// Flags matter only for RSA, where `SHA2_256` wins if both are set, as in
/// OpenSSH's own agent.
pub fn sign(key: &KeypairData, data: &[u8], flags: u32) -> Result<Signature, SshSignError> {
    match key {
        KeypairData::Ed25519(keypair) => keypair.try_sign(data).map_err(|_| SshSignError::Failed),
        KeypairData::Ecdsa(EcdsaKeypair::NistP521 { .. }) => Err(SshSignError::P521Refused),
        KeypairData::Ecdsa(keypair) => keypair.try_sign(data).map_err(|_| SshSignError::Failed),
        KeypairData::Rsa(keypair) => sign_rsa(keypair, data, rsa_hash(flags)?),
        other => Err(unsupported(other)),
    }
}

fn unsupported(key: &KeypairData) -> SshSignError {
    SshSignError::Unsupported(
        key.algorithm()
            .map(|algorithm| algorithm.to_string())
            .unwrap_or_else(|_| "encrypted".into()),
    )
}

/// The SHA-2 hash an RSA request's flags select.
pub fn rsa_hash(flags: u32) -> Result<HashAlg, SshSignError> {
    if flags & SSH_AGENT_RSA_SHA2_256 != 0 {
        Ok(HashAlg::Sha256)
    } else if flags & SSH_AGENT_RSA_SHA2_512 != 0 {
        Ok(HashAlg::Sha512)
    } else {
        Err(SshSignError::RsaSha1Refused)
    }
}

fn sign_rsa(keypair: &RsaKeypair, data: &[u8], hash: HashAlg) -> Result<Signature, SshSignError> {
    let ring_key = ring_keypair(keypair)?;
    let padding: &'static dyn ring::signature::RsaEncoding = match hash {
        HashAlg::Sha256 => &ring::signature::RSA_PKCS1_SHA256,
        HashAlg::Sha512 => &ring::signature::RSA_PKCS1_SHA512,
        _ => return Err(SshSignError::Failed),
    };
    let mut signature = vec![0u8; ring_key.public().modulus_len()];
    ring_key
        .sign(
            padding,
            &ring::rand::SystemRandom::new(),
            data,
            &mut signature,
        )
        .map_err(|_| SshSignError::Failed)?;
    Signature::new(Algorithm::Rsa { hash: Some(hash) }, signature).map_err(|_| SshSignError::Failed)
}

/// Load an OpenSSH RSA key into ring.
pub fn ring_keypair(keypair: &RsaKeypair) -> Result<ring::rsa::KeyPair, SshSignError> {
    let int = |value: &Mpint| {
        value
            .as_positive_bytes()
            .map(rsa::BigUint::from_bytes_be)
            .ok_or_else(|| SshSignError::RsaKeyRejected("a component is not positive".into()))
    };
    // Rulings 59, 63: the rsa crate builds the key here, once per signature,
    // when the agent decodes it from the vault. Its standard construction
    // validates it and derives the CRT values OpenSSH does not store; its
    // PKCS#8 export hands ring the full key. Every signature is ring's.
    let built = rsa::RsaPrivateKey::from_components(
        int(&keypair.public.n)?,
        int(&keypair.public.e)?,
        int(&keypair.private.d)?,
        vec![int(&keypair.private.p)?, int(&keypair.private.q)?],
    )
    .map_err(|error| SshSignError::RsaKeyRejected(error.to_string()))?;
    let pkcs8 = built
        .to_pkcs8_der()
        .map_err(|error| SshSignError::RsaKeyRejected(error.to_string()))?;
    drop(built);
    let loaded = ring::rsa::KeyPair::from_pkcs8(pkcs8.as_bytes())
        .map_err(|rejected| SshSignError::RsaKeyRejected(rejected.to_string()));
    // `SecretDocument` zeroizes on drop; ring holds its own copy.
    drop(pkcs8);
    loaded
}

#[cfg(test)]
#[path = "ssh_sign_tests.rs"]
pub(crate) mod tests;
