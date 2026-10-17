"""Exact workspace control/lowering bodies over real neutral/AccessKit types.

Native event loop, namespace/shell fixture and receipt drivers are doubles.
No whole desktop build or physical accessibility acceptance is implied.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source=ROOT/'ports/pelt/desktop/workspace_viewer.rs';ax=source.with_suffix('')/'accessibility.rs'
parent=source.read_text(encoding='utf-8-sig');a11y=ax.read_text(encoding='utf-8-sig');pieces={}
def cut(text,name,start,end):
    a=text.index(start);b=text.index(end,a);pieces[name]=text[a:b];return pieces[name]
types=cut(a11y,'neutral_child_types','/// A Pelt-owned action target','/// Per-window platform bridge')
types=types.replace('genet_scripted_dom::NodeId','usize') # shell DOM double only
append=cut(a11y,'append_document_child','fn append_document_child(', '/// Build a composite tree for an already-laid-out tearout.')
role=cut(a11y,'role_and_actions','    fn accesskit_role(', '    /// Flatten every supported child projection')
primary=cut(a11y,'primary_projection','    pub(in crate::workspace_viewer) fn workspace_accessibility_projection(', '    /// Build the currently retained composite tree without a window.')
primary=primary.replace('pub(in crate::workspace_viewer)','pub')
validation=cut(parent,'receipt_validation','    if config.workspace_receipt.is_some()','    let tree = tree_from_urls(&config.urls);')
drivers=cut(parent,'receipt_driver_guards','        if self.config.workspace_receipt.is_some() && self.redraws > 0 && !self.receipt_complete {','\n        self.refresh_chrome();\n        let (logical_width, logical_height)')
rust=r'''
#![allow(dead_code,unused_imports)]
mod lowering {
use std::collections::{HashMap,HashSet};
use accesskit::{Action,Affine,HasPopup,Live,Node as AccessNode,NodeId as AccessNodeId,Orientation,Rect as AccessRect,Role,Toggled,TreeUpdate};
use inker::{A11yCapability,DocumentA11yAction,DocumentA11yNodeId,DocumentA11yProjection,DocumentA11yRole,DocumentA11yNode,DocumentA11yState,DocumentA11ySupport};
use pelt_core::{PeltSessionIdentity,WorkspaceRect};
use workbench::TileId;
'''+types+r'''
struct WorkspaceAccessibility {ids:HashMap<(TileId,PeltSessionIdentity,AccessNodeId),AccessNodeId>}
impl WorkspaceAccessibility {
fn new()->Self{Self{ids:HashMap::new()}}
fn child_global_id(&mut self,t:TileId,s:PeltSessionIdentity,n:AccessNodeId,_:&HashSet<AccessNodeId>)->AccessNodeId{let next=AccessNodeId((1<<63)+self.ids.len() as u64);*self.ids.entry((t,s,n)).or_insert(next)}
fn retain_document_namespaces(&mut self,_:&HashMap<TileId,DocumentA11ySession>){}
fn clear_stale_document_focus(&mut self,_:&HashMap<AccessNodeId,WorkspaceA11yActionTarget>){}
fn child_id_is_reserved(&self,n:AccessNodeId)->bool{n.0 >= (1<<63)}
fn frisket_focus(&self)->Option<&u64>{None}
}
struct FrisketA11yProjection{tree:TreeUpdate,root:AccessNodeId,nodes:HashMap<AccessNodeId,usize>,content_nodes:HashMap<TileId,AccessNodeId>}
struct FrisketFixture;
impl FrisketFixture {fn accessibility_projection(&self,_:Option<&u64>)->Option<FrisketA11yProjection>{Some(shell())}}
fn shell()->FrisketA11yProjection {
let root=AccessNodeId(1);let aperture=AccessNodeId(2);let mut top=AccessNode::new(Role::Window);top.set_children([aperture]);
FrisketA11yProjection{tree:TreeUpdate{tree:Some(accesskit::Tree::new(root)),nodes:vec![(root,top),(aperture,AccessNode::new(Role::Region))],tree_id:accesskit::TreeId::ROOT,focus:root},root,nodes:HashMap::new(),content_nodes:HashMap::from([(TileId(1),aperture)])}
}
struct WorkspaceApp{accessibility:WorkspaceAccessibility,frisket:FrisketFixture}
impl WorkspaceApp {
fn document_accessibility_children(&self,_:&FrisketA11yProjection)->(HashMap<TileId,DocumentA11ySession>,Vec<DocumentA11yChild>){(HashMap::new(),vec![child()])}
'''+role+primary+'\n}\n'+append+r'''
fn child()->DocumentA11yChild {
DocumentA11yChild {tile:TileId(1),session_identity:PeltSessionIdentity{instance_id:17,generation:1},aperture:AccessNodeId(2),content_rect:WorkspaceRect::new(10.0,20.0,400.0,300.0),placement_generation:1,
projection:DocumentA11yProjection::new(1,DocumentA11ySupport::new(A11yCapability::Partial,["fixture"]).unwrap(),DocumentA11yNodeId::new(1),vec![DocumentA11yNode {
id:DocumentA11yNodeId::new(1),parent:None,children:vec![],role:DocumentA11yRole::TextField,name:Some("Account".into()),description:Some("Use your public account name".into()),value:Some("orchard".into()),numeric_value:None,numeric_minimum:None,numeric_maximum:None,bounds:None,state:DocumentA11yState::default(),actions:vec![DocumentA11yAction::SetValue],}])}
}
fn assert_dropped(tree:&TreeUpdate){let (_,node)=tree.nodes.iter().find(|(_,n)|n.label()==Some("Account")).unwrap();assert_eq!(node.role(),Role::TextInput);assert_eq!(node.value(),Some("orchard"));assert!(node.supports_action(Action::SetValue));assert_eq!(node.description(),None);}
#[test]fn secondary_helper_drops_real_neutral_description(){let c=child();assert_eq!(c.projection.nodes()[0].description.as_deref(),Some("Use your public account name"));let mut s=shell();let mut a=WorkspaceAccessibility::new();let mut actions=HashMap::new();let ids=s.tree.nodes.iter().map(|(id,_)|*id).collect();append_document_child(&mut a,&mut s.tree,&mut actions,&ids,c);assert_dropped(&s.tree);}
#[test]fn copied_primary_lowering_drops_real_neutral_description(){let mut a=WorkspaceApp{accessibility:WorkspaceAccessibility::new(),frisket:FrisketFixture};assert_dropped(&a.workspace_accessibility_projection().unwrap().tree);}
#[test]fn accesskit_can_retain_supplemental_description_control(){let mut n=AccessNode::new(Role::TextInput);n.set_description("Use your public account name");assert_eq!(n.description(),Some("Use your public account name"));}
}
mod receipt_guards {
#[derive(Clone,Copy,PartialEq)]enum WorkspaceReceipt{Mixed,Other}
struct Config{workspace_receipt:Option<WorkspaceReceipt>,interaction_receipt:bool,capability_receipt:bool,tearout_receipt:bool,tearout_cancellation_receipt:bool,urls:Vec<String>,workspace_size_matrix:Option<Vec<(u32,u32)>>}
fn validate(config:&Config)->Result<(),String>{
'''+validation+r'''
Ok(())}
struct ActiveEventLoop;impl ActiveEventLoop{fn exit(&self){}}
struct WorkspaceApp{config:Config,redraws:u32,receipt_complete:bool,receipt_error:Option<String>,pending_workspace_assertion:Option<String>,cap_ready:bool,interaction_completes:bool,cap_calls:usize,interaction_calls:usize,tearout_calls:usize,cancel_calls:usize}
impl WorkspaceApp {
fn capability_receipt_ready(&self)->bool{self.cap_ready}
fn validate_capability_receipt(&mut self)->Result<(),String>{self.cap_calls+=1;Ok(())}
fn drive_receipt_step(&mut self)->Result<(),String>{self.interaction_calls+=1;if self.interaction_completes {self.receipt_complete=true;}Ok(())}
fn drive_workspace_receipt(&mut self)->Result<Option<String>,String>{Ok(None)}
fn drive_tearout_receipt(&mut self,_:&ActiveEventLoop)->Result<bool,String>{self.tearout_calls+=1;Ok(false)}
fn drive_tearout_cancellation_receipt(&mut self,_:&ActiveEventLoop)->Result<bool,String>{self.cancel_calls+=1;Ok(false)}
fn run_drivers(&mut self,event_loop:&ActiveEventLoop){
'''+drivers+r'''
}}
fn app()->WorkspaceApp {WorkspaceApp{config:Config{workspace_receipt:None,interaction_receipt:false,capability_receipt:false,tearout_receipt:false,tearout_cancellation_receipt:false,urls:vec!["a".into(),"b".into(),"c".into(),"d".into()],workspace_size_matrix:None},redraws:1,receipt_complete:false,receipt_error:None,pending_workspace_assertion:None,cap_ready:false,interaction_completes:false,cap_calls:0,interaction_calls:0,tearout_calls:0,cancel_calls:0}}
#[test]fn admitted_capability_completion_skips_requested_interaction(){let mut a=app();a.config.interaction_receipt=true;a.config.capability_receipt=true;a.cap_ready=true;assert!(validate(&a.config).is_ok());a.run_drivers(&ActiveEventLoop);assert!(a.receipt_complete);assert_eq!(a.cap_calls,1);assert_eq!(a.interaction_calls,0);}
#[test]fn capability_pending_control_runs_interaction(){let mut a=app();a.config.interaction_receipt=true;a.config.capability_receipt=true;assert!(validate(&a.config).is_ok());a.run_drivers(&ActiveEventLoop);assert!(!a.receipt_complete);assert_eq!(a.interaction_calls,1);}
#[test]fn admitted_interaction_completion_skips_requested_tearout(){let mut a=app();a.config.interaction_receipt=true;a.config.tearout_receipt=true;a.interaction_completes=true;assert!(validate(&a.config).is_ok());a.run_drivers(&ActiveEventLoop);assert!(a.receipt_complete);assert_eq!(a.interaction_calls,1);assert_eq!(a.tearout_calls,0);}
#[test]fn named_and_dual_tearout_validation_controls_refuse(){let mut a=app();a.config.workspace_receipt=Some(WorkspaceReceipt::Other);a.config.interaction_receipt=true;assert!(validate(&a.config).is_err());a.config.workspace_receipt=None;a.config.tearout_receipt=true;a.config.tearout_cancellation_receipt=true;assert!(validate(&a.config).is_err());}
}
'''
folder=HERE/'cpu-pelt-workspace-runtime';folder.mkdir(exist_ok=True)
manifest=folder/'Cargo.toml';assert not manifest.exists()
manifest.write_text('''[workspace]
[package]
name = "authority-success-pelt-workspace"
version = "0.0.0"
edition = "2024"
[dependencies]
accesskit = "=0.24.1"
inker = { path = "../../../../crates/inker/inker" }
workbench = { path = "../../../../crates/cambium/workbench" }
pelt-core = { package = "authority-success-pelt-core", path = "../cpu-pelt-core-runtime" }
[lib]
path = "../pelt_workspace_owner_probe.rs"
''',encoding='utf-8')
rs=HERE/'pelt_workspace_owner_probe.rs';rs.write_text(rust,encoding='utf-8')
lock=folder/'Cargo.lock';assert not lock.exists();lock.write_bytes((ROOT/'Cargo.lock').read_bytes())
target=Path('C:/t/cargo-targets/mere').resolve()
command=['cargo','test','--offline','--manifest-path',str(manifest),'--config','build.build-dir="C:/t/cargo-targets/mere/build"','--lib','-j','2','--','--test-threads=1','--nocapture']
run=subprocess.run(command,cwd=ROOT,env=dict(os.environ,CARGO_TARGET_DIR=str(target)),capture_output=True,text=True,timeout=600)
(HERE/'pelt_workspace_owner_probe_output.log').write_text(run.stdout+run.stderr,encoding='utf-8')
print(run.stdout);print(run.stderr[-6000:])
assert run.returncode==0 and '7 passed' in run.stdout
removed=[]
for p in (target/'build/debug/deps').glob('authority_success_pelt_workspace-*'):
    if p.suffix not in ['.exe','.pdb']:continue
    exe=p if p.suffix=='.exe' else p.with_suffix('.exe')
    if exe.name not in run.stderr:continue
    raw=p.read_bytes();assert (p.suffix=='.exe' and raw.startswith(b'MZ')) or (p.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(raw),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':run.returncode,'passed':7,'original_tests':0,
 'source_sha256':{p.relative_to(ROOT).as_posix():sha(p) for p in [source,ax]},'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},
 'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in [Path(__file__),rs,manifest,lock,HERE/'pelt_workspace_owner_probe_output.log']},
 'qualification':'Seven source-extracted CPU cases. Complete unchanged primary child-lowering method, secondary append helper and role/action conversion over actual local Inker/Pelt core geometry/session-identity and exact lock-seeded neutral contract plus AccessKit0.24.1. Deterministic shell/namespace fixture. Exact config validation and render-driver guard prefix with deterministic driver/native event-loop doubles. Type-only shell NodeId and method visibility adapted; no executable lowering/guard rewrite. No whole desktop compilation, original desktop tests, real engine DOM, native window/GPU, display/input or physical AT run.'}
with (HERE/'pelt_workspace_owner_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':run.returncode,'passed':7,'removed_outputs':len(removed)}))
