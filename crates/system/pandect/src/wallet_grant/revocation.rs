// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wallet's revocation ledger.
//!
//! `personae::carry` mints the signed statements and `notochord` owns the
//! ledger that folds them; this module is where the wallet keeps its copy on
//! disk. That split is the reconciliation ruling in miniature: the grammar and
//! its evaluator are shared, and the ledger is the application's own.
//!
//! The roster's `revoked` list survives, demoted to what it always should have
//! been. It is a projection of the fold, kept because asking "is this device
//! revoked?" should not mean re-reading every certificate; the statements are
//! the record, and the list is the index.

use std::io;
use std::path::{Path, PathBuf};

use identity::carry::{DeviceGrantSet, revoke_device_grant_set};
use insigne::delegation::SignedDelegationRevocation;
use notochord::RevocationLedger;

use crate::wallet_store::identity_grants_dir;

use super::{
    CarryRef, DeviceGrantError, DeviceId, decode_cbor, encode_cbor, load_device_grant_set,
};

/// `<data_root>/identity/grants/revocations.cbor`
pub fn revocation_ledger_path(data_root: &Path) -> PathBuf {
    identity_grants_dir(data_root).join("revocations.cbor")
}

/// Load the wallet's revocation ledger, empty when none has been written.
pub fn load_revocation_ledger(data_root: &Path) -> io::Result<RevocationLedger> {
    match std::fs::read(revocation_ledger_path(data_root)) {
        Ok(bytes) => decode_cbor(bytes.as_slice())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, DeviceGrantError::Decode)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(RevocationLedger::new()),
        Err(e) => Err(e),
    }
}

/// Persist the revocation ledger, returning its content ref.
pub fn save_revocation_ledger(data_root: &Path, ledger: &RevocationLedger) -> io::Result<CarryRef> {
    let bytes = encode_cbor(ledger)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, DeviceGrantError::Encode))?;
    let path = revocation_ledger_path(data_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &bytes)?;
    Ok(CarryRef::of(bytes.as_slice()))
}

/// Fold checked revocation statements into the wallet's ledger.
///
/// Returns how many were accepted. Each statement is checked first and only
/// a checked one folds, so a rejected count is a real signal rather than a
/// rounding error: statements arriving from a peer are exactly as
/// trustworthy as their signatures.
pub fn fold_revocations(
    data_root: &Path,
    statements: &[SignedDelegationRevocation],
) -> io::Result<usize> {
    let mut ledger = load_revocation_ledger(data_root)?;
    let mut accepted = 0;
    for checked in statements
        .iter()
        .filter_map(|statement| statement.check().ok())
    {
        ledger.fold(checked);
        accepted += 1;
    }
    if accepted > 0 {
        save_revocation_ledger(data_root, &ledger)?;
    }
    Ok(accepted)
}

/// Mint and fold the statements withdrawing one device's whole grant set.
pub fn revoke_device_certificates(
    data_root: &Path,
    master_seed: [u8; 32],
    device_id: DeviceId,
    at_ms: u64,
) -> io::Result<Vec<SignedDelegationRevocation>> {
    let set = load_device_grant_set(data_root, device_id)?;
    if set.is_empty() {
        return Ok(Vec::new());
    }
    let statements = revoke_device_grant_set(master_seed, &set, at_ms)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fold_revocations(data_root, &statements)?;
    Ok(statements)
}

/// Whether the ledger withdraws every certificate a device holds.
///
/// Asks the statements rather than the roster list. Partial revocation is
/// visible here rather than flattened: a device whose persona authority was
/// withdrawn while its transport authority stands is not revoked, and saying
/// otherwise would be the flattening the split exists to remove.
pub fn device_is_fully_revoked(data_root: &Path, device_id: DeviceId) -> io::Result<bool> {
    let set = load_device_grant_set(data_root, device_id)?;
    if set.is_empty() {
        return Ok(false);
    }
    let ledger = load_revocation_ledger(data_root)?;
    Ok(set
        .certificates()
        .all(|certificate| ledger.revokes(&certificate.certificate)))
}

/// Which certificates in a set the ledger has withdrawn.
pub fn revoked_certificate_count(data_root: &Path, set: &DeviceGrantSet) -> io::Result<usize> {
    let ledger = load_revocation_ledger(data_root)?;
    Ok(set
        .certificates()
        .filter(|certificate| ledger.revokes(&certificate.certificate))
        .count())
}
