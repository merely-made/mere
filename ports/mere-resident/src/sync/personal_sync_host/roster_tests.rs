// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use muniment::MemoryBackend;
use personae::{IdentityProvider, InMemoryProvider};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use uuid::Uuid;

async fn roster_host(
    seed: u8,
    graph: [u8; 32],
    roster: SyncRoster,
) -> (tempfile::TempDir, PersonalSyncHost) {
    let directory = tempfile::tempdir().unwrap();
    let identity = InMemoryProvider::from_seed([seed; 32]);
    let host = PersonalSyncHost::open(
        &identity,
        PersonalSyncHostConfig {
            graph,
            store_path: directory.path().join("roster-test.redb"),
            roster,
            selection: SyncSelection::default(),
            peer_tickets: Vec::new(),
            peer_hints: Vec::new(),
            paired_nodes: Vec::new(),
            relay_urls: Vec::new(),
        },
    )
    .await
    .unwrap();
    (directory, host)
}

async fn authored_add_node(
    writer: &InMemoryProvider,
    graph: [u8; 32],
    id: Uuid,
) -> p2panda_core::Operation<PersonalGraphExt> {
    let roster = SyncRoster::new([writer.master_public_key().to_bytes()]);
    let mut replica = PersonalGraphReplica::for_identity(
        MemoryBackend::new(),
        graph,
        writer,
        roster,
        SyncSelection::default(),
    )
    .unwrap();
    replica
        .author(vec![PersonalGraphEvent::AddNode {
            id,
            address: format!("https://{id}.test/"),
            title: "Roster test node".into(),
        }])
        .await
        .unwrap()
}

fn poll_pending_once<F: Future>(future: Pin<&mut F>) {
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(future.poll(&mut context), Poll::Pending));
}

fn projection_has_node(projection: &graphshell::personal_sync::SyncProjection, id: Uuid) -> bool {
    projection.graph.get_node_key_by_id(id).is_some()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelling_a_blocked_roster_update_leaves_roster_unchanged() {
    let identity = InMemoryProvider::from_seed([0xD7; 32]);
    let own_root = identity.master_public_key().to_bytes();
    let graph = [0xD8; 32];
    let initial = SyncRoster::new([own_root]);
    let (_directory, host) = roster_host(0xD7, graph, initial.clone()).await;

    let added_writer = InMemoryProvider::from_seed([0xD9; 32]);
    let requested = SyncRoster::new([own_root, added_writer.master_public_key().to_bytes()]);

    // Force set_roster to reach its first blocked await. Polling the future
    // directly makes cancellation deterministic and needs no timing window.
    let replica_guard = host.replica.lock().await;
    let mut update = Box::pin(host.set_roster(requested));
    poll_pending_once(update.as_mut());

    // The baseline published the intake copy before waiting for the replica.
    // The single-owner implementation waits before changing its sole roster,
    // so cancellation here leaves the authority unchanged.
    drop(update);
    drop(replica_guard);

    let intake_roster = host.roster().await;
    let projection = host.replica.lock().await.projection().await;
    assert_eq!(intake_roster, initial);
    assert!(
        projection.is_ok(),
        "the projection authority must remain usable after cancellation"
    );
    host.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn intake_queued_before_roster_update_uses_old_policy_then_retries_after_admission() {
    let owner = InMemoryProvider::from_seed([0xE1; 32]);
    let own_root = owner.master_public_key().to_bytes();
    let initial = SyncRoster::new([own_root]);
    let (directory, host) = roster_host(0xE1, [0xE2; 32], initial).await;
    let writer = InMemoryProvider::from_seed([0xE3; 32]);
    let writer_root = writer.master_public_key().to_bytes();
    let node = Uuid::from_u128(0xe4);
    let operation = authored_add_node(&writer, [0xE2; 32], node).await;
    let expanded = SyncRoster::new([own_root, writer_root]);

    // Queue intake first and the update second behind the same mutex. Tokio's
    // FIFO mutex then makes the policy transition occur after this intake.
    let replica_guard = host.replica.lock().await;
    let mut incoming = Box::pin(accept_incoming(
        &host.replica,
        &host.keys,
        [0xE2; 32],
        &operation,
    ));
    poll_pending_once(incoming.as_mut());
    let mut update = Box::pin(host.set_roster(expanded));
    poll_pending_once(update.as_mut());
    drop(replica_guard);

    let (accepted, ()) = tokio::join!(incoming, update);
    assert!(matches!(
        accepted,
        Err(stickleback::ProcessError::Rejected(ref reject))
            if reject.code == "personal-graph-writer-not-admitted"
    ));
    let before_retry = host.replica.lock().await.projection().await.unwrap();
    assert!(!projection_has_node(&before_retry, node));
    let retry = accept_incoming(&host.replica, &host.keys, [0xE2; 32], &operation)
        .await
        .unwrap();
    assert!(retry, "the same operation is accepted after admission");
    let projection = host.replica.lock().await.projection().await.unwrap();
    assert!(projection_has_node(&projection, node));
    host.close().await.unwrap();
    drop(directory);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn roster_update_queued_before_intake_admits_and_projects_new_writer() {
    let owner = InMemoryProvider::from_seed([0xE5; 32]);
    let own_root = owner.master_public_key().to_bytes();
    let (directory, host) = roster_host(0xE5, [0xE7; 32], SyncRoster::new([own_root])).await;
    let writer = InMemoryProvider::from_seed([0xE8; 32]);
    let node = Uuid::from_u128(0xe9);
    let operation = authored_add_node(&writer, [0xE7; 32], node).await;
    let expanded = SyncRoster::new([own_root, writer.master_public_key().to_bytes()]);

    // Reverse queue order. Intake must wait until the new projection roster
    // has been installed, then accept the operation under that same policy.
    let replica_guard = host.replica.lock().await;
    let mut update = Box::pin(host.set_roster(expanded));
    poll_pending_once(update.as_mut());
    let mut incoming = Box::pin(accept_incoming(
        &host.replica,
        &host.keys,
        [0xE7; 32],
        &operation,
    ));
    poll_pending_once(incoming.as_mut());
    drop(replica_guard);

    let ((), accepted) = tokio::join!(update, incoming);
    assert!(accepted.unwrap());
    let projection = host.replica.lock().await.projection().await.unwrap();
    assert!(projection_has_node(&projection, node));
    host.close().await.unwrap();
    drop(directory);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn removed_writer_is_refused_without_poisoning_owner_projection() {
    let owner = InMemoryProvider::from_seed([0xEA; 32]);
    let own_root = owner.master_public_key().to_bytes();
    let writer = InMemoryProvider::from_seed([0xEB; 32]);
    let writer_root = writer.master_public_key().to_bytes();
    let (directory, host) =
        roster_host(0xEA, [0xED; 32], SyncRoster::new([own_root, writer_root])).await;
    let node = Uuid::from_u128(0xee);
    // This writer has not previously contributed a stored operation. Build a
    // signed candidate, remove its root, then deliver the candidate afterward.
    let operation = authored_add_node(&writer, [0xED; 32], node).await;
    host.set_roster(SyncRoster::new([own_root])).await;

    let accepted = accept_incoming(&host.replica, &host.keys, [0xED; 32], &operation).await;
    assert!(matches!(
        accepted,
        Err(stickleback::ProcessError::Rejected(ref reject))
            if reject.code == "personal-graph-writer-not-admitted"
    ));
    let projection = host.replica.lock().await.projection().await.unwrap();
    assert_eq!(projection.graph.node_count(), 0);
    assert!(!projection_has_node(&projection, node));
    host.close().await.unwrap();
    drop(directory);
}
