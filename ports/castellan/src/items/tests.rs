// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use tempfile::tempdir;

use super::*;
use crate::otp::{
    OtpItemError, OtpItemStore, OtpReleaseError, OtpReleaseGate, OtpReleaseParticipantClaim,
};
use crate::resident::CastellanResident;

const KEY: [u8; 32] = [0x5a; 32];
const RFC6238_SHA1_SECRET_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

fn uri() -> String {
    format!("otpauth://totp/Merely:mark?secret={RFC6238_SHA1_SECRET_BASE32}&issuer=Merely&digits=8")
}

fn participant() -> OtpReleaseParticipantClaim {
    OtpReleaseParticipantClaim::unverified("local:test", "items:test").unwrap()
}

fn claim(root: &Path) -> CastellanResident {
    CastellanResident::claim(
        root.join("records"),
        [0x61; 32],
        root.join("freshness"),
        [0x62; 32],
    )
    .unwrap()
}

/// Copy one record, unchanged, to another persona's path: a mis-filing.
fn refile<T: Serialize + DeserializeOwned>(storage: &SealedRecordStorage, from: &str, to: &str) {
    let record: T = storage.load_record(from).unwrap().unwrap();
    storage.save_record(to, &record).unwrap();
}

fn json_records(directory: &Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(directory)
        .map(|entries| {
            entries
                .map(|entry| entry.unwrap().path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "json")
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_store_refuses_records_filed_under_another_persona_by_name() {
    let dir = tempdir().unwrap();
    let storage = SealedRecordStorage::open_with_key(dir.path(), KEY);
    let (work, burner) = (PersonaId::new(), PersonaId::new());
    let burner_items = ItemStore::new(storage.clone(), burner);
    let otp = OtpItemStore::over(burner_items.clone())
        .import_otpauth_uri(&uri())
        .unwrap();
    let (item, credential) = (otp.item_id(), otp.credential_id());
    let work_items = ItemStore::new(storage.clone(), work);
    let named = |error: ItemStoreError| {
        let text = error.to_string();
        assert!(
            matches!(
                error,
                ItemStoreError::PersonaMismatch { expected, found }
                    if expected == work && found == burner
            ),
            "{text}"
        );
        assert!(text.contains(&work.as_uuid().to_string()), "{text}");
        assert!(text.contains(&burner.as_uuid().to_string()), "{text}");
    };

    // The burner's index, filed under the work persona: refused.
    refile::<StoredIndex>(
        &storage,
        &burner_items.index_path(),
        &work_items.index_path(),
    );
    named(work_items.list().unwrap_err());
    named(work_items.get(item).unwrap_err());

    // A work index naming the burner's item record, re-filed: refused.
    let index = StoredIndex {
        items: vec![item],
        ..StoredIndex::empty(work)
    };
    storage
        .save_record(work_items.index_path(), &index)
        .unwrap();
    refile::<StoredItem>(
        &storage,
        &burner_items.item_path(item),
        &work_items.item_path(item),
    );
    named(work_items.list().unwrap_err());
    named(work_items.get(item).unwrap_err());

    // Work metadata over the burner's payload, re-filed: refused at exercise.
    let metadata = StoredItem::new(work, otp.item().clone());
    storage
        .save_record(work_items.item_path(item), &metadata)
        .unwrap();
    assert_eq!(work_items.get(item).unwrap().as_ref(), Some(otp.item()));
    refile::<StoredPayload>(
        &storage,
        &burner_items.payload_path(credential),
        &work_items.payload_path(credential),
    );
    match OtpItemStore::over(work_items).release_tile_at_unix_time(item, credential, 59) {
        Err(OtpItemError::Store(error)) => named(error),
        Err(other) => panic!("expected a persona refusal, got {other}"),
        Ok(_) => panic!("a payload filed under another persona was exercised"),
    }

    // The burner still reads and exercises its own.
    assert_eq!(burner_items.get(item).unwrap().as_ref(), Some(otp.item()));
    let tile = OtpItemStore::over(burner_items)
        .release_tile_at_unix_time(item, credential, 59)
        .unwrap();
    assert_eq!(tile.code_at_unix_time(59), Some("94287082"));
}

#[test]
fn an_interrupted_insert_never_shows_an_item_without_its_payload() {
    let dir = tempdir().unwrap();
    let persona = PersonaId::new();
    let base = dir
        .path()
        .join("records/castellan/items/v1")
        .join(persona.as_uuid().to_string());
    let crash_after = |step: WriteStep| {
        let resident = claim(dir.path());
        let store = OtpItemStore::over(resident.items(persona).crashing_after(step));
        let error = store.import_otpauth_uri(&uri()).unwrap_err();
        assert!(error.to_string().contains("simulated crash"), "{error}");
    };

    // Stopped after the payload: it is on disk, and nothing shows.
    crash_after(WriteStep::Payloads);
    let resident = claim(dir.path());
    assert_eq!(json_records(&base.join("payloads")).len(), 1);
    assert!(json_records(&base.join("items")).is_empty());
    assert!(resident.items(persona).list().unwrap().is_empty());
    drop(resident);

    // Stopped after the metadata: it is on disk too, and still nothing shows,
    // by listing or by its id.
    crash_after(WriteStep::Metadata);
    let resident = claim(dir.path());
    assert_eq!(json_records(&base.join("payloads")).len(), 2);
    let orphans = json_records(&base.join("items"));
    assert_eq!(orphans.len(), 1);
    let orphan = ItemId::parse(orphans[0].file_stem().unwrap().to_str().unwrap()).unwrap();
    assert!(resident.items(persona).list().unwrap().is_empty());
    assert_eq!(resident.items(persona).get(orphan).unwrap(), None);

    // A whole insert shows, and every shown credential has its payload.
    let otp = resident
        .otp_items(persona)
        .import_otpauth_uri(&uri())
        .unwrap();
    let shown = resident.items(persona).list().unwrap();
    assert_eq!(shown, vec![otp.item().clone()]);
    for item in &shown {
        for credential in &item.credentials {
            resident
                .otp_items(persona)
                .release_tile_at_unix_time(item.id, credential.id, 59)
                .unwrap();
        }
    }
}

#[test]
fn listing_and_metadata_reads_never_open_a_payload() {
    let dir = tempdir().unwrap();
    let otp_items = OtpItemStore::new(
        SealedRecordStorage::open_with_key(dir.path(), KEY),
        PersonaId::new(),
    );
    let otp = otp_items.import_otpauth_uri(&uri()).unwrap();
    let (item, credential) = (otp.item_id(), otp.credential_id());
    // A well-formed envelope whose ciphertext cannot be opened.
    std::fs::write(
        dir.path().join(otp_items.items().payload_path(credential)),
        r#"{"version":2,"generation":1,"deleted":false,"nonce":[0,0,0,0,0,0,0,0,0,0,0,0],"ciphertext":[1,2,3]}"#,
    )
    .unwrap();

    assert_eq!(otp_items.items().list().unwrap(), vec![otp.item().clone()]);
    assert_eq!(
        otp_items.items().get(item).unwrap().as_ref(),
        Some(otp.item())
    );
    assert_eq!(otp_items.get(item, credential).unwrap(), Some(otp.clone()));
    assert_eq!(
        otp_items
            .seconds_remaining_at(item, credential, 59)
            .unwrap(),
        Some(1)
    );
    let gate = OtpReleaseGate::new(otp_items);
    let request = gate.petition(item, credential, participant()).unwrap();

    // Only exercising the credential opens its payload, and that fails.
    let error = gate.approve(request.id).unwrap_err();
    assert!(
        error.to_string().contains("decrypt sealed record"),
        "{error}"
    );
}

#[test]
fn a_quarantined_item_is_never_petitioned_or_exercised() {
    let dir = tempdir().unwrap();
    let storage = SealedRecordStorage::open_with_key(dir.path(), KEY);
    let persona = PersonaId::new();
    let otp_items = OtpItemStore::new(storage.clone(), persona);
    let otp = otp_items.import_otpauth_uri(&uri()).unwrap();
    let (item, credential) = (otp.item_id(), otp.credential_id());
    let mut quarantined = otp.item().clone();
    quarantined.state = ItemState::Quarantined;
    storage
        .save_record(
            otp_items.items().item_path(item),
            &StoredItem::new(persona, quarantined.clone()),
        )
        .unwrap();

    assert_eq!(otp_items.items().list().unwrap(), vec![quarantined]);
    let gate = OtpReleaseGate::new(otp_items.clone());
    assert!(matches!(
        gate.petition(item, credential, participant()),
        Err(OtpReleaseError::Item(OtpItemError::Store(ItemStoreError::Quarantined(id)))) if id == item
    ));
    assert!(matches!(
        otp_items.release_tile_at_unix_time(item, credential, 59),
        Err(OtpItemError::Store(ItemStoreError::Quarantined(id))) if id == item
    ));
}
