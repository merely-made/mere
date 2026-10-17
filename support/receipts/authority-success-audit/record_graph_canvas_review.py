"""Record complete graph-canvas reading, identity checks and adjacent toolkit scope."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='graph_canvas_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('retained_events_review_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==1531
assert all(sha(ROOT/n)==h for n,h in guards.items()) and all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index;index_hash=sha(index)
inv,cov=load('source_inventory.json'),load('read_coverage.json');assert inv['commit']==head and load('pending_read.json')==[]
whole=sorted(n for n,v in inv['files'].items() if n.startswith('crates/cambium/cambium/src/') and cov.get(n,{}).get('sha256')==v['sha256'] and cov[n]['through']==v['chars'] and n not in guards)
assert len(whole)==6 and sum(inv['files'][n]['chars'] for n in whole)==192209
assert all(sha(ROOT/n)==inv['files'][n]['sha256'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==2007 and stats['read_chars']==23940608 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probe=load('graph_focus_probe_receipt.json');assert probe['checks']==9 and probe['build_exits']==[0,0] and probe['run_exit']==0
assert probe['generated_output_removed'] and all(not Path(o['path']).exists() for o in probe['removed_outputs'])
assert all(sha(ROOT/n)==h for n,h in probe['source_sha256'].items()) and all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
notes,authority=load('review_notes.json'),load('authority_map.json');assert notes['findings'][-1]['id']=='Q298' and authority['entries'][-1]['id']=='A745'
correction=load('retained_events_location_receipt.json')
assert next(f for f in notes['findings'] if f['id']=='Q298')['source'].endswith('runner.rs:1506')
assert next(f for f in notes['findings'] if f['id']=='Q296')['source'].startswith('crates/cambium/cambium/src/propagation.rs:104;')
rows=[
 ('Q299','Reordering graph nodes separates the focused logical identity from the keyboard activation target',
 'crates/cambium/cambium/src/graph_canvas.rs:1640,1983',
 'Node targets are an unkeyed Vec retained by position, while focus emphasis is retained as a logical Id. Reordering two existing logical nodes reuses the focused DOM node for the other Id without a focus transition. A current-toolkit probe focuses node 1 then swaps the graph nodes: the same DOM seat now represents node 2, node 1 still carries the focused class, and Enter emits activation for node 2. The callback-owned and component-owned APIs both reproduce it. Initial focus and an explicit blur/refocus to logical node 1 work as controls. The optional data-key is correctly emitted, but only automation metadata; its presence does not preserve view identity. No paint leaf, native input or persisted owner action was executed.',
 'Key retained node targets by logical identity, or explicitly reconcile the DOM focus seat when order/membership changes. Reconcile local emphasis against valid identities without silently transferring keyboard activation to a different node.'),
 ('Q300','Graph relation and Expand buttons are pointer actions missing from keyboard traversal',
 'crates/cambium/cambium/src/graph_canvas.rs:1575,1744',
 'Node buttons use focusable/on_focus, but relation targets and the Expand button use bare on_click. The current runner registers explicit focusability or key handlers rather than deriving Tab seats from HTML button tags. A current-toolkit probe finds both buttons and verifies their pointer callbacks, but neither is in focusables(); Tab cycles only between node buttons. This is absence from toolkit keyboard traversal, not a claim about every accessibility bridge or headed host. The existing graph-canvas source tests invoke these actions directly by pointer and do not test their Tab reachability.',
 'Use the shared clickable/button focusability path for relation and Expand actions, preserving relation hover, endpoint stacking and consumer action ownership. Verify complete Tab traversal and Enter/Space activation through the runner.'),
]
findings=[{'id':i,'title':t,'source':s,'status':'verified original current Cambium crate over hash-bound retained dependencies','evidence':e,'verification':NAME,'diagnostics':['graph_focus_probe_receipt.json'],
 'repair_triage':{'disposition':'documented; no production changes','reason':r}} for i,t,s,e,r in rows];notes['findings'].extend(findings)
arows=[
 ('Graph projection versus graph truth','The Swatch is a bounded projection; caller actions, graph storage and viewport policy remain external.',
 ['crates/cambium/cambium/src/graph_canvas.rs','crates/cambium/cambium/src/atlas.rs'],
 'Node positions and ordinary relation routes use shared viewport math for paint and native targets. Node footprints clip endpoints in pixel space; unknown endpoints are omitted. Atlas fields use their separate aspect-preserving world view and polygon pick path. Source tests cover selected geometries, not universal GPU/host parity. The original full graph_canvas source is now completely read.'),
 ('Logical versus retained input identity','Stable graph ids and automation keys do not make positional view retention stable.',
 ['crates/cambium/cambium/src/graph_canvas.rs','graph_focus_probe_receipt.json'],
 'Q299 verifies stale logical focus emphasis beside a DOM focus seat rebound to another node after reordering, through both API ownership variants. The shared model and one renderer still need explicit retained identity reconciliation.'),
 ('Graph action reachability','A native-looking button and a pointer callback do not establish keyboard reachability.',
 ['crates/cambium/cambium/src/graph_canvas.rs','crates/cambium/cambium/src/runner.rs'],
 'Q300 verifies missing Tab seats for relations and Expand, beside working pointer callbacks and keyboard-reachable nodes. Direct pointer tests cannot qualify the keyboard path. The low-level wrappers genuinely delegate to one full interaction surface rather than retaining separate graph algorithms.'),
 ('Component emphasis and paint refresh','Component-local DOM emphasis and caller-registered leaf paint are distinct update seams.',
 ['crates/cambium/cambium/src/graph_canvas.rs'],
 'graph_canvas stores focus/hover/relation emphasis locally; the callback-per-axis family instead lets a caller mirror those axes. Paint leaf registration/refresh remains host-owned. The executed probe checks DOM classes, focus seats and emitted events, not whether a host refreshes the leaf from equivalent emphasis. That consumer seam remains a follow-up, not a verified paint failure.'),
 ('Frame presentation reuse and repeated traversal','Presented Frisket shares stack rendering but independently walks split/rail/drawer presentation.',
 ['crates/cambium/cambium/src/frisket_presentation.rs','crates/cambium/cambium/src/frisket.rs'],
 'The separate walker renders derived fractions, minimum widths, rails and overlay drawers while delegating render_stack. Canonical TileTree remains caller-owned. Closed presented stacks omit their content subtree, unlike ordinary Disclosure hidden-but-mounted content in Q298. Source tests cover close/focus return and unchanged tree facts; they were read, not rerun here.'),
 ('Grid arrangement and retained row identity','GridSpec owns geometry and the cell callback owns content; row retention is still positional.',
 ['crates/cambium/cambium/src/grid.rs'],
 'The grid materializes a bounded VirtualWindow and publishes full ARIA counts, with caller-owned scrolling/sorting. Its rows and cells are unkeyed vectors. Stateful cell reuse/focus across a moving window requires a supported consumer or dedicated probe before promotion; the existing text/sparkline tests only check content and bounded DOM counts.'),
 ('Highlight vocabulary and callback action conversion','Highlighting shares one role-class map while action conversion uses a sealed result protocol.',
 ['crates/cambium/cambium/src/highlight.rs','crates/cambium/cambium/src/optional_action.rs','crates/cambium/cambium/src/hover.rs'],
 'The optional highlight assembly maps Illume lexer kinds to Tinct roles and uses one macro vocabulary for classes/CSS; unknown languages intentionally render plain source. It was read, but not compiled in the default-feature diagnostics. Action is an open opt-in marker; OptionalAction is sealed despite the introductory wording calling the marker sealed. Unit maps to no action rather than an operation-success claim. Hover uses a declared single path and shared runner routing/deferred-rebuild policy; nested-observer behavior remains separate from click/key multi-listener support.'),
]
entries=[{'id':f'A{746+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,'state':'current source read and qualified current-crate graph checks','verification':NAME} for i,(d,c,im,a) in enumerate(arows)];authority['entries'].extend(entries)
notes.setdefault('review_followups',[]).extend([
 {'verification':NAME,'area':'Virtualized grid state identity','detail':'Rows/cells are unkeyed Vec values. Inspect or reproduce stateful component/focus retention when VirtualWindow moves, including caller-managed sort identity; source text/sparkline receipts do not cover it.'},
 {'verification':NAME,'area':'Graph component emphasis versus retained leaf paint','detail':'Trace actual hosts registering/refeshing paint_leaf for graph_canvas component-local hover/focus. DOM-class changes are verified; no mismatched painted emphasis claim is made yet.'}])
next_read={'path':'crates/cambium/cambium/src/instrument.css','start':0,'reason':'Continue unread toolkit stylesheet, multi-runner, opened-link and overlay/input/retained-mount code, then rootstock and native/web hosts and all remaining Mere source.'}
notes['current_area']='Complete graph-canvas source and adjacent presented frame/grid/highlight/hover/action conversion reviewed';notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,'graph_focus_probe_receipt.json','retained_events_location_receipt.json'],'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Nine graph focus/reachability checks pass through a fresh current default-feature Cambium build over retained exact dependency artifacts. No full-current supplier graph, GPU or headed host gate.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'graph_canvas_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_graph_canvas_review.py','graph_canvas_diff_check.txt','retained_events_location_receipt.json','graph_focus_probe.rs','run_graph_focus_probe.py','graph_focus_probe_receipt.json','graph_focus_probe_compile_0.log','graph_focus_probe_compile_1.log','graph_focus_probe_output.log']
addition=f'''

### Graph-canvas identity and keyboard reachability, 2026-10-17

Six further toolkit sources / 192,209 characters are completely read, including every line of the 131,970-character graph_canvas module and its tests, presented Frisket, grid, optional highlighting, hover and callback action conversion. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters at unchanged HEAD {head}. There are no stale hashes or pending chunks; the full-source objective remains incomplete.

Q299 verifies that graph logical identity and native focus can separate after a node reorder. Target views are positional vectors, while focus emphasis is held by logical Id. Focus node 1, swap the two nodes, and the retained focused button now represents node 2 while node 1 still carries the focused class. Enter activates node 2. Both callback-owned and component-owned graph forms reproduce it, with initial focus and explicit blur/refocus controls. The data-key still identifies the displayed node; it is automation metadata and does not key retained view state.

Q300 verifies that relation and Expand actions work by pointer but are missing from toolkit Tab traversal. Those buttons use bare on_click, while node buttons use explicit focus registration. Tab cycles between nodes and never reaches either action; direct pointer callbacks work as controls. Source tests exercise relation/Expand pointer activation without establishing keyboard reachability.

The [graph checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds nine successful focus/reachability checks through a fresh original current default-feature Cambium compile over twelve exact retained dependency artifacts. The checks inspect actual current runner focus, DOM identity/classes, Tab traversal and emitted callbacks. They do not execute paint leaves, native input, GPU, accessibility bridges, owner persistence or a fresh full-current transitive supplier/Cargo graph. Optional highlight code was fully read but is not compiled by these feature-off diagnostics. The audit rlib, executable and PDB were recorded and removed.

A746-A752 distinguish shared graph geometry from source truth; logical ids from positional retention; pointer success from keyboard reachability; component-local emphasis from host leaf refresh; shared stack rendering from separately walked presentation structure; virtualized arrangement from row identity; and actual shared highlight/action vocabularies. Presented Frisket correctly omits closed content rather than merely hiding it, leaving a useful contrast with the executed Disclosure failure Q298. Stateful grid reuse and host paint-emphasis refresh remain qualified follow-ups. The source tests were read rather than broadly rerun.

Exact declaration locations in the mutable index were corrected for Q296 (propagation.rs:104) and Q298 (runner.rs:1506), with the prior immutable checkpoint retained and a separate location receipt. No finding outcome changed. No production code or shared WIP was edited, the prior audit document is an exact prefix, and the Git index is unchanged. No isolated target, Cargo home or worktree was introduced. Next is the remaining toolkit stylesheet, multi-runner, opened-link, overlay/input and retained-mount code, then rootstock/native/web hosts and remaining Mere source.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Complete graph-canvas source reading and focused identity/reachability checks; full source incomplete',
 'prior_receipt':{'path':'retained_events_review_receipt.json','sha256':sha(HERE/'retained_events_review_receipt.json')},'coverage':stats,'whole_current_files':whole,
 'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,'shared_working_file_sha256':old['shared_working_file_sha256'],
 'shared_working_file_preservation_scope':'All non-audit shared files exact; audit document preserved through chained exact prefixes.',
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,
 'diagnostics':{'graph_focus_probe_receipt.json':{'build_exits':probe['build_exits'],'run_exit':probe['run_exit'],'checks':probe['checks']}},'source_location_corrections':correction,
 'pending_verification':None,'next_read':next_read,'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'git_index_sha256':index_hash,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Current default-feature original toolkit compiled over exact retained libraries. No fresh full-current dependency graph, optional-feature compilation, paint/GUI or owner action. Compilation does not add reading coverage.'}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256']}.items()) and all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
