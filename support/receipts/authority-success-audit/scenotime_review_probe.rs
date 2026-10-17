//! Public scene/trace validation and host-clock diagnostics; no renderer or source model.
use sceno::{Scene, SourceRef, SourceIx, InstanceId, ProjectedItem, Transform2, Footprint, Representation, Vec2, Size2, LinkKey, LinkStratum, RoutedRelation};
use scenotime::{SceneSnapshot, SceneEpoch, Revision, SceneDiff, SceneOp, SceneTrace, TraceStep, ProjectionClock, ApplyOutcome};

fn scene() -> Scene {
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("audit", "one"));
    scene.items.push(ProjectedItem { source, space: Scene::WORLD, transform: Transform2::IDENTITY,
        footprint: Footprint::Rect { size: Size2::new(20.0, 20.0) }, representation: Representation::Card,
        layer: 0, visible: true, hit: None, channels: vec![] });
    scene
}
fn snapshot() -> SceneSnapshot { SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene()).unwrap() }
fn diff(operation: SceneOp) -> SceneDiff { SceneDiff { epoch: SceneEpoch(1), base: Revision(1), revision: Revision(2), operations: vec![operation] } }
#[test] fn finite_trace_round_trip_control() {
    let trace = SceneTrace::from_steps(snapshot(), vec![TraceStep::diff("raise", diff(SceneOp::SetItemLayer { index: InstanceId(0), layer: 3 }))]).unwrap();
    let bytes = serde_json::to_vec(&trace).unwrap();
    assert_eq!(serde_json::from_slice::<SceneTrace>(&bytes).unwrap(), trace);
    println!("CONTROL finite-trace validates-encodes-reopens=true");
}
#[test] fn nonfinite_scene_values_validate_and_serialize_but_do_not_reopen() {
    for field in ["item-position", "item-scale", "footprint-width", "scene-bounds"] {
        let mut scene = scene();
        match field {
            "item-position" => scene.items[0].transform.translate.x = f32::NAN,
            "item-scale" => scene.items[0].transform.scale = f32::INFINITY,
            "footprint-width" => scene.items[0].footprint = Footprint::Rect { size: Size2::new(f32::NAN, 20.0) },
            _ => scene.bounds.origin.x = f32::INFINITY,
        }
        let state = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap();
        state.validate().unwrap();
        let bytes = serde_json::to_vec(&state).unwrap();
        assert!(serde_json::from_slice::<SceneSnapshot>(&bytes).is_err());
        let trace = SceneTrace::new(state).unwrap();
        let bytes = serde_json::to_vec(&trace).unwrap();
        assert!(serde_json::from_slice::<SceneTrace>(&bytes).is_err());
        println!("OBSERVED nonfinite-{field} snapshot-and-trace-validate=true serialization-success=true own-bytes-reopen=false");
    }
}
#[test] fn nonfinite_diff_commits_revision_and_builds_a_non_reopenable_trace() {
    let before = snapshot();
    let mut bounds = before.tables.bounds;
    bounds.origin.y = f32::NAN;
    let change = diff(SceneOp::SetBounds { bounds });
    let mut live = before.clone();
    assert_eq!(live.apply_diff(&change), Ok(ApplyOutcome::Applied));
    assert_eq!(live.revision, Revision(2));
    let trace = SceneTrace::from_steps(before, vec![TraceStep::diff("nonfinite bounds", change)]).unwrap();
    assert_eq!(trace.head().revision, Revision(2));
    assert!(serde_json::from_slice::<SceneTrace>(&serde_json::to_vec(&trace).unwrap()).is_err());
    println!("OBSERVED nonfinite-diff committed-revision=2 trace-built=true trace-serialized=true own-trace-bytes-reopen=false");
}
#[test] fn monotonic_clock_and_repeated_timestamp_control() {
    let mut clock = ProjectionClock::default();
    assert_eq!(clock.observe(100.0), 0.0);
    assert_eq!(clock.observe(110.0), 10.0);
    assert_eq!(clock.observe(110.0), 10.0);
    println!("CONTROL monotonic-clock repeated-host-timestamp adds-nothing=true");
}
#[test] fn qualified_backward_clock_sample_rewinds_the_delta_baseline() {
    let mut clock = ProjectionClock::default();
    clock.observe(100.0); clock.observe(110.0);
    assert_eq!(clock.observe(105.0), 10.0);
    assert_eq!(clock.observe(110.0), 15.0);
    println!("QUALIFIED host-samples=[100,110,105,110] elapsed=15 already-seen-host-time-charged-again=true normal-browser-monotonic-clock-not-exercised=true");
}
#[test] fn qualified_snapshot_checks_do_not_canonicalize_routed_link_keys() {
    let canonical = LinkKey::new(LinkStratum::Surface, "b", "a");
    let reversed: LinkKey = serde_json::from_str(r#"{"stratum":"surface","first":"b","second":"a"}"#).unwrap();
    assert_ne!(canonical.stable_id(), reversed.stable_id());
    let mut scene = scene();
    scene.relations.push(RoutedRelation { from: InstanceId(0), to: InstanceId(0), space: Scene::WORLD, points: vec![Vec2::ZERO], kind: None, weight: None,
        link_key: Some(reversed), marks: Default::default(), strength: None });
    let snapshot = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap();
    snapshot.validate().unwrap();
    assert_eq!(snapshot.tables.items[0].as_ref().unwrap().source, SourceIx(0));
    println!("QUALIFIED noncanonical-link-key snapshot-admitted=true differing-view-token=true consumer-grouping-failure-not-exercised=true");
}
