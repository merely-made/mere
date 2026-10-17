#![allow(dead_code,private_interfaces,unused_variables)]
extern crate self as log;
extern crate self as cubecl_common;
#[macro_export]macro_rules! warn{($($t:tt)*)=>{{let _=format!($($t)*);}}}
use std::{sync::{Arc,Mutex},num::NonZero};
pub mod device {
 #[derive(Debug,Copy,Clone,PartialEq,Eq)]pub struct DeviceId{pub type_id:u16,pub index_id:u16}
 impl DeviceId{pub fn new(type_id:u16,index_id:u16)->Self{Self{type_id,index_id}}}
 pub trait Device:Sized{fn from_id(id:DeviceId)->Self;fn to_id(&self)->DeviceId;}
}
mod exact_device {
use cubecl_common::device::{Device, DeviceId};

/// The device struct when using the `wgpu` backend.
///
/// Note that you need to provide the device index when using a GPU backend.
///
/// # Example
///
/// ```ignore
/// use cubecl_wgpu::WgpuDevice;
///
/// let device_gpu_1 = WgpuDevice::DiscreteGpu(0); // First discrete GPU found.
/// let device_gpu_2 = WgpuDevice::DiscreteGpu(1);  // Second discrete GPU found.
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub enum WgpuDevice {
    /// Discrete GPU with the given index. The index is the index of the discrete GPU in the list
    /// of all discrete GPUs found on the system.
    DiscreteGpu(usize),

    /// Integrated GPU with the given index. The index is the index of the integrated GPU in the
    /// list of all integrated GPUs found on the system.
    IntegratedGpu(usize),

    /// Virtual GPU with the given index. The index is the index of the virtual GPU in the list of
    /// all virtual GPUs found on the system.
    VirtualGpu(usize),

    /// CPU.
    Cpu,

    /// The best available device found with the current [graphics API](crate::GraphicsApi).
    ///
    /// This will prioritize GPUs wgpu recognizes as "high power". Additionally, you can override this using
    /// the `CUBECL_WGPU_DEFAULT_DEVICE` environment variable. This variable is spelled as if i was a `WgpuDevice`,
    /// so for example `CUBECL_WGPU_DEFAULT_DEVICE=IntegratedGpu(1)` or `CUBECL_WGPU_DEFAULT_DEVICE=Cpu`
    #[default]
    DefaultDevice,

    /// Deprecated, use [`DefaultDevice`](WgpuDevice::DefaultDevice).
    #[deprecated]
    BestAvailable,

    /// Use an externally created, existing, wgpu setup. This is helpful when using `CubeCL` in conjunction
    /// with some existing wgpu setup (eg. egui or bevy), as resources can be transferred in & out of `CubeCL`.
    ///
    /// # Notes
    ///
    /// This can be initialized with [`init_device`](crate::runtime::init_device).
    Existing(u32),
}

impl Device for WgpuDevice {
    fn from_id(device_id: DeviceId) -> Self {
        match device_id.type_id {
            0 => Self::DiscreteGpu(device_id.index_id as usize),
            1 => Self::IntegratedGpu(device_id.index_id as usize),
            2 => Self::VirtualGpu(device_id.index_id as usize),
            3 => Self::Cpu,
            4 => Self::DefaultDevice,
            5 => Self::Existing(device_id.index_id as u32),
            _ => Self::DefaultDevice,
        }
    }

    fn to_id(&self) -> DeviceId {
        #[allow(deprecated)]
        match self {
            Self::DiscreteGpu(index) => DeviceId::new(0, *index as u16),
            Self::IntegratedGpu(index) => DeviceId::new(1, *index as u16),
            Self::VirtualGpu(index) => DeviceId::new(2, *index as u16),
            Self::Cpu => DeviceId::new(3, 0),
            Self::BestAvailable | WgpuDevice::DefaultDevice => DeviceId::new(4, 0),
            Self::Existing(id) => DeviceId::new(5, *id as u16),
        }
    }
}

}
use exact_device::WgpuDevice;use device::{Device,DeviceId};
mod wgpu {
 use super::*;
 pub const COPY_BUFFER_ALIGNMENT:u64=4;
 #[derive(Clone,Debug)]pub struct Buffer(pub Arc<Mutex<Vec<u8>>>);
 #[derive(Default)]pub struct QueueState{pub pending:Vec<(Buffer,usize,Vec<u8>)>,pub submissions:usize}
 #[derive(Default)]pub struct Queue(pub Arc<Mutex<QueueState>>);
 impl Queue {
  pub fn write_buffer(&self,b:&Buffer,o:u64,d:&[u8]){self.0.lock().unwrap().pending.push((b.clone(),o as usize,d.to_vec()));}
  pub fn write_buffer_with(&self,_b:&Buffer,_o:u64,_n:NonZero<u64>)->Option<WriteView>{panic!("unaligned staging branch not exercised")}
  pub fn submit<I:IntoIterator<Item=CommandBuffer>>(&self,_i:I)->SubmissionIndex{let mut s=self.0.lock().unwrap();let pending=std::mem::take(&mut s.pending);for(b,o,d)in pending{b.0.lock().unwrap()[o..o+d.len()].copy_from_slice(&d);}s.submissions+=1;SubmissionIndex(s.submissions)}
 }
 pub struct WriteView(Vec<u8>);impl WriteView{pub fn slice(&mut self,r:std::ops::Range<usize>)->&mut[u8]{&mut self.0[r]}}
 pub struct Device;impl Device{pub fn create_command_encoder(&self,_d:&CommandEncoderDescriptor<'_>)->CommandEncoder{CommandEncoder}pub fn poll(&self,_p:PollType)->Result<(),&'static str>{Ok(())}}
 pub struct CommandEncoder;impl CommandEncoder{pub fn finish(self)->CommandBuffer{CommandBuffer}}
 pub struct CommandBuffer;
 pub struct CommandEncoderDescriptor<'a>{pub label:Option<&'a str>}
 pub struct SubmissionIndex(pub usize);
 pub enum PollType{Wait{submission_index:Option<SubmissionIndex>,timeout:Option<std::time::Duration>}}
 #[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum DeviceType{Other,IntegratedGpu,DiscreteGpu,VirtualGpu,Cpu}
 #[derive(Clone,Debug)]pub struct Adapter{pub kind:DeviceType,pub name:&'static str}
 #[derive(Debug)]pub struct AdapterInfo{pub device_type:DeviceType,pub name:&'static str}
 impl Adapter{pub fn get_info(&self)->AdapterInfo{AdapterInfo{device_type:self.kind,name:self.name}}}
 #[derive(Clone,Copy)]pub struct Backend;
 pub struct Backends;impl Backends{pub fn all()->Self{Self}}impl From<Backend> for Backends{fn from(_:Backend)->Self{Self}}
 pub struct InstanceDescriptor{pub backends:Backends}impl InstanceDescriptor{pub fn new_without_display_handle()->Self{Self{backends:Backends}}}
 pub struct Instance{pub adapters:Vec<Adapter>}
 thread_local!{pub static ADAPTERS:std::cell::RefCell<Vec<Adapter>>=const{std::cell::RefCell::new(Vec::new())};}
 impl Instance{pub fn new(_:InstanceDescriptor)->Self{Self{adapters:ADAPTERS.with(|a|a.borrow().clone())}}pub async fn enumerate_adapters(&self,_b:Backends)->Vec<Adapter>{self.adapters.clone()}}
}
struct WgpuResource{buffer:wgpu::Buffer,offset:u64,size:u64}
#[derive(Copy,Clone)]struct StreamErrorMode{ignore:bool,flush:bool}
type ServerError=&'static str;
struct MemManage;impl MemManage{fn memory_cleanup(&mut self,_v:bool){}fn release_uniforms(&mut self){}}
struct SubmissionLoad;impl SubmissionLoad{fn regulate(&mut self,_d:&wgpu::Device,_t:usize,_i:wgpu::SubmissionIndex){}}
struct WgpuStream{tasks_count:usize,pending_write_count:usize,device:wgpu::Device,queue:wgpu::Queue,compute_pass:Option<()>,encoder:wgpu::CommandEncoder,submission_load:SubmissionLoad,mem_manage:MemManage}
impl WgpuStream{
 fn new()->Self{Self{tasks_count:0,pending_write_count:0,device:wgpu::Device,queue:wgpu::Queue::default(),compute_pass:None,encoder:wgpu::CommandEncoder,submission_load:SubmissionLoad,mem_manage:MemManage}}
 fn flush_errors(&mut self,_m:StreamErrorMode)->Result<(),ServerError>{Ok(())}
fn write_to_buffer(&mut self, resource: &WgpuResource, data: &[u8]) {
        // Nothing to write for zero-sized resources.
        if resource.size == 0 {
            return;
        }

        // Copying into a buffer has to be 4 byte aligned. We can safely do so, as
        // memory is also aligned (see WgpuStorage). Per the WebGPU spec, this
        // just has to be a multiple of 4: https://www.w3.org/TR/webgpu/#dom-gpuqueue-writebuffer
        let copy_align = wgpu::COPY_BUFFER_ALIGNMENT;
        let size = resource.size.next_multiple_of(copy_align);

        if size == data.len() as u64 {
            // write_buffer is the recommended way to write this data, as:
            // - On WebGPU, from WASM, this can save a copy to the JS memory.
            // - On devices with unified memory, this could skip the staging buffer entirely.
            self.queue
                .write_buffer(&resource.buffer, resource.offset, data);
        } else {
            // For sizes not aligned we need to only write a part of the staging buffer, do this
            // with `write_buffer_with`.
            let mut buffer = self
                .queue
                .write_buffer_with(
                    &resource.buffer,
                    resource.offset,
                    NonZero::new(size).unwrap(),
                )
                .expect("Internal error: Failed to call `write_buffer_with`, this likely means no staging buffer could be allocated.");
            buffer.slice(0..data.len()).copy_from_slice(data);
        }

        self.pending_write_count += 1;

        // Prevent wgpu staging buffer pool exhaustion during bulk writes (e.g. model
        // loading with hundreds of tensors). queue.write_buffer() is async — wgpu
        // copies data into an internal staging buffer, then transfers to GPU on the
        // next queue.submit(). Without periodic submits, hundreds of writes accumulate
        // and staging buffers get recycled before the GPU copy completes, silently
        // corrupting early tensors.
        // See: https://github.com/tracel-ai/cubecl/issues/1120
        const MAX_PENDING_WRITES: usize = 64;

        if self.pending_write_count >= MAX_PENDING_WRITES {
            // Submit a fresh, empty command buffer to flush all pending write_buffer work.
            // wgpu flushes its internal staging-buffer copies on any queue.submit(),
            // so we don't need to touch the main compute encoder here.
            let write_flush_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("CubeCL Write Flush Encoder"),
                    });
            let index = self.queue.submit([write_flush_encoder.finish()]);

            // Wait for the GPU to finish processing these writes before continuing.
            #[cfg(not(target_family = "wasm"))]
            if let Err(e) = self.device.poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            }) {
                log::warn!("wgpu: write flush poll failed ({e})");
            }

            self.pending_write_count = 0;
        }
    }
pub fn flush(&mut self, mode: StreamErrorMode) -> Result<(), ServerError> {
        if self.tasks_count == 0 {
            return self.flush_errors(mode);
        }

        // End the current compute pass.
        self.compute_pass = None;

        // Submit the pending actions to the queue. This will _first_ submit the
        // pending uniforms copy operations, then the main tasks.
        let tasks_encoder = {
            std::mem::replace(&mut self.encoder, {
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("CubeCL Tasks Encoder"),
                    })
            })
        };

        // This will _first_ fire off all pending write_buffer work.
        let index = self.queue.submit([tasks_encoder.finish()]);

        self.submission_load
            .regulate(&self.device, self.tasks_count, index);

        // Cleanup allocations and deallocations.
        self.mem_manage.memory_cleanup(false);
        self.mem_manage.release_uniforms();

        self.tasks_count = 0;
        self.pending_write_count = 0;

        self.flush_errors(mode)
    }
}
fn block_on<F:std::future::Future>(f:F)->F::Output{let mut f=std::pin::pin!(f);let mut cx=std::task::Context::from_waker(std::task::Waker::noop());loop{if let std::task::Poll::Ready(v)=f.as_mut().poll(&mut cx){return v;}}}
fn enumerate_all_adapters(i:wgpu::Instance,_b:wgpu::Backend)->Vec<wgpu::Adapter>{i.adapters}
struct Runtime;
impl Runtime{
fn enumerate_devices(type_id: u16, info: &wgpu::Backend) -> Vec<DeviceId> {
        #[cfg(target_family = "wasm")]
        {
            let _ = type_id;
            let _ = info;
            // WebGPU only supports a single device currently.
            vec![DeviceId::new(0, 0)]
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            });

            let adapters = enumerate_all_adapters(instance, *info);
            adapters
                .into_iter()
                .filter(|adapter| {
                    // Default doesn't filter device types.
                    if type_id == 4 {
                        return true;
                    }

                    let device_type = adapter.get_info().device_type;

                    let adapter_type_id = match device_type {
                        wgpu::DeviceType::Other => 4,
                        wgpu::DeviceType::IntegratedGpu => 1,
                        wgpu::DeviceType::DiscreteGpu => 0,
                        wgpu::DeviceType::VirtualGpu => 2,
                        wgpu::DeviceType::Cpu => 3,
                    };

                    adapter_type_id == type_id
                })
                .enumerate()
                .map(|(index, adapter)| match adapter.get_info().device_type {
                    wgpu::DeviceType::DiscreteGpu => DeviceId::new(0, index as u16),
                    wgpu::DeviceType::IntegratedGpu => DeviceId::new(1, index as u16),
                    wgpu::DeviceType::VirtualGpu => DeviceId::new(2, index as u16),
                    wgpu::DeviceType::Cpu => DeviceId::new(3, 0),
                    wgpu::DeviceType::Other => DeviceId::new(4, 0),
                })
                .collect()
        }
    }
fn enumerate_all_devices(info: &wgpu::Backend) -> Vec<DeviceId> {
        #[cfg(target_family = "wasm")]
        {
            let _ = info;
            // WebGPU only supports a single device currently.
            vec![DeviceId::new(0, 0)]
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            });
            let adapters = enumerate_all_adapters(instance, *info);
            adapters
                .into_iter()
                .enumerate()
                .map(|(index, adapter)| match adapter.get_info().device_type {
                    wgpu::DeviceType::DiscreteGpu => DeviceId::new(0, index as u16),
                    wgpu::DeviceType::IntegratedGpu => DeviceId::new(1, index as u16),
                    wgpu::DeviceType::VirtualGpu => DeviceId::new(2, index as u16),
                    wgpu::DeviceType::Cpu => DeviceId::new(3, 0),
                    wgpu::DeviceType::Other => DeviceId::new(4, 0),
                })
                .collect()
        }
    }
}
async fn select_from_adapter_list(
    num: usize,
    error: &str,
    instance: &wgpu::Instance,
    device: &WgpuDevice,
    backend: wgpu::Backend,
) -> wgpu::Adapter {
    let mut adapters_other = Vec::new();
    let mut adapters = Vec::new();

    instance
        .enumerate_adapters(backend.into())
        .await
        .into_iter()
        .for_each(|adapter| {
            let device_type = adapter.get_info().device_type;

            if let wgpu::DeviceType::Other = device_type {
                adapters_other.push(adapter);
                return;
            }

            let is_same_type = match device {
                WgpuDevice::DiscreteGpu(_) => device_type == wgpu::DeviceType::DiscreteGpu,
                WgpuDevice::IntegratedGpu(_) => device_type == wgpu::DeviceType::IntegratedGpu,
                WgpuDevice::VirtualGpu(_) => device_type == wgpu::DeviceType::VirtualGpu,
                WgpuDevice::Cpu => device_type == wgpu::DeviceType::Cpu,
                #[allow(deprecated)]
                WgpuDevice::DefaultDevice | WgpuDevice::BestAvailable => true,
                WgpuDevice::Existing(_) => {
                    unreachable!("Cannot select an adapter for an existing device.")
                }
            };

            if is_same_type {
                adapters.push(adapter);
            }
        });

    if adapters.len() <= num {
        if adapters_other.len() <= num {
            panic!(
                "{}, adapters {:?}, other adapters {:?}",
                error,
                adapters
                    .into_iter()
                    .map(|adapter| adapter.get_info())
                    .collect::<Vec<_>>(),
                adapters_other
                    .into_iter()
                    .map(|adapter| adapter.get_info())
                    .collect::<Vec<_>>(),
            );
        }

        return adapters_other.remove(num);
    }

    adapters.remove(num)
}
fn resource()->WgpuResource{WgpuResource{buffer:wgpu::Buffer(Arc::new(Mutex::new(vec![0;4]))),offset:0,size:4}}
fn mode()->StreamErrorMode{StreamErrorMode{ignore:false,flush:true}}
#[test]fn a_write_only_flush_returns_ok_but_does_not_submit(){let mut s=WgpuStream::new();let r=resource();s.write_to_buffer(&r,&[1,2,3,4]);assert_eq!(s.tasks_count,0);assert_eq!(s.pending_write_count,1);assert!(s.flush(mode()).is_ok());assert_eq!(s.queue.0.lock().unwrap().submissions,0);assert_eq!(s.queue.0.lock().unwrap().pending.len(),1);assert_eq!(*r.buffer.0.lock().unwrap(),vec![0;4]);}
#[test]fn a_compute_task_flush_control_submits_pending_writes(){let mut s=WgpuStream::new();let r=resource();s.write_to_buffer(&r,&[1,2,3,4]);s.tasks_count=1;assert!(s.flush(mode()).is_ok());assert_eq!(s.queue.0.lock().unwrap().submissions,1);assert_eq!(s.pending_write_count,0);assert_eq!(*r.buffer.0.lock().unwrap(),vec![1,2,3,4]);}
#[test]fn sixty_four_write_threshold_control_submits(){let mut s=WgpuStream::new();let r=resource();for _ in 0..64{s.write_to_buffer(&r,&[1,2,3,4]);}assert_eq!(s.queue.0.lock().unwrap().submissions,1);assert_eq!(s.pending_write_count,0);}
fn adapters()->wgpu::Instance{let adapters=vec![wgpu::Adapter{kind:wgpu::DeviceType::IntegratedGpu,name:"integrated"},wgpu::Adapter{kind:wgpu::DeviceType::DiscreteGpu,name:"discrete0"},wgpu::Adapter{kind:wgpu::DeviceType::DiscreteGpu,name:"discrete1"}];wgpu::ADAPTERS.with(|a|*a.borrow_mut()=adapters.clone());wgpu::Instance{adapters}}
#[test]fn enumerate_all_uses_global_index_then_selection_uses_per_type_index(){let instance=adapters();let ids=Runtime::enumerate_all_devices(&wgpu::Backend);assert_eq!(ids[1],DeviceId::new(0,1));let device=WgpuDevice::from_id(ids[1]);let selected=block_on(select_from_adapter_list(1,"none",&instance,&device,wgpu::Backend));assert_eq!(selected.name,"discrete1");assert_ne!(selected.name,instance.adapters[1].name);let device=WgpuDevice::from_id(ids[2]);assert!(std::panic::catch_unwind(||block_on(select_from_adapter_list(2,"none",&instance,&device,wgpu::Backend))).is_err());}
#[test]fn default_type_enumeration_has_same_mixed_adapter_mismatch(){let instance=adapters();let ids=Runtime::enumerate_devices(4,&wgpu::Backend);let device=WgpuDevice::from_id(ids[1]);assert_eq!(block_on(select_from_adapter_list(1,"none",&instance,&device,wgpu::Backend)).name,"discrete1");}
#[test]fn filtered_type_enumeration_control_uses_the_matching_index_space(){let instance=adapters();let ids=Runtime::enumerate_devices(0,&wgpu::Backend);assert_eq!(ids,vec![DeviceId::new(0,0),DeviceId::new(0,1)]);for(i,id)in ids.into_iter().enumerate(){let device=WgpuDevice::from_id(id);assert_eq!(block_on(select_from_adapter_list(i,"none",&instance,&device,wgpu::Backend)).name,instance.adapters[i+1].name);}}
#[test]fn existing_device_ids_truncate_before_u32_overflow_guard(){assert_ne!(WgpuDevice::Existing(0),WgpuDevice::Existing(65536));assert_eq!(WgpuDevice::Existing(0).to_id(),WgpuDevice::Existing(65536).to_id());}
