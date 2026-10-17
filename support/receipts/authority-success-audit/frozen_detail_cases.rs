use audit_graphshell_client::frozen::FrozenScene;
use sceno::{Scene,SourceRef,InstanceId,ProjectedItem,Transform2,Footprint,Representation};
use scenotime::{SceneSnapshot,SceneEpoch,Revision};
use std::collections::HashMap;
fn frozen()->FrozenScene{
 let mut scene=Scene::new();
 for (i,id) in ["plain","hidden-a","hidden-b"].iter().enumerate(){
  let source=scene.intern_source(SourceRef::new("fixture.details",*id));
  scene.items.push(ProjectedItem{source,space:Scene::WORLD,transform:Transform2::translation(i as f32*20.0,0.0),footprint:Footprint::Point,representation:Representation::Card,layer:0,visible:true,hit:None,channels:vec![]});
 }
 scene.folds.push(sceno::Fold{members:vec![InstanceId(1),InstanceId(2)],stand_in:sceno::StandIn::Summary{label:Some("Two hidden records".into())},rule:None,boundary:None,label:None});
 let snapshot=SceneSnapshot::from_dense(SceneEpoch(1),Revision(0),scene).unwrap();snapshot.validate().unwrap();
 let names=HashMap::from([(InstanceId(0),"Plain record".into()),(InstanceId(1),"Hidden A".into()),(InstanceId(2),"Hidden B".into())]);
 let details=HashMap::from([(InstanceId(0),"PLAIN_FACT_2026_07_24".into()),(InstanceId(1),"FOLDED_FACT_2026_08_13".into())]);
 FrozenScene::freeze_snapshot_with_details(&snapshot,"Readable facts",&names,&details)
}
#[test]fn flat_html_and_rows_preserve_the_supplied_fact(){
 let f=frozen();assert_eq!(f.instances[0].detail.as_deref(),Some("PLAIN_FACT_2026_07_24"));
 assert!(f.to_html("facts").contains("PLAIN_FACT_2026_07_24"));
 assert!(f.rows().iter().any(|(_,_,d)|d.contains("PLAIN_FACT_2026_07_24")));
}
#[test]fn accesskit_tree_discards_the_flat_fact_that_html_exposes(){
 let f=frozen();let tree=f.to_ux_tree("facts");
 assert!(tree.nodes.iter().any(|(_,n)|n.label()==Some("Plain record")));
 assert!(tree.nodes.iter().all(|(_,n)| !n.label().unwrap_or("").contains("PLAIN_FACT_2026_07_24") && !n.description().unwrap_or("").contains("PLAIN_FACT_2026_07_24")));
 assert!(f.to_html("facts").contains("PLAIN_FACT_2026_07_24"));
}
#[test]fn folding_hides_readable_facts_from_every_frozen_output(){
 let f=frozen();let member=f.folds.iter().flat_map(|x|x.members.iter()).find(|i|i.name=="Hidden A").unwrap();
 assert_eq!(member.detail.as_deref(),Some("FOLDED_FACT_2026_08_13"));
 let html=f.to_html("facts");assert!(html.contains("Hidden A"));assert!(!html.contains("FOLDED_FACT_2026_08_13"));
 assert!(f.rows().iter().all(|(_,_,d)|!d.contains("FOLDED_FACT_2026_08_13")));
 let tree=f.to_ux_tree("facts");assert!(tree.nodes.iter().any(|(_,n)|n.label()==Some("Hidden A")));
 assert!(tree.nodes.iter().all(|(_,n)|!n.label().unwrap_or("").contains("FOLDED_FACT_2026_08_13")&&!n.description().unwrap_or("").contains("FOLDED_FACT_2026_08_13")));
}
