// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable, signed capability delegation: issuing.
//!
//! The statements and their checks moved to insigne on 2026-09-24 (the insigne
//! proofs plan) and are re-exported here at their old paths until every
//! consumer imports them from insigne. Issuing stays here because it needs a
//! persona's keys: bring [`Issue`] into scope to call
//! `SignedDelegationCertificate::issue` and `SignedDelegationRevocation::issue`.

pub use insigne::delegation::{
    CapabilityScope, DelegationCertificate, DelegationId, DelegationParent, DelegationRevocation,
    SignedDelegationCertificate, SignedDelegationRevocation, delegation_signing_salt, path_covers,
};
use thiserror::Error;

use crate::IdentityProvider;

/// Sign a delegation statement with the key derived for its scope, attested
/// by the provider's master.
pub trait Issue: Sized {
    /// The statement this signs.
    type Statement;

    /// Sign `statement` with `provider`'s keys.
    fn issue<P: IdentityProvider>(
        provider: &P,
        statement: Self::Statement,
    ) -> Result<Self, DelegationError>;
}

impl Issue for SignedDelegationCertificate {
    type Statement = DelegationCertificate;

    fn issue<P: IdentityProvider>(
        provider: &P,
        certificate: DelegationCertificate,
    ) -> Result<Self, DelegationError> {
        if !certificate.is_well_formed() {
            return Err(DelegationError::MalformedCertificate);
        }
        if provider.master_public_key().to_bytes() != certificate.issuer {
            return Err(DelegationError::WrongIssuer);
        }
        let (signer, signature) =
            sign_scoped(provider, &certificate.scope, &certificate.signing_bytes())?;
        Ok(Self::from_parts(certificate, signer, signature))
    }
}

impl Issue for SignedDelegationRevocation {
    type Statement = DelegationRevocation;

    fn issue<P: IdentityProvider>(
        provider: &P,
        revocation: DelegationRevocation,
    ) -> Result<Self, DelegationError> {
        if !revocation.is_well_formed() {
            return Err(DelegationError::MalformedRevocation);
        }
        if provider.master_public_key().to_bytes() != revocation.issuer {
            return Err(DelegationError::WrongIssuer);
        }
        let (signer, signature) =
            sign_scoped(provider, &revocation.scope, &revocation.signing_bytes())?;
        Ok(Self::from_parts(revocation, signer, signature))
    }
}

/// Attest the scope's derived key and sign `message` with it.
fn sign_scoped<P: IdentityProvider>(
    provider: &P,
    scope: &CapabilityScope,
    message: &[u8],
) -> Result<(crate::DerivedKeyAttestation, Vec<u8>), DelegationError> {
    let salt = delegation_signing_salt(scope);
    let signer = provider
        .attest_derived_key(&salt)
        .map_err(|_| DelegationError::Identity)?;
    let keypair = provider
        .derive_keypair(&salt)
        .map_err(|_| DelegationError::Identity)?;
    Ok((signer, keypair.sign(message).to_bytes().to_vec()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
/// Failure while issuing a signed delegation statement.
pub enum DelegationError {
    /// Required certificate fields or time bounds are invalid.
    #[error("delegation certificate is malformed")]
    MalformedCertificate,
    /// Required revocation fields are invalid.
    #[error("delegation revocation is malformed")]
    MalformedRevocation,
    /// The supplied identity is not the statement's declared issuer.
    #[error("signing identity does not match delegation issuer")]
    WrongIssuer,
    /// The identity provider could not produce its scoped signing proof.
    #[error("identity provider could not derive delegation signer")]
    Identity,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryProvider;

    fn scope(path: &str, actions: &[&str]) -> CapabilityScope {
        CapabilityScope {
            domain: "moot".into(),
            resource: vec![9; 32],
            path_prefix: path.into(),
            actions: actions.iter().map(|action| (*action).into()).collect(),
        }
    }

    fn certificate(
        provider: &InMemoryProvider,
        parent: DelegationParent,
        subject: [u8; 32],
        path: &str,
        actions: &[&str],
        depth: u16,
    ) -> DelegationCertificate {
        DelegationCertificate::new(
            parent,
            provider.master_public_key().to_bytes(),
            subject,
            scope(path, actions),
            5,
            10,
            Some(100),
            depth,
            [depth as u8 + 1; 32],
        )
    }

    #[test]
    fn certificate_verifies_and_binds_every_signed_field() {
        let issuer = InMemoryProvider::from_seed([1; 32]);
        let subject = InMemoryProvider::from_seed([2; 32]);
        let signed = SignedDelegationCertificate::issue(
            &issuer,
            certificate(
                &issuer,
                DelegationParent::Root([7; 32]),
                subject.master_public_key().to_bytes(),
                "moot/fauna",
                &["read", "write"],
                2,
            ),
        )
        .unwrap();
        assert!(signed.verify());

        let mut tampered = signed.clone();
        tampered.certificate.scope.path_prefix = "moot/secret".into();
        assert!(!tampered.verify());
    }

    #[test]
    fn child_must_narrow_scope_time_and_delegation_depth() {
        let root_holder = InMemoryProvider::from_seed([1; 32]);
        let child_holder = InMemoryProvider::from_seed([2; 32]);
        let leaf = InMemoryProvider::from_seed([3; 32]);
        let parent = certificate(
            &root_holder,
            DelegationParent::Root([7; 32]),
            child_holder.master_public_key().to_bytes(),
            "moot/fauna",
            &["read", "write"],
            2,
        );
        let child = certificate(
            &child_holder,
            DelegationParent::Certificate(parent.id()),
            leaf.master_public_key().to_bytes(),
            "moot/fauna/research",
            &["read"],
            1,
        );
        assert!(child.attenuates(&parent));

        let mut widened = child.clone();
        widened.scope.path_prefix = "moot".into();
        assert!(!widened.attenuates(&parent));
        let mut endless = child.clone();
        endless.expires_at_ms = None;
        assert!(!endless.attenuates(&parent));
        let mut deep = child;
        deep.remaining_delegation_depth = 2;
        assert!(!deep.attenuates(&parent));
    }

    #[test]
    fn revocation_is_bound_to_issuer_scope_and_target() {
        let issuer = InMemoryProvider::from_seed([1; 32]);
        let signed = SignedDelegationRevocation::issue(
            &issuer,
            DelegationRevocation::new(
                DelegationId([8; 32]),
                issuer.master_public_key().to_bytes(),
                scope("moot/fauna", &["write"]),
                50,
                [5; 32],
            ),
        )
        .unwrap();
        assert!(signed.verify());

        let mut tampered = signed;
        tampered.revocation.certificate = DelegationId([9; 32]);
        assert!(!tampered.verify());
    }
}
