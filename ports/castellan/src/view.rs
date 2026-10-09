// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Castellan's secret-free read model of the resident Personae host.
//!
//! The view types live in [`dramatis::view`] (dramatis repo plan, ruling
//! D24) and are re-exported here. What stays is [`load_carry_view`], which
//! reads pandect's public carry lane into a [`CarryView`].

use std::io;
use std::path::Path;

use pandect::{
    DeviceExposure, DeviceMode, RecoveryPolicy, load_device_grant_set, load_device_roster,
    load_identity_wallet, load_wrapped_epoch_record, requires_epoch_material,
};

// The view types, at their old paths (dramatis repo plan, ruling D27).
pub use dramatis::view::*;

/// Load the public carry lane. Secret seed and epoch bridge APIs are not used.
pub fn load_carry_view(data_root: &Path) -> io::Result<CarryView> {
    let mut view = CarryView::default();
    let Some(wallet) = load_identity_wallet(data_root)? else {
        view.unavailable
            .push("identity wallet manifest is absent".to_string());
        return Ok(view);
    };

    view.recovery_policy = Some(recovery_policy_label(wallet.recovery_policy).to_string());
    view.personas = wallet
        .personas
        .iter()
        .map(|persona| persona.persona_id.as_uuid().to_string())
        .collect();
    view.personas.sort();

    let roster = load_device_roster(data_root)?;
    if let Some(roster) = roster {
        for device in roster.devices {
            let device_id = device.device_id.as_uuid().to_string();
            let grant_ref = device.grant_ref.map(|reference| reference.to_string());
            view.devices.push(DeviceView {
                device_id: device_id.clone(),
                label: device.label,
                mode: device_mode_label(device.mode).to_string(),
                exposure: exposure_label(device.exposure).to_string(),
                public_key_fingerprint: format!(
                    "blake3:{}",
                    blake3::hash(&device.device_pubkey.0).to_hex()
                ),
                revoked: roster.revoked.contains(&device.device_id),
                grant_ref: grant_ref.clone(),
            });

            // A grant is a set now, so the projection summarises across its
            // members: the union of every certificate's actions, the personas
            // that issued one each, and the earliest expiry, which is the one
            // that actually ends the device's authority.
            let grant = load_device_grant_set(data_root, device.device_id)?;
            match grant.is_empty() {
                false => {
                    // A report of the checks' outcome for display: the view is
                    // serialized, so it carries the verdict, never a conclusion.
                    let signature_valid = Some(
                        grant
                            .certificates()
                            .all(|certificate| certificate.check().is_ok()),
                    );
                    let mut scopes: Vec<String> = grant
                        .certificates()
                        .flat_map(|certificate| certificate.certificate.scope.actions.iter())
                        .map(|action| action.to_string())
                        .collect();
                    scopes.sort();
                    scopes.dedup();
                    let mut wrapped_epoch_count = 0;
                    for certificate in grant.personas.values() {
                        if requires_epoch_material(certificate)
                            && let Some(record) =
                                load_wrapped_epoch_record(data_root, certificate.certificate.id())?
                        {
                            wrapped_epoch_count += record.epochs.len();
                        }
                    }
                    view.grants.push(DeviceGrantView {
                        device_id,
                        grant_ref,
                        signature_valid,
                        issued_at_ms: grant
                            .certificates()
                            .map(|certificate| certificate.certificate.issued_at_ms)
                            .min()
                            .unwrap_or_default(),
                        expires_at_ms: grant
                            .certificates()
                            .filter_map(|certificate| certificate.certificate.expires_at_ms)
                            .min(),
                        personas: grant
                            .personas
                            .keys()
                            .map(|persona| persona.as_uuid().to_string())
                            .collect(),
                        scopes,
                        // `no-subdelegation` is no longer an atom carried in a
                        // list nobody read; depth 0 in the grammar is the fact.
                        attenuations: grant
                            .certificates()
                            .all(|c| c.certificate.remaining_delegation_depth == 0)
                            .then(|| vec!["no-subdelegation".to_string()])
                            .unwrap_or_default(),
                        wrapped_epoch_count,
                    });
                },
                true if device.grant_ref.is_some() => view
                    .unavailable
                    .push(format!("grant bytes unavailable for device {device_id}")),
                true => {},
            }
        }
    } else {
        view.unavailable.push("device roster is absent".to_string());
    }

    view.devices
        .sort_by(|left, right| left.device_id.cmp(&right.device_id));
    view.grants
        .sort_by(|left, right| left.device_id.cmp(&right.device_id));
    Ok(view)
}

fn device_mode_label(mode: DeviceMode) -> &'static str {
    match mode {
        DeviceMode::Copy => "seed copy",
        DeviceMode::RemoteAuth => "delegated",
    }
}

fn exposure_label(exposure: DeviceExposure) -> &'static str {
    match exposure {
        DeviceExposure::HiddenClient => "hidden client",
        DeviceExposure::ExposedEgress => "exposed egress",
    }
}

fn recovery_policy_label(policy: RecoveryPolicy) -> &'static str {
    match policy {
        RecoveryPolicy::SeedAndDeviceHandover => "seed and device handover",
        RecoveryPolicy::SeedPhraseOnly => "seed phrase only",
    }
}

#[cfg(test)]
mod tests {
    use pandect::{
        DeviceId, DevicePublicKey, DeviceRecord, DeviceRoster, IdentityWalletManifest, PersonaId,
        PersonaWalletRef, save_device_roster, save_identity_wallet,
    };

    use super::*;

    #[test]
    fn carry_adapter_projects_public_roster_facts_without_secret_bridges() {
        let root = std::env::temp_dir().join(format!(
            "graphshell-h4-carry-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let device_id = DeviceId::new();
        let mut wallet = IdentityWalletManifest::default();
        wallet.personas.push(PersonaWalletRef {
            persona_id: PersonaId::default_persona(),
        });
        let roster = DeviceRoster {
            schema_version: pandect::WALLET_SCHEMA_VERSION,
            devices: vec![DeviceRecord {
                device_id,
                device_pubkey: DevicePublicKey([0x33; 32]),
                label: "Travel laptop".to_string(),
                mode: DeviceMode::RemoteAuth,
                exposure: DeviceExposure::HiddenClient,
                carriage: pandect::CarriagePolicy::default(),
                grant_ref: None,
            }],
            revoked: vec![device_id],
        };
        save_identity_wallet(&root, &wallet).unwrap();
        save_device_roster(&root, &roster).unwrap();

        let projected = load_carry_view(&root).unwrap();
        assert_eq!(projected.personas.len(), 1);
        assert_eq!(projected.devices.len(), 1);
        assert_eq!(projected.devices[0].label, "Travel laptop");
        assert!(projected.devices[0].revoked);
        let json = serde_json::to_string(&projected).unwrap();
        assert!(!json.contains("device_seed"));
        assert!(!json.contains("epoch_secret"));
        assert!(!json.contains("private_epoch"));

        std::fs::remove_dir_all(root).unwrap();
    }
}
