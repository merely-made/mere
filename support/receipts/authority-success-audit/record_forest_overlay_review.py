"""Checkpoint forest move/overlay evidence and adjacent source-reading coverage."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='forest_overlay_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('graph_canvas_review_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==1537
assert all(sha(ROOT/n)==h for n,h in guards.items()) and all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index;index_hash=sha(index)
inv,cov=load('source_inventory.json'),load('read_coverage.json');assert inv['commit']==head and load('pending_read.json')==[]
whole=sorted(n for n,v in inv['files'].items() if n.startswith('crates/cambium/cambium/src/') and cov.get(n,{}).get('sha256')==v['sha256'] and cov[n]['through']==v['chars'] and n not in guards)
assert len(whole)==10 and sum(inv['files'][n]['chars'] for n in whole)==143052
assert all(sha(ROOT/n)==inv['files'][n]['sha256'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==2017 and stats['read_chars']==24083660 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probe=load('forest_overlay_probe_v2_receipt.json');failed=load('forest_overlay_probe_failure_receipt.json');discard=load('retained_mount_truncated_read.json')
assert probe['checks']==7 and probe['build_exits']==[0,0] and probe['run_exit']==0
assert probe['generated_output_removed'] and all(not Path(o['path']).exists() for o in probe['removed_outputs'])
assert all(sha(ROOT/n)==h for n,h in probe['source_sha256'].items()) and all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
assert failed['build_exits']==[0,1] and failed['run_exit'] is None and failed['checks']==0 and failed['generated_output_removed']
assert sha(HERE/'forest_overlay_probe_failed_long_type.txt')==failed['long_type_sha256']
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-forest-overlay-probe.*'))
assert discard['unacknowledged_read'][0]['path'].endswith('/retained_mount.rs')
notes,authority=load('review_notes.json'),load('authority_map.json');assert notes['findings'][-1]['id']=='Q300' and authority['entries'][-1]['id']=='A752'
rows=[
 ('Q301','Forest projections replace moved view state despite claiming cross-window preservation',
 'crates/cambium/cambium/src/multi.rs:31,132,203; crates/cambium/cambium/src/runner.rs:214',
 'push_forest_projection shares one ScriptedDom but creates a separate RunnerTree/GenetCtx for each window. Each rebuild drains its own portable nursery before the next projection runs. Sharing the document therefore does not make parked children claimable across contexts. In current original toolkit probes, moving the same stateful Frisket tile from projection 0 to 1 or from 1 to 0 updates shared owner truth, retires the original node and resets component-local count from 1 to 0. An unchanged-owner refresh preserves both, and the distinct RetainedMount path preserves its actual runner/node/count across a destination-first mount. The forest source tests only establish sibling roots and shared counter refresh, not the advertised cross-window preservation. No native windows, scroll or animation were exercised.',
 'Qualify the forest API documentation to its actual shared-document/single-state guarantee and route retained owner moves through the tested RetainedMount contract. General cross-context portable adoption would require coordinated custody and lifecycle changes; defer that broad repair under the audit-diversion limit. This is broader than Q294 single-tree ordering loss.'),
 ('Q302','The modal overlay flag advertises modal isolation while background controls remain keyboard-active',
 'crates/cambium/cambium/src/overlay_surface.rs:101,171; crates/cambium/cambium/src/runner.rs:1506',
 'OverlaySurface::modal(true) adds a modal class and, for Dialog, aria-modal=true; it does not establish a focus scope or make background controls inert. A current toolkit runner renders a background button and this modal dialog, focuses the inside button, Tabs to the background, and Space invokes its callback while aria-modal remains true. A nonmodal control keeps the same background reachability without that ARIA claim. No in-tree .modal(true) product call site was found in the searched Cambium/ports Rust source, so this is a verified public API/semantic gap rather than a demonstrated deployed modal bypass. No headed host or accessibility bridge was executed.',
 'Either implement modality through a shared focus/input scope owned by the host/runner, or expose this flag as semantics-only with an explicit required host contract. Avoid having each overlay invent another partial modal controller; broader focus-scope design stays documented.'),
]
findings=[{'id':i,'title':t,'source':s,'status':'verified original current default-feature toolkit over retained exact dependency artifacts','evidence':e,'verification':NAME,'diagnostics':['forest_overlay_probe_v2_receipt.json'],
 'repair_triage':{'disposition':'documented; no production changes','reason':r}} for i,t,s,e,r in rows];notes['findings'].extend(findings)
arows=[
 ('Shared state versus retained window custody','GenetMultiRunner really owns one State, but each projection still owns a separate context/nursery.',
 ['crates/cambium/cambium/src/multi.rs','crates/cambium/cambium/src/runner.rs','crates/cambium/cambium/src/context.rs'],
 'Shared-state updates fan through RunnerTree rather than a second dispatch engine. N-doms and forest projections differ in DOM topology, not context ownership. Q301 proves the documented forest preservation claim is stronger than the implementation; insertion order alone cannot connect the per-tree nurseries.'),
 ('Retained owner mount authority','RetainedMount moves an actual independently owned runner container, rather than reconstructing a slot child.',
 ['crates/cambium/cambium/src/retained_mount.rs','forest_overlay_probe_v2_receipt.json'],
 'The mount view forwards no messages and checks the same document. Withdrawal parks only a body still under the source wrapper, preserving destination-first transfers from stale teardown. The executed positive control preserves node/local state. Hosts still own current-station input checks, window-scoped layout and runner-before-final-mount destruction order; reading its other tests is not a full host custody gate.'),
 ('Overlay behavior is not one implementation','Detail popovers use OverlaySurface, but the general Popover implements its own dismissal and placement path.',
 ['crates/cambium/cambium/src/overlay_surface.rs','crates/cambium/cambium/src/detail_popover.rs','crates/cambium/cambium/src/popover.rs','crates/cambium/cambium/src/overlay.rs'],
 'OverlaySurface delegates measured flip/clamp placement to overlay geometry. Popover separately renders CSS-anchored panels, fixed dismissal layers, Escape handling and return-focus state; it does not call OverlaySurface. The shared-behavior-beneath-popovers wording therefore has a narrower actual scope. Their placement inputs differ legitimately, but dismissal/focus policy remains repeated and nested-overlay behavior needs a shared contract.'),
 ('Modal semantics versus effective input policy','aria-modal is a semantic assertion, not a focus/input isolation implementation.',
 ['crates/cambium/cambium/src/overlay_surface.rs','crates/cambium/cambium/src/runner.rs'],
 'Q302 verifies background Tab/Space activation while the toolkit advertises aria-modal. No product modal call site or headed/a11y failure was established; downstream host enforcement remains unreviewed. Ordinary edge-aware placement and outside/Escape dismissal are real implemented behavior.'),
 ('Opened-link records and action authority','The opened-link widget carries observed source context and assertion identity; hosts must re-resolve before effects.',
 ['crates/cambium/cambium/src/opened_link.rs','crates/cambium/scenes/scenomise/src/opened_link.rs'],
 'The view delegates opened scope/layout compilation to Scenomise and uses the compiled reading for both timeline and chronological fact list. Action targets retain source binding/revision, canonical link, source direction and local assertion id. Legacy rows have no assertion action. The test source preserves reversed bucket identities and excludes withdrawn actions through its supplied action policy; it was read, not rerun.'),
 ('Controlled radio/range inputs','Radio state remains caller-stored, and RangeScrubber routes pointer/key/value requests through one reconcile policy.',
 ['crates/cambium/cambium/src/radio.rs','crates/cambium/cambium/src/range_scrubber.rs'],
 'Radio handles its own roving focus and selection requests. RangeScrubber genuinely shares clamp/quantization among input routes and refuses disabled/nonfinite requests; endpoint and disabled-focus source tests are read. Finite endpoint arithmetic overflow and Cancel-preview semantics still need bounded checks before any stronger numeric/cancellation claim.'),
 ('Uniform DOM element contract','GenetElement uses one NodeId/document representation, while replacements remain explicit DOM lifetime changes.',
 ['crates/cambium/cambium/src/pod.rs','crates/cambium/cambium/src/instrument.css'],
 'SuperElement up/downcast is identity rather than a second node-erasure layer; AnyElement replacement inserts the new node and removes the old under a known parent. Instrument styling derives workspace colors/type from host variables. These contracts do not confer storage, runner custody or input authority.'),
]
entries=[{'id':f'A{753+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,'state':'current source read with qualified forest/modal checks','verification':NAME} for i,(d,c,im,a) in enumerate(arows)];authority['entries'].extend(entries)
notes.setdefault('review_followups',[]).extend([
 {'verification':NAME,'area':'Overlay family convergence','detail':'General Popover independently implements dismissal/return focus while DetailPopover delegates OverlaySurface. Trace nested overlays and native fixed-position behavior before asserting topmost-only dismissal or full-window interception.'},
 {'verification':NAME,'area':'Controlled range arithmetic and cancellation','detail':'is_valid accepts finite min/max without checking finite max-min; inspect overflow behavior for extreme finite ranges and Preview emission on PointerPhase::Cancel. No numeric or cancel counterexample executed this pass.'}])
next_read={'path':'crates/cambium/cambium/src/reorderable_list.rs','start':0,'reason':'Read remaining toolkit reorder/resize/selection/settings/slider/splice/surface/workspace modules and tests, then rootstock and native/web hosts and all remaining Mere source.'}
notes['current_area']='Multi-runner, retained mount, overlay families and controlled radio/range reviewed';notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,'forest_overlay_probe_v2_receipt.json','forest_overlay_probe_failure_receipt.json','retained_mount_truncated_read.json'],'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Seven current-crate forest/retained-mount/modal checks pass. Initial audit fixture failed Action inference; generated long-type compiler text is retained in evidence and target output removed. A truncated source emission was discarded without ACK and re-emitted in complete smaller chunks.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'forest_overlay_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_forest_overlay_review.py','forest_overlay_diff_check.txt','retained_mount_truncated_read.json','forest_overlay_probe.rs','run_forest_overlay_probe.py','forest_overlay_probe_failure_receipt.json','forest_overlay_probe_compile_0.log','forest_overlay_probe_compile_1.log','forest_overlay_probe_failed_long_type.txt',
 'forest_overlay_probe_v2.rs','run_forest_overlay_probe_v2.py','forest_overlay_probe_v2_receipt.json','forest_overlay_probe_v2_compile_0.log','forest_overlay_probe_v2_compile_1.log','forest_overlay_probe_v2_output.log']
addition=f'''

### Forest window custody and overlay behavior, 2026-10-17

Ten more toolkit sources / 143,052 characters are completely read: instrument styling, multi-runner, opened links, overlay geometry/surfaces, uniform DOM elements, popovers, radio/range controls and retained mounts. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters at unchanged HEAD {head}, without stale hashes or pending chunks. A retained-mount emission exceeded its output budget; that pending chunk was explicitly discarded without credit and completely re-emitted in two smaller acknowledged chunks. The full-source goal remains incomplete.

Q301 verifies a false forest-preservation claim. Forest projections share one document, but each creates its own context and drains its portable nursery before the next projection rebuilds. Moving a stateful Frisket tile between projections replaces the original node and resets component-local count 1 to 0 in both directions, including source-before-target. Shared owner truth updates correctly and unchanged-owner refresh preserves state. The separate RetainedMount path preserves the actual owner runner/node/count across a destination-first transfer. Thus a shared document and one application State do not themselves establish cross-window view custody. Existing multi-runner source tests prove sibling roots and shared counter updates, not preservation of the moved child. This is broader than Q294 single-tree ordering loss.

Q302 verifies that the public modal overlay flag advertises aria-modal=true while leaving background controls in ordinary Tab traversal; Space activates the background callback. The nonmodal control has the same reachability without the modal assertion. No .modal(true) product call site was found in the searched in-tree Cambium/ports Rust code, so this is an API/semantic gap, not a deployed product bypass or headed accessibility result.

The [forest/overlay checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds seven successful focused checks through the original current default-feature toolkit compiled over twelve exact retained dependency artifacts. The first audit fixture failed Action type inference; its compiler logs and generated long-type text remain retained. Cleanup initially refused that extra compiler text; after the compiler exited it was marker-checked, copied into audit evidence and removed from the stable target. A distinct typed v2 succeeds. Its rlib/executable/PDB were recorded and removed. No native windows, scroll/animation preservation, layout, GPU, network, owner persistence, accessibility bridge or fresh full-current Cargo supplier graph was tested.

A753-A759 distinguish single shared app state from per-projection custody, the real independently owned RetainedMount path, repeated overlay implementations, semantic modality from input isolation, exact opened-link action context, controlled radio/range policy and one DOM element representation. General Popover separately implements anchored dismissal/Escape/focus return; DetailPopover uses OverlaySurface. The shared overlay-behavior claim is therefore narrower than the full family. RetainedMount explicitly leaves current-station input checks, scoped layout and destruction order to the host. Those host contracts remain to be read and verified.

Opened-link presentation delegates compilation to Scenomise and preserves source binding/revision, direction and assertion identity in effects requested from the host; it does not grant source-edit authority. RangeScrubber shares a real reconcile policy, with extreme finite arithmetic and cancel-preview behavior retained as unverified follow-ups. Existing source tests were read rather than broadly rerun. No production code or shared WIP was changed, prior audit text is an exact prefix and the Git index is unchanged. No isolated target, Cargo home or worktree was introduced. Broad cross-context preservation and modal focus policy remain documented under the audit-diversion rule. Next is the remaining toolkit modules/tests, then rootstock and native/web hosts and all other unread Mere source.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Forest/overlay custody audit with complete adjacent source reading; full source incomplete',
 'prior_receipt':{'path':'graph_canvas_review_receipt.json','sha256':sha(HERE/'graph_canvas_review_receipt.json')},'coverage':stats,'whole_current_files':whole,
 'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,'shared_working_file_sha256':old['shared_working_file_sha256'],
 'shared_working_file_preservation_scope':'All non-audit shared files exact; audit document preserved through chained exact prefixes.',
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,
 'diagnostics':{'forest_overlay_probe_v2_receipt.json':{'build_exits':probe['build_exits'],'run_exit':probe['run_exit'],'checks':probe['checks']}},'retained_failed_diagnostic':'forest_overlay_probe_failure_receipt.json',
 'reading_integrity':'Truncated unacknowledged emission discarded; source re-emitted and fully acknowledged. No skipped/truncated text credited.',
 'pending_verification':None,'next_read':next_read,'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'git_index_sha256':index_hash,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Current default-feature toolkit compiled over retained exact libraries. No new full-current supplier graph or native window/GPU/a11y/persistence gate. Source tests read, not broadly rerun.'}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256']}.items()) and all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
