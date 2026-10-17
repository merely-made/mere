//! Bounded observations of current APIs. No product patches or GPU execution.
use kernel::geometry::PortablePoint;
use kernel::graph::{Graph,NodeKey};
use pictograph::canvas::{Canvas,AffinityBlend};

fn edgeless(n:usize) -> (Graph,Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = (0..n).map(|i| kernel::graph::apply::add_node(&mut graph,None,
        format!("https://signal-audit-{i}.test/"),PortablePoint::zero())).collect();
    (graph,keys)
}

#[test]
fn empty_affinity_restarts_a_paused_backend_on_every_unchanged_frame() {
    let (graph,_) = edgeless(3);
    let mut canvas = Canvas::with_graph(graph.clone());
    let mut control = Canvas::with_graph(graph);
    canvas.set_physics_paused(true);
    control.set_physics_paused(true);
    canvas.set_affinity_blend(AffinityBlend::ContentOnly);
    canvas.set_content_affinity(Some(Vec::new()));
    canvas.set_cluster_by_affinity(true);
    assert!(!canvas.is_settling());
    canvas.frame(800,600);
    let budget = canvas.physics_tick_demand().1;
    assert!(budget > 0);
    for _ in 0..400 {
        canvas.frame(800,600);
        control.frame(800,600);
        assert_eq!(canvas.physics_tick_demand().1,budget);
    }
    assert!(canvas.physics_paused());
    assert!(canvas.is_settling());
    assert!(!control.is_settling());
    assert!(canvas.field_clock().elapsed > control.field_clock().elapsed);
    let elapsed = canvas.field_clock().elapsed;
    canvas.set_cluster_by_affinity(false);
    for _ in 0..400 { canvas.frame(800,600); }
    assert!(!canvas.is_settling());
    println!("Empty authoritative affinity kept budget={budget} after400 unchanged CPU frames; clock advanced to{elapsed:?} while paused. Disabled control stayed idle; turning affinity off allowed the remaining budget to drain.");
}

#[test]
fn edgeless_spectral_disclosure_is_the_uniterated_start_not_missing_structure() {
    let (graph,keys) = edgeless(3);
    assert_eq!(graph.projected_relations().count(),0);
    let zero = pictograph::signals::spectral_coords(&graph,0);
    let iterated = pictograph::signals::spectral_coords(&graph,200);
    assert_eq!(iterated.len(),3);
    assert_eq!(zero,iterated);
    assert!(iterated.values().all(|(x,y)| x.is_finite() && y.is_finite()));
    assert!(iterated.values().any(|(x,y)| x.abs()>0.1 || y.abs()>0.1));
    let mut registry = pictograph::signals::ChannelRegistry::new();
    assert_eq!(registry.spectral(&graph,200),&iterated);
    registry.spectral(&graph,200);
    assert_eq!(registry.runs().spectral,1);
    // Shared projected importance does see a newly supplied Resource edge.
    let mut linked = graph.clone();
    linked.try_assert_semantic_statement(keys[0],keys[1],kernel::graph::SemanticStatementSpec {
        predicate:kernel::graph::predicate_iri(kernel::graph::SemanticSubKind::Cites).into(),
        recognized_sub_kind:Some(kernel::graph::SemanticSubKind::Cites),..Default::default()
    }).unwrap();
    assert_eq!(pictograph::signals::degree_importance(&linked).lookup(keys[0]),Some(1.0));
    assert!(!pictograph::signals::spectral_coords(&linked,200).is_empty());
    assert!(pictograph::signals::spectral_coords(&edgeless(0).0,200).is_empty());
    assert!(pictograph::signals::spectral_coords(&edgeless(1).0,200).is_empty());
    println!("Three edgeless nodes disclose finite nonzero coordinates identical at0 and200 iterations; registry caches them once. Empty/single-node controls return the promised empty map. Resource degree control succeeds.");
}
