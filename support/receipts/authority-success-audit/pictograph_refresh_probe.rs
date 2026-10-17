//! Audit observations against unchanged current public Canvas APIs.
use kernel::geometry::PortablePoint;
use kernel::graph::Graph;
use pictograph::canvas::Canvas;
use scenograph::attention::Attention;
use scenograph::suggestion::SuggestionSettings;

fn scene()->Canvas {
 let mut g=Graph::new();
 for (i,url) in ["https://a.test/page","https://b.test/page","https://held.test/one","https://held.test/two"].iter().enumerate() {
  kernel::graph::apply::add_node(&mut g,None,(*url).into(),PortablePoint::new(i as f32*80.0,0.0));
 }
 let mut c=Canvas::with_graph(g);c.resize(800,600);c.set_physics_paused(true);c
}
fn id(c:&Canvas,url:&str)->uuid::Uuid {c.graph().get_node_by_url(url).unwrap().1.id}
fn on(n:u32)->SuggestionSettings {SuggestionSettings{on:true,at_once:n,..Default::default()}}

#[test]
fn filtered_other_offers_consume_page_link_minting_budget() {
 let mut c=scene();let page=id(&c,"https://a.test/page");
 let existing=[id(&c,"https://held.test/one"),id(&c,"https://held.test/two")];
 c.offer_suggestions(existing.into_iter().map(|id|(id,"blocked-source".into())));
 let mut settings=on(2);settings.sources=vec!["links".into()];c.set_suggestion_settings(settings.clone());
 c.offer_page_links(page,vec!["https://new.test/one".into(),"https://new.test/two".into()]);
 assert!(c.derived_members().is_empty());assert!(c.members_with_attention(Attention::Suggestion).is_empty());
 assert_eq!(settings.admit([("one","links".into()),("two","links".into())],4,c.plate_fill()).len(),2);
 c.offer_suggestions(std::iter::empty());c.set_suggestion_settings(settings);
 assert_eq!(c.derived_members().len(),2);
 println!("OBSERVATION: two source-filtered offers reserve entire link mint budget; removing them admits both links");
}

#[test]
fn changing_other_offers_does_not_remove_derived_nodes_that_lose_admission() {
 let mut c=scene();let page=id(&c,"https://a.test/page");c.set_suggestion_settings(on(2));
 c.offer_page_links(page,vec!["https://new.test/one".into(),"https://new.test/two".into()]);
 let derived=c.derived_members();assert_eq!(derived.len(),2);
 let existing=[id(&c,"https://held.test/one"),id(&c,"https://held.test/two")];
 c.offer_suggestions(existing.into_iter().map(|id|(id,"other".into())));
 let admitted=c.members_with_attention(Attention::Suggestion);assert_eq!(admitted.len(),2);
 assert!(admitted.iter().all(|id| !derived.contains(id)));assert_eq!(c.derived_members(),derived);
 assert!(derived.iter().all(|id| c.graph().get_node_by_id(*id).is_some()));
 println!("OBSERVATION: two minted links remain real graph members after both lose suggestion admission to new offers");
}

#[test]
fn changing_offer_origin_leaves_old_edge_and_kept_provenance() {
 let mut c=scene();let a=id(&c,"https://a.test/page");let b=id(&c,"https://b.test/page");
 let url="https://new.test/shared".to_string();c.set_suggestion_settings(on(2));
 c.offer_page_links(a,vec![url.clone()]);c.offer_page_links(b,vec![url]);
 let derived=c.derived_members()[0];c.offer_page_links(a,Vec::<String>::new());
 let ak=c.graph().get_node_key_by_id(a).unwrap();let bk=c.graph().get_node_key_by_id(b).unwrap();let dk=c.graph().get_node_key_by_id(derived).unwrap();
 assert!(c.graph().projected_relations().any(|(_,r)|r.from==ak&&r.to==dk));
 assert!(!c.graph().projected_relations().any(|(_,r)|r.from==bk&&r.to==dk));
 c.select_member(derived);c.frame(800,600);let kept=c.take_kept_links();assert_eq!(kept.len(),1);assert_eq!(kept[0].origin,a);
 println!("OBSERVATION: only page B still offers shared URL, but derived edge and kept-link origin still name page A");
}

#[test]
fn pending_kept_link_survives_wholesale_graph_switch() {
 let mut c=scene();let a=id(&c,"https://a.test/page");c.set_suggestion_settings(on(1));
 c.offer_page_links(a,vec!["https://new.test/kept".into()]);let derived=c.derived_members()[0];
 c.select_member(derived);c.frame(800,600);
 let mut replacement=Graph::new();kernel::graph::apply::add_node(&mut replacement,None,"https://different-space.test/".into(),PortablePoint::zero());
 c.set_graph(replacement);assert!(c.graph().get_node_by_id(a).is_none());
 let pending=c.take_kept_links();assert_eq!(pending.len(),1);assert_eq!(pending[0].origin,a);assert_eq!(pending[0].member,derived);
 println!("OBSERVATION: old graph's pending keep intent is returned after a different graph replaces it; host writes not executed");
}

#[test]
fn tiny_plate_mints_a_link_that_immediately_is_not_admitted() {
 let mut g=Graph::new();let k=kernel::graph::apply::add_node(&mut g,None,"https://a.test/page".into(),PortablePoint::zero());let page=g.get_node(k).unwrap().id;
 let mut c=Canvas::with_graph(g);c.resize(60,60);c.set_physics_paused(true);c.set_suggestion_settings(on(1));
 let before=c.suggestion_settings().budget(1,c.plate_fill());assert_eq!(before,1);
 c.offer_page_links(page,vec!["https://new.test/one".into()]);assert_eq!(c.derived_members().len(),1);
 assert!(c.members_with_attention(Attention::Suggestion).is_empty());
 println!("OBSERVATION: link minted under pre-mint plate budget is immediately not admitted by post-mint plate budget");
}
