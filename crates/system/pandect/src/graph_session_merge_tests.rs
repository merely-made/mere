// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Session merge-in over the shared test surface (engine split D272): a part
//! picked before the pick record merges as a pick, and an operation's lost
//! acknowledgement reconciles. The rest of the merge suite runs over
//! incipit-graph in Graphshell's incipit-session.

use super::*;
use crate::test_surface::{COPIED_FROM, Delta, PUBLICATION, Surface, show_edits};
use muniment::MemoryBackend;

type Sessions<B> = MereSessions<B, Surface>;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn other() -> Author {
    Author::person("persona-b").via("knot-editor")
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn add(n: u128) -> Vec<Delta> {
    show_edits(id(n), &format!("https://{n}.test/")).1
}

fn retitle(node: Uuid, title: &str) -> Vec<Delta> {
    show_edits(node, &format!("https://{title}.test/")).1
}

fn title(graph: &Surface, node: Uuid) -> Option<String> {
    let iri = graph.iri_of(node)?;
    Some(
        iri.trim_start_matches("https://")
            .trim_end_matches(".test/")
            .to_owned(),
    )
}

/// A session holding surfaces 1 and 2 related, and a lone surface 3.
async fn parent(mere: &Sessions<MemoryBackend>) -> GraphSession<MemoryBackend, Surface> {
    let made = mere.mint(person(), Some("Reading".into())).await.unwrap();
    let mut parent = mere.open_with(made.session_id).await.unwrap();
    let mut edits = add(1);
    edits.extend(add(2));
    edits.extend(add(3));
    edits.extend(retitle(id(1), "one"));
    edits.extend(retitle(id(2), "two"));
    parent.apply(person(), edits).await.unwrap();
    parent
        .edit_now(person(), |graph| {
            graph.assert(id(1), id(2), "https://merge.test/vocab#cites");
        })
        .unwrap();
    parent.flush(wall_clock_now()).await.unwrap();
    parent
}

#[test]
fn a_part_picked_before_the_pick_record_merges_as_a_pick() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let mut target = parent(&mere).await;
        let container = target.manifest().root_graph_id;
        // The host copies surface 1 under a new id and names its origin.
        let one = id(0x101);
        let mut copied = Surface::default();
        copied.show(one, "https://one.test/");
        copied.set_facet(
            one,
            COPIED_FROM,
            serde_json::json!({"origin": id(1), "container": container.as_uuid()}),
        );
        let part = mere
            .pick(&target, container, copied, person())
            .await
            .unwrap();
        // Rewrite its manifest as parts were written before `picked_from`.
        let mut legacy = part.clone();
        let picked = legacy.picked_from.take().unwrap();
        legacy.parent_session = Some(picked.session);
        legacy.forked_at = Some(picked.at);
        store
            .apply(&[pretty(Keys::new(legacy.session_id).at(MANIFEST), &legacy).unwrap()])
            .await
            .unwrap();
        let mut part = mere.open_with(part.session_id).await.unwrap();
        assert_eq!(part.manifest().picked_from, None);
        part.apply(other(), retitle(one, "legacy-part"))
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &part).await.unwrap();
        assert_eq!(plan.matched(), MergeMatch::Pick);
        mere.apply_merge(&mut target, &mut part, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        assert_eq!(title(target.graph(), id(1)).as_deref(), Some("legacy-part"));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn operation_storage_refusal_and_lost_ack_reconcile() {
    use std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    };
    #[derive(Clone)]
    struct FaultBackend {
        store: MemoryBackend,
        fault: Arc<AtomicU8>,
    }
    #[async_trait::async_trait]
    impl Backend for FaultBackend {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            self.store.get(key).await
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            self.store.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.store.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.store.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.store.scan(start, end).await
        }
        async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
            self.store.apply(ops).await
        }
        async fn transact(&self, transaction: muniment::TransactFn) -> Result<(), StoreError> {
            let fault = self.fault.swap(0, Ordering::SeqCst);
            if fault == 1 {
                return Err(StoreError::NotTransactional);
            }
            self.store.transact(transaction).await?;
            if fault == 2 {
                return Err(StoreError::NotTransactional);
            }
            Ok(())
        }
    }
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone()).with_publication(&PUBLICATION);
        let session = parent(&mere).await.id();
        let fault = Arc::new(AtomicU8::new(1));
        let backend = FaultBackend {
            store,
            fault: fault.clone(),
        };
        let faulty = Sessions::new_with(backend.clone()).with_publication(&PUBLICATION);
        let mut writer = faulty.open_with(session).await.unwrap();
        let mut pending = faulty.open_with(session).await.unwrap();
        let mut rejected = faulty
            .open_with(session)
            .await
            .unwrap()
            .with_validator(Arc::new(|graph: &Surface| {
                if graph.shown(id(79)).is_some() {
                    Err("replayed graph rejected".into())
                } else {
                    Ok(())
                }
            }))
            .unwrap();
        let make_deltas = || {
            add(79)
                .iter()
                .map(|delta| Surface::edit_of(delta, &person()).unwrap())
                .collect::<Vec<_>>()
        };
        assert!(
            writer
                .transact_operation_published(
                    [20; 32],
                    b"same".to_vec(),
                    b"result".to_vec(),
                    person(),
                    make_deltas(),
                    wall_clock_now()
                )
                .await
                .is_err()
        );
        assert!(writer.graph().shown(id(79)).is_none());
        assert!(writer.operation_receipt([20; 32]).await.unwrap().is_none());
        assert!(
            writer
                .reconcile_operation([20; 32])
                .await
                .unwrap()
                .is_none()
        );
        fault.store(2, Ordering::SeqCst);
        assert!(
            writer
                .transact_operation_published(
                    [20; 32],
                    b"same".to_vec(),
                    b"result".to_vec(),
                    person(),
                    make_deltas(),
                    wall_clock_now()
                )
                .await
                .is_err()
        );
        assert!(writer.graph().shown(id(79)).is_none());
        let old_cursor = writer.journal().live_cursor();

        // A reload must never discard local unsaved work, even when the receipt
        // is present in durable storage.
        pending
            .edit_now(person(), |graph| {
                graph.show(id(1), "https://unsaved.test/");
            })
            .unwrap();
        assert!(pending.reconcile_operation([20; 32]).await.is_err());
        assert_eq!(title(pending.graph(), id(1)).as_deref(), Some("unsaved"));
        assert!(pending.graph().shown(id(79)).is_none());

        // Product validation rejects a reconstructed graph without replacing the
        // previously held graph, history or settings.
        assert!(rejected.reconcile_operation([20; 32]).await.is_err());
        assert!(rejected.graph().shown(id(79)).is_none());
        assert_eq!(rejected.journal().live_cursor(), old_cursor);

        writer.set_current_session(43);
        writer.set_checkpoint_interval(37);
        let validation: GraphValidator<Surface> = Arc::new(|_| Ok(()));
        let admission: GraphAdmissionValidator<Surface> =
            Arc::new(|_, _| Err("new admission refused".into()));
        writer = writer
            .with_validator(validation.clone())
            .unwrap()
            .with_admission_validator(admission.clone());
        let (reconciled, current_generation) =
            writer.reconcile_operation([20; 32]).await.unwrap().unwrap();
        assert_eq!(writer.graph().surface_count(), 4);
        assert!(writer.graph().shown(id(79)).is_some());
        assert!(writer.journal().live_cursor() > old_cursor);
        assert_eq!(writer.graph().current_session(), 43);
        assert_eq!(writer.checkpoint_interval, 37);
        assert!(Arc::ptr_eq(writer.validator.as_ref().unwrap(), &validation));
        assert!(Arc::ptr_eq(
            writer.admission_validator.as_ref().unwrap(),
            &admission
        ));
        assert_eq!(current_generation, reconciled.generation);
        // Reconciliation does not consume a second operation or repeat a graph edit.
        let current_cursor = writer.journal().live_cursor();
        assert_eq!(
            writer
                .reconcile_operation([20; 32])
                .await
                .unwrap()
                .unwrap()
                .0,
            reconciled
        );
        assert_eq!(writer.journal().live_cursor(), current_cursor);

        let mut reopened = faulty.open_with(session).await.unwrap();
        let receipt = reopened.operation_receipt([20; 32]).await.unwrap().unwrap();
        assert_eq!(receipt, reconciled);
        assert!(reopened.graph().shown(id(79)).is_some());
        let retry = reopened
            .transact_operation_published(
                [20; 32],
                b"same".to_vec(),
                b"result".to_vec(),
                person(),
                make_deltas(),
                wall_clock_now(),
            )
            .await
            .unwrap();
        assert_eq!(receipt, retry);
        assert_eq!(reopened.graph().surface_count(), 4);
    });
}
