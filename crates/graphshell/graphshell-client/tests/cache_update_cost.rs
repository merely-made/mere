// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Count allocations on the calling thread, excluding fixture construction.
//! This detects cache/map copying even when timing on a shared host is noisy.
use chirograph::*;
use graphshell_client::ClientState;
use sceno::*;
use scenotime::{Revision, SceneDiff, SceneEpoch, SceneSnapshot};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! { static ALLOCATED: Cell<Option<usize>> = const { Cell::new(None) }; }
struct Counting;
#[global_allocator]
static ALLOCATOR: Counting = Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCATED.try_with(|count| {
            if let Some(bytes) = count.get() {
                count.set(Some(bytes + layout.size()));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let _ = ALLOCATED.try_with(|count| {
            if let Some(bytes) = count.get() {
                count.set(Some(bytes + size));
            }
        });
        unsafe { System.realloc(ptr, layout, size) }
    }
}
fn measured(f: impl FnOnce()) -> usize {
    ALLOCATED.with(|count| count.set(Some(0)));
    f();
    ALLOCATED.with(|count| count.replace(None).unwrap())
}
fn mount(client: &mut ClientState, session: &ProjectionSession, size: usize) {
    let bytes = vec![17; size];
    let resource = ResourceResponse::new(session.clone(), bytes);
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("fixture", "one"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Point,
        representation: Representation::Glyph,
        layer: 0,
        visible: true,
        hit: None,
        channels: vec![],
    });
    let key = PresentationKey("one".into());
    let presentation = PresentationManifest {
        bindings: vec![PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        }],
        offers: [(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::ImageV1 {
                    mime_type: "image/png".into(),
                },
                resource: resource.resource,
                byte_size: size as u64,
                requires: PresentationCapability::Image,
                semantics: PresentationSemantics {
                    label: "One".into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: vec![],
                },
            }],
        )]
        .into(),
    };
    client
        .apply_snapshot(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: session.clone(),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
            presentation,
            cache_policy: CachePolicy {
                retention: CacheRetention::MemoryOnly,
                expires_at_ms: None,
                purge_on_revocation: true,
            },
        })
        .unwrap();
    client.apply_resource(resource).unwrap();
}
fn diff(session: &ProjectionSession, base: u64) -> ProjectionDiff {
    ProjectionDiff {
        version: ProtocolVersion::V1,
        session: session.clone(),
        scene: SceneDiff {
            epoch: SceneEpoch(1),
            base: Revision(base),
            revision: Revision(base + 1),
            operations: vec![],
        },
        presentation: vec![],
        status: None,
    }
}
#[test]
fn small_diff_and_replay_allocations_do_not_scale_with_unrelated_cached_bytes() {
    let session = ProjectionSession("active".into());
    let mut small = ClientState::default();
    let mut large = ClientState::default();
    mount(&mut small, &session, 1024);
    mount(&mut large, &session, 4 * 1024 * 1024);
    for i in 0..32 {
        mount(
            &mut large,
            &ProjectionSession(format!("other:{i}")),
            1024 * 1024,
        );
    }
    let update = diff(&session, 1);
    let small_diff = measured(|| {
        small.apply_diff(&update).unwrap();
    });
    let large_diff = measured(|| {
        large.apply_diff(&update).unwrap();
    });
    assert_eq!(
        small_diff, large_diff,
        "a small update must not allocate for cached bytes or unrelated resource entries"
    );
    let small_reply = ResumeReply::Diffs(vec![diff(&session, 2), diff(&session, 3)]);
    let large_reply = small_reply.clone();
    let small_resume = measured(|| {
        small.apply_resume(&session, small_reply).unwrap();
    });
    let large_resume = measured(|| {
        large.apply_resume(&session, large_reply).unwrap();
    });
    assert_eq!(
        small_resume, large_resume,
        "replay must stage scene changes without cloning the client cache"
    );
}
