"""Exact numerical refusal expression from the split-cube correctness gate."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source='support/patches/cubek-matmul/src/tiled/eval/split_cubes/mod.rs'
text=(ROOT/source).read_text(encoding='utf-8-sig')
condition=text.split('            if ',1)[1].split(' {',1)[0]
assert condition=='(have - want).abs() > want.abs() * 1e-3 + 1e-3'
code='fn rejects(have:f32,want:f32)->bool{'+condition+'}\n'+r'''
#[test]fn nan_output_is_not_rejected(){assert!(!rejects(f32::NAN,8.0));}
#[test]fn wrong_finite_output_is_rejected(){assert!(rejects(80.0,8.0));}
#[test]fn equal_finite_output_control(){assert!(!rejects(8.0,8.0));}
#[test]fn infinite_output_is_rejected(){assert!(rejects(f32::INFINITY,8.0));}
'''
rs=HERE/'matmul_split_verify_probe.rs'
with rs.open('x',encoding='utf-8') as f:f.write(code)
exe=Path('C:/t/cargo-targets/mere/audit-matmul-split-verify.exe');pdb=exe.with_suffix('.pdb')
assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'matmul_split_verify_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'matmul_split_verify_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '4 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':4,'source_sha256':{source:sha(ROOT/source)},'compile_command':command,'removed_outputs':removed,
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_matmul_split_verify_probe.py',rs.name,'matmul_split_verify_compile.txt','matmul_split_verify_run.txt']},
 'qualification':'Exact f32 comparison expression extracted from split-cube verify, without any semantic substitutions. NaN actual output is admitted against finite expected; wrong finite/infinite outputs rejected and finite equal control admitted. Tests predicate only, not a complete verify run, benchmark, DSL lowering, physical GPU NaN occurrence or atomic benchmark repetition.'}
with (HERE/'matmul_split_verify_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
