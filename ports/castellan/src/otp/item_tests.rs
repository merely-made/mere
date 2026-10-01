// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use base64::Engine as _;
use tempfile::tempdir;

use super::*;
use crate::otp::{OtpReleaseGate, OtpReleaseParticipantClaim};

const RFC6238_SHA1_SECRET: &[u8] = b"12345678901234567890";
const RFC6238_SHA1_SECRET_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
const STEAM_SHARED_SECRET: &str = "zvIayp3JPvtvX/QGHqsqKBk/44s=";

fn store(root: &std::path::Path, persona: PersonaId) -> OtpItemStore {
    OtpItemStore::new(
        SealedRecordStorage::open_with_key(root, [0x71; 32]),
        persona,
    )
}

fn provision_uri() -> String {
    format!("otpauth://totp/Merely:mark?secret={RFC6238_SHA1_SECRET_BASE32}&issuer=Merely&digits=8")
}

#[test]
fn imported_item_reopens_with_its_secret_free_metadata() {
    let dir = tempdir().unwrap();
    let persona = PersonaId::new();
    let items = store(dir.path(), persona);
    let item = items.import_otpauth_uri(&provision_uri()).unwrap();

    assert_eq!(item.account(), "mark");
    assert_eq!(item.issuer(), Some("Merely"));
    assert_eq!(item.code_style(), OtpCodeStyle::Decimal { digits: 8 });

    let reopened = store(dir.path(), persona);
    assert_eq!(
        reopened.get(item.item_id(), item.credential_id()).unwrap(),
        Some(item.clone())
    );
    assert_eq!(
        reopened
            .seconds_remaining_at(item.item_id(), item.credential_id(), 59)
            .unwrap(),
        Some(1)
    );
}

#[test]
fn stored_record_contains_neither_seed_nor_display_metadata_in_plaintext() {
    let dir = tempdir().unwrap();
    let items = store(dir.path(), PersonaId::new());
    let item = items.import_otpauth_uri(&provision_uri()).unwrap();

    // The split layout writes three records; none is plaintext.
    for path in [
        items.items().payload_path(item.credential_id()),
        items.items().item_path(item.item_id()),
        items.items().index_path(),
    ] {
        let bytes = std::fs::read(dir.path().join(path)).unwrap();
        let disk = String::from_utf8(bytes).unwrap();
        assert!(!disk.contains(RFC6238_SHA1_SECRET_BASE32));
        assert!(!disk.contains("Merely"));
        assert!(!disk.contains("\"account\":\"mark\""));
    }
}

#[test]
fn an_item_handle_has_no_meaning_in_another_persona_namespace() {
    let dir = tempdir().unwrap();
    let owner = store(dir.path(), PersonaId::new());
    let item = owner.import_otpauth_uri(&provision_uri()).unwrap();
    let other = store(dir.path(), PersonaId::new());

    assert_eq!(
        other.get(item.item_id(), item.credential_id()).unwrap(),
        None
    );
    assert!(matches!(
        other.seconds_remaining_at(item.item_id(), item.credential_id(), 59),
        Err(OtpItemError::NotFound { item: id, .. }) if id == item.item_id()
    ));
}

#[test]
fn deletion_closes_the_item_without_affecting_the_persona_namespace() {
    let dir = tempdir().unwrap();
    let items = store(dir.path(), PersonaId::new());
    let item = items.import_otpauth_uri(&provision_uri()).unwrap();

    items.delete(item.item_id()).unwrap();

    assert_eq!(
        items.get(item.item_id(), item.credential_id()).unwrap(),
        None
    );
}

#[test]
fn steam_guard_is_an_explicit_stored_style_and_uses_the_release_tile() {
    let dir = tempdir().unwrap();
    let items = store(dir.path(), PersonaId::new());
    let item = items
        .import_steam_guard("mark", STEAM_SHARED_SECRET)
        .unwrap();

    assert_eq!(item.issuer(), Some("Steam"));
    assert_eq!(item.code_style(), OtpCodeStyle::SteamGuard);
    assert_eq!(item.code_style().character_count(), 5);
    assert_eq!(item.mode(), OtpMode::Totp { period: 30, t0: 0 });
    let tile = items
        .release_tile_at_unix_time(item.item_id(), item.credential_id(), 1_616_374_841)
        .unwrap();
    assert_eq!(tile.code_at_unix_time(1_616_374_841), Some("2F9J5"));
    assert_eq!(tile.code_at_unix_time(1_616_374_860), None);
}

#[test]
fn steam_guard_import_does_not_reclassify_otpauth_extensions() {
    let dir = tempdir().unwrap();
    let items = store(dir.path(), PersonaId::new());
    let item = items
        .import_otpauth_uri(&format!(
            "otpauth://totp/Steam:mark?secret={RFC6238_SHA1_SECRET_BASE32}&issuer=Steam&encoder=steam"
        ))
        .unwrap();

    assert_eq!(item.code_style(), OtpCodeStyle::Decimal { digits: 6 });
}

#[test]
fn an_imported_item_is_titled_by_its_issuer_or_else_its_account() {
    let dir = tempdir().unwrap();
    let items = store(dir.path(), PersonaId::new());
    let issued = items.import_otpauth_uri(&provision_uri()).unwrap();
    let bare = items
        .import_otpauth_uri(&format!(
            "otpauth://totp/alice?secret={RFC6238_SHA1_SECRET_BASE32}"
        ))
        .unwrap();
    let steam = items
        .import_steam_guard("mark", STEAM_SHARED_SECRET)
        .unwrap();

    assert_eq!(issued.item().title, "Merely");
    assert_eq!(issued.item().subtitle.as_deref(), Some("mark"));
    assert_eq!(bare.item().title, "alice");
    assert_eq!(bare.item().subtitle, None);
    assert_eq!(steam.item().title, "Steam");
    assert_eq!(steam.item().subtitle.as_deref(), Some("mark"));
    assert_eq!(issued.item().credentials.len(), 1);
    assert_eq!(issued.item().state, ItemState::Vault);
}

/// Every chatelaine value castellan hands out (import results, reads, the
/// index listing, petitions and tiles) is free of the seeds it describes,
/// in its serialized form and in its debug form, and HOTP counters stay
/// sealed with them.
#[test]
fn no_chatelaine_value_in_the_read_model_holds_secret_material() {
    let dir = tempdir().unwrap();
    let items = store(dir.path(), PersonaId::new());
    let totp = items.import_otpauth_uri(&provision_uri()).unwrap();
    let hotp = items
        .import_otpauth_uri(&format!(
            "otpauth://hotp/Merely:mark?secret={RFC6238_SHA1_SECRET_BASE32}&issuer=Merely&counter=41"
        ))
        .unwrap();
    let steam = items
        .import_steam_guard("mark", STEAM_SHARED_SECRET)
        .unwrap();
    let gate = OtpReleaseGate::new(items.clone());
    let participant = OtpReleaseParticipantClaim::unverified("local:test", "read:test").unwrap();
    let request = gate
        .petition(hotp.item_id(), hotp.credential_id(), participant)
        .unwrap();
    let released = gate.approve(request.id).unwrap();

    let steam_secret = base64::engine::general_purpose::STANDARD
        .decode(STEAM_SHARED_SECRET)
        .unwrap();
    let forbidden = [
        String::from_utf8(RFC6238_SHA1_SECRET.to_vec()).unwrap(),
        RFC6238_SHA1_SECRET_BASE32.to_string(),
        serde_json::to_string(RFC6238_SHA1_SECRET).unwrap(),
        STEAM_SHARED_SECRET.to_string(),
        serde_json::to_string(&steam_secret).unwrap(),
        "counter".to_string(),
    ];
    let mut shown: Vec<Item> = vec![
        totp.item().clone(),
        hotp.item().clone(),
        steam.item().clone(),
        request.credential.item().clone(),
        released.tile().credential().item().clone(),
    ];
    shown.extend(items.items().list().unwrap());
    shown.extend(items.items().get(hotp.item_id()).unwrap());
    assert_eq!(shown.len(), 9);
    for item in &shown {
        for rendered in [serde_json::to_string(item).unwrap(), format!("{item:?}")] {
            for secret in &forbidden {
                assert!(
                    !rendered.contains(secret.as_str()),
                    "a read-model value carries {secret:?}"
                );
            }
        }
    }
    // The HOTP mode is a bare tag: the counter (41, then 42) has no field.
    assert!(
        serde_json::to_string(hotp.item())
            .unwrap()
            .contains("\"mode\":\"hotp\"")
    );
}
