use forme::*;
use std::collections::{HashMap,HashSet};

fn tree() -> GraphTree<u64> { GraphTree::new(LayoutMode::SplitPanes,ProjectionLens::Traversal) }
fn attach(t:&mut GraphTree<u64>, id:u64, parent:Option<u64>) {
    t.apply(NavAction::Attach {member:id, provenance:match parent {
        Some(source)=>Provenance::Traversal {source,edge_kind:None},None=>Provenance::Anchor}});
}
fn fixture(name:&str)->std::path::PathBuf {
    let p=std::env::current_dir().unwrap().join("support/receipts/authority-success-audit/forme-fixtures").join(name);
    assert!(!p.exists());std::fs::create_dir(&p).unwrap();p
}

#[test]
fn full_parity_promotes_unobserved_axes_to_matches() {
    let mut t=tree();attach(&mut t,1,None);attach(&mut t,2,Some(1));
    t.apply(NavAction::Activate(1));t.apply(NavAction::ToggleExpand(1));
    let mut snapshot=parity::ExternalTreeSnapshot {
        members:HashSet::from([1,2]),children:HashMap::new(),active:None,
        visible:HashSet::from([1]),expanded:HashSet::new(),visible_order:vec![]};
    let r=parity::compare(&t,&snapshot);
    assert!(r.full_match() && r.is_clean());
    snapshot.children.insert(1,vec![]); // Explicitly all-root topology now detects child2.
    assert!(!parity::compare(&t,&snapshot).structural_match());
    snapshot.children.insert(1,vec![2]);snapshot.active=Some(2);
    assert!(!parity::compare(&t,&snapshot).active_matches);
    println!("Full match true with topology/active/expansion/order unobserved; explicit differing observations refuse parity.");
}

#[test]
fn duplicate_reorder_breaks_topology_invariants() {
    let mut t=tree();attach(&mut t,1,None);attach(&mut t,2,Some(1));attach(&mut t,3,Some(1));
    t.topology().assert_invariants();
    assert!(t.apply(NavAction::Reorder {parent:1,new_order:vec![2,2]}).structure_changed);
    assert_eq!(t.children_of(&1),&[2,2,3]);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||t.topology().assert_invariants())).is_err());
    t.apply(NavAction::Reorder {parent:1,new_order:vec![3,2]});
    t.topology().assert_invariants();
    println!("Reorder [2,2] retains duplicate child; invariant assertion fails; unique reorder repairs it.");
}

#[test]
fn failed_store_save_stamps_unsaved_document() {
    let dir=fixture("stamp");let blocker=dir.join("blocked");std::fs::write(&blocker,b"occupied").unwrap();
    let graph=ArrangementNodeId::new().as_uuid();let mut d=FormeDocument::new(graph,None);
    let before=d.clone();assert!(store::save_forme(&blocker,&mut d,100).is_err());
    assert_eq!((d.created_at_ms,d.updated_at_ms),(100,100));assert_ne!(d,before);
    store::save_forme(&dir,&mut d,200).unwrap();let loaded=store::load_forme(&dir,d.id).unwrap().unwrap();
    assert_eq!((loaded.created_at_ms,loaded.updated_at_ms),(100,200));
    std::fs::remove_file(store::forme_path(&dir,d.id)).unwrap();std::fs::remove_dir(store::formes_dir(&dir)).unwrap();
    std::fs::remove_file(blocker).unwrap();std::fs::remove_dir(dir).unwrap();
    println!("Failed save stamps created/updated100; first successful save at200 persists created100.");
}

#[test]
fn load_by_id_accepts_another_documents_identity_and_list_accepts_arbitrary_filename() {
    let dir=fixture("identity");let graph=ArrangementNodeId::new().as_uuid();
    let mut a=FormeDocument::new(graph,Some("A".into()));let mut b=FormeDocument::new(graph,Some("B".into()));
    store::save_forme(&dir,&mut a,100).unwrap();store::save_forme(&dir,&mut b,200).unwrap();
    assert_eq!(store::load_forme(&dir,b.id).unwrap().unwrap().id,b.id);
    let a_path=store::forme_path(&dir,a.id);let b_path=store::forme_path(&dir,b.id);
    std::fs::copy(&a_path,&b_path).unwrap();
    assert_eq!(store::load_forme(&dir,b.id).unwrap().unwrap().id,a.id);
    std::fs::rename(&b_path,store::formes_dir(&dir).join("not-a-uuid.json")).unwrap();
    let all=store::load_all_formes(&dir).unwrap();assert_eq!(all.len(),2);assert!(all.iter().all(|d|d.id==a.id));
    std::fs::remove_file(a_path).unwrap();std::fs::remove_file(store::formes_dir(&dir).join("not-a-uuid.json")).unwrap();
    std::fs::remove_dir(store::formes_dir(&dir)).unwrap();std::fs::remove_dir(dir).unwrap();
    println!("Request B returns A from a mismatched valid file; arbitrary .json filename admits duplicate A into list.");
}

#[test]
fn cold_parent_hides_active_child_geometry_but_not_accessibility() {
    let mut t=tree();attach(&mut t,1,None);attach(&mut t,2,Some(1));t.apply(NavAction::Activate(2));
    let area=Rect::new(0.,0.,800.,600.);let layout=t.compute_layout(area);
    assert_eq!(layout.active,Some(2));assert!(layout.pane_rects.is_empty());
    assert!(layout.tab_order.iter().any(|tab|tab.member==2));
    assert!(emit_ux_tree(&t).children.iter().any(|n|n.member==Some(2)));
    t.apply(NavAction::SetLifecycle(1,Lifecycle::Warm));
    assert!(t.compute_layout(area).pane_rects.contains_key(&2));
    println!("Active child2 under Cold root has tab/UX Pane but no pane rect; Warm parent restores geometry.");
}

#[test]
fn nested_horizontal_subtree_displaces_outer_split_handle() {
    let mut t=tree();attach(&mut t,1,None);attach(&mut t,2,Some(1));attach(&mut t,3,None);
    for id in 1..=3 {t.apply(NavAction::SetLifecycle(id,Lifecycle::Active));}
    let mut override_value=LayoutOverride {min_width:None,min_height:None,flex_grow:None,flex_shrink:None,
        preferred_split:Some(SplitDirection::Horizontal),split_ratio:None};
    t.apply(NavAction::SetLayoutOverride(1,override_value.clone()));
    let area=Rect::new(0.,0.,1000.,600.);let layout=t.compute_layout(area);
    let left=layout.pane_rects.get(&2).unwrap();let right=layout.pane_rects.get(&3).unwrap();
    assert!((left.x+left.w-right.x).abs()<1.);
    let handle=layout.split_boundaries.iter().find(|b|b.before==1 && b.after==3).unwrap();
    assert!((handle.axis_position-right.x).abs()>10.);
    println!("Outer split handle at{}, actual subtree edge{}",handle.axis_position,right.x);
    override_value.preferred_split=Some(SplitDirection::Vertical);t.apply(NavAction::SetLayoutOverride(1,override_value));
    let control=t.compute_layout(area);let handle=control.split_boundaries.iter().find(|b|b.before==1 && b.after==3).unwrap();
    assert!((handle.axis_position-control.pane_rects.get(&3).unwrap().x).abs()<1.);
}
