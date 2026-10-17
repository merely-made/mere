"""One authorized upstream merge, preserving every shared dirty file byte."""
from pathlib import Path
import hashlib,json,subprocess
from datetime import datetime,timezone
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()
def save(n,v):
 with (HERE/n).open('x',encoding='utf-8') as f:json.dump(v,f,indent=2);f.write('\n')
old=json.loads((HERE/'embedding_source_review_receipt.json').read_text())
assert git('rev-parse','HEAD')==old['basis_commit']
assert git('rev-parse','origin/main')=='8c299283f2145d38069f100fbedb25a54b00b746'
assert not git('diff','--cached','--name-only')
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256'],**old['auxiliary_read_sha256']}
assert all(sha(ROOT/n)==h for n,h in guards.items())
assert all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
assert sha(ROOT/old['document']['path'])==old['document']['sha256']
cov=json.loads((HERE/'read_coverage.json').read_text());inv=json.loads((HERE/'source_inventory.json').read_text())
assert json.loads((HERE/'pending_read.json').read_text())==[]
fresh={n:v['sha256'] for n,v in cov.items() if n not in guards and v['through']==inv['files'][n]['chars'] and n.startswith('crates/intel/esp/')}
assert len(fresh)==18
assert all(sha(ROOT/n)==h for n,h in fresh.items())
dirty=git('diff','--name-only').splitlines()+git('ls-files','--others','--exclude-standard').splitlines()
shared={n:sha(ROOT/n) for n in dirty if not n.startswith('support/receipts/authority-success-audit/')}
incoming=git('diff','--name-only','HEAD...origin/main').splitlines()
assert not set(shared).intersection(incoming)
before={'date_utc':datetime.now(timezone.utc).isoformat(),'head':git('rev-parse','HEAD'),'remote':git('rev-parse','origin/main'),'shared_sha256':shared,'guard_sha256':guards,'fresh_decoder_reads_sha256':fresh,'incoming_paths':incoming,'coverage_sha256':sha(HERE/'read_coverage.json'),'index_sha256':sha(ROOT/git('rev-parse','--git-path','index'))}
save('presentation_main_sync_before.json',before)
result=subprocess.run(['git','merge','--no-edit',before['remote']],cwd=ROOT,capture_output=True,text=True)
with (HERE/'presentation_main_sync_merge.log').open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
assert result.returncode==0,result.stderr
assert all(sha(ROOT/n)==h for n,h in shared.items()) and all(sha(ROOT/n)==h for n,h in fresh.items())
assert not git('diff','--cached','--name-only')
assert not git('diff','--name-only','--diff-filter=U')
changed=git('diff','--name-only',before['head'],'HEAD').splitlines()
assert all(sha(ROOT/n)==h for n,h in guards.items() if n not in changed)
after={'head':git('rev-parse','HEAD'),'remote':before['remote'],'changed_paths':changed,'changed_guard_sha256':{n:sha(ROOT/n) for n in guards if n in changed},'guard_after_sha256':{n:sha(ROOT/n) for n in guards},'shared_files_preserved':len(shared),'fresh_decoder_reads_preserved':len(fresh),'index_sha256':sha(ROOT/git('rev-parse','--git-path','index')),'qualification':'Authorized upstream merge only; no audit-authored production edit. Changed reviewed files lose coverage until reread. Suppliers for diagnostics remain exact retained artifacts, not a current whole-workspace build.'}
save('presentation_main_sync_after.json',after)
print(json.dumps({'head':after['head'],'changed_paths':len(changed),'shared_files_preserved':len(shared),'changed_protected_sources':len(after['changed_guard_sha256'])},indent=2))
