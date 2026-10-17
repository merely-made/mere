"""Exact reader validator bodies with explicit model config and other validators stubbed."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubek-matmul/src/multi_level/components/global/read/strategy/'
names=['sync_partial_cyclic','async_partial_cyclic','sync_full_cyclic','async_full_cyclic','sync_full_ordered']
paths=[base+n+'.rs' for n in names]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
methods={n:function((ROOT/p).read_text(encoding='utf-8-sig'),'fn validate_with_config(') for n,p in zip(names,paths)}
code=r'''#![allow(dead_code,unused_variables)]
use std::marker::PhantomData;
type InvalidConfigError=Box<dyn std::fmt::Debug>;
struct FormattedConfigError;impl FormattedConfigError{fn new(f:impl FnOnce()->String)->InvalidConfigError{Box::new(f())}}
struct DeviceProperties;
#[derive(Clone,Copy)]struct Dtype;impl Dtype{fn size_bits(&self)->usize{32}}
#[derive(Clone,Copy)]struct StageMemoryConfig{dtype:Dtype,vector_size:u32,num_planes:u32,elements_per_tile:u32,tiles_row:u32,tiles_col:u32,contiguous_width:u32}
impl StageMemoryConfig{
 fn elements_per_tile(&self)->u32{self.elements_per_tile}
 fn tiles_per_stage(&self)->u32{self.tiles_row*self.tiles_col}
 fn elements_per_stage(&self)->u32{self.elements_per_tile()*self.tiles_per_stage()}
 fn elements_per_tile_along_contiguous_dim(&self)->u32{self.contiguous_width}
 fn tiles_per_stage_along_row(&self)->u32{self.tiles_row}
 fn tiles_per_stage_along_col(&self)->u32{self.tiles_col}
}
#[derive(Clone,Copy)]struct GlobalMemoryConfig{dtype:Dtype,vector_size:usize}
#[derive(Clone,Copy)]enum ReaderMode{Strict,Relaxed}
#[derive(Clone,Copy,PartialEq)]enum StageIdent{Lhs,Rhs}
#[derive(Clone,Copy)]enum InputLoadFlow{MainOnly,LoadOnly}
#[derive(Clone,Copy)]struct GlobalReaderConfig{smem_config:StageMemoryConfig,gmem_config:GlobalMemoryConfig,reader_mode:ReaderMode,plane_dim:u32,stage_ident:StageIdent,input_load_flow:InputLoadFlow}
impl GlobalReaderConfig{fn loading_units_count(&self)->u32{self.plane_dim*self.smem_config.num_planes}fn loading_planes_count(&self)->u32{self.smem_config.num_planes}}
trait TilingOrder{}
struct TestOrder;impl TilingOrder for TestOrder{}
struct OrderedTilingOrder;impl TilingOrder for OrderedTilingOrder{}
struct ContiguousTilingLayout<T>(PhantomData<T>);impl<T:TilingOrder>ContiguousTilingLayout<T>{fn check(_:StageMemoryConfig)->Result<(),InvalidConfigError>{Ok(())}}
fn validate_swizzle_atom_size(_:StageMemoryConfig)->Result<(),InvalidConfigError>{Ok(())}
fn validate_async_barrier(_: &DeviceProperties)->Result<(),InvalidConfigError>{Ok(())}
fn validate_async_copy(_: &DeviceProperties,_a:&Dtype,_b:&Dtype)->Result<(),InvalidConfigError>{Ok(())}
const ASYNC_COPY_WIDTH:u32=128;
trait LoadingValidation{fn validate_with_config(device_props:&DeviceProperties,config:&GlobalReaderConfig)->Result<(),InvalidConfigError>;}
'''
for n in names:
 if n=='sync_full_ordered':code+='struct Ordered;impl LoadingValidation for Ordered{\n'+methods[n]+'\n}\n'
 else:code+=f'struct {n}<TO>(PhantomData<TO>);impl<TO:TilingOrder>LoadingValidation for {n}<TO>{{\n'+methods[n]+'\n}\n'
code+=r'''
fn config(vectors:u32)->GlobalReaderConfig{GlobalReaderConfig{smem_config:StageMemoryConfig{dtype:Dtype,vector_size:4,num_planes:1,elements_per_tile:4,tiles_row:1,tiles_col:vectors,contiguous_width:4},gmem_config:GlobalMemoryConfig{dtype:Dtype,vector_size:4},reader_mode:ReaderMode::Strict,plane_dim:32,stage_ident:StageIdent::Lhs,input_load_flow:InputLoadFlow::MainOnly}}
fn last_start(c:&GlobalReaderConfig)->u32{let v=4;let units=c.loading_units_count();let total=c.smem_config.elements_per_stage()/v;let tasks=total.div_ceil(units);(units-1)*v+(tasks-1)*units*v}
#[test]fn sync_partial_strict_admits_start_exactly_at_stage_end(){let c=config(63);assert_eq!(last_start(&c),c.smem_config.elements_per_stage());assert!(sync_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_ok());}
#[test]fn async_partial_strict_admits_start_exactly_at_stage_end(){let c=config(63);assert_eq!(last_start(&c),c.smem_config.elements_per_stage());assert!(async_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_ok());}
#[test]fn sync_full_rejects_same_imbalanced_workload(){assert!(sync_full_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(63)).is_err());}
#[test]fn async_full_rejects_same_imbalanced_workload(){assert!(async_full_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(63)).is_err());}
#[test]fn sync_partial_balanced_control(){assert!(sync_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(64)).is_ok());}
#[test]fn async_partial_balanced_control(){assert!(async_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(64)).is_ok());}
#[test]fn sync_partial_further_overrun_control_is_rejected(){let c=config(62);assert!(last_start(&c)>c.smem_config.elements_per_stage());assert!(sync_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_err());}
#[test]fn async_partial_further_overrun_control_is_rejected(){let c=config(62);assert!(last_start(&c)>c.smem_config.elements_per_stage());assert!(async_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_err());}
fn ordered_config(flow:InputLoadFlow)->GlobalReaderConfig{let mut c=config(1);c.smem_config.elements_per_tile=256;c.smem_config.contiguous_width=16;c.input_load_flow=flow;c}
#[test]fn ordered_main_flow_control(){assert!(Ordered::validate_with_config(&DeviceProperties,&ordered_config(InputLoadFlow::MainOnly)).is_ok());}
#[test]fn ordered_loader_validator_alone_also_accepts_load_only(){assert!(Ordered::validate_with_config(&DeviceProperties,&ordered_config(InputLoadFlow::LoadOnly)).is_ok());}
'''
rs=HERE/'matmul_reader_validation_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-matmul-reader-validation.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_reader_validation_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_reader_validation_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '10 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':10,'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in methods.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_reader_validation_probe.py',rs.name,'matmul_reader_validation_compile.txt','matmul_reader_validation_run.txt']},
 'qualification':'Exact host validator bodies for sync/async partial/full cyclic and ordered readers. Config shape/count/type methods are explicit scalar doubles; tiling/swizzle/async capability validators are deliberately stubbed Ok. Proves these predicates admit the end-start boundary and ordered validator alone ignores load flow; does not prove complete validator/blueprint/routine admission of any actual tile config, concrete caller reachability, CubeDSL lowering, GPU access or deployed fault. Full/balanced/further-overrun/ordered-main controls included. Owned exe/PDB removed.'}
with (HERE/'matmul_reader_validation_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':10,'removed_outputs':len(removed)}))
