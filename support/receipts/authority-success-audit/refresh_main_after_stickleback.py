"""Refresh changed main without transferring stale source-reading credit."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):
    with (HERE/n).open('x',encoding='utf-8') as f: json.dump(v,f,indent=2); f.write('\n')
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
name='main_refresh_after_stickleback_receipt.json'; assert not (HERE/name).exists()
old=load('stickleback_drop_review_receipt.json'); head=git('rev-parse','HEAD')
assert head!=old['basis_commit'] and git('branch','--show-current')=='main' and load('pending_read.json')==[]
prior_inv=load('source_inventory.json'); coverage=load('read_coverage.json')
for n in ['source_inventory.json','read_coverage.json']:
    (HERE/('before_main_refresh_'+n)).write_bytes((HERE/n).read_bytes())
failed=HERE/'record_stickleback_completion_review.py'
(HERE/'record_stickleback_completion_review_initial.py').write_bytes(failed.read_bytes())
save('stickleback_completion_initial_failure.json',{'command':'python -X utf8 support/receipts/authority-success-audit/record_stickleback_completion_review.py',
    'exit':1,'error':'AssertionError: head == old[basis_commit] failed before any report or notes mutation',
    'runner_path':'record_stickleback_completion_review_initial.py','runner_sha256':sha(failed),
    'observed_head':head,'old_basis':old['basis_commit'],'report_matches_old_hash':sha(ROOT/old['document']['path'])==old['document']['sha256']})
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
def changes(items):
    return {p:{'before':v,'after':sha(ROOT/p) if (ROOT/p).is_file() else None} for p,v in items.items() if not (ROOT/p).is_file() or sha(ROOT/p)!=v}
protected_changes=changes(guards); runtime_changes=changes(old['runtime_input_sha256'])
shared_changes=changes({p:v for p,v in old['shared_working_file_sha256'].items() if p!=old['document']['path']})
assert not changes(old['compile_source_sha256']) and not changes(old['auxiliary_read_sha256'])
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
inventory_run=subprocess.run(['python','-X','utf8',str(HERE/'read_coverage.py'),'inventory'],cwd=ROOT,capture_output=True,text=True)
assert inventory_run.returncode==0
current_inv=load('source_inventory.json'); assert current_inv['commit']==head==git('rev-parse','HEAD')
added=sorted(set(current_inv['files'])-set(prior_inv['files']))
removed=sorted(set(prior_inv['files'])-set(current_inv['files']))
changed={p:{'before':prior_inv['files'][p],'after':v} for p,v in current_inv['files'].items() if p in prior_inv['files'] and v['sha256']!=prior_inv['files'][p]['sha256']}
status=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'old_basis_commit':old['basis_commit'],'basis_commit':head,
    'kind':'externally advanced shared main; refreshed inventory and invalidated changed source credit',
    'old_inventory_sha256':sha(HERE/'before_main_refresh_source_inventory.json'),
    'preserved_coverage_sha256':sha(HERE/'before_main_refresh_read_coverage.json'),
    'source_inventory_sha256':sha(HERE/'source_inventory.json'),'coverage_unchanged':sha(HERE/'read_coverage.json')==sha(HERE/'before_main_refresh_read_coverage.json'),
    'inventory_added':added,'inventory_removed':removed,'inventory_changed':changed,'coverage':status,
    'protected_source_changes':protected_changes,'runtime_input_changes':runtime_changes,'shared_working_file_changes':shared_changes,
    'current_git_index_sha256':sha(index),'current_shared_working_file_sha256':{p:sha(ROOT/p) for p in old['shared_working_file_sha256']},
    'current_runtime_input_sha256':{p:sha(ROOT/p) for p in old['runtime_input_sha256']},
    'report_unchanged':sha(ROOT/old['document']['path'])==old['document']['sha256'],
    'stickleback_30_compile_and_read_hashes_unchanged':True,'audit_authored_production_edits':False,
    'old_evidence_sha256':{p:sha(HERE/p) for p in ['before_main_refresh_source_inventory.json','before_main_refresh_read_coverage.json','record_stickleback_completion_review_initial.py','stickleback_completion_initial_failure.json','refresh_main_after_stickleback.py']},
    'qualification':'External main publication changed old inference sources and added source; no pull, merge, reset, commit or production edit by audit. Changed prior source credit is invalid until reread at new hash. All historical findings retain their recorded basis. Current Stickleback source hashes match both diagnostic receipts; retained suppliers are qualified, not a fresh Cargo graph at new Cargo.lock.'}
save(name,receipt)
print(json.dumps({'receipt':name,'basis_commit':head,'added_files':len(added),'changed_files':len(changed),'removed_files':len(removed),
    'changed_protected_sources':len(protected_changes),'changed_runtime_inputs':list(runtime_changes),'coverage':status},indent=2))
