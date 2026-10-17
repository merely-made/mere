"""Scoped current Reciprocity and MootId source, including their original tests."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='moothold_reciprocity_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
libs=json.loads((HERE/'gemot_owner_probe_receipt.json').read_text())['libraries'];serde=libs['serde']
assert sha(Path(serde['path']))==serde['sha256']
sources={p:sha(ROOT/p) for p in ['crates/moot/moothold/src/reciprocity.rs','crates/moot/gemot/src/moot/id.rs']}
binary=TARGET/'authority-success-moothold-reciprocity.exe'
assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','--extern','serde='+serde['path'],'-o',str(binary)]
build=run=None;removed=[]
try:
    build=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=60)
    with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
    if build.returncode==0:
        run=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
        with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
        print(run.stdout+run.stderr)
    else:print(build.stderr)
finally:
    for p in sorted(TARGET.glob(binary.stem+'.*')):
        assert p.resolve().parent==TARGET;data=p.read_bytes()
        assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.lib' and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o','.exp'] and data.startswith(b'\x64\x86'))
        removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/p)==h for p,h in sources.items())
checks=0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;',run.stdout)[1])
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256':sources,'supplier':serde,'command':command,'build_exit':build.returncode,'run_exit':None if run is None else run.returncode,
    'checks':checks,'original_tests':5,'audit_cases':3,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),
    'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),
    'qualification':'Two unchanged complete current modules (Reciprocity and actual Gemot MootId) compiled with minimal Gemot namespace scaffolding and one exact retained serde supplier. Five original module tests and three audit cases execute CPU/in-memory arithmetic only. Not the complete Moothold/Gemot owner, current Cargo graph, durable credit ledger, real request router, network or resource-service execution.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['build_exit','run_exit','checks','generated_output_removed']}))
assert build.returncode==0 and run is not None and run.returncode==0 and checks==8
