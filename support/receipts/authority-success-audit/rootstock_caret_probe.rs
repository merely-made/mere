use cambium::{AnyView,GenetCtx,GenetElement,TextInput,textarea};
use cambium_rootstock::{Host,HostState,HostHooks,HostOptions,HostWake,Runner,ScriptedDom,FocusedTextSlot,KeyPress,NamedKey};
use std::{cell::RefCell,rc::Rc,sync::Arc};
type V=Box<dyn AnyView<TextInput,(),GenetCtx,GenetElement>>;
type Logic=fn(&TextInput)->V;
fn view(s:&TextInput)->V{Box::new(textarea(s))}
fn focused(r:&Runner<TextInput,Logic,V>)->Option<FocusedTextSlot<TextInput>>{
 let node=r.focus()?;Some(FocusedTextSlot{node,get:Box::new(|s|s),get_mut:Box::new(|s|s)})
}
fn run(input:TextInput,key:NamedKey,layout:bool)->usize{
 let mut state:HostState<TextInput,Logic,V>=HostState::new();
 state.shared.sheet="[role=textbox]{display:block;white-space:pre-wrap;width:280px;height:100px}".into();
 let mut r=Runner::new(Rc::new(RefCell::new(ScriptedDom::new())),view as Logic,input);r.set_focus(Some(r.root()));state.runner=Some(r);
 let wake=HostWake::new(state.wake_pending.clone(),Arc::new(||{}));let mut hooks=HostHooks::inert();hooks.focused_text=Box::new(focused);
 let mut host=Host::new(HostOptions::default(),None,hooks,state,wake);
 if layout{host.relayout(300.,200.);assert!(host.s.layout.is_some());}
 host.key(&KeyPress::named(key));
 host.s.runner.as_ref().unwrap().state().caret_position().byte
}
fn input(text:&str,caret:usize,anchor:Option<usize>)->TextInput{let mut t=TextInput::new(text);if let Some(anchor)=anchor{t.set_caret_byte(anchor,false);t.set_caret_byte(caret,true);}else{t.set_caret_byte(caret,false);}t}
fn main(){
 let cases=[("multiline Home",input("ab\ncd",4,None),NamedKey::Home,3usize,0usize),
 ("multiline End",input("ab\ncd",1,None),NamedKey::End,2,5),
 ("selection Left",input("abcd",3,Some(1)),NamedKey::ArrowLeft,1,2),
 ("grapheme Right",input("a\u{301}b",0,None),NamedKey::ArrowRight,3,0)];
 for (label,t,key,correct,actual) in cases{
  assert_eq!(run(t.clone(),key,false),correct);println!("PASS {label} no-layout control keeps toolkit caret byte {correct}");
  let observed=run(t,key,true);assert_eq!(observed,actual);println!("PASS {label} current retained-layout correction overwrites caret byte to {observed}");
 }
}
