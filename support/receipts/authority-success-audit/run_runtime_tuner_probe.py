"""Exact schedule/cache commit bodies, with an explicitly refusing CPU client double."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubecl-runtime/src/'
paths=[base+p for p in ['tune/schedule.rs','tune/tune_cache.rs','tune/tuner.rs','tune/log.rs','client.rs']]
schedule,cache,tuner,log,client=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
parts={'run_batch':function(schedule,"pub(crate) fn run_batch<'a"),'run_plan':function(schedule,"pub(crate) fn run_plan<'a"),'process_request':function(tuner,'async fn process_request<K:'),'cache':cache}
code=r'''#![allow(dead_code,unused_mut,unused_variables,private_interfaces)]
extern crate alloc;
extern crate self as cubecl_environment;
extern crate self as cubecl_common;
extern crate self as futures_util;
extern crate self as log;
#[macro_export]macro_rules! warn {($($arg:tt)*)=>{{let _=format!($($arg)*);}}}
pub mod collections {pub use std::collections::HashMap;}
pub mod future {
 pub fn block_on<F:std::future::Future>(f:F)->F::Output {
  let w=std::task::Waker::noop();let mut cx=std::task::Context::from_waker(w);let mut f=std::pin::pin!(f);
  loop {if let std::task::Poll::Ready(v)=f.as_mut().poll(&mut cx){return v;}}
 }
 pub async fn join_all<I:IntoIterator>(i:I)->Vec<<I::Item as std::future::Future>::Output> where I::Item:std::future::Future {
  let mut out=Vec::new();for f in i {out.push(f.await);}out
 }
}
pub mod profile {pub use std::time::Instant;}
pub mod dry_run {pub struct RealRun;impl RealRun{pub fn new()->Self{Self}}}
use std::{time::Duration,marker::PhantomData,sync::{Arc,atomic::{AtomicUsize,Ordering}}};
struct Mutex<T>(std::sync::Mutex<T>);impl<T> Mutex<T>{fn new(v:T)->Self{Self(std::sync::Mutex::new(v))}fn lock(&self)->std::sync::MutexGuard<'_,T>{self.0.lock().unwrap()}}
trait AutotuneKey:Clone+std::fmt::Debug+Eq+std::hash::Hash{}impl<T:Clone+std::fmt::Debug+Eq+std::hash::Hash> AutotuneKey for T{}
#[derive(Debug,Clone)]enum AutotuneError {Unknown{name:String,err:String},InvalidSamples{name:String},NotMeasured{name:String},Skip{name:String}}
#[derive(Debug,Clone,PartialEq)]struct Score(u64);impl Score{fn score(&self)->u64{self.0}}
#[derive(Debug,Clone,PartialEq)]struct AutotuneOutcome{index:usize,computation:Score}
mod cache {
'''+cache.replace('use super::{AutotuneError, AutotuneKey, AutotuneOutcome};','use super::{AutotuneError, AutotuneKey, AutotuneOutcome};')+r'''
}
use cache::{AutotuneResult,TuneCache,TuneCacheResult};
trait TuneInputs {type At<'a>:Clone+Send;}
impl TuneInputs for (){type At<'a>=();}
trait AutotuneOutput{}impl AutotuneOutput for (){}
struct TuneFn<F: TuneInputs,O>{name:String,marker:PhantomData<(F,O)>}
#[derive(Clone)]struct Client{entered:Arc<AtomicUsize>}
impl Client {fn exclusive<R,F:FnOnce()->R>(&self,_f:F)->Result<R,&'static str>{Err("exclusive acquisition refused before task execution")}}
#[derive(Debug)]struct Batch{entries:Vec<usize>}
impl Batch{fn indices(&self)->Vec<usize>{self.entries.clone()}fn is_empty(&self)->bool{self.entries.is_empty()}fn unconditional(entries:Vec<usize>)->Self{Self{entries}}}
#[derive(Debug)]struct TunePlan{batches:std::collections::VecDeque<Batch>}
impl TunePlan{fn next(&mut self)->Batch{self.batches.pop_front().unwrap_or(Batch{entries:vec![]})}}
#[derive(Debug)]struct BatchOutcome{results:Vec<(usize,AutotuneResult)>,steps:Vec<(String,Duration)>,short_circuit:Option<String>,any_success:bool,decided:Option<usize>,unmeasured:Vec<usize>}
#[derive(Debug)]struct PlanOutcome{steps:Vec<(String,Duration)>,short_circuit:Option<String>,decided:Option<usize>}
struct Schedule;
impl Schedule{
 async fn drive<'a,F:TuneInputs,O:AutotuneOutput>(&mut self,_b:Batch,_a:&[&TuneFn<F,O>],_i:F::At<'a>,c:&Client)->BatchOutcome{c.entered.fetch_add(1,Ordering::SeqCst);panic!("refusing client must not enter drive")}
'''+parts['run_batch']+'\n'+parts['run_plan']+r'''
}
struct Logger;
struct LogContext;
trait LogExt{fn push_tuning_step(&mut self,n:String,d:Duration);fn log_result<K:AutotuneKey>(&self,l:&mut Logger,k:&K,r:&[AutotuneResult]);}
impl LogExt for Option<LogContext>{fn push_tuning_step(&mut self,_n:String,_d:Duration){}fn log_result<K:AutotuneKey>(&self,_l:&mut Logger,_k:&K,_r:&[AutotuneResult]){}}
struct PendingBench{index:usize,name:String,launch:Option<Duration>}
async fn resolve_bench(_:PendingBench)->AutotuneResult{panic!("no pending samples in probe")}
struct TuneRequest<K:AutotuneKey>{key:K,results:Vec<AutotuneResult>,log_context:Option<LogContext>,pending:Vec<PendingBench>,decided:Option<usize>}
'''+parts['process_request']+r'''
fn op()->TuneFn<(),()>{TuneFn{name:"candidate0".into(),marker:PhantomData}}
#[test]fn refusal_before_execution_becomes_a_settled_unmeasured_winner(){
 let client=Client{entered:Arc::new(AtomicUsize::new(0))};let op=op();let mut results=vec![AutotuneResult::error(AutotuneError::Skip{name:op.name.clone()})];
 let mut plan=TunePlan{batches:[Batch{entries:vec![0]}].into()};
 let outcome=Schedule.run_plan(&17u32,&mut plan,&[&op],&(),&client,&mut results);
 assert_eq!(client.entered.load(Ordering::SeqCst),0);assert_eq!(outcome.decided,Some(0));
 assert!(matches!(results[0].outcome,Err(AutotuneError::Unknown{..})));
 let cache=Mutex::new(TuneCache::<u32>::new("audit","cpu-double"));cache.lock().mark_pending(17);
 let request=TuneRequest{key:17,results,log_context:None,pending:vec![],decided:outcome.decided};
 assert!(matches!(future::block_on(process_request(request,&cache,&Mutex::new(Logger))),TuneCacheResult::Hit{fastest_index:0}));
 assert_eq!(cache.lock().settled(&17),Some(0));
}
#[test]fn no_profile_measurement_is_a_different_all_failed_policy(){
 let client=Client{entered:Arc::new(AtomicUsize::new(0))};let op=op();let mut results=vec![AutotuneResult::error(AutotuneError::NotMeasured{name:op.name.clone()})];
 let mut plan=TunePlan{batches:[].into()};
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||Schedule.run_plan(&17u32,&mut plan,&[&op],&(),&client,&mut results))).is_err());
}
#[test]fn measured_success_control_is_committed_by_result_index(){
 let cache=Mutex::new(TuneCache::<u32>::new("audit","cpu-double"));
 let results=vec![AutotuneResult::success(AutotuneOutcome{index:4,computation:Score(10)}),AutotuneResult::success(AutotuneOutcome{index:7,computation:Score(2)})];
 let request=TuneRequest{key:17,results,log_context:None,pending:vec![],decided:None};
 assert!(matches!(future::block_on(process_request(request,&cache,&Mutex::new(Logger))),TuneCacheResult::Hit{fastest_index:7}));assert_eq!(cache.lock().settled(&17),Some(7));
}
'''
rs=HERE/'runtime_tuner_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-runtime-tuner.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'runtime_tuner_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'runtime_tuner_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '3 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':3,'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in parts.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_runtime_tuner_probe.py',rs.name,'runtime_tuner_compile.txt','runtime_tuner_run.txt']},
 'qualification':'Exact run_batch/run_plan/process_request plus full tune_cache.rs under non-persistence cfg. Explicit CPU client refuses exclusive acquisition and never enters drive; shape-equivalent plan/candidate/error/score/log/mutex types. No pending profile or real kernel, backend/GPU, browser, persistent Store, current contention or full runtime build. Non-persistence cache settled witness only; source independently excludes unmeasured decisions from disk. Owned exe/PDB removed.'}
with (HERE/'runtime_tuner_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':3,'removed_outputs':len(removed)}))
