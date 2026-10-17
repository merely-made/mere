"""Compile the unchanged whole Gemot library over exact retained suppliers."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
DEBUG = TARGET / 'build/debug'
FP = DEBUG / '.fingerprint'
NAME = 'gemot_owner_probe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE / (NAME+'_receipt.json')).exists()
family = FP / 'gemot-2a2dd68ccc68833a/lib-gemot.json'
deps = {r[1]: r[3] for r in json.loads(family.read_text())['deps']}
names = ['proofs','identity','p2panda_core','p2panda_store','muniment','serde','thiserror','stickleback',
         'mien','mooting','insigne','servitor','redb','p2panda_auth','p2panda_encryption']
prefixes = ('mere-proofs-','personae-','identity-','p2panda-core-','p2panda-store-','muniment-',
            'serde-','thiserror-','stickleback-','mien-','mooting-','insigne-','servitor-','redb-',
            'p2panda-auth-','p2panda-encryption-')
index = []
for folder in FP.iterdir():
    if not folder.is_dir() or not folder.name.startswith(prefixes): continue
    suffix = folder.name.rsplit('-',1)[1]
    for meta in folder.glob('lib-*.json'):
        crate = meta.name[4:-5]; marker = folder / ('lib-'+crate)
        artifact = DEBUG / 'deps' / ('lib'+crate+'-'+suffix+'.rlib')
        if not marker.is_file() or not artifact.is_file(): continue
        encoded = marker.read_text().strip()
        if len(encoded) == 16: index.append((crate,int.from_bytes(bytes.fromhex(encoded),'little'),artifact,meta))
libs = {}
for name in names:
    aliases = {'identity','personae'} if name == 'identity' else {name}
    choices = [r for r in index if r[0] in aliases and r[1] == deps[name]]
    assert len(choices) == 1, (name,deps[name],len(choices))
    crate,value,artifact,meta = choices[0]
    libs[name] = {'path':str(artifact),'sha256':sha(artifact),'fingerprint_value':value,
                  'metadata_path':str(meta),'metadata_sha256':sha(meta)}
inv = json.loads((HERE / 'source_inventory.json').read_text())['files']
sources = [p for p in inv if p.startswith('crates/moot/gemot/')]
source_hashes = {p:sha(ROOT/p) for p in sources}
rows = [json.loads(s) for s in (HERE/'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
native = sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths',[])})
binary = TARGET / 'authority-success-gemot-owner.exe'
library = TARGET / 'libauthority_success_gemot_owner.rlib'
assert not list(TARGET.glob(binary.stem+'.*')) and not library.exists()
common = ['-L','dependency='+str(DEBUG/'deps'),'-C','debuginfo=0']
for p in native: common += ['-L',p]
owner_cmd = ['rustc','--edition=2024','--crate-name','gemot_audit','--crate-type','rlib',str(ROOT/'crates/moot/gemot/src/lib.rs'),*common,'-o',str(library)]
for n,v in libs.items(): owner_cmd += ['--extern',n+'='+v['path']]
consumer_cmd = ['rustc','--edition=2024',str(HERE/(NAME+'.rs')),*common,'-o',str(binary),'--extern','gemot_audit='+str(library)]
for n in ['identity','mooting','proofs','muniment','p2panda_core']: consumer_cmd += ['--extern',n+'='+libs[n]['path']]
env = dict(os.environ); env['CARGO_PKG_VERSION'] = '0.1.0'
owner = consumer = run = None
removed = []
try:
    owner = subprocess.run(owner_cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=180)
    with (HERE/(NAME+'_owner_compile.log')).open('x',encoding='utf-8') as f:f.write(owner.stdout+owner.stderr)
    if owner.returncode == 0:
        consumer = subprocess.run(consumer_cmd,cwd=ROOT,capture_output=True,text=True,timeout=180)
        with (HERE/(NAME+'_consumer_compile.log')).open('x',encoding='utf-8') as f:f.write(consumer.stdout+consumer.stderr)
        if consumer.returncode == 0:
            run = subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=30)
            with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
            print(run.stdout+run.stderr)
        else: print(consumer.stderr)
    else: print(owner.stderr)
finally:
    owned = sorted(TARGET.glob(binary.stem+'.*')) + ([library] if library.exists() else [])
    for p in owned:
        assert p.resolve().parent == TARGET
        data = p.read_bytes()
        assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix in ['.lib','.rlib'] and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o','.exp'] and data.startswith(b'\x64\x86'))
        removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/p)==h for p,h in source_hashes.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks = 0 if run is None or run.returncode else sum(line.startswith('PASS ') for line in run.stdout.splitlines())
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256':source_hashes,'libraries':libs,'family':{'path':str(family),'sha256':sha(family)},
    'owner_command':owner_cmd,'consumer_command':consumer_cmd,'CARGO_PKG_VERSION':'0.1.0',
    'owner_build_exit':owner.returncode,'consumer_build_exit':None if consumer is None else consumer.returncode,
    'run_exit':None if run is None else run.returncode,'checks':checks,'audit_cases':4,'original_tests':0,
    'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'removed_outputs':removed,
    'generated_output_removed':not library.exists() and not list(TARGET.glob(binary.stem+'.*')),
    'qualification':'Unchanged complete current Gemot library freshly compiled over fifteen exact retained suppliers selected from the named Cargo fingerprint family; public consumer freshly compiled. Four sequential CPU/MemoryBackend cases use actual operation signatures, admission, Tulpa projection and aggregate snapshots. No fresh Cargo dependency graph, original package suite, disk, live LogSync, headed app, external transport or training/tensor execution.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['owner_build_exit','consumer_build_exit','run_exit','checks','generated_output_removed']}))
assert owner.returncode==0 and consumer is not None and consumer.returncode==0 and run is not None and run.returncode==0 and checks==4
