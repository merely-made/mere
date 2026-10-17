"""Run original Backstep tests and two public-API state counterexamples, std-only."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='backstep_state_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source=ROOT/'crates/cambium/backstep/src/lib.rs';fixture=HERE/(NAME+'.rs');binary=TARGET/'authority-success-backstep-state-probe.exe'
assert not (HERE/(NAME+'_receipt.json')).exists() and not list(TARGET.glob(binary.stem+'.*'))
sources={p.relative_to(ROOT).as_posix():sha(p) for p in [source,fixture]}
command=['rustc','--edition=2024','--test',str(fixture),'-C','debuginfo=0','-o',str(binary)]
ran=None;removed=[]
try:
 built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=90)
 with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(built.stdout+built.stderr)
 if built.returncode==0:
  ran=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
 else:print(built.stderr)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items())
checks=0 if ran is None or ran.returncode else int(re.search(r'test result: ok\. (\d+) passed;',ran.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'runner_sha256':sha(Path(__file__)),'commands':[command],'build_exits':[built.returncode],'run_exit':None if ran is None else ran.returncode,'checks':checks,'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'scope':'Eight unmodified original Backstep unit tests and two public History API counterexamples, directly compiled with the standard library. No retained supplier artifacts. Actual Scenomise editor save/UI, persistence or native browser execution not run.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','run_exit','checks','generated_output_removed']}))
assert built.returncode==0 and ran.returncode==0 and checks==10
