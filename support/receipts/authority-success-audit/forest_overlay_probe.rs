use cambium::*;
use genet_scripted_dom::{NodeId,ScriptedDom};
use layout_dom_api::{LayoutDom,LocalName,Namespace};
use std::{cell::RefCell,rc::Rc};
use workbench::{Tile,TileId,TileTree,TabStack,ContentSource,DocumentRef};
fn dom()->DomHandle{Rc::new(RefCell::new(ScriptedDom::new()))}
fn attr<'a>(d:&'a ScriptedDom,n:NodeId,k:&str)->Option<&'a str>{d.attribute(n,&Namespace::from(""),&LocalName::from(k))}
fn find(d:&ScriptedDom,n:NodeId,k:&str,v:&str)->Option<NodeId>{if attr(d,n,k)==Some(v){return Some(n)}d.dom_children(n).find_map(|c|find(d,c,k,v))}
struct Shared{owner:usize}
type WindowView=Box<dyn AnyView<Shared,(),GenetCtx,GenetElement>>;
fn tile(id:u64)->Tile{Tile{id:TileId(id),title:format!("tile {id}"),content:ContentSource::Document(DocumentRef(format!("doc:{id}"))),accent:None}}
fn fill(t:&Tile)->Slot<Shared,()>{Slot::View(Box::new(component(t.id.0,|_:&u64|0u32,|_:&u64,_:&u64,_:&mut u32|{},|id:&u64,count:&u32|->ComponentView<u32,()>{Box::new(button(count.to_string(),|count:&mut u32,_|*count+=1).attr("id",format!("counter-{id}")).attr("data-count",count.to_string()))},|_:&mut Shared,_:()|{})))}
fn projection(window:usize)->impl FnMut(&Shared)->WindowView{move|s:&Shared|{let id=if s.owner==window{7}else{10+window as u64};let tree=TileTree::Stack(TabStack{tabs:vec![tile(id)],active:0});Box::new(frisket_with(&tree,|_:&mut Shared,_|{},fill))}}
struct Modal{modal:bool,background:u32}
fn modal_view(s:&Modal)->Box<dyn AnyView<Modal,(),GenetCtx,GenetElement>>{Box::new(el("div",(button("Background",|s:&mut Modal,_|s.background+=1).attr("id","background"),overlay_surface(&OverlaySurface::new((0.,0.,1.,1.),(100.,100.),(0.,0.,300.,300.)).with_role(OverlayRole::Dialog).modal(s.modal).dismiss_on_outside_click(false),button("Inside",|_:&mut Modal,_|{}).attr("id","inside"),|_:&mut Modal,_|{}))))}
fn main(){
 for (source,dest) in [(0,1),(1,0)]{
  let d=dom();let mut r=GenetMultiRunner::new(Shared{owner:source});let a=r.push_forest_projection(d.clone(),projection(0));let b=r.push_forest_projection(d.clone(),projection(1));let ids=[a,b];
  assert_eq!(d.borrow().parent(r.window_root(a).unwrap()),Some(d.borrow().document()));assert_eq!(d.borrow().parent(r.window_root(b).unwrap()),Some(d.borrow().document()));
  let old=find(&d.borrow(),r.root(ids[source]).unwrap(),"id","counter-7").unwrap();r.dispatch_click(ids[source],old,PointerClick::at((1.,1.)));assert_eq!(attr(&d.borrow(),old,"data-count"),Some("1"));
  r.update(|_|{});assert_eq!(find(&d.borrow(),r.root(ids[source]).unwrap(),"id","counter-7"),Some(old));assert_eq!(attr(&d.borrow(),old,"data-count"),Some("1"));println!("PASS forest {source}->{dest} unchanged-owner control retains local count and node");
  r.update(|s|s.owner=dest);let new=find(&d.borrow(),r.root(ids[dest]).unwrap(),"id","counter-7").unwrap();assert_ne!(old,new);assert!(!d.borrow().is_live(old));assert_eq!(attr(&d.borrow(),new,"data-count"),Some("0"));assert_eq!(r.state().owner,dest);println!("PASS forest {source}->{dest} move updates shared truth but replaces node and resets component count");
 }
 let d=dom();let mount=RetainedMount::new(d.clone());let mut owner=GenetAppRunner::new_at(d.clone(),mount.node(),|count:&u32|button(count.to_string(),|count:&mut u32,_|*count+=1),0u32);let original=owner.root();owner.dispatch_click(original,PointerClick::at((1.,1.)));
 let m=mount.clone();let mut source=GenetAppRunner::new(d.clone(),move|enabled:&bool|->Box<dyn AnyView<bool,(),GenetCtx,GenetElement>>{if *enabled{Box::new(m.view::<bool,()>())}else{Box::new(el::<_,bool,()>("div",()))}},true);
 let m=mount.clone();let mut destination=GenetAppRunner::new(d.clone(),move|_:&()|m.view::<(),()>(),());source.update(|s|*s=false);destination.update(|_|{});assert_eq!(owner.root(),original);assert!(d.borrow().is_live(original));assert_eq!(d.borrow().parent(mount.node()),Some(destination.root()));assert_eq!(*owner.state(),1);owner.dispatch_click(original,PointerClick::at((1.,1.)));assert_eq!(*owner.state(),2);println!("PASS separate RetainedMount control preserves actual runner node/state across destination-first mount");
 drop(destination);drop(source);drop(owner);drop(mount);
 let d=dom();let mut r=GenetAppRunner::new(d.clone(),modal_view,Modal{modal:true,background:0});let inside=find(&d.borrow(),r.root(),"id","inside").unwrap();let background=find(&d.borrow(),r.root(),"id","background").unwrap();let dialog=find(&d.borrow(),r.root(),"role","dialog").unwrap();assert_eq!(attr(&d.borrow(),dialog,"aria-modal"),Some("true"));r.set_focus(Some(inside));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));assert_eq!(r.focus(),Some(background));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));assert_eq!(r.state().background,1);println!("PASS aria-modal dialog leaves background in Tab traversal and Space invokes background callback");
 r.update(|s|s.modal=false);assert_eq!(attr(&d.borrow(),dialog,"aria-modal"),None);assert!(r.focusables().contains(&background));println!("PASS nonmodal control exposes background without modal ARIA claim");
}
