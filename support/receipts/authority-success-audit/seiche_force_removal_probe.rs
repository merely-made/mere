//! Native observations of force removal and the quadtree's minimum-cell rung.
use euclid::default::Point2D;
use seiche::{AnchorSpring, BarnesHutConfig, Boundary, NodeKey, Simulation, repulsion_forces};

fn seeded() -> (Simulation, NodeKey) {
    let key = NodeKey::new(0);
    let mut simulation = Simulation::new();
    simulation.sync_nodes([(key, Point2D::new(100.0, 0.0))]);
    (simulation, key)
}

#[test]
fn removing_the_last_builtin_keeps_its_old_force_alive() {
    let (mut removed, key) = seeded();
    let (mut zeroed, _) = seeded();
    for sim in [&mut removed, &mut zeroed] {
        sim.add_force(Boundary { strength: 10.0 });
        sim.tick(1.0 / 60.0);
    }
    assert_eq!(removed.position_of(key), zeroed.position_of(key));
    assert_eq!(removed.velocity_of(key), zeroed.velocity_of(key));
    let first = removed.velocity_of(key).unwrap().x;
    removed.clear_forces();
    zeroed.set_forces(vec![Box::new(Boundary { strength: 0.0 })]);
    removed.tick(1.0 / 60.0);
    zeroed.tick(1.0 / 60.0);
    let stale = removed.velocity_of(key).unwrap().x;
    let coast = zeroed.velocity_of(key).unwrap().x;
    assert_eq!(removed.force_count(), 0);
    assert!(stale < first && coast > first && coast < 0.0);
    println!("Boundary removed: first vx={first}, next vx={stale}; zero-force coasting control vx={coast}");
}

#[test]
fn removing_the_last_anchor_has_the_same_persistent_force_problem() {
    let (mut removed, key) = seeded();
    let (mut zeroed, _) = seeded();
    for sim in [&mut removed, &mut zeroed] {
        sim.set_anchor_force(Some(AnchorSpring::new([(key, (0.0, 0.0))])));
        sim.tick(1.0 / 60.0);
    }
    let first = removed.velocity_of(key).unwrap().x;
    removed.set_anchor_force(None);
    zeroed.set_anchor_force(Some(AnchorSpring::new([(key, (0.0, 0.0))]).with_stiffness(0.0)));
    removed.tick(1.0 / 60.0);
    zeroed.tick(1.0 / 60.0);
    let stale = removed.velocity_of(key).unwrap().x;
    let coast = zeroed.velocity_of(key).unwrap().x;
    assert_eq!(removed.anchor_count(), 0);
    assert!(stale < first && coast > first && coast < 0.0);
    println!("Anchor removed: first vx={first}, next vx={stale}; zero-stiffness coasting control vx={coast}");
}

#[test]
fn zero_theta_still_merges_distinct_nearby_bodies_at_the_minimum_cell() {
    let positions = [Point2D::new(0.0, 0.0), Point2D::new(0.2, 0.0)];
    let force = |min_cell_size| repulsion_forces(&positions,
        BarnesHutConfig { theta: 0.0, min_cell_size }, 1.0, 1.0, 0.01);
    let merged = force(1.0);
    let resolved = force(0.01);
    assert!((merged[0].x + 20.0).abs() < 0.001);
    assert!((resolved[0].x + 5.0).abs() < 0.001);
    assert!((merged[0].x / resolved[0].x - 4.0).abs() < 0.001);
    println!("theta=0, distance=0.2, epsilon=0.01: min-cell 1 gives {}, min-cell 0.01 gives {} (pairwise value -5)", merged[0].x, resolved[0].x);
}
