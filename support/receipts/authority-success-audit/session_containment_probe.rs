use cambium::*;
use genet_scripted_dom::{NodeId,ScriptedDom};
use layout_dom_api::LayoutDom;
use mere_surface_api::{SurfaceAvailability,SurfaceDescriptor,SurfaceUnavailableReason,ProviderId,SurfaceId,SurfaceSourceShape,SourceKindId};
use sprigging::LeafRegistry;
use std::{cell::RefCell,rc::Rc,panic::{catch_unwind,AssertUnwindSafe}};
struct Guest{descriptor:SurfaceDescriptor,dom:DomHandle,leaves:LeafRegistry<u64>,calls:u32,fail_at:u32}
impl RetainedSurfaceSession for Guest{
 fn descriptor(&self)->&SurfaceDescriptor{&self.descriptor}
 fn availability(&self)->SurfaceAvailability{SurfaceAvailability::Available}
 fn dom(&self)->DomHandle{self.dom.clone()}
 fn root(&self)->NodeId{self.dom.borrow().document()}
 fn focus(&self)->Option<NodeId>{None}
 fn set_focus(&mut self,_:Option<NodeId>)->Vec<SurfaceEffect>{vec![]}
 fn focus_traverse(&mut self,_:bool)->Vec<SurfaceEffect>{vec![]}
 fn focusables(&self)->Vec<NodeId>{vec![]}
 fn pointer_capture(&self)->Option<NodeId>{None}
 fn pointer_target(&self,_:NodeId)->Option<NodeId>{None}
 fn hover_target(&self,_:NodeId)->Option<NodeId>{None}
 fn wheel_target(&self,_:NodeId)->Option<NodeId>{None}
 fn sync_viewport(&mut self,_:SurfaceViewport)->Vec<SurfaceEffect>{vec![]}
 fn dispatch(&mut self,_:ResolvedSurfaceEvent)->Vec<SurfaceEffect>{vec![]}
 fn leaves(&mut self)->Option<&mut LeafRegistry<u64>>{self.calls+=1;if self.calls==self.fail_at{panic!("guest leaves failure")};Some(&mut self.leaves)}
}
fn guest(fail_at:u32)->Guest{Guest{descriptor:SurfaceDescriptor{provider_id:ProviderId::from("audit"),surface_id:SurfaceId::from("audit.leaves"),label:"audit".into(),accepted_source:SurfaceSourceShape::One(SourceKindId::from("audit"))},dom:Rc::new(RefCell::new(ScriptedDom::new())),leaves:LeafRegistry::new(),calls:0,fail_at}}
fn main(){
 let mut c=ContainedSession::new(guest(1));
 assert!(catch_unwind(AssertUnwindSafe(||assert!(c.leaves().is_none()))).is_ok());
 assert_eq!(c.failure().unwrap().call,"leaves");assert_eq!(c.availability(),SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Unhealthy));
 println!("PASS first leaf call panic is caught and retires guest");
 let mut c=ContainedSession::new(guest(2));
 assert!(catch_unwind(AssertUnwindSafe(||{let _=c.leaves();})).is_err());
 assert!(c.failure().is_none());assert_eq!(c.availability(),SurfaceAvailability::Available);
 println!("PASS second leaf call panic escapes containment and guest still reports available");
 let mut c=ContainedSession::new(guest(99));assert!(c.leaves().is_some());assert!(c.failure().is_none());
 println!("PASS stable registry control returns leaves without retiring guest");
}
