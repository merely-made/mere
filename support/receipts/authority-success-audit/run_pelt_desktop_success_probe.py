"""Execute exact viewer startup/success bodies with deterministic platform doubles.

This does not compile the desktop crate or exercise an actual event loop/GPU.
"""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source=ROOT/'ports/pelt/desktop/static_viewer.rs'
smoke=ROOT/'ports/pelt/desktop/smoke_netrender.rs'
text=source.read_text(encoding='utf-8-sig')
pieces={}
def between(name,start,end):
    a=text.index(start);b=text.index(end,a)
    pieces[name]=text[a:b]
    return pieces[name]

entry=between('run_headed_with','pub(crate) fn run_headed_with<','\n#[cfg(feature = "present")]\npub(crate) mod windowed')
resumed=between('resumed','        fn resumed(&mut self, event_loop: &ActiveEventLoop)', '\n        fn window_event(')
outcome=between('outcome','        pub(crate) fn outcome(&self)', '\n        fn window_title(')
net=smoke.read_text(encoding='utf-8-sig');net=net[net.index('pub fn run_netrender_smoke()'):]
pieces['run_netrender_smoke']=net

prefix=r'''
#![allow(dead_code,unused_imports)]
use std::sync::Arc;
use std::cell::Cell;
thread_local! { static MODE: Cell<u8> = const {Cell::new(0)}; }
fn mode()->u8 { MODE.with(Cell::get) }
#[derive(Clone,Debug,PartialEq)]
pub struct StaticViewerOutcome {url:String,created_window:bool,redraws:u32,size:(u32,u32),product_receipt:Option<String>}
pub struct StaticViewerConfig {url:String, product_receipt:Option<String>}
pub struct Window;
impl Window { fn inner_size(&self)->Size {Size {width:800,height:600}} fn scale_factor(&self)->f64 {1.0} fn set_title(&self,_:&str){} fn request_redraw(&self){} }
pub struct Size{width:u32,height:u32}
pub struct SurfaceHost;
impl SurfaceHost {fn boot(_:Arc<Window>,_:u32,_:u32,_:NetrenderOptions)->Result<Self,String>{if mode()==2 {Err("injected host boot failure".into())}else{Ok(Self)}}}
#[derive(Default)]
pub struct NetrenderOptions{tile_cache_size:Option<usize>,enable_vello:bool,unused:bool}
fn pelt_window_attributes(_:String,_:u32,_:u32){}
pub struct ActiveEventLoop;
impl ActiveEventLoop {fn exit(&self){} fn create_window(&self,_:())->Result<Window,String>{if mode()==1 {Err("injected window failure".into())}else{Ok(Window)}}}
mod winit {pub mod event_loop {
    pub struct EventLoop;
    impl EventLoop {pub fn new()->Result<Self,String>{Ok(Self)} pub fn run_app<C:crate::windowed::ViewerContent>(&self, app:&mut crate::windowed::ViewerApp<C>)->Result<(),String>{app.resumed(&crate::ActiveEventLoop);if crate::mode()==3 {app.fail();}Ok(())}}
}}
mod windowed {
use super::*;
pub trait ViewerContent {fn address(&self)->Option<&str>{None}}
pub struct ViewerApp<C:ViewerContent> {config:StaticViewerConfig,doc:C,window:Option<Arc<Window>>,host:Option<SurfaceHost>,width:u32,height:u32,scale_factor:f32,redraws:u32,receipt_outcome:Option<String>,receipt_failure:Option<String>}
impl<C:ViewerContent> ViewerApp<C> {
pub fn new(config:StaticViewerConfig,doc:C)->Self {Self{config,doc,window:None,host:None,width:800,height:600,scale_factor:1.0,redraws:0,receipt_outcome:None,receipt_failure:None}}
pub fn fail(&mut self){self.receipt_failure=Some("injected semantic receipt failure".into());}
fn window_title(&self)->String {"Pelt".into()}
'''
# Only visibility is widened for the extracted callback so the event-loop double
# can invoke it. Its executable body is unchanged.
resumed_public=resumed.replace('        fn resumed(', '        pub fn resumed(',1)
runtime=prefix+outcome+resumed_public+'\n}}\n'+entry
runtime+=r'''
#[derive(Clone,Copy,Debug)] pub struct E;
impl std::fmt::Display for E {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"stub")}}
mod wgpu {
#[derive(Default)] pub struct TextureDescriptor<'a>{pub label:Option<&'a str>,pub size:Extent3d,pub mip_level_count:u32,pub sample_count:u32,pub dimension:TextureDimension,pub format:TextureFormat,pub usage:TextureUsages,pub view_formats:&'a [TextureFormat]}
#[derive(Default)] pub struct Extent3d{pub width:u32,pub height:u32,pub depth_or_array_layers:u32}
#[derive(Default)] pub enum TextureDimension{#[default]D2}
#[derive(Default)] pub enum TextureFormat{#[default]Rgba8Unorm,Rgba8UnormSrgb}
#[derive(Default)] pub struct TextureViewDescriptor<'a>{pub label:Option<&'a str>,pub format:Option<TextureFormat>}
#[derive(Default)] pub struct TextureUsages;
impl TextureUsages {pub const STORAGE_BINDING:Self=Self;pub const TEXTURE_BINDING:Self=Self;pub const COPY_SRC:Self=Self;}
impl std::ops::BitOr for TextureUsages{type Output=Self;fn bitor(self,_:Self)->Self{Self}}
pub struct Texture;impl Texture{pub fn create_view(&self,_:&TextureViewDescriptor)->(){}}
#[derive(Clone)]pub struct Device;impl Device{pub fn create_texture(&self,_:&TextureDescriptor)->Texture{Texture}}
}
mod netrender {
pub use crate::NetrenderOptions;
pub struct Handles{pub device:crate::wgpu::Device}
pub fn boot()->Result<Handles,crate::E>{Ok(Handles{device:crate::wgpu::Device})}
pub struct Renderer{pub wgpu_device:Readback}
pub struct Readback;
impl Readback {pub fn read_rgba8_texture(&self,_:&crate::wgpu::Texture,w:u32,h:u32)->Vec<u8>{vec![if crate::mode()==4 {0}else{255};(w*h*4) as usize]}}
pub fn create_netrender_instance(_:Handles,_:NetrenderOptions)->Result<Renderer,crate::E>{Ok(Renderer{wgpu_device:Readback})}
pub struct Scene;impl Scene{pub fn new(_:u32,_:u32)->Self{Self}pub fn push_rect(&mut self,_:f32,_:f32,_:f32,_:f32,_:[f32;4]){}}
#[derive(Default)]pub struct ColorLoad;
impl Renderer {pub fn render_vello(&self,_:&Scene,_:&(),_:ColorLoad){}}
}
#[derive(Debug)]pub struct NetrenderSmokeOutcome{pub width:u32,pub height:u32,pub painted_pixels:usize}
'''+net+r'''
#[cfg(test)]mod audit_tests {
use super::*;
struct Content;impl windowed::ViewerContent for Content{}
fn startup(m:u8)->Result<StaticViewerOutcome,String>{MODE.with(|v|v.set(m));run_headed_with(StaticViewerConfig{url:"fixture".into(),product_receipt:Some("article".into())},Content)}
#[test]fn failed_window_is_ok_without_receipt(){let o=startup(1).unwrap();assert!(!o.created_window);assert_eq!(o.redraws,0);assert_eq!(o.product_receipt,None);}
#[test]fn failed_host_is_ok_without_receipt(){let o=startup(2).unwrap();assert!(!o.created_window);assert_eq!(o.redraws,0);assert_eq!(o.product_receipt,None);}
#[test]fn semantic_failure_positive_control_is_err(){assert!(startup(3).unwrap_err().contains("semantic receipt failure"));}
#[test]fn successful_startup_control_has_window(){assert!(startup(0).unwrap().created_window);}
#[test]fn blank_netrender_is_ok_zero_pixels(){MODE.with(|v|v.set(4));assert_eq!(run_netrender_smoke().unwrap().painted_pixels,0);}
#[test]fn wrong_white_netrender_is_ok_full_pixels(){MODE.with(|v|v.set(0));assert_eq!(run_netrender_smoke().unwrap().painted_pixels,64*64);}
}
'''
rs=HERE/'pelt_desktop_success_probe.rs'
rs.write_text(runtime,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-pelt-desktop-success-probe.exe')
assert exe.parent.is_dir() and not exe.exists()
pdb=exe.with_suffix('.pdb');assert not pdb.exists()
compile_cmd=['rustc','--edition','2024','--test','-C','debuginfo=0',str(rs),'-o',str(exe)]
build=subprocess.run(compile_cmd,cwd=ROOT,text=True,capture_output=True)
(HERE/'pelt_desktop_success_compile.txt').write_text(build.stdout+build.stderr,encoding='utf-8')
assert build.returncode==0,build.stderr
run=subprocess.run([str(exe),'--test-threads=1','--nocapture'],cwd=ROOT,text=True,capture_output=True)
(HERE/'pelt_desktop_success_output.txt').write_text(run.stdout+run.stderr,encoding='utf-8')
assert run.returncode==0 and '6 passed' in run.stdout,run.stdout+run.stderr
removed=[]
for p in [exe,pdb]:
    if p.exists():
        assert p.resolve().parent==Path('C:/t/cargo-targets/mere').resolve()
        removed.append({'path':str(p),'sha256':sha(p),'bytes':p.stat().st_size});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'compile_command':compile_cmd,'compile_exit':build.returncode,'exit':run.returncode,'passed':6,
 'source_sha256':{str(p.relative_to(ROOT)).replace('\\','/'):sha(p) for p in [source,smoke]},
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},'removed_outputs':removed,
 'evidence_sha256':{p.name:sha(p) for p in [Path(__file__),rs,HERE/'pelt_desktop_success_compile.txt',HERE/'pelt_desktop_success_output.txt']},
 'qualification':'Exact source-extracted startup/caller/outcome and complete netrender-smoke executable bodies with deterministic event-loop/window/SurfaceHost/GPU doubles. Callback visibility widened only. No whole desktop compilation, actual OS/GPU startup failure, real pixels, presentation or physical AT. CPU failure-propagation proof, not platform acceptance.'}
with (HERE/'pelt_desktop_success_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps(record,indent=2))
