use conatus::{BodyCommand, BodyDesc, ColliderDesc, ColliderShape, Engine, EngineConfig, Phase,
    SystemError, Transform};

fn engine() -> Engine {
    Engine::new(EngineConfig { gravity: [0.0;3], ..EngineConfig::default() }).unwrap()
}
fn body() -> BodyDesc {
    BodyDesc::dynamic().with_collider(ColliderDesc::new(ColliderShape::sphere(1.0)))
}
fn fail_once(runtime: &mut Engine) {
    let mut first = true;
    runtime.add_system(Phase::AfterPhysics, "one admitted failure", move |_| {
        if std::mem::take(&mut first) { Err(SystemError::new("injected after physics")) } else { Ok(()) }
    });
}

#[test]
fn an_error_consumes_granted_clock_steps_that_never_reach_physics() {
    let mut failed = engine();
    fail_once(&mut failed);
    assert!(failed.advance(50_000).is_err());
    assert_eq!(failed.clock().steps_taken(), 3);
    assert_eq!(failed.bodies().tick(), 1);
    let retry = failed.advance(0).unwrap();
    assert_eq!(retry.steps, 0);
    assert_eq!(retry.deferred_steps, 0);
    assert_eq!(failed.bodies().tick(), 1);
    let mut control = engine();
    let normal = control.advance(50_000).unwrap();
    assert_eq!(normal.steps, 3);
    assert_eq!(control.bodies().tick(), 3);
    println!("After injected after-physics error: clock3, actual tick1, retry at elapsed0 has no backlog. No-error control executes3.");
}

#[test]
fn an_error_discards_applied_command_results_changes_and_prior_removals() {
    let run = |fault: bool| {
        let mut runtime = engine();
        let removed = runtime.bodies_mut().spawn(BodyDesc::fixed()).unwrap();
        runtime.advance(0).unwrap();
        runtime.bodies_mut().despawn(removed).unwrap();
        let command = runtime.queue(BodyCommand::Spawn { body: BodyDesc::fixed() });
        if fault { fail_once(&mut runtime); }
        let attempted = runtime.step(1);
        assert_eq!(runtime.bodies().len(), 1); // Spawn was actually applied.
        let after = runtime.advance(0).unwrap();
        if fault {
            assert!(attempted.is_err());
            assert!(after.commands.is_empty() && after.changed.is_empty() && after.removed.is_empty());
        } else {
            let published = attempted.unwrap();
            assert_eq!(published.commands.len(), 1);
            assert_eq!(published.commands[0].id, command);
            assert_eq!(published.changed.len(), 1);
            assert_eq!(published.removed, vec![removed]);
        }
    };
    run(true);
    run(false);
    println!("Error path loses applied spawn result, changed body and preexisting despawn notification; subsequent successful zero-step frame publishes none. No-error control publishes all three.");
}

#[test]
fn impulse_changes_velocity_but_zero_step_publication_omits_it() {
    for torque in [false, true] {
        let mut runtime = engine();
        let id = runtime.bodies_mut().spawn(body()).unwrap();
        runtime.step(1).unwrap();
        if torque { runtime.bodies_mut().apply_torque_impulse(id, [2.0,0.0,0.0]).unwrap(); }
        else { runtime.bodies_mut().apply_impulse(id, [2.0,0.0,0.0]).unwrap(); }
        let actual = runtime.bodies().state(id).unwrap();
        let speed = if torque { actual.velocity.angular[0] } else { actual.velocity.linear[0] };
        assert!(speed > 0.0, "impulse did not change the actual state");
        let frame = runtime.advance(0).unwrap();
        assert!(frame.changed.is_empty());
        runtime.bodies_mut().set_velocity(id, actual.velocity, true).unwrap();
        let control = runtime.advance(0).unwrap();
        assert_eq!(control.changed.len(), 1);
        assert_eq!(control.changed[0].velocity, actual.velocity);
        println!("torque={torque}: impulse changes speed to{speed}; changed[] remains empty until identical set_velocity marks it dirty.");
    }
}

#[test]
fn extreme_finite_rotation_admission_observation() {
    let mut runtime = engine();
    let rotation = [f32::MAX, 0.0, 0.0, 1.0];
    let id = runtime.bodies_mut().spawn(BodyDesc::fixed().at(Transform { rotation, ..Transform::IDENTITY })).unwrap();
    let actual = runtime.bodies().state(id).unwrap().transform.rotation;
    let norm: f32 = actual.iter().map(|v| v*v).sum();
    assert!(!norm.is_finite() || (norm-1.0).abs() > 0.1, "normalization unexpectedly produced a usable unit rotation: {actual:?}");
    let mut control = engine();
    let good = control.bodies_mut().spawn(BodyDesc::fixed().at(Transform { rotation:[2.0,0.0,0.0,0.0], ..Transform::IDENTITY })).unwrap();
    let finite = control.bodies().state(good).unwrap().transform.rotation;
    let good_norm: f32 = finite.iter().map(|v|v*v).sum();
    assert!((good_norm-1.0).abs() < 0.001);
    println!("Finite extreme quaternion accepted as{actual:?}, norm squared{norm}; moderate nonunit control normalizes to{finite:?}.");
}

#[test]
fn extreme_finite_ray_direction_reports_clear_space_through_a_present_body() {
    let mut world = conatus::BodyWorld::new([0.0;3]);
    let id = world.spawn(BodyDesc::fixed().with_collider(ColliderDesc::new(ColliderShape::cuboid([1.0;3])))).unwrap();
    world.refresh_queries();
    let filter = conatus::SpatialFilter::default();
    let normal = world.raycast([0.0,4.0,0.0], [0.0,-1.0,0.0], 10.0, true, filter).unwrap().unwrap();
    assert_eq!(normal.collider.body(), id);
    assert!((normal.distance-3.0).abs()<0.001);
    let extreme = world.raycast([0.0,4.0,0.0], [0.0,-f32::MAX,0.0], 10.0, true, filter).unwrap();
    assert!(extreme.is_none());
    let moderate = world.raycast([0.0,4.0,0.0], [0.0,-10.0,0.0], 10.0, true, filter).unwrap().unwrap();
    assert_eq!(moderate, normal);
    println!("Same finite ray direction: normal and10x hit at3 units, f32::MAX magnitude is accepted as Ok(None) after norm overflow.");
}
