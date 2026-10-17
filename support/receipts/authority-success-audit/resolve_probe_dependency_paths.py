"""Resolve dependencies after qualifying the successful --no-deps observation."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='probe_dependency_resolution_receipt.json';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/NAME).exists()
prior_path=HERE/'probe_dependency_paths_receipt.json';prior=json.loads(prior_path.read_text())
assert prior['metadata_exit']==0
assert all(sha(ROOT/p)==h for p,h in prior['source_sha256'].items())
lock=ROOT/'crates/probes/wing-three-paths/Cargo.lock';before=sha(lock)
command=['cargo','metadata','--offline','--locked','--format-version','1','--manifest-path','crates/probes/wing-three-paths/Cargo.toml']
result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=30)
log=HERE/'wing_three_paths_dependency_resolution_output.log'
with log.open('x',encoding='utf-8') as out:out.write(result.stdout+result.stderr)
assert sha(lock)==before
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256':prior['source_sha256'],'paths':prior['paths'],'command':command,'metadata_exit':result.returncode,
    'stderr':result.stderr,'lock_before_sha256':before,'lock_after_sha256':sha(lock),
    'prior_no_deps_attempt':{'path':prior_path.name,'sha256':sha(prior_path),'metadata_exit':0,
        'qualification':'--no-deps succeeded but did not resolve missing manifests. The first runner assertion expected a failure and was wrong; no compiler ran. Preserve this successful shallow observation rather than count it as full resolution.'},
    'runner_sha256':sha(Path(__file__)),'output_sha256':sha(log),
    'qualification':'Full locked/offline Cargo dependency metadata resolution, without compilation or execution. This supersedes the first --no-deps gate for buildability evidence. Exact missing local paths remain separately listed; no GPU, upstream render-library, radio, current compiler or physical acceptance claim. Source hashes and probe lock unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as out:json.dump(receipt,out,indent=2);out.write('\n')
print(json.dumps(receipt,indent=2))
assert result.returncode!=0 and 'mesocosm' in result.stderr.lower()
