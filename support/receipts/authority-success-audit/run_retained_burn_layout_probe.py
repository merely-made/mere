"""CPU layout/range evidence for the retained, root-unselected Burn backend."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base='support/patches/burn-cubecl/src/'
paths=[base+'kernel/conv/im2col.rs',base+'kernel/conv/conv_transpose2d/transpose_direct.rs',base+'fusion/registry.rs']
texts=[(ROOT/p).read_text(encoding='utf-8-sig') for p in paths];pieces={}
def cut(s,key,start,end=None):
 a=s.index(start);b=s.index(end,a) if end else len(s);pieces[key]=s[a:b];return pieces[key]
spatial=cut(texts[0],'is_spatial_contiguous','fn is_spatial_contiguous(', '\nfn from_handle(')
registry_error=cut(texts[2],'registry_error','#[derive(Debug, Clone, PartialEq, Eq)]\npub enum RegistryError', '\nimpl core::fmt::Display')
registry_core=cut(texts[2],'registry_core_and_original_tests','type Slot = Box<dyn Any + Send + Sync>;')
ranges=cut(texts[1],'direct_transpose_range_arithmetic','    let stride_0_i =', '    let idx_input_batch =')
code='#![allow(dead_code)]\nuse core::any::Any;\n'+registry_error+registry_core+spatial+r'''
struct Args{conv_stride_0:usize,conv_stride_1:usize,dilation_0:usize,dilation_1:usize,padding_0:usize,padding_1:usize}
struct Input{height:usize,width:usize}
impl Input{fn shape(&self,dim:usize)->usize{match dim{2=>self.height,3=>self.width,_=>panic!("fixture shape")}}}
fn clamp(v:i32,a:i32,b:i32)->i32{v.clamp(a,b)}
fn clamp_min(v:i32,a:i32)->i32{v.max(a)}
fn direct_ranges(kernel_h:usize,kernel_w:usize,out_y:usize,out_x:usize,input:&Input,args:&Args)->((usize,usize),(usize,usize)){
'''+ranges+r'''
 ((y_start,y_end),(x_start,x_end))
}
#[cfg(test)]mod layout_cases{
use super::*;
fn source_offsets(shape:&[usize],strides:&[usize])->Vec<usize>{
 let count:usize=shape.iter().product();(0..count).map(|index|{
  let mut rest=index;let mut offset=0;for d in (0..shape.len()).rev(){offset+=(rest%shape[d])*strides[d];rest/=shape[d];}offset
 }).collect()
}
fn flattened_offsets(shape:&[usize],strides:&[usize])->Vec<usize>{
 let c=shape.len()-1;let rows:usize=shape[..c].iter().product();
 (0..rows).flat_map(|row|(0..shape[c]).map(move|col|row*strides[c-1]+col*strides[c])).collect()
}
#[test]fn sliced_batch_gap_is_wrongly_declared_flattenable(){
 // Slice one H row from each of two [2,2,2,1] batch images.
 let shape=[2,1,2,1];let strides=[4,2,1,1];
 assert!(is_spatial_contiguous(&shape,&strides));
 let original=source_offsets(&shape,&strides);let flattened=flattened_offsets(&shape,&strides);
 assert_eq!(original,vec![0,1,4,5]);assert_eq!(flattened,vec![0,1,2,3]);assert_ne!(original,flattened);
 println!("retained reshape_input accepts batch gap: source={original:?}, flattened={flattened:?}");
}
#[test]fn truly_contiguous_batch_control_agrees(){
 let shape=[2,1,2,1];let strides=[2,2,1,1];assert!(is_spatial_contiguous(&shape,&strides));assert_eq!(source_offsets(&shape,&strides),flattened_offsets(&shape,&strides));
}
#[test]fn spatial_gap_control_is_refused(){assert!(!is_spatial_contiguous(&[2,2,2,1],&[8,4,1,1]));}
#[test]fn channel_gap_control_is_refused(){assert!(!is_spatial_contiguous(&[2,2,2,1],&[8,4,2,2]));}
fn args(stride:usize)->Args{Args{conv_stride_0:stride,conv_stride_1:stride,dilation_0:1,dilation_1:1,padding_0:0,padding_1:0}}
#[test]fn stride_wider_than_unit_kernel_misses_valid_origin(){
 let(r,c)=direct_ranges(1,1,0,0,&Input{height:2,width:2},&args(3));
 assert_eq!((r,c),((0,0),(0,0)));
 // For transposed convolution, input(0,0)*weight(0,0) contributes
 // to output(0,0) when stride=3,padding=0,dilation=1,kernel=1.
 assert_eq!(0usize*3+0usize*1,0);assert!(!(r.0..r.1).contains(&0));
 println!("retained direct transpose unit kernel stride3 excludes valid origin: rows={r:?} columns={c:?}");
}
#[test]fn unit_stride_unit_kernel_origin_control(){
 let(r,c)=direct_ranges(1,1,0,0,&Input{height:2,width:2},&args(1));assert!((r.0..r.1).contains(&0)&&(c.0..c.1).contains(&0));
}
#[test]fn unit_stride_three_kernel_origin_control(){
 let(r,c)=direct_ranges(3,3,0,0,&Input{height:2,width:2},&args(1));assert!((r.0..r.1).contains(&0)&&(c.0..c.1).contains(&0));
}
}
'''
rs=HERE/'retained_burn_layout_probe.rs';rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-retained-burn-layout.exe');pdb=exe.with_suffix('.pdb');assert exe.parent.is_dir() and not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'retained_burn_layout_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'retained_burn_layout_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0 and '13 passed; 0 failed' in r.stdout
removed=[]
for out in [exe,pdb]:
 if not out.exists():continue
 assert out.resolve().parent==exe.parent.resolve();b=out.read_bytes()
 assert (out.suffix=='.exe' and b.startswith(b'MZ')) or (out.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(out),'bytes':len(b),'sha256':sha(out)});out.unlink()
names={'burn-cubecl','cubecl-runtime','cubecl-wgpu','cubek-matmul','cubek-reduce','burn-remote'}
lock=tomllib.loads((ROOT/'Cargo.lock').read_text(encoding='utf-8-sig'))
selection=[{'name':p['name'],'version':p['version'],'source':p.get('source')} for p in lock['package'] if p['name'] in names]
assert next(p for p in selection if p['name']=='burn-cubecl')['source'].startswith('registry+')
(HERE/'retained_burn_root_selection.json').write_text(json.dumps(selection,indent=2)+'\n',encoding='utf-8')
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':13,'original_registry_tests':6,'new_layout_cases':4,'new_range_cases':3,'compile_command':command,
 'source_sha256':{p:sha(ROOT/p) for p in paths},'root_selection_input_sha256':{p:sha(ROOT/p) for p in ['Cargo.toml','Cargo.lock']},
 'extracted_body_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_retained_burn_layout_probe.py','retained_burn_layout_probe.rs','retained_burn_layout_compile.txt','retained_burn_layout_run.txt','retained_burn_root_selection.json']},
 'qualification':'Retained burn-cubecl copy is not root-selected; root uses registry0.22.0, retained manifest0.22.0-pre.4.13 CPU tests:6 unchanged original type-erased registry core tests;4 actual layout predicate cases with independently enumerated source/flattened addresses;3 exact extracted direct-transpose integer-range blocks using scalar arg/input/clamp doubles. No CubeCL macro expansion, shader/backend build, actual GPU/tensor convolution or current registry backend failure claimed. Two exact owned exe/PDB removed.'}
with (HERE/'retained_burn_layout_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':13,'removed_outputs':len(removed),'root_burn_source':'registry'}))
