"""Compile the unchanged current Parking module over exact retained suppliers."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='commons_parking_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
source='crates/moot/commons/src/parking.rs'
source_hash=sha(ROOT/source)
rows=[json.loads(line) for line in (HERE/'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
libs={}
for name in ['muniment','stickleback','p2panda_core','serde','async_trait']:
    suffix='.dll' if name=='async_trait' else '.rlib'
    matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith(suffix) for f in r['filenames'])]
    if name=='serde':matches=[r for r in matches if 'rc' in r['features']]
    assert len(matches)==1,(name,len(matches))
    row=matches[0];p=Path(next(f for f in row['filenames'] if f.endswith(suffix)))
    libs[name]={'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}
binary=TARGET/'authority-success-commons-parking.exe'
assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024','--test',str(HERE/'commons_parking_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,value in libs.items():command+=['--extern',name+'='+value['path']]
build=None;run=None;removed=[]
try:
    build=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=60)
    with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
    if build.returncode==0:
        run=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
        with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
        print(run.stdout+run.stderr)
    else:print(build.stderr)
finally:
    for path in sorted(TARGET.glob(binary.stem+'.*')):
        assert path.resolve().parent==TARGET;data=path.read_bytes()
        assert (path.suffix=='.exe' and data.startswith(b'MZ')) or (path.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (path.suffix=='.lib' and data.startswith(b'!<arch>\n')) or (path.suffix=='.exp' and data.startswith(b'd\x86'))
        removed.append({'path':str(path),'bytes':len(data),'sha256':sha(path)});path.unlink()
assert sha(ROOT/source)==source_hash and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;',run.stdout)[1])
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':{source:source_hash},'libraries':libs,'command':command,'build_exit':build.returncode,'run_exit':None if run is None else run.returncode,'checks':checks,'original_tests':0,'audit_cases':3,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/'commons_parking_probe.rs'),'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'qualification':'Unchanged complete current private Parking module compiled over five exact retained suppliers. Three audit cases use real MemoryBackend storage; wrapper only schedules two valid list snapshots before writes and delegates every write batch unchanged. Sequential control and deterministic two-writer count/byte-bound counterexamples. No whole Commons/Chat/EncryptedReplica, fresh Cargo graph, live LogSync, physical carrier or disk execution.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['build_exit','run_exit','checks','generated_output_removed']}))
assert build.returncode==0 and run is not None and run.returncode==0 and checks==3
