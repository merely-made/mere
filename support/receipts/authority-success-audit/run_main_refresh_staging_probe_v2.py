"""Fresh offline CPU-only supplier graph after old compiled suppliers disappeared."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='main_refresh_staging_probe_v2';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
initial=HERE/'run_main_refresh_staging_probe.py'
failure=HERE/'main_refresh_staging_initial_setup_failure.json'
assert not failure.exists()
failure.write_text(json.dumps({'exit':1,'stage':'retained supplier lookup, before compilation or test fixtures',
 'error':'FileNotFoundError: prior libsemver-9d5968e7bccfd5af.rlib no longer present; old build/debug dependency family absent',
 'runner_sha256':sha(initial),'projection_files_created':['main_refresh_staging_production.rs','main_refresh_comparison_projection.rs'],
 'qualification':'Prior test receipts are historical evidence; missing supplier artifacts cannot be revalidated or reused. No production edit, compilation, model test, installer or runtime fixture ran in this attempt.'},indent=2)+'\n',encoding='utf-8')
paths=['crates/intel/esp/src/infer/provider.rs','crates/intel/esp/src/infer/decoder/config.rs',
 'crates/intel/esp/src/infer/decoder/architecture_tests.rs','crates/system/luggage/src/error.rs',
 'crates/system/luggage/src/release.rs','crates/system/luggage/src/staging.rs']
sources={p:sha(ROOT/p) for p in paths}
staging=(ROOT/paths[-1]).read_text(encoding='utf-8').split('#[cfg(test)]')[0]
assert (HERE/'main_refresh_staging_production.rs').read_text()==staging
architecture=(ROOT/paths[2]).read_text(encoding='utf-8');expression='let max = logits'+architecture.split('let max = logits',1)[1].split('assert!(',1)[0]
assert (HERE/'main_refresh_comparison_projection.rs').read_text()=='pub fn compare(logits: &[f32], expected: &[serde_json::Value]) -> f32 {\n'+expression+'max\n}\n'
assert all(not (HERE/p).exists() for p in ['staging-trust-reload','staging-trust-discard'])
manifest=HERE/'cpu-staging-runtime/Cargo.toml'
command=['cargo','test','--offline','--manifest-path',str(manifest),'--lib','-j','2','--','--test-threads=1']
env=dict(os.environ);env['CARGO_TARGET_DIR']=str(TARGET);env['AUDIT_STAGING_DIRECTORY']=str(HERE)
result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=240)
with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
print(result.stdout);print(result.stderr[-5000:])
passed=0 if result.returncode else int(re.search(r'test result: ok\. (\d+) passed;',result.stdout).group(1))
assert all(sha(ROOT/p)==v for p,v in sources.items())
removed=[]
for p in (TARGET/'debug/deps').glob('authority_success_refresh_staging-*'):
    if p.suffix not in ['.exe','.pdb']:continue
    data=p.read_bytes();assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
    removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'source_sha256':sources,'command':command,'exit':result.returncode,'passed_tests':passed,'audit_tests':3,'original_tests':max(0,passed-3),
 'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/'main_refresh_staging_probe.rs'),
 'manifest_sha256':sha(manifest),'lock_sha256':sha(manifest.with_name('Cargo.lock')),
 'projections':{p:sha(HERE/p) for p in ['main_refresh_staging_production.rs','main_refresh_comparison_projection.rs']},
 'prior_setup_failure':{'path':failure.name,'sha256':sha(failure)},'removed_outputs':removed,
 'fixture_files':{str(p.relative_to(HERE)): {'bytes':p.stat().st_size,'sha256':sha(p)} for name in ['staging-trust-reload','staging-trust-discard'] for p in (HERE/name).rglob('*') if p.is_file()},
 'qualification':'Fresh offline small CPU dependency graph under audit-owned standalone manifest in existing shared approved Cargo target. Exact current config/portable provider and Luggage core error/release tests. Unchanged production staging methods before cfg(test) over an audit Update facade that panics on install, never invoked. Three audit tests: metadata+payload substitution after reload, ignored discard failures, projected exact NaN comparator. Includes intact, one-file tamper and finite-mismatch controls. No full native Luggage/ESP owner, artifact/manifest signature test, OS installer, real model/GPU or network. Old retained supplier setup failure preserved. Small fixture files retained, test executable/PDB removed; reusable dependency cache remains in stable shared target, not an isolated target.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({'exit':result.returncode,'passed_tests':passed,'original_tests':receipt['original_tests'],'removed_test_binaries':len(removed)}))
assert result.returncode==0 and passed>=3
