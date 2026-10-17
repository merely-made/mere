use std::{any::Any,collections::HashMap,sync::{Arc,Mutex}};
use inker::*;
use pelt_core::*;

#[derive(Default)]
struct Probe { inputs:Vec<SessionInput> }
struct Engine {id:&'static str,probe:Arc<Mutex<Probe>>}
struct Page {id:&'static str,address:String,probe:Arc<Mutex<Probe>>}
struct Clock;
impl PeltClock for Clock {fn now_ms(&self)->f64{0.0}}
impl SessionEngine<String> for Engine {
    fn engine_id(&self)->&str{self.id}
    fn a11y_capability(&self)->A11yCapability{A11yCapability::Partial}
    fn spawn(&self,r:&SessionSpawnRequest)->Result<Box<dyn DocumentSession<String>>,SessionError>{
        Ok(Box::new(Page{id:self.id,address:r.address.clone(),probe:self.probe.clone()}))
    }
}
impl DocumentSession<String> for Page {
    fn frame(&mut self,_:u32,_:u32)->String{format!("{}:{}",self.id,self.address)}
    fn scroll_by(&mut self,_:f32,_:f32)->bool{false}
    fn scroll_for_key(&mut self,_:SessionScrollKey)->bool{false}
    fn click_at(&mut self,_:f32,_:f32)->SessionClick{SessionClick::Handled}
    fn links(&self)->Vec<SessionLink>{vec![]}
    fn input(&mut self,input:SessionInput)->SessionInputResult{
        let capture=match input{SessionInput::PointerButton{state:SessionButtonState::Pressed,..}=>Some(true),SessionInput::Cancel=>Some(false),_=>None};
        self.probe.lock().unwrap().inputs.push(input);
        SessionInputResult{effect:SessionEffect::Handled,cursor:None,editable:false,capture}
    }
    fn as_any_ref(&self)->&dyn Any{self}
    fn as_any(&mut self)->&mut dyn Any{self}
}
fn sessions(probe:&Arc<Mutex<Probe>>)->SessionRegistry<String>{
    let mut sessions=SessionRegistry::new();
    for id in ["first","second"]{sessions.register(Box::new(Engine{id,probe:probe.clone()}));}
    sessions
}
fn registries(probe:&Arc<Mutex<Probe>>)->PeltRegistries<String>{
    use routing::*;
    PeltRegistries::new(sessions(probe),SurfaceEngineRegistry::new(),EngineRoutePolicy{
      rules:vec![EngineRouteRule::new(["first"],"first",SurfaceContractMode::CompositedTexture)],
      fallback:EngineRouteRule::new(std::iter::empty::<&str>(),"second",SurfaceContractMode::CompositedTexture),per_host_overrides:HashMap::new()},
      "audit-pelt-core","second",EngineProfileBinding{user_data_dir:"AUDIT inert profile; no store".into()})
}
fn content(probe:&Arc<Mutex<Probe>>)->PeltContent<String>{
    PeltContent::routed(registries(probe),PeltTileRequest::new("first://fixture/start",(100,100)),Arc::new(||Box::new(Clock))).unwrap()
}

#[test]
fn adopted_session_frames_but_capability_inspection_panics_without_factory(){
    let probe=Arc::new(Mutex::new(Probe::default()));
    for registered in [false,true]{
        let registry=if registered{sessions(&probe)}else{SessionRegistry::new()};
        let mut controller=PeltController::from_session_shared(Arc::new(registry),Arc::new(SurfaceEngineRegistry::new()),
          PeltControllerConfig::new("first","first://fixture/adopted",(100,100)),Clock,
          Box::new(Page{id:"first",address:"first://fixture/adopted".into(),probe:probe.clone()}));
        assert_eq!(controller.frame(100,100),"first:first://fixture/adopted");
        let observed=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||controller.a11y_capability()));
        if registered{assert_eq!(observed.unwrap(),A11yCapability::Partial)}else{assert!(observed.is_err())}
    }
    println!("AUDIT adopted no-factory session frames normally, then capability inspection panics; registered control returns Partial");
}

#[test]
fn back_restores_engine_but_retains_later_route_metadata(){
    let probe=Arc::new(Mutex::new(Probe::default()));let mut content=content(&probe);
    assert_eq!(content.document().unwrap().engine_id(),"first");assert_eq!(content.route().active_engine(),"first");
    assert!(content.command(SessionNavigationCommand::Address("second://fixture/later".into())).navigated);
    assert_eq!(content.document().unwrap().engine_id(),"second");assert_eq!(content.route().active_engine(),"second");
    assert!(content.command(SessionNavigationCommand::Back).navigated);
    assert_eq!(content.document().unwrap().engine_id(),"first");assert_eq!(content.address(),Some("first://fixture/start"));
    assert_eq!(content.route().active_engine(),"second");assert_eq!(content.route().selected_engine(),"second");
    assert!(content.command(SessionNavigationCommand::Reload).navigated);
    assert_eq!(content.document().unwrap().engine_id(),"first");assert_eq!(content.route().active_engine(),"second");
    assert!(content.command(SessionNavigationCommand::Forward).navigated);
    assert_eq!(content.document().unwrap().engine_id(),content.route().active_engine());
    println!("AUDIT Back and Reload present first engine while selected/active route reports second; Forward agrees again");
}

#[test]
fn pool_drops_session_cancel_for_nonzero_pointer_captures(){
    let probe=Arc::new(Mutex::new(Probe::default()));let mut pool=PeltControllerPool::<u64,u64,String>::new();pool.insert(1,content(&probe));
    pool.set_placements([(10,PeltPlacement{content:1,window:7,rect:WorkspaceRect::new(0.,0.,100.,100.),visible:true,focused:true,scale_factor:1.})]);
    for id in [37,49]{assert!(pool.input(&10,PeltInput::Pointer(mouse_pointer(id,PointerPhase::Down,PhysicalPosition{x:20.,y:20.},Some(MouseButton::Left),PointerButtons::PRIMARY,Default::default()))).handled)}
    probe.lock().unwrap().inputs.clear();
    let ignored=pool.input(&10,SessionInput::Cancel.into());
    assert!(!ignored.handled);assert!(probe.lock().unwrap().inputs.is_empty());
    assert_eq!(pool.captured_placement(37),Some(&10));assert_eq!(pool.captured_placement(49),Some(&10));
    pool.input_at(7,SessionInput::Focus(false).into());assert!(!pool.has_pointer_capture());
    assert!(probe.lock().unwrap().inputs.iter().any(|i|*i==SessionInput::Cancel));
    let mut direct=PeltController::new(sessions(&probe),SurfaceEngineRegistry::new(),PeltControllerConfig::new("first","first://fixture/direct",(100,100)),Clock).unwrap();
    assert!(direct.input(SessionInput::Cancel).handled);
    println!("AUDIT placement Session Cancel discarded without pointer0 admission; nonzero captures survive. Window focus-loss helper and direct controller Cancel deliver");
}
