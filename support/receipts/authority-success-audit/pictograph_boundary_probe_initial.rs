//! Bounded audit observations, not repairs or headed acceptance.
use kernel::geometry::PortablePoint;
use kernel::graph::{Graph, NodeKey, SemanticStatementSpec, SemanticSubKind, predicate_iri};
use pictograph::canvas::{Canvas, CameraView, DynamicsTarget};
use pictograph::canvas::camera_host::CameraHost;
use pictograph::canvas::controls::CanvasCommand;
use scenograph::link::{LinkKey, LinkStratum};

fn graph() -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = (0..3).map(|i| kernel::graph::apply::add_node(&mut graph,None,
        format!("https://audit-{i}.test/"),PortablePoint::zero())).collect();
    (graph, keys)
}
fn spec() -> SemanticStatementSpec {
    SemanticStatementSpec { predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites), ..Default::default() }
}

#[test]
fn degree_encodings_ignore_a_resource_link_that_the_shared_channel_counts() {
    let (mut graph, keys) = graph();
    graph.try_assert_semantic_statement(keys[0], keys[1], spec()).unwrap();
    assert_eq!(graph.resource_relations().count(),1);
    assert_eq!(graph.projected_relations().count(),1);
    assert_eq!(graph.neighbors_undirected(keys[0]).count(),0);
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_size_by_degree(true);
    canvas.set_height_by_degree(true);
    assert_eq!(canvas.node_size(keys[0]),36.0);
    assert_eq!(canvas.node_height(keys[0]),0.0);
    let weights = canvas.channel_values(pictograph::canvas::channels::Channel::Weight(
        pictograph::canvas::channels::WeightSource::Degree));
    let pictograph::canvas::ChannelValues::Weights(weights) = weights else { panic!() };
    assert_eq!(weights.iter().find(|(key,_)| *key == keys[0]).unwrap().1,2.0);
    assert_eq!(weights.iter().find(|(key,_)| *key == keys[2]).unwrap().1,1.0);
    // A Surface-owned relation exercises the old calculation as a control.
    let mut surface = canvas.graph().clone();
    kernel::graph::apply::assert_relation(&mut surface, keys[0],keys[1],
        kernel::graph::EdgeAssertion::Semantic { sub_kind: SemanticSubKind::UserGrouped,
            label: None, decay_progress: None });
    let mut control = Canvas::with_graph(surface);
    control.set_size_by_degree(true);
    control.set_height_by_degree(true);
    assert_eq!(control.node_size(keys[0]),44.0);
    assert_eq!(control.node_height(keys[0]),16.0);
    println!("Resource Cites contributes degree to the shared weight channel but not face size/height; Surface UserGrouped control grows and lifts the endpoint");
}

#[test]
fn camera_zoom_helper_clears_a_ctrl_key_that_is_still_held() {
    let mut canvas = Canvas::with_graph(Graph::new());
    canvas.set_camera(CameraView { offset:(0.0,0.0), zoom:1.0 });
    canvas.set_ctrl(true);
    CameraHost::wheel_zoom_px(&mut canvas,(0.0,0.0),40.0);
    let after = canvas.camera().zoom;
    assert!((after-1.15).abs() < 1e-5);
    canvas.wheel(0.0,40.0);
    assert_eq!(canvas.camera().zoom,after);
    // Reasserting the physically unchanged key restores the zoom behavior.
    canvas.set_ctrl(true);
    canvas.wheel(0.0,40.0);
    assert!(canvas.camera().zoom > after);
    println!("Programmatic zoom clears stored Ctrl; a subsequent wheel pans until the host sends Ctrl=true again");
}

#[test]
fn explicit_pan_command_becomes_zoom_while_ctrl_is_held() {
    let mut canvas = Canvas::with_graph(Graph::new());
    canvas.set_camera(CameraView { offset:(0.0,0.0), zoom:1.0 });
    canvas.set_ctrl(true);
    CanvasCommand::Pan {dx:0.0,dy:40.0}.apply(&mut canvas,(800,600));
    assert!((canvas.camera().zoom-1.15).abs() < 1e-5);
    let mut control = Canvas::with_graph(Graph::new());
    control.set_camera(CameraView { offset:(0.0,0.0), zoom:1.0 });
    CanvasCommand::Pan {dx:0.0,dy:40.0}.apply(&mut control,(800,600));
    assert_eq!(control.camera().zoom,1.0);
    println!("Explicit Pan shares the raw modifier-sensitive wheel path and zooms with Ctrl held; the released-key control does not zoom");
}

#[test]
fn retired_field_still_emits_visual_coupling_paint() {
    use kernel::graph::{Field,FieldId,FieldDefinition,ScalarField,Coupling,CouplingId,
        CouplingResponse,NodeSelector,COUPLING_VOCAB};
    use kernel::graph::apply::{GraphDelta,apply_graph_delta};
    let (mut graph, keys) = graph();
    let field = FieldId::from_uuid(graph.get_node(keys[0]).unwrap().id);
    apply_graph_delta(&mut graph,GraphDelta::AddField {
        field: Field::new(field,FieldDefinition::Scalar(ScalarField::Const(1.0))) });
    let coupling_id = CouplingId::from_uuid(graph.get_node(keys[1]).unwrap().id);
    apply_graph_delta(&mut graph,GraphDelta::AddCoupling { coupling: Coupling::new(
        coupling_id,field,NodeSelector::All,
        CouplingResponse::open(format!("{COUPLING_VOCAB}visual/halo")),1.0) });
    let projection = cartography::Projection { nodes:vec![cartography::PositionedNode {
        node:keys[0],position:PortablePoint::zero(),radius:10.0 }], ..Default::default() };
    let style = pictograph::canvas::scene_paint::ScenePaintStyle::default();
    let before = pictograph::canvas::coupling_paint::visual_overlays(&graph,&projection,&style);
    assert_eq!(before.len(),1);
    apply_graph_delta(&mut graph,GraphDelta::RetireField {id:field});
    assert!(!graph.field(field).unwrap().is_active());
    let after = pictograph::canvas::coupling_paint::visual_overlays(&graph,&projection,&style);
    assert_eq!(after.len(),1);
    assert_eq!(format!("{before:?}"),format!("{after:?}"));
    println!("Retirement leaves visual halo paint identical; this public painter has no active-field check while Canvas coupling-force rebuild does");
}

fn folded() -> (Canvas, LinkKey, String, Vec<NodeKey>) {
    let (mut graph,keys) = graph();
    let (_,record) = graph.try_assert_semantic_statement(keys[1],keys[2],spec()).unwrap();
    let key = LinkKey::new(LinkStratum::Resource,
        graph.resource(graph.shown_resource_id(keys[1]).unwrap()).unwrap().canonical_iri(),
        graph.resource(graph.shown_resource_id(keys[2]).unwrap()).unwrap().canonical_iri());
    let members = keys[..2].iter().map(|key| graph.get_node(*key).unwrap().id.to_string()).collect::<Vec<_>>();
    let fold = curation::FoldRecord::selection(curation::FoldId::new("audit-fold"),"audit",members).unwrap();
    let mut canvas = Canvas::with_graph(graph);
    assert!(canvas.restore_folds([fold]).is_empty());
    assert_eq!(canvas.fold_routes().len(),1);
    (canvas,key,record.statement_id,keys)
}
fn controller_rows(canvas:&Canvas,keys:&[NodeKey]) -> usize {
    let report = canvas.inspect_dynamics(DynamicsTarget::Pair {
        first:canvas.graph().get_node(keys[1]).unwrap().id,
        second:canvas.graph().get_node(keys[2]).unwrap().id }).unwrap();
    report.pair_controller_edges.unwrap()
}
#[test]
fn fold_boundary_ignores_canonical_link_hide() {
    let (mut canvas,key,_,keys) = folded();
    assert_eq!(controller_rows(&canvas,&keys),1);
    canvas.set_hidden_link_keys([key]);
    assert_eq!(controller_rows(&canvas,&keys),0);
    assert_eq!(canvas.fold_routes().len(),1);
    assert_eq!(canvas.fold_routes()[0].count,1);
    println!("Canonical hide removes the source controller row but leaves its fold boundary route");
}
#[test]
fn fold_boundary_ignores_exact_assertion_hide() {
    let (mut canvas,_,assertion,keys) = folded();
    assert_eq!(controller_rows(&canvas,&keys),1);
    canvas.set_hidden_assertion_ids([assertion]);
    assert_eq!(controller_rows(&canvas,&keys),0);
    assert_eq!(canvas.fold_routes().len(),1);
    println!("Exact assertion hide removes the controller row but fold_routes counts and returns it");
}

#[test]
fn exact_assertion_hide_disagrees_with_the_shared_topology_inspector() {
    let (mut canvas,_,assertion,keys) = folded();
    let target = DynamicsTarget::Pair {
        first:canvas.graph().get_node(keys[1]).unwrap().id,
        second:canvas.graph().get_node(keys[2]).unwrap().id };
    let before = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(before.pair_controller_edges,Some(1));
    assert_eq!(before.pair_inputs.unwrap().visible_rows,1);
    canvas.set_hidden_assertion_ids([assertion]);
    let after = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(after.pair_controller_edges,Some(0));
    let inputs = after.pair_inputs.unwrap();
    assert_eq!(inputs.visible_rows,1);
    assert!(!inputs.cells[0].hidden);
    println!("The same dynamics inspection reports zero controller rows but one visible input row after exact assertion hide");
}

#[test]
fn unsupported_root_weight_is_explicitly_refused_control() {
    use pictograph::canvas::dynamics_spec::{self,DynamicsSpec,Node};
    let mut node = Node::preset(pictograph::canvas::PhysicsLaw::Springs.id());
    let Node::Preset { weight, .. } = &mut node else { unreachable!() };
    *weight=2.0;
    assert!(dynamics_spec::bind(&DynamicsSpec::new(node)).is_err());
    println!("Ruled-out candidate: shared derive refuses an unsupported weighted root before Canvas binding");
}

#[test]
fn canceled_self_link_press_can_still_select_on_release() {
    use pictograph::canvas::PointerButton;
    let (mut graph,keys) = graph();
    graph.try_assert_semantic_statement(keys[0],keys[0],spec()).unwrap();
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_camera(CameraView {offset:(0.0,0.0),zoom:1.0});
    assert!(canvas.selected_link_keys().is_empty());
    // Midpoint of the Resource self-link arch for the default36px body.
    canvas.pointer_down(PointerButton::Left,9.0,-34.0);
    assert!(!canvas.pointer_cancel());
    canvas.pointer_up(PointerButton::Left,9.0,-34.0);
    assert_eq!(canvas.selected_link_keys().len(),1);
    println!("pointer_cancel misses self_link_press; it reports no changed interaction and the later release still selects the link");
}

#[test]
fn size_mutation_restarts_the_controller_while_canvas_stays_paused() {
    let (graph,keys) = graph();
    let id = graph.get_node(keys[0]).unwrap().id;
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_physics_paused(true);
    assert!(!canvas.is_settling());
    let before = canvas.field_clock().elapsed;
    canvas.set_node_size(id,80.0);
    assert!(canvas.physics_paused());
    assert!(canvas.is_settling());
    assert!(canvas.physics_tick_demand().1 > 0);
    canvas.frame(800,600);
    assert!(canvas.field_clock().elapsed > before);
    assert!(canvas.physics_paused());
    println!("A size mutation bypasses settle_physics, starts the backend budget and advances its clock while Canvas still reports paused");
}

#[test]
fn node_color_uses_a_stale_copy_of_the_canonical_palette() {
    let (graph,keys) = graph();
    let id = graph.get_node(keys[0]).unwrap().id;
    let mut canvas = Canvas::with_graph(graph);
    canvas.select_member(id);
    assert_eq!(canvas.node_color(keys[0]),"#f7a440");
    assert_eq!(pictograph::canvas::palette::SELECTED.bg,[232,150,40]);
    assert_ne!(pictograph::canvas::palette::SELECTED.bg,[247,164,64]);
    assert_eq!(canvas.node_state_color(keys[0]),"#5a8fc8");
    assert_eq!(pictograph::canvas::palette::IDLE.bg,[54,92,156]);
    println!("Canvas chrome-facing color accessors use old hex constants while the canonical palette/CSS declarations use different RGB values");
}

#[test]
fn selected_pair_unrelate_ignores_resource_semantics() {
    let (mut graph,keys) = graph();
    graph.try_assert_semantic_statement(keys[0],keys[1],spec()).unwrap();
    let ids = keys[..2].iter().map(|key| graph.get_node(*key).unwrap().id).collect::<Vec<_>>();
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_selected_members(&ids);
    assert_eq!(canvas.retract_selected_relation(),0);
    assert_eq!(canvas.graph().resource_relations().count(),1);
    // Explicitly addressing the same kind reaches the sanctioned Resource path.
    assert_eq!(canvas.retract_relation_between_members(ids[0],ids[1],
        kernel::graph::RelationSelector::Semantic(SemanticSubKind::Cites)),1);
    assert_eq!(canvas.graph().resource_relations().count(),0);
    println!("Pair Unrelate only enumerates Surface edge semantics; the same Cites is removable through the explicit member/selector action");
}
