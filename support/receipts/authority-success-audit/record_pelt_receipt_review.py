"""Append complete Pelt desktop receipt/test source coverage and recovery evidence."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='pelt_receipt_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('pelt_workspace_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256'])
runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256'];external=dict(old.get('external_context_sha256',{}))
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(Path(p))==h for p,h in external.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json')
paths=[p for p in i if p.startswith('ports/pelt/desktop/workspace_viewer/receipts') or p=='ports/pelt/desktop/workspace_viewer/tests.rs']
assert len(paths)==6 and sum(i[p]['chars'] for p in paths)==294587
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():assert h==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars']
desktop=[p for p in i if p.startswith('ports/pelt/desktop/')]
assert len(desktop)==32 and sum(i[p]['chars'] for p in desktop)==985367
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in desktop)
assert load('pending_read.json')==[]
probe=load('pelt_receipt_recovery_probe_receipt.json');assert probe['exit']==0 and probe['passed']==5
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert len(probe['removed_outputs'])==2 and all(not Path(p['path']).exists() for p in probe['removed_outputs'])
# A fresh fetch was performed before recording. Qualify exact current upstream
# failure bodies without installing those commits over the shared audit report.
remote=subprocess.check_output(['git','show',f'{upstream}:ports/pelt/desktop/workspace_viewer.rs'],cwd=ROOT,text=True)
routing=subprocess.check_output(['git','show',f'{upstream}:ports/pelt/desktop/workspace_viewer/receipts/routing.rs'],cwd=ROOT,text=True)
cuts={
 'workspace_receipt_timeout_error':(routing,'    pub(in crate::workspace_viewer) fn workspace_receipt_timeout_error(','    pub(in crate::workspace_viewer) fn drive_mixed_workspace_receipt_step('),
 'primary_failed_acquire':(remote,'        let Some(swap) = host.acquire() else {','\n        let swap_target = swap'),
 'primary_startup_retry':(remote,'fn primary_startup_retry(','\nfn workspace_idle_flow(')}
upstream_bodies={}
for key,(text,start,end) in cuts.items():
    a=text.index(start);b=text.index(end,a);h=hashlib.sha256(text[a:b].encode()).hexdigest()
    assert h==probe['extracted_body_sha256'][key];upstream_bodies[key]=h
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q413' and authority['entries'][-1]['id']=='A1237'
notes['findings'].append({'id':'Q414','title':'Workspace semantic success disables capture recovery and its deadline before a receipt exists',
 'source':'ports/pelt/desktop/workspace_viewer.rs:2451-2456,2661-2730,5129-5131,5354-5383; receipts/routing.rs:68-145',
 'evidence':'A named semantic assertion sets receipt_complete before capture acquisition. If host.acquire then returns None, both workspace timeout branches ignore completed semantics and the recovery-redraw branch requires !receipt_complete. After at least one prior presentation, the separate startup predicate also refuses to schedule a retry. Exact complete timeout method, failed-acquire statement and startup predicate reproduce no deadline, no error and no requested redraw for semantically complete Appearance and Mixed fixtures. Pending-semantics controls get either retry or expired error; first-presentation retry control also works. All failure bodies remain unchanged at freshly fetched origin/main.',
 'status':'reproduced exact source-extracted CPU recovery behavior; documented, not repaired',
 'scope':'Five source-extracted CPU cases with deterministic unavailable-acquire, native-statistics, event-loop and tearout-timeout doubles. No whole desktop/original-suite build, observed native hang, real GPU failure or physical input/AT run. A later external event can wake the window; the defect is that the bounded receipt no longer owns recovery or a deadline.',
 'repair_triage':{'disposition':'documented in read-only full-source audit','reason':'Separate semantic completion from captured/presented artifact completion. Retain a receipt deadline and recovery wake until the named outcome exists or a typed terminal error is returned; do not change the legitimate first-presentation-only startup predicate into a global poll loop.'},
 'verification':NAME,'diagnostics':['pelt_receipt_recovery_probe_receipt.json']})
rows=[
 ('Receipt code is shared by tests and headed drivers but not all host plumbing is exercised','Named drivers call the same retained host helpers used by the window.',
  ['ports/pelt/desktop/workspace_viewer/receipts.rs','ports/pelt/desktop/workspace_viewer/receipts/a11y.rs','ports/pelt/desktop/workspace_viewer/receipts/chrome.rs','ports/pelt/desktop/workspace_viewer/tests.rs'],
  'Their CPU composition closures explicitly invoke Frisket/workspace frame and mark-visible, bypassing SurfaceHost acquire, queue present, capture and OS events. Direct typed accessibility requests plus installed-bridge status are not physical AT ingress. Full source read qualifies the assertion scope; original desktop tests were not run in this pass.'),
 ('Semantic progress and artifact success still share an incomplete completion model','receipt_complete stops semantic drivers, but a workspace receipt outcome requires later acquisition, capture and presentation.',
  ['ports/pelt/desktop/workspace_viewer.rs','ports/pelt/desktop/workspace_viewer/receipts/routing.rs'],
  'Q414 proves the completion flag removes both timeout and recovery wake before an artifact exists. Q411 remains the separate unsupported-multi-driver false-success issue. Final event-loop outcome checks refuse a missing artifact only if the loop ends; they do not supply the wake/deadline needed to finish a stalled bounded run.'),
 ('Reader held-source reuse has an actual source-level counting control','Shared Reader engine consumes a held Livery response and the route can restore Livery from it.',
  ['ports/pelt/desktop/workspace_viewer/receipts/reader.rs','ports/pelt/desktop/workspace_viewer/tests.rs'],
  'The original assert_reader_workspace_receipt records fetch calls and requires exactly one article and one neighbor acquisition after repeated Reader/Livery selection. That is stronger than only comparing retained body/title/lineage in the headed driver. It was fully read, not executed here. Reader accessibility intentionally exposes virtual Focus while withholding Click/SetValue until host source handoff exists.'),
 ('Host theme authoring tests preserve source and explicit activation policy','Appearance controls alter Chrome and application choice; document engines retain their own theme/content.',
  ['ports/pelt/desktop/workspace_viewer/receipts/chrome.rs','ports/pelt/desktop/workspace_viewer/tests.rs'],
  'Original tests compare held session identity, address/history/aperture around authored CSS mode activation, restored stores, saved active-definition edits without implicit application, application fonts and author CSS precedence. Headed DPI receipt requires an actual >=1.25 monitor, whereas CPU tests deliberately supply 2x geometry. Computed CSS is not physical font realization or physical input acceptance.'),
 ('Native capability recipes admit explicit fallback separately from real composition','Mixed and Chrome receipts inspect native Scry when available and Livery fallback otherwise.',
  ['ports/pelt/desktop/workspace_viewer/receipts/routing.rs','ports/pelt/desktop/workspace_viewer/receipts/chrome.rs','ports/pelt/desktop/workspace_viewer/tests.rs'],
  'Windows native readiness needs an imported view, frames/imports, multiple waits/compositions and an explicit producer ready message; resize matrices also check current geometry, dimensions and advancing counters. Non-Windows or no-Scry tests explicitly validate fallback, so their success cannot qualify native imports. The WebView2 recipe documents fresh shared destination per capture rather than unsafe reuse; concrete cross-producer generation reset remains unproved.'),
 ('Accessibility regression source covers several real lifetime boundaries','Original tests verify namespacing, foreign TreeId refusal, current geometry and session replacement.',
  ['ports/pelt/desktop/workspace_viewer/receipts/a11y.rs','ports/pelt/desktop/workspace_viewer/tests.rs'],
  'Tests reject pre-scroll Click/SetValue, stale replacement nodes, malformed/protected edits and attempts to borrow another tile capture; they check nested scroll, zoom transform, retained text/IME and sibling preservation. This mitigates speculative bypass claims based solely on duplicated desktop validation. Q412 description loss remains outside their assertions; no description regression test is present in this file.'),
 ('Receipt step mutation is hand-written across copied drivers','Each named driver uses the shared receipt_step but separately owns increment/return rules.',
  ['ports/pelt/desktop/workspace_viewer/receipts/a11y.rs','ports/pelt/desktop/workspace_viewer/receipts/chrome.rs','ports/pelt/desktop/workspace_viewer/receipts/reader.rs'],
  'AccessibilityClick sets step3 at the end of case2 and then the common increment advances to4, bypassing case3 Ready polling. This unreachable guard is source-confirmed. Current normal render marks presented Loading documents Ready between driver calls, so no accepted non-Ready real document or fresh false receipt is yet shown; keep this as a maintenance/receipt-boundary candidate rather than inventing a native failure.'),
 ('Original desktop tests leave ownership composition gaps despite broad happy-path checks','All32 desktop files are now read; coverage of specific host failure paths remains narrower.',
  ['ports/pelt/desktop/workspace_viewer/tests.rs','ports/pelt/desktop/workspace_viewer/receipts/routing.rs'],
  'The pending-surface-preflight test only constructs cache=None and asserts it, without acquiring a real pending surface or running transfer. Startup/idle tests are pure predicates. Original tests do not exercise Q411 mixed legacy drivers, Q413 focus-loss chrome gesture retirement or Q414 semantic-complete acquisition failure. Read coverage is complete for desktop, but neither test-source presence nor these CPU witnesses is whole-desktop/GPU acceptance.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1238):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'all Pelt desktop source read; original desktop/native execution not qualified this pass','verification':NAME})
notes['review_followups'].extend([
 {'area':'AccessibilityClick step3 reachability','detail':'Case2 assigns3 and common increment immediately moves to4, so the explicit Ready stage is skipped. Current ordinary post-frame Ready promotion mitigates a broad false-receipt claim. Reconcile the counter discipline and delayed-loading/capture boundary in the owning receipt work; source observation only.','verification':NAME},
 {'area':'Native receipt versus CPU composition','detail':'All original desktop test source now read. Shared helpers/real retained engine fixtures are useful, but frame/mark-visible closures bypass acquire/present/capture and physical input/AT. Q414 proves one such missing composition. Preserve claimed evidence at the exact scope; no current original desktop suite was executed.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2881 and stats['read_chars']==34438279 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
remaining=sorted(p for p,v in i.items() if p.startswith('ports/pelt/') and c.get(p,{}).get('through',0)!=v['chars'])
assert len(remaining)==72
notes['completed_source_areas'].append({'prefix':'Pelt desktop receipt dispatcher, four receipt implementation modules and original workspace tests','files':6,'chars':294587,'verification':NAME,'basis_commit':head,'qualification':'Completes all32 inventoried desktop files985367 chars.5 extracted-body recovery cases pass; original desktop test source was read only, not executed. Remaining Pelt has72 source files.'})
next_read={'path':remaining[0],'start':0,'reason':'Finish remaining72 Pelt source/examples/reftests/root entrypoints, then Signalman, Tabard desktop, scripts and supplier source without narrowing the denominator.'}
pending='No audit process running.5 exact source-body recovery tests pass with deterministic platform doubles;2 owned exe/PDB removed. All32 Pelt desktop files read, original desktop suite/native renderer/input/AT not executed. Full Pelt/repository remain incomplete. Fresh fetch unchanged; eight reviewed commits still uninstalled because of shared report overlap.'
notes['current_area']='Pelt desktop source completely read; remaining Pelt example/reftest and root source next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'pelt_receipt_recovery_probe_receipt.json'])
addition=f'''

### Pelt desktop receipt/test completion and capture recovery review, 2026-10-17

All32 inventoried Pelt desktop files are now read (985,367 characters). This pass adds the receipt dispatcher, all four receipt modules and original140,493-character workspace test source:6 files,294,587 characters. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with no stale/pending credit. Remaining Pelt has72 source/example/reftest/root files; repository completion remains unproven. Fresh fetch found unchanged upstream {upstream}; local main remains {head}. The eight reviewed upstream commits remain uninstalled because the shared report overlaps, and no upstream build receipt is implied.

Q414: semantic assertion sets receipt_complete before capture acquisition. A failed acquire then gets neither timeout nor recovery redraw because both gate on semantic incompletion; after any prior presentation, the separate startup retry also refuses a wake. Final required-outcome checks only help if the event loop ends. Five CPU cases execute the exact complete timeout/acquire-recovery/startup bodies with deterministic unavailable-swap/statistics/event-loop doubles. Completed Appearance/Mixed have no retry/deadline/error, while pending controls get retry or expired error. These bodies are unchanged upstream. This proves the missing bounded recovery path, not an observed native hang; a later external event may wake the window. Both exact run-owned exe/PDB outputs were hashed and removed. No whole desktop build, original desktop suite, real native/GPU/input or physical AT ran.

A1238-A1245 tie receipt claims to their actual assertion boundaries. Reader source reuse has a real original fetch-count control (one article and one neighbor), and virtual Reader accessibility deliberately withholds link activation until host-held destination source exists. Theme tests compare held instance/history/aperture and preserve explicit activation after library edits; computed CSS and supplied2x geometry are not physical font/input evidence. Mixed native proofs require importer/wait/composition progress and a producer ready message, whereas no-Scry/non-Windows tests explicitly validate fallback. Original accessibility tests cover stale scroll/session actions, namespaces, foreign trees, protected edits, capture interference, nested geometry and sibling preservation, mitigating speculative general bypass claims. They still omit neutral child descriptions and the mixed-driver/chrome-focus/acquire-after-success compositions Q411-Q414.

The test composition closures call Frisket/workspace frame and mark-visible directly, bypassing SurfaceHost acquire/present/capture. A pending-preflight test merely constructs cache=None; it is not a native pending-surface receipt. AccessibilityClick's case2 assigns3 then the common increment advances to4, making its explicit Ready stage unreachable. Current normal frame promotion can still make the destination Ready, so this remains a named maintenance candidate rather than a fresh claim of accepted non-Ready output. Step machines and shared completion state remain tracked alongside ownership migrations.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [capture recovery CPU witnesses](../../../support/receipts/authority-success-audit/pelt_receipt_recovery_probe_receipt.json). Prior report bytes, guarded source/shared WIP and index are preserved. No production edits, new target/home/worktree or running audit process. Continue remaining Pelt sources at {remaining[0]}.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_pelt_receipt_review.py','pelt_receipt_recovery_probe_receipt.json',*probe['evidence_sha256']]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'pelt_workspace_review_receipt.json','sha256':sha(HERE/'pelt_workspace_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'upstream_unchanged_failure_body_sha256':upstream_bodies,'new_findings':['Q414'],'authority_entries':ids,'all_pelt_desktop_inventory_source_read':True,'desktop_files':32,'desktop_chars':985367,'remaining_pelt_files':72,'all_pelt_inventory_source_read':False,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full source completion.6 new files294587 chars complete all32 desktop source985367 chars.5 exact extracted-body CPU cases with deterministic platform doubles; original desktop suite/native/GPU/input/AT not run.2 owned binaries removed; no isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
