use cambium::*;
use genet_scripted_dom::{NodeId,ScriptedDom};
use layout_dom_api::{LayoutDom,Namespace,LocalName};
use workbench::{Workspace,Tile,TileId,TileTree,TabStack,ContentSource,DocumentRef,FloatEvent,RelativeRect,WorkspaceEvent,TileEvent};
use std::{cell::RefCell,rc::Rc};
struct State{workspace:Workspace}
type V=Box<dyn AnyView<State,(),GenetCtx,GenetElement>>;
fn attr<'a>(d:&'a ScriptedDom,n:NodeId,k:&str)->Option<&'a str>{d.attribute(n,&Namespace::from(""),&LocalName::from(k))}
fn find(d:&ScriptedDom,n:NodeId,k:&str,v:&str)->Option<NodeId>{if attr(d,n,k)==Some(v){return Some(n)}d.dom_children(n).find_map(|c|find(d,c,k,v))}
fn fill(t:&Tile)->Slot<State,()>{Slot::View(Box::new(component(t.id.0,|_:&u64|0u32,|_:&u64,_:&u64,_:&mut u32|{},|id:&u64,count:&u32|->ComponentView<u32,()>{Box::new(button(count.to_string(),|count:&mut u32,_|*count+=1).attr("data-owner",id.to_string()).attr("data-count",count.to_string()))},|_:&mut State,_:()|{})))}
fn view(s:&State)->V{Box::new(workspace_view(&WorkspaceModel{workspace:&s.workspace,current:None,float_layer_visible:true},|_:&mut State,_|{},fill))}
fn tile(id:u64)->Tile{Tile{id:TileId(id),title:format!("tile {id}"),content:ContentSource::Document(DocumentRef(format!("doc:{id}"))),accent:None}}
fn main(){
 let mut workspace=Workspace::new(TileTree::Stack(TabStack{tabs:vec![tile(1),tile(2),tile(3)],active:0}));
 for id in [1,2]{workspace.apply(&WorkspaceEvent::from(FloatEvent::Float{tile:TileId(id),rect:RelativeRect{x:0.1,y:0.1,width:0.4,height:0.4}}));}
 let d:DomHandle=Rc::new(RefCell::new(ScriptedDom::new()));let mut r=GenetAppRunner::new(d.clone(),view,State{workspace});
 let one=find(&d.borrow(),r.root(),"data-owner","1").unwrap();let two=find(&d.borrow(),r.root(),"data-owner","2").unwrap();
 r.dispatch_click(one,PointerClick::at((1.,1.)));assert_eq!(attr(&d.borrow(),one,"data-count"),Some("1"));assert_eq!(attr(&d.borrow(),two,"data-count"),Some("0"));
 r.update(|_|{});assert_eq!(find(&d.borrow(),r.root(),"data-owner","1"),Some(one));assert_eq!(attr(&d.borrow(),one,"data-count"),Some("1"));println!("PASS unchanged float order preserves owner-one node and local count");
 r.update(|s|{s.workspace.apply(&WorkspaceEvent::Tile(TileEvent::Activated(TileId(1))));});
 let new_one=find(&d.borrow(),r.root(),"data-owner","1").unwrap();let new_two=find(&d.borrow(),r.root(),"data-owner","2").unwrap();
 assert_eq!(new_two,one);assert_eq!(new_one,two);assert_eq!(attr(&d.borrow(),new_two,"data-count"),Some("1"));assert_eq!(attr(&d.borrow(),new_one,"data-count"),Some("0"));
 println!("PASS raising float one reassigns its local count/node to owner two and resets owner one");
 r.dispatch_click(new_two,PointerClick::at((1.,1.)));assert_eq!(attr(&d.borrow(),new_two,"data-count"),Some("2"));println!("PASS wrong-owner count remains live and increments through actual current callback");
}
