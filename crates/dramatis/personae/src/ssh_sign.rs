// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Agent signatures, by key type.
//!
//! - Ed25519 and ECDSA (P-256, P-384, P-521) sign through `ssh-key`'s
//!   per-type signers, which are RustCrypto's.
//! - RSA signs through `ring`, `rsa-sha2-256` or `rsa-sha2-512` as the
//!   request's flags ask. The `rsa` crate's private-key operations are not
//!   constant-time (RUSTSEC-2023-0071, unpatched), so nothing here reaches
//!   them: `PrivateKey::try_sign` is never called, because its RSA arm is
//!   the `rsa` crate's.
//!
//! OpenSSH keys store `d`, `p`, `q` and `q⁻¹ mod p` but not the CRT
//! exponents ring needs, so `d mod (p-1)` and `d mod (q-1)` are derived on
//! each load with `crypto-bigint`, whose reduction is constant-time in the
//! dividend and varies only with the divisor's bit length, which is public.

use crypto_bigint::{Encoding, NonZero, U4096};
use signature::Signer;
use ssh_key::private::{KeypairData, RsaKeypair};
use ssh_key::{Algorithm, HashAlg, Mpint, Signature};
use zeroize::{Zeroize, Zeroizing};

/// `SSH_AGENT_RSA_SHA2_256` (draft-miller-ssh-agent §3.6.1).
pub const SSH_AGENT_RSA_SHA2_256: u32 = 0x02;
/// `SSH_AGENT_RSA_SHA2_512`.
pub const SSH_AGENT_RSA_SHA2_512: u32 = 0x04;

/// ring's RSA bounds: `n` of 2048 to 4096 bits, each prime a multiple of
/// 512 bits. Width of the CRT arithmetic, in bytes.
const RSA_MAX_BYTES: usize = 512;

/// Why a signature was not produced.
#[derive(Debug, thiserror::Error)]
pub enum SshSignError {
    /// An RSA request without a SHA-2 flag asks for `ssh-rsa` (SHA-1).
    #[error("RSA signing needs rsa-sha2-256 or rsa-sha2-512; ssh-rsa (SHA-1) is not signed")]
    RsaSha1Refused,
    /// ring would not load the key (size, exponent or consistency).
    #[error("RSA key not usable for signing: {0}")]
    RsaKeyRejected(String),
    /// A key type the agent does not sign.
    #[error("the agent does not sign {0} keys")]
    Unsupported(String),
    /// The signer failed.
    #[error("signing failed")]
    Failed,
}

/// Sign `data` with `key`, honouring the agent protocol's flags.
///
/// Flags matter only for RSA, where `SHA2_256` wins if both are set, as in
/// OpenSSH's own agent.
pub fn sign(key: &KeypairData, data: &[u8], flags: u32) -> Result<Signature, SshSignError> {
    match key {
        KeypairData::Ed25519(keypair) => keypair.try_sign(data).map_err(|_| SshSignError::Failed),
        KeypairData::Ecdsa(keypair) => keypair.try_sign(data).map_err(|_| SshSignError::Failed),
        KeypairData::Rsa(keypair) => sign_rsa(keypair, data, rsa_hash(flags)?),
        other => Err(SshSignError::Unsupported(
            other
                .algorithm()
                .map(|algorithm| algorithm.to_string())
                .unwrap_or_else(|_| "encrypted".into()),
        )),
    }
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

/// Load an OpenSSH RSA key into ring, deriving the CRT exponents.
pub fn ring_keypair(keypair: &RsaKeypair) -> Result<ring::rsa::KeyPair, SshSignError> {
    let n = positive(&keypair.public.n)?;
    let e = positive(&keypair.public.e)?;
    let d = positive(&keypair.private.d)?;
    let p = positive(&keypair.private.p)?;
    let q = positive(&keypair.private.q)?;
    let q_inv = positive(&keypair.private.iqmp)?;
    let dp = crt_exponent(d, p)?;
    let dq = crt_exponent(d, q)?;
    let components = ring::rsa::KeyPairComponents {
        public_key: ring::rsa::PublicKeyComponents { n, e },
        d,
        p,
        q,
        dP: &dp[..],
        dQ: &dq[..],
        qInv: q_inv,
    };
    ring::rsa::KeyPair::from_components(&components)
        .map_err(|rejected| SshSignError::RsaKeyRejected(rejected.to_string()))
}

fn positive(value: &Mpint) -> Result<&[u8], SshSignError> {
    value
        .as_positive_bytes()
        .filter(|bytes| !bytes.is_empty() && bytes.len() <= RSA_MAX_BYTES)
        .ok_or_else(|| SshSignError::RsaKeyRejected("component out of range".into()))
}

/// `d mod (prime - 1)`, big-endian, as wide as `prime`. Zeroizing is best
/// effort: `U4096` is `Copy`, so the arithmetic leaves stack copies.
fn crt_exponent(d: &[u8], prime: &[u8]) -> Result<Zeroizing<Vec<u8>>, SshSignError> {
    let mut d = widen(d);
    let mut modulus = widen(prime).wrapping_sub(&U4096::ONE);
    let divisor = Option::<NonZero<U4096>>::from(NonZero::new(modulus))
        .ok_or_else(|| SshSignError::RsaKeyRejected("prime is one".into()))?;
    let mut remainder = d.rem(&divisor);
    let bytes = Zeroizing::new(remainder.to_be_bytes());
    d.zeroize();
    modulus.zeroize();
    remainder.zeroize();
    Ok(Zeroizing::new(
        bytes[RSA_MAX_BYTES - prime.len()..].to_vec(),
    ))
}

fn widen(bytes: &[u8]) -> U4096 {
    let mut padded = Zeroizing::new([0u8; RSA_MAX_BYTES]);
    padded[RSA_MAX_BYTES - bytes.len()..].copy_from_slice(bytes);
    U4096::from_be_slice(&padded[..])
}

#[cfg(test)]
#[path = "ssh_sign_tests.rs"]
pub(crate) mod tests;
