"""Compile unchanged native OPFS probe root plus explicit audit cases."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='opfs_owner_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
paths=list((ROOT/'ports/muniment-opfs-probe/src').rglob('*.rs'))+list((ROOT/'crates/eidetic/muniment/src').rglob('*.rs'))
sources={p.relative_to(ROOT).as_posix():sha(p) for p in paths}
sources['ports/muniment-opfs-probe/Cargo.toml']=sha(ROOT/'ports/muniment-opfs-probe/Cargo.toml')
manifest=HERE/'cpu-opfs-runtime/Cargo.toml';lock=manifest.with_name('Cargo.lock');assert not lock.exists()
lock.write_bytes((ROOT/'ports/muniment-opfs-probe/Cargo.lock').read_bytes())
command=['cargo','test','--offline','--manifest-path',str(manifest),'--config','build.build-dir="C:/t/cargo-targets/mere/build"','--lib','--tests','-j','2','--','--test-threads=1','--nocapture']
result=subprocess.run(command,cwd=ROOT,env=dict(os.environ,CARGO_TARGET_DIR=str(TARGET)),capture_output=True,text=True,timeout=600)
with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
print(result.stdout);print(result.stderr[-4500:])
assert all(sha(ROOT/p)==h for p,h in sources.items())
removed=[]
for stem in ['muniment_opfs_probe','opfs_audit_contracts']:
    for p in (TARGET/'build/debug/deps').glob(stem+'-*'):
        if p.suffix not in ['.exe','.pdb']:continue
        exe=p if p.suffix=='.exe' else p.with_suffix('.exe')
        if exe.name not in result.stderr:continue
        raw=p.read_bytes();assert (p.suffix=='.exe' and raw.startswith(b'MZ')) or (p.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
        removed.append({'path':str(p),'bytes':len(raw),'sha256':sha(p)});p.unlink()
results=[{'passed':int(a),'failed':int(b),'ignored':int(c)} for a,b,c in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',result.stdout)];passed=sum(r['passed'] for r in results)
record={'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':result.returncode,'source_sha256':sources,'results':results,'passed':passed,'original_tests':max(0,passed-2),'audit_tests':2,'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_opfs_owner_probe.py','opfs_owner_probe.rs','opfs_owner_probe_output.log','cpu-opfs-runtime/Cargo.toml','cpu-opfs-runtime/Cargo.lock']},
 'qualification':'Actual unchanged OPFS probe lib.rs native root, actual local Muniment and exact redb4.2. Full original native unit suite and two audit CPU cases. Fault backend SharedMemory receives actual bytes; digest anomaly uses an explicitly defective supplied Backend with real MemoryBackend control. No browser, OPFS sync handle, IndexedDB, real worker kill, power loss, promotion, network, native portability fixture or production storage execution. Stable shared target/build-dir; exact completed run exe/PDB outputs removed.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':result.returncode,'passed':passed,'original_tests':record['original_tests'],'removed_outputs':len(removed)}))
assert result.returncode==0 and passed>=2
