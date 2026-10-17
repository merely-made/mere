"""One-shot retained UI source checkpoint; no product mutation."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'retained_ui_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, v):
    (HERE / n).write_text(json.dumps(v, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('personal_sync_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 999
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
names = ['forme_workspace/tests.rs', 'web_tree.rs', 'web_tree/appearance.rs',
    'web_tree/commands.rs', 'web_tree/controls.rs', 'web_tree/coupling.rs',
    'web_tree/dynamics_inspection.rs', 'web_tree/field_inspection.rs',
    'web_tree/field_inspection_probe.rs', 'web_tree/forme.rs',
    'web_tree/host_boundary.rs', 'web_tree/inspection.rs',
    'web_tree/link_inspection.rs', 'web_tree/owner_waypoint.rs',
    'web_tree/product.rs', 'web_tree/product/commands.rs',
    'web_tree/product/detail.rs', 'web_tree/product/fields.rs',
    'web_tree/product/find.rs', 'web_tree/product_off.rs',
    'web_tree/reading.rs', 'web_tree/remote_off.rs']
whole = ['ports/graphshell/src/' + n for n in names]
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
assert len(whole) == len(set(whole)) == 22
assert sum(inv[n]['chars'] for n in whole) == 440402
assert all(cov[n]['sha256'] == inv[n]['sha256'] == sha(ROOT / n)
    and cov[n]['through'] == inv[n]['chars'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1493 and stats['total_files'] == 3727
assert stats['read_chars'] == 20276318 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probe = load('forme_stale_save_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 2 and probe['generated_output_removed']
assert all(sha(ROOT / n) == h for n, h in probe['source_sha256'].items())
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-forme-stale-save-probe.*'))
assert not (HERE / 'personal_sync_build_terminal.json').exists()
assert not (HERE / 'personal_sync_probe_receipt.json').exists()
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command',
    "@(Get-CimInstance Win32_Process -Filter \"Name='cargo.exe' OR Name='rustc.exe'\" | Where-Object { $_.ProcessId -eq 22716 -or $_.ParentProcessId -eq 22716 } | Select-Object ProcessId,ParentProcessId,CreationDate) | ConvertTo-Json -Compress"], text=True))
assert any(p['ProcessId'] == 22716 for p in live)
save('retained_ui_personal_sync_build_live.json', {
    'observed_utc': datetime.now(timezone.utc).isoformat(), 'session_id': 60676,
    'processes': live, 'stdout_bytes': (HERE / 'personal_sync_build.jsonl').stat().st_size,
    'scope': 'Original build is live. No terminal outcome and no personal-sync diagnostic execution.'})
pending = dict(old['pending_verification'])
pending['state'] = 'Original session60676 still live; fresh process and artifact observation. Five prepared tests remain unexecuted.'
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q252' and authority['entries'][-1]['id'] == 'A609'
rows = [
    ('Q253', 'Independent Forme sessions acknowledge a stale overwrite that the rules editor refuses',
     'ports/graphshell/src/forme_session.rs:PreparedSave::save; forme_workspace.rs:save; web_tree/forme.rs:save',
     'Two public FormeSession instances load one MemoryBackend slot. The first saves a changed left bound successfully. The stale second saves a changed top bound successfully and reload loses the acknowledged first edit. The equivalent PresentationSession control rejects the stale baseline with changed-elsewhere and retains its draft. The retained UI uses the blind Forme proposal save, local request tokens and per-page retired-write fences; these do not compare an independently changed stored baseline.',
     'native public API reproduction and refusal control: 2 passed; UI caller source-confirmed',
     ['forme_stale_save_probe_receipt.json'],
     'Use a transactional stored-baseline guard shared with the rules editor while preserving draft/history on refusal. This affects cross-tab or independent-client editing semantics; do not substitute an in-memory token for a storage comparison. Browser/multitab and physical-disk behavior remain untested.'),
    ('Q254', 'Retained form requests are dropped after displaying a handoff message',
     'ports/graphshell/src/web_tree/reading.rs:Reading::poll; web_tree.rs:hooks retained request loop',
     'Reading drains each PeltHostEffect. Navigation is pushed to the outer requests queue, while the submission-only branch tests is_some, displays This retained view handed its form request to the host, then drops the payload. The outer frame hook only consumes (access,address) navigation/export tuples. No submission queue, consumer, outcome or retry is present in this complete composition. The finding is conditional on a retained engine emitting a submission; it does not claim arbitrary stored clip forms execute.',
     'source-confirmed dropped effect; no browser form trial', [],
     'Report an explicit unsupported/refused submission unless a real owner accepts the full method, fields and destination and returns an outcome. Connecting network execution needs caller policy and is outside a quick audit patch.'),
    ('Q255', 'The retained command API bypasses shared hosted-fit semantics',
     'ports/graphshell/src/web_tree/controls.rs:named_command; web_tree.rs:TreePage::command; web_tree/commands.rs:run; viewer_commands.rs:action',
     'For a hosted dataset the toolbar and retained menu select CanvasCommand::FitVisible through hosted-aware logic. TreePage::command instead consults the copied controls::named_command table, which maps the same view:fit ID unconditionally to Fit. The shared viewer_commands::action already accepts hosted context. Fit calls fit_to_content and resumes view-follow; FitVisible has its own visible framing path. The scenario/API command logs success through this actual current caller while executing different policy from the UI.',
     'source-confirmed divergent route; no browser framing trial', [],
     'Delegate the command API to the shared hosted-aware action mapping and keep its explicit capabilities/outcome. A second string-to-command table must not silently redefine an existing command ID.'),
]
for ident, title, source, evidence, status, diagnostics, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source,
        'status': status, 'evidence': evidence, 'verification': NAME,
        'diagnostics': diagnostics, 'scope': 'Current source at unchanged basis; native evidence limited to Q253 public in-memory API test and rules-workspace control.',
        'repair_triage': {'disposition': 'documented; production changes excluded by active goal', 'reason': repair}})
for c in notes['candidates']:
    if c.get('title') == 'Forme save has no stored-baseline comparison across independent editing sessions':
        c['followup'] = {'verification': NAME, 'finding': 'Q253', 'result': 'Public native stale overwrite reproduced; actual retained UI save caller traced. Browser multiwriter trial unexecuted.'}
notes['ruled_out'].extend([
    {'title': 'Retained product fields are not disconnected from the shared caret and IME hook',
     'source': 'web_tree.rs:hooks::focused_text; web_tree/appearance.rs:focused_text',
     'note': 'The appearance fallback explicitly delegates to product::focused_text. Reading only the outer hook would miss this real route. No new native/headed IME trial.', 'verification': NAME},
    {'title': 'Secondary Forme drag is filtered by its actual retained caller',
     'source': 'ports/graphshell/src/web_tree/controls.rs:TreePage::pointer',
     'note': 'Down/Move/Up enter Forme pointer handling only for Primary; cancellation separately releases both lanes. The local unguarded Forme helper does not establish an auxiliary-button product bug.', 'verification': NAME},
    {'title': 'Failed retained clip intake preserves the created member for retry',
     'source': 'ports/graphshell/src/web_tree/product.rs:store_clip_file/poll',
     'note': 'The completed member is recovered into pending_intake before handling an intake failure. Storage acknowledgement and optional Canvas refresh errors are explicitly distinguished.', 'verification': NAME},
])
arows = [
    ('Retained product saves', 'The UI moves its actual App into async writes and reports after acknowledgement.', ['SavedProduct::poll', 'persist_operation'], 'Errors retain pending operations; post-save Canvas refresh is a separately worded outcome.'),
    ('Visibility ownership', 'View hides validate actual graph owner and persisted desired sets.', ['save_link_visibility', 'source_session_id'], 'Busy and uncertain saves keep their source locked until owner resolution.'),
    ('Clip retry ownership', 'Intake completion restores the actual created member for retries.', ['store_clip_file', 'CompletedEdit::Intake'], 'The apparent lost-member retry defect is ruled out by the actual completion path.'),
    ('Forme storage protocol', 'Forme blind-put differs from PresentationSession transactional replacement.', ['PreparedSave::save', 'PresentationSession::prepare_save'], 'Q253 native reproduction and working stale-refusal control; no browser trial.'),
    ('Forme local lifetime', 'Request tokens and per-page retired-writer fences are real local guards.', ['OwnerLifecycle', 'FormeWriteFence'], 'They do not provide an independent stored-baseline comparison.'),
    ('Pointer ownership', 'TreePage gates Forme pointer gestures to the primary button.', ['TreePage::pointer', 'forme::pointer'], 'Caller qualification rules out an apparent helper-level auxiliary drag bug.'),
    ('Retained exports', 'Clip exports bind access, persona, graph and content session.', ['ClipExportScope', 'PendingExport::drop', 'Reading::poll'], 'Hidden, replaced or changed-owner exports are cancelled/revalidated; browser handoff is explicitly distinguished from completed download.'),
    ('Retained form effects', 'Reading has navigation/export routing but drops submission payloads.', ['Reading::poll', 'Reading::take_requests', 'hooks'], 'Q254 source-confirmed; a handoff message supplies no downstream submission outcome.'),
    ('Field inspection', 'Inspection delegates to held-record model functions.', ['region_candidates', 'inspect_field', 'inspect_coupling'], 'Boundary visibility, selector membership, force eligibility and evaluated effects remain explicitly distinct.'),
    ('Link inspection', 'Opened links compile through shared Scenomise/Cambium models.', ['compile_opened_link', 'opened_link_swatch', 'opened_event'], 'Actions revalidate current source, exact assertion and endpoints; withdrawal checks current personal authorship.'),
    ('Dynamics inspection', 'Current stage and stored schedule are independently read.', ['Canvas::inspect_dynamics', 'dynamics_inspection::section'], 'No controller stepping or source write is introduced by this inspector.'),
    ('Command routing', 'Shared viewer actions coexist with a copied named-command switch.', ['viewer_commands::action', 'controls::named_command', 'TreePage::command'], 'Q255: hosted view:fit selects different actions through two active routes.'),
    ('Text input delegation', 'The final appearance adapter delegates to the product focused field mapper.', ['hooks::focused_text', 'appearance::focused_text', 'product::fields'], 'The shared caret and IME route is present; apparent missing-hook claim ruled out.'),
    ('Feature-off boundaries', 'Unavailable local product and remote connections have inert explicit adapters.', ['product_off.rs', 'remote_off.rs'], 'Uninhabited product state cannot fabricate save success; remote connect returns an explicit feature-off error.'),
    ('Diagnostic ownership', 'Owner waypoints and external input requests are observation boundaries.', ['owner_waypoint', 'host_boundary'], 'Markers do not constitute independent storage or browser-input acceptance receipts. External input remains headed-driver owned.'),
    ('GPU ownership', 'The retained producer borrows the host RenderCore device.', ['attach_gpu', 'CanvasProducer::suspend'], 'Canvas and remote board share the host physics device; suspension drops cached handles. No headed/GPU execution this pass.'),
]
entries = [{'id': f'A{610+i:03d}', 'domain': d, 'claim': c, 'implementations': impl,
    'assessment': a, 'state': 'complete current source review; Q253 native diagnostic qualified separately', 'verification': NAME}
    for i, (d, c, impl, a) in enumerate(arows)]
assert len(entries) == 16
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/src/web_tree/presentation.rs', 'start': 0,
    'reason': 'Read the remaining retained editors, projection submodules and host routes, then all other unread Mere source.'}
notes['current_area'] = 'Retained product UI, Forme caller, reading host, inspectors and command composition'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'forme_stale_save_probe_receipt.json'], 'production_edits_this_turn': False,
    'next_read': next_read, 'pending_verification': pending,
    'diagnostic_outcomes': '2 Forme/rules public native checks passed. Personal-sync build still live; its 5 prepared checks did not run.'}
addition = f'''

### Retained UI and independent Forme saves, 2026-10-17

At {head}, this pass completely reads 22 files / 440,402 characters, including web_tree.rs, product.rs, Forme UI, retained reading host, all access/link/field/dynamics inspector adapters, command composition and selected feature-off/input helpers. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. Full Mere review remains incomplete. Source inventory was refreshed; all acknowledged hashes remain current.

Q253 promotes the prior Forme candidate with 2 passing public native checks. Two independent sessions load one MemoryBackend slot; a first acknowledged left-bound edit disappears after a stale second top-bound save also succeeds. The equivalent PresentationSession rejects the stale baseline and keeps its draft. The retained Forme UI uses that blind proposal save. Local request tokens and its per-page retired-writer fence do not compare independent stored changes. This is public in-memory API evidence plus a traced UI caller, not a browser/multitab or physical-disk trial. The exact qualified earlier current-build libraries, source hashes, test output and marker-verified generated-output removal are recorded in forme_stale_save_probe_receipt.json. A storage-baseline guard must preserve draft/history on refusal.

Q254 is source-confirmed: Reading::poll drains a submission-only Pelt effect, displays a host-handoff message and drops the full submission. Only navigation/export address tuples reach the outer frame-hook queue. The complete current composition has no downstream submission owner, outcome or retry. This is conditional on a retained engine emitting a submission; no form/browser trial ran. Explicit refusal is appropriate until a policy-aware owner accepts the complete request.

Q255 is another live duplicate: toolbar/menu hosted-fit routing chooses FitVisible, while TreePage::command uses controls::named_command and maps the same view:fit ID to Fit unconditionally. viewer_commands::action already accepts hosted context. Fit also resumes view-follow, so these are different policies beneath the same command ID. This route divergence is source-confirmed, without a browser framing trial.

Several suspicions were ruled out. Product's caret/IME hook is reached through appearance::focused_text, which explicitly delegates to product::focused_text. TreePage filters Forme pointer gestures to Primary before calling the helper. Failed clip intake recovers the created member into pending_intake. Saved product operations wait for acknowledgement and distinguish saved data from an optional Canvas refresh failure. Link actions revalidate actual source, endpoints and exact assertion; personal withdrawal is separately checked. Field inspection distinguishes held records, visible rectangular boundaries, selector membership and recognized-but-unevaluated effects. Dynamics inspection separates stored schedules from the current stage. Feature-off adapters cannot construct a live SavedProduct and explicitly refuse remote connection. Passive owner markers and host-input requests remain instrumentation, not acceptance evidence. GPU handles are borrowed from the host RenderCore and retired on suspension; no GPU/headed trial ran here.

A610-A625 record these genuine shared owners and copied routes. The 999 prior protected production/test hashes, cited evidence and canonical document prefix remain unchanged. No production/original-test code was edited. A fresh fetch confirms origin/main at 161368bfd2f33c2ea4002ebd86565b2102c1a780, with this basis 9 ahead / 4 behind. Those upstream commits remain unmerged while live shared builds use the checkout. No isolated target, Cargo home or worktree was created. The ordinary stable Mere target remains owned by its live builds. The Forme diagnostic outputs were removed after their receipt was recorded.

Personal-sync build session60676 remains live and is independently process-confirmed. Q250-Q252 remain source-only; its five prepared tests did not execute. Continue polling that same handle, write an authoritative terminal receipt only after actual completion, then run the preserved prepared diagnostic. Next whole source is web_tree/presentation.rs and the remaining retained editor/projection/host modules; all other unread Mere areas remain in scope. The [retained UI checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds this progress. Full-source goal remains active.
'''
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
(HERE / 'retained_ui_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_retained_ui_review.py', 'forme_stale_save_probe.rs', 'run_forme_stale_save_probe.py',
    'forme_stale_save_probe_receipt.json', 'forme_stale_save_probe_compile.log',
    'forme_stale_save_probe_output.log', 'retained_ui_personal_sync_build_live.json', 'retained_ui_diff_check.txt']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'personal_sync_review_receipt.json', 'sha256': sha(HERE / 'personal_sync_review_receipt.json')},
    'kind': 'Complete retained UI source review and qualified Forme stale-save diagnostic; full Mere goal incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 440402,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'new_findings': ['Q253', 'Q254', 'Q255'], 'source_only_findings': ['Q254', 'Q255'],
    'authority_entries': [e['id'] for e in entries], 'new_native_tests_run': 2,
    'production_edits_this_pass': False, 'pending_verification': pending, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
        'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {n: sha(HERE / n) for n in evidence}, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Two synthetic MemoryBackend public native tests; source-only web caller/dispatch findings. No browser/multitab, network, physical disk, OS input or GPU trial. Original personal-sync build remains live; no prepared tests ran.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'whole_files': len(whole),
    'new_findings': receipt['new_findings'], 'new_native_tests_run': 2,
    'protected_paths': len(guards), 'retained_bytes': budget, 'pending_session': 60676}, indent=2))
