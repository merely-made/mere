use cambium::*;
use genet_scripted_dom::{NodeId,ScriptedDom};
use layout_dom_api::{LayoutDom,LocalName,Namespace};
use std::{cell::RefCell,rc::Rc};
struct App { swatch:GraphCanvasSwatch<u8,()>,focused:Option<u8>,activated:Vec<u8>,relations:Vec<String>,expanded:bool }
fn app()->App {App{swatch:GraphCanvasSwatch::new(1,GraphCanvasSubgraph{nodes:[(1,0.2),(2,0.8)].into_iter().map(|(id,x)|GraphCanvasNode{id,kind:(),position:(x,0.5),label:format!("node {id}"),key:Some(format!("node-{id}"))}).collect(),edges:vec![]}).with_relations(vec![GraphCanvasRelation{id:"relation".into(),link_key:None,marks:Default::default(),strength:1.,from:1,to:2,kind:"link".into(),label:"relation".into(),route:vec![],visible:true,emphasized:false}]),focused:None,activated:vec![],relations:vec![],expanded:false}}
type Child=Box<dyn AnyView<App,(),GenetCtx,GenetElement>>;
fn low(s:&App)->Child {let mut swatch=s.swatch.clone();swatch.focus=s.focused;Box::new(graph_canvas_swatch_with_focus_and_drag_and_relations(&swatch,|s:&mut App,id|s.activated.push(id),|_:&mut App,_|{},|s:&mut App,id|s.focused=id,|_:&mut App,_|{},|s:&mut App,id|s.relations.push(id),|_:&mut App,_|{},|s:&mut App|s.expanded=true,|_:&mut App,_|{},|_:&mut App,_|{}))}
fn high(s:&App)->Child {Box::new(graph_canvas(&s.swatch,|s:&mut App,e|match e{GraphCanvasEvent::Activate(id)=>s.activated.push(id),GraphCanvasEvent::RelationActivate(id)=>s.relations.push(id),GraphCanvasEvent::Expand=>s.expanded=true,_=>{}}))}
fn dom()->DomHandle{Rc::new(RefCell::new(ScriptedDom::new()))}
fn attr<'a>(d:&'a ScriptedDom,n:NodeId,k:&str)->Option<&'a str>{d.attribute(n,&Namespace::from(""),&LocalName::from(k))}
fn find(d:&ScriptedDom,n:NodeId,k:&str,v:&str)->Option<NodeId>{if attr(d,n,k)==Some(v){return Some(n)}d.dom_children(n).find_map(|c|find(d,c,k,v))}
fn main(){
 for (name,view) in [("callback",low as fn(&App)->Child),("component",high as fn(&App)->Child)]{
  let d=dom();let mut r=GenetAppRunner::new(d.clone(),view,app());let one=find(&d.borrow(),r.root(),"data-key","node-1").unwrap();
  r.set_focus(Some(one));assert_eq!(r.focus(),Some(one));assert!(attr(&d.borrow(),one,"class").unwrap().split_whitespace().any(|c|c=="focused"));println!("PASS {name} initial logical node focus control");
  r.update(|s|s.swatch.graph.nodes.swap(0,1));let new_one=find(&d.borrow(),r.root(),"data-key","node-1").unwrap();let two=find(&d.borrow(),r.root(),"data-key","node-2").unwrap();
  assert_eq!(one,two);assert_ne!(one,new_one);assert_eq!(r.focus(),Some(two));assert!(attr(&d.borrow(),new_one,"class").unwrap().split_whitespace().any(|c|c=="focused"));assert!(!attr(&d.borrow(),two,"class").unwrap().split_whitespace().any(|c|c=="focused"));
  r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));assert_eq!(r.state().activated,[2]);println!("PASS {name} reorder keeps old DOM focus on node 2 while node 1 retains focused class; Enter activates 2");
  r.set_focus(None);r.set_focus(Some(new_one));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));assert_eq!(r.state().activated,[2,1]);println!("PASS {name} explicit focus repair control activates logical node 1");
 }
 let d=dom();let mut r=GenetAppRunner::new(d.clone(),low,app());let one=find(&d.borrow(),r.root(),"data-key","node-1").unwrap();let two=find(&d.borrow(),r.root(),"data-key","node-2").unwrap();let relation=find(&d.borrow(),r.root(),"data-relation-id","relation").unwrap();let expand=find(&d.borrow(),r.root(),"aria-label","Expand graph").unwrap();
 let seats=r.focusables();assert!(seats.contains(&one)&&seats.contains(&two));assert!(!seats.contains(&relation)&&!seats.contains(&expand));r.set_focus(Some(one));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));assert_eq!(r.focus(),Some(two));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));assert_eq!(r.focus(),Some(one));println!("PASS relation and Expand buttons are absent from Tab seats while node buttons cycle normally");
 r.dispatch_click(relation,PointerClick::at((1.,1.)));assert_eq!(r.state().relations,["relation"]);println!("PASS relation pointer activation control invokes its callback");
 r.dispatch_click(expand,PointerClick::at((1.,1.)));assert!(r.state().expanded);println!("PASS Expand pointer activation control invokes its callback");
}
