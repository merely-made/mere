// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Output-only proposal. Never treats retained root pixels as acquired pixels.
use super::*;
use cambium_rootstock::BrowserAcquiredCaptureFrame;
use genet_render_host::{SurfaceCaptureBudget, SurfaceCaptureReservation};
#[wasm_bindgen(inline_js="export function surface_fixture_copy_rgba(view) { return new Uint8Array(view); }")]
extern "C" { #[wasm_bindgen(catch)] fn surface_fixture_copy_rgba(view:&js_sys::Uint8Array)->Result<js_sys::Uint8Array,JsValue>; }

pub(super) enum Artifact {
    Acquired(BrowserAcquiredCaptureFrame),
    Reference { frame:Frame, seal:Value, _credit:SurfaceCaptureReservation },
}
impl Artifact {
    fn rgba(&self)->&[u8] { match self { Self::Acquired(f)=>f.frame().rgba(),Self::Reference{frame,..}=>&frame.rgba } }
    fn extent(&self)->(u32,u32) { match self {
        Self::Acquired(f)=>{let m=f.frame().metadata();(m.observed_width,m.observed_height)},
        Self::Reference{frame,..}=>(frame.width,frame.height),
    } }
    fn role(&self)->&'static str { match self {Self::Acquired(_)=>"actual-acquired-final-surface",Self::Reference{..}=>"retained-root-or-registered-producer-reference"} }
}
/// Credits for fixture-created reference targets survive every failure path.
/// Weak Core records never retain a device across RAF. Clearing raw artifacts
/// cannot refund a target whose owning Core still exists. Retirement mutates
/// no renderer here; only the explicit source-reference path owns that action.
struct DiagnosticTarget {
    core:std::sync::Weak<RenderCore>, generation:u64, private_tile_key:Option<u64>,
    bytes:u64, map_completed:bool, private_tile_retired:bool, operation_finished:Rc<Cell<bool>>,
    credit:Option<SurfaceCaptureReservation>,
}
thread_local! { static TARGETS:RefCell<Vec<DiagnosticTarget>>=const {RefCell::new(Vec::new())}; }
pub(super) struct TargetOperation { finished:Rc<Cell<bool>> }
impl Drop for TargetOperation { fn drop(&mut self){self.finished.set(true);} }
pub(super) fn hold_target(budget:&Arc<SurfaceCaptureBudget>,core:std::sync::Weak<RenderCore>,generation:u64,private_tile_key:Option<u64>,bytes:u64)->Result<(usize,TargetOperation),String>{
    let credit=budget.reserve(bytes).map_err(|e|format!("reference GPU target credit refused: {e:?}"))?;
    let finished=Rc::new(Cell::new(false));
    TARGETS.with(|t|{let mut t=t.borrow_mut();let id=t.len();t.push(DiagnosticTarget{core,generation,private_tile_key,bytes,map_completed:false,private_tile_retired:false,operation_finished:finished.clone(),credit:Some(credit)});Ok((id,TargetOperation{finished}))})
}
pub(super) fn target_mapped(id:usize,private_tile_retired:bool)->Result<(),String>{
    TARGETS.with(|t|{let mut t=t.borrow_mut();let target=t.get_mut(id).ok_or("diagnostic target identity absent")?;
        target.map_completed=true;target.private_tile_retired=private_tile_retired;Ok(())})
}
/// Only actual Core closure releases conservative target credits, including
/// failure paths whose map or private tile retirement was never proved.
/// Drop credit owners after leaving the ledger borrow.
pub(super) fn target_ledger()->Value {
    let (rows,retired)=TARGETS.with(|t|{let mut t=t.borrow_mut();let mut retired=Vec::new();
        for target in t.iter_mut(){if target.core.upgrade().is_none() && target.operation_finished.get(){if let Some(c)=target.credit.take(){retired.push(c)}}}
        let rows=t.iter().map(|v|json!({"generation":v.generation,"private_tile_key":v.private_tile_key,
            "target_bytes":v.bytes,"map_completed":v.map_completed,"private_tile_retired":v.private_tile_retired,
            "credit_held":v.credit.is_some(),"operation_finished":v.operation_finished.get(),"core_closed":v.core.upgrade().is_none()})).collect::<Vec<_>>();(rows,retired)});
    drop(retired);json!(rows)
}
struct Store {
    budget:Arc<SurfaceCaptureBudget>, entries:BTreeMap<String,Artifact>,
    work_credit:Option<SurfaceCaptureReservation>, active_export:Option<js_sys::Uint8Array>,
}
thread_local! { static STORE:RefCell<Option<Store>>=const {RefCell::new(None)}; }
pub(super) fn initialize(budget:Arc<SurfaceCaptureBudget>)->Result<(),String> {
    let _=target_ledger();
    if TARGETS.with(|t|t.borrow().iter().any(|v|v.credit.is_some())){return Err("prior diagnostic target credit is still live".into())}
    STORE.with(|s| {let mut s=s.borrow_mut();if s.is_some(){return Err("artifact session already owns credits".into())}
        *s=Some(Store{budget,entries:BTreeMap::new(),work_credit:None,active_export:None});Ok(())})
}
pub(super) fn retain(name:String,frame:Artifact)->Result<Value,String> {
    STORE.with(|s| {let mut s=s.borrow_mut();let s=s.as_mut().ok_or("artifact session absent")?;
        if s.entries.contains_key(&name){return Err("duplicate immutable artifact name".into())}
        let (w,h)=frame.extent();let bytes=frame_bytes(w,h)?;
        if bytes!=frame.rgba().len(){return Err("artifact extent mismatch".into())}
        let seal=match &frame {Artifact::Reference{seal,..}=>Some(seal.clone()),Artifact::Acquired(_)=>None};
        let item=json!({"name":name,"role":frame.role(),"width":w,"height":h,"bytes":bytes,"sha256":digest_bytes(frame.rgba()),"reference_seal":seal});
        s.entries.insert(name,frame);Ok(item)})
}
pub(super) fn reserved()->Result<u64,String>{let _=target_ledger();STORE.with(|s|s.borrow().as_ref().map(|s|s.budget.reserved_bytes()).ok_or("artifact session absent".into()))}
/// One exported JS byte allocation and at most three Node raw/PNG scratch
/// allocations are admitted together, plus scanline tags and a 1MiB chunk.
/// The fixed driver must delete its export and release this owner in finally.
pub(super) fn export(name:&str)->Result<js_sys::Uint8Array,String>{
    STORE.with(|s| {let mut s=s.borrow_mut();let s=s.as_mut().ok_or("artifact session absent")?;
        if s.work_credit.is_some(){return Err("one serial export/comparison only".into())}
        let frame=s.entries.get(name).ok_or("unknown immutable artifact")?;
        let (_,height)=frame.extent();let bytes=u64::try_from(frame.rgba().len()).map_err(|_|"export extent overflow")?;
        let filtered=bytes.checked_add(u64::from(height)).ok_or("PNG scanline extent overflow")?;
        let zlib_extra=filtered.checked_add(16382).map(|n|n/16383).and_then(|n|n.checked_mul(5)).and_then(|n|n.checked_add(64)).ok_or("PNG zlib bound overflow")?;
        let peak=bytes.checked_mul(4).and_then(|n|n.checked_add(u64::from(height)*2)).and_then(|n|n.checked_add(zlib_extra)).and_then(|n|n.checked_add(1024*1024)).ok_or("export peak overflow")?;
        let credit=s.budget.reserve(peak).map_err(|e|format!("export transient credit refused: {e:?}"))?;
        // No clone of Rust raw Vec. The JS copy is admitted before allocation.
        // The borrowed view never escapes; the catch import copies synchronously
        // and turns a JS allocation refusal into a typed fixture failure.
        let view=unsafe { js_sys::Uint8Array::view(frame.rgba()) };
        let array=surface_fixture_copy_rgba(&view).map_err(|_|"JS export allocation refused")?;
        s.work_credit=Some(credit);s.active_export=Some(array.clone());Ok(array)})
}
pub(super) fn compare_admission(first:&str,second:&str)->Result<(),String>{
    STORE.with(|s| {let mut s=s.borrow_mut();let s=s.as_mut().ok_or("artifact session absent")?;
        if s.work_credit.is_some(){return Err("one serial export/comparison only".into())}
        let a=s.entries.get(first).ok_or("first comparison artifact absent")?;
        let b=s.entries.get(second).ok_or("second comparison artifact absent")?;
        if a.extent()!=b.extent(){return Err("comparison extent mismatch".into())}
        let bytes=u64::try_from(a.rgba().len()).map_err(|_|"comparison extent overflow")?.checked_mul(2).and_then(|n|n.checked_add(1024*1024)).ok_or("comparison peak overflow")?;
        s.work_credit=Some(s.budget.reserve(bytes).map_err(|e|format!("comparison transient credit refused: {e:?}"))?);Ok(())})
}
pub(super) fn release_work(){ STORE.with(|s|{if let Some(s)=s.borrow_mut().as_mut(){drop(s.active_export.take());drop(s.work_credit.take());}}); }
pub(super) fn clear()->u64 {
    // Take before dropping: no storage RefMut spans frame/credit destruction.
    let _=target_ledger();
    let retired=STORE.with(|s|s.borrow_mut().take());
    let budget=retired.as_ref().map(|s|s.budget.clone());drop(retired);
    budget.map(|b|b.reserved_bytes()).unwrap_or_else(||TARGETS.with(|t|t.borrow().iter().filter(|v|v.credit.is_some()).map(|v|v.bytes).sum()))
}
