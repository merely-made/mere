// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retiring grants written before the certificate format.
//!
//! The migration posture is re-issue, with no legacy decoder. That choice has
//! a consequence worth stating plainly: **the migration cannot be automatic.**
//! What a grant permitted lived in the signed payload, and refusing to decode
//! the payload means refusing to learn the scopes. A device's authority has to
//! be restated by whoever is re-commissioning it.
//!
//! Not everything is lost with the payload, though, and this module recovers
//! what survived elsewhere in the wallet so the restatement is a confirmation
//! rather than an act of memory:
//!
//! - the roster keeps each device's label, mode, exposure, and public key;
//! - each persona wallet keeps a capability slot named for the device, so the
//!   persona set is recoverable without reading a single grant byte.
//!
//! Legacy grants live at `identity/grants/<device_id>.cbor` and certificates
//! at `identity/grants/<device_id>/`, so the two never collide and a
//! half-migrated wallet is always legible.
//!
//! Re-issuing signs with the wallet's seed, so it is custody, in castellan
//! (dramatis repo plan, DR-B, ruling D8); the survey and retirement stay.

use std::io;
use std::path::Path;

use identity::PersonaId;

use crate::wallet_store::{
    device_grant_path, load_device_roster, load_identity_wallet, load_persona_wallet,
};

use super::{
    DeviceExposure, DeviceId, DeviceMode, DevicePublicKey, load_device_grant_set,
    remote_auth_capability_slot_id,
};

/// One device still carrying a grant written before the certificate format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyGrant {
    /// The device the stale grant file belongs to.
    pub device_id: DeviceId,
    /// Operator-facing label from the roster.
    pub label: String,
    /// How the device was enrolled.
    pub mode: DeviceMode,
    /// Whether the device serves as an egress anchor.
    pub exposure: DeviceExposure,
    /// The device's public key, still known from the roster.
    pub holder: DevicePublicKey,
    /// Personas recovered from their own wallets' capability slots.
    ///
    /// Recovered rather than decoded: each persona that granted this device
    /// holds a slot named for it, so the persona set survives the payload.
    pub personas: Vec<PersonaId>,
    /// Whether a certificate set has already been issued for this device.
    ///
    /// True means the stale file is residue and [`retire_legacy_grant`] can
    /// remove it; false means the device has no working authority at all.
    pub reissued: bool,
}

impl LegacyGrant {
    /// Whether this device currently has no usable authority.
    ///
    /// A device in this state fails closed everywhere: `load_device_grant_set`
    /// returns an empty set, and enrol, refresh, and revoke all refuse it.
    pub fn is_stranded(&self) -> bool {
        !self.reissued
    }
}

/// Find every device whose grant predates the certificate format.
///
/// Reads the roster and the persona wallets. It never opens a legacy grant
/// file, only observes that one exists, which is what keeps the no-decoder
/// posture honest.
pub fn survey_legacy_grants(data_root: &Path) -> io::Result<Vec<LegacyGrant>> {
    let Some(roster) = load_device_roster(data_root)? else {
        return Ok(Vec::new());
    };
    let known_personas: Vec<PersonaId> = load_identity_wallet(data_root)?
        .map(|wallet| wallet.personas.iter().map(|p| p.persona_id).collect())
        .unwrap_or_default();

    let mut found = Vec::new();
    for device in &roster.devices {
        if !device_grant_path(data_root, device.device_id).exists() {
            continue;
        }
        let slot_id = remote_auth_capability_slot_id(device.device_id);
        let mut personas = Vec::new();
        for &persona in &known_personas {
            let Some(wallet) = load_persona_wallet(data_root, persona)? else {
                continue;
            };
            if wallet
                .capability_slots
                .iter()
                .any(|slot| slot.slot_id == slot_id)
            {
                personas.push(persona);
            }
        }
        let reissued = !load_device_grant_set(data_root, device.device_id)?.is_empty();
        found.push(LegacyGrant {
            device_id: device.device_id,
            label: device.label.clone(),
            mode: device.mode,
            exposure: device.exposure,
            holder: device.device_pubkey,
            personas,
            reissued,
        });
    }
    Ok(found)
}

/// Why a device has no certificates, when the answer is the migration.
///
/// A stranded device otherwise reports only that its certificates are absent,
/// which reads like corruption rather than like a pending re-commissioning.
/// This turns that into an attributable message at the two seams that refuse.
/// The error text for a device whose only grant predates certificates.
pub fn legacy_grant_hint(data_root: &Path, device_id: DeviceId) -> String {
    if device_grant_path(data_root, device_id).exists() {
        format!(
            "device {} still carries a grant written before the certificate format;              re-issue it (see survey_legacy_grants) rather than treating this as corruption",
            device_id.as_uuid()
        )
    } else {
        format!(
            "device grant certificates missing for {}",
            device_id.as_uuid()
        )
    }
}

/// Delete the stale grant file for a device that now has certificates.
///
/// Refuses while the device is stranded, so the only record that a device ever
/// had authority cannot be removed before that authority is restored.
pub fn retire_legacy_grant(data_root: &Path, device_id: DeviceId) -> io::Result<bool> {
    let path = device_grant_path(data_root, device_id);
    if !path.exists() {
        return Ok(false);
    }
    if load_device_grant_set(data_root, device_id)?.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "refusing to retire the legacy grant for device {}: no certificates \
                 have been issued for it yet",
                device_id.as_uuid()
            ),
        ));
    }
    std::fs::remove_file(&path)?;
    Ok(true)
}
