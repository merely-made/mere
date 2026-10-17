"""Exact host batch-layout payload builder and declared CPU interpretation of layout body."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source='support/patches/cubek-matmul/src/multi_level/components/global/memory/layout.rs'
s=(ROOT/source).read_text(encoding='utf-8-sig')
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
host=function(s[s.index('impl BatchLayoutLaunch {'):],'pub fn from_handle(')
body=function(s[s.index('impl Layout for BatchLayout {'):],'fn to_source_pos(')
assert body.count('#[unroll]')==1
interpreted=body.replace('#[unroll]','')
code=r'''#![allow(dead_code)]
struct TensorBinding{shape:Vec<usize>,strides:Vec<usize>}
struct MatmulProblem{out_batches:Vec<usize>}
struct BatchLayoutLaunch{batch_shape:Vec<u32>,batch_strides:Vec<usize>}
impl BatchLayoutLaunch{
 fn new(batch_shape:Vec<u32>,batch_strides:Vec<usize>)->Self{Self{batch_shape,batch_strides}}
'''+host+r'''
}
#[derive(Clone,Copy)]struct FastDivmod(u32);impl FastDivmod{fn div_mod(&self,x:u32)->(u32,u32){(x/self.0,x%self.0)}}
struct Sequence<T>(Vec<T>);impl<T:Clone> Sequence<T>{fn reversed(&self)->Self{Self(self.0.iter().rev().cloned().collect())}fn len(&self)->usize{self.0.len()}}
impl<T>std::ops::Index<usize> for Sequence<T>{type Output=T;fn index(&self,i:usize)->&T{&self.0[i]}}
trait Layout{type Coordinates;type SourceCoordinates;fn to_source_pos(&self,pos:Self::Coordinates)->Self::SourceCoordinates;}
struct BatchLayout{batch_shape:Sequence<FastDivmod>,batch_strides:Sequence<usize>}
impl From<BatchLayoutLaunch> for BatchLayout{fn from(v:BatchLayoutLaunch)->Self{Self{batch_shape:Sequence(v.batch_shape.into_iter().map(FastDivmod).collect()),batch_strides:Sequence(v.batch_strides)}}}
impl Layout for BatchLayout{type Coordinates=usize;type SourceCoordinates=usize;
'''+interpreted+r'''
}
fn payload(shape:Vec<usize>,strides:Vec<usize>,out_batches:Vec<usize>)->BatchLayoutLaunch{BatchLayoutLaunch::from_handle(&TensorBinding{shape,strides},&MatmulProblem{out_batches})}
#[test]fn unbatched_operand_against_batched_output_has_no_stride_for_its_batch_axis(){let p=payload(vec![2,3],vec![3,1],vec![4]);assert_eq!(p.batch_shape,vec![4]);assert!(p.batch_strides.is_empty());let l=BatchLayout::from(p);assert!(std::panic::catch_unwind(||l.to_source_pos(0)).is_err());}
#[test]fn shorter_batch_rank_has_fewer_strides_than_output_divisors(){let p=payload(vec![3,2,3],vec![6,3,1],vec![2,3]);assert_eq!(p.batch_shape,vec![2,3]);assert_eq!(p.batch_strides,vec![6]);let l=BatchLayout::from(p);assert!(std::panic::catch_unwind(||l.to_source_pos(0)).is_err());}
#[test]fn same_rank_nonbroadcast_control_maps_offsets(){let l=BatchLayout::from(payload(vec![4,2,3],vec![6,3,1],vec![4]));assert_eq!((0..4).map(|i|l.to_source_pos(i)).collect::<Vec<_>>(),vec![0,6,12,18]);}
#[test]fn same_rank_singleton_control_maps_zero(){let l=BatchLayout::from(payload(vec![1,2,3],vec![6,3,1],vec![4]));assert_eq!((0..4).map(|i|l.to_source_pos(i)).collect::<Vec<_>>(),vec![0,0,0,0]);}
#[test]fn unbatched_output_control_has_no_batch_axis_to_index(){let l=BatchLayout::from(payload(vec![2,3],vec![3,1],vec![]));assert_eq!(l.to_source_pos(0),0);}
#[test]fn same_rank_two_axis_broadcast_control(){let l=BatchLayout::from(payload(vec![2,1,2,3],vec![6,6,3,1],vec![2,3]));assert_eq!((0..6).map(|i|l.to_source_pos(i)).collect::<Vec<_>>(),vec![0,0,0,6,6,6]);}
'''
rs=HERE/'matmul_batch_layout_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-matmul-batch-layout.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_batch_layout_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_batch_layout_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '6 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':6,'source_sha256':{source:sha(ROOT/source)},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in {'from_handle':host,'to_source_pos_original':body,'to_source_pos_interpreted':interpreted}.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_batch_layout_probe.py',rs.name,'matmul_batch_layout_compile.txt','matmul_batch_layout_run.txt']},
 'qualification':'Exact host BatchLayoutLaunch.from_handle with shape-equivalent TensorBinding/problem/generated-launch DTO. Kernel layout to_source_pos body interpreted as CPU Rust after removal of one #[unroll] attribute; Sequence/reversed/FastDivmod are explicit scalar doubles. Proves actual host payload rank mismatch and scalar interpretation bounds failure, not CubeDSL lowering, GPU memory access, actual launcher acceptance or current product incidence. Same-rank/unbatched controls pass. No full crate build or GPU. Owned exe/PDB removed.'}
with (HERE/'matmul_batch_layout_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':6,'removed_outputs':len(removed)}))
