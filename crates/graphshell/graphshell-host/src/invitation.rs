// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One owner inviting one guest into one session, over the holder's custody
//! identity. Lifted from Graphshell co-op (stack lift plan, case 1060) so a
//! second product admits its peers the same way.
//!
//! A [`Contact`] is a person's attested transport key under the product's
//! salt; an [`Invitation`] is the owner's delegation to one guest's key for
//! one session, which the guest presents in its hello and the owner checks on
//! admission ([`Invitation::admits`]); a [`Card`] is the owner's contact plus
//! the transport ticket a guest dials. [`InvitationTerms`] carries what
//! differs per product: the admission domain, the key salt, the network label
//! and the projection path. Nothing here signs except through
//! [`personae::IdentityProvider`] and the holder's [`Signer`]; the root acts
//! stay in the custodian.

use crate::admission::open_session_for;
use crate::network_carrier::projection_binding;
use insigne::{
    DerivedKeyAttestation,
    delegation::{
        CapabilityScope, DelegationCertificate, DelegationParent, DelegationRevocation,
        SignedDelegationCertificate, SignedDelegationRevocation,
    },
};
use notochord::{
    AdmittedPrincipal, NetworkId, ProfileRef, RequestedAction, SessionHello, TrafficClass,
};
use personae::IdentityProvider;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What one product's invitations name. A guest's key is derived under
/// `salt`; its grant covers `domain` at `path_root/<session>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvitationTerms {
    pub domain: &'static str,
    pub salt: &'static [u8],
    pub network_label: &'static [u8],
    pub path_root: &'static str,
}

impl InvitationTerms {
    pub fn path(&self, session: Uuid) -> String {
        format!("{}/{}", self.path_root, session)
    }
    pub fn network(&self, owner: [u8; 32], session: Uuid) -> [u8; 32] {
        *blake3::hash(&[self.network_label, &owner, session.as_bytes()].concat()).as_bytes()
    }
    pub fn scope(&self, owner: [u8; 32], session: Uuid) -> CapabilityScope {
        CapabilityScope {
            domain: self.domain.into(),
            resource: self.network(owner, session).to_vec(),
            path_prefix: self.path(session),
            actions: ["connect".into()].into_iter().collect(),
        }
    }
    fn invitation_scope(&self, owner: [u8; 32], session: Uuid, write: bool) -> CapabilityScope {
        let mut value = self.scope(owner, session);
        if write {
            value.actions.insert("edit".into());
        }
        value
    }
}

/// Root acts stay in the custodian. Tests inject an in-memory owner explicitly.
pub trait Signer: IdentityProvider {
    fn certificate(
        &self,
        value: DelegationCertificate,
    ) -> Result<SignedDelegationCertificate, String>;
    fn revocation(&self, value: DelegationRevocation)
    -> Result<SignedDelegationRevocation, String>;
}

/// An in-memory owner, which tests inject explicitly; a product's owner signs
/// through its custodian. Test builds only: no product enables `test-signer`
/// outside its dev-dependencies (stack-lift D4).
#[cfg(any(test, feature = "test-signer"))]
impl Signer for personae::InMemoryProvider {
    fn certificate(
        &self,
        value: DelegationCertificate,
    ) -> Result<SignedDelegationCertificate, String> {
        use personae::delegation::Issue;
        SignedDelegationCertificate::issue(self, value).map_err(|e| e.to_string())
    }
    fn revocation(
        &self,
        value: DelegationRevocation,
    ) -> Result<SignedDelegationRevocation, String> {
        use personae::delegation::Issue;
        SignedDelegationRevocation::issue(self, value).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "custody-client")]
impl Signer for castellan_client::custody_identity::CustodyIdentity {
    fn certificate(
        &self,
        value: DelegationCertificate,
    ) -> Result<SignedDelegationCertificate, String> {
        self.issue_certificate(value).map_err(|e| e.to_string())
    }
    fn revocation(
        &self,
        value: DelegationRevocation,
    ) -> Result<SignedDelegationRevocation, String> {
        self.issue_revocation(value).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub version: u16,
    pub binding: DerivedKeyAttestation,
}

impl Contact {
    pub fn create(identity: &dyn IdentityProvider, terms: &InvitationTerms) -> Result<Self, String> {
        Ok(Self {
            version: 1,
            binding: identity
                .attest_derived_key(terms.salt)
                .map_err(|e| e.to_string())?,
        })
    }
    pub fn check(&self, terms: &InvitationTerms) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported contact version".into());
        }
        self.binding
            .check(terms.salt)
            .map_err(|e| format!("invalid contact attestation: {e:?}"))?;
        Ok(())
    }
    pub fn root(&self) -> [u8; 32] {
        *self.binding.master()
    }
    pub fn peer(&self) -> [u8; 32] {
        *self.binding.derived()
    }
}

/// The owner's grant to one guest for one session. The serialized field keeps
/// co-op's name, `graph`, so invitations saved before the lift still load.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invitation {
    pub version: u16,
    #[serde(rename = "graph")]
    pub session: Uuid,
    pub owner: Contact,
    pub guest: Contact,
    pub write: bool,
    pub grant: SignedDelegationCertificate,
}

impl Invitation {
    /// Issue a grant from `signer`, the owner, to `guest` for `session`,
    /// valid until `expires` (milliseconds), at most thirty days ahead.
    pub fn issue(
        signer: &dyn Signer,
        terms: &InvitationTerms,
        session: Uuid,
        guest: Contact,
        write: bool,
        now: u64,
        expires: u64,
    ) -> Result<Self, String> {
        guest.check(terms)?;
        let owner = signer.master_public_key().to_bytes();
        if guest.root() == owner {
            return Err("invite requires the owner and a distinct guest".into());
        }
        if expires <= now || expires - now > 30 * 24 * 60 * 60 * 1000 {
            return Err("choose an expiry within thirty days".into());
        }
        let mut nonce = [0; 32];
        rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce);
        let grant = signer.certificate(DelegationCertificate::new(
            DelegationParent::Root(owner),
            owner,
            guest.peer(),
            terms.invitation_scope(owner, session, write),
            now.saturating_sub(30_000),
            now.saturating_sub(30_000),
            Some(expires),
            0,
            nonce,
        ))?;
        let invitation = Self {
            version: 1,
            session,
            owner: Contact::create(signer, terms)?,
            guest,
            write,
            grant,
        };
        invitation.check(terms)?;
        Ok(invitation)
    }

    pub fn check(&self, terms: &InvitationTerms) -> Result<(), String> {
        self.owner.check(terms)?;
        self.guest.check(terms)?;
        let c = &self.grant.certificate;
        self.grant
            .check()
            .map_err(|e| format!("invalid invitation signature: {e:?}"))?;
        if self.version != 1
            || c.issuer != self.owner.root()
            || c.subject != self.guest.peer()
            || c.parent != DelegationParent::Root(self.owner.root())
            || c.scope != terms.invitation_scope(self.owner.root(), self.session, self.write)
            || c.remaining_delegation_depth != 0
        {
            return Err("invitation does not bind this owner, graph and guest".into());
        }
        Ok(())
    }

    /// Whether `peer` is this invitation's guest with live, sufficient access.
    /// The owner keeps revocation; pass whether it has revoked this grant.
    pub fn guest_access(&self, peer: [u8; 32], write: bool, revoked: bool, now: u64)
    -> Result<[u8; 32], String> {
        if self.guest.peer() != peer {
            return Err("this guest is not invited".into());
        }
        if revoked {
            return Err("guest access was revoked".into());
        }
        if self.grant.certificate.expires_at_ms.is_none_or(|end| now > end) {
            return Err("guest access expired".into());
        }
        if write && !self.write {
            return Err("guest has read-only access".into());
        }
        Ok(self.guest.root())
    }

    /// The owner's admission check: the admitted principal is this guest,
    /// connecting to this session's path with exactly this grant.
    pub fn admits(
        &self,
        terms: &InvitationTerms,
        principal: &AdmittedPrincipal,
        chain: &[SignedDelegationCertificate],
        revoked: bool,
        now: u64,
    ) -> Result<(), String> {
        self.guest_access(principal.subject, false, revoked, now)?;
        if principal.action.domain != terms.domain
            || principal.action.path != terms.path(self.session)
            || principal.action.action != "connect"
            || chain.len() != 1
            || chain[0] != self.grant
        {
            return Err("invitation does not authorize this graph route".into());
        }
        Ok(())
    }

    /// The owner's revocation of this grant.
    pub fn revoke(&self, signer: &dyn Signer, now: u64) -> Result<SignedDelegationRevocation, String> {
        if signer.master_public_key().to_bytes() != self.owner.root() {
            return Err("only the owner can end guest access".into());
        }
        let mut nonce = [0; 32];
        rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce);
        signer.revocation(DelegationRevocation::new(
            self.grant.certificate.id(),
            self.owner.root(),
            self.grant.certificate.scope.clone(),
            now,
            nonce,
        ))
    }

    /// The guest's hello for this invitation, signed by its derived transport
    /// key, bound to its transport peer id.
    pub fn hello(
        &self,
        terms: &InvitationTerms,
        identity: &dyn IdentityProvider,
        profile: ProfileRef,
        local_peer: transport::PeerID,
    ) -> Result<SessionHello, String> {
        if identity.master_public_key().to_bytes() != self.guest.root() {
            return Err("contact or invitation belongs to another graph or person".into());
        }
        let key = identity.derive_keypair(terms.salt).map_err(|e| e.to_string())?;
        let signing = personae::InMemoryProvider::from_seed(key.to_seed());
        let mut nonce = [0; 32];
        rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce);
        open_session_for(
            &signing,
            NetworkId(terms.network(self.owner.root(), self.session)),
            profile,
            RequestedAction {
                domain: terms.domain.into(),
                path: terms.path(self.session),
                action: "connect".into(),
            },
            TrafficClass::Interactive,
            nonce,
            &projection_binding(local_peer),
            vec![self.grant.clone()],
        )
        .map_err(|e| e.to_string())
    }
}

/// The owner's contact and the transport ticket a guest dials.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    pub version: u16,
    #[serde(rename = "graph")]
    pub session: Uuid,
    pub owner: Contact,
    pub ticket: String,
}

impl Card {
    pub fn check(&self, terms: &InvitationTerms, invitation: &Invitation) -> Result<(), String> {
        invitation.check(terms)?;
        self.owner.check(terms)?;
        if self.version != 1
            || self.session != invitation.session
            || self.owner.binding != invitation.owner.binding
        {
            return Err("contact or invitation belongs to another graph or person".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use personae::InMemoryProvider;

    const TERMS: InvitationTerms = InvitationTerms {
        domain: "test.invitation",
        salt: b"test/invitation/v1",
        network_label: b"test.invitation/v1",
        path_root: "/services/projection/test",
    };

    #[test]
    fn an_issued_invitation_checks_keeps_its_saved_field_and_refuses_another_salt() {
        let owner = InMemoryProvider::from_seed([1; 32]);
        let guest = InMemoryProvider::from_seed([2; 32]);
        let session = Uuid::from_bytes([7; 16]);
        let invitation = Invitation::issue(&owner, &TERMS,
            session, Contact::create(&guest, &TERMS).unwrap(), false, 1_000_000, 2_000_000).unwrap();
        invitation.check(&TERMS).unwrap();
        let json = serde_json::to_value(&invitation).unwrap();
        assert!(json.get("graph").is_some(), "saved invitations keep co-op's field name");
        let other = InvitationTerms { salt: b"other/v1", ..TERMS };
        assert!(invitation.check(&other).is_err());
        assert!(invitation.guest_access(invitation.guest.peer(), false, false, 1_500_000).is_ok());
        assert!(invitation.guest_access(invitation.guest.peer(), true, false, 1_500_000).is_err());
        assert!(invitation.guest_access(invitation.guest.peer(), false, true, 1_500_000).is_err());
        assert!(invitation.guest_access(invitation.guest.peer(), false, false, 2_000_001).is_err());
        assert!(Invitation::issue(&owner, &TERMS, session,
            Contact::create(&owner, &TERMS).unwrap(), false, 1_000_000, 2_000_000).is_err());
    }
}
