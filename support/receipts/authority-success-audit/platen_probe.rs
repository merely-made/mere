use forme::{Arrangement,GraphMemberId};
use platen::{TileLayout,TreeGeometry,TreeBranch,Axis,PlanSlot,project_tree};
use uuid::Uuid;
fn m(n:u128)->GraphMemberId {Uuid::from_u128(n)}
fn split(fractions:&[f32])->TreeGeometry {
    TreeGeometry::Split {axis:Axis::Row,children:fractions.iter().enumerate().map(|(i,&fraction)|TreeBranch {fraction,node:TreeGeometry::leaf(m(i as u128+1))}).collect()}
}
fn shares(g:&TreeGeometry)->Vec<f32> {match g {TreeGeometry::Split {children,..}=>children.iter().map(|b|b.fraction).collect(),_=>panic!("split expected")}}

#[test]
fn sanitizer_and_live_normalizer_leave_invalid_shares_and_persist_can_panic() {
    let mut negative=split(&[-1.,2.]);negative.sanitize();assert_eq!(shares(&negative),vec![-1.,2.]);
    let mut huge=split(&[f32::MAX,f32::MAX]);huge.sanitize();assert_eq!(shares(&huge),vec![0.,0.]);
    let mut ordinary=split(&[3.,1.]);ordinary.sanitize();assert_eq!(shares(&ordinary),vec![0.75,0.25]);
    let mut layout=TileLayout::new();layout.open_split(&[m(1),m(2)]);layout.set_weights(&[f32::MAX,f32::MAX]);
    assert_eq!(layout.weights(),vec![0.,0.]);assert!(!layout.canonical_roundtrips());
    assert!(std::panic::catch_unwind(||layout.to_persisted_json()).is_err());
    layout.set_weights(&[3.,1.]);assert_eq!(layout.weights(),vec![0.75,0.25]);assert!(layout.to_persisted_json().is_ok());
    println!("sanitize leaves[-1,2]; finite MAX shares become[0,0] in both paths; debug fallible persist panics, ordinary weights serialize.");
}

#[test]
fn failed_open_into_missing_target_removes_existing_member() {
    let mut layout=TileLayout::new();layout.open_stack(&[m(1),m(2)]);layout.activate(m(2));
    assert!(!layout.open_in_slot_of(m(2),m(99)));assert_eq!(layout.open_members(),vec![m(1)]);
    let mut control=TileLayout::new();control.open_stack(&[m(1),m(2)]);
    assert!(!control.move_to_slot_of(m(2),m(99)));assert_eq!(control.open_members(),vec![m(1),m(2)]);
    assert!(control.open_in_slot_of(m(2),m(1)));assert!(control.has_tile(m(2)));
    println!("open_in_slot_of returns false and removes held member2 for unknown target99; guarded move control preserves it.");
}

#[test]
fn late_bridge_does_not_merge_previously_placed_stacks() {
    let mut arr=Arrangement::new();let a=arr.add_member_intent(m(1));let b=arr.add_member_intent(m(2));let c=arr.add_member_intent(m(3));
    arr.stack(a,c);arr.stack(b,c);let plan=project_tree(&arr);
    assert_eq!(plan.slots.len(),2);assert_eq!(plan.tile_count(),3);
    let groups:Vec<Vec<_>>=plan.slots.iter().map(|s|match s {PlanSlot::Tile(t)=>vec![t.node],PlanSlot::Tabs(v)=>v.iter().map(|t|t.node).collect()}).collect();
    assert!(!groups.iter().any(|g|g.contains(&b)&&g.contains(&c)));
    let mut chain=Arrangement::new();let a=chain.add_member_intent(m(1));let b=chain.add_member_intent(m(2));let c=chain.add_member_intent(m(3));chain.stack(a,b);chain.stack(b,c);
    assert_eq!(project_tree(&chain).slots.len(),1);
    println!("Edges1-3 and2-3 project to two slots, violating2-3 grouping; insertion-aligned chain projects to one.");
}

#[test]
fn geometry_overrides_arrangement_members_and_stacking_without_a_refusal() {
    let mut arr=Arrangement::new();let a=arr.add_member_intent(m(1));let b=arr.add_member_intent(m(2));arr.stack(a,b);
    let stale=TreeGeometry::leaf(m(1));let missing=TileLayout::from_arrangement(&arr,Some(&stale));assert_eq!(missing.open_members(),vec![m(1)]);
    let geometry=split(&[0.5,0.5]);let changed=TileLayout::from_arrangement(&arr,Some(&geometry));
    assert_eq!(changed.slot_count(),2);assert_eq!(changed.tile_count(),2);assert!(changed.canonical_roundtrips());
    let control=TileLayout::from_arrangement(&arr,None);assert_eq!(control.slot_count(),1);assert_eq!(control.tile_count(),2);
    println!("Same arrangement1 StackedWith2: stale geometry drops2; split geometry yields two cells while no geometry yields one. Self-roundtrip still true.");
}

#[test]
fn closing_an_inactive_earlier_tab_changes_the_active_member() {
    let mut layout=TileLayout::new();layout.open_stack(&[m(1),m(2),m(3)]);layout.activate(m(2));
    assert!(layout.close_tile(m(1)));let view=layout.slot_views().next().unwrap();assert_eq!(view.members[view.active],m(3));
    let mut control=TileLayout::new();control.open_stack(&[m(1),m(2),m(3)]);control.activate(m(3));control.close_tile(m(1));let view=control.slot_views().next().unwrap();assert_eq!(view.members[view.active],m(3));
    println!("Closing inactive1 from[1,2,3] with active2 switches active to3; last-active clamp control retains3.");
}

#[test]
fn duplicate_geometry_members_survive_admission_close_and_canonical_check() {
    let mut arr=Arrangement::new();arr.add_member_intent(m(1));
    let geom=TreeGeometry::Stack {members:vec![m(1),m(1)],active:0};
    let mut layout=TileLayout::from_arrangement(&arr,Some(&geom));
    assert_eq!(layout.tile_count(),2);assert!(layout.canonical_roundtrips());
    let ux=platen::project_tile_layout(&layout);let unique:std::collections::HashSet<_>=ux.nodes.iter().map(|(id,_)|*id).collect();
    assert!(unique.len()<ux.nodes.len());
    assert!(layout.close_tile(m(1)));assert!(layout.has_tile(m(1)));
    let mut control=TileLayout::new();control.open_stack(&[m(1),m(1)]);assert_eq!(control.tile_count(),1);
    println!("Loaded duplicate members pass canonical check, create duplicate AccessKit IDs, and one remains after close; normal open_stack deduplicates.");
}

fn tile(id:u64)->workbench::Tile {
    workbench::Tile {id:workbench::TileId(id),title:format!("tile{id}"),
        content:workbench::ContentSource::Open {kind:"audit".into(),id:id.to_string()},accent:None}
}

#[test]
fn shared_reducer_reports_applied_after_self_edge_drop_deletes_the_tile() {
    use workbench::*;
    let mut retained=Workbench::new(TileTree::stack(vec![tile(1),tile(2)],0));
    let outcome=retained.apply(&TileEvent::Dragged {tile:TileId(1),to:DropTarget::Edge {tile:TileId(1),edge:Edge::Right}});
    assert_eq!(outcome,WorkbenchOutcome::Applied);assert!(retained.tree().find(TileId(1)).is_none());
    assert_eq!(retained.tree().tiles().len(),1);
    let mut control=Workbench::new(TileTree::stack(vec![tile(1),tile(2)],0));
    assert_eq!(control.apply(&TileEvent::Dragged {tile:TileId(1),to:DropTarget::Edge {tile:TileId(99),edge:Edge::Right}}),WorkbenchOutcome::Unchanged);
    assert_eq!(control.tree().tiles().len(),2);
    assert_eq!(control.apply(&TileEvent::Dragged {tile:TileId(1),to:DropTarget::Edge {tile:TileId(2),edge:Edge::Right}}),WorkbenchOutcome::Applied);
    assert_eq!(control.tree().tiles().len(),2);
    println!("Shared reducer returns Applied after dropping1 onto its own edge deletes1; unknown target is Unchanged; other-tile edge preserves both.");
}

#[test]
fn shared_reducer_repeats_platens_active_index_loss() {
    use workbench::*;
    let mut t=TileTree::stack(vec![tile(1),tile(2),tile(3)],1);
    assert!(t.apply(&TileEvent::Closed(TileId(1))));
    let TileTree::Stack(s)=t else {panic!("stack expected")};assert_eq!(s.tabs[s.active].id,TileId(3));
    println!("Shared Workbench reducer also switches active2 to3 when inactive1 closes.");
}

#[test]
fn float_resolve_does_not_sanitize_nonfinite_host_area() {
    use workbench::*;
    let f=FloatingTile {tile:tile(1),rect:RelativeRect {x:0.25,y:0.25,width:0.5,height:0.5},
        constraints:FloatSizeConstraints::default(),z:1,pinned:false,visible:true};
    assert_eq!(f.resolve((0.,0.,400.,400.)),(100.,100.,200.,200.));
    assert!(std::panic::catch_unwind(||f.resolve((f32::NAN,0.,400.,400.))).is_err());
    let (x,_,_,_)=f.resolve((0.,0.,f32::NAN,400.));assert!(x.is_nan());
    let bad_rect=FloatingTile {rect:RelativeRect {x:f32::NAN,y:0.,width:f32::INFINITY,height:0.5},..f};
    assert_eq!(bad_rect.resolve((0.,0.,400.,400.)),(0.,0.,0.,200.));
    println!("Nonfinite host x panics and host width yields NaN despite nonfinite-input promise; nonfinite rect control is sanitized.");
}
