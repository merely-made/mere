//! Public collider admission and actual fluid/runtime geometry observations.
use euclid::default::{Point2D, Vector2D};
use seiche::{Basin, FluidParams, NodeCollider, NodeKey, Simulation};

#[test]
fn rounded_zero_half_panics_instead_of_clamping_like_the_other_shapes() {
    let key = NodeKey::new(0);
    let mut sim = Simulation::new();
    sim.sync_nodes([(key, Point2D::zero())]);
    sim.set_node_colliders([(key, NodeCollider::Ball { radius: 0.0 })]);
    sim.set_node_colliders([(key, NodeCollider::Square { half: 0.0 })]);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sim.set_node_colliders([(key, NodeCollider::RoundedSquare { half: 0.0, border: 0.0 })]);
    }));
    assert!(panic.is_err());
    println!("Zero Ball/Square clamp successfully; zero RoundedSquare panics on clamp bounds.");
}

fn particle_after_tick(radius: f32, x: f32, tangible: bool) -> f32 {
    let key = NodeKey::new(0);
    let mut sim = Simulation::new();
    sim.sync_nodes([(key, Point2D::zero())]);
    sim.set_node_colliders([(key, NodeCollider::Ball { radius })]);
    sim.pin(key, Point2D::zero());
    sim.set_nodes_tangible(tangible);
    sim.load_fluid(FluidParams {
        gravity: Vector2D::zero(), damping: 1.0, viscosity: 0.0, solver_iters: 0,
        ..FluidParams::default()
    }, Basin { min_x: -1000.0, max_x: 1000.0, floor_y: 1000.0 },
        Point2D::new(x, 0.0), 1, 1, 16.0);
    sim.tick(1.0 / 60.0);
    assert_eq!(sim.position_of(key).unwrap(), Point2D::zero());
    sim.fluid_particles().next().unwrap().x
}

#[test]
fn fluid_contacts_use_the_default_radius_after_the_live_node_is_resized() {
    assert_eq!(particle_after_tick(18.0, 20.0, true), 25.0);
    assert_eq!(particle_after_tick(80.0, 60.0, false), 60.0);
    assert_eq!(particle_after_tick(80.0, 60.0, true), 60.0);
    assert_eq!(particle_after_tick(4.0, 20.0, true), 25.0);
    println!("Fluid radius7: default node18 pushes x20 to25; resized node80 leaves particle x60 inside; node4 incorrectly pushes clear x20 to25. Tangibility-off control leaves x60 unchanged.");
}
