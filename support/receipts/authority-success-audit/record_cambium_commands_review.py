"""Checkpoint current toolkit reading and command/text counterexamples."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='cambium_commands_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('mere_view_review_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==1495
assert all(sha(ROOT/n)==h for n,h in guards.items()) and all(sha(HERE/n)==h for n,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index;index_hash=sha(index)
inv,cov=load('source_inventory.json'),load('read_coverage.json');assert inv['commit']==head and load('pending_read.json')==[]
whole=sorted(n for n,v in inv['files'].items() if n.startswith('crates/cambium/cambium/src/') and cov.get(n,{}).get('sha256')==v['sha256'] and cov[n]['through']==v['chars'] and n not in guards)
assert len(whole)==19 and sum(inv['files'][n]['chars'] for n in whole)==239007
assert all(sha(ROOT/n)==inv['files'][n]['sha256'] for n in whole)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==1984 and stats['read_chars']==23437274 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probes={n:load(n) for n in ['command_tree_probe_receipt.json','command_tree_probe_v2_receipt.json','text_grapheme_probe_receipt.json','text_grapheme_probe_v2_receipt.json']}
assert probes['command_tree_probe_receipt.json']['run_exit']==101 and probes['command_tree_probe_receipt.json']['checks']==3
assert probes['text_grapheme_probe_receipt.json']['build_exit']==1
assert probes['command_tree_probe_v2_receipt.json']['checks']==6 and probes['text_grapheme_probe_v2_receipt.json']['checks']==7
for n,p in probes.items():
 if '_v2_' in n:assert p['build_exit']==p['run_exit']==0
 assert p['generated_output_removed'] and all(not Path(o['path']).exists() for o in p['removed_outputs'])
 assert all(sha(ROOT/s)==h for s,h in p['source_sha256'].items()) and all(sha(Path(v['path']))==v['sha256'] for v in p['libraries'].values())
notes,authority=load('review_notes.json'),load('authority_map.json');assert notes['findings'][-1]['id']=='Q288' and authority['entries'][-1]['id']=='A731'
rows=[
 ('Q289','Context menus accept recursive command trees but cannot disclose grandchildren',
 'crates/cambium/cambium/src/command_surface.rs:337,439; crates/cambium/cambium/src/command_menu_bar.rs:433',
 'The recursive CommandItem tree is shared, but interaction is not: CommandState stores one submenu parent while CommandMenuBarState stores a recursive path. For File -> Export -> HTML, the actual current context-menu module renders Export but never HTML. Enter emits Activate([0,0]) for the non-leaf Export; pointer activation merely reopens the first submenu. The same tree through the actual current menubar reaches HTML and emits Activate([0,0,0]). CommandEvent documentation does acknowledge depth-one submenus, so this finding concerns accepted unsupported tree structure and misleading non-leaf activation, not an undocumented promise of recursive context-menu support. No application command backend was executed.',
 'command_tree_probe_v2_receipt.json',
 'Either reject/explicitly lower deeper trees at the shared boundary or use a shared recursive traversal policy. Keep presentation-specific focus behavior separate while giving the same command tree a declared supported shape.'),
 ('Q290','Open submenus still activate children after the parent group becomes disabled',
 'crates/cambium/cambium/src/command_surface.rs:184,439; crates/cambium/cambium/src/command_menu_bar.rs:136,433',
 'Both engines retain open submenu paths across a model refresh that disables the ancestor group. Their activation checks examine the selected child only. A live headless runner opens File, refreshes File as disabled because Disconnected while its Open child is still individually enabled, and Enter emits Activate([0,0]) in both context-menu and menubar modules. An individually disabled child remains inert as its control. This is inconsistent UI availability/invalidation, not evidence of backend authorization bypass; command executors were not invoked.',
 'command_tree_probe_v2_receipt.json',
 'Validate every ancestor when normalizing an open path and before emitting activation. Close/refuse stale open paths through one shared command availability rule.'),
 ('Q291','Inserting a combining mark corrupts the grapheme caret and moves subsequent text past the suffix',
 'crates/cambium/cambium/src/controls/text_input/core.rs:380',
 'insert_str increments its grapheme caret by the inserted fragment grapheme count without resegmenting the splice. A combining mark joins the preceding grapheme, so inserting U+0301 after a produces one grapheme with caret=2; the first Backspace removes no text. In ab, insertion at byte 1 produces a+acute+b but caret_position jumps after b, and the next x produces a+acute+bx instead of a+acute+xb. Current original TextInput/editor modules reproduce both paths through TextCommand::Insert. Constructing the same combined buffer initially and ordinary x insertion behave correctly as controls. No native IME, host field or persisted document was exercised.',
 'text_grapheme_probe_v2_receipt.json',
 'Track the splice endpoint in byte space and derive a valid caret from segmentation of the resulting buffer, with explicit boundary affinity. Cover combining, ZWJ and regional-indicator joins across the insertion boundary.'),
 ('Q292','TextCommand reports no change when a boundary caret move cancels visible composition',
 'crates/cambium/cambium/src/controls/text_input/command.rs:172,200',
 'TextInput::apply documents whether logical or composition state changed, but Move, SelectAll and SetSelection compare only caret_selection after clearing preedit. The native module probe puts a composition at the start of a, applies GraphemeLeft without extension, and receives false even though the composition vanishes and rendered text changes from candidate+a to a. Explicit CancelComposition returns true as its control. Only the Move counterexample was executed; the other two branches are source-traced. No audited caller was established to gate redraw/persistence on this false result, so user-visible missed redraw is not claimed.',
 'text_grapheme_probe_v2_receipt.json',
 'Include composition changes in the declared changed result, or narrow the API contract and return separate committed/selection/composition change flags.'),
]
findings=[{'id':i,'title':t,'source':s,'status':'verified original current modules over hash-bound retained dependencies','evidence':e,'verification':NAME,'diagnostics':[d],
 'repair_triage':{'disposition':'documented; no production changes','reason':r}} for i,t,s,e,d,r in rows];notes['findings'].extend(findings)
arows=[
 ('Action compatibility delegation','The old action-list names delegate instead of retaining another palette engine.',
 ['crates/cambium/cambium/src/action_list.rs','crates/cambium/cambium/src/command_surface.rs'],
 'ActionItem/ActionListState alias the shared types, and action_list maps the shared palette result to the legacy top-level event. That adapter is real reuse; its single-index result does not promise a recursive command path.'),
 ('Command model versus navigation authority','Sharing CommandItem does not share recursive navigation or path invalidation.',
 ['crates/cambium/cambium/src/command_surface.rs','crates/cambium/cambium/src/command_menu_bar.rs'],
 'Palette, picker and context menu genuinely share command_surface. The menubar separately implements traversal, selection, disclosure, item lookup, activation and ancestor normalization over the same tree. Q289 demonstrates depth/capability drift; Q290 demonstrates the same missing disabled-ancestor invalidation in both engines. A shared vocabulary is narrower than a shared interaction authority.'),
 ('Atlas geometry ownership','Atlas paint and picking share an aspect-preserving projection while the caller owns scene and motion.',
 ['crates/cambium/cambium/src/atlas.rs'],
 'World paint/routes/footprint picking reuse GraphCanvasAtlasView; retained callout outlines follow normalized moving nodes. Picking checks actual footprints and explicit priority/source order. Painter conversion handles circles, rectangles and polygons; paths are omitted. Tests read cover projection, polygon corners and callout motion, not every unsupported shape or invalid-bound input.'),
 ('Component local-state boundary','Reusable components retain local state and emit typed events through a parent mapper.',
 ['crates/cambium/cambium/src/component.rs'],
 'The Genet-specific boundary deliberately stays out of renderer-neutral Meristem. Props reconciliation, message dirtiness and equality memoization operate on one retained child tree. Probe identity is automation metadata, not routing or keyed-sequence authority.'),
 ('Event registry and portable-child ownership','Routing registries have different composition contracts and the context nursery owns parked children within a rebuild.',
 ['crates/cambium/cambium/src/context.rs'],
 'Click/key handlers retain per-node lists and path-specific removal. Focus, pointer, value, file, hover and wheel registrations each retain a single path per node; do not infer multi-listener support from the click/key contract. Portable children are bucketed by concrete key/view/state types and drained at rebuild end. Nested same-element focus observer composition and portable-key collision behavior remain consumer follow-ups, not verified bugs.'),
 ('Text editing and history boundaries','Plain/styled fields share TextInput, with explicit low-level and journaled command entry points.',
 ['crates/cambium/cambium/src/controls/field.rs','crates/cambium/cambium/src/controls/text_input/command.rs','crates/cambium/cambium/src/controls/text_input/core.rs','crates/cambium/cambium/src/editor.rs'],
 'Field bodies delegate to styled_field; text commands apply over one buffer. External EditHistory is an explicit alternative for transaction grouping, and low-level methods are public, so the command path is not the exclusive mutation authority. Q291 exposes invalid caret arithmetic at the shared splice; Q292 exposes the changed-result mismatch. Hard-line and logical-word motion remain distinct from layout-aware visual navigation.'),
 ('Scene reading versus source edit authority','Backdrop/detail helpers expose facts and selection without acquiring document edit authority.',
 ['crates/cambium/cambium/src/backdrop_scene.rs','crates/cambium/cambium/src/detail_panel.rs','crates/cambium/cambium/src/arrangement.rs'],
 'Backdrop controls emit occurrence keys while the embedded host owns scene input/rendering. Detail rows are inert label/value facts, deliberately distinct from activatable sectioned-list rows. Arrangement emits CSS placement and explicit extent; its DOM retention tests do not themselves prove painted hit testing or accessibility on a headed host.'),
]
entries=[{'id':f'A{732+i:03d}','domain':d,'claim':c,'implementations':im,'assessment':a,'state':'current source reviewed with bounded diagnostics','verification':NAME} for i,(d,c,im,a) in enumerate(arows)];authority['entries'].extend(entries)
notes.setdefault('review_followups',[]).extend([
 {'verification':NAME,'area':'Context single-path observer composition','detail':'Trace nested on_focus and portable-key uniqueness contracts before treating single-path registry replacement or nursery key collisions as actual bugs.'},
 {'verification':NAME,'area':'Text host policy and changed result','detail':'Continue current native/web/rootstock key/clipboard/IME callers. The composition changed-flag counterexample has no verified redraw-gating consumer; single-line filtering and modified character behavior still require host traces.'}])
next_read={'path':'crates/cambium/cambium/src/detail_popover.rs','start':0,'reason':'Continue unread toolkit source after detail_panel (editor already read), then rootstock and native/web hosts, then remaining Mere source.'}
notes['current_area']='Cambium toolkit command engines, context and text input reviewed';notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,*probes],'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,
 'diagnostic_outcomes':'Six command-tree checks and seven TextInput checks pass over retained dependencies. Initial command fixture borrow panic and TextInput module-path compilation failure remain preserved; repaired attempts use distinct files. No full-current Cargo or native host gate.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'cambium_commands_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
evidence=['record_cambium_commands_review.py','cambium_commands_diff_check.txt']
for n in ['command_tree_probe','command_tree_probe_v2','text_grapheme_probe','text_grapheme_probe_v2']:
 evidence += [n+'.rs','run_'+n+'.py',n+'_receipt.json',n+'_compile.log']
 if (HERE/(n+'_output.log')).exists():evidence.append(n+'_output.log')
addition=f'''

### Cambium command engines and text editing, 2026-10-17

Nineteen further toolkit sources / 239,007 characters are completely read: compatibility actions, arrangement, atlas, backdrop reading, both command engines, component/context, every basic form/text module, inert detail panels and editor history. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters at unchanged HEAD {head}. The inventory has no stale hashes or pending chunk. Full Mere remains incomplete.

Q289 records incompatible command-tree interpretation. The shared context engine only stores one submenu parent, while the menubar independently stores a recursive path. With the same File -> Export -> HTML tree, context-menu Enter emits an activation for non-leaf Export and pointer disclosure never renders HTML; the menubar reaches the HTML leaf with its complete path. The context API does document a depth-one limit, but it accepts deeper recursive input without refusing or lowering that unsupported structure. Q290 records stale availability in both engines: after an open File group becomes disabled, its individually enabled child still emits an activation. The individually disabled child control remains inert. These are UI routing findings, not exercised backend authorization failures.

Q291 records broken grapheme caret arithmetic at the shared insertion splice. Appending a combining acute accent to a leaves one grapheme with caret 2, and the first Backspace deletes no text. Inserting that accent between a and b jumps the caret after b, so the next x produces a+acute+bx. Precombined-buffer and ordinary insertion controls pass. Q292 records a narrower changed-result defect: a boundary caret move clears a visible composition but returns false despite the documented logical-or-composition change contract. No redraw-gating consumer was established, so missed redraw or persistence is not claimed.

The [toolkit checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds six passing command checks and seven passing text checks. Command modules are current original source included by path over retained real Cambium runner/ScriptedDom artifacts; the text diagnostic includes current original leaf/editor modules through an audit-only path wrapper. An initial command fixture panicked on its own RefCell borrow after three successful observations, and the initial text wrapper failed Rust module resolution. Both original attempts remain intact; distinct corrected attempts pass. No native window, OS keyboard, physical IME, GPU, network, persisted document, command backend or fresh full-current Cargo graph was exercised. All probe outputs under the stable target were identified and removed.

A732-A738 distinguish real action-list delegation from separately implemented menubar navigation; one atlas transform from caller-owned terrain/motion; component local state from parent authority; multi-listener click/key registries from single-path observer registries; shared text storage from explicit alternative history entry points; and inert fact/selection controls from source editing. Single-path nested focus composition, portable-child key collisions, host text policy and oversized Mere tier-card reachability remain qualified follow-ups. The existing source tests were read, not broadly rerun.

No production code or shared WIP was changed, the prior audit document is an exact prefix, and the Git index is unchanged. No isolated target, Cargo home or worktree was created. Next is unread toolkit source after detail_panel, followed by rootstock and native/web host ownership. The full-source goal stays active.
'''
doc.write_bytes(prefix+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Further toolkit reading and qualified command/text probes; full source incomplete',
 'prior_receipt':{'path':'mere_view_review_receipt.json','sha256':sha(HERE/'mere_view_review_receipt.json')},'coverage':stats,'whole_current_files':whole,
 'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,'shared_working_file_sha256':old['shared_working_file_sha256'],
 'shared_working_file_preservation_scope':'All non-audit shared files exact; audit document preserved as chained exact prefixes.',
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,
 'diagnostics':{n:{'build_exit':p['build_exit'],'run_exit':p['run_exit'],'checks':p['checks']} for n,p in probes.items()},'pending_verification':None,'next_read':next_read,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'git_index_sha256':index_hash,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Exact current modules over retained byte-bound libraries, not full-current transitive Cargo or headed acceptance. Existing source tests read rather than broadly rerun.'}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256']}.items()) and all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
