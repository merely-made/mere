"""Compile the actual unchanged Tabard crate root and CPU-only original tests."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='tabard_owner_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
paths=list((ROOT/'crates/system/tabard').rglob('*.rs'))+list((ROOT/'crates/cambium/tinct').rglob('*.rs'))
sources={p.relative_to(ROOT).as_posix():sha(p) for p in paths}
manifest=HERE/'cpu-tabard-runtime/Cargo.toml'
command=['cargo','test','--offline','--manifest-path',str(manifest),'--config',
 'build.build-dir="C:/t/cargo-targets/mere/build"','--lib','--tests','-j','2','--','--test-threads=1','--nocapture']
env=dict(os.environ);env['CARGO_TARGET_DIR']=str(TARGET)
result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=240)
with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
print(result.stdout);print(result.stderr[-2500:])
assert all(sha(ROOT/p)==h for p,h in sources.items())
removed=[]
for stem in ['tabard','audit_contracts','categories','choice_resolution','library','mode_artifacts','portable','typography','workshop']:
    for p in (TARGET/'build/debug/deps').glob(stem+'-*'):
        if p.suffix not in ['.exe','.pdb']:continue
        # Do not remove pre-existing unrelated owners' binaries. These exact
        # executables must be named in this completed Cargo run's own output.
        exe=p if p.suffix=='.exe' else p.with_suffix('.exe')
        if exe.name not in result.stderr:continue
        raw=p.read_bytes();assert (p.suffix=='.exe' and raw.startswith(b'MZ')) or (p.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
        removed.append({'path':str(p),'bytes':len(raw),'sha256':sha(p)});p.unlink()
results=[{'passed':int(a),'failed':int(b),'ignored':int(c)} for a,b,c in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',result.stdout)]
passed=sum(r['passed'] for r in results)
record={'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':result.returncode,'source_sha256':sources,
 'results':results,'passed':passed,'original_tests':max(0,passed-2),'audit_tests':2,'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_tabard_owner_probe.py','tabard_owner_probe.rs','tabard_owner_probe_output.log','cpu-tabard-runtime/Cargo.toml','cpu-tabard-runtime/Cargo.lock']},
 'qualification':'Actual current Tabard lib.rs crate root and complete transitive source modules, actual local Tinct path supplier. Original unit tests and seven selected unchanged pure CPU integration files execute; original theme.rs Genet/Livery integration is intentionally not linked. Two additional public-API cases with controls, no synthetic registry/runtime facade. Owned temporary files are created/cleaned by original tests; no live host settings/user files, browser, font engine realization, Genet CSS/layout/GPU, network or external consumer. Stable shared Cargo target/build-dir, generated test exe/PDB removed only for binaries named by this completed run; unrelated active compilers untouched.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':result.returncode,'passed':passed,'original_tests':record['original_tests'],'removed_outputs':len(removed)}))
assert result.returncode==0 and passed>=2
