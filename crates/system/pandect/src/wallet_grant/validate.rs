// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pre-flight checks: every grant spec and enrollment bundle is validated
//! against wallet state before anything is written.

use std::io;
use std::path::Path;

use crate::wallet_store::*;

use super::*;

/// Check a remote-auth grant spec against the wallet before issuing.
pub fn validate_remote_auth_spec(data_root: &Path, spec: &RemoteAuthGrantSpec) -> io::Result<()> {
    for &persona in &spec.personas {
        if load_persona_wallet(data_root, persona)?.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("persona wallet missing for {}", persona.as_uuid()),
            ));
        }
    }
    for wrapped in &spec.wrapped_private_epochs {
        if !spec.personas.contains(&wrapped.persona_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "wrapped epoch persona {} is not authorized by this grant",
                    "<blinded>"
                ),
            ));
        }
    }
    if spec.scopes.iter().any(|scope| scope == "private.read")
        && spec.wrapped_private_epochs.is_empty()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "remote-auth grant with private.read must carry wrapped private epoch material",
        ));
    }
    Ok(())
}

/// Check a paired remote-auth grant spec against the wallet before issuing.
pub fn validate_paired_remote_auth_spec(
    data_root: &Path,
    spec: &PairedRemoteAuthGrantSpec,
) -> io::Result<()> {
    for &persona in &spec.personas {
        if load_persona_wallet(data_root, persona)?.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("persona wallet missing for {}", persona.as_uuid()),
            ));
        }
    }
    for epoch in &spec.private_epochs {
        if !spec.personas.contains(&epoch.persona_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "plaintext private epoch persona {} is not authorized by this grant",
                    epoch.persona_id.as_uuid()
                ),
            ));
        }
    }
    if spec.scopes.iter().any(|scope| scope == "private.read") && spec.private_epochs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "remote-auth pairing grant with private.read must carry plaintext private epochs",
        ));
    }
    Ok(())
}

/// Check an enrollment bundle against this device's delegated identity
/// `local`, which the caller loads (castellan holds it since DR-B): every
/// certificate checks, addresses this device and names its key. `None` is
/// refused as a missing identity, after the bundle's own checks.
pub fn validate_remote_auth_enrollment_bundle(
    bundle: &RemoteAuthEnrollmentBundle,
    local: Option<&LocalDeviceIdentity>,
) -> io::Result<()> {
    if bundle.grant.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "remote-auth enrollment bundle carries no grant certificates",
        ));
    }
    check_grant_set(&bundle.grant, "remote-auth enrollment bundle grant")?;

    let local = local.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "local delegated-device identity missing; generate a pairing response first",
        )
    })?;
    // Every certificate in the set must address this device and name this
    // holder. The old envelope carried one device id and one delegatee for the
    // whole grant; a set has to be checked member by member, or one stray
    // certificate would ride in on the others' validity.
    for certificate in bundle.grant.certificates() {
        match certificate_device_id(certificate) {
            Some(device) if device == local.device_id => {},
            Some(device) => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "grant targets device {}, but local delegated identity is {}",
                        device.as_uuid(),
                        local.device_id.as_uuid()
                    ),
                ));
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "grant certificate does not address a device",
                ));
            },
        }
        if certificate.certificate.subject != local.public_key().0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "grant subject does not match the local delegated-device identity",
            ));
        }
    }
    let earliest_expiry = bundle
        .grant
        .certificates()
        .filter_map(|certificate| certificate.certificate.expires_at_ms)
        .min();
    if is_expired(earliest_expiry, unix_time_ms()?) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "remote-auth enrollment grant for device {} is expired",
                local.device_id.as_uuid()
            ),
        ));
    }

    let grant_personas: BTreeSet<_> = bundle
        .grant
        .personas
        .keys()
        .map(|persona| *persona.as_uuid())
        .collect();
    let bundled_personas: BTreeSet<_> = bundle
        .persona_wallets
        .iter()
        .map(|wallet| *wallet.persona_id.as_uuid())
        .collect();
    if grant_personas != bundled_personas {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "enrollment bundle persona wallets do not match the grant persona set",
        ));
    }
    Ok(())
}
