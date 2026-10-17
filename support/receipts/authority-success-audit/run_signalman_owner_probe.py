"""Extract unchanged Signalman reservations and stop/event methods for CPU probes."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sources=['ports/signalman/src/bin/mere-signalman-first-owner.rs','ports/signalman/src/lib.rs']
counter=(ROOT/sources[0]).read_text(encoding='utf-8-sig'); station=(ROOT/sources[1]).read_text(encoding='utf-8-sig');pieces={}
def cut(text,key,start,end):
 a=text.index(start);b=text.index(end,a);pieces[key]=text[a:b];return pieces[key]
reservation=cut(counter,'reservation','fn reserve_next_counter(', '\nasync fn claim(')
event=cut(station,'next_event','    pub async fn next_event(&self)', '    /// Deliberately stop the station,')
stop=cut(station,'stop','    pub async fn stop(&self)', '    /// Revoke this station at the host,')
revoke=cut(station,'revoke','    pub async fn revoke(&self)', '    fn authorize_now(')
code=r'''
#![allow(dead_code)]
use ::std::{error::Error,fs::OpenOptions,io,io::Write,path::{Path,PathBuf},sync::{Arc,atomic::{AtomicBool,Ordering}}};
use tokio::sync::Mutex;
use ::std::time::Duration;
#[derive(serde::Serialize,serde::Deserialize)]struct ControllerCounter{last_used:u64}
static READS: ::std::sync::OnceLock<::std::sync::Barrier> = ::std::sync::OnceLock::new();
static FIRST_DONE:(::std::sync::Mutex<bool>,::std::sync::Condvar)=(::std::sync::Mutex::new(false),::std::sync::Condvar::new());
// Delay only the return of an actual disk read. Both callers have read the
// same existing counter, and B resumes after A has completely committed.
mod std {
 pub use ::std::*;
 pub mod fs {
  pub fn read_to_string(p:impl AsRef<::std::path::Path>)->::std::io::Result<String>{
   let result=::std::fs::read_to_string(p);
   if let Some(barrier)=crate::READS.get(){
    barrier.wait();
    if ::std::thread::current().name()==Some("counter-B"){
     let (lock,wake)=&crate::FIRST_DONE;
     let mut done=lock.lock().unwrap();while !*done {done=wake.wait(done).unwrap();}
    }
   }
   result
  }
  pub use ::std::fs::rename;
 }
}
'''+reservation+r'''
#[derive(Debug)]enum SitedStationError{Stopped,Station(()),Denied}
type Event=();type RemoteAuthRevocationOutcome=();
struct FakeStation;
impl FakeStation{async fn next_event(&mut self)->Option<Event>{::std::future::pending().await}}
#[derive(Clone)]struct Lease(Arc<AtomicBool>);
impl Lease{
 fn close(&self){self.0.store(true,Ordering::SeqCst);}
 fn revoke(&self)->Result<RemoteAuthRevocationOutcome,SitedStationError>{self.close();Ok(())}
}
struct SitedStation{station:Arc<Mutex<Option<FakeStation>>>,lease:Lease,watchdog:tokio::task::JoinHandle<()>}
fn remaining_window(a:u64,b:u64)->Duration{Duration::from_millis(b-a)}
impl SitedStation {
 fn authorize_now(&self)->Result<(u64,u64),SitedStationError>{if self.lease.0.load(Ordering::SeqCst){Err(SitedStationError::Denied)}else{Ok((0,3_600_000))}}
 async fn stop_after_deadline<T>(&self)->Result<T,SitedStationError>{Err(SitedStationError::Denied)}
'''+event+stop+revoke+r'''
}
#[test]
fn concurrent_counter_reservations_both_succeed_with_same_value(){
 let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("counter-fixture");assert!(!root.exists());::std::fs::create_dir(&root).unwrap();
 ::std::fs::write(controller_counter_path(&root),br#"{"last_used":0}"#).unwrap();
 READS.set(::std::sync::Barrier::new(2)).unwrap();
 let a=root.clone();let first=::std::thread::Builder::new().name("counter-A".into()).spawn(move||{
  let result=reserve_next_counter(&a).map_err(|e|e.to_string());
  let(lock,wake)=&FIRST_DONE;*lock.lock().unwrap()=true;wake.notify_one();result
 }).unwrap();
 let b=root.clone();let second=::std::thread::Builder::new().name("counter-B".into()).spawn(move||reserve_next_counter(&b).map_err(|e|e.to_string())).unwrap();
 let a=first.join().unwrap().unwrap();let b=second.join().unwrap().unwrap();assert_eq!((a,b),(1,1));
 let raw=::std::fs::read(controller_counter_path(&root)).unwrap();assert_eq!(serde_json::from_slice::<ControllerCounter>(&raw).unwrap().last_used,1);
 // B's successful reservation left no remaining record of A's spend.
 ::std::fs::remove_file(controller_counter_path(&root)).unwrap();::std::fs::remove_dir(&root).unwrap();
 println!("counter-A=1 counter-B=1 disk=1; both actual synced writes and renames succeeded");
}
async fn cancellation_case(revoking:bool){
 let owner=Arc::new(SitedStation{station:Arc::new(Mutex::new(Some(FakeStation))),lease:Lease(Arc::new(AtomicBool::new(false))),watchdog:tokio::spawn(::std::future::pending())});
 let waiter=owner.clone();let task=tokio::spawn(async move{waiter.next_event().await});tokio::task::yield_now().await;
 assert!(owner.station.try_lock().is_err(),"actual next_event holds mutex");
 let action=async{if revoking {owner.revoke().await.map(|_|())}else{owner.stop().await;Ok(())}};
 assert!(tokio::time::timeout(Duration::from_millis(10),action).await.is_err());
 assert!(owner.lease.0.load(Ordering::SeqCst));assert!(!task.is_finished());
 assert!(owner.station.try_lock().is_err());
 // Releasing the simulated radio wait is sufficient for the original stop.
 task.abort();let _=task.await;owner.stop().await;assert!(owner.station.lock().await.is_none());
 println!("{} closed lease but could not cancel pending event; explicit task abort releases mutex",if revoking{"revoke"}else{"stop"});
}
#[tokio::test(start_paused=true)]async fn stop_cannot_interrupt_in_flight_event(){cancellation_case(false).await;}
#[tokio::test(start_paused=true)]async fn successful_revoke_cannot_interrupt_in_flight_event(){cancellation_case(true).await;}
#[tokio::test(start_paused=true)]async fn idle_stop_control_completes(){
 let owner=SitedStation{station:Arc::new(Mutex::new(Some(FakeStation))),lease:Lease(Arc::new(AtomicBool::new(false))),watchdog:tokio::spawn(::std::future::pending())};
 owner.stop().await;assert!(owner.station.lock().await.is_none());assert!(owner.lease.0.load(Ordering::SeqCst));
}
'''
rs=HERE/'signalman_owner_probe.rs';rs.write_text(code,encoding='utf-8')
manifest=HERE/'cpu-signalman-runtime/Cargo.toml';lock=manifest.with_name('Cargo.lock')
assert not lock.exists();lock.write_bytes((ROOT/'Cargo.lock').read_bytes())
target=Path('C:/t/cargo-targets/mere');outputs=target/'build/debug/deps';before={p.name for p in outputs.glob('authority_success_signalman*') if p.suffix in {'.exe','.pdb'}};assert not before
command=['cargo','test','--offline','--manifest-path',str(manifest),'--target-dir',str(target),'--config','build.build-dir="C:/t/cargo-targets/mere/build"','--lib','--','--nocapture','--test-threads=1']
result=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,env=os.environ,timeout=180)
log=HERE/'signalman_owner_probe_run.txt';log.write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr)
assert result.returncode==0 and '4 passed; 0 failed' in result.stdout
removed=[]
for out in outputs.glob('authority_success_signalman*'):
 if out.suffix not in {'.exe','.pdb'}:continue
 assert out.resolve().parent==outputs.resolve();raw=out.read_bytes()
 assert (out.suffix=='.exe' and raw.startswith(b'MZ')) or (out.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(out),'bytes':len(raw),'sha256':sha(out)});out.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':result.returncode,'passed':4,'command':command,
 'source_sha256':{p:sha(ROOT/p) for p in sources},'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},
 'evidence_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in [Path(__file__),manifest,lock,rs,log]},'removed_outputs':removed,'retained_isolation':[],
 'qualification':'Exact reservation, stop, revoke, next_event bodies extracted without edits. Counter uses real filesystem, serde, synced writes/renames with deterministic delays after actual read to schedule two callers; no radio or board replay verifier run. Station uses actual Tokio mutex/time/select with virtual time, fake pending radio, lease/revoke/clock and disabled watchdog doubles; tests prove mutex/cancellation behavior, not full hardware cessation or real Djinn revocation. Four cases include idle-stop control. Source unchanged.'}
with (HERE/'signalman_owner_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':4,'removed_outputs':len(removed)}))
