"""Scoped current Registry owners, keeping failed runs and exact source basis."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve(); NAME='registry_owner_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
files=list((ROOT/'crates/system/registry').rglob('*.rs'))
sources={p.relative_to(ROOT).as_posix():sha(p) for p in files}
viewer=(ROOT/'crates/system/registry/src/viewer.rs').read_text()
start=viewer.index('#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]\npub struct ViewerSubsystemCapabilities')
end=viewer.index('#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]\npub struct ViewerDescriptor',start)
projection=viewer[start:end]
with (HERE/'registry_viewer_capabilities_projection.rs').open('x',encoding='utf-8') as f:f.write(projection)
manifest=HERE/'cpu-registry-runtime/Cargo.toml'
command=['cargo','test','--offline','--manifest-path',str(manifest),
 '--config','build.build-dir="C:/t/cargo-targets/mere/build"','--lib','-j','2','--','--test-threads=1','--nocapture']
env=dict(os.environ); env['CARGO_TARGET_DIR']=str(TARGET)
assert not any(k in env for k in ['GRAPHSHELL_DISABLE_MODS','GRAPHSHELL_DISABLE_VERSO'])
result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=240)
with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
print(result.stdout); print(result.stderr[-3500:])
assert all(sha(ROOT/p)==h for p,h in sources.items())
removed=[]
for p in (TARGET/'build/debug/deps').glob('authority_success_registry-*'):
    if p.suffix not in ['.exe','.pdb']:continue
    raw=p.read_bytes()
    assert (p.suffix=='.exe' and raw.startswith(b'MZ')) or (p.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(raw),'sha256':sha(p)}); p.unlink()
m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',result.stdout)
passed=int(m.group(1)) if m else 0
ignored=int(m.group(3)) if m else 0
record={'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':result.returncode,
 'source_sha256':sources,'passed':passed,'ignored':ignored,'audit_tests':5,'original_tests':max(0,passed-5),
 'projection':{'path':'registry_viewer_capabilities_projection.rs','sha256':sha(HERE/'registry_viewer_capabilities_projection.rs'),
 'source':'crates/system/registry/src/viewer.rs','normalized_text_start':start,'normalized_text_end':end,'exact':True},
 'evidence_sha256':{p:sha(HERE/p) for p in ['registry_owner_probe.rs','run_registry_owner_probe.py','registry_owner_probe_output.log','cpu-registry-runtime/Cargo.toml','cpu-registry-runtime/Cargo.lock']},
 'removed_outputs':removed,'retained_isolation':[],
 'qualification':'Whole unchanged diagnostics/input/layout/mod_loader source modules with their original unit tests and actual command-menu path dependency. Exact data-only ViewerSubsystemCapabilities projection enables the extension record type, without any viewer routing or graph-kernel facade. Five audit tests use real public owner methods and documented injected activation/removal callbacks with explicitly synthetic effects/failures. No native mod/WASM instantiation, product startup, runtime host wiring, network, browser, GPU or user data. The absent native runtime/global capability case has no activation hook at all. Stable shared target/build-dir explicit; generated executable and PDB removed.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':result.returncode,'passed':passed,'ignored':ignored,'removed_outputs':len(removed)}))
assert result.returncode==0 and passed>=5
