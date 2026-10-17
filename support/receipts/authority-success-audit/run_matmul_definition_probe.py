"""Exact selected matmul broadcast, benchmark prepare and CPU-reference bodies."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubek-matmul/src/'
paths=[base+p for p in ['definition/base.rs','eval/benchmarks/gemm/benchmark.rs','eval/benchmarks/gemm/problem.rs','eval/cpu_reference.rs']]
definition,bench,catalog,reference=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
externalbase=Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cubek-test-utils-0.3.0/src/test_tensor')
external=[externalbase/p for p in ['host_data.rs','strides.rs']]
host,strides=[p.read_text(encoding='utf-8-sig') for p in external]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
parts={'broadcast_batches':function(definition,'pub fn broadcast_batches('),'prepare':function(bench,'fn prepare('),'matmul_cpu_reference':function(reference,'pub fn matmul_cpu_reference('),'num_batches':function(definition,'pub fn num_batches('),'get_f64':function(host,'pub fn get_f64(&self, index:'),'strided_index':function(host,'fn strided_index('),'compute_strides':function(strides[strides.index('impl StridedLayout {'):],'pub fn compute_strides(')}
code=r'''#![allow(dead_code,private_interfaces)]
use std::{cmp::max,ops::Deref};
type Shape=Vec<usize>;
#[derive(Debug,Clone,PartialEq)]struct Strides(Vec<usize>);impl Strides{fn new(s:&[usize])->Self{Self(s.to_vec())}}impl From<Vec<usize>>for Strides{fn from(v:Vec<usize>)->Self{Self(v)}}impl Deref for Strides{type Target=[usize];fn deref(&self)->&[usize]{&self.0}}
enum StridedLayout{RowMajor,ColMajor,Explicit(Vec<usize>)}
impl StridedLayout{
'''+parts['compute_strides']+r'''
}
'''+parts['broadcast_batches']+r'''
#[derive(Debug)]struct HostDataVec(Vec<f64>);impl HostDataVec{fn get_f64(&self,i:usize)->f64{self.0[i]}}impl From<(Vec<f64>,u8)>for HostDataVec{fn from(v:(Vec<f64>,u8))->Self{Self(v.0)}}
#[derive(Debug)]struct HostData{data:HostDataVec,shape:Shape,strides:Strides}
impl HostData{
'''+parts['get_f64']+'\n'+parts['strided_index']+r'''
}
struct Progress;impl Progress{fn set_total(&self,_v:u64){}fn bump(&self){}}
struct GlobalDtypes{out:u8}
struct MatmulProblem{m:usize,n:usize,k:usize,lhs_batches:Shape,rhs_batches:Shape,out_batches:Shape,out_shape:Shape,global_dtypes:GlobalDtypes}
impl MatmulProblem{
'''+parts['num_batches']+r'''
}
'''+parts['matmul_cpu_reference']+r'''
#[derive(Clone,Copy)]enum MatrixLayout{RowMajor,ColMajor}
struct Metadata{shape:Vec<usize>,strides:Strides}
impl Metadata{fn rank(&self)->usize{self.shape.len()}fn strides_mut(&mut self)->&mut[usize]{&mut self.strides.0}}
struct TensorHandle{metadata:Metadata}
type Client=();
struct Device;impl Device{fn client(&self)->Client{()}}
fn make_uniform(_c:&Client,shape:[usize;3],_d:u8,_s:u64,_lo:f32,_hi:f32)->TensorHandle{let shape=shape.to_vec();let strides=StridedLayout::RowMajor.compute_strides(&shape);TensorHandle{metadata:Metadata{shape,strides}}}
struct Dtypes{lhs_global:u8,rhs_global:u8}
struct GemmBench{b:usize,m:usize,n:usize,k:usize,lhs_layout:MatrixLayout,rhs_layout:MatrixLayout,device:Device,dtypes:Dtypes}
trait Benchmark{type Input;fn prepare(&self)->Self::Input;}
impl Benchmark for GemmBench{type Input=(TensorHandle,TensorHandle);
'''+parts['prepare']+r'''
}
fn host(shape:Vec<usize>,v:Vec<f64>)->HostData{let strides=StridedLayout::RowMajor.compute_strides(&shape);HostData{data:HostDataVec(v),shape,strides}}
fn problem(lhs:Shape,rhs:Shape,out:Shape)->MatmulProblem{let out_shape=out.iter().copied().chain([1,1]).collect();MatmulProblem{m:1,n:1,k:2,lhs_batches:lhs,rhs_batches:rhs,out_batches:out,out_shape,global_dtypes:GlobalDtypes{out:0}}}
fn bench(m:usize,k:usize,n:usize,lhs_layout:MatrixLayout,rhs_layout:MatrixLayout)->GemmBench{GemmBench{b:1,m,k,n,lhs_layout,rhs_layout,device:Device,dtypes:Dtypes{lhs_global:0,rhs_global:0}}}
fn span(t:&TensorHandle)->usize{t.metadata.shape.iter().zip(t.metadata.strides.iter()).map(|(d,s)|(d-1)*s).sum::<usize>()+1}
#[test]fn zero_and_singleton_broadcast_loses_empty_dimension(){assert_eq!(broadcast_batches(&[0],&[1]),Some(vec![1]));assert_eq!(broadcast_batches(&[1],&[0]),Some(vec![1]));}
#[test]fn implicit_leading_singleton_also_loses_empty_dimension(){assert_eq!(broadcast_batches(&[],&[0]),Some(vec![1]));}
#[test]fn ordinary_broadcast_control(){assert_eq!(broadcast_batches(&[2,1],&[3]),Some(vec![2,3]));}
#[test]fn conflicting_broadcast_control(){assert_eq!(broadcast_batches(&[0],&[2]),None);}
#[test]fn equal_zero_broadcast_control(){assert_eq!(broadcast_batches(&[0],&[0]),Some(vec![0]));}
#[test]fn rectangular_lhs_column_stride_view_exceeds_its_allocation(){let(l,_)=bench(2,3,2,MatrixLayout::ColMajor,MatrixLayout::RowMajor).prepare();assert_eq!(l.metadata.shape,vec![1,2,3]);assert_eq!(l.metadata.strides.0,vec![6,1,3]);assert_eq!(span(&l),8);assert_eq!(l.metadata.shape.iter().product::<usize>(),6);}
#[test]fn rectangular_rhs_column_stride_view_exceeds_its_allocation(){let(_,r)=bench(2,2,3,MatrixLayout::RowMajor,MatrixLayout::ColMajor).prepare();assert_eq!(r.metadata.shape,vec![1,2,3]);assert_eq!(r.metadata.strides.0,vec![6,1,3]);assert_eq!(span(&r),8);}
#[test]fn tall_column_stride_view_aliases_distinct_logical_indices(){let(l,_)=bench(3,2,2,MatrixLayout::ColMajor,MatrixLayout::RowMajor).prepare();let s=&l.metadata.strides;assert_eq!(2*s[1],s[2]);assert_eq!(l.metadata.shape,vec![1,3,2]);}
#[test]fn square_column_stride_control(){let(l,r)=bench(3,3,3,MatrixLayout::ColMajor,MatrixLayout::ColMajor).prepare();assert_eq!(span(&l),9);assert_eq!(span(&r),9);assert_eq!(l.metadata.strides.0,vec![9,1,3]);}
#[test]fn row_layout_control(){let(l,r)=bench(2,3,4,MatrixLayout::RowMajor,MatrixLayout::RowMajor).prepare();assert_eq!(span(&l),6);assert_eq!(span(&r),12);}
#[test]fn cpu_reference_same_rank_control(){let p=problem(vec![],vec![],vec![]);let lhs=host(vec![1,2],vec![2.,3.]);let rhs=host(vec![2,1],vec![4.,5.]);assert_eq!(matmul_cpu_reference(&lhs,&rhs,&p,None).data.0,vec![23.]);}
#[test]fn cpu_reference_same_rank_broadcast_control(){let p=problem(vec![1],vec![2],vec![2]);let lhs=host(vec![1,1,2],vec![2.,3.]);let rhs=host(vec![2,2,1],vec![4.,5.,6.,7.]);assert_eq!(matmul_cpu_reference(&lhs,&rhs,&p,None).data.0,vec![23.,33.]);}
#[test]fn cpu_reference_valid_rank_mismatch_panics_in_host_stride_lookup(){let p=problem(vec![],vec![2],vec![2]);let lhs=host(vec![1,2],vec![2.,3.]);let rhs=host(vec![2,2,1],vec![4.,5.,6.,7.]);assert!(std::panic::catch_unwind(||matmul_cpu_reference(&lhs,&rhs,&p,None)).is_err());}
'''
rs=HERE/'matmul_definition_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-matmul-definition.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_definition_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_definition_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '13 passed; 0 failed' in r.stdout
import numpy
numpy_result={'version':numpy.__version__,'zero_singleton':list(numpy.broadcast_shapes((0,),(1,))),'singleton_zero':list(numpy.broadcast_shapes((1,),(0,))),'empty_zero':list(numpy.broadcast_shapes((),(0,)))}
assert all(numpy_result[p]==[0] for p in ['zero_singleton','singleton_zero','empty_zero'])
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':13,'source_sha256':{p:sha(ROOT/p) for p in paths},'external_context_sha256':{str(p):sha(p) for p in external},'numpy_control':numpy_result,'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in parts.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_definition_probe.py',rs.name,'matmul_definition_compile.txt','matmul_definition_run.txt']},
 'qualification':'Exact broadcast_batches/GemmBench.prepare/matmul_cpu_reference/num_batches with exact cached HostData get_f64/strided_index and StridedLayout.compute_strides. Shape/Strides containers, tensor builder metadata/client/device/host-data payload/progress/problem fields doubled; generated row-major strides match actual shared layout helper. NumPy2.4.3 broadcast oracle. CPU layout/index behavior only, no actual TestInput/GPU launch, strategy/bounds selection, full crate build or deployed benchmark/reference receipt. Backend-reference helpers are feature-gated.'}
with (HERE/'matmul_definition_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':13,'removed_outputs':len(removed),'numpy_control':numpy_result}))
