// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, hex, identity, peer, require};
use ::identity::{IdentityProvider, InMemoryProvider};
use mesh::{
    BlobRef, DeterminismClass, JobId, JobSpec, JobState, MESH_AUTHOR_SALT, MeshEvent, MeshExt,
    MeshStore, MeshStoreError, ResourceId, to_operation,
};
use muniment::RedbBackend;
use p2panda_core::cbor::{decode_cbor_strict, encode_cbor};
use p2panda_core::{Body, Operation};
use serde_json::{Value, json};
use std::path::Path;
use stickleback::{
    DropRecord, ProcessError, ProcessOutcome, decode_operation_record, operation_record,
};

const EXPECTED_MESH: [u8; 32] = [0x4d; 32];

pub async fn run(peer_path: &Path) -> Result<Value> {
    let peer_path = peer_path.canonicalize()?;
    let peer_digest = identity::file_digest(&peer_path)?;
    let mere_identity = identity::mere_identity()?;
    let key = InMemoryProvider::from_seed([0x31; 32]).derive_keypair(MESH_AUTHOR_SALT)?;
    let event = MeshEvent::JobPostedV2 {
        spec: Box::new(JobSpec::simple(
            ResourceId::parse("mesh.blake3/v1")?,
            "input",
            BlobRef::blake3(b"retained receive input fixture"),
            "digest",
            32,
            DeterminismClass::Exact,
        )),
        nonce: 1,
        at_ms: 42,
    };
    let original = to_operation(&key, EXPECTED_MESH, &event, 0, None);
    let record_bytes = encode_cbor(&operation_record(&original, true))?;
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("receive.redb");
    let store = MeshStore::at_path(&database)?;
    let mut observations = Vec::new();
    let mut transport_runs = Vec::new();

    let recovered = peer::recover(&peer_path, "authentic-original", &record_bytes, true)?;
    let generation = &recovered.response["report"]["transmitter"]["generations"][0];
    let source_bound = record_bytes.len().div_ceil(128).div_ceil(2);
    let join = recovered.scenario["receivers"][1]["join_slot"]
        .as_u64()
        .ok_or("join slot")?;
    require(
        (source_bound as u64) < join,
        "late lane must join after source-drain bound",
    )?;
    for (lane, expected_outcome) in [("pinned-four", "inserted"), ("late-one", "duplicate")] {
        let bytes = recovered.recovered(lane, &record_bytes)?;
        let operation = decode(&bytes)?;
        require(
            exact_operation(&operation, &original),
            "inner operation bytes or identity changed",
        )?;
        let before = snapshot(&store).await?;
        let outcome = store.accept_outcome(EXPECTED_MESH, &operation).await?;
        let label = outcome_label(outcome);
        require(
            label == expected_outcome,
            "unexpected durable admission outcome",
        )?;
        let after = snapshot(&store).await?;
        require(
            after["operation_count"] == 1 && after["board_jobs"] == 1,
            "owner should retain exactly one operation and posted job",
        )?;
        if label == "duplicate" {
            require(before == after, "duplicate changed durable owner state")?;
        }
        observations.push(json!({"id":format!("receive-{lane}"),"receiver_lane":lane,
            "completion_slot":recovered.receiver(lane)?["generations"][0]["completion_slot"],
            "operation_id":hex(operation.hash.as_bytes()),"record_bytes_equal":true,
            "header_bytes_equal":true,"body_bytes_equal":true,"outcome":label,
            "owner_before":before,"owner_after":after}));
    }
    let late_join_observation = json!({"join_slot":join,
        "source_drain_upper_bound_exclusive":source_bound,
        "derivation":"one retained generation; at least two fresh-symbol lanes each slot under C4/floor1/one-manifest",
        "total_source_symbols":generation["total_source_symbols"],
        "fresh_sent":generation["fresh_sent"]});
    transport_runs.push(recovered.observation());

    let wrong_mesh = to_operation(&key, [0x99; 32], &event, 0, None);
    let mut bad_body = operation_record(&original, true);
    let DropRecord::Operation {
        inline_body: Some(body),
        ..
    } = &mut bad_body
    else {
        return Err("fixture requires an inline operation body".into());
    };
    body[0] ^= 1;
    let mut bad_signature = operation_record(&original, true);
    let DropRecord::Operation { header, .. } = &mut bad_signature else {
        return Err("fixture requires an operation header".into());
    };
    let signature = original.header.signature.to_bytes();
    let positions: Vec<_> = header
        .windows(signature.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == signature).then_some(index))
        .collect();
    require(
        positions.len() == 1,
        "expected one signature byte string in original header",
    )?;
    header[positions[0]] ^= 1;
    require(
        *header != original.header.encode(),
        "damaged raw signature header did not differ",
    )?;
    for (id, record, expected_stage) in [
        (
            "wrong-mesh",
            operation_record(&wrong_mesh, true),
            "mesh-policy",
        ),
        ("damaged-inner-body", bad_body, "operation-structure"),
        ("damaged-inner-signature", bad_signature, "operation-codec"),
    ] {
        let input = encode_cbor(&record)?;
        require(
            input != record_bytes,
            "negative record must differ from original",
        )?;
        let recovered = peer::recover(&peer_path, id, &input, false)?;
        let bytes = recovered.recovered("pinned-four", &input)?;
        let before = snapshot(&store).await?;
        let rejection = match decode(&bytes) {
            Err(error) if expected_stage == "operation-codec" => {
                require(
                    error.to_string().contains("invalid signature"),
                    "unexpected header decode refusal",
                )?;
                json!({"stage":"operation-codec","code":"invalid-signature","detail":error.to_string()})
            },
            Err(error) => return Err(error),
            Ok(operation) => match store.accept_outcome(EXPECTED_MESH, &operation).await {
                Err(MeshStoreError::Process(ProcessError::Rejected(refusal)))
                    if expected_stage == "mesh-policy" && refusal.code == "wrong-mesh" =>
                {
                    json!({"stage":"mesh-policy","code":refusal.code,"detail":refusal.detail})
                },
                Err(MeshStoreError::Process(ProcessError::InvalidOperation(detail)))
                    if expected_stage == "operation-structure" && detail.contains("payload") =>
                {
                    json!({"stage":"operation-structure","code":"payload-mismatch","detail":detail})
                },
                Err(error) => return Err(format!("unexpected refusal for {id}: {error}").into()),
                Ok(_) => return Err(format!("owner admitted {id}").into()),
            },
        };
        let after = snapshot(&store).await?;
        require(before == after, "refused inner record changed owner state")?;
        observations.push(json!({"id":id,"receiver_lane":"pinned-four",
            "outer_authenticated_reconstruction":true,"record_bytes_equal":true,
            "rejection":rejection,"owner_before":before,"owner_after":after,
            "owner_unchanged":true}));
        transport_runs.push(recovered.observation());
    }

    let before_reopen = snapshot(&store).await?;
    drop(store);
    let reopened = MeshStore::at_path(&database)?;
    let after_reopen = snapshot(&reopened).await?;
    require(
        before_reopen == after_reopen,
        "redb close/reopen changed owner projection",
    )?;
    let retained = reopened
        .operation(&original.hash)
        .await?
        .ok_or("original missing after reopen")?;
    require(
        exact_operation(&retained, &original),
        "durable operation bytes changed",
    )?;
    observations.push(json!({"id":"durable-owner-reopen","kind":"in_process_handle_close_and_reopen",
        "owner_before":before_reopen,"owner_after":after_reopen,
        "operation_id":hex(retained.hash.as_bytes()),"header_bytes_equal":true,"body_bytes_equal":true}));
    require(
        peer_digest == identity::file_digest(&peer_path)?,
        "peer executable changed during campaign",
    )?;
    Ok(json!({
        "schema":"mere.retained-receive-exploration/v1","kind":"owner_admission_design_probe",
        "captured_unix_seconds":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        "mere_implementation":mere_identity,
        "retinue_peer":{"path":peer_path,"executable_blake3":peer_digest},
        "limits":{"single_original_octets":peer::MAX_OBJECT_BYTES,
            "subprocess_request_response_octets":peer::MAX_PEER_BYTES,
            "child_timeout_seconds":peer::CHILD_TIMEOUT_SECONDS},
        "scope":{"actual_raptorq_codec":true,"production_mesh_admission":true,"durable_redb":true,
            "modeled_receiver_lanes":true,"physical_throughput_claim":false,
            "reticulum_carrier_exercised":false,"quic_carrier_exercised":false,
            "mcu_or_rf_exercised":false,"job_execution_or_blob_delivery":false},
        "fixture":{"public_throwaway_identity":true,"operation_id":hex(original.hash.as_bytes()),
            "author_key_hex":hex(original.header.verifying_key.as_bytes()),
            "expected_mesh_hex":hex(&EXPECTED_MESH),"original_record_hex":hex(&record_bytes),
            "late_join":late_join_observation},
        "observations":observations,"transport_runs":transport_runs
    }))
}

fn decode(bytes: &[u8]) -> Result<Operation<MeshExt>> {
    require(bytes.len() <= peer::MAX_OBJECT_BYTES, "record byte budget")?;
    let record: DropRecord = decode_cbor_strict(bytes)?;
    require(
        encode_cbor(&record)? == bytes,
        "noncanonical or trailing operation record bytes",
    )?;
    decode_operation_record(&record)?.ok_or_else(|| "expected an operation record".into())
}

fn exact_operation(left: &Operation<MeshExt>, right: &Operation<MeshExt>) -> bool {
    left.hash == right.hash
        && left.header.encode() == right.header.encode()
        && left.body.as_ref().map(Body::to_bytes) == right.body.as_ref().map(Body::to_bytes)
}

fn outcome_label(outcome: ProcessOutcome) -> &'static str {
    match outcome {
        ProcessOutcome::Inserted { .. } => "inserted",
        ProcessOutcome::Duplicate => "duplicate",
        ProcessOutcome::HydratedPayload => "hydrated-payload",
    }
}

async fn snapshot(store: &MeshStore<RedbBackend>) -> Result<Value> {
    let operations = store.ops(EXPECTED_MESH).await?;
    let board = store.board(EXPECTED_MESH).await?;
    let retained: Vec<_> = operations.iter().map(|operation| json!({
        "operation_id":hex(operation.hash.as_bytes()),"header_hex":hex(&operation.header.encode()),
        "body_hex":operation.body.as_ref().map(|body| hex(&body.to_bytes()))
    })).collect();
    let jobs: Vec<_> = board
        .jobs()
        .map(|job| json!({"id":hex(&job.id.0),"state":job.state}))
        .collect();
    if let Some(operation) = operations.first() {
        require(
            board
                .job(JobId(*operation.hash.as_bytes()))
                .is_some_and(|job| job.state == JobState::Posted),
            "stored V2 post did not materialize as a posted job",
        )?;
    }
    Ok(
        json!({"operation_count":store.sync_store().operation_count().await?,
        "board_jobs":board.len(),"operations":retained,"jobs":jobs}),
    )
}
