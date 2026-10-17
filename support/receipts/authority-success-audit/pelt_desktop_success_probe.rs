
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
        pub(crate) fn outcome(&self) -> StaticViewerOutcome {
            StaticViewerOutcome {
                url: self.doc.address().unwrap_or(&self.config.url).to_owned(),
                created_window: self.window.is_some(),
                redraws: self.redraws,
                size: if self.window.is_some() {
                    (self.width, self.height)
                } else {
                    (0, 0)
                },
                product_receipt: self.receipt_outcome.clone(),
            }
        }

        pub(crate) fn receipt_failure(&self) -> Option<&str> {
            self.receipt_failure.as_deref()
        }
        pub fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let attributes =
                super::pelt_window_attributes(self.window_title(), self.width, self.height);
            let window = match event_loop.create_window(attributes) {
                Ok(window) => Arc::new(window),
                Err(err) => {
                    eprintln!("[pelt-viewer] could not create window: {err}");
                    event_loop.exit();
                    return;
                },
            };
            let size = window.inner_size();
            self.width = size.width.max(1);
            self.height = size.height.max(1);
            self.scale_factor = window.scale_factor() as f32;
            window.set_title(&self.window_title());
            let options = NetrenderOptions {
                tile_cache_size: Some(64),
                enable_vello: true,
                ..Default::default()
            };
            match SurfaceHost::boot(window.clone(), self.width, self.height, options) {
                Ok(host) => self.host = Some(host),
                Err(err) => {
                    eprintln!("[pelt-viewer] {err}");
                    event_loop.exit();
                    return;
                },
            }
            window.request_redraw();
            self.window = Some(window);
        }

}}
pub(crate) fn run_headed_with<C: windowed::ViewerContent + 'static>(
    config: StaticViewerConfig,
    content: C,
) -> Result<StaticViewerOutcome, String> {
    use winit::event_loop::EventLoop;

    let event_loop =
        EventLoop::new().map_err(|error| format!("could not create event loop: {error}"))?;
    let mut app = windowed::ViewerApp::new(config, content);
    event_loop
        .run_app(&mut app)
        .map_err(|error| format!("viewer event loop failed: {error}"))?;
    if let Some(error) = app.receipt_failure() {
        return Err(error.to_owned());
    }
    Ok(app.outcome())
}

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
pub fn run_netrender_smoke() -> Result<NetrenderSmokeOutcome, String> {
    const DIM: u32 = 64;

    let handles =
        netrender::boot().map_err(|error| format!("netrender wgpu boot failed: {error}"))?;
    let device = handles.device.clone();
    let renderer = netrender::create_netrender_instance(
        handles,
        netrender::NetrenderOptions {
            tile_cache_size: Some(32),
            enable_vello: true,
            ..Default::default()
        },
    )
    .map_err(|error| format!("netrender renderer init failed: {error:?}"))?;

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pelt netrender smoke target"),
        size: wgpu::Extent3d {
            width: DIM,
            height: DIM,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor {
        label: Some("pelt netrender smoke view"),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        ..Default::default()
    });

    let mut scene = netrender::Scene::new(DIM, DIM);
    scene.push_rect(0.0, 0.0, DIM as f32, DIM as f32, [1.0, 0.0, 0.0, 1.0]);
    renderer.render_vello(&scene, &view, netrender::ColorLoad::default());

    let bytes = renderer.wgpu_device.read_rgba8_texture(&target, DIM, DIM);
    let painted_pixels = bytes
        .chunks_exact(4)
        .filter(|rgba| rgba[0] != 0 || rgba[1] != 0 || rgba[2] != 0 || rgba[3] != 0)
        .count();

    Ok(NetrenderSmokeOutcome {
        width: DIM,
        height: DIM,
        painted_pixels,
    })
}

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
