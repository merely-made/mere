"""Retained 0.10 backend CPU predicates and exact WGSL formatter/helper discovery."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/cubecl-wgpu/src/'
paths=[base+p for p in ['backend/vulkan.rs','backend/vulkan/features.rs','compiler/wgsl/base.rs','compiler/wgsl/extension.rs','compiler/wgsl/instructions.rs','compiler/wgsl/compiler.rs','compiler/wgsl/shader.rs']]
vk,features,values,extension,instructions,compiler,shader=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths]
def function(s,start):
 a=s.index(start);b=s.index('{',a);depth=1;n=b+1
 while depth:
  if s[n]=='{':depth+=1
  elif s[n]=='}':depth-=1
  n+=1
 return s[a:n]
discover=function(compiler,'fn register_extensions(')
pred=vk[vk.index('let storage16 ='):vk.index('for ty in default_types')]
code=r'''#![allow(dead_code,private_interfaces,non_camel_case_types)]
extern crate self as cubecl_core;
extern crate self as half;
pub type f16=u16;
pub mod ir {
 pub type Id=u16;
 #[derive(Debug,Clone,PartialEq)]pub enum ConstantValue{UInt(u64),Int(i64),Float(f64),Bool(bool)}
 impl std::fmt::Display for ConstantValue{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{match self {Self::UInt(v)=>write!(f,"{v}"),Self::Int(v)=>write!(f,"{v}"),Self::Float(v)=>write!(f,"{v}"),Self::Bool(v)=>write!(f,"{v}")}}}
}
mod compiler {pub mod wgsl {
 mod base {
'''+values+r'''
 }
 pub use base::*;
 #[derive(Debug,Clone)]pub struct Subgroup;
 impl std::fmt::Display for Subgroup{fn fmt(&self,_f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{panic!("subgroup formatter not exercised")}}
 mod extension {
'''+extension+r'''
 }
 pub use extension::*;
 mod instructions {
'''+instructions+r'''
 }
 pub use instructions::*;
 use crate::compiler::wgsl as wgsl;
'''+discover+r'''
 #[cfg(test)]mod audit {
 use super::*;
 fn input()->Variable{Variable::Named{name:"x".into(),item:Item::Scalar(Elem::F32),is_array:false}}
 fn output()->Variable{Variable::LocalMut{id:1,item:Item::Scalar(Elem::Bool)}}
 fn check()->Instruction{Instruction::IsNan{input:input(),out:output()}}
 #[test]fn direct_helper_registration_control(){let e=register_extensions(&[check()]);assert_eq!(e.len(),2);let calls=check().to_string();let defs=e.iter().map(ToString::to_string).collect::<String>();assert!(calls.contains("is_nan_1_f32"));assert!(defs.contains("fn is_nan_1_f32"));}
 #[test]fn if_loop_and_range_registration_control(){
  let wrappers=vec![Instruction::IfElse{cond:output(),instructions_if:vec![],instructions_else:vec![check()]},Instruction::Loop{instructions:vec![check()]},Instruction::RangeLoop{i:input(),start:input(),end:input(),step:None,inclusive:false,instructions:vec![check()]}];
  for wrapper in wrappers {assert_eq!(register_extensions(&[wrapper]).len(),2);}
 }
 #[test]fn switch_case_calls_helper_without_registering_definition(){let switch=Instruction::Switch{value:input(),instructions_default:vec![],cases:vec![(input(),vec![check()])]};assert!(switch.to_string().contains("is_nan_1_f32"));assert!(register_extensions(&[switch]).is_empty());}
 #[test]fn switch_default_calls_helper_without_registering_definition(){let switch=Instruction::Switch{value:input(),instructions_default:vec![check()],cases:vec![]};assert!(switch.to_string().contains("is_nan_1_f32"));assert!(register_extensions(&[switch]).is_empty());}
 #[test]fn vec4_index_assignment_omits_statement_terminator(){
  let instruction=Instruction::IndexAssign{index:Variable::LocalMut{id:0,item:Item::Vec4(Elem::U32)},rhs:Variable::LocalMut{id:1,item:Item::Vec4(Elem::F32)},out:Variable::GlobalOutputArray(0,Item::Vec4(Elem::F32))};
  let text=instruction.to_string();assert!(text.contains("= vec4("));assert!(!text.trim_end().ends_with(';'));
 }
 #[test]fn vec3_index_assignment_terminator_control(){
  let instruction=Instruction::IndexAssign{index:Variable::LocalMut{id:0,item:Item::Vec3(Elem::U32)},rhs:Variable::LocalMut{id:1,item:Item::Vec3(Elem::F32)},out:Variable::GlobalOutputArray(0,Item::Vec3(Elem::F32))};assert!(instruction.to_string().trim_end().ends_with(';'));
 }
 }
}}
mod storage_predicate {
 const TRUE:u32=1;
 #[derive(Clone,Copy)]struct Storage16{uniform_and_storage_buffer16_bit_access:u32}
 #[derive(Clone,Copy)]struct Storage8{uniform_and_storage_buffer8_bit_access:u32}
 struct ExtendedFeatures{buf_16:Option<Storage16>,buf_8:Option<Storage8>}
 fn predicates(ext_feat:&ExtendedFeatures)->(bool,bool){
'''+pred+r'''
 (storage16,storage8)
 }
 #[test]fn sixteen_only_falsely_admits_eight(){assert_eq!(predicates(&ExtendedFeatures{buf_16:Some(Storage16{uniform_and_storage_buffer16_bit_access:1}),buf_8:None}),(true,true));}
 #[test]fn eight_only_is_falsely_excluded(){assert_eq!(predicates(&ExtendedFeatures{buf_16:None,buf_8:Some(Storage8{uniform_and_storage_buffer8_bit_access:1})}),(false,false));}
 #[test]fn both_absent_control(){assert_eq!(predicates(&ExtendedFeatures{buf_16:None,buf_8:None}),(false,false));}
 #[test]fn both_present_control(){assert_eq!(predicates(&ExtendedFeatures{buf_16:Some(Storage16{uniform_and_storage_buffer16_bit_access:1}),buf_8:Some(Storage8{uniform_and_storage_buffer8_bit_access:1})}),(true,true));}
}
'''
# Exercise the exact private mask selector and emitted text in its own source module.
masktests=r'''
#[cfg(test)]mod audit_masks {
 use super::*;
 fn numeric(s:&str)->u64{u64::from_str_radix(s.trim_start_matches("0x").trim_end_matches('u'),16).unwrap()}
 fn classify(elem:Elem,bits:u64)->(bool,bool){let(inf,mask,_)=select_inf_bits_abs_mask_uint(&elem);let abs=bits&numeric(mask);(abs>numeric(inf),abs==numeric(inf))}
 #[test]fn f32_classification_control(){assert_eq!(classify(Elem::F32,f32::INFINITY.to_bits() as u64),(false,true));assert_eq!(classify(Elem::F32,f32::NAN.to_bits() as u64),(true,false));}
 #[test]fn f64_nan_and_infinity_are_lost_by_swapped_constants(){assert_eq!(classify(Elem::F64,f64::INFINITY.to_bits()),(false,false));assert_eq!(classify(Elem::F64,f64::NAN.to_bits()),(false,false));let nan=Extension::IsNanPrimitive(Elem::F64).to_string();let inf=Extension::IsInfPrimitive(Elem::F64).to_string();assert!(nan.contains("bits & 0x7ff0000000000000"));assert!(nan.contains("> 0x7fffffffffffffff"));assert!(inf.contains("== 0x7fffffffffffffff"));}
}
'''
needle='\n }\n pub use extension::*;'
assert code.count(needle)==1;code=code.replace(needle,masktests+needle)
rs=HERE/'retained_wgpu_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-retained-wgpu.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'retained_wgpu_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'retained_wgpu_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '12 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':12,'source_sha256':{p:sha(ROOT/p) for p in paths},'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in {'register_extensions':discover,'storage_predicates':pred,'full_values':values,'full_extension':extension,'full_instructions':instructions}.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_retained_wgpu_probe.py',rs.name,'retained_wgpu_compile.txt','retained_wgpu_run.txt']},
 'qualification':'Retained 0.10 backend, root lock selects registry0.11. Exact full WGSL base/extension/instructions source and extracted register_extensions; only core Id/constant Display/f16 storage/Subgroup types doubled, no Subgroup execution. Exact Vulkan storage predicate with feature structs. CPU emitted text/helper-list/IEEE-bit classification witnesses only. No parser/Naga validation, GPU/backend/actual Vulkan feature query, concrete caller or shipped failure. Owned exe/PDB removed.'}
with (HERE/'retained_wgpu_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':12,'removed_outputs':len(removed)}))
