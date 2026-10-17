"""Fresh complete Mien source over eight retained fingerprint-matched suppliers."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve();DEBUG=TARGET/'build/debug'
NAME='mien_owner_probe';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
assert not (HERE/(NAME+'_receipt.json')).exists()
prior=load('gemot_owner_probe_receipt.json');supplier=prior['libraries']['mien']
meta=Path(supplier['metadata_path']);assert sha(meta)==supplier['metadata_sha256']
deps={r[1]:r[3] for r in json.loads(meta.read_text())['deps']}
names=['identity','insigne','p2panda_core','p2panda_store','muniment','serde','thiserror','stickleback']
libs={n:prior['libraries'][n] for n in names}
assert all(v['fingerprint_value']==deps[n] and sha(Path(v['path']))==v['sha256'] for n,v in libs.items())
files=[p for p in load('source_inventory.json')['files'] if p.startswith('crates/moot/mien/')]
sources={p:sha(ROOT/p) for p in files};assert len(files)==11
native=[]
rows=[json.loads(s) for s in (HERE/'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
binary=TARGET/'authority-success-mien-owner.exe';library=TARGET/'libauthority_success_mien_owner.rlib'
assert not library.exists() and not list(TARGET.glob(binary.stem+'.*'))
common=['-L','dependency='+str(DEBUG/'deps'),'-C','debuginfo=0']
for p in native:common+=['-L',p]
owner_cmd=['rustc','--edition=2024','--crate-name','mien_audit','--crate-type','rlib',str(ROOT/'crates/moot/mien/src/lib.rs'),*common,'-o',str(library)]
for n,v in libs.items():owner_cmd+=['--extern',n+'='+v['path']]
consumer_cmd=['rustc','--edition=2024',str(HERE/(NAME+'.rs')),*common,'-o',str(binary),'--extern','mien_audit='+str(library)]
for n in ['identity','muniment']:consumer_cmd+=['--extern',n+'='+libs[n]['path']]
owner=consumer=run=None;removed=[]
try:
    owner=subprocess.run(owner_cmd,cwd=ROOT,capture_output=True,text=True,timeout=180)
    with (HERE/(NAME+'_owner_compile.log')).open('x',encoding='utf-8') as f:f.write(owner.stdout+owner.stderr)
    if owner.returncode==0:
        consumer=subprocess.run(consumer_cmd,cwd=ROOT,capture_output=True,text=True,timeout=180)
        with (HERE/(NAME+'_consumer_compile.log')).open('x',encoding='utf-8') as f:f.write(consumer.stdout+consumer.stderr)
        if consumer.returncode==0:
            run=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=30)
            with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
            print(run.stdout+run.stderr)
        else:print(consumer.stderr)
    else:print(owner.stderr)
finally:
    owned=sorted(TARGET.glob(binary.stem+'.*'))+([library] if library.exists() else [])
    for p in owned:
        assert p.resolve().parent==TARGET;data=p.read_bytes()
        assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix in ['.lib','.rlib'] and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o','.exp'] and data.startswith(b'\x64\x86'))
        removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/p)==h for p,h in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if run is None or run.returncode else sum(s.startswith('PASS ') for s in run.stdout.splitlines())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256':sources,'libraries':libs,'supplier_metadata':{'path':str(meta),'sha256':sha(meta)},
    'owner_command':owner_cmd,'consumer_command':consumer_cmd,'owner_build_exit':owner.returncode,
    'consumer_build_exit':None if consumer is None else consumer.returncode,'run_exit':None if run is None else run.returncode,
    'checks':checks,'audit_cases':6,'original_tests':0,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),
    'removed_outputs':removed,'generated_output_removed':not library.exists() and not list(TARGET.glob(binary.stem+'.*')),
    'qualification':'Unchanged complete current Mien production library and public consumer freshly compiled over eight exact retained suppliers matched to the retained Mien Cargo dependency fingerprint. Six sequential CPU/MemoryBackend diagnostic cases use actual signed/attested operations, common admission, store projection and ledger arithmetic. No fresh Cargo graph, original suite, disk, live LogSync, actual external governance role, hosted product or physical peer execution. Shared-store cross-Moot case uses raw author_seed; Moot-scoped derived keys and one physical store per aggregate are distinct isolation controls.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['owner_build_exit','consumer_build_exit','run_exit','checks','generated_output_removed']}))
assert owner.returncode==0 and consumer is not None and consumer.returncode==0 and run is not None and run.returncode==0 and checks==6
