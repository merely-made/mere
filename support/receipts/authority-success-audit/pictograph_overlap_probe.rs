//! Observe public diagnostic counts against CPU body bounds, without production edits.
use kernel::geometry::PortablePoint;
use kernel::graph::Graph;
use pictograph::canvas::{Canvas, CameraView};

fn sample(size: f32, distance: f32) -> (usize, usize, bool) {
    let mut graph = Graph::new();
    let a = kernel::graph::apply::add_node(&mut graph, None,
        "https://overlap-a.test/".into(), PortablePoint::zero());
    let b = kernel::graph::apply::add_node(&mut graph, None,
        "https://overlap-b.test/".into(), PortablePoint::zero());
    let ids = [graph.get_node(a).unwrap().id, graph.get_node(b).unwrap().id];
    let mut canvas = Canvas::with_graph(graph);
    for id in ids { canvas.set_node_size(id, size); }
    canvas.set_physics_paused(true);
    canvas.seed_cartography([(ids[0], (0.0, 0.0)), (ids[1], (distance, 0.0))]);
    canvas.set_camera(CameraView { offset: (200.0, 200.0), zoom: 1.0 });
    // Body readback intentionally reports only uncovered hits. Read each
    // body alone so the other body's occlusion cannot clip its measured bounds.
    canvas.scope_to_exact_members([ids[0]]);
    canvas.frame(800, 600);
    let left = canvas.node_body_rect(a).unwrap();
    canvas.scope_to_exact_members([ids[1]]);
    canvas.frame(800, 600);
    let right = canvas.node_body_rect(b).unwrap();
    canvas.clear_scope();
    canvas.frame(800, 600);
    assert!((left.2 - left.0 - size).abs() < 0.02, "requested {size}: left {left:?}, right {right:?}");
    assert!((right.2 - right.0 - size).abs() < 0.02);
    let intersects = left.0 < right.2 && right.0 < left.2
        && left.1 < right.3 && right.1 < left.3;
    (canvas.layout_stats().overlaps, canvas.layout_stats_without_stretch().overlaps,
        intersects)
}

#[test]
fn resized_bodies_disagree_with_both_overlap_diagnostics() {
    assert_eq!(sample(36.0, 50.0), (0, 0, false));
    assert_eq!(sample(80.0, 50.0), (0, 0, true));
    assert_eq!(sample(36.0, 30.0), (1, 1, true));
    assert_eq!(sample(16.0, 30.0), (1, 1, false));
    println!("Both diagnostics count zero for 80px bodies 50px apart, and one for 16px bodies 30px apart. Default 36px controls agree with the CPU bounds.");
}
