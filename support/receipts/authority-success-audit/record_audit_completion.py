"""Verify the full source-audit requirements and append the architectural closeout."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='audit_completion_receipt.json';OLD='source_reading_complete_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
digest=lambda b:hashlib.sha256(b).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
assert not (HERE/NAME).exists();old=load(OLD)
head=git('rev-parse','HEAD').decode().strip();upstream=git('rev-parse','origin/main').decode().strip()
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in {**guards,**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(Path(p))==h for p,h in old['external_context_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index').decode().strip());index=index if index.is_absolute() else ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json');assert load('pending_read.json')==[]
assert all(c[p]['sha256']==v['sha256']==sha(ROOT/p) and c[p]['through']==v['chars'] for p,v in i.items())
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in git('ls-files','--cached','--others','--exclude-standard','-z').decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats=={'complete_files':3738,'total_files':3738,'read_chars':39556704,'total_chars':39556704,'changed_since_inventory':[],'pending_chunks':0}
first=load('upstream_ownership_review_receipt.json');second=load('upstream_a9b5dd66_review.json')
assert first['basis_commit']==head and first['upstream_commit']==second['from'] and second['to']==upstream
assert first['upstream_remaining_changed_file_diffs']==[]
assert set(git('diff','--name-only',head,first['upstream_commit']).decode().splitlines())==set(first['upstream_diff_metadata'])
for p,m in first['upstream_diff_metadata'].items():
 assert digest(git('diff','--unified=3',head,first['upstream_commit'],'--',p))==m['diff_sha256']
 assert digest(git('show',first['upstream_commit']+':'+p))==m['upstream_git_bytes_sha256']
changed=set(git('diff','--name-only',second['from'],second['to']).decode().splitlines())
source_changed={p for p in changed if Path(p).suffix.lower() in module.SOURCE or Path(p).name in module.SOURCE_NAMES}
assert source_changed==set(second['source_diffs_read'])
assert set(second['documentation_diffs_read'])<=changed
for p,h in second['git_source_blob_sha256'].items():assert digest(git('show',upstream+':'+p))==h
assert sha(HERE/'upstream_a9b5dd66_review.diff')==second['diff_sha256']
ahead,behind=map(int,git('rev-list','--left-right','--count','HEAD...origin/main').split());assert (ahead,behind)==(0,16)
extra_context={p:digest(git('show',upstream+':'+p)) for p in ['support/receipts/scroll-boundary-20261017/supplier-final-gate.json','validation/http-source-provenance-20261017/tests.log']}
notes=load('review_notes.json');authority=load('authority_map.json')
assert [x['id'] for x in notes['findings']]==[f'Q{j:03}' for j in range(1,466)]
assert [x['id'] for x in authority['entries']]==[f'A{j:03}' for j in range(1,1537)]
def references(value):
 if isinstance(value,dict):
  for k,v in value.items():
   if k in ['verification','receipt','consumer_verification'] and isinstance(v,str) and v.endswith('.json'):yield v
   elif isinstance(v,(dict,list)):yield from references(v)
 elif isinstance(value,list):
  for v in value:yield from references(v)
assert all(x.get('title') and (x.get('source') or x.get('sources')) and (list(references(x)) or x.get('evidence') or x.get('consumer_evidence')) for x in notes['findings'])
assert all(all(x.get(k) for k in ['domain','claim','implementations','assessment']) for x in authority['entries'])
refs=set(references(notes['findings']))|set(references(authority['entries']))
assert all((HERE/p).is_file() for p in refs)
checked=0
for p in HERE.glob('*.json'):
 try:d=json.loads(p.read_text(encoding='utf-8-sig'))
 except (ValueError,UnicodeError):continue
 if not isinstance(d,dict):continue
 for rel,h in d.get('evidence_sha256',{}).items():
  assert (HERE/rel).is_file() and sha(HERE/rel)==h,(p.name,rel)
  checked+=1
for name in ['reduce_contract_probe_receipt.json','final_receipt_contract_probe_receipt.json']:
 d=load(name);assert d['exit']==0
 assert all(not Path(x['path']).exists() for x in d['removed_outputs'])
assert old['retained_isolation']==[]
requirements=[
 {'requirement':'Read all live source without narrowing the corpus','result':'proved','evidence':{'files':3738,'chars':39556704,'lines':sum(v['lines'] for v in i.values()),'membership':'tracked plus non-ignored untracked source and explicit source-like files, excluding audit-generated evidence','all_sha_bound_and_contiguously_acknowledged':True,'pending_chunks':0}},
 {'requirement':'Record individual bugs with evidence and qualification','result':'proved','evidence':{'findings':465,'unique_contiguous_ids':True,'all_have_source_and_evidence_or_nested_receipt':True,'receipt_references_resolve':len(refs),'qualification':'Historical, repaired, source-only, synthetic boundary, compiled public API, real runtime and planned followups remain distinguishable per entry; count is not a count of current unfixed defects.'}},
 {'requirement':'Track duplicated authority, misleading single-owner claims and unfinished migrations','result':'proved','evidence':{'authority_entries':1536,'unique_contiguous_ids':True,'all_have_domain_claim_implementations_assessment':True,'candidate_followups':len(notes['candidates']),'ruled_out':len(notes['ruled_out']),'qualification':'Full review records supported consolidation as well as duplicated policy; candidates stay unconfirmed and deferred, not implied bugs or forgotten unread areas.'}},
 {'requirement':'Refresh and account for changes on main','result':'proved_with_scope','evidence':{'fresh_fetch':'origin/main unchanged at'+upstream,'working_head':head,'ahead':ahead,'behind':behind,'upstream_installed':False,'first_segment_files':len(first['upstream_diff_metadata']),'second_segment_source_files':len(source_changed),'second_segment_doc_files':len(second['documentation_diffs_read']),'second_segment_other_files':sorted(changed-source_changed-set(second['documentation_diffs_read'])),'exact_source_diff_and_git_blob_hashes_match':True,'qualification':'Working snapshot fully read; all incoming source diffs separately reviewed. Earlier ff-only refusal due shared report overlap preserved WIP. Historical supplier gate fully inspected at closeout; HTTP log only first2000characters inspected, not full execution-log or fresh-run qualification.'}},
 {'requirement':'Preserve concurrent work and make no new production change in the active read-only pass','result':'proved','evidence':{'source_hashes':3738,'shared_non_source_guards':True,'index_sha256':sha(index),'report_original_prefix_preserved':True,'qualification':'Earlier explicitly authorized contained repairs are retained with quick_repairs_receipt and per-finding status. This final read-only pass did not modify product code or absorb shared WIP.'}},
 {'requirement':'Keep receipts bounded and clean up owned build outputs','result':'proved','evidence':{'cap_bytes':100_000_000,'all_saved_evidence_hash_references_checked':checked,'all_hash_references_match':True,'new_exe_pdb_removed':True,'retained_isolation':[],'shared_target':'C:/t/cargo-targets/mere','qualification':'Compact diagnostic sources/logs/receipts are retained in the audit directory; existing shared build cache remains shared. No audit helper process remains.'}}
]
completion={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'requirements':requirements,'classification':'full source audit complete at declared working snapshot with separately reviewed incoming source diffs','verification_limits':'No renewed whole-workspace all-feature/GPU/browser/native/physical acceptance claim. Open defects and qualified candidates are the audit deliverable; repairs and wider runtime qualification remain subsequent work.'}
notes['completion']=completion
notes.setdefault('historical_top_level_basis',[]).append(notes['basis_commit']);notes['basis_commit']=head
authority['prior_current_basis_at_completion']=authority.get('current_basis');authority['current_basis']=head
authority['completion_scope']={'working_head':head,'upstream':upstream,'upstream_installed':False,'authority_entries':1536,'verification':NAME,'historical_entries_notice':authority.get('historical_entries_notice')}
notes['repair_policy']['full_source_obligation']='Fulfilled:3738/3738current source files fullyread andSHAverified at the completion basis. Individual repair/runtime-qualification work remains separately scoped.'
notes['current_area']='Full source audit complete';notes['next_area']='Audit findings and ownership migrations are documented for subsequent repair work.'
notes['checkpoint'].update({'classification':'complete','classification_of_previous_turn':'progress','goal_status':'full source audit completion verified','coverage':stats,'finding_count':465,'authority_entries':1536,'next_read':None,'pending_verification':'No required source-audit work remains. Defect repairs and wider runtime qualification remain explicitly documented subsequent work.','audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].append(NAME)
addition=f'''

### Full source audit closeout, 2026-10-17

The full source review is complete at working HEAD{head}, including the preserved shared working changes:3,738files,1,043,171lines and39,556,704characters. Every file has contiguous acknowledged reading tied to its currentSHA256; live tracked/non-ignored membership was independently reconciled with the inventory. No unread source, stale hash or pending chunk remains. Current manifests/workflows/scenarios, optional/ignored tests, retained dependency patches and source-bearing historical receipts were included.

The findings register containsQ001-Q465, with individual source, evidence, status, verification limits and repair disposition. Repaired and superseded records are retained. The authority map containsA001-A1536 with the claimed owner, actual implementations and assessment. The212candidate followups remain explicitly unconfirmed;23ruled-out concerns preserve contrary evidence. Those registers distinguish source inspection, synthetic boundary probes, freshly compiled public APIs and actual product/backend execution. Reading source and preserving old receipts do not turn conditional or unexecuted tests into live acceptance.

The recurring strain is unfinished ownership migration. Graph/session history, current projections, saved/live content, custody lifetimes, transport admission and host presentation each have a legitimate owner, but callers still repeat parts of the owning layer's decisions. A local single-authority claim is often true only for a narrow field or stage. The audit records those valid boundaries alongside the remaining copies: shared output/reduction bodies and extrema helpers are real consolidation, while forced/inferred admission, vector-layout reconstruction, typed/dynamic shared allocation, response/input lowering and receipt verdict policy still have divergent implementations.

The most useful repair direction is to finish those boundaries one operation at a time: make graph/pane/session or device identity explicit at dispatch; let durable/source owners decide admission and revision; keep projections and caches tied to those identities; share pure validation and result classification where semantics match. Preserve deliberately different privacy, authorization and transport policies. Removing a duplicated owner decision is more valuable than splitting a large file while leaving the decision copied.

Misleading success is also repeated across layers. Numeric validation can skip or accept NaN, benchmark proof can check only one output or a different shape, ignored-test filters can execute zero tests, fresh files can stand in for valid software receipts, and cleanup attempts can be reported as completed resource closure. The final probes qualify those exact boundaries without claiming actual GPU/custody/browser failures. Earlier contained repairs retain their own receipts; this final read-only pass changed no production source.

Fresh gitfetch found origin/main unchanged at{upstream}. The working checkout remains16commitsbehind because a prior ff-only update would overwrite the shared report. All incoming source changes are separately covered by verified Git-object diff receipts:38changed files through00e44428, then6source files and4documentation files througha9b5dd66. Incoming receipt/log files are identified separately. This is a complete working-source audit plus an incoming-source diff review; upstream was not installed and its reported supplier/consumer runs were not rerun here.

Completion checks resolve every findings/authority receipt reference and verify{checked:,}saved evidence-hash references with no missing or mismatched bytes. The source/index/sharedWIP guards and exact prior report prefix remain preserved. Retained audit evidence stays below100MB; new diagnosticEXE/PDB outputs were removed. No isolated target,Cargohome orworktree remains. The stableMere build cache is shared and retained for its existing owner.

[Requirement-by-requirement completion receipt](../../../support/receipts/authority-success-audit/{NAME}). [Findings register](../../../support/receipts/authority-success-audit/review_notes.json). [Authority and migration map](../../../support/receipts/authority-success-audit/authority_map.json). Repairs and broader runtime qualification remain named followup work; the requested full-source audit itself has no remaining required work.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':completion['date_utc'],'prior_receipt':{'path':OLD,'sha256':sha(HERE/OLD)},'coverage':stats,'source_sha256':{p:v['sha256'] for p,v in i.items()},'preserved_production_and_test_hashes':guards,'completion':completion,'completion_ready':True,'new_findings':[],'authority_entries':[],'next_read':None,'pending_verification':'No required audit work remains; documented defects and candidates are subsequent repair/qualification work.','upstream_extra_receipt_context_sha256':extra_context,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':digest(prefix)},'evidence_sha256':{p:sha(HERE/p) for p in ['record_audit_completion.py','source_reading_complete_receipt.json','review_notes.json','authority_map.json','source_inventory.json','read_coverage.json']},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':completion['verification_limits']}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
print(json.dumps({'receipt':NAME,'completion_ready':True,'requirements':[(x['requirement'],x['result']) for x in requirements],'coverage':stats,'findings':465,'authority_entries':1536,'evidence_hash_references_verified':checked,'retained_bytes_before_receipt':budget},indent=2))
