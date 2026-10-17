"""One-shot completion of the current retained-tree source review."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'retained_editor_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, v):
    (HERE / n).write_text(json.dumps(v, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('retained_ui_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1021
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
names = ['applet.rs', 'gesture_steps.rs', 'gestures.rs', 'grouping.rs', 'instrument.rs',
    'lane.rs', 'lane/overlay_observation.rs', 'moot_resources.rs', 'node_presentation.rs',
    'observe.rs', 'physics.rs', 'practice.rs', 'practice_off.rs', 'presentation.rs',
    'presenter.rs', 'producer_scene.rs', 'product/projection.rs',
    'product/projection/comparison.rs', 'product/projection/options.rs',
    'product/promoted_links.rs', 'product/query.rs', 'promoted_links.rs',
    'query_view.rs', 'remote.rs', 'reservoir.rs', 'responsive_links.rs', 'speed.rs',
    'visibility.rs']
whole = ['ports/graphshell/src/web_tree/' + n for n in names]
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
assert len(whole) == len(set(whole)) == 28
assert sum(inv[n]['chars'] for n in whole) == 481955
assert all(cov[n]['sha256'] == inv[n]['sha256'] == sha(ROOT / n)
    and cov[n]['through'] == inv[n]['chars'] for n in whole)
tree = [n for n in inv if n == 'ports/graphshell/src/web_tree.rs'
    or n.startswith('ports/graphshell/src/web_tree/')]
assert all(cov[n]['sha256'] == inv[n]['sha256'] == sha(ROOT / n)
    and cov[n]['through'] == inv[n]['chars'] for n in tree)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1521 and stats['total_files'] == 3727
assert stats['read_chars'] == 20758273 and not stats['changed_since_inventory'] and not stats['pending_chunks']
trace = [
    'ports/graphshell/src/presentation_editor.rs',
    'ports/graphshell/src/web_tree.rs', 'ports/graphshell/src/web_tree/product.rs',
    'ports/graphshell/src/web_tree/forme.rs',
    'crates/cambium/scenes/scenomise/src/authoring.rs',
    'crates/cambium/cambium-rootstock/src/input.rs',
    'crates/cambium/cambium-genet-web-host/src/events.rs']
trace_hashes = {n: sha(ROOT / n) for n in trace}
assert not (HERE / 'personal_sync_build_terminal.json').exists()
assert not (HERE / 'personal_sync_probe_receipt.json').exists()
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command',
    "@(Get-CimInstance Win32_Process -Filter \"Name='cargo.exe' OR Name='rustc.exe'\" | Where-Object { $_.ProcessId -eq 22716 -or $_.ParentProcessId -eq 22716 } | Select-Object ProcessId,ParentProcessId,CreationDate) | ConvertTo-Json -Compress"], text=True))
assert any(p['ProcessId'] == 22716 for p in live)
save('retained_editor_personal_sync_build_live.json', {
    'observed_utc': datetime.now(timezone.utc).isoformat(), 'session_id': 60676,
    'processes': live, 'stdout_bytes': (HERE / 'personal_sync_build.jsonl').stat().st_size,
    'scope': 'Original build confirmed live; no terminal result and no personal-sync tests executed.'})
pending = dict(old['pending_verification'])
pending['state'] = 'Original session60676 still live, with current Cargo/rustc process observation. Five prepared tests unexecuted.'
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q255' and authority['entries'][-1]['id'] == 'A625'
rows = [
    ('Q256', 'Retained presentation rules keep the initial graph owner after active graph replacement',
     'ports/graphshell/src/web_tree.rs:boot_inner/hooks; web_tree/presentation.rs:Pane::open/saved_rules/save/refresh; presentation_editor.rs:slot/prepare_save; web_tree/node_presentation.rs:refresh',
     'Pane::open loads PresentationSession using the initial product.forme_source graph session. It is called only at boot. Later graph replacements reconcile Forme and query drafts, but do not replace or retire presentation.session/backend. saved_rules returns the old committed rules without a current-owner check; the live node and Forme paths resolve them against the newly active graph. Subsequent proposals still use slot(next.graph_id) from the old PresentationSession, so a successful save reports rules saved for the old graph while the UI presents the new one. Transactional stale-baseline protection within that slot does not fix the wrong active owner.',
     'Bind the presentation pane, pending save and live adoption to actual graph-session identity. Retire or park drafts explicitly on owner change and load the new owner before adoption. Keep old-slot completion acknowledgement distinct from new-owner UI state.'),
    ('Q257', 'Projection save acknowledgement can mark a replacement draft clean',
     'ports/graphshell/src/web_tree/product/projection.rs:projection_action/load_projection_input/saved; web_tree/product.rs:poll; web_tree.rs:hooks incoming projection source; scenomise/src/authoring.rs:mark_saved',
     'Projection save retains only pending_save=true. While its App is owned by async persist_operation, the root frame hook still accepts a changed data-projection-dataset and calls load_projection_input directly. That replaces the editor/dataset but leaves pending_save set. When the older write succeeds, SavedProduct::poll calls projection.saved(), which invokes mark_saved on the replacement current editor and reports Saved executable projection. The written snapshot belongs to the earlier editor. The shared editor method marks its current history position; it has no proposal identity supplied by this caller.',
     'Capture exact editor/dataset/owner revision with the save proposal and mark only that acknowledged snapshot saved. Preserve newer edits as dirty or retire the old completion when the editor is replaced. A Boolean pending flag does not identify persisted content.'),
    ('Q258', 'Touch cancellation is converted to an activating release before shared host cancellation',
     'ports/graphshell/src/web_tree/gestures.rs:Listeners::pointer_other/HostHandle; web_tree.rs:boot_inner release closure; cambium-rootstock/src/input.rs:release/cancel_pointer; cambium-genet-web-host/src/events.rs:pointercancel; web_tree/forme.rs:pointer',
     'For a touch forwarded to the host, the root capture listener handles pointercancel by invoking HostHandle.release. Its closure calls host.release(), which dispatches PointerPhase::Up, before the canvas listener can call cancel_pointer. Up can consume capture and take Forme finish_preview(page,true), whereas Cancel would clear the pending drop without acceptance. The comment saying the host has no cancel is stale: cancel_pointer exists and the web host uses it. Scenario pointer cancel queues HostPointer::Cancel directly and does not exercise this browser capture adapter, so such passing controls do not prove touch cancellation.',
     'Use the shared cancellation API for pointercancel and distinguish cancellation from intentional first-finger release when beginning a pinch. Add a browser event route check covering a live Forme drop preview, rather than only direct HostPointer cancellation.'),
]
for ident, title, source, evidence, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source,
        'status': 'source-confirmed current caller path; no new runtime reproduction',
        'evidence': evidence, 'verification': NAME, 'diagnostics': [],
        'scope': 'Full current retained-tree source read and relevant shared owner methods traced. No browser graph-switch, save-race or touch-cancel trial ran.',
        'repair_triage': {'disposition': 'documented; no production changes under active goal', 'reason': repair}})
notes['candidates'].extend([
    {'title': 'Promoted-link comment draft and selected UUID lack an owner lease until dispatch',
     'source': 'ports/graphshell/src/web_tree/promoted_links.rs:Pane/section/add_comment',
     'verification': NAME, 'note': 'Pending writes validate scope, but selected/comment state persists without an owner reset. Trace graph replacement with a colliding promoted Resource UUID before claiming a cross-owner comment retarget. No new runtime trial.'},
    {'title': 'Absent page semantic-tree instrumentation becomes empty negative observations',
     'source': 'ports/graphshell/src/web_tree/lane.rs:semantic_tree/snapshot',
     'verification': NAME, 'note': 'Missing/failing graphshellSemanticTree returns None, then several heard observations default to zero/empty. Verify actual scenario availability gates and negative-only assertions before claiming an overall false pass; no planted-instrument-removal trial ran.'},
])
for c in notes['candidates']:
    if c.get('title') == 'Remote generic board replacement retains matching physics IDs across scoped identities':
        c['followup'] = {'verification': NAME, 'note': 'Retained resident routes explicitly use sync_reading with attachment/layout identity. Generic live projection calls sync_scoped with identity=None; inspect whether protocol epoch alone spans endpoint replacement before promoting transient body-state carryover to a product defect.'}
arows = [
    ('Presentation rule lifetime', 'Transactional rule storage is real, but the retained pane lacks active-owner rebinding.', ['Pane::open', 'PresentationSession::slot', 'saved_rules'], 'Q256: old saved rules can adopt new graph facts and writes remain in the old slot.'),
    ('Live node adoption', 'NodeRuntime evaluates saved rules using actual visible access contexts.', ['NodeRuntime', 'node_presentation::refresh/install'], 'Visible-node budget, epoch and input checks are real; they do not establish the saved rules graph owner.'),
    ('Query draft lifetime', 'Query views retain exact graph/persona scope and expected Resource spec.', ['query_view::load/save/poll', 'SavedProduct::save_query'], 'Owner changes retire drafts; pending outcomes are reread after storage acknowledgement.'),
    ('Query projection truth', 'Shown bindings and source-stratum relations stay distinct in the read-only view.', ['project_view', 'presented_links', 'QueryProjection::access_for'], 'Current-source and exact Resource/access checks protect open/freeze actions. The DOM stroke realization is a UI adapter, not an additional durable graph owner.'),
    ('Promoted review writes', 'Freeze and comment writes share the actual product operation persistence owner.', ['freeze_link', 'comment_promoted_link', 'promoted_links::poll'], 'Failed saves remain unconfirmed; source leases guard pending completion. Unsaved comment/selection owner lifetime remains a candidate.'),
    ('Projection source and solve', 'The retained editor reuses shared typed compilation and occurrence selection.', ['ProjectionEditor', 'compile_editor', 'snapshot_editor'], 'Placement reuse and dynamics refresh are separate actual paths; no native solve was rerun.'),
    ('Projection save identity', 'The UI acknowledgement tracks only a Boolean pending flag.', ['ProjectionControls::saved', 'load_projection_input', 'SavedProduct::poll'], 'Q257: a prior successful write can mark a newly replaced editor clean.'),
    ('Arrangement controls', 'Registry declarations supply option values, defaults and refusal text.', ['arrangement_options', 'arrangement_defaults', 'OptionKind::refusal'], 'This is implemented shared authority; retained select/text controls are local adapters.'),
    ('Comparison selection', 'Comparison cells apply shared arrangement and dynamics data to one draft.', ['ApplyComparison', 'compare_arrangements_dynamics', 'compare_dynamics'], 'Refused cells are explicit and disabled; temporary scene geometry is not source truth.'),
    ('Producer scene', 'Presentation and export use the same retained producer scene source.', ['CanvasProducer::produce', 'remember_scene'], 'Export refuses dirty or resized state; it returns the cached scene and does not tick simulation.'),
    ('Physics controls', 'Retained and older page controls call shared typed physics policy.', ['physics_controls', 'PhysicsChoice', 'web_speed'], 'Local UI state/catalogue bindings are adapters; no separate durable physics authority appears here.'),
    ('Gesture cancellation', 'The outer touch adapter still assumes the host lacks cancellation.', ['pointer_other', 'HostHandle::release', 'Host::cancel_pointer'], 'Q258: root capture generates Up before the modern canvas cancellation listener runs.'),
    ('Gesture receipts', 'Synthetic gestures and direct scenario pointers use different entry paths.', ['gesture_steps', 'TreeLane::app_step', 'HostPointer::Cancel'], 'Synthetic wheel evidence does not prove native page scroll, and direct pointer cancel does not prove the DOM touch capture route.'),
    ('Hosted selection and folds', 'The host presenter and grouped dataset retain source occurrence identities.', ['HostPresenter', 'capture_host_selection', 'change_group'], 'Folds preserve coordinates and rebind visible members. Explicit navigation uses the shared URL admission helper.'),
    ('Remote composition', 'TreeRemote shares RemoteSession and WebRTC link logic with the older page.', ['RemoteHost', 'MereRouteSession', 'RemoteBoard'], 'Resident reading explicitly clears/reconciles attachment/layout ownership; generic projection identity qualification remains narrower.'),
    ('Applet host boundary', 'The retained UI delegates pack review and action lowering to CapsuleMount.', ['review_applet', 'accept/lower', 'Readings'], 'Generation checks and explicit keep writes are present; core CapsuleMount/Readings source still requires whole reading next. No new guest, signature or IndexedDB runtime gate.'),
    ('Community bridge', 'The resource UI delegates owner tokens and outcomes to its supplying Host.', ['moot_resource_prepare_dispatch', 'moot_resource_complete', 'resource_coop::view'], 'This wrapper does not itself hold peer authority; portable Host/core sources remain unread in this area.'),
    ('Diagnostics', 'Overlay observers bound reads and distinguish disabled/incomplete/observed states.', ['overlay_observation', 'visibility', 'speed::PaceWindow'], 'Passive observations are not browser acceptance. Visibility requires actual hidden/shown events; cached speed sample identity and semantic absence need caller qualification.'),
]
entries = [{'id': f'A{626+i:03d}', 'domain': d, 'claim': c, 'implementations': impl,
    'assessment': a, 'state': 'current complete retained-tree source review; no new runtime tests', 'verification': NAME}
    for i, (d, c, impl, a) in enumerate(arows)]
assert len(entries) == 18
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/src/capsule_applet.rs', 'start': 0,
    'reason': 'Read portable CapsuleMount/Readings and community Host/core after their retained wrappers, then all other unread source.'}
notes['current_area'] = 'Entire current retained-tree module including editors, producers, scenarios, gestures, remote and applet wrappers'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME], 'production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': pending, 'diagnostic_outcomes': 'No new runtime tests. Original personal-sync build live; five prepared checks unexecuted.'}
addition = f'''

### Retained editors, input and scenario completion, 2026-10-17

This pass completely reads 28 files / 481,955 characters at {head}. Together with prior current-hash reads, web_tree.rs and every inventoried source beneath web_tree/ are now fully read ({len(tree)} files). This includes all retained editor/projection submodules, node adoption, the 99,148-character scenario lane, gestures, applet/community wrappers, remote/resident adapters, physics/speed and bounded observers. Full Mere coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. The full-source goal remains incomplete.

Q256: presentation::Pane loads rules using the initial actual graph session at boot and never rebinds when the active graph is replaced. Forme and query drafts have owner-change handling; the rules pane does not. Its old saved rules can resolve against new graph facts, and successful later saves still target the initial graph's slot. Transactional baseline checks within that old slot do not establish the active owner. Add actual owner lifecycle and distinguish admitted old-slot completion from a new-owner UI. This is a source-confirmed current path, not a browser graph-switch trial.

Q257: projection saving retains only pending_save=true. While an older write is in flight, the root frame hook can accept a changed data-projection-dataset and replace the editor directly, leaving that flag set. When the older operation succeeds, projection.saved() marks the replacement current editor clean and reports a saved executable projection. Persisted content belongs to the earlier snapshot. Bind acknowledgement to exact editor/source/owner revision and keep newer state dirty. This is source-confirmed; no browser save-race trial ran.

Q258: the root touch capture listener calls host.release() for pointercancel on a host-forwarded touch. That emits Up before the canvas listener reaches the now-existing cancel_pointer API. Up can accept a Forme drop and consume capture; the later cancellation can no longer take the intended Cancel path. The comment that the host lacks cancellation is obsolete. Direct scenario pointer cancel queues HostPointer::Cancel and bypasses this DOM capture adapter, so its success cannot qualify browser touch cancellation. Use real cancellation and separately preserve the deliberate first-finger release when beginning a pinch. No browser touch trial ran.

A626-A643 track these gaps alongside actual reuse: shared rule storage, typed projection compilation, registry-owned arrangement options, one occurrence selection, shared physics controls, common presentation/export scene production, HostPresenter/grouped source identities, RemoteSession/WebRTC composition, and exact query Resource/access checks. Query drafts explicitly retire after owner changes; freeze/open actions revalidate source revisions. Promoted writes recheck pending source scope and failed saves remain unconfirmed. Their unsaved comment/selected-UUID lifetime across owner changes remains a candidate, without a demonstrated retarget.

Resident boards explicitly use sync_reading with attachment/layout identity. The generic projection path passes no additional identity to sync_scoped; endpoint/epoch replacement and transient body carryover still need qualification. Applet wrappers delegate review/lowering to CapsuleMount, retain generations and explicit keep writes, but the portable CapsuleMount/Readings source is still unread and is next. Community wrapper outcomes delegate to a supplying Host; the wrapper is not peer authority. The review does not promote those calls to fresh signature, guest, retention or network acceptance.

Scenario source distinguishes direct HostPointer steps, synthetic browser events and externally performed fixture input. Read-only overlay observations are bounded and expose disabled/incomplete states; actual visibility instrumentation requires real hidden/shown transitions. Missing semantic-tree instrumentation defaults some negative observations to empty values, a candidate pending actual scenario availability/negative-control qualification. Passive geometry and status markers are not headed acceptance. No new native, browser, GPU, storage or network tests ran this pass.

All 1,021 prior protected production/test hashes and cited prior evidence remain unchanged; the canonical document prefix is preserved. No production/original-test code changed. No isolated target, Cargo home or worktree was created. Personal-sync build session60676 remains independently confirmed live in the stable shared Mere target. Its five prepared checks and Q250-Q252 native qualification remain pending. Poll that same handle; do not restart after an observation timeout. The [retained editor checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds coverage, trace hashes, findings and pending verification. Next is capsule_applet.rs, followed by its Readings and community Host/core, with every other unread Mere area still in scope.
'''
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
(HERE / 'retained_editor_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
evidence = ['record_retained_editor_review.py', 'retained_editor_personal_sync_build_live.json', 'retained_editor_diff_check.txt']
evidence_hashes = {n: sha(HERE / n) for n in evidence}
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'retained_ui_review_receipt.json', 'sha256': sha(HERE / 'retained_ui_review_receipt.json')},
    'kind': 'Complete current retained-tree source review; three source-confirmed findings; full Mere objective incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 481955,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'trace_source_sha256': trace_hashes,
    'preserved_production_and_test_hashes': guards,
    'completed_area': {'name': 'ports/graphshell/src/web_tree.rs and web_tree/',
        'files': len(tree), 'source_sha256': {n: sha(ROOT / n) for n in tree}},
    'new_findings': ['Q256', 'Q257', 'Q258'], 'source_only_findings': ['Q256', 'Q257', 'Q258'],
    'authority_entries': [e['id'] for e in entries], 'new_native_tests_run': 0,
    'production_edits_this_pass': False, 'pending_verification': pending, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
        'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': evidence_hashes, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Source-only caller and shared owner tracing. No new runtime tests. Personal-sync original feature build remains live; no prepared checks executed. Portable applet/community core still unread.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'whole_files': len(whole), 'whole_chars': 481955,
    'completed_retained_tree_files': len(tree), 'new_findings': receipt['new_findings'],
    'protected_paths': len(guards), 'new_native_tests_run': 0, 'pending_session': 60676,
    'retained_bytes': budget}, indent=2))
