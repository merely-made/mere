// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use graphshell_door::blob_door::{hex, hex32};
use graphshell_host::lifecycle::AdmittedEndpointContext;

use super::*;
use crate::settings::ResidentContentSettings;

const SCOPE: [u8; 32] = [0x41; 32];

async fn custody(root: &std::path::Path) -> ResidentBlobCustody {
    custody_with_store(root, None).await
}

/// A custody over `root`, its blob store at `store` when given. A second
/// custody on one root models a restart without waiting on the first store's
/// file lock: the scope-owner record, which is what a restart keeps, is shared.
async fn custody_with_store(
    root: &std::path::Path,
    store: Option<std::path::PathBuf>,
) -> ResidentBlobCustody {
    ResidentBlobCustody::open(
        root,
        &ResidentContentSettings {
            gc_interval_seconds: 60,
            root: store,
            ..ResidentContentSettings::default()
        },
    )
    .await
    .unwrap()
}

/// The endpoint the catalog's factory builds for an admitted `app`: the same
/// refusals (revoked), through the catalog's own selection.
fn open(door: &BlobDoor, app: &str) -> Result<Box<dyn ResidentEndpoint>, String> {
    let mut catalog = ResidentEndpointCatalog::new();
    door.register(&mut catalog).unwrap();
    let context = AdmittedEndpointContext::new(BlobEndpoint::session(), [7; 32]).with_application(app);
    catalog
        .open(BLOB_CUSTODY_ROUTE, &context)
        .map_err(|error| error.to_string())?;
    Ok(Box::new(BlobEndpoint::new(door.clone(), app.to_string())))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_route_opens_only_for_an_admitted_application() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut catalog = ResidentEndpointCatalog::new();
    let route = door.register(&mut catalog).unwrap();
    assert_eq!(route.id(), BLOB_CUSTODY_ROUTE);
    assert!(door.register(&mut catalog).is_err(), "one registration");
    let anonymous = AdmittedEndpointContext::new(BlobEndpoint::session(), [7; 32]);
    assert!(catalog.open(BLOB_CUSTODY_ROUTE, &anonymous).is_err());
    let named = anonymous.with_application("knot-editor");
    assert!(catalog.open(BLOB_CUSTODY_ROUTE, &named).is_ok());
}

fn invoke(
    endpoint: &mut dyn ResidentEndpoint,
    call: u64,
    op: BlobOp,
) -> Result<BlobOutcome, String> {
    let result = endpoint.invoke(IntentInvocation {
        session: BlobEndpoint::session(),
        target: InstanceId(0),
        observed_epoch: SceneEpoch(1),
        observed_revision: Revision(1),
        intent: BLOB_CALL_INTENT.into(),
        payload: serde_json::to_vec(&BlobRequest { call, op }).unwrap(),
    })?;
    // Read the outcome the way a client does: from the snapshot's ledger.
    let snapshot = endpoint.snapshot(BlobEndpoint::request())?;
    let offer = snapshot.presentation.offers.values().flatten().next().unwrap();
    let card: PortableCardV1 = serde_json::from_slice(
        &endpoint
            .resource(ResourceRequest {
                session: BlobEndpoint::session(),
                resource: offer.resource,
            })?
            .bytes,
    )
    .unwrap();
    let ledger: BlobLedger = serde_json::from_slice(
        &endpoint
            .resource(ResourceRequest {
                session: BlobEndpoint::session(),
                resource: card.media[0],
            })?
            .bytes,
    )
    .unwrap();
    let outcome = ledger
        .outcomes
        .into_iter()
        .find(|answered| answered.call == call)
        .expect("every call has an outcome")
        .outcome;
    match (result, outcome) {
        (IntentResult::Rejected { reason }, BlobOutcome::Refused { reason: kept }) => {
            assert_eq!(reason, kept, "the intent and the ledger give one reason");
            Err(reason)
        },
        (IntentResult::Accepted, outcome) => Ok(outcome),
        (result, outcome) => panic!("{result:?} does not match {outcome:?}"),
    }
}

fn bind(endpoint: &mut dyn ResidentEndpoint, scope: [u8; 32], call: u64) -> Result<BlobOutcome, String> {
    invoke(
        endpoint,
        call,
        BlobOp::BindScope {
            scope: hex32(&scope),
        },
    )
}

fn put(
    endpoint: &mut dyn ResidentEndpoint,
    call: u64,
    namespace: &str,
    subject: &[u8],
    bytes: &[u8],
) -> Result<BlobOutcome, String> {
    let mut call = call;
    invoke(
        endpoint,
        call,
        BlobOp::PutBegin {
            scope: hex32(&SCOPE),
            namespace: namespace.into(),
            subject: hex(subject),
            len: bytes.len() as u64,
            hash: hex32(blake3::hash(bytes).as_bytes()),
        },
    )?;
    let mut offset = 0u64;
    for part in bytes.chunks(MAX_PART_BYTES) {
        call += 1;
        invoke(
            endpoint,
            call,
            BlobOp::PutPart {
                offset,
                bytes: part.to_vec(),
            },
        )?;
        offset += part.len() as u64;
    }
    invoke(endpoint, call + 1, BlobOp::PutCommit)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_blob_is_put_leased_listed_and_read_back_in_chunks() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut endpoint = open(&door, "knot-editor").unwrap();
    assert_eq!(bind(&mut *endpoint, SCOPE, 1).unwrap(), BlobOutcome::Count { count: 0 });

    // Three parts, so the upload path crosses its part limit.
    let bytes: Vec<u8> = (0..(MAX_PART_BYTES * 2 + 123)).map(|n| (n % 251) as u8).collect();
    let outcome = put(&mut *endpoint, 10, "knot.evidence", b"artifact-1", &bytes).unwrap();
    let hash = *blake3::hash(&bytes).as_bytes();
    assert_eq!(outcome, BlobOutcome::Hash { hash: hex32(&hash) });

    // The lease is the in-process lease: same tag, readable from the store.
    let lease = BlobLease::new(BlobScope::new(SCOPE), "knot.evidence", b"artifact-1").unwrap();
    assert_eq!(
        door.custody.blobs().lease_hash(&lease).await.unwrap(),
        Some(BlobHash::from_bytes(hash))
    );
    assert_eq!(
        invoke(
            &mut *endpoint,
            30,
            BlobOp::LeaseHash {
                scope: hex32(&SCOPE),
                namespace: "knot.evidence".into(),
                subject: hex(b"artifact-1"),
            },
        )
        .unwrap(),
        BlobOutcome::MaybeHash {
            hash: Some(hex32(&hash))
        }
    );

    // Read it back through the carrier's chunks, each frame verifying itself.
    let mut got = Vec::new();
    loop {
        let chunk = endpoint
            .resource_chunk(ResourceChunkRequest {
                session: BlobEndpoint::session(),
                resource: ContentHash(hash),
                offset: got.len() as u64,
                length: 100_000,
            })
            .unwrap();
        let piece = chunk.decode().expect("the chunk verifies");
        assert_eq!(chunk.total_len, bytes.len() as u64);
        got.extend_from_slice(&piece);
        if chunk.is_final(piece.len()) {
            break;
        }
    }
    assert_eq!(got, bytes);

    // Releasing the lease stops the reads: the hash is no longer this
    // application's to fetch.
    assert_eq!(
        invoke(
            &mut *endpoint,
            40,
            BlobOp::Release {
                scope: hex32(&SCOPE),
                namespace: "knot.evidence".into(),
                subject: hex(b"artifact-1"),
            },
        )
        .unwrap(),
        BlobOutcome::Bool { value: true }
    );
    assert!(endpoint
        .resource_chunk(ResourceChunkRequest {
            session: BlobEndpoint::session(),
            resource: ContentHash(hash),
            offset: 0,
            length: 0,
        })
        .is_err());
    drop(endpoint);
    drop(door);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_scope_belongs_to_its_first_binder_and_the_record_survives_a_restart() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut knot = open(&door, "knot-editor").unwrap();
    let mut other = open(&door, "other-app").unwrap();
    bind(&mut *knot, SCOPE, 1).unwrap();
    let refused = bind(&mut *other, SCOPE, 1).unwrap_err();
    assert!(refused.contains("knot-editor"), "{refused}");
    // Other calls on a scope the application never bound are refused too.
    let unbound = invoke(
        &mut *other,
        2,
        BlobOp::Retain {
            scope: hex32(&SCOPE),
            hash: hex32(&[1; 32]),
        },
    )
    .unwrap_err();
    assert!(unbound.contains("knot-editor"), "{unbound}");
    drop((knot, other, door));

    // A new resident process: the record is on disk, so the application that
    // connects first after a restart does not inherit the scope.
    let restarted =
        BlobDoor::new(custody_with_store(temp.path(), Some(temp.path().join("second-store"))).await);
    let mut other = open(&restarted, "other-app").unwrap();
    assert!(bind(&mut *other, SCOPE, 1).is_err());
    let mut knot = open(&restarted, "knot-editor").unwrap();
    assert_eq!(bind(&mut *knot, SCOPE, 1).unwrap(), BlobOutcome::Count { count: 0 });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_residents_own_scope_is_never_an_applications() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut app = open(&door, "knot-editor").unwrap();
    bind(&mut *app, SCOPE, 1).unwrap();
    // The resident takes the scope for an in-process lane: the application's
    // claim is gone and its calls are refused.
    door.custody.reserve_scope(BlobScope::new(SCOPE)).unwrap();
    let refused = bind(&mut *app, SCOPE, 2).unwrap_err();
    assert!(refused.contains("resident"), "{refused}");
    assert_eq!(door.custody.scope_owner(BlobScope::new(SCOPE)).as_deref(), Some("resident"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serving_authority_is_the_in_process_authorizers_and_needs_a_lease() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut app = open(&door, "knot-editor").unwrap();
    bind(&mut *app, SCOPE, 1).unwrap();
    let hash = *blake3::hash(b"evidence").as_bytes();

    // No lease, no serving.
    let refused = invoke(
        &mut *app,
        2,
        BlobOp::Retain {
            scope: hex32(&SCOPE),
            hash: hex32(&hash),
        },
    )
    .unwrap_err();
    assert!(refused.contains("no lease"), "{refused}");

    let bytes = b"evidence";
    put(&mut *app, 10, "knot.evidence", b"e1", bytes).unwrap();
    let peer = [9u8; 32];
    let scope = |op: fn(String, String) -> BlobOp, value: String| op(hex32(&SCOPE), value);
    assert_eq!(
        invoke(
            &mut *app,
            20,
            BlobOp::Retain {
                scope: hex32(&SCOPE),
                hash: hex32(&hash),
            }
        )
        .unwrap(),
        BlobOutcome::Bool { value: true }
    );
    assert_eq!(
        invoke(
            &mut *app,
            21,
            scope(|scope, peer| BlobOp::AllowReader { scope, peer }, hex32(&peer))
        )
        .unwrap(),
        BlobOutcome::Bool { value: true }
    );
    // The same decision the transport's blob server asks of the shared authorizer.
    assert!(door
        .custody
        .authorizer()
        .allows(BlobScope::new(SCOPE), &peer, BlobHash::from_bytes(hash)));
    assert_eq!(
        invoke(
            &mut *app,
            22,
            BlobOp::Allows {
                scope: hex32(&SCOPE),
                peer: hex32(&peer),
                hash: hex32(&hash),
            }
        )
        .unwrap(),
        BlobOutcome::Bool { value: true }
    );
    // Replacing readers with the same set is a no-op, as in process.
    assert_eq!(
        invoke(
            &mut *app,
            23,
            BlobOp::ReplaceReaders {
                scope: hex32(&SCOPE),
                peers: vec![hex32(&peer)],
            }
        )
        .unwrap(),
        BlobOutcome::Bool { value: false }
    );
    assert_eq!(
        invoke(
            &mut *app,
            24,
            BlobOp::DenyReader {
                scope: hex32(&SCOPE),
                peer: hex32(&peer),
            }
        )
        .unwrap(),
        BlobOutcome::Bool { value: true }
    );
    assert!(!door
        .custody
        .authorizer()
        .allows(BlobScope::new(SCOPE), &peer, BlobHash::from_bytes(hash)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_bad_upload_never_reaches_the_store() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut app = open(&door, "knot-editor").unwrap();
    bind(&mut *app, SCOPE, 1).unwrap();
    let declare = |len: u64, hash: [u8; 32]| BlobOp::PutBegin {
        scope: hex32(&SCOPE),
        namespace: "knot.evidence".into(),
        subject: hex(b"x"),
        len,
        hash: hex32(&hash),
    };

    // A hash that does not match the bytes.
    invoke(&mut *app, 2, declare(3, [0; 32])).unwrap();
    invoke(&mut *app, 3, BlobOp::PutPart { offset: 0, bytes: b"abc".to_vec() }).unwrap();
    let mismatch = invoke(&mut *app, 4, BlobOp::PutCommit).unwrap_err();
    assert!(mismatch.contains("declared hash"), "{mismatch}");

    // A part out of order, then past the declared length.
    invoke(&mut *app, 5, declare(3, *blake3::hash(b"abc").as_bytes())).unwrap();
    let out_of_order =
        invoke(&mut *app, 6, BlobOp::PutPart { offset: 1, bytes: b"bc".to_vec() }).unwrap_err();
    assert!(out_of_order.contains("not 1"), "{out_of_order}");
    let past = invoke(&mut *app, 7, BlobOp::PutPart { offset: 0, bytes: b"abcd".to_vec() }).unwrap_err();
    assert!(past.contains("past"), "{past}");

    // A second declaration while one is staged, a namespace the store rejects.
    assert!(invoke(&mut *app, 8, declare(3, [0; 32])).unwrap_err().contains("already staged"));
    invoke(&mut *app, 9, BlobOp::PutAbort).unwrap();
    let bad_namespace = invoke(
        &mut *app,
        10,
        BlobOp::PutBegin {
            scope: hex32(&SCOPE),
            namespace: "has space".into(),
            subject: hex(b"x"),
            len: 0,
            hash: hex32(&[0; 32]),
        },
    )
    .unwrap_err();
    assert!(bad_namespace.contains("namespace"), "{bad_namespace}");
    assert!(door
        .custody
        .blobs()
        .leased_hashes(BlobScope::new(SCOPE))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_migration_behaves_as_the_in_process_call_does() {
    let temp = tempfile::tempdir().unwrap();
    let legacy_root = temp.path().join("old.blobs");
    let legacy = transport::BlobStore::open(&legacy_root).await.unwrap();
    legacy.put_bytes_named(b"first".to_vec(), b"old/first").await.unwrap();
    legacy.put_bytes_named(b"second".to_vec(), b"old/second").await.unwrap();
    legacy.shutdown().await.unwrap();

    let door = BlobDoor::new(custody(&temp.path().join("data")).await);
    let mut app = open(&door, "knot-editor").unwrap();
    bind(&mut *app, SCOPE, 1).unwrap();
    let migrate = |namespace: &str, source: String| BlobOp::MigrateLegacy {
        scope: hex32(&SCOPE),
        namespace: namespace.into(),
        source,
    };
    let source = legacy_root.display().to_string();
    assert_eq!(
        invoke(&mut *app, 2, migrate("legacy.knot", source.clone())).unwrap(),
        BlobOutcome::Migration {
            state: MigrationState::Copied,
            blobs: 2
        }
    );
    assert_eq!(
        door.custody.blobs().leased_hashes(BlobScope::new(SCOPE)).await.unwrap().len(),
        2
    );
    assert_eq!(
        invoke(&mut *app, 3, migrate("legacy.knot", source.clone())).unwrap(),
        BlobOutcome::Migration {
            state: MigrationState::AlreadyComplete,
            blobs: 2
        }
    );
    let absent = temp.path().join("missing").display().to_string();
    assert_eq!(
        invoke(&mut *app, 4, migrate("legacy.knot", absent)).unwrap(),
        BlobOutcome::Migration {
            state: MigrationState::SourceAbsent,
            blobs: 0
        }
    );
    // The door's narrowings: a legacy namespace, an absolute path.
    assert!(invoke(&mut *app, 5, migrate("knot.evidence", source)).is_err());
    assert!(invoke(&mut *app, 6, migrate("legacy.knot", "relative/old.blobs".into())).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revocation_cuts_a_live_session_and_keeps_every_lease() {
    let temp = tempfile::tempdir().unwrap();
    let door = BlobDoor::new(custody(temp.path()).await);
    let mut app = open(&door, "knot-editor").unwrap();
    bind(&mut *app, SCOPE, 1).unwrap();
    let bytes = b"kept";
    put(&mut *app, 10, "knot.evidence", b"k", bytes).unwrap();
    let hash = *blake3::hash(bytes).as_bytes();
    let peer = [3u8; 32];
    invoke(&mut *app, 20, BlobOp::Retain { scope: hex32(&SCOPE), hash: hex32(&hash) }).unwrap();
    invoke(&mut *app, 21, BlobOp::AllowReader { scope: hex32(&SCOPE), peer: hex32(&peer) }).unwrap();
    let scope = BlobScope::new(SCOPE);
    assert!(door.custody.authorizer().allows(scope, &peer, BlobHash::from_bytes(hash)));

    door.revoke("knot-editor").await.unwrap();
    assert!(!app.is_live());
    assert!(app.snapshot(BlobEndpoint::request()).is_err());
    assert!(invoke(&mut *app, 22, BlobOp::PutAbort).is_err());
    assert!(open(&door, "knot-editor").is_err(), "the route refuses to open again");
    // Serving stopped; nothing was deleted.
    assert!(!door.custody.authorizer().allows(scope, &peer, BlobHash::from_bytes(hash)));
    assert!(door.custody.authorizer().readers(scope).is_empty());
    assert_eq!(door.custody.blobs().leased_hashes(scope).await.unwrap().len(), 1);
    assert_eq!(door.custody.scope_owner(scope).as_deref(), Some("knot-editor"));
    // Other applications are untouched.
    assert!(open(&door, "other-app").is_ok());

    // Restored, the application re-binds and finds its lease, as after a restart.
    door.restore("knot-editor");
    let mut again = open(&door, "knot-editor").unwrap();
    assert_eq!(bind(&mut *again, SCOPE, 1).unwrap(), BlobOutcome::Count { count: 1 });
}
