"""Authorized clean upstream merge, dirty-byte preservation and source requalification."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
NAME='upstream_175c_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
pre=load('upstream_175c_premerge_receipt.json');old=load('graphshell_completion_review_receipt.json')
head=git('rev-parse','HEAD');assert head=='95515a1313785513efb38a433412941da7b68ed1'
assert git('rev-parse','HEAD^1')==pre['head']==old['basis_commit']
assert git('rev-parse','HEAD^2')==pre['origin']==git('rev-parse','origin/main')
assert git('rev-parse','HEAD^{tree}')==pre['merge_tree']
assert len(pre['shared_working_file_sha256'])==37
assert all(sha(ROOT/n)==d for n,d in pre['shared_working_file_sha256'].items())
assert sha(HERE/pre['audit_receipt']['path'])==pre['audit_receipt']['sha256']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert len(guards)==1462
changed=[n for n,d in guards.items() if sha(ROOT/n)!=d]
assert changed==['ports/djinn/src/resident_mere.rs']
transition_name='transition_ef78a4c5ce38_23bcf5a71ecc.json'
transition=load(transition_name)
assert transition['path']==changed[0] and transition['git_basis']==pre['head']
assert transition['prior_coverage']['sha256']==guards[changed[0]]
assert transition['current_sha256']==sha(ROOT/changed[0]) and transition.get('acknowledged_utc')
assert all(sha(HERE/n)==d for n,d in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest()==old['document']['sha256']
assert load('pending_read.json')==[] and not (HERE/'pending_transition.json').exists()
inv,cov=load('source_inventory.json'),load('read_coverage.json')
assert inv['commit']==head
before=load('upstream_175c_premerge_inventory.json')['files'];files=inv['files']
new=sorted(n for n in files if n not in before)
assert len(new)==4
assert all(cov.get(n,{}).get('sha256')==files[n]['sha256'] and cov[n]['through']==files[n]['chars'] for n in new)
assert all(sha(ROOT/n)==files[n]['sha256'] for n in new)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1936 and stats['total_files']==3731 and stats['read_chars']==22788727
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
gs=[n for n in files if n.startswith('ports/graphshell/')]
assert len(gs)==529 and all(cov.get(n,{}).get('sha256')==files[n]['sha256'] and cov[n]['through']==files[n]['chars'] for n in gs)
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-chronicle-mount-probe.*'))
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q285' and authority['entries'][-1]['id']=='A722'
arows=[
 ('Coordinated upstream integration','The user-authorized main merge preserves all shared dirty files and local commits.',
  ['95515a13','upstream_175c_premerge_receipt.json','175c4148'],
  'Clean merge-tree and zero overlap precede integration; both parents and expected tree match afterward. All 37 dirty/untracked non-audit files remain byte-identical before the audit document append. No stash, branch, worktree, reset, source rewrite or remote push was used.'),
 ('Current reading provenance','Only previously complete source can carry forward through exact unchanged spans and reviewed hunks.',
  [transition_name,'review_transition.py','source_inventory.json'],
  'One reviewed source changed: resident_mere gains 8,477 characters of natal admission/tamper tests. Old source is reconstructed from the old Git basis under ordinary LF/CRLF conversion; 218,638 unchanged characters are proven equal and one complete changed hunk was emitted and acknowledged. Six changed Cambium implementation/test files were unread already and remain unread.'),
 ('Incoming evidence and source scope','Executable historical cache preparation and source diffs count as code; logs and JSON receipts are observations.',
  ['refresh_inventory_with_diffs.py','natal-cap-20261017/narrow-lock-cache.py','mere-intake-narrow-final.diff','incremental_tests.rs','span_editing.scn'],
  'Four new inventoried source files are fully read. The cache helper prepares a private cached-ref view on its original machine; it is not run here. Natal tests cover both house systems and supplied observation variants with tamper refusal; incremental tests compare drained mutation stages to fresh style/layout/paint/hit testing. Debug builds ignore the two 1,000-row gates. Span scenarios exercise edits on copies and discard restoration. Source reading is not a rerun of retained upstream logs.'),
]
entries=[{'id':f'A{723+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,
 'state':'verified current merge and source-reading scope','verification':NAME} for i,(d,c,im,a) in enumerate(arows)]
authority['entries'].extend(entries)
next_read={'path':'crates/cambium/mere-overmap/','start':0,
 'reason':'Finish remaining shared Mere-overmap source and consumer tests, then unread Cambium hosts/rootstock including incoming incremental restyle, and all other unread Mere source.'}
notes['current_area']='Graphshell complete at current merged main; upstream source inventory and invalidated read requalified'
notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress',
 'goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,'upstream_175c_premerge_receipt.json',transition_name],
 'audit_authored_production_edits_this_turn':False,'user_authorized_upstream_merge':True,
 'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Authorized clean merge and dirty-byte checks; four new source files fully read and one earlier read requalified by exact spans/hunk. No new compiler/runtime test after integration; earlier receipts retain their source/supplier/artifact basis.'}
notes.setdefault('upstream_integrations',[]).append({'verification':NAME,'previous_head':pre['head'],
 'upstream':pre['origin'],'merged_head':head,'dirty_files_byte_preserved':37,
 'coverage_requalified_source':changed,'new_source_files_read':new,
 'runtime_qualification':'No fresh full-current transitive gate after supplier/lock changes. Existing results remain bound to their original builds.'})
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
assert diff.returncode==0,diff.stdout+diff.stderr
with (HERE/'upstream_175c_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_upstream_175c_review.py','upstream_175c_premerge_receipt.json',
 'upstream_175c_premerge_inventory.json','refresh_inventory_with_diffs.py',transition_name,'upstream_175c_diff_check.txt']
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
addition=f'''

### Current upstream integration and source requalification, 2026-10-17

Authorized upstream intake is now complete at {head}, a clean merge of local c1b5d471 and fetched origin/main 175c4148. All nine earlier local commits remain ancestors. The preflight merge tree equals the resulting committed tree, and none of upstream's 42 changed paths overlapped shared dirty/untracked files. All 37 such non-audit files were verified byte-identical after the merge, including the entire prior audit-document content before this append. No shared work was stashed, reset, staged, committed into the merge or pushed. The previously live Mere build handles were authoritatively absent before integration.

The source denominator is now 3,731 files / 39,427,411 characters. A new inventory wrapper preserves earlier rules and additionally includes source-bearing .diff files, avoiding omission of the incoming narrow intake patch. Logs, JSON receipt/data and binary captures still do not count as read code. Four added sources are completely read: Cambium incremental_tests.rs, the historical narrow-lock-cache.py and .diff, and Graphshell span_editing.scn. The historical helper creates cache/ref machinery under its original machine's testing path; it was read, not executed, and no isolated Cargo home was created here.

Exactly one previously reviewed source changed: ports/djinn/src/resident_mere.rs. Its old 218,638 reviewed characters are verified text-identical under the ordinary checkout conversion, and its complete 8,477-character added test hunk was emitted and explicitly acknowledged under the existing transition rule. The tests admit both natal house systems, with/without supplied observations, and reject altered cusp/motion facts through the shared resident. They are read, not rerun. The six other modified Cambium source files were already unread and remain unread; this integration does not promote their existing retained test logs to new audit execution.

All 529 current Graphshell source files / 4,260,912 characters are complete. Full current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale hash or pending reading chunk. The new span scenario edits explicit copies and checks discard restoration. The incremental test fixture compares styles, geometry, scrolling, sampled hit tests and paint lists with a fresh layout after successive DOM mutations; two 1,000-row tests are ignored in debug builds. No tests, build, native host, browser, GPU, network or physical workflow were rerun after this merge.

Q281-Q285 source text remains unchanged; their earlier exact diagnostic limits still apply. All prior runtime receipts retain their original source/supplier/build-artifact basis. In particular, the Chronicle adapter probe remains a fresh original-module check over retained dependency bytes, not a new whole-current Cargo gate after this lock/supplier intake. A723-A725 record integration, reading provenance and incoming-source limits. The [current upstream checkpoint](../../../support/receipts/authority-success-audit/{NAME}) preserves the premerge inventory, dirty-byte receipt and checked source transition.

No audit-authored production changes or isolated target, Cargo home or worktree were introduced. Local main contains the fetched upstream and is ten commits ahead; the full audit remains active. Next is the remainder of shared Mere-overmap and Cambium owners, then all other unread Mere source.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,
 'prior_receipt':{'path':'graphshell_completion_review_receipt.json','sha256':sha(HERE/'graphshell_completion_review_receipt.json')},
 'kind':'Authorized current upstream merge with byte-preserved WIP and requalified source reading; full Mere incomplete',
 'coverage':stats,'whole_current_files':[*new,*changed],
 'source_sha256':{n:sha(ROOT/n) for n in [*new,*changed]},
 'preserved_production_and_test_hashes':{n:sha(ROOT/n) for n in guards},
 'authorized_upstream_replaced_protected_source':{n:{'previous_sha256':guards[n],'current_sha256':sha(ROOT/n),'reading_transition':transition_name} for n in changed},
 'shared_working_file_sha256':pre['shared_working_file_sha256'],
 'shared_working_file_preservation_scope':'All 37 exact before the owned audit-document append; after append that prior document is a checked exact byte prefix.',
 'merged_parents':{'local':pre['head'],'upstream':pre['origin']},'expected_merge_tree':pre['merge_tree'],
 'completed_subtree':{'prefix':'ports/graphshell/','whole_files':529,'chars':4260912},
 'new_findings':[],'authority_entries':[e['id'] for e in entries],
 'audit_authored_production_edits_this_pass':False,'authorized_upstream_code_changes':True,
 'new_runtime_or_compilation_after_merge':0,'pending_verification':None,'next_read':next_read,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'retained_isolation':[],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Incoming test sources and code patch/helper were read, not executed. Source continuity is proved by reviewed old text plus a complete emitted changed hunk, not a claim that unchanged text was emitted again. Prior runtime gates remain bound to original exact supplier/artifact graphs.'}
save(NAME,receipt)
print(json.dumps({'receipt':NAME,'head':head,'coverage':stats,'shared_files_preserved':37,
 'new_source_files_read':4,'changed_read_requalified':1,'graphshell_complete':529,
 'retained_bytes':budget},indent=2))
