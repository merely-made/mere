"""Fresh original-module probe over exact retained successful-build libraries."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='chronicle_mount_probe_v3'; out=HERE/(NAME+'_receipt.json')
assert not out.exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['chirograph','graphshell_client','graphshell_endpoint','graphshell_host','sceno','serde','serde_json']:
    candidates=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name
                and any(f.endswith('.rlib') for f in r['filenames'])]
    if name=='serde':
        candidates=[r for r in candidates if 'rc' in r['features']]
    assert len(candidates)==1,(name,len(candidates))
    row=candidates[0]; path=Path(next(f for f in row['filenames'] if f.endswith('.rlib')))
    libs[name]={'path':str(path),'sha256':sha(path),'package_id':row['package_id'],'features':row['features']}
source_names=['ports/graphshell/src/distillery_w1.rs',
    'crates/graphshell/graphshell-client/src/lib.rs','crates/graphshell/graphshell-host/src/live_endpoint.rs']
sources={n:sha(ROOT/n) for n in source_names}
old=json.loads((HERE/'native_receipt_review_receipt.json').read_text())
known={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/n)==d for n,d in known.items())
binary=TARGET/'authority-success-chronicle-mount-probe.exe'
assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024',str(HERE/(NAME+'.rs')),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,lib in libs.items(): command+=['--extern',name+'='+lib['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for path in native:command+=['-L',path]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=60)
with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(built.stdout+built.stderr)
removed=[];ran=None;binary_sha=None
try:
    if built.returncode==0:
        binary_sha=sha(binary)
        ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=30)
        with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
        print(ran.stdout)
finally:
    for p in sorted(TARGET.glob(binary.stem+'.*')):
        assert p.resolve().parent==TARGET;data=p.read_bytes()
        assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')),str(p)
        removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==d for n,d in sources.items())
assert all(sha(Path(lib['path']))==lib['sha256'] for lib in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,
    'prior_build_log':{'path':'capture_graphshell_build.jsonl','sha256':sha(HERE/'capture_graphshell_build.jsonl')},
    'probe_sha256':sha(HERE/(NAME+'.rs')),'runner_sha256':sha(Path(__file__)),
    'command':command,'native_search_paths':native,'build_exit':built.returncode,
    'run_exit':None if ran is None else ran.returncode,'checks':0 if ran is None else ran.stdout.count('PASS '),
    'binary_sha256':binary_sha,'removed_outputs':removed,
    'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),
    'scope':'Current original distillery_w1.rs compiled fresh by path, not copied or edited, against exact recorded library bytes from an earlier successful Cargo build. Synthetic local endpoint injects wrong-session diff, missing-base diff, invalid snapshot and self-validating unrequested resource. No transitive full-current Cargo gate, actual Distillery process, carrier/network/browser/GPU or original module tests executed.'}
with out.open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'receipt':out.name,'build_exit':built.returncode,'run_exit':receipt['run_exit'],'checks':receipt['checks']}))
assert built.returncode==0,built.stderr
assert ran.returncode==0,ran.stderr
