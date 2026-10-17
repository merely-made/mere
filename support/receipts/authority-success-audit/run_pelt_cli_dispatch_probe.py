"""Run Pelt's unchanged argument/early-dispatch prefix with deterministic host doubles."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
viewer=ROOT/'ports/pelt/viewer.rs';text=viewer.read_text(encoding='utf-8-sig');pieces={}
def cut(s,key,start,end):
    a=s.index(start);b=s.index(end,a);pieces[key]=s[a:b];return pieces[key]
selected=cut(text,'selected_engine','#[derive(Clone, Copy, Debug, Eq, PartialEq)]\nenum SelectedEngine', 'pub(crate) fn main()')
prefix=cut(text,'cli_parser_and_early_dispatch','pub(crate) fn main()', '    if with_tiles {\n        if workspace_receipt.is_some()')
prefix=prefix.replace('pub(crate) fn main()', 'fn source_prefix()',1)+'\n    println!("AUDIT owner dispatch reached");\n}\n'
parsers=cut(text,'size_receipt_parser_helpers','fn parse_size(value:', '/// Dispatch a smolweb URL to the owned headed document viewer.')
tile=cut(text,'tile_engine_parser','fn parse_tile_engine(', 'fn capability_fixture_urls(')
workspace=cut(text,'workspace_receipt_parser','fn parse_workspace_receipt(', '#[cfg(test)]\nmod workspace_receipt_tests')
engine=cut(text,'engine_parser','fn parse_engine(', '\nfn print_help()')
override=cut(text,'workspace_profile_engine','            let profile_engine = match selected_engine {', '            if let Some(engine) = profile_engine {')
static=(ROOT/'ports/pelt/desktop/static_viewer.rs').read_text(encoding='utf-8-sig')
products=cut(static,'product_receipt_types','pub enum ProductReceipt {', 'pub(crate) fn validate_receipt_profile(')
parent=(ROOT/'ports/pelt/desktop/workspace_viewer.rs').read_text(encoding='utf-8-sig')
receipts=cut(parent,'workspace_receipt_enum','pub enum WorkspaceReceipt {','\nimpl WorkspaceReceipt {')
receipt_id=cut(parent,'workspace_receipt_id','    pub fn id(self)','\n    fn default_size(')
code=r'''
#![allow(dead_code,unused_variables,unused_mut,unused_assignments)]
use std::env;
const VERSION:&str="audit dispatch fixture";
mod genet_host_api {
#[derive(Clone,Copy,Debug)]pub enum EngineProfile{Livery,Scripted}
impl std::str::FromStr for EngineProfile{type Err=String;fn from_str(s:&str)->Result<Self,String>{match s.to_ascii_lowercase().as_str(){"livery"=>Ok(Self::Livery),"scripted"=>Ok(Self::Scripted),_=>Err("unknown fixture engine".into())}}}
pub struct DeferredShellEngine;impl DeferredShellEngine{pub fn new(_:EngineProfile)->Self{Self}}
pub struct Capabilities{pub javascript:bool,pub webdriver:bool,pub devtools:bool,pub webgpu:bool,pub webxr:bool}
pub trait ShellEngine{fn capabilities(&self)->Capabilities;}
impl ShellEngine for DeferredShellEngine{fn capabilities(&self)->Capabilities{Capabilities{javascript:false,webdriver:false,devtools:false,webgpu:false,webxr:false}}}
}
use genet_host_api::{DeferredShellEngine,EngineProfile,ShellEngine};
mod inker{pub mod routing{pub const ENGINE_GENET_READER:&str="genet.reader";pub const ENGINE_GENET_SCRIPTED:&str="genet.scripted";pub const ENGINE_GENET_SCRIPTED_NOVA:&str="genet.scripted.nova";}}
mod pelt_desktop {
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
'''+products+'\n#[derive(Clone,Copy,Debug,Eq,PartialEq)]\n'+receipts+'\nimpl WorkspaceReceipt {\n'+receipt_id+'\n}\n}\n'+selected+prefix+parsers+tile+workspace+engine+r'''
fn print_help(){println!("AUDIT help");}
fn run_optional_netrender_smoke(){println!("AUDIT netrender called");}
fn run_optional_webgl_wgpu_smoke(){println!("AUDIT webgl called");}
fn workspace_override(js_engine:&str)->Option<&'static str>{let selected_engine=SelectedEngine::Scripted;
'''+override+r'''
profile_engine}
fn main(){if let Ok(js)=env::var("AUDIT_WORKSPACE_JS"){println!("AUDIT workspace engine={:?}",workspace_override(&js));return;}source_prefix();}
'''
rs=HERE/'pelt_cli_dispatch_probe.rs';rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-pelt-cli-dispatch.exe');pdb=exe.with_suffix('.pdb');assert exe.parent.is_dir() and not exe.exists() and not pdb.exists()
command=['rustc','--edition','2024','--cfg','feature="scripted"','--cfg','feature="scripted-nova"','--cfg','feature="reader"',str(rs),'-C','debuginfo=0','-o',str(exe)]
environment=dict(os.environ,CARGO_MANIFEST_DIR=str(ROOT/'ports/pelt'))
build=subprocess.run(command,cwd=ROOT,env=environment,text=True,capture_output=True)
(HERE/'pelt_cli_dispatch_compile.txt').write_text(build.stdout+build.stderr,encoding='utf-8');assert build.returncode==0,build.stderr
artifact=HERE/'pelt_cli_dispatch_must_not_exist.png';assert not artifact.exists()
cases=[]
def run(name,args,expected_exit,required,forbidden=(),js=None):
    env=dict(environment)
    if js is not None:env['AUDIT_WORKSPACE_JS']=js
    result=subprocess.run([str(exe),*args],cwd=ROOT,env=env,text=True,capture_output=True)
    output=result.stdout+result.stderr
    assert result.returncode==expected_exit,(name,result.returncode,output)
    assert all(token in output for token in required),(name,output)
    assert not any(token in output for token in forbidden),(name,output)
    assert not artifact.exists()
    cases.append({'name':name,'args':args,'workspace_js_fixture':js,'exit':result.returncode,'stdout':result.stdout,'stderr':result.stderr,'artifact_exists':False})
run('named_product_skipped_by_smoke',['--product-receipt','article','--artifact',str(artifact),'--netrender-smoke'],0,['AUDIT netrender called'],['owner dispatch reached'])
run('named_workspace_skipped_by_smoke',['--workspace-receipt','appearance','--artifact',str(artifact),'--netrender-smoke'],0,['AUDIT netrender called'],['owner dispatch reached'])
run('workspace_missing_artifact_validation_skipped',['--workspace-receipt','mixed','--netrender-smoke'],0,['AUDIT netrender called'],['owner dispatch reached'])
run('second_requested_smoke_skipped',['--netrender-smoke','--webgl-wgpu-smoke'],0,['AUDIT netrender called'],['AUDIT webgl called','owner dispatch reached'])
run('product_required_artifact_control_refuses_before_smoke',['--product-receipt','article','--netrender-smoke'],2,['needs --artifact'],['AUDIT netrender called'])
run('named_workspace_without_smoke_reaches_later_dispatch_control',['--workspace-receipt','appearance','--artifact',str(artifact)],0,['AUDIT owner dispatch reached'],['AUDIT netrender called'])
run('workspace_unknown_js_silently_selects_boa',[],0,['Some("genet.scripted")'],js='misspelled-backend')
run('workspace_boa_control',[],0,['Some("genet.scripted")'],js='boa')
run('workspace_nova_control',[],0,['Some("genet.scripted.nova")'],js='nova')
(HERE/'pelt_cli_dispatch_cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
removed=[]
for out in [exe,pdb]:
    if not out.exists():continue
    assert out.resolve().parent==Path('C:/t/cargo-targets/mere').resolve();raw=out.read_bytes()
    assert (out.suffix=='.exe' and raw.startswith(b'MZ')) or (out.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(out),'bytes':len(raw),'sha256':sha(out)});out.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'compile_command':command,'compile_exit':build.returncode,'exit':0,'passed':len(cases),
 'source_sha256':{p.relative_to(ROOT).as_posix():sha(p) for p in [viewer,ROOT/'ports/pelt/desktop/static_viewer.rs',ROOT/'ports/pelt/desktop/workspace_viewer.rs']},'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},
 'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p.name:sha(p) for p in [Path(__file__),rs,HERE/'pelt_cli_dispatch_compile_attempt0.txt',HERE/'pelt_cli_dispatch_compile.txt',HERE/'pelt_cli_dispatch_cases.json']},
 'qualification':'Nine real child-process invocations of exact source-extracted CLI parser/early-smoke dispatch prefix, parser helpers and workspace profile-engine selection. Main function renamed and source-prefix fixture stops only after the actual smoke early-return branches. Engine-capability/engine-ID and smoke host doubles; original receipt enum/id/metadata definitions extracted. Scripted/Nova/Reader cfg enabled only to inspect branch selection; neither JS backend compiled or executed. Owner-dispatch control is reachability only, not receipt execution. No actual Pelt desktop binary, GPU, browser, native window, physical input/AT or user storage mutation; expected PNG never created.'}
with (HERE/'pelt_cli_dispatch_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':len(cases),'removed_outputs':len(removed),'cases':[c['name'] for c in cases]},indent=2))
