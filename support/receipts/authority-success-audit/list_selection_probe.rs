use cambium::*;
use genet_scripted_dom::{NodeId,ScriptedDom};
use layout_dom_api::LayoutDom;
use std::{cell::RefCell,rc::Rc};
#[derive(Debug,PartialEq)] enum Out { Navigate }
impl Action for Out {}
type V=Box<dyn AnyView<u32,Out,GenetCtx,GenetElement>>;
fn view(_: &u32)->V { Box::new(sectioned_list(&[ListSection::new("Recent",vec![ListRow::plain("alpha")])],|s:&mut u32,_,_|{*s+=1;Out::Navigate})) }
fn main(){
 let d:DomHandle=Rc::new(RefCell::new(ScriptedDom::new()));
 let mut r=GenetAppRunner::new(d.clone(),view,0u32);
 let n:NodeId=d.borrow().all_with_class(r.root(),"list-row")[0];
 assert_eq!(r.dispatch_click(n,PointerClick::at((1.,1.))),vec![Out::Navigate]);
 assert_eq!(*r.state(),1);println!("PASS list pointer control updates state and returns navigation action");
 r.set_focus(Some(n));
 for key in [NamedKey::Enter,NamedKey::Space]{
  let before=*r.state();let actions=r.dispatch_key(KeyEvent::new(Key::Named(key)));
  assert_eq!(*r.state(),before+1);assert!(actions.is_empty());
  println!("PASS list {key:?} invokes handler but drops navigation action");
 }
 let d:DomHandle=Rc::new(RefCell::new(ScriptedDom::new()));
 let mut r=GenetAppRunner::new(d,|s:&SelectionState|segmented_control(s,&[SelectionItem::new("a"),SelectionItem::new("b")]),SelectionState::single(1));
 let n=r.focusables()[0];r.set_focus(Some(n));r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
 assert_eq!(r.state().selected,[0]);println!("PASS enabled segmented control selects prior enabled item");
 let d:DomHandle=Rc::new(RefCell::new(ScriptedDom::new()));
 let mut r=GenetAppRunner::new(d,|disabled:&SelectionState|segmented_control(disabled,&[SelectionItem::new("a").disabled(true),SelectionItem::new("b").disabled(true)]),SelectionState::single(1));
 assert!(r.focusables().is_empty());r.set_focus(Some(r.root()));
 // Explicit root routing is supported but is not normal Tab traversal.
 r.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
 assert_eq!(r.state().selected,[0]);println!("PASS directly routed arrow selects disabled item zero when no enabled positions exist");
}
