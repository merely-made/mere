"""Compile current Forme directly against measured Cargo dependencies during shared lock contention."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith(('crates/forme/forme/','crates/forme/curation/'))}
assert len(sources)==28 and all(sha(ROOT/p)==v for p,v in sources.items())
prior=json.loads((HERE/'mere_facade_review_receipt.json').read_text())
assert all(sha(ROOT/p)==h for p,h in {**prior['source_sha256'],**prior['preserved_production_and_test_hashes']}.items())
records=[json.loads(line) for line in (HERE/'mere_graph_library_build.jsonl').read_text().splitlines() if line.startswith('{')]
deps={}
for name in ['serde','serde_json','uuid','taffy','curation','petgraph']:
    record=next(r for r in records if r.get('reason')=='compiler-artifact' and r['target']['name']==name)
    path=next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
    assert path.exists();deps[name]={'path':str(path),'sha256':sha(path),'package_id':record['package_id'],'features':record['features']}
lib=TARGET/'libauthority_success_forme.rlib';unit=TARGET/'authority-success-forme-unit.exe'
probe=TARGET/'authority-success-forme-probe.exe';curation=TARGET/'authority-success-curation-unit.exe'
outputs=[lib,unit,unit.with_suffix('.pdb'),probe,probe.with_suffix('.pdb'),curation,curation.with_suffix('.pdb')]
assert all(not p.exists() and p.parent==TARGET for p in outputs)
base=['rustc','--edition=2024','-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
extern=[]
for name,info in deps.items():extern+=['--extern',name+'='+info['path']]
cfg=['--cfg','feature="store"','--cfg','feature="petgraph"']
commands=[];runs={}
def execute(name,command):
    commands.append({'name':name,'command':command})
    ran=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=120)
    (HERE/(name+'.log')).write_text(ran.stdout+ran.stderr,encoding='utf-8')
    runs[name]={'exit':ran.returncode,'log':name+'.log'}
    assert ran.returncode==0,ran.stdout+ran.stderr
    return ran.stdout
fixture=HERE/'forme-fixtures';assert not fixture.exists()
try:
    execute('forme_direct_library_build',base+['--crate-name','forme','--crate-type','rlib',str(ROOT/'crates/forme/forme/src/lib.rs')]+extern+cfg+['-C','metadata=authority_success_forme','-o',str(lib)])
    library_hash=sha(lib)
    execute('forme_probe_build',base+['--test',str(HERE/'forme_probe.rs'),'--extern','forme='+str(lib),'-o',str(probe)])
    fixture.mkdir()
    output=execute('forme_probe_output',[str(probe),'--nocapture','--test-threads=1']);print(output)
    assert '6 passed; 0 failed' in output
    assert not list(fixture.iterdir());fixture.rmdir()
    fixtures=[]
    for file in (TARGET/'build/debug/.fingerprint').glob('mere-kernel-*/lib-kernel.json'):
        rlib=TARGET/'build/debug/deps'/('libkernel-'+file.parent.name.removeprefix('mere-kernel-')+'.rlib')
        if 'fixtures' in json.loads(file.read_text()).get('features','') and rlib.exists(): fixtures.append(rlib)
    kernel=max(fixtures,key=lambda p:p.stat().st_mtime_ns)
    execute('forme_direct_unit_build',base+['--crate-name','forme','--test',str(ROOT/'crates/forme/forme/src/lib.rs')]+extern+cfg+['--extern','kernel='+str(kernel),'-o',str(unit)])
    units=execute('forme_direct_unit_output',[str(unit),'--skip','store::tests','--test-threads=1']);print(units[-350:])
    execute('curation_direct_unit_build',base+['--crate-name','curation','--test',str(ROOT/'crates/forme/curation/src/lib.rs')]+extern+['-o',str(curation)])
    curated=execute('curation_direct_unit_output',[str(curation),'--test-threads=1']);print(curated[-300:])
finally:
    for path in outputs:
        if path.exists():
            raw=path.read_bytes()
            assert raw.startswith((b'MZ',b'!<arch>\n',b'Microsoft C/C++ MSF')),path
            path.unlink()
assert all(sha(ROOT/p)==v for p,v in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in deps.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'tests':6,'test_exit':0,
    'source_sha256':sources,'dependencies':deps,'direct_library_sha256':library_hash,
    'kernel_import_dependency':{'path':str(kernel),'sha256':sha(kernel),'scope':'cached fixtures dependency imported by original unit sources; no fresh kernel build claim'},
    'commands':commands,'runs':runs,'generated_output_removed':all(not p.exists() for p in outputs) and not fixture.exists(),
    'scope':'Current Forme Rust library compiled directly with store/petgraph against exact hashed prior Cargo dependency artifacts; six observations, original unit tests excluding four store tests, current curation original unit tests. Cargo package gate failed before compilation on shared target lock; no Cargo success claimed. No product UI, renderer or GPU.'}
(HERE/'forme_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
