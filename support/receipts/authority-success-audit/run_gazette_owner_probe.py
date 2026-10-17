"""Execute current unmodified Gazette crate root on an offline CPU graph."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='gazette_owner_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
old=json.loads((HERE/'design_receipt_review_receipt.json').read_text())
paths=[*old['preserved_production_and_test_hashes'],*old['source_sha256']]
paths+=['ports/gazette/src/lib.rs','ports/gazette/src/intake.rs','ports/gazette/src/intake/tests.rs','ports/gazette/src/ledger.rs','ports/gazette/Cargo.toml','Cargo.lock']
sources={p:sha(ROOT/p) for p in paths}
manifest=HERE/'cpu-gazette-runtime/Cargo.toml'
# Seed the exact current lock rather than silently selecting newer cached Git heads.
lock=manifest.with_name('Cargo.lock');assert not lock.exists();lock.write_bytes((ROOT/'Cargo.lock').read_bytes())
command=['cargo','test','--offline','--manifest-path',str(manifest),'--config','build.build-dir="C:/t/cargo-targets/mere/build"','--lib','--tests','-j','2','--','--test-threads=1','--nocapture']
result=subprocess.run(command,cwd=ROOT,env=dict(os.environ,CARGO_TARGET_DIR=str(TARGET)),capture_output=True,text=True,timeout=600)
with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
print(result.stdout);print(result.stderr[-3500:])
assert all(sha(ROOT/p)==h for p,h in sources.items())
removed=[]
for stem in ['gazette','gazette_audit_contracts']:
    for p in (TARGET/'build/debug/deps').glob(stem+'-*'):
        if p.suffix not in ['.exe','.pdb']:continue
        exe=p if p.suffix=='.exe' else p.with_suffix('.exe')
        if exe.name not in result.stderr:continue
        raw=p.read_bytes();assert (p.suffix=='.exe' and raw.startswith(b'MZ')) or (p.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
        removed.append({'path':str(p),'bytes':len(raw),'sha256':sha(p)});p.unlink()
results=[{'passed':int(a),'failed':int(b),'ignored':int(c)} for a,b,c in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',result.stdout)]
passed=sum(r['passed'] for r in results)
record={'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':result.returncode,'source_sha256':sources,'results':results,'passed':passed,'original_tests':max(0,passed-3),'audit_tests':3,'removed_outputs':removed,'retained_isolation':[],
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_gazette_owner_probe.py','gazette_owner_probe.rs','gazette_owner_probe_output.log','cpu-gazette-runtime/Cargo.toml','cpu-gazette-runtime/Cargo.lock']},
 'qualification':'Actual unchanged current Gazette lib.rs root and transitive local source modules, actual Gaz/Genet path suppliers resolved from current lock seed. Full original unit suite plus three public audit cases; parity integration source read but not run. Original fetch test uses loopback TCP only. No external resolution, live host/contact store, secrets, persona network policy, browser/layout/GPU or product integration. Stable approved shared target/build-dir; only exact completed run executable/PDB outputs removed.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':result.returncode,'passed':passed,'original_tests':record['original_tests'],'removed_outputs':len(removed)}))
assert result.returncode==0 and passed>=3
