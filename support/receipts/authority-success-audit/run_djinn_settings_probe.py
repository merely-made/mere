"""Current public Djinn settings interleavings; bounded synthetic fixture only."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:sha(ROOT/p) for p in inventory if p.startswith('ports/djinn/') or p in ['ports/distillery/src/inference_protocol.rs','ports/distillery/src/inference_protocol/recall.rs']}
sources.update({p:sha(ROOT/p) for p in ['ports/graphshell/src/personal_sync.rs','ports/graphshell/src/transfer_offer.rs','ports/graphshell/src/receipts/mod.rs']})
rows=[json.loads(line) for line in (HERE/'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['djinn','personae','graphshell','muniment','uuid','tokio']:
 records=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(p.endswith('.rlib') for p in r['filenames'])]
 assert len(records)==1,(name,len(records))
 record=records[0];path=next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
 if path.parent==TARGET/'debug':
  matches=[q for q in (TARGET/'build/debug/deps').glob('lib'+name+'-*.rlib') if sha(q)==sha(path)]
  assert matches;path=matches[0]
 libs[name]={'path':str(path),'sha256':sha(path),'features':record['features'],'package_id':record['package_id'],'target_source':record['target']['src_path']}
binary=TARGET/'authority-success-djinn-settings-probe.exe';pdb=binary.with_suffix('.pdb')
assert not binary.exists() and not pdb.exists()
cmd=['rustc','--edition=2024','--test',str(HERE/'djinn_settings_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for n,v in libs.items():cmd+=['--extern',n+'='+v['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native:cmd+=['-L',p]
built=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True)
(HERE/'djinn_settings_probe_build.log').write_text(built.stdout+built.stderr,encoding='utf-8');assert built.returncode==0,built.stderr
fixture=TARGET/'authority-success-settings-fixture'
assert fixture.parent==TARGET and not fixture.exists()
fixture.mkdir();(fixture/'AUDIT-FIXTURE-MARKER').write_text('authority-success audit generated settings fixture\n')
digest=sha(binary)
try:
 ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=60)
 (HERE/'djinn_settings_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
 assert binary.read_bytes().startswith(b'MZ');binary.unlink()
 if pdb.exists():assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF');pdb.unlink()
assert (fixture/'AUDIT-FIXTURE-MARKER').read_text()=='authority-success audit generated settings fixture\n'
assert sorted(f.name for f in fixture.iterdir())==['AUDIT-FIXTURE-MARKER','settings']
settings_dir=fixture/'settings'
assert sorted(f.name for f in settings_dir.iterdir())==['added.json','removed.json']
fixture_sha={str(f.relative_to(fixture)):sha(f) for f in settings_dir.iterdir()}
for f in settings_dir.iterdir():
 assert 'authority-success-synthetic-graph' in f.read_text();f.unlink()
settings_dir.rmdir();(fixture/'AUDIT-FIXTURE-MARKER').unlink();fixture.rmdir()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/p)==h for p,h in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'fixture_sha256':fixture_sha,'fixture_removed':not fixture.exists(),'tests':3,'test_exit':ran.returncode,'source_sha256':sources,'libraries':libs,'native_search_paths':native,
 'probe_build_command':cmd,'probe_build_exit':built.returncode,'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists(),
 'scope':'Two explicit public settings read/write interleavings characterize the production resident hint read-modify-save critical section against real pair_device/unpair_device calls. A third public in-memory replica observation composes the exact host synthetic-selector builder chain and observes a previously filtered carrier. Synthetic app root/public keys/in-memory graph only. No full host open, daemon timing race, real peer connection, vault or user settings operation executed. Passing observations are not repairs.'}
(HERE/'djinn_settings_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
