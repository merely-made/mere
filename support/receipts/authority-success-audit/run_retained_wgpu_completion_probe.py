"""Exact retained queue-flush and adapter enumeration/selection source with CPU doubles."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubecl-wgpu/src/'
paths=[base+p for p in ['compute/stream.rs','runtime.rs','device.rs']]
stream,runtime,device=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
parts={'write_to_buffer':function(stream,'fn write_to_buffer('),'flush':function(stream,'pub fn flush('),'enumerate_all_devices':function(runtime,'fn enumerate_all_devices('),'enumerate_devices':function(runtime,'fn enumerate_devices('),'select_from_adapter_list':function(runtime,'async fn select_from_adapter_list('),'device':device}
code=r'''#![allow(dead_code,private_interfaces,unused_variables)]
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
'''+device+r'''
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
'''+parts['write_to_buffer']+'\n'+parts['flush']+r'''
}
fn block_on<F:std::future::Future>(f:F)->F::Output{let mut f=std::pin::pin!(f);let mut cx=std::task::Context::from_waker(std::task::Waker::noop());loop{if let std::task::Poll::Ready(v)=f.as_mut().poll(&mut cx){return v;}}}
fn enumerate_all_adapters(i:wgpu::Instance,_b:wgpu::Backend)->Vec<wgpu::Adapter>{i.adapters}
struct Runtime;
impl Runtime{
'''+parts['enumerate_devices']+'\n'+parts['enumerate_all_devices']+r'''
}
'''+parts['select_from_adapter_list']+r'''
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
'''
rs=HERE/'retained_wgpu_completion_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-retained-wgpu-completion.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'retained_wgpu_completion_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'retained_wgpu_completion_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '7 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
external=[Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/api')/p for p in ['queue.rs','device.rs','buffer.rs','adapter.rs']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':7,'source_sha256':{p:sha(ROOT/p) for p in paths},'external_context_sha256':{str(p):sha(p) for p in external},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in parts.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_retained_wgpu_completion_probe.py',rs.name,'retained_wgpu_completion_compile.txt','retained_wgpu_completion_run.txt']},
 'qualification':'Exact write_to_buffer/flush/enumerate_devices/enumerate_all_devices/select_from_adapter_list plus full device.rs. CPU queue, adapter inventory, DeviceId and stream field doubles; aligned write path only. Queue submit model matches inspected local wgpu30.0.1 API contract. No actual sync callback, real GPU, driver enumeration, existing-device registration, full backend build or current deployed failure. Root-unselected retained0.10 copy. Owned exe/PDB removed.'}
with (HERE/'retained_wgpu_completion_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':7,'removed_outputs':len(removed)}))
