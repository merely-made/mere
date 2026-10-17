"""Checkpoint six complete upstream rereads and the public frozen-detail probe."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'frozen_document_reread_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE/n).write_text(json.dumps(value, indent=2)+'\n', encoding='utf-8')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE/NAME).exists()
old = load('pictograph_shared_image_reread_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
aux = old['auxiliary_read_sha256']
assert len(guards) == 1944
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
assert stats['complete_files'] == 2402 and stats['read_chars'] == 28195662
assert len(stats['changed_since_inventory']) == 14 and not stats['pending_chunks']
reread = sorted(set(old['coverage']['changed_since_inventory']) - set(stats['changed_since_inventory']))
assert len(reread) == 6 and sum(inv['files'][n]['chars'] for n in reread) == 210113
assert all(cov[n]['sha256'] == sha(ROOT/n) == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] for n in reread)
areas = []
for area in ['crates/graphshell/graphshell-client/', 'crates/inker/document-canvas/']:
    files = [n for n in inv['files'] if n.startswith(area)]
    assert all(cov[n]['sha256'] == sha(ROOT/n) == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] for n in files)
    areas.append({'prefix': area, 'files': len(files), 'chars': sum(inv['files'][n]['chars'] for n in files), 'verification': NAME, 'basis_commit': head, 'qualification': 'Complete matching-hash source coverage; diagnostic execution is separately bounded.'})
probe = load('frozen_detail_probe_receipt.json')
assert probe['basis_commit'] == head and probe['build_exit'] == probe['cases_build_exit'] == probe['run_exit'] == 0
assert probe['checks'] == 3 and probe['generated_output_removed']
assert probe['runner_sha256'] == sha(HERE/'run_frozen_detail_probe.py')
assert probe['cases_sha256'] == sha(HERE/'frozen_detail_cases.rs')
assert all(sha(ROOT/n) == h for n, h in probe['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['libraries'].values())
assert all(not Path(v['path']).exists() for v in probe['removed_outputs'])
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q350' and authority['entries'][-1]['id'] == 'A967'
finding = {
    'id': 'Q351',
    'title': 'Frozen realization routes discard retained per-instance readable detail',
    'source': 'crates/graphshell/graphshell-client/src/frozen.rs:377,562,619,840',
    'status': 'reproduced through current complete production library public APIs with accesskit enabled; documented, not repaired',
    'evidence': 'A validated SceneSnapshot supplies distinct detail strings through freeze_snapshot_with_details. Flat rows and HTML preserve the ordinary instance detail, while the AccessKit tree retains its name but drops that detail from labels and descriptions. A folded member still holds its supplied detail in FrozenFold.members, but rows, HTML disclosure/alternate table and AccessKit all omit it while retaining the member name. The separately written output recipes therefore do not deliver the same readable facts.',
    'scope': 'Three public API cases compile the complete current graphshell-client library over seven hash-bound retained suppliers. Data returned by rows, HTML and UxTree only; no DOM execution, OS screen reader, Tokio pump, renderer, fresh Cargo dependency graph or original library unit suite.',
    'verification': NAME,
    'diagnostics': ['frozen_detail_probe_receipt.json'],
    'repair_triage': {'disposition': 'documented under active read-only audit goal', 'reason': 'Share readable instance lowering or explicitly define supported fact parity across HTML, rows and AccessKit. Carry visible detail into accessible descriptions and folded disclosure content; cover ordinary and folded details together. Do not infer full platform accessibility acceptance from name-only reachability tests.'},
}
notes['findings'].append(finding)
rows = [
    ('Frozen reading shares fold membership but independently rebuilds output content', 'HTML, rows and AccessKit are described as the same reading answer.', ['crates/graphshell/graphshell-client/src/frozen.rs', 'crates/graphshell/graphshell-client/src/fold.rs'], 'FoldReading actually shares membership, stand-in and badge interpretation with the remote reader. Output recipes are still separate: Q351 proves readable detail loss across routes. Earlier explicit item-order divergence Q063 remains source-visible; this pass does not rerun that older probe.'),
    ('Frozen DOM and accessibility checks establish narrower facts than assistive delivery', 'Estimated DOM fixtures and UxTree tests check frozen output.', ['crates/graphshell/graphshell-client/src/frozen.rs'], 'The renamed resolve_estimated_fixture calls explicitly qualify estimated DOM selector checks. UxTree tests mainly collect labels, roles and reachability and do not establish detail parity. The grid announcement helper models header association in Rust rather than exercising a platform screen reader. Three new public API cases verify data output, not an OS adapter.'),
    ('Document paint lowering preserves concrete shaped font and shared image ownership', 'FontTable selects actual shaped faces and decoded pixels use Arc<Vec<u8>>.', ['crates/inker/document-canvas/Cargo.toml', 'crates/inker/document-canvas/src/paint_list.rs', 'crates/inker/document-canvas/src/types.rs'], 'The paint builder interns FontFaceId and carries the chosen face bytes rather than resolving requested family labels again. Per-list image interning clones the decoded image Arc into the sidecar. Source tests check pointer equality and actual font bytes; those original tests were read but not executed in this pass. This boundary does not own host acquisition, decode policy or validation of arbitrary public packet data.'),
    ('Retained document source sharing is separate from appearance policy and resource discovery', 'SmolwebDocument holds Arc<EngineDocument> with independent retained layout and session state.', ['crates/system/document-lanes/src/smolweb.rs'], 'Shared constructors retain the supplied source Arc while style changes invalidate geometry and preserve source/folds/focus/scroll. Real layout remains document-canvas-owned. Separate shallow image discovery/admission/retention scans still underlie Q155, and scheme/body parser selection still underlies Q154. The frame method sets presented after scene construction, so that local flag alone is not evidence of compositor or surface presentation.'),
]
entries = [{'id': f'A{968+i}', 'domain': d, 'claim': c, 'implementations': paths, 'assessment': assessment, 'state': 'six full current source rereads; three fresh frozen public API cases; document lane and paint code source-only this pass', 'verification': NAME} for i, (d, c, paths, assessment) in enumerate(rows)]
authority['entries'].extend(entries)
notes['completed_source_areas'].extend(areas)
for area in notes['completed_source_areas']:
    pending = area.get('pending_current_revision_reread')
    if pending:
        remaining = [n for n in pending['paths'] if n in stats['changed_since_inventory']]
        area['pending_current_revision_reread'] = {'verification': NAME, 'basis_commit': head, 'paths': remaining, 'resolved_paths': pending.get('resolved_paths', []) + [n for n in pending['paths'] if n not in remaining]}
        if not remaining:
            area['current_revision_restored_by'] = NAME
transition = {'verification': NAME, 'basis_commit': head, 'reread_files': reread, 'reread_chars': 210113, 'remaining_upstream_rereads': stats['changed_since_inventory'], 'qualification': 'Whole emitted, reviewed and acknowledged sources; no diff-only coverage.'}
notes.setdefault('source_revision_refreshes', []).append(transition)
authority.setdefault('source_revision_refreshes', []).append(transition)
next_read = {'path': 'ports/dramatis/Cargo.toml', 'start': 0, 'reason': 'Finish fourteen upstream-changed files, then all remaining unread source.'}
notes['current_area'] = 'Frozen realization and document image/source ownership current rereads complete'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': [NAME, 'frozen_detail_probe_receipt.json'], 'audit_authored_production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None, 'diagnostic_outcomes': 'Three fresh frozen-detail public API cases pass, reproducing Q351. Complete production client library compiled with accesskit over seven retained suppliers; no original unit suite, fresh Cargo graph, OS, native or GPU acceptance.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE/'frozen_document_diff_check.txt').open('x', encoding='utf-8') as f:
    f.write(diff.stdout + diff.stderr)
addition = f'''

### Frozen reading and retained document ownership reread, 2026-10-17

Six upstream-changed files /210,113characters have been fully reread at HEAD {head}. Current coverage is {stats['complete_files']:,}/{stats['total_files']:,}files and {stats['read_chars']:,}/{stats['total_chars']:,}characters. Graphshell-client and document-canvas now have complete matching-hash inventory coverage again. Fourteen changed files remain for whole-file rereading, followed by all ordinary unread source. The full audit remains active and incomplete.

Q351 reproduces a content mismatch behind the frozen paths' same-answer claim. Supplied flat instance detail survives rows and HTML but disappears from AccessKit labels/descriptions. Folded member detail remains in the frozen data yet disappears from every output route, including the HTML disclosure and alternate table. Member names remain present. Shared FoldReading membership is real; independently rebuilt output content is the failing seam. A968-A971 record that distinction alongside estimated DOM/test limits and the actual shared font, decoded image and portable-document ownership.

Three fresh public API cases pass against the complete current graphshell-client production library with accesskit enabled and seven exact retained suppliers. They reproduce returned-data loss and include a passing flat HTML/rows control. They do not run the original unit suite, fresh Cargo dependency graph, Tokio pump, browser DOM, OS screen reader, compositor or native GPU path. Generated library/executable/debug outputs were hashed and removed. Document-canvas and Smolweb tests were read, not executed in this pass.

The document lane shares source via Arc while keeping appearance layout/state local; decoded pixel sidecars now clone Arc handles. Concrete shaped font bytes remain the font owner. Earlier Q154 parser-selection and Q155 shallow nested-image discovery remain source-visible and were not newly rerun. Its presented flag is set after scene construction, so that local state alone cannot establish a completed surface presentation.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [public API diagnostic](../../../support/receipts/authority-success-audit/frozen_detail_probe_receipt.json). No production edits. Protected source hashes, auxiliary artifacts, prior evidence, index and shared WIP are preserved; the prior document is an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or new build output. Next: Dramatis manifest and interaction source, then remaining changed Graphshell files.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_frozen_document_reread.py', 'frozen_document_diff_check.txt', 'run_frozen_detail_probe.py', 'frozen_detail_cases.rs', 'frozen_detail_probe_library_compile.log', 'frozen_detail_probe_cases_compile.log', 'frozen_detail_probe_output.log', 'frozen_detail_probe_receipt.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'kind': 'Frozen and document ownership source reread; repository incomplete', 'prior_receipt': {'path': 'pictograph_shared_image_reread_receipt.json', 'sha256': sha(HERE/'pictograph_shared_image_reread_receipt.json')}, 'coverage': stats, 'completed_source_areas': areas, 'whole_current_files': reread, 'source_sha256': {n: sha(ROOT/n) for n in reread}, 'auxiliary_read_sha256': aux, 'preserved_production_and_test_hashes': guards, 'shared_working_file_sha256': shared, 'new_findings': ['Q351'], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False, 'source_revision_refresh': transition, 'diagnostics': {'frozen_detail_probe_receipt.json': {'build_exit': 0, 'cases_build_exit': 0, 'run_exit': 0, 'checks': 3, 'original_tests': 0, 'audit_cases': 3, 'generated_output_removed': True}}, 'pending_verification': None, 'next_read': next_read, 'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()}, 'evidence_sha256': {n: sha(HERE/n) for n in evidence}, 'git_index_sha256': index_hash, 'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'limits': probe['qualification']}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n) == h for n, h in {**guards, **receipt['source_sha256'], **aux}.items())
assert all(sha(HERE/n) == h for n, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n) == h for n, h in shared.items() if n != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}), 'retained_bytes': budget}, indent=2))
