"""Run CPU-only original config/core tests and bounded staging/math diagnostics."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='main_refresh_staging_probe';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
paths=['crates/intel/esp/src/infer/provider.rs','crates/intel/esp/src/infer/decoder/config.rs',
 'crates/intel/esp/src/infer/decoder/architecture_tests.rs','crates/system/luggage/src/error.rs',
 'crates/system/luggage/src/release.rs','crates/system/luggage/src/staging.rs']
sources={p:sha(ROOT/p) for p in paths}
staging=(ROOT/paths[-1]).read_text(encoding='utf-8').split('#[cfg(test)]')[0]
projection=HERE/'main_refresh_staging_production.rs';assert not projection.exists();projection.write_text(staging,encoding='utf-8')
architecture=(ROOT/paths[2]).read_text(encoding='utf-8')
expression=architecture.split('let max = logits',1)[1].split('assert!(',1)[0]
expression='let max = logits'+expression
comparison=HERE/'main_refresh_comparison_projection.rs';assert not comparison.exists()
comparison.write_text('pub fn compare(logits: &[f32], expected: &[serde_json::Value]) -> f32 {\n'+expression+'max\n}\n',encoding='utf-8')
old=json.loads((HERE/'decoder_text_math_probe_v2_receipt.json').read_text())
libs={n:old['libraries'][n] for n in ['serde','serde_json','zeroize']}
rows=[json.loads(s) for s in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
for name in ['semver','time','url','thiserror','log','blake3']:
    choices=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
    if name=='time':choices=[r for r in choices if {'parsing','formatting'}<=set(r['features'])]
    if name=='url':choices=[r for r in choices if 'serde' in r['features']]
    if name=='thiserror':choices=[r for r in choices if '@1.' in r['package_id']]
    assert len(choices)==1,(name,len(choices))
    row=choices[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')))
    libs[name]={'path':str(p),'sha256':sha(p),'package_id':row['package_id'],'features':row['features']}
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
binary=TARGET/'authority-success-main-refresh-staging.exe';assert not list(TARGET.glob(binary.stem+'.*'))
assert all(not (HERE/p).exists() for p in ['staging-trust-reload','staging-trust-discard'])
command=['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for n,v in libs.items():command+=['--extern',n+'='+v['path']]
env=dict(os.environ);env['AUDIT_STAGING_DIRECTORY']=str(HERE)
results={};removed=[];passed=0
try:
    for label,cmd in [('compile',command),('output',[str(binary),'--test-threads=1'])]:
        r=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=60)
        with (HERE/(NAME+'_'+label+'.log')).open('x',encoding='utf-8') as f:f.write(r.stdout+r.stderr)
        results[label]={'command':cmd,'exit':r.returncode}
        if r.returncode or label=='output':print(r.stdout+r.stderr)
        if r.returncode:break
        if label=='output':passed=int(re.search(r'test result: ok\. (\d+) passed;',r.stdout).group(1))
finally:
    for p in sorted(TARGET.glob(binary.stem+'.*')):
        assert p.resolve().parent==TARGET;data=p.read_bytes()
        assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.lib' and data.startswith(b'!<arch>\n')) or (p.suffix in ['.exp','.o'] and data.startswith(b'\x64\x86'))
        removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/p)==v for p,v in sources.items())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'source_sha256':sources,'libraries':libs,'results':results,'passed_tests':passed,'audit_tests':3,'original_tests':max(0,passed-3),
 'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),
 'projections':{p.name:sha(p) for p in [projection,comparison]},'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),
 'qualification':'CPU-only exact current decoder config and portable provider sources plus actual Luggage core error/release sources; their original unit tests compile unchanged. Production staging source is included unchanged up to its cfg(test) section over an audit Update facade whose install method always panics and was never invoked. Staging load/take_verified/discard are unchanged actual methods, not mocked hash or I/O. The independent-logit comparator is an exact expression projection, not a whole architecture/model run. Three audit tests include intact and single-file-tamper/finite-mismatch controls. No full native owner, signing/download tests, OS installer, real model/GPU, network or actual user files. Two small audit-owned fixture directories retained; generated executable/PDB/link outputs removed.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({'exits':{k:v['exit'] for k,v in results.items()},'passed_tests':passed,'original_tests':receipt['original_tests'],'generated_output_removed':receipt['generated_output_removed']}))
assert len(results)==2 and all(v['exit']==0 for v in results.values()) and passed>=3
