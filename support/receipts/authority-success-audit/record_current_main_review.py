"""One-shot post-sync freshness checkpoint. Historical receipts remain intact."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,value,key):
 at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
 if at is None:items.append(value)
 else:items[at]=value
assert not (HERE/'current_main_review_receipt.json').exists()
transition=load('main_sync_transition_receipt.json');old=load('mesh_board_review_receipt.json');probe=load('mesh_current_probe_receipt.json')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
assert head==transition['after_commit'] and transition['remote_is_ancestor'] and transition['prior_local_history_preserved']
assert probe['tests']==9 and probe['test_exit']==0 and probe['generated_output_removed']
assert '123 passed; 0 failed' in (HERE/'mesh_current_lib_tests.log').read_text(encoding='utf-8-sig')
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
assert all(sha(ROOT/p)==h for p,h in transition['preserved_dirty_and_untracked_sha256'].items())
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1173 and stats['total_files']==3355 and stats['read_chars']==15421049 and not stats['pending_chunks']
invalid=stats['changed_since_inventory'];assert len(invalid)==13
preserved_before={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
changes={p:{'before_sha256':h,'current_sha256':sha(ROOT/p)} for p,h in preserved_before.items() if sha(ROOT/p)!=h}
assert set(changes).issubset(transition['changed_tracked_paths']),set(changes)-set(transition['changed_tracked_paths'])
preserved_after={p:sha(ROOT/p) for p in preserved_before}
notes=load('review_notes.json');authority=load('authority_map.json')
entry={'id':'A385','domain':'Audit freshness across incoming main changes',
 'claim':'Previously reviewed source and fresh current gates remain distinct.',
 'implementations':['Fetched origin/main641f684a','Merged into local main04474731 preserving local history/WIP','Mesh and Stickleback source unchanged','Workspace manifest and lock changed','123 current tests and9 current native observations','13 read source files invalidated by changed hashes'],
 'assessment':'Latest fetched main is integrated. Current Mesh findings survive fresh artifacts, but earlier read coverage for changed files is removed until rereading. Historic receipts stay on their original basis and are not relabeled current.',
 'state':'Current fetched origin integrated; full-source goal active','verification':'current_main_review_receipt.json'}
upsert(authority['entries'],entry,'id')
notes['basis_commit']=head
notes.setdefault('basis_transitions',[]).append({'from':transition['before_commit'],'to':head,'fetched_origin_main':transition['fetched_origin_main'],
 'receipt':'current_main_review_receipt.json','invalidated_read_paths':invalid,'historical_findings_qualification':'Earlier receipts retain original commit/source hashes. Changed areas require reread/requalification; current five Mesh findings rerun.'})
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':['mesh_board_review_receipt.json','main_sync_transition_receipt.json','current_main_review_receipt.json'],
 'next_read':old['next_read'],'production_edits_this_turn':False,'upstream_changes_integrated':True,
 'diagnostic_outcomes':'On merged main,123 unchanged Mesh library tests pass and9 native public API/MemoryBackend observations reproduce the same five defects.',
 'user_instruction':'Full-source audit with current upstream; preserve WIP and avoid production repairs during read-only continuation.',
 'coverage_note':'Ten new Mesh files read. After sync13 previously read files became stale and3 source files were added. Current coverage1173/3355 and15421049/36897906 chars; Mesh23/43 unchanged. Revisit invalidated paths alongside remaining source.'}
notes['current_area']='Current-main Mesh board/lease/store checkpoint and audit freshness transition'
upsert(notes['source_qualifications'],{'area':'Current fetched main integration and Mesh requalification',
 'note':'origin/main641f684a merged as04474731. Both prior local commits and36 unrelated dirty/untracked file byte hashes preserved; only append-only audit document temporarily stashed and exact dirty suffix restored; own stash removed. Mesh/Stickleback source unchanged, workspace manifest and lock updated. Fresh current build,123 library tests and9 diagnostic observations pass. Five defects remain reproduced.13 earlier source files invalidated; current full-source coverage1173/3355,15,421,049/36,897,906 chars. Earlier receipts remain historical on original basis. Stable target reused, probe executable/PDB removed, no isolated home/target/worktree.'},'area')
doc=transition['document']['path'];dest=ROOT/doc;before=dest.read_bytes()
assert sha(dest)==transition['document']['after_sha256']
heading='### Current main refresh and Mesh requalification, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nThe audit was at4523cb6f when a fresh fetch found14 incoming origin/main commits through641f684a, alongside5 local-only commits. The clean merge is04474731. Prior local history is an ancestor, and36 unrelated dirty/untracked file hashes are unchanged. The only incoming/dirty overlap was this audit document: its append-only dirty suffix was preserved exactly around the incoming tracked additions, with the original full bytes retained as bounded recovery evidence. Only that document was temporarily stashed; the owned stash is removed. No production repair, branch/worktree, isolated Cargo home or isolated target was created.\n\nMesh and Stickleback source are unchanged by the merge; the workspace manifest and lock changed. Fresh locked/offline current-main library build succeeds, all123 original Mesh library tests pass, and all9 native public API/MemoryBackend diagnostic assertions pass again. Q198-Q202 remain reproduced. Passing these diagnostic assertions means observing the defects; neither live network divergence nor forged wire-signature admission is claimed. Prior build/probe/review receipts retain their original hashes and basis rather than being overwritten as current.\n\nFresh source inventory has3,355 files and36,897,906 characters. Thirteen previously read source files changed upstream (workspace manifest, eleven Pictograph source/test files, two Pandect files; the explicit receipt list is authoritative), so their old read credit is removed pending reread. Current coverage is1,173/3,355 complete files and15,421,049/36,897,906 characters. Mesh remains23/43 files and325,472/556,938 characters read. The next Mesh file is drop_export.rs; changed earlier areas also remain in the full-source obligation. The lower current coverage reflects freshness invalidation, not discarded reading evidence.\n\nThe [current main review receipt](../../../support/receipts/authority-success-audit/current_main_review_receipt.json) binds the new gates, preserved WIP, source invalidations, original receipt chain, default100 MB audit evidence cap and generated-output cleanup. No cited receipt is overwritten. Full-source objective remains active.\n'
# The thirteen paths are one workspace manifest, ten Pictograph files and two Pandect files.
addition=addition.replace('eleven Pictograph','ten Pictograph')
for a,b in [('at4523','at 4523'),('found14','found 14'),('through641','through 641'),('alongside5','alongside 5'),('is044','is 044'),('and36','and 36'),('all123','all 123'),('all9','all 9'),('has3,355','has 3,355'),('and36,897','and 36,897'),('is1,173','is 1,173'),('and15,421','and 15,421'),('remains23','remains 23'),('and325','and 325'),('default100','default 100')]:addition=addition.replace(a,b)
dest.write_bytes(before+addition.encode())
save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'current_main_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
files=['mesh_current_build.jsonl','mesh_current_build_stderr.log','mesh_current_lib_tests.log','run_mesh_current_probe.py','mesh_current_probe_receipt.json','mesh_current_probe_build.log','mesh_current_probe_output.log','main_sync_transition_receipt.json','current_main_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'fetched_origin_main':transition['fetched_origin_main'],
 'kind':'Current upstream integration and Mesh requalification; full source objective incomplete','production_edits_this_pass':False,
 'upstream_changes_integrated':True,'coverage':stats,'mesh_coverage':old['mesh_coverage'],'source_invalidations':invalid,
 'preserved_production_and_test_hashes':preserved_after,'source_sha256':old['source_sha256'],
 'upstream_changed_previously_guarded_sources':changes,'next_read':old['next_read'],
 'new_findings':old['new_findings'],'authority_entries':['A385'],'library_build_exit':0,'library_tests':123,'library_test_exit':0,'native_observations':9,'native_probe_exit':0,
 'document':{'path':doc,'sha256':sha(dest),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in files},'retained_isolation':[],'generated_probe_removed':probe['generated_output_removed'],
 'receipt_budget':{'cap_bytes':100_000_000,'audit_retained_bytes':sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()),'required':'Compact source/coverage, build/test/probe and merge-preservation evidence only; no captures'},
 'limits':['Current means latest freshly fetched origin/main641f684a integrated, not a claim remote cannot move again.',
 '13 changed source files no longer count read; their prior findings retain original basis pending requalification.',
 'Current Mesh source unchanged; five observed defects persist with fresh build/test/probe artifacts.',
 'No production repair, user data, network campaign, redb reopening, worker/GPU/browser or physical-device run.',
 'Preserved document recovery bytes retained as merge evidence under budget; own temporary stash removed.']}
assert receipt['receipt_budget']['audit_retained_bytes']<100_000_000
assert all(sha(ROOT/p)==h for p,h in transition['preserved_dirty_and_untracked_sha256'].items())
save('current_main_review_receipt.json',receipt)
print(json.dumps({'head':head,'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'retained_bytes':receipt['receipt_budget']['audit_retained_bytes']}))
