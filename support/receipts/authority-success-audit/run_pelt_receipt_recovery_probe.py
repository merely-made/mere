"""Execute exact receipt timeout/acquire-recovery bodies with an unavailable swap double."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
parent=ROOT/'ports/pelt/desktop/workspace_viewer.rs'
routing=parent.with_suffix('')/'receipts/routing.rs'
p=parent.read_text(encoding='utf-8-sig');r=routing.read_text(encoding='utf-8-sig');pieces={}
def cut(s,name,start,end):
    a=s.index(start);b=s.index(end,a);pieces[name]=s[a:b];return pieces[name]
receipt=cut(p,'receipt_enum','pub enum WorkspaceReceipt {','\nimpl WorkspaceReceipt {')
ident=cut(p,'receipt_id','    pub fn id(self)', '\n    fn default_size(')
timeout=cut(r,'workspace_receipt_timeout_error','    pub(in crate::workspace_viewer) fn workspace_receipt_timeout_error(', '    pub(in crate::workspace_viewer) fn drive_mixed_workspace_receipt_step(')
timeout=timeout.replace('pub(in crate::workspace_viewer)','pub')
failed=cut(p,'primary_failed_acquire','        let Some(swap) = host.acquire() else {', '\n        let swap_target = swap')
startup=cut(p,'primary_startup_retry','fn primary_startup_retry(', '\nfn workspace_idle_flow(')
code=r'''
#![allow(dead_code,unused_variables)]
use std::cell::Cell;
use std::time::{Instant,Duration};
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
'''+receipt+'\nimpl WorkspaceReceipt {\n'+ident+'\n}\n'+startup+r'''
struct Config{workspace_receipt:Option<WorkspaceReceipt>,tearout_receipt:bool,tearout_cancellation_receipt:bool,workspace_receipt_stage_timeout:Duration}
#[derive(Default)]struct Stats{frames:u32,imports:u32,waits:u32,compositions:u32}
struct NativeSurfaces;impl NativeSurfaces{fn stats(&self)->Stats{Stats::default()}}
struct Acquirer;impl Acquirer{fn acquire(&self)->Option<()>{None}}
struct ActiveEventLoop{exits:Cell<u32>}impl ActiveEventLoop{fn exit(&self){self.exits.set(self.exits.get()+1)}}
struct WorkspaceApp{config:Config,receipt_complete:bool,workspace_receipt_stage_started:Instant,width:u32,height:u32,native_surfaces:NativeSurfaces,mixed_content_ready_frame:Option<u32>,mixed_verified_sizes:Vec<(u32,u32)>,receipt_error:Option<String>,requested_redraws:Cell<u32>}
impl WorkspaceApp {
fn request_redraw(&self){self.requested_redraws.set(self.requested_redraws.get()+1)}
fn tearout_receipt_timeout_error(&self)->Option<String>{None}
'''+timeout+r'''
fn attempt_capture_acquire(&mut self,event_loop:&ActiveEventLoop){let host=Acquirer;
'''+failed+r'''
drop(swap);
}}
fn app(receipt:WorkspaceReceipt,complete:bool,expired:bool)->WorkspaceApp{WorkspaceApp{config:Config{workspace_receipt:Some(receipt),tearout_receipt:false,tearout_cancellation_receipt:false,workspace_receipt_stage_timeout:Duration::from_secs(30)},receipt_complete:complete,workspace_receipt_stage_started:if expired {Instant::now()-Duration::from_secs(60)}else{Instant::now()},width:960,height:640,native_surfaces:NativeSurfaces,mixed_content_ready_frame:None,mixed_verified_sizes:vec![],receipt_error:None,requested_redraws:Cell::new(0)}}
#[test]fn semantic_completion_disables_error_and_recovery_while_capture_is_unavailable(){let mut a=app(WorkspaceReceipt::Appearance,true,true);assert!(a.workspace_receipt_timeout_error().is_none());let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert_eq!(a.requested_redraws.get(),0);assert!(a.receipt_error.is_none());assert_eq!(e.exits.get(),0);assert!(!primary_startup_retry(1,a.workspace_receipt_stage_started,Instant::now()));}
#[test]fn pending_semantics_get_a_recovery_redraw_control(){let mut a=app(WorkspaceReceipt::Appearance,false,false);let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert_eq!(a.requested_redraws.get(),1);assert!(a.receipt_error.is_none());}
#[test]fn pending_semantics_expiry_is_an_error_control(){let mut a=app(WorkspaceReceipt::Appearance,false,true);let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert!(a.receipt_error.as_deref().unwrap().contains("appearance workspace receipt timed out"));assert_eq!(e.exits.get(),1);assert_eq!(a.requested_redraws.get(),0);}
#[test]fn completed_mixed_semantics_also_disable_capture_recovery(){let mut a=app(WorkspaceReceipt::Mixed,true,true);let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert!(a.receipt_error.is_none());assert_eq!(a.requested_redraws.get(),0);assert_eq!(e.exits.get(),0);}
#[test]fn first_presentation_only_startup_retry_control(){let now=Instant::now();assert!(primary_startup_retry(0,now,now));assert!(!primary_startup_retry(1,now,now));}
'''
rs=HERE/'pelt_receipt_recovery_probe.rs';rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-pelt-receipt-recovery.exe');pdb=exe.with_suffix('.pdb');assert exe.parent.is_dir() and not exe.exists() and not pdb.exists()
command=['rustc','--edition','2024','--test','-C','debuginfo=0',str(rs),'-o',str(exe)]
build=subprocess.run(command,cwd=ROOT,text=True,capture_output=True)
(HERE/'pelt_receipt_recovery_compile.txt').write_text(build.stdout+build.stderr,encoding='utf-8');assert build.returncode==0,build.stderr
run=subprocess.run([str(exe),'--test-threads=1','--nocapture'],cwd=ROOT,text=True,capture_output=True)
(HERE/'pelt_receipt_recovery_output.txt').write_text(run.stdout+run.stderr,encoding='utf-8');assert run.returncode==0 and '5 passed' in run.stdout,run.stdout+run.stderr
removed=[]
for out in [exe,pdb]:
    if not out.exists():continue
    assert out.resolve().parent==Path('C:/t/cargo-targets/mere').resolve();raw=out.read_bytes()
    assert (out.suffix=='.exe' and raw.startswith(b'MZ')) or (out.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(out),'bytes':len(raw),'sha256':sha(out)});out.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'compile_command':command,'compile_exit':build.returncode,'exit':run.returncode,'passed':5,
 'source_sha256':{x.relative_to(ROOT).as_posix():sha(x) for x in [parent,routing]},'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{x.name:sha(x) for x in [Path(__file__),rs,HERE/'pelt_receipt_recovery_compile.txt',HERE/'pelt_receipt_recovery_output.txt']},
 'qualification':'Complete unchanged timeout method, complete failed-acquire let-else/recovery statement, startup predicate and receipt enum/id extracted from current source. Timeout method visibility widened only. Deterministic unavailable-acquire, native-statistics, event-loop and tearout-timeout doubles. Proves missing retry/deadline after semantic completion, not an observed whole-app/GPU stall. No whole desktop compilation or original desktop tests, native window, GPU, physical input/AT or external state mutation.'}
with (HERE/'pelt_receipt_recovery_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':5,'removed_outputs':len(removed),'qualification':record['qualification']},indent=2))
