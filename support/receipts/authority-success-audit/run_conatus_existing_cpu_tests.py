"""Run original CPU tests with rustc against the current Cargo-built dependencies.

This deliberately does not call itself a Cargo/package/GPU gate: optional-resident
integration files are absent and Cargo's unused GPU dev-dependency graph is not built.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
probe=json.loads((HERE/'conatus_cpu_probe_receipt.json').read_text())
assert probe['test_exit']==0
sources=probe['source_sha256']
assert all(sha(ROOT/p)==h for p,h in sources.items())
conatus=Path(probe['library_sha256']['conatus']['path'])
assert sha(conatus)==probe['library_sha256']['conatus']['sha256']
artifacts=[json.loads(line) for line in (HERE/'conatus_cpu_library_build.jsonl').read_text().splitlines() if line.startswith('{')]
libraries={name:next(Path(p) for x in artifacts if x.get('reason')=='compiler-artifact' and x['target']['name']==name for p in x['filenames'] if p.endswith('.rlib')) for name in ['serde','nisus','rapier3d']}
results=[]
for label,source,externs in [
    ('library',ROOT/'crates/conatus/conatus/src/lib.rs',libraries),
    ('integration',HERE/'conatus_existing_cpu_tests.rs',{'conatus':conatus}),
]:
    binary=TARGET/f'authority-success-conatus-{label}-tests.exe'; pdb=binary.with_suffix('.pdb')
    assert binary.parent==TARGET and not binary.exists() and not pdb.exists()
    command=['rustc','--edition=2024','--test','--cfg','feature="default"',str(source),
        '-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
    if label=='library': command+=['--crate-name','conatus']
    for name,path in externs.items(): command+=['--extern',name+'='+str(path)]
    built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=120)
    (HERE/f'conatus_existing_{label}_build.log').write_text(built.stdout+built.stderr,encoding='utf-8')
    assert built.returncode==0,built.stdout+built.stderr
    try:
        ran=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=120)
        (HERE/f'conatus_existing_{label}_tests.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
    finally:
        assert binary.read_bytes().startswith(b'MZ'); binary.unlink()
        if pdb.exists():
            assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF'); pdb.unlink()
    matched=re.search(r'test result: ok\. (\d+) passed; 0 failed',ran.stdout)
    results.append({'kind':label,'command':command,'build_exit':built.returncode,'test_exit':ran.returncode,
        'passed':int(matched[1]) if matched else None,'generated_output_removed':not binary.exists() and not pdb.exists()})
    print(ran.stdout[-600:])
    assert ran.returncode==0 and matched,ran.stderr
assert all(sha(ROOT/p)==h for p,h in sources.items())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'results':results,'source_sha256':sources,
    'library_sha256':{name:{'path':str(p),'sha256':sha(p)} for name,p in {**libraries,'conatus':conatus}.items()},
    'scope':'Original default-feature library tests and four original CPU integration files compiled with rustc. Not a Cargo package/doctest or resident-GPU gate. Unchanged sources; generated executables/PDBs removed.',
    'evidence_sha256':{f'conatus_existing_{label}_{suffix}':sha(HERE/f'conatus_existing_{label}_{suffix}') for label in ['library','integration'] for suffix in ['build.log','tests.log']}}
(HERE/'conatus_existing_cpu_tests_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'passed':sum(r['passed'] for r in results),'results':[{k:r[k] for k in ['kind','passed','test_exit']} for r in results]}))
