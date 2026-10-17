//! Tick-demand observations through the real inline and actor backends.
use euclid::default::Point2D;
use seiche::{Declared, Density, Force, ForceContext, GravityLocus, NodeKey, Physics, Simulation, Term};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Same force execution, with continued demand supplied as the control.
struct LiveTide(GravityLocus);
impl Declared for LiveTide {
    fn terms(&self) -> Vec<Term> { self.0.terms() }
}
impl Force for LiveTide {
    fn apply(&self, ctx: &mut ForceContext<'_>, dt: f32) { self.0.apply(ctx, dt); }
    fn wants_tick(&self) -> bool { true }
}
fn tide(live: bool) -> Simulation {
    let mut sim = Simulation::new();
    sim.sync_nodes([(NodeKey::new(0), Point2D::zero())]);
    let force = GravityLocus::tidal((0.0, 0.0), 200.0, 12.0);
    if live { sim.add_force(LiveTide(force)); } else { sim.add_force(force); }
    sim
}

#[test]
fn tidal_force_parks_when_the_inline_budget_ends() {
    for live in [false, true] {
        let sim = tide(live);
        let mut view = sim.view();
        let mut physics = Physics::inline(sim, 2);
        for _ in 0..20 { physics.advance_frame(&mut view); }
        assert_eq!(physics.pace().ticks, if live { 20 } else { 2 });
        assert_eq!(physics.is_settling(), live);
        if !live {
            let frozen = view.position_of(NodeKey::new(0));
            for _ in 0..10 { assert!(!physics.advance_frame(&mut view)); }
            assert_eq!(view.position_of(NodeKey::new(0)), frozen);
        }
    }
    println!("Inline tide stops at its two-tick budget and stays frozen; the same force with wants_tick=true reaches20 ticks and keeps running.");
}

#[test]
fn tidal_force_also_parks_on_the_real_actor() {
    for live in [false, true] {
        let sim = tide(live);
        let mut view = sim.view();
        let mut physics = Physics::inline(sim, 2);
        physics.offload(Arc::new(|| {}));
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            physics.refresh(&mut view);
            if (!live && physics.pace().ticks == 2 && !physics.is_settling())
                || (live && physics.pace().ticks >= 4) { break; }
            assert!(Instant::now() < deadline, "actor observation timed out");
            std::thread::sleep(Duration::from_millis(2));
        }
        if !live {
            let frozen = view.position_of(NodeKey::new(0));
            std::thread::sleep(Duration::from_millis(60));
            physics.refresh(&mut view);
            assert_eq!(physics.pace().ticks, 2);
            assert_eq!(view.position_of(NodeKey::new(0)), frozen);
        } else {
            assert!(physics.is_settling());
        }
        physics.halt();
    }
    println!("Actual actor tide parks at two ticks; live-demand control continues past the budget. Both actors are halted and dropped.");
}

#[test]
fn an_empty_density_flow_spends_ticks_forever_without_starting() {
    let mut empty = Simulation::new();
    empty.set_forces(vec![Box::new(Density::new([], 16))]);
    let mut view = empty.view();
    let mut physics = Physics::inline(empty, 1);
    for _ in 0..180 { assert!(physics.advance_frame(&mut view)); }
    assert!(view.positions().next().is_none());
    assert_eq!(physics.pace().ticks, 180);
    assert_eq!(physics.tick_demand(), (true, 0));
    physics.halt();
    assert!(!physics.is_settling());

    let mut placed = Simulation::new();
    placed.sync_nodes([(NodeKey::new(0), Point2D::zero())]);
    placed.set_forces(vec![Box::new(Density::new([], 16))]);
    let mut view = placed.view();
    let mut physics = Physics::inline(placed, 1);
    for _ in 0..180 { physics.advance_frame(&mut view); }
    assert!(!physics.is_settling());
    assert!(physics.pace().ticks < 180);

    let empty = Simulation::new();
    let mut view = empty.view();
    let mut physics = Physics::inline(empty, 1);
    physics.advance_frame(&mut view);
    assert_eq!(physics.pace().ticks, 1);
    assert!(!physics.is_settling());
    println!("Empty Density continues180 ticks with zero budget and zero bodies; populated Density finishes, empty no-force control stops after one tick, and explicit Halt stops the empty Density loop.");
}
