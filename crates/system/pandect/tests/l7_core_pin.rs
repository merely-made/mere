// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens D32: the session manifest pins the core pack's digest for
//! the journal, and pandect's baseline and checkpoint records pin it for
//! their snapshots. A stored digest other than the embedded core's still
//! reads through the embedded core, and records written before D32, with no
//! pin, still read.

mod support;

use std::time::UNIX_EPOCH;

use mere::dataset::View;
use mere::model::Author;
use mere::profile::core::{self, ids};
use muniment::{Backend, MemoryBackend};
use pandect::{MereSessions, SessionId};
use serde_json::Value;
use support::Surface;
use uuid::Uuid;

type Sessions = MereSessions<MemoryBackend, Surface>;

const KEYS: [&str; 3] = ["manifest.json", "baseline.json", "checkpoint.json"];

/// A session over a baseline, holding one cites edge, flushed and
/// checkpointed.
async fn cited_session(backend: &MemoryBackend) -> SessionId {
    let sessions = Sessions::new_with(backend.clone());
    let author = Author::person("l7-core-pin");
    let mut baseline = Surface::default();
    baseline.show(Uuid::from_u128(0x701), "https://pin.test/baseline");
    let mut session = sessions.begin_with(author.clone(), Some(baseline));
    let id = session.id();
    let (a, b) = (Uuid::from_u128(0x702), Uuid::from_u128(0x703));
    session
        .edit_now(author, |graph| {
            graph.show(a, "https://pin.test/a");
            graph.show(b, "https://pin.test/b");
            graph.assert(a, b, core::iri(ids::REL_CITES));
        })
        .unwrap();
    session.flush(UNIX_EPOCH).await.unwrap();
    session.checkpoint().await.unwrap();
    id
}

fn key(id: SessionId, file: &str) -> String {
    format!("sessions/{}/{file}", id.as_uuid())
}

async fn stored(backend: &MemoryBackend, id: SessionId, file: &str) -> Value {
    let bytes = backend.get(&key(id, file)).await.unwrap().expect(file);
    serde_json::from_slice(&bytes).unwrap()
}

/// Rewrite each record's pin: `Some` sets it, `None` removes it.
async fn repin(backend: &MemoryBackend, id: SessionId, pin: Option<&str>) {
    for file in KEYS {
        let mut record = stored(backend, id, file).await;
        let fields = record.as_object_mut().unwrap();
        match pin {
            Some(pin) => fields.insert("core_pack".into(), pin.into()),
            None => fields.remove("core_pack"),
        };
        let bytes = serde_json::to_vec_pretty(&record).unwrap();
        backend.put(&key(id, file), &bytes).await.unwrap();
    }
}

/// Whether the reopened session still holds the cites edge.
async fn cites(backend: &MemoryBackend, id: SessionId) -> bool {
    let session = Sessions::new_with(backend.clone())
        .open_with(id)
        .await
        .unwrap();
    let pin = session.pin(session.journal().live_cursor()).unwrap();
    pin.dataset()
        .quads(View::Live)
        .any(|q| q.predicate.as_str() == core::iri(ids::REL_CITES))
}

#[test]
fn a_session_pins_the_core_pack_in_its_manifest_baseline_and_checkpoint() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let id = cited_session(&backend).await;
        assert!(core::digest().starts_with("blake3:"), "{}", core::digest());
        for file in KEYS {
            assert_eq!(
                stored(&backend, id, file).await["core_pack"],
                core::digest(),
                "{file}"
            );
        }
        assert!(cites(&backend, id).await);
    });
}

#[test]
fn another_core_s_pin_and_no_pin_both_read_through_the_embedded_core() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let id = cited_session(&backend).await;
        let other = format!("blake3:{}", "0".repeat(64));
        repin(&backend, id, Some(&other)).await;
        assert_eq!(
            stored(&backend, id, "manifest.json").await["core_pack"],
            other
        );
        assert!(
            cites(&backend, id).await,
            "a digest other than the embedded core's"
        );
        repin(&backend, id, None).await;
        assert!(
            stored(&backend, id, "manifest.json")
                .await
                .get("core_pack")
                .is_none()
        );
        assert!(cites(&backend, id).await, "records written before D32");
    });
}
