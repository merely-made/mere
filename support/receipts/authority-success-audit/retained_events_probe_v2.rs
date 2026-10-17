use cambium::*;
use genet_scripted_dom::{NodeId,ScriptedDom};
use layout_dom_api::{LayoutDom,LocalName,Namespace};
use std::{cell::RefCell,rc::Rc};
use workbench::{TileTree,TileBranch,TabStack,Tile,TileId,TileEvent,TilePath,SplitAxis,DropTarget,ContentSource,DocumentRef};
fn dom()->DomHandle{Rc::new(RefCell::new(ScriptedDom::new()))}
fn attr<'a>(d:&'a ScriptedDom,n:NodeId,k:&str)->Option<&'a str>{d.attribute(n,&Namespace::from(""),&LocalName::from(k))}
fn find(d:&ScriptedDom,n:NodeId,k:&str,v:&str)->Option<NodeId>{if attr(d,n,k)==Some(v){return Some(n)}d.dom_children(n).find_map(|c|find(d,c,k,v))}
#[derive(Default)]struct Events{capture:bool,log:Vec<&'static str>}
type EventView=Box<dyn AnyView<Events,(),GenetCtx,GenetElement>>;
fn click_view(s:&Events)->EventView{Box::new(on_click(el("div",on_click(el::<_,Events,()>("button","child"),|s:&mut Events,_|s.log.push("child"))),|s:&mut Events,_|s.log.push("parent")).capture(s.capture))}
fn key_view(s:&Events)->EventView{Box::new(on_key(el("div",on_key(el::<_,Events,()>("button","child"),|s:&mut Events,_|s.log.push("child"))),|s:&mut Events,_|s.log.push("parent")).capture(s.capture).focusable(false))}
fn focus_view(_: &Events)->EventView{Box::new(focusable(on_focus(on_focus(el::<_,Events,()>("button","focus"),|s:&mut Events,_|s.log.push("inner")),|s:&mut Events,_|s.log.push("outer"))))}
fn tile(id:u64)->Tile{Tile{id:TileId(id),title:format!("tile {id}"),content:ContentSource::Document(DocumentRef(format!("doc:{id}"))),accent:None}}
struct Frame{tree:TileTree}
type FrameView=Box<dyn AnyView<Frame,(),GenetCtx,GenetElement>>;
fn fill(t:&Tile)->Slot<Frame,()>{Slot::View(Box::new(component(t.id.0,|_:&u64|0u32,|_:&u64,_:&u64,_:&mut u32|{},|id:&u64,count:&u32|->ComponentView<u32,()>{Box::new(button(format!("{id}: {count}"),|count:&mut u32,_|*count+=1).attr("data-count",count.to_string()))},|_:&mut Frame,_:()|{})))}
fn frame_view(s:&Frame)->FrameView{Box::new(frisket_with(&s.tree,|s:&mut Frame,e|{s.tree.apply(&e);},fill))}
fn frame()->Frame{Frame{tree:TileTree::Split{axis:SplitAxis::Row,children:vec![TileBranch{fraction:0.5,tree:TileTree::Stack(TabStack{tabs:vec![tile(1),tile(2)],active:0})},TileBranch{fraction:0.5,tree:TileTree::Stack(TabStack{tabs:vec![tile(3),tile(4)],active:0})}]}}}

struct Stop { mode:u8,log:Vec<&'static str> }
type StopView=Box<dyn AnyView<Stop,(),GenetCtx,GenetElement>>;
fn stop_click(_: &Stop)->StopView { Box::new(on_click(el("div",on_click(on_click(el::<_,Stop,()>("button","child"),|s:&mut Stop,e:PointerClick|{s.log.push("inner");match s.mode{1=>e.stop_propagation(),2=>e.stop_immediate_propagation(),_=>{}}}),|s:&mut Stop,_|s.log.push("sibling"))),|s:&mut Stop,_|s.log.push("ancestor"))) }
fn stop_key(_: &Stop)->StopView { Box::new(on_key(el("div",on_key(on_key(el::<_,Stop,()>("button","child"),|s:&mut Stop,e:KeyEvent|{s.log.push("inner");match s.mode{1=>e.stop_propagation(),2=>e.stop_immediate_propagation(),_=>{}}}),|s:&mut Stop,_|s.log.push("sibling"))),|s:&mut Stop,_|s.log.push("ancestor")).focusable(false)) }
#[derive(Default)]struct Requested { request:bool,log:Vec<FocusPhase> }
fn requested_view(s:&Requested)->Box<dyn AnyView<Requested,(),GenetCtx,GenetElement>> {Box::new(on_focus(request_focus(focusable(el::<_,Requested,()>("button","focus")),s.request),|s:&mut Requested,e:FocusEvent|s.log.push(e.phase)))}
struct Hidden{ disclosure:DisclosureState,presses:u32 }
fn hidden_view(s:&Hidden)->Box<dyn AnyView<Hidden,(),GenetCtx,GenetElement>> {Box::new(disclosure(&s.disclosure,button("hidden action",|s:&mut Hidden,_|s.presses+=1).attr("id","hidden-action"),|s:&mut Hidden|s.disclosure.toggle()))}

fn main(){
 let d=dom();let mut r=GenetAppRunner::new(d.clone(),click_view,Events::default());let child=d.borrow().dom_children(r.root()).next().unwrap();
 r.dispatch_click(child,PointerClick::at((1.,1.)));assert_eq!(r.state().log,["child","parent"]);println!("PASS initial bubble click order control");
 r.update(|s|{s.capture=true;s.log.clear()});assert_eq!(d.borrow().dom_children(r.root()).next(),Some(child));
 r.dispatch_click(child,PointerClick::at((1.,1.)));assert_eq!(r.state().log,["child","parent"]);println!("PASS retained click listener ignores capture change and stays in bubble phase");
 let d=dom();let mut r=GenetAppRunner::new(d.clone(),click_view,Events{capture:true,..Default::default()});let child=d.borrow().dom_children(r.root()).next().unwrap();
 r.dispatch_click(child,PointerClick::at((1.,1.)));assert_eq!(r.state().log,["parent","child"]);println!("PASS initially captured click listener control uses capture phase");
 let d=dom();let mut r=GenetAppRunner::new(d.clone(),key_view,Events::default());let child=d.borrow().dom_children(r.root()).next().unwrap();r.set_focus(Some(child));
 r.dispatch_key(KeyEvent::new(Key::Character("x".into())));assert_eq!(r.state().log,["child","parent"]);
 r.update(|s|{s.capture=true;s.log.clear()});r.dispatch_key(KeyEvent::new(Key::Character("x".into())));assert_eq!(r.state().log,["parent","child"]);println!("PASS retained key listener control correctly updates capture phase");
 let mut r=GenetAppRunner::new(dom(),focus_view,Events::default());r.set_focus(Some(r.root()));r.set_focus(None);assert_eq!(r.state().log,["outer","outer"]);println!("PASS nested same-element focus observers silently retain only outer handler");
 for (id,dest,preserved) in [(1,1,true),(3,0,false)]{
  let d=dom();let mut r=GenetAppRunner::new(d.clone(),frame_view,frame());
  let old={let d=d.borrow();let content=find(&d,r.root(),FRISKET_TILE_ATTR,&id.to_string()).unwrap();d.dom_children(content).next().unwrap()};
  r.dispatch_click(old,PointerClick::at((1.,1.)));assert_eq!(attr(&d.borrow(),old,"data-count"),Some("1"));
  r.update(|s|{assert!(s.tree.apply(&TileEvent::Dragged{tile:TileId(id),to:DropTarget::Stack{stack:TilePath(vec![dest]),index:0}}))});
  let d=d.borrow();let content=find(&d,r.root(),FRISKET_TILE_ATTR,&id.to_string()).unwrap();let new=d.dom_children(content).next().unwrap();
  if preserved{assert_eq!(old,new);assert_eq!(attr(&d,new,"data-count"),Some("1"));println!("PASS earlier-to-later Frisket move preserves node and component local count");}
  else{assert_ne!(old,new);assert!(!d.is_live(old));assert_eq!(attr(&d,new,"data-count"),Some("0"));println!("PASS later-to-earlier Frisket move retires node and resets component local count");}
 }
 for mode in 0..3 {
  let d=dom();let mut r=GenetAppRunner::new(d.clone(),stop_click,Stop{mode,log:vec![]});let child=d.borrow().dom_children(r.root()).next().unwrap();r.dispatch_click(child,PointerClick::at((1.,1.)));
  assert_eq!(r.state().log,if mode==0{vec!["inner","sibling","ancestor"]}else{vec!["inner"]});
  println!("PASS click stop-mode {mode}: observed {:?}",r.state().log);
  let d=dom();let mut r=GenetAppRunner::new(d.clone(),stop_key,Stop{mode,log:vec![]});let child=d.borrow().dom_children(r.root()).next().unwrap();r.set_focus(Some(child));r.dispatch_key(KeyEvent::new(Key::Character("x".into())));
  assert_eq!(r.state().log,if mode==0{vec!["inner","sibling","ancestor"]}else{vec!["inner"]});
  println!("PASS key stop-mode {mode}: observed {:?}",r.state().log);
 }
 let mut r=GenetAppRunner::new(dom(),requested_view,Requested::default());assert!(r.focus().is_none());r.update(|s|s.request=true);assert_eq!(r.focus(),Some(r.root()));assert!(r.state().log.is_empty());
 println!("PASS requested programmatic focus changes the seat without delivering Gained");
 r.set_focus(None);r.set_focus(Some(r.root()));assert_eq!(r.state().log,[FocusPhase::Lost,FocusPhase::Gained]);
 println!("PASS direct focus setter control delivers Lost and Gained");
 let d=dom();let mut r=GenetAppRunner::new(d.clone(),hidden_view,Hidden{disclosure:DisclosureState::new("probe-disclosure","Details"),presses:0});
 let trigger=find(&d.borrow(),r.root(),"id","probe-disclosure-trigger").unwrap();let hidden=find(&d.borrow(),r.root(),"id","hidden-action").unwrap();let panel=find(&d.borrow(),r.root(),"id","probe-disclosure-panel").unwrap();assert_eq!(attr(&d.borrow(),panel,"hidden"),Some("true"));
 r.set_focus(Some(trigger));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));assert_eq!(r.focus(),Some(hidden));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));assert_eq!(r.state().presses,1);assert!(!r.state().disclosure.expanded);
 println!("PASS collapsed disclosure hidden action remains tabbable and Space-activatable");
 r.update(|s|s.disclosure.expanded=true);assert_eq!(attr(&d.borrow(),panel,"hidden"),None);r.set_focus(Some(trigger));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));assert_eq!(r.focus(),Some(hidden));
 println!("PASS expanded disclosure control reaches its visible action");

}
