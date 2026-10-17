"""Carry forward earlier explicit inclusions beyond the reader's base predicate."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
name='main_inventory_scope_correction_receipt.json'; assert not (HERE/name).exists()
refresh=load('main_refresh_after_stickleback_receipt.json'); inv=load('source_inventory.json')
assert sha(HERE/'source_inventory.json')==refresh['source_inventory_sha256']
(HERE/'base_predicate_main_refresh_source_inventory.json').write_bytes((HERE/'source_inventory.json').read_bytes())
restored={}
for p in refresh['inventory_removed']:
    path=ROOT/p; assert path.is_file()
    raw=path.read_bytes(); text=raw.decode('utf-8-sig',errors='replace')
    restored[p]={'sha256':hashlib.sha256(raw).hexdigest(),'chars':len(text),'lines':len(text.splitlines())}
    inv['files'][p]=restored[p]
inv['files']=dict(sorted(inv['files'].items()))
inv['scope']+=' Explicit prior inclusions of chromium/firefox extension manifests and natal-cap source diff are preserved beyond the base predicate; no still-present old source is removed.'
(HERE/'source_inventory.json').write_text(json.dumps(inv,indent=2)+'\n',encoding='utf-8')
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':inv['commit'],
    'prior_refresh_receipt_sha256':sha(HERE/'main_refresh_after_stickleback_receipt.json'),
    'base_predicate_inventory_snapshot_sha256':sha(HERE/'base_predicate_main_refresh_source_inventory.json'),
    'restored_explicit_inclusions':restored,'actual_removed_files':[], 'source_inventory_sha256':sha(HERE/'source_inventory.json'),
    'coverage':stats,'runner_sha256':sha(Path(__file__)),
    'qualification':'The base inventory predicate omitted three previously explicit source inclusions. All remain present and are restored; denominator never intentionally narrows. Earlier coverage hashes remain unchanged and only matching bytes earn credit. No source edits.'}
with (HERE/name).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({'receipt':name,'restored':list(restored),'coverage':stats},indent=2))
