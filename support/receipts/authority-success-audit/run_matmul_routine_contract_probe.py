"""Bounded exact source-fragment CPU witnesses; no device or whole-launcher claim."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
base = 'support/patches/cubek-matmul/src/multi_level/'
paths = [base+p for p in ['routines/gemm/launch.rs', 'routines/gemv_unit_perpendicular/launch.rs',
    'routines/batch/gemv_innerproduct.rs', 'definition/blueprint.rs',
    'components/batch/partitioned_matmul/setup.rs']]
sources = [(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
gemm = sources[0].split('    let rank = rhs.shape().len();', 1)[1].split('    let plane_size', 1)[0]
gemm = '    let rank = rhs.shape().len();'+gemm
gemv = sources[1].split('    let m = lhs.shape()', 1)[1].split('    if m != 1', 1)[0]
gemv = '    let rank = rhs.shape().len();\n    let m = lhs.shape()'+gemv
builder = sources[3].split('        let k_group = self.stage_buffering;', 1)[1].split('        BatchMatmulBlueprint {', 1)[0]
builder = '        let k_group = self.stage_buffering;'+builder
validator = sources[4].split('        let k_group = blueprint.tiling_scheme', 1)[1].split('        GMM::validate_blueprint', 1)[0]
validator = '        let k_group = blueprint.tiling_scheme'+validator
assert 'stage_buffering: 1,' in sources[3]
helper = sources[2].split('fn infer_blueprint_vecmat(', 1)[1]
assert '.stage_buffering(' not in helper
assert sources[2].count('infer_blueprint_vecmat(') == 3
code = r'''#![allow(dead_code)]
struct InputBinding(Vec<usize>);impl InputBinding{fn shape(&self)->&[usize]{&self.0}}
#[derive(Clone,Copy)]struct Scheme{m:u32,n:u32,k:u32}
impl Scheme{fn elements_per_stage_along_m(&self)->u32{self.m}fn elements_per_stage_along_n(&self)->u32{self.n}fn elements_per_stage_along_k(&self)->u32{self.k}}
struct Builder{m:u32,n:u32,k:u32,stage_buffering:u32,tiling_scheme:Scheme}
struct Blueprint{check_k_bounds:bool,tiling_scheme:Scheme}
#[derive(Debug)]enum MatmulSetupError{InvalidConfig(Box<String>)}
struct Problem{k:usize}
struct GMM;struct Stages;
impl GMM{fn num_stages()->Stages{Stages}}
impl Stages{fn stage_buffering(&self)->u32{2}}
'''
code += 'fn gemm(lhs:InputBinding,rhs:InputBinding)->(usize,usize,usize){\n'+gemm+'(m,n,k)}\n'
code += 'fn gemv(lhs:InputBinding,rhs:InputBinding)->(usize,usize,usize){\n'+gemv+'(m,n,k)}\n'
code += 'impl Builder{fn bounds(self)->(bool,bool,bool){\n'+builder+'(check_m_bounds,check_n_bounds,check_k_bounds)}}\n'
code += 'fn group_admission(blueprint:&Blueprint,problem:&Problem)->Result<(),MatmulSetupError>{\n'+validator+'Ok(())}\n'
code += r'''
fn operands(a:&[usize],b:&[usize])->(InputBinding,InputBinding){(InputBinding(a.to_vec()),InputBinding(b.to_vec()))}
type LaunchShape=fn(InputBinding,InputBinding)->(usize,usize,usize);
fn short_lhs_panics(f:LaunchShape){let(a,b)=operands(&[1,32],&[2,32,64]);assert!(std::panic::catch_unwind(||f(a,b)).is_err());}
fn long_lhs_misreads(f:LaunchShape){let(a,b)=operands(&[2,1,32],&[32,64]);let got=f(a,b);assert_eq!(got,(2,64,1));assert_ne!(got,(1,64,32));}
fn equal_rank_control(f:LaunchShape){let(a,b)=operands(&[2,1,32],&[2,32,64]);assert_eq!(f(a,b),(1,64,32));}
#[test]fn gemm_rank2_lhs_rank3_rhs_panics(){short_lhs_panics(gemm)}
#[test]fn gemv_rank2_lhs_rank3_rhs_panics(){short_lhs_panics(gemv)}
#[test]fn gemm_rank3_lhs_rank2_rhs_misreads_dimensions(){long_lhs_misreads(gemm)}
#[test]fn gemv_rank3_lhs_rank2_rhs_misreads_dimensions(){long_lhs_misreads(gemv)}
#[test]fn gemm_equal_rank_control(){equal_rank_control(gemm)}
#[test]fn gemv_equal_rank_control(){equal_rank_control(gemv)}
fn inferred(k:u32,buffering:u32)->Blueprint{let scheme=Scheme{m:1,n:1,k:32};let(_,_,check_k_bounds)=Builder{m:1,n:64,k,stage_buffering:buffering,tiling_scheme:scheme}.bounds();Blueprint{check_k_bounds,tiling_scheme:scheme}}
#[test]fn double_vecmat_single_stage_bounds_refuse_odd_stage_count(){let bp=inferred(96,1);assert!(!bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:96}).is_err());}
#[test]fn double_vecmat_even_stage_control(){let bp=inferred(128,1);assert!(!bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:128}).is_ok());}
#[test]fn explicit_double_stage_bounds_admit_tail_control(){let bp=inferred(96,2);assert!(bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:96}).is_ok());}
#[test]fn single_stage_nonmultiple_gets_bounds_control(){let bp=inferred(97,1);assert!(bp.check_k_bounds);assert!(group_admission(&bp,&Problem{k:97}).is_ok());}
'''
rs = HERE/'matmul_routine_contract_probe.rs'
with rs.open('x', encoding='utf-8') as f: f.write(code)
exe = Path('C:/t/cargo-targets/mere/audit-matmul-routine-contract.exe')
pdb = exe.with_suffix('.pdb')
assert not exe.exists() and not pdb.exists()
command = ['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r = subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_routine_contract_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8')
assert r.returncode == 0,r.stderr
r = subprocess.run([str(exe),'--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_routine_contract_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8')
print(r.stdout+r.stderr)
assert r.returncode == 0 and '10 passed; 0 failed' in r.stdout
removed = []
for p in [exe,pdb]:
    if not p.exists(): continue
    assert p.resolve().parent == exe.parent.resolve()
    b = p.read_bytes()
    assert (p.suffix == '.exe' and b.startswith(b'MZ')) or (p.suffix == '.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)})
    p.unlink()
record = {'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':10,
    'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,
    'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in {'gemm':gemm,'gemv':gemv,'bounds':builder,'group_admission':validator}.items()},
    'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_routine_contract_probe.py',rs.name,'matmul_routine_contract_compile.txt','matmul_routine_contract_run.txt']},
    'qualification':'Six exact launcher extent-fragment CPU tests use a shape-only InputBinding double, not the complete launcher or its normalization. Two rank-broadcast inputs panic; reverse rank inputs silently read incorrect dimensions; equal-rank controls pass. Four exact builder-bounds/first group-validator fragments use scalar DTO doubles and a double-stage GMM. infer_blueprint_vecmat is source-inspected, not executed: both callers share helper without stage_buffering and exact real builder default is1. Witness proves selected host fragments and missing helper setting, not full validator admission, actual algorithm/GPU result, CubeDSL or complete crate build. Owned executable/PDB removed.'}
with (HERE/'matmul_routine_contract_probe_receipt.json').open('x',encoding='utf-8') as f: json.dump(record,f,indent=2);f.write('\n')
