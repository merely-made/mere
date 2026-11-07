// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Discloses a signed block-adapter pack through a Moot collection, as `capsule-library`
//! discloses an applet, and writes what a host needs to show the review.
//!
//! `block_adapter_disclosure PACK.json COMPONENT.wasm SEED-FILE OUT-DIR`
//!
//! The author shares the pack into a Moot; the founder selects it into a collection; the
//! contribution travels as a plain drop into a second, retaining member, which re-verifies
//! the pack against its contributor and its blobs and writes `OUT-DIR/moot-disclosure.json`
//! plus `packs/<revision>.json` and `components/<revision>.wasm`. A host (the web page through
//! `build_block_adapters.mjs --moot`, the native shell through `--moot-disclosure`) lists each
//! entry as a disclosed capsule, and `CapsuleMount::review` then verifies it once more.
//!
//! The seed file holds the 32-byte author seed as 64 hex digits and must be the key that
//! signed the pack. The real author key is the author's; this proof is for a throwaway seed.
//! One process, no network: the transport and hosting steps of `capsule-library` are not
//! repeated here.
use eidetic::pack::{PACK_SCHEMA, PackManifest, PackPartRole, PackVerdict, verify_pack};
use eidetic::schema::TrustEnvelope;
use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
use gemot::moot::records::{
    CollectionChange, CollectionId, CollectionRef, ContributionRef, collection_cap, fauna_cap,
};
use gemot::moot::{
    AvailabilityPolicy, ErasurePolicy, KeepBound, MootAccessLevel, MootError, MootFile, MootId,
    MootMember, MootMembershipAction, MootRetentionSettings, PolicyRevision,
};
use mien::Policy;
use personae::{IdentityProvider, InMemoryProvider};
use proofs::BlobRef;
use serde::{Deserialize, Serialize};
use servitor::cap_path;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use stickleback::{DropExportProfile, DropLimits, DropRecord, read_plain_drop, write_plain_drop};

type AnyError = Box<dyn Error + Send + Sync>;
const MOOT: [u8; 32] = [0x81; 32];
const COLLECTION: CollectionId = CollectionId([0x82; 32]);
const BLOCK_ADAPTER: &str = "power:block-adapter";

#[derive(Clone, Serialize, Deserialize)]
struct SignedPack {
    manifest: PackManifest,
    trust: TrustEnvelope,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    *eidetic::schema::Hash::of(bytes).as_bytes()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn ensure(ok: bool, why: &str) -> Result<(), AnyError> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn identity(seed: u8) -> InMemoryProvider {
    InMemoryProvider::from_seed([seed; 32])
}
fn collection() -> CollectionRef {
    CollectionRef { moot_id: MOOT, collection_id: COLLECTION }
}
fn retention() -> MootRetentionSettings {
    MootRetentionSettings {
        revision: PolicyRevision(proofs::Digest::blake3(b"block-adapter-disclosure proof")),
        availability: AvailabilityPolicy { promised_floor: KeepBound::Forever },
        erasure: ErasurePolicy { history_ceiling: KeepBound::UntilCheckpoint },
    }
}
async fn open(root: &Path, founder: &InMemoryProvider) -> Result<MootFile, AnyError> {
    fs::create_dir_all(root)?;
    Ok(MootFile::open(root, MootId(MOOT), founder.master_public_key().to_bytes(), retention()).await?)
}

/// A pack is a disclosable block adapter when its signature holds, it names its contributor,
/// its one part is the component the blobs hold, and it asks for the block-adapter power alone.
fn verify_adapter(
    pack: &SignedPack,
    contributor: &str,
    blobs: &BTreeMap<[u8; 32], Vec<u8>>,
) -> Result<(), AnyError> {
    ensure(verify_pack(&pack.manifest, &pack.trust) == PackVerdict::Trusted, "pack signature failed")?;
    ensure(pack.manifest.author == contributor, "contribution does not name the pack author")?;
    ensure(
        pack.manifest.requested_scopes == [BLOCK_ADAPTER.to_string()],
        "a block adapter asks for power:block-adapter and nothing else",
    )?;
    ensure(pack.manifest.parts.len() == 1, "a block adapter pack has one part")?;
    let part = &pack.manifest.parts[0];
    ensure(part.role == PackPartRole::WasmComponent, "the part is not a component")?;
    let bytes = blobs.get(part.blob.0.as_bytes()).ok_or("component missing")?;
    ensure(
        part.bytes == bytes.len() as u64 && part.blob == eidetic::ManifestId::of_blob(bytes),
        "component hash or size mismatch",
    )
}

#[derive(Serialize)]
struct Entry {
    name: String,
    version: String,
    author: String,
    /// The share's manifest id: the blake3 of the pack's exact bytes, which is the review's
    /// `expected_pack_hash`.
    pack_hash: String,
    pack_file: String,
    component_file: String,
    component_hash: String,
    component_bytes: u64,
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure(args.len() == 4, "usage: block_adapter_disclosure PACK.json COMPONENT.wasm SEED-FILE OUT-DIR")?;
    let pack_bytes = fs::read(&args[0])?;
    let component = fs::read(&args[1])?;
    let seed_hex = fs::read_to_string(&args[2])?;
    let seed_hex = seed_hex.trim();
    ensure(seed_hex.len() == 64, "the seed is 32 bytes as 64 hex digits")?;
    let mut seed = [0u8; 32];
    for (i, b) in seed.iter_mut().enumerate() {
        *b = u8::from_str_radix(&seed_hex[2 * i..2 * i + 2], 16)?;
    }
    let out = Path::new(&args[3]);
    ensure(!out.exists(), "the output directory must be new")?;
    let pack: SignedPack = serde_json::from_slice(&pack_bytes)?;
    let author = InMemoryProvider::from_seed(seed);
    let author_hex = hex(&author.master_public_key().to_bytes());
    ensure(pack.manifest.author == author_hex, "the seed is not the key that signed the pack")?;
    let founder = identity(0xe5);
    let mut blobs = BTreeMap::new();
    blobs.insert(hash(&component), component.clone());
    verify_adapter(&pack, &author_hex, &blobs)?;
    let pack_id = hash(&pack_bytes);
    blobs.insert(pack_id, pack_bytes.clone());

    // The publisher's Moot: the author may contribute; the founder selects.
    let source = open(&out.join("publisher"), &founder).await?;
    let mut rules = ConstitutionRules::founder_only(founder.master_public_key().to_bytes());
    rules.admission = Policy::MembersOnly { rate_limit: 100, rate_window_ms: 60_000 };
    rules.grant(CapabilityGrant {
        id: [1; 32],
        subject: author.master_public_key().to_bytes(),
        path_prefix: cap_path(&fauna_cap()),
        not_before_ms: 0,
        expires_at_ms: None,
        delegation_depth: 0,
    });
    for (i, cap) in [collection_cap(collection()), fauna_cap()].into_iter().enumerate() {
        rules.grant(CapabilityGrant {
            id: [i as u8 + 2; 32],
            subject: founder.master_public_key().to_bytes(),
            path_prefix: cap_path(&cap),
            not_before_ms: 0,
            expires_at_ms: None,
            delegation_depth: 0,
        });
    }
    source.found([0xe5; 32], None, None, rules, now_ms()).await?;
    source
        .update_membership_for_identity(
            &founder,
            MootMembershipAction::Create {
                initial_members: [(&founder, MootAccessLevel::Manage), (&author, MootAccessLevel::Write)]
                    .into_iter()
                    .map(|(m, access)| MootMember { member: m.master_public_key().to_bytes(), access })
                    .collect(),
            },
        )
        .await?;
    source
        .declare_collection_for_identity(&founder, COLLECTION, "Block adapters".into(), None, now_ms())
        .await?;
    let share = source
        .share_authorized_for_identity(
            &author,
            pack_id,
            PACK_SCHEMA.into(),
            pack.manifest.name.clone(),
            now_ms(),
        )
        .await?
        .operation;
    let heads = source
        .authorized_collection(COLLECTION, now_ms())
        .await?
        .ok_or("collection missing")?
        .heads;
    source
        .set_collection_membership_for_identity(
            &founder,
            collection(),
            heads,
            CollectionChange::SetMembership {
                contribution: ContributionRef { moot_id: MOOT, share },
                included: true,
            },
            now_ms(),
        )
        .await?;

    // Negative controls: a stranger cannot contribute, and a pack that does not verify as an
    // adapter is not disclosed even when someone is allowed to share it.
    let before = source.object_store().ops(MOOT).await?.len();
    let stranger_refused = matches!(
        source
            .share_authorized_for_identity(&identity(0xd4), [9; 32], PACK_SCHEMA.into(), "intruder".into(), now_ms())
            .await,
        Err(MootError::Unauthorized(_))
    ) && source.object_store().ops(MOOT).await?.len() == before;
    let mut changed = blobs.clone();
    changed.insert(hash(&component), b"substituted component".to_vec());
    let substituted_refused = verify_adapter(&pack, &author_hex, &changed).is_err();
    let mut wrong_author = pack.clone();
    wrong_author.manifest.version = "other".into();
    let forged_refused = verify_adapter(&wrong_author, &author_hex, &blobs).is_err();
    let mut view_ask = pack.clone();
    view_ask.manifest.requested_scopes.push("power:capsule-view".into());
    let mixed_refused = verify_adapter(&view_ask, &author_hex, &blobs).is_err();
    ensure(
        stranger_refused && substituted_refused && forged_refused && mixed_refused,
        "negative controls failed",
    )?;

    // The contribution travels as a plain drop, with its blobs, to a retaining member.
    let mut metadata = Vec::new();
    source
        .export_plain_drop(&mut metadata, DropExportProfile::default(), DropLimits::default())
        .await?;
    let (_, mut records) = read_plain_drop(Cursor::new(&metadata), DropLimits::default())?;
    for (blob_hash, bytes) in &blobs {
        records.push(DropRecord::BlobChunk { blob_hash: *blob_hash, offset: 0, bytes: bytes.clone() });
    }
    let mut drop = Vec::new();
    write_plain_drop(&mut drop, &records, DropLimits::default())?;
    let received = open(&out.join("retainer"), &founder).await?;
    received.import_plain_drop(Cursor::new(&drop), DropLimits::default()).await?;
    let selected = received
        .authorized_collection(COLLECTION, now_ms())
        .await?
        .ok_or("imported collection missing")?;
    ensure(selected.effective_selected.len() == 1, "the collection did not converge")?;
    let fauna = received.authorized_fauna(now_ms()).await?;
    let (_, records) = read_plain_drop(Cursor::new(&drop), DropLimits::default())?;
    let mut held = BTreeMap::new();
    for record in records {
        if let DropRecord::BlobChunk { blob_hash, bytes, .. } = record {
            ensure(hash(&bytes) == blob_hash, "a blob does not match its address")?;
            held.insert(blob_hash, bytes);
        }
    }

    // The retaining member discloses what it verified, and only that.
    fs::create_dir_all(out.join("packs"))?;
    fs::create_dir_all(out.join("components"))?;
    let mut entries = Vec::new();
    for member in &selected.effective_selected {
        let contribution = fauna
            .iter()
            .find(|e| e.op_hash == member.share)
            .ok_or("selection has no contribution")?;
        let bytes = held.get(&contribution.manifest_id).ok_or("selected pack missing")?;
        let signed: SignedPack = serde_json::from_slice(bytes)?;
        verify_adapter(&signed, &hex(&contribution.shared_by), &held)?;
        let part = &signed.manifest.parts[0];
        let body = &held[part.blob.0.as_bytes()];
        ensure(
            received.object_store().sync_store().get_blob(&BlobRef::blake3(body)).await? == Some(body.clone()),
            "the Moot store lost the component bytes",
        )?;
        let revision = hex(&contribution.manifest_id);
        fs::write(out.join("packs").join(format!("{revision}.json")), bytes)?;
        fs::write(out.join("components").join(format!("{revision}.wasm")), body)?;
        entries.push(Entry {
            name: signed.manifest.name.clone(),
            version: signed.manifest.version.clone(),
            author: signed.manifest.author.clone(),
            pack_hash: revision.clone(),
            pack_file: format!("packs/{revision}.json"),
            component_file: format!("components/{revision}.wasm"),
            component_hash: hex(part.blob.0.as_bytes()),
            component_bytes: part.bytes,
        });
    }
    let collection_revision = hex(&hash(&serde_json::to_vec(&selected.version)?));
    let disclosure = serde_json::json!({
        "moot": hex(&MOOT),
        "collection": hex(&COLLECTION.0),
        // The blake3 of the collection version's compact JSON: the same collection state a
        // reviewer sees named in the review.
        "revision": collection_revision,
        "entries": entries,
        "negative_controls": {
            "stranger_contribution_refused": stranger_refused,
            "substituted_component_refused": substituted_refused,
            "forged_pack_refused": forged_refused,
            "mixed_power_refused": mixed_refused,
        },
        "proof_scope": "One process, no network; the contribution travels as a plain drop to a retaining member",
    });
    fs::write(out.join("moot-disclosure.json"), serde_json::to_vec_pretty(&disclosure)?)?;
    println!(
        "Disclosed {} block adapter(s) through moot {} collection {} revision {}",
        entries.len(),
        hex(&MOOT),
        hex(&COLLECTION.0),
        collection_revision
    );
    Ok(())
}
