"""CPU witnesses for selected runtime stream and guard ownership."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubecl-runtime/src/'
paths=[base+p for p in ['client.rs','dry_run.rs','logging/observer.rs','memory_management/policy.rs']]
client,dry,observer,policy=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
window=function(client,"fn allocation_window<'a")
obs=observer[observer.index('pub struct LaunchObservation {'):observer.index('impl core::fmt::Debug for LaunchObservation')]
obs_functions='\n'.join(function(observer,k) for k in ['pub(crate) fn is_observing()',"pub(crate) fn notify_launch("])
mode=policy[policy.index('#[derive(Default, Clone, Copy, Debug)]'):policy.index('/// The pool position stamped on persistent-pool slices')]
dry_without_serial=dry.replace('    #[serial_test::serial]\n','')
code=r'''#![allow(dead_code,private_interfaces)]
extern crate alloc;
extern crate self as cubecl_environment;
pub mod sync{pub use std::sync::atomic::{AtomicUsize,Ordering};}
mod dry {
'''+dry_without_serial+r'''
#[cfg(test)]mod audit {
 use super::*;
 #[test]fn moving_real_run_then_dropping_it_leaves_origin_enabled(){
  let _dry=DryRun::new();let real=RealRun::new();
  std::thread::spawn(move||drop(real)).join().unwrap();
  assert_eq!(real_run::depth(),1);assert_eq!(launch_mode(),LaunchMode::Execute);
  real_run::exit();assert_eq!(launch_mode(),LaunchMode::Skip);
 }
 #[test]fn foreign_real_run_drop_disables_another_threads_live_guard(){
  let _dry=DryRun::new();let foreign=RealRun::new();
  std::thread::spawn(move||{
   let own=RealRun::new();assert_eq!(launch_mode(),LaunchMode::Execute);
   drop(foreign);assert_eq!(real_run::depth(),0);assert_eq!(launch_mode(),LaunchMode::Skip);
   drop(own);
  }).join().unwrap();
  assert_eq!(real_run::depth(),1);real_run::exit();
 }
}
}
mod allocation {
use std::{collections::HashMap,sync::{Arc,Mutex}};
'''+mode+r'''
#[derive(Clone,Copy,Hash,PartialEq,Eq,Debug)]struct StreamId(u64);
impl StreamId{fn current()->Self{Self(1)}}
#[derive(Default)]struct Server{modes:HashMap<StreamId,MemoryAllocationMode>,events:Vec<(StreamId,MemoryAllocationMode)>}
impl Server{fn allocation_mode(&mut self,mode:MemoryAllocationMode,stream:StreamId){self.modes.insert(stream,mode);self.events.push((stream,mode));}}
#[derive(Default,Clone)]struct Device(Arc<Mutex<Server>>);
impl Device{fn submit(&self,call:impl FnOnce(&mut Server)){call(&mut self.0.lock().unwrap());}}
#[derive(Clone)]struct Client{device:Device,pinned:Option<StreamId>}
impl Client{
 fn stream_id(&self)->StreamId{self.pinned.unwrap_or_else(StreamId::current)}
 fn observed_mode(&self)->MemoryAllocationMode{self.device.0.lock().unwrap().modes.get(&self.stream_id()).copied().unwrap_or_default()}
'''+window+r'''
}
#[cfg(test)]mod cases{
use super::*;
#[test]fn ordinary_stream_window_control_routes_and_resets(){
 let client=Client{device:Device::default(),pinned:None};let observed=client.clone();
 client.allocation_window(MemoryAllocationMode::Persistent,(),move|()|assert!(matches!(observed.observed_mode(),MemoryAllocationMode::Persistent)));
 assert!(matches!(client.observed_mode(),MemoryAllocationMode::Auto));assert_eq!(client.device.0.lock().unwrap().events.len(),2);
}
#[test]fn pinned_client_window_changes_other_stream_not_its_allocation_stream(){
 let client=Client{device:Device::default(),pinned:Some(StreamId(2))};let observed=client.clone();
 client.allocation_window(MemoryAllocationMode::Persistent,(),move|()|assert!(matches!(observed.observed_mode(),MemoryAllocationMode::Auto)));
 let server=client.device.0.lock().unwrap();assert_eq!(server.events.len(),2);assert!(server.events.iter().all(|(stream,_)|*stream==StreamId(1)));
}
#[test]fn unwinding_callback_skips_reset_command(){
 let client=Client{device:Device::default(),pinned:None};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||client.allocation_window(MemoryAllocationMode::Dedicated,(),|()|panic!("synthetic caller unwind"))));
 assert!(result.is_err());assert!(matches!(client.observed_mode(),MemoryAllocationMode::Dedicated));assert_eq!(client.device.0.lock().unwrap().events.len(),1);
}
}
}
mod observation{
use std::{sync::{Arc,Mutex},sync::atomic::{AtomicBool,Ordering}};
struct RwLock<T>(std::sync::RwLock<T>);
impl<T>RwLock<T>{const fn new(v:T)->Self{Self(std::sync::RwLock::new(v))}fn read(&self)->std::sync::RwLockReadGuard<'_,T>{self.0.read().unwrap()}fn write(&self)->std::sync::RwLockWriteGuard<'_,T>{self.0.write().unwrap()}}
trait LaunchObserver:Send+Sync{fn launched(&self,kernel:&'static str);}
static OBSERVING:AtomicBool=AtomicBool::new(false);
static OBSERVER:RwLock<Option<Arc<dyn LaunchObserver>>>=RwLock::new(None);
'''+obs+obs_functions+r'''
#[cfg(test)]mod cases{
use super::*;
#[derive(Default)]struct Recorder(Mutex<Vec<&'static str>>);
impl LaunchObserver for Recorder{fn launched(&self,k:&'static str){self.0.lock().unwrap().push(k);}}
#[test]fn reverse_order_control_restores_and_ends_observers(){
 let a=Arc::new(Recorder::default());let b=Arc::new(Recorder::default());
 let outer=LaunchObservation::new(a.clone());let inner=LaunchObservation::new(b.clone());
 notify_launch("inner");drop(inner);notify_launch("outer");drop(outer);notify_launch("ended");
 assert_eq!(*a.0.lock().unwrap(),vec!["outer"]);assert_eq!(*b.0.lock().unwrap(),vec!["inner"]);assert!(!is_observing());
}
#[test]fn documented_taken_order_loses_active_then_resurrects_ended_observer(){
 let a=Arc::new(Recorder::default());let b=Arc::new(Recorder::default());
 let first=LaunchObservation::new(a.clone());let second=LaunchObservation::new(b.clone());
 drop(first);notify_launch("second_still_alive");assert!(b.0.lock().unwrap().is_empty());
 drop(second);notify_launch("both_ended");assert_eq!(*a.0.lock().unwrap(),vec!["both_ended"]);assert!(is_observing());
 *OBSERVER.write()=None;OBSERVERING_CLEANUP();
}
#[allow(non_snake_case)]fn OBSERVERING_CLEANUP(){OBSERVING.store(false,Ordering::Relaxed);}
}
}
'''
rs=HERE/'runtime_scope_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-runtime-scope.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024','--cfg','feature="std"',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'runtime_scope_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'runtime_scope_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '11 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':11,'original_dry_run_tests':4,'thread_affinity_cases':2,'allocation_window_cases':3,'observation_cases':2,'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in {'allocation_window':window,'LaunchObservation':obs,'observation_functions':obs_functions,'MemoryAllocationMode':mode,'dry_run_without_serial_test_attributes':dry_without_serial}.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_runtime_scope_probe.py',rs.name,'runtime_scope_compile.txt','runtime_scope_run.txt']},
 'qualification':'Selected runtime source: full dry_run.rs with original4 tests (only serial_test attributes removed; runner serializes),2 additional actual std-thread guard tests. Exact allocation_window function with explicit immediate-device/server/stream doubles; no allocation/GPU claim. Exact observer guard/new/drop/notify methods with minimal observer trait and std RwLock adapter; sequential documented-order/stack-order cases, no fabricated concurrency proof.11 CPU cases pass. Owned exe/PDB removed; no full backend/runtime build or device execution.'}
with (HERE/'runtime_scope_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':11,'removed_outputs':len(removed)}))
