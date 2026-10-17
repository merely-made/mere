"""Finish exact upstream diff review without integrating shared report WIP."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='upstream_ownership_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
def save(n,v):
    with (HERE/n).open('x',encoding='utf-8') as f:json.dump(v,f,indent=2);f.write('\n')
assert not (HERE/NAME).exists()
old=load('main_refresh_luggage_review_receipt.json');remote=load('upstream_after_inference_review_receipt.json')
head,upstream=git('rev-parse','HEAD'),git('rev-parse','origin/main');assert head==old['basis_commit'] and upstream==old['upstream_commit']
assert git('rev-list','--left-right','--count','HEAD...origin/main').split()==['0','8']
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==2179
aux,runtime,shared=old['auxiliary_read_sha256'],old['runtime_input_sha256'],old['shared_working_file_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
tail=['Cargo.lock','design_docs/2026-10-14_retained_document_resource_brief.md','design_docs/DOC_README.md',
 'design_docs/mere_docs/implementation_strategy/2026-08-15_projection_grammar_adoption_plan.md',
 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md',
 'design_docs/receipts/2026-10-17_remaining_three_adoption.json','ports/graphshell/web/Cargo.lock']
tailbytes=subprocess.check_output(['git','diff','--unified=3',head,upstream,'--',*tail],cwd=ROOT)
assert (HERE/'upstream_tail_review.txt').read_bytes()==tailbytes and len(tailbytes.decode('utf-8'))==66001
paths=git('diff','--name-only',head,upstream).splitlines();assert len(paths)==38 and set(paths)==set(remote['changed_files'])
metadata={p:{'diff_sha256':hashlib.sha256(subprocess.check_output(['git','diff','--unified=3',head,upstream,'--',p],cwd=ROOT)).hexdigest(),
 'upstream_git_bytes_sha256':hashlib.sha256(subprocess.check_output(['git','show',upstream+':'+p],cwd=ROOT)).hexdigest()} for p in paths}
for p,v in remote['reviewed_diffs'].items():assert metadata[p]==v
published=json.loads(subprocess.check_output(['git','show',upstream+':design_docs/receipts/2026-10-17_remaining_three_adoption.json'],cwd=ROOT))
published_sources={p:{'advertised':h,'upstream_git_bytes_sha256':metadata[p]['upstream_git_bytes_sha256'],
 'matches':h==metadata[p]['upstream_git_bytes_sha256']} for p,h in published['source_sha256'].items()}
brief='design_docs/2026-07-22_auto-update_brief.md';brief_hash=sha(ROOT/brief)
notes,authority=load('review_notes.json'),load('authority_map.json');assert authority['entries'][-1]['id']=='A1148'
q=next(f for f in notes['findings'] if f['id']=='Q383')
q['prior_design_record']={'path':brief,'lines':'374-401','sha256':brief_hash,
 'qualification':'Already documented and independently probed in the2026-10-12pre-adoption comparison. Q383 is a fresh scoped confirmation during this source audit, not a newly discovered product issue or a TUF adoption ruling.'}
for a in authority['entries']:
    if a['id']=='A1142':a['assessment']+=' The2026-10-12pre-adoption brief already documents joint staged metadata/payload tampering and keeps adoption gated; this audit reconfirms that existing open migration.'
rows=[
 ('Rich requests gain a shared lowerer while two host adapters still map responses','RemoteFetcher and configured NetFetch both call Request::from_host and preserve status/headers/body/final identity.',['crates/system/document-lanes/src/remote.rs','crates/system/fetch/src/handle.rs','ports/pelt/core/src/native.rs'],
  'Deadline covers permit wait or request/body under the rich path; zero concurrency refuses. Legacy byte fetch keeps success-status filtering, while held HTTPS bytes explicitly refuse invented transport metadata. Response DTO conversion remains repeated in both adapters. Resources now routes legacy plain GET through read_request, whose trait default refuses; only NetFetch currently implements Fetch locally, so third-party plain-only compatibility is a documented API followup, not a demonstrated product regression.'),
 ('Keeping a derived link now reaches the durable graph owner with its edge','create_link delegates known origin to MereHost::create_linked_address, grouping address and Hyperlink in one graph mutation.',['ports/graphshell/src/local_intake.rs','ports/graphshell/src/product.rs','ports/graphshell/src/web_tree/product.rs'],
  'Existing address receives the relation without a duplicate node; missing source falls back explicitly to plain address. Upstream tests assert one change and idempotent relation. This is source review of an actual owner call, not a fresh test run. Upstream report separately says no host yet adopts pandect facet storage, even for positions: restoring CartographyGeometry and converging facet persistence are different migrations.'),
 ('Input ownership distinguishes physical keys, keyboard text and paste across adapters','Egui/iced, retained Graphshell, Pelt Surface and native document routes now separate key events from text payloads.',['crates/cambium/cambium-rootstock/src/embedding/egui_input.rs','crates/cambium/cambium-rootstock/src/embedding/iced_input.rs','ports/graphshell/src/retained_content/view.rs','ports/pelt/core/src/content.rs','ports/pelt/desktop/workspace_viewer.rs','ports/pelt/desktop/static_viewer.rs'],
  'Surface key carries no duplicate synthesized text; native surface routes keep their separate path. Printable/control/modifier filtering still has several implementations, with egui handling its exact control strings separately from iced/native all-control filtering. Trace IME/AltGr/focus ownership before calling every input path unified. Tests were read, not executed here.'),
 ('Upstream publication receipts retain their own source and platform basis','All38diffs at00e44428 are read, including docs, locks and the pre-existing adoption JSON.',paths,
  'No source checkout integration or upstream test rerun. Root/web locks move Genet7f13b85f to917f88e6; root also changes Bole/Vano and welding becomes published0.15.0. The upstream Sets note records focused passes but one reproduced unchanged wall-clock failure in the full suite, with receiving-consumer adoption pending. Published resource receipt names a prior Genet/source/lock basis and inaccessible external logs; per-file comparison is recorded rather than silently treating it as fresh exact-head proof. Private Isocosm authoring is a documented app-local consumer, not a shared Mere resource API or independently inspected Isocosm implementation.')]
entries=[{'id':f'A{1149+i}','domain':d,'claim':c,'implementations':ps,'assessment':a,
 'state':'upstream-only exact diff review complete; checkout source unchanged; full audit active','verification':NAME} for i,(d,c,ps,a) in enumerate(rows)]
authority['entries'].extend(entries)
notes['review_followups'].append({'area':'Upstream migration completion','detail':'All38diffs reviewed read-only; integrate only while preserving shared report. Verify Resource rich/legacy API compatibility, consolidate response/text lowering only where semantics match, and keep canonical Resource/legacy Surface facets plus host facet-store adoption explicit. Q383 is prior-known and freshly reconfirmed, not a new trust-policy decision.','verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats==old['coverage']
notes['checkpoint']['new_receipts'].append(NAME)
notes['checkpoint']['authority_entries']=len(authority['entries'])
notes['checkpoint']['pending_verification']='No owned process live. All38upstream diffs read; upstream tests not rerun and fast-forward still blocked by report overlap. Current checkout coverage2649/3738. Registry and remaining repository source unread; native installers, full ESP/Luggage owners and real models/GPU remain unexecuted. Old findings retain recorded scope.'
next_read={'path':'crates/system/registry/Cargo.toml','start':0,'reason':'Continue the full source inventory through Registry; upstream diff review complete, actual fast-forward still requires preserving the shared report.'}
notes['checkpoint']['next_read']=next_read;notes['next_area']=next_read['reason']
addition=f'''

### Upstream ownership diff completion, 2026-10-17

All38changed-file diffs from local {head} to upstream {upstream} are now read through exact Git objects. The initial combined tail output was truncated and earns no credit: its7file/66,001character tail was preserved, re-emitted in full as0:22000,22000:44000,44000:66001, and reviewed completely. This is an upstream diff review, not a whole upstream checkout/source receipt or fresh test run. Local full-source coverage remains{stats['complete_files']:,}/{stats['total_files']:,}files with no stale/pending credit. Git still refuses fast-forward because it would overwrite the shared report; no stash,revert,commit or production edits.

A1149-A1152 track the resource and input/link ownership changes. Both HTTP adapters now use the shared host-request lowerer and preserve non-success HTTP metadata in rich responses; their response DTO lowering remains repeated. Absolute rich-request deadlines and zero-concurrency refusal are real source changes. Held HTTPS fixture bytes refuse invented response metadata. Legacy Fetch compatibility with a custom plain-only implementation needs separate checking, with only NetFetch implementing that trait locally. Kept links call one graph-owner mutation for address plus Hyperlink and preserve an existing address; this does not finish pandect facet-store adoption, explicitly still absent in all hosts in upstream's report. Input adapters separate key events from keyboard text/paste while still repeating printable/control/modifier rules. Focus/IME/AltGr and trait-compatibility followups remain documented rather than redesigning them here.

The fetched locks advance Genet7f13b85f to917f88e6 and change other supplier edges; published welding0.15.0 replaces its moving Git pin. These are not installed in this checkout. The upstream Sets qualification reports focused passes and a reproduced unchanged timing-control failure, not a full library pass. Its receiving Turnstone adoption and live cache invalidation/headed/GPU scope remain open. Resource adoption's JSON carries an earlier source/lock and external-log basis; actual current per-file hash matches/mismatches are preserved in the receipt, without treating upstream-authored tests as locally rerun proof. The documented private Isocosm authoring consumer remains app-local with shared API extraction deferred, not new shared Mere authority.

Q383's staged trust gap was already documented and independently probed in the canonical auto-update brief's2026-10-12comparison (lines374-401). This pass is a fresh scoped confirmation of an existing open trust/apply migration, not a new discovery or authority to adopt a trust provider. The brief already separates update trust, install recovery and product policy; those gates remain intact.

Evidence: [upstream completion](../../../support/receipts/authority-success-audit/{NAME}) and [full re-emitted tail](../../../support/receipts/authority-success-audit/upstream_tail_review.txt). Source/WIP/index hashes and previous report bytes remain intact; no process live or isolated target retained. No new tests or finding IDs beyond Q382-Q385; only four further authority entries. Next: Registry source and the rest of the full inventory.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_upstream_ownership_review.py','upstream_tail_review.txt'];budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,
 'prior_receipt':{'path':'main_refresh_luggage_review_receipt.json','sha256':sha(HERE/'main_refresh_luggage_review_receipt.json')},
 'coverage':stats,'source_sha256':{},'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,
 'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'upstream_diff_metadata':metadata,
 'upstream_reviewed_changed_files':38,'upstream_remaining_changed_file_diffs':[],'upstream_is_not_installed':True,
 'published_resource_receipt_source_comparison':published_sources,
 'prior_design_context':{'path':brief,'sha256':brief_hash,'read_scope':'targeted pre-adoption comparison sections, not full-document source credit'},
 'new_findings':[],'authority_entries':[e['id'] for e in entries],'next_read':next_read,
 'pending_verification':notes['checkpoint']['pending_verification'],'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],
 'shared_generated_cache':old['shared_generated_cache'],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'qualification':'Exact upstream diff review only; local current-source coverage unchanged. The truncated first tail output was re-emitted fully before recording completion. No upstream compiler/test, native install or real model execution. Canonical prior trust comparison reconciled; previous source/evidence/report/WIP/index preserved.'}
save(NAME,receipt)
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'upstream_diffs_reviewed':38,'coverage':stats,'authority_entries':receipt['authority_entries'],
 'published_resource_source_matches':sum(v['matches'] for v in published_sources.values()),'published_resource_sources':len(published_sources),
 'prior_known_Q383_context_recorded':True,'retained_receipt_bytes':budget,'next_read':next_read},indent=2))
