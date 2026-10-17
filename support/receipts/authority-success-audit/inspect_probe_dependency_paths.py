"""Read-only current manifest-path inventory and locked/offline metadata gate."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='probe_dependency_paths_receipt.json';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/NAME).exists()
sources=['crates/probes/pack-distribution/Cargo.toml','crates/probes/wing-three-paths/Cargo.toml','.cargo/config.toml']
hashes={p:sha(ROOT/p) for p in sources};paths=[]
for name in sources[:2]:
    p=ROOT/name;data=tomllib.loads(p.read_text(encoding='utf-8-sig'))
    for dep,value in data.get('dependencies',{}).items():
        if isinstance(value,dict) and 'path' in value:
            target=(p.parent/value['path']).resolve()
            paths.append({'manifest':name,'dependency':dep,'resolved':str(target),'manifest_exists':(target/'Cargo.toml').is_file()})
missing=[r for r in paths if not r['manifest_exists']]
assert {r['dependency'] for r in missing}=={'mesocosm-core','mesocosm-mesh','mesocosm-render'}
lock=ROOT/'crates/probes/wing-three-paths/Cargo.lock';lock_before=sha(lock) if lock.is_file() else None
command=['cargo','metadata','--offline','--locked','--no-deps','--format-version','1','--manifest-path','crates/probes/wing-three-paths/Cargo.toml']
result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=30)
with (HERE/'wing_three_paths_metadata_output.log').open('x',encoding='utf-8') as out:out.write(result.stdout+result.stderr)
assert all(sha(ROOT/p)==h for p,h in hashes.items())
assert (sha(lock) if lock.is_file() else None)==lock_before
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256':hashes,'paths':paths,'command':command,'metadata_exit':result.returncode,'stderr':result.stderr,
    'lock_before_sha256':lock_before,'lock_after_sha256':sha(lock) if lock.is_file() else None,
    'runner_sha256':sha(Path(__file__)),'metadata_output_sha256':sha(HERE/'wing_three_paths_metadata_output.log'),
    'qualification':'Current exact local path existence plus locked/offline/no-deps Cargo metadata, no compilation or execution. Missing three legacy nested Mesocosm dependency manifests is a checkout/standalone-probe buildability finding, not a GPU rendering or upstream library defect. Existing pack-distribution Retinue and Netrender paths are present. No lock changed, worktree/target/home created, or source modified.'}
with (HERE/NAME).open('x',encoding='utf-8') as out:json.dump(receipt,out,indent=2);out.write('\n')
print(json.dumps(receipt,indent=2))
assert result.returncode!=0 and 'mesocosm' in result.stderr.lower()
