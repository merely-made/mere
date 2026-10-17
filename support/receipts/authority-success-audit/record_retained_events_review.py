"""Checkpoint retained input/focus and pane-preservation review without product edits."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='retained_events_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('cambium_commands_review_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==1514
assert all(sha(ROOT/n)==h for n,h in guards.items()) and all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index;index_hash=sha(index)
inv,cov=load('source_inventory.json'),load('read_coverage.json');assert inv['commit']==head and load('pending_read.json')==[]
whole=sorted(n for n,v in inv['files'].items() if n.startswith('crates/cambium/cambium/src/') and cov.get(n,{}).get('sha256')==v['sha256'] and cov[n]['through']==v['chars'] and n not in guards)
assert len(whole)==17 and sum(inv['files'][n]['chars'] for n in whole)==311125
assert all(sha(ROOT/n)==inv['files'][n]['sha256'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==2001 and stats['read_chars']==23748399 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probes={n:load(n) for n in ['retained_events_probe_receipt.json','retained_events_probe_v2_receipt.json']}
assert probes['retained_events_probe_receipt.json']['checks']==7 and probes['retained_events_probe_v2_receipt.json']['checks']==17
for p in probes.values():
 assert p['build_exits']==[0,0] and p['run_exit']==0 and p['generated_output_removed']
 assert all(not Path(o['path']).exists() for o in p['removed_outputs'])
 assert all(sha(ROOT/n)==h for n,h in p['source_sha256'].items()) and all(sha(Path(v['path']))==v['sha256'] for v in p['libraries'].values())
notes,authority=load('review_notes.json'),load('authority_map.json');assert notes['findings'][-1]['id']=='Q292' and authority['entries'][-1]['id']=='A738'
rows=[
 ('Q293','Retained click listeners ignore changes to capture mode while key listeners update correctly',
 'crates/cambium/cambium/src/event.rs:243; crates/cambium/cambium/src/key.rs:331',
 'OnClick::rebuild refreshes its registration only when the node or routing path changes, omitting self.capture != prev.capture. With the same live nodes, changing an ancestor from bubble to capture leaves child,parent click order. A freshly built capture listener produces parent,child, and the parallel OnKey::rebuild correctly updates capture mode in place. The original current toolkit runner/wrappers reproduce all three cases. The registration logic was copied between sibling wrappers and has drifted; no native platform input or application action was exercised.',
 'Reconcile capture as part of click listener registration identity, using the same changed-field discipline already implemented by OnKey. Keep ordinary node/path retention and multi-listener removal intact.'),
 ('Q294','Frisket promises pane-state preservation that only holds in one tree-order direction',
 'crates/cambium/cambium/src/frisket.rs:33,289,946; crates/cambium/cambium/src/portable.rs:25',
 'PortableKeyed explicitly documents source-before-target preservation; target-before-source builds fresh and later tears down the parked original. Frisket advertises cross-stack DOM/view-state preservation without that caveat, and its test explicitly chooses earlier source then later target. A fresh current toolkit probe moves an active tile with component-local count=1 between two non-collapsing stacks. Left-to-right preserves the same node/count; right-to-left retires the old node and initializes count=0. This verifies local presentation-state loss, not loss of a persisted source document. Multi-window behavior and layout rendering were not exercised.',
 'Document the actual bound at the Frisket contract now. Order-independent preservation needs a coordinated pre-park/adoption or retained-mount design; keep that larger repair separate from a one-direction passing receipt. Broad lifetime/ownership repair is deferred under the user audit-diversion limit.'),
 ('Q295','Composed focus observers silently replace the inner handler on the same element',
 'crates/cambium/cambium/src/context.rs:register_focus; crates/cambium/cambium/src/focus.rs:50,79',
 'OnFocus is an ElementView and accepts another OnFocus as its child, but both register the same node in a single HashMap<NodeId,path>. Building the outer observer overwrites the inner route. A current toolkit runner focus/blur cycle delivers only the outer handler twice, while both typed observers remain in the view tree. The module describes one handler per child, so the verified issue is accepted silent wrapper composition with a lost inner observer, not a documented multi-listener guarantee. No existing product consumer requiring nested focus observers was established.',
 'Either support a per-node observer list with path-specific teardown, as click/key already do, or make the single-observer composition limit explicit and enforce it rather than silently overwriting.'),
 ('Q296','stop_propagation suppresses same-node listeners despite the shared cancellation contract',
 'crates/cambium/cambium/src/propagation.rs:108; crates/cambium/cambium/src/runner.rs:344,519,711',
 'Propagation says ordinary stop allows the current node other listeners, while immediate stop also suppresses them. RunnerTree flattens node/phase groups to paths, then breaks on stopped() after each listener in both click and key dispatch. The immediate flag has no distinct dispatcher consumption. Current toolkit probes with two same-node listeners plus an ancestor show all three without cancellation, but only the first with either ordinary or immediate stop. The stale comment claiming at most one listener per node contradicts the current vector registries. Only native toolkit synthetic dispatch was executed; no JavaScript dispatcher parity run was made.',
 'Retain node/phase grouping through dispatch and distinguish ordinary stop from immediate stop when deciding whether to continue same-node listeners. Test both event types against the declared contract before claiming convergence.'),
 ('Q297','View-requested focus updates the focus seat without notifying focus observers',
 'crates/cambium/cambium/src/runner.rs:153,230,372,565; crates/cambium/cambium/src/focus_request.rs:38',
 'Direct set_focus and click focus call route_focus_transition, but build/rebuild consumption of FocusRequest assigns self.focus directly. With a current request_focus/on_focus widget initially unfocused, changing requested to true seats focus on the live root and leaves the observer log empty. Direct set_focus(None) and set_focus(root) then emit Lost and Gained as a control. This contradicts on_focus saying programmatic focus changes emit transitions and leaves observer-maintained state distinct from the actual focus seat. The rebuild counterexample was executed; initial-build notification absence is source-traced. No headed focus or accessibility adapter was run.',
 'Commit all focus-seat changes through one transition/publication path, including deferred view requests, with a bounded reconciliation rule for handlers that request another focus change. Broader event/rebuild ordering needs care; record rather than introduce a second local workaround.'),
 ('Q298','Collapsed disclosure actions remain in Tab traversal and can activate while hidden',
 'crates/cambium/cambium/src/disclosure.rs:119; crates/cambium/cambium/src/runner.rs:1536',
 'Disclosure keeps its child controls mounted under a hidden=true panel. collect_focusables walks every descendant carrying a key/focusability registration without pruning authored hidden ancestors. In the current toolkit runner a closed disclosure trigger Tabs directly to its hidden action, and Space invokes that action while the panel remains closed. Opening the panel makes the same traversal an ordinary visible-control case. This is an actual component plus synthetic native-runner keyboard path, not a headed rendering, screen-reader or backend authorization receipt. Computed-style visibility is a separate host concern; the counterexample only relies on the authored hidden attribute.',
 'Apply authored hidden/inert ancestry at the common focus eligibility boundary and keep computed-style visibility supplied by the host. Test collapsed/expanded controls through actual focus traversal, including transitions that hide a focused subtree.'),
]
findings=[{'id':i,'title':t,'source':s,'status':'verified original current Cambium crate over hash-bound retained dependency artifacts','evidence':e,'verification':NAME,'diagnostics':['retained_events_probe_v2_receipt.json'],
 'repair_triage':{'disposition':'documented; no production changes','reason':r}} for i,t,s,e,r in rows];notes['findings'].extend(findings)
arows=[
 ('Disclosure and focus eligibility','Shared disclosure rendering does not currently share a visibility-aware keyboard eligibility rule.',
 ['crates/cambium/cambium/src/disclosure.rs','crates/cambium/cambium/src/runner.rs'],
 'Accordion and recursive trees reuse disclosure_control, and trees retain separate active versus selected identity. Generic disclosure content stays mounted with hidden semantics; Q298 proves the runner still traverses/activates its hidden controls. Reuse of a control renderer does not establish a single effective focus authority.'),
 ('Borrowed source fold provenance','A byte-length witness validates length and range structure, not source identity or revision.',
 ['crates/cambium/cambium/src/fold_projection.rs'],
 'Projection borrows source and never maintains another editable buffer. It rejects malformed/crossing byte ranges, collapses nesting and clips valid styles. Equal-length source replacements are not excluded by a length witness; no live stale-source consumer defect was established. Row text normalizes emitted line breaks to newline while line metadata retains original CRLF spans.'),
 ('Pane frame and retention claims','Frisket shares the view and hit-semantic vocabulary, while the caller retains its authoritative TileTree.',
 ['crates/cambium/cambium/src/frisket.rs','crates/cambium/cambium/src/portable.rs','crates/cambium/cambium/src/keyed.rs'],
 'The frame delegates tab rendering and emits TileEvents; it does not mutate owner tree truth. Q294 qualifies its broad preservation claim with an executed direction-dependent local-state reset. Keyed same-parent single-element movement and PortableKeyed cross-parent nursery adoption have related but different supported contracts, not an interchangeable guarantee.'),
 ('Listener reconciliation drift','Click and key wrappers share routing vocabulary but independently reconcile listener metadata.',
 ['crates/cambium/cambium/src/event.rs','crates/cambium/cambium/src/key.rs'],
 'Both register node/path/capture metadata in one GenetCtx. Key also reconciles capture/focusability changes; click omits capture updates. Q293 proves that source similarity and common registry ownership do not give equal behavior.'),
 ('Focus transition authority','A single stored focus seat has several update implementations with different observer behavior.',
 ['crates/cambium/cambium/src/focus.rs','crates/cambium/cambium/src/focus_request.rs','crates/cambium/cambium/src/runner.rs'],
 'Q295 verifies silent same-node observer replacement. Q297 verifies deferred request consumption bypasses Lost/Gained routing that direct setters and clicks perform. Focusable registration refcounts and one-shot request edges are real shared machinery, but they do not consolidate transition publication.'),
 ('Cancellation and dispatch contract','Shared cancellation flags are weaker than a shared native/JavaScript dispatch implementation or proof.',
 ['crates/cambium/cambium/src/propagation.rs','crates/cambium/cambium/src/runner.rs'],
 'The native documentation claims one cancellation contract with the JavaScript side. Q296 proves native ordinary stop and immediate stop behave identically for same-node listeners despite the stated distinction. The exact JavaScript supplier implementation was not rerun here, so this is a local contract contradiction rather than a new cross-runtime parity receipt.'),
 ('File requests and compatibility menus','Explicit host-owned effects and legacy click renderers must keep their narrower contracts visible.',
 ['crates/cambium/cambium/src/file.rs','crates/cambium/cambium/src/menu.rs','crates/cambium/cambium/src/lib.rs'],
 'File requests use a documented latest-wins slot and a registered answer route; request/answer identity and single-path observer limitations remain host-trace follow-ups. Legacy menu renders click-only rows for editor-owned completion state instead of delegating to command_surface; its docs explicitly reserve keyboard ownership to callers. It is not the same kind of compatibility delegation as action_list. Deprecated Serval names are re-exports, not separate backends.'),
]
entries=[{'id':f'A{739+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,'state':'current source read with bounded current-crate diagnostics','verification':NAME} for i,(d,c,im,a) in enumerate(arows)];authority['entries'].extend(entries)
for f in notes.get('review_followups',[]):
 if f.get('area') in ['Context single-path observer composition','Context single-path observer composition and portable keys']:
  f['resolution']={'finding':'Q295','verification':NAME,'remaining':'Global portable-key namespace/collision behavior still requires a supported consumer trace.'}
next_read={'path':'crates/cambium/cambium/src/graph_canvas.rs','start':0,'reason':'Continue unread graph-canvas, grids, overlay/input wrappers and remaining toolkit source; then rootstock and native/web hosts and all remaining Mere source.'}
notes['current_area']='Cambium retained events, disclosure, pane movement and focus reviewed';notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,*probes],'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Fresh default-feature original Cambium compiled over retained dependencies. Initial seven-check probe and extended seventeen-check probe both succeed; extended checks include earlier seven. No full-current Cargo graph or headed host gate.'}
notes.setdefault('review_followups',[]).append({'verification':NAME,'area':'Remaining host custody traces','detail':'Read current native/web/rootstock and multi-runner consumers for focus eligibility, file-request response binding, pane mount lifetime and global portable-key namespaces. Toolkit observations do not yet prove all host effects.'})
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'retained_events_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_retained_events_review.py','retained_events_diff_check.txt']
for n in ['retained_events_probe','retained_events_probe_v2']:
 evidence += [n+'.rs','run_'+n+'.py',n+'_receipt.json',n+'_compile_0.log',n+'_compile_1.log',n+'_output.log']
addition=f'''

### Retained input, focus and pane-state preservation, 2026-10-17

Seventeen more toolkit sources / 311,125 characters are completely read, including disclosure, element and event wrappers, file/focus helpers, source-fold projection, Frisket, keyed/portable sequences, the runner, propagation, exports and legacy menus. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters at unchanged HEAD {head}, with no stale inventory hashes or pending chunk. Full Mere remains incomplete.

Q293 proves sibling event-wrapper drift: changing capture on a retained click listener is ignored, while the parallel key listener updates correctly. Initial capture and bubble controls behave as declared. Q294 proves that Frisket's cross-stack state-preservation claim exceeds PortableKeyed's documented source-before-target limit. An active tile carrying component-local count 1 keeps its node/count on an earlier-to-later move, but a later-to-earlier move retires the original node and starts at count 0. Both stacks keep spare tabs, so tree collapse is not the explanation. No persisted document loss is claimed. The existing Frisket test explicitly selects the preserving direction; that is a bounded receipt, not proof of both directions.

Q295 records accepted nested focus wrappers silently retaining only the outer observer. Q296 proves that both click and key dispatch stop all remaining paths after ordinary stop_propagation, including other listeners on the same node, contrary to the declared ordinary-versus-immediate distinction. The old at-most-one-listener comment survived the registry conversion to per-node vectors. No current JavaScript dispatcher was executed, so this is a native contract failure rather than a new cross-runtime parity result.

Q297 proves that a view-requested focus change seats focus without the Gained notification delivered by the direct setter; deferred request consumption bypasses the shared transition helper. Q298 proves that a collapsed disclosure's mounted hidden action remains in Tab traversal and Space invokes it while the panel stays closed. The expanded control behaves normally. The counterexample uses the authored hidden attribute, not a claim about every CSS/computed-style visibility rule or an authorization bypass.

The [retained-event checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds a fresh original current default-feature Cambium compile against twelve exact retained dependency artifacts. The initial seven-check probe and expanded seventeen-check probe both pass; the expanded set includes those first seven, so these are seventeen distinct checks, not twenty-four independent cases. They use the actual current runner, components, wrappers, disclosure and Frisket through synthetic keyboard/pointer input and inspect DOM identity/local state. No native window, OS input, GPU, accessibility adapter, network, persisted document or fresh full-current transitive Cargo graph was exercised. Compiling source does not count as reading it; unread toolkit files remain unread in the ledger. Both fresh rlibs and their executables/PDBs were identified, hashed and removed.

A739-A745 record effective focus eligibility, source-fold provenance, frame versus tree ownership, listener reconciliation drift, transition publication, cancellation-contract limits and explicit host/compatibility boundaries. FoldProjection borrows the source and validates byte lengths/ranges; a length witness is not a revision/identity witness. Legacy menu is a declared click-only renderer with caller-owned editor keyboard state, while old action-list names actually delegate to the shared command engine. Deprecated backend names are aliases rather than duplicate implementations.

No production code or shared WIP was changed, the prior audit document remains an exact prefix, and the Git index is unchanged. No isolated target, Cargo home or worktree was created. Broad order-independent pane retention and focus-publication repairs stay documented rather than diverting the audit. Next is unread graph-canvas, grids, overlay/input wrappers and remaining toolkit code, then rootstock and native/web hosts. The full-source goal stays active.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Retained input/focus/frame reading and verified current-crate counterexamples; full source incomplete',
 'prior_receipt':{'path':'cambium_commands_review_receipt.json','sha256':sha(HERE/'cambium_commands_review_receipt.json')},'coverage':stats,'whole_current_files':whole,
 'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,'shared_working_file_sha256':old['shared_working_file_sha256'],
 'shared_working_file_preservation_scope':'All non-audit shared files exact; audit document preserved through chained exact prefixes.',
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,
 'diagnostics':{n:{'build_exits':p['build_exits'],'run_exit':p['run_exit'],'checks':p['checks']} for n,p in probes.items()},'distinct_checks_in_extended_probe':17,
 'pending_verification':None,'next_read':next_read,'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'git_index_sha256':index_hash,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Current default-feature original Cambium crate compiled over exact retained artifacts, not a new full-current supplier/Cargo graph or headed host acceptance. Compilation does not count as reading unread source.'}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256']}.items()) and all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
