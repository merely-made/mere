// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where wallet state lives under a data root: the filename constants'
//! composition rules, plus the persona-directory membership test.

use std::path::{Path, PathBuf};
use std::{fs, io};

use identity::PersonaId;
use uuid::Uuid;

use crate::engine_profile_store::PERSONAS_DIR;

use super::{
    DEVICE_ROSTER_FILENAME, DeviceId, IDENTITY_AUTO_UNLOCK_ROOT_FILENAME, IDENTITY_DIR,
    IDENTITY_GRANTS_DIR, IDENTITY_SEED_FILENAME, IDENTITY_WALLET_FILENAME,
    LOCAL_DEVICE_IDENTITY_FILENAME, PERSONA_EPOCH_BRIDGE_FILENAME, PERSONA_WALLET_FILENAME,
    REMOTE_AUTH_WRAPPING_KEYS_FILENAME,
};

/// `<data_root>/identity/`
pub fn identity_dir(data_root: &Path) -> PathBuf {
    data_root.join(IDENTITY_DIR)
}

/// `<data_root>/identity/wallet.json`
pub fn identity_wallet_path(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(IDENTITY_WALLET_FILENAME)
}

/// `<data_root>/identity/device-roster.json`
pub fn device_roster_path(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(DEVICE_ROSTER_FILENAME)
}

/// `<data_root>/identity/grants/`
pub fn identity_grants_dir(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(IDENTITY_GRANTS_DIR)
}

/// `<data_root>/identity/master.seed`
pub fn identity_seed_path(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(IDENTITY_SEED_FILENAME)
}

/// `<data_root>/identity/local-device.json`
pub fn local_device_identity_path(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(LOCAL_DEVICE_IDENTITY_FILENAME)
}

/// `<data_root>/identity/remote-auth-wrapping-keys.json`
pub fn remote_auth_wrapping_keys_path(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(REMOTE_AUTH_WRAPPING_KEYS_FILENAME)
}

/// `<data_root>/identity/vault-root.auto.json`
pub fn identity_auto_unlock_root_path(data_root: &Path) -> PathBuf {
    identity_dir(data_root).join(IDENTITY_AUTO_UNLOCK_ROOT_FILENAME)
}

/// `<data_root>/identity/grants/<device_id>.cbor`
pub fn device_grant_path(data_root: &Path, device_id: DeviceId) -> PathBuf {
    identity_grants_dir(data_root).join(format!("{}.cbor", device_id.as_uuid()))
}

/// `<data_root>/personas/<persona_id>/wallet.json`
pub fn persona_wallet_path(data_root: &Path, persona: PersonaId) -> PathBuf {
    data_root
        .join(PERSONAS_DIR)
        .join(persona.as_uuid().to_string())
        .join(PERSONA_WALLET_FILENAME)
}

/// The personas that actually exist under `data_root`: directories named by a
/// persona UUID that hold a wallet manifest.
///
/// The wallet file is the membership test, deliberately. `personas/<uuid>/`
/// also hosts engine profiles ([`crate::engine_profile_store`]), so a bare
/// directory does not mean a persona lives there — and a caller resolving
/// "which persona am I?" against directories alone would happily pick an
/// engine-profile shell with no keys in it.
///
/// Sorted by UUID so the answer is stable across runs. This is the wallet
/// lane's half of the family "which persona?" question — the vault lane's is
/// `personae::roster` — and the resolution ladder for this lane grows here
/// when something exists to write a remembered choice.
pub fn list_personas(data_root: &Path) -> io::Result<Vec<PersonaId>> {
    let dir = data_root.join(PERSONAS_DIR);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut personas = Vec::new();
    for entry in entries {
        let entry = entry?;
        let Ok(uuid) = entry.file_name().to_string_lossy().parse::<Uuid>() else {
            continue;
        };
        let persona = PersonaId::from_uuid(uuid);
        if persona_wallet_path(data_root, persona).is_file() {
            personas.push(persona);
        }
    }
    personas.sort_by_key(|persona| *persona.as_uuid());
    Ok(personas)
}

/// Why no single wallet persona could be chosen under a data root.
#[derive(Debug)]
pub enum PersonaResolutionError {
    /// The personas directory could not be read.
    List { root: PathBuf, error: io::Error },
    /// No persona wallet exists yet.
    Missing { root: PathBuf },
    /// Several do, and nothing says which.
    Several {
        root: PathBuf,
        personas: Vec<PersonaId>,
    },
}

impl std::fmt::Display for PersonaResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List { root, error } => {
                write!(
                    f,
                    "could not list personas under {}: {error}",
                    root.display()
                )
            },
            Self::Missing { root } => write!(
                f,
                "no persona wallet exists under {} yet; pair this device first, or name a persona",
                root.display()
            ),
            Self::Several { root, personas } => write!(
                f,
                "several personas live under {}; name one of: {}",
                root.display(),
                personas
                    .iter()
                    .map(|persona| persona.as_uuid().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl std::error::Error for PersonaResolutionError {}

/// The wallet persona to act for under `data_root`: `explicit` when given,
/// otherwise the sole persona wallet. Zero or several wallets are refused with
/// what exists, because guessing among real cryptographic identities is how a
/// document gets sealed to somebody else (family shared identity plan,
/// 2026-08-09). A remembered choice joins this ladder when something exists to
/// write one.
pub fn resolve_persona(
    data_root: &Path,
    explicit: Option<PersonaId>,
) -> Result<PersonaId, PersonaResolutionError> {
    if let Some(persona) = explicit {
        return Ok(persona);
    }
    let personas = list_personas(data_root).map_err(|error| PersonaResolutionError::List {
        root: data_root.to_path_buf(),
        error,
    })?;
    match personas.as_slice() {
        [only] => Ok(*only),
        [] => Err(PersonaResolutionError::Missing {
            root: data_root.to_path_buf(),
        }),
        _ => Err(PersonaResolutionError::Several {
            root: data_root.to_path_buf(),
            personas,
        }),
    }
}

/// `<data_root>/personas/<persona_id>/private-epoch-bridge.json`
pub fn persona_epoch_bridge_path(data_root: &Path, persona: PersonaId) -> PathBuf {
    data_root
        .join(PERSONAS_DIR)
        .join(persona.as_uuid().to_string())
        .join(PERSONA_EPOCH_BRIDGE_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::super::*;
    use super::*;

    #[test]
    fn listing_personas_counts_wallets_not_directories() {
        let root = temp_data_root("list-personas");
        assert_eq!(list_personas(&root).unwrap(), Vec::new(), "no dir yet");

        // An engine-profile shell: a persona-shaped directory with no wallet.
        // This is the case the wallet-file membership test exists for.
        let shell = PersonaId::from_uuid(Uuid::from_u128(0xEE));
        fs::create_dir_all(
            root.join(PERSONAS_DIR)
                .join(shell.as_uuid().to_string())
                .join("engine-profiles"),
        )
        .unwrap();
        // Something that is not a persona at all.
        fs::create_dir_all(root.join(PERSONAS_DIR).join("not-a-uuid")).unwrap();

        let real = PersonaId::from_uuid(Uuid::from_u128(0x22));
        let later = PersonaId::from_uuid(Uuid::from_u128(0x11));
        for persona in [real, later] {
            save_persona_wallet(
                &root,
                &PersonaWalletManifest::new(persona, fixture_chain_root(), fixture_epoch()),
            )
            .unwrap();
        }

        assert_eq!(
            list_personas(&root).unwrap(),
            vec![later, real],
            "wallets only, sorted by UUID so the answer is stable"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolving_a_persona_takes_the_sole_wallet_or_says_what_exists() {
        let root = temp_data_root("resolve-persona");
        assert!(matches!(
            resolve_persona(&root, None),
            Err(PersonaResolutionError::Missing { .. })
        ));

        let first = PersonaId::from_uuid(Uuid::from_u128(0x31));
        save_persona_wallet(
            &root,
            &PersonaWalletManifest::new(first, fixture_chain_root(), fixture_epoch()),
        )
        .unwrap();
        assert_eq!(resolve_persona(&root, None).unwrap(), first);

        let second = PersonaId::from_uuid(Uuid::from_u128(0x32));
        save_persona_wallet(
            &root,
            &PersonaWalletManifest::new(second, fixture_chain_root(), fixture_epoch()),
        )
        .unwrap();
        match resolve_persona(&root, None) {
            Err(PersonaResolutionError::Several { personas, .. }) => {
                assert_eq!(personas, vec![first, second]);
            },
            other => panic!("several wallets must be refused with both named: {other:?}"),
        }
        assert_eq!(
            resolve_persona(&root, Some(second)).unwrap(),
            second,
            "an explicit persona wins"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn identity_paths_compose_under_identity_root() {
        let root = Path::new("/data");
        assert_eq!(identity_dir(root), Path::new("/data").join("identity"));
        assert_eq!(
            identity_wallet_path(root),
            Path::new("/data").join("identity").join("wallet.json")
        );
        assert_eq!(
            identity_seed_path(root),
            Path::new("/data").join("identity").join("master.seed")
        );
        assert_eq!(
            local_device_identity_path(root),
            Path::new("/data")
                .join("identity")
                .join("local-device.json")
        );
        assert_eq!(
            device_roster_path(root),
            Path::new("/data")
                .join("identity")
                .join("device-roster.json")
        );
        assert_eq!(
            device_grant_path(root, fixture_device()),
            Path::new("/data")
                .join("identity")
                .join("grants")
                .join(format!("{}.cbor", fixture_device().as_uuid()))
        );
    }

    #[test]
    fn persona_wallet_path_composes_under_persona_root() {
        let path = persona_wallet_path(Path::new("/data"), fixture_persona());
        let expected = Path::new("/data")
            .join("personas")
            .join(fixture_persona().as_uuid().to_string())
            .join("wallet.json");
        assert_eq!(path, expected);
    }
}
