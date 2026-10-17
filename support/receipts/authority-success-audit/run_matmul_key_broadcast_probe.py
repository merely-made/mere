"""Exact key extent fragment and full broadcast assertion helper with explicit outcomes."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
paths=['support/patches/cubek-matmul/src/tune_key.rs','support/patches/cubek-matmul/tests/harness/broadcast.rs']
key=(ROOT/paths[0]).read_text(encoding='utf-8-sig')
extent='        let ndims = lhs_logical.len();'+key.split('        let ndims = lhs_logical.len();',1)[1].split('        let lhs_storage',1)[0]
source=(ROOT/paths[1]).read_text(encoding='utf-8-sig')
passed='pub(crate) fn passed('+source.split('pub(crate) fn passed(',1)[1].split('/// A strategy',1)[0]
helper='pub(crate) fn assert_batch_broadcast('+source.split('pub(crate) fn assert_batch_broadcast(',1)[1]
code=r'''#![allow(dead_code)]
use std::{cell::Cell,rc::Rc};
#[derive(Clone,Debug)]enum ValidationResult{Pass,Fail,Skipped,Error}
#[derive(Clone,Debug)]enum TestOutcome{Validated(ValidationResult),CompileError}
#[derive(Clone,Debug)]struct Strategy{baseline:TestOutcome,broadcast:TestOutcome,calls:Rc<Cell<usize>>}
impl std::fmt::Display for Strategy{fn fmt(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{write!(f,"stub strategy")}}
enum MatrixLayout{ColMajor,RowMajor}
#[derive(Clone,Copy)]enum Broadcast{BatchLhs,BatchRhs}
impl Broadcast{fn label(self)->&'static str{"model axis"}}
mod cubecl{pub fn test_device()->Device{Device}pub struct Device;impl Device{pub fn client(self){}}}
fn make_problem(_:Broadcast,zero_stride:bool,_:MatrixLayout,_:MatrixLayout)->bool{zero_stride}
fn run_with_strides(_:(),broadcast:bool,strategy:Strategy)->TestOutcome{if broadcast{strategy.calls.set(strategy.calls.get()+1);strategy.broadcast}else{strategy.baseline}}
'''
code+='fn dimensions(lhs_logical:&[usize],rhs_logical:&[usize])->(usize,usize,usize){\n'+extent+'(m,n,k)}\n'+passed+helper
code+=r'''
#[test]fn key_lhs_rank3_rhs_rank2_panics(){assert!(std::panic::catch_unwind(||dimensions(&[2,1,32],&[32,64])).is_err());}
#[test]fn key_lhs_rank2_rhs_rank3_misreads_n(){assert_eq!(dimensions(&[1,32],&[2,32,64]),(1,32,32));assert_ne!(dimensions(&[1,32],&[2,32,64]),(1,64,32));}
#[test]fn equal_rank2_key_control(){assert_eq!(dimensions(&[1,32],&[32,64]),(1,64,32));}
#[test]fn equal_rank3_key_control(){assert_eq!(dimensions(&[2,1,32],&[2,32,64]),(1,64,32));}
fn strategy(baseline:TestOutcome,broadcast:TestOutcome)->(Strategy,Rc<Cell<usize>>){let calls=Rc::new(Cell::new(0));(Strategy{baseline,broadcast,calls:calls.clone()},calls)}
#[test]fn wrong_numeric_baseline_silently_skips_broadcast(){let(s,c)=strategy(TestOutcome::Validated(ValidationResult::Fail),TestOutcome::Validated(ValidationResult::Fail));assert_batch_broadcast(s);assert_eq!(c.get(),0);}
#[test]fn baseline_compile_error_also_skips(){let(s,c)=strategy(TestOutcome::CompileError,TestOutcome::Validated(ValidationResult::Fail));assert_batch_broadcast(s);assert_eq!(c.get(),0);}
#[test]fn passing_controls_execute_both_broadcast_axes(){let(s,c)=strategy(TestOutcome::Validated(ValidationResult::Pass),TestOutcome::Validated(ValidationResult::Pass));assert_batch_broadcast(s);assert_eq!(c.get(),2);}
#[test]fn passing_baseline_wrong_broadcast_still_panics(){let(s,c)=strategy(TestOutcome::Validated(ValidationResult::Pass),TestOutcome::Validated(ValidationResult::Fail));assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||assert_batch_broadcast(s))).is_err());assert_eq!(c.get(),1);}
'''
rs=HERE/'matmul_key_broadcast_probe.rs'
with rs.open('x',encoding='utf-8') as f:f.write(code)
exe=Path('C:/t/cargo-targets/mere/audit-matmul-key-broadcast.exe');pdb=exe.with_suffix('.pdb')
assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_key_broadcast_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_key_broadcast_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '8 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':8,'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in {'key_extent':extent,'passed':passed,'assert_batch_broadcast':helper}.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_key_broadcast_probe.py',rs.name,'matmul_key_broadcast_compile.txt','matmul_key_broadcast_run.txt']},
 'qualification':'Four exact key extent-fragment cases over already logical shape slices; no actual Metadata folding, generated key/anchor/serde/tuner or frontend normalization executed. Four full exact broadcast assertion helper cases with explicit Client/problem/strategy/outcome doubles: wrong numeric baseline and compile error skip all broadcasts; passing controls run both, passing baseline + wrong broadcast panics. No full crate, real backend/device or production invocation proof. Owned exe/PDB removed.'}
with (HERE/'matmul_key_broadcast_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
