"""Preserve the failed external-path harness; compile byte-identical source tree."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, shutil, subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]; TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='registry_owner_probe_v2'; sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
failure=json.loads((HERE/'registry_owner_probe_receipt.json').read_text())
assert failure['exit']==101 and failure['passed']==0
assert all(sha(HERE/p)==h for p,h in failure['evidence_sha256'].items())
tree=HERE/'registry-source-composition'; assert not tree.exists(); tree.mkdir()
original=ROOT/'crates/system/registry/src'; copied={}
for p in original.rglob('*.rs'):
    relative=p.relative_to(original); dest=tree/relative; dest.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(p,dest); assert sha(p)==sha(dest)
    copied[relative.as_posix()]={'source':p.relative_to(ROOT).as_posix(),'sha256':sha(p)}
harness=(HERE/'registry_owner_probe.rs').read_text()
harness=re.sub(r'#\[path = "\.\./\.\./\.\./crates/system/registry/src/[^\"]+"\]\n','',harness)
assert '#[path' not in harness
(tree/'lib_probe.rs').write_text(harness,encoding='utf-8')
shutil.copyfile(HERE/'registry_viewer_capabilities_projection.rs',tree/'registry_viewer_capabilities_projection.rs')
manifest_dir=HERE/'cpu-registry-source-runtime'; assert not manifest_dir.exists(); manifest_dir.mkdir()
manifest=manifest_dir/'Cargo.toml'
manifest.write_text((HERE/'cpu-registry-runtime/Cargo.toml').read_text().replace('path = "../registry_owner_probe.rs"','path = "../registry-source-composition/lib_probe.rs"'),encoding='utf-8')
shutil.copyfile(HERE/'cpu-registry-runtime/Cargo.lock',manifest.with_name('Cargo.lock'))
command=['cargo','test','--offline','--manifest-path',str(manifest),'--config',
 'build.build-dir="C:/t/cargo-targets/mere/build"','--lib','-j','2','--','--test-threads=1','--nocapture']
env=dict(os.environ); env['CARGO_TARGET_DIR']=str(TARGET)
result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=240)
with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
print(result.stdout); print(result.stderr[-3500:])
assert all(sha(ROOT/v['source'])==v['sha256']==sha(tree/p) for p,v in copied.items())
removed=[]
for p in (TARGET/'build/debug/deps').glob('authority_success_registry-*'):
    if p.suffix not in ['.exe','.pdb']:continue
    raw=p.read_bytes()
    assert (p.suffix=='.exe' and raw.startswith(b'MZ')) or (p.suffix=='.pdb' and raw.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(raw),'sha256':sha(p)}); p.unlink()
m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',result.stdout)
passed=int(m.group(1)) if m else 0; ignored=int(m.group(3)) if m else 0
evidence=['run_registry_owner_probe_v2.py','registry_owner_probe_v2_output.log',
 'cpu-registry-source-runtime/Cargo.toml','cpu-registry-source-runtime/Cargo.lock']
record={'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':result.returncode,
 'passed':passed,'ignored':ignored,'audit_tests':5,'original_tests':max(0,passed-5),'copied_source':copied,
 'copied_tree_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in tree.rglob('*') if p.is_file()},
 'failed_initial_receipt':{'path':'registry_owner_probe_receipt.json','sha256':sha(HERE/'registry_owner_probe_receipt.json')},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'removed_outputs':removed,'retained_isolation':[],
 'qualification':'Whole byte-identical Registry source tree preserved; only diagnostics/input/layout/mod_loader modules linked via an audit root. Original unit tests for those modules execute, actual command-menu path dependency. Exact data-only ViewerSubsystemCapabilities projection supplies the extension-record field, without viewer selection/graph-kernel execution. Five audit cases exercise actual public owners with synthetic documented activation/removal callbacks. No native mod/WASM instance, product host, network, browser or GPU. First external-path harness compiler failure preserved, not counted as tests. Stable shared target/build-dir explicitly configured; audit executable/PDB removed.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':result.returncode,'passed':passed,'ignored':ignored,'removed_outputs':len(removed)}))
assert result.returncode==0 and passed>=5
