"""Checkpoint completed shared view reading and narrowly qualified diagnostics."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
NAME='mere_view_review_receipt.json'; assert not (HERE/NAME).exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
old=load('upstream_175c_review_receipt.json'); head=git('rev-parse','HEAD')
assert head==old['basis_commit']=='95515a1313785513efb38a433412941da7b68ed1'
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}; assert len(guards)==1466
assert all(sha(ROOT/n)==h for n,h in guards.items())
assert all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path']; prefix=doc.read_bytes(); assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
inv,cov=load('source_inventory.json'),load('read_coverage.json'); assert inv['commit']==head
assert load('pending_read.json')==[]
regions=['crates/cambium/mere-overmap/','crates/cambium/mere-view/']
whole=sorted(n for n in inv['files'] if any(n.startswith(p) for p in regions))
whole += ['crates/cambium/cambium/Cargo.toml','crates/cambium/cambium/examples/component_catalog.css','crates/cambium/cambium/examples/component_catalog.rs']
assert len(whole)==31
assert all(cov.get(n,{}).get('sha256')==inv['files'][n]['sha256'] and cov[n]['through']==inv['files'][n]['chars'] for n in whole)
assert all(sha(ROOT/n)==inv['files'][n]['sha256'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1965 and stats['read_chars']==23198267 and not stats['changed_since_inventory'] and not stats['pending_chunks']
facet=load('route_facet_probe_receipt.json'); view=load('mere_view_state_probe_v2_receipt.json'); failed=load('mere_view_state_probe_receipt.json')
assert facet['build_exit']==facet['run_exit']==0 and facet['checks']==3
assert view['build_exits']==[0,0] and view['run_exit']==0 and view['checks']==7
assert failed['build_exits']==[0,1] and failed['run_exit'] is None
assert all(r['generated_output_removed'] for r in [facet,view,failed])
assert all(not Path(o['path']).exists() for r in [facet,view,failed] for o in r['removed_outputs'])
for r in [facet,view]:
 assert all(sha(ROOT/n)==h for n,h in r['source_sha256'].items())
 assert all(sha(Path(v['path']))==v['sha256'] for v in r['libraries'].values())
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q285' and authority['entries'][-1]['id']=='A725'
rows=[
 ('Q286','Arbitrary facet fields shadow the canonical address used by saved-view actions',
 'crates/cambium/mere-overmap/src/route.rs:635; crates/cambium/mere-overmap/src/route_session.rs:524; ports/graphshell/src/bin/graphshell_mere.rs:281',
 'verified exact current facet functions over real retained types; saved-view consumer path source-traced',
 'add_node_facets prepends flattened arbitrary facet fields to PortableCard display values. Both shared view_invocation and native Connection::take select the first value labelled Address for ViewIntent.focus; shared view_saved uses the same label. The exact current functions over real Chartulary facet storage produce a supplied facet Address before the still-present canonical owner address. No-facet and lowercase-address controls keep the canonical first match. This verifies ambiguous data selection and traces its use in saved-view payload construction; it does not claim a completed owner SetView action, durable wrong focus, GUI exploit or authorization bypass.',
 'route_facet_probe_receipt.json',
 'Carry canonical identity/address separately from arbitrary human-labelled display values, and have actions use that typed owner material. Both native and shared route consumers need the same rule.'),
 ('Q287','Mere-view assigns focus after folding, bypassing its stand-in remapping',
 'crates/cambium/mere-view/src/view.rs:681; crates/cambium/mere-view/src/folds.rs:84',
 'verified original current Mere-view crate over hash-bound retained dependency libraries',
 'MereView::swatch applies folds::project while the new Swatch focus is still None, then assigns state.focused afterward. The fold projector remaps focus only when it receives a member focus, so its intended focus_request path is unreachable from this caller. A public-method probe builds a valid summary fold over a,b: a disappears, the summary exists, focus remains a, and focus_request is None. Unfolded a and visible nonmember c remain valid controls. Rendered DOM focus movement, native keyboard input and GPU paint were not exercised.',
 'mere_view_state_probe_v2_receipt.json',
 'Provide hover/focus inputs before the final fold transform and avoid overwriting transformed focus. Verify both summary and member stand-ins through the actual component host before claiming UI acceptance.'),
 ('Q288','Hand drafts survive replacement by a different merge with the same conflict key',
 'crates/cambium/mere-view/src/view.rs:338; crates/cambium/mere-view/src/view.rs:1493',
 'verified original current Mere-view crate over hash-bound retained dependency libraries',
 'MereViewState retains merge_drafts solely by conflict key, without the pending MergeModel source or generation. Replacing Some(merge from branch 1) directly with Some(merge from branch 2), sharing a title conflict key but different values, preserves the old hand draft. The renderer prefers that draft over the new target/chosen-hand seed and uses it in the Edit by hand request. Public state/event methods reproduce retention; same-merge continuity, a different key, and None cleanup pass as controls. Actual rendered button invocation and durable owner merge application were not executed.',
 'mere_view_state_probe_v2_receipt.json',
 'Bind drafts to a pending merge identity and conflict evidence; clear or explicitly rebase them when that identity changes. Preserve ordinary same-plan editing continuity.'),
]
findings=[{'id':i,'title':t,'source':s,'status':st,'evidence':e,'verification':NAME,'diagnostics':[d],
 'repair_triage':{'disposition':'documented; no production changes','reason':r}} for i,t,s,st,e,d,r in rows]
notes['findings'].extend(findings)
arows=[
 ('Shared route wire authority','Mere-overmap owns the shared request and media vocabulary, while each receiver must bind it to the requested scope.',
  ['crates/cambium/mere-overmap/src/wire.rs','crates/cambium/mere-overmap/Cargo.toml'],
  'All five package sources are read. Wire-only hosts avoid the optional Cambium presentation cone. SessionGraphV1 validates schema and checked kernel hydration with facets; remote path parsing combines named scope with grant checks. These do not independently prove reply/session/revision admission in every consumer.'),
 ('Display fields versus action authority','Human-labelled facet text is not a canonical machine address.',
  ['crates/cambium/mere-overmap/src/route.rs','crates/cambium/mere-overmap/src/route_session.rs','ports/graphshell/src/bin/graphshell_mere.rs'],
  'Q286 verifies arbitrary field precedence over canonical Address. Shared and native route adapters repeat first-label interpretation, so a shared schema alone does not enforce a single action-identity authority.'),
 ('Mere component versus graph ownership','Mere-view receives a model and emits requests; it does not own the durable graph or session store.',
  ['crates/cambium/mere-view/src/model.rs','crates/cambium/mere-view/src/view.rs','crates/cambium/mere-view/src/layout.rs'],
  'All 23 package sources are read. Layout builds a temporary adjacency graph and delegates registered strategies. Unknown strategy fallback is explicit and tested. Local editing, synthetic harnesses and remote owner orchestration remain distinct; the model/request component is genuinely shared without making those adapters one implementation.'),
 ('Projection focus ownership','The final fold transform must receive the focus it promises to remap.',
  ['crates/cambium/mere-view/src/folds.rs','crates/cambium/mere-view/src/view.rs','mere_view_state_probe_v2_receipt.json'],
  'Q287 verifies caller ordering defeats the projector remap. Fold boundary aggregation clears source LinkKey and marks instead of inventing evidence. Layout still includes all members; presentation folding does not rewrite source graph truth.'),
 ('Pending-merge presentation state','Conflict keys alone do not identify the merge to which an edited value belongs.',
  ['crates/cambium/mere-view/src/model.rs','crates/cambium/mere-view/src/view.rs','mere_view_state_probe_v2_receipt.json'],
  'Q288 verifies cross-branch draft retention. The widget stores presentation drafts, while choice/application remains a host request. Neither request emission nor a model saying chosen proves a durable successful merge.'),
 ('Acceptance surface scope','Headless semantic and retained-command checks are narrower than rendered host acceptance.',
  ['crates/cambium/mere-view/examples/harness.rs','crates/cambium/mere-view/examples/fold_acceptance.rs','crates/cambium/mere-view/tests/host_routing.rs','crates/cambium/cambium/examples/component_catalog.rs'],
  'The complete Cambium catalog/CSS, Mere-view tests, twelve scenarios and examples are read, not run here. Catalog runner tests route synthetic input and retained paint commands; HTML receipts omit custom-leaf pixels. Fold acceptance hand-builds the Swatch callbacks rather than exercising MereView fold wiring. Its snapshot comparison omits external facet sidecars. Current diagnostics only compile Mere-view and inspect public state/Swatches.'),
]
entries=[{'id':f'A{726+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,'state':'current source reviewed with qualified bounded probes','verification':NAME} for i,(d,c,im,a) in enumerate(arows)]
authority['entries'].extend(entries)
for follow in notes.get('review_followups',[]):
 if follow.get('area')=='Mere generic record values':
  follow['resolution']={'finding':'Q286','verification':NAME,'scope':'Facet precedence verified; saved-view consumer source trace, no durable owner action run.'}
notes.setdefault('review_followups',[]).append({'verification':NAME,'area':'Mere-view oversized tier cards',
 'detail':'Card height includes all archive chips while graph footprint clamps to canvas and container CSS clips overflow; inspect actual current host geometry and reachability before asserting unavailable archive controls. No rendered experiment yet.'})
next_read={'path':'crates/cambium/cambium/src/','start':0,'reason':'Read the remaining Cambium toolkit source, then rootstock and native/web hosts; full Mere source remains incomplete.'}
notes['objective']='Full-source audit of Mere with file-by-file coverage, duplicated authority and misleading success findings. Preserve concurrent work; no production edits under the active goal.'
notes['current_area']='Mere-overmap and Mere-view complete; Cambium catalog complete'
notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':[NAME,'route_facet_probe_receipt.json','mere_view_state_probe_receipt.json','mere_view_state_probe_v2_receipt.json'],
 'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Facet probe 3 checks and current original Mere-view 7 checks pass over retained dependency libraries. Initial consumer compile failed due to Rust rlib filename prefix; preserved and repaired in v2. No new full-current Cargo or GUI gate.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True); assert diff.returncode==0
with (HERE/'mere_view_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_mere_view_review.py','run_route_facet_probe.py','route_facet_probe.rs','route_facet_probe_receipt.json','route_facet_probe_compile.log','route_facet_probe_output.log',
 'mere_view_state_probe.rs','run_mere_view_state_probe.py','mere_view_state_probe_receipt.json','mere_view_state_probe_compile_0.log','mere_view_state_probe_compile_1.log',
 'run_mere_view_state_probe_v2.py','mere_view_state_probe_v2_receipt.json','mere_view_state_probe_v2_compile_0.log','mere_view_state_probe_v2_compile_1.log','mere_view_state_probe_v2_output.log','mere_view_diff_check.txt']
addition=f'''

### Shared route and Mere-view completion, 2026-10-17

All five Mere-overmap source files and all 23 Mere-view source files are now read, together with Cambium's complete component catalog, stylesheet and manifest. This pass adds 29 complete files and 409,540 reviewed characters. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters at unchanged merged HEAD {head}. There are no changed inventory hashes or pending chunks. The full-source objective remains active and incomplete.

Q286 records an action-identity collision: add_node_facets prepends arbitrary human-labelled facet fields, while shared and native saved-view adapters take the first Address label for ViewIntent.focus. Exact current functions over real retained facet/card types reproduce a facet shadowing the still-present canonical address; no-facet and lowercase controls pass. The saved-view payload consumer is source-traced, but no owner SetView action, durable wrong focus, GUI exploit or authorization bypass was run.

Q287 records a projection wiring defect. MereView applies its fold projector before assigning focus, then overwrites the projector's result with state.focused. A valid fold removes a focused member from the Swatch while leaving focus on that removed key and supplying no stand-in focus request. Q288 records merge draft identity loss: replacing a pending merge directly with another branch's merge sharing a conflict key retains the old hand draft. The renderer uses that draft instead of the new target seed for its hand-choice request. State/event probes verify both observations and their ordinary no-fold, visible-node, same-merge, different-key and absent-merge controls.

The [view checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds three successful facet checks and seven successful public Mere-view checks. The entire original current Mere-view crate was freshly compiled, over hash-bound dependency libraries retained from an earlier successful Graphshell build. The initial downstream compile failed because the audit rlib filename lacked Rust's required lib prefix; its logs and receipt remain intact, and the corrected v2 succeeds. This is not a fresh full-current Cargo graph, rendered DOM, native input, GPU, network, persistence or owner merge-apply receipt. All diagnostic rlibs, executables and PDBs were removed after their identities were recorded.

A726-A731 distinguish shared wire vocabulary, machine address authority, model/request boundaries, final projection focus, merge draft identity and acceptance evidence. Mere-view genuinely shares a presentation component and registered layouts; that does not unify native and remote owner orchestration. Read acceptance sources cover synthetic runner input and retained paint commands. The fold acceptance example hand-builds Swatch callbacks and compares graph snapshots without external facet sidecars; those checks do not establish the actual MereView fold wiring or all durable truth. The tests, scenarios and component catalog were read, not executed here.

The possible oversized tier-card reachability issue remains a follow-up, pending current host geometry evidence. No production code was edited. Shared dirty source remains byte-identical, and the previous audit document remains an exact byte prefix. No isolated target, Cargo home or worktree was created. Next is remaining Cambium toolkit source, rootstock and native/web host ownership.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Completed shared route/view reading and bounded diagnostics; full source incomplete',
 'prior_receipt':{'path':'upstream_175c_review_receipt.json','sha256':sha(HERE/'upstream_175c_review_receipt.json')},
 'coverage':stats,'whole_current_files':whole,'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,
 'shared_working_file_sha256':old['shared_working_file_sha256'],'shared_working_file_preservation_scope':'All non-audit shared files byte-identical; the original audit document is preserved as exact prefixes through appended checkpoints.',
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,
 'diagnostics':{'route_facet_probe_receipt.json':3,'mere_view_state_probe_v2_receipt.json':7},'retained_failed_diagnostic':'mere_view_state_probe_receipt.json',
 'pending_verification':None,'next_read':next_read,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Current original Mere-view compiled over exact retained dependency artifacts. Facet helper extraction is verbatim. No fresh full-current Cargo gate, DOM, headed host, GPU, network or durable owner action.'}
save(NAME,receipt)
assert sha(doc)==receipt['document']['sha256'] and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256']}.items())
assert all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
