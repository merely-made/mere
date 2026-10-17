"""Current-artifact public fetch probes; no production edits or user state."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith('crates/system/fetch/')}
assert len(sources)==11 and all(sha(ROOT/p)==h for p,h in sources.items())
rows=[json.loads(line) for line in (HERE/'fetch_library_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['fetch','netfetcher','pandect','uuid','muniment','eidetic','tokio','url']:
    records=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(p.endswith('.rlib') for p in r['filenames'])]
    assert len(records)==1,(name,len(records))
    record=records[0];path=next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
    if path.parent==TARGET/'debug':
        matches=[q for q in (TARGET/'build/debug/deps').glob('lib'+name+'-*.rlib') if sha(q)==sha(path)]
        assert matches;path=matches[0]
    libs[name]={'path':str(path),'sha256':sha(path),'features':record['features'],'package_id':record['package_id'],'target_source':record['target']['src_path']}
binary=TARGET/'authority-success-fetch-probe.exe';pdb=binary.with_suffix('.pdb')
assert not binary.exists() and not pdb.exists()
command=['rustc','--edition=2024','--test',str(HERE/'fetch_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,info in libs.items():command+=['--extern',name+'='+info['path']]
native_paths=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native_paths:command+=['-L',p]
compiled=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'fetch_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8');assert compiled.returncode==0,compiled.stderr
digest=sha(binary)
try:
    ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=60)
    (HERE/'fetch_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
    assert binary.read_bytes().startswith(b'MZ');binary.unlink()
    if pdb.exists():assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF');pdb.unlink()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/p)==h for p,h in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'tests':11,'test_exit':ran.returncode,
    'source_sha256':sources,'libraries':libs,'native_search_paths':native_paths,'probe_build_command':command,'probe_build_exit':compiled.returncode,
    'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Eleven public API observations using current all-feature Cargo library. Synthetic cookies in process-global jar and MemoryBackend only. Host-only flip loss is an export collision, not an executed WebView import. Owned HTTP/Gopher/Spartan loopback listeners and joined fixture threads. Titan identity fails before connection and dummy key bytes never enter TLS. Header-capacity panic caught without huge allocation. No remote user data, real credentials, public server, browser, GPU or durable user files.'}
(HERE/'fetch_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
