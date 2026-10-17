from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='search_boundaries_probe_v2'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
libs={}
for name in ['serde','serde_json','thiserror','eidetic','genet_text']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 if name=='serde':matches=[r for r in matches if 'rc' in r['features']]
 if name=='thiserror':matches=[r for r in matches if r['package_id'].endswith('#thiserror@1.0.69')]
 assert len(matches)==1,(name,len(matches))
 r=matches[0];p=Path(next(f for f in r['filenames'] if f.endswith('.rlib')))
 libs[name]={'path':str(p),'sha256':sha(p),'features':r['features'],'package_id':r['package_id']}
inv=json.loads((HERE/'source_inventory.json').read_text())['files'];sources={n:v['sha256'] for n,v in inv.items() if n.startswith('crates/intel/eidetic-search/')}
assert len(sources)==11 and all(sha(ROOT/n)==h for n,h in sources.items())
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
base=['rustc','--edition=2024','-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for p in native:base+=['-L',p]
extern=[]
for n,v in libs.items():extern+=['--extern',n+'='+v['path']]
lib=TARGET/'libauthority_success_eidetic_search.rlib';binary=TARGET/'authority-success-search-boundaries.exe';fixture=TARGET/'authority-success-trail-fixture'
assert not lib.exists() and not list(TARGET.glob(binary.stem+'.*')) and not fixture.exists()
env=os.environ.copy();env['CARGO_PKG_VERSION']='0.0.1';env['MERE_AUDIT_SEARCH_FIXTURE']=str(fixture)
commands=[base+['--crate-name','eidetic_search','--crate-type','rlib',str(ROOT/'crates/intel/eidetic-search/src/lib.rs')]+extern+['-C','metadata=authority_success_search','-o',str(lib)],base+['--test',str(HERE/(NAME+'.rs'))]+extern+['--extern','eidetic_search='+str(lib),'-o',str(binary)]]
builds=[];ran=None;removed=[];fixture_files={}
try:
 for i,cmd in enumerate(commands):
  build=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=120)
  with (HERE/(NAME+'_compile_'+str(i)+'.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
  builds.append(build.returncode)
  if build.returncode:print(build.stderr);break
 if builds==[0,0]:
  fixture.mkdir();(fixture/'.audit-generated').write_text('mere authority audit disposable search fixtures\n',encoding='utf-8')
  ran=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,timeout=45)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
finally:
 if fixture.exists():
  assert fixture.resolve().parent==TARGET and fixture.name=='authority-success-trail-fixture'
  assert (fixture/'.audit-generated').read_text()=='mere authority audit disposable search fixtures\n'
  allowed={'.audit-generated',*(d+'/'+n for d in ['stem','replacement','transient'] for n in ['trail-projection.json','mere-search-spec.json','unrelated-marker.txt'])}
  files=[p for p in fixture.rglob('*') if p.is_file()]
  assert all(p.relative_to(fixture).as_posix() in allowed and p.stat().st_size<20000 for p in files)
  fixture_files={p.relative_to(fixture).as_posix():{'bytes':p.stat().st_size,'sha256':sha(p)} for p in files}
  for p in files:p.unlink()
  for p in sorted([p for p in fixture.rglob('*') if p.is_dir()],key=lambda p:len(p.parts),reverse=True):p.rmdir()
  fixture.rmdir()
 for p in [lib,*sorted(TARGET.glob(binary.stem+'.*'))]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p==lib and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if ran is None or ran.returncode else int(re.search(r'test result: ok\. (\d+) passed;',ran.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'native_search_paths':native,'build_exits':builds,'run_exit':None if ran is None else ran.returncode,'checks':checks,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'fixture_files_before_cleanup':fixture_files,'removed_outputs':removed,'generated_output_removed':not fixture.exists() and not lib.exists() and not list(TARGET.glob(binary.stem+'.*')),'qualification':'Fresh current entire eidetic-search production library over exact retained Eidetic/Genet text/serde/JSON/thiserror suppliers. Five public-API integration observations plus nine original BM25/fusion module tests. No full Cargo dev/example graph, ESP/embedding, live corpus or product acceptance; destructive-path observation only touches marker-identified disposable fixtures under the approved reusable target.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','run_exit','checks','generated_output_removed']}))
assert builds==[0,0] and ran is not None and ran.returncode==0 and checks==14
