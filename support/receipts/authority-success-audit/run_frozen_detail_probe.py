"""Fresh complete current graphshell-client library; exact retained suppliers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,re
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='frozen_detail_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
source=ROOT/'crates/graphshell/graphshell-client/src/lib.rs'
sources={p.relative_to(ROOT).as_posix():sha(p) for p in source.parent.rglob('*.rs')}
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')];libs={}
for name in ['chirograph','sceno','scenotime','uxtree','accesskit','serde','serde_json']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 if name=='serde':matches=[r for r in matches if 'rc' in r['features']]
 assert len(matches)==1,(name,len(matches))
 r=matches[0];p=Path(next(f for f in r['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':r['features'],'package_id':r['package_id']}
library=TARGET/'libauthority_success_graphshell_facts.rlib';binary=TARGET/'authority-success-frozen-detail.exe'
assert not library.exists() and not list(TARGET.glob(binary.stem+'.*'))
base=['rustc','--edition=2024','-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
build_cmd=base+['--crate-type=rlib','--crate-name=audit_graphshell_client','--cfg','feature="accesskit"',str(source),'-o',str(library)]
for n,v in libs.items():build_cmd+=['--extern',n+'='+v['path']]
test_cmd=base+['--test',str(HERE/'frozen_detail_cases.rs'),'--extern','audit_graphshell_client='+str(library),'--extern','sceno='+libs['sceno']['path'],'--extern','scenotime='+libs['scenotime']['path'],'-o',str(binary)]
build=None;testbuild=None;run=None;removed=[]
try:
 build=subprocess.run(build_cmd,cwd=ROOT,capture_output=True,text=True,timeout=90)
 with (HERE/(NAME+'_library_compile.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
 if build.returncode==0:
  testbuild=subprocess.run(test_cmd,cwd=ROOT,capture_output=True,text=True,timeout=60)
  with (HERE/(NAME+'_cases_compile.log')).open('x',encoding='utf-8') as f:f.write(testbuild.stdout+testbuild.stderr)
  if testbuild.returncode==0:
   run=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
   with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
   print(run.stdout+run.stderr)
  else:print(testbuild.stderr)
 else:print(build.stderr)
finally:
 for p in [library,*sorted(TARGET.glob(binary.stem+'.*'))]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix in ['.rlib','.lib'] and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.exp' and data.startswith(b'd\x86'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;',run.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':sources,'libraries':libs,'library_command':build_cmd,'cases_command':test_cmd,'build_exit':build.returncode,'cases_build_exit':None if testbuild is None else testbuild.returncode,'run_exit':None if run is None else run.returncode,'checks':checks,'original_tests':0,'audit_cases':3,'runner_sha256':sha(Path(__file__)),'cases_sha256':sha(HERE/'frozen_detail_cases.rs'),'removed_outputs':removed,'generated_output_removed':not library.exists() and not list(TARGET.glob(binary.stem+'.*')),'qualification':'Complete current graphshell-client production library compiled from source with accesskit only, over7exact retained suppliers. Three public API cases use validated real scene snapshots. No original library unit suite, fresh dependency/Cargo graph, Tokio pump, DOM/OS screen reader, renderer or native/GPU run.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['build_exit','cases_build_exit','run_exit','checks','generated_output_removed']}))
assert build.returncode==0 and testbuild is not None and testbuild.returncode==0 and run is not None and run.returncode==0 and checks==3
