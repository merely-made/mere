// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{MESH, Result, hex, network, require, source::Source, unhex};
use identity::{IdentityProvider, InMemoryProvider};
use mesh::ingress::{RecordIngress, read_exact_inventory, submit_original};
use mesh::{
    BlobRef, DeterminismClass, JobSpec, MESH_AUTHOR_SALT, MeshEvent, MeshStore, ResourceId,
    to_operation,
};
use p2panda_core::cbor::encode_cbor;
use p2panda_core::{Body, Operation};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use stickleback::inventory::{
    CandidateBudget, Inventory, InventoryLimits, PeerInventories, RetainedEntry,
};
use stickleback::{DropRecord, operation_record};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use transport::{P2pandaTransport, Transport};

fn originals() -> Vec<Operation<mesh::MeshExt>> {
    let provider = InMemoryProvider::from_seed([23; 32]);
    let key = provider.derive_keypair(MESH_AUTHOR_SALT).unwrap();
    let mut operations = Vec::new();
    let mut backlink = None;
    for index in 0..4 {
        let spec = JobSpec::simple(
            ResourceId::parse("mesh.echo/v1").unwrap(),
            "payload",
            BlobRef::blake3(&[index as u8]),
            "result",
            1024,
            DeterminismClass::Exact,
        );
        let event = MeshEvent::JobPostedV2 {
            spec: Box::new(spec),
            nonce: index as u64,
            at_ms: 1000 + index as u64,
        };
        let operation = to_operation(&key, MESH, &event, index, backlink);
        backlink = Some(*operation.hash.as_bytes());
        operations.push(operation);
    }
    operations
}

async fn inventory(
    client: &mut network::Client,
    grant: &[&str],
) -> Result<stickleback::inventory::Summary> {
    let hello = client.hello("inventory", grant)?;
    let mut stream = client.connect().await?;
    Ok(read_exact_inventory(&mut stream, &hello, MESH, network::CALL).await?)
}

async fn submit(client: &mut network::Client, bytes: &[u8]) -> Result<mesh::ingress::SubmitResult> {
    let hello = client.hello("submit", &["submit"])?;
    let mut stream = client.connect().await?;
    Ok(submit_original(&mut stream, &hello, bytes, network::CALL).await?)
}

async fn recover(
    source: &mut Source,
    generation: u64,
    expected: &[u8],
) -> Result<(Vec<u8>, Value)> {
    let metadata = source
        .ok(json!({"op":"lookup_metadata","generation":generation}))
        .await?;
    require(
        metadata["metadata"]["digest_hex"] == blake3::hash(expected).to_hex().as_str(),
        "owner metadata digest",
    )?;
    let result = source
        .ok(json!({"op":"recover","generation":generation}))
        .await?;
    let recovered = &result["recovered"];
    let bytes = unhex(
        recovered["original_bytes_hex"]
            .as_str()
            .ok_or("decoder bytes")?,
    )?;
    require(
        bytes == expected
            && recovered["actual_decoder_output"] == true
            && recovered["application_admission"] == false,
        "actual opaque decoder output",
    )?;
    require(
        result["preexisting_selected_object"] == false
            && result["recovery_kind"] == "repair_only_delivery_loopback",
        "fresh repair reconstruction",
    )?;
    Ok((bytes, result))
}

async fn framing_refusal(client: &mut network::Client, bytes: &[u8], case: &str) -> Result<()> {
    let hello = client.hello("submit", &["submit"])?;
    let mut stream = client.connect().await?;
    let reply =
        notochord::initiate_session(&mut stream, &hello, &notochord::HandshakeLimits::default())
            .await?;
    require(
        matches!(reply, notochord::SessionReply::Accept { .. }),
        "framing fixture admission",
    )?;
    notochord::write_frame(&mut stream, blake3::hash(bytes).as_bytes(), 32).await?;
    match case {
        "oversized declaration" => stream.write_all(&16385u32.to_be_bytes()).await?,
        "truncated record" => {
            stream
                .write_all(&(bytes.len() as u32).to_be_bytes())
                .await?;
            stream.write_all(&bytes[..1]).await?;
        },
        "trailing byte" => {
            stream
                .write_all(&(bytes.len() as u32).to_be_bytes())
                .await?;
            stream.write_all(bytes).await?;
            stream.write_all(&[0]).await?;
        },
        _ => return Err("framing case".into()),
    }
    // Transport shutdown may itself observe the receiver's reset on refusal.
    let _ = tokio::time::timeout(network::CALL, stream.shutdown()).await?;
    let mut response = Vec::new();
    let _ = tokio::time::timeout(
        network::CALL,
        (&mut stream).take(2049).read_to_end(&mut response),
    )
    .await?;
    require(
        response.is_empty(),
        "invalid framing produced an owner receipt",
    )
}

pub async fn run(peer: &Path) -> Result<Value> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("ingress.redb");
    let store = MeshStore::at_path(&database)?;
    let server = Arc::new(
        P2pandaTransport::builder_from_seed_ref(&[22; 32])
            .bind()
            .await?,
    );
    let service = RecordIngress::new(store.clone(), MESH, network::config())?;
    let handle = service.clone().mount(server.clone()).await?;
    let mut client = network::Client::bind(&server).await?;
    let originals = originals();
    let records = originals
        .iter()
        .map(|op| encode_cbor(&operation_record(op, true)))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    require(
        records.iter().all(|b| b.len() <= 16384),
        "record size profile",
    )?;
    require(
        client.transport.local_peer_id().to_bytes()
            != *originals[0].header.verifying_key.as_bytes(),
        "transport subject must differ from original operation author",
    )?;
    let mut source = Source::start(peer).await?;
    source
        .ok(
            json!({"op":"init","public_fixture":true,"namespace_hex":hex(&MESH),
        "authority_seed_hex":hex(&[24;32]),"channels":4,"manifest_interval":5,"repair_floor":1}),
        )
        .await?;
    source.ok(json!({"op":"publish","generation":1,"original_bytes_hex":hex(&records[0]),"expires_at":2048})).await?;
    let first_snapshot = source.ok(json!({"op":"snapshot"})).await?;
    require(
        first_snapshot["inventory"]
            .as_array()
            .ok_or("first inventory")?
            .len()
            == 1,
        "initial live source",
    )?;
    let (first, first_recovery) = recover(&mut source, 1, &records[0]).await?;
    let first_result = submit(&mut client, &first).await?;
    require(
        first_result.outcome == "inserted",
        "first resident owner insertion",
    )?;
    for (index, bytes) in records.iter().enumerate().skip(1) {
        source.now += 1;
        source
            .ok(json!({"op":"publish","generation":index+1,
            "original_bytes_hex":hex(bytes),"expires_at":2048}))
            .await?;
    }
    let source_snapshot = source.ok(json!({"op":"snapshot"})).await?;
    require(
        source_snapshot["inventory"]
            .as_array()
            .ok_or("source inventory")?
            .len()
            == 4,
        "four live originals",
    )?;
    let refused = source
        .command(json!({"op":"publish","generation":5,
        "original_bytes_hex":hex(&records[0]),"expires_at":2048}))
        .await?;
    require(refused["status"] == "refused", "producer capacity refusal")?;
    let after_capacity = source.ok(json!({"op":"snapshot"})).await?;
    require(
        after_capacity["inventory"] == source_snapshot["inventory"],
        "capacity refusal changed retained set",
    )?;

    let mut local = Inventory::new(
        MESH,
        InventoryLimits {
            entries: 4,
            retained_bytes: 65536,
            object_bytes: 16384,
        },
    )?;
    for item in source_snapshot["inventory"].as_array().unwrap() {
        let digest = unhex(item["digest_hex"].as_str().ok_or("source digest")?)?;
        local.retain(
            RetainedEntry {
                digest: digest.try_into().map_err(|_| "digest length")?,
                bytes: item["object_octets"].as_u64().ok_or("source length")? as usize,
                expires_at: item["expires_at"].as_u64().ok_or("source expiry")?,
            },
            source.now as u64,
        )?;
    }
    let remote_before = inventory(&mut client, &["inventory"]).await?;
    let mut views = PeerInventories::new(MESH, 8, 32)?;
    let receiver = server.local_peer_id().to_bytes();
    require(
        views
            .observe(receiver, &remote_before, source.now as u64)?
            .complete,
        "complete exact inventory",
    )?;
    let mut candidates = views.offers(
        &local,
        &receiver,
        source.now as u64,
        CandidateBudget::new(4, 65536)?,
    )?;
    require(
        candidates.len() == 3,
        "exact inventory should skip already retained first object",
    )?;
    candidates.sort_by_key(|o| {
        records
            .iter()
            .position(|b| blake3::hash(b).as_bytes() == &o.digest)
            .unwrap()
    });
    let mut deliveries =
        vec![json!({"generation":1,"recovery":first_recovery,"owner_result":first_result})];
    for offer in &candidates {
        let index = records
            .iter()
            .position(|b| blake3::hash(b).as_bytes() == &offer.digest)
            .ok_or("candidate has no retained owner")?;
        let (bytes, recovery) = recover(&mut source, (index + 1) as u64, &records[index]).await?;
        require(
            blake3::hash(&bytes).as_bytes() == &offer.digest,
            "retrieved candidate digest",
        )?;
        let result = submit(&mut client, &bytes).await?;
        require(
            result.outcome == "inserted"
                && result.operation_id.as_deref()
                    == Some(hex(originals[index].hash.as_bytes()).as_str()),
            "candidate did not reach original owner identity",
        )?;
        deliveries.push(json!({"generation":index+1,"candidate_digest":hex(&offer.digest),"recovery":recovery,"owner_result":result}));
    }
    let source_report = source.finish().await?;
    let remote_after = inventory(&mut client, &["inventory"]).await?;
    views.observe(
        receiver,
        &remote_after,
        source_report["final_now_slot"].as_u64().unwrap(),
    )?;
    require(
        views
            .offers(
                &local,
                &receiver,
                source_report["final_now_slot"].as_u64().unwrap(),
                CandidateBudget::new(4, 65536)?,
            )?
            .is_empty(),
        "receiver exact inventory did not reconcile",
    )?;
    let duplicate = submit(&mut client, &records[0]).await?;
    require(duplicate.outcome == "duplicate", "duplicate owner result")?;
    let mut preservation = Vec::new();
    for (index, original) in originals.iter().enumerate() {
        let stored = store
            .operation(&original.hash)
            .await?
            .ok_or("stored signed operation absent")?;
        require(
            stored.header.encode() == original.header.encode()
                && stored.body.as_ref().map(Body::to_bytes)
                    == original.body.as_ref().map(Body::to_bytes),
            "stored signed bytes changed",
        )?;
        require(
            service
                .retained_original(blake3::hash(&records[index]).as_bytes())?
                .as_deref()
                == Some(records[index].as_slice()),
            "retained whole record changed",
        )?;
        preservation.push(json!({"generation":index+1,"operation_id":hex(original.hash.as_bytes()),
            "record_blake3":blake3::hash(&records[index]).to_hex().to_string(),"record_bytes_equal":true,
            "header_bytes_equal":true,"body_bytes_equal":true}));
    }
    let jobs_before = store.board(MESH).await?.jobs().count();
    require(
        store.ops(MESH).await?.len() == 4 && jobs_before == 4,
        "one durable effect per original",
    )?;

    let calls_before = service.stats().owner_calls;
    let mut denied = Vec::new();
    let only_submit = inventory(&mut client, &["submit"]).await;
    require(
        matches!(only_submit,Err(ref e) if e.to_string().contains("session not admitted")),
        "submission permission leaked inventory access",
    )?;
    denied.push(json!({"case":"submit-only cannot read inventory","error":only_submit.err().unwrap().to_string()}));
    let hello = client.hello("submit", &["inventory"])?;
    let mut stream = client.connect().await?;
    let only_read = submit_original(&mut stream, &hello, &records[0], network::CALL).await;
    require(
        matches!(only_read, Err(mesh::ingress::IngressError::Denied(_))),
        "inventory permission leaked submission",
    )?;
    denied.push(
        json!({"case":"inventory-only cannot submit","error":only_read.err().unwrap().to_string()}),
    );
    require(
        service.stats().owner_calls == calls_before,
        "denied session reached owner",
    )?;

    let mut negatives = Vec::new();
    let mut bad_body = operation_record(&originals[0], true);
    let DropRecord::Operation {
        inline_body: Some(ref mut body),
        ..
    } = bad_body
    else {
        return Err("body fixture".into());
    };
    body[0] ^= 1;
    let mut bad_signature = operation_record(&originals[0], true);
    let DropRecord::Operation { ref mut header, .. } = bad_signature else {
        return Err("signature fixture".into());
    };
    let signature = originals[0].header.signature.to_bytes();
    let offset = header
        .windows(signature.len())
        .position(|s| s == signature)
        .ok_or("signature offset")?;
    header[offset] ^= 1;
    let author = InMemoryProvider::from_seed([23; 32]).derive_keypair(MESH_AUTHOR_SALT)?;
    let event = MeshEvent::JobPostedV2 {
        spec: Box::new(JobSpec::simple(
            ResourceId::parse("mesh.echo/v1")?,
            "payload",
            BlobRef::blake3(b"wrong"),
            "result",
            1024,
            DeterminismClass::Exact,
        )),
        nonce: 100,
        at_ms: 1000,
    };
    let wrong = to_operation(&author, [0x99; 32], &event, 0, None);
    for (label, bytes) in [
        ("damaged body", encode_cbor(&bad_body)?),
        ("damaged signature", encode_cbor(&bad_signature)?),
        ("wrong mesh", encode_cbor(&operation_record(&wrong, true))?),
    ] {
        let outcome = submit(&mut client, &bytes).await?;
        require(outcome.outcome == "refused", "invalid record admitted")?;
        require(
            store.ops(MESH).await?.len() == 4 && store.board(MESH).await?.jobs().count() == 4,
            "refusal changed durable owner",
        )?;
        negatives.push(json!({"case":label,"result":outcome,"owner_unchanged":true}));
    }

    let before_framing = service.stats().owner_calls;
    let mut framing = Vec::new();
    for case in ["oversized declaration", "truncated record", "trailing byte"] {
        framing_refusal(&mut client, &records[0], case).await?;
        require(
            service.stats().owner_calls == before_framing,
            "framing failure reached admission owner",
        )?;
        framing.push(json!({"case":case,"owner_called":false,"owner_receipt":false}));
    }

    // Actual shared ingress: two stalled handshakes occupy both active slots;
    // four more reserve the carrier queue, and another attempt is refused.
    let mut stalls = Vec::new();
    for _ in 0..6 {
        let mut stream = client.connect().await?;
        stream.write_all(&[0]).await?;
        stream.flush().await?;
        stalls.push(stream);
    }
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if service.stats().active == 2 && handle.queue_stats().pending == 4 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    let full = handle.queue_stats();
    let mut rejected = client.connect().await?;
    let _ = rejected.write_all(&[0]).await;
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if handle.queue_stats().refused > 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            if service.stats().active == 0 && handle.queue_stats().pending == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    drop(stalls);
    drop(rejected);
    let healthy = submit(&mut client, &records[0]).await?;
    require(
        healthy.outcome == "duplicate" && service.stats().peak_active == 2,
        "healthy call after stalls",
    )?;
    require(store.ops(MESH).await?.len() == 4, "stalls altered owner")?;
    let queue = handle.queue_stats();
    let stats = service.stats();
    require(
        stats.timeouts >= 2 && queue.peak_pending <= 4 && stats.peak_active <= 2,
        "shared budget observation",
    )?;
    let paths = client.transport.peer_paths(server.local_peer_id()).await?;
    require(
        paths.iter().any(|p| p.active)
            && paths
                .iter()
                .filter(|p| p.active)
                .all(|p| matches!(&p.addr,transport::PeerAddr::Direct(a) if a.ip().is_loopback())),
        "unexpected active path",
    )?;
    let path_observation = json!(
        paths
            .iter()
            .map(|p| json!({"addr":p.addr.to_string(),"active":p.active}))
            .collect::<Vec<_>>()
    );
    handle.shutdown().await?;
    drop(service);
    client.transport.close().await?;
    server.close().await?;
    drop(store);
    let reopened = MeshStore::at_path(&database)?;
    require(
        reopened.ops(MESH).await?.len() == 4 && reopened.board(MESH).await?.jobs().count() == 4,
        "durable owner reopen",
    )?;
    for original in &originals {
        let stored = reopened
            .operation(&original.hash)
            .await?
            .ok_or("reopened operation")?;
        require(
            stored.header.encode() == original.header.encode()
                && stored.body.as_ref().map(Body::to_bytes)
                    == original.body.as_ref().map(Body::to_bytes),
            "reopened signed bytes",
        )?;
    }
    drop(reopened);
    let expected = directory.path().join("originals.json");
    std::fs::write(
        &expected,
        serde_json::to_vec(&records.iter().map(|r| hex(r)).collect::<Vec<_>>())?,
    )?;
    let status = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(std::env::current_exe()?)
            .args([
                "--reopen",
                database.to_str().ok_or("database path")?,
                expected.to_str().ok_or("fixture path")?,
            ])
            .kill_on_drop(true)
            .status(),
    )
    .await??;
    require(status.success(), "fresh process durable owner reopen")?;
    Ok(
        json!({"schema":"mere.exact-inventory-admitted-composition/v1",
        "source":source_report,"deliveries":deliveries,"preservation":preservation,
        "receiver_inventory_before_hex":hex(&remote_before.encode()),"receiver_inventory_after_hex":hex(&remote_after.encode()),
        "candidate_count":3,"source_live_objects":4,"source_capacity_refusal":refused,
        "duplicate":duplicate,"permission_refusals":denied,"invalid_records":negatives,
        "framing_refusals":framing,
        "owner":{"operations":4,"posted_jobs":4,"same_existing_mesh_admission_owner":true,
            "durable_reopen":"closed_handles_then_independent_process","signed_bytes_preserved":true},
        "service_stats":stats,"queue":{"at_capacity_pending":full.pending,"peak_pending":queue.peak_pending,"refused":queue.refused},
        "following_stalls":healthy,"active_dial_path_observation":path_observation,
        "scope":{"real_quic":true,"real_notochord_admission":true,"actual_raptorq_decoder":true,
            "retained_frame_delivery":"host repair-only loopback before real opaque QUIC handoff",
            "native_resident_ui":false,"physical_radio":false,"worker_execution":false},
        "endpoints_closed":true}),
    )
}
