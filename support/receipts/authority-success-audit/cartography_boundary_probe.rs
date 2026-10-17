//! Audit observations; passing tests reproduce current behavior, not repairs.
use std::collections::HashMap;
use cartography::{AffinityScores, IntelligenceSignals, LayoutStrategy, NodeFilter,
    NodeOrder, Overlay, ProjectionRequest, Signal, ViewIntent, ORDER_TIMELINE};
use kernel::geometry::PortablePoint;
use kernel::graph::{Graph, NodeKey, SemanticStatementSpec, SemanticSubKind, predicate_iri};

fn fixture() -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = (0..3).map(|i| kernel::graph::apply::add_node(
        &mut graph, None, format!("audit://{i}"), PortablePoint::zero()
    )).collect();
    (graph, keys)
}
fn ordered(keys: &[NodeKey]) -> IntelligenceSignals {
    IntelligenceSignals::new().with(ORDER_TIMELINE,
        Signal::Order(NodeOrder { order: keys.to_vec() }))
}

#[test]
fn affinity_lookup_prefers_storage_order_over_requested_direction() {
    let a = NodeKey::new(0);
    let b = NodeKey::new(1);
    let reverse_first = AffinityScores { pairs: vec![((b,a), 0.2), ((a,b), 0.8)] };
    assert_eq!(reverse_first.lookup(a,b), Some(0.2));
    let forward_first = AffinityScores { pairs: vec![((a,b), 0.8), ((b,a), 0.2)] };
    assert_eq!(forward_first.lookup(a,b), Some(0.8));
    assert_eq!(AffinityScores { pairs: vec![((b,a), 0.2)] }.lookup(a,b), Some(0.2));
    println!("An explicitly asymmetric affinity query returns the reverse pair when that pair is stored first; forward and reverse-only controls pass");
}

#[test]
fn arrangement_regions_disappear_in_portable_scene_lowering() {
    let (graph, keys) = fixture();
    let signals = ordered(&keys);
    let mut intent = ViewIntent::default();
    intent.axis_values = Some(keys.iter().enumerate().map(|(i,key)|
        (*key, cartography::AxisValue::Categorical(format!("group-{i}")))).collect());
    let request = ProjectionRequest { graph: &graph, signals: &signals, intent };
    let mut projection = cartography::adapters::PartitionAdapter::default().project(&request);
    assert!(!projection.regions.is_empty());
    assert!(projection.metadata.faults.is_empty());
    let lowered = cartography::scene_from_projection(&projection,
        |key| graph.get_node(key).unwrap().id.to_string(), |_| None);
    assert_eq!(lowered.items.len(), keys.len());
    assert!(lowered.regions.is_empty());
    // Cluster halos do lower through the same API; only arrangement regions vanish.
    projection.overlays.push(Overlay::ClusterHalo { cluster_id: "control".into(),
        members: keys, label: Some("halo control".into()), confidence: 0.5 });
    let control = cartography::scene_from_projection(&projection,
        |key| graph.get_node(key).unwrap().id.to_string(), |_| None);
    assert_eq!(control.regions.len(), 1);
    assert_eq!(control.regions[0].label.as_deref(), Some("halo control"));
    println!("Partition regions exist in the projection, but scene lowering drops all of them while retaining a halo overlay control");
}

#[test]
fn generic_adapter_replaces_footprint_bounds_with_point_bounds() {
    let (graph, all) = fixture();
    let keys = &all[..1];
    let signals = ordered(keys);
    let intent = ViewIntent { extents: Some(HashMap::from([(keys[0], (10.0,10.0))])),
        ..Default::default() };
    let request = ProjectionRequest { graph: &graph, signals: &signals, intent };
    let adapter = cartography::adapters::GridAdapter::default();
    let (score, returned_keys) = cartography::adapters::score_from_request(&request,
        sceno::Arrangement::Grid(sceno::Grid { origin: adapter.origin, cell: sceno::Vec2::ZERO,
            columns: 1, gap: adapter.pitch }), &cartography::adapters::Disclosures::default()).unwrap();
    let solved = scenomise::solve(&score);
    assert_eq!((solved.bounds.size.w, solved.bounds.size.h), (10.0,10.0));
    let projection = cartography::adapters::project_score("grid.default", &request, &score, &returned_keys);
    assert_eq!((projection.content_bounds.size.width, projection.content_bounds.size.height), (0.0,0.0));
    let lowered = cartography::scene_from_projection(&projection,
        |key| graph.get_node(key).unwrap().id.to_string(), |_| Some((10.0,10.0)));
    assert_eq!((lowered.bounds.size.w, lowered.bounds.size.h), (0.0,0.0));
    assert_eq!(lowered.items[0].footprint, sceno::Footprint::Rect { size: sceno::Size2::new(10.0,10.0) });
    println!("Generic solve-to-projection conversion shrinks valid 10x10 content bounds to 0x0; portable lowering retains the footprint but copies the zero bounds");
}

#[test]
fn explicit_view_filter_is_ignored_by_the_exported_strategy() {
    let (graph, keys) = fixture();
    let signals = ordered(&keys);
    let projection = cartography::adapters::GridAdapter::default().project(&ProjectionRequest {
        graph: &graph, signals: &signals,
        intent: ViewIntent::default().with_filter(NodeFilter::Explicit(vec![keys[0]]))
    });
    assert_eq!(projection.nodes.len(), 3);
    assert!(projection.metadata.faults.is_empty());
    assert!(projection.metadata.coverage.limits.is_empty());
    // Host-preselected order works, but is another input than ViewIntent.filter.
    let selected = ordered(&keys[..1]);
    let control = cartography::adapters::GridAdapter::default().project(&ProjectionRequest {
        graph: &graph, signals: &selected, intent: Default::default()
    });
    assert_eq!(control.nodes.len(), 1);
    println!("Exported Grid strategy accepts an explicit one-node ViewIntent filter yet places all three without a fault; selecting the order separately works");
}

#[test]
fn omitted_endpoint_is_fabricated_at_origin_in_positioned_edge() {
    let (mut graph, keys) = fixture();
    graph.try_assert_semantic_statement(keys[0],keys[1],SemanticStatementSpec {
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites), ..Default::default()
    }).unwrap();
    let signals = ordered(&keys[..1]);
    let request = ProjectionRequest { graph: &graph, signals: &signals, intent: Default::default() };
    let projection = cartography::adapters::score::projection_from_positions("audit", &request,
        HashMap::from([(keys[0],PortablePoint::new(50.0,60.0))]));
    assert_eq!(projection.nodes.len(),1);
    assert_eq!(projection.edges.len(),1);
    assert_eq!(projection.edges[0].path,vec![PortablePoint::new(50.0,60.0),PortablePoint::zero()]);
    assert!(projection.metadata.coverage.limits.iter().any(|limit| limit.reason == "relation pairs omitted from projection"));
    let lowered = cartography::scene_from_projection(&projection, |key| graph.get_node(key).unwrap().id.to_string(), |_| None);
    assert!(lowered.relations.is_empty());
    println!("Qualified edge boundary: raw projection fabricates an absent endpoint at origin, coverage reports omission, and portable lowering drops the edge");
}

#[test]
fn stale_order_is_skipped_by_generic_score_but_panics_in_spiral() {
    let (graph, mut keys) = fixture();
    keys.push(NodeKey::new(1000));
    let signals = ordered(&keys).with(cartography::WEIGHT_RECENCY,
        Signal::Weights(cartography::ImportanceWeights::default()));
    let generic = cartography::adapters::GridAdapter::default().project(&ProjectionRequest {
        graph: &graph, signals: &signals, intent: Default::default()
    });
    assert_eq!(generic.nodes.len(),3);
    let spiral = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
        cartography::project_spiral_score(&graph,&signals,ORDER_TIMELINE,None,None)));
    assert!(spiral.is_err());
    println!("Qualified signal admission boundary: a missing order key is skipped in generic score construction but panics in Spiral; current host registry pairing was not violated in a live caller");
}

#[test]
fn partition_adds_another_one_to_already_offset_degree_weights() {
    let (graph, keys) = fixture();
    // These are the channel's documented degree-plus-one values, as produced
    // by Pictograph's reviewed degree_weights function for an isolated node
    // and the endpoints of a detached edge. No Pictograph runtime is claimed.
    let weights = HashMap::from([(keys[0],1.0),(keys[1],2.0),(keys[2],2.0)]);
    let signals = ordered(&keys).with(cartography::WEIGHT_DEGREE,
        Signal::Weights(cartography::ImportanceWeights {
            weights: keys.iter().map(|key| (*key,weights[key])).collect()
        }));
    let intent = ViewIntent { axis_values: Some(keys.iter().map(|key|
        (*key,cartography::AxisValue::Categorical("all".into()))).collect()), ..Default::default() };
    let request = ProjectionRequest { graph: &graph, signals: &signals, intent };
    let actual = cartography::adapters::PartitionAdapter::default().project(&request);
    let config = sceno::Partition { bounds: sceno::Rect::new(sceno::Vec2::new(-240.0,-180.0),
        sceno::Size2::new(480.0,360.0)), ..Default::default() };
    let direct = |weights| {
        cartography::adapters::project_arrangement("control",&request,
            sceno::Arrangement::Partition(config.clone()),
            &cartography::adapters::Disclosures::from_intent(&request).with_weight(weights),vec![])
    };
    let unchanged = direct(weights.clone());
    let plus_one = direct(weights.iter().map(|(key,w)| (*key,w+1.0)).collect());
    let position = |projection: &cartography::Projection,key| projection.nodes.iter()
        .find(|node| node.node == key).unwrap().position;
    for key in &keys { assert_eq!(position(&actual,*key),position(&plus_one,*key)); }
    assert_ne!(position(&actual,keys[0]),position(&unchanged,keys[0]));
    println!("Partition reinterprets degree-plus-one weights1/2/2 as2/3/3, changing the isolated share from1/5 to2/8; unchanged-weight direct solve is the control");
}
