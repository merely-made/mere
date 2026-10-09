// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use kernel::graph::{
    CoverageLayer, Graph, SemanticStatementSpec,
    apply::{self, GraphDelta, apply_graph_delta},
};
use kernel::persistence::PersistedResourceFacet;
use muniment::Backend;
#[cfg(not(target_arch = "wasm32"))]
use muniment::MemoryBackend;
use pandect::addressable_graph::*;
use serde_json::json;
use uuid::Uuid;

fn source(count: usize) -> (Graph, Vec<Uuid>, Vec<Uuid>) {
    let mut graph = Graph::new();
    let mut surfaces = vec![];
    let mut resources = vec![];
    for i in 0..count {
        let id = Uuid::from_u128(i as u128 + 1);
        let key = apply::add_node(
            &mut graph,
            Some(id),
            format!("https://residency.test/{i}"),
            Default::default(),
        );
        assert!(matches!(
            apply_graph_delta(
                &mut graph,
                GraphDelta::SetNodeBody {
                    key,
                    body: Some(format!("body-{i}-{}", "x".repeat(8192)))
                }
            ),
            apply::GraphDeltaResult::NodeMetadataUpdated(true)
        ));
        if i == 1 {
            let parent = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
            apply_graph_delta(&mut graph, GraphDelta::BranchHistory { child: key, parent });
        }
        for tick in [10, 20] {
            apply_graph_delta(
                &mut graph,
                GraphDelta::ReplayNavigateNodeById {
                    node_id: id,
                    url: format!("https://residency.test/{i}"),
                    transition: kernel::graph::NodeHistoryTransitionKind::UrlTyped,
                    timestamp_ms: tick + i as u64 * 100,
                    last_session_visited: 0,
                },
            );
        }
        let resource = graph.shown_resource_id(key).unwrap();
        let mut record = graph.resource_record(resource).unwrap();
        record.facets.push(PersistedResourceFacet {
            facet: "test.payload".into(),
            value_json: json!(format!("resource-{i}-{}", "y".repeat(8192))).to_string(),
        });
        assert!(matches!(
            apply_graph_delta(
                &mut graph,
                GraphDelta::ReplaySetResourceRecordById {
                    resource_id: resource,
                    record: Some(record)
                }
            ),
            apply::GraphDeltaResult::Applied
        ));
        graph
            .facets_mut()
            .set(
                id,
                chartulary::FacetId::new("reader.highlight"),
                json!({"value":i}),
                &chartulary::AcceptAll,
            )
            .unwrap();
        surfaces.push(id);
        resources.push(resource);
    }
    for pair in resources.windows(2) {
        graph
            .try_assert_semantic_statement_by_resource_ids(
                pair[0],
                pair[1],
                SemanticStatementSpec {
                    predicate: "urn:test:next".into(),
                    label: Some("exact retained claim".into()),
                    provenance_iri: Some("urn:test:author".into()),
                    asserted_at_ms: Some(123),
                    graph_scope: kernel::types::GraphScope::Custom("urn:test:scope".into()),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    (graph, surfaces, resources)
}
fn roots(id: Uuid) -> ResidencyRoots {
    ResidencyRoots {
        projection_surfaces: [id].into(),
        ..Default::default()
    }
}

async fn neighborhood_controls<B: Backend + Clone>(backend: B) {
    let (source, surfaces, resources) = source(64);
    let expected = source.to_snapshot_at(0);
    let facets = source.facets().clone();
    let store = AddressableGraphStore::new(backend.clone(), Uuid::from_u128(700));
    store.write_recorded_graph(&source).await.unwrap();
    store
        .set_policy(ResidencyPolicy::Neighborhood { hops: 1 })
        .await
        .unwrap();
    drop(source);
    let mut view = store.open(roots(surfaces[0])).await.unwrap();
    assert_eq!(view.graph().nodes().count(), 2);
    assert_eq!(view.graph().resource_nodes().count(), 2);
    assert_eq!(
        view.status(GraphAddress::Resource(resources[2])),
        ResidencyStatus::NotLoaded
    );
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[2])),
        ResidencyStatus::NotLoaded
    );
    assert_eq!(
        view.status(GraphAddress::Resource(Uuid::from_u128(900))),
        ResidencyStatus::Absent
    );
    let note = view.graph().coverage_for_resource_refs(&[resources[2]]);
    assert!(
        note.limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency && l.resources.contains(&resources[2]))
    );
    assert!(
        !note
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession)
    );
    assert_eq!(
        view.demand(GraphAddress::Resource(resources[2]))
            .await
            .unwrap(),
        ResidencyStatus::Loaded
    );
    let held: std::collections::BTreeSet<_> = view
        .graph()
        .resource_nodes()
        .map(|r| r.id().to_string())
        .collect();
    let rows: Vec<_> = expected
        .resource_edges
        .iter()
        .filter(|e| held.contains(&e.from_node_id) && held.contains(&e.to_node_id))
        .cloned()
        .collect();
    assert_eq!(view.graph().to_snapshot_at(0).resource_edges, rows);
    let before = view.graph().revision();
    view.reconcile(roots(surfaces[30])).await.unwrap();
    assert_eq!(view.graph().nodes().count(), 3);
    assert_eq!(
        view.status(GraphAddress::Resource(resources[0])),
        ResidencyStatus::NotLoaded
    );
    assert!(
        view.graph().revision() > before,
        "residency swaps invalidate available-data caches"
    );
    let partial = view.footprint();
    assert!(
        AddressableGraphStore::new(backend.clone(), Uuid::from_u128(700))
            .write_recorded_graph(view.graph())
            .await
            .is_err(),
        "partial projection must not erase evicted truth"
    );
    view.set_policy(ResidencyPolicy::Full).await.unwrap();
    assert!(
        !view
            .graph()
            .coverage_note()
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency)
    );
    assert_eq!(
        view.graph().to_snapshot_at(0).nodes.len(),
        expected.nodes.len()
    );
    assert_eq!(view.graph().to_snapshot_at(0).resources, expected.resources);
    assert_eq!(
        view.graph().to_snapshot_at(0).resource_edges,
        expected.resource_edges
    );
    assert_eq!(
        view.graph().to_snapshot_at(0).navigation,
        expected.navigation
    );
    assert_eq!(view.graph().facets(), &facets);
    assert!(partial.retained_record_bytes() * 3 < view.footprint().retained_record_bytes());
    // The persisted Full setting is read by a newly opened view.
    drop(view);
    let view = AddressableGraphStore::new(backend, Uuid::from_u128(700))
        .open(Default::default())
        .await
        .unwrap();
    assert_eq!(view.graph().nodes().count(), expected.nodes.len());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn memory_neighborhood_demand_full_and_no_partial_overwrite() {
    pollster::block_on(neighborhood_controls(MemoryBackend::new()));
}

async fn session_controls<B: Backend + Clone>(backend: B) {
    use pandect::graph_session::MereSessions;
    let sessions = MereSessions::new(backend.clone());
    let (graph, surfaces, resources) = source(32);
    let mut session = sessions.begin_recorded(
        kernel::graph::Author::person("resident-author"),
        Some(graph),
    );
    let id = session.id();
    let graph_id = *session.manifest().root_graph_id.as_uuid();
    let generation = session.publish_resident_checkpoint().await.unwrap();
    AddressableGraphStore::new(backend.clone(), graph_id)
        .set_policy(ResidencyPolicy::Neighborhood { hops: 0 })
        .await
        .unwrap();
    drop(session);
    // Production reader takes the manifest and addressable records, never a full
    // baseline/replay. Deliberately remove those files to demonstrate the seam.
    let prefix = format!("sessions/{}/", id.as_uuid());
    for key in backend.list(&prefix).await.unwrap() {
        if !key.ends_with("manifest.json") {
            backend.delete(&key).await.unwrap();
        }
    }
    let view = sessions
        .open_resident_graph(id, roots(surfaces[0]))
        .await
        .unwrap();
    assert_eq!(view.generation(), generation);
    assert_eq!(view.graph().nodes().count(), 1);
    assert_eq!(
        view.status(GraphAddress::Resource(resources[20])),
        ResidencyStatus::NotLoaded
    );
    let mut legacy = sessions.begin(kernel::graph::Author::person("legacy"), Some(Graph::new()));
    assert!(legacy.publish_resident_checkpoint().await.is_err());
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn recorded_session_manifest_only_reader_and_legacy_refusal() {
    pollster::block_on(session_controls(MemoryBackend::new()));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn redb_real_store_controls() {
    let temp = tempfile::tempdir().unwrap();
    pollster::block_on(async {
        let backend = muniment::RedbBackend::open(temp.path().join("graph.redb")).unwrap();
        neighborhood_controls(backend.clone()).await;
        session_controls(backend.clone()).await;
        identity_and_refusal_controls(backend.clone()).await;
        read_footprint_controls(backend).await;
        let reopened = muniment::RedbBackend::open(temp.path().join("graph.redb")).unwrap();
        let view = AddressableGraphStore::new(reopened, Uuid::from_u128(700))
            .open(Default::default())
            .await
            .unwrap();
        assert_eq!(view.graph().nodes().count(), 64);
    });
}
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn indexeddb_real_store_controls() {
    let name = format!("graph-residency-{}", Uuid::new_v4());
    let backend = muniment::IndexedDbBackend::open(&name, "records")
        .await
        .unwrap();
    neighborhood_controls(backend.clone()).await;
    session_controls(backend.clone()).await;
    identity_and_refusal_controls(backend.clone()).await;
    read_footprint_controls(backend).await;
    let reopened = muniment::IndexedDbBackend::open(&name, "records")
        .await
        .unwrap();
    let view = AddressableGraphStore::new(reopened, Uuid::from_u128(700))
        .open(Default::default())
        .await
        .unwrap();
    assert_eq!(view.graph().nodes().count(), 64);
}

async fn identity_and_refusal_controls<B: Backend + Clone>(backend: B) {
    let (mut graph, surfaces, resources) = source(24);
    let overlap = resources[5];
    apply::add_node(
        &mut graph,
        Some(overlap),
        "https://residency.test/0".into(),
        Default::default(),
    );
    let pinned = graph.get_node_key_by_id(surfaces[18]).unwrap();
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodePinned {
            key: pinned,
            is_pinned: true,
        },
    );
    let author = kernel::graph::Author::person("schema-author");
    for (predicate, layer) in [
        (
            "urn:test:surface-custom",
            kernel::graph::GraphStratum::Surface,
        ),
        (
            "urn:test:resource-custom",
            kernel::graph::GraphStratum::Resource,
        ),
    ] {
        graph
            .write_as(author.clone(), |g| g.declare_predicate(predicate, layer))
            .unwrap();
    }
    let a = graph.get_node_key_by_id(surfaces[2]).unwrap();
    let b = graph.get_node_key_by_id(surfaces[3]).unwrap();
    assert!(
        apply::assert_semantic_predicate_in_scope(
            &mut graph,
            a,
            b,
            "urn:test:surface-custom".into(),
            kernel::types::GraphScope::Custom("surface-scope".into())
        )
        .is_some()
    );
    // Parallel scope/provenance variants and a self relation must retain their
    // original ordinal copies and handles through individual demand/full reads.
    for (a, b, scope) in [
        (resources[5], resources[6], "parallel"),
        (resources[5], resources[5], "self"),
    ] {
        graph
            .try_assert_semantic_statement_by_resource_ids(
                a,
                b,
                SemanticStatementSpec {
                    predicate: "urn:test:next".into(),
                    graph_scope: kernel::types::GraphScope::Custom(scope.into()),
                    provenance_iri: Some("urn:second-author".into()),
                    asserted_at_ms: Some(77),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let frozen = graph
        .freeze_resource_selection_with_nonce(
            json!({"kind":"portable"}),
            vec![resources[5], resources[6]],
            Uuid::from_u128(880),
        )
        .unwrap();
    let frozen_record = graph.resource_record(frozen).unwrap();
    let expected = graph.to_snapshot_at(0);
    let expected_facets = graph.facets().clone();
    let id = Uuid::from_u128(701);
    let store = AddressableGraphStore::new(backend.clone(), id);
    store.write_recorded_graph(&graph).await.unwrap();
    store
        .set_policy(ResidencyPolicy::Neighborhood { hops: 0 })
        .await
        .unwrap();
    let mut root = roots(overlap);
    root.focused = Some(GraphAddress::Resource(resources[15]));
    root.pinned.insert(GraphAddress::Surface(surfaces[20]));
    let mut view = store.open(root).await.unwrap();
    assert_eq!(
        view.status(GraphAddress::Surface(overlap)),
        ResidencyStatus::Loaded
    );
    assert_eq!(
        view.status(GraphAddress::Resource(overlap)),
        ResidencyStatus::NotLoaded
    );
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[0])),
        ResidencyStatus::Loaded,
        "co-shown aliases follow the zero-hop binding"
    );
    for i in [18, 20] {
        assert_eq!(
            view.status(GraphAddress::Surface(surfaces[i])),
            ResidencyStatus::Loaded
        );
    }
    assert_eq!(
        view.status(GraphAddress::Resource(resources[15])),
        ResidencyStatus::Loaded
    );
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[15])),
        ResidencyStatus::NotLoaded,
        "focused Resource does not invent a Surface dependency"
    );
    assert_eq!(
        view.graph()
            .effective_predicate_stratum("urn:test:surface-custom")
            .unwrap(),
        kernel::graph::GraphStratum::Surface
    );
    assert_eq!(
        view.graph()
            .effective_predicate_stratum("urn:test:resource-custom")
            .unwrap(),
        kernel::graph::GraphStratum::Resource
    );
    assert_eq!(
        view.demand(GraphAddress::Resource(frozen)).await.unwrap(),
        ResidencyStatus::Loaded
    );
    assert_eq!(view.graph().resource_record(frozen).unwrap(), frozen_record);
    assert!(
        view.graph()
            .open_frozen_selection(frozen)
            .unwrap()
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency && l.resources.contains(&resources[5]))
    );
    view.demand(GraphAddress::Resource(resources[5]))
        .await
        .unwrap();
    view.demand(GraphAddress::Resource(resources[6]))
        .await
        .unwrap();
    let held: std::collections::BTreeSet<_> = view
        .graph()
        .resource_nodes()
        .map(|r| r.id().to_string())
        .collect();
    let rows: Vec<_> = expected
        .resource_edges
        .iter()
        .filter(|e| held.contains(&e.from_node_id) && held.contains(&e.to_node_id))
        .cloned()
        .collect();
    assert_eq!(view.graph().to_snapshot_at(0).resource_edges, rows);
    view.demand(GraphAddress::Surface(surfaces[2]))
        .await
        .unwrap();
    view.demand(GraphAddress::Surface(surfaces[3]))
        .await
        .unwrap();
    assert_eq!(view.graph().to_snapshot_at(0).edges, expected.edges);
    // Reconciliation computes the union of all open projection neighborhoods.
    let mut several = roots(surfaces[2]);
    several.projection_surfaces.insert(surfaces[10]);
    view.reconcile(several).await.unwrap();
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[2])),
        ResidencyStatus::Loaded
    );
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[10])),
        ResidencyStatus::Loaded
    );
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[3])),
        ResidencyStatus::NotLoaded
    );
    view.set_policy(ResidencyPolicy::Neighborhood { hops: 1 })
        .await
        .unwrap();
    assert_eq!(
        view.status(GraphAddress::Surface(surfaces[3])),
        ResidencyStatus::Loaded,
        "Surface relation costs one hop too"
    );
    view.set_policy(ResidencyPolicy::Neighborhood { hops: 0 })
        .await
        .unwrap();
    let before = serde_json::to_value(view.graph().to_snapshot_at(0)).unwrap();
    let revision = view.graph().revision();
    assert!(view.reconcile(roots(Uuid::nil())).await.is_err());
    assert_eq!(
        serde_json::to_value(view.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    assert_eq!(view.graph().revision(), revision);
    let generation = view.generation().to_string();
    // Missing and damaged addressed payloads are failures, never Absent. Neither
    // a failed demand nor a failed setting change publishes a candidate view.
    let prefix = format!("graphs/{id}/resident/v1/nodes/resource/{}/", resources[12]);
    let key = backend.list(&prefix).await.unwrap().pop().unwrap();
    let bytes = backend.get(&key).await.unwrap().unwrap();
    backend.delete(&key).await.unwrap();
    assert!(
        view.demand(GraphAddress::Resource(resources[12]))
            .await
            .is_err()
    );
    assert_eq!(
        view.status(GraphAddress::Resource(resources[12])),
        ResidencyStatus::NotLoaded
    );
    backend.put(&key, b"corrupt").await.unwrap();
    assert!(view.set_policy(ResidencyPolicy::Full).await.is_err());
    assert_eq!(view.policy(), ResidencyPolicy::Neighborhood { hops: 0 });
    assert_eq!(view.generation(), generation);
    assert_eq!(
        serde_json::to_value(view.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    backend.put(&key, &bytes).await.unwrap();
    let key0 = graph.get_node_key_by_id(surfaces[2]).unwrap();
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodeBody {
            key: key0,
            body: Some("new checkpoint body".into()),
        },
    );
    let new_generation = AddressableGraphStore::new(backend.clone(), id)
        .write_recorded_graph(&graph)
        .await
        .unwrap();
    assert_ne!(new_generation, generation);
    assert_eq!(view.generation(), generation);
    view.refresh().await.unwrap();
    assert_eq!(view.generation(), new_generation);
    assert!(view.graph().revision() > revision);
    let unchanged = view.graph().revision();
    view.refresh().await.unwrap();
    assert_eq!(
        view.graph().revision(),
        unchanged,
        "unchanged read context must not rederive graph caches"
    );
    view.set_policy(ResidencyPolicy::Full).await.unwrap();
    assert_eq!(
        serde_json::to_value(view.graph().to_snapshot_at(0)).unwrap(),
        serde_json::to_value(graph.to_snapshot_at(0)).unwrap()
    );
    assert_eq!(view.graph().facets(), &expected_facets);
    // A fresh reader uses the persisted Full policy and retains frozen payloads.
    let mut fresh = AddressableGraphStore::new(backend.clone(), id)
        .open(Default::default())
        .await
        .unwrap();
    assert_eq!(
        fresh.graph().resource_record(frozen).unwrap(),
        frozen_record
    );
    fresh
        .set_policy(ResidencyPolicy::Neighborhood { hops: 0 })
        .await
        .unwrap();
    assert_eq!(
        fresh.status(GraphAddress::Resource(resources[5])),
        ResidencyStatus::NotLoaded
    );
    // Empty graphs remain a valid Full control.
    let empty = AddressableGraphStore::new(backend, Uuid::from_u128(702));
    empty.write_recorded_graph(&Graph::new()).await.unwrap();
    let empty = empty.open(Default::default()).await.unwrap();
    assert_eq!(empty.graph().nodes().count(), 0);
    assert!(empty.graph().coverage_note().limits.is_empty());
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn memory_identity_context_corruption_and_refresh() {
    pollster::block_on(identity_and_refusal_controls(MemoryBackend::new()));
}

#[cfg(not(target_arch = "wasm32"))]
trait ProbeBackend: Backend + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<B: Backend + Sync> ProbeBackend for B {}
#[cfg(target_arch = "wasm32")]
trait ProbeBackend: Backend {}
#[cfg(target_arch = "wasm32")]
impl<B: Backend> ProbeBackend for B {}
#[derive(Clone)]
struct ReadProbe<B> {
    backend: B,
    reads: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    bytes: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl<B> ReadProbe<B> {
    fn new(backend: B) -> Self {
        Self {
            backend,
            reads: Default::default(),
            bytes: Default::default(),
        }
    }
    fn reset(&self) {
        self.reads.store(0, std::sync::atomic::Ordering::SeqCst);
        self.bytes.store(0, std::sync::atomic::Ordering::SeqCst);
    }
    fn counts(&self) -> (usize, usize) {
        (
            self.reads.load(std::sync::atomic::Ordering::SeqCst),
            self.bytes.load(std::sync::atomic::Ordering::SeqCst),
        )
    }
}
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch="wasm32",async_trait::async_trait(?Send))]
impl<B: ProbeBackend> Backend for ReadProbe<B> {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, muniment::StoreError> {
        let data = self.backend.get(key).await?;
        if key.contains("/nodes/") {
            self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        self.bytes.fetch_add(
            data.as_ref().map_or(0, Vec::len),
            std::sync::atomic::Ordering::SeqCst,
        );
        Ok(data)
    }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), muniment::StoreError> {
        self.backend.put(key, bytes).await
    }
    async fn delete(&self, key: &str) -> Result<(), muniment::StoreError> {
        self.backend.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, muniment::StoreError> {
        self.backend.list(prefix).await
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, muniment::StoreError> {
        self.backend.scan(start, end).await
    }
    async fn apply(&self, ops: &[muniment::WriteOp]) -> Result<(), muniment::StoreError> {
        self.backend.apply(ops).await
    }
}
async fn read_footprint_controls<B: ProbeBackend + Clone>(backend: B) {
    let probe = ReadProbe::new(backend);
    let (graph, surfaces, _) = source(64);
    let store = AddressableGraphStore::new(probe.clone(), Uuid::from_u128(703));
    store.write_recorded_graph(&graph).await.unwrap();
    store
        .set_policy(ResidencyPolicy::Neighborhood { hops: 1 })
        .await
        .unwrap();
    drop(graph);
    probe.reset();
    let mut view = store.open(roots(surfaces[0])).await.unwrap();
    let (reads, partial_bytes) = probe.counts();
    let partial = view.footprint().retained_record_bytes();
    assert_eq!(
        reads, 4,
        "only two addressed Surface/Resource pairs, not a full payload scan"
    );
    probe.reset();
    view.set_policy(ResidencyPolicy::Full).await.unwrap();
    let (full_reads, full_bytes) = probe.counts();
    assert_eq!(full_reads, 124, "Full fetches the remaining 62 pairs");
    assert!(partial_bytes * 3 < full_bytes);
    assert!(partial * 3 < view.footprint().retained_record_bytes());
    println!(
        "resident records={reads}/128; read bytes partial={partial_bytes}, full addition={full_bytes}; retained bytes partial={partial}, full={}",
        view.footprint().retained_record_bytes()
    );
}
#[cfg(not(target_arch = "wasm32"))]
mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    std::thread_local! {static LIVE:Cell<isize>=const {Cell::new(0)};}
    pub(super) fn live() -> isize {
        LIVE.with(Cell::get)
    }
    fn account(change: isize) {
        let _ = LIVE.try_with(|live| live.set(live.get() + change));
    }
    pub(super) struct Counting;
    // Integration-test instrumentation only. System owns every allocation;
    // thread-local accounting isolates this view from concurrent tests.
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let p = unsafe { System.alloc(layout) };
            if !p.is_null() {
                account(layout.size() as isize);
            }
            p
        }
        unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
            account(-(layout.size() as isize));
            unsafe { System.dealloc(p, layout) }
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn live_allocation_scoped_partial_vs_full() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let (graph, surfaces, _) = source(128);
        let id = Uuid::from_u128(704);
        AddressableGraphStore::new(backend.clone(), id)
            .write_recorded_graph(&graph)
            .await
            .unwrap();
        drop(graph);
        AddressableGraphStore::new(backend.clone(), id)
            .set_policy(ResidencyPolicy::Neighborhood { hops: 1 })
            .await
            .unwrap();
        let baseline = allocation::live();
        let partial = AddressableGraphStore::new(backend.clone(), id)
            .open(roots(surfaces[0]))
            .await
            .unwrap();
        let partial_live = allocation::live() - baseline;
        assert_eq!(partial.graph().nodes().count(), 2);
        drop(partial);
        AddressableGraphStore::new(backend.clone(), id)
            .set_policy(ResidencyPolicy::Full)
            .await
            .unwrap();
        let baseline = allocation::live();
        let full = AddressableGraphStore::new(backend, id)
            .open(Default::default())
            .await
            .unwrap();
        let full_live = allocation::live() - baseline;
        assert_eq!(full.graph().nodes().count(), 128);
        println!("additional live allocation partial={partial_live}, full={full_live}");
        assert!(
            partial_live > 0 && partial_live * 3 < full_live,
            "partial={partial_live}, full={full_live}"
        );
    });
}
