use sprigging::{ColorF, GraphCanvas, GraphGlyphNode, LeafRegistry, PaintCmd, RenderedLeaves, SceneSlot, Size, Swatch};
fn size() -> Size { Size { width: 100.0, height: 60.0 } }

#[test]
fn replacing_scene_leaf_with_path_a_leaf_keeps_the_old_scene_instead() {
    let mut leaves = LeafRegistry::new();
    let mut cached = RenderedLeaves::new();
    let mut slot = SceneSlot::new(size());
    slot.set_scene(vello::Scene::new());
    leaves.insert(7, Box::new(slot));
    assert_eq!(leaves.render_into(|_| Some(size()), &mut cached), 1);
    assert_eq!(cached.scenes().count(), 1);
    leaves.insert(7, Box::new(Swatch::new(ColorF::WHITE, size())));
    assert_eq!(leaves.render_into(|_| Some(size()), &mut cached), 1);
    assert_eq!(cached.scenes().count(), 1, "replacement still carries retired leaf's scene");
    assert_eq!(cached.path_a_entries().count(), 0);
    assert!(matches!(cached.get(7).unwrap(), [PaintCmd::DrawExternalTexture(_)]));
}

#[test]
fn replacing_scene_slot_with_fresh_empty_slot_reuses_retired_content_without_paint() {
    let mut leaves = LeafRegistry::new();
    let mut cached = RenderedLeaves::new();
    let mut slot = SceneSlot::new(size());
    slot.set_scene(vello::Scene::new());
    leaves.insert(7, Box::new(slot));
    leaves.render_into(|_| Some(size()), &mut cached);
    leaves.remove(&7);
    leaves.insert(7, Box::new(SceneSlot::new(size())));
    assert_eq!(leaves.render_into(|_| Some(size()), &mut cached), 0);
    assert_eq!(cached.scenes().count(), 1);
}

#[test]
fn changing_public_graph_radius_moves_current_hit_position_but_keeps_old_paint() {
    let mut leaves = LeafRegistry::new();
    let mut cached = RenderedLeaves::new();
    leaves.insert(7, Box::new(GraphCanvas::new(vec![GraphGlyphNode { x: 0.0, y: 0.0, color: ColorF::WHITE }], vec![], size())));
    leaves.render_into(|_| Some(size()), &mut cached);
    let old_position = leaves.get_mut_as::<GraphCanvas>(&7).unwrap().node_local_position(0, size()).unwrap();
    let old_paint = format!("{:?}", cached.get(7).unwrap());
    let graph = leaves.get_mut_as::<GraphCanvas>(&7).unwrap();
    graph.node_radius += 10.0;
    let new_position = graph.node_local_position(0, size()).unwrap();
    assert_eq!(new_position, (old_position.0 + 10.0, old_position.1 + 10.0));
    assert_eq!(leaves.render_into(|_| Some(size()), &mut cached), 0);
    assert_eq!(format!("{:?}", cached.get(7).unwrap()), old_paint);
}
