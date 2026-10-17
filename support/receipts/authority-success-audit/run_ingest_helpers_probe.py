from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='ingest_helpers_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
extraction=json.loads((HERE/'ingest_helpers_extraction.json').read_text());assert all(sha(ROOT/n)==h for n,h in extraction['source_sha256'].items())
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']=='serde_json' and any(f.endswith('.rlib') for f in r['filenames'])];assert len(matches)==1
row=matches[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')));libs={'serde_json':{'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}}
binary=TARGET/'authority-success-ingest-helpers.exe';assert not list(TARGET.glob(binary.stem+'.*'))
cmd=['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'--extern','serde_json='+str(p),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
build=None;ran=None;removed=[]
try:
 build=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90)
 with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
 if build.returncode==0:
  ran=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
 else:print(build.stderr)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in extraction['source_sha256'].items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if ran is None or ran.returncode else int(re.search(r'test result: ok\. (\d+) passed;',ran.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),**extraction,'libraries':libs,'commands':[cmd],'build_exits':[build.returncode],'run_exit':None if ran is None else ran.returncode,'checks':checks,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*'))}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','run_exit','checks','generated_output_removed']}))
assert build.returncode==0 and ran is not None and ran.returncode==0 and checks==4
