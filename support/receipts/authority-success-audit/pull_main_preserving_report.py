"""Pull by fetch plus pinned fast-forward, preserving the sole overlapping dirty file."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[3]
HERE=Path(__file__).resolve().parent
OUT=HERE/'pull-main-preservation'
REPORT='design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
sha=lambda b:hashlib.sha256(b).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
assert git('branch','--show-current').decode().strip()=='main'
assert not git('diff','--cached','--name-only').strip()
before=git('rev-parse','HEAD').decode().strip()
target=git('rev-parse','origin/main').decode().strip()
assert subprocess.run(['git','merge-base','--is-ancestor',before,target],cwd=ROOT).returncode==0
dirty=set(filter(None,git('diff','--name-only','-z','HEAD').decode().split('\0')))
incoming=set(git('diff','--name-only',before,target).decode().splitlines())
assert dirty&incoming=={REPORT},dirty&incoming
untracked=set(filter(None,git('ls-files','--others','--exclude-standard','-z').decode().split('\0')))
assert not (untracked&incoming)
held=dirty|{p for p in untracked if not p.startswith('support/receipts/authority-success-audit/')}
saved={p:(ROOT/p).read_bytes() for p in held}
base=git('show',before+':'+REPORT);remote=git('show',target+':'+REPORT);mine=saved[REPORT]
base_lines=base.splitlines(keepends=True);remote_lines=remote.splitlines(keepends=True)
local_lines=mine.splitlines(keepends=True)
assert mine.replace(b'\r\n',b'\n').startswith(base)
ops=[x for x in difflib.SequenceMatcher(None,base_lines,remote_lines,autojunk=False).get_opcodes() if x[0]!='equal']
assert len(ops)==1 and ops[0][0]=='insert',ops
_,bstart,bend,rstart,rend=ops[0];assert bstart==bend
assert all(x.replace(b'\r\n',b'\n')==y for x,y in zip(local_lines[:len(base_lines)],base_lines))
offset=sum(len(x) for x in local_lines[:bstart])
ending=b'\r\n' if local_lines[bstart-1].endswith(b'\r\n') else b'\n'
addition=b''.join(remote_lines[rstart:rend]).replace(b'\n',ending)
merged=mine[:offset]+addition+mine[offset:]
assert merged[:offset]+merged[offset+len(addition):]==mine
assert merged.replace(b'\r\n',b'\n').startswith(remote)
OUT.mkdir(exist_ok=False)
for name,data in [('report.before.bin',mine),('report.base.bin',base),('report.upstream.bin',remote),('report.merged.bin',merged)]:
 (OUT/name).write_bytes(data)
plan={'before':before,'target':target,'tracked_dirty':sorted(dirty),'untracked_product':sorted(held-dirty),'preserved_sha256':{p:sha(b) for p,b in saved.items()},'report':{'path':REPORT,'before_sha256':sha(mine),'merged_sha256':sha(merged),'insert_offset':offset,'insert_bytes':len(addition),'existing_bytes_retained_exactly':True},'qualification':'Before/after buffers retained; only one upstream insertion is combined with existing report bytes. No staging/stash/commit/reset, other dirty files untouched.'}
(OUT/'plan.json').write_text(json.dumps(plan,indent=2)+'\n',encoding='utf-8')
# Compare-and-swap checks immediately before temporarily clearing the overlap.
assert git('rev-parse','HEAD').decode().strip()==before
assert all((ROOT/p).read_bytes()==b for p,b in saved.items())
clean_before=git('cat-file','--filters',before+':'+REPORT)
clean_after=git('cat-file','--filters',target+':'+REPORT)
subprocess.run(['git','restore','--source='+before,'--worktree','--',REPORT],cwd=ROOT,check=True)
assert (ROOT/REPORT).read_bytes()==clean_before
result=subprocess.run(['git','merge','--ff-only',target],cwd=ROOT,text=True,capture_output=True)
(OUT/'fast-forward.log').write_text(result.stdout+result.stderr,encoding='utf-8')
current=git('rev-parse','HEAD').decode().strip()
if result.returncode:
 assert current==before and (ROOT/REPORT).read_bytes()==clean_before
 (ROOT/REPORT).write_bytes(mine)
 raise RuntimeError('Fast-forward failed; original report restored; see saved log')
assert current==target
assert (ROOT/REPORT).read_bytes()==clean_after,'Concurrent report change after fast-forward; backups preserved'
(ROOT/REPORT).write_bytes(merged)
assert all((ROOT/p).read_bytes()==b for p,b in saved.items() if p!=REPORT)
assert (ROOT/REPORT).read_bytes()==merged
assert not git('diff','--cached','--name-only').strip()
assert set(filter(None,git('diff','--name-only','-z','HEAD').decode().split('\0')))==dirty
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
record={**plan,'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'head':current,'ahead_behind':git('rev-list','--left-right','--count','HEAD...origin/main').decode().strip(),'dirty_files_preserved':len(saved)-1,'report_before_recoverable':True,'report_local_bytes_preserved_by_exact_splice':True,'source_code_merge_conflicts':0,'new_production_edits':False,'evidence_sha256':{p.name:sha(p.read_bytes()) for p in OUT.iterdir() if p.is_file()},'scope':'Pull/integration only. Earlier audit completion and source hashes remain historical to their stated basis; incoming source diffs were reviewed separately. No tests rerun for this fast-forward.'}
(OUT/'receipt.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'head':current,'commits_pulled':int(git('rev-list','--count',before+'..'+current)),'ahead_behind':record['ahead_behind'],'other_local_files_preserved_exactly':len(saved)-1,'audit_report_existing_bytes_preserved':True,'receipt':str(OUT/'receipt.json')},indent=2))
