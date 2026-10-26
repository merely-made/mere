// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A delegation signature's purpose and audience, checked against the bytes
//! actually signed (§5.6). The H1 signing consumer is Graphshell's guest
//! invitation and its withdrawal: insigne delegation certificates and
//! revocations in the `mere.graphshell` domain, signed by that resource's
//! scope key. The custodian decodes the canonical statement, rebuilds it
//! with insigne and requires byte equality, then checks that its domain is
//! the granted audience, its issuer is the open profile's master and the
//! salt is exactly that statement's scope key. Anything else is not a
//! delegation and is refused, so a delegation grant is no signing oracle.

use insigne::delegation::{
    CapabilityScope, DelegationCertificate, DelegationId, DelegationParent, DelegationRevocation,
    delegation_signing_salt,
};

use crate::Refusal;

const CERTIFICATE_DOMAIN: &[u8] = b"personae/delegation-certificate/v1";
const REVOCATION_DOMAIN: &[u8] = b"personae/delegation-revocation/v1";
const MAX_STATEMENT_BYTES: usize = 64 * 1024;

/// Check that `message` is a well-formed delegation statement in
/// `audience`, issued by `master`, whose scope key is exactly `salt`.
pub fn check(audience: &str, master: &[u8; 32], salt: &[u8], message: &[u8]) -> Result<(), Refusal> {
    if message.len() > MAX_STATEMENT_BYTES {
        return Err(Refusal("statement-too-large"));
    }
    let mut reader = Reader(message);
    let domain = reader.bytes().map_err(|_| Refusal("not-a-delegation"))?;
    let (scope, issuer, canonical) = if domain == CERTIFICATE_DOMAIN {
        let certificate = certificate(&mut reader)?;
        if !certificate.is_well_formed() {
            return Err(Refusal("statement-malformed"));
        }
        (certificate.scope.clone(), certificate.issuer, certificate.signing_bytes())
    } else if domain == REVOCATION_DOMAIN {
        let revocation = revocation(&mut reader)?;
        if !revocation.is_well_formed() {
            return Err(Refusal("statement-malformed"));
        }
        (revocation.scope.clone(), revocation.issuer, revocation.signing_bytes())
    } else {
        return Err(Refusal("not-a-delegation"));
    };
    if !reader.0.is_empty() || canonical != message {
        return Err(Refusal("statement-not-canonical"));
    }
    if scope.domain != audience {
        return Err(Refusal("audience-denied"));
    }
    if &issuer != master {
        return Err(Refusal("issuer-not-profile"));
    }
    if delegation_signing_salt(&scope) != salt {
        return Err(Refusal("salt-not-statement-scope"));
    }
    Ok(())
}

fn certificate(reader: &mut Reader<'_>) -> Result<DelegationCertificate, Refusal> {
    if reader.u16()? != 1 {
        return Err(Refusal("statement-version"));
    }
    let parent = match reader.u8()? {
        0 => DelegationParent::Root(reader.array()?),
        1 => DelegationParent::Certificate(DelegationId(reader.array()?)),
        _ => return Err(Refusal("statement-malformed")),
    };
    let issuer = reader.array()?;
    let subject = reader.array()?;
    let scope = scope(reader)?;
    let issued_at = reader.u64()?;
    let not_before = reader.u64()?;
    let expires = match reader.u8()? {
        0 => None,
        1 => Some(reader.u64()?),
        _ => return Err(Refusal("statement-malformed")),
    };
    let depth = reader.u16()?;
    let nonce = reader.array()?;
    Ok(DelegationCertificate::new(
        parent, issuer, subject, scope, issued_at, not_before, expires, depth, nonce,
    ))
}

fn revocation(reader: &mut Reader<'_>) -> Result<DelegationRevocation, Refusal> {
    if reader.u16()? != 1 {
        return Err(Refusal("statement-version"));
    }
    let certificate = DelegationId(reader.array()?);
    let issuer = reader.array()?;
    let scope = scope(reader)?;
    let at = reader.u64()?;
    let nonce = reader.array()?;
    Ok(DelegationRevocation::new(certificate, issuer, scope, at, nonce))
}

fn scope(reader: &mut Reader<'_>) -> Result<CapabilityScope, Refusal> {
    let domain = reader.string()?;
    let resource = reader.bytes()?.to_vec();
    let path_prefix = reader.string()?;
    let count = reader.u64()?;
    if count > 256 {
        return Err(Refusal("statement-malformed"));
    }
    let mut actions = std::collections::BTreeSet::new();
    for _ in 0..count {
        actions.insert(reader.string()?);
    }
    Ok(CapabilityScope {
        domain,
        resource,
        path_prefix,
        actions,
    })
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Refusal> {
        if self.0.len() < n {
            return Err(Refusal("statement-truncated"));
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }

    fn u8(&mut self) -> Result<u8, Refusal> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Refusal> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().expect("two bytes")))
    }

    fn u64(&mut self) -> Result<u64, Refusal> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("eight bytes")))
    }

    fn array(&mut self) -> Result<[u8; 32], Refusal> {
        Ok(self.take(32)?.try_into().expect("thirty-two bytes"))
    }

    fn bytes(&mut self) -> Result<&'a [u8], Refusal> {
        let len = usize::try_from(self.u64()?).map_err(|_| Refusal("statement-malformed"))?;
        self.take(len)
    }

    fn string(&mut self) -> Result<String, Refusal> {
        String::from_utf8(self.bytes()?.to_vec()).map_err(|_| Refusal("statement-malformed"))
    }
}
