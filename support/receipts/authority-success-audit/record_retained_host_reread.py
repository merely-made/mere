"""Restore all upstream-invalidated source coverage without production edits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'retained_host_reread_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE/n).write_text(json.dumps(value, indent=2)+'\n', encoding='utf-8')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE/NAME).exists()
old = load('frozen_document_reread_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
aux = old['auxiliary_read_sha256']
assert len(guards) == 1946
assert all(sha(ROOT/n) == h for n, h in {**guards, **aux}.items())
assert all(sha(HERE/n) == h for n, h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT/old['document']['path']
prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT/n) == h for n, h in shared.items() if n != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute():
    index = ROOT/index
index_hash = sha(index)
assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
assert inv['commit'] == head and load('pending_read.json') == []
stats = json.loads(subprocess.check_output(['python', str(HERE/'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2416 and stats['read_chars'] == 28596551
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
reread = sorted(old['coverage']['changed_since_inventory'])
assert len(reread) == 14 and sum(inv['files'][n]['chars'] for n in reread) == 400889
assert all(cov[n]['sha256'] == sha(ROOT/n) == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] for n in reread)
areas = []
for area, expected_files, expected_chars in [('ports/graphshell/', 529, 4261985), ('ports/dramatis/', 10, 56730)]:
    files = [n for n in inv['files'] if n.startswith(area)]
    assert len(files) == expected_files and sum(inv['files'][n]['chars'] for n in files) == expected_chars
    assert all(cov[n]['sha256'] == sha(ROOT/n) == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] for n in files)
    areas.append({'prefix': area, 'files': len(files), 'chars': expected_chars, 'verification': NAME, 'basis_commit': head, 'qualification': 'All inventoried source has complete current matching-hash coverage. This pass executes no browser, native host or Cargo tests.'})
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q351' and authority['entries'][-1]['id'] == 'A971'
rows = [
    ('Retained semantic queries use the live accessibility projection', 'TreeLane role/name queries now require ctx.a11y_projection.', ['ports/graphshell/src/web_tree/lane.rs', 'ports/dramatis/tests/interaction.rs', 'ports/dramatis/Cargo.toml'], 'focus-retained, reveal, exact assertion actions and checkbox steps resolve through matching_with_projection and refuse a missing projection where they require a target. Class queries identify authored DOM nodes without inventing role/name semantics. Dramatis keeps matching_estimated_fixture only for explicit fixture node discovery while Harness interactions use the shared host. Read tests establish intended scope; no fresh interaction execution in this pass.'),
    ('Focused text routing consumes the retained focus identity', 'A shared focused_field_has_label checks the focused node and authored text-value marker.', ['ports/graphshell/src/web_tree.rs', 'ports/graphshell/src/web_tree/appearance.rs', 'ports/graphshell/src/web_tree/commands.rs', 'ports/graphshell/src/web_tree/coupling.rs', 'ports/graphshell/src/web_tree/presentation.rs', 'ports/graphshell/src/web_tree/product/fields.rs', 'ports/graphshell/src/web_tree/promoted_links.rs', 'ports/graphshell/src/web_tree/query_view.rs'], 'The callback already receives runner.focus, so exact authored aria-label plus data-cambium-text-value maps that node to the application TextInput without a second guessed semantic query. Pane visibility and busy guards remain local. Application-wide label uniqueness and dynamic projection-option labels are a separate routing assumption, not proven by this helper.'),
    ('Viewer menus and command API still have separate dispatch recipes', 'Menu actions share viewer_commands while TreePage::command uses controls::named_command first.', ['ports/graphshell/src/web_tree.rs', 'ports/graphshell/src/web_tree/commands.rs'], 'Menu invocation rechecks current disabled reasons and uses hosted-fit semantics from viewer_commands. Explicit command API continues to take the older named-command path. Q255 remains source-visible after the upstream query migration; current rereading is not a new runtime reproduction. Local Product menu delegation is actual sharing and should not be counted as another persisted command owner.'),
    ('Touch cancellation retains a separate legacy release path', 'Gesture HostHandle still carries a release closure.', ['ports/graphshell/src/web_tree.rs', 'ports/graphshell/src/web_tree/gestures.rs'], 'The right-pointer capture route calls pointer_cancel explicitly. The hosted single-touch pointercancel route still calls host.release, with the stale comment that the host has no cancel, even though shared host cancellation exists. This preserves earlier Q258; current source confirms it remains, without a fresh browser event execution. Shared wheel/two-finger math and painted-overlay policy are real reuse.'),
    ('Presentation rule persistence remains tied to the initially opened source', 'Pane::open captures backend and graph ID once; refresh rereads current occurrence facts.', ['ports/graphshell/src/web_tree.rs', 'ports/graphshell/src/web_tree/presentation.rs'], 'Current hooks refresh bound/live facts but do not reopen the PresentationSession after Product graph replacement. Q256 remains source-visible. Prepared save completion is correctly acknowledged through finish_save before displaying saved, and unsupported renderer effects remain explicitly decision-only. Sharing evaluation does not reconcile the separate persistence-owner lifetime.'),
    ('Coupling inspection distinguishes source facts from evaluated force', 'Strength drafts bind exact coupling and source session; inspection reports recognition rather than execution.', ['ports/graphshell/src/web_tree/coupling.rs', 'ports/graphshell/src/web_tree/field_inspection.rs'], 'Candidate validation happens before queueing the product operation, and success text follows the product coupling result. Error retains an open draft. Region discovery is explicitly rectangular view discovery and boundary visibility is not lifecycle/coupling authority. Changing/closing inspection can leave the separate bound editor draft; exact-draft targeting is deliberate and must stay visible rather than silently retarget.'),
    ('Query completion and promoted-link completion use different confirmation recipes', 'Query views retain failed pending saves; promoted links separately hide unconfirmed freeze owners.', ['ports/graphshell/src/web_tree/query_view.rs', 'ports/graphshell/src/web_tree/promoted_links.rs'], 'Query write status waits on product selection lock and rereads the expected saved spec; owner changes retire its draft. Promoted links require saved state plus readable owner/comment and scope. Their failed freeze path moves owner into an unconfirmed set and clears pane.pending; retry reconciliation needs the product operation trace before claiming an irreversible hidden item. Comment input clears on queue, so retained failure/retry UX is another caller-trace followup.'),
    ('Browser semantic measurements and host stamps establish different boundaries', 'Scenario fields read graphshellSemanticTree JSON while after_frame publishes available presentation stamps.', ['ports/graphshell/src/web_tree.rs', 'ports/graphshell/src/web_tree/lane.rs'], 'Heard names/buttons/descriptions are read from the page semantic instrument; missing instrument returns absent/default counts, which alone cannot prove an absence assertion. This is not OS assistive-stack output. after_frame only advances data-context-generation and data-presentation-sequence when ctx.presentation is Some. data-ready is boot/mount readiness, not itself a fresh displayed frame or graphics acceptance.'),
    ('Standalone manifest repeats supplier policy without proving a second runtime', 'Graphshell web is an independent workspace and restates dependency/patch/profile policy.', ['ports/graphshell/web/Cargo.toml', 'ports/graphshell/src/web_tree.rs'], 'The current manifest restates Genet main, exact Netrender revision and the matched p2panda family; that is a real duplicate configuration maintenance surface. No fresh standalone Cargo graph was resolved, so do not infer runtime duplication or a repaired supplier cone. web_tree module introduction still describes the earlier phase before controls moved although the file now composes full retained product/editor controls.'),
]
entries = [{'id': f'A{972+i}', 'domain': domain, 'claim': claim, 'implementations': paths, 'assessment': assessment, 'state': 'fourteen complete current source rereads; source-qualified observations only; no new runtime tests', 'verification': NAME} for i, (domain, claim, paths, assessment) in enumerate(rows)]
authority['entries'].extend(entries)
notes['completed_source_areas'].extend(areas)
for area in notes['completed_source_areas']:
    pending = area.get('pending_current_revision_reread')
    if pending:
        area['pending_current_revision_reread'] = {'verification': NAME, 'basis_commit': head, 'paths': [], 'resolved_paths': pending.get('resolved_paths', []) + pending['paths']}
        area['current_revision_restored_by'] = NAME
transition = {'verification': NAME, 'basis_commit': head, 'reread_files': reread, 'reread_chars': 400889, 'remaining_upstream_rereads': [], 'qualification': 'All fourteen files emitted, reviewed and acknowledged in full. The latest upstream merge no longer invalidates any previous read coverage.'}
notes.setdefault('source_revision_refreshes', []).append(transition)
authority.setdefault('source_revision_refreshes', []).append(transition)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Promoted-link retry reconciliation', 'detail': 'Trace global failed operation retry through Product before asserting whether unconfirmed freeze owners become visible again. Pane.poll clears its pending token and retains an unconfirmed owner set; current success check cannot itself revisit that owner. Also trace queued comment recovery after the input clears. No new concrete product failure is claimed here.'},
    {'verification': NAME, 'area': 'Focused field mapping uniqueness', 'detail': 'The focused node is authoritative, but field selection maps it to application storage by authored label. Check dynamic projection-option labels against ordinary pane labels and duplicate labels before asserting incorrect TextInput routing. No collision fixture was executed in this pass.'},
])
next_read = {'path': 'crates/intel/mora-cmudict/Cargo.toml', 'start': 0, 'reason': 'Continue ordinary unread source after all upstream rereads are restored, then reference-data and Moot owners.'}
notes['current_area'] = 'All upstream-invalidated source coverage restored; Graphshell and Dramatis current'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': [NAME], 'audit_authored_production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None, 'diagnostic_outcomes': 'Fourteen whole-file rereads restore current coverage. No new runtime diagnostic, bug promotion, browser/native/GPU execution or Cargo graph resolution this pass.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE/'retained_host_diff_check.txt').open('x', encoding='utf-8') as f:
    f.write(diff.stdout + diff.stderr)
addition = f'''

### Current retained-host source reread, 2026-10-17

The fourteen remaining upstream-changed files /400,889characters are fully reread at unchanged HEAD {head}. All prior source coverage is current again: {stats['complete_files']:,}/{stats['total_files']:,}files and {stats['read_chars']:,}/{stats['total_chars']:,}characters. Graphshell port coverage is all529inventory files /4,261,985characters; Dramatis is all10 /56,730characters. The full audit remains active and incomplete, with1,315ordinary unread files remaining.

A972-A980 record actual query sharing and remaining separate authorities. Semantic role/name scenario actions now consume the retained accessibility projection and refuse a missing required projection. Authored class lookup remains a DOM identity operation. Focused text routing uses the already-focused node and its authored label/value marker; it does not need another estimated semantic query. Dramatis explicitly names estimated fixture lookups and keeps retained host interaction claims bounded.

Earlier Q255 command-route divergence, Q256 initial presentation-store owner lifetime and Q258 touch cancellation-as-release remain visible in the current complete source. No fresh runtime reproduction is claimed. Coupling edits still bind exact source/coupling and acknowledge product results. Rule persistence uses finish_save before reporting saved. Query and promoted-link writes implement separate confirmation recipes; promoted freeze retry reconciliation and dynamic field-label collisions remain caller-trace followups, not newly promoted bugs.

Browser semantic-tree JSON is instrument output, not proof of OS assistive delivery. Its missing/default counts cannot alone prove an absence assertion. Boot data-ready is mount readiness; presentation sequence is published only when a host stamp is available. The standalone web manifest necessarily restates workspace dependency policy, making configuration drift a real maintenance surface without proving two loaded runtimes. Module prose still describes the early control-free tree phase although current code composes the full product editors.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}). This pass reads source only and executes no new Cargo, interaction, browser, native, renderer or GPU test. No production edits. Prior evidence, protected sources, auxiliary artifacts, shared WIP and Git index are preserved, and all previous document bytes remain an exact prefix. Diff check passes with line-ending warnings captured separately from errors. No retained isolated target, Cargo home, worktree or new build output. Next: unread mora-cmudict, reference-data and Moot sources.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_retained_host_reread.py', 'retained_host_diff_check.txt']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'kind': 'Fourteen upstream rereads restore all current coverage; full repository incomplete', 'prior_receipt': {'path': 'frozen_document_reread_receipt.json', 'sha256': sha(HERE/'frozen_document_reread_receipt.json')}, 'coverage': stats, 'completed_source_areas': areas, 'whole_current_files': reread, 'source_sha256': {n: sha(ROOT/n) for n in reread}, 'auxiliary_read_sha256': aux, 'preserved_production_and_test_hashes': guards, 'shared_working_file_sha256': shared, 'new_findings': [], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False, 'source_revision_refresh': transition, 'diagnostics': {}, 'pending_verification': None, 'next_read': next_read, 'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()}, 'evidence_sha256': {n: sha(HERE/n) for n in evidence}, 'git_index_sha256': index_hash, 'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'limits': 'Source-only pass. Existing findings source-revalidated, not rerun. No standalone Cargo dependency graph or browser/native/GPU/assistive execution. 1315 files remain unread.'}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n) == h for n, h in {**guards, **receipt['source_sha256'], **aux}.items())
assert all(sha(HERE/n) == h for n, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n) == h for n, h in shared.items() if n != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': [], 'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}), 'retained_bytes': budget}, indent=2))
