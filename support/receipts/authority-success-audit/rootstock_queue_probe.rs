use cambium::{AnyView,GenetCtx,GenetElement,el};
use cambium_rootstock::{Host,HostState,HostHooks,HostOptions,HostWake,HostPointer,WheelUnit,Modifiers,Runner,ScriptedDom};
use std::{cell::RefCell,rc::Rc,sync::Arc};
type V=Box<dyn AnyView<(),(),GenetCtx,GenetElement>>;
type Logic=fn(&())->V;
fn view(_: &())->V{Box::new(el::<_,(),()>("div",()))}
fn wheel(dy:f32)->HostPointer{HostPointer::Wheel{x:0.,y:0.,dx:0.,dy,unit:WheelUnit::Pixels,modifiers:Modifiers{ctrl:true,..Modifiers::NONE}}}
fn run(enqueue:bool)->Vec<f32>{
 let mut state:HostState<(),Logic,V>=HostState::new();state.runner=Some(Runner::new(Rc::new(RefCell::new(ScriptedDom::new())),view as Logic,()));
 let wake=HostWake::new(state.wake_pending.clone(),Arc::new(||{}));
 let samples=Rc::new(RefCell::new(Vec::new()));let out=samples.clone();let mut hooks=HostHooks::inert();
 hooks.after_dispatch=Box::new(move|ctx|{let mut values=out.borrow_mut();values.push(ctx.ui_zoom);let first=values.len()==1;drop(values);if enqueue && first{ctx.pointer.push(wheel(-1.));}});
 let mut host=Host::new(HostOptions::default(),None,hooks,state,wake);
 host.s.pending_pointer.extend([wheel(-1.),wheel(1.)]);host.drain_pointer();
 assert!(host.s.pending_pointer.is_empty());let result=samples.borrow().clone();result
}
fn main(){
 assert_eq!(run(false),vec![1.1,1.0]);println!("PASS initial queue control preserves zoom-in then zoom-out ordering");
 assert_eq!(run(true),vec![1.1,1.25,1.1]);println!("PASS callback-queued event recursively overtakes the existing second queued event");
 println!("PASS single outer drain consumes the callback event instead of leaving it for a later drain");
}
