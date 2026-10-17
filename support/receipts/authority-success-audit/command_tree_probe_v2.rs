pub use cambium::{Action, ElementView, GenetCtx, GenetElement, View, Key, KeyEvent, NamedKey,
    PointerClick, el, on_click, on_key, FocusPhase, focusable_if, on_focus, request_focus};
#[path = "../../../crates/cambium/cambium/src/command_surface.rs"]
mod reviewed_surface;
pub use reviewed_surface::*;
#[path = "../../../crates/cambium/cambium/src/command_menu_bar.rs"]
mod reviewed_bar;
pub use reviewed_bar::*;
use std::{cell::RefCell, rc::Rc};
use cambium::{DomHandle, GenetAppRunner};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, Namespace, LocalName};
fn tree() -> Vec<CommandItem> { vec![CommandItem::new("File").with_children([
    CommandItem::new("Export").with_children([CommandItem::new("HTML")])])] }
fn dom() -> DomHandle { Rc::new(RefCell::new(ScriptedDom::new())) }
fn find(d: &ScriptedDom, n: NodeId, value: &str) -> Option<NodeId> {
    if d.attribute(n, &Namespace::from(""), &LocalName::from("data-key")) == Some(value) {return Some(n)}
    d.dom_children(n).find_map(|c|find(d,c,value))
}
fn key(k: NamedKey) -> KeyEvent { KeyEvent::new(Key::Named(k)) }
fn main() {
    let d=dom(); let mut menu=GenetAppRunner::new(d.clone(), |s:&CommandState|command_menu(s,&tree(),0.,0.),CommandState::default());
    menu.set_focus(Some(menu.root())); menu.dispatch_key(key(NamedKey::Enter));
    assert_eq!(menu.state().submenu,Some(0));
    assert!(find(&d.borrow(),menu.root(),"Export").is_some());
    assert!(find(&d.borrow(),menu.root(),"HTML").is_none());
    assert_eq!(menu.dispatch_key(key(NamedKey::Enter)),[CommandEvent::Activate(vec![0,0])]);
    println!("PASS context keyboard activates non-leaf Export while HTML remains absent");
    let export=find(&d.borrow(),menu.root(),"Export").unwrap();
    assert!(menu.dispatch_click(export,PointerClick::at((1.,1.))).is_empty());
    assert_eq!(menu.state().submenu,Some(0));
    assert!(find(&d.borrow(),menu.root(),"HTML").is_none());
    println!("PASS context pointer disclosure reopens root submenu without exposing HTML");
    let d=dom();let mut bar=GenetAppRunner::new(d.clone(),|s:&CommandMenuBarState|command_menu_bar(s,&tree(),false),CommandMenuBarState::default());
    bar.set_focus(Some(bar.root()));bar.dispatch_key(key(NamedKey::Enter));bar.dispatch_key(key(NamedKey::Enter));
    assert!(find(&d.borrow(),bar.root(),"HTML").is_some());
    assert_eq!(bar.dispatch_key(key(NamedKey::Enter)),[CommandEvent::Activate(vec![0,0,0])]);
    println!("PASS menubar control reaches same tree HTML leaf with full original path");
    let shared=Rc::new(RefCell::new(vec![CommandItem::new("File").with_children([CommandItem::new("Open")])]));
    let input=shared.clone();let d=dom();let mut menu=GenetAppRunner::new(d.clone(),move|s:&CommandState|command_menu(s,&input.borrow(),0.,0.),CommandState::default());
    menu.set_focus(Some(menu.root()));menu.dispatch_key(key(NamedKey::Enter));
    let disabled=shared.borrow()[0].clone().disabled_because("Disconnected");
    shared.borrow_mut()[0]=disabled;
    menu.update(|_|{});
    assert_eq!(menu.dispatch_key(key(NamedKey::Enter)),[CommandEvent::Activate(vec![0,0])]);
    println!("PASS context child still activates after its open parent becomes disabled");
    let shared=Rc::new(RefCell::new(vec![CommandItem::new("File").with_children([CommandItem::new("Open")])]));
    let input=shared.clone();let d=dom();let mut bar=GenetAppRunner::new(d.clone(),move|s:&CommandMenuBarState|command_menu_bar(s,&input.borrow(),false),CommandMenuBarState::default());
    bar.set_focus(Some(bar.root()));bar.dispatch_key(key(NamedKey::Enter));
    let disabled=shared.borrow()[0].clone().disabled_because("Disconnected");
    shared.borrow_mut()[0]=disabled;
    bar.update(|_|{});
    assert_eq!(bar.dispatch_key(key(NamedKey::Enter)),[CommandEvent::Activate(vec![0,0])]);
    println!("PASS menubar child still activates after its open parent becomes disabled");
    let items=vec![CommandItem::new("File").with_children([CommandItem::new("Open").disabled(true)])];
    let mut menu=GenetAppRunner::new(dom(),move|s:&CommandState|command_menu(s,&items,0.,0.),CommandState::default());
    menu.set_focus(Some(menu.root()));menu.dispatch_key(key(NamedKey::Enter));
    assert!(menu.dispatch_key(key(NamedKey::Enter)).is_empty());
    println!("PASS individually disabled child remains inert");
}
