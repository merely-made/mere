"""Exact early-return prefix, with CPU metadata/output doubles, retained Burn only."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source='support/patches/burn-cubecl/src/kernel/matmul/tune/base.rs'
s=(ROOT/source).read_text(encoding='utf-8-sig')
prefix=s[s.index('pub fn matmul_autotune('):s.index('\n    let client = lhs.client.clone();',s.index('pub fn matmul_autotune('))]
code=r'''#![allow(dead_code,private_interfaces)]
type DType=();
#[derive(Clone)] struct Meta(Vec<usize>);
impl Meta { fn shape(&self)->&[usize]{&self.0} }
#[derive(Clone)] struct CubeTensor {meta:Meta, values:Vec<i32>, dispatched:bool}
fn t(shape:&[usize],fill:i32)->CubeTensor{CubeTensor{meta:Meta(shape.to_vec()),values:vec![fill;shape.iter().product()],dispatched:false}}
// Allocation double deliberately exposes the fact that the extracted prefix never fills it.
fn init_matmul_output(lhs:&CubeTensor,rhs:&CubeTensor,_dtype:DType)->CubeTensor{t(&[lhs.meta.shape()[0],rhs.meta.shape()[1]],23)}
'''+prefix+r'''
    CubeTensor { dispatched:true, ..output }
}
#[cfg(test)] mod cases {
 use super::*;
 #[test] fn zero_inner_dimension_leaves_supplied_nonempty_output_unchanged(){
  let result=matmul_autotune(t(&[2,0],0),t(&[0,3],0),Some(t(&[2,3],17)),());
  let empty_dot:i32=std::iter::empty::<i32>().sum();
  assert_eq!(empty_dot,0);assert_eq!(result.values,vec![17;6]);assert!(!result.dispatched);
  assert_ne!(result.values,vec![empty_dot;6]);
 }
 #[test] fn zero_inner_dimension_returns_allocator_double_without_initializing(){
  let result=matmul_autotune(t(&[2,0],0),t(&[0,3],0),None,());
  assert_eq!(result.values,vec![23;6]);assert!(!result.dispatched);
 }
 #[test] fn zero_rows_control_has_empty_output(){
  let result=matmul_autotune(t(&[0,2],0),t(&[2,3],0),Some(t(&[0,3],17)),());
  assert!(result.values.is_empty());assert!(!result.dispatched);
 }
 #[test] fn zero_columns_control_has_empty_output(){
  let result=matmul_autotune(t(&[2,3],0),t(&[3,0],0),Some(t(&[2,0],17)),());
  assert!(result.values.is_empty());assert!(!result.dispatched);
 }
 #[test] fn nonzero_dimensions_reach_dispatch_marker(){
  let result=matmul_autotune(t(&[2,1],0),t(&[1,3],0),Some(t(&[2,3],17)),());
  assert!(result.dispatched);
 }
}
'''
rs=HERE/'retained_burn_zero_matmul_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-retained-burn-zero-matmul.exe');pdb=exe.with_suffix('.pdb')
assert exe.parent.is_dir() and not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'retained_burn_zero_matmul_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'retained_burn_zero_matmul_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '5 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
paths=[source,'support/patches/burn-cubecl/src/kernel/matmul/utils.rs','support/patches/burn-cubecl/src/ops/numeric.rs']
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':5,'source_sha256':{p:sha(ROOT/p) for p in paths},'extracted_prefix_sha256':hashlib.sha256(prefix.encode()).hexdigest(),'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_retained_burn_zero_matmul_probe.py',rs.name,'retained_burn_zero_matmul_compile.txt','retained_burn_zero_matmul_run.txt']},
 'qualification':'Exact retained matmul_autotune initialization/early-return prefix, compiled with CPU metadata/tensor/output allocator doubles. Supplied-output case proves no zeroing or dispatch before return for nonempty MxN and K0. Allocator sentinel demonstrates absent initialization in this prefix, not actual uninitialized GPU bytes. M0/N0 and nonempty controls included. No CubeCL/backend/shader compilation or registry backend failure claimed; retained copy is root-unselected. Exact owned exe/PDB removed.'}
with (HERE/'retained_burn_zero_matmul_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':5,'removed_outputs':len(removed)}))
