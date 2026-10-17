"""Run read-only API probes with owned synthetic directory fixtures, then clean them."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith(('crates/system/errand/','crates/nematic/nematic/','crates/inker/inker/'))}
assert all(sha(ROOT/p)==h for p,h in sources.items())
rows=[json.loads(line) for line in (HERE/'errand_composition_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['errand','nematic','inker','url','tokio','tracing']:
    records=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(p.endswith('.rlib') for p in r['filenames'])]
    assert len(records)==1,(name,len(records))
    record=records[0];path=next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
    if path.parent==TARGET/'debug':
        matches=[q for q in (TARGET/'build/debug/deps').glob('lib'+name+'-*.rlib') if sha(q)==sha(path)]
        assert matches;path=matches[0]
    libs[name]={'path':str(path),'sha256':sha(path),'features':record['features'],'package_id':record['package_id']}
binary=TARGET/'authority-success-errand-probe.exe';pdb=binary.with_suffix('.pdb')
assert not binary.exists() and not pdb.exists()
command=['rustc','--edition=2024','--test',str(HERE/'errand_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,info in libs.items():command+=['--extern',name+'='+info['path']]
native_paths=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native_paths:command+=['-L',p]
compiled=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'errand_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8');assert compiled.returncode==0,compiled.stderr
digest=sha(binary)
fixture=TARGET/'authority-success-errand-fixture';published=fixture/'published';outside=fixture/'outside';link=published/'link'
assert fixture.parent==TARGET and not fixture.exists()
marker=fixture/'AUDIT_OWNED_GENERATED_FIXTURE';marker_bytes=b'errand authority-success synthetic fixture only\n'
try:
    published.mkdir(parents=True);outside.mkdir();marker.write_bytes(marker_bytes)
    (outside/'sentinel.txt').write_bytes(b'audit-owned outside sentinel')
    (published/'.audit-hidden.txt').write_bytes(b'audit-owned hidden fixture')
    made=subprocess.run(['powershell','-NoProfile','-Command',f"New-Item -ItemType Junction -Path '{link}' -Target '{outside}' -ErrorAction Stop | Out-Null"],capture_output=True,text=True)
    assert made.returncode==0,made.stderr
    # Only our exact, resolved target is admitted before the probe or cleanup.
    assert link.resolve()==outside.resolve()
    env={**os.environ,'ERRAND_AUDIT_FIXTURE':str(published)}
    ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,timeout=60)
    (HERE/'errand_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
    assert binary.read_bytes().startswith(b'MZ');binary.unlink()
    if pdb.exists():assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF');pdb.unlink()
    assert fixture.parent==TARGET and fixture.resolve().parent==TARGET and marker.read_bytes()==marker_bytes
    if link.exists():
        assert link.resolve()==outside.resolve()
        # rmdir removes the junction itself on Windows; it does not recurse into the target.
        os.rmdir(link)
    for p,expected in [(outside/'sentinel.txt',b'audit-owned outside sentinel'),(published/'.audit-hidden.txt',b'audit-owned hidden fixture')]:
        assert p.read_bytes()==expected;p.unlink()
    published.rmdir();outside.rmdir();marker.unlink();fixture.rmdir()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/p)==h for p,h in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'tests':10,'test_exit':ran.returncode,
    'source_sha256':sources,'libraries':libs,'native_search_paths':native_paths,'probe_build_command':command,'probe_build_exit':compiled.returncode,
    'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists() and not fixture.exists(),
    'fixture_scope':'Owned files and a Windows junction under the stable Mere Cargo target, marker-checked and individually removed. No user files or recursive deletion.',
    'scope':'Ten current Cargo-built CPU/API observations. One loopback listener is never accepted, exercises timeout, then closes. No remote network, TLS handshake, browser/script, user persistence, GPU or public server.'}
(HERE/'errand_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
