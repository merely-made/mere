"""Preserve shared WIP while merging the fetched remote. One-shot, bounded evidence."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda b:hashlib.sha256(b).hexdigest()
def git(*args,check=True):
 r=subprocess.run(['git',*args],cwd=ROOT,capture_output=True)
 if check and r.returncode:raise RuntimeError(r.stderr.decode(errors='replace'))
 return r
def value(*args):return git(*args).stdout.decode().strip()
doc='design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
receipt=HERE/'main_sync_transition_receipt.json';assert not receipt.exists()
before=value('rev-parse','HEAD');remote=value('rev-parse','origin/main')
assert before=='4523cb6f4b3998a27dc474ba30a7f1df537c5ec3'
assert remote=='641f684a76250501654104fbcd239ee8ae22d710'
assert not value('diff','--cached','--name-only')
tree=value('merge-tree','--write-tree',before,remote)
assert len(tree)==40
dirty=value('diff','--name-only').splitlines()
incoming=set(value('diff','--name-only',before+'...'+remote).splitlines())
assert set(dirty)&incoming=={doc},set(dirty)&incoming
untracked=value('ls-files','--others','--exclude-standard').splitlines()
assert not (set(untracked)&incoming)
guard={p:sha((ROOT/p).read_bytes()) for p in dirty+untracked if p!=doc and not p.startswith('support/receipts/authority-success-audit/')}
raw=(ROOT/doc).read_bytes();base=git('show',before+':'+doc).stdout
assert raw.replace(b'\r\n',b'\n').startswith(base)
assert base.endswith(b'\n')
lines=raw.splitlines(keepends=True);count=len(base.splitlines(keepends=True))
assert b''.join(lines[:count]).replace(b'\r\n',b'\n')==base
suffix=b''.join(lines[count:]);assert suffix
recovery=HERE/'main_sync_document_before.bin';assert not recovery.exists();recovery.write_bytes(raw)
prior=json.loads((HERE/'mesh_board_review_receipt.json').read_text())
assert prior['document']['sha256']==sha(raw)
stashed=git('stash','push','--message','mere authority audit document before main sync','--',doc)
stash=value('rev-parse','refs/stash')
assert (ROOT/doc).read_bytes().replace(b'\r\n',b'\n')==base
merged=git('merge','--no-edit',remote,check=False)
(HERE/'main_sync_merge.log').write_bytes(merged.stdout+merged.stderr)
assert merged.returncode==0,'merge failed; original document safely retained in own stash '+stash
after=value('rev-parse','HEAD');assert value('rev-parse','HEAD^{tree}')==tree
assert git('merge-base','--is-ancestor',remote,'HEAD',check=False).returncode==0
tracked=git('show','HEAD:'+doc).stdout;current=(ROOT/doc).read_bytes()
assert current.replace(b'\r\n',b'\n')==tracked
(ROOT/doc).write_bytes(current+suffix)
assert (ROOT/doc).read_bytes().endswith(suffix)
assert all(sha((ROOT/p).read_bytes())==h for p,h in guard.items())
assert not value('diff','--name-only','--diff-filter=U')
assert not value('diff','--cached','--name-only')
assert value('rev-parse','refs/stash')==stash
git('stash','drop','stash@{0}')
changed=value('diff','--name-only',before,after).splitlines()
out={'date_utc':datetime.now(timezone.utc).isoformat(),'before_commit':before,'fetched_origin_main':remote,
 'after_commit':after,'merge_tree':tree,'remote_is_ancestor':True,'prior_local_history_preserved':git('merge-base','--is-ancestor',before,'HEAD',check=False).returncode==0,
 'prior_review_receipt':'mesh_board_review_receipt.json','prior_review_receipt_sha256':sha((HERE/'mesh_board_review_receipt.json').read_bytes()),
 'changed_tracked_paths':changed,'preserved_dirty_and_untracked_sha256':guard,
 'document':{'path':doc,'before_sha256':sha(raw),'after_sha256':sha((ROOT/doc).read_bytes()),'preserved_appendix_bytes':len(suffix),'preserved_appendix_sha256':sha(suffix),'prior_full_document_recovery':recovery.name,'prior_full_document_recovery_sha256':sha(recovery.read_bytes()),'strategy':'temporarily stash only append-only dirty audit document; merge cleanly; append exact original dirty suffix to new tracked document; verify all other WIP bytes unchanged; remove own temporary stash'},
 'mesh_and_stickleback_source_changed':any(p.startswith(('crates/mesh/mesh/','crates/stickleback/')) for p in changed),
 'lock_or_workspace_manifest_changed':any(p in ['Cargo.lock','Cargo.toml'] for p in changed),
 'temporary_stash_removed':True,'retained_isolation':[],'merge_log_sha256':sha((HERE/'main_sync_merge.log').read_bytes())}
assert not out['mesh_and_stickleback_source_changed']
receipt.write_text(json.dumps(out,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'head':after,'origin_main':remote,'dirty_files_preserved':len(guard),'source_paths_changed':len(changed),'mesh_source_changed':False,'document_appendix_preserved':True}))
