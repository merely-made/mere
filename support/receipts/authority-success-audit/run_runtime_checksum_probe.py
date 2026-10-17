"""Show candidate-name checksum preimage collisions without replacing its hasher."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
path='support/patches/cubecl-runtime/src/tune/operation.rs'
s=(ROOT/path).read_text(encoding='utf-8-sig');a=s.index('    pub fn compute_checksum(')
body=s[s.index('        let mut checksum',a):s.index('        format!',a)]
code=r'''#![allow(dead_code)]
struct Function{name:String}struct Candidate{function:Function}struct Set{tunables:Vec<Candidate>}
impl Set{fn checksum_preimage(&self)->String{
'''+body+r'''
checksum
}}
fn names(values:&[&str])->Set{Set{tunables:values.iter().map(|name|Candidate{function:Function{name:(*name).into()}}).collect()}}
#[test]fn renamed_candidates_with_same_count_share_the_hash_input(){
 let a=names(&["ab","c"]).checksum_preimage();let b=names(&["a","bc"]).checksum_preimage();assert_eq!(a,"abc");assert_eq!(a,b);
}
#[test]fn_a_changed_candidate_count_shares_the_hash_input(){
 let a=names(&["abc"]).checksum_preimage();let b=names(&["ab","c"]).checksum_preimage();assert_eq!(a,b);
}
#[test]fn_changed_concatenation_control_is_distinct(){assert_ne!(names(&["ab","c"]).checksum_preimage(),names(&["a","bd"]).checksum_preimage());}
'''
# Keep test names ordinary Rust functions; the body under review is unchanged.
code=code.replace('#[test]fn_a_changed','#[test]fn a_changed').replace('#[test]fn_changed','#[test]fn changed')
rs=HERE/'runtime_checksum_probe.rs';assert not rs.exists();rs.write_text(code,encoding='utf-8')
exe=Path('C:/t/cargo-targets/mere/audit-runtime-checksum.exe');pdb=exe.with_suffix('.pdb');assert not exe.exists() and not pdb.exists()
command=['rustc','--test','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]
r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
(HERE/'runtime_checksum_compile.txt').write_text(r.stdout+r.stderr,encoding='utf-8');assert r.returncode==0,r.stderr
r=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=ROOT,text=True,capture_output=True,timeout=20)
(HERE/'runtime_checksum_run.txt').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr)
assert r.returncode==0 and '3 passed; 0 failed' in r.stdout
removed=[]
for p in [exe,pdb]:
 if not p.exists():continue
 assert p.resolve().parent==exe.parent.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':3,'source_sha256':{path:sha(ROOT/path)},'extracted_accumulation_prefix_sha256':hashlib.sha256(body.encode()).hexdigest(),'compile_command':command,'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_runtime_checksum_probe.py',rs.name,'runtime_checksum_compile.txt','runtime_checksum_run.txt']},
 'qualification':'Exact compute_checksum name-accumulation prefix before StableHasher call, compiled with minimal candidate/function/set structs. Three CPU cases: same-count rename and changed-count name lists produce identical String preimage, a distinct concatenation control differs. No replacement hasher: equal input necessarily produces same deterministic StableHasher result, but that crate was not built/run. No actual persistent cache load, stale index execution, tuner or GPU run. Owned exe/PDB removed.'}
with (HERE/'runtime_checksum_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':3,'removed_outputs':len(removed)}))
