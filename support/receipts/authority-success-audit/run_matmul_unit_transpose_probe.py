"""CPU interpretation of the exact transposed-load loop with explicit vector/slice doubles."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source='support/patches/cubek-matmul/src/multi_level/tile/variants/unit.rs'
text=(ROOT/source).read_text(encoding='utf-8-sig')
body=text.split('fn strided_tile_to_transposed_unit_tile<',1)[1].split(') {',1)[1].rsplit('}',1)[0]
assert body.count('#[unroll]')==1
# The loop and arithmetic remain exact. Only the CubeDSL unroll attribute is
# removed; compile-time values/cast/intrinsics are supplied by visible doubles.
body=body.replace('#[unroll]','')
code=r'''#![allow(dead_code)]
trait Comptime:Sized{fn comptime(self)->Self{self}}impl Comptime for u32{}impl Comptime for usize{}
trait Size{fn value()->usize;}struct Width<const V:usize>;impl<const V:usize>Size for Width<V>{fn value()->usize{V}}
struct E2;impl E2{fn cast_from(v:i32)->i32{v}}
struct Layout{num_rows:u32,num_cols:u32}
struct UnitTile{data:Vec<i32>,layout:Layout}
struct Vector(Vec<i32>);impl Vector{fn extract(&self,i:usize)->i32{self.0[i]}}
struct StridedTile{data:Vec<i32>,input_cols:usize,vector_size:usize}
impl StridedTile{fn get_vector(&self,row:u32,col:u32)->Vector{let start=row as usize*self.input_cols+col as usize*self.vector_size;Vector(self.data[start..start+self.vector_size].to_vec())}}
'''
code+='fn load<N:Size>(strided_tile:&StridedTile,unit_tile:&mut UnitTile){'+body+'}\n'
code+=r'''
fn run<const V:usize>(rows:u32,cols:u32)->Vec<i32>{let n=(rows*cols)as usize;let input=StridedTile{data:(0..n as i32).collect(),input_cols:rows as usize,vector_size:V};let mut output=UnitTile{data:vec![-999;n],layout:Layout{num_rows:rows,num_cols:cols}};load::<Width<V>>(&input,&mut output);output.data}
fn transpose(rows:u32,cols:u32)->Vec<i32>{(0..rows).flat_map(|r|(0..cols).map(move|c|(c*rows+r)as i32)).collect()}
#[test]fn vector2_transpose_overwrites_and_leaves_holes(){let got=run::<2>(4,2);assert_eq!(got,vec![0,4,2,6,3,7,-999,-999]);assert_ne!(got,transpose(4,2));}
#[test]fn vector1_control_is_correct(){assert_eq!(run::<1>(4,2),transpose(4,2));}
#[test]fn vector2_single_input_vector_control(){assert_eq!(run::<2>(2,2),transpose(2,2));}
#[test]fn valid_transposed_input_contiguous_axis_is_rejected_by_wrong_axis_assert(){assert_eq!(2u32%2,0);assert!(std::panic::catch_unwind(||run::<2>(2,3)).is_err());}
'''
rs=HERE/'matmul_unit_transpose_probe.rs'
with rs.open('x',encoding='utf-8') as f:f.write(code)
exe=Path('C:/t/cargo-targets/mere/audit-matmul-unit-transpose.exe');pdb=exe.with_suffix('.pdb')
assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_unit_transpose_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_unit_transpose_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '4 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':4,'source_sha256':{source:sha(ROOT/source)},'compile_command':command,'removed_outputs':removed,'extracted_loop_sha256':hashlib.sha256(body.encode()).hexdigest(),
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_unit_transpose_probe.py',rs.name,'matmul_unit_transpose_compile.txt','matmul_unit_transpose_run.txt','matmul_unit_transpose_initial.rs','matmul_unit_transpose_initial_compile.txt']},
 'qualification':'Exact transposed UnitTile load loop CPU interpretation, removing one CubeDSL unroll attribute only; explicit comptime/vector/cast/slice doubles. Vector2 multi-vector rows overwrite indices and leave output holes; vector1 and single-vector controls pass. Separate wrong-axis divisibility assertion rejects valid input-row width. Whole-source search found no explicit true transposed-load constructor in current owned source; no current product invocation, CubeDSL lowering, GPU fault or full crate receipt claimed. Owned exe/PDB removed.'}
with (HERE/'matmul_unit_transpose_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
