// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Codicils over the D245 surface (engine split D265, D266): saved and opened
//! through an installed hook naming the surface's own schemas.

mod support;

use eidetic::Timestamp;
use mere::model::{Author, ResourceNode};
use mere::session::SessionGraph;
use muniment::MemoryBackend;
use pandect::graph_codicil::{
    CodicilHook, KERNEL_CODICIL, RedactionPolicy, list_graph_codicils_with,
    load_profiled_graph_codicil_sealed_with, open_codicil_as_session_sealed_with,
    save_graph_snapshot_codicil_sealed_with,
};
use pandect::{MereSessions, SessionError};
use uuid::Uuid;

use support::Surface;

static SURFACE: CodicilHook<Surface> = CodicilHook {
    graph_schema: b"p8.surface-snapshot/v1",
    profiled_schema: b"p8.surface-snapshot-profiled/v1",
    v3_profiled_schema: b"p8.surface-snapshot/v0-profiled",
    v2_schema: b"p8.surface-snapshot/v0-facets",
    v1_schema: b"p8.surface-snapshot/v0",
    redact: |_, _| {},
};

#[test]
fn a_codicil_saves_and_opens_through_an_installed_hook() {
    pollster::block_on(async {
        let mut store = MemoryBackend::new();
        let mut graph = Surface::default();
        let shown = Uuid::from_u128(0xf1);
        graph.show(shown, "https://p8.test/codicil");
        let id = save_graph_snapshot_codicil_sealed_with(
            &mut store,
            None,
            &SURFACE,
            graph.to_snapshot(),
            graph.facets().clone(),
            RedactionPolicy::default(),
            Timestamp(1),
        )
        .await
        .unwrap();
        let opened = open_codicil_as_session_sealed_with(&mut store, None, &SURFACE, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(opened.to_snapshot(), graph.to_snapshot());
        let listed = list_graph_codicils_with(&mut store, &SURFACE)
            .await
            .unwrap();
        assert_eq!(listed.iter().map(|m| m.id).collect::<Vec<_>>(), [id]);

        // A session opens from it through the sessions' installed hook.
        let author = Author::person("p8-codicil");
        let sessions = MereSessions::<_, Surface>::new_with(store.clone()).with_codicil(&SURFACE);
        let manifest = sessions
            .open_codicil_checked_with(id, author.clone(), |graph| {
                graph.shown(shown).map(|_| ()).ok_or("nothing shown".into())
            })
            .await
            .unwrap();
        assert!(manifest.codicil_read_only);
        let session = sessions.open_with(manifest.session_id).await.unwrap();
        assert_eq!(
            session.graph().shown(shown),
            Some(ResourceNode::for_term("https://p8.test/codicil").id())
        );

        // Controls: sessions with no hook refuse; the kernel's schemas do
        // not name the surface's codicil, nor list it.
        let bare = MereSessions::<_, Surface>::new_with(store.clone())
            .open_codicil_checked_with(id, author, |_| Ok(()))
            .await;
        assert!(
            matches!(&bare, Err(SessionError::Validation(why)) if why.contains("no codicil hook")),
            "{bare:?}"
        );
        let foreign =
            load_profiled_graph_codicil_sealed_with(&mut store, None, &KERNEL_CODICIL, id)
                .await
                .unwrap_err();
        assert!(
            foreign.to_string().contains("is not a graph codicil"),
            "{foreign}"
        );
        assert!(
            list_graph_codicils_with(&mut store, &KERNEL_CODICIL)
                .await
                .unwrap()
                .is_empty()
        );
    });
}

/// The kernel's hook stores exactly what the kernel stored through eidetic's
/// typed layer before it (D266): the same manifest and blob, byte for byte.
#[test]
fn the_kernel_hook_stores_what_the_typed_layer_stored() {
    use eidetic::{
        PrivacyClass, ProvenanceOrigin, ProvenanceRecord, TrustEnvelope, save_typed_sealed,
    };
    use muniment::Backend;
    use pandect::graph_codicil::{GraphCodicil, save_graph_snapshot_codicil_sealed};

    async fn contents(store: &MemoryBackend) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for key in store.list("").await.unwrap() {
            out.push((key.clone(), store.get(&key).await.unwrap().unwrap()));
        }
        out.sort();
        out
    }

    pollster::block_on(async {
        let mut graph = kernel::graph::Graph::new();
        kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://p8.test/kernel-codicil".into(),
            euclid::Point2D::new(1.0, 2.0),
        );
        let at = Timestamp(5);
        let mut hooked = MemoryBackend::new();
        let id = save_graph_snapshot_codicil_sealed(
            &mut hooked,
            None,
            graph.to_snapshot(),
            graph.facets().clone(),
            RedactionPolicy::include_all(),
            at,
        )
        .await
        .unwrap();
        let mut typed = MemoryBackend::new();
        let provenance = ProvenanceRecord {
            origin: ProvenanceOrigin::Generated,
            upstream: Vec::new(),
            tooling: Some(format!(
                "pandect/graph-codicil@{}",
                env!("CARGO_PKG_VERSION")
            )),
            generated_at: at,
        };
        let typed_id = save_typed_sealed(
            &mut typed,
            None,
            &GraphCodicil::<kernel::graph::Graph> {
                snapshot: graph.to_snapshot(),
                facets: graph.facets().clone(),
                sessions: Vec::new(),
            },
            Vec::new(),
            PrivacyClass::LocalOnly,
            provenance,
            TrustEnvelope::self_asserted(),
            at,
        )
        .await
        .unwrap();
        assert_eq!(id, typed_id);
        assert_eq!(contents(&hooked).await, contents(&typed).await);
        // Control: another moment's save is another manifest.
        let mut later = MemoryBackend::new();
        save_graph_snapshot_codicil_sealed(
            &mut later,
            None,
            graph.to_snapshot(),
            graph.facets().clone(),
            RedactionPolicy::include_all(),
            Timestamp(6),
        )
        .await
        .unwrap();
        assert_ne!(contents(&later).await, contents(&typed).await);
    });
}
