//! CPU layout/hit-test observations, not raster acceptance or production repairs.
use kernel::geometry::PortablePoint;
use kernel::graph::{Graph,NodeKey,EdgeAssertion,SemanticSubKind};
use pictograph::canvas::{Canvas,CameraView,PointerButton};

fn raised() -> (Canvas,NodeKey) {
    let mut graph=Graph::new();
    let keys=(0..4).map(|i| kernel::graph::apply::add_node(&mut graph,None,
        format!("https://height-audit-{i}.test/"),PortablePoint::zero())).collect::<Vec<_>>();
    for &key in &keys[1..] {
        kernel::graph::apply::assert_relation(&mut graph,keys[0],key,
            EdgeAssertion::Semantic {sub_kind:SemanticSubKind::UserGrouped,
                label:None,decay_progress:None});
    }
    let placements=keys.iter().enumerate().map(|(i,&key)|
        (graph.get_node(key).unwrap().id,(i as f32*200.0,0.0))).collect::<Vec<_>>();
    let mut canvas=Canvas::with_graph(graph);
    canvas.seed_cartography(placements);
    canvas.set_physics_paused(true);
    canvas.set_height_by_degree(true);
    (canvas,keys[0])
}
fn center(rect:(f32,f32,f32,f32))->(f32,f32) {
    ((rect.0+rect.2)*0.5,(rect.1+rect.3)*0.5)
}

#[test]
fn face_and_body_separate_by_the_degree_lift_at_each_zoom() {
    let (mut canvas,key)=raised();
    assert_eq!(canvas.node_height(key),48.0);
    for zoom in [0.5,1.0,2.0] {
        canvas.set_camera(CameraView {offset:(150.0,200.0),zoom});
        canvas.frame(800,600);
        let body=center(canvas.node_body_rect(key).expect("CPU Livery body hit"));
        let face=center(canvas.node_face_rect(key).expect("derived face paint rect"));
        let anchor=canvas.screen_position_of(key).unwrap();
        assert!((body.1-(anchor.1-48.0*zoom)).abs()<0.02);
        assert!((face.1-anchor.1).abs()<0.02);
        assert!((face.1-body.1-48.0*zoom).abs()<0.02);
        // Same graph/camera with height off is the aligned control.
        canvas.set_height_by_degree(false);
        canvas.frame(800,600);
        let body=center(canvas.node_body_rect(key).unwrap());
        let face=center(canvas.node_face_rect(key).unwrap());
        assert!((body.1-face.1).abs()<0.02);
        canvas.set_height_by_degree(true);
    }
    println!("Degree3 lifts the CPU body24/48/96px at zoom0.5/1/2 while the face stays on the ground anchor; height-off controls align.");
}

#[test]
fn face_center_misses_the_raised_nodes_pick_while_body_center_selects_it() {
    let (mut canvas,key)=raised();
    canvas.set_camera(CameraView {offset:(150.0,200.0),zoom:1.0});
    canvas.frame(800,600);
    let face=center(canvas.node_face_rect(key).unwrap());
    let body=center(canvas.node_body_rect(key).unwrap());
    canvas.pointer_down(PointerButton::Left,face.0,face.1);
    canvas.pointer_up(PointerButton::Left,face.0,face.1);
    assert!(canvas.selected_members().is_empty());
    canvas.pointer_down(PointerButton::Left,body.0,body.1);
    canvas.pointer_up(PointerButton::Left,body.0,body.1);
    assert_eq!(canvas.selected_members(),vec![canvas.graph().get_node(key).unwrap().id]);
    println!("Clicking the generated face center selects nothing; clicking its raised CPU body center selects the node.");
}

#[test]
fn suggestion_body_shrinks_while_its_derived_face_keeps_the_full_size() {
    let (mut canvas,key)=raised();
    canvas.set_height_by_degree(false);
    canvas.set_camera(CameraView {offset:(150.0,200.0),zoom:1.0});
    canvas.set_graph_grammar(true);
    canvas.set_suggestion_settings(scenograph::suggestion::SuggestionSettings {
        on:true,..Default::default()
    });
    let id=canvas.graph().get_node(key).unwrap().id;
    canvas.set_suggested_members([id]);
    canvas.frame(800,600);
    let body=canvas.node_body_rect(key).unwrap();
    let face=canvas.node_face_rect(key).unwrap();
    let body_width=body.2-body.0;
    let face_width=face.2-face.0;
    assert!((body_width-16.2).abs()<0.02);
    assert!((face_width-25.92).abs()<0.02);
    assert!(face_width>body_width);
    // Turning the grammar off restores the intended face inset.
    canvas.set_graph_grammar(false);
    canvas.frame(800,600);
    let body=canvas.node_body_rect(key).unwrap();
    let face=canvas.node_face_rect(key).unwrap();
    let ratio=(face.2-face.0)/(body.2-body.0);
    assert!((ratio-0.72).abs()<0.002);
    println!("Suggestion body shrinks to{body_width:.2}px but its derived face stays{face_width:.2}px; grammar-off control restores0.72 inset.");
}
