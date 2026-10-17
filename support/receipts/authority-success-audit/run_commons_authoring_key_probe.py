"""Fresh current whole Commons library and public encrypted-edit consumers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='commons_authoring_key_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
selection=HERE/'commons_supplier_selection.json'
libs=json.loads(selection.read_text())['libraries']
assert all(sha(Path(r['path']))==r['sha256'] and sha(Path(r['fingerprint_path']))==r['fingerprint_sha256'] for r in libs.values())
sources={p.relative_to(ROOT).as_posix():sha(p) for p in (ROOT/'crates/moot/commons/src').rglob('*.rs')}
rows=[json.loads(l) for l in (HERE/'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
library=TARGET/'libauthority_success_commons_spine.rlib'
binary=TARGET/'authority-success-commons-authoring-keys.exe'
assert not library.exists() and not list(TARGET.glob(binary.stem+'.*'))
base=['rustc','--edition=2024','-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for path in native:base+=['-L',path]
build_command=base+['--crate-type=rlib','--crate-name=audit_commons_spine',str(ROOT/'crates/moot/commons/src/lib.rs'),'-o',str(library)]
for n,r in libs.items():build_command+=['--extern',n+'='+r['path']]
test_command=base+['--test',str(HERE/'commons_authoring_key_cases.rs'),'--extern','audit_commons_spine='+str(library),'-o',str(binary)]
for n in ['chartulary','muniment','stickleback']:test_command+=['--extern',n+'='+libs[n]['path']]
build=None;testbuild=None;run=None;removed=[]
try:
    build=subprocess.run(build_command,cwd=ROOT,capture_output=True,text=True,timeout=90)
    with (HERE/(NAME+'_library_compile.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
    if build.returncode==0:
        testbuild=subprocess.run(test_command,cwd=ROOT,capture_output=True,text=True,timeout=60)
        with (HERE/(NAME+'_cases_compile.log')).open('x',encoding='utf-8') as f:f.write(testbuild.stdout+testbuild.stderr)
        if testbuild.returncode==0:
            run=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
            with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
            print(run.stdout+run.stderr)
        else:print(testbuild.stderr)
    else:print(build.stderr)
finally:
    for path in [library,*sorted(TARGET.glob(binary.stem+'.*'))]:
        if not path.exists():continue
        assert path.resolve().parent==TARGET;data=path.read_bytes()
        assert (path.suffix=='.exe' and data.startswith(b'MZ')) or (path.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (path.suffix in ['.lib','.rlib'] and data.startswith(b'!<arch>\n')) or (path.suffix in ['.o','.exp'] and (data.startswith(b'\x64\x86') or (data.startswith(b'\0\0\xff\xff') and data[6:8]==b'\x64\x86')))
        removed.append({'path':str(path),'bytes':len(data),'sha256':sha(path)});path.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items())
checks=0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;',run.stdout)[1])
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':sources,'supplier_selection_sha256':sha(selection),'libraries':libs,'native_link_searches':native,'library_command':build_command,'cases_command':test_command,'build_exit':build.returncode,'cases_build_exit':None if testbuild is None else testbuild.returncode,'run_exit':None if run is None else run.returncode,'checks':checks,'original_tests':0,'audit_cases':3,'runner_sha256':sha(Path(__file__)),'cases_sha256':sha(HERE/'commons_authoring_key_cases.rs'),'removed_outputs':removed,'generated_output_removed':not library.exists() and not list(TARGET.glob(binary.stem+'.*')),'qualification':'Complete current Commons production library built from source over fourteen exact retained fingerprint-selected suppliers. Three consumer cases call public EncryptedReplica and GroupKeys APIs with MemoryBackend: ordinary insertion, same-epoch handle replacement and clearing the handle inside the edit callback. No original Commons suite, fresh supplier/Cargo graph, live group rotation, network, radio or durable disk execution.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['build_exit','cases_build_exit','run_exit','checks','generated_output_removed']}))
assert build.returncode==0 and testbuild is not None and testbuild.returncode==0 and run is not None and run.returncode==0 and checks==3
