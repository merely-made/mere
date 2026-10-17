from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='decoder_text_math_probe_v2'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
manifest=json.loads((HERE/'decoder_text_math_source.json').read_text());assert all(sha(ROOT/n)==h for n,h in manifest['source_sha256'].items())
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')];libs={}
for name in ['serde','serde_json','zeroize']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 if name=='serde':matches=[r for r in matches if 'rc' in r['features']]
 assert len(matches)==1,(name,len(matches))
 r=matches[0];p=Path(next(f for f in r['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':r['features'],'package_id':r['package_id']}
token=TARGET/'build/debug/deps/libtokenizers-4cf55218e93967d1.rlib'
fingerprint=TARGET/'build/debug/.fingerprint/tokenizers-4cf55218e93967d1/lib-tokenizers.json'
dep=TARGET/'build/debug/deps/tokenizers-4cf55218e93967d1.d'
assert 'tokenizers-0.20.4/' in dep.read_text().replace('\\','/')
libs['tokenizers']={'path':str(token),'sha256':sha(token),'version':'0.20.4','fingerprint':str(fingerprint),'fingerprint_sha256':sha(fingerprint),'dep_info_sha256':sha(dep),'features':json.loads(fingerprint.read_text())['features']}
binary=TARGET/'authority-success-decoder-text-math-v2.exe';assert not list(TARGET.glob(binary.stem+'.*'))
cmd=['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for n,v in libs.items():cmd+=['--extern',n+'='+v['path']]
for p in sorted((TARGET/'build/debug/build').glob('onig_sys-*/out')):cmd+=['-L','native='+str(p)]
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
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.lib' and data.startswith(b'!<arch>\n')) or (p.suffix=='.exp' and data.startswith(b'd\x86'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in manifest['source_sha256'].items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if ran is None or ran.returncode else int(re.search(r'test result: ok\. (\d+) passed;',ran.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),**manifest,'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'libraries':libs,'command':cmd,'build_exit':build.returncode,'run_exit':None if ran is None else ran.returncode,'checks':checks,'audit_cases':6,'original_tests':checks-6 if checks else None,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*'))}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['build_exit','run_exit','checks','original_tests','generated_output_removed']}))
assert build.returncode==0 and ran is not None and ran.returncode==0 and checks>=6
