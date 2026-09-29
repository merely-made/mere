// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The delegation adapter: chain validation and the local revocation ledger.
//!
//! insigne supplies the statement grammar (certificates, attenuation,
//! revocations) and its checks, and personae issues statements; this module
//! walks a presented chain against the owner's accepted roots, depth budget,
//! clock, and revocation ledger. It works on checked statements only, and
//! creates none of its own.

use std::collections::BTreeMap;

use insigne::delegation::{
    DelegationCertificate, DelegationId, DelegationParent, SignedDelegationCertificate,
};
use insigne::{CheckedCertificate, CheckedRevocation};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::types::ChainFault;

/// A root authority this node accepts chains from.
///
/// A chain's first certificate must claim `DelegationParent::Root(authority)`
/// and be issued by exactly `issuer`; the pair is what the owner trusts, not
/// the authority id alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedRoot {
    /// Application-owned root authority id.
    pub authority: [u8; 32],
    /// Personae master public key entitled to issue under that authority.
    pub issuer: [u8; 32],
}

/// This node's record of certificates their issuers have withdrawn.
///
/// The ledger is local (plan D5): it folds verified revocation statements
/// and answers membership during chain validation. Revoking a parent
/// cascades to every chain below it, because every chain that relies on the
/// parent must present it and validation checks each presented certificate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RevocationLedger {
    revoked: BTreeMap<DelegationId, [u8; 32]>,
}

#[derive(Serialize, Deserialize)]
struct RevocationRecord {
    certificate: DelegationId,
    issuer: [u8; 32],
}

impl Serialize for RevocationLedger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.revoked
            .iter()
            .map(|(certificate, issuer)| RevocationRecord {
                certificate: *certificate,
                issuer: *issuer,
            })
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RevocationLedger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let records = Vec::<RevocationRecord>::deserialize(deserializer)?;
        Ok(Self {
            revoked: records
                .into_iter()
                .map(|record| (record.certificate, record.issuer))
                .collect(),
        })
    }
}

impl RevocationLedger {
    /// An empty ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one checked revocation statement.
    ///
    /// Only a checked statement folds: check it first with
    /// [`insigne::SignedDelegationRevocation::check`], so the ledger never holds a
    /// withdrawal whose signature did not check.
    pub fn fold(&mut self, revocation: CheckedRevocation<'_>) {
        let statement = revocation.revocation();
        self.revoked.insert(statement.certificate, statement.issuer);
    }

    /// Whether this ledger revokes the given certificate.
    ///
    /// A recorded statement counts only when its declared issuer matches the
    /// certificate's issuer: nobody withdraws authority they did not grant.
    pub fn revokes(&self, certificate: &DelegationCertificate) -> bool {
        self.revoked.get(&certificate.id()) == Some(&certificate.issuer)
    }

    /// Number of recorded revocations.
    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    /// Whether the ledger is empty.
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }
}

/// Validate one presented delegation chain, root grant first, subject last.
///
/// The checks, in order: chain presence and depth budget, every signature
/// and signer attestation, termination at a locally accepted root, link
/// integrity and strict attenuation at every step, revocation of any member,
/// validity window of every member at `now_ms`, and finally that the leaf
/// names `subject`. Whether the leaf covers a concrete path and action is
/// the caller's question, asked afterwards via
/// [`DelegationCertificate::covers`] of the checked leaf this returns.
pub fn validate_chain<'a>(
    chain: &'a [SignedDelegationCertificate],
    subject: [u8; 32],
    trusted_roots: &[TrustedRoot],
    ledger: &RevocationLedger,
    max_depth: u16,
    now_ms: u64,
) -> Result<CheckedCertificate<'a>, ChainFault> {
    if chain.is_empty() {
        return Err(ChainFault::Empty);
    }
    if chain.len() > usize::from(max_depth) {
        return Err(ChainFault::DepthExceeded);
    }
    let checked = chain
        .iter()
        .map(SignedDelegationCertificate::check)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ChainFault::BadSignature)?;

    let first = checked[0].certificate();
    let anchored = match first.parent {
        DelegationParent::Root(authority) => trusted_roots
            .iter()
            .any(|root| root.authority == authority && root.issuer == first.issuer),
        DelegationParent::Certificate(_) => false,
    };
    if !anchored {
        return Err(ChainFault::UntrustedRoot);
    }

    for pair in checked.windows(2) {
        let parent = pair[0].certificate();
        let child = pair[1].certificate();
        if child.parent != DelegationParent::Certificate(parent.id())
            || child.issuer != parent.subject
        {
            return Err(ChainFault::BrokenLink);
        }
        if !child.attenuates(parent) {
            return Err(ChainFault::NotAttenuated);
        }
    }

    for link in &checked {
        let certificate = link.certificate();
        if ledger.revokes(certificate) {
            return Err(ChainFault::Revoked);
        }
        if now_ms < certificate.not_before_ms {
            return Err(ChainFault::NotYetValid);
        }
        if certificate
            .expires_at_ms
            .is_some_and(|expires| now_ms > expires)
        {
            return Err(ChainFault::Expired);
        }
    }

    let leaf = checked[checked.len() - 1];
    if leaf.certificate().subject != subject {
        return Err(ChainFault::SubjectMismatch);
    }
    Ok(leaf)
}

#[cfg(test)]
mod tests {
    use insigne::delegation::{CapabilityScope, DelegationRevocation, SignedDelegationRevocation};
    use personae::delegation::Issue;
    use personae::{IdentityProvider, InMemoryProvider};

    use super::*;

    fn scope() -> CapabilityScope {
        CapabilityScope {
            domain: "mere.network".into(),
            resource: vec![3; 32],
            path_prefix: "/services/murm".into(),
            actions: ["connect".to_string()].into_iter().collect(),
        }
    }

    #[test]
    fn only_a_checked_revocation_folds() {
        let issuer = InMemoryProvider::from_seed([1; 32]);
        let signed = SignedDelegationRevocation::issue(
            &issuer,
            DelegationRevocation::new(
                DelegationId([8; 32]),
                issuer.master_public_key().to_bytes(),
                scope(),
                50,
                [5; 32],
            ),
        )
        .unwrap();

        // A tampered statement yields no conclusion, so there is nothing to fold.
        let mut tampered = signed.clone();
        tampered.revocation.at_ms = 51;
        assert_eq!(tampered.check(), Err(insigne::CheckFault::BadSignature));

        let mut ledger = RevocationLedger::new();
        ledger.fold(signed.check().expect("an issued revocation checks"));
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn a_revocation_by_a_different_issuer_does_not_count() {
        let issuer = InMemoryProvider::from_seed([1; 32]);
        let stranger = InMemoryProvider::from_seed([2; 32]);
        let certificate = DelegationCertificate::new(
            DelegationParent::Root([7; 32]),
            issuer.master_public_key().to_bytes(),
            [9; 32],
            scope(),
            5,
            10,
            Some(100),
            1,
            [1; 32],
        );
        let signed = SignedDelegationRevocation::issue(
            &stranger,
            DelegationRevocation::new(
                certificate.id(),
                stranger.master_public_key().to_bytes(),
                scope(),
                50,
                [5; 32],
            ),
        )
        .unwrap();

        let mut ledger = RevocationLedger::new();
        ledger.fold(signed.check().expect("a stranger's own revocation checks"));
        assert!(
            !ledger.revokes(&certificate),
            "a stranger cannot withdraw authority they did not grant"
        );
    }
}
