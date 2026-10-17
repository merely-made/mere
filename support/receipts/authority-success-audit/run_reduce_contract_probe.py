"""Exact host validation and scalar interpretation of retained reduction contracts."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
prefix = 'support/patches/cubek-reduce/src/'
paths = [prefix+p for p in ['lib.rs','components/instructions/base.rs','components/instructions/topk.rs','routines/unit.rs','routines/cube.rs']]
sources = {p:(ROOT/p).read_text(encoding='utf-8-sig') for p in paths}
def function(text, name):
    start = text.index('fn '+name+'(') if 'fn '+name+'(' in text else text.index('fn '+name+'<')
    opening = text.index('{', start)
    depth = 1
    end = opening+1
    while depth:
        depth += (text[end]=='{')-(text[end]=='}')
        end += 1
    return text[start:end]
fragments = {name:function(sources[paths[0]],name) for name in ['validate_axis','validate_shapes']}
fragments['reaches'] = function(sources[paths[1]],'reaches')
fragments['topk_insert'] = function(sources[paths[2]],'topk_insert')
unit = sources[paths[3]]
fragments['unit_guard'] = 'if working_cubes != launched_cubes'+unit.split('if working_cubes != launched_cubes',1)[1].split('\n\n',1)[0]
cube = sources[paths[4]]
fragments['cube_guard'] = 'if blueprint.num_shared_accumulators != cube_dim.x as usize'+cube.split('if blueprint.num_shared_accumulators != cube_dim.x as usize',1)[1].split('\n                    if cube_dim.x',1)[0]
def adapt(text):
    return re.sub(r'#\[(comptime|unroll)\]\s*','',text).replace('N: Numeric, S: Size','N: Numeric').replace('Vector<N, S>','Vector<N>').replace('Vector<u32, S>','Vector<u32>')
code = r'''#![allow(dead_code,private_bounds,private_interfaces)]
#[derive(Debug)] enum ReduceError { InvalidAxis{axis:usize,rank:usize},ReduceAxisTooSmall{axis_length:usize,k:usize},MismatchOutputShape{expected_shape:Vec<usize>,output_shape:Vec<usize>}, Validation{details:&'static str} }
trait Numeric:Copy+PartialOrd {} impl Numeric for f32{} impl Numeric for u32{}
#[derive(Clone,Copy)]struct Vector<N>(N);
impl<N:Numeric> Vector<N>{fn vector_size(self)->usize{1} fn extract(self,_:usize)->N{self.0} fn unwrap(self)->Self{self} fn greater_than(&self,o:&Self)->Vector<bool>{Vector(self.0>o.0)} fn less_than(&self,o:&Self)->Vector<bool>{Vector(self.0<o.0)} fn equal(&self,o:&Self)->Vector<bool>{Vector(self.0==o.0)}}
trait Comptime{fn comptime(self)->Self;} impl Comptime for usize{fn comptime(self)->Self{self}}
type Array<T> = Vec<T>;
enum Value<T>{None,Single(T),Multiple(Array<T>)}
impl<T> Value<T>{fn multiple_mut(&mut self)->&mut Array<T>{match self{Self::Multiple(a)=>a,_=>panic!("expected multiple")}}}
fn select_many<N>(p:Vector<bool>,a:Vector<N>,b:Vector<N>)->Vector<N>{if p.0{a}else{b}}
'''
code += '\n'.join(fragments[n] for n in ['validate_axis','validate_shapes'])+'\n'
code += '\n'.join(adapt(fragments[n]) for n in ['reaches','topk_insert'])+'\n'
code += 'fn unit_guard(working_cubes:usize,launched_cubes:usize,blueprint:UnitBlueprint)->Result<(),ReduceError>{'+fragments['unit_guard']+'Ok(())}\n'
code += 'fn cube_guard(blueprint:CubeBlueprint,cube_dim:CubeDim)->Result<(),ReduceError>{'+fragments['cube_guard']+'Ok(())}\n'
code += r'''
#[derive(Clone,Copy)]struct Idle(bool);impl Idle{fn is_enabled(self)->bool{self.0}}
struct UnitBlueprint{unit_idle:Idle}struct CubeBlueprint{num_shared_accumulators:usize}struct CubeDim{x:u32,y:u32}
fn validate(input:&[usize],output:&[usize],axis:usize,k:Option<usize>)->Result<(),ReduceError>{validate_axis(input.len(),axis)?;validate_shapes(input,output,axis,k)}
#[test]fn axis_equal_rank_panics_after_acceptance(){assert!(validate_axis(2,2).is_ok());assert!(std::panic::catch_unwind(||validate(&[2,3],&[2,1],2,None)).is_err());}
#[test]fn rank_zero_axis_zero_panics_after_acceptance(){assert!(std::panic::catch_unwind(||validate(&[],&[],0,None)).is_err());}
#[test]fn axis_greater_rank_returns_error_control(){assert!(matches!(validate(&[2,3],&[2,1],3,None),Err(ReduceError::InvalidAxis{..})));}
#[test]fn legal_axis_control(){assert!(validate(&[2,3],&[2,1],1,None).is_ok());}
#[test]fn zero_k_is_admitted(){assert!(validate(&[2,3],&[2,0],1,Some(0)).is_ok());}
fn fold(row:&[f32],k:usize,with_coords:bool)->(Vec<f32>,Option<Vec<u32>>){
    // Same finite seed as TopK::null_accumulator; width one, no Cube DSL/backend.
    let mut elements=vec![Vector(f32::MIN);k];
    let mut coords=if with_coords{Value::Multiple(vec![Vector(u32::MAX);k])}else{Value::None};
    for (i,&x) in row.iter().enumerate(){let c=if with_coords{Value::Single(Vector(i as u32))}else{Value::None};topk_insert(&mut elements,&mut coords,Vector(x),&c,k);}
    (elements.into_iter().map(|v|v.0).collect(),match coords{Value::Multiple(v)=>Some(v.into_iter().map(|v|v.0).collect()),_=>None})
}
#[test]fn topk_negative_infinity_is_rejected_by_finite_seed(){let(v,c)=fold(&[f32::NEG_INFINITY],1,true);assert_eq!(v,vec![f32::MIN]);assert_eq!(c,Some(vec![u32::MAX]));}
#[test]fn topk_values_only_also_retains_seed(){assert_eq!(fold(&[f32::NEG_INFINITY],1,false).0,vec![f32::MIN]);}
#[test]fn topk_mixed_finite_negative_infinity_keeps_invalid_second_slot(){let(v,c)=fold(&[3.,f32::NEG_INFINITY],2,true);assert_eq!(v,vec![3.,f32::MIN]);assert_eq!(c,Some(vec![0,u32::MAX]));}
#[test]fn seed_equal_coordinate_control(){let(v,c)=fold(&[f32::MIN,f32::MIN],2,true);assert_eq!(v,vec![f32::MIN,f32::MIN]);assert_eq!(c,Some(vec![0,1]));}
#[test]fn finite_sort_control(){let(v,c)=fold(&[1.,8.,3.],2,true);assert_eq!(v,vec![8.,3.]);assert_eq!(c,Some(vec![1,2]));}
#[test]fn zero_k_insert_panics(){assert!(std::panic::catch_unwind(||fold(&[1.],0,true)).is_err());}
#[test]fn topk_nan_order_changes_winner(){assert_eq!(fold(&[f32::NAN,1.],1,true).0,vec![1.]);assert!(fold(&[1.,f32::NAN],1,true).0[0].is_nan());}
#[test]fn forced_unit_enabled_idle_rejected_on_extra_cubes(){assert!(unit_guard(1,2,UnitBlueprint{unit_idle:Idle(true)}).is_err());}
#[test]fn forced_unit_disabled_idle_admits_extra_cubes(){assert!(unit_guard(1,2,UnitBlueprint{unit_idle:Idle(false)}).is_ok());}
#[test]fn forced_unit_partial_cube_guard_never_compares_units(){let units=1usize;let cube_units=32usize;let cubes=units.div_ceil(cube_units);assert!(unit_guard(cubes,cubes,UnitBlueprint{unit_idle:Idle(false)}).is_ok());assert!(cubes*cube_units>units);}
#[test]fn forced_cube_rejects_inferred_per_plane_slot_count(){let d=CubeDim{x:32,y:4};assert!(cube_guard(CubeBlueprint{num_shared_accumulators:d.y as usize},d).is_err());}
#[test]fn forced_cube_accepts_lane_count_instead_of_plane_count(){let d=CubeDim{x:32,y:4};assert!(cube_guard(CubeBlueprint{num_shared_accumulators:d.x as usize},d).is_ok());}
'''
rs=HERE/'reduce_contract_probe.rs'
with rs.open('x',encoding='utf-8') as f:f.write(code)
exe=Path('C:/t/cargo-targets/mere/audit-reduce-contract.exe');pdb=exe.with_suffix('.pdb')
assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'reduce_contract_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'reduce_contract_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '17 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
    if not p.exists():continue
    assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
    assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'passed':17,'exit':0,'source_sha256':{p:sha(ROOT/p) for p in paths},'extracted_fragment_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in fragments.items()},'compile_command':command,'removed_outputs':removed,'evidence_sha256':{p:sha(HERE/p) for p in ['run_reduce_contract_probe.py',rs.name,'reduce_contract_compile.txt','reduce_contract_run.txt']},'qualification':'Exact two host validation functions, exact forced-unit and forced-cube branch predicates with explicit launch/blueprint doubles. Top-k exact reaches + whole topk_insert with Cube attributes removed and one-component vector/Value/Array doubles; initialization explicitly copies finite f32::MIN seed. Tests pin CPU control flow only. No Cube DSL expansion, real hardware selection, generated kernel, public Client call, synchronization, actual GPU result or crate suite executed. NaN case exposes ordering difference, not a settled TopK NaN contract. Owned EXE/PDB removed.'}
with (HERE/'reduce_contract_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
