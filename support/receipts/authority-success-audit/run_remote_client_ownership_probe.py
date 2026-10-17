"""Unchanged remote helpers/rendezvous with actual Tokio and explicit IR doubles."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/burn-remote/src/'
paths=[base+p for p in ['lib.rs','client/runner.rs','client/service/pending.rs','server/local_comm.rs','shared/task.rs']]
texts=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
finish=function(texts[0],'fn finishes_within(')
counter=function(texts[1],'fn get_next_transfer_id(')
transfer=texts[4][texts[4].index('#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]'):texts[4].index('/// Unforgeable, bounded-use capability')]
transfer=transfer.replace(', Serialize, Deserialize','')
code=r'''#![allow(dead_code,private_interfaces)]
extern crate self as burn_ir;
pub trait BackendIr {type Handle;}
pub type HandleKind<B> = <B as BackendIr>::Handle;
struct Cpu;impl BackendIr for Cpu{type Handle=i32;}
mod shared {
 pub type SessionId=u64;
 pub type RequestId=u64;
 pub enum TaskResponseContent {SyncBackend(Result<(),()>)}
'''+transfer+'\n}\n'+finish+'\nmod pending {\n'+texts[2]+'\n}\nmod local {\n'+texts[3]+'\n}\n'
for name in ['independent_process_a','independent_process_b']:
 code+='mod '+name+' { use std::sync::Mutex; use crate::shared::LocalTransferId; static TRANSFER_COUNTER:Mutex<Option<LocalTransferId>>=Mutex::new(None);\n'+counter+'\npub fn next()->LocalTransferId{get_next_transfer_id()}\n}\n'
code+=r'''
#[cfg(test)]mod cases {
 use super::*;
 fn rt()->tokio::runtime::Runtime{tokio::runtime::Builder::new_current_thread().build().unwrap()}
 #[test]fn panic_in_server_survival_body_is_reported_finished(){
  assert!(finishes_within(std::time::Duration::from_secs(2),||{assert_eq!(1,2,"synthetic wrong server result");}));
 }
 #[test]fn completed_body_control_is_finished(){assert!(finishes_within(std::time::Duration::from_secs(2),||{}));}
 #[test]fn blocked_body_control_is_not_finished_and_is_released(){
  let(release,wait)=std::sync::mpsc::channel();let(done,exited)=std::sync::mpsc::channel();
  assert!(!finishes_within(std::time::Duration::from_millis(5),move||{wait.recv().unwrap();done.send(()).unwrap();}));
  release.send(()).unwrap();exited.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
 }
 #[test]fn independent_client_counters_collide_in_shared_server_registry(){rt().block_on(async{
  // Module-local statics stand for two separately linked client processes, each starting at0.
  let a=independent_process_a::next();let b=independent_process_b::next();assert_eq!(a,b);
  let server=local::LocalCommService::<Cpu>::new();
  server.expose(100,a,111).await;server.expose(200,b,222).await;
  assert_eq!(server.take(a).await,222);assert!(server.is_empty().await);
  // A second client's payload was returned for the first client's transfer id;
  // one pending payload has been overwritten and the other target has nothing left.
  println!("separate client counter id collision returned second session payload for first id");
 });}
 #[test]fn distinct_transfer_ids_control_preserves_both_payloads(){rt().block_on(async{
  let server=local::LocalCommService::<Cpu>::new();
  let a=shared::LocalTransferId::from(31);let b=shared::LocalTransferId::from(32);
  server.expose(100,a,111).await;server.expose(200,b,222).await;
  assert_eq!(server.take(a).await,111);assert_eq!(server.take(b).await,222);assert!(server.is_empty().await);
 });}
 #[test]fn source_session_purge_control_preserves_other_source(){rt().block_on(async{
  let server=local::LocalCommService::<Cpu>::new();
  let a=shared::LocalTransferId::from(41);let b=shared::LocalTransferId::from(42);
  server.expose(100,a,111).await;server.expose(200,b,222).await;
  server.purge_session(100).await;assert_eq!(server.take(b).await,222);assert!(server.is_empty().await);
 });}
}
'''
rs=HERE/'remote_client_ownership_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
deps=Path('C:/t/cargo-targets/mere/build/debug/deps');tokio=deps/'libtokio-de8334592412b821.rlib';assert tokio.is_file()
exe=Path('C:/t/cargo-targets/mere/audit-remote-client-ownership.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024','--cfg','feature="client"',str(rs),'--extern','tokio='+str(tokio),'-L','dependency='+str(deps),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'remote_client_ownership_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'remote_client_ownership_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '12 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':12,'original_pending_response_tests':6,'helper_cases':3,'rendezvous_cases':3,'source_sha256':{p:sha(ROOT/p) for p in paths},'external_context_sha256':{str(tokio):sha(tokio)},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extraction_sha256':{'finishes_within':hashlib.sha256(finish.encode()).hexdigest(),'get_next_transfer_id':hashlib.sha256(counter.encode()).hexdigest(),'LocalTransferId_without_serde_derives':hashlib.sha256(transfer.encode()).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_remote_client_ownership_probe.py',rs.name,'remote_client_ownership_compile.txt','remote_client_ownership_run.txt']},
 'qualification':'Exact finishes_within and counter functions; full unchanged PendingResponses and LocalCommService source compiled against actual existing Tokio. Backend/tensor payload is i32, SessionId u64 and response content a minimal enum; LocalTransferId only loses serde derives. Two module-local counter statics represent independent process-local state; no live separate processes, real tensor allocation, server, transport, credential or GPU. Demonstrates counter namespace/registry overwrite and panic admission, not an observed real deployment mixup.6 unchanged pending-response tests also run. Owned exe/PDB removed.'}
with (HERE/'remote_client_ownership_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':12,'removed_outputs':len(removed)}))
