"""Save the list/session source review without modifying production code."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'list_session_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def git(*args): return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
def save(n, value): (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('forest_overlay_review_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1547
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / n) == h for n, h in old['shared_working_file_sha256'].items() if n != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
index = index if index.is_absolute() else ROOT / index
index_hash = sha(index)
inv, cov = load('source_inventory.json'), load('read_coverage.json')
assert inv['commit'] == head and load('pending_read.json') == []
whole = sorted(n for n, v in inv['files'].items() if n.startswith('crates/cambium/cambium/src/') and n not in guards and cov.get(n, {}).get('sha256') == v['sha256'] and cov[n]['through'] == v['chars'])
assert len(whole) == 15 and sum(inv['files'][n]['chars'] for n in whole) == 232028
partial = 'crates/cambium/cambium/src/tests.rs'
assert cov[partial]['through'] == 10463 and inv['files'][partial]['chars'] == 131598
assert all(sha(ROOT / n) == inv['files'][n]['sha256'] for n in [*whole, partial])
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2032 and stats['read_chars'] == 24326151
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
probes = {}
for name, checks in [('list_selection_probe', 5), ('session_containment_probe', 3)]:
    r = load(name + '_receipt.json')
    assert r['build_exits'] == [0, 0] and r['run_exit'] == 0 and r['checks'] == checks
    assert r['generated_output_removed'] and all(not Path(o['path']).exists() for o in r['removed_outputs'])
    assert all(sha(ROOT / n) == h for n, h in r['source_sha256'].items())
    assert all(sha(Path(v['path'])) == v['sha256'] for v in r['libraries'].values())
    probes[name + '_receipt.json'] = {'build_exits': r['build_exits'], 'run_exit': r['run_exit'], 'checks': checks}
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q302' and authority['entries'][-1]['id'] == 'A759'
findings = [
    {'id': 'Q303', 'title': 'Sectioned-list keyboard activation discards the action it promises to bubble',
     'source': 'crates/cambium/cambium/src/sectioned_list.rs:140',
     'evidence': 'The click closure returns OptionalAction from the shared row handler. The key closure invokes the same handler but assigns its result to let _, then returns unit. Current original toolkit execution with a nonunit Navigate action proves pointer activation returns Navigate, whereas both Enter and Space mutate the same local hit counter but return no action. Existing source tests only check in-place state changes from clicks and never assert a keyboard action. Source commentary explicitly promises that the key action bubbles. Searches of current in-tree crates/ports Rust found only the module test call site; a deployed product navigation failure was not established.',
     'diagnostics': ['list_selection_probe_receipt.json'],
     'repair_triage': {'disposition': 'documented; production unchanged under active read-only audit goal', 'reason': 'Contained repair: return the OptionalAction from handled keyboard branches and use a compatible empty result for other keys; verify both direct and optional action outputs. No new command authority is needed.'}},
    {'id': 'Q304', 'title': 'ContainedSession invokes the second leaf-registry call outside its panic boundary',
     'source': 'crates/cambium/cambium/src/surface.rs:465',
     'evidence': 'ContainedSession::leaves first calls session.leaves().is_some() inside catch_unwind, then calls session.leaves() again outside the catch to return the mutable reference. A deterministic valid trait implementation returning Some on its first call and panicking on its second proves the panic escapes to an outer harness catch, failure() remains None and availability still reports Available. A first-call panic control is caught and retires the guest as Unhealthy; a stable registry control succeeds. This contradicts the every-call-contained guarantee. The test uses current original toolkit code and synthetic trait implementations, not a demonstrated failing deployed guest or a native host crash.',
     'diagnostics': ['session_containment_probe_receipt.json'],
     'repair_triage': {'disposition': 'documented; production unchanged under active read-only audit goal', 'reason': 'Keep the actual mutable-reference-producing invocation inside the boundary. Refactor disjoint field borrowing or the registry access contract rather than using an unguarded second call; returned leaf measure/paint panics are a separate host boundary to inspect.'}}
]
for f in findings:
    f['status'] = 'verified original current default-feature toolkit over retained exact dependency artifacts'
    f['verification'] = NAME
notes['findings'].extend(findings)
rows = [
 ('Tab engines remain separate across public families', 'The three legacy tab-strip constructors do share tab_bar_view, and the newer tabs/segments/chips share selection_bar, but these are two separate roving engines.', ['tabs.rs','selection_bar.rs','radio.rs'], 'tabs.rs implements step_index, per-item key routing, pointer activation and request_tab_focus independently of selection_bar enabled-position navigation and request_focus. RadioGroup is another separate one-of-N path. The one-tab-bar claim holds within its legacy constructor family, not across the toolkit. Feature differences such as close actions, manual activation, disabled choices and generic parent state must be preserved before consolidation.'),
 ('Reordering has shared gesture output but caller-owned persistence', 'Pointer and keyboard reordering share ReorderMove; the caller alone changes the durable collection.', ['reorderable_list.rs'], 'Keyed items retain row identity, while transient drag state snapshots the original order and cancels if it changes. Existing tests read cover keyboard cancellation/drop, pointer equivalence, Alt+Arrow and removal during capture; they were not broadly executed this pass. Pointer destination assumes equal row-height spacing and needs a real variable-row consumer before stronger geometry claims.'),
 ('Resize and split geometry are separate operations', 'Both return requested caller geometry, but resize retains a grab and split exposes pure ratio/slot math.', ['resize_handle.rs','split.rs'], 'Neither persists owner size/layout. Split uses its own ratio clamp and keyboard step; ResizeBounds has ordered pixel clamps. Moving-handle local-coordinate accumulation and inverted-bound Home/End semantics require host geometry tracing before promotion to bugs.'),
 ('List action routing diverges by input route', 'Sectioned-list explicitly allows a caller action but only pointer dispatch returns it.', ['sectioned_list.rs'], 'Q303 verifies local mutation can make keyboard activation look successful while the requested outward action disappears. It needs a shared return policy, not another navigation channel.'),
 ('Settings draft versus committed authority', 'SettingRow local drafts emit only through Apply, with parent value/control changes resetting the draft.', ['setting_row.rs','slider.rs','select.rs'], 'SettingRow reuses TextInput, Slider, Toggle and RadioGroup rather than storing another committed provider value. Numeric settings conversion is separate from RangeScrubber reconcile and the basic Slider; finite arithmetic, missing bounds, empty/unknown choice handling and setting-id changes remain source-review follow-ups rather than fresh acceptance claims.'),
 ('Surface erasure delegates behavior but containment has a hole', 'RunnerSurfaceSession delegates concrete runner input and product closures; ContainedSession guards most calls but not its second leaves invocation.', ['surface.rs'], 'Descriptor/availability/action interpretation stay product-owned. Per-session leaf registries prevent key collisions across separate sessions. Q304 proves the mutable-reference return workaround breaks the claimed panic boundary. Layout, hit targets, scrolling, lifetime and leaf execution remain host contracts.'),
 ('Field projection and host-owned leaves have bounded shared implementations', 'Plain/styled/caret field bodies share field_children_impl; tag helpers and the deprecated Chisel spelling delegate to existing constructors.', ['styled_field.rs','tags.rs','summary_body.rs','status_bar.rs'], 'Field text styling is not a second editor: edit/edit_multiline remain input owners and committed-text metadata excludes IME/ghost visual text. Host pools deliberately leave descendants outside view reconciliation. StatusBar delegates to the separate general Popover family; source tests read cover overflow priority/action reachability and focus return. Neither summary/status presentation establishes document, save or provider authority.')
]
entries = [{'id': f'A{760+i}', 'domain': d, 'claim': c, 'implementations': ['crates/cambium/cambium/src/'+n for n in paths], 'assessment': a, 'state': 'current source read; focused list/session checks only', 'verification': NAME} for i,(d,c,paths,a) in enumerate(rows)]
authority['entries'].extend(entries)
notes.setdefault('review_followups', []).extend([
 {'verification': NAME, 'area': 'Selection bar with no enabled positions', 'detail': 'Focused API probe verifies explicitly setting focus to the non-tabbable root and routing an arrow changes selected to disabled index zero when enabled_positions is empty. Normal Tab traversal exposes no seats in this case; no ordinary host route was established, so keep as qualified follow-up rather than product bug.'},
 {'verification': NAME, 'area': 'Remaining geometry and draft boundaries', 'detail': 'Trace variable-height reorder rows, resize local coordinates after handle movement, settings spec identity and numeric conversions, and actual leaf execution containment in native/web/rootstock hosts. These were source observations, not tested product failures.'}
])
next_read = {'path': partial, 'start': 10463, 'reason': 'Finish complete toolkit integration-test source, then remaining text/tree/workspace modules, rootstock/native/web hosts and all other unread Mere source.'}
notes['current_area'] = 'Remaining list/geometry/settings/field/surface/tab modules reviewed; toolkit integration tests begun'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': [NAME,*probes], 'audit_authored_production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None, 'diagnostic_outcomes': 'Eight focused checks pass across two fresh current-crate probes over hash-bound retained dependencies. The all-disabled root-routing observation remains qualified; no normal host route was proved.'}
diff = subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
assert diff.returncode == 0
with (HERE/'list_session_diff_check.txt').open('x',encoding='utf-8') as f: f.write(diff.stdout+diff.stderr)
evidence = ['record_list_session_review.py','list_session_diff_check.txt']
for n in ['list_selection_probe','session_containment_probe']:
    evidence += [n+'.rs','run_'+n+'.py',n+'_receipt.json',n+'_compile_0.log',n+'_compile_1.log',n+'_output.log']
addition = f'''

### List actions, tab engines and session containment, 2026-10-17

Fifteen more toolkit files / 232,028 characters are fully read, plus the first 10,463 characters of the integration-test module. Coverage is now {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, without stale source hashes or pending chunks. A fresh fetch reports this HEAD includes origin/main (ahead 10, behind 0). Full-source review remains incomplete.

Q303 verifies a misleading keyboard success path in sectioned_list: Enter and Space run the row handler and update local state but discard its returned navigation action; pointer activation returns that same action. The source explicitly claims the key action bubbles, while its existing tests only check local click mutations. No product call site beyond module tests was found in the searched in-tree Rust source. Q304 verifies a broken panic boundary: ContainedSession asks leaves() twice, guarding only the first invocation. A second-call panic escapes and leaves the session reporting Available with no recorded failure. A first-call panic control retires the guest correctly, and a stable registry control succeeds. These are current API counterexamples, not headed product failures.

The [list/session checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds eight focused checks through two fresh compilations of original current default-feature Cambium over exact retained dependency artifacts. Generated rlib/executable/PDB outputs were recorded and removed. No fresh complete supplier graph, native input/layout/GPU, persistence, network or assistive-technology gate was run. Existing source tests were read, not broadly rerun.

A760-A766 track the separate tab_bar_view and selection_bar roving engines alongside the separate RadioGroup path, shared reorder output versus caller persistence, resize/split geometry boundaries, the list action discrepancy, local settings drafts versus parent truth, surface/containment ownership, and genuinely shared field projection/tag constructors. The one-tab-bar claim is true inside its constructor family, but the toolkit has another independent tab engine with different disabled/manual-selection features. Consolidation must preserve those differences. An all-disabled selection-bar check required explicitly focusing its non-tabbable root; ordinary Tab exposes no seats, so this remains a qualified follow-up. Variable-row geometry, moving resize-handle coordinates, numeric/setting identity boundaries and host leaf-execution containment remain to be traced.

Production and concurrent shared files remain byte-preserved, prior document text remains an exact prefix, and the index is unchanged. No branch, worktree, isolated target or Cargo home was created. Continue at toolkit tests.rs character 10,463, then the remaining toolkit and hosts.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json',notes)
save('authority_map.json',authority)
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'kind': 'List/session authority and success-path review; full source incomplete', 'prior_receipt': {'path': 'forest_overlay_review_receipt.json','sha256': sha(HERE/'forest_overlay_review_receipt.json')}, 'coverage': stats, 'whole_current_files': whole, 'source_sha256': {n:sha(ROOT/n) for n in whole}, 'partial_source': {'path': partial,'sha256': sha(ROOT/partial),'through':10463,'total_chars':131598}, 'preserved_production_and_test_hashes': guards, 'shared_working_file_sha256': old['shared_working_file_sha256'], 'new_findings': [f['id'] for f in findings], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass':False, 'diagnostics':probes, 'next_read': next_read, 'pending_verification':None, 'document': {'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()}, 'evidence_sha256':{n:sha(HERE/n) for n in evidence}, 'git_index_sha256':index_hash, 'upstream': {'fetched':True,'origin_main':git('rev-parse','origin/main'),'ahead_behind':git('rev-list','--left-right','--count','HEAD...origin/main')}, 'retained_isolation':[], 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget}, 'limits':'Current default-feature toolkit over retained exact libraries. No whole-current supplier graph or native/host/GPU/persistence/a11y acceptance. Direct root-routed disabled-selection observation is not a normal Tab path.'}
save(NAME,receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n)==h for n,h in {**guards,**receipt['source_sha256']}.items())
assert sha(ROOT/partial) == receipt['partial_source']['sha256']
assert all(sha(HERE/n)==h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n)==h for n,h in old['shared_working_file_sha256'].items() if n != old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_whole_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
